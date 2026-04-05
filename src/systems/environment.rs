use crate::{
    CharacterController, CharacterSwimming,
    state::{EnvironmentDepth, EnvironmentModifiers},
    surfaces::{EnvironmentVolume, WaterVolume},
};
use avian3d::prelude::*;
use bevy::prelude::*;

pub(crate) fn update_environment_state(
    mut controllers: Query<(
        &Transform,
        &CharacterController,
        &crate::CharacterControllerState,
        &mut EnvironmentModifiers,
        &CollidingEntities,
        Option<&CharacterSwimming>,
    )>,
    water_volumes: Query<(Entity, &Collider, &Position, &Rotation, &WaterVolume)>,
    env_volumes: Query<
        (Entity, &Collider, &Position, &Rotation, &EnvironmentVolume),
        Without<WaterVolume>,
    >,
) {
    for (transform, controller, state, mut env, colliding_entities, can_swim) in &mut controllers {
        // Reset modifiers each tick.
        env.depth = EnvironmentDepth::None;
        env.active_volume = None;
        env.speed_multiplier = 1.0;
        env.acceleration_multiplier = 1.0;
        env.gravity_multiplier = 1.0;

        if can_swim.is_none() {
            // Without CharacterSwimming, the controller ignores swim volumes.
            // Generic EnvironmentVolumes still apply their multipliers.
        }

        let center = transform.translation;
        let eye = center
            + Vec3::Y
                * if state.crouching {
                    controller.crouch_view_height
                } else {
                    controller.standing_view_height
                };

        // Process WaterVolume overlaps.
        for (water_entity, collider, position, rotation, water) in
            water_volumes.iter_many(colliding_entities.iter())
        {
            if can_swim.is_none() {
                continue;
            }
            let level = if collider.contains_point(position.0, rotation.0, eye) {
                EnvironmentDepth::Submerged
            } else if collider.contains_point(position.0, rotation.0, center) {
                EnvironmentDepth::Medium
            } else {
                EnvironmentDepth::Shallow
            };
            if level > env.depth {
                env.depth = level;
                env.active_volume = Some(water_entity);
                env.speed_multiplier = water.speed_multiplier;
                env.acceleration_multiplier = water.acceleration_multiplier;
                env.gravity_multiplier = water.gravity_multiplier;
            } else if level == env.depth && level > EnvironmentDepth::None {
                env.speed_multiplier = env.speed_multiplier.min(water.speed_multiplier);
                env.acceleration_multiplier = env
                    .acceleration_multiplier
                    .min(water.acceleration_multiplier);
                env.gravity_multiplier = env.gravity_multiplier.min(water.gravity_multiplier);
            }
        }

        // Process generic EnvironmentVolume overlaps.
        for (env_entity, collider, position, rotation, volume) in
            env_volumes.iter_many(colliding_entities.iter())
        {
            let level = if collider.contains_point(position.0, rotation.0, eye) {
                EnvironmentDepth::Submerged
            } else if collider.contains_point(position.0, rotation.0, center) {
                EnvironmentDepth::Medium
            } else {
                EnvironmentDepth::Shallow
            };
            // For swim volumes, require CharacterSwimming.
            if volume.swim_volume && can_swim.is_none() {
                continue;
            }
            if level > env.depth {
                env.depth = level;
                env.active_volume = Some(env_entity);
                env.speed_multiplier = volume.speed_multiplier;
                env.acceleration_multiplier = volume.acceleration_multiplier;
                env.gravity_multiplier = volume.gravity_multiplier;
            } else if level == env.depth && level > EnvironmentDepth::None {
                env.speed_multiplier = env.speed_multiplier.min(volume.speed_multiplier);
                env.acceleration_multiplier = env
                    .acceleration_multiplier
                    .min(volume.acceleration_multiplier);
                env.gravity_multiplier = env.gravity_multiplier.min(volume.gravity_multiplier);
            }
        }
    }
}

#[cfg(test)]
#[path = "environment_tests.rs"]
mod tests;
