use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_enhanced_input::context::InputContextAppExt;
use bevy_enhanced_input::prelude::{
    Action, Axial, Bidirectional, Binding, Bindings, Cancel as InputCancel, Cardinal, Complete,
    EnhancedInputPlugin, Fire, InputAction, Press, Scale, Start, actions, bindings,
};
use saddle_camera_fps_camera::{FpsCamera, FpsCameraConfig, FpsCameraIntent, FpsCameraRuntime, FpsCameraSystems};

#[derive(Component)]
pub struct ExampleOverlay;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExampleSystems {
    Overlay,
}

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub struct MoveAction;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub struct MouseLookAction;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub struct AnalogLookAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct JumpAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct SprintAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct CrouchAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct AimAction;

#[derive(Debug, InputAction)]
#[action_output(f32)]
pub struct LeanAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct FreeLookAction;

pub struct ExampleCameraControlsPlugin;

impl Plugin for ExampleCameraControlsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EnhancedInputPlugin>() {
            app.add_plugins(EnhancedInputPlugin);
        }

        app.add_input_context::<FpsCamera>()
            .configure_sets(
                Update,
                ExampleSystems::Overlay.after(FpsCameraSystems::SyncProjection),
            )
            .add_observer(cache_move_axis)
            .add_observer(clear_move_axis_on_cancel)
            .add_observer(clear_move_axis_on_complete)
            .add_observer(cache_mouse_look)
            .add_observer(cache_analog_look)
            .add_observer(clear_analog_look_on_cancel)
            .add_observer(clear_analog_look_on_complete)
            .add_observer(cache_jump_press)
            .add_observer(cache_sprint_active)
            .add_observer(clear_sprint_active_on_cancel)
            .add_observer(clear_sprint_active_on_complete)
            .add_observer(cache_crouch_active)
            .add_observer(clear_crouch_active_on_cancel)
            .add_observer(clear_crouch_active_on_complete)
            .add_observer(cache_aim_active)
            .add_observer(clear_aim_active_on_cancel)
            .add_observer(clear_aim_active_on_complete)
            .add_observer(cache_lean_axis)
            .add_observer(clear_lean_axis_on_cancel)
            .add_observer(clear_lean_axis_on_complete)
            .add_observer(cache_free_look_active)
            .add_observer(clear_free_look_on_cancel)
            .add_observer(clear_free_look_on_complete)
            .add_systems(
                Update,
                (
                    capture_cursor.run_if(input_just_pressed(MouseButton::Left)),
                    release_cursor.run_if(input_just_pressed(KeyCode::Escape)),
                    sync_overlay.in_set(ExampleSystems::Overlay),
                ),
            );
    }
}

pub fn spawn_reference_world(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    title: &str,
    accent: Color,
    instructions: &str,
) {
    commands.spawn((
        Name::new("Example Sun"),
        DirectionalLight {
            illuminance: 20_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(12.0, 18.0, 7.0).looking_at(Vec3::new(0.0, 0.5, -12.0), Vec3::Y),
    ));
    commands.spawn((
        Name::new("Example Fill Light"),
        PointLight {
            intensity: 60_000.0,
            range: 40.0,
            ..default()
        },
        Transform::from_xyz(0.0, 6.5, -12.0),
    ));
    commands.spawn((
        Name::new("Example Floor"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(80.0, 80.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.09, 0.10, 0.12),
            perceptual_roughness: 1.0,
            ..default()
        })),
    ));

    for index in 0..10 {
        let z = -(index as f32) * 6.0;
        let stripe_color = if index % 2 == 0 {
            Color::srgb(0.14, 0.15, 0.18)
        } else {
            Color::srgb(0.18, 0.20, 0.24)
        };
        commands.spawn((
            Name::new(format!("Floor Stripe {index}")),
            Mesh3d(meshes.add(Cuboid::new(18.0, 0.02, 2.8))),
            MeshMaterial3d(materials.add(stripe_color)),
            Transform::from_xyz(0.0, 0.01, z),
        ));

        for side in [-1.0, 1.0] {
            commands.spawn((
                Name::new(format!("Corridor Pillar {index} {side}")),
                Mesh3d(meshes.add(Cuboid::new(0.7, 3.5, 0.7))),
                MeshMaterial3d(materials.add(if side < 0.0 {
                    accent
                } else {
                    Color::srgb(0.24, 0.26, 0.31)
                })),
                Transform::from_xyz(side * 5.8, 1.75, z),
            ));
        }
    }

    commands.spawn((
        Name::new("Left Wall"),
        Mesh3d(meshes.add(Cuboid::new(0.3, 3.2, 66.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.30, 0.18, 0.15))),
        Transform::from_xyz(-7.5, 1.6, -24.0),
    ));
    commands.spawn((
        Name::new("Right Wall"),
        Mesh3d(meshes.add(Cuboid::new(0.3, 3.2, 66.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.14, 0.20, 0.29))),
        Transform::from_xyz(7.5, 1.6, -24.0),
    ));
    commands.spawn((
        Name::new("End Gate"),
        Mesh3d(meshes.add(Cuboid::new(8.0, 3.6, 0.35))),
        MeshMaterial3d(materials.add(accent)),
        Transform::from_xyz(0.0, 1.8, -55.0),
    ));

    commands.spawn((
        Name::new("Example Overlay"),
        ExampleOverlay,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(18.0),
            width: Val::Px(420.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.04, 0.07, 0.80)),
        Text::new(format!("{title}\n{instructions}")),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

pub fn spawn_fps_camera(
    commands: &mut Commands,
    config: FpsCameraConfig,
    translation: Vec3,
) -> Entity {
    commands
        .spawn((
            Name::new("Example FPS Camera"),
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: config.fov.base_fov,
                ..default()
            }),
            Transform::from_translation(translation),
            FpsCamera,
            config,
            actions!(FpsCamera[
                (
                    Action::<MoveAction>::new(),
                    Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
                ),
                (
                    Action::<MouseLookAction>::new(),
                    Bindings::spawn((Spawn((Binding::mouse_motion(), Scale::splat(1.0))),)),
                ),
                (Action::<AnalogLookAction>::new(), Bindings::spawn(Axial::right_stick())),
                (
                    Action::<JumpAction>::new(),
                    Press::default(),
                    bindings![KeyCode::Space, GamepadButton::South],
                ),
                (
                    Action::<SprintAction>::new(),
                    bindings![KeyCode::ShiftLeft, GamepadButton::LeftTrigger2],
                ),
                (
                    Action::<CrouchAction>::new(),
                    bindings![KeyCode::ControlLeft, KeyCode::KeyC, GamepadButton::East],
                ),
                (
                    Action::<AimAction>::new(),
                    bindings![MouseButton::Right, GamepadButton::RightTrigger2],
                ),
                (
                    Action::<LeanAction>::new(),
                    Bindings::spawn((
                        Bidirectional::new(KeyCode::KeyQ, KeyCode::KeyE),
                        Bidirectional::new(GamepadButton::DPadLeft, GamepadButton::DPadRight),
                    )),
                ),
                (
                    Action::<FreeLookAction>::new(),
                    bindings![KeyCode::AltLeft, GamepadButton::West],
                ),
            ]),
        ))
        .id()
}

