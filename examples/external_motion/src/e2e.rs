use bevy::{ecs::message::Messages, prelude::*};
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet, action::Action, actions::assertions, init_scenario, scenario::Scenario,
};
use saddle_camera_fps_camera::{
    CameraRecoilRequest, CameraShakeRequest, FpsCamera, FpsCameraExternalMotion, FpsCameraRuntime,
    FpsCameraSystems,
};

use crate::{DemoPlayer, ViewmodelRoot};

pub struct ExternalMotionE2EPlugin;

impl Plugin for ExternalMotionE2EPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(E2EPlugin);
        app.configure_sets(Update, E2ESet.before(FpsCameraSystems::ReadIntent));

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
                    "[fps_external_motion:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
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
        "fps_external_motion_smoke" => Some(build_smoke()),
        "fps_external_motion_recoil" => Some(build_recoil()),
        _ => None,
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec!["fps_external_motion_smoke", "fps_external_motion_recoil"]
}

fn camera_entity(world: &mut World) -> Entity {
    let mut cameras = world.query_filtered::<Entity, With<FpsCamera>>();
    cameras
        .single(world)
        .expect("external motion example should spawn exactly one FPS camera")
}

fn build_smoke() -> Scenario {
    Scenario::builder("fps_external_motion_smoke")
        .description(
            "Boot the external-motion integration demo, verify the controller bridge, then capture the baseline frame.",
        )
        .then(Action::WaitFrames(90))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let camera = camera_entity(world);
            let bridge = world
                .get::<FpsCameraExternalMotion>(camera)
                .expect("camera should expose external motion");
            assert!(bridge.enabled);

            let runtime = world
                .get::<FpsCameraRuntime>(camera)
                .expect("camera should expose runtime diagnostics");
            assert!(runtime.grounded);

            let mut players = world.query_filtered::<Entity, With<DemoPlayer>>();
            assert!(players.single(world).is_ok());

            let mut viewmodels = world.query_filtered::<Entity, With<ViewmodelRoot>>();
            assert!(viewmodels.single(world).is_ok());
        })))
        .then(Action::Screenshot("fps_external_motion_smoke".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("fps_external_motion_smoke summary"))
        .build()
}

fn build_recoil() -> Scenario {
    Scenario::builder("fps_external_motion_recoil")
        .description(
            "Inject shake and recoil into the external-motion demo and assert that the runtime reports the effect stack.",
        )
        .then(Action::WaitFrames(60))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let camera = camera_entity(world);
            world
                .resource_mut::<Messages<CameraShakeRequest>>()
                .write(CameraShakeRequest {
                    entity: camera,
                    trauma: 0.75,
                    duration_override: None,
                });
            world
                .resource_mut::<Messages<CameraRecoilRequest>>()
                .write(CameraRecoilRequest {
                    entity: camera,
                    pitch: 7.5_f32.to_radians(),
                    yaw: 1.8_f32.to_radians(),
                    duration_override: None,
                });
        })))
        .then(Action::WaitFrames(3))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let camera = camera_entity(world);
            let runtime = world
                .get::<FpsCameraRuntime>(camera)
                .expect("camera runtime should exist");
            assert!(runtime.trauma > 0.1);
            assert!(runtime.recoil_offset.length() > 0.01);
        })))
        .then(Action::Screenshot("fps_external_motion_recoil".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("fps_external_motion_recoil summary"))
        .build()
}
