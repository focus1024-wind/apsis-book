---
title: The WGS 84 ellipsoid and geodetic coordinates
summary: Latitude, longitude, and height above an ellipsoid, and the closed form conversion to Cartesian coordinates.
tags: geodesy, wgs84, latitude, ellipsoid
---

# The WGS 84 ellipsoid and geodetic coordinates

A position on or near the Earth is usually written as a latitude, a
longitude, and a height. To convert that to Cartesian coordinates, the shape
of the Earth must be chosen first. The kernels use the WGS 84 ellipsoid.

## Definition

The WGS 84 ellipsoid is an oblate ellipsoid of revolution defined by two
numbers:

| Parameter | Symbol | Value | Unit |
|-----------|--------|-------|------|
| Semi-major axis | $a$ | 6 378 137.0 | m |
| Inverse flattening | $1/f$ | 298.257 223 563 | dimensionless |
| Flattening | $f$ | 1 / 298.257 223 563 | dimensionless |
| Semi-minor axis | $b = a(1 - f)$ | 6 356 752.314 245 | m |
| First eccentricity squared | $e^2 = 2f - f^2$ | 0.006 694 379 990 13 | dimensionless |

Every other quantity is derived from the two defining numbers, so the
derived values agree with the published ones to the precision of the
definitions rather than being independently rounded.

Three coordinates describe a point:

- **Geodetic latitude** $\phi$, the angle between the ellipsoid normal at the
  point and the equatorial plane.
- **Longitude** $\lambda$, positive east of the prime meridian.
- **Height** $h$, measured along the ellipsoid normal, positive outward.

Geodetic latitude is not geocentric latitude. Geocentric latitude is the
angle between the radius vector to the point and the equatorial plane. The
two agree at the equator and at the poles and differ by up to about 0.19
degrees at mid latitudes, which is about 21 kilometers on the ground.

## Mathematical meaning

The conversion from geodetic to Cartesian coordinates is closed form. With

$$
e^2 = 2f - f^2, \qquad
N = \frac{a}{\sqrt{1 - e^2 \sin^2 \phi}}
$$

the Cartesian coordinates in the Earth-fixed frame are

$$
\begin{aligned}
x &= (N + h) \cos\phi \cos\lambda \\
y &= (N + h) \cos\phi \sin\lambda \\
z &= \left( N (1 - e^2) + h \right) \sin\phi
\end{aligned}
$$

$N$ is the distance from the point to the rotation axis measured in the
equatorial plane for a point at zero height. It grows from $a$ at the
equator to $a / \sqrt{1 - e^2}$ at the pole.

The reverse conversion, from Cartesian coordinates to geodetic latitude and
height, has no elementary closed form and is solved by iteration or by a
series. The implementation uses a method that converges in a fixed number of
steps to the precision of the double precision representation. The
conversion returns no value when the point lies exactly on the rotation
axis, because no longitude is defined there.

The meridian radius of curvature, in the plane of a meridian, is

$$
M = \frac{a (1 - e^2)}{\left(1 - e^2 \sin^2 \phi\right)^{3/2}}
$$

It is the radius of curvature of the ellipsoid in the north-south direction
at that latitude. It appears in geodetic computations that integrate along a
meridian.

## Unit convention

- Latitude and longitude are in **radians** in the interface, and in
  **degrees** in the tools, which convert on the boundary.
- Height is in **meters** above the ellipsoid. It may be negative, which
  places the point below the ellipsoid surface.
- The ellipsoid parameters are in **meters** and are dimensionless where the
  quantity is a ratio.

## Applicability

The WGS 84 ellipsoid is a global average. The actual surface of the Earth
differs from it by up to about 100 meters, which is the geoid undulation. A
height above the ellipsoid is therefore not a height above mean sea level.
The two differ by the geoid undulation at the location, and a caller that
needs an orthometric height must apply that correction from a geoid model,
which these kernels do not include.

The ellipsoid defines a smooth surface and the conversion is exact for that
surface. It does not model the terrain, the atmosphere, or the local
deflection of the vertical. The deflection of the vertical, which is the
angle between the ellipsoid normal and the true direction of gravity, is
below about 0.01 degrees, which is about 1 kilometer over a 10,000
kilometer range, and matters for precise pointing.

## Comparison of Earth models

| Model | Semi-major axis | Inverse flattening | Intended use | Used here |
|-------|-----------------|--------------------|--------------|-----------|
| WGS 84 | 6 378 137.0 m | 298.257 223 563 | GPS and modern geodesy | Yes |
| WGS 72 | 6 378 135.0 m | 298.26 | Older satellite work; the SGP4 fit | For the propagator |
| GRS 80 | 6 378 137.0 m | 298.257 222 101 | Geodetic reference | No |
| Sphere | Any radius | Not applicable | Simple estimates | No |

WGS 84 and GRS 80 differ only in the flattening, by about 0.0001 percent, so
positions computed with one are indistinguishable from the other at the
meter level. WGS 72 differs from WGS 84 in the semi-major axis by 2 meters,
which is large enough to matter, and SGP4 was fitted against it.

## Source

- National Imagery and Mapping Agency, *Department of Defense World Geodetic
  System 1984*, Technical Report TR8350.2, third edition, 2000.

## See also

- [[reference-frames]] for the frames the coordinates are expressed in
- [[geodesy-wgs84]] is used by the propagator only for reporting altitude;
  the propagator itself uses the gravity model in [[sgp4]]
- Try the conversion in the online tool: *Geodetic position to Earth-fixed
  coordinates*
