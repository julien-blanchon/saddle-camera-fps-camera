#[cfg(feature = "e2e")]
mod e2e;

use avian3d::prelude::*;
use bevy::{
    input::common_conditions::input_just_pressed,
    light::NotShadowCaster,
    prelude::*,
    transform::TransformSystems,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowPlugin},
};
use bevy_enhanced_input::prelude::{
    Action, Axial, Binding, Bindings, Cardinal, DeadZone, Fire, InputAction, Scale, actions,
    bindings,
};
use bevy_enhanced_input::preset::WithBundle;
use saddle_camera_fps_camera::{
    CameraRecoilRequest, CameraShakeRequest, FpsCamera, FpsCameraConfig, FpsCameraEffectsPlugin,
    FpsCameraExternalMotion, FpsCameraPlugin, FpsCameraRuntime, FpsCameraSystems,
    ShakeNoiseProfile,
};
use saddle_camera_fps_camera_example_common as common;
use saddle_character_controller::{
    CharacterController, CharacterControllerPlugin, CharacterControllerState,
    CharacterControllerSystems, CharacterLanded, CharacterLook, CharacterPush, CrouchAction,
    JumpAction, LookAction, MoveAction, SprintAction, TraverseAction,
};
use saddle_pane::prelude::*;

#[derive(Component)]
struct DemoPlayer;

#[derive(Component)]
struct ViewmodelRoot;

#[derive(Component)]
struct MovingPlatform {
    origin: Vec3,
    translation_axis: Vec3,
    amplitude: f32,
    speed: f32,
    phase: f32,
}

#[derive(Resource, Default)]
struct PendingLandingImpulse(f32);

#[derive(Resource, Pane)]
#[pane(title = "Character Controller", position = "top-left")]
struct ControllerPane {
    #[pane(tab = "Movement", slider, min = 4.0, max = 18.0, step = 0.25)]
    speed: f32,
    #[pane(tab = "Movement", slider, min = 1.0, max = 2.0, step = 0.05)]
    sprint_speed_scale: f32,
    #[pane(tab = "Movement", slider, min = 0.2, max = 1.2, step = 0.05)]
    step_size: f32,
    #[pane(tab = "Jump", slider, min = 1.0, max = 3.2, step = 0.1)]
    jump_height: f32,
    #[pane(tab = "Runtime", monitor)]
    grounded: bool,
    #[pane(tab = "Runtime", monitor)]
    movement_mode: String,
}

impl Default for ControllerPane {
    fn default() -> Self {
        let controller = CharacterController::default();
        Self {
            speed: controller.speed,
            sprint_speed_scale: controller.sprint_speed_scale,
            step_size: controller.step_size,
            jump_height: controller.jump_height,
            grounded: true,
            movement_mode: "Grounded".into(),
        }
    }
}

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct FireWeaponAction;

