use bevy::{ecs::message::Messages, prelude::*};
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet,
    action::Action,
    actions::{assertions, inspect},
    init_scenario,
    scenario::Scenario,
};
use saddle_camera_fps_camera::{
    CameraRecoilRequest, CameraShakeRequest, ComfortConfig, FpsCamera, FpsCameraConfig,
    FpsCameraIntent, FpsCameraRuntime,
};

use crate::LabCameraEntity;

pub struct FpsCameraLabE2EPlugin;

impl Plugin for FpsCameraLabE2EPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(E2EPlugin);
        app.configure_sets(
            Update,
            E2ESet.before(saddle_camera_fps_camera::FpsCameraSystems::ReadIntent),
        );

        let args: Vec<String> = std::env::args().collect();
        let (scenario_name, handoff) = parse_e2e_args(&args);

        if let Some(name) = scenario_name {
            if let Some(mut scenario) = scenario_by_name(&name) {
                if handoff {
                    scenario.actions.push(Action::Handoff);
                }
                init_scenario(app, scenario);
            } else {
                error!(
                    "[fps_camera_lab:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
}

#[derive(Resource, Clone, Copy)]
struct BaselineComfortSample {
    translation_mag: f32,
    rotation_mag: f32,
}

fn parse_e2e_args(args: &[String]) -> (Option<String>, bool) {
    let mut scenario_name = None;
    let mut handoff = false;

    for arg in args.iter().skip(1) {
        if arg == "--handoff" {
            handoff = true;
        } else if !arg.starts_with('-') && scenario_name.is_none() {
            scenario_name = Some(arg.clone());
        }
    }

    if !handoff {
        handoff = std::env::var("E2E_HANDOFF").is_ok_and(|value| value == "1" || value == "true");
    }

    (scenario_name, handoff)
}

fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "fps_camera_smoke" => Some(build_smoke()),
        "fps_camera_look" => Some(build_look()),
        "fps_camera_movement" => Some(build_movement()),
        "fps_camera_effects" => Some(build_effects()),
        "fps_camera_comfort" => Some(build_comfort()),
        _ => None,
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec![
        "fps_camera_smoke",
        "fps_camera_look",
        "fps_camera_movement",
        "fps_camera_effects",
        "fps_camera_comfort",
    ]
}

fn camera_entity(world: &World) -> Option<Entity> {
    world
        .get_resource::<LabCameraEntity>()
        .map(|resource| resource.0)
}

fn runtime(world: &World) -> Option<FpsCameraRuntime> {
    let entity = camera_entity(world)?;
    world.get::<FpsCameraRuntime>(entity).cloned()
}

fn config_mut(world: &mut World) -> Option<Mut<'_, FpsCameraConfig>> {
    let entity = camera_entity(world)?;
    world.get_mut::<FpsCameraConfig>(entity)
}

fn intent_mut(world: &mut World) -> Option<Mut<'_, FpsCameraIntent>> {
    let entity = camera_entity(world)?;
    world.get_mut::<FpsCameraIntent>(entity)
}

