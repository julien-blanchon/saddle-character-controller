# Configuration

This document separates core runtime configuration from optional convenience modules.

## Core `CharacterController`

| Field | Type | Default | Practical Range | Effect |
| --- | --- | --- | --- | --- |
| `filter` | `SpatialQueryFilter` | `default()` | project-specific | Base query filter for casts and depenetration. Use it to exclude layers or categories globally. |
| `standing_view_height` | `f32` | `1.7` | `0.5..3.0` | Eye / camera height used by examples and by the optional environment helper when classifying depth. |
| `crouch_view_height` | `f32` | `1.2` | `0.3..standing_view_height` | Eye / camera height while crouched. |
| `speed` | `f32` | `12.0` | `2.0..30.0` | Base grounded move speed before sprint, crouch, global scale, and surface multipliers. |
| `sprint_speed_scale` | `f32` | `1.5` | `1.0..2.5` | Multiplier applied when sprint input is active. |
| `air_speed` | `f32` | `1.5` | `0.5..5.0` | Extra scale on the air-speed cap used by Quake-style air acceleration. |
| `max_air_wish_speed` | `f32` | `0.76` | `0.2..3.0` | Wish-speed cap used during air acceleration. Lower values reduce air gain and strafe-jump headroom. |
| `gravity` | `f32` | `29.0` | `9.8..60.0` | Base gravity magnitude outside helper-provided swim behavior. |
| `fall_gravity_multiplier` | `f32` | `1.0` | `1.0..3.0` | Extra gravity while descending. Useful for faster falls without changing jump takeoff. |
| `terminal_velocity` | `f32` | `50.0` | `10.0..200.0` | Clamp for downward speed. |
| `friction_hz` | `f32` | `12.0` | `0.0..20.0` | Ground friction applied before acceleration for Quake / Source feel. |
| `acceleration_hz` | `f32` | `8.0` | `2.0..20.0` | Ground acceleration rate. |
| `air_acceleration_hz` | `f32` | `12.0` | `0.0..30.0` | Air acceleration rate. Higher values enable stronger air strafing. |
| `stop_speed` | `f32` | `2.54` | `0.0..10.0` | Friction control floor. Larger values make small residual speeds die faster. |
| `jump_height` | `f32` | `1.8` | `0.2..5.0` | Desired jump apex height used to derive jump takeoff speed from gravity. |
| `coyote_time` | `Duration` | `100ms` | `0..250ms` | Grace window after losing ground during which jump still counts as grounded. |
| `jump_input_buffer` | `Duration` | `150ms` | `0..250ms` | Buffer window for jump presses that happen slightly before landing. |
| `ground_distance` | `f32` | `0.05` | `0.0..0.2` | Distance threshold for accepting a nearby ground hit as grounded. |
| `snap_distance` | `f32` | `0.1` | `0.0..0.4` | Downward snap allowance after grounded movement. Helps stairs and downhill stability. |
| `step_size` | `f32` | `0.7` | `0.1..1.2` | Maximum upward step probe for autostep. |
| `step_down_detection_distance` | `f32` | `0.2` | `0.05..0.8` | Downward probe distance used after step-up and during snap-to-ground checks. |
| `min_walk_angle` | `f32` radians | `40°` | `10°..60°` | Minimum walkable floor angle. Steeper surfaces become slide-only unless overridden. |
| `skin_width` | `f32` | `0.01` | `0.001..0.05` | Shape-cast skin width for move-and-slide and depenetration. |
| `capsule_radius` | `f32` | `0.4` | `0.2..1.0` | Standing and crouched capsule radius. |
| `capsule_half_height` | `f32` | `0.9` | `radius..2.0` | Standing capsule half-height. |
| `crouch_height` | `f32` | `1.3` | `radius*2..standing_height` | Total crouched capsule height. |
| `crouch_speed_scale` | `f32` | `0.33` | `0.1..1.0` | Move-speed multiplier while crouched. |
| `auto_bhop` | `bool` | `false` | `false` or `true` | Skips the usual ground-friction slowdown during the first grounded frame, enabling easier bhop chains. |
| `global_speed_scale` | `f32` | `1.0` | `0.1..3.0` | Global scalar applied to grounded move speed. Useful for accessibility or game-wide tuning. |
| `max_speed` | `f32` | `100.0` | `10.0..500.0` | Safety clamp on total velocity magnitude. |
| `unground_speed` | `f32` | `10.0` | `0.0..50.0` | Relative upward support speed above which the controller stops treating the support as stable ground. |
| `jump_cut_gravity_multiplier` | `f32` | `3.0` | `1.0..6.0` | Extra gravity applied when jump button is released during ascent. Enables variable-height jumps. |
| `max_air_jumps` | `u32` | `0` | `0..5` | Number of additional jumps allowed while airborne (double jump = 1, triple = 2). |
| `controller_mode` | `ControllerMode` | `Enabled` | `Enabled`, `SenseOnly`, `Disabled` | Controls whether the controller runs full simulation, probe-only sensing, or is completely disabled. |
| `slide_gravity_scale` | `f32` | `1.0` | `0.2..3.0` | Gravity multiplier while standing on a steep non-walkable surface. |
| `support_velocity_policy` | `SupportVelocityPolicy` | `Horizontal` | `None`, `Horizontal`, `Full` | Default platform-inheritance mode when the contacted surface does not override it. |
| `support_rotation_policy` | `SupportRotationPolicy` | `YawOnly` | `None`, `YawOnly` | Default rotating-platform inheritance mode. |
| `support_detach_grace` | `Duration` | `120ms` | `0..250ms` | Time window during which support velocity is preserved after ground loss. |

