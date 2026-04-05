use bevy::prelude::*;

use crate::{CameraEffectStack, FpsCameraConfig, config::MovementConfig};

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
#[require(
    Transform,
    FpsCameraConfig,
    FpsCameraIntent,
    FpsCameraRuntime,
    FpsCameraExternalEffects,
    FpsCameraInternalState
)]
pub struct FpsCamera;

#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct FpsCameraIntent {
    pub move_axis: Vec2,
    pub look_delta: Vec2,
    pub look_analog: Vec2,
    pub jump_pressed: bool,
    pub sprint_pressed: bool,
    pub crouch_pressed: bool,
    pub aim_pressed: bool,
    pub lean: f32,
    pub free_look: bool,
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct FpsCameraRuntime {
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub grounded: bool,
    pub speed: f32,
    pub speed_ratio: f32,
    pub fall_speed: f32,
    pub crouch_alpha: f32,
    pub sprint_alpha: f32,
    pub aim_alpha: f32,
    pub lean_alpha: f32,
    pub free_look_offset: Vec2,
    pub eye_height: f32,
    pub bob_phase: f32,
    pub trauma: f32,
    pub visual_fov: f32,
    pub recent_landing_impulse: f32,
    pub head_bob_offset: Vec3,
    pub landing_offset: Vec3,
    pub shake_offset: Vec3,
    pub recoil_offset: Vec2,
    pub lean_offset: Vec3,
    pub tilt_roll: f32,
    pub viewmodel_translation: Vec3,
    pub viewmodel_rotation: Vec3,
    pub render_translation: Vec3,
    pub render_rotation: Vec3,
    pub effect_stack: CameraEffectStack,
}

impl Default for FpsCameraRuntime {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            grounded: true,
            speed: 0.0,
            speed_ratio: 0.0,
            fall_speed: 0.0,
            crouch_alpha: 0.0,
            sprint_alpha: 0.0,
            aim_alpha: 0.0,
            lean_alpha: 0.0,
            free_look_offset: Vec2::ZERO,
            eye_height: MovementConfig::default().eye_height,
            bob_phase: 0.0,
            trauma: 0.0,
            visual_fov: 0.0,
            recent_landing_impulse: 0.0,
            head_bob_offset: Vec3::ZERO,
            landing_offset: Vec3::ZERO,
            shake_offset: Vec3::ZERO,
            recoil_offset: Vec2::ZERO,
            lean_offset: Vec3::ZERO,
            tilt_roll: 0.0,
            viewmodel_translation: Vec3::ZERO,
            viewmodel_rotation: Vec3::ZERO,
            render_translation: Vec3::ZERO,
            render_rotation: Vec3::ZERO,
            effect_stack: CameraEffectStack::default(),
        }
    }
}

#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct FpsCameraExternalMotion {
    pub enabled: bool,
    pub position: Vec3,
    pub velocity: Vec3,
    pub grounded: bool,
    pub landing_impulse: f32,
    pub crouch_alpha: Option<f32>,
    pub sprint_alpha: Option<f32>,
}

#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct FpsCameraExternalEffects {
    pub enabled: bool,
    pub translation: Vec3,
    pub rotation: Vec3,
    pub fov_delta: f32,
    pub weight: f32,
}

#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct FpsCameraCollisionFeedback {
    pub blocked: bool,
    pub nearest_distance: f32,
    pub push_normal: Vec3,
}

#[derive(Component, Debug, Clone)]
pub(crate) struct FpsCameraInternalState {
    pub initialized: bool,
    pub base_ground_height: f32,
    pub last_planar_position: Vec2,
    pub look_smoothing: Vec2,
    pub recoil: Vec2,
    pub landing_amount: f32,
    pub tilt_roll: f32,
    pub lean_alpha: f32,
    pub cumulative_bob_phase: f32,
    pub next_footstep_phase: f32,
    pub shake_time: f32,
    pub recent_look_delta: Vec2,
    pub viewmodel_translation: Vec3,
    pub viewmodel_rotation: Vec3,
    pub shake_decay_override: Option<f32>,
    pub recoil_recovery_override: Option<crate::DecayConfig>,
    pub previous_grounded: bool,
}

impl Default for FpsCameraInternalState {
    fn default() -> Self {
        Self {
            initialized: false,
            base_ground_height: 0.0,
            last_planar_position: Vec2::ZERO,
            look_smoothing: Vec2::ZERO,
            recoil: Vec2::ZERO,
            landing_amount: 0.0,
            tilt_roll: 0.0,
            lean_alpha: 0.0,
            cumulative_bob_phase: 0.0,
            next_footstep_phase: std::f32::consts::PI,
            shake_time: 0.0,
            recent_look_delta: Vec2::ZERO,
            viewmodel_translation: Vec3::ZERO,
            viewmodel_rotation: Vec3::ZERO,
            shake_decay_override: None,
            recoil_recovery_override: None,
            previous_grounded: true,
        }
    }
}

#[cfg(test)]
#[path = "components_tests.rs"]
mod components_tests;
