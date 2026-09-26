---
id: keplerian-elements
title: Keplerian elements
chapter: elements
summary: The six classical orbital elements, the two-body solution, and the anomalies.
tags: orbit, kepler, elements, anomalies
---

# Keplerian elements

Six numbers describe an elliptical orbit around a central body when only the
gravity of that body acts. They are the classical orbital elements, and they
are the form in which an orbit is easiest to inspect and to plan a maneuver
against. This document defines them and the conversions.

## Definition

| Element | Symbol | Meaning | Unit |
|---------|--------|---------|------|
| Semi-major axis | `a` | Size of the ellipse | km |
| Eccentricity | `e` | Shape, `0` for a circle, below `1` for an ellipse | dimensionless |
| Inclination | `i` | Tilt of the orbit plane against the equator | rad |
| Right ascension of the ascending node | `Omega` | Where the orbit crosses the equator going north | rad |
| Argument of perigee | `omega` | Angle from the ascending node to perigee, in the orbit plane | rad |
| True anomaly | `nu` | Angle from perigee to the satellite | rad |

The first two give the shape and size. The second two give the orientation
of the plane. The last two give the position of the perigee within the plane
and of the satellite along the orbit.

The orbit is an ellipse with one focus at the center of the Earth, as
Kepler's first law states. The point closest to the Earth is perigee and the
farthest is apogee.

## Mathematical meaning

The radius as a function of the true anomaly follows from the conic
equation:

```text
r = p / (1 + e cos(nu)),    p = a (1 - e^2)
```

where `p` is the semi-latus rectum. The velocity follows from the vis-viva
equation:

```text
v^2 = mu (2 / r - 1 / a)
```

where `mu` is the gravitational parameter of the Earth.

Two derived quantities appear often:

```text
perigee radius r_p = a (1 - e)
apogee radius  r_a = a (1 + e)
period         T   = 2 pi sqrt(a^3 / mu)      (Kepler's third law)
```

The conversion to a Cartesian state places the satellite in the perifocal
frame, whose first axis points to perigee and whose third axis is along the
orbit normal:

```text
r = p / (1 + e cos nu)
x = r cos(nu),  y = r sin(nu),  z = 0
vx = -sqrt(mu / p) sin(nu)
vy =  sqrt(mu / p) (e + cos(nu))
```

and then rotates by the argument of perigee, the inclination, and the right
ascension of the ascending node.

### The three anomalies

Three angles name a position along the orbit. They agree at perigee and at
apogee and differ in between.

| Anomaly | Symbol | Defined as | Geometric meaning |
|---------|--------|------------|-------------------|
| True anomaly | `nu` | Angle at the focus from perigee | Actual direction of the satellite |
| Eccentric anomaly | `E` | Angle at the center of the ellipse | Auxiliary angle of the projecting circle |
| Mean anomaly | `M` | `M = E - e sin(E)` | Angle a fictitious body on a circular orbit would have |

The relations are

```text
tan(nu / 2) = sqrt((1 + e) / (1 - e)) tan(E / 2)
M = E - e sin(E)
```

`M = E - e sin(E)` is Kepler's equation. It is transcendental, so `E` is
recovered from `M` by iteration; the implementation uses Newton's method
with a fixed iteration budget and a tolerance, which converges quickly for
the eccentricities an Earth orbit has. The equation has no closed form
solution, and any implementation that claims one is approximating.

The eccentric anomaly is the quantity most computations use internally,
because both conversions above are simple in it.

## Unit convention

- The semi-major axis is in **kilometers**.
- Angles in the interface are in **radians**. A tool that accepts degrees
  converts on the input boundary and reports both.
- The eccentricity is **dimensionless**.
- The gravitational parameter `mu` is in **cubic kilometers per second
  squared**.

## Applicability

The two-body solution assumes a point mass and a point mass, with no other
force. An Earth orbit is affected by the oblateness of the Earth, the drag
of the atmosphere, the gravity of the Sun and the Moon, and the pressure of
sunlight. Each of these moves the satellite away from the ellipse.

The Keplerian elements computed from a state vector at one instant are
therefore the elements of the **osculating** orbit, the ellipse that touches
the true trajectory at that instant. They change along the orbit. They are
not the mean elements of a two-line element set, and converting one to the
other is a different operation from propagating either.

The two-body solution is exact for its assumptions and is the right model
for:
- Inspecting an instantaneous state.
- Planning an impulsive maneuver, where the change is applied to the
  osculating state.
- Approximate period and geometry estimates.

It is the wrong model for reproducing a catalogue position. Use [[sgp4]].

## Comparison of element descriptions

| Description | Input | Model | Changes along the orbit | Used here |
|-------------|-------|-------|-------------------------|-----------|
| Mean elements | Two-line element set | SGP4 | No, until refit | As the propagator input |
| Osculating elements | A state vector | Two-body | Yes | As an output of the propagator |
| Position and velocity | Any propagator | Any | Yes | Internally |
| Equinoctial elements | A state vector | Two-body | Yes | No |
| Position on a time grid | Any propagator | Any | Yes | Interpolated, see [[ephemeris-interpolation]] |

Equinoctial elements avoid the singularity that the classical elements have
at zero inclination and at zero eccentricity. The classical form is used
here because that is the form the two-line element format and the published
literature use, and because an Earth orbit almost always has a nonzero
inclination.

## Source

- The two-body solution and the anomalies follow the standard treatment in
  Vallado, *Fundamentals of Astrodynamics and Applications*.
- The element conversion follows the formulation in the same reference.

## See also

- [[tle-format]] for the mean elements the propagator consumes
- [[sgp4]] for the propagator and its relation to these elements
- [[perturbations]] for what the two-body model leaves out
- Try the conversion in the online tool: *Keplerian elements to a state
  vector*
