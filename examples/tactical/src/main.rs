//! # FPS Camera — Tactical Example
//!
//! Precision preset: low bob, stronger ADS (aim-down-sight), deliberate lean,
//! and free-look support. Good for tactical shooters and stealth games.

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
use saddle_camera_fps_camera::{
    AimConfig, FpsCamera, FpsCameraConfig, FpsCameraIntent, FpsCameraPlugin, FpsCameraRuntime,
    FpsCameraSystems, HeadBobConfig, LeanConfig, TiltConfig,
};
use saddle_pane::prelude::*;

// ---------------------------------------------------------------------------
// Input actions
// ---------------------------------------------------------------------------

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct MoveAction;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct MouseLookAction;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct AnalogLookAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct JumpAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct SprintAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct CrouchAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct AimAction;

#[derive(Debug, InputAction)]
#[action_output(f32)]
struct LeanAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct FreeLookAction;

// ---------------------------------------------------------------------------
// Pane — tactical-specific tunables: aim, lean, tilt
// ---------------------------------------------------------------------------

#[derive(Resource, Pane)]
#[pane(title = "FPS Camera — Tactical", position = "top-right")]
struct FpsCameraPane {
    #[pane(tab = "Aim", slider, min = 0.1, max = 1.0, step = 0.05)]
    aim_sensitivity_scale: f32,
    #[pane(tab = "Aim", slider, min = 0.4, max = 1.0, step = 0.02)]
    aim_fov_multiplier: f32,
    #[pane(tab = "Lean", slider, min = 0.0, max = 30.0, step = 0.5)]
    lean_max_angle_degrees: f32,
    #[pane(tab = "Lean", slider, min = 0.0, max = 0.4, step = 0.01)]
    lean_lateral_offset: f32,
    #[pane(tab = "Tilt", slider, min = 0.0, max = 6.0, step = 0.5)]
    tilt_max_roll_degrees: f32,
    #[pane(tab = "Runtime", monitor)]
    speed: f32,
    #[pane(tab = "Runtime", monitor)]
    aim_alpha: f32,
    #[pane(tab = "Runtime", monitor)]
    visual_fov_degrees: f32,
}

impl Default for FpsCameraPane {
    fn default() -> Self {
        Self {
            aim_sensitivity_scale: 0.5,
            aim_fov_multiplier: 0.72,
            lean_max_angle_degrees: 14.0,
            lean_lateral_offset: 0.14,
            tilt_max_roll_degrees: 2.0,
            speed: 0.0,
            aim_alpha: 0.0,
            visual_fov_degrees: 70.0,
        }
    }
}

#[derive(Component)]
struct Overlay;

#[derive(Component)]
struct CursorIndicator;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        FpsCameraPlugin::default(),
        EnhancedInputPlugin,
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        saddle_pane::PanePlugin,
    ))
    .add_input_context::<FpsCamera>()
    .register_pane::<FpsCameraPane>()
    .add_observer(on_move)
    .add_observer(on_move_cancel)
    .add_observer(on_move_complete)
    .add_observer(on_mouse_look)
    .add_observer(on_analog_look)
    .add_observer(on_analog_look_cancel)
    .add_observer(on_analog_look_complete)
    .add_observer(on_jump)
    .add_observer(on_sprint)
    .add_observer(on_sprint_cancel)
    .add_observer(on_sprint_complete)
    .add_observer(on_crouch)
    .add_observer(on_crouch_cancel)
    .add_observer(on_crouch_complete)
    .add_observer(on_aim)
    .add_observer(on_aim_cancel)
    .add_observer(on_aim_complete)
    .add_observer(on_lean)
    .add_observer(on_lean_cancel)
    .add_observer(on_lean_complete)
    .add_observer(on_free_look)
    .add_observer(on_free_look_cancel)
    .add_observer(on_free_look_complete)
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (
            capture_cursor.run_if(input_just_pressed(MouseButton::Left)),
            release_cursor.run_if(input_just_pressed(KeyCode::Escape)),
            toggle_cursor.run_if(input_just_pressed(KeyCode::Tab)),
            sync_pane.after(FpsCameraSystems::SyncProjection),
            sync_overlay.after(FpsCameraSystems::SyncProjection),
            sync_cursor_indicator.after(FpsCameraSystems::SyncProjection),
        ),
    );
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_reference_scene(&mut commands, &mut meshes, &mut materials);

    // Tactical configuration — precision focus.
    // Low bob, tight ADS with halved sensitivity and zoomed FOV,
    // generous lean angle with lateral translation, and subtle tilt.
    let config = FpsCameraConfig {
        head_bob: HeadBobConfig {
            amplitude: Vec3::new(0.014, 0.02, 0.008),
            stride_length: 1.8,
            ..default()
        },
        aim: AimConfig {
            sensitivity_scale: 0.5,
            fov_multiplier: 0.72,
            ..default()
        },
        lean: LeanConfig {
            max_angle: 14.0_f32.to_radians(),
            lateral_offset: 0.14,
            ..default()
        },
        tilt: TiltConfig {
            max_roll: 2.0_f32.to_radians(),
            ..default()
        },
        ..default()
    };

    commands.spawn((
        Name::new("FPS Camera"),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: config.fov.base_fov,
            ..default()
        }),
        Transform::from_xyz(0.0, 1.62, 8.0),
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
    ));
}

