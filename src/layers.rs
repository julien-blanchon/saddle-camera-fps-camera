use bevy::prelude::*;

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct CameraEffectLayer {
    pub translation: Vec3,
    pub rotation: Vec3,
    pub fov_delta: f32,
    pub weight: f32,
    pub enabled: bool,
}

impl CameraEffectLayer {
    pub fn weighted(translation: Vec3, rotation: Vec3, fov_delta: f32, weight: f32) -> Self {
        Self {
            translation,
            rotation,
            fov_delta,
            weight,
            enabled: true,
        }
    }
}

impl Default for CameraEffectLayer {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Vec3::ZERO,
            fov_delta: 0.0,
            weight: 1.0,
            enabled: true,
        }
    }
}

#[derive(Reflect, Debug, Clone, Default, PartialEq)]
pub struct CameraEffectStack {
    pub translation: Vec3,
    pub rotation: Vec3,
    pub fov_delta: f32,
}

pub fn compose_effect_stack(layers: &[CameraEffectLayer]) -> CameraEffectStack {
    let mut translation = Vec3::ZERO;
    let mut rotation = Vec3::ZERO;
    let mut fov_delta = 0.0;

    for layer in layers {
        if !layer.enabled || layer.weight <= 0.0 {
            continue;
        }

        translation += layer.translation * layer.weight;
        rotation += layer.rotation * layer.weight;
        fov_delta += layer.fov_delta * layer.weight;
    }

    CameraEffectStack {
        translation,
        rotation,
        fov_delta,
    }
}

#[cfg(test)]
#[path = "layers_tests.rs"]
mod tests;
