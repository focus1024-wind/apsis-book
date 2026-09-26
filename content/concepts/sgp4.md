---
id: sgp4
title: SGP4 and SDP4 propagation
chapter: propagation
summary: The analytical propagator that two-line element sets are fitted to, its two branches, and its frame and units.
tags: sgp4, sdp4, propagation, tle
---

# SGP4 and SDP4 propagation

SGP4 is the analytical orbit propagator that two-line element sets are
fitted to. It takes mean elements and returns a position and a velocity at a
requested instant. It is the only way to reproduce the published position of
a catalogued satellite, because the mean elements are defined by the fit to
this model.

## Definition

The name stands for Simplified General Perturbations, model 4. The model is
a closed form analytical propagation: it evaluates the secular and long
period effects of the perturbations that dominate the orbit, rather than
integrating the equations of motion.

Two branches exist and the propagator selects between them by orbital
period:

| Branch | Condition | Additional terms |
|--------|-----------|------------------|
| SGP4 | Period below 225 minutes | Secular oblateness and drag |
| SDP4 | Period at or above 225 minutes | Resonance and luni-solar perturbations |

The threshold is about 5,877 kilometers altitude for a near-circular orbit.
Below it the atmosphere and the oblateness dominate and the Sun and Moon do
not; above it the reverse is true and the resonance with the Earth's gravity
field matters.

## Mathematical meaning

The propagation begins by recovering the size and shape of the orbit from
the mean motion, which requires undoing the correction that was applied when
the elements were fitted:

```text
a1 = (xke / n0)^(2/3)
delta1 = 1.5 k2 (3 cos^2(i0) - 1) / (a1^2 (1 - e0^2)^(3/2))
a0 = a1 (1 - delta1/3 - delta1^2 - 134/81 delta1^3)
delta0 = 1.5 k2 (3 cos^2(i0) - 1) / (a0^2 (1 - e0^2)^(3/2))
n0' = n0 / (1 + delta0)         (the un-Kozai mean motion)
a0'' = a0 / (1 - delta0)
```

The drag term `BSTAR` produces a secular decay of the semi-major axis and
the eccentricity, and the oblateness produces secular rates in the argument
of perigee, the right ascension, and the mean anomaly. The propagation then
evaluates the position from the corrected mean anomaly.

The deep space branch adds the resonance with the Earth's gravity field for
orbits whose period is a simple fraction of the Earth's rotation, and the
long period perturbations from the Sun and Moon.

The implementation follows the algorithm of the paper and the revised
reference implementation distributed with it, including its treatment of the
drag coefficient and of the `isimp` simplification used for very low
perigee.

## Unit convention

**Inputs** are the mean elements from a two-line element set. The format
writes the orientation angles in degrees and the mean motion in revolutions
per day; the propagator converts to radians and radians per minute
internally. The remaining units are the ones the format uses: `BSTAR` in
inverse Earth radii, the derivatives in revolutions per day to the matching
power.

**Outputs** are in the True Equator Mean Equinox frame:

- Position in **kilometers**.
- Velocity in **kilometers per second**.

The gravitational parameter and the Earth radius used internally are the
ones of the chosen gravity model, in **cubic kilometers per second squared**
and **kilometers**.

**Time** is measured from the element set epoch, in **minutes**.

## Applicability

The model is a fit, and its accuracy is bounded by the accuracy of the
elements it is given. Over the interval around the epoch for which the
elements were fitted, which is days, the position error against the true
orbit is typically about one kilometer. The error grows with the interval,
because the drag term is a single number that cannot capture the variation
of the atmospheric density, which depends on the activity of the Sun.

Atmospheric drag is the dominant error source for a low Earth orbit and the
error grows fastest when the Sun is active. A satellite below about 500
kilometers altitude may have a propagation interval of only a day or two
before the error exceeds a few kilometers. A satellite in a high orbit has
a much longer useful interval.

The model is not valid for:
- A satellite that has decayed. The implementation reports this rather than
  returning a position below the surface.
- Elements whose eccentricity is one or above, which do not describe a
  closed orbit and are not parseable by the SGP4 element preparation.
- Intervals far from the epoch, where the error is unbounded and the model
  gives no estimate of it.

The frame is TEME, not the J2000.0 inertial frame. Converting to an
Earth-fixed frame needs the Earth rotation and the equation of the
equinoxes, which is the small angle between the mean and true equinoxes and
which SGP4 does not model. See [[reference-frames]].

## Comparison of propagators

| Method | Accuracy over days | Computational cost | Input | Reference | Implemented in apsis-rs |
|--------|--------------------|--------------------|-------|-----------|-------------------------|
| SGP4 and SDP4 | About 1 km, degrading with time and solar activity | Very low, closed form | Two-line element set | Vallado 2006 | Yes |
| HPOP, numerical integration of a high order field | Meters over days, decimeters with a good force model | High, a numerical integration per step | Precise initial state and force model | Each operator's own model | No |
| Numerical integration of the two-body plus J2 | Grows without bound because the model is incomplete | Moderate | State and gravity model | Standard textbook models | No |
| Two-body Keplerian propagation | Grows without bound because the model is incomplete | Very low | Osculating elements | Kepler | Yes, as `apsis-kepler` |
| General perturbations with a different fit | Depends on the fit and the element source | Low | Matching element set | Each model's own reference | No |
| Machine learning correction of SGP4 | Improved over the fit interval | Moderate, needs training data | TLE plus a trained model | Current research | No |

The last two rows are outside the scope of the workspace and are listed
because a user comparing methods will encounter them.

## Source

- Vallado, Crawford, Hujsak, and Kelso, *Revisiting Spacetrack Report #3*,
  AIAA 2006-6753, 2006, and the revised implementation distributed with that
  paper.
- Gravity model constants: WGS 72 values as used by that implementation,
  which match the WGS 72 gravity field. WGS 84 is offered for element sets
  fitted against it, from NIMA TR8350.2.

## See also

- [[tle-format]] for the input
- [[keplerian-elements]] for the osculating elements the state converts to
- [[perturbations]] for what the model includes and leaves out
- [[propagator-comparison]] for the wider comparison
- Try the propagator in the online tool: *Propagate with SGP4*
