use crate::{
    CharacterController, CharacterControllerState, CharacterLook,
    components::{CharacterColliderCache, CharacterControllerScratch, refresh_collider_cache},
};
use avian3d::prelude::*;
use bevy::prelude::*;

type RefreshShapesQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static CharacterController,
        &'static CharacterControllerState,
        &'static mut CharacterColliderCache,
        &'static mut CharacterControllerScratch,
        &'static mut Collider,
    ),
    Or<(
        Changed<CharacterController>,
        Changed<CharacterControllerState>,
    )>,
>;

pub(crate) fn setup_new_controllers(
    mut query: Query<
        (
            Entity,
            &mut CharacterController,
            &mut CharacterControllerState,
            &mut CharacterColliderCache,
            &mut CharacterControllerScratch,
            &Transform,
            Option<&CharacterLook>,
        ),
        Added<CharacterController>,
    >,
) {
    for (entity, mut controller, mut state, mut cache, mut scratch, transform, look) in &mut query {
        controller.filter.excluded_entities.insert(entity);
        refresh_collider_cache(&mut cache, &controller);
        scratch.move_config.skin_width = controller.skin_width;
        state.orientation = look
            .map(CharacterLook::orientation)
            .unwrap_or(transform.rotation);
    }
}

pub(crate) fn refresh_character_shapes(mut query: RefreshShapesQuery) {
    for (controller, state, mut cache, mut scratch, mut collider) in &mut query {
        refresh_collider_cache(&mut cache, controller);
        scratch.move_config.skin_width = controller.skin_width;
        *collider = cache.active(state).clone();
    }
}
