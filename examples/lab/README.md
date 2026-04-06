# Character Controller Lab

Crate-local standalone lab app for inspecting the shared `saddle-character-controller` crate in a real Bevy application, including a controller + state-machine + IK integration pass wired through Git dependencies.

## Purpose

- verify that the shared crate works in a real workspace app rather than only as a standalone example
- exercise the cross-crate 3D character stack: controller locomotion, animation-state selection, and IK look targeting in one scene
- expose controller state, support motion, environment modifiers, input buffers, and cast counts through an on-screen overlay
- keep moving-platform, slope, stair, water, and pushable-object coverage available without polluting the shared crate runtime surface

## Status

Working

## Run

```bash
cargo run -p saddle-character-controller-lab
```

## E2E

```bash
cargo run -p saddle-character-controller-lab --features e2e -- controller_smoke
cargo run -p saddle-character-controller-lab --features e2e -- controller_platform_rotation
cargo run -p saddle-character-controller-lab --features e2e -- controller_flying_noclip
```

## Notes

- The lab keeps the scene generic: one controller, slopes, stairs, a moving platform, a conveyor strip, a swim volume, and a pushable crate.
- The overlay text shows movement mode, current ground/support target, current speed, cast count, environment modifiers, buffered input timers, animation state/binding, and IK solve error.
- The live `saddle-pane` UI exposes controller tuning plus look-target distance and weight so the integration can be tweaked without recompiling.