fn main() {
    let mut app = App::new();
    app.insert_resource(Time::<Fixed>::from_hz(72.0))
        .init_resource::<PendingLandingImpulse>()
        .init_resource::<ControllerPane>()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "saddle-camera-fps-camera external_motion".into(),
                    resolution: (1440, 900).into(),
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default(),
            FpsCameraPlugin::default(),
            FpsCameraEffectsPlugin::default(),
            CharacterControllerPlugin::always_on(FixedUpdate),
        ));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::ExternalMotionE2EPlugin);
    common::add_debug_pane(&mut app);
    app.register_pane::<ControllerPane>()
        .add_observer(fire_weapon)
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            animate_platforms.before(CharacterControllerSystems::Grounding),
        )
        .add_systems(
            Update,
            (
                capture_cursor.run_if(input_just_pressed(MouseButton::Left)),
                release_cursor.run_if(input_just_pressed(KeyCode::Escape)),
                relay_character_landings.before(FpsCameraSystems::ReadIntent),
                sync_camera_external_motion.before(FpsCameraSystems::UpdateLocomotion),
                sync_camera_look
                    .after(FpsCameraSystems::ReadIntent)
                    .before(FpsCameraSystems::UpdateCameraState),
                sync_controller_pane,
            ),
        )
        .add_systems(
            PostUpdate,
            sync_viewmodel_pose
                .after(FpsCameraSystems::SyncTransform)
                .before(TransformSystems::Propagate),
        );
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    common::spawn_reference_world(
        &mut commands,
        &mut meshes,
        &mut materials,
        "FPS Camera External Motion",
        Color::srgb(0.88, 0.42, 0.18),
        "WASD / left stick moves the controller. Mouse / right stick looks. Space jumps. Shift sprints.\nLeft click fires recoil. Ride the moving bridge to feel support motion and viewmodel lag.",
    );
    spawn_collision_corridor(&mut commands);
    spawn_gameplay_props(&mut commands, &mut meshes, &mut materials);

    spawn_player(&mut commands);
    let camera = spawn_bridge_camera(&mut commands);
    spawn_viewmodel(&mut commands, &mut meshes, &mut materials, camera);
}

fn spawn_player(commands: &mut Commands) -> Entity {
    let controller = CharacterController {
        speed: 10.0,
        sprint_speed_scale: 1.35,
        jump_height: 1.9,
        step_size: 0.65,
        standing_view_height: 1.65,
        crouch_view_height: 1.15,
        ..default()
    };
    let transform = Transform::from_xyz(0.0, 2.2, 8.0);

    commands
        .spawn((
            Name::new("Player Controller"),
            DemoPlayer,
            controller,
            CharacterPush::default(),
            CharacterLook {
                sensitivity: Vec2::splat(0.0022),
                ..default()
            },
            transform,
            actions!(CharacterController[
                (
                    Action::<MoveAction>::new(),
                    DeadZone::default(),
                    Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
                ),
                (
                    Action::<LookAction>::new(),
                    Bindings::spawn((
                        Spawn((Binding::mouse_motion(), Scale::splat(0.0024))),
                        Axial::right_stick().with((Scale::splat(0.06), DeadZone::default())),
                    )),
                ),
                (Action::<JumpAction>::new(), bindings![KeyCode::Space, GamepadButton::South]),
                (
                    Action::<SprintAction>::new(),
                    bindings![KeyCode::ShiftLeft, GamepadButton::LeftTrigger2],
                ),
                (
                    Action::<CrouchAction>::new(),
                    bindings![KeyCode::ControlLeft, KeyCode::KeyC, GamepadButton::East],
                ),
                (
                    Action::<TraverseAction>::new(),
                    bindings![KeyCode::KeyE, GamepadButton::RightTrigger],
                ),
                (
                    Action::<FireWeaponAction>::new(),
                    bindings![MouseButton::Left, GamepadButton::RightTrigger2],
                ),
            ]),
        ))
        .id()
}

fn spawn_bridge_camera(commands: &mut Commands) -> Entity {
    let mut config = FpsCameraConfig::default();
    config.look.sensitivity = Vec2::new(0.0024, 0.0022);
    config.fov.base_fov = 79.0_f32.to_radians();
    config.fov.sprint_boost = 6.0_f32.to_radians();
    config.shake.noise_profile = ShakeNoiseProfile::Handheld;
    config.viewmodel.enabled = true;
    config.viewmodel.translation_scale = Vec3::new(0.012, 0.012, 0.05);
    config.viewmodel.rotation_scale = Vec3::new(0.22, 0.42, 0.10);
    config.viewmodel.movement_scale = Vec3::new(0.010, 0.0, 0.030);
    config.viewmodel.max_translation = Vec3::new(0.09, 0.07, 0.12);
    config.viewmodel.max_rotation = Vec3::new(0.26, 0.36, 0.20);

    commands
        .spawn((
            Name::new("Bridge FPS Camera"),
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: config.fov.base_fov,
                ..default()
            }),
            Transform::from_xyz(0.0, 3.85, 8.0),
            FpsCamera,
            config,
            FpsCameraExternalMotion {
                enabled: true,
                ..default()
            },
        ))
        .id()
}

