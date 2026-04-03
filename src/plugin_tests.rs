use bevy::{
    app::PostStartup,
    ecs::{message::Messages, schedule::ScheduleLabel},
    prelude::*,
    time::TimeUpdateStrategy,
};

use crate::{
    AimConfig, CameraRecoilRequest, CameraShakeRequest, CrouchConfig, DecayConfig, FootstepEvent,
    FpsCamera, FpsCameraConfig, FpsCameraExternalMotion, FpsCameraIntent, FpsCameraPlugin,
    FpsCameraRuntime, FpsCameraSystems, LandedEvent, LeanConfig, RecoilConfig,
};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct ActivateSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct DeactivateSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct SimulationSchedule;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AfterCameraState;

#[derive(Resource, Default, Debug, PartialEq, Eq)]
struct OrderLog(Vec<&'static str>);

#[derive(Resource, Default, Debug)]
struct Landings(Vec<LandedEvent>);

fn push_state_marker(mut log: ResMut<OrderLog>) {
    log.0.push("camera");
}

fn push_after_marker(mut log: ResMut<OrderLog>) {
    log.0.push("after");
}

fn collect_landed_events(mut reader: MessageReader<LandedEvent>, mut landings: ResMut<Landings>) {
    landings.0.extend(reader.read().copied());
}

fn spawn_camera(
    app: &mut App,
    config: FpsCameraConfig,
    intent: FpsCameraIntent,
    external_motion: Option<FpsCameraExternalMotion>,
) -> Entity {
    let mut entity = app.world_mut().spawn((
        FpsCamera,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        Transform::from_xyz(0.0, config.movement.eye_height, 0.0),
        config,
        intent,
    ));

    if let Some(external_motion) = external_motion {
        entity.insert(external_motion);
    }

    entity.id()
}

fn start_runtime(app: &mut App) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.finish();
    app.world_mut().run_schedule(PostStartup);
}

#[test]
fn plugin_builds_with_custom_schedule_labels_and_ordering_points() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_schedule(ActivateSchedule)
        .init_schedule(DeactivateSchedule)
        .init_schedule(SimulationSchedule)
        .init_resource::<OrderLog>()
        .add_plugins(FpsCameraPlugin::new(
            ActivateSchedule,
            DeactivateSchedule,
            SimulationSchedule,
        ))
        .configure_sets(
            SimulationSchedule,
            FpsCameraSystems::UpdateCameraState.before(AfterCameraState),
        )
        .add_systems(
            SimulationSchedule,
            (
                push_state_marker.in_set(FpsCameraSystems::UpdateCameraState),
                push_after_marker.in_set(AfterCameraState),
            ),
        );

    app.finish();
    app.world_mut().run_schedule(ActivateSchedule);
    app.world_mut().run_schedule(SimulationSchedule);

    assert_eq!(
        app.world().resource::<OrderLog>().0,
        vec!["camera", "after"]
    );
}

#[test]
fn always_on_constructor_activates_runtime_after_startup() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(FpsCameraPlugin::always_on(Update));

    app.finish();
    app.world_mut().run_schedule(PostStartup);
    app.update();
}

#[test]
fn messages_register_correctly() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(FpsCameraPlugin::default());

    assert!(app.world().contains_resource::<Messages<FootstepEvent>>());
    assert!(app.world().contains_resource::<Messages<LandedEvent>>());
    assert!(app
        .world()
        .contains_resource::<Messages<CameraShakeRequest>>());
    assert!(app
        .world()
        .contains_resource::<Messages<CameraRecoilRequest>>());
}

#[test]
fn projection_sync_updates_actual_projection_component() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(FpsCameraPlugin::always_on(Update));

    spawn_camera(
        &mut app,
        FpsCameraConfig::default(),
        FpsCameraIntent::default(),
        None,
    );

    start_runtime(&mut app);
    app.update();

    let mut query = app.world_mut().query::<(&FpsCameraRuntime, &Projection)>();
    let Ok((runtime, projection)) = query.single(app.world()) else {
        panic!("expected a single fps camera with projection");
    };
    let Projection::Perspective(perspective) = projection else {
        panic!("expected perspective projection");
    };
    assert!((perspective.fov - runtime.visual_fov).abs() < 0.000_1);
}

#[test]
fn disabled_crouch_config_keeps_eye_height_and_alpha_at_standing_values() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(FpsCameraPlugin::always_on(Update));

    let config = FpsCameraConfig {
        crouch: CrouchConfig {
            enabled: false,
            eye_height: 0.9,
            ..default()
        },
        ..default()
    };
    spawn_camera(
        &mut app,
        config.clone(),
        FpsCameraIntent {
            crouch_pressed: true,
            ..default()
        },
        None,
    );

    start_runtime(&mut app);
    app.update();

    let mut query = app.world_mut().query::<&FpsCameraRuntime>();
    let runtime = query.single(app.world()).expect("expected one camera");
    assert_eq!(runtime.crouch_alpha, 0.0);
    assert!((runtime.eye_height - config.movement.eye_height).abs() < 0.000_1);
}

