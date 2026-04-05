use std::f32::consts::{PI, TAU};

use bevy::{ecs::message::MessageReader, prelude::*};

use crate::{
    CameraEffectLayer, FpsCameraConfig, FpsCameraExternalEffects, FpsCameraRuntime, LandedEvent,
    components::{FpsCamera, FpsCameraCollisionFeedback, FpsCameraInternalState},
    compose_effect_stack, decay_scalar, decay_vec2, decay_vec3,
    messages::{CameraRecoilRequest, CameraShakeRequest, FootstepEvent},
};

fn duration_to_decay_rate(duration_secs: f32) -> Option<f32> {
    (duration_secs > 0.0).then(|| 6.0 / duration_secs.max(0.001))
}

pub(crate) fn apply_shake_requests(
    mut requests: MessageReader<CameraShakeRequest>,
    mut query: Query<
        (
            &FpsCameraConfig,
            &mut FpsCameraRuntime,
            &mut FpsCameraInternalState,
        ),
        With<FpsCamera>,
    >,
) {
    for request in requests.read() {
        if let Ok((config, mut runtime, mut internal)) = query.get_mut(request.entity) {
            runtime.trauma = (runtime.trauma + request.trauma).clamp(0.0, config.shake.max_trauma);
            internal.shake_decay_override =
                request.duration_override.and_then(duration_to_decay_rate);
        }
    }
}

pub(crate) fn apply_recoil_requests(
    mut requests: MessageReader<CameraRecoilRequest>,
    mut query: Query<(&FpsCameraConfig, &mut FpsCameraInternalState), With<FpsCamera>>,
) {
    for request in requests.read() {
        if let Ok((config, mut internal)) = query.get_mut(request.entity) {
            if !config.recoil.enabled {
                internal.recoil = Vec2::ZERO;
                continue;
            }
            internal.recoil_recovery_override = request
                .duration_override
                .and_then(|duration| duration_to_decay_rate(duration).map(crate::DecayConfig::new));
            internal.recoil.x = (internal.recoil.x + request.pitch)
                .clamp(-config.recoil.max_pitch, config.recoil.max_pitch);
            internal.recoil.y = (internal.recoil.y + request.yaw)
                .clamp(-config.recoil.max_yaw, config.recoil.max_yaw);
        }
    }
}

pub(crate) fn bob_offset(config: &FpsCameraConfig, phase: f32, gait_scale: f32) -> Vec3 {
    if !config.head_bob.enabled || gait_scale <= 0.0 {
        return Vec3::ZERO;
    }

    let amplitude = config.head_bob.amplitude * gait_scale * config.comfort.bob_weight;
    Vec3::new(
        phase.sin() * amplitude.x,
        (phase * 2.0).sin() * amplitude.y,
        (phase + PI * 0.5).sin() * amplitude.z,
    )
}

pub(crate) fn idle_sway(config: &FpsCameraConfig, time_secs: f32) -> (Vec3, Vec2) {
    let sway = config.head_bob.idle_sway_translation;
    let rot = config.head_bob.idle_sway_rotation;
    let t = time_secs * config.head_bob.idle_sway_frequency * TAU;
    (
        Vec3::new(
            t.sin() * sway.x,
            (t * 0.5).sin() * sway.y,
            (t * 0.8).cos() * sway.z,
        ) * config.comfort.bob_weight,
        Vec2::new((t * 0.6).sin() * rot.x, (t * 0.8).cos() * rot.y) * config.comfort.bob_weight,
    )
}

pub(crate) fn gait_scale(
    config: &FpsCameraConfig,
    speed_ratio: f32,
    sprint_alpha: f32,
    crouch_alpha: f32,
) -> f32 {
    let mut scale = speed_ratio.clamp(0.0, 1.2);
    scale *= 1.0 + sprint_alpha * (config.head_bob.sprint_multiplier - 1.0);
    scale *= 1.0 + crouch_alpha * (config.head_bob.crouch_multiplier - 1.0);
    scale
}

