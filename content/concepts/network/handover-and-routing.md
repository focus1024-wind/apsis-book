---
title: Topology, handover and routing
tags: isl, handover, intersatellite, routing, disruption
summary: When there is more than one way across the network, the question of which way, and what it costs when the choice changes.
---

# Topology, handover and routing

A constellation network is not a set of point-to-point links to ground. It
is a graph whose nodes move, and every problem of graph routing has to be
re-asked every time the graph changes shape.

## Definition

An **inter-satellite link** is a radio link between two satellites, giving
laser or radio crosslink connectivity between them. **Handover** is the
transfer of an active session from one satellite to another. **Routing** is
the choice of path across the graph. **Disruption tolerance** is the
mechanism that keeps a session alive through a handover.

The distinction that matters operationally is between a link that is planned
and a link that exists opportunistically, because the first can be scheduled
and the second cannot be.

## Mathematics

### Visibility of a crosslink

Two satellites can see each other when the elevation of one above the other's
horizon is above a mask, which is the same geometry as the ground case with
the observer at orbit altitude. The consequence is that crosslinks are
intermittent: a pair of satellites in adjacent planes are in contact for a
fraction of their relative motion, not continuously, and the contact interval
is shorter for lower orbits.

### Why handover costs

The cost of a handover is not the signalling; it is the buffering. A session
in flight holds state, and that state must be transferred or reconstructed on
the far side. The cost rises with the data rate on the session and with how
much of the protocol's state is not reconstructible from the data itself.

This is why a system that can tolerate disruption pays for it in the
protocol rather than in the network, and why the disruption-tolerant
protocols are built around the assumption that a handover is normal.

### Graph shape over time

Because nodes move, the adjacency matrix is a function of time, and a route
optimal at one instant is not optimal at the next. Two approaches follow:

- **Scheduled routing**, which uses a precomputed time-varying topology and
  plans against it. Predictable, and limited to links that exist on schedule.
- **Opportunistic routing**, which uses whatever is present. Reaches further,
  and can fail at a moment when nothing can be done.

### Capacity per node

A satellite is a network node with a fixed set of radio terminals. Its
throughput is therefore shared between every session it carries, and the
capacity available to one session falls as the constellation gets busier at
the same location.

## Unit conventions

- Crosslink data rate in bits per second per link; aggregate in the same.
- Handover interruption time in milliseconds.
- Link elevation in degrees; the horizon geometry uses the same quantities as
  the ground case.

## Applicability

Routing decisions are constrained by the physics long before they are
constrained by the protocol. A route that needs a link which is only present
for 30 seconds per orbit is not a route; it is a hope.

The hardest case is not the dense shell, where a route exists, but the sparse
case, where sometimes no path crosses a given region for minutes. Designs
handle it by accepting the disruption and reconstructing state rather than by
searching for a route that does not exist.

Capacity per node explains a behaviour that looks anomalous from outside: a
service gets slower in a city during busy hours even though coverage is
unchanged, because the same satellites are serving more sessions.

## Standards

- Inter-satellite link characterisation and the laser physical layer: the
  CCSDS Inter-Satellite Laser Communications specifications
- Session continuity and disruption tolerance: the recommendations for
  handover and for the service data, in the CCSDS Space Link Protocol
  family

## See also

- [[access-and-availability]] for the ground access a handover moves between
- [[constellations]] for the geometry that decides adjacency
- [[interference]] for the physical layer under the topology