# Character Controller Lab

Crate-local standalone lab app for inspecting the shared `saddle-character-controller` crate in a real Bevy application.

## Purpose

- verify that the shared crate works in a real workspace app rather than only as a standalone example
- expose controller state, support motion, water modifiers, input buffers, and cast counts through an on-screen overlay
- keep moving-platform, slope, stair, water, and pushable-object coverage available without polluting the shared crate runtime surface

## Status

Working

## Run

```bash
cargo run -p saddle-character-controller-lab
```

## Notes

- The lab keeps the scene generic: one controller, slopes, stairs, a moving platform, a conveyor strip, a water volume, and a pushable crate.
- The overlay text shows movement mode, current ground/support target, current speed, cast count, water modifiers, and buffered input timers.
