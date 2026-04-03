use saddle_camera_fps_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_fps_camera::{FpsCameraConfig, FpsCameraPlugin};

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
    common::spawn_reference_world(
        &mut commands,
        &mut meshes,
        &mut materials,
        "FPS Camera Basic",
        Color::srgb(0.78, 0.36, 0.22),
        "Minimal first-person look + move setup with the default runtime.",
    );
    common::spawn_fps_camera(
        &mut commands,
        FpsCameraConfig::default(),
        Vec3::new(0.0, 1.62, 8.0),
    );
}
