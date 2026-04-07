use crate::{
    CharacterController, CharacterControllerState, ExternalMotion,
    abilities::{
        dash::CharacterDash,
        flying::{CharacterFlying, FlightCollisionMode},
        mantle::CharacterMantle,
        swimming::CharacterSwimming,
        wall_kick::CharacterWallKick,
    },
    components::{
        CharacterColliderCache, CharacterControllerScratch, CharacterGravity, CharacterMotionStats,
        CharacterPush, PendingLanding,
    },
    helpers::{
        ages_recently, apply_ground_friction, clamp_velocity, classify_mode, expire_old,
        ground_probe_distance, horizontal, horizontal_speed, inherited_support_rotation,
        inherited_support_velocity, is_walkable, jump_velocity_for_height,
        quake_acceleration_delta, safe_dt, step_height_allowed, support_detach_velocity,
        travel_towards, wish_velocity_3d, wish_velocity_from_input,
    },
    intent::AccumulatedInput,
    state::{
        ControllerMode, DashState, EnvironmentDepth, EnvironmentModifiers, GroundContact,
        MantleState,
    },
    surfaces::{MovementSurface, SupportRotationPolicy, SupportVelocityPolicy},
};
use avian3d::{
    character_controller::move_and_slide::{DepenetrationConfig, MoveAndSlideHitResponse},
    prelude::*,
};
use bevy::prelude::*;
use core::time::Duration;

#[derive(Clone, Copy, Debug)]
struct SurfaceProfile {
    traction_multiplier: f32,
    acceleration_multiplier: f32,
    speed_multiplier: f32,
    jump_multiplier: f32,
    conveyor_velocity: Vec3,
    inherit_velocity_policy: Option<SupportVelocityPolicy>,
    inherit_rotation_policy: Option<SupportRotationPolicy>,
    slide_only: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct SupportMotion {
    linear_velocity: Vec3,
    angular_velocity: Vec3,
}

impl Default for SurfaceProfile {
    fn default() -> Self {
        Self {
            traction_multiplier: 1.0,
            acceleration_multiplier: 1.0,
            speed_multiplier: 1.0,
            jump_multiplier: 1.0,
            conveyor_velocity: Vec3::ZERO,
            inherit_velocity_policy: None,
            inherit_rotation_policy: None,
            slide_only: false,
        }
    }
}

type ControllerQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static CharacterController,
        &'static mut CharacterControllerState,
        &'static mut CharacterMotionStats,
        &'static mut AccumulatedInput,
        &'static mut LinearVelocity,
        &'static mut Transform,
        &'static CharacterColliderCache,
        &'static mut CharacterControllerScratch,
        &'static EnvironmentModifiers,
        Option<&'static CharacterFlying>,
        Option<&'static CharacterMantle>,
        Option<&'static CharacterWallKick>,
        Option<&'static CharacterSwimming>,
        (
            Option<&'static CharacterPush>,
            Option<&'static mut ExternalMotion>,
            Option<&'static mut CharacterDash>,
            Option<&'static CharacterGravity>,
        ),
    ),
>;

type SupportColliderQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Position,
        &'static Rotation,
        Option<&'static ColliderOf>,
    ),
    Without<CharacterController>,
>;

type SupportBodyQuery<'w, 's> = Query<
    'w,
    's,
    (
        Option<&'static LinearVelocity>,
        Option<&'static AngularVelocity>,
        Option<&'static ComputedCenterOfMass>,
    ),
    Without<CharacterController>,
>;

pub(crate) fn run_movement_prepare(
    mut controllers: ControllerQuery,
    move_and_slide: MoveAndSlide,
    surfaces: Query<&MovementSurface>,
    support_colliders: SupportColliderQuery,
    support_bodies: SupportBodyQuery,
    time: Res<Time>,
) {
    let dt = safe_dt(time.delta_secs());
    if dt == 0.0 {
        return;
    }

    for (
        entity,
        controller,
        mut state,
        mut stats,
        mut input,
        mut linear_velocity,
        mut transform,
        cache,
        mut scratch,
        env,
        flying,
        mantle,
        _wall_kick,
        swimming,
        (_push, external_motion, mut dash, _gravity_override),
    ) in &mut controllers
    {
        // --- Controller mode gate ---
        match controller.controller_mode {
            ControllerMode::Disabled => continue,
            ControllerMode::SenseOnly => {
                // Only do grounding probes, skip movement.
                let probe = ground_probe(
                    entity,
                    controller,
                    &move_and_slide,
                    &surfaces,
                    &support_colliders,
                    cache,
                    &state,
                    &transform,
                    state.support_velocity,
                    dt,
                    &mut stats,
                );
                apply_support_state(
                    controller,
                    &mut state,
                    probe,
                    &support_colliders,
                    &support_bodies,
                    &surfaces,
                    dt,
                );
                if state.ground.is_some_and(|ground| ground.walkable) {
                    state.grounded_frames = state.grounded_frames.saturating_add(1);
                    state.time_since_grounded = 0.0;
                } else {
                    state.grounded_frames = 0;
                    state.time_since_grounded += dt;
                }
                state.movement_mode = classify_mode(
                    flying.is_some_and(|flight| flight.enabled),
                    swimming.is_some(),
                    env.depth,
                    state.ground,
                    state.mantle.is_some(),
                    state.dash.is_some(),
                );
                scratch.skip_execute = true;
                continue;
            }
            ControllerMode::Enabled => { /* fall through to full simulation */ }
        }

        scratch.contacts.clear();
        scratch.pending_jump = false;
        scratch.pending_landing = None;
        scratch.pending_mode_change = None;
        scratch.pending_support_change = None;
        scratch.previous_mode = state.movement_mode;
        scratch.previous_support = state.ground.map(|ground| ground.entity);
        scratch.previous_grounded = state.ground.is_some_and(|ground| ground.walkable);
        stats.shape_casts_last_tick = 0;

        if let Some(mut external_motion) = external_motion {
            linear_velocity.0 += external_motion.velocity_delta;
            external_motion.velocity_delta = Vec3::ZERO;
        }

        depenetrate_character(
            entity,
            controller,
            &move_and_slide,
            cache,
            &state,
            &mut transform,
        );
        stats.shape_casts_last_tick += 1;

        state.time_since_wall_kick += dt;
        state.detach_time += dt;

        // Tick dash cooldown timer.
        if let Some(dash_cfg) = dash.as_deref_mut() {
            dash_cfg.time_since_dash += dt;
        }

        let pre_move_ground = ground_probe(
            entity,
            controller,
            &move_and_slide,
            &surfaces,
            &support_colliders,
            cache,
            &state,
            &transform,
            state.support_velocity,
            dt,
            &mut stats,
        );
        apply_support_state(
            controller,
            &mut state,
            pre_move_ground,
            &support_colliders,
            &support_bodies,
            &surfaces,
            dt,
        );

        if flying.is_some_and(|flight| flight.enabled) {
            state.ground = None;
            state.support_velocity = Vec3::ZERO;
            state.support_angular_velocity = Vec3::ZERO;
            state.detach_time = controller.support_detach_grace.as_secs_f32();
            state.mantle = None;
        } else if state.ground.is_some_and(|ground| ground.walkable) {
            apply_support_rotation(
                effective_support_rotation_policy(
                    controller,
                    state.ground,
                    &surfaces,
                    &support_colliders,
                ),
                &mut state,
                &mut transform,
                dt,
            );
        }

        if flying.is_some_and(|flight| flight.enabled) {
            state.crouching = false;
        } else {
            resolve_crouch(
                entity,
                controller,
                &move_and_slide,
                cache,
                &mut state,
                &mut input,
                &transform,
            );
        }

        expire_old(
            &mut input.jump_pressed_for,
            controller.jump_input_buffer.as_secs_f32(),
        );
        if let Some(mantle) = mantle {
            expire_old(
                &mut input.traverse_pressed_for,
                mantle.input_buffer.as_secs_f32(),
            );
        }
        // Expire dash input buffer using the dash cooldown as the window.
        if let Some(dash_cfg) = dash.as_ref() {
            expire_old(&mut input.dash_pressed_for, dash_cfg.cooldown.as_secs_f32());
        }

        let support_velocity = if state.ground.is_some_and(|ground| ground.walkable) {
            inherited_support_velocity(
                effective_support_policy(
                    controller,
                    pre_move_ground,
                    &surfaces,
                    &support_colliders,
                ),
                state.support_velocity,
            )
        } else {
            support_detach_velocity(
                state.support_velocity,
                state.detach_time,
                controller.support_detach_grace.as_secs_f32(),
                controller.support_velocity_policy,
            )
        };

        if state.mantle.is_some() && mantle.is_none() {
            state.mantle = None;
        }

        scratch.computed_support_velocity = support_velocity;
        scratch.pre_move_ground = pre_move_ground;
        scratch.skip_execute = false;
    }
}