#[test]
fn disabled_external_motion_does_not_override_internal_motion_state() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(FpsCameraPlugin::always_on(Update));

    let external_motion = FpsCameraExternalMotion {
        enabled: false,
        position: Vec3::new(100.0, 3.0, 200.0),
        velocity: Vec3::new(9.0, 0.0, 9.0),
        grounded: false,
        landing_impulse: 1.0,
        crouch_alpha: Some(1.0),
        sprint_alpha: Some(1.0),
    };
    spawn_camera(
        &mut app,
        FpsCameraConfig::default(),
        FpsCameraIntent::default(),
        Some(external_motion),
    );

    start_runtime(&mut app);
    app.update();

    let mut query = app.world_mut().query::<&FpsCameraRuntime>();
    let runtime = query.single(app.world()).expect("expected one camera");
    assert!(runtime.position.length() < 0.01);
    assert_eq!(runtime.crouch_alpha, 0.0);
    assert_eq!(runtime.sprint_alpha, 0.0);
}

#[test]
fn movement_sprint_transition_is_independent_from_ads_transition() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(FpsCameraPlugin::always_on(Update));

    let config = FpsCameraConfig {
        movement: crate::MovementConfig {
            sprint_transition: DecayConfig::new(1.0),
            ..default()
        },
        aim: AimConfig {
            transition: DecayConfig::new(100.0),
            ..default()
        },
        ..default()
    };
    spawn_camera(
        &mut app,
        config,
        FpsCameraIntent {
            sprint_pressed: true,
            ..default()
        },
        None,
    );

    start_runtime(&mut app);
    app.update();

    let mut query = app.world_mut().query::<&FpsCameraRuntime>();
    let runtime = query.single(app.world()).expect("expected one camera");
    assert!(runtime.sprint_alpha < 0.1);
}

#[test]
fn disabled_aim_and_lean_ignore_input() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(FpsCameraPlugin::always_on(Update));

    let config = FpsCameraConfig {
        aim: AimConfig {
            enabled: false,
            fov_multiplier: 0.5,
            ..default()
        },
        lean: LeanConfig {
            enabled: false,
            max_angle: 0.5,
            ..default()
        },
        ..default()
    };
    spawn_camera(
        &mut app,
        config.clone(),
        FpsCameraIntent {
            aim_pressed: true,
            lean: 1.0,
            ..default()
        },
        None,
    );

    start_runtime(&mut app);
    app.update();

    let mut query = app.world_mut().query::<(&FpsCameraRuntime, &Transform)>();
    let (runtime, transform) = query.single(app.world()).expect("expected one camera");
    assert_eq!(runtime.aim_alpha, 0.0);
    assert_eq!(runtime.lean_alpha, 0.0);
    assert!((runtime.visual_fov - config.fov.base_fov).abs() < 0.000_1);
    assert!(transform.rotation.to_euler(EulerRot::YXZ).2.abs() < 0.000_1);
}

#[test]
fn disabled_recoil_config_rejects_recoil_requests() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(FpsCameraPlugin::always_on(Update));

    let config = FpsCameraConfig {
        recoil: RecoilConfig {
            enabled: false,
            ..default()
        },
        ..default()
    };
    let entity = spawn_camera(&mut app, config, FpsCameraIntent::default(), None);

    start_runtime(&mut app);
    app.world_mut()
        .resource_mut::<Messages<CameraRecoilRequest>>()
        .write(CameraRecoilRequest {
            entity,
            pitch: 0.25,
            yaw: 0.15,
            duration_override: None,
        });
    app.update();

    let runtime = app
        .world()
        .get::<FpsCameraRuntime>(entity)
        .expect("runtime should exist");
    assert_eq!(runtime.recoil_offset, Vec2::ZERO);
}

#[test]
fn landed_event_reports_non_zero_impact_speed_after_jump() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Landings>()
        .add_plugins(FpsCameraPlugin::always_on(Update))
        .add_systems(
            Update,
            collect_landed_events.after(FpsCameraSystems::UpdateCameraState),
        );

    let entity = spawn_camera(
        &mut app,
        FpsCameraConfig::default(),
        FpsCameraIntent {
            jump_pressed: true,
            ..default()
        },
        None,
    );

    start_runtime(&mut app);
    app.update();
    {
        let mut runtime = app
            .world_mut()
            .get_mut::<FpsCameraRuntime>(entity)
            .expect("runtime should exist");
        runtime.position.y = 0.0;
        runtime.velocity = Vec3::new(0.0, -6.0, 0.0);
        runtime.grounded = true;
        runtime.recent_landing_impulse = 0.35;
    }
    {
        let mut internal = app
            .world_mut()
            .get_mut::<crate::components::FpsCameraInternalState>(entity)
            .expect("internal state should exist");
        internal.previous_grounded = false;
    }
    app.update();

    let landings = &app.world().resource::<Landings>().0;
    assert!(
        !landings.is_empty(),
        "expected a landing event after the jump"
    );
    assert!(landings[0].impact_speed > 0.0);
}
