---
id: reference-frames
title: Reference frames
chapter: geometry
summary: The inertial and Earth-fixed frames, the typed positions that carry their frame, and the rotation between them.
tags: frames, eci, ecef, geodetic, rotation
---

# Reference frames

A position is a set of numbers only when the frame they are measured in is
known. Two frames are used in the kernels: one fixed in inertial space and
one fixed to the rotating Earth. This document defines them and the
transformation between them.

## Definition

| Frame | Origin | Axes | Rotates with the Earth |
|-------|--------|------|------------------------|
| ECI, Earth-centered inertial | Earth center | Mean equator and equinox of J2000.0 | No |
| ECEF, Earth-centered Earth-fixed | Earth center | Rotates with the Earth | Yes |
| Geodetic | On the ellipsoid | Latitude, longitude, height | Yes |

A point fixed to the ground has constant coordinates in the Earth-fixed
frame and circles the origin in the inertial frame. A vector fixed in
inertial space, such as the direction to a distant star, has constant
coordinates in the inertial frame.

The kernels attach the frame to the type. `Eci`, `Ecef`, and `Geodetic` are
distinct types, so a position expressed in one frame cannot be passed where
another is expected. A conversion is an explicit method call.

## Mathematical meaning

The transformation between the inertial and Earth-fixed frames is a
rotation about the third axis by the Earth Rotation Angle:

```text
r_ecef = R3(theta) r_eci
r_eci  = R3(theta)^T r_ecef
```

where `theta` is the Earth Rotation Angle and `R3` is the rotation matrix

```text
R3(theta) = [ cos(theta)   sin(theta)  0
             -sin(theta)   cos(theta)  0
              0            0           1 ]
```

The rotation is orthogonal, so it preserves lengths and angles. This is a
useful check: a conversion that changes the length of a vector by more than
rounding is wrong.

The transformation between Earth-fixed Cartesian coordinates and geodetic
coordinates is not a rotation, because the ellipsoid is not a sphere. It is
solved by the standard closed form described in [[geodesy-wgs84]].

## Unit convention

- Cartesian positions in `Eci` and `Ecef` are in **meters**.
- Velocities in the inertial frames of the propagators are in **kilometers
  per second** and positions in **kilometers**; the orbit layer works in the
  units the element sets and the SGP4 theory use.
- Geodetic latitude and longitude are in **radians**, and height is in
  **meters** above the ellipsoid.

The difference between the meter convention of the coordinate layer and the
kilometer convention of the orbit layer is deliberate rather than an
oversight: each follows the convention of the theory it implements. A
conversion between the layers is an explicit multiplication, not an implicit
one.

## Applicability

An inertial frame is an idealization. The frame used here is aligned with
the mean equator and equinox of J2000.0 and treated as inertial, which
neglects precession and nutation. Over the intervals a two-line element set
is valid, which is days, the neglected motion is far below the accuracy of
the propagation. Over years it is not, and a caller that needs it must use a
frame implementation that models precession and nutation.

The Earth Rotation Angle defines the rotation between the frames. It does
not include polar motion, which is the small motion of the rotation axis
within the Earth. Polar motion is below 0.5 arcseconds, which is about 15
meters on the ground, and is neglected here.

## Comparison of frame conventions

| Convention | Axes defined by | Includes precession | Includes nutation | Used here |
|------------|-----------------|---------------------|-------------------|-----------|
| J2000.0 inertial | Mean equator and equinox of 2000-01-01 12:00 TT | No | No | Yes |
| True of date | True equator and equinox of the date | Yes | Yes | No |
| Mean of date | Mean equator and equinox of the date | Yes | No | No |
| TEME, True Equator Mean Equinox | True equator, mean equinox | Implicitly | Yes for the pole | Yes, as the output of SGP4 |
| Earth-fixed | Rotates with the Earth | Not applicable | Not applicable | Yes |
| ITRF | A realization of the Earth-fixed frame | Not applicable | Not applicable | As WGS 84, approximated |

TEME is the frame a two-line element set is written for. It is the output of
SGP4 and is not the same as the J2000.0 inertial frame. Converting from TEME
to the Earth-fixed frame requires the Earth rotation and the equation of the
equinoxes, which is the small angle between the mean and true equinoxes.

## Source

- Earth Rotation Angle: *IERS Conventions (2010)*, equation 8.32, as
  implemented by `apsis-frames`.
- The frame definitions follow the usage in Vallado, *Fundamentals of
  Astrodynamics and Applications*, and in the *Explanatory Supplement to the
  Astronomical Almanac*.

## See also

- [[earth-rotation-angle]] for the rotation angle
- [[geodesy-wgs84]] for the ellipsoid and the Cartesian conversion
- [[sgp4]] for the frame the propagator returns
- Try the rotation in the online tool: *Rotate between inertial and
  Earth-fixed frames*
