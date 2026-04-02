# Saddle Character Controller

Reusable 3D kinematic character controller for Bevy, built on `avian3d` and `bevy_enhanced_input`.

The crate is meant to stay generic. It does not know about any project state machine, game vocabulary, or camera setup. Consumers wire it into their own schedules, attach optional traversal components as needed, and order against the public system sets.

For always-on apps and examples, `CharacterControllerPlugin::always_on(FixedUpdate)` is the simplest entrypoint. For real games, prefer `CharacterControllerPlugin::new(...)` so activation and teardown stay aligned with your own state flow.

## Quick Start

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
use saddle_character_controller::*;

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
        .add_plugins(CharacterControllerPlugin::new(
            OnEnter(DemoState::Gameplay),
            OnExit(DemoState::Gameplay),
            FixedUpdate,
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

| Type | Purpose |
| --- | --- |
| `CharacterControllerPlugin` | Registers the controller runtime with injectable activate, deactivate, and update schedules |
| `CharacterControllerSystems` | Public ordering hooks: `ReadInput`, `PreMovement`, `Grounding`, `Movement`, `PostMovement`, `Presentation` |
| `CharacterController` | Core movement tuning and shape configuration |
| `CharacterControllerState` | Readable runtime state: mode, ground contact, support motion, crouch state, resolved water modifiers, and mantle state |
| `AccumulatedInput` | Buffered action state consumed by the movement pipeline |
| `CharacterLook` | Yaw/pitch and sensitivity state for camera-facing integrations |
| `CharacterMotionStats` | Debug-friendly runtime stats such as speed, grounded time, support entity, and cast count |
| `CharacterSwimming` | Optional swimming configuration |
| `CharacterMantle` | Optional mantle / ledge-pull configuration |
| `CharacterWallKick` | Optional wall-kick configuration |
| `CharacterPush` | Optional rigidbody push hook |
| `ExternalMotion` | Generic external velocity channel for wind, launchers, recoil, or gameplay impulses |
| `MovementSurface` | Per-surface traction, acceleration, speed, jump, conveyor, and inheritance overrides |
| `SupportVelocityPolicy` | Support inheritance mode: `None`, `Horizontal`, `Full` |
| `WaterVolume` / `WaterLevel` | Generic swim volume marker and runtime depth classification |
| Messages | `CharacterJumped`, `CharacterLanded`, `MovementModeChanged`, `SupportBodyChanged` |

## Current Feature Scope

Supported in v0.1:

- Quake / Source style ground acceleration, air acceleration, and friction ordering
- Coyote time and jump input buffering
- Capsule-based ground probing, slope classification, snap-to-ground, and step-up motion
- Moving-platform support with detach grace and per-surface inheritance override
- Crouch shape swap with uncrouch obstruction check
- Swimming volumes and vertical swim input
- Mantling and wall kicks as optional traversal layers
- Per-surface movement modifiers and conveyor velocity
- Push impulses into dynamic rigidbodies
- Runtime state reflection and optional debug gizmos

Deferred or intentionally minimal in v0.1:

- Ladder / climb-volume support
- Slide crouch and prone-style extra shape profiles
- Root-motion hooks and animation graph integration
- Custom gravity directions
- Built-in pickup / carry logic
- Deterministic serialization and prediction helpers

## Pipeline

The runtime is staged and orderable:

1. `ReadInput`
2. `PreMovement`
3. `Grounding`
4. `Movement`
5. `PostMovement`
6. `Presentation`

`bevy_enhanced_input` action events are evaluated in `PreUpdate`, then buffered into `AccumulatedInput`. The actual controller simulation can run on any injected schedule, including `FixedUpdate`.

## Examples

| Example | Purpose | Run |
| --- | --- | --- |
| `basic` | Flat ground, move, jump, crouch, and first-person look | `cargo run -p saddle-character-controller-example-basic` |
| `slopes_and_stairs` | Walkable vs non-walkable slopes, step-up, and snap behavior | `cargo run -p saddle-character-controller-example-slopes-and-stairs` |
| `moving_platforms` | Support inheritance, detach grace, and conveyor-like surfaces | `cargo run -p saddle-character-controller-example-moving-platforms` |
| `advanced_movement` | Auto-bhop, surf-friendly surfaces, debug draw, and higher-speed tuning | `cargo run -p saddle-character-controller-example-advanced-movement` |
| `traversal` | Mantle and wall-kick setup on a simple obstacle course | `cargo run -p saddle-character-controller-example-traversal` |
| `water` | Entering, swimming through, and exiting water volumes | `cargo run -p saddle-character-controller-example-water` |
| `third_person` | Generic third-person follow camera driven from controller state | `cargo run -p saddle-character-controller-example-third-person` |
| `stress_many_controllers` | Lightweight many-controller perf smoke | `cargo run -p saddle-character-controller-example-stress-many-controllers` |

## Workspace Lab

The standalone examples verify the shared crate in isolation. The workspace also includes a crate-local lab app for richer integration checks at
`shared/character/saddle-character-controller/examples/lab`:

```bash
cargo run -p saddle-character-controller-lab
```

## More Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
- [Movement Model](docs/movement-model.md)
- [Debugging](docs/debugging.md)
- [Performance](docs/performance.md)