fn cache_move_axis(
    trigger: On<Fire<MoveAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.move_axis = trigger.value;
    }
}

fn clear_move_axis_on_cancel(
    trigger: On<InputCancel<MoveAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.move_axis = Vec2::ZERO;
    }
}

fn clear_move_axis_on_complete(
    trigger: On<Complete<MoveAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.move_axis = Vec2::ZERO;
    }
}

fn cache_mouse_look(
    trigger: On<Fire<MouseLookAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.look_delta += trigger.value;
    }
}

fn cache_analog_look(
    trigger: On<Fire<AnalogLookAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.look_analog = trigger.value;
    }
}

fn clear_analog_look_on_cancel(
    trigger: On<InputCancel<AnalogLookAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.look_analog = Vec2::ZERO;
    }
}

fn clear_analog_look_on_complete(
    trigger: On<Complete<AnalogLookAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.look_analog = Vec2::ZERO;
    }
}

fn cache_jump_press(
    trigger: On<Start<JumpAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.jump_pressed = true;
    }
}

fn cache_sprint_active(
    trigger: On<Fire<SprintAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.sprint_pressed = trigger.value;
    }
}

fn clear_sprint_active_on_cancel(
    trigger: On<InputCancel<SprintAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.sprint_pressed = false;
    }
}

fn clear_sprint_active_on_complete(
    trigger: On<Complete<SprintAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.sprint_pressed = false;
    }
}

fn cache_crouch_active(
    trigger: On<Fire<CrouchAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.crouch_pressed = trigger.value;
    }
}

fn clear_crouch_active_on_cancel(
    trigger: On<InputCancel<CrouchAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.crouch_pressed = false;
    }
}

fn clear_crouch_active_on_complete(
    trigger: On<Complete<CrouchAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.crouch_pressed = false;
    }
}

fn cache_aim_active(
    trigger: On<Fire<AimAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.aim_pressed = trigger.value;
    }
}

fn clear_aim_active_on_cancel(
    trigger: On<InputCancel<AimAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.aim_pressed = false;
    }
}

fn clear_aim_active_on_complete(
    trigger: On<Complete<AimAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.aim_pressed = false;
    }
}

fn cache_lean_axis(
    trigger: On<Fire<LeanAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.lean = trigger.value;
    }
}

fn clear_lean_axis_on_cancel(
    trigger: On<InputCancel<LeanAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.lean = 0.0;
    }
}

fn clear_lean_axis_on_complete(
    trigger: On<Complete<LeanAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.lean = 0.0;
    }
}

fn cache_free_look_active(
    trigger: On<Fire<FreeLookAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.free_look = trigger.value;
    }
}

fn clear_free_look_on_cancel(
    trigger: On<InputCancel<FreeLookAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.free_look = false;
    }
}

fn clear_free_look_on_complete(
    trigger: On<Complete<FreeLookAction>>,
    mut query: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = query.get_mut(trigger.context) {
        intent.free_look = false;
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

fn sync_overlay(
    runtime: Single<(&FpsCameraRuntime, &FpsCameraConfig), With<FpsCamera>>,
    mut overlay: Single<&mut Text, With<ExampleOverlay>>,
) {
    let (runtime, config) = runtime.into_inner();
    overlay.0 = format!(
        "FPS Camera\nWASD move, mouse look, Space jump, Shift sprint, Ctrl crouch\nRMB aim, Q/E lean, Alt free look, Esc release cursor\n\n\
        yaw {:.2}  pitch {:.2}\n\
        speed {:.2}  ratio {:.2}  grounded {}\n\
        crouch {:.2}  sprint {:.2}  aim {:.2}\n\
        trauma {:.2}  bob {:.2}  fov {:.1}\n\
        comfort bob {:.2} roll {:.2} shake {:.2}",
        runtime.yaw,
        runtime.pitch,
        runtime.speed,
        runtime.speed_ratio,
        runtime.grounded,
        runtime.crouch_alpha,
        runtime.sprint_alpha,
        runtime.aim_alpha,
        runtime.trauma,
        runtime.bob_phase,
        runtime.visual_fov.to_degrees(),
        config.comfort.bob_weight,
        config.comfort.roll_weight,
        config.comfort.shake_weight,
    );
}
