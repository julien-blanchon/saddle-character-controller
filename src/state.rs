use bevy::prelude::*;

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementMode {
    Grounded,
    Airborne,
    Flying,
    Swimming,
    Dashing,
    Mantling,
    Sliding,
    Custom(u8),
}

/// Readable runtime state published by the controller each tick.
///
/// This component is automatically required by [`CharacterController`] and should be treated
/// as read-only by consumer code. The controller simulation writes to it; downstream systems
/// and presentation layers read from it.
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

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct DashState {
    pub direction: Vec3,
    pub remaining: f32,
    pub speed: f32,
}

/// Controller operational mode.
///
/// `Enabled` runs the full simulation. `SenseOnly` still updates ground probing and
/// environment detection but does not move the character. `Disabled` skips all processing.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ControllerMode {
    #[default]
    Enabled,
    /// Still probes ground and environment but does not move.
    SenseOnly,
    /// Skips all controller processing.
    Disabled,
}

/// Depth classification for environment volumes (water, lava, fog, etc.).
///
/// `WaterLevel` is provided as a convenience alias.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EnvironmentDepth {
    None,
    Shallow,
    Medium,
    Submerged,
}

/// Convenience alias — semantically identical to [`EnvironmentDepth`].
pub type WaterLevel = EnvironmentDepth;

/// Runtime environment modifiers applied by overlapping volumes (water, lava, etc.).
///
/// This component is automatically required by [`CharacterController`]. Environment detection
/// systems (like the built-in water system) write to this component; the movement pipeline
/// reads from it. Game-specific environment volumes should write to these fields rather than
/// patching `CharacterControllerState` directly.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
pub struct EnvironmentModifiers {
    pub depth: EnvironmentDepth,
    pub active_volume: Option<Entity>,
    pub speed_multiplier: f32,
    pub acceleration_multiplier: f32,
    pub gravity_multiplier: f32,
}

impl Default for EnvironmentModifiers {
    fn default() -> Self {
        Self {
            depth: EnvironmentDepth::None,
            active_volume: None,
            speed_multiplier: 1.0,
            acceleration_multiplier: 1.0,
            gravity_multiplier: 1.0,
        }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
pub struct CharacterControllerState {
    pub movement_mode: MovementMode,
    pub controller_mode: ControllerMode,
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
    pub mantle: Option<MantleState>,
    pub dash: Option<DashState>,
    pub last_jump_speed: f32,
    /// Number of air jumps consumed since last grounding. Reset on landing.
    pub air_jumps_used: u32,
    /// True while ascending from a jump and the jump button is still held.
    pub jump_held: bool,
}

impl Default for CharacterControllerState {
    fn default() -> Self {
        Self {
            movement_mode: MovementMode::Airborne,
            controller_mode: ControllerMode::default(),
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
            mantle: None,
            dash: None,
            last_jump_speed: 0.0,
            air_jumps_used: 0,
            jump_held: false,
        }
    }
}
