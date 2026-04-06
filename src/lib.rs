mod components;
mod config;
mod effects;
mod input;
mod layers;
mod messages;
mod movement;
mod springs;

pub use components::{
    FpsCamera, FpsCameraCollisionFeedback, FpsCameraExternalEffects, FpsCameraExternalMotion,
    FpsCameraIntent, FpsCameraRuntime,
};
pub use config::{
    AimConfig, AnalogLookConfig, CollisionConfig, ComfortConfig, CrouchConfig, FovConfig,
    FpsCameraConfig, FreeLookConfig, HeadBobConfig, JumpConfig, LandingImpactConfig, LeanConfig,
    LookConfig, MovementConfig, RecoilConfig, ShakeConfig, ShakeNoiseProfile, TiltConfig,
    ViewmodelLagConfig,
};
pub use layers::{CameraEffectLayer, CameraEffectStack, compose_effect_stack};
pub use messages::{CameraRecoilRequest, CameraShakeRequest, FootstepEvent, LandedEvent};
pub use springs::{DecayConfig, decay_scalar, decay_vec2, decay_vec3};

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
    transform::TransformSystems,
};

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FpsCameraSystems {
    ReadIntent,
    UpdateLocomotion,
    UpdateCameraState,
    ComposeEffects,
    SyncProjection,
    SyncTransform,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

#[derive(Resource, Default)]
struct FpsCameraRuntimeActive(bool);

pub struct FpsCameraPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl FpsCameraPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for FpsCameraPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for FpsCameraPlugin {
    fn build(&self, app: &mut App) {
        initialize_runtime_schedule(app, self.deactivate_schedule);

        app.init_resource::<FpsCameraRuntimeActive>()
            .register_type::<AimConfig>()
            .register_type::<AnalogLookConfig>()
            .register_type::<CameraEffectLayer>()
            .register_type::<CameraEffectStack>()
            .register_type::<CollisionConfig>()
            .register_type::<ComfortConfig>()
            .register_type::<CrouchConfig>()
            .register_type::<DecayConfig>()
            .register_type::<FovConfig>()
            .register_type::<FreeLookConfig>()
            .register_type::<FpsCamera>()
            .register_type::<FpsCameraConfig>()
            .register_type::<FpsCameraExternalEffects>()
            .register_type::<FpsCameraCollisionFeedback>()
            .register_type::<FpsCameraExternalMotion>()
            .register_type::<FpsCameraIntent>()
            .register_type::<FpsCameraRuntime>()
            .register_type::<HeadBobConfig>()
            .register_type::<JumpConfig>()
            .register_type::<LandingImpactConfig>()
            .register_type::<LeanConfig>()
            .register_type::<LookConfig>()
            .register_type::<MovementConfig>()
            .register_type::<RecoilConfig>()
            .register_type::<ShakeConfig>()
            .register_type::<ShakeNoiseProfile>()
            .register_type::<TiltConfig>()
            .register_type::<ViewmodelLagConfig>()
            .add_systems(self.activate_schedule, activate_runtime)
            .add_systems(self.deactivate_schedule, deactivate_runtime)
            .configure_sets(
                self.update_schedule,
                (
                    FpsCameraSystems::ReadIntent,
                    FpsCameraSystems::UpdateLocomotion,
                    FpsCameraSystems::UpdateCameraState,
                    FpsCameraSystems::ComposeEffects,
                    FpsCameraSystems::SyncProjection,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                (
                    (input::ensure_initialized, input::apply_look_intent)
                        .chain()
                        .in_set(FpsCameraSystems::ReadIntent),
                    movement::sync_external_motion.in_set(FpsCameraSystems::UpdateLocomotion),
                    effects::update_core_camera_state.in_set(FpsCameraSystems::UpdateCameraState),
                    effects::compose_core_camera.in_set(FpsCameraSystems::ComposeEffects),
                    effects::sync_projection.in_set(FpsCameraSystems::SyncProjection),
                )
                    .run_if(runtime_is_active),
            )
            .configure_sets(PostUpdate, FpsCameraSystems::SyncTransform)
            .add_systems(
                PostUpdate,
                effects::sync_transform
                    .in_set(FpsCameraSystems::SyncTransform)
                    .before(TransformSystems::Propagate)
                    .run_if(runtime_is_active),
            );
    }
}

pub struct FpsCameraLocomotionPlugin {
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl FpsCameraLocomotionPlugin {
    pub fn new(update_schedule: impl ScheduleLabel) -> Self {
        Self {
            update_schedule: update_schedule.intern(),
        }
    }
}

impl Default for FpsCameraLocomotionPlugin {
    fn default() -> Self {
        Self::new(Update)
    }
}

impl Plugin for FpsCameraLocomotionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            self.update_schedule,
            movement::update_internal_locomotion
                .after(movement::sync_external_motion)
                .in_set(FpsCameraSystems::UpdateLocomotion)
                .run_if(runtime_is_active),
        );
    }
}

pub struct FpsCameraEffectsPlugin {
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl FpsCameraEffectsPlugin {
    pub fn new(update_schedule: impl ScheduleLabel) -> Self {
        Self {
            update_schedule: update_schedule.intern(),
        }
    }
}

impl Default for FpsCameraEffectsPlugin {
    fn default() -> Self {
        Self::new(Update)
    }
}

impl Plugin for FpsCameraEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<FootstepEvent>()
            .add_message::<LandedEvent>()
            .add_message::<CameraShakeRequest>()
            .add_message::<CameraRecoilRequest>()
            .add_systems(
                self.update_schedule,
                (
                    (
                        effects::apply_shake_requests,
                        effects::apply_recoil_requests,
                    )
                        .chain()
                        .in_set(FpsCameraSystems::ReadIntent),
                    effects::update_camera_state
                        .after(effects::update_core_camera_state)
                        .in_set(FpsCameraSystems::UpdateCameraState),
                    effects::compose_effects
                        .after(effects::compose_core_camera)
                        .in_set(FpsCameraSystems::ComposeEffects),
                )
                    .run_if(runtime_is_active),
            );
    }
}

pub struct FpsCameraLegacyPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl FpsCameraLegacyPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for FpsCameraLegacyPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for FpsCameraLegacyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FpsCameraPlugin {
                activate_schedule: self.activate_schedule,
                deactivate_schedule: self.deactivate_schedule,
                update_schedule: self.update_schedule,
            },
            FpsCameraLocomotionPlugin {
                update_schedule: self.update_schedule,
            },
            FpsCameraEffectsPlugin {
                update_schedule: self.update_schedule,
            },
        ));
    }
}

fn initialize_runtime_schedule(app: &mut App, deactivate_schedule: Interned<dyn ScheduleLabel>) {
    if deactivate_schedule == NeverDeactivateSchedule.intern() {
        app.init_schedule(NeverDeactivateSchedule);
    }
}

fn activate_runtime(mut runtime: ResMut<FpsCameraRuntimeActive>) {
    runtime.0 = true;
}

fn deactivate_runtime(mut runtime: ResMut<FpsCameraRuntimeActive>) {
    runtime.0 = false;
}

fn runtime_is_active(runtime: Res<FpsCameraRuntimeActive>) -> bool {
    runtime.0
}

#[cfg(test)]
#[path = "plugin_tests.rs"]
mod plugin_tests;
