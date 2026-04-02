use bevy::prelude::*;

use crate::{
    FpsCameraConfig,
    effects::{
        advance_distance_driven_bob, bob_offset, dynamic_fov_target, footstep_crossed,
        landing_offset, shake_intensity, trauma_decay, trauma_shake,
    },
    springs::decay_vec2,
};

#[test]
fn head_bob_channels_behave_as_expected() {
    let config = FpsCameraConfig::default();
    let offset = bob_offset(&config, std::f32::consts::FRAC_PI_2, 1.0);
    assert!(offset.x.abs() > 0.0);
    assert!(offset.y.abs() <= config.head_bob.amplitude.y);
}

#[test]
fn bob_does_not_advance_while_idle_if_distance_driven() {
    let phase = advance_distance_driven_bob(1.3, 0.0, 1.5);
    assert_eq!(phase, 1.3);
}

#[test]
fn dynamic_fov_tracks_speed_ratio() {
    let config = FpsCameraConfig::default();
    let slow = dynamic_fov_target(&config, 0.1, 0.0, 0.0);
    let fast = dynamic_fov_target(&config, 1.0, 1.0, 0.0);
    assert!(fast > slow);
}

#[test]
fn disabled_ads_does_not_change_fov_target() {
    let mut config = FpsCameraConfig::default();
    config.aim.enabled = false;
    let base = dynamic_fov_target(&config, 0.4, 0.2, 0.0);
    let aimed = dynamic_fov_target(&config, 0.4, 0.2, 1.0);
    assert_eq!(base, aimed);
}

#[test]
fn landing_impact_scales_with_severity() {
    let config = FpsCameraConfig::default();
    let mild = landing_offset(&config, 0.2).0.y.abs();
    let heavy = landing_offset(&config, 0.9).0.y.abs();
    assert!(heavy > mild);
}

#[test]
fn disabled_landing_effect_returns_zero_offset() {
    let mut config = FpsCameraConfig::default();
    config.landing.enabled = false;
    assert_eq!(landing_offset(&config, 1.0), (Vec3::ZERO, Vec3::ZERO));
}

#[test]
fn trauma_decays_over_time() {
    assert!(trauma_decay(1.0, 2.0, 0.1) < 1.0);
}

#[test]
fn shake_intensity_mapping_is_correct() {
    assert_eq!(shake_intensity(0.0), 0.0);
    assert!(shake_intensity(0.5) < shake_intensity(1.0));
}

#[test]
fn footstep_events_occur_at_expected_gait_phases() {
    assert!(footstep_crossed(2.9, 3.3, std::f32::consts::PI));
    assert!(!footstep_crossed(1.0, 2.0, std::f32::consts::PI));
}

#[test]
fn recoil_recovers_toward_neutral() {
    let recovered = decay_vec2(
        Vec2::new(0.3, -0.2),
        Vec2::ZERO,
        crate::DecayConfig::new(12.0),
        0.016,
    );
    assert!(recovered.length() < Vec2::new(0.3, -0.2).length());
}

#[test]
fn trauma_shake_is_zero_without_trauma() {
    let config = FpsCameraConfig::default();
    let (translation, rotation) = trauma_shake(&config, 0.0, 0.5);
    assert_eq!(translation, Vec3::ZERO);
    assert_eq!(rotation, Vec3::ZERO);
}
