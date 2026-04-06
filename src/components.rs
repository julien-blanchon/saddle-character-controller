use crate::{
    state::{CharacterControllerState, ControllerMode, EnvironmentModifiers},
    surfaces::{SupportRotationPolicy, SupportVelocityPolicy},
};
use avian3d::{
    character_controller::move_and_slide::MoveAndSlideConfig,
    parry::shape::{Capsule, SharedShape},
    prelude::*,
};
use bevy::prelude::*;
use core::time::Duration;
use std::sync::Arc;

const DEFAULT_CAPSULE_RADIUS: f32 = 0.4;
const DEFAULT_CAPSULE_HALF_HEIGHT: f32 = 0.9;
const DEFAULT_CROUCH_HEIGHT: f32 = 1.3;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
#[require(
    crate::AccumulatedInput,
    CharacterControllerState,
    EnvironmentModifiers,
    CharacterMotionStats,
    CharacterColliderCache,
    CharacterControllerScratch,
    Collider = Collider::capsule(DEFAULT_CAPSULE_RADIUS, 2.0 * DEFAULT_CAPSULE_HALF_HEIGHT - 2.0 * DEFAULT_CAPSULE_RADIUS),
    CollidingEntities,
    CustomPositionIntegration,
    LinearVelocity = LinearVelocity::ZERO,
    RigidBody = RigidBody::Kinematic,
    SpeculativeMargin::ZERO,
    Transform,
    TransformInterpolation
)]
pub struct CharacterController {
    pub filter: SpatialQueryFilter,
    pub standing_view_height: f32,
    pub crouch_view_height: f32,
    pub speed: f32,
    pub sprint_speed_scale: f32,
    pub air_speed: f32,
    pub max_air_wish_speed: f32,
    pub gravity: f32,
    pub fall_gravity_multiplier: f32,
    /// Extra gravity applied when the jump button is released during ascent.
    /// Set to 1.0 for fixed-height jumps. Higher values (2.0–4.0) give snappy
    /// variable-height jumps. Default: 3.0.
    pub jump_cut_gravity_multiplier: f32,
    pub terminal_velocity: f32,
    pub friction_hz: f32,
    pub acceleration_hz: f32,
    pub air_acceleration_hz: f32,
    pub stop_speed: f32,
    pub jump_height: f32,
    /// Maximum extra jumps available while airborne (0 = no air jumps, 1 = double jump, etc.).
    pub max_air_jumps: u32,
    pub coyote_time: Duration,
    pub jump_input_buffer: Duration,
    pub ground_distance: f32,
    pub snap_distance: f32,
    pub step_size: f32,
    pub step_down_detection_distance: f32,
    pub min_walk_angle: f32,
    pub skin_width: f32,
    pub capsule_radius: f32,
    pub capsule_half_height: f32,
    pub crouch_height: f32,
    pub crouch_speed_scale: f32,
    pub auto_bhop: bool,
    pub global_speed_scale: f32,
    pub max_speed: f32,
    pub unground_speed: f32,
    pub slide_gravity_scale: f32,
    pub support_velocity_policy: SupportVelocityPolicy,
    pub support_rotation_policy: SupportRotationPolicy,
    pub support_detach_grace: Duration,
    /// Controller operational mode. See [`ControllerMode`].
    pub controller_mode: ControllerMode,
}

impl Default for CharacterController {
    fn default() -> Self {
        Self {
            filter: SpatialQueryFilter::default(),
            standing_view_height: 1.7,
            crouch_view_height: 1.2,
            speed: 12.0,
            sprint_speed_scale: 1.5,
            air_speed: 1.5,
            max_air_wish_speed: 0.76,
            gravity: 29.0,
            fall_gravity_multiplier: 1.0,
            jump_cut_gravity_multiplier: 3.0,
            terminal_velocity: 50.0,
            friction_hz: 12.0,
            acceleration_hz: 8.0,
            air_acceleration_hz: 12.0,
            stop_speed: 2.54,
            jump_height: 1.8,
            max_air_jumps: 0,
            coyote_time: Duration::from_millis(100),
            jump_input_buffer: Duration::from_millis(150),
            ground_distance: 0.05,
            snap_distance: 0.1,
            step_size: 0.7,
            step_down_detection_distance: 0.2,
            min_walk_angle: 40.0_f32.to_radians(),
            skin_width: 0.01,
            capsule_radius: DEFAULT_CAPSULE_RADIUS,
            capsule_half_height: DEFAULT_CAPSULE_HALF_HEIGHT,
            crouch_height: DEFAULT_CROUCH_HEIGHT,
            crouch_speed_scale: 0.33,
            auto_bhop: false,
            global_speed_scale: 1.0,
            max_speed: 100.0,
            unground_speed: 10.0,
            slide_gravity_scale: 1.0,
            support_velocity_policy: SupportVelocityPolicy::Horizontal,
            support_rotation_policy: SupportRotationPolicy::YawOnly,
            support_detach_grace: Duration::from_millis(120),
            controller_mode: ControllerMode::default(),
        }
    }
}

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

/// Per-entity gravity override. When present, this takes priority over
/// `CharacterController::gravity`.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
pub struct CharacterGravity {
    pub magnitude: f32,
    pub direction: Vec3,
}

