---
title: Constellations
tags: constellation, shell, planes, phasing, coverage
summary: Why a constellation is an arithmetic problem about planes, phasing and shell, and what the arithmetic forces the design to be.
---

# Constellations

A constellation is a set of satellites in planes chosen so that their
combined coverage is what the service needs. The parameters that matter are
few, the constraints are many, and the design is the intersection.

## Definition

A constellation is described by:

- The number of **planes** and their **inclination**.
- The number of satellites **per plane** and their phasing within it.
- The **shell**, meaning the altitude or semi-major axis.
- The **walk**, the relative motion between adjacent planes, which keeps the
  pattern from sitting still relative to the Earth.

Coverage is the fraction of the surface that sees at least one satellite at
a given elevation, at a given instant or averaged over a period. The two are
different numbers and a design that meets the instantaneous figure can be
poorly covered at any given place and time.

## Mathematics

### The geometry of a shell

At altitude `h`, the Earth subtends a half angle from the satellite, and the
ground area visible above an elevation angle `e` is

```text
A(e) = 2 pi R^2 ( 1 + cos(e) )
```

For `e = 0`, the visible cap is a hemisphere. The fraction of the globe a
satellite can see at once is therefore fixed by its altitude alone, and no
amount of satellites improves it beyond covering more of that hemisphere at
the same time.

### Cost per unit coverage

The cost of a constellation scales with the number of satellites, and the
number needed for a given coverage scales roughly with the area to be
covered divided by the area one satellite covers at that elevation. Two
consequences follow:

- A high shell covers more per satellite and costs more per satellite to
  place, and the balance sets the altitude.
- The expensive part of a shell is launch, and launches carry a limited
  number of satellites, which is what makes plane count a launch-rate
  decision as much as a coverage one.

### Phasing and the seam

Satellites in a plane spread along it at equal intervals. If the phasing is
not chosen well, the satellites align over the poles or over the equator in
a way that leaves a seam of poor coverage. Walker delta describes the
phasing; a delta that is not coprime with the satellite count produces
repeats.

### Walk

A relative motion between planes rotates the whole pattern. A sun-synchronous
orbit has a nodal drift of about one degree per day, which is what keeps the
planes aligned with the terminator. The walk is therefore not decoration:
it is a service requirement, and it costs propellant.

## Unit conventions

- Altitude above the equatorial radius, kilometres.
- Inclination in degrees when describing a constellation, because that is how
  shells are quoted, and in radians inside any calculation.
- Phasing as an integer offset.

## Applicability

The design point is set by three competing constraints: coverage at low
elevation pushes to a high shell, which pushes to fewer satellites; revisit
time for the ground segment pushes to more planes, which pushes to more
launches; and the propagation medium of the band in use pushes some of it
below the rain-limited altitudes.

The hard constraint is often the physical one rather than the coverage one: a
satellite cannot occupy a shell that the link budget or the propagation
environment forbids, and a design that ignores this is a design that
degrades in the worst weather.

Coverage figures are also easy to misread. Averaged over the service area,
a constellation may have excellent coverage; the failure is that a
particular user at a particular time is below the elevation angle, which is
an availability question rather than a coverage one.

## Standards

- Constellation filings for the geostationary and non-geostationary regimes:
  the Radio Regulations, Annex 4
- Phase and coverage conventions are not standardised; they are engineering
  practice.

## See also

- [[access-and-availability]] for the question coverage does not answer
- [[solar-radiation-pressure]] for why a plane drifts
- [[link-budget]] for which shells are usable