fn spawn_viewmodel(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    camera: Entity,
) {
    let body_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.20, 0.24),
        metallic: 0.12,
        perceptual_roughness: 0.38,
        ..default()
    });
    let accent_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.90, 0.44, 0.20),
        emissive: LinearRgba::rgb(0.18, 0.06, 0.02),
        perceptual_roughness: 0.28,
        ..default()
    });

    commands.entity(camera).with_children(|parent| {
        parent
            .spawn((
                Name::new("Viewmodel Root"),
                ViewmodelRoot,
                Visibility::Inherited,
                Transform::from_xyz(0.34, -0.32, -0.72),
            ))
            .with_children(|viewmodel| {
                viewmodel.spawn((
                    Name::new("Receiver"),
                    NotShadowCaster,
                    Mesh3d(meshes.add(Cuboid::new(0.18, 0.16, 0.78))),
                    MeshMaterial3d(body_material.clone()),
                    Transform::from_xyz(0.02, -0.02, -0.02),
                ));
                viewmodel.spawn((
                    Name::new("Barrel"),
                    NotShadowCaster,
                    Mesh3d(meshes.add(Cuboid::new(0.08, 0.08, 0.62))),
                    MeshMaterial3d(body_material.clone()),
                    Transform::from_xyz(0.01, 0.03, -0.62),
                ));
                viewmodel.spawn((
                    Name::new("Grip"),
                    NotShadowCaster,
                    Mesh3d(meshes.add(Cuboid::new(0.10, 0.22, 0.16))),
                    MeshMaterial3d(accent_material.clone()),
                    Transform::from_xyz(0.03, -0.18, 0.12),
                ));
                viewmodel.spawn((
                    Name::new("Sight"),
                    NotShadowCaster,
                    Mesh3d(meshes.add(Cuboid::new(0.06, 0.08, 0.16))),
                    MeshMaterial3d(accent_material),
                    Transform::from_xyz(0.00, 0.11, -0.12),
                ));
            });
    });
}

fn spawn_collision_corridor(commands: &mut Commands) {
    commands.spawn((
        Name::new("Ground Collider"),
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
    ));
    commands.spawn((
        Name::new("Left Wall Collider"),
        RigidBody::Static,
        Collider::cuboid(0.3, 3.2, 66.0),
        Transform::from_xyz(-7.5, 1.6, -24.0),
    ));
    commands.spawn((
        Name::new("Right Wall Collider"),
        RigidBody::Static,
        Collider::cuboid(0.3, 3.2, 66.0),
        Transform::from_xyz(7.5, 1.6, -24.0),
    ));
    commands.spawn((
        Name::new("End Gate Collider"),
        RigidBody::Static,
        Collider::cuboid(8.0, 3.6, 0.35),
        Transform::from_xyz(0.0, 1.8, -55.0),
    ));

    for index in 0..10 {
        let z = -(index as f32) * 6.0;
        for side in [-1.0, 1.0] {
            commands.spawn((
                Name::new(format!("Pillar Collider {index} {side}")),
                RigidBody::Static,
                Collider::cuboid(0.7, 3.5, 0.7),
                Transform::from_xyz(side * 5.8, 1.75, z),
            ));
        }
    }
}

