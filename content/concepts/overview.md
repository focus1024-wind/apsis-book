---
id: overview
title: Concept overview
chapter: overview
summary: The scope of the ApsisRS kernels and how the concept set is organized.
tags: overview, scope
---

# Concept overview

ApsisRS is a workspace of computational kernels for commercial spaceflight
and satellite internet systems. The kernels are written so that they can be
audited: they depend on nothing outside the Rust standard library, every
quantity carries its unit in the type system, and every algorithm names the
public standard it follows.

This documentation explains the concepts the kernels implement. It is
written for someone who has the numbers in front of them and needs to know
what they mean, which units they are in, and where the method stops being
valid. Each document gives the definition, the mathematical meaning, the
unit convention, the boundary of applicability, and the public source.

## The layer order

The workspace is organized into layers with a one way dependency rule. A
layer may use the layers to its left and never the layers to its right.

```text
foundation -> physics -> orbit -> rf -> network -> ops -> sim
```

The layers that this service exposes are the first three:

| Layer | Contents | Examples |
|-------|----------|----------|
| foundation | Shared types, units, linear algebra | `Vec3`, `Mat3`, quantity newtypes |
| physics | Time, frames, geodesy | `Epoch`, `EarthRotationAngle`, `WGS84` |
| orbit | Elements, propagation, maneuvers | `TwoLineElement`, `Satellite`, `Orbit` |

The remaining layers cover radio frequency links, constellation networks,
operations scheduling, and simulation. They are outside the scope of the
interface catalog this service presents, which is generated from the
foundation, physics, and orbit crates.

## How to read a concept document

A concept document answers five questions:

1. **Definition.** What the concept is, in one paragraph.
2. **Mathematical meaning.** The equations, or the description of the
   operation, with the symbols defined.
3. **Units.** The unit of every quantity, and the convention for angles.
4. **Applicability.** Where the method is valid and where it is not.
5. **Source.** The public standard or paper the implementation follows.

Where more than one method solves the same problem, the document lists them
side by side in a comparison table with the columns *method*, *accuracy*,
*computational cost*, *where it applies*, *reference*, and *whether
`apsis-rs` implements it*.

## Angle convention

Angles in the public interface of `apsis-rs` are in **radians** unless the
name says otherwise. The element set format is the one exception: the
orientation angles of a two-line element set are in **degrees**, because
that is how the format writes them, and the parser keeps them in degrees.
The conversion to radians happens in the propagator, not in the parser.

Longitude is positive east. Latitude is geodetic, measured from the
ellipsoid normal, not geocentric.

## Time convention

An instant is stored as a Modified Julian Date on the International Atomic
Time scale. Storing one uniform scale makes the difference between two
instants exact elapsed time. See [[time-scales]].

## Where to go next

- The structure of the workspace: [[architecture-apsis-rs]]
- Time: [[time-scales]], [[leap-seconds]], [[calendar-and-mjd]]
- Frames and the Earth: [[reference-frames]], [[geodesy-wgs84]]
- Elements and propagation: [[tle-format]], [[sgp4]], [[keplerian-elements]]
- How the methods compare: [[propagator-comparison]]
