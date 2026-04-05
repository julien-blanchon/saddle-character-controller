use crate::{CharacterController, CharacterControllerState, CharacterLook};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{Cancel as InputCancel, Complete, Fire, InputAction, Start};

#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component, Debug)]
pub struct AccumulatedInput {
    pub move_axis: Vec2,
    pub look_axis: Vec2,
    pub jump_pressed_for: Option<f32>,
    pub jump_held: bool,
    pub traverse_pressed_for: Option<f32>,
    pub dash_pressed_for: Option<f32>,
    pub sprint_active: bool,
    pub crouch_active: bool,
    pub ascend_active: bool,
}

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub struct MoveAction;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub struct LookAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct JumpAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct SprintAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct CrouchAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct AscendAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct TraverseAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct DashAction;

pub(crate) fn cache_move_axis(
    trigger: On<Fire<MoveAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.move_axis = trigger.value;
    }
}

pub(crate) fn clear_move_axis_on_cancel(
    trigger: On<InputCancel<MoveAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.move_axis = Vec2::ZERO;
    }
}

pub(crate) fn clear_move_axis_on_complete(
    trigger: On<Complete<MoveAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.move_axis = Vec2::ZERO;
    }
}

pub(crate) fn cache_jump_press(
    trigger: On<Start<JumpAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.jump_pressed_for = Some(0.0);
        input.jump_held = true;
    }
}

pub(crate) fn clear_jump_held_on_cancel(
    trigger: On<InputCancel<JumpAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.jump_held = false;
    }
}

pub(crate) fn clear_jump_held_on_complete(
    trigger: On<Complete<JumpAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.jump_held = false;
    }
}

pub(crate) fn cache_dash_press(
    trigger: On<Start<DashAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.dash_pressed_for = Some(0.0);
    }
}

pub(crate) fn cache_traverse_press(
    trigger: On<Start<TraverseAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.traverse_pressed_for = Some(0.0);
    }
}

pub(crate) fn cache_sprint_active(
    trigger: On<Fire<SprintAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.sprint_active = trigger.value;
    }
}

pub(crate) fn clear_sprint_active_on_cancel(
    trigger: On<InputCancel<SprintAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.sprint_active = false;
    }
}

pub(crate) fn clear_sprint_active_on_complete(
    trigger: On<Complete<SprintAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.sprint_active = false;
    }
}

pub(crate) fn cache_crouch_active(
    trigger: On<Fire<CrouchAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.crouch_active = trigger.value;
    }
}

pub(crate) fn clear_crouch_active_on_cancel(
    trigger: On<InputCancel<CrouchAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.crouch_active = false;
    }
}

pub(crate) fn clear_crouch_active_on_complete(
    trigger: On<Complete<CrouchAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.crouch_active = false;
    }
}

pub(crate) fn cache_ascend_active(
    trigger: On<Fire<AscendAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.ascend_active = trigger.value;
    }
}

pub(crate) fn clear_ascend_active_on_cancel(
    trigger: On<InputCancel<AscendAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.ascend_active = false;
    }
}

pub(crate) fn clear_ascend_active_on_complete(
    trigger: On<Complete<AscendAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.ascend_active = false;
    }
}

pub(crate) fn apply_look_axis(
    trigger: On<Fire<LookAction>>,
    mut query: Query<
        (
            &mut AccumulatedInput,
            Option<&mut CharacterLook>,
            &mut CharacterControllerState,
        ),
        With<CharacterController>,
    >,
) {
    let Ok((mut input, look, mut state)) = query.get_mut(trigger.context) else {
        return;
    };
    input.look_axis = trigger.value;
    let Some(mut look) = look else {
        return;
    };
    let scaled = trigger.value * look.sensitivity;
    look.yaw -= scaled.x;
    look.pitch = (look.pitch - scaled.y).clamp(look.min_pitch, look.max_pitch);
    state.orientation = look.orientation();
}

pub(crate) fn clear_look_axis_on_cancel(
    trigger: On<InputCancel<LookAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.look_axis = Vec2::ZERO;
    }
}

pub(crate) fn clear_look_axis_on_complete(
    trigger: On<Complete<LookAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.look_axis = Vec2::ZERO;
    }
}

pub(crate) fn tick_input_buffers(mut query: Query<&mut AccumulatedInput>, time: Res<Time>) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    for mut input in &mut query {
        if let Some(age) = input.jump_pressed_for.as_mut() {
            *age += dt;
        }
        if let Some(age) = input.traverse_pressed_for.as_mut() {
            *age += dt;
        }
        if let Some(age) = input.dash_pressed_for.as_mut() {
            *age += dt;
        }
    }
}
