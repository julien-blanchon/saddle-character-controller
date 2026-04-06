use crate::{
    CharacterController, CharacterControllerSystems,
    abilities::swimming::CharacterSwimming,
    state::{EnvironmentDepth, EnvironmentModifiers},
    systems::activation,
};
use avian3d::prelude::*;
use bevy::{
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

/// Generic environment volume that applies movement modifiers when the controller overlaps it.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
#[require(Sensor, Transform, GlobalTransform)]
pub struct EnvironmentVolume {
    pub speed_multiplier: f32,
    pub acceleration_multiplier: f32,
    pub gravity_multiplier: f32,
}

impl Default for EnvironmentVolume {
    fn default() -> Self {
        Self {
            speed_multiplier: 1.0,
            acceleration_multiplier: 1.0,
            gravity_multiplier: 1.0,
        }
    }
}

/// Optional swim-mode volume used by the convenience environment adapter.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component, Debug)]
#[require(Sensor, Transform, GlobalTransform)]
pub struct SwimVolume {
    pub speed_multiplier: f32,
    pub acceleration_multiplier: f32,
    pub gravity_multiplier: f32,
}

impl Default for SwimVolume {
    fn default() -> Self {
        Self {
            speed_multiplier: 1.0,
            acceleration_multiplier: 1.0,
            gravity_multiplier: 1.0,
        }
    }
}

pub struct CharacterControllerEnvironmentPlugin {
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl CharacterControllerEnvironmentPlugin {
    pub fn new(update_schedule: impl ScheduleLabel) -> Self {
        Self {
            update_schedule: update_schedule.intern(),
        }
    }
}

impl Plugin for CharacterControllerEnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EnvironmentVolume>()
            .register_type::<SwimVolume>()
            .add_systems(
                self.update_schedule,
                update_environment_state
                    .in_set(CharacterControllerSystems::Grounding)
                    .run_if(activation::runtime_is_active),
            );
    }
}

pub(crate) fn update_environment_state(
    mut controllers: Query<(
        &Transform,
        &CharacterController,
        &crate::CharacterControllerState,
        &mut EnvironmentModifiers,
        &CollidingEntities,
        Option<&CharacterSwimming>,
    )>,
    swim_volumes: Query<(Entity, &Collider, &Position, &Rotation, &SwimVolume)>,
    env_volumes: Query<
        (Entity, &Collider, &Position, &Rotation, &EnvironmentVolume),
        Without<SwimVolume>,
    >,
) {
    for (transform, controller, state, mut env, colliding_entities, swimming) in &mut controllers {
        env.depth = EnvironmentDepth::None;
        env.active_volume = None;
        env.speed_multiplier = 1.0;
        env.acceleration_multiplier = 1.0;
        env.gravity_multiplier = 1.0;

        let center = transform.translation;
        let eye = center
            + Vec3::Y
                * if state.crouching {
                    controller.crouch_view_height
                } else {
                    controller.standing_view_height
                };

        if swimming.is_some() {
            for (swim_entity, collider, position, rotation, volume) in
                swim_volumes.iter_many(colliding_entities.iter())
            {
                let depth = classify_volume_depth(collider, position.0, rotation.0, center, eye);
                apply_deepest_volume(
                    &mut env,
                    swim_entity,
                    depth,
                    volume.speed_multiplier,
                    volume.acceleration_multiplier,
                    volume.gravity_multiplier,
                );
            }
        }

        for (env_entity, collider, position, rotation, volume) in
            env_volumes.iter_many(colliding_entities.iter())
        {
            let depth = classify_volume_depth(collider, position.0, rotation.0, center, eye);
            apply_deepest_volume(
                &mut env,
                env_entity,
                depth,
                volume.speed_multiplier,
                volume.acceleration_multiplier,
                volume.gravity_multiplier,
            );
        }
    }
}

fn classify_volume_depth(
    collider: &Collider,
    position: Vec3,
    rotation: Quat,
    center: Vec3,
    eye: Vec3,
) -> EnvironmentDepth {
    if collider.contains_point(position, rotation, eye) {
        EnvironmentDepth::Submerged
    } else if collider.contains_point(position, rotation, center) {
        EnvironmentDepth::Medium
    } else {
        EnvironmentDepth::Shallow
    }
}

fn apply_deepest_volume(
    env: &mut EnvironmentModifiers,
    volume_entity: Entity,
    depth: EnvironmentDepth,
    speed_multiplier: f32,
    acceleration_multiplier: f32,
    gravity_multiplier: f32,
) {
    if depth > env.depth {
        env.depth = depth;
        env.active_volume = Some(volume_entity);
        env.speed_multiplier = speed_multiplier;
        env.acceleration_multiplier = acceleration_multiplier;
        env.gravity_multiplier = gravity_multiplier;
    } else if depth == env.depth && depth > EnvironmentDepth::None {
        env.speed_multiplier = env.speed_multiplier.min(speed_multiplier);
        env.acceleration_multiplier = env.acceleration_multiplier.min(acceleration_multiplier);
        env.gravity_multiplier = env.gravity_multiplier.min(gravity_multiplier);
    }
}

#[cfg(test)]
#[path = "environment_tests.rs"]
mod tests;
