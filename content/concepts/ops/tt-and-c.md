---
title: Telemetry, tracking and command
tags: tt-c, telemetry, telecommand, ranging, pass, pass-prediction
summary: The operational chain that turns a spacecraft into something that can be flown, and where each part of it fails.
---

# Telemetry, tracking and command

Telemetry, tracking and command is the chain by which a spacecraft is
observed and controlled. It is a measurement chain, an estimation problem,
and a scheduling problem, and the failure modes of each are different.

## Definition

- **Telemetry** is the stream of spacecraft state sent down: attitude, orbit
  parameters, power, temperatures, and a health status.
- **Tracking** is the measurement of where the spacecraft is and how it is
  moving, from range and range rate.
- **Command** is the instruction stream sent up, including the telecommands
  themselves and the upload of a schedule.

The chain is: a station knows where the spacecraft ought to be, measures
where it is, compares, and uploads the difference. Every stage has a latency,
and the control design is largely about what to do about the total.

## Mathematics

### Orbit determination from tracking

A range measurement constrains the position along one line of sight; a range
rate constrains it along the derivative. A single station with both obtains
a two-dimensional state for a three-dimensional orbit, which is why
determination needs geometry and time rather than one station.

The estimate is a weighted least squares over the observations in an arc, and
the covariance of the result says how well the arc determined the orbit. This
is the number to look at: an orbit with a large covariance will drift
predictably and lose the arc that would have fixed it.

### The measurement error budget

Each measurement has a variance from noise, from the atmosphere, from the
clock, and from the station geometry. The geometry term is usually largest
and is a property of the orbit: a low-inclination orbit seen from a station
near its plane is well determined along the orbit and poorly determined
across it.

### Pass prediction and scheduling

A pass is a visibility interval for a station. Prediction is a forward
propagation of the orbit, and it inherits every error of that propagation,
which is why a predicted pass that is 50 km long is worth less than its
nominal length suggests.

Scheduling is the assignment of passes to stations to meet the requirements
per spacecraft, which is a scheduling problem with hard deadlines.

### Command latency and safety

The round trip from a command decision to it taking effect is dominated by
propagation delay and by a ground processing time of tens of seconds to
minutes. A safety rule that must act inside that latency must therefore be
implemented on the spacecraft, where it is bounded by the reaction of the
processor rather than by the link. This is the origin of the spacecraft's
autonomous safe mode and of the monitor-based command verification that
precedes execution.

## Unit conventions

- Range in kilometres, range rate in metres per second or kilometres per
  second.
- Pointing angles in degrees when passed to the spacecraft, because that is
  the convention of the telecommand dictionaries.
- Time is a spacecraft clock, usually TAI or GPS time; the offset from a
  ground reference is part of the tracking problem, not a detail of it.

## Applicability

The chain degrades in different ways at different bands. A Ku band station
can close a link in weather that Ku cannot; an S band ranging system trades
accuracy for robustness and works in rain. A mission that needs both carries
both, because the requirement that fails in the worst weather is usually
tracking, not commanding.

The clock is a limiting element throughout, and a mission that ignores it
discovers this when its tracking residuals stop improving.

Commanding is also where the interaction with interference appears: an uplink
that is jammed is not merely degraded, and the ground segment needs a way to
know whether a command arrived.

## Standards

- Telemetry and telecommand coding: CCSDS Space Packet Protocol, and the
  telemetry and telecommand dictionaries in the CCSDS Blue Book
- Time and frequency: CCSDS Time and Frequency Formats, which define the
  onboard time format
- Station keeping and pass prediction: not standardised; mission practice

## See also

- [[doppler-and-rate]] for the measurement the chain depends on
- [[link-budget]] for whether a pass exists
- [[operations-planning]] for what the chain is planned against
- [[interference]] for the uplink