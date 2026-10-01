---
title: Concept overview
tags: overview, scope, how-to-read
summary: What this documentation covers, how it is organised, and how to read a concept page that describes more than one implementation.
---

# Concept overview

This documentation describes spacecraft, orbits, and the measurement and
communication around them. It is a teaching resource: it explains what the
quantities are, what units they carry, how they are computed, and where each
method stops being valid. It is not a manual for one library, and a page here
usually describes a family of approaches rather than a single one.

## What it covers

The subject is a satellite from launch to link closure:

- **Foundations.** Units, dimensions, and the numerical behaviour of a
  computed number.
- **Time and geometry.** Time scales, leap seconds, calendars, reference
  frames, the Earth rotation angle, and the reference ellipsoid.
- **Orbit.** Orbital elements and their formats, the perturbation sources
  that move an orbit, the models that propagate it, and the manoeuvres that
  change it.
- **Radio and propagation.** How motion changes the received frequency, what
  makes a link close, what the atmosphere does, and what interference does.
- **Constellation and network.** How satellites are arranged, when one can
  be used, and how a session survives the satellite handing it over.
- **Operations.** The measurement and control chain, and the planning that
  allocates fuel, contact, and safety.
- **Simulation.** What a simulation can and cannot establish.

## How it is organised

The sections follow the layers a spacecraft system is naturally built in,
from the arithmetic at the bottom to the network at the top. The ordering is
a reading order, not a dependency: a page on link budgets assumes the orbit
and the atmosphere, not the other way round.

Each page follows the same shape, so a reader knows where to look:

1. **Definition** of the quantity or the model.
2. **Mathematics**, with the symbols defined.
3. **Unit conventions**, including the ones that differ between fields.
4. **Applicability**, where the method stops being valid.
5. **Standards**, naming the public source the definition follows.
6. **See also**, linking the concepts that depend on this one.

A cross reference names another page; for example the page on
[[time-scales]] is reached from here, and the page on [[maneuvers]] refers
back to it.

## How to read a page

Two things are worth knowing in advance.

A page names more than one implementation wherever more than one is used, and
says which is which. Where a specific library is relevant, it is named; where
it is not, the page is about the problem rather than about a product.

The **unit conventions** section is not housekeeping. A number without its
unit and its reference is not a physical quantity, and a large part of the
practical errors in this field are unit errors rather than concept errors.

## Where the implementation is

The pages above describe the field. A separate section describes what one
particular library implements of it, including how it is organised and where
its algorithms come from. That separation is deliberate: the concepts stand
on their own, and the implementation is one answer to them rather than their
definition.

The online tools exist for the same reason. A reader who has just read about
a propagation model can run it, which is a stronger check than reading.

## See also

- [[units-and-dimensions]] for the convention that runs through every page
- [[numerical-methods]] for what "computed" means on these pages