pub(crate) fn advance_distance_driven_bob(
    current_phase: f32,
    distance: f32,
    stride_length: f32,
) -> f32 {
    if distance <= 0.0 {
        current_phase
    } else {
        current_phase + distance / stride_length.max(0.001) * TAU
    }
}

pub(crate) fn dynamic_fov_target(
    config: &FpsCameraConfig,
    speed_ratio: f32,
    sprint_alpha: f32,
    aim_alpha: f32,
) -> f32 {
    let motion_boost = config.comfort.dynamic_fov_weight
        * (config.fov.speed_boost * speed_ratio.clamp(0.0, 1.0)
            + config.fov.sprint_boost * sprint_alpha.clamp(0.0, 1.0));
    let aim_multiplier = if config.aim.enabled {
        1.0 + aim_alpha * (config.aim.fov_multiplier - 1.0)
    } else {
        1.0
    };
    (config.fov.base_fov + motion_boost) * aim_multiplier
}

pub(crate) fn landing_offset(config: &FpsCameraConfig, landing_amount: f32) -> (Vec3, Vec3) {
    if !config.landing.enabled {
        return (Vec3::ZERO, Vec3::ZERO);
    }
    let weight = config.comfort.landing_weight * landing_amount.clamp(0.0, 1.0);
    (
        Vec3::new(0.0, -config.landing.translation_amount * weight, 0.0),
        Vec3::new(-config.landing.pitch_amount * weight, 0.0, 0.0),
    )
}

pub(crate) fn trauma_decay(current: f32, decay_rate: f32, dt: f32) -> f32 {
    (current - decay_rate * dt).max(0.0)
}

pub(crate) fn shake_intensity(trauma: f32) -> f32 {
    trauma.clamp(0.0, 1.0).powi(2)
}

pub(crate) fn trauma_shake(config: &FpsCameraConfig, trauma: f32, time_secs: f32) -> (Vec3, Vec3) {
    let intensity = shake_intensity(trauma) * config.comfort.shake_weight;
    let t = time_secs * config.shake.frequency + config.shake.seed;
    let (translation_frequency, rotation_frequency, phase_offset) = match config.shake.noise_profile
    {
        crate::ShakeNoiseProfile::Standard => (
            Vec3::new(1.07, 1.31, 0.89),
            Vec3::new(0.81, 1.17, 1.43),
            Vec3::new(0.0, 0.35, 0.72),
        ),
        crate::ShakeNoiseProfile::Handheld => (
            Vec3::new(0.72, 0.91, 0.56),
            Vec3::new(0.66, 0.84, 1.02),
            Vec3::new(0.15, 0.48, 0.91),
        ),
        crate::ShakeNoiseProfile::Explosion => (
            Vec3::new(1.64, 1.48, 1.22),
            Vec3::new(1.21, 1.42, 1.67),
            Vec3::new(0.45, 0.83, 1.14),
        ),
        crate::ShakeNoiseProfile::Rumble => (
            Vec3::new(0.42, 0.51, 0.37),
            Vec3::new(0.34, 0.48, 0.61),
            Vec3::new(0.09, 0.31, 0.57),
        ),
    };

    let translation = Vec3::new(
        (t * translation_frequency.x + phase_offset.x).sin(),
        (t * translation_frequency.y + phase_offset.y).cos(),
        (t * translation_frequency.z + phase_offset.z).sin(),
    ) * config.shake.translation_amplitude
        * intensity;
    let rotation = Vec3::new(
        (t * rotation_frequency.x + phase_offset.y).sin(),
        (t * rotation_frequency.y + phase_offset.z).cos(),
        (t * rotation_frequency.z + phase_offset.x).sin(),
    ) * config.shake.rotation_amplitude
        * intensity;

    (translation, rotation)
}

