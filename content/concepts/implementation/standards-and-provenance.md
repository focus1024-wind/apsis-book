---
title: Standards and provenance
summary: Which public standard or paper each implemented algorithm follows, and how that attribution is recorded.
tags: standards, references, provenance
---

# Standards and provenance

An implementation is only as trustworthy as the source it follows. Every
algorithm in the workspace names its source in the module documentation, and
this document collects those citations in one place.

## Why the source is part of the interface

Two implementations of the same named model can disagree by kilometers. The
reasons are not errors but choices:

- **The gravity model.** SGP4 was fitted against WGS 72. An element set
  distributed by Space-Track is intended to be used with WGS 72. Using WGS
  84 instead shifts the position by kilometers, and neither result is
  wrong; they answer to different fits.
- **The definition of an angle.** Latitude may be geodetic or geocentric.
  The two differ by up to about 0.19 degrees at mid latitudes, which is
  about 21 kilometers on the ground.
- **The time scale.** A position at an instant is only meaningful with the
  scale that instant is on.

Naming the source makes these choices checkable.

## Sources by topic

| Topic | Source | Used by |
|-------|--------|---------|
| SGP4 and SDP4 | Vallado, Crawford, Hujsak, Kelso, *Revisiting Spacetrack Report #3*, AIAA 2006-6753, 2006 | `apsis-sgp4` |
| Two-line element format | CelesTrak, *FAQs: Two-Line Element Set Format*, documenting the original NORAD columns and the checksum rule | `apsis-tle` |
| Gravity model constants | WGS 72 values as used by the SGP4 reference implementation; WGS 84 from NIMA TR8350.2 | `apsis-sgp4` |
| Earth Rotation Angle | *IERS Conventions (2010)*, equation 8.32 | `apsis-frames` |
| Geodetic conversions | NIMA TR8350.2, *Department of Defense World Geodetic System 1984*, third edition, 2000 | `apsis-geodesy` |
| Julian and Modified Julian Date | IAU conventions as described in the *Explanatory Supplement to the Astronomical Almanac* | `apsis-time` |
| Terrestrial Time offset | IAU 2000 Resolution B1.9, defining `TT - TAI = 32.184 s` | `apsis-time` |
| Leap seconds | IERS Bulletin C, embedded as a table by `apsis-leap` | `apsis-leap` |
| Calendar algorithms | Howard Hinnant, *chrono-Compatible Low-Level Date Algorithms*, 2015 | `apsis-time` |
| Discrete event simulation | Zeigler, Praehofer, Kim, *Theory of Modeling and Simulation*, second edition, 2000 | `apsis-sim-engine` |

## The difference between a standard and a paper

A *standard* is maintained and has an authority. WGS 84, the IERS
conventions, and IERS Bulletin C are standards. A *paper* describes an
algorithm and its validation. The SGP4 paper is a paper: the algorithm is
fixed by the paper, and the reference implementation distributed with it is
the operative description.

When the two disagree, the implementations here follow the paper and record
the disagreement, because the element sets were fitted to the paper's model.

## Reproducing a published value

A published position for a catalogued satellite can be reproduced when four
things agree:

1. The element set, including its epoch and its checksum.
2. The gravity model. Use WGS 72 unless the source says otherwise.
3. The frame. SGP4 returns the True Equator Mean Equinox frame.
4. The time. The propagation time is measured from the element epoch.

Failing on the fourth is common: the epoch in the element set is a UTC
instant, and a propagation by "one day" means one day of uniform elapsed
time, not one UTC calendar day. When the interval crosses a leap second
insertion the two differ by a second. See [[time-scales]] and
[[leap-seconds]].

## See also

- [[sgp4]] for the propagation model
- [[geodesy-wgs84]] for the ellipsoid
- [[reference-frames]] for the frame
