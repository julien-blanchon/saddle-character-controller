use crate::MovementMode;
use bevy::prelude::*;

#[derive(Message, Clone, Debug)]
pub struct CharacterJumped {
    pub entity: Entity,
}

#[derive(Message, Clone, Debug)]
pub struct CharacterLanded {
    pub entity: Entity,
    pub impact_speed: f32,
    pub inherited_platform_velocity: Vec3,
}

#[derive(Message, Clone, Debug)]
pub struct MovementModeChanged {
    pub entity: Entity,
    pub previous: MovementMode,
    pub current: MovementMode,
}

#[derive(Message, Clone, Debug)]
pub struct SupportBodyChanged {
    pub entity: Entity,
    pub previous: Option<Entity>,
    pub current: Option<Entity>,
}
