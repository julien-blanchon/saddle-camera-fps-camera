use bevy::prelude::*;

use crate::{
    input::{apply_look, resolve_look_delta, wrap_angle},
    AimConfig, AnalogLookConfig, DecayConfig, FpsCameraConfig, FreeLookConfig, LookConfig,
    MovementConfig,
};

fn config() -> FpsCameraConfig {
    FpsCameraConfig {
        look: LookConfig {
            sensitivity: Vec2::new(0.01, 0.02),
            invert_x: false,
            invert_y: false,
            smoothing: DecayConfig {
                decay_rate: 0.0,
                snap_threshold: 0.0,
            },
            pitch_min: -0.5,
            pitch_max: 0.5,
            analog: AnalogLookConfig {
                enabled: true,
                max_radians_per_second: Vec2::new(2.0, 1.0),
                deadzone: 0.2,
                outer_deadzone: 0.0,
                exponent: 1.0,
            },
        },
        aim: AimConfig::default(),
        free_look: FreeLookConfig::default(),
        movement: MovementConfig::default(),
        ..default()
    }
}

#[test]
fn pitch_clamps_at_configured_limits() {
    let (yaw, pitch) = apply_look(0.0, 0.0, Vec2::new(0.2, 10.0), -0.5, 0.5);
    assert_eq!(yaw, 0.2);
    assert_eq!(pitch, 0.5);
}

#[test]
fn yaw_remains_numerically_stable_across_large_rotation() {
    let wrapped = wrap_angle(4000.0);
    assert!(wrapped.abs() <= std::f32::consts::PI);
}

#[test]
fn zero_delta_yields_no_change() {
    let config = config();
    let intent = crate::FpsCameraIntent::default();
    let (delta, _) = resolve_look_delta(&intent, &config, Vec2::ZERO, 0.0, 0.016);
    assert_eq!(delta, Vec2::ZERO);
}

#[test]
fn sensitivity_scales_mouse_input() {
    let config = config();
    let intent = crate::FpsCameraIntent {
        look_delta: Vec2::new(5.0, -3.0),
        ..default()
    };
    let (delta, _) = resolve_look_delta(&intent, &config, Vec2::ZERO, 0.0, 0.016);
    assert!(delta.abs_diff_eq(Vec2::new(-0.05, 0.06), 1e-6));
}

#[test]
fn invert_flags_flip_axes() {
    let mut config = config();
    config.look.invert_x = true;
    config.look.invert_y = true;
    let intent = crate::FpsCameraIntent {
        look_delta: Vec2::new(2.0, 1.0),
        ..default()
    };
    let (delta, _) = resolve_look_delta(&intent, &config, Vec2::ZERO, 0.0, 0.016);
    assert_eq!(delta, Vec2::new(0.02, 0.02));
}

#[test]
fn raw_mouse_look_is_not_dt_scaled() {
    let config = config();
    let intent = crate::FpsCameraIntent {
        look_delta: Vec2::new(4.0, 2.0),
        ..default()
    };
    let (small_dt, _) = resolve_look_delta(&intent, &config, Vec2::ZERO, 0.0, 0.008);
    let (large_dt, _) = resolve_look_delta(&intent, &config, Vec2::ZERO, 0.0, 0.032);
    assert_eq!(small_dt, large_dt);
}

#[test]
fn smoothing_behaves_predictably() {
    let mut config = config();
    config.look.smoothing = DecayConfig::new(8.0);

    let intent = crate::FpsCameraIntent {
        look_delta: Vec2::new(5.0, 0.0),
        ..default()
    };
    let (first, state) = resolve_look_delta(&intent, &config, Vec2::ZERO, 0.0, 0.016);
    let (second, _) = resolve_look_delta(&intent, &config, state, 0.0, 0.016);

    assert!(first.x.abs() < 0.05);
    assert!(second.x.abs() >= first.x.abs());
}
