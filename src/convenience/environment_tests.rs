use avian3d::prelude::*;
use bevy::{ecs::entity::EntityHashSet, prelude::*};

use crate::{
    CharacterController, CharacterControllerState,
    convenience::environment::{CharacterSwimming, SwimVolume},
    state::{EnvironmentDepth, EnvironmentModifiers},
};

#[test]
fn swim_state_uses_deepest_overlap_and_applies_volume_multipliers() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, super::update_environment_state);

    let shallow = app
        .world_mut()
        .spawn((
            Collider::cuboid(4.0, 0.6, 4.0),
            Position(Vec3::new(0.0, 0.25, 0.0)),
            Rotation::default(),
            SwimVolume {
                speed_multiplier: 0.8,
                acceleration_multiplier: 0.9,
                gravity_multiplier: 0.7,
            },
        ))
        .id();
    let deep = app
        .world_mut()
        .spawn((
            Collider::cuboid(4.0, 3.0, 4.0),
            Position(Vec3::new(0.0, 1.5, 0.0)),
            Rotation::default(),
            SwimVolume {
                speed_multiplier: 0.45,
                acceleration_multiplier: 0.6,
                gravity_multiplier: 0.35,
            },
        ))
        .id();

    let mut overlaps = EntityHashSet::default();
    overlaps.insert(shallow);
    overlaps.insert(deep);

    let controller = app
        .world_mut()
        .spawn((
            CharacterController::default(),
            CharacterControllerState::default(),
            EnvironmentModifiers::default(),
            CharacterSwimming::default(),
            Transform::from_xyz(0.0, 0.5, 0.0),
            CollidingEntities(overlaps),
        ))
        .id();

    app.update();

    let env = app
        .world()
        .get::<EnvironmentModifiers>(controller)
        .expect("environment modifiers should exist");
    assert_eq!(env.depth, EnvironmentDepth::Submerged);
    assert_eq!(env.active_volume, Some(deep));
    assert!((env.speed_multiplier - 0.45).abs() < 0.0001);
    assert!((env.acceleration_multiplier - 0.6).abs() < 0.0001);
    assert!((env.gravity_multiplier - 0.35).abs() < 0.0001);
}

#[test]
fn swim_state_resets_when_swimming_is_disabled() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, super::update_environment_state);

    let swim_volume = app
        .world_mut()
        .spawn((
            Collider::cuboid(4.0, 3.0, 4.0),
            Position(Vec3::new(0.0, 1.5, 0.0)),
            Rotation::default(),
            SwimVolume {
                speed_multiplier: 0.2,
                acceleration_multiplier: 0.3,
                gravity_multiplier: 0.4,
            },
        ))
        .id();

    let mut overlaps = EntityHashSet::default();
    overlaps.insert(swim_volume);

    let controller = app
        .world_mut()
        .spawn((
            CharacterController::default(),
            CharacterControllerState::default(),
            EnvironmentModifiers {
                depth: EnvironmentDepth::Submerged,
                active_volume: Some(swim_volume),
                speed_multiplier: 0.2,
                acceleration_multiplier: 0.3,
                gravity_multiplier: 0.4,
            },
            Transform::from_xyz(0.0, 0.5, 0.0),
            CollidingEntities(overlaps),
        ))
        .id();

    app.update();

    let env = app
        .world()
        .get::<EnvironmentModifiers>(controller)
        .expect("environment modifiers should exist");
    assert_eq!(env.depth, EnvironmentDepth::None);
    assert_eq!(env.active_volume, None);
    assert_eq!(env.speed_multiplier, 1.0);
    assert_eq!(env.acceleration_multiplier, 1.0);
    assert_eq!(env.gravity_multiplier, 1.0);
}
