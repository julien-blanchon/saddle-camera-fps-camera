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

impl CameraEffectStack {
    pub const fn new(translation: Vec3, rotation: Vec3, fov_delta: f32) -> Self {
        Self {
            translation,
            rotation,
            fov_delta,
        }
    }

    pub fn with_layer(mut self, layer: CameraEffectLayer) -> Self {
        self.add_layer(layer);
        self
    }

    pub fn add_layer(&mut self, layer: CameraEffectLayer) {
        if !layer.enabled || layer.weight <= 0.0 {
            return;
        }

        self.translation += layer.translation * layer.weight;
        self.rotation += layer.rotation * layer.weight;
        self.fov_delta += layer.fov_delta * layer.weight;
    }
}

pub fn compose_effect_stack(layers: &[CameraEffectLayer]) -> CameraEffectStack {
    let mut stack = CameraEffectStack::default();

    for layer in layers {
        stack.add_layer(*layer);
    }

    stack
}

#[cfg(test)]
#[path = "layers_tests.rs"]
mod tests;
