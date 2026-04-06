use bevy::prelude::*;
use core::time::Duration;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
pub struct CharacterWallKick {
    pub power: f32,
    pub upward_factor: f32,
    pub distance: f32,
    pub input_buffer: Duration,
    pub cooldown: Duration,
    pub max_wall_angle: f32,
}

impl Default for CharacterWallKick {
    fn default() -> Self {
        Self {
            power: 0.9,
            upward_factor: 1.0,
            distance: 0.4,
            input_buffer: Duration::from_millis(150),
            cooldown: Duration::from_millis(300),
            max_wall_angle: 40.0_f32.to_radians(),
        }
    }
}

pub struct CharacterControllerWallKickPlugin;

impl Plugin for CharacterControllerWallKickPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterWallKick>();
    }
}