fn clamp_vec3_magnitude(value: Vec3, limit: Vec3) -> Vec3 {
    Vec3::new(
        value.x.clamp(-limit.x, limit.x),
        value.y.clamp(-limit.y, limit.y),
        value.z.clamp(-limit.z, limit.z),
    )
}

fn update_viewmodel_lag(
    config: &FpsCameraConfig,
    runtime: &mut FpsCameraRuntime,
    internal: &mut FpsCameraInternalState,
    dt: f32,
) {
    if !config.viewmodel.enabled {
        internal.viewmodel_translation = Vec3::ZERO;
        internal.viewmodel_rotation = Vec3::ZERO;
        runtime.viewmodel_translation = Vec3::ZERO;
        runtime.viewmodel_rotation = Vec3::ZERO;
        return;
    }

    let local_velocity = Quat::from_rotation_y(-runtime.yaw) * runtime.velocity;
    let look = internal.recent_look_delta;
    let target_translation = clamp_vec3_magnitude(
        Vec3::new(
            -look.x * config.viewmodel.translation_scale.x
                + local_velocity.x * config.viewmodel.movement_scale.x,
            look.y * config.viewmodel.translation_scale.y
                + runtime.crouch_alpha * config.viewmodel.movement_scale.y,
            -look.y * config.viewmodel.translation_scale.z
                - local_velocity.z * config.viewmodel.movement_scale.z,
        ),
        config.viewmodel.max_translation,
    );
    let target_rotation = clamp_vec3_magnitude(
        Vec3::new(
            look.y * config.viewmodel.rotation_scale.x,
            -look.x * config.viewmodel.rotation_scale.y,
            -look.x * config.viewmodel.rotation_scale.z,
        ),
        config.viewmodel.max_rotation,
    );

    internal.viewmodel_translation = decay_vec3(
        internal.viewmodel_translation,
        target_translation,
        config.viewmodel.response,
        dt,
    );
    internal.viewmodel_rotation = decay_vec3(
        internal.viewmodel_rotation,
        target_rotation,
        config.viewmodel.response,
        dt,
    );
    runtime.viewmodel_translation = internal.viewmodel_translation;
    runtime.viewmodel_rotation = internal.viewmodel_rotation;
}

pub(crate) fn footstep_crossed(previous: f32, current: f32, threshold: f32) -> bool {
    previous < threshold && current >= threshold
}