pub(crate) fn run_movement_execute(
    mut controllers: ControllerQuery,
    move_and_slide: MoveAndSlide,
    surfaces: Query<&MovementSurface>,
    support_colliders: SupportColliderQuery,
    time: Res<Time>,
) {
    let dt = safe_dt(time.delta_secs());
    if dt == 0.0 {
        return;
    }

    for (
        entity,
        controller,
        mut state,
        mut stats,
        mut input,
        mut linear_velocity,
        mut transform,
        cache,
        mut scratch,
        env,
        flying,
        mantle,
        wall_kick,
        swimming,
        (_push, _external_motion, mut dash, gravity_override),
    ) in &mut controllers
    {
        if scratch.skip_execute {
            continue;
        }

        let support_velocity = scratch.computed_support_velocity;
        let pre_move_ground = scratch.pre_move_ground;

        if let Some(mantle_state) = state.mantle {
            if let Some(mantle_config) = mantle {
                advance_mantle(
                    entity,
                    controller,
                    mantle_config,
                    mantle_state,
                    cache,
                    &move_and_slide,
                    dt,
                    &mut state,
                    &mut linear_velocity,
                    &mut transform,
                    &mut stats,
                );
            }
        } else if state.dash.is_some() {
            // Active dash: advance it.
            advance_dash(
                entity,
                controller,
                cache,
                &move_and_slide,
                dt,
                &mut state,
                &mut linear_velocity,
                &mut transform,
                &mut scratch,
                &mut stats,
                dash.as_deref_mut(),
            );
        } else {
            let surface_profile = pre_move_ground
                .map(|ground| {
                    surface_profile_for_entity(ground.entity, &surfaces, &support_colliders)
                })
                .unwrap_or_default();
            let move_speed = configured_move_speed(controller, &state, &input, surface_profile);
            let wish_velocity =
                wish_velocity_from_input(state.orientation, input.move_axis, move_speed);
            let mut wish_velocity_3d =
                wish_velocity_3d(state.orientation, input.move_axis, move_speed);

            if env.depth > EnvironmentDepth::Shallow {
                if input.ascend_active {
                    let vertical = swimming
                        .unwrap_or(&CharacterSwimming::default())
                        .ascent_speed_scale
                        * move_speed;
                    wish_velocity_3d += Vec3::Y * vertical;
                }
                wish_velocity_3d = wish_velocity_3d.clamp_length_max(move_speed);
            }

            // Try to start a dash before normal movement.
            if try_start_dash(controller, &mut state, &mut input, &mut dash, wish_velocity) {
                // Dash just started; advance it for this tick.
                advance_dash(
                    entity,
                    controller,
                    cache,
                    &move_and_slide,
                    dt,
                    &mut state,
                    &mut linear_velocity,
                    &mut transform,
                    &mut scratch,
                    &mut stats,
                    dash.as_deref_mut(),
                );
            } else if let Some(flying) = flying.filter(|flight| flight.enabled) {
                fly_move(
                    entity,
                    controller,
                    flying,
                    cache,
                    &move_and_slide,
                    dt,
                    wish_velocity_3d,
                    &input,
                    &mut linear_velocity,
                    &mut transform,
                    &mut state,
                    &mut scratch,
                    &mut stats,
                );
            } else {
                if try_start_mantle(
                    entity,
                    controller,
                    mantle,
                    cache,
                    &move_and_slide,
                    &surfaces,
                    &support_colliders,
                    &mut state,
                    &mut input,
                    &transform,
                    &mut stats,
                ) {
                    continue;
                }

                let effective_gravity = gravity_override
                    .map(|g| g.magnitude)
                    .unwrap_or(controller.gravity);

                let jump_speed =
                    jump_velocity_for_height(effective_gravity, controller.jump_height)
                        * surface_profile.jump_multiplier;
                let pre_jump_vertical_speed = linear_velocity.y;
                let jumped = try_jump(
                    entity,
                    controller,
                    cache,
                    &move_and_slide,
                    wall_kick,
                    jump_speed,
                    wish_velocity,
                    &mut state,
                    &mut input,
                    &mut linear_velocity,
                    &mut transform,
                    &mut stats,
                );
                if jumped {
                    scratch.pending_jump = true;
                    state.last_jump_speed = jump_speed;
                }

                if let Some(swim) = swimming.filter(|_| env.depth > EnvironmentDepth::Shallow) {
                    water_move(
                        entity,
                        controller,
                        swim,
                        cache,
                        &move_and_slide,
                        dt,
                        support_velocity,
                        wish_velocity_3d,
                        env,
                        &mut linear_velocity,
                        &mut transform,
                        &mut state,
                        &mut scratch,
                        &mut stats,
                    );
                } else if state.ground.is_some_and(|ground| ground.walkable) {
                    if !(controller.auto_bhop && state.grounded_frames <= 1) {
                        let friction_velocity = apply_ground_friction(
                            horizontal(linear_velocity.0),
                            controller.stop_speed,
                            controller.friction_hz,
                            surface_profile.traction_multiplier,
                            dt,
                        );
                        linear_velocity.x = friction_velocity.x;
                        linear_velocity.z = friction_velocity.z;
                    }
                    ground_move(
                        entity,
                        controller,
                        cache,
                        &move_and_slide,
                        dt,
                        support_velocity + surface_profile.conveyor_velocity,
                        wish_velocity * surface_profile.acceleration_multiplier,
                        &mut linear_velocity,
                        &mut transform,
                        &mut state,
                        &mut scratch,
                        &mut stats,
                    );
                } else {
                    air_move(
                        entity,
                        controller,
                        cache,
                        &move_and_slide,
                        dt,
                        support_velocity,
                        wish_velocity,
                        &mut linear_velocity,
                        &mut transform,
                        &mut state,
                        &mut scratch,
                        &mut stats,
                    );
                }

                if env.depth <= EnvironmentDepth::Shallow {
                    let gravity_scale = if state.ground.is_some_and(|ground| !ground.walkable) {
                        controller.slide_gravity_scale
                    } else if !input.jump_held && linear_velocity.y > 0.0 {
                        // Variable-height jump: apply stronger gravity when jump
                        // button is released during ascent.
                        controller.jump_cut_gravity_multiplier
                    } else if linear_velocity.y < 0.0 {
                        controller.fall_gravity_multiplier
                    } else {
                        1.0
                    };
                    linear_velocity.y -= effective_gravity * gravity_scale * 0.5 * dt;
                }
                linear_velocity.y = linear_velocity.y.max(-controller.terminal_velocity);
                let _ = pre_jump_vertical_speed;
            }
        }
    }
}