## Core Optional Components

### `CharacterFlying`

| Field | Type | Default | Practical Range | Effect |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `false` | `false` or `true` | Enables flying mode. Flying takes priority over grounded and swim-mode classification. |
| `speed` | `f32` | `14.0` | `2.0..40.0` | Base flight speed before sprint scaling. |
| `sprint_speed_scale` | `f32` | `1.4` | `1.0..3.0` | Multiplier applied while sprint input is active during flight. |
| `acceleration_hz` | `f32` | `8.0` | `1.0..20.0` | Flight acceleration rate toward the requested movement direction. |
| `drag_hz` | `f32` | `6.0` | `0.0..20.0` | Flight damping applied when input relaxes or direction changes. |
| `vertical_speed_scale` | `f32` | `1.0` | `0.0..3.0` | Scales ascend and descend input while flying. |
| `collision_mode` | `FlightCollisionMode` | `Slide` | `Slide`, `NoClip` | `Slide` uses the normal move-and-slide path; `NoClip` moves directly through geometry. |

### `CharacterMantle`

| Field | Type | Default | Practical Range | Effect |
| --- | --- | --- | --- | --- |
| `max_height` | `f32` | `1.0` | `0.2..2.0` | Highest ledge the mantle probe will try to clear. |
| `max_distance` | `f32` | `0.3` | `0.1..1.0` | Forward reach of the mantle wall probe. |
| `min_wall_angle` | `f32` radians | `50°` | `20°..80°` | Minimum wall steepness for a mantle candidate. |
| `speed` | `f32` | `5.0` | `1.0..15.0` | Pull-up travel speed toward the mantle target. |
| `pull_up_height` | `f32` | `0.3` | `0.0..1.0` | Extra upward offset applied after finding the top surface. |
| `input_buffer` | `Duration` | `60ms` | `0..200ms` | Buffered traverse input window used for mantle attempts. |

### `CharacterWallKick`

| Field | Type | Default | Practical Range | Effect |
| --- | --- | --- | --- | --- |
| `power` | `f32` | `0.9` | `0.1..2.0` | Scalar on the derived wall-kick launch impulse. |
| `upward_factor` | `f32` | `1.0` | `0.0..2.0` | Upward contribution added to the wall-kick launch direction. |
| `distance` | `f32` | `0.4` | `0.1..1.0` | Forward wall probe distance used to find a kickable wall. |
| `input_buffer` | `Duration` | `150ms` | `0..250ms` | Buffered jump input window for wall kicks. |
| `cooldown` | `Duration` | `300ms` | `0..1000ms` | Minimum time between consecutive wall kicks. |
| `max_wall_angle` | `f32` radians | `40°` | `20°..80°` | Maximum wall normal elevation for a kickable wall. |

### `CharacterDash`

| Field | Type | Default | Practical Range | Effect |
| --- | --- | --- | --- | --- |
| `speed` | `f32` | `28.0` | `10.0..50.0` | Velocity magnitude during a dash. |
| `duration` | `Duration` | `180ms` | `50..500ms` | How long the dash lasts. |
| `cooldown` | `Duration` | `400ms` | `0..2000ms` | Minimum time between consecutive dashes. |
| `cancel_gravity` | `bool` | `true` | `true` or `false` | Whether gravity is zeroed during the dash. |
| `max_air_dashes` | `u32` | `1` | `0..5` | Number of dashes allowed before landing. Reset on ground contact. |

### `CharacterGravity`

