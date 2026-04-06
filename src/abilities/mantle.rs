use bevy::prelude::*;
use core::time::Duration;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
pub struct CharacterMantle {
    pub max_height: f32,
    pub max_distance: f32,
    pub min_wall_angle: f32,
    pub speed: f32,
    pub pull_up_height: f32,
    pub input_buffer: Duration,
}

impl Default for CharacterMantle {
    fn default() -> Self {
        Self {
            max_height: 1.0,
            max_distance: 0.3,
            min_wall_angle: 50.0_f32.to_radians(),
            speed: 5.0,
            pull_up_height: 0.3,
            input_buffer: Duration::from_millis(60),
        }
    }
}

pub struct CharacterControllerMantlePlugin;

impl Plugin for CharacterControllerMantlePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterMantle>();
    }
}
