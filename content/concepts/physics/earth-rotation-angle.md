---
title: Earth Rotation Angle
summary: The angle between the inertial and Earth-fixed frames and the linear expression that gives it.
tags: frames, earth-rotation, iers, ut1
---

# Earth Rotation Angle

The Earth Rotation Angle is the angle through which the Earth has turned. It
is the quantity that relates the inertial and Earth-fixed frames, and it is
simple enough to be computed from one expression. This document gives the
definition, the expression, and its limits.

## Definition

The Earth Rotation Angle, written $\theta$ or ERA, is the angle measured
along the true equator from the Celestial Intermediate Origin to the
Terrestrial Intermediate Origin. In practice it is the angle between the
inertial and Earth-fixed frames, and that is how it is used here.

It is a function of UT1, the time scale defined by the rotation of the Earth.

## Mathematical meaning

The angle is computed from the interval in days of UT1 since the J2000.0
epoch by a linear expression:

$$
\theta = 0.779\,057\,273\,2640 + 1.002\,737\,909\,350\,795 \, d
$$

where `d` is the number of days of UT1 since 2000-01-01 12:00 UT1, and the
result is in turns. The constant term is the angle at the epoch itself,
which is about 280.46 degrees. The rate is slightly more than one turn per
day because the Earth rotates once in a sidereal day, which is shorter than
a solar day.

The angle is not reduced to one turn. The value returned is a continuous
function of time, so the difference between two angles is meaningful even
when the interval is longer than a day. A caller that needs an angle in
$[0, 2\pi)$ reduces it explicitly.

The rotation between the frames is a clockwise turn about the third axis:

$$
R_3(\theta): \text{inertial} \to \text{Earth-fixed}, \qquad
R_3(\theta)^{\mathsf{T}}: \text{Earth-fixed} \to \text{inertial}
$$

## Unit convention

- The angle is stored in **turns** and offered in **turns** and in
  **radians**.
- The time argument is a number of **days of UT1 since J2000.0**, or a
  **Julian Date on the UT1 scale**, from which the interval is computed.

## Applicability

The linear expression is accurate to the precision of the formulation. Its
residual against the full IAU 2000 model is below a milliarcsecond over
decades, which is below a millimeter at the Earth's surface, so for the
positioning accuracy these kernels target it is exact.

The limitation is not the formula but the argument. UT1 differs from UTC by
the rotational irregularity that the leap second system approximates, and
the difference is called `UT1 - UTC` or DUT1. It is published with a
resolution of a day and can reach 0.9 seconds, which is about 400 meters at
the equator. A caller that has a UTC instant and no DUT1 value must decide
what to assume; using UTC in place of UT1 makes an error bounded by the same
0.9 seconds.

Polar motion, the motion of the rotation axis within the Earth, is not part
of the Earth Rotation Angle. It is below 0.5 arcseconds, about 15 meters on
the ground, and is neglected here.

## Comparison of approaches

| Approach | Accuracy | Cost | Input needed | Used here |
|----------|----------|------|--------------|-----------|
| IAU 2000 linear expression | Below a milliarcsecond over decades | One multiply and one add | UT1 | Yes |
| Full IAU 2000/2006 CIO-based transformation | Microarcsecond | Precession and nutation series | UT1 and TT | No |
| Greenwich Mean Sidereal Time | Arcsecond over decades | Apparent sidereal time series | UT1 and TT | No |
| Fixed rotation rate from a reference epoch | Grows without bound | One multiply | Epoch only | No |

Greenwich Mean Sidereal Time and the Earth Rotation Angle differ by a
rotation about the pole by the accumulated precession in right ascension.
The Earth Rotation Angle is the quantity the IERS conventions define and the
one this implementation uses.

## Source

- *IERS Conventions (2010)*, equation 8.32, as implemented by
  `apsis-frames`.

## See also

- [[reference-frames]] for the frames the rotation relates
- [[time-scales]] for UT1 against UTC
- [[calendar-and-mjd]] for the days since J2000.0
- Try the rotation in the online tool: *Rotate between inertial and
  Earth-fixed frames*
