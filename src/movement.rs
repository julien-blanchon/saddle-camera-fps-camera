use bevy::prelude::*;

use crate::{
    FpsCameraConfig, FpsCameraExternalMotion, FpsCameraIntent, FpsCameraRuntime,
    components::{FpsCamera, FpsCameraInternalState},
    springs::decay_scalar,
};

pub(crate) fn move_toward(current: f32, target: f32, max_delta: f32) -> f32 {
    let delta = target - current;
    if delta.abs() <= max_delta {
        target
    } else {
        current + delta.signum() * max_delta
    }
}

pub(crate) fn movement_basis(yaw: f32) -> (Vec3, Vec3) {
    let rotation = Quat::from_rotation_y(yaw);
    let forward = rotation * Vec3::NEG_Z;
    let right = rotation * Vec3::X;
    (forward.normalize_or_zero(), right.normalize_or_zero())
}

pub(crate) fn desired_planar_velocity(move_axis: Vec2, yaw: f32, speed: f32) -> Vec3 {
    let move_axis = move_axis.clamp_length_max(1.0);
    if move_axis == Vec2::ZERO {
        return Vec3::ZERO;
    }

    let (forward, right) = movement_basis(yaw);
    let desired = right * move_axis.x + forward * move_axis.y;
    desired.normalize_or_zero() * speed
}

pub(crate) fn jump_velocity(gravity: f32, height: f32) -> f32 {
    (2.0 * gravity * height).sqrt()
}

pub(crate) fn integrate_planar_velocity(
    current: Vec3,
    desired: Vec3,
    grounded: bool,
    acceleration: f32,
    deceleration: f32,
    air_acceleration: f32,
    max_air_speed: f32,
    dt: f32,
) -> Vec3 {
    let mut next = current;
    let current_planar = Vec2::new(current.x, current.z);
    let desired_planar = Vec2::new(desired.x, desired.z);

    let accel = if grounded {
        if desired_planar.length_squared() > 0.0 {
            acceleration
        } else {
            deceleration
        }
    } else {
        air_acceleration
    };

    next.x = move_toward(current.x, desired.x, accel * dt);
    next.z = move_toward(current.z, desired.z, accel * dt);

    if !grounded {
        let planar = Vec2::new(next.x, next.z);
        let limited = planar.clamp_length_max(max_air_speed.max(current_planar.length()));
        next.x = limited.x;
        next.z = limited.y;
    }

    next
}

pub(crate) fn target_speed(config: &FpsCameraConfig, sprint_alpha: f32, crouch_alpha: f32) -> f32 {
    let sprint_speed = config.movement.walk_speed * config.movement.sprint_multiplier;
    let base = config
        .movement
        .walk_speed
        .lerp(sprint_speed, sprint_alpha.clamp(0.0, 1.0));
    base * (1.0 + crouch_alpha * (config.crouch.speed_multiplier - 1.0))
}

