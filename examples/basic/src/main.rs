//! # FPS Camera -- Basic Example
//!
//! The simplest first-person camera: WASD movement, mouse look, and default config.
//! This example is fully self-contained so you can copy it into your own project.

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
    DecayConfig, FpsCamera, FpsCameraConfig, FpsCameraIntent, FpsCameraPlugin, FpsCameraRuntime,
    FpsCameraSystems,
};
use saddle_pane::prelude::*;

// ---------------------------------------------------------------------------
// Input action definitions -- bevy_enhanced_input needs one type per action
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
// Debug pane -- live editing of camera parameters via saddle-pane
// ---------------------------------------------------------------------------

#[derive(Resource, Pane)]
#[pane(title = "FPS Camera", position = "top-right")]
struct FpsCameraPane {
    #[pane(tab = "Camera", slider, min = 0.0005, max = 0.01, step = 0.0001)]
    sensitivity_x: f32,
    #[pane(tab = "Camera", slider, min = 0.0005, max = 0.01, step = 0.0001)]
    sensitivity_y: f32,
    #[pane(tab = "Camera", slider, min = 50.0, max = 110.0, step = 1.0)]
    base_fov_degrees: f32,
    #[pane(tab = "Camera", slider, min = 0.0, max = 10.0, step = 0.25)]
    sprint_boost_degrees: f32,
    #[pane(tab = "Motion", slider, min = 0.0, max = 1.0, step = 0.05)]
    bob_weight: f32,
    #[pane(tab = "Motion", slider, min = 0.0, max = 1.0, step = 0.05)]
    shake_weight: f32,
    #[pane(tab = "Motion", slider, min = 0.0, max = 1.0, step = 0.05)]
    roll_weight: f32,
    #[pane(tab = "Motion")]
    viewmodel_enabled: bool,
    #[pane(tab = "Motion", slider, min = 4.0, max = 30.0, step = 0.5)]
    viewmodel_response: f32,
    #[pane(tab = "Runtime", monitor)]
    speed: f32,
    #[pane(tab = "Runtime", monitor)]
    sprint_alpha: f32,
    #[pane(tab = "Runtime", monitor)]
    visual_fov_degrees: f32,
    #[pane(tab = "Runtime", monitor)]
    trauma: f32,
    #[pane(tab = "Runtime", monitor)]
    grounded: bool,
}

impl Default for FpsCameraPane {
    fn default() -> Self {
        let config = FpsCameraConfig::default();
        Self {
            sensitivity_x: config.look.sensitivity.x,
            sensitivity_y: config.look.sensitivity.y,
            base_fov_degrees: config.fov.base_fov.to_degrees(),
            sprint_boost_degrees: config.fov.sprint_boost.to_degrees(),
            bob_weight: config.comfort.bob_weight,
            shake_weight: config.comfort.shake_weight,
            roll_weight: config.comfort.roll_weight,
            viewmodel_enabled: config.viewmodel.enabled,
            viewmodel_response: config.viewmodel.response.decay_rate,
            speed: 0.0,
            sprint_alpha: 0.0,
            visual_fov_degrees: config.fov.base_fov.to_degrees(),
            trauma: 0.0,
            grounded: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Overlay -- HUD text showing runtime state
// ---------------------------------------------------------------------------

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
            sync_pane_to_camera.after(FpsCameraSystems::SyncProjection),
            sync_overlay.after(FpsCameraSystems::SyncProjection),
            sync_cursor_indicator.after(FpsCameraSystems::SyncProjection),
        ),
    );
    app.run();
}

// ---------------------------------------------------------------------------
// Setup -- spawn the world, camera, and HUD
// ---------------------------------------------------------------------------

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_reference_scene(&mut commands, &mut meshes, &mut materials);

    let config = FpsCameraConfig::default();
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

// ---------------------------------------------------------------------------
// Reference scene -- floor, pillars, walls, overlay text
// ---------------------------------------------------------------------------

fn spawn_reference_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let accent = Color::srgb(0.78, 0.36, 0.22);

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

    // Overlay HUD
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
            "FPS Camera Basic\nMinimal first-person look + move setup with the default runtime.",
        ),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));

    // Cursor state indicator
    commands.spawn((
        Name::new("Cursor Indicator"),
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
        BackgroundColor(Color::srgba(0.3, 0.15, 0.0, 0.70)),
        Text::new("[Click to capture mouse]"),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 0.6, 0.9)),
    ));
}

// ---------------------------------------------------------------------------
// Cursor grab / release -- click to capture, Escape to release, Tab to toggle
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

// ---------------------------------------------------------------------------
// Pane <-> camera sync
// ---------------------------------------------------------------------------