pub(crate) fn run_movement_finalize(
    mut controllers: ControllerQuery,
    move_and_slide: MoveAndSlide,
    surfaces: Query<&MovementSurface>,
    support_colliders: SupportColliderQuery,
    support_bodies: SupportBodyQuery,
    time: Res<Time>,
) {
    let dt = safe_dt(time.delta_secs());
    if dt == 0.0 {
        return;
    }

    for (
        entity,
        controller,
        mut state,
        mut stats,
        _input,
        mut linear_velocity,
        mut transform,
        cache,
        mut scratch,
        env,
        flying,
        _mantle,
        _wall_kick,
        swimming,
        (_push, _external_motion, mut dash, _gravity_override),
    ) in &mut controllers
    {
        if scratch.skip_execute {
            continue;
        }

        let mut post_ground = if flying.is_some_and(|flight| flight.enabled) {
            None
        } else {
            ground_probe(
                entity,
                controller,
                &move_and_slide,
                &surfaces,
                &support_colliders,
                cache,
                &state,
                &transform,
                state.support_velocity,
                dt,
                &mut stats,
            )
        };
        // Don't snap to ground on the frame the character just jumped — snap_to_ground
        // casts far enough downward (snap_distance + step_down_detection_distance) to
        // reach the ground even after one tick of jump velocity, which would cancel the
        // jump entirely.
        if !scratch.pending_jump {
            if post_ground.is_none() || post_ground.is_some_and(|ground| !ground.walkable) {
                snap_to_ground(
                    entity,
                    controller,
                    &move_and_slide,
                    cache,
                    &state,
                    &mut transform,
                    &mut stats,
                );
                post_ground = ground_probe(
                    entity,
                    controller,
                    &move_and_slide,
                    &surfaces,
                    &support_colliders,
                    cache,
                    &state,
                    &transform,
                    state.support_velocity,
                    dt,
                    &mut stats,
                );
            }
        }
        if flying.is_some_and(|flight| flight.enabled) {
            post_ground = None;
        }

        let was_grounded = scratch.previous_grounded;
        let landed = !was_grounded && post_ground.is_some_and(|ground| ground.walkable);
        let effective_support_before = scratch.previous_support;

        apply_support_state(
            controller,
            &mut state,
            post_ground,
            &support_colliders,
            &support_bodies,
            &surfaces,
            dt,
        );

        if state.ground.is_some_and(|ground| ground.walkable) {
            state.grounded_frames = state.grounded_frames.saturating_add(1);
            state.time_since_grounded = 0.0;
            // Reset air jumps on landing.
            state.air_jumps_used = 0;
            // Reset air dashes on landing.
            if let Some(dash_cfg) = dash.as_deref_mut() {
                dash_cfg.air_dashes_used = 0;
            }
        } else {
            state.grounded_frames = 0;
            state.time_since_grounded += dt;
        }

        stats.current_speed = linear_velocity.length();
        stats.horizontal_speed = horizontal_speed(linear_velocity.0);
        stats.last_ground_normal = state
            .ground
            .map(|ground| ground.normal)
            .unwrap_or(Vec3::ZERO);
        stats.last_support_entity = state.ground.map(|ground| ground.entity);
        if state.ground.is_some_and(|ground| ground.walkable) {
            stats.grounded_time += dt;
            stats.airborne_time = 0.0;
        } else {
            stats.airborne_time += dt;
            stats.grounded_time = 0.0;
        }

        state.movement_mode = classify_mode(
            flying.is_some_and(|flight| flight.enabled),
            swimming.is_some(),
            env.depth,
            state.ground,
            state.mantle.is_some(),
            state.dash.is_some(),
        );
        if state.movement_mode != scratch.previous_mode {
            scratch.pending_mode_change = Some((scratch.previous_mode, state.movement_mode));
        }

        let current_support = state.ground.map(|ground| ground.entity);
        if current_support != effective_support_before {
            scratch.pending_support_change = Some((effective_support_before, current_support));
        }

        if landed {
            scratch.pending_landing = Some(PendingLanding {
                impact_speed: linear_velocity.y.abs(),
                inherited_platform_velocity: state.support_velocity,
            });
        }

        linear_velocity.0 = clamp_velocity(linear_velocity.0, controller.max_speed);
    }
}

