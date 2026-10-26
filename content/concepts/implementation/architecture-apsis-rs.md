---
title: Crate architecture
summary: The layer structure of the apsis crate, the modules in each layer, and the dependency rule that keeps them ordered.
tags: architecture, layers, modules
---

# Crate architecture

`apsis` is one crate with no dependency outside the standard library. Its
public surface is grouped into seven layers, each a directory under `src`, and
each layer is re-exported as a module of the crate:

```text
foundation -> physics -> orbit -> rf -> network -> ops -> sim
```

A module is reached through the crate root, so `apsis::apsis_sgp4` is the
propagator and `apsis::apsis_time` is the time model. The names keep the
subject they carried when the layers were separate crates, so a path quoted in
a paper, a bug report, or an earlier version of this documentation still
resolves.

## The layer rule

The rule is that a layer may use the layers to its left and no layer to its
right. It gives two properties that matter in practice:

- **A change does not propagate upward.** A new physical constant or a
  corrected unit conversion is made in one layer and every layer above it
  sees the change without edits.
- **A layer can be understood alone.** Reading the `orbit` layer requires
  reading `physics` and `foundation`, and requires reading nothing about
  radio links or scheduling.

The rule is checked by `scripts/check-layers.sh` in the library's repository,
so a module that refers to a layer to its right fails the check rather than
becoming a habit. `apsis_core` and `apsis_math` are the exceptions the rule
allows: both sit in `foundation` and every layer may use them.

## Layers and modules

| Layer | Modules | Responsibility |
|-------|---------|----------------|
| foundation | `apsis_core`, `apsis_units`, `apsis_math`, `apsis_macros` | Constants, errors, quantity newtypes, linear algebra |
| physics | `apsis_time`, `apsis_leap`, `apsis_coord`, `apsis_geodesy`, `apsis_frames` | Time scales, leap seconds, frames, ellipsoids |
| orbit | `apsis_tle`, `apsis_sgp4`, `apsis_kepler`, `apsis_ephemeris`, `apsis_maneuver`, `apsis_orbit` | Element parsing, propagation, two-body motion, interpolation, maneuvers |
| rf | `apsis_rf`, `apsis_antenna`, `apsis_doppler`, `apsis_atmosphere`, `apsis_noise`, `apsis_interference`, `apsis_link` | Radio links and antennas |
| network | `apsis_constellation`, `apsis_coverage`, `apsis_access`, `apsis_topology`, `apsis_isl`, `apsis_routing`, `apsis_handover`, `apsis_mesh` | Constellations and networks |
| ops | `apsis_resource`, `apsis_task`, `apsis_scheduler`, `apsis_conflict`, `apsis_schedule` | Resources, tasks, schedules |
| sim | `apsis_sim_engine`, `apsis_scenario`, `apsis_event`, `apsis_output`, `apsis_sim` | Discrete event simulation |

The module named after its layer, such as `apsis_orbit`, is the layer's entry
point: it re-exports what the other modules of the layer define and adds the
type that ties them together.

## Features

The crate has no default features. Each feature enables one layer and the
features of the layers below it.

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

This service enables `full`, because it documents every layer and its
interface catalog is extracted from all of them. The module graph in the
interface section is generated from that catalog, so it describes this build.

## Error strategy

Two kinds of error type exist.

`apsis_core` defines a general `Error` with a coarse `ErrorKind`. It exists
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

- The service presents this structure as a graph: `GET /api/graph/modules`
- The interface catalog: `GET /api/api`
