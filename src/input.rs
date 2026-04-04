use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use crate::{
    FpsCameraConfig, FpsCameraIntent, FpsCameraRuntime,
    components::{FpsCamera, FpsCameraInternalState},
    springs::decay_vec2,
};

pub(crate) fn ensure_initialized(
    mut query: Query<
        (
            &Transform,
            &FpsCameraConfig,
            &mut FpsCameraRuntime,
            &mut FpsCameraInternalState,
        ),
        With<FpsCamera>,
    >,
) {
    for (transform, config, mut runtime, mut internal) in &mut query {
        if internal.initialized {
            continue;
        }

        let logical_position = transform.translation - Vec3::Y * config.movement.eye_height;
        let (yaw, pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);

        runtime.position = logical_position;
        runtime.velocity = Vec3::ZERO;
        runtime.yaw = yaw;
        runtime.pitch = pitch.clamp(config.look.pitch_min, config.look.pitch_max);
        runtime.grounded = true;
        runtime.eye_height = config.movement.eye_height;
        runtime.visual_fov = config.fov.base_fov;
        runtime.render_translation = transform.translation;
        runtime.render_rotation = Vec3::new(runtime.pitch, runtime.yaw, 0.0);
        internal.base_ground_height = logical_position.y;
        internal.last_planar_position = logical_position.xz();
        internal.previous_grounded = true;
        internal.initialized = true;
    }
}

pub(crate) fn apply_shake_input_inversion(mut delta: Vec2, invert_x: bool, invert_y: bool) -> Vec2 {
    if invert_x {
        delta.x = -delta.x;
    }
    if invert_y {
        delta.y = -delta.y;
    }
    delta
}

pub(crate) fn wrap_angle(angle: f32) -> f32 {
    let wrapped = angle.rem_euclid(TAU);
    if wrapped > PI { wrapped - TAU } else { wrapped }
}

pub(crate) fn apply_look(
    yaw: f32,
    pitch: f32,
    delta: Vec2,
    pitch_min: f32,
    pitch_max: f32,
) -> (f32, f32) {
    let next_yaw = wrap_angle(yaw + delta.x);
    let next_pitch = (pitch + delta.y).clamp(pitch_min, pitch_max);
    (next_yaw, next_pitch)
}

pub(crate) fn analog_curve(input: Vec2, deadzone: f32, outer_deadzone: f32, exponent: f32) -> Vec2 {
    let magnitude = input.length();
    if magnitude <= deadzone {
        return Vec2::ZERO;
    }

    let usable_range = (1.0 - deadzone - outer_deadzone).max(f32::EPSILON);
    let normalized = ((magnitude - deadzone) / usable_range).clamp(0.0, 1.0);
    let curved = normalized.powf(exponent.max(0.01));
    input.normalize_or_zero() * curved
}

pub(crate) fn resolve_look_delta(
    intent: &FpsCameraIntent,
    config: &FpsCameraConfig,
    smoothing_state: Vec2,
    aim_alpha: f32,
    dt: f32,
) -> (Vec2, Vec2) {
    let aim_scale = if config.aim.enabled {
        1.0 + aim_alpha * (config.aim.sensitivity_scale - 1.0)
    } else {
        1.0
    };

    let mut mouse = Vec2::new(-intent.look_delta.x, -intent.look_delta.y);
    mouse = apply_shake_input_inversion(mouse, config.look.invert_x, config.look.invert_y);
    mouse *= config.look.sensitivity * aim_scale;

    let analog_input = Vec2::new(-intent.look_analog.x, -intent.look_analog.y);
    let analog_input =
        apply_shake_input_inversion(analog_input, config.look.invert_x, config.look.invert_y);
    let analog_curve = if config.look.analog.enabled {
        analog_curve(
            analog_input,
            config.look.analog.deadzone,
            config.look.analog.outer_deadzone,
            config.look.analog.exponent,
        )
    } else {
        Vec2::ZERO
    };
    let analog = analog_curve * config.look.analog.max_radians_per_second * dt * aim_scale;

    let target = mouse + analog;
    let smoothed = if config.look.smoothing.decay_rate > 0.0 {
        decay_vec2(smoothing_state, target, config.look.smoothing, dt)
    } else {
        target
    };

    (smoothed, smoothed)
}

pub(crate) fn apply_look_intent(
    time: Res<Time>,
    mut query: Query<
        (
            &FpsCameraConfig,
            &mut FpsCameraIntent,
            &mut FpsCameraRuntime,
            &mut FpsCameraInternalState,
        ),
        With<FpsCamera>,
    >,
) {
    let dt = time.delta_secs();
    for (config, mut intent, mut runtime, mut internal) in &mut query {
        let (delta, smoothing_state) = resolve_look_delta(
            &intent,
            config,
            internal.look_smoothing,
            runtime.aim_alpha,
            dt,
        );
        internal.look_smoothing = smoothing_state;
        internal.recent_look_delta = delta;

        if intent.free_look && config.free_look.enabled {
            runtime.free_look_offset.x = (runtime.free_look_offset.x + delta.x)
                .clamp(-config.free_look.yaw_limit, config.free_look.yaw_limit);
            runtime.free_look_offset.y = (runtime.free_look_offset.y + delta.y)
                .clamp(-config.free_look.pitch_limit, config.free_look.pitch_limit);
        } else {
            let (yaw, pitch) = apply_look(
                runtime.yaw,
                runtime.pitch,
                delta,
                config.look.pitch_min,
                config.look.pitch_max,
            );
            runtime.yaw = yaw;
            runtime.pitch = pitch;
        }

        intent.look_delta = Vec2::ZERO;
        intent.look_analog = Vec2::ZERO;
    }
}

#[cfg(test)]
#[path = "look_tests.rs"]
mod tests;
