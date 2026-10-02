---
title: Solar radiation pressure
tags: radiation-pressure, srp, sun-synchronous, photon, area-to-mass
summary: The acceleration a sunlit surface receives from photon momentum, why it is a century scale effect rather than a perturbation, and how it shapes a constellation.
---

# Solar radiation pressure

A satellite is a body held together, not a body falling. The force that keeps
a small object from accelerating away, or from being pulled back in, is the
momentum carried by the light it reflects and absorbs. It is usually ignored
in a low Earth orbit model and dominant in a high one.

## Definition

**Solar radiation pressure** is the momentum flux carried by sunlight at a
distance `r` from the Sun. It is not a force on a satellite but on the light
it intercepts; the acceleration the satellite feels is the flux multiplied
by the satellite's area-to-mass ratio and by a reflectivity factor.

The pressure at one astronomical unit is about `4.56e-6 N/m^2`. Divided by
the area-to-mass ratio of a spacecraft, it gives an acceleration of the same
order as `J_2` on a large satellite at low altitude.

## Mathematics

### The flux

The flux falls as the inverse square of the distance,

$$
P(r) = P_{1\,\text{au}} \left( \frac{1\,\text{au}}{r} \right)^2
$$

with `P_1au = 4.56e-6 N/m^2` for total solar irradiance at 1 au.

### The acceleration

For a satellite with projected area `A`, mass `m`, and a coefficient
`C_R` that is 1 for absorption, 2 for perfect reflection and between for a
typical surface, the acceleration is

$$
a_{\text{srp}} = C_R \frac{P_{1\,\text{au}} A}{m} \left( \frac{1\,\text{au}}{r} \right)^2
$$

The `A/m` ratio is the whole story. It ranges from about `0.002 m^2/kg` for a
large communications satellite to about `0.05` for a small cube of a
CubeSat, a factor of 25 that is entirely attributable to structure.

### Three components

The acceleration along the Sun-satellite line has three distinguishable
parts, because the force acts where the light strikes:

- A **constant radial component**, which slightly lowers the effective
  gravitational parameter and is absorbed into it.
- A **transverse component** along the orbit normal or the velocity
  direction, which produces a secular change in the mean motion, that is in
  the semi-major axis.
- A **short-period variation** with the orbital period, because the geometry
  changes over an orbit when the satellite has a nonzero area ratio.

Only the middle one changes the orbit over time; the first is a constant and
the third averages out.

### Eclipse

In eclipse the satellite receives no radiation pressure. The effect is a
periodic reduction of the transverse acceleration, which produces a
cyclical variation in semi-major axis with the eclipse season rather than the
orbit, and it is why a radiation pressure constant is calibrated from the
mean over a long arc.

## Unit conventions

- Flux in watts per square metre; `P_1au = 1361 W/m^2` for total solar
  irradiance, which corresponds to the pressure above.
- Area-to-mass ratio in square metres per kilogram.
- Distances in kilometres; the `1 au` ratio is dimensionless so the unit
  does not matter there, but the acceleration that results must be in
  kilometres per second squared to be consistent with the orbit.

## Applicability

The effect is a century-scale concern in the semi-major axis. For a high
Earth orbit with a large area-to-mass ratio, the semi-major axis drifts
noticeably over a few years, which is why the planning of a constellation in
the geosynchronous or high Earth orbit includes radiation pressure in its
station-keeping budget. For a low Earth orbit it is a perturbation beside
`J_2` and atmospheric drag.

The mean motion acceleration from the transverse component is what a model
should capture. The short-period term is smaller than it looks because it
averages, but for a sun-synchronous orbit it is not a perturbation at all,
because it is the mechanism: the nodal rate is chosen by tuning it against
`J_2`, and the two balance.

The eclipse term matters only where the radiation pressure is large, and it
matters because it is biased rather than symmetric.

## Standards

- Solar irradiance: IAU 2015 Resolution B3
- The pressure and flux conversions follow that resolution
- The effect as modelled in practice is described in Vallado,
  *Fundamentals of Astrodynamics and Applications*

## See also

- [[perturbations]] for the size of the effect beside `J_2` and drag
- [[atmospheric-drag]] for the other century-scale term
- [[constellations]] for why a sun-synchronous plane is chosen by balance