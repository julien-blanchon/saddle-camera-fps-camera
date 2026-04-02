use bevy::prelude::*;

#[derive(Message, Debug, Clone, Copy)]
pub struct FootstepEvent {
    pub entity: Entity,
    pub phase: f32,
    pub speed: f32,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct LandedEvent {
    pub entity: Entity,
    pub impact_speed: f32,
    pub landing_impulse: f32,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct CameraShakeRequest {
    pub entity: Entity,
    pub trauma: f32,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct CameraRecoilRequest {
    pub entity: Entity,
    pub pitch: f32,
    pub yaw: f32,
}