pub(crate) fn update_camera_state(
    time: Res<Time>,
    mut footstep_writer: MessageWriter<FootstepEvent>,
    mut landed_writer: MessageWriter<LandedEvent>,
    mut query: Query<
        (
            Entity,
            &FpsCameraConfig,
            Option<&FpsCameraExternalEffects>,
            &mut FpsCameraRuntime,
            &mut FpsCameraInternalState,
        ),
        With<FpsCamera>,
    >,
) {
    let dt = time.delta_secs();

    for (entity, config, _external_effects, mut runtime, mut internal) in &mut query {
        if runtime.free_look_offset != Vec2::ZERO {
            runtime.free_look_offset = decay_vec2(
                runtime.free_look_offset,
                Vec2::ZERO,
                config.free_look.recenter,
                dt,
            );
        }

        let shake_decay = internal
            .shake_decay_override
            .unwrap_or(config.shake.decay_rate);
        runtime.trauma = trauma_decay(runtime.trauma, shake_decay, dt);
        if runtime.trauma <= f32::EPSILON {
            internal.shake_decay_override = None;
        }
        if config.recoil.enabled {
            let recoil_decay = internal
                .recoil_recovery_override
                .unwrap_or(config.recoil.recovery);
            internal.recoil = decay_vec2(internal.recoil, Vec2::ZERO, recoil_decay, dt);
            if internal.recoil.length_squared() <= 0.000_001 {
                internal.recoil_recovery_override = None;
            }
        } else {
            internal.recoil = Vec2::ZERO;
        }
        internal.landing_amount =
            decay_scalar(internal.landing_amount, 0.0, config.landing.response, dt);
        internal.shake_time += dt;

        let tilt_target = if config.tilt.enabled {
            let local_velocity = Quat::from_rotation_y(-runtime.yaw) * runtime.velocity;
            (-local_velocity.x
                / (config.movement.walk_speed * config.movement.sprint_multiplier).max(0.001))
            .clamp(-1.0, 1.0)
                * config.tilt.max_roll
                * config.comfort.roll_weight
        } else {
            0.0
        };
        internal.tilt_roll =
            decay_scalar(internal.tilt_roll, tilt_target, config.tilt.response, dt);
        runtime.tilt_roll = internal.tilt_roll;

        let lean_target = runtime.lean_alpha;
        internal.lean_alpha = if config.lean.enabled {
            decay_scalar(internal.lean_alpha, lean_target, config.lean.response, dt)
        } else {
            0.0
        };
        runtime.lean_alpha = internal.lean_alpha;

        if runtime.grounded && !internal.previous_grounded {
            let landing_impulse = runtime.recent_landing_impulse.max(
                (runtime.fall_speed / config.movement.terminal_velocity.max(1.0))
                    .clamp(0.0, config.landing.max_impulse),
            );
            if landing_impulse > 0.0 {
                internal.landing_amount = landing_impulse;
                landed_writer.write(LandedEvent {
                    entity,
                    impact_speed: runtime.fall_speed,
                    landing_impulse,
                });
            }
        }
        internal.previous_grounded = runtime.grounded;

        let planar_position = runtime.position.xz();
        let planar_distance = planar_position.distance(internal.last_planar_position);
        internal.last_planar_position = planar_position;

        if runtime.grounded && runtime.speed > 0.05 {
            let phase_scale = gait_scale(
                config,
                runtime.speed_ratio,
                runtime.sprint_alpha,
                runtime.crouch_alpha,
            );
            internal.cumulative_bob_phase = advance_distance_driven_bob(
                internal.cumulative_bob_phase,
                planar_distance,
                config.head_bob.stride_length / phase_scale.max(0.25),
            );

            while footstep_crossed(
                internal.next_footstep_phase - PI,
                internal.cumulative_bob_phase,
                internal.next_footstep_phase,
            ) {
                footstep_writer.write(FootstepEvent {
                    entity,
                    phase: internal.next_footstep_phase.rem_euclid(TAU),
                    speed: runtime.speed,
                });
                internal.next_footstep_phase += PI;
            }
        }

        runtime.bob_phase = internal.cumulative_bob_phase.rem_euclid(TAU);
        runtime.recoil_offset = if config.recoil.enabled {
            internal.recoil
        } else {
            Vec2::ZERO
        };
        runtime.recent_landing_impulse = internal.landing_amount;
        update_viewmodel_lag(config, &mut runtime, &mut internal, dt);
    }
}

