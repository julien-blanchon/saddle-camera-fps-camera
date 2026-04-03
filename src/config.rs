use bevy::prelude::*;

use crate::DecayConfig;

#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct FpsCameraConfig {
    pub look: LookConfig,
    pub movement: MovementConfig,
    pub crouch: CrouchConfig,
    pub jump: JumpConfig,
    pub head_bob: HeadBobConfig,
    pub fov: FovConfig,
    pub shake: ShakeConfig,
    pub tilt: TiltConfig,
    pub landing: LandingImpactConfig,
    pub recoil: RecoilConfig,
    pub viewmodel: ViewmodelLagConfig,
    pub aim: AimConfig,
    pub lean: LeanConfig,
    pub free_look: FreeLookConfig,
    pub comfort: ComfortConfig,
}

#[derive(Reflect, Debug, Clone)]
pub struct LookConfig {
    pub sensitivity: Vec2,
    pub invert_x: bool,
    pub invert_y: bool,
    pub smoothing: DecayConfig,
    pub pitch_min: f32,
    pub pitch_max: f32,
    pub analog: AnalogLookConfig,
}

impl Default for LookConfig {
    fn default() -> Self {
        Self {
            sensitivity: Vec2::new(0.0022, 0.0020),
            invert_x: false,
            invert_y: false,
            smoothing: DecayConfig {
                decay_rate: 0.0,
                snap_threshold: 0.0,
            },
            pitch_min: -1.50,
            pitch_max: 1.50,
            analog: AnalogLookConfig::default(),
        }
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct AnalogLookConfig {
    pub enabled: bool,
    pub max_radians_per_second: Vec2,
    pub deadzone: f32,
    pub outer_deadzone: f32,
    pub exponent: f32,
}

impl Default for AnalogLookConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_radians_per_second: Vec2::new(3.6, 2.8),
            deadzone: 0.18,
            outer_deadzone: 0.05,
            exponent: 1.35,
        }
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct MovementConfig {
    pub eye_height: f32,
    pub walk_speed: f32,
    pub sprint_multiplier: f32,
    pub sprint_transition: DecayConfig,
    pub acceleration: f32,
    pub deceleration: f32,
    pub air_acceleration: f32,
    pub air_control: f32,
    pub max_air_speed: f32,
    pub gravity: f32,
    pub terminal_velocity: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            eye_height: 1.62,
            walk_speed: 4.8,
            sprint_multiplier: 1.45,
            sprint_transition: DecayConfig::new(12.0),
            acceleration: 30.0,
            deceleration: 34.0,
            air_acceleration: 10.0,
            air_control: 0.55,
            max_air_speed: 4.0,
            gravity: 22.0,
            terminal_velocity: 45.0,
        }
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct CrouchConfig {
    pub enabled: bool,
    pub eye_height: f32,
    pub speed_multiplier: f32,
    pub transition: DecayConfig,
}

impl Default for CrouchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            eye_height: 1.1,
            speed_multiplier: 0.58,
            transition: DecayConfig::new(14.0),
        }
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct JumpConfig {
    pub enabled: bool,
    pub height: f32,
    pub fall_multiplier: f32,
    pub landing_velocity_threshold: f32,
}

impl Default for JumpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            height: 1.2,
            fall_multiplier: 1.15,
            landing_velocity_threshold: 2.5,
        }
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct HeadBobConfig {
    pub enabled: bool,
    pub amplitude: Vec3,
    pub stride_length: f32,
    pub sprint_multiplier: f32,
    pub crouch_multiplier: f32,
    pub idle_sway_translation: Vec3,
    pub idle_sway_rotation: Vec2,
    pub idle_sway_frequency: f32,
}

impl Default for HeadBobConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            amplitude: Vec3::new(0.025, 0.045, 0.018),
            stride_length: 1.55,
            sprint_multiplier: 1.35,
            crouch_multiplier: 0.65,
            idle_sway_translation: Vec3::new(0.004, 0.006, 0.003),
            idle_sway_rotation: Vec2::new(0.006, 0.004),
            idle_sway_frequency: 1.3,
        }
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct FovConfig {
    pub base_fov: f32,
    pub speed_boost: f32,
    pub sprint_boost: f32,
    pub response: DecayConfig,
}

