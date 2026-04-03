use crate::surfaces::WaterLevel;
use bevy::prelude::*;

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementMode {
    Grounded,
    Airborne,
    Flying,
    Swimming,
    Mantling,
    Sliding,
    Custom(u8),
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct GroundContact {
    pub entity: Entity,
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub walkable: bool,
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct MantleState {
    pub target_position: Vec3,
    pub wall_normal: Vec3,
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
pub struct CharacterControllerState {
    pub movement_mode: MovementMode,
    pub orientation: Quat,
    pub ground: Option<GroundContact>,
    pub support_velocity: Vec3,
    pub support_angular_velocity: Vec3,
    pub last_support_entity: Option<Entity>,
    pub crouching: bool,
    pub grounded_frames: u32,
    pub time_since_grounded: f32,
    pub time_since_wall_kick: f32,
    pub detach_time: f32,
    pub water_level: WaterLevel,
    pub water_volume: Option<Entity>,
    pub water_speed_multiplier: f32,
    pub water_acceleration_multiplier: f32,
    pub water_gravity_multiplier: f32,
    pub mantle: Option<MantleState>,
    pub last_jump_speed: f32,
}

impl Default for CharacterControllerState {
    fn default() -> Self {
        Self {
            movement_mode: MovementMode::Airborne,
            orientation: Quat::IDENTITY,
            ground: None,
            support_velocity: Vec3::ZERO,
            support_angular_velocity: Vec3::ZERO,
            last_support_entity: None,
            crouching: false,
            grounded_frames: 0,
            time_since_grounded: f32::MAX / 4.0,
            time_since_wall_kick: f32::MAX / 4.0,
            detach_time: f32::MAX / 4.0,
            water_level: WaterLevel::None,
            water_volume: None,
            water_speed_multiplier: 1.0,
            water_acceleration_multiplier: 1.0,
            water_gravity_multiplier: 1.0,
            mantle: None,
            last_jump_speed: 0.0,
        }
    }
}
