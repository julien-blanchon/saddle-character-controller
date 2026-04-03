use bevy::prelude::*;
use saddle_animation_ik::IkChainState;
use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};
use saddle_character_controller::{
    AccumulatedInput, CharacterControllerState, CharacterFlying, CharacterMotionStats,
    FlightCollisionMode,
};
use saddle_character_state_machine::CharacterStateMachineRuntime;

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "controller_smoke",
        "controller_platform_rotation",
        "controller_flying_noclip",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "controller_smoke" => Some(controller_smoke()),
        "controller_platform_rotation" => Some(controller_platform_rotation()),
        "controller_flying_noclip" => Some(controller_flying_noclip()),
        _ => None,
    }
}

fn controller_entity(world: &mut World) -> Entity {
    let mut query = world.query_filtered::<Entity, With<crate::LabController>>();
    query
        .single(world)
        .expect("lab controller should exist for e2e")
}

fn set_controller_pose(translation: Vec3) -> Action {
    Action::Custom(Box::new(move |world| {
        let entity = controller_entity(world);
        let mut entity_ref = world.entity_mut(entity);
        let mut transform = entity_ref
            .get_mut::<Transform>()
            .expect("controller should have transform");
        transform.translation = translation;
    }))
}

fn set_flight(enabled: bool, collision_mode: FlightCollisionMode) -> Action {
    Action::Custom(Box::new(move |world| {
        let entity = controller_entity(world);
        let mut entity_ref = world.entity_mut(entity);
        let mut flight = entity_ref
            .get_mut::<CharacterFlying>()
            .expect("controller should expose CharacterFlying");
        flight.enabled = enabled;
        flight.collision_mode = collision_mode;
        flight.speed = 10.0;
    }))
}

fn set_input(move_axis: Vec2, ascend_active: bool) -> Action {
    Action::Custom(Box::new(move |world| {
        let entity = controller_entity(world);
        let mut entity_ref = world.entity_mut(entity);
        let mut input = entity_ref
            .get_mut::<AccumulatedInput>()
            .expect("controller should expose AccumulatedInput");
        input.move_axis = move_axis;
        input.ascend_active = ascend_active;
        input.sprint_active = false;
        input.crouch_active = false;
    }))
}

fn controller_state_is_finite(world: &World) -> bool {
    let diagnostics = world.resource::<crate::LabDiagnostics>();
    diagnostics.controller_position.is_finite()
        && diagnostics.support_angular_speed.is_finite()
        && diagnostics.current_speed.is_finite()
        && diagnostics.look_error.is_finite()
}

fn controller_smoke() -> Scenario {
    Scenario::builder("controller_smoke")
        .description("Boot the crate-local controller lab, verify the runtime state remains numerically stable, and capture the baseline showcase scene.")
        .then(Action::WaitFrames(45))
        .then(assertions::resource_exists::<crate::LabDiagnostics>(
            "controller diagnostics resource exists",
        ))
        .then(assertions::component_where::<CharacterControllerState, crate::LabController>(
            "controller state exists",
            |_| true,
        ))
        .then(assertions::component_where::<CharacterMotionStats, crate::LabController>(
            "controller stats exist",
            |_| true,
        ))
        .then(assertions::component_where::<
            CharacterStateMachineRuntime,
            crate::LabController,
        >("state-machine runtime exists", |_| true))
        .then(assertions::component_where::<IkChainState, crate::LabLookController>(
            "look IK chain state exists",
            |_| true,
        ))
        .then(assertions::custom(
            "controller runtime values remain finite",
            controller_state_is_finite,
        ))
        .then(assertions::custom(
            "pane mirrors grounded baseline state",
            |world| {
                let pane = world.resource::<crate::LabPane>();
                pane.movement_mode == "Grounded"
                    && pane.grounded_on == "Lab Ground"
                    && pane.animation_state == "Idle"
                    && pane.animation_binding == "idle"
                    && pane.look_error.is_finite()
            },
        ))
        .then(Action::Screenshot("controller_smoke".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("controller_smoke"))
        .build()
}

fn controller_platform_rotation() -> Scenario {
    Scenario::builder("controller_platform_rotation")
        .description("Place the controller on the rotating platform, verify support angular velocity is inherited and the controller grounds on the named platform, then capture the result.")
        .then(Action::WaitFrames(20))
        .then(set_controller_pose(Vec3::new(-6.0, 2.5, 5.0)))
        .then(set_input(Vec2::ZERO, false))
        .then(Action::WaitFrames(35))
        .then(assertions::custom(
            "controller grounds on the rotating platform",
            |world| {
                world.resource::<crate::LabDiagnostics>().grounded_on == "Lab Platform A"
            },
        ))
        .then(assertions::custom(
            "support angular velocity is non-zero on the rotating platform",
            |world| world.resource::<crate::LabDiagnostics>().support_angular_speed > 0.1,
        ))
        .then(assertions::custom(
            "pane shows rotating-platform support diagnostics",
            |world| {
                let pane = world.resource::<crate::LabPane>();
                pane.grounded_on == "Lab Platform A"
                    && pane.support_angular_speed > 0.1
                    && pane.animation_binding == "idle"
                    && pane.look_error.is_finite()
            },
        ))
        .then(Action::Screenshot("platform_rotation".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("controller_platform_rotation"))
        .build()
}

fn controller_flying_noclip() -> Scenario {
    Scenario::builder("controller_flying_noclip")
        .description("Enable the new flying no-clip mode, start the controller inside the mantle block, and verify it remains in flying mode while moving upward through solid geometry.")
        .then(Action::WaitFrames(20))
        .then(set_flight(true, FlightCollisionMode::NoClip))
        .then(set_controller_pose(Vec3::new(0.0, 1.2, -2.0)))
        .then(set_input(Vec2::ZERO, true))
        .then(Action::WaitFrames(12))
        .then(assertions::custom(
            "controller enters flying mode",
            |world| world.resource::<crate::LabDiagnostics>().movement_mode == "Flying",
        ))
        .then(assertions::custom(
            "controller remains centered in the mantle block while rising",
            |world| {
                let diagnostics = world.resource::<crate::LabDiagnostics>();
                (diagnostics.controller_position.x.abs() < 0.25)
                    && ((diagnostics.controller_position.z + 2.0).abs() < 0.25)
                    && diagnostics.controller_position.y > 1.45
            },
        ))
        .then(assertions::custom(
            "pane mirrors flying no-clip state",
            |world| {
                let pane = world.resource::<crate::LabPane>();
                pane.flight_enabled
                    && pane.flight_noclip
                    && pane.movement_mode == "Flying"
                    && pane.current_speed > 0.0
                    && pane.animation_state == "Airborne"
                    && pane.animation_binding == "airborne"
                    && pane.look_error.is_finite()
            },
        ))
        .then(Action::Screenshot("flying_noclip".into()))
        .then(Action::WaitFrames(1))
        .then(set_input(Vec2::ZERO, false))
        .then(assertions::log_summary("controller_flying_noclip"))
        .build()
}