impl Default for CharacterGravity {
    fn default() -> Self {
        Self {
            magnitude: 29.0,
            direction: Vec3::NEG_Y,
        }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
pub struct CharacterPush {
    pub impulse_scale: f32,
}

impl Default for CharacterPush {
    fn default() -> Self {
        Self { impulse_scale: 5.0 }
    }
}

#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component, Debug)]
pub struct ExternalMotion {
    pub velocity_delta: Vec3,
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
pub struct CharacterLook {
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: Vec2,
    pub min_pitch: f32,
    pub max_pitch: f32,
}

impl Default for CharacterLook {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            sensitivity: Vec2::ONE,
            min_pitch: -89.0_f32.to_radians(),
            max_pitch: 89.0_f32.to_radians(),
        }
    }
}

impl CharacterLook {
    pub fn orientation(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0)
    }
}

#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component, Debug)]
pub struct CharacterMotionStats {
    pub current_speed: f32,
    pub horizontal_speed: f32,
    pub grounded_time: f32,
    pub airborne_time: f32,
    pub last_ground_normal: Vec3,
    pub last_support_entity: Option<Entity>,
    pub shape_casts_last_tick: u32,
}

#[derive(Component, Clone, Debug)]
pub(crate) struct CharacterColliderCache {
    pub standing_collider: Collider,
    pub crouching_collider: Collider,
    pub hand_collider: Collider,
    pub standing_height: f32,
    pub standing_half_height: f32,
    pub crouching_height: f32,
    pub radius: f32,
}

impl Default for CharacterColliderCache {
    fn default() -> Self {
        let standing_collider = Collider::capsule(
            DEFAULT_CAPSULE_RADIUS,
            2.0 * DEFAULT_CAPSULE_HALF_HEIGHT - 2.0 * DEFAULT_CAPSULE_RADIUS,
        );
        let crouching_collider = standing_collider.clone();
        Self {
            standing_collider,
            crouching_collider,
            hand_collider: Collider::cuboid(0.2, 0.1, 0.2),
            standing_height: 2.0 * DEFAULT_CAPSULE_HALF_HEIGHT,
            standing_half_height: DEFAULT_CAPSULE_HALF_HEIGHT,
            crouching_height: DEFAULT_CROUCH_HEIGHT,
            radius: DEFAULT_CAPSULE_RADIUS,
        }
    }
}

impl CharacterColliderCache {
    pub fn active<'a>(&'a self, state: &CharacterControllerState) -> &'a Collider {
        if state.crouching {
            &self.crouching_collider
        } else {
            &self.standing_collider
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TouchContact {
    pub entity: Entity,
    pub point: Vec3,
    pub normal: Vec3,
}

impl Default for TouchContact {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            point: Vec3::ZERO,
            normal: Vec3::ZERO,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PendingLanding {
    pub impact_speed: f32,
    pub inherited_platform_velocity: Vec3,
}

#[derive(Component, Debug)]
pub(crate) struct CharacterControllerScratch {
    pub move_config: MoveAndSlideConfig,
    pub contacts: Vec<TouchContact>,
    pub pending_mode_change: Option<(crate::MovementMode, crate::MovementMode)>,
    pub pending_support_change: Option<(Option<Entity>, Option<Entity>)>,
    pub pending_jump: bool,
    pub pending_landing: Option<PendingLanding>,
    pub previous_mode: crate::MovementMode,
    pub previous_support: Option<Entity>,
    pub previous_grounded: bool,
}

impl Default for CharacterControllerScratch {
    fn default() -> Self {
        let mut move_config = MoveAndSlideConfig::default();
        move_config.planes.reserve(4);
        Self {
            move_config,
            contacts: Vec::with_capacity(8),
            pending_mode_change: None,
            pending_support_change: None,
            pending_jump: false,
            pending_landing: None,
            previous_mode: crate::MovementMode::Airborne,
            previous_support: None,
            previous_grounded: false,
        }
    }
}

pub(crate) fn refresh_collider_cache(
    cache: &mut CharacterColliderCache,
    controller: &CharacterController,
) {
    let standing_height = controller.capsule_half_height * 2.0;
    let standing_segment = (standing_height - controller.capsule_radius * 2.0).max(0.0);
    cache.standing_collider = Collider::capsule(controller.capsule_radius, standing_segment);

    let crouch_half_height = (controller.crouch_height * 0.5).max(controller.capsule_radius);
    let crouch_segment = (controller.crouch_height - controller.capsule_radius * 2.0).max(0.0);
    let crouch_inner = Collider::from(SharedShape(Arc::new(Capsule::new_y(
        crouch_segment * 0.5,
        controller.capsule_radius,
    ))));
    let crouch_offset = Vec3::Y * ((controller.crouch_height - standing_height) * 0.5);
    cache.crouching_collider =
        Collider::compound(vec![(crouch_offset, Rotation::default(), crouch_inner)]);
    cache.hand_collider = Collider::cuboid(0.2, 0.1, controller.capsule_radius.max(0.2));
    cache.standing_height = standing_height;
    cache.standing_half_height = controller.capsule_half_height;
    cache.crouching_height = crouch_half_height * 2.0;
    cache.radius = controller.capsule_radius;
}

#[cfg(test)]
#[path = "components_tests.rs"]
mod tests;
