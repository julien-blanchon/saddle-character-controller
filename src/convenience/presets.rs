use crate::CharacterController;
use core::time::Duration;

/// Convenience tuning presets for demos and quick prototypes.
///
/// These helpers live outside the movement core so the crate can stay explicit about
/// what is opinionated demo tuning versus what is foundational controller behavior.
pub struct CharacterControllerPreset;

impl CharacterControllerPreset {
    /// Standard FPS / exploration controller with Quake-style movement.
    pub fn default_fps() -> CharacterController {
        CharacterController::default()
    }

    /// Responsive platformer: higher gravity, larger coyote time, double jump.
    pub fn platformer() -> CharacterController {
        CharacterController {
            gravity: 38.0,
            fall_gravity_multiplier: 1.6,
            jump_cut_gravity_multiplier: 4.0,
            jump_height: 2.4,
            max_air_jumps: 1,
            coyote_time: Duration::from_millis(130),
            jump_input_buffer: Duration::from_millis(180),
            speed: 10.0,
            air_acceleration_hz: 18.0,
            ..Default::default()
        }
    }

    /// Slow, deliberate exploration controller (walking simulator style).
    pub fn explorer() -> CharacterController {
        CharacterController {
            speed: 5.0,
            sprint_speed_scale: 1.2,
            gravity: 20.0,
            jump_height: 1.0,
            air_acceleration_hz: 4.0,
            ..Default::default()
        }
    }

    /// Fast arena / bunny-hop tuning with auto-bhop and air strafe.
    pub fn arena() -> CharacterController {
        CharacterController {
            speed: 14.0,
            sprint_speed_scale: 1.0,
            air_speed: 3.0,
            max_air_wish_speed: 1.2,
            air_acceleration_hz: 30.0,
            auto_bhop: true,
            gravity: 24.0,
            jump_height: 2.0,
            ..Default::default()
        }
    }
}
