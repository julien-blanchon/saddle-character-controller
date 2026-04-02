use crate::{AccumulatedInput, CharacterControllerState, CharacterMotionStats};
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct CharacterControllerRuntime(pub bool);

pub(crate) fn activate_runtime(
    mut runtime: ResMut<CharacterControllerRuntime>,
    mut query: Query<(
        &mut AccumulatedInput,
        &mut CharacterControllerState,
        &mut CharacterMotionStats,
    )>,
) {
    runtime.0 = true;
    for (mut input, mut state, mut stats) in &mut query {
        *input = AccumulatedInput::default();
        state.time_since_grounded = f32::MAX / 4.0;
        state.time_since_wall_kick = f32::MAX / 4.0;
        state.detach_time = f32::MAX / 4.0;
        stats.grounded_time = 0.0;
        stats.airborne_time = 0.0;
    }
}

pub(crate) fn deactivate_runtime(mut runtime: ResMut<CharacterControllerRuntime>) {
    runtime.0 = false;
}

pub(crate) fn runtime_is_active(runtime: Res<CharacterControllerRuntime>) -> bool {
    runtime.0
}
