# Movement Model

## Ground Acceleration

Ground movement follows the Quake / Source convention:

1. apply friction
2. compute wish direction from the current view orientation and move axis
3. accelerate toward that wish direction

That ordering matters. Friction-before-accel makes small counter-strafes and low-speed corrections feel more responsive than accel-before-friction.

The helper used by the crate is:

```text
current_speed_along_wish = dot(velocity, wish_dir)
add_speed = capped_wish_speed - current_speed_along_wish
accel_speed = min(wish_speed * acceleration_hz * dt, add_speed)
velocity += wish_dir * accel_speed
```

This allows the controller to keep the classic “gain speed by changing heading” behavior without overshooting along the current wish direction.

## Air Acceleration And Strafe Gain

Air control uses the same acceleration rule but with a separate cap:

- `air_acceleration_hz`
- `max_air_wish_speed * air_speed`

Because the cap is applied to speed along the wish direction instead of total speed, a player can still gain total velocity through air strafing. That is the same core reason bunnyhopping and strafe-jumping work in Quake-derived movement.

## Friction

Ground friction is applied as:

```text
control = max(speed, stop_speed)
drop = control * friction_hz * traction_multiplier * dt
new_speed = max(speed - drop, 0)
velocity *= new_speed / speed
```

Important consequences:

- it never reverses the velocity vector
- `stop_speed` makes tiny residual velocities die quickly
- `MovementSurface::traction_multiplier` changes how slippery the floor feels
- `auto_bhop` skips the usual first grounded friction pass so chained jumps retain more speed

## Jump Derivation

Jump takeoff speed is derived from desired height and gravity:

```text
jump_speed = sqrt(2 * gravity * jump_height)
```

That keeps jump tuning readable:

- change `jump_height` when you want a different apex
- change `gravity` when you want a different arc shape and global fall feel
- use `fall_gravity_multiplier` when you want faster descents without changing takeoff

`MovementSurface::jump_multiplier` scales the derived speed on a per-surface basis.

## Coyote Time And Input Buffering

The runtime favors recent intent over frame-perfect timing:

- `coyote_time` allows grounded jumps briefly after leaving the ground
- `jump_input_buffer` preserves a jump press until the controller can legally consume it
- `CharacterMantle::input_buffer` and `CharacterWallKick::input_buffer` do the same for traversal actions

Input buffering is timer-based rather than boolean-based so fixed-step and render-step timing stay predictable.

## Ground And Support Classification

Grounding is based on a downward shape cast of the active capsule profile.

The hit becomes walkable when:

- the floor normal satisfies `normal.y >= cos(min_walk_angle)`
- the surface is not marked `slide_only`

The accepted contact is stored in `CharacterControllerState::ground` and drives:

- movement mode classification
- support velocity inheritance
- snap-to-ground
- grounded vs airborne timers

## Moving Platforms

Support velocity comes from the contacted body's motion at the contact point:

- linear velocity
- angular velocity contribution around the support body's center of mass
- optional `MovementSurface::conveyor_velocity`

Inheritance is then filtered by `SupportVelocityPolicy`:

- `None`
- `Horizontal`
- `Full`

If the controller leaves the support, the inherited velocity can persist for `support_detach_grace`.

## Water Volumes

Swimming activates when the controller has `CharacterSwimming` and the resolved `WaterLevel` rises above `Feet`.

The final swim motion uses two layers:

- the actor-level `CharacterSwimming` values for baseline acceleration, slowdown, ascent, and gravity
- the active `WaterVolume` multipliers for speed, acceleration, and gravity

Those resolved multipliers are stored on `CharacterControllerState` so debugging tools can see the exact environment contribution that was active on the last tick.

## Slopes, Sliding, And Surf-Like Surfaces

Slope behavior has two layers:

- normal-based walkability from `min_walk_angle`
- explicit per-surface override from `slide_only`

Steep surfaces remain valid collision surfaces, but they stop counting as grounded walkable support. That enables:

- normal sliding
- surf-style ramps when acceleration and air tuning are permissive
- separate `slide_gravity_scale` tuning for how aggressively the controller falls along steep slopes

## Step And Snap Logic

The controller first attempts direct move-and-slide motion. If forward motion is blocked, it tries:

1. upward step probe by `step_size`
2. forward move from the raised position
3. downward recovery by `step_down_detection_distance + snap_distance`

If the stepped result is not better than the direct result, the direct move wins.

After grounded motion, the runtime also performs a downward snap using `snap_distance + step_down_detection_distance` for downhill stability.
