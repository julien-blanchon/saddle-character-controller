use crate::{
    CharacterControllerState, CharacterJumped, CharacterLanded, CharacterMotionStats,
    MovementModeChanged, SupportBodyChanged, components::PendingLanding, intent::AccumulatedInput,
};
use avian3d::prelude::*;
use bevy::prelude::*;

pub(crate) fn apply_push_forces(
    controllers: Query<(
        &crate::CharacterPush,
        &crate::components::CharacterControllerScratch,
    )>,
    colliders: Query<&ColliderOf>,
    mut bodies: Query<(&RigidBody, Forces)>,
) {
    for (push, scratch) in &controllers {
        for contact in &scratch.contacts {
            let Ok(collider_of) = colliders.get(contact.entity) else {
                continue;
            };
            let Ok((rigid_body, mut forces)) = bodies.get_mut(collider_of.body) else {
                continue;
            };
            if !rigid_body.is_dynamic() {
                continue;
            }
            let impulse = -contact.normal * push.impulse_scale;
            forces.apply_linear_impulse_at_point(impulse, contact.point);
        }
    }
}

pub(crate) fn emit_controller_messages(
    controllers: Query<(
        Entity,
        &crate::components::CharacterControllerScratch,
        &CharacterControllerState,
        &CharacterMotionStats,
    )>,
    mut jumped: MessageWriter<CharacterJumped>,
    mut landed: MessageWriter<CharacterLanded>,
    mut modes: MessageWriter<MovementModeChanged>,
    mut supports: MessageWriter<SupportBodyChanged>,
) {
    for (entity, scratch, _state, _stats) in &controllers {
        if scratch.pending_jump {
            jumped.write(CharacterJumped { entity });
        }
        if let Some(PendingLanding {
            impact_speed,
            inherited_platform_velocity,
        }) = scratch.pending_landing
        {
            landed.write(CharacterLanded {
                entity,
                impact_speed,
                inherited_platform_velocity,
            });
        }
        if let Some((previous, current)) = scratch.pending_mode_change {
            modes.write(MovementModeChanged {
                entity,
                previous,
                current,
            });
        }
        if let Some((previous, current)) = scratch.pending_support_change {
            supports.write(SupportBodyChanged {
                entity,
                previous,
                current,
            });
        }
    }
}

pub(crate) fn clear_per_tick_input(mut query: Query<&mut AccumulatedInput>) {
    for mut input in &mut query {
        input.look_axis = Vec2::ZERO;
    }
}

pub(crate) fn sync_active_collider(
    mut query: Query<
        (
            &crate::CharacterControllerState,
            &crate::components::CharacterColliderCache,
            &mut Collider,
        ),
        Changed<crate::CharacterControllerState>,
    >,
) {
    for (state, cache, mut collider) in &mut query {
        *collider = cache.active(state).clone();
    }
}
