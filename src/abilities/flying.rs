use bevy::prelude::*;

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlightCollisionMode {
    Slide,
    NoClip,
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
pub struct CharacterFlying {
    pub enabled: bool,
    pub speed: f32,
    pub sprint_speed_scale: f32,
    pub acceleration_hz: f32,
    pub drag_hz: f32,
    pub vertical_speed_scale: f32,
    pub collision_mode: FlightCollisionMode,
}

impl Default for CharacterFlying {
    fn default() -> Self {
        Self {
            enabled: false,
            speed: 14.0,
            sprint_speed_scale: 1.4,
            acceleration_hz: 8.0,
            drag_hz: 6.0,
            vertical_speed_scale: 1.0,
            collision_mode: FlightCollisionMode::Slide,
        }
    }
}

/// Adds flying movement mode. Attach [`CharacterFlying`] to a controller entity.
pub struct CharacterControllerFlyingPlugin;

impl Plugin for CharacterControllerFlyingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterFlying>()
            .register_type::<FlightCollisionMode>();
    }
}
