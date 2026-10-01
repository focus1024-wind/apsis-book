---
title: Ephemeris sampling and interpolation
summary: When to sample a track once and query it many times, and what the interpolation costs in accuracy.
tags: ephemeris, interpolation, sampling, hermite
---

# Ephemeris sampling and interpolation

A coverage search needs the position of a satellite at many instants. Asking
the propagator for each one is correct and slow. Sampling the track once and
interpolating between the samples is fast and introduces a small, bounded
error. This document gives the trade.

## Definition

An **ephemeris** is a table of satellite states at a sequence of instants.
A **sample** holds the time, the position, and the velocity at one instant.

**Interpolation** evaluates the state between two samples without calling
the propagator, using the two samples that bracket the requested instant.

## Mathematical meaning

The interpolation is a cubic Hermite polynomial per component. It uses the
position at both endpoints and the velocity at both endpoints, so it matches
the value and the first derivative at the ends of every interval:

```text
p(s) = h00(s) p0 + h10(s) dt v0 + h01(s) p1 + h11(s) dt v1
```

where `s` is the fraction along the interval in `[0, 1]`, `dt` is the
interval length, and

```text
h00 =  2 s^3 - 3 s^2 + 1
h10 =      s^3 - 2 s^2 + s
h01 = -2 s^3 + 3 s^2
h11 =      s^3 -   s^2
```

are the Hermite basis polynomials. The velocity is the derivative of the
same polynomial with respect to time.

Because the velocity is used at both ends, the interpolation is `C^1`: the
position and the velocity are continuous across a sample boundary. A table
built from position alone would have a discontinuous velocity at every
sample, which is visible in a plot as a kink and is wrong for anything that
differentiates the track.

The interpolation error over one interval scales as the fourth power of the
interval length times the fourth derivative of the position. That is the
quantity that sets the sample spacing.

## Unit convention

- Sample times are in **seconds** from whatever origin the caller chose. A
  sampler defined against a propagator usually takes its times in the
  propagator's own unit, minutes from an element epoch, and converts to
  seconds for the table.
- Position is in **kilometers** and velocity in **kilometers per second**,
  the units of the propagator that produced the table.

## Applicability

The interpolation is valid inside the interval the table covers. Outside it,
the polynomial extrapolates and the error grows quickly; a query outside the
covered range returns no value unless the caller explicitly asks for the
nearest sample.

The error is bounded by the sample spacing. A useful rule for a low Earth
orbit, where the period is about 90 minutes:

| Sample spacing | Interpolation error |
|----------------|---------------------|
| 60 s | Below a meter |
| 300 s | A few meters |
| 600 s | Tens of meters |
| 1800 s | Hundreds of meters |

These values are for a smooth low Earth orbit. An orbit with a high
eccentricity, where the angular rate changes sharply at perigee, needs a
shorter spacing near perigee or a uniform spacing set by the perigee rate.

The table also stores the velocity, so it can answer a query for the
velocity as well as the position. A table built from positions alone cannot,
and would need a finite difference, which is less accurate.

## Comparison of query strategies

| Strategy | Cost per query | Accuracy | Memory | When it applies |
|----------|----------------|----------|--------|-----------------|
| Propagate on demand | One propagation | The model's own | None | A few thousand queries |
| Uniform table with Hermite interpolation | Tens of operations | Set by the spacing | One record per sample | Many queries over a known interval |
| Non-uniform table with tight spacing at perigee | Tens of operations | Set by the local spacing | More samples near perigee | A high eccentricity orbit |
| Chebyshev polynomial per arc | Tens of operations | Set by the degree and the arc | A coefficient set per arc | A long smooth arc |
| Two-body propagation from osculating elements | Very low | Degrades fast because the model is incomplete | None | A rough check only |

Chebyshev approximation is used by precise ephemeris products and offers a
tighter error for a given number of coefficients than a uniform table. It is
inconvenient for a table built directly from a propagator, because the
coefficients are fitted to the arc rather than read off it, which is why a
uniform table is the usual form for a table derived from propagation.

## Source

- Cubic Hermite interpolation is the standard two point, two derivative
  polynomial; the basis above is the usual form.
- The use of an interpolated table for access and coverage searches is the
  standard approach in mission analysis, as described in Vallado,
  *Fundamentals of Astrodynamics and Applications*.

## See also

- [[sgp4]] for the propagator that produces the samples
- [[propagator-comparison]] for the cost of propagating against
  interpolating
- Try the sampler in the online tool: *Sample a ground track*