// ---------------------------------------------------------------------------
// Dash helpers
// ---------------------------------------------------------------------------

/// Attempt to begin a dash. Returns `true` if a dash was started.
fn try_start_dash(
    _controller: &CharacterController,
    state: &mut CharacterControllerState,
    input: &mut AccumulatedInput,
    dash: &mut Option<Mut<CharacterDash>>,
    wish_velocity: Vec3,
) -> bool {
    let Some(dash_cfg) = dash.as_deref_mut() else {
        return false;
    };
    // Already dashing?
    if state.dash.is_some() {
        return false;
    }
    // Input buffered?
    if !ages_recently(input.dash_pressed_for, dash_cfg.cooldown.as_secs_f32()) {
        return false;
    }
    // Cooldown check.
    if dash_cfg.time_since_dash < dash_cfg.cooldown.as_secs_f32() {
        return false;
    }
    // Air dash budget.
    let grounded = state.ground.is_some_and(|g| g.walkable);
    if !grounded
        && dash_cfg.max_air_dashes > 0
        && dash_cfg.air_dashes_used >= dash_cfg.max_air_dashes
    {
        return false;
    }

    // Lock direction.
    let direction = if wish_velocity.length_squared() > 0.001 {
        wish_velocity.normalize()
    } else {
        let fwd = state.orientation * Vec3::NEG_Z;
        horizontal(fwd).normalize_or_zero()
    };
    if direction == Vec3::ZERO {
        return false;
    }

    input.dash_pressed_for = None;
    dash_cfg.time_since_dash = 0.0;
    if !grounded {
        dash_cfg.air_dashes_used += 1;
    }
    state.dash = Some(DashState {
        direction,
        remaining: dash_cfg.duration.as_secs_f32(),
        speed: dash_cfg.speed,
    });
    true
}

/// Advance an active dash, consuming remaining time. Ends the dash when duration expires.
#[allow(clippy::too_many_arguments)]
fn advance_dash(
    entity: Entity,
    controller: &CharacterController,
    cache: &CharacterColliderCache,
    move_and_slide: &MoveAndSlide,
    dt: f32,
    state: &mut CharacterControllerState,
    linear_velocity: &mut LinearVelocity,
    transform: &mut Transform,
    scratch: &mut CharacterControllerScratch,
    stats: &mut CharacterMotionStats,
    dash: Option<&mut CharacterDash>,
) {
    let Some(mut dash_state) = state.dash else {
        return;
    };
    let cancel_gravity = dash.as_ref().is_none_or(|d| d.cancel_gravity);

    // Drive velocity along the locked direction.
    linear_velocity.0 = dash_state.direction * dash_state.speed;
    if cancel_gravity {
        linear_velocity.y = 0.0;
    }

    move_character(
        entity,
        controller,
        cache,
        move_and_slide,
        dt,
        linear_velocity,
        transform,
        state,
        scratch,
        stats,
    );

    dash_state.remaining -= dt;
    if dash_state.remaining <= 0.0 {
        state.dash = None;
    } else {
        state.dash = Some(dash_state);
    }
}

// ---------------------------------------------------------------------------
// Core helpers
// ---------------------------------------------------------------------------

fn depenetrate_character(
    entity: Entity,
    controller: &CharacterController,
    move_and_slide: &MoveAndSlide,
    cache: &CharacterColliderCache,
    state: &CharacterControllerState,
    transform: &mut Transform,
) {
    let depenetration = DepenetrationConfig::from(&move_and_slide_config(
        controller,
        state.ground.map(|ground| ground.normal),
    ));
    let offset = move_and_slide.depenetrate(
        cache.active(state),
        transform.translation,
        transform.rotation,
        &depenetration,
        &filter_for_entity(controller, entity),
    );
    transform.translation += offset;
}

#[allow(clippy::too_many_arguments)]
fn ground_probe(
    entity: Entity,
    controller: &CharacterController,
    move_and_slide: &MoveAndSlide,
    surfaces: &Query<&MovementSurface>,
    support_colliders: &SupportColliderQuery,
    cache: &CharacterColliderCache,
    state: &CharacterControllerState,
    transform: &Transform,
    support_velocity: Vec3,
    dt: f32,
    stats: &mut CharacterMotionStats,
) -> Option<GroundContact> {
    // NOTE: ground probing is skipped when environment depth is beyond Shallow
    // because the character is swimming; this is now read from the EnvironmentModifiers
    // component passed to `run_controllers`. Since `ground_probe` is a plain fn
    // without access to `env`, callers gate on `env.depth` *before* calling when
    // needed (e.g. water_move branch). The old `state.water_level` field no longer
    // exists; however the ground probe itself was previously gated here. We keep the
    // probe running unconditionally so that `SenseOnly` mode and landing detection
    // still work even while submerged (the swim branch simply ignores the result).
    if linear_velocity_moving_up_rapidly(state, support_velocity, controller) {
        return None;
    }
    let cast_distance = ground_probe_distance(controller, support_velocity, dt);
    let hit = move_and_slide.cast_move(
        cache.active(state),
        transform.translation,
        transform.rotation,
        Vec3::NEG_Y * cast_distance,
        controller.skin_width,
        &filter_for_entity(controller, entity),
    );
    stats.shape_casts_last_tick += 1;
    let hit = hit?;
    let surface_profile = surface_profile_for_entity(hit.entity, surfaces, support_colliders);
    let walkable = is_walkable(
        hit.normal1,
        controller.min_walk_angle,
        surface_profile.slide_only,
    );
    Some(GroundContact {
        entity: hit.entity,
        point: hit.point1,
        normal: hit.normal1,
        distance: hit.distance,
        walkable,
    })
}

fn linear_velocity_moving_up_rapidly(
    state: &CharacterControllerState,
    support_velocity: Vec3,
    controller: &CharacterController,
) -> bool {
    let relative_up_speed = state.support_velocity.y.max(support_velocity.y).abs();
    relative_up_speed > controller.unground_speed
}

