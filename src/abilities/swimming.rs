use bevy::prelude::*;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
pub struct CharacterSwimming {
    pub acceleration_hz: f32,
    pub gravity: f32,
    pub slowdown: f32,
    pub ascent_speed_scale: f32,
}

impl Default for CharacterSwimming {
    fn default() -> Self {
        Self {
            acceleration_hz: 6.0,
            gravity: 2.4,
            slowdown: 0.6,
            ascent_speed_scale: 1.0,
        }
    }
}

pub struct CharacterControllerSwimmingPlugin;

impl Plugin for CharacterControllerSwimmingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterSwimming>();
    }
}
