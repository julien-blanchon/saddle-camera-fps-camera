//! # FPS Camera — Grounded Example
//!
//! Heavier preset for ground-feel: loud head bob, strong landing compression,
//! crouch eye-height shift, and sprint-amplified stride.

use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_enhanced_input::context::InputContextAppExt;
use bevy_enhanced_input::prelude::{
    actions, bindings, Action, Axial, Bidirectional, Binding, Bindings, Cancel as InputCancel,
    Cardinal, Complete, EnhancedInputPlugin, Fire, InputAction, Press, Scale, Start,
};
use saddle_camera_fps_camera::{
    CrouchConfig, FpsCamera, FpsCameraConfig, FpsCameraIntent, FpsCameraPlugin,
    FpsCameraRuntime, FpsCameraSystems, HeadBobConfig, LandingImpactConfig,
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
// Pane — grounded-specific tunables
// ---------------------------------------------------------------------------

#[derive(Resource, Pane)]
#[pane(title = "FPS Camera — Grounded", position = "top-right")]
struct FpsCameraPane {
    #[pane(tab = "Bob", slider, min = 0.0, max = 0.1, step = 0.005)]
    bob_x: f32,
    #[pane(tab = "Bob", slider, min = 0.0, max = 0.12, step = 0.005)]
    bob_y: f32,
    #[pane(tab = "Bob", slider, min = 0.8, max = 2.5, step = 0.05)]
    stride_length: f32,
    #[pane(tab = "Bob", slider, min = 1.0, max = 2.5, step = 0.1)]
    sprint_multiplier: f32,
    #[pane(tab = "Crouch", slider, min = 0.6, max = 1.4, step = 0.05)]
    crouch_eye_height: f32,
    #[pane(tab = "Landing", slider, min = 0.0, max = 0.5, step = 0.01)]
    landing_translation: f32,
    #[pane(tab = "Landing", slider, min = 0.0, max = 20.0, step = 0.5)]
    landing_pitch_degrees: f32,
    #[pane(tab = "Runtime", monitor)]
    speed: f32,
    #[pane(tab = "Runtime", monitor)]
    grounded: bool,
    #[pane(tab = "Runtime", monitor)]
    crouch_alpha: f32,
}

impl Default for FpsCameraPane {
    fn default() -> Self {
        Self {
            bob_x: 0.03,
            bob_y: 0.06,
            stride_length: 1.35,
            sprint_multiplier: 1.5,
            crouch_eye_height: 0.95,
            landing_translation: 0.18,
            landing_pitch_degrees: 9.0,
            speed: 0.0,
            grounded: true,
            crouch_alpha: 0.0,
        }
    }
}

#[derive(Component)]
struct Overlay;

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
            sync_pane.after(FpsCameraSystems::SyncProjection),
            sync_overlay.after(FpsCameraSystems::SyncProjection),
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

    // Grounded configuration — emphasises physicality.
    // Loud head bob with a short stride, strong landing compression, and low crouch.
    let config = FpsCameraConfig {
        head_bob: HeadBobConfig {
            amplitude: Vec3::new(0.03, 0.06, 0.02),
            stride_length: 1.35,
            sprint_multiplier: 1.5,
            ..default()
        },
        crouch: CrouchConfig {
            eye_height: 0.95,
            ..default()
        },
        landing: LandingImpactConfig {
            translation_amount: 0.18,
            pitch_amount: 9.0_f32.to_radians(),
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
    let accent = Color::srgb(0.34, 0.60, 0.44);

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
            "FPS Camera Grounded\n\
            A heavier preset for sprint, crouch, jump, landing compression, and louder bob.",
        ),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

// ---------------------------------------------------------------------------
// Cursor
// ---------------------------------------------------------------------------

fn capture_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor.visible = false;
    cursor.grab_mode = CursorGrabMode::Locked;
}

fn release_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor.visible = true;
    cursor.grab_mode = CursorGrabMode::None;
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
        p.bob_x = config.head_bob.amplitude.x;
        p.bob_y = config.head_bob.amplitude.y;
        p.stride_length = config.head_bob.stride_length;
        p.sprint_multiplier = config.head_bob.sprint_multiplier;
        p.crouch_eye_height = config.crouch.eye_height;
        p.landing_translation = config.landing.translation_amount;
        p.landing_pitch_degrees = config.landing.pitch_amount.to_degrees();
    }

    if pane.is_changed() && !added {
        config.head_bob.amplitude.x = pane.bob_x;
        config.head_bob.amplitude.y = pane.bob_y;
        config.head_bob.stride_length = pane.stride_length;
        config.head_bob.sprint_multiplier = pane.sprint_multiplier;
        config.crouch.eye_height = pane.crouch_eye_height;
        config.landing.translation_amount = pane.landing_translation;
        config.landing.pitch_amount = pane.landing_pitch_degrees.to_radians();
    }

    let p = pane.bypass_change_detection();
    p.speed = runtime.speed;
    p.grounded = runtime.grounded;
    p.crouch_alpha = runtime.crouch_alpha;
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
        "FPS Camera Grounded\nWASD move, mouse look, Space jump, Shift sprint, Ctrl crouch\n\
        RMB aim, Q/E lean, Esc release cursor\n\n\
        speed {:.2}  grounded {}  crouch {:.2}\n\
        bob amp ({:.3}, {:.3})  stride {:.2}  sprint x{:.1}\n\
        landing translation {:.2}  pitch {:.1} deg",
        runtime.speed,
        runtime.grounded,
        runtime.crouch_alpha,
        config.head_bob.amplitude.x,
        config.head_bob.amplitude.y,
        config.head_bob.stride_length,
        config.head_bob.sprint_multiplier,
        config.landing.translation_amount,
        config.landing.pitch_amount.to_degrees(),
    );
}