pub(crate) fn compose_effects(
    time: Res<Time>,
    mut query: Query<
        (
            &FpsCameraConfig,
            Option<&FpsCameraExternalEffects>,
            &mut FpsCameraRuntime,
            &mut FpsCameraInternalState,
        ),
        With<FpsCamera>,
    >,
) {
    let time_secs = time.elapsed_secs();
    let dt = time.delta_secs();

    for (config, external_effects, mut runtime, internal) in &mut query {
        let gait = gait_scale(
            config,
            runtime.speed_ratio,
            runtime.sprint_alpha,
            runtime.crouch_alpha,
        );
        let bob = if runtime.speed > 0.05 && runtime.grounded {
            bob_offset(config, runtime.bob_phase, gait)
        } else {
            Vec3::ZERO
        };
        let (idle_translation, idle_rotation) = if runtime.speed <= 0.05 {
            idle_sway(config, time_secs)
        } else {
            (Vec3::ZERO, Vec2::ZERO)
        };

        let (landing_translation, landing_rotation) =
            landing_offset(config, internal.landing_amount);
        let (shake_translation, shake_rotation) =
            trauma_shake(config, runtime.trauma, internal.shake_time);

        let (lean_translation, lean_rotation) = if config.lean.enabled {
            (
                Vec3::new(internal.lean_alpha * config.lean.lateral_offset, 0.0, 0.0),
                Vec3::new(
                    0.0,
                    0.0,
                    internal.lean_alpha * config.lean.max_angle * config.comfort.roll_weight,
                ),
            )
        } else {
            (Vec3::ZERO, Vec3::ZERO)
        };
        let tilt_rotation = Vec3::new(0.0, 0.0, internal.tilt_roll);
        let recoil_rotation = if config.recoil.enabled {
            Vec3::new(internal.recoil.x, internal.recoil.y, 0.0)
        } else {
            Vec3::ZERO
        };
        let free_look_rotation =
            Vec3::new(runtime.free_look_offset.y, runtime.free_look_offset.x, 0.0);

        let mut layers = vec![
            CameraEffectLayer::weighted(
                bob + idle_translation,
                Vec3::new(idle_rotation.y, 0.0, idle_rotation.x),
                0.0,
                1.0,
            ),
            CameraEffectLayer::weighted(landing_translation, landing_rotation, 0.0, 1.0),
            CameraEffectLayer::weighted(shake_translation, shake_rotation, 0.0, 1.0),
            CameraEffectLayer::weighted(lean_translation, lean_rotation, 0.0, 1.0),
            CameraEffectLayer::weighted(
                Vec3::ZERO,
                tilt_rotation + recoil_rotation + free_look_rotation,
                0.0,
                1.0,
            ),
        ];

        if let Some(external) = external_effects.filter(|effect| effect.enabled) {
            layers.push(CameraEffectLayer::weighted(
                external.translation,
                external.rotation,
                external.fov_delta,
                external.weight.max(0.0),
            ));
        }

        let stack = compose_effect_stack(&layers);
        runtime.effect_stack = stack.clone();
        runtime.head_bob_offset = bob + idle_translation;
        runtime.landing_offset = landing_translation;
        runtime.shake_offset = shake_translation;
        runtime.lean_offset = lean_translation;
        runtime.render_translation =
            runtime.position + Vec3::Y * runtime.eye_height + stack.translation;
        runtime.render_rotation = Vec3::new(runtime.pitch, runtime.yaw, 0.0) + stack.rotation;

        let fov_target = dynamic_fov_target(
            config,
            runtime.speed_ratio,
            runtime.sprint_alpha,
            runtime.aim_alpha,
        ) + stack.fov_delta;
        runtime.visual_fov = decay_scalar(runtime.visual_fov, fov_target, config.fov.response, dt);
    }
}

pub(crate) fn sync_projection(
    mut query: Query<(&FpsCameraConfig, &FpsCameraRuntime, &mut Projection), With<FpsCamera>>,
) {
    for (_config, runtime, mut projection) in &mut query {
        if let Projection::Perspective(perspective) = projection.as_mut() {
            perspective.fov = runtime.visual_fov;
        }
    }
}

pub(crate) fn sync_transform(
    mut query: Query<
        (
            &FpsCameraConfig,
            &FpsCameraRuntime,
            Option<&FpsCameraCollisionFeedback>,
            &mut Transform,
        ),
        With<FpsCamera>,
    >,
) {
    for (config, runtime, collision, mut transform) in &mut query {
        let mut final_translation = runtime.render_translation;

        if config.collision.enabled
            && let Some(feedback) = collision.filter(|f| f.blocked)
        {
            let push = feedback.push_normal * config.collision.push_margin;
            final_translation += push;
            let _ = feedback.nearest_distance;
        }

        transform.translation = final_translation;
        transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            runtime.render_rotation.y,
            runtime.render_rotation.x,
            runtime.render_rotation.z,
        );
    }
}

#[cfg(test)]
#[path = "effects_tests.rs"]
mod tests;