pub(crate) fn update_locomotion(
    time: Res<Time>,
    mut query: Query<
        (
            &FpsCameraConfig,
            &mut FpsCameraIntent,
            Option<&FpsCameraExternalMotion>,
            &mut FpsCameraRuntime,
            &mut FpsCameraInternalState,
        ),
        With<FpsCamera>,
    >,
) {
    let dt = time.delta_secs();

    for (config, mut intent, external_motion, mut runtime, internal) in &mut query {
        let previous_vertical_velocity = runtime.velocity.y;
        let external_motion = external_motion.filter(|motion| motion.enabled);
        let sprint_target = if intent.sprint_pressed { 1.0 } else { 0.0 };
        runtime.sprint_alpha = decay_scalar(
            runtime.sprint_alpha,
            sprint_target,
            config.movement.sprint_transition,
            dt,
        );

        if config.aim.enabled {
            let aim_target = if intent.aim_pressed { 1.0 } else { 0.0 };
            runtime.aim_alpha =
                decay_scalar(runtime.aim_alpha, aim_target, config.aim.transition, dt);
        } else {
            runtime.aim_alpha = 0.0;
        }

        runtime.lean_alpha = if config.lean.enabled {
            intent.lean.clamp(-1.0, 1.0)
        } else {
            0.0
        };

        let crouch_target = if !config.crouch.enabled {
            0.0
        } else if let Some(external) = external_motion {
            external
                .crouch_alpha
                .unwrap_or(if intent.crouch_pressed { 1.0 } else { 0.0 })
        } else if intent.crouch_pressed {
            1.0
        } else {
            0.0
        };
        runtime.crouch_alpha = decay_scalar(
            runtime.crouch_alpha,
            crouch_target,
            config.crouch.transition,
            dt,
        );
        if !config.crouch.enabled {
            runtime.crouch_alpha = 0.0;
        }

        if let Some(external) = external_motion {
            runtime.position = external.position;
            runtime.velocity = external.velocity;
            runtime.grounded = external.grounded;
            runtime.recent_landing_impulse = external.landing_impulse;
            if let Some(sprint_alpha) = external.sprint_alpha {
                runtime.sprint_alpha = sprint_alpha.clamp(0.0, 1.0);
            }
        } else {
            let crouch_alpha = if config.crouch.enabled {
                runtime.crouch_alpha
            } else {
                0.0
            };
            let speed = target_speed(config, runtime.sprint_alpha, crouch_alpha);
            let desired = desired_planar_velocity(intent.move_axis, runtime.yaw, speed);
            let desired = if runtime.grounded {
                desired
            } else {
                desired * config.movement.air_control
            };

            let mut velocity = integrate_planar_velocity(
                runtime.velocity,
                desired,
                runtime.grounded,
                config.movement.acceleration,
                config.movement.deceleration,
                config.movement.air_acceleration,
                config.movement.max_air_speed,
                dt,
            );

            let was_grounded = runtime.grounded;
            if runtime.grounded && intent.jump_pressed && config.jump.enabled {
                velocity.y = jump_velocity(config.movement.gravity, config.jump.height);
                runtime.grounded = false;
            } else if !runtime.grounded {
                let gravity = config.movement.gravity
                    * if velocity.y < 0.0 {
                        config.jump.fall_multiplier
                    } else {
                        1.0
                    };
                velocity.y = (velocity.y - gravity * dt).max(-config.movement.terminal_velocity);
            } else {
                velocity.y = 0.0;
            }

            runtime.position += velocity * dt;
            if runtime.position.y <= internal.base_ground_height {
                runtime.position.y = internal.base_ground_height;
                runtime.grounded = true;
                if !was_grounded && velocity.y < -config.jump.landing_velocity_threshold {
                    runtime.recent_landing_impulse = (-velocity.y
                        / config.movement.terminal_velocity.max(1.0))
                    .clamp(0.0, config.landing.max_impulse);
                }
                velocity.y = 0.0;
            } else {
                runtime.grounded = false;
            }

            runtime.velocity = velocity;
        }

        runtime.eye_height = if config.crouch.enabled {
            config.movement.eye_height.lerp(
                config.crouch.eye_height,
                runtime.crouch_alpha.clamp(0.0, 1.0),
            )
        } else {
            config.movement.eye_height
        };
        runtime.speed = Vec2::new(runtime.velocity.x, runtime.velocity.z).length();
        runtime.speed_ratio = (runtime.speed
            / (config.movement.walk_speed * config.movement.sprint_multiplier).max(0.001))
        .clamp(0.0, 1.25);
        runtime.fall_speed = if runtime.grounded && previous_vertical_velocity < 0.0 {
            -previous_vertical_velocity
        } else {
            (-runtime.velocity.y).max(0.0)
        };

        intent.jump_pressed = false;
    }
}

#[cfg(test)]
#[path = "movement_tests.rs"]
mod tests;
