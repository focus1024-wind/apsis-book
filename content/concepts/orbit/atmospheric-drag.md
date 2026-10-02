---
title: Atmospheric drag and the density model
tags: drag, atmosphere, density, ballistic-coefficient, sun-solar-activity
summary: Why the top of the atmosphere is the hardest part of an orbit model, how density is estimated, and what the activity of the Sun does to the answer.
---

# Atmospheric drag and the density model

Drag is the only perturbation that removes energy from an orbit. Everything
else trades altitude against inclination or direction; drag trades altitude
against lifetime, and it is why most low Earth orbits end.

## Definition

**Drag** is the aerodynamic force opposite the velocity. Its magnitude is

$$
F = -\tfrac{1}{2} \rho v^2 C_d A
$$

with $\rho$ the atmospheric density, $v$ the speed, `Cd` a drag coefficient of
order one, and $A$ the projected area. What matters to the orbit is the
combination

$$
B = \frac{C_d A}{m}
$$

the inverse of the **ballistic coefficient**, and $B$ is what a drag model
actually needs.

Above about 700 km the density is small enough that the force is comparable
to other perturbations, and it varies by orders of magnitude over the
altitude range of the transition region, which is the source of most of the
difficulty.

## Mathematics

### Density and altitude

The density falls by about a factor of ten for every 15 km of altitude near
500 km, and the profile is not a power law but a sum of exponential layers
separated by inversions. Around 700 to 1000 km there is a region where the
density is a factor of several higher than an exponential fit predicts, and a
satellite can gain altitude while moving through it rather than losing it.

### Temperature and the scale height

In an isothermal layer the density falls exponentially with a scale height

$$
H = \frac{k T}{m g}
$$

with $T$ the temperature, which itself depends on the solar activity. A warmer
thermosphere means a larger scale height, a higher density at every altitude,
and faster decay.

### Solar activity

The thermosphere is heated by the Sun, and the heating follows the solar
ultraviolet flux. At solar maximum the density at 400 km can be a factor of
several above its minimum, and the decay rate of a satellite in the
transition region changes by the same factor. This is why an ephemeris fitted
over one solar cycle does not transfer to another.

### Effect on the orbit

The effect on the semi-major axis is a secular decay, with a rate that scales
with $\rho / m$ and with the inverse of the semi-major axis cubed, because the
orbit speed falls with altitude. The practical consequence is that the decay
rate is not constant over the life of the orbit; a satellite that starts in a
dense layer decays quickly and then slows as it sinks into a thinner one.

### Analytic models

A usable model expresses the density as an exponential sum in altitude, with
exponents and coefficients that depend on solar activity, latitude and day or
night. Two models are common enough to name: the exponential model used by
the SGP4 theory, which has a closed form and therefore a fitted coefficient,
and the standard atmosphere models such as US-76, which are tables and more
accurate but not invertible in closed form.

## Unit conventions

- Density in kilograms per cubic metre; grams per cubic centimetre is the
  unit a data product is more likely to use.
- Area-to-mass ratio in square metres per kilogram.
- Ballistic coefficient as area-to-mass, not the reciprocal that some sources
  use.
- Altitude in kilometres.

## Applicability

A drag model is the weakest part of most orbit models. It is weakest at
exactly the altitude where it matters most, in the transition region between
700 and 1000 km, where a factor of two in density is a factor of two in the
predicted lifetime and the density itself is uncertain by that much.

A model fitted to a specific satellite over a specific arc is good for that
satellite over that arc and should not be extrapolated to a different area to
mass, a different altitude, or a different solar cycle.

A mean motion derived by fitting an ephemeris over a long arc absorbs the drag
as an average and is therefore insensitive to the density model's errors,
which is why such derived values are more reliable than a propagated one.

## Standards

- US Standard Atmosphere 1976, and its updates, as the reference profile
- The density coefficients used by the SGP4 theory: Vallado et al.,
  *Revisiting Spacetrack Report #3*, AIAA 2006-6753

## See also

- [[perturbations]] for the ordering of drag against $J_2$ and third body
- [[ephemeris-interpolation]] for a track that stops needing a density model
- [[sgp4]] for a propagation that carries a fitted drag term