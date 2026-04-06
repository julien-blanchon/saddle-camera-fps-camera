use saddle_camera_fps_camera_example_common as common;
#[cfg(feature = "e2e")]
mod e2e;

use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    remote::{RemotePlugin, http::RemoteHttpPlugin},
};
#[cfg(feature = "brp")]
use bevy_brp_extras::BrpExtrasPlugin;
use saddle_camera_fps_camera::{
    CameraRecoilRequest, CameraShakeRequest, FpsCamera, FpsCameraConfig, FpsCameraLegacyPlugin,
    HeadBobConfig,
};

#[derive(Component)]
struct LabCrosshair;

#[cfg_attr(not(feature = "e2e"), allow(dead_code))]
#[derive(Resource, Clone, Copy)]
struct LabCameraEntity(Entity);

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "fps_camera_lab".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        FpsCameraLegacyPlugin::default(),
        common::ExampleCameraControlsPlugin,
        RemotePlugin::default(),
    ));
    common::add_debug_pane(&mut app);
    #[cfg(feature = "brp")]
    app.add_plugins(BrpExtrasPlugin::with_http_plugin(
        RemoteHttpPlugin::default(),
    ));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::FpsCameraLabE2EPlugin);

    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            trigger_recoil.run_if(input_just_pressed(KeyCode::KeyR)),
            trigger_shake.run_if(input_just_pressed(KeyCode::KeyH)),
        ),
    );
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let config = FpsCameraConfig {
        head_bob: HeadBobConfig {
            amplitude: Vec3::new(0.022, 0.034, 0.013),
            ..default()
        },
        ..default()
    };

    common::spawn_reference_world(
        &mut commands,
        &mut meshes,
        &mut materials,
        "FPS Camera Lab",
        Color::srgb(0.82, 0.44, 0.20),
        "Workspace lab for BRP and crate-local E2E.\nLeft click locks cursor. R recoil. H shake.",
    );
    let camera = common::spawn_fps_camera(&mut commands, config, Vec3::new(0.0, 1.62, 8.0));
    commands.insert_resource(LabCameraEntity(camera));

    commands.spawn((
        Name::new("Lab Crosshair"),
        LabCrosshair,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            width: Val::Px(10.0),
            height: Val::Px(10.0),
            margin: UiRect::left(Val::Px(-5.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
    ));

    commands.spawn((
        Name::new("Lab Accent Cube"),
        Mesh3d(meshes.add(Cuboid::new(1.2, 1.2, 1.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.92, 0.58, 0.24))),
        Transform::from_xyz(0.0, 0.6, -10.0),
    ));
}

fn trigger_recoil(
    camera: Single<Entity, With<FpsCamera>>,
    mut recoil_writer: MessageWriter<CameraRecoilRequest>,
) {
    recoil_writer.write(CameraRecoilRequest {
        entity: *camera,
        pitch: 7.0_f32.to_radians(),
        yaw: 2.0_f32.to_radians(),
        duration_override: None,
    });
}

fn trigger_shake(
    camera: Single<Entity, With<FpsCamera>>,
    mut shake_writer: MessageWriter<CameraShakeRequest>,
) {
    shake_writer.write(CameraShakeRequest {
        entity: *camera,
        trauma: 0.7,
        duration_override: None,
    });
}