fn spawn_reference_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let accent = Color::srgb(0.24, 0.44, 0.74);

    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 20_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(12.0, 18.0, 7.0).looking_at(Vec3::new(0.0, 0.5, -12.0), Vec3::Y),
    ));
    commands.spawn((
        Name::new("Fill Light"),
        PointLight {
            intensity: 60_000.0,
            range: 40.0,
            ..default()
        },
        Transform::from_xyz(0.0, 6.5, -12.0),
    ));
    commands.spawn((
        Name::new("Floor"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(80.0, 80.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.09, 0.10, 0.12),
            perceptual_roughness: 1.0,
            ..default()
        })),
    ));
    for i in 0..10 {
        let z = -(i as f32) * 6.0;
        let stripe = if i % 2 == 0 {
            Color::srgb(0.14, 0.15, 0.18)
        } else {
            Color::srgb(0.18, 0.20, 0.24)
        };
        commands.spawn((
            Name::new(format!("Floor Stripe {i}")),
            Mesh3d(meshes.add(Cuboid::new(18.0, 0.02, 2.8))),
            MeshMaterial3d(materials.add(stripe)),
            Transform::from_xyz(0.0, 0.01, z),
        ));
        for side in [-1.0, 1.0] {
            commands.spawn((
                Name::new(format!("Pillar {i} {side}")),
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
        Name::new("Overlay"),
        Overlay,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(18.0),
            width: Val::Px(420.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.04, 0.07, 0.80)),
        Text::new(
            "FPS Camera Tactical\n\
            Precision preset: low bob, stronger ADS, deliberate lean, and free-look support.",
        ),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));

    commands.spawn((
        Name::new("Cursor State Indicator"),
        CursorIndicator,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Px(18.0),
            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
            margin: UiRect::left(Val::Px(-100.0)),
            width: Val::Px(200.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.70)),
        Text::new("[Click to capture mouse]"),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 0.6, 0.9)),
    ));
}

// ---------------------------------------------------------------------------
// Cursor
// ---------------------------------------------------------------------------

fn is_cursor_captured(cursor: &CursorOptions) -> bool {
    matches!(
        cursor.grab_mode,
        CursorGrabMode::Locked | CursorGrabMode::Confined
    )
}

fn capture_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor.visible = false;
    cursor.grab_mode = CursorGrabMode::Locked;
}

fn release_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor.visible = true;
    cursor.grab_mode = CursorGrabMode::None;
}

fn toggle_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    if is_cursor_captured(&cursor) {
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
    } else {
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
    }
}

fn sync_cursor_indicator(
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
    mut indicator: Query<(&mut Text, &mut BackgroundColor, &mut TextColor), With<CursorIndicator>>,
) {
    let Ok((mut text, mut bg, mut color)) = indicator.single_mut() else {
        return;
    };
    if is_cursor_captured(&cursor) {
        text.0 = "Mouse captured [Tab/Esc to release]".into();
        bg.0 = Color::srgba(0.0, 0.2, 0.0, 0.50);
        color.0 = Color::srgba(0.7, 1.0, 0.7, 0.7);
    } else {
        text.0 = "Mouse free [Click to capture, Tab to toggle]".into();
        bg.0 = Color::srgba(0.3, 0.15, 0.0, 0.70);
        color.0 = Color::srgba(1.0, 1.0, 0.6, 0.9);
    }
}

// ---------------------------------------------------------------------------
// Pane sync
// ---------------------------------------------------------------------------

