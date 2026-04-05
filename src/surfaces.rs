use avian3d::prelude::Sensor;
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

/// Generic environment volume that applies movement modifiers when the controller
/// overlaps it. `WaterVolume` is the built-in convenience type; games can create
/// custom volumes (lava, fog, quicksand) using the same pattern.
///
/// Attach to an entity with a `Sensor` collider. The built-in environment system
/// will detect overlap and write the modifiers into [`EnvironmentModifiers`](crate::EnvironmentModifiers).
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
#[require(Sensor, Transform, GlobalTransform)]
pub struct EnvironmentVolume {
    pub speed_multiplier: f32,
    pub acceleration_multiplier: f32,
    pub gravity_multiplier: f32,
    /// If true, this volume can trigger swimming mode when `CharacterSwimming` is present.
    pub swim_volume: bool,
}

impl Default for EnvironmentVolume {
    fn default() -> Self {
        Self {
            speed_multiplier: 1.0,
            acceleration_multiplier: 1.0,
            gravity_multiplier: 1.0,
            swim_volume: false,
        }
    }
}

/// Convenience constructor for water-type environment volumes.
///
/// A `WaterVolume` is an [`EnvironmentVolume`] with `swim_volume: true`.
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
