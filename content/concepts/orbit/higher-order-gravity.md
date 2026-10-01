---
title: Higher order gravity
tags: gravity, geodesy, harmonics, potential, tesseral
summary: What the oblateness term leaves out, how the Earth's gravity field is expanded, and why the expansion does not converge where you would like it to.
---

# Higher order gravity

Treating the Earth as a point mass is the first approximation of orbital
mechanics and the source of its largest single error. The second is to model
the Earth as an axisymmetric ellipsoid, which captures the oblateness and
nothing else. The third is to expand the full potential, which is what a
high-accuracy model does and where its difficulties begin.

## Definition

The **geopotential** of a body is the potential energy per unit mass due to its
gravitational field. Near the Earth it is conventionally written

```text
U(r, theta, lambda) = (mu / r) [ 1
  + sum_n (R/r)^n * sum_m P_nm(sin theta)
      * [ C_nm cos(m lambda) + S_nm sin(m lambda) ] ]
```

- `r` is the distance from the centre, `theta` the geocentric colatitude and
  `lambda` the longitude.
- `P_nm` are the Legendre functions, real polynomials on `sin(theta)`.
- `C_nm` and `S_nm` are the **geopotential coefficients**; the normalised ones
  are what a model publishes.
- `R` is a reference radius.

For `n = 0` there is one coefficient, the gravitational parameter. For each
higher degree there are `2n + 1` zonal, tesseral and sectorial terms.

`J_2` is the second zonal coefficient, about `1.0826e-3`, and it dominates
everything above it. The next largest are `J_3`, `J_4`, each smaller than
the last.

## Mathematics

### What each term does

- **Zonal** coefficients (`m = 0`) are axisymmetric and cause secular drift
  in the node and the argument of perigee, and no change in inclination.
  This is the classical J2 problem.
- **Tesseral** coefficients (`0 < m < n`) are longitude dependent and cause
  short-period variation in every element including the inclination. This is
  why real orbits wobble rather than precessing smoothly.
- **Sectorial** coefficients (`m = n`) couple the radius and the longitude
  directly and are the smallest contribution.

### The truncation problem

The series is a power series in `(R/r)`, which converges only for `r > R`,
that is, outside the Earth. For a satellite at `r` above the surface the
ratio is `R/r < 1` and the series converges, but **slowly**, because `R/r` is
close to one at low altitude. The number of terms needed for a given accuracy
grows as the altitude falls, which is why a low Earth orbit model truncates
hard and a geostationary one can carry many terms.

This is the opposite of intuition: the model is *worse* where the field is
strongest, not where it is weakest.

### Normalisation

The coefficients published by a model are normalised so that the unnormalised
form does not grow with degree:

```text
[ C_nm  S_nm ] = [ 1 / sqrt( (1-delta) (n-m)! (n+m)! ) ] * [ Cbar_nm  Sbar_nm ]
```

A model is defined by its degree `N` and by the coefficients below it. Degree
180 with the first few hundred coefficients is typical.

## Unit conventions

- Potential and coefficients dimensionless; `mu` is the gravitational
  parameter.
- `R` is the equatorial radius, about 6378.137 km.
- Angle in radians; the Legendre functions are polynomials on the sine of the
  colatitude.

## Applicability

A truncated gravity field is a compromise. Cutting at a low degree saves
computation and costs accuracy, and the cost is largest at low altitude where
`(R/r)` approaches one. The numerical consequences are worse than the
positional ones: a truncated field is smooth, but its derivatives, which
drive the short-period terms, are not.

A model fitted to a force function rather than to a potential is a
alternative used when the field is known from measurements along a specific
orbit; it does not generalise to other altitudes or inclinations.

The distinction between a normalised and unnormalised coefficient is a
frequent source of silent error, because the two differ by a factor that
depends on the degree.

## Standards

- EGM2008 and its successors: the standard Earth gravity models, published by
  the National Geospatial-Intelligence Agency
- The degree and normalisation convention: *IERS Conventions*, section 5

## See also

- [[perturbations]] for where the J2 term sits among the other perturbations
- [[geodesy-wgs84]] for the ellipsoid that the second-order model assumes
- [[numerical-methods]] for the series as a method with a truncation error