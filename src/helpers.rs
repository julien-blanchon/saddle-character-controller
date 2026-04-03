use crate::{
    CharacterController,
    state::GroundContact,
    surfaces::{SupportRotationPolicy, SupportVelocityPolicy},
};
use bevy::prelude::*;

pub(crate) const MIN_SPEED_EPSILON: f32 = 0.001;
pub(crate) const MAX_REASONABLE_SPEED: f32 = 10_000.0;

pub(crate) fn safe_dt(dt: f32) -> f32 {
    if dt.is_finite() && dt > 0.0 { dt } else { 0.0 }
}

pub(crate) fn horizontal(vec: Vec3) -> Vec3 {
    Vec3::new(vec.x, 0.0, vec.z)
}

pub(crate) fn horizontal_speed(vec: Vec3) -> f32 {
    horizontal(vec).length()
}

pub(crate) fn jump_velocity_for_height(gravity: f32, jump_height: f32) -> f32 {
    if gravity <= 0.0 || jump_height <= 0.0 {
        0.0
    } else {
        (2.0 * gravity * jump_height).sqrt()
    }
}

pub(crate) fn quake_acceleration_delta(
    velocity: Vec3,
    wish_dir: Vec3,
    wish_speed: f32,
    acceleration_hz: f32,
    dt: f32,
    speed_cap: Option<f32>,
) -> Vec3 {
    let dt = safe_dt(dt);
    if dt == 0.0 {
        return Vec3::ZERO;
    }
    let Ok(wish_dir) = Dir3::new(wish_dir) else {
        return Vec3::ZERO;
    };
    let capped_speed = speed_cap.map_or(wish_speed, |cap| wish_speed.min(cap));
    let current_speed = velocity.dot(*wish_dir);
    let add_speed = capped_speed - current_speed;
    if add_speed <= 0.0 {
        return Vec3::ZERO;
    }
    let accel_speed = (wish_speed * acceleration_hz * dt).min(add_speed);
    *wish_dir * accel_speed
}

pub(crate) fn apply_ground_friction(
    velocity: Vec3,
    stop_speed: f32,
    friction_hz: f32,
    surface_traction: f32,
    dt: f32,
) -> Vec3 {
    let dt = safe_dt(dt);
    if dt == 0.0 {
        return velocity;
    }
    let speed = velocity.length();
    if speed < MIN_SPEED_EPSILON {
        return velocity;
    }
    let control = speed.max(stop_speed.max(0.0));
    let drop = control * friction_hz.max(0.0) * surface_traction.max(0.0) * dt;
    let new_speed = (speed - drop).max(0.0);
    if speed <= MIN_SPEED_EPSILON {
        Vec3::ZERO
    } else {
        velocity * (new_speed / speed)
    }
}

pub(crate) fn is_walkable(normal: Vec3, min_walk_angle: f32, slide_only: bool) -> bool {
    !slide_only && normal.y >= min_walk_angle.cos()
}

pub(crate) fn clamp_velocity(velocity: Vec3, max_speed: f32) -> Vec3 {
    if !velocity.is_finite() {
        return Vec3::ZERO;
    }
    if max_speed.is_finite() && max_speed > 0.0 {
        velocity.clamp_length_max(max_speed.min(MAX_REASONABLE_SPEED))
    } else {
        velocity.clamp_length_max(MAX_REASONABLE_SPEED)
    }
}

pub(crate) fn travel_towards(delta: Vec3, speed: f32, dt: f32) -> Vec3 {
    let dt = safe_dt(dt);
    if dt == 0.0 || speed <= 0.0 {
        Vec3::ZERO
    } else {
        delta.clamp_length_max(speed * dt)
    }
}

pub(crate) fn wish_velocity_from_input(orientation: Quat, move_axis: Vec2, speed: f32) -> Vec3 {
    let mut forward = orientation * Vec3::NEG_Z;
    forward.y = 0.0;
    forward = forward.normalize_or_zero();
    let mut right = orientation * Vec3::X;
    right.y = 0.0;
    right = right.normalize_or_zero();
    let raw = move_axis.y * forward + move_axis.x * right;
    raw.normalize_or_zero() * speed.max(0.0)
}

pub(crate) fn wish_velocity_3d(orientation: Quat, move_axis: Vec2, speed: f32) -> Vec3 {
    let forward = orientation * Vec3::NEG_Z;
    let right = orientation * Vec3::X;
    let raw = move_axis.y * forward + move_axis.x * right;
    raw.normalize_or_zero() * speed.max(0.0)
}

pub(crate) fn inherited_support_velocity(
    policy: SupportVelocityPolicy,
    support_velocity: Vec3,
) -> Vec3 {
    match policy {
        SupportVelocityPolicy::None => Vec3::ZERO,
        SupportVelocityPolicy::Horizontal => horizontal(support_velocity),
        SupportVelocityPolicy::Full => support_velocity,
    }
}

pub(crate) fn ground_probe_distance(
    controller: &CharacterController,
    support_velocity: Vec3,
    dt: f32,
) -> f32 {
    if support_velocity.y < 0.0 {
        controller.ground_distance - support_velocity.y * dt
    } else {
        controller.ground_distance
    }
}

pub(crate) fn classify_mode(
    can_fly: bool,
    can_swim: bool,
    water_level: crate::WaterLevel,
    ground: Option<GroundContact>,
    mantle_active: bool,
) -> crate::MovementMode {
    if mantle_active {
        crate::MovementMode::Mantling
    } else if can_fly {
        crate::MovementMode::Flying
    } else if can_swim && water_level > crate::WaterLevel::Feet {
        crate::MovementMode::Swimming
    } else if let Some(ground) = ground {
        if ground.walkable {
            crate::MovementMode::Grounded
        } else {
            crate::MovementMode::Sliding
        }
    } else {
        crate::MovementMode::Airborne
    }
}

pub(crate) fn inherited_support_rotation(
    policy: SupportRotationPolicy,
    support_angular_velocity: Vec3,
) -> Vec3 {
    match policy {
        SupportRotationPolicy::None => Vec3::ZERO,
        SupportRotationPolicy::YawOnly => Vec3::Y * support_angular_velocity.y,
    }
}

#[allow(dead_code)]
pub(crate) fn corner_correction_delta(offset: Vec3, max_distance: f32) -> Option<Vec3> {
    let horizontal = horizontal(offset);
    if horizontal.length() <= max_distance.max(0.0) {
        Some(horizontal)
    } else {
        None
    }
}

pub(crate) fn step_height_allowed(step_height: f32, step_size: f32) -> bool {
    step_height <= step_size
}

pub(crate) fn support_detach_velocity(
    support_velocity: Vec3,
    detach_time: f32,
    detach_grace: f32,
    policy: SupportVelocityPolicy,
) -> Vec3 {
    if detach_time <= detach_grace.max(0.0) {
        inherited_support_velocity(policy, support_velocity)
    } else {
        Vec3::ZERO
    }
}

pub(crate) fn ages_recently(age: Option<f32>, window: f32) -> bool {
    age.is_some_and(|value| value <= window.max(0.0))
}

pub(crate) fn expire_old(age: &mut Option<f32>, window: f32) {
    if age.is_some_and(|value| value > window.max(0.0)) {
        *age = None;
    }
}

#[cfg(test)]
#[path = "helpers_tests.rs"]
mod tests;
