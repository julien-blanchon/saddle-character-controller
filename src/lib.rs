#[cfg(feature = "enhanced-input")]
pub mod adapters;
pub mod convenience;

mod components;
mod helpers;
mod intent;
mod messages;
mod state;
mod surfaces;
mod systems;

pub use components::{
    CharacterController, CharacterDash, CharacterFlying, CharacterGravity, CharacterLook,
    CharacterMantle, CharacterMotionStats, CharacterPush, CharacterWallKick, ExternalMotion,
    FlightCollisionMode,
};
pub use intent::AccumulatedInput;
pub use messages::{CharacterJumped, CharacterLanded, MovementModeChanged, SupportBodyChanged};
pub use state::{
    CharacterControllerState, ControllerMode, DashState, EnvironmentDepth, EnvironmentModifiers,
    GroundContact, MantleState, MovementMode, WaterLevel,
};
pub use surfaces::{
    CharacterControllerDebugDraw, MovementSurface, SupportRotationPolicy, SupportVelocityPolicy,
};

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterControllerSystems {
    ReadInput,
    PreMovement,
    Grounding,
    Movement,
    PostMovement,
    Presentation,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct CharacterControllerPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl CharacterControllerPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CharacterControllerDebugDraw>()
            .init_resource::<systems::activation::CharacterControllerRuntime>()
            .add_message::<CharacterJumped>()
            .add_message::<CharacterLanded>()
            .add_message::<MovementModeChanged>()
            .add_message::<SupportBodyChanged>()
            .register_type::<AccumulatedInput>()
            .register_type::<CharacterController>()
            .register_type::<CharacterControllerState>()
            .register_type::<CharacterDash>()
            .register_type::<CharacterFlying>()
            .register_type::<CharacterGravity>()
            .register_type::<CharacterLook>()
            .register_type::<CharacterMantle>()
            .register_type::<CharacterMotionStats>()
            .register_type::<CharacterPush>()
            .register_type::<CharacterWallKick>()
            .register_type::<ControllerMode>()
            .register_type::<DashState>()
            .register_type::<EnvironmentDepth>()
            .register_type::<EnvironmentModifiers>()
            .register_type::<ExternalMotion>()
            .register_type::<FlightCollisionMode>()
            .register_type::<GroundContact>()
            .register_type::<MantleState>()
            .register_type::<MovementMode>()
            .register_type::<CharacterControllerDebugDraw>()
            .register_type::<MovementSurface>()
            .register_type::<SupportRotationPolicy>()
            .register_type::<SupportVelocityPolicy>()
            .add_systems(
                self.activate_schedule,
                systems::activation::activate_runtime,
            )
            .add_systems(
                self.deactivate_schedule,
                systems::activation::deactivate_runtime,
            )
            .configure_sets(
                self.update_schedule,
                (
                    CharacterControllerSystems::ReadInput,
                    CharacterControllerSystems::PreMovement,
                    CharacterControllerSystems::Grounding,
                    CharacterControllerSystems::Movement,
                    CharacterControllerSystems::PostMovement,
                    CharacterControllerSystems::Presentation,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                (
                    intent::tick_input_buffers.in_set(CharacterControllerSystems::ReadInput),
                    systems::prepare::setup_new_controllers
                        .in_set(CharacterControllerSystems::PreMovement),
                    systems::prepare::refresh_character_shapes
                        .in_set(CharacterControllerSystems::PreMovement),
                    systems::movement::run_controllers.in_set(CharacterControllerSystems::Movement),
                    systems::finalize::apply_push_forces
                        .in_set(CharacterControllerSystems::PostMovement),
                    systems::finalize::sync_active_collider
                        .in_set(CharacterControllerSystems::PostMovement),
                    systems::finalize::emit_controller_messages
                        .in_set(CharacterControllerSystems::PostMovement),
                    systems::finalize::clear_per_tick_input
                        .in_set(CharacterControllerSystems::Presentation),
                )
                    .run_if(systems::activation::runtime_is_active),
            )
            .configure_sets(PostUpdate, CharacterControllerSystems::Presentation)
            .add_systems(
                PostUpdate,
                systems::presentation::debug_draw
                    .in_set(CharacterControllerSystems::Presentation)
                    .run_if(systems::activation::runtime_is_active),
            );
    }
}

#[cfg(test)]
#[path = "plugin_tests.rs"]
mod plugin_tests;
