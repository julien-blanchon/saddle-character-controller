# Saddle Character Controller

Reusable 3D kinematic character controller for Bevy, built on `avian3d`.

The controller is a **pure body simulation** — it handles physics, grounding, movement, and collisions. It does **not** own orientation or camera logic. Orientation is a `Quat` on `CharacterControllerState` that external code (a camera controller, AI, or your game) writes to.

Abilities (dash, flying, mantling, wall-kick, swimming) are **opt-in plugins** — add only what your game needs.

## Quick Start

```rust
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use saddle_character_controller::{
    CharacterController,
    CharacterControllerPlugin,
    CharacterFlying,
    adapters::enhanced_input::{
        CharacterControllerEnhancedInputPlugin, CrouchAction, JumpAction,
        MoveAction, SprintAction,
    },
    abilities::{
        flying::CharacterControllerFlyingPlugin,
        dash::CharacterControllerDashPlugin,
    },
};

fn main() {
    App::new()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_plugins((
            // Core body simulation
            CharacterControllerPlugin::always_on(FixedUpdate),
            CharacterControllerEnhancedInputPlugin,
            // Pick only the abilities you need:
            CharacterControllerFlyingPlugin,
            CharacterControllerDashPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Player"),
        CharacterController::default(),
        CharacterFlying::default(), // only because we added FlyingPlugin
        Transform::from_xyz(0.0, 2.0, 0.0),
        actions!(CharacterController[
            (
                Action::<MoveAction>::new(),
                DeadZone::default(),
                Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
            ),
            (Action::<JumpAction>::new(), bindings![KeyCode::Space, GamepadButton::South]),
            (Action::<SprintAction>::new(), bindings![KeyCode::ShiftLeft]),
            (Action::<CrouchAction>::new(), bindings![KeyCode::ControlLeft]),
        ]),
    ));
    // Orientation: write to CharacterControllerState::orientation from your camera or input system.
}
```

## Architecture

### Core Plugin

`CharacterControllerPlugin` provides the body simulation frame:

| System Set | Purpose |
| --- | --- |
| `ReadInput` | Tick input buffers |
| `PreMovement` | Shape refresh, controller init |
| `Grounding` | Environment detection (opt-in) |
| `MovementPrepare` | Depenetrate, ground probe, support, crouch, input expiry |
| `MovementExecute` | Ability plugins and core movement run here |
| `MovementFinalize` | Post-ground probe, snap, stats, mode classify |
| `PostMovement` | Push forces, collider sync, messages |
| `Presentation` | Debug draw |

### Ability Plugins (opt-in)

| Plugin | Component | Purpose |
| --- | --- | --- |
| `CharacterControllerFlyingPlugin` | `CharacterFlying` | Spectator/flight mode |
| `CharacterControllerDashPlugin` | `CharacterDash` | Direction-locked burst movement |
| `CharacterControllerMantlePlugin` | `CharacterMantle` | Ledge climbing |
| `CharacterControllerWallKickPlugin` | `CharacterWallKick` | Wall jump |
| `CharacterControllerSwimmingPlugin` | `CharacterSwimming` | Water movement |

Add the plugin, attach the component to your entity, done.

### Custom Movement

Write your own systems in `MovementExecute`. Set `MovementOverride::active` to suppress core movement when your custom logic is in control:

```rust
fn grapple_system(
    mut q: Query<(&mut LinearVelocity, &mut MovementOverride, &MyGrapple)>,
) {
    for (mut vel, mut mov, grapple) in &mut q {
        if grapple.active {
            mov.active = Some("grapple");
            mov.suppress_gravity = true;
            vel.0 = grapple.direction * grapple.speed;
        } else {
            mov.active = None;
        }
    }
}
```

### Core Types

| Type | Purpose |
| --- | --- |
| `CharacterController` | Movement tuning: speed, gravity, jump, crouch, step size, etc. |
| `AccumulatedInput` | Buffered input: move_axis, jump, sprint, crouch |
| `CharacterControllerState` | Runtime state: orientation, ground contact, movement mode, crouching |
| `MovementOverride` | Ability/custom movement takeover flag |
| `CharacterMotionStats` | Read-only stats: speed, grounded time, support entity |
| `MovementMode` | `Grounded`, `Airborne`, `Sliding`, `Custom(u8)` |
| `ExternalMotion` | External velocity injection |
| `CharacterGravity` | Per-entity gravity override |
| `CharacterPush` | Push dynamic bodies on contact |
| Messages | `CharacterJumped`, `CharacterLanded`, `MovementModeChanged`, `SupportBodyChanged` |

### Orientation

The controller does **not** own look/orientation. `CharacterControllerState::orientation` is a `Quat` that determines movement direction. External code writes it:

- **FPS camera**: use `saddle-camera-fps-camera` with `FpsCameraExternalMotion`, sync `runtime.yaw/pitch` → `state.orientation`
- **Third person**: write orientation from your camera's yaw
- **AI**: set orientation toward the target

### Adapter & Convenience

| Module | Purpose |
| --- | --- |
| `adapters::enhanced_input` | `bevy_enhanced_input` action schema + plugin |
| `convenience::environment` | Environment volume detector + `SwimVolume` |

## Examples

| Example | Purpose |
| --- | --- |
| `basic` | Flat ground, walk, jump, crouch, FPS camera |
| `slopes_and_stairs` | Slope limits, step-up, snap behavior |
| `moving_platforms` | Platform support, detach grace, conveyor surfaces |
| `advanced_movement` | Auto-bhop, surf-friendly surfaces, debug draw |
| `traversal` | Mantle and wall-kick obstacle course |
| `water` | SwimVolume entry, swimming, exit |
| `third_person` | Third-person follow camera |
| `stress_many_controllers` | Many-controller perf smoke test |
| `lab` | Full integration with animation state machine and IK |

Run with: `cargo run -p saddle-character-controller-example-basic`
