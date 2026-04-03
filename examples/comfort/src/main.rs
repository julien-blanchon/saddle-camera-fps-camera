use saddle_camera_fps_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_fps_camera::{
    ComfortConfig, FpsCameraConfig, FpsCameraPlugin, HeadBobConfig, TiltConfig,
};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        FpsCameraPlugin::default(),
        common::ExampleCameraControlsPlugin,
    ));
    common::add_debug_pane(&mut app);
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let config = FpsCameraConfig {
        comfort: ComfortConfig::low_motion(),
        head_bob: HeadBobConfig {
            amplitude: Vec3::new(0.012, 0.015, 0.004),
            idle_sway_translation: Vec3::new(0.002, 0.002, 0.001),
            idle_sway_rotation: Vec2::new(0.002, 0.002),
            ..default()
        },
        tilt: TiltConfig {
            max_roll: 1.0_f32.to_radians(),
            ..default()
        },
        ..default()
    };

    common::spawn_reference_world(
        &mut commands,
        &mut meshes,
        &mut materials,
        "FPS Camera Comfort",
        Color::srgb(0.72, 0.66, 0.24),
        "Low-motion preset with damped bob, roll, shake, landing, and dynamic FOV.",
    );
    common::spawn_fps_camera(&mut commands, config, Vec3::new(0.0, 1.62, 8.0));
}
