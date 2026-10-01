---
title: Workspace architecture
summary: The layer structure of the apsis-rs workspace, the crates in each layer, and the dependency rule that keeps them ordered.
tags: architecture, layers, crates
---

# Workspace architecture

`apsis-rs` is a Cargo workspace whose member crates are grouped into layers.
The grouping is enforced by a rule rather than by convention: a crate in one
layer may depend on crates in the layers to its left and on no crate to its
right. The published facade crate `apsis-rs` re-exports the layers and turns
each one into a Cargo feature.

```text
foundation -> physics -> orbit -> rf -> network -> ops -> sim
```

## The layer rule

The rule gives two properties that matter in practice:

- **A change does not propagate upward.** A new physical constant or a
  corrected unit conversion is made in one layer and every layer above it
  sees the change without edits.
- **A layer can be understood alone.** Reading the `orbit` layer requires
  reading `physics` and `foundation`, and requires reading nothing about
  radio links or scheduling.

A layer crate declares its dependencies explicitly, so the rule is visible
in each manifest and a violation fails the build rather than becoming a
habit.

## Layers and crates

| Layer | Crates | Responsibility |
|-------|--------|----------------|
| foundation | `apsis-core`, `apsis-units`, `apsis-math`, `apsis-macros` | Constants, errors, quantity newtypes, linear algebra |
| physics | `apsis-time`, `apsis-leap`, `apsis-coord`, `apsis-geodesy`, `apsis-frames` | Time scales, leap seconds, frames, ellipsoids |
| orbit | `apsis-tle`, `apsis-sgp4`, `apsis-kepler`, `apsis-ephemeris`, `apsis-maneuver`, `apsis-orbit` | Element parsing, propagation, two-body motion, interpolation, maneuvers |
| rf | `apsis-rf`, `apsis-antenna`, `apsis-doppler`, `apsis-atmosphere`, `apsis-noise`, `apsis-interference`, `apsis-link` | Radio links and antennas |
| network | `apsis-constellation`, `apsis-coverage`, `apsis-access`, `apsis-topology`, `apsis-isl`, `apsis-routing`, `apsis-handover`, `apsis-mesh` | Constellations and networks |
| ops | `apsis-resource`, `apsis-task`, `apsis-scheduler`, `apsis-conflict`, `apsis-schedule` | Resources, tasks, schedules |
| sim | `apsis-sim-engine`, `apsis-scenario`, `apsis-event`, `apsis-output`, `apsis-sim` | Discrete event simulation |

## Features of the facade

The facade crate has no default features. Each feature enables one layer and
the features of the layers below it.

| Feature | Enables |
|---------|---------|
| `foundation` | foundation |
| `physics` | foundation, physics |
| `orbit` | foundation, physics, orbit |
| `rf` | everything above, plus rf |
| `network` | everything above, plus network |
| `ops` | everything above, plus ops |
| `sim` | every layer, including sim |
| `full` | an alias for `sim` |

This service builds against `orbit`, which is the smallest feature set that
contains the element, propagation, time, and geometry kernels it needs. The
crate structure graph shown in the interface section is generated from the
catalog, so it describes the enabled set rather than the whole workspace.

## Error strategy

Two kinds of error type exist.

`apsis-core` defines a general `Error` with a coarse `ErrorKind`. It exists
so that a function whose failure has no interesting structure can report it
without inventing a type. It is deliberately **not** the error type of the
layers.

A layer with a domain specific failure defines its own error enum. The
element parser has `ElementErrorKind` with variants such as `Checksum` and
`TooShort`; the propagator has `PropagationErrorKind` with variants such as
`InvalidElements` and `Decayed`. A caller matches on the variant, so the
variant names name the conditions a caller would want to distinguish. Each
error also carries optional detail text for a message, and implements
`Display` and `std::error::Error`.

## See also

- The service presents this structure as a graph: `GET /api/graph/crates`
- The interface catalog: `GET /api/api`