fn build_smoke() -> Scenario {
    Scenario::builder("fps_camera_smoke")
        .description(
            "Boot the lab, assert the runtime is alive and grounded, then capture a baseline screenshot.",
        )
        .then(Action::WaitFrames(90))
        .then(assertions::entity_exists::<FpsCamera>("camera entity exists"))
        .then(assertions::component_satisfies::<FpsCameraRuntime>(
            "camera starts grounded",
            |runtime| runtime.grounded && runtime.visual_fov > 0.0,
        ))
        .then(assertions::log_summary("fps_camera_smoke summary"))
        .then(inspect::dump_component_json::<FpsCameraRuntime>("smoke_runtime"))
        .then(Action::Screenshot("fps_camera_smoke".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_look() -> Scenario {
    Scenario::builder("fps_camera_look")
        .description(
            "Inject controlled look intent, verify yaw and pitch change, then capture before and after screenshots.",
        )
        .then(Action::WaitFrames(60))
        .then(Action::Screenshot("fps_camera_look_before".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world: &mut World| {
            if let Some(mut intent) = intent_mut(world) {
                intent.look_delta = Vec2::new(80.0, -40.0);
            }
        })))
        .then(Action::WaitFrames(2))
        .then(assertions::component_satisfies::<FpsCameraRuntime>(
            "look changes rotation",
            |runtime| runtime.yaw.abs() > 0.05 && runtime.pitch.abs() > 0.02,
        ))
        .then(assertions::log_summary("fps_camera_look summary"))
        .then(Action::Screenshot("fps_camera_look_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_movement() -> Scenario {
    Scenario::builder("fps_camera_movement")
        .description(
            "Drive move, sprint, crouch, and jump intent directly. Assert runtime speed, crouch alpha, airborne state, and landing recovery.",
        )
        .then(Action::WaitFrames(60))
        .then(Action::Custom(Box::new(|world: &mut World| {
            if let Some(mut intent) = intent_mut(world) {
                intent.move_axis = Vec2::Y;
            }
        })))
        .then(Action::WaitFrames(25))
        .then(assertions::component_satisfies::<FpsCameraRuntime>(
            "forward move builds speed",
            |runtime| runtime.speed > 0.5 && runtime.position.z < 7.7,
        ))
        .then(Action::Custom(Box::new(|world: &mut World| {
            if let Some(mut intent) = intent_mut(world) {
                intent.move_axis = Vec2::X;
            }
        })))
        .then(Action::WaitFrames(20))
        .then(assertions::component_satisfies::<FpsCameraRuntime>(
            "strafe produces lateral displacement",
            |runtime| runtime.speed > 0.5 && runtime.position.x > 0.4,
        ))
        .then(Action::Custom(Box::new(|world: &mut World| {
            if let Some(mut intent) = intent_mut(world) {
                intent.move_axis = Vec2::Y;
                intent.sprint_pressed = true;
            }
        })))
        .then(Action::WaitFrames(25))
        .then(assertions::component_satisfies::<FpsCameraRuntime>(
            "sprint raises sprint alpha",
            |runtime| runtime.sprint_alpha > 0.5 && runtime.speed_ratio > 0.7,
        ))
        .then(Action::Custom(Box::new(|world: &mut World| {
            if let Some(mut intent) = intent_mut(world) {
                intent.sprint_pressed = false;
                intent.crouch_pressed = true;
            }
        })))
        .then(Action::WaitFrames(25))
        .then(assertions::component_satisfies::<FpsCameraRuntime>(
            "crouch lowers eye height",
            |runtime| runtime.crouch_alpha > 0.5 && runtime.eye_height < 1.4,
        ))
        .then(Action::Custom(Box::new(|world: &mut World| {
            if let Some(mut intent) = intent_mut(world) {
                intent.crouch_pressed = false;
                intent.move_axis = Vec2::ZERO;
                intent.jump_pressed = true;
            }
        })))
        .then(Action::WaitFrames(10))
        .then(assertions::component_satisfies::<FpsCameraRuntime>(
            "jump becomes airborne",
            |runtime| !runtime.grounded && runtime.position.y > 0.0,
        ))
        .then(Action::WaitFrames(70))
        .then(assertions::component_satisfies::<FpsCameraRuntime>(
            "landing returns grounded state",
            |runtime| runtime.grounded && runtime.recent_landing_impulse <= 1.0,
        ))
        .then(assertions::log_summary("fps_camera_movement summary"))
        .then(Action::Screenshot("fps_camera_movement".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_effects() -> Scenario {
    Scenario::builder("fps_camera_effects")
        .description(
            "Inject shake and recoil requests, assert the runtime updates, and capture active plus recovered frames.",
        )
        .then(Action::WaitFrames(60))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let Some(entity) = camera_entity(world) else {
                return;
            };
            world
                .resource_mut::<Messages<CameraShakeRequest>>()
                .write(CameraShakeRequest {
                    entity,
                    trauma: 0.75,
                });
            world
                .resource_mut::<Messages<CameraRecoilRequest>>()
                .write(CameraRecoilRequest {
                    entity,
                    pitch: 8.0_f32.to_radians(),
                    yaw: 2.0_f32.to_radians(),
                });
        })))
        .then(Action::WaitFrames(2))
        .then(assertions::component_satisfies::<FpsCameraRuntime>(
            "effects become active",
            |runtime| runtime.trauma > 0.2 && runtime.recoil_offset.length() > 0.02,
        ))
        .then(Action::Screenshot("fps_camera_effects_active".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(90))
        .then(assertions::component_satisfies::<FpsCameraRuntime>(
            "effects recover",
            |runtime| runtime.trauma < 0.2 && runtime.recoil_offset.length() < 0.02,
        ))
        .then(assertions::log_summary("fps_camera_effects summary"))
        .then(Action::Screenshot("fps_camera_effects_recovered".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_comfort() -> Scenario {
    Scenario::builder("fps_camera_comfort")
        .description(
            "Compare baseline and low-motion comfort under the same injected shake and verify reduced effect magnitudes.",
        )
        .then(Action::WaitFrames(60))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let Some(entity) = camera_entity(world) else {
                return;
            };
            world
                .resource_mut::<Messages<CameraShakeRequest>>()
                .write(CameraShakeRequest {
                    entity,
                    trauma: 0.8,
                });
        })))
        .then(Action::WaitFrames(2))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let runtime = runtime(world).expect("runtime should exist");
            world.insert_resource(BaselineComfortSample {
                translation_mag: runtime.effect_stack.translation.length(),
                rotation_mag: runtime.effect_stack.rotation.length(),
            });
        })))
        .then(Action::Screenshot("fps_camera_comfort_baseline".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let Some(entity) = camera_entity(world) else {
                return;
            };
            if let Some(mut config) = config_mut(world) {
                config.comfort = ComfortConfig::low_motion();
            }
            world
                .resource_mut::<Messages<CameraShakeRequest>>()
                .write(CameraShakeRequest {
                    entity,
                    trauma: 0.8,
                });
        })))
        .then(Action::WaitFrames(2))
        .then(assertions::custom(
            "low motion reduces shake envelope",
            |world| {
                let baseline = world
                    .get_resource::<BaselineComfortSample>()
                    .expect("baseline comfort sample should exist");
                let runtime = runtime(world).expect("runtime should exist");
                runtime.effect_stack.translation.length() < baseline.translation_mag
                    && runtime.effect_stack.rotation.length() < baseline.rotation_mag
            },
        ))
        .then(assertions::log_summary("fps_camera_comfort summary"))
        .then(Action::Screenshot("fps_camera_comfort_low_motion".into()))
        .then(Action::WaitFrames(1))
        .build()
}
