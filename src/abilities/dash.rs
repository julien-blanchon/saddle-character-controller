use bevy::prelude::*;
use core::time::Duration;

/// Optional dash ability. Attach to a controller entity to enable dashing.
///
/// When triggered by your input adapter, the controller enters a direction-locked,
/// gravity-free burst of movement for the configured duration.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
pub struct CharacterDash {
    /// Dash speed in units per second.
    pub speed: f32,
    /// Total dash duration.
    pub duration: Duration,
    /// Cooldown between dashes.
    pub cooldown: Duration,
    /// If true, gravity is cancelled during the dash.
    pub cancel_gravity: bool,
    /// Maximum air dashes before needing to land (0 = unlimited).
    pub max_air_dashes: u32,
    /// Time since last dash ended. Managed by the controller; do not set manually.
    pub time_since_dash: f32,
    /// Air dashes used since last grounding.
    pub air_dashes_used: u32,
}

impl Default for CharacterDash {
    fn default() -> Self {
        Self {
            speed: 28.0,
            duration: Duration::from_millis(180),
            cooldown: Duration::from_millis(400),
            cancel_gravity: true,
            max_air_dashes: 1,
            time_since_dash: f32::MAX / 4.0,
            air_dashes_used: 0,
        }
    }
}

pub struct CharacterControllerDashPlugin;

impl Plugin for CharacterControllerDashPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterDash>();
    }
}
