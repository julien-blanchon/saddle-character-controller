use crate::{CharacterControllerDebugDraw, CharacterControllerState, CharacterMotionStats};
use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;

pub(crate) fn debug_draw(
    config: Res<CharacterControllerDebugDraw>,
    query: Query<(
        &Transform,
        &CharacterControllerState,
        &CharacterMotionStats,
        &LinearVelocity,
    )>,
    mut gizmos: Gizmos,
) {
    if !config.enabled {
        return;
    }
    for (transform, state, _stats, linear_velocity) in &query {
        if config.draw_velocity {
            gizmos.arrow(
                transform.translation,
                transform.translation + linear_velocity.0.clamp_length_max(5.0),
                Color::srgb(0.15, 0.55, 0.95),
            );
        }
        if config.draw_ground {
            if let Some(ground) = state.ground {
                gizmos.arrow(
                    ground.point,
                    ground.point + ground.normal,
                    Color::srgb(0.2, 0.85, 0.3),
                );
            }
        }
        if config.draw_support {
            gizmos.arrow(
                transform.translation,
                transform.translation + state.support_velocity.clamp_length_max(2.0),
                Color::srgb(0.95, 0.7, 0.15),
            );
        }
    }
}
