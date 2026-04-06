use bevy::prelude::*;

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportVelocityPolicy {
    None,
    Horizontal,
    Full,
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportRotationPolicy {
    None,
    YawOnly,
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
    pub inherit_rotation_policy: Option<SupportRotationPolicy>,
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
            inherit_rotation_policy: None,
            slide_only: false,
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
    pub draw_capsule: bool,
    pub draw_environment: bool,
    pub draw_dash: bool,
}

impl Default for CharacterControllerDebugDraw {
    fn default() -> Self {
        Self {
            enabled: false,
            draw_velocity: true,
            draw_ground: true,
            draw_support: true,
            draw_capsule: false,
            draw_environment: false,
            draw_dash: false,
        }
    }
}
