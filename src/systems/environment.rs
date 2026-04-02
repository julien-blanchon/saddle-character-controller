use crate::{
    CharacterController, CharacterControllerState, CharacterSwimming, WaterLevel, WaterVolume,
};
use avian3d::prelude::*;
use bevy::prelude::*;

pub(crate) fn update_water_state(
    mut controllers: Query<(
        &Transform,
        &CharacterController,
        &mut CharacterControllerState,
        &CollidingEntities,
        Option<&CharacterSwimming>,
    )>,
    waters: Query<(Entity, &Collider, &Position, &Rotation, &WaterVolume)>,
) {
    for (transform, controller, mut state, colliding_entities, can_swim) in &mut controllers {
        if can_swim.is_none() {
            state.water_level = WaterLevel::None;
            state.water_volume = None;
            state.water_speed_multiplier = 1.0;
            state.water_acceleration_multiplier = 1.0;
            state.water_gravity_multiplier = 1.0;
            continue;
        }

        state.water_level = WaterLevel::None;
        state.water_volume = None;
        state.water_speed_multiplier = 1.0;
        state.water_acceleration_multiplier = 1.0;
        state.water_gravity_multiplier = 1.0;
        let center = transform.translation;
        let eye = center
            + Vec3::Y
                * if state.crouching {
                    controller.crouch_view_height
                } else {
                    controller.standing_view_height
                };

        for (water_entity, collider, position, rotation, water) in
            waters.iter_many(colliding_entities.iter())
        {
            let level = if collider.contains_point(position.0, rotation.0, eye) {
                WaterLevel::Head
            } else if collider.contains_point(position.0, rotation.0, center) {
                WaterLevel::Waist
            } else {
                WaterLevel::Feet
            };
            if level > state.water_level {
                state.water_level = level;
                state.water_volume = Some(water_entity);
                state.water_speed_multiplier = water.speed_multiplier;
                state.water_acceleration_multiplier = water.acceleration_multiplier;
                state.water_gravity_multiplier = water.gravity_multiplier;
            } else if level == state.water_level && level > WaterLevel::None {
                state.water_speed_multiplier =
                    state.water_speed_multiplier.min(water.speed_multiplier);
                state.water_acceleration_multiplier = state
                    .water_acceleration_multiplier
                    .min(water.acceleration_multiplier);
                state.water_gravity_multiplier =
                    state.water_gravity_multiplier.min(water.gravity_multiplier);
            }
        }
    }
}

#[cfg(test)]
#[path = "environment_tests.rs"]
mod tests;
