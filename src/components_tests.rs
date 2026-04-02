use bevy::prelude::*;

use crate::{
    FpsCamera, FpsCameraConfig, FpsCameraExternalEffects, FpsCameraIntent, FpsCameraRuntime,
};

#[test]
fn fps_camera_requires_core_runtime_components_on_spawn() {
    let mut world = World::new();
    let entity = world.spawn(FpsCamera).id();
    let entity_ref = world.entity(entity);

    assert!(entity_ref.contains::<Transform>());
    assert!(entity_ref.contains::<FpsCameraConfig>());
    assert!(entity_ref.contains::<FpsCameraIntent>());
    assert!(entity_ref.contains::<FpsCameraRuntime>());
    assert!(entity_ref.contains::<FpsCameraExternalEffects>());
}