fn apply_support_state(
    controller: &CharacterController,
    state: &mut CharacterControllerState,
    ground: Option<GroundContact>,
    support_colliders: &SupportColliderQuery,
    support_bodies: &SupportBodyQuery,
    surfaces: &Query<&MovementSurface>,
    dt: f32,
) {
    if let Some(ground) = ground {
        let support_motion = calculate_support_motion(
            ground.point,
            ground.entity,
            support_colliders,
            support_bodies,
            surfaces,
            dt,
        );
        state.ground = Some(ground);
        state.last_support_entity = Some(ground.entity);
        state.detach_time = 0.0;
        state.support_velocity = support_motion.linear_velocity;
        state.support_angular_velocity = support_motion.angular_velocity;
    } else {
        state.ground = None;
        state.support_velocity = support_detach_velocity(
            state.support_velocity,
            state.detach_time,
            controller.support_detach_grace.as_secs_f32(),
            controller.support_velocity_policy,
        );
        state.support_angular_velocity = Vec3::ZERO;
    }
}

fn calculate_support_motion(
    contact_point: Vec3,
    ground_entity: Entity,
    support_colliders: &SupportColliderQuery,
    support_bodies: &SupportBodyQuery,
    surfaces: &Query<&MovementSurface>,
    dt: f32,
) -> SupportMotion {
    let mut motion = SupportMotion::default();
    let Ok((position, rotation, collider_of)) = support_colliders.get(ground_entity) else {
        return motion;
    };
    let body_entity = collider_of.map_or(ground_entity, |collider_of| collider_of.body);
    let Ok((linear_velocity, angular_velocity, center_of_mass)) = support_bodies.get(body_entity)
    else {
        return motion;
    };

    let linear_velocity = linear_velocity.map_or(Vec3::ZERO, |velocity| velocity.0);
    let angular_velocity = angular_velocity.map_or(Vec3::ZERO, |velocity| velocity.0);
    let center_of_mass = center_of_mass.map_or(Vec3::ZERO, |center| center.0);
    let ground_center = rotation.0 * center_of_mass + position.0;
    let platform_transform = Transform::IDENTITY
        .with_translation(ground_center)
        .with_rotation(rotation.0);
    let next_platform_transform = Transform::IDENTITY
        .with_translation(ground_center + linear_velocity * dt)
        .with_rotation(Quat::from_scaled_axis(angular_velocity * dt) * rotation.0);
    motion.linear_velocity = next_platform_transform.transform_point(
        platform_transform
            .compute_affine()
            .inverse()
            .transform_point3(contact_point),
    ) - contact_point;
    motion.linear_velocity /= dt.max(0.0001);
    motion.linear_velocity +=
        surface_profile_for_entity(ground_entity, surfaces, support_colliders).conveyor_velocity;
    motion.angular_velocity = angular_velocity;
    motion
}

fn apply_support_rotation(
    policy: SupportRotationPolicy,
    state: &mut CharacterControllerState,
    transform: &mut Transform,
    dt: f32,
) {
    let inherited = inherited_support_rotation(policy, state.support_angular_velocity);
    if inherited == Vec3::ZERO {
        return;
    }

    let delta = Quat::from_scaled_axis(inherited * dt);
    state.orientation = (delta * state.orientation).normalize();

    if policy != SupportRotationPolicy::None {
        transform.rotation = state.orientation;
    }
}

fn resolve_crouch(
    entity: Entity,
    controller: &CharacterController,
    move_and_slide: &MoveAndSlide,
    cache: &CharacterColliderCache,
    state: &mut CharacterControllerState,
    input: &mut AccumulatedInput,
    transform: &Transform,
) {
    if input.crouch_active {
        state.crouching = true;
        return;
    }
    if !state.crouching {
        return;
    }
    state.crouching = false;
    if move_and_slide
        .spatial_query
        .shape_intersections(
            cache.active(state),
            transform.translation,
            transform.rotation,
            &filter_for_entity(controller, entity),
        )
        .is_empty()
    {
        return;
    }
    state.crouching = true;
}

fn configured_move_speed(
    controller: &CharacterController,
    state: &CharacterControllerState,
    input: &AccumulatedInput,
    surface: SurfaceProfile,
) -> f32 {
    let mut speed = controller.speed * controller.global_speed_scale * surface.speed_multiplier;
    if input.sprint_active {
        speed *= controller.sprint_speed_scale;
    }
    if state.crouching {
        speed *= controller.crouch_speed_scale;
    }
    speed
}

#[allow(clippy::too_many_arguments)]
fn try_start_mantle(
    entity: Entity,
    controller: &CharacterController,
    mantle: Option<&CharacterMantle>,
    cache: &CharacterColliderCache,
    move_and_slide: &MoveAndSlide,
    surfaces: &Query<&MovementSurface>,
    support_colliders: &SupportColliderQuery,
    state: &mut CharacterControllerState,
    input: &mut AccumulatedInput,
    transform: &Transform,
    stats: &mut CharacterMotionStats,
) -> bool {
    let Some(mantle) = mantle else {
        return false;
    };
    if !ages_recently(
        input.traverse_pressed_for,
        mantle.input_buffer.as_secs_f32(),
    ) {
        return false;
    }

    let mut flat_forward = state.orientation * Vec3::NEG_Z;
    flat_forward.y = 0.0;
    flat_forward = flat_forward.normalize_or_zero();
    if flat_forward == Vec3::ZERO {
        return false;
    }

    let wall_hit = move_and_slide.cast_move(
        cache.active(state),
        transform.translation,
        transform.rotation,
        flat_forward * mantle.max_distance,
        controller.skin_width,
        &filter_for_entity(controller, entity),
    );
    stats.shape_casts_last_tick += 1;
    let Some(wall_hit) = wall_hit else {
        return false;
    };
    let wall_angle = wall_hit.normal1.y.acos();
    if wall_angle < mantle.min_wall_angle {
        return false;
    }

    let forward_offset = controller.capsule_radius + mantle.max_distance;
    let probe_origin =
        transform.translation + flat_forward * forward_offset + Vec3::Y * mantle.max_height;
    let down_hit = move_and_slide.cast_move(
        &cache.standing_collider,
        probe_origin,
        transform.rotation,
        Vec3::NEG_Y * (mantle.max_height + cache.standing_height),
        controller.skin_width,
        &filter_for_entity(controller, entity),
    );
    stats.shape_casts_last_tick += 1;
    let Some(down_hit) = down_hit else {
        return false;
    };
    let surface_profile = surface_profile_for_entity(down_hit.entity, surfaces, support_colliders);
    if !is_walkable(
        down_hit.normal1,
        controller.min_walk_angle,
        surface_profile.slide_only,
    ) {
        return false;
    }

    let target_position =
        probe_origin + Vec3::NEG_Y * down_hit.distance + Vec3::Y * mantle.pull_up_height;
    let path_hit = move_and_slide.cast_move(
        &cache.standing_collider,
        transform.translation,
        transform.rotation,
        target_position - transform.translation,
        controller.skin_width,
        &filter_for_entity(controller, entity),
    );
    stats.shape_casts_last_tick += 1;
    if path_hit.is_some() {
        return false;
    }

    input.traverse_pressed_for = None;
    state.mantle = Some(MantleState {
        target_position,
        wall_normal: wall_hit.normal1,
    });
    true
}