fn spawn_gameplay_props(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    for (index, position) in [
        Vec3::new(-2.2, 0.65, 2.0),
        Vec3::new(2.4, 0.65, -4.0),
        Vec3::new(-2.8, 0.65, -10.0),
    ]
    .into_iter()
    .enumerate()
    {
        commands.spawn((
            Name::new(format!("Push Crate {}", index + 1)),
            Mesh3d(meshes.add(Cuboid::new(1.3, 1.3, 1.3))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.34, 0.24, 0.18),
                perceptual_roughness: 0.85,
                ..default()
            })),
            Transform::from_translation(position),
            RigidBody::Dynamic,
            Collider::cuboid(1.3, 1.3, 1.3),
        ));
    }

    commands.spawn((
        Name::new("Moving Bridge"),
        MovingPlatform {
            origin: Vec3::new(0.0, 0.55, -18.0),
            translation_axis: Vec3::X,
            amplitude: 3.0,
            speed: 0.95,
            phase: 0.0,
        },
        Mesh3d(meshes.add(Cuboid::new(6.0, 0.25, 3.4))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.18, 0.28, 0.38),
            metallic: 0.05,
            perceptual_roughness: 0.72,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.55, -18.0),
        RigidBody::Kinematic,
        Collider::cuboid(6.0, 0.25, 3.4),
        LinearVelocity::ZERO,
        AngularVelocity::ZERO,
    ));

    for (index, x) in [-3.8, -1.2, 1.5, 4.1].into_iter().enumerate() {
        commands.spawn((
            Name::new(format!("Target Totem {}", index + 1)),
            Mesh3d(meshes.add(Cuboid::new(0.8, 2.6, 0.8))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: if index % 2 == 0 {
                    Color::srgb(0.88, 0.40, 0.22)
                } else {
                    Color::srgb(0.16, 0.46, 0.78)
                },
                emissive: LinearRgba::rgb(0.06, 0.03, 0.01),
                perceptual_roughness: 0.32,
                ..default()
            })),
            Transform::from_xyz(x, 1.3, -34.0 - index as f32 * 4.5),
            RigidBody::Static,
            Collider::cuboid(0.8, 2.6, 0.8),
        ));
    }
}

fn animate_platforms(
    time: Res<Time<Fixed>>,
    mut query: Query<(&MovingPlatform, &mut Transform, &mut LinearVelocity)>,
) {
    let t = time.elapsed_secs();
    for (platform, mut transform, mut velocity) in &mut query {
        let axis = platform.translation_axis.normalize_or_zero();
        let phase = t * platform.speed + platform.phase;
        transform.translation = platform.origin + axis * (platform.amplitude * phase.sin());
        velocity.0 = axis * (platform.amplitude * platform.speed * phase.cos());
    }
}

fn sync_camera_external_motion(
    mut pending_landing: ResMut<PendingLandingImpulse>,
    player: Query<
        (
            &Transform,
            &LinearVelocity,
            &CharacterController,
            &CharacterControllerState,
        ),
        With<DemoPlayer>,
    >,
    mut camera: Query<&mut FpsCameraExternalMotion, With<FpsCamera>>,
) {
    let Ok((transform, velocity, controller, state)) = player.single() else {
        return;
    };
    let Ok(mut camera) = camera.single_mut() else {
        return;
    };
    let view_height = if state.crouching {
        controller.crouch_view_height
    } else {
        controller.standing_view_height
    };
    let landing_impulse = std::mem::take(&mut pending_landing.0);

    camera.enabled = true;
    camera.position = transform.translation;
    camera.velocity = velocity.0;
    camera.grounded = state.ground.is_some();
    camera.landing_impulse = landing_impulse;
    camera.eye_height = Some(view_height);
    camera.crouch_alpha = Some(if state.crouching { 1.0 } else { 0.0 });
    camera.sprint_alpha = Some(
        (velocity.0.xz().length() / (controller.speed * controller.sprint_speed_scale))
            .clamp(0.0, 1.0),
    );
}

fn sync_camera_look(
    player: Query<&CharacterLook, With<DemoPlayer>>,
    mut camera: Query<&mut FpsCameraRuntime, With<FpsCamera>>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    let Ok(mut camera) = camera.single_mut() else {
        return;
    };
    camera.yaw = player.yaw;
    camera.pitch = player.pitch;
}

