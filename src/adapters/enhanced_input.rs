use crate::{AccumulatedInput, CharacterController};
use bevy::prelude::*;
use bevy_enhanced_input::context::InputContextAppExt;
use bevy_enhanced_input::prelude::{
    Cancel as InputCancel, Complete, EnhancedInputPlugin, Fire, InputAction,
};

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub struct MoveAction;

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

pub struct CharacterControllerEnhancedInputPlugin;

impl Plugin for CharacterControllerEnhancedInputPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EnhancedInputPlugin>() {
            app.add_plugins(EnhancedInputPlugin);
        }

        app.add_input_context::<CharacterController>()
            .add_observer(cache_move_axis)
            .add_observer(clear_move_axis_on_cancel)
            .add_observer(clear_move_axis_on_complete)
            .add_observer(cache_jump_press)
            .add_observer(clear_jump_held_on_cancel)
            .add_observer(clear_jump_held_on_complete)
            .add_observer(cache_traverse_press)
            .add_observer(cache_dash_press)
            .add_observer(cache_sprint_active)
            .add_observer(clear_sprint_active_on_cancel)
            .add_observer(clear_sprint_active_on_complete)
            .add_observer(cache_crouch_active)
            .add_observer(clear_crouch_active_on_cancel)
            .add_observer(clear_crouch_active_on_complete)
            .add_observer(cache_ascend_active)
            .add_observer(clear_ascend_active_on_cancel)
            .add_observer(clear_ascend_active_on_complete);
    }
}

pub(crate) fn cache_move_axis(
    trigger: On<Fire<MoveAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.set_move_axis(trigger.value);
    }
}

pub(crate) fn clear_move_axis_on_cancel(
    trigger: On<InputCancel<MoveAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.set_move_axis(Vec2::ZERO);
    }
}

pub(crate) fn clear_move_axis_on_complete(
    trigger: On<Complete<MoveAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.set_move_axis(Vec2::ZERO);
    }
}

// Jump uses Fire + manual edge detection instead of Start for reliability.
// press_jump() is only called on the rising edge (when jump_held is false).
pub(crate) fn cache_jump_press(
    trigger: On<Fire<JumpAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        if !input.jump_held {
            input.press_jump();
        }
    }
}

pub(crate) fn clear_jump_held_on_cancel(
    trigger: On<InputCancel<JumpAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.release_jump();
    }
}

pub(crate) fn clear_jump_held_on_complete(
    trigger: On<Complete<JumpAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.release_jump();
    }
}

// Dash and traverse also use Fire + edge detection for reliability.
pub(crate) fn cache_dash_press(
    trigger: On<Fire<DashAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        if input.dash_pressed_for.is_none() {
            input.press_dash();
        }
    }
}

pub(crate) fn cache_traverse_press(
    trigger: On<Fire<TraverseAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        if input.traverse_pressed_for.is_none() {
            input.press_traverse();
        }
    }
}

pub(crate) fn cache_sprint_active(
    trigger: On<Fire<SprintAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.set_sprint_active(trigger.value);
    }
}

pub(crate) fn clear_sprint_active_on_cancel(
    trigger: On<InputCancel<SprintAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.set_sprint_active(false);
    }
}

pub(crate) fn clear_sprint_active_on_complete(
    trigger: On<Complete<SprintAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.set_sprint_active(false);
    }
}

pub(crate) fn cache_crouch_active(
    trigger: On<Fire<CrouchAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.set_crouch_active(trigger.value);
    }
}

pub(crate) fn clear_crouch_active_on_cancel(
    trigger: On<InputCancel<CrouchAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.set_crouch_active(false);
    }
}

pub(crate) fn clear_crouch_active_on_complete(
    trigger: On<Complete<CrouchAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.set_crouch_active(false);
    }
}

pub(crate) fn cache_ascend_active(
    trigger: On<Fire<AscendAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.set_ascend_active(trigger.value);
    }
}

pub(crate) fn clear_ascend_active_on_cancel(
    trigger: On<InputCancel<AscendAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.set_ascend_active(false);
    }
}

pub(crate) fn clear_ascend_active_on_complete(
    trigger: On<Complete<AscendAction>>,
    mut query: Query<&mut AccumulatedInput, With<CharacterController>>,
) {
    if let Ok(mut input) = query.get_mut(trigger.context) {
        input.set_ascend_active(false);
    }
}
