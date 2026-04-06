# Saddle Character Controller

Reusable 3D kinematic character controller for Bevy, built on `avian3d`.

The movement core is intentionally input-stack-agnostic. It owns simulation, state, and ordering hooks. Input mappings, swim-volume detection, and preset tuning now live in explicit adapter and convenience modules instead of being auto-installed by the core plugin.

## Quick Start

### Core-only dependency

```toml
[dependencies]
saddle-character-controller = { git = "https://github.com/julien-blanchon/saddle-character-controller", default-features = false }
bevy = "0.18"
avian3d = "0.6"
```

This gives you the controller runtime plus `AccumulatedInput`. Your game can write that component from any input stack it wants.

### With the provided enhanced-input adapter

```toml
[dependencies]
saddle-character-controller = { git = "https://github.com/julien-blanchon/saddle-character-controller" }
bevy = "0.18"
avian3d = "0.6"
bevy_enhanced_input = "0.24"
```

```rust
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use saddle_character_controller::{
    CharacterController,
    CharacterControllerPlugin,
    CharacterLook,
    adapters::enhanced_input::{
        AscendAction, CharacterControllerEnhancedInputPlugin, CrouchAction, JumpAction,
        LookAction, MoveAction, SprintAction, TraverseAction,
    },
    convenience::environment::{CharacterControllerEnvironmentPlugin, CharacterSwimming},
};

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DemoState {
    #[default]
    Gameplay,
}

fn main() {
    App::new()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .init_state::<DemoState>()
        .add_plugins((
            CharacterControllerPlugin::new(
                OnEnter(DemoState::Gameplay),
                OnExit(DemoState::Gameplay),
                FixedUpdate,
            ),
            CharacterControllerEnhancedInputPlugin,
            CharacterControllerEnvironmentPlugin::new(FixedUpdate),
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Player"),
        CharacterController::default(),
        CharacterLook {
            sensitivity: Vec2::splat(0.0025),
            ..default()
        },
        CharacterSwimming::default(),
        Transform::from_xyz(0.0, 2.0, 0.0),
        actions!(CharacterController[
            (
                Action::<MoveAction>::new(),
                DeadZone::default(),
                Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
            ),
            (
                Action::<LookAction>::new(),
                Bindings::spawn((
                    Spawn((Binding::mouse_motion(), Scale::splat(0.0025))),
                    Axial::right_stick().with((Scale::splat(0.06), DeadZone::default())),
                )),
            ),
            (Action::<JumpAction>::new(), bindings![KeyCode::Space, GamepadButton::South]),
            (Action::<SprintAction>::new(), bindings![KeyCode::ShiftLeft]),
            (Action::<CrouchAction>::new(), bindings![KeyCode::ControlLeft]),
            (Action::<TraverseAction>::new(), bindings![KeyCode::KeyE]),
            (Action::<AscendAction>::new(), bindings![KeyCode::KeyQ]),
        ]),
    ));
}
```

## Public API

### Core runtime

| Type | Purpose |
| --- | --- |
| `CharacterControllerPlugin` | Registers the simulation runtime with injectable activate, deactivate, and update schedules |
| `CharacterControllerSystems` | Public ordering hooks: `ReadInput`, `PreMovement`, `Grounding`, `Movement`, `PostMovement`, `Presentation` |
| `CharacterController` | Core movement tuning and collider configuration |
| `AccumulatedInput` | Generic buffered input state consumed by the movement pipeline |
| `CharacterControllerState` | Runtime state: movement mode, ground contact, support motion, crouch state, dash state, mantle state |
| `CharacterLook` | Yaw/pitch state for camera-facing integrations |
| `CharacterMotionStats` | Debug-friendly runtime stats such as speed, grounded time, support entity, and cast count |
| `CharacterFlying` | Optional flying / spectator configuration |
| `CharacterMantle` | Optional mantle / ledge-pull configuration |
| `CharacterWallKick` | Optional wall-kick configuration |
| `CharacterDash` | Optional dash ability |
| `CharacterGravity` | Optional per-entity gravity override |
| `CharacterPush` | Optional rigidbody push hook |
| `ExternalMotion` | Generic external velocity channel for gameplay impulses |
| `EnvironmentModifiers` | Runtime environment state: depth classification, active volume, and speed/acceleration/gravity multipliers |
| `ControllerMode` | Toggle controller behavior: `Enabled`, `SenseOnly`, or `Disabled` |
| `MovementSurface` | Per-surface traction, acceleration, speed, jump, conveyor, and inheritance overrides |
| Messages | `CharacterJumped`, `CharacterLanded`, `MovementModeChanged`, `SupportBodyChanged` |