fn sync_pane(
    mut pane: ResMut<FpsCameraPane>,
    mut cameras: Query<(&mut FpsCameraConfig, &FpsCameraRuntime), With<FpsCamera>>,
) {
    let Some((mut config, runtime)) = cameras.iter_mut().next() else {
        return;
    };
    let added = pane.is_added();

    if added {
        let p = pane.bypass_change_detection();
        p.aim_sensitivity_scale = config.aim.sensitivity_scale;
        p.aim_fov_multiplier = config.aim.fov_multiplier;
        p.lean_max_angle_degrees = config.lean.max_angle.to_degrees();
        p.lean_lateral_offset = config.lean.lateral_offset;
        p.tilt_max_roll_degrees = config.tilt.max_roll.to_degrees();
    }

    if pane.is_changed() && !added {
        config.aim.sensitivity_scale = pane.aim_sensitivity_scale;
        config.aim.fov_multiplier = pane.aim_fov_multiplier;
        config.lean.max_angle = pane.lean_max_angle_degrees.to_radians();
        config.lean.lateral_offset = pane.lean_lateral_offset;
        config.tilt.max_roll = pane.tilt_max_roll_degrees.to_radians();
    }

    let p = pane.bypass_change_detection();
    p.speed = runtime.speed;
    p.aim_alpha = runtime.aim_alpha;
    p.visual_fov_degrees = runtime.visual_fov.to_degrees();
}

// ---------------------------------------------------------------------------
// Overlay
// ---------------------------------------------------------------------------

fn sync_overlay(
    runtime: Single<(&FpsCameraRuntime, &FpsCameraConfig), With<FpsCamera>>,
    mut overlay: Single<&mut Text, With<Overlay>>,
) {
    let (runtime, config) = runtime.into_inner();
    overlay.0 = format!(
        "FPS Camera Tactical\nWASD move, mouse look, RMB aim, Q/E lean, Alt free look\n\
        Tab toggle cursor, Esc release cursor\n\n\
        speed {:.2}  aim {:.2}  fov {:.1}\n\
        aim sens {:.2}  aim fov x{:.2}\n\
        lean max {:.1} deg  offset {:.2}\n\
        tilt max roll {:.1} deg",
        runtime.speed,
        runtime.aim_alpha,
        runtime.visual_fov.to_degrees(),
        config.aim.sensitivity_scale,
        config.aim.fov_multiplier,
        config.lean.max_angle.to_degrees(),
        config.lean.lateral_offset,
        config.tilt.max_roll.to_degrees(),
    );
}

// ---------------------------------------------------------------------------
// Input observers
// ---------------------------------------------------------------------------

fn on_move(trigger: On<Fire<MoveAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.move_axis = trigger.value;
    }
}
fn on_move_cancel(
    trigger: On<InputCancel<MoveAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.move_axis = Vec2::ZERO;
    }
}
fn on_move_complete(
    trigger: On<Complete<MoveAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.move_axis = Vec2::ZERO;
    }
}
fn on_mouse_look(
    trigger: On<Fire<MouseLookAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
    window: Single<&CursorOptions, With<PrimaryWindow>>,
) {
    if !is_cursor_captured(&window) {
        return;
    }
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.look_delta += trigger.value;
    }
}
fn on_analog_look(
    trigger: On<Fire<AnalogLookAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.look_analog = trigger.value;
    }
}
fn on_analog_look_cancel(
    trigger: On<InputCancel<AnalogLookAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.look_analog = Vec2::ZERO;
    }
}
fn on_analog_look_complete(
    trigger: On<Complete<AnalogLookAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.look_analog = Vec2::ZERO;
    }
}
fn on_jump(trigger: On<Start<JumpAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.jump_pressed = true;
    }
}
fn on_sprint(trigger: On<Fire<SprintAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.sprint_pressed = trigger.value;
    }
}
fn on_sprint_cancel(
    trigger: On<InputCancel<SprintAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.sprint_pressed = false;
    }
}
fn on_sprint_complete(
    trigger: On<Complete<SprintAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.sprint_pressed = false;
    }
}
fn on_crouch(trigger: On<Fire<CrouchAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.crouch_pressed = trigger.value;
    }
}
fn on_crouch_cancel(
    trigger: On<InputCancel<CrouchAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.crouch_pressed = false;
    }
}
fn on_crouch_complete(
    trigger: On<Complete<CrouchAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.crouch_pressed = false;
    }
}
fn on_aim(trigger: On<Fire<AimAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.aim_pressed = trigger.value;
    }
}
fn on_aim_cancel(
    trigger: On<InputCancel<AimAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.aim_pressed = false;
    }
}
fn on_aim_complete(
    trigger: On<Complete<AimAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.aim_pressed = false;
    }
}
fn on_lean(trigger: On<Fire<LeanAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.lean = trigger.value;
    }
}
fn on_lean_cancel(
    trigger: On<InputCancel<LeanAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.lean = 0.0;
    }
}
fn on_lean_complete(
    trigger: On<Complete<LeanAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.lean = 0.0;
    }
}
fn on_free_look(
    trigger: On<Fire<FreeLookAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.free_look = trigger.value;
    }
}
fn on_free_look_cancel(
    trigger: On<InputCancel<FreeLookAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.free_look = false;
    }
}
fn on_free_look_complete(
    trigger: On<Complete<FreeLookAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut i) = q.get_mut(trigger.context) {
        i.free_look = false;
    }
}
