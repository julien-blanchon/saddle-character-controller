use avian3d::prelude::Rotation;

use super::*;

#[test]
fn crouch_profile_is_shorter_than_standing_profile() {
    let controller = CharacterController::default();
    let mut cache = CharacterColliderCache::default();
    refresh_collider_cache(&mut cache, &controller);

    let standing_aabb = cache
        .standing_collider
        .aabb(Vec3::ZERO, Rotation::default());
    let crouching_aabb = cache
        .crouching_collider
        .aabb(Vec3::ZERO, Rotation::default());

    let standing_height = standing_aabb.max.y - standing_aabb.min.y;
    let crouching_height = crouching_aabb.max.y - crouching_aabb.min.y;

    assert!(crouching_height < standing_height);
    assert!(cache.crouching_height < cache.standing_height);
}
