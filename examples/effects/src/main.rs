use saddle_camera_fps_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_fps_camera::{
    CameraRecoilRequest, CameraShakeRequest, FpsCamera, FpsCameraConfig, FpsCameraPlugin,
};

#[derive(Resource)]
struct EffectPulseTimer(Timer);

fn main() {
    let mut app = App::new();
    app.insert_resource(EffectPulseTimer(Timer::from_seconds(
        1.5,
        TimerMode::Repeating,
    )));
    app.add_plugins((
        DefaultPlugins,
        FpsCameraPlugin::default(),
        common::ExampleCameraControlsPlugin,
    ));
    app.add_systems(Startup, setup);
    app.add_systems(Update, pulse_effects);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let config = FpsCameraConfig {
        shake: saddle_camera_fps_camera::ShakeConfig {
            translation_amplitude: Vec3::new(0.05, 0.06, 0.04),
            rotation_amplitude: Vec3::new(0.05, 0.05, 0.04),
            ..default()
        },
        recoil: saddle_camera_fps_camera::RecoilConfig {
            max_pitch: 16.0_f32.to_radians(),
            ..default()
        },
        ..default()
    };

    common::spawn_reference_world(
        &mut commands,
        &mut meshes,
        &mut materials,
        "FPS Camera Effects",
        Color::srgb(0.68, 0.24, 0.22),
        "Timed recoil + trauma pulses show additive effects on top of the base look state.",
    );
    common::spawn_fps_camera(&mut commands, config, Vec3::new(0.0, 1.62, 8.0));
}

fn pulse_effects(
    time: Res<Time>,
    mut timer: ResMut<EffectPulseTimer>,
    camera: Single<Entity, With<FpsCamera>>,
    mut shake_writer: MessageWriter<CameraShakeRequest>,
    mut recoil_writer: MessageWriter<CameraRecoilRequest>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let camera = *camera;
        shake_writer.write(CameraShakeRequest {
            entity: camera,
            trauma: 0.55,
        });
        recoil_writer.write(CameraRecoilRequest {
            entity: camera,
            pitch: 7.0_f32.to_radians(),
            yaw: 1.5_f32.to_radians(),
        });
    }
}
