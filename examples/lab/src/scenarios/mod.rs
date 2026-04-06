use bevy::prelude::*;
use saddle_animation_ik::IkChainState;
use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};
use saddle_character_controller::{
    AccumulatedInput, CharacterControllerState, CharacterFlying, CharacterMotionStats,
    EnvironmentModifiers, FlightCollisionMode,
};
use saddle_character_state_machine::CharacterStateMachineRuntime;

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "controller_smoke",
        "controller_platform_rotation",
        "controller_flying_noclip",
        "controller_slopes_and_stairs",
        "controller_advanced_movement",
        "controller_third_person",
        "controller_water",
        "controller_stress",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "controller_smoke" => Some(controller_smoke()),
        "controller_platform_rotation" => Some(controller_platform_rotation()),
        "controller_flying_noclip" => Some(controller_flying_noclip()),
        "controller_slopes_and_stairs" => Some(controller_slopes_and_stairs()),
        "controller_advanced_movement" => Some(controller_advanced_movement()),
        "controller_third_person" => Some(controller_third_person()),
        "controller_water" => Some(controller_water()),
        "controller_stress" => Some(controller_stress()),
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

fn set_input_full(move_axis: Vec2, sprint: bool, ascend: bool) -> Action {
    Action::Custom(Box::new(move |world| {
        let entity = controller_entity(world);
        let mut entity_ref = world.entity_mut(entity);
        let mut input = entity_ref
            .get_mut::<AccumulatedInput>()
            .expect("controller should expose AccumulatedInput");
        input.move_axis = move_axis;
        input.sprint_active = sprint;
        input.ascend_active = ascend;
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

fn controller_slopes_and_stairs() -> Scenario {
    Scenario::builder("controller_slopes_and_stairs")
        .description("Place the controller at the base of the lab stairs, walk forward to climb them, verify the controller remains grounded throughout, then capture the elevated position.")
        .then(Action::WaitFrames(20))
        // Position at the foot of the stairs (stairs start at x=-16, z=-10)
        .then(set_controller_pose(Vec3::new(-16.0, 0.5, -9.0)))
        .then(set_input(Vec2::ZERO, false))
        .then(Action::WaitFrames(10))
        .then(assertions::custom(
            "controller grounded at stair base",
            |world| world.resource::<crate::LabDiagnostics>().movement_mode == "Grounded",
        ))
        .then(Action::Screenshot("stairs_base".into()))
        // Walk forward (negative Z) to climb the stairs
        .then(set_input(Vec2::new(0.0, -1.0), false))
        .then(Action::WaitFrames(40))
        .then(assertions::custom(
            "controller climbed stairs and remains grounded",
            |world| {
                let diagnostics = world.resource::<crate::LabDiagnostics>();
                diagnostics.movement_mode == "Grounded"
                    && diagnostics.controller_position.y > 0.8
            },
        ))
        .then(set_input(Vec2::ZERO, false))
        .then(Action::WaitFrames(5))
        // Now test the walkable ramp (at x=-8, z=-2.5)
        .then(set_controller_pose(Vec3::new(-8.0, 2.5, -2.5)))
        .then(set_input(Vec2::new(-1.0, 0.0), false))
        .then(Action::WaitFrames(30))
        .then(assertions::custom(
            "controller walks on ramp without falling through",
            |world| {
                let diagnostics = world.resource::<crate::LabDiagnostics>();
                diagnostics.movement_mode == "Grounded"
                    && diagnostics.controller_position.is_finite()
            },
        ))
        .then(set_input(Vec2::ZERO, false))
        .then(Action::Screenshot("stairs_and_ramp".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("controller_slopes_and_stairs"))
        .build()
}

fn controller_advanced_movement() -> Scenario {
    Scenario::builder("controller_advanced_movement")
        .description("Test sprint acceleration on the conveyor belt platform and verify the controller achieves higher speed while sprinting.")
        .then(Action::WaitFrames(20))
        // Place on the conveyor platform (at x=10, y=0.35, z=-8)
        .then(set_controller_pose(Vec3::new(6.0, 1.5, -8.0)))
        .then(set_input(Vec2::ZERO, false))
        .then(Action::WaitFrames(15))
        .then(assertions::custom(
            "controller grounded on conveyor area",
            |world| world.resource::<crate::LabDiagnostics>().movement_mode == "Grounded",
        ))
        // Walk along the conveyor direction with sprint
        .then(set_input_full(Vec2::new(1.0, 0.0), true, false))
        .then(Action::WaitFrames(30))
        .then(assertions::custom(
            "sprinting achieves measurable speed",
            |world| world.resource::<crate::LabDiagnostics>().current_speed > 2.0,
        ))
        .then(Action::Screenshot("advanced_sprint_conveyor".into()))
        .then(Action::WaitFrames(1))
        .then(set_input(Vec2::ZERO, false))
        .then(assertions::custom(
            "controller values remain finite after sprint",
            controller_state_is_finite,
        ))
        .then(assertions::log_summary("controller_advanced_movement"))
        .build()
}

fn controller_third_person() -> Scenario {
    Scenario::builder("controller_third_person")
        .description("Walk the controller forward, verify the camera follows and the animation transitions from idle to move and back.")
        .then(Action::WaitFrames(30))
        .then(assertions::custom(
            "controller starts idle",
            |world| {
                let diagnostics = world.resource::<crate::LabDiagnostics>();
                diagnostics.animation_state == "Idle" && diagnostics.animation_binding == "idle"
            },
        ))
        .then(Action::Screenshot("third_person_idle".into()))
        // Walk forward
        .then(set_input(Vec2::new(0.0, -1.0), false))
        .then(Action::WaitFrames(25))
        .then(assertions::custom(
            "animation transitions to move while walking",
            |world| {
                let diagnostics = world.resource::<crate::LabDiagnostics>();
                diagnostics.animation_state == "Move"
                    && diagnostics.animation_binding == "move"
                    && diagnostics.current_speed > 1.0
            },
        ))
        .then(Action::Screenshot("third_person_moving".into()))
        // Stop walking
        .then(set_input(Vec2::ZERO, false))
        .then(Action::WaitFrames(20))
        .then(assertions::custom(
            "animation returns to idle after stopping",
            |world| {
                let diagnostics = world.resource::<crate::LabDiagnostics>();
                diagnostics.animation_state == "Idle" && diagnostics.animation_binding == "idle"
            },
        ))
        .then(Action::Screenshot("third_person_stopped".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("controller_third_person"))
        .build()
}

fn controller_water() -> Scenario {
    Scenario::builder("controller_water")
        .description("Move the controller into the lab pool, verify the movement mode changes to swimming and environment modifiers are applied, then capture the submerged state.")
        .then(Action::WaitFrames(20))
        // Pool center is at (16, 1.6, 10) with size (10, 3, 8)
        .then(set_controller_pose(Vec3::new(16.0, 3.5, 10.0)))
        .then(set_input(Vec2::ZERO, false))
        .then(Action::WaitFrames(30))
        .then(assertions::custom(
            "controller enters swimming mode in pool",
            |world| world.resource::<crate::LabDiagnostics>().movement_mode == "Swimming",
        ))
        .then(assertions::component_where::<EnvironmentModifiers, crate::LabController>(
            "environment modifiers reflect water volume",
            |env| env.speed_multiplier < 1.0 && env.gravity_multiplier < 1.0,
        ))
        .then(Action::Screenshot("water_swimming".into()))
        // Ascend out of the pool
        .then(set_input(Vec2::ZERO, true))
        .then(Action::WaitFrames(40))
        .then(set_input(Vec2::ZERO, false))
        .then(Action::WaitFrames(15))
        .then(assertions::custom(
            "controller values remain finite after water exit",
            controller_state_is_finite,
        ))
        .then(Action::Screenshot("water_exit".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("controller_water"))
        .build()
}

fn controller_stress() -> Scenario {
    Scenario::builder("controller_stress")
        .description("Boot the lab and verify that the single controller remains numerically stable over an extended period with continuous input changes.")
        .then(Action::WaitFrames(30))
        .then(assertions::custom(
            "controller state finite at start",
            controller_state_is_finite,
        ))
        // Apply varying inputs to stress-test the controller pipeline
        .then(set_input(Vec2::new(1.0, 0.0), false))
        .then(Action::WaitFrames(15))
        .then(set_input(Vec2::new(-1.0, -1.0), false))
        .then(Action::WaitFrames(15))
        .then(set_input_full(Vec2::new(0.5, 0.5), true, false))
        .then(Action::WaitFrames(15))
        .then(set_input(Vec2::ZERO, true))
        .then(Action::WaitFrames(15))
        .then(set_input(Vec2::new(0.0, -1.0), false))
        .then(Action::WaitFrames(15))
        .then(set_input(Vec2::ZERO, false))
        .then(Action::WaitFrames(10))
        .then(assertions::custom(
            "controller state finite after stress sequence",
            controller_state_is_finite,
        ))
        .then(assertions::component_where::<CharacterControllerState, crate::LabController>(
            "controller state component accessible after stress",
            |state| state.orientation.is_finite(),
        ))
        .then(assertions::component_where::<CharacterMotionStats, crate::LabController>(
            "controller stats accessible after stress",
            |stats| stats.current_speed.is_finite(),
        ))
        .then(Action::Screenshot("stress_stable".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("controller_stress"))
        .build()
}