#[allow(clippy::too_many_arguments)]
fn advance_mantle(
    entity: Entity,
    controller: &CharacterController,
    mantle_config: &CharacterMantle,
    mantle: MantleState,
    cache: &CharacterColliderCache,
    move_and_slide: &MoveAndSlide,
    dt: f32,
    state: &mut CharacterControllerState,
    linear_velocity: &mut LinearVelocity,
    transform: &mut Transform,
    stats: &mut CharacterMotionStats,
) {
    let delta = mantle.target_position - transform.translation;
    if delta.length_squared() <= 0.0001 {
        state.mantle = None;
        linear_velocity.0 = Vec3::ZERO;
        return;
    }
    let travel = travel_towards(delta, mantle_config.speed, dt);
    if travel == Vec3::ZERO {
        state.mantle = None;
        linear_velocity.0 = Vec3::ZERO;
        return;
    }
    let out = move_and_slide.move_and_slide(
        &cache.standing_collider,
        transform.translation,
        transform.rotation,
        travel / dt,
        Duration::from_secs_f32(dt),
        &move_and_slide_config(controller, None),
        &filter_for_entity(controller, entity),
        |_| MoveAndSlideHitResponse::Accept,
    );
    stats.shape_casts_last_tick += 1;
    transform.translation = out.position;
    linear_velocity.0 = out.projected_velocity;
    if mantle
        .target_position
        .distance_squared(transform.translation)
        <= 0.01
    {
        state.mantle = None;
    } else {
        state.mantle = Some(mantle);
    }
}

#[allow(clippy::too_many_arguments)]
fn try_jump(
    entity: Entity,
    controller: &CharacterController,
    cache: &CharacterColliderCache,
    move_and_slide: &MoveAndSlide,
    wall_kick: Option<&CharacterWallKick>,
    jump_speed: f32,
    wish_velocity: Vec3,
    state: &mut CharacterControllerState,
    input: &mut AccumulatedInput,
    linear_velocity: &mut LinearVelocity,
    transform: &mut Transform,
    stats: &mut CharacterMotionStats,
) -> bool {
    let grounded = state.ground.is_some_and(|ground| ground.walkable);
    let can_ground_jump =
        grounded || state.time_since_grounded <= controller.coyote_time.as_secs_f32();
    if can_ground_jump {
        if !ages_recently(
            input.jump_pressed_for,
            controller.jump_input_buffer.as_secs_f32(),
        ) {
            return false;
        }
        input.jump_pressed_for = None;
        state.ground = None;
        state.grounded_frames = 0;
        state.time_since_grounded = controller.coyote_time.as_secs_f32();
        linear_velocity.y = jump_speed + state.support_velocity.y;
        return true;
    }

    // Air jump check: if not grounded and no coyote time, try air jump.
    if controller.max_air_jumps > state.air_jumps_used {
        if ages_recently(
            input.jump_pressed_for,
            controller.jump_input_buffer.as_secs_f32(),
        ) {
            input.jump_pressed_for = None;
            state.air_jumps_used += 1;
            linear_velocity.y = jump_speed;
            return true;
        }
    }

    let Some(wall_kick) = wall_kick else {
        return false;
    };
    if !ages_recently(input.jump_pressed_for, wall_kick.input_buffer.as_secs_f32()) {
        return false;
    }
    if state.time_since_wall_kick <= wall_kick.cooldown.as_secs_f32() {
        return false;
    }

    let direction = if wish_velocity.length_squared() > 0.001 {
        wish_velocity.normalize_or_zero()
    } else {
        horizontal(linear_velocity.0).normalize_or_zero()
    };
    if direction == Vec3::ZERO {
        return false;
    }
    let wall_hit = move_and_slide.cast_move(
        cache.active(state),
        transform.translation,
        transform.rotation,
        direction * wall_kick.distance,
        controller.skin_width,
        &filter_for_entity(controller, entity),
    );
    stats.shape_casts_last_tick += 1;
    let Some(wall_hit) = wall_hit else {
        return false;
    };
    if wall_hit.normal1.y < -0.01 {
        return false;
    }
    let wall_angle = wall_hit.normal1.y.acos();
    if wall_angle < wall_kick.max_wall_angle {
        return false;
    }

    input.jump_pressed_for = None;
    state.time_since_wall_kick = 0.0;
    let away = horizontal(wall_hit.normal1).normalize_or_zero();
    let jump_dir = (Vec3::Y * wall_kick.upward_factor + away + direction).normalize_or_zero();
    linear_velocity.0 += jump_dir * (jump_speed * wall_kick.power);
    true
}

