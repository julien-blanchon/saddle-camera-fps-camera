use bevy::prelude::*;

use crate::{compose_effect_stack, CameraEffectLayer, CameraEffectStack};

#[test]
fn additive_ordering_is_deterministic() {
    let layers = [
        CameraEffectLayer::weighted(Vec3::X, Vec3::new(1.0, 0.0, 0.0), 0.1, 1.0),
        CameraEffectLayer::weighted(Vec3::Y, Vec3::new(0.0, 1.0, 0.0), 0.2, 1.0),
    ];
    let first = compose_effect_stack(&layers);
    let second = compose_effect_stack(&layers);
    assert_eq!(first, second);
}

#[test]
fn zero_weight_layers_have_no_effect() {
    let layers = [CameraEffectLayer::weighted(Vec3::ONE, Vec3::ONE, 1.0, 0.0)];
    let stack = compose_effect_stack(&layers);
    assert_eq!(stack.translation, Vec3::ZERO);
    assert_eq!(stack.rotation, Vec3::ZERO);
    assert_eq!(stack.fov_delta, 0.0);
}

#[test]
fn weighted_layers_scale_correctly() {
    let layers = [CameraEffectLayer::weighted(Vec3::X, Vec3::Y, 1.0, 0.5)];
    let stack = compose_effect_stack(&layers);
    assert_eq!(stack.translation, Vec3::new(0.5, 0.0, 0.0));
    assert_eq!(stack.rotation, Vec3::new(0.0, 0.5, 0.0));
    assert_eq!(stack.fov_delta, 0.5);
}

#[test]
fn disabling_one_effect_leaves_others_intact() {
    let mut disabled = CameraEffectLayer::weighted(Vec3::ONE, Vec3::ONE, 1.0, 1.0);
    disabled.enabled = false;
    let active = CameraEffectLayer::weighted(Vec3::X, Vec3::Z, 0.25, 1.0);
    let stack = compose_effect_stack(&[disabled, active]);
    assert_eq!(stack.translation, Vec3::X);
    assert_eq!(stack.rotation, Vec3::Z);
    assert_eq!(stack.fov_delta, 0.25);
}

#[test]
fn stack_builder_accumulates_weighted_layers() {
    let stack = CameraEffectStack::default()
        .with_layer(CameraEffectLayer::weighted(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::ZERO,
            0.0,
            1.0,
        ))
        .with_layer(CameraEffectLayer::weighted(
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::ZERO,
            0.5,
            0.5,
        ));

    assert_eq!(stack.translation, Vec3::new(1.0, 1.0, 0.0));
    assert!((stack.fov_delta - 0.25).abs() < 0.0001);
}
