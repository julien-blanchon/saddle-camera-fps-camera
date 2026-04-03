use saddle_camera_fps_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_fps_camera::{
    CrouchConfig, FpsCameraConfig, FpsCameraPlugin, HeadBobConfig, LandingImpactConfig,
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

    common::spawn_reference_world(
        &mut commands,
        &mut meshes,
        &mut materials,
        "FPS Camera Grounded",
        Color::srgb(0.34, 0.60, 0.44),
        "A heavier preset for sprint, crouch, jump, landing compression, and louder bob.",
    );
    common::spawn_fps_camera(&mut commands, config, Vec3::new(0.0, 1.62, 8.0));
}
