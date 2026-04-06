use crate::{
    AccumulatedInput, CharacterController, CharacterControllerPlugin, CharacterControllerState,
    CharacterControllerSystems, CharacterJumped, CharacterLanded, CharacterMotionStats,
    MovementMode, MovementModeChanged, SupportBodyChanged,
    components::{CharacterControllerScratch, PendingLanding},
    systems::{activation::CharacterControllerRuntime, finalize, prepare},
};
use avian3d::prelude::{Collider, LinearVelocity, PhysicsPlugins, RigidBody};
use bevy::{
    app::PostStartup,
    ecs::{message::MessageCursor, schedule::ScheduleLabel},
    prelude::*,
};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct ActivateSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct DeactivateSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct SimulationSchedule;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AfterControllerMovement;

#[derive(Resource, Default, Debug, PartialEq, Eq)]
struct OrderLog(Vec<&'static str>);

fn push_movement_marker(mut log: ResMut<OrderLog>) {
    log.0.push("movement");
}

fn push_after_marker(mut log: ResMut<OrderLog>) {
    log.0.push("after");
}

#[test]
fn plugin_builds_with_custom_schedule_labels_and_ordering_points() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PhysicsPlugins::default()))
        .init_schedule(ActivateSchedule)
        .init_schedule(DeactivateSchedule)
        .init_schedule(SimulationSchedule)
        .init_resource::<OrderLog>()
        .add_plugins(CharacterControllerPlugin::new(
            ActivateSchedule,
            DeactivateSchedule,
            SimulationSchedule,
        ))
        .configure_sets(
            SimulationSchedule,
            CharacterControllerSystems::MovementExecute.before(AfterControllerMovement),
        )
        .add_systems(
            SimulationSchedule,
            (
                push_movement_marker.in_set(CharacterControllerSystems::MovementExecute),
                push_after_marker.in_set(AfterControllerMovement),
            ),
        );

    app.finish();
    app.world_mut().run_schedule(SimulationSchedule);

    assert_eq!(
        app.world().resource::<OrderLog>().0,
        vec!["movement", "after"]
    );
}

#[test]
fn always_on_constructor_activates_runtime_after_startup() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, PhysicsPlugins::default()))
        .add_plugins(CharacterControllerPlugin::always_on(Update));

    app.finish();
    assert!(!app.world().resource::<CharacterControllerRuntime>().0);

    app.world_mut().run_schedule(PostStartup);

    assert!(app.world().resource::<CharacterControllerRuntime>().0);
}

#[test]
fn character_controller_requires_core_runtime_components_on_spawn() {
    let mut world = World::new();
    let entity = world.spawn(CharacterController::default()).id();
    let entity_ref = world.entity(entity);

    assert!(entity_ref.contains::<AccumulatedInput>());
    assert!(entity_ref.contains::<CharacterControllerState>());
    assert!(entity_ref.contains::<CharacterMotionStats>());
    assert!(entity_ref.contains::<Collider>());
    assert!(entity_ref.contains::<LinearVelocity>());
    assert!(entity_ref.contains::<RigidBody>());
}

#[test]
fn prepare_system_initializes_filter_and_orientation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, prepare::setup_new_controllers);

    let rotation = Quat::from_euler(EulerRot::YXZ, 0.8, -0.35, 0.0);
    let entity = app
        .world_mut()
        .spawn((
            CharacterController::default(),
            Transform::from_xyz(0.0, 1.0, 2.0).with_rotation(rotation),
        ))
        .id();

    app.update();

    let controller = app.world().get::<CharacterController>(entity).unwrap();
    let state = app.world().get::<CharacterControllerState>(entity).unwrap();

    assert!(controller.filter.excluded_entities.contains(&entity));
    assert!(state.orientation.dot(rotation).abs() > 0.999_999);
}

#[test]
fn finalize_system_emits_controller_messages() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<CharacterJumped>()
        .add_message::<CharacterLanded>()
        .add_message::<MovementModeChanged>()
        .add_message::<SupportBodyChanged>()
        .add_systems(Update, finalize::emit_controller_messages);

    let entity = app
        .world_mut()
        .spawn((
            {
                CharacterControllerScratch {
                    pending_jump: true,
                    pending_landing: Some(PendingLanding {
                        impact_speed: 14.0,
                        inherited_platform_velocity: Vec3::new(1.0, 0.0, 0.0),
                    }),
                    pending_mode_change: Some((MovementMode::Airborne, MovementMode::Grounded)),
                    pending_support_change: Some((
                        Some(Entity::from_bits(11)),
                        Some(Entity::from_bits(12)),
                    )),
                    ..default()
                }
            },
            CharacterControllerState::default(),
            CharacterMotionStats::default(),
        ))
        .id();

    app.update();

    let mut jumped_cursor = MessageCursor::<CharacterJumped>::default();
    let jumped: Vec<_> = jumped_cursor
        .read(app.world().resource::<Messages<CharacterJumped>>())
        .cloned()
        .collect();
    assert_eq!(jumped.len(), 1);
    assert_eq!(jumped[0].entity, entity);

    let mut landed_cursor = MessageCursor::<CharacterLanded>::default();
    let landed: Vec<_> = landed_cursor
        .read(app.world().resource::<Messages<CharacterLanded>>())
        .cloned()
        .collect();
    assert_eq!(landed.len(), 1);
    assert_eq!(landed[0].impact_speed, 14.0);
    assert_eq!(
        landed[0].inherited_platform_velocity,
        Vec3::new(1.0, 0.0, 0.0)
    );

    let mut mode_cursor = MessageCursor::<MovementModeChanged>::default();
    let mode_changes: Vec<_> = mode_cursor
        .read(app.world().resource::<Messages<MovementModeChanged>>())
        .cloned()
        .collect();
    assert_eq!(mode_changes.len(), 1);
    assert_eq!(mode_changes[0].entity, entity);
    assert_eq!(mode_changes[0].previous, MovementMode::Airborne);
    assert_eq!(mode_changes[0].current, MovementMode::Grounded);

    let mut support_cursor = MessageCursor::<SupportBodyChanged>::default();
    let support_changes: Vec<_> = support_cursor
        .read(app.world().resource::<Messages<SupportBodyChanged>>())
        .cloned()
        .collect();
    assert_eq!(support_changes.len(), 1);
    assert_eq!(support_changes[0].entity, entity);
    assert_eq!(support_changes[0].previous, Some(Entity::from_bits(11)));
    assert_eq!(support_changes[0].current, Some(Entity::from_bits(12)));
}
