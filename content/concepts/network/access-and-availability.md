---
title: Visibility, access and obstruction
tags: visibility, access, elevation, horizon, obstruction, availability
summary: The difference between a satellite being overhead and a user being able to use it, and what the gap between them is called.
---

# Visibility, access and obstruction

A satellite in view is not a satellite in use. The distinction is the
difference between coverage and availability, and almost every disappointing
service is explained by it.

## Definition

**Visibility** is the geometric condition: the satellite is above the local
horizon. **Access** is the condition that includes the link: the elevation is
high enough for the link to close, the pointing can be resolved, and the
propagation is acceptable. **Availability** is the fraction of the time an
access is available to a particular user.

**Obstruction** is anything in the way: terrain, a building, another vehicle,
and in the worst case the payload of a satellite in a higher orbit passing
across the line of sight.

## Mathematics

### The geometric pass

A satellite in a circular orbit at semi-major axis $a$ passes a latitude
$\phi$ with a maximum elevation $e_{\text{max}}$ that follows from the geometry,

$e_{\text{max}}$ follows from the geometry: it is the angle whose tangent is the
perpendicular distance from the observer to the orbit plane divided by the
distance along the line of sight.

with two limits worth naming. At the pole a satellite in an inclined orbit
reaches only `90 - inclination` degrees of elevation at best, so a
high-inclination constellation has poor coverage at high latitude from any
single satellite, and relies on many satellites rather than on high ones.

At the equator a satellite in an equatorial orbit passes directly overhead and
the elevation approaches 90 degrees, which is the best case in the system.

### The elevation mask

The link budget requires a minimum elevation because below it the path
length grows, the pointing becomes coarse, and the Doppler rate becomes
large enough that tracking is harder. The mask is a design parameter
rather than a physical one, and choosing a lower mask increases coverage and
degrades the link on every pass, which is a trade rather than a free
choice.

### Obstruction and its statistics

Terrestrial obstruction is binary for a ground user, so the relevant figure
is a **probability of availability** rather than an average. For a satellite
obstruction, the relevant figure is the probability that an intersection
occurs during an access, which depends on the relative motion and on the
angular separation of the two orbits.

## Unit conventions

- Elevation and azimuth in degrees when describing a pass, radians in any
  computation.
- Access time in seconds; availability as a fraction or as a percentage.
- Angles of separation in degrees.

## Applicability

The gap between coverage and availability is large and is usually the whole
story of a service complaint. A constellation may have 100 percent
geometric coverage over a country and a service that is available for 96
percent of the time, and the remaining four percent is what a user
remembers.

The obstruction term is what makes the ground segment matter. The same
constellation with a different set of ground station sites has a different
availability, and no amount of satellites moves it, because the obstruction
is local.

A low elevation mask raises the headline coverage number and lowers the
per-pass link margin at the same time. Those two effects are usually not
reported together, which is why a coverage figure without a mask is not
interpretable.

## Standards

- Elevation mask conventions and availability reporting: not standardised;
  the relevant practice is the regulatory service definition in the
  licensing.

## See also

- [[constellations]] for the coverage figure this is measured against
- [[link-budget]] for why the mask is not zero
- [[handover-and-routing]] for what happens between accesses