use bevy::prelude::*;

use crate::{decay_scalar, decay_vec3, DecayConfig};

#[test]
fn convergence_to_target() {
    let mut value = 0.0;
    for _ in 0..20 {
        value = decay_scalar(value, 1.0, DecayConfig::new(12.0), 0.016);
    }
    assert!(value > 0.9);
}

#[test]
fn no_pathological_overshoot_for_critically_damped_style_decay() {
    let mut value = 0.0;
    for _ in 0..10 {
        value = decay_scalar(value, 1.0, DecayConfig::new(14.0), 0.016);
        assert!(value <= 1.0);
    }
}

#[test]
fn behavior_is_acceptably_frame_rate_independent() {
    let mut at_sixty = 0.0;
    for _ in 0..60 {
        at_sixty = decay_scalar(at_sixty, 1.0, DecayConfig::new(8.0), 1.0 / 60.0);
    }
    let mut at_one_twenty = 0.0;
    for _ in 0..120 {
        at_one_twenty = decay_scalar(at_one_twenty, 1.0, DecayConfig::new(8.0), 1.0 / 120.0);
    }
    assert!((at_sixty - at_one_twenty).abs() < 0.02);
}

#[test]
fn near_instant_response_behaves_as_snap() {
    let snapped = decay_vec3(Vec3::ZERO, Vec3::ONE, DecayConfig::new(10_000.0), 0.016);
    assert!(snapped.distance(Vec3::ONE) < 0.001);
}
