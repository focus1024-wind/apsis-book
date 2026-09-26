---
id: propagator-comparison
title: Choosing a propagator
chapter: propagation
summary: A comparison of the propagation methods, with the cost and the accuracy each one buys.
tags: propagation, comparison, sgp4, hpop
---

# Choosing a propagator

More than one method answers the question "where is the satellite". They
differ in what they require as input, in what they cost, and in how long the
answer stays true. This document compares them and gives the decision rule
that follows.

## The comparison

| Method | Accuracy | Computational cost | Input required | Reference | Implemented in apsis-rs |
|--------|----------|--------------------|----------------|-----------|-------------------------|
| Two-body Keplerian | Exact for the model; error grows without bound against the real orbit | Very low: one evaluation of Kepler's equation | Osculating elements and a state | Kepler, standard treatment in Vallado | Yes |
| SGP4 and SDP4 | About 1 km over the fit interval, degrading with time and solar activity | Very low: closed form evaluation | Two-line element set | Vallado et al. 2006 | Yes |
| Numerical integration with `J2` only | Error grows without bound because drag and third body are absent | Moderate: an integration step | Initial state and a gravity model | Textbook models | No |
| HPOP, numerical integration of a high order field | Meters over days, decimeters with a good force model and a good initial state | High: an integration per step, several steps per minute of arc | Precise initial state, force model, and station data | Each operator's own model | No |
| Interpolated table from a propagator | Bounded by the interpolation error of the table | Low per query, high to build the table | A table already sampled | Standard interpolation | Yes, for interpolation of an existing track |
| Machine learning correction of SGP4 | Improved within the fit interval, unbounded outside it | Moderate per query, high to train | TLE plus a trained model and its training data | Current research | No |

## How to read the table

The accuracy column is not comparable across rows without its interval. A
method that is exact for its own assumptions can be far from the real orbit,
and a method with a large numerical cost can be no better than its input.
The honest comparison is **error against the real trajectory as a function
of the interval from the reference epoch**, and that curve depends on the
altitude and on the activity of the Sun.

Two statements that hold generally:

- SGP4 is the best available model for a two-line element set, because the
  elements are defined by a fit to it. A more expensive model given the same
  elements is **worse**, not better, because it answers a different question.
- A high precision model is only as good as its initial state. Without a
  precise orbit determination from tracking data, it will not reach the
  accuracy its force model would allow.

## The decision rule

| Requirement | Use |
|-------------|-----|
| Reproduce a catalogue position | SGP4 with the matching gravity model |
| Inspect one state or plan an impulsive maneuver | The two-body model on the osculating state |
| Query a track many times over a short interval | Sample once, then interpolate; see [[ephemeris-interpolation]] |
| Predict over weeks | SGP4, with a stated error that grows and a check against a fresher element set |
| Sub-meter accuracy over days | A precise orbit determination and a force model propagator, which are outside this workspace |
| A rough geometry check | The two-body model, with the understanding that it drifts |

## Cost in practice

The cost column in the table is relative between methods, not absolute. Some
orders of magnitude, for a single evaluation on a modern processor:

| Method | Cost per evaluation | Notes |
|--------|---------------------|-------|
| Two-body | Tens of floating point operations | One Newton iteration for Kepler's equation |
| SGP4 | A few hundred floating point operations | Closed form, no iteration beyond the element preparation |
| SDP4 | A few thousand | The resonance and luni-solar terms add work |
| Numerical integration | Milliseconds | Thousands of evaluations of the force model per day of propagation |
| Interpolation | Tens of operations | A cubic Hermite evaluation per component |

The ratio between SGP4 and a numerical integration is several orders of
magnitude, which is why a coverage or access search uses an interpolated
table rather than the propagator directly.

## Source

- SGP4 accuracy and the model: Vallado, Crawford, Hujsak, and Kelso,
  *Revisiting Spacetrack Report #3*, AIAA 2006-6753, 2006.
- Force models and their relative size: Vallado, *Fundamentals of
  Astrodynamics and Applications*.

## See also

- [[sgp4]] for the model the element sets are fitted to
- [[perturbations]] for what each model includes
- [[ephemeris-interpolation]] for the table approach
- [[keplerian-elements]] for the two-body model
