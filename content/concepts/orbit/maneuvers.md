---
title: Orbit manoeuvres
tags: maneuvers, impulse, finite-burn, hohmann, delta-v
summary: How a change in orbit is made, what it costs in propellant, and why the cheapest transfer between two circular orbits is the one everybody uses.
---

# Orbit manoeuvres

A manoeuvre changes an orbit. The satellite is at one point on a closed curve and
must end up at another point on a different closed curve, and the change has to
be paid for in propellant. This is the subject: how large the change is, how
much it costs, and how the cost changes when the orbit is not circular.

## Definition

A **delta-v** is the magnitude of a change in velocity. A manoeuvre is planned
as a sequence of delta-v, and the propellant cost follows from their sum by the
rocket equation.

An **impulse** applies the whole change in zero time. It is the right model for
a burn that is short compared with the orbital period, which for a low Earth
orbit means anything under a minute.

A **finite burn** applies a thrust over a duration long enough that the
satellite moves while thrusting. The distinction is not academic: a low Earth
orbital transfer assisted by electric propulsion can last for hours and its
gravity loss is a substantial fraction of the ideal cost.

## Mathematics

### The rocket equation

The velocity a vehicle can produce depends on the exhaust velocity of its
propellant and on the mass ratio,

$$
\Delta v = v_e \ln\frac{m_0}{m_f}
$$

with $v_e$ the exhaust velocity. Since a logarithmic function grows slowly,
doubling the propellant gives far less than double the velocity. This is why
the delta-v of a mission is the primary constraint on its design, and why
staging exists.

### The Hohmann transfer

Between two coplanar circular orbits the cheapest two-impulse transfer is the
Hohmann transfer. The first burn raises or lowers the apoapsis to match the
target, the satellite coasts through half a revolution, and the second burn
circularises.

With $r_1$, $r_2$ the two radii and $a_t = (r_1 + r_2) / 2$ the transfer
semi-major axis, the two burns follow from the vis-viva equation,

$$
\begin{aligned}
\Delta v_1 &= \sqrt{\mu \left( \frac{2}{r_1} - \frac{1}{a_t} \right)} - \sqrt{\frac{\mu}{r_1}} \\
\Delta v_2 &= \sqrt{\frac{\mu}{r_2}} - \sqrt{\mu \left( \frac{2}{r_2} - \frac{1}{a_t} \right)}
\end{aligned}
$$

and the coast takes half the period of the transfer ellipse,

$$
t_{\text{coast}} = \pi \sqrt{\frac{a_t^3}{\mu}}
$$

A raise from 300 km to geostationary costs about 4.1 km/s, which is a
large fraction of what a launch vehicle can provide, and most of it is the
first burn.

### Plane changes

A transfer that also changes inclination is more expensive than a coplanar
one. The combined manoeuvre, raising the apoapsis and rotating the plane at
the same time, costs more than doing either alone. The optimum for a given
radius change and a given inclination change is a two-burn sequence in which
the plane change is made at the smaller radius, where the velocity is lowest
and therefore the same angular rotation is cheapest in delta-v.

$$
\Delta v_{\text{plane}} \sim 2 V \sin\frac{\Delta i}{2}
$$

with $V$ the circular speed at the radius where the rotation is performed.

### Finite burns

Over a finite burn the satellite is still moving against gravity and the
thrust does not act along the instantaneous velocity. The achieved velocity
change is less than the ideal $a t$, and the shortfall is the gravity loss,

$$
L_{\text{gravity}} = \left\lvert \Delta v_{\text{ideal}} \right\rvert - \left\lvert \Delta v_{\text{achieved}} \right\rvert
$$

which grows with burn duration. A chemical burn is short enough to ignore it.
An electric thruster is not.

## Unit conventions

- Delta-v and velocity in kilometres per second.
- Radii in kilometres.
- The gravitational parameter $\mu$ in cubic kilometres per second squared.
- Inclinations in radians; degrees hide a factor of about 57 when used in a
  sine.

## Applicability

The Hohmann transfer is optimal only for coplanar circular orbits, and only
among two-impulse transfers. Real missions are rarely that: the orbits are
elliptical, inclined, or both, and a bi-elliptic transfer can beat Hohmann
when the ratio of the two radii exceeds about 11.94, at the cost of a much
longer flight.

The impulsive model is adequate when the burn is a small fraction of the
orbital period. It is not adequate for a spiral orbit raise by electric
propulsion, where the "burn" is the whole mission.

A plane change is rarely worth it. Raising the inclination to reach a
different ground track costs delta-v proportional to the inclination and
often exceeds the entire launch budget, which is why planes are chosen at
launch and then maintained rather than changed.

## Standards

- Hohmann transfer: derived from the vis-viva equation; see Vallado,
  *Fundamentals of Astrodynamics and Applications*
- Rocket equation: Tsiolkovsky, 1903

## See also

- [[keplerian-elements]] for the orbit being changed
- [[sgp4]] for propagating the result of a manoeuvre
- [[perturbations]] for what the propellant estimate omits