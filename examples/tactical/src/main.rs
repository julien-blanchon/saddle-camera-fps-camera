use saddle_camera_fps_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_fps_camera::{
    AimConfig, FpsCameraConfig, FpsCameraPlugin, HeadBobConfig, LeanConfig, TiltConfig,
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

    common::spawn_reference_world(
        &mut commands,
        &mut meshes,
        &mut materials,
        "FPS Camera Tactical",
        Color::srgb(0.24, 0.44, 0.74),
        "Precision preset: low bob, stronger ADS, deliberate lean, and free-look support.",
    );
    common::spawn_fps_camera(&mut commands, config, Vec3::new(0.0, 1.62, 8.0));
}
