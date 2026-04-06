# Debugging

## What To Inspect First

When movement feels wrong, check these components in order:

1. `CharacterController`
2. `CharacterControllerState`
3. `AccumulatedInput`
4. `CharacterMotionStats`
5. `LinearVelocity`

That usually tells you whether the problem is:

- input never arriving
- grounding classifying the floor differently than expected
- support velocity being inherited unexpectedly
- a shape profile mismatch during crouch
- a tuning issue rather than a logic issue

## Common Failure Modes

| Symptom | Likely Cause | What To Inspect |
| --- | --- | --- |
| No movement at all | `AccumulatedInput` never changes, your input adapter is missing, or the simulation schedule is not running | `AccumulatedInput`, your adapter plugin, your injected schedule wiring |
| Jump press feels lost | buffer too short or simulation running less often than expected | `AccumulatedInput.jump_pressed_for`, `jump_input_buffer`, fixed timestep |
| Character slides on expected floor | `min_walk_angle` too strict or `MovementSurface.slide_only = true` | `CharacterControllerState.ground`, floor normal, `MovementSurface` |
| Platform carry feels wrong | wrong `SupportVelocityPolicy` or detach grace too long/short | `CharacterControllerState.support_velocity`, `SupportBodyChanged`, `support_detach_grace` |
| Cannot stand back up | blocked uncrouch due to geometry overhead | `CharacterControllerState.crouching`, nearby collider layout |
| Swim volume never activates | missing `CharacterControllerEnvironmentPlugin`, missing `SwimVolume`, missing `CharacterSwimming`, or sensor overlap not happening | `EnvironmentModifiers`, `CollidingEntities`, swim collider setup |
| Mantle never triggers | traversal action not buffered or ledge top not walkable | `AccumulatedInput.traverse_pressed_for`, `CharacterMantle`, target surface normal |

## BRP Workflows

If your app or sandbox exposes BRP, these commands are useful:

```bash
brp world query saddle_character_controller::CharacterController
brp world query saddle_character_controller::CharacterControllerState
brp world query saddle_character_controller::AccumulatedInput
brp world query saddle_character_controller::CharacterMotionStats
brp extras screenshot /tmp/character-controller.png
```

Recommended inspection order during a live session:

1. confirm the entity exists and has the expected optional components
2. inspect `AccumulatedInput` while pressing keys or moving the stick
3. inspect `CharacterControllerState.ground`, `support_velocity`, and `movement_mode`
4. inspect `CharacterMotionStats.shape_casts_last_tick` if perf or probing logic looks suspicious

## Debug Gizmos

The crate exposes `CharacterControllerDebugDraw` as an opt-in resource. When enabled, the current implementation draws:

- current linear velocity
- ground normal
- support velocity

Recommended extensions in app space if you need more detail:

- mantle target point
- step probe volumes
- current wish direction
- surface-profile color coding

## Schedule And Ordering Checks

The crate exposes `CharacterControllerSystems` explicitly so you can order your own systems around it.

Typical integrations:

```rust
app.configure_sets(
    FixedUpdate,
    MyGameplaySystems::AnimatePlatforms.before(CharacterControllerSystems::Grounding),
);

app.configure_sets(
    FixedUpdate,
    CharacterControllerSystems::PostMovement.before(MyGameplaySystems::FireFootsteps),
);
```

If moving platforms or camera follow behavior looks one frame late, ordering is the first thing to verify.

## Workspace Lab

For richer runtime debugging in this workspace, prefer the dedicated crate-local lab app under
`shared/character/saddle-character-controller/examples/lab`:

```bash
cargo run -p saddle-character-controller-lab
```

It keeps the runtime generic while exposing an overlay with support motion, cast counts, environment modifiers, and buffered inputs.
