# Architecture

## Overview

`saddle-character-controller` keeps a stable movement core and layers optional traversal features on top of it. The core entity owns:

- immutable-ish tuning in `CharacterController`
- runtime state in `CharacterControllerState`
- buffered actions in `AccumulatedInput`
- derived debug metrics in `CharacterMotionStats`

Optional abilities are additive components:

- `CharacterSwimming`
- `CharacterMantle`
- `CharacterWallKick`
- `CharacterPush`

That keeps the base crate useful for a plain FPS or exploration controller without forcing every consumer into every advanced feature.

## Update Pipeline

The public pipeline is exposed through `CharacterControllerSystems`:

1. `ReadInput`
2. `PreMovement`
3. `Grounding`
4. `Movement`
5. `PostMovement`
6. `Presentation`

Within that flow, the runtime currently does:

1. Tick buffered jump / traverse timers.
2. Initialize newly added controllers and refresh active collider shapes.
3. Classify water depth from overlapping `WaterVolume` sensors.
4. Depenetrate stale overlaps.
5. Probe ground using the active capsule.
6. Resolve support-body velocity and detach grace.
7. Resolve crouch shape transitions.
8. Expire stale buffered actions.
9. Apply movement-mode-specific acceleration, friction, gravity, jump, mantle, wall-kick, and slide motion.
10. Re-probe ground after motion and update support attachment.
11. Publish movement messages and clear per-frame look accumulation.
12. Draw optional debug gizmos in `PostUpdate`.

## Input Model

The crate uses `bevy_enhanced_input` action events and stores the results in `AccumulatedInput`.

Important detail:

- action evaluation happens in `PreUpdate`
- controller simulation can run in any injected schedule, typically `FixedUpdate`
- buffered timers allow jump and traverse input to survive between render-rate and fixed-rate updates

This is why the crate can stay reusable even though `bevy_enhanced_input` registers contexts against compile-time schedules while the controller plugin uses runtime-provided `Interned<dyn ScheduleLabel>` activation and simulation schedules.

## Simulation vs Presentation

The controller itself only owns logical movement state:

- current movement mode
- support velocity and support entity
- water level
- mantle target
- view orientation

The crate does not impose a specific camera or render-body architecture. Consumers can:

- attach a first-person camera directly from `CharacterLook` and the configured eye height
- drive a third-person follow camera from `CharacterControllerState::orientation`
- attach an animated mesh child that follows the logical controller entity
- interpolate a separate visual proxy if they want stricter fixed-step presentation

Examples in this crate show both first-person and third-person wiring without changing the controller runtime itself.

## Grounding Model

Grounding is shape-based rather than ray-only.

The active collider profile is the standing or crouched capsule from `CharacterColliderCache`. Ground classification uses:

- a downward shape cast from the active body
- walkability derived from `min_walk_angle`
- optional `MovementSurface::slide_only` override
- post-move snap logic for downhill stability
- step-up and step-down probes for natural stairs and curb handling

`CharacterControllerState::ground` stores the last accepted contact, including:

- entity
- point
- normal
- distance
- walkable classification

## Moving Platforms

Support-body handling is a first-class part of the runtime:

- the current support entity is derived from the accepted ground hit
- support velocity is computed from the platform's linear and angular velocity at the contact point
- support angular velocity is exposed separately in runtime state for live inspection and downstream hooks
- `MovementSurface::conveyor_velocity` is added on top
- `SupportVelocityPolicy` selects full, horizontal-only, or no inheritance
- a detach grace window preserves momentum briefly after leaving the platform

Per-surface override is supported through `MovementSurface::inherit_velocity_policy`.

## Ability Layering

The controller stays extensible by keeping optional features in their own components:

- `CharacterSwimming` enables swim-mode acceleration, gravity, and ascend input
- `CharacterMantle` enables buffered ledge traversal attempts
- `CharacterWallKick` enables wall-based jump redirects
- `CharacterPush` enables contact-driven rigidbody impulses
- `ExternalMotion` provides a generic velocity-delta channel for gameplay code

This pattern keeps the public API configuration-driven instead of accumulating a single monolithic movement struct.
