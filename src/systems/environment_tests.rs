use avian3d::prelude::*;
use bevy::{ecs::entity::EntityHashSet, prelude::*};

use crate::{
    CharacterController, CharacterControllerState, CharacterSwimming, WaterLevel, WaterVolume,
};

#[test]
fn water_state_uses_deepest_overlap_and_applies_volume_multipliers() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, super::update_water_state);

    let shallow = app
        .world_mut()
        .spawn((
            Collider::cuboid(4.0, 0.6, 4.0),
            Position(Vec3::new(0.0, 0.25, 0.0)),
            Rotation::default(),
            WaterVolume {
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
            WaterVolume {
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
            CharacterSwimming::default(),
            Transform::from_xyz(0.0, 0.5, 0.0),
            CollidingEntities(overlaps),
        ))
        .id();

    app.update();

    let state = app
        .world()
        .get::<CharacterControllerState>(controller)
        .expect("controller state should exist");
    assert_eq!(state.water_level, WaterLevel::Head);
    assert_eq!(state.water_volume, Some(deep));
    assert!((state.water_speed_multiplier - 0.45).abs() < 0.0001);
    assert!((state.water_acceleration_multiplier - 0.6).abs() < 0.0001);
    assert!((state.water_gravity_multiplier - 0.35).abs() < 0.0001);
}

#[test]
fn water_state_resets_when_swimming_is_disabled() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, super::update_water_state);

    let water = app
        .world_mut()
        .spawn((
            Collider::cuboid(4.0, 3.0, 4.0),
            Position(Vec3::new(0.0, 1.5, 0.0)),
            Rotation::default(),
            WaterVolume {
                speed_multiplier: 0.2,
                acceleration_multiplier: 0.3,
                gravity_multiplier: 0.4,
            },
        ))
        .id();

    let mut overlaps = EntityHashSet::default();
    overlaps.insert(water);

    let controller = app
        .world_mut()
        .spawn((
            CharacterController::default(),
            CharacterControllerState {
                water_level: WaterLevel::Head,
                water_volume: Some(water),
                water_speed_multiplier: 0.2,
                water_acceleration_multiplier: 0.3,
                water_gravity_multiplier: 0.4,
                ..default()
            },
            Transform::from_xyz(0.0, 0.5, 0.0),
            CollidingEntities(overlaps),
        ))
        .id();

    app.update();

    let state = app
        .world()
        .get::<CharacterControllerState>(controller)
        .expect("controller state should exist");
    assert_eq!(state.water_level, WaterLevel::None);
    assert_eq!(state.water_volume, None);
    assert_eq!(state.water_speed_multiplier, 1.0);
    assert_eq!(state.water_acceleration_multiplier, 1.0);
    assert_eq!(state.water_gravity_multiplier, 1.0);
}