| Field | Type | Default | Practical Range | Effect |
| --- | --- | --- | --- | --- |
| `magnitude` | `f32` | `29.0` | `0.0..60.0` | Gravity strength for this entity, overriding `CharacterController::gravity`. |
| `direction` | `Vec3` | `-Y` | unit vector | Direction of gravity pull. Allows custom gravity directions per entity. |

### `CharacterPush`

| Field | Type | Default | Practical Range | Effect |
| --- | --- | --- | --- | --- |
| `impulse_scale` | `f32` | `5.0` | `0.0..20.0` | Linear impulse applied into dynamic rigidbodies when the controller hits them. |

### `CharacterLook`

| Field | Type | Default | Practical Range | Effect |
| --- | --- | --- | --- | --- |
| `yaw` | `f32` radians | `0.0` | any | Horizontal view rotation. |
| `pitch` | `f32` radians | `0.0` | clamped by `min_pitch` and `max_pitch` | Vertical view rotation. |
| `sensitivity` | `Vec2` | `(1.0, 1.0)` | `0.001..0.1` for mouse-like setups | Multiplier applied to look input before yaw/pitch integration. |
| `min_pitch` | `f32` radians | `-89°` | `-89°..0°` | Lower pitch clamp. |
| `max_pitch` | `f32` radians | `89°` | `0°..89°` | Upper pitch clamp. |

### `MovementSurface`

| Field | Type | Default | Practical Range | Effect |
| --- | --- | --- | --- | --- |
| `traction_multiplier` | `f32` | `1.0` | `0.0..2.0` | Scales ground friction. Lower values feel slippery. |
| `acceleration_multiplier` | `f32` | `1.0` | `0.0..2.0` | Scales grounded acceleration. |
| `speed_multiplier` | `f32` | `1.0` | `0.0..2.0` | Scales requested grounded move speed. |
| `jump_multiplier` | `f32` | `1.0` | `0.0..2.0` | Scales derived jump speed from this surface. |
| `conveyor_velocity` | `Vec3` | `Vec3::ZERO` | project-specific | Extra velocity applied from the surface itself. |
| `inherit_velocity_policy` | `Option<SupportVelocityPolicy>` | `None` | `None`, `Some(...)` | Per-surface override for support velocity inheritance. |
| `inherit_rotation_policy` | `Option<SupportRotationPolicy>` | `None` | `None`, `Some(...)` | Per-surface override for rotating-support inheritance. |
| `slide_only` | `bool` | `false` | `false` or `true` | Forces the surface to classify as non-walkable even if its normal passes the slope check. |

## Convenience Module: `convenience::environment`

These types are not installed by the core plugin. Add `CharacterControllerEnvironmentPlugin` or write your own detector systems.

### `CharacterSwimming`

| Field | Type | Default | Practical Range | Effect |
| --- | --- | --- | --- | --- |
| `acceleration_hz` | `f32` | `6.0` | `1.0..15.0` | Swim acceleration rate. |
| `gravity` | `f32` | `2.4` | `0.0..10.0` | Downward pull while swimming. |
| `slowdown` | `f32` | `0.6` | `0.1..1.0` | Scalar applied to requested move velocity before acceleration. |
| `ascent_speed_scale` | `f32` | `1.0` | `0.0..2.0` | Multiplier applied to upward swim input. |

### `EnvironmentVolume`

| Field | Type | Default | Practical Range | Effect |
| --- | --- | --- | --- | --- |
| `speed_multiplier` | `f32` | `1.0` | `0.0..2.0` | Multiplies requested move velocity inside the volume. |
| `acceleration_multiplier` | `f32` | `1.0` | `0.0..2.0` | Multiplies acceleration rate while the volume is active. |
| `gravity_multiplier` | `f32` | `1.0` | `0.0..2.0` | Multiplies gravity while the volume is active. |

### `SwimVolume`

| Field | Type | Default | Practical Range | Effect |
| --- | --- | --- | --- | --- |
| `speed_multiplier` | `f32` | `1.0` | `0.0..2.0` | Multiplies requested swim velocity inside the volume, including ascend movement. |
| `acceleration_multiplier` | `f32` | `1.0` | `0.0..2.0` | Multiplies `CharacterSwimming::acceleration_hz` while the volume is active. |
| `gravity_multiplier` | `f32` | `1.0` | `0.0..2.0` | Multiplies `CharacterSwimming::gravity` while the volume is active. |

## Convenience Module: `convenience::presets`

`CharacterControllerPreset` is an opinionated tuning catalog for demos and quick prototypes:

- `default_fps()`
- `platformer()`
- `explorer()`
- `arena()`

Treat those as starting points rather than part of the simulation contract.
