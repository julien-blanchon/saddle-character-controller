# Performance

## Hot Path

The controller hot path is dominated by shape casts and `move_and_slide` work. Pure math such as acceleration, friction, and timer checks is cheap.

The crate avoids per-frame hot-path allocation by:

- keeping contact scratch storage on `CharacterControllerScratch`
- reusing `MoveAndSlideConfig`
- storing runtime state on components instead of rebuilding transient structs

## Approximate Cast Budget Per Controller

These are rough per-fixed-tick numbers for the current implementation:

| Situation | Typical Cast Count |
| --- | --- |
| Flat grounded move | 2 to 4 |
| Grounded move with snap and support refresh | 4 to 6 |
| Step-up attempt | +3 |
| Mantle attempt | +3 |
| Wall-kick attempt | +1 |
| Overlap recovery | up to 1 depenetration pass |

`CharacterMotionStats.shape_casts_last_tick` exposes the observed count so you can validate real scenes instead of relying on estimates.

## Known Cost Drivers

- many simultaneous controllers
- geometry that forces frequent step retries
- debug gizmos enabled in large scenes
- very high fixed-update rates
- moving platforms with large contact counts

## Cheap Defaults

The current defaults are biased toward a reusable baseline:

- debug visuals are disabled by default
- there is no per-frame heap growth in the main movement path
- step logic uses bounded retries rather than arbitrary loops
- surface resolution is simple component lookup on the hit collider or owning body

## Profiling Guidance

Use the bundled `stress_many_controllers` example first:

```bash
cargo run -p saddle-character-controller-example-stress-many-controllers
```

Recommended measurements:

1. one controller on simple flat ground
2. ten controllers on simple flat ground
3. fifty controllers on mixed ramps and platforms

Track:

- frame time
- fixed-step time
- `CharacterMotionStats.shape_casts_last_tick`
- whether debug draw is disabled

## When To Optimize Further

Optimize only after measuring. The most likely next steps are:

- lowering fixed-step rate
- simplifying environment collision
- reducing step / mantle complexity in specific modes
- batching platform animation so support velocities stay coherent
- disabling debug visualization outside targeted diagnostics
