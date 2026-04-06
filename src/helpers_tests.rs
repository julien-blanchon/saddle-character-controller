use super::*;

#[test]
fn quake_accel_stationary_to_wish_speed() {
    let delta = quake_acceleration_delta(Vec3::ZERO, Vec3::X, 10.0, 8.0, 0.25, None);
    assert_eq!(delta, Vec3::X * 10.0);
}

#[test]
fn quake_accel_no_overshoot() {
    let delta = quake_acceleration_delta(Vec3::X * 9.0, Vec3::X, 10.0, 100.0, 1.0, None);
    assert_eq!(delta, Vec3::X);
}

#[test]
fn quake_accel_air_strafe_can_gain_total_speed() {
    let delta = quake_acceleration_delta(Vec3::X * 10.0, Vec3::Z, 1.5, 12.0, 0.1, Some(0.76));
    let next = Vec3::X * 10.0 + delta;
    assert!(next.length() > 10.0);
}

#[test]
fn friction_reduces_speed_without_reversing() {
    let next = apply_ground_friction(Vec3::X * 10.0, 1.0, 8.0, 1.0, 0.1);
    assert!(next.x > 0.0);
    assert!(next.length() < 10.0);
}

#[test]
fn friction_before_accel_changes_result() {
    let friction_then_accel = apply_ground_friction(Vec3::X * 5.0, 1.0, 8.0, 1.0, 0.1)
        + quake_acceleration_delta(Vec3::X * 5.0, Vec3::X, 10.0, 8.0, 0.1, None);
    let accel_then_friction = apply_ground_friction(
        Vec3::X * 5.0 + quake_acceleration_delta(Vec3::X * 5.0, Vec3::X, 10.0, 8.0, 0.1, None),
        1.0,
        8.0,
        1.0,
        0.1,
    );
    assert_ne!(friction_then_accel, accel_then_friction);
}

#[test]
fn jump_velocity_matches_height_config() {
    let velocity = jump_velocity_for_height(29.0, 1.8);
    let reached_height = velocity * velocity / (2.0 * 29.0);
    assert!((reached_height - 1.8).abs() < 0.001);
}

#[test]
fn coyote_time_accepts_recent_ground_loss() {
    assert!(ages_recently(Some(0.09), 0.1));
    assert!(!ages_recently(Some(0.11), 0.1));
}

#[test]
fn jump_buffer_fires_on_landing() {
    assert!(ages_recently(Some(0.05), 0.15));
}

#[test]
fn jump_buffer_expires_when_too_old() {
    let mut age = Some(0.2);
    expire_old(&mut age, 0.15);
    assert!(age.is_none());
}

#[test]
fn traction_threshold_flips_once() {
    let min_walk_angle = 40.0_f32.to_radians();
    let just_walkable = Vec3::new(0.0, min_walk_angle.cos(), 0.0);
    let just_steep = Vec3::new(0.0, min_walk_angle.cos() - 0.001, 0.0);
    assert!(is_walkable(just_walkable, min_walk_angle, false));
    assert!(!is_walkable(just_steep, min_walk_angle, false));
}

#[test]
fn step_up_rejects_too_tall_obstacle() {
    assert!(!step_height_allowed(0.8, 0.7));
}

#[test]
fn support_velocity_is_applied_on_platform() {
    let inherited =
        inherited_support_velocity(SupportVelocityPolicy::Horizontal, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(inherited, Vec3::new(1.0, 0.0, 3.0));
}

#[test]
fn support_rotation_can_be_limited_to_yaw() {
    let inherited =
        inherited_support_rotation(SupportRotationPolicy::YawOnly, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(inherited, Vec3::new(0.0, 2.0, 0.0));
}

#[test]
fn support_velocity_detach_policy_respected() {
    let inherited = support_detach_velocity(
        Vec3::new(2.0, 0.0, 0.0),
        0.05,
        0.1,
        SupportVelocityPolicy::Full,
    );
    assert_eq!(inherited, Vec3::new(2.0, 0.0, 0.0));
    let lost = support_detach_velocity(
        Vec3::new(2.0, 0.0, 0.0),
        0.2,
        0.1,
        SupportVelocityPolicy::Full,
    );
    assert_eq!(lost, Vec3::ZERO);
}

#[test]
fn corner_correction_only_applies_within_limit() {
    assert!(corner_correction_delta(Vec3::new(0.1, 0.0, 0.1), 0.2).is_some());
    assert!(corner_correction_delta(Vec3::new(1.0, 0.0, 0.0), 0.2).is_none());
}

#[test]
fn zero_dt_is_safe() {
    assert_eq!(
        quake_acceleration_delta(Vec3::X, Vec3::Y, 10.0, 8.0, 0.0, None),
        Vec3::ZERO
    );
    assert_eq!(safe_dt(0.0), 0.0);
}

#[test]
fn extreme_speed_is_clamped_or_rejected_safely() {
    let clamped = clamp_velocity(Vec3::splat(100_000.0), 50.0);
    assert!(clamped.length() <= 50.001);
}

#[test]
fn travel_towards_respects_speed_limit() {
    let travel = travel_towards(Vec3::new(0.0, 0.0, 10.0), 4.0, 0.25);
    assert_eq!(travel, Vec3::new(0.0, 0.0, 1.0));
}

#[test]
fn flying_mode_takes_priority_over_ground_and_water() {
    let ground = Some(GroundContact {
        entity: Entity::PLACEHOLDER,
        point: Vec3::ZERO,
        normal: Vec3::Y,
        distance: 0.0,
        walkable: true,
    });

    assert_eq!(
        classify_mode(
            true,
            true,
            crate::EnvironmentDepth::Submerged,
            ground,
            false,
            false
        ),
        crate::MovementMode::FLYING
    );
}