#[allow(clippy::too_many_arguments)]
fn fly_move(
    entity: Entity,
    controller: &CharacterController,
    flying: &CharacterFlying,
    cache: &CharacterColliderCache,
    move_and_slide: &MoveAndSlide,
    dt: f32,
    mut wish_velocity: Vec3,
    input: &AccumulatedInput,
    linear_velocity: &mut LinearVelocity,
    transform: &mut Transform,
    state: &mut CharacterControllerState,
    scratch: &mut CharacterControllerScratch,
    stats: &mut CharacterMotionStats,
) {
    let flight_speed = if input.sprint_active {
        flying.speed * flying.sprint_speed_scale
    } else {
        flying.speed
    }
    .max(0.0)
        * controller.global_speed_scale.max(0.0);

    let vertical_speed = flight_speed * flying.vertical_speed_scale.max(0.0);
    if input.ascend_active {
        wish_velocity += Vec3::Y * vertical_speed;
    }
    if input.crouch_active {
        wish_velocity -= Vec3::Y * vertical_speed;
    }
    wish_velocity = wish_velocity.clamp_length_max(flight_speed.max(vertical_speed));

    linear_velocity.0 = apply_ground_friction(linear_velocity.0, 0.0, flying.drag_hz, 1.0, dt);
    linear_velocity.0 += quake_acceleration_delta(
        linear_velocity.0,
        wish_velocity,
        wish_velocity.length(),
        flying.acceleration_hz,
        dt,
        None,
    );
    linear_velocity.0 = clamp_velocity(linear_velocity.0, controller.max_speed);

    match flying.collision_mode {
        FlightCollisionMode::NoClip => {
            transform.translation += linear_velocity.0 * dt;
        }
        FlightCollisionMode::Slide => {
            move_character(
                entity,
                controller,
                cache,
                move_and_slide,
                dt,
                linear_velocity,
                transform,
                state,
                scratch,
                stats,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn ground_move(
    entity: Entity,
    controller: &CharacterController,
    cache: &CharacterColliderCache,
    move_and_slide: &MoveAndSlide,
    dt: f32,
    support_velocity: Vec3,
    wish_velocity: Vec3,
    linear_velocity: &mut LinearVelocity,
    transform: &mut Transform,
    state: &mut CharacterControllerState,
    scratch: &mut CharacterControllerScratch,
    stats: &mut CharacterMotionStats,
) {
    linear_velocity.y = 0.0;
    linear_velocity.0 += quake_acceleration_delta(
        horizontal(linear_velocity.0),
        wish_velocity,
        wish_velocity.length(),
        controller.acceleration_hz,
        dt,
        None,
    );
    linear_velocity.y = 0.0;
    linear_velocity.0 += support_velocity;
    let mut movement = linear_velocity.0 * dt;
    movement.y = 0.0;
    if move_and_slide
        .cast_move(
            cache.active(state),
            transform.translation,
            transform.rotation,
            movement,
            controller.skin_width,
            &filter_for_entity(controller, entity),
        )
        .is_none()
    {
        transform.translation += movement;
        linear_velocity.0 -= support_velocity;
        snap_to_ground(
            entity,
            controller,
            move_and_slide,
            cache,
            state,
            transform,
            stats,
        );
        return;
    }
    step_move(
        entity,
        controller,
        cache,
        move_and_slide,
        dt,
        support_velocity,
        linear_velocity,
        transform,
        state,
        scratch,
        stats,
    );
    linear_velocity.0 -= support_velocity;
    snap_to_ground(
        entity,
        controller,
        move_and_slide,
        cache,
        state,
        transform,
        stats,
    );
}

#[allow(clippy::too_many_arguments)]
fn air_move(
    entity: Entity,
    controller: &CharacterController,
    cache: &CharacterColliderCache,
    move_and_slide: &MoveAndSlide,
    dt: f32,
    support_velocity: Vec3,
    wish_velocity: Vec3,
    linear_velocity: &mut LinearVelocity,
    transform: &mut Transform,
    state: &mut CharacterControllerState,
    scratch: &mut CharacterControllerScratch,
    stats: &mut CharacterMotionStats,
) {
    linear_velocity.0 += quake_acceleration_delta(
        linear_velocity.0,
        wish_velocity,
        wish_velocity.length(),
        controller.air_acceleration_hz,
        dt,
        Some(controller.max_air_wish_speed * controller.air_speed),
    );
    linear_velocity.0 += support_velocity;
    step_move(
        entity,
        controller,
        cache,
        move_and_slide,
        dt,
        Vec3::ZERO,
        linear_velocity,
        transform,
        state,
        scratch,
        stats,
    );
    linear_velocity.0 -= support_velocity;
}

#[allow(clippy::too_many_arguments)]
fn water_move(
    entity: Entity,
    controller: &CharacterController,
    swimming: &CharacterSwimming,
    cache: &CharacterColliderCache,
    move_and_slide: &MoveAndSlide,
    dt: f32,
    support_velocity: Vec3,
    wish_velocity: Vec3,
    env: &EnvironmentModifiers,
    linear_velocity: &mut LinearVelocity,
    transform: &mut Transform,
    state: &mut CharacterControllerState,
    scratch: &mut CharacterControllerScratch,
    stats: &mut CharacterMotionStats,
) {
    let slowed = wish_velocity * swimming.slowdown * env.speed_multiplier.max(0.0);
    linear_velocity.0 += quake_acceleration_delta(
        linear_velocity.0,
        slowed,
        slowed.length(),
        swimming.acceleration_hz * env.acceleration_multiplier.max(0.0),
        dt,
        None,
    );
    linear_velocity.y -= swimming.gravity * env.gravity_multiplier.max(0.0) * dt;
    linear_velocity.0 += support_velocity;
    step_move(
        entity,
        controller,
        cache,
        move_and_slide,
        dt,
        Vec3::ZERO,
        linear_velocity,
        transform,
        state,
        scratch,
        stats,
    );
    linear_velocity.0 -= support_velocity;
}

#[allow(clippy::too_many_arguments)]
fn step_move(
    entity: Entity,
    controller: &CharacterController,
    cache: &CharacterColliderCache,
    move_and_slide: &MoveAndSlide,
    dt: f32,
    extra_base_velocity: Vec3,
    linear_velocity: &mut LinearVelocity,
    transform: &mut Transform,
    state: &mut CharacterControllerState,
    scratch: &mut CharacterControllerScratch,
    stats: &mut CharacterMotionStats,
) {
    let original_position = transform.translation;
    let original_velocity = linear_velocity.0;

    move_character(
        entity,
        controller,
        cache,
        move_and_slide,
        dt,
        linear_velocity,
        transform,
        state,
        scratch,
        stats,
    );
    let direct_position = transform.translation;
    let direct_velocity = linear_velocity.0;

    transform.translation = original_position;
    linear_velocity.0 = original_velocity;

    let step_up = move_and_slide.cast_move(
        cache.active(state),
        transform.translation,
        transform.rotation,
        Vec3::Y * controller.step_size,
        controller.skin_width,
        &filter_for_entity(controller, entity),
    );
    stats.shape_casts_last_tick += 1;
    let up_distance = step_up.map_or(controller.step_size, |hit| hit.distance);
    transform.translation += Vec3::Y * up_distance;

    let forward_check = horizontal(linear_velocity.0 + extra_base_velocity).normalize_or_zero();
    if forward_check != Vec3::ZERO {
        let blocked = move_and_slide.cast_move(
            cache.active(state),
            transform.translation,
            transform.rotation,
            forward_check * 0.2,
            controller.skin_width,
            &filter_for_entity(controller, entity),
        );
        stats.shape_casts_last_tick += 1;
        if blocked.is_some() {
            transform.translation = direct_position;
            linear_velocity.0 = direct_velocity;
            return;
        }
    }

    move_character(
        entity,
        controller,
        cache,
        move_and_slide,
        dt,
        linear_velocity,
        transform,
        state,
        scratch,
        stats,
    );
    let step_down = move_and_slide.cast_move(
        cache.active(state),
        transform.translation,
        transform.rotation,
        Vec3::NEG_Y * (controller.step_down_detection_distance + controller.snap_distance),
        controller.skin_width,
        &filter_for_entity(controller, entity),
    );
    stats.shape_casts_last_tick += 1;
    let Some(step_down) = step_down else {
        transform.translation = direct_position;
        linear_velocity.0 = direct_velocity;
        return;
    };
    if !step_height_allowed(
        step_down.distance,
        controller.step_down_detection_distance + controller.snap_distance,
    ) {
        transform.translation = direct_position;
        linear_velocity.0 = direct_velocity;
        return;
    }
    transform.translation += Vec3::NEG_Y * step_down.distance;
    if step_down.normal1.y < controller.min_walk_angle.cos() {
        transform.translation = direct_position;
        linear_velocity.0 = direct_velocity;
        return;
    }

    let direct_distance = direct_position
        .xz()
        .distance_squared(original_position.xz());
    let step_distance = transform
        .translation
        .xz()
        .distance_squared(original_position.xz());
    if direct_distance >= step_distance {
        transform.translation = direct_position;
        linear_velocity.0 = direct_velocity;
    }
}

#[allow(clippy::too_many_arguments)]
fn move_character(
    entity: Entity,
    controller: &CharacterController,
    cache: &CharacterColliderCache,
    move_and_slide: &MoveAndSlide,
    dt: f32,
    linear_velocity: &mut LinearVelocity,
    transform: &mut Transform,
    state: &CharacterControllerState,
    scratch: &mut CharacterControllerScratch,
    stats: &mut CharacterMotionStats,
) {
    let filter = filter_for_entity(controller, entity);
    let out = move_and_slide.move_and_slide(
        cache.active(state),
        transform.translation,
        transform.rotation,
        linear_velocity.0,
        Duration::from_secs_f32(dt),
        &move_and_slide_config(controller, state.ground.map(|ground| ground.normal)),
        &filter,
        |hit| {
            scratch.contacts.push(crate::components::TouchContact {
                entity: hit.entity,
                point: hit.point,
                normal: **hit.normal,
            });
            MoveAndSlideHitResponse::Accept
        },
    );
    stats.shape_casts_last_tick += 1;
    transform.translation = out.position;
    linear_velocity.0 = out.projected_velocity;
}

fn snap_to_ground(
    entity: Entity,
    controller: &CharacterController,
    move_and_slide: &MoveAndSlide,
    cache: &CharacterColliderCache,
    state: &CharacterControllerState,
    transform: &mut Transform,
    stats: &mut CharacterMotionStats,
) {
    let start = transform.translation + Vec3::Y * controller.snap_distance;
    let hit = move_and_slide.cast_move(
        cache.active(state),
        start,
        transform.rotation,
        Vec3::NEG_Y * (controller.snap_distance + controller.step_down_detection_distance),
        controller.skin_width,
        &filter_for_entity(controller, entity),
    );
    stats.shape_casts_last_tick += 1;
    let Some(hit) = hit else {
        return;
    };
    if hit.intersects()
        || hit.distance <= controller.ground_distance
        || hit.normal1.y < controller.min_walk_angle.cos()
    {
        return;
    }
    transform.translation = start + Vec3::NEG_Y * hit.distance;
}

fn move_and_slide_config(
    controller: &CharacterController,
    ground_normal: Option<Vec3>,
) -> MoveAndSlideConfig {
    let mut config = MoveAndSlideConfig {
        skin_width: controller.skin_width,
        ..default()
    };
    if let Some(ground_normal) = ground_normal.and_then(|normal| Dir3::new(normal).ok()) {
        config.planes.push(ground_normal);
    }
    config
}

fn filter_for_entity(controller: &CharacterController, entity: Entity) -> SpatialQueryFilter {
    let mut filter = controller.filter.clone();
    filter.excluded_entities.insert(entity);
    filter
}

fn surface_profile_for_entity(
    entity: Entity,
    surfaces: &Query<&MovementSurface>,
    support_colliders: &SupportColliderQuery,
) -> SurfaceProfile {
    if let Ok(surface) = surfaces.get(entity) {
        return SurfaceProfile {
            traction_multiplier: surface.traction_multiplier,
            acceleration_multiplier: surface.acceleration_multiplier,
            speed_multiplier: surface.speed_multiplier,
            jump_multiplier: surface.jump_multiplier,
            conveyor_velocity: surface.conveyor_velocity,
            inherit_velocity_policy: surface.inherit_velocity_policy,
            inherit_rotation_policy: surface.inherit_rotation_policy,
            slide_only: surface.slide_only,
        };
    }
    let Ok((_position, _rotation, collider_of)) = support_colliders.get(entity) else {
        return SurfaceProfile::default();
    };
    let Some(collider_of) = collider_of else {
        return SurfaceProfile::default();
    };
    if let Ok(surface) = surfaces.get(collider_of.body) {
        SurfaceProfile {
            traction_multiplier: surface.traction_multiplier,
            acceleration_multiplier: surface.acceleration_multiplier,
            speed_multiplier: surface.speed_multiplier,
            jump_multiplier: surface.jump_multiplier,
            conveyor_velocity: surface.conveyor_velocity,
            inherit_velocity_policy: surface.inherit_velocity_policy,
            inherit_rotation_policy: surface.inherit_rotation_policy,
            slide_only: surface.slide_only,
        }
    } else {
        SurfaceProfile::default()
    }
}

fn effective_support_policy(
    controller: &CharacterController,
    ground: Option<GroundContact>,
    surfaces: &Query<&MovementSurface>,
    support_colliders: &SupportColliderQuery,
) -> SupportVelocityPolicy {
    ground
        .map(|ground| surface_profile_for_entity(ground.entity, surfaces, support_colliders))
        .and_then(|surface| surface.inherit_velocity_policy)
        .unwrap_or(controller.support_velocity_policy)
}

fn effective_support_rotation_policy(
    controller: &CharacterController,
    ground: Option<GroundContact>,
    surfaces: &Query<&MovementSurface>,
    support_colliders: &SupportColliderQuery,
) -> SupportRotationPolicy {
    ground
        .map(|ground| surface_profile_for_entity(ground.entity, surfaces, support_colliders))
        .and_then(|surface| surface.inherit_rotation_policy)
        .unwrap_or(controller.support_rotation_policy)
}
