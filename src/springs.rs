use bevy::{math::StableInterpolate, prelude::*};

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct DecayConfig {
    pub decay_rate: f32,
    pub snap_threshold: f32,
}

impl DecayConfig {
    pub const fn new(decay_rate: f32) -> Self {
        Self {
            decay_rate,
            snap_threshold: 0.000_1,
        }
    }
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self::new(8.0)
    }
}

pub fn decay_scalar(current: f32, target: f32, config: DecayConfig, dt: f32) -> f32 {
    if config.decay_rate <= 0.0 {
        return target;
    }

    let mut value = current;
    value.smooth_nudge(&target, config.decay_rate, dt);
    if (value - target).abs() <= config.snap_threshold {
        target
    } else {
        value
    }
}

pub fn decay_vec2(current: Vec2, target: Vec2, config: DecayConfig, dt: f32) -> Vec2 {
    if config.decay_rate <= 0.0 {
        return target;
    }

    let mut value = current;
    value.smooth_nudge(&target, config.decay_rate, dt);
    if value.distance(target) <= config.snap_threshold {
        target
    } else {
        value
    }
}

pub fn decay_vec3(current: Vec3, target: Vec3, config: DecayConfig, dt: f32) -> Vec3 {
    if config.decay_rate <= 0.0 {
        return target;
    }

    let mut value = current;
    value.smooth_nudge(&target, config.decay_rate, dt);
    if value.distance(target) <= config.snap_threshold {
        target
    } else {
        value
    }
}

#[cfg(test)]
#[path = "springs_tests.rs"]
mod tests;