fn sync_pane_to_camera(
    mut pane: ResMut<FpsCameraPane>,
    mut cameras: Query<(&mut FpsCameraConfig, &FpsCameraRuntime), With<FpsCamera>>,
) {
    let Some((mut config, runtime)) = cameras.iter_mut().next() else {
        return;
    };
    let pane_added = pane.is_added();

    if pane_added {
        let pane = pane.bypass_change_detection();
        pane.sensitivity_x = config.look.sensitivity.x;
        pane.sensitivity_y = config.look.sensitivity.y;
        pane.base_fov_degrees = config.fov.base_fov.to_degrees();
        pane.sprint_boost_degrees = config.fov.sprint_boost.to_degrees();
        pane.bob_weight = config.comfort.bob_weight;
        pane.shake_weight = config.comfort.shake_weight;
        pane.roll_weight = config.comfort.roll_weight;
        pane.viewmodel_enabled = config.viewmodel.enabled;
        pane.viewmodel_response = config.viewmodel.response.decay_rate;
    }

    if pane.is_changed() && !pane_added {
        config.look.sensitivity = Vec2::new(pane.sensitivity_x, pane.sensitivity_y);
        config.fov.base_fov = pane.base_fov_degrees.to_radians();
        config.fov.sprint_boost = pane.sprint_boost_degrees.to_radians();
        config.comfort.bob_weight = pane.bob_weight;
        config.comfort.shake_weight = pane.shake_weight;
        config.comfort.roll_weight = pane.roll_weight;
        config.viewmodel.enabled = pane.viewmodel_enabled;
        config.viewmodel.response = DecayConfig::new(pane.viewmodel_response.max(0.0));
    }

    let pane = pane.bypass_change_detection();
    pane.speed = runtime.speed;
    pane.sprint_alpha = runtime.sprint_alpha;
    pane.visual_fov_degrees = runtime.visual_fov.to_degrees();
    pane.trauma = runtime.trauma;
    pane.grounded = runtime.grounded;
}

// ---------------------------------------------------------------------------
// Overlay sync -- mirrors runtime state as HUD text
// ---------------------------------------------------------------------------

fn sync_overlay(
    runtime: Single<(&FpsCameraRuntime, &FpsCameraConfig), With<FpsCamera>>,
    mut overlay: Single<&mut Text, With<Overlay>>,
) {
    let (runtime, config) = runtime.into_inner();
    overlay.0 = format!(
        "FPS Camera Basic\nWASD move, mouse look, Space jump, Shift sprint, Ctrl crouch\n\
        RMB aim, Q/E lean, Alt free look, Tab toggle cursor\n\n\
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
// Input observers -- translate bevy_enhanced_input events into FpsCameraIntent
// ---------------------------------------------------------------------------

fn on_move(trigger: On<Fire<MoveAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.move_axis = trigger.value;
    }
}
fn on_move_cancel(
    trigger: On<InputCancel<MoveAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.move_axis = Vec2::ZERO;
    }
}
fn on_move_complete(
    trigger: On<Complete<MoveAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.move_axis = Vec2::ZERO;
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
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.look_delta += trigger.value;
    }
}
fn on_analog_look(
    trigger: On<Fire<AnalogLookAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.look_analog = trigger.value;
    }
}
fn on_analog_look_cancel(
    trigger: On<InputCancel<AnalogLookAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.look_analog = Vec2::ZERO;
    }
}
fn on_analog_look_complete(
    trigger: On<Complete<AnalogLookAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.look_analog = Vec2::ZERO;
    }
}
fn on_jump(trigger: On<Start<JumpAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.jump_pressed = true;
    }
}
fn on_sprint(trigger: On<Fire<SprintAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.sprint_pressed = trigger.value;
    }
}
fn on_sprint_cancel(
    trigger: On<InputCancel<SprintAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.sprint_pressed = false;
    }
}
fn on_sprint_complete(
    trigger: On<Complete<SprintAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.sprint_pressed = false;
    }
}
fn on_crouch(trigger: On<Fire<CrouchAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.crouch_pressed = trigger.value;
    }
}
fn on_crouch_cancel(
    trigger: On<InputCancel<CrouchAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.crouch_pressed = false;
    }
}
fn on_crouch_complete(
    trigger: On<Complete<CrouchAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.crouch_pressed = false;
    }
}
fn on_aim(trigger: On<Fire<AimAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.aim_pressed = trigger.value;
    }
}
fn on_aim_cancel(
    trigger: On<InputCancel<AimAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.aim_pressed = false;
    }
}
fn on_aim_complete(
    trigger: On<Complete<AimAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.aim_pressed = false;
    }
}
fn on_lean(trigger: On<Fire<LeanAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.lean = trigger.value;
    }
}
fn on_lean_cancel(
    trigger: On<InputCancel<LeanAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.lean = 0.0;
    }
}
fn on_lean_complete(
    trigger: On<Complete<LeanAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.lean = 0.0;
    }
}
fn on_free_look(
    trigger: On<Fire<FreeLookAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.free_look = trigger.value;
    }
}
fn on_free_look_cancel(
    trigger: On<InputCancel<FreeLookAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.free_look = false;
    }
}
fn on_free_look_complete(
    trigger: On<Complete<FreeLookAction>>,
    mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>,
) {
    if let Ok(mut intent) = q.get_mut(trigger.context) {
        intent.free_look = false;
    }
}