### Adapter and convenience modules

| Module | Purpose |
| --- | --- |
| `adapters::enhanced_input` | Optional `bevy_enhanced_input` action schema plus `CharacterControllerEnhancedInputPlugin` |
| `convenience::environment` | Optional environment-volume detector plugin plus `CharacterSwimming`, `EnvironmentVolume`, and `SwimVolume` |
| `convenience::presets` | Opinionated demo/prototype presets such as `CharacterControllerPreset::platformer()` |

## Feature Scope

Core runtime:

- Quake / Source style ground acceleration, air acceleration, and friction ordering
- Coyote time and jump input buffering
- Variable-height jump and configurable air jumps
- Dash ability, flying, mantling, wall kicks, crouch shape swap
- Capsule-based grounding, snap-to-ground, step-up motion
- Moving-platform support with detach grace and optional yaw inheritance
- Per-surface movement modifiers and conveyor velocity
- Per-entity gravity override and external motion injection
- Runtime state reflection and optional debug gizmos

Optional convenience layers:

- `bevy_enhanced_input` mapping adapter
- Environment-volume detection into `EnvironmentModifiers`
- Swim-mode tuning and `SwimVolume`
- Preset tuning helpers for demos and quick prototypes

## Pipeline

The runtime is staged and orderable:

1. `ReadInput`
2. `PreMovement`
3. `Grounding`
4. `Movement`
5. `PostMovement`
6. `Presentation`

The core plugin only advances buffered timers and simulation. Adapter plugins are responsible for writing `AccumulatedInput`, and environment helpers are responsible for writing `EnvironmentModifiers`.

## Examples

| Example | Purpose | Run |
| --- | --- | --- |
| `basic` | Flat ground, move, jump, crouch, and first-person look | `cargo run -p saddle-character-controller-example-basic` |
| `slopes_and_stairs` | Walkable vs non-walkable slopes, step-up, and snap behavior | `cargo run -p saddle-character-controller-example-slopes-and-stairs` |
| `moving_platforms` | Support inheritance, detach grace, and conveyor-like surfaces | `cargo run -p saddle-character-controller-example-moving-platforms` |
| `advanced_movement` | Auto-bhop, surf-friendly surfaces, debug draw, and higher-speed tuning | `cargo run -p saddle-character-controller-example-advanced-movement` |
| `traversal` | Mantle and wall-kick setup on a simple obstacle course | `cargo run -p saddle-character-controller-example-traversal` |
| `water` | Entering, swimming through, and exiting a `SwimVolume` | `cargo run -p saddle-character-controller-example-water` |
| `third_person` | Generic third-person follow camera driven from controller state | `cargo run -p saddle-character-controller-example-third-person` |
| `stress_many_controllers` | Lightweight many-controller perf smoke | `cargo run -p saddle-character-controller-example-stress-many-controllers` |

Every standalone example includes a live `saddle-pane` debug panel and an on-screen control overlay.

## Workspace Lab

The workspace also includes a crate-local lab app for richer integration checks at `examples/lab`:

```bash
cargo run -p saddle-character-controller-lab
```

The lab integrates the controller with `saddle-character-state-machine` and `saddle-animation-ik`, exposes runtime diagnostics on-screen, and ships crate-local E2E scenarios.

## More Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
- [Movement Model](docs/movement-model.md)
- [Debugging](docs/debugging.md)
- [Performance](docs/performance.md)
