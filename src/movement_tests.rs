use bevy::prelude::*;

use crate::{
    FpsCameraConfig,
    movement::{
        desired_planar_velocity, integrate_planar_velocity, jump_velocity, movement_basis,
        target_speed,
    },
};

#[test]
fn movement_resolves_relative_to_camera_yaw() {
    let velocity = desired_planar_velocity(Vec2::Y, std::f32::consts::FRAC_PI_2, 3.0);
    assert!(velocity.x.abs() > 2.9);
    assert!(velocity.z.abs() < 0.01);
}

#[test]
fn diagonal_move_is_normalized() {
    let velocity = desired_planar_velocity(Vec2::new(1.0, 1.0), 0.0, 6.0);
    assert!((velocity.length() - 6.0).abs() < 0.000_1);
}

#[test]
fn sprint_multiplier_works() {
    let config = FpsCameraConfig::default();
    let base = target_speed(&config, 0.0, 0.0);
    let sprint = target_speed(&config, 1.0, 0.0);
    assert!(sprint > base);
}

#[test]
fn crouch_speed_multiplier_works() {
    let config = FpsCameraConfig::default();
    let standing = target_speed(&config, 0.0, 0.0);
    let crouched = target_speed(&config, 0.0, 1.0);
    assert!(crouched < standing);
}

#[test]
fn jump_parameters_derive_expected_launch_velocity() {
    let velocity = jump_velocity(20.0, 1.25);
    assert!((velocity - (50.0f32).sqrt()).abs() < 0.000_1);
}

#[test]
fn grounded_and_airborne_control_differ() {
    let desired = Vec3::new(5.0, 0.0, 0.0);
    let grounded =
        integrate_planar_velocity(Vec3::ZERO, desired, true, 30.0, 30.0, 10.0, 3.0, 0.016);
    let airborne =
        integrate_planar_velocity(Vec3::ZERO, desired, false, 30.0, 30.0, 10.0, 3.0, 0.016);
    assert!(grounded.x > airborne.x);
}

#[test]
fn crouch_eye_height_transitions_smoothly() {
    let config = FpsCameraConfig::default();
    let height = config
        .movement
        .eye_height
        .lerp(config.crouch.eye_height, 0.5);
    assert!(height < config.movement.eye_height);
    assert!(height > config.crouch.eye_height);
}

#[test]
fn movement_basis_stays_orthogonal() {
    let (forward, right) = movement_basis(1.2);
    assert!(forward.dot(right).abs() < 0.000_1);
}