fn relay_character_landings(
    player: Query<Entity, With<DemoPlayer>>,
    camera: Query<Entity, With<FpsCamera>>,
    mut pending_landing: ResMut<PendingLandingImpulse>,
    mut landings: MessageReader<CharacterLanded>,
    mut shakes: MessageWriter<CameraShakeRequest>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    let Ok(camera) = camera.single() else {
        return;
    };

    for landed in landings.read() {
        if landed.entity != player {
            continue;
        }

        let landing_impulse = (landed.impact_speed / 15.0).clamp(0.0, 1.0);
        pending_landing.0 = pending_landing.0.max(landing_impulse);
        shakes.write(CameraShakeRequest {
            entity: camera,
            trauma: (landing_impulse * 0.28).clamp(0.05, 0.35),
            duration_override: Some(0.18),
        });
    }
}

fn fire_weapon(
    _trigger: On<Fire<FireWeaponAction>>,
    camera: Query<Entity, With<FpsCamera>>,
    mut recoils: MessageWriter<CameraRecoilRequest>,
    mut shakes: MessageWriter<CameraShakeRequest>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };

    recoils.write(CameraRecoilRequest {
        entity: camera,
        pitch: 1.45_f32.to_radians(),
        yaw: 0.28_f32.to_radians(),
        duration_override: Some(0.12),
    });
    shakes.write(CameraShakeRequest {
        entity: camera,
        trauma: 0.06,
        duration_override: Some(0.08),
    });
}

fn sync_controller_pane(
    mut pane: ResMut<ControllerPane>,
    mut controller: Query<(&mut CharacterController, &CharacterControllerState), With<DemoPlayer>>,
) {
    let Ok((mut controller, state)) = controller.single_mut() else {
        return;
    };

    if pane.is_changed() && !pane.is_added() {
        controller.speed = pane.speed;
        controller.sprint_speed_scale = pane.sprint_speed_scale;
        controller.step_size = pane.step_size;
        controller.jump_height = pane.jump_height;
    }

    pane.grounded = state.ground.is_some_and(|ground| ground.walkable);
    pane.movement_mode = format!("{:?}", state.movement_mode);
}

fn sync_viewmodel_pose(
    camera: Query<(&FpsCameraRuntime, &Children), With<FpsCamera>>,
    mut viewmodels: Query<&mut Transform, With<ViewmodelRoot>>,
) {
    let Ok((runtime, children)) = camera.single() else {
        return;
    };
    let base_translation = Vec3::new(0.34, -0.32, -0.72);
    let base_rotation = Vec3::new(-0.18, 0.26, -0.04);
    let recoil_translation = Vec3::new(
        -runtime.recoil_offset.y * 0.10,
        -runtime.recoil_offset.x * 0.06,
        runtime.recoil_offset.x.abs() * 0.08,
    );
    let recoil_rotation = Vec3::new(
        runtime.recoil_offset.x * 0.45,
        -runtime.recoil_offset.y * 0.30,
        runtime.recoil_offset.y * 0.18,
    );

    for child in children.iter() {
        let Ok(mut transform) = viewmodels.get_mut(child) else {
            continue;
        };
        transform.translation =
            base_translation + runtime.viewmodel_translation + recoil_translation;
        transform.rotation = Quat::from_euler(
            EulerRot::XYZ,
            base_rotation.x + runtime.viewmodel_rotation.x + recoil_rotation.x,
            base_rotation.y + runtime.viewmodel_rotation.y + recoil_rotation.y,
            base_rotation.z + runtime.viewmodel_rotation.z + recoil_rotation.z,
        );
    }
}

fn capture_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor.visible = false;
    cursor.grab_mode = CursorGrabMode::Locked;
}

fn release_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor.visible = true;
    cursor.grab_mode = CursorGrabMode::None;
}