// ---------------------------------------------------------------------------
// Input observers
// ---------------------------------------------------------------------------

fn on_move(trigger: On<Fire<MoveAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.move_axis = trigger.value; }
}
fn on_move_cancel(trigger: On<InputCancel<MoveAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.move_axis = Vec2::ZERO; }
}
fn on_move_complete(trigger: On<Complete<MoveAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.move_axis = Vec2::ZERO; }
}
fn on_mouse_look(trigger: On<Fire<MouseLookAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.look_delta += trigger.value; }
}
fn on_analog_look(trigger: On<Fire<AnalogLookAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.look_analog = trigger.value; }
}
fn on_analog_look_cancel(trigger: On<InputCancel<AnalogLookAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.look_analog = Vec2::ZERO; }
}
fn on_analog_look_complete(trigger: On<Complete<AnalogLookAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.look_analog = Vec2::ZERO; }
}
fn on_jump(trigger: On<Start<JumpAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.jump_pressed = true; }
}
fn on_sprint(trigger: On<Fire<SprintAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.sprint_pressed = trigger.value; }
}
fn on_sprint_cancel(trigger: On<InputCancel<SprintAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.sprint_pressed = false; }
}
fn on_sprint_complete(trigger: On<Complete<SprintAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.sprint_pressed = false; }
}
fn on_crouch(trigger: On<Fire<CrouchAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.crouch_pressed = trigger.value; }
}
fn on_crouch_cancel(trigger: On<InputCancel<CrouchAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.crouch_pressed = false; }
}
fn on_crouch_complete(trigger: On<Complete<CrouchAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.crouch_pressed = false; }
}
fn on_aim(trigger: On<Fire<AimAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.aim_pressed = trigger.value; }
}
fn on_aim_cancel(trigger: On<InputCancel<AimAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.aim_pressed = false; }
}
fn on_aim_complete(trigger: On<Complete<AimAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.aim_pressed = false; }
}
fn on_lean(trigger: On<Fire<LeanAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.lean = trigger.value; }
}
fn on_lean_cancel(trigger: On<InputCancel<LeanAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.lean = 0.0; }
}
fn on_lean_complete(trigger: On<Complete<LeanAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.lean = 0.0; }
}
fn on_free_look(trigger: On<Fire<FreeLookAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.free_look = trigger.value; }
}
fn on_free_look_cancel(trigger: On<InputCancel<FreeLookAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.free_look = false; }
}
fn on_free_look_complete(trigger: On<Complete<FreeLookAction>>, mut q: Query<&mut FpsCameraIntent, With<FpsCamera>>) {
    if let Ok(mut i) = q.get_mut(trigger.context) { i.free_look = false; }
}
