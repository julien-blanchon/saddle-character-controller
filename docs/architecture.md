# Architecture

## Overview

`saddle-character-controller` now has three layers:

1. Core runtime
   - simulation, state, messages, and ordering hooks
   - consumes `AccumulatedInput`
   - consumes `EnvironmentModifiers`
2. Input adapters
   - write `AccumulatedInput`
   - example: `adapters::enhanced_input::CharacterControllerEnhancedInputPlugin`
3. Convenience helpers
   - write `EnvironmentModifiers`
   - provide opinionated presets or demo-oriented setup
   - example: `convenience::environment::CharacterControllerEnvironmentPlugin`

That keeps the controller reusable in projects that do not use `bevy_enhanced_input`, do not want built-in swim-volume detection, or want their own tuning catalog.

## Core Entity Layout

The core entity owns:

- `CharacterController`
- `CharacterControllerState`
- `AccumulatedInput`
- `CharacterMotionStats`

Optional runtime abilities remain additive:

- `CharacterFlying`
- `CharacterMantle`
- `CharacterWallKick`
- `CharacterDash`
- `CharacterGravity`
- `CharacterPush`
- `ExternalMotion`

Swim-mode tuning now lives in `convenience::environment::CharacterSwimming` instead of the root API surface.

## Update Pipeline

The public pipeline is exposed through `CharacterControllerSystems`:

1. `ReadInput`
2. `PreMovement`
3. `Grounding`
4. `Movement`
5. `PostMovement`
6. `Presentation`

Within that flow, the core runtime currently does:

1. Tick buffered jump / traverse / dash timers.
2. Initialize newly added controllers and refresh active collider shapes.
3. Depenetrate stale overlaps.
4. Probe ground using the active capsule.
5. Resolve support-body velocity and detach grace.
6. Resolve crouch shape transitions.
7. Expire stale buffered actions.
8. Apply movement-mode-specific acceleration, friction, gravity, jump, flying, mantle, wall-kick, dash, and slide motion.
9. Re-probe ground after motion and update support attachment.
10. Publish movement messages and clear per-frame look accumulation.
11. Draw optional debug gizmos in `PostUpdate`.

Environment detection is no longer auto-installed by the core plugin. If you use the provided helper plugin, it schedules its volume classification in `CharacterControllerSystems::Grounding`.

## Input Model

The core plugin never assumes a concrete input stack.

The contract is:

- some upstream system writes into `AccumulatedInput`
- the controller consumes that buffer on its injected simulation schedule
- buffered timers let jump / traverse / dash input survive between render-rate updates and fixed-rate simulation

The optional enhanced-input adapter keeps the old behavior, but it is explicit now:

- add `adapters::enhanced_input::CharacterControllerEnhancedInputPlugin`
- attach `actions!(CharacterController[..])` to entities that should receive those bindings

Projects using another stack can skip that plugin and write `AccumulatedInput` directly.

## Environment Model

`EnvironmentModifiers` is still the bridge between movement and world context:

- `depth`
- `active_volume`
- `speed_multiplier`
- `acceleration_multiplier`
- `gravity_multiplier`

The movement core reads those resolved modifiers, but it does not decide how they are produced.

The convenience environment plugin provides one implementation:

- `EnvironmentVolume` for generic speed / acceleration / gravity multipliers
- `SwimVolume` for swim-mode depth classification when `CharacterSwimming` is present

Projects can replace that plugin with their own detector systems without changing the controller runtime.

## Simulation vs Presentation

The controller owns logical state only:

- current movement mode
- support velocity and support entity
- environment depth and modifiers
- air jump count and dash state
- mantle target
- view orientation

The crate does not impose a camera or render-body architecture. Consumers can:

- attach a first-person camera from `CharacterLook` and the configured eye height
- drive a third-person follow camera from `CharacterControllerState::orientation`
- attach an animated mesh child that follows the logical controller
- interpolate a separate visual proxy if they want stricter fixed-step presentation

## Moving Platforms

Support-body handling remains a first-class part of the runtime:

- accepted ground hits define the current support entity
- support velocity is computed from the platform's linear and angular motion at the contact point
- `MovementSurface::conveyor_velocity` layers on top
- `SupportVelocityPolicy` selects full, horizontal-only, or no inheritance
- `SupportRotationPolicy` selects whether yaw from rotating supports is inherited
- `support_detach_grace` preserves momentum briefly after leaving the platform

Per-surface overrides are supported through `MovementSurface::inherit_velocity_policy` and `MovementSurface::inherit_rotation_policy`.
