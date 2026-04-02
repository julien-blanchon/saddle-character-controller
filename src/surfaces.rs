use avian3d::prelude::Sensor;
use bevy::prelude::*;

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportVelocityPolicy {
    None,
    Horizontal,
    Full,
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
pub struct MovementSurface {
    pub traction_multiplier: f32,
    pub acceleration_multiplier: f32,
    pub speed_multiplier: f32,
    pub jump_multiplier: f32,
    pub conveyor_velocity: Vec3,
    pub inherit_velocity_policy: Option<SupportVelocityPolicy>,
    pub slide_only: bool,
}

impl Default for MovementSurface {
    fn default() -> Self {
        Self {
            traction_multiplier: 1.0,
            acceleration_multiplier: 1.0,
            speed_multiplier: 1.0,
            jump_multiplier: 1.0,
            conveyor_velocity: Vec3::ZERO,
            inherit_velocity_policy: None,
            slide_only: false,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WaterLevel {
    None,
    Feet,
    Waist,
    Head,
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
#[require(Sensor, Transform, GlobalTransform)]
pub struct WaterVolume {
    pub speed_multiplier: f32,
    pub acceleration_multiplier: f32,
    pub gravity_multiplier: f32,
}

impl Default for WaterVolume {
    fn default() -> Self {
        Self {
            speed_multiplier: 1.0,
            acceleration_multiplier: 1.0,
            gravity_multiplier: 1.0,
        }
    }
}

#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource, Debug)]
pub struct CharacterControllerDebugDraw {
    pub enabled: bool,
    pub draw_velocity: bool,
    pub draw_ground: bool,
    pub draw_support: bool,
}

impl Default for CharacterControllerDebugDraw {
    fn default() -> Self {
        Self {
            enabled: false,
            draw_velocity: true,
            draw_ground: true,
            draw_support: true,
        }
    }
}
