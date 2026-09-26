---
id: perturbations
title: Orbital perturbations
chapter: propagation
summary: The forces that make an orbit deviate from an ellipse, and which of them each model includes.
tags: orbit, perturbations, drag, oblateness, third-body
---

# Orbital perturbations

A Keplerian orbit is an ellipse that repeats forever. A real orbit does not.
The forces besides the point mass gravity of the Earth deform the ellipse
and change its orientation. This document lists the forces and says which of
them each model in the workspace includes.

## Definition

A perturbation is any acceleration on the satellite other than the central
point mass gravity of the Earth. The main ones, in rough order of magnitude
for a low Earth orbit, are:

| Perturbation | Source | Typical size at low Earth orbit | Character |
|--------------|--------|--------------------------------|-----------|
| Oblateness, `J2` | The Earth is not a sphere | `1e-3` of central gravity | Secular in the node and perigee |
| Atmospheric drag | Residual atmosphere | `1e-7` to `1e-4` of central gravity, depending on altitude and solar activity | Secular decay of the orbit |
| Third body, Sun and Moon | Gravity of the Sun and Moon | `1e-6` at low Earth orbit, larger at high altitude | Long period |
| Solar radiation pressure | Photon momentum | `1e-9` to `1e-8` of central gravity | Long period, depends on area and mass |
| Higher zonals and tesserals | The Earth's gravity field | Diminishing series | Secular and resonant |
| Relativity | General relativity | `1e-9` of central gravity at low Earth orbit | Secular in the node |
| Tides and solid Earth tides | The Earth deforms | Below the above | Long period |

The list is ordered by the size of the effect on a low Earth orbit. The
order changes with altitude: above a few thousand kilometers the third body
terms exceed drag, and the resonance with the Earth's gravity field becomes
the dominant long period effect.

## Mathematical meaning

The central gravity is

```text
a_central = -mu r / |r|^3
```

The `J2` term, which models the equatorial bulge, is

```text
a_J2 = -1.5 J2 mu R^2 / r^5 *
       [ x (1 - 5 z^2/r^2), y (1 - 5 z^2/r^2), z (3 - 5 z^2/r^2) ]
```

with `R` the equatorial radius and `z` measured along the rotation axis. It
has no component that changes the energy, so the size of the orbit is
unaffected to first order; what it changes is the orientation. Its secular
effects are

```text
dOmega/dt = -1.5 n J2 (R/a)^2 cos(i) / (1 - e^2)^2
domega/dt =  0.75 n J2 (R/a)^2 (5 cos^2(i) - 1) / (1 - e^2)^2
```

The first makes a low inclination orbit precess slowly westward and a polar
orbit precess in right ascension. Both are large enough to matter within a
day.

Drag is not a conservative force. It removes energy and angular momentum,
which lowers the semi-major axis and the eccentricity. Its size depends on
the atmospheric density, which depends strongly on altitude and on the
activity of the Sun. SGP4 models the effect with a single fitted number, the
`BSTAR` term, rather than with a density model.

Third body gravity and solar radiation pressure are first order effects on
an orbit at high altitude, where they cannot be modelled by a single fitted
number, and are treated by numerical integration in a force model propagator.

## Unit convention

- Accelerations are in **kilometers per second squared** where a force model
  is used, and are dimensionless ratios against the central gravity in this
  document for readability.
- The zonal coefficients such as `J2` are **dimensionless**, and the
  reference radius they are scaled by is in **kilometers**.
- The `BSTAR` term is in **inverse Earth radii**.

## Applicability

The choice of which perturbations to include follows from the altitude and
the interval, not from a rule:

- **A few hours, anywhere.** The central gravity and `J2` suffice for
  everything except a precise range.
- **Days, low Earth orbit.** Drag must be included, and its uncertainty is
  the dominant error.
- **Days, geostationary orbit.** Third body gravity and solar radiation
  pressure must be included; drag is negligible.
- **Weeks or more.** The uncertainty of any model exceeds the effect it is
  trying to capture, and the answer should be treated as a prediction with
  an error that grows.

SGP4 makes the choice for the caller by fitting the perturbations it models
to the element set. That is its strength and its limit: it always agrees
with the elements, and it cannot represent a force the fit did not include.

## Comparison of what each model includes

| Perturbation | Two-body | SGP4 | SDP4 | Force model integration |
|--------------|----------|------|------|-------------------------|
| Central gravity | Yes | Yes | Yes | Yes |
| `J2` oblateness | No | Yes, secular | Yes, secular | Yes, full field |
| Drag | No | Yes, one fitted term | Yes, one fitted term | Yes, with a density model |
| Third body | No | No | Yes, long period | Yes |
| Solar radiation pressure | No | No | No | Yes |
| Resonance with the Earth's field | No | No | Yes | Yes, in the field |
| Higher zonals and tesserals | No | No | Partial | Yes |
| Relativity | No | No | No | Optional |

## Source

- The force models and their orders of magnitude follow Vallado,
  *Fundamentals of Astrodynamics and Applications*.
- The SGP4 treatment of oblateness and drag follows Vallado et al.,
  *Revisiting Spacetrack Report #3*, AIAA 2006-6753, 2006.

## See also

- [[sgp4]] for the model that includes the first two
- [[keplerian-elements]] for the orbit without any perturbation
- [[propagator-comparison]] for the methods side by side