impl Default for FovConfig {
    fn default() -> Self {
        Self {
            base_fov: 85.0_f32.to_radians(),
            speed_boost: 8.0_f32.to_radians(),
            sprint_boost: 3.0_f32.to_radians(),
            response: DecayConfig::new(10.0),
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ShakeNoiseProfile {
    #[default]
    Standard,
    Handheld,
    Explosion,
    Rumble,
}

#[derive(Reflect, Debug, Clone)]
pub struct ShakeConfig {
    pub translation_amplitude: Vec3,
    pub rotation_amplitude: Vec3,
    pub decay_rate: f32,
    pub frequency: f32,
    pub max_trauma: f32,
    pub seed: f32,
    pub noise_profile: ShakeNoiseProfile,
}

impl Default for ShakeConfig {
    fn default() -> Self {
        Self {
            translation_amplitude: Vec3::new(0.03, 0.04, 0.02),
            rotation_amplitude: Vec3::new(0.03, 0.04, 0.02),
            decay_rate: 1.85,
            frequency: 27.0,
            max_trauma: 1.0,
            seed: 0.37,
            noise_profile: ShakeNoiseProfile::Standard,
        }
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct TiltConfig {
    pub enabled: bool,
    pub max_roll: f32,
    pub response: DecayConfig,
}

impl Default for TiltConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_roll: 4.5_f32.to_radians(),
            response: DecayConfig::new(16.0),
        }
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct LandingImpactConfig {
    pub enabled: bool,
    pub translation_amount: f32,
    pub pitch_amount: f32,
    pub max_impulse: f32,
    pub response: DecayConfig,
}

impl Default for LandingImpactConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            translation_amount: 0.14,
            pitch_amount: 7.0_f32.to_radians(),
            max_impulse: 1.0,
            response: DecayConfig::new(10.0),
        }
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct RecoilConfig {
    pub enabled: bool,
    pub recovery: DecayConfig,
    pub max_pitch: f32,
    pub max_yaw: f32,
}

impl Default for RecoilConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            recovery: DecayConfig::new(18.0),
            max_pitch: 14.0_f32.to_radians(),
            max_yaw: 9.0_f32.to_radians(),
        }
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct ViewmodelLagConfig {
    pub enabled: bool,
    pub translation_scale: Vec3,
    pub rotation_scale: Vec3,
    pub movement_scale: Vec3,
    pub response: DecayConfig,
    pub max_translation: Vec3,
    pub max_rotation: Vec3,
}

impl Default for ViewmodelLagConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            translation_scale: Vec3::new(0.045, 0.028, 0.030),
            rotation_scale: Vec3::new(0.030, 0.036, 0.050),
            movement_scale: Vec3::new(0.008, 0.004, 0.010),
            response: DecayConfig::new(18.0),
            max_translation: Vec3::new(0.12, 0.10, 0.10),
            max_rotation: Vec3::new(
                7.0_f32.to_radians(),
                8.0_f32.to_radians(),
                10.0_f32.to_radians(),
            ),
        }
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct AimConfig {
    pub enabled: bool,
    pub transition: DecayConfig,
    pub sensitivity_scale: f32,
    pub fov_multiplier: f32,
}

impl Default for AimConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            transition: DecayConfig::new(16.0),
            sensitivity_scale: 0.65,
            fov_multiplier: 0.84,
        }
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct LeanConfig {
    pub enabled: bool,
    pub max_angle: f32,
    pub lateral_offset: f32,
    pub response: DecayConfig,
}

impl Default for LeanConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_angle: 10.0_f32.to_radians(),
            lateral_offset: 0.09,
            response: DecayConfig::new(15.0),
        }
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct FreeLookConfig {
    pub enabled: bool,
    pub yaw_limit: f32,
    pub pitch_limit: f32,
    pub recenter: DecayConfig,
}

impl Default for FreeLookConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            yaw_limit: 55.0_f32.to_radians(),
            pitch_limit: 18.0_f32.to_radians(),
            recenter: DecayConfig::new(12.0),
        }
    }
}

#[derive(Reflect, Debug, Clone)]
pub struct ComfortConfig {
    pub bob_weight: f32,
    pub roll_weight: f32,
    pub shake_weight: f32,
    pub dynamic_fov_weight: f32,
    pub landing_weight: f32,
}

impl ComfortConfig {
    pub fn low_motion() -> Self {
        Self {
            bob_weight: 0.18,
            roll_weight: 0.10,
            shake_weight: 0.15,
            dynamic_fov_weight: 0.12,
            landing_weight: 0.18,
        }
    }

    pub fn vr_mode() -> Self {
        Self {
            bob_weight: 0.0,
            roll_weight: 0.0,
            shake_weight: 0.08,
            dynamic_fov_weight: 0.0,
            landing_weight: 0.06,
        }
    }
}

impl Default for ComfortConfig {
    fn default() -> Self {
        Self {
            bob_weight: 1.0,
            roll_weight: 1.0,
            shake_weight: 1.0,
            dynamic_fov_weight: 1.0,
            landing_weight: 1.0,
        }
    }
}
