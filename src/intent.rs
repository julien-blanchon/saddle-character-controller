use bevy::prelude::*;

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

impl AccumulatedInput {
    pub fn set_move_axis(&mut self, move_axis: Vec2) {
        self.move_axis = move_axis;
    }

    pub fn set_look_axis(&mut self, look_axis: Vec2) {
        self.look_axis = look_axis;
    }

    pub fn press_jump(&mut self) {
        self.jump_pressed_for = Some(0.0);
        self.jump_held = true;
    }

    pub fn release_jump(&mut self) {
        self.jump_held = false;
    }

    pub fn press_traverse(&mut self) {
        self.traverse_pressed_for = Some(0.0);
    }

    pub fn press_dash(&mut self) {
        self.dash_pressed_for = Some(0.0);
    }

    pub fn set_sprint_active(&mut self, sprint_active: bool) {
        self.sprint_active = sprint_active;
    }

    pub fn set_crouch_active(&mut self, crouch_active: bool) {
        self.crouch_active = crouch_active;
    }

    pub fn set_ascend_active(&mut self, ascend_active: bool) {
        self.ascend_active = ascend_active;
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
