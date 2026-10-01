---
title: Scenario and event simulation
tags: simulation, scenario, discrete-event, montecarlo, validation
summary: What a simulation of a spacecraft system can and cannot tell you, and why the number it produces is not the answer.
---

# Scenario and event simulation

A simulation of a spacecraft system is a model of a model. What it is good
for is comparison between designs and exploration of a parameter space; what
it is bad for is certifying a single number.

## Definition

A **scenario** is a specified set of initial conditions and events: the
orbit, the failure mode, the environment, and the period simulated. Two runs
with the same scenario and different implementation are a comparison.

A **discrete event** is an instant at which something changes state: a
command is executed, a sensor reports, a component fails. An event-driven
simulation advances from one such instant to the next rather than in fixed
time steps, and its accuracy depends on how the events are scheduled rather
than on a step size.

**Monte Carlo** repeats a scenario with sampled parameters to obtain a
distribution rather than a value.

## Mathematics

### What the output means

A simulation result is a value of a quantity that was computed under
assumptions. The uncertainty in it has three parts, and they are usually
very different in size:

- The uncertainty of the model, which is the largest and is the reason to
  simulate rather than to measure.
- The sampling uncertainty of the run, which shrinks as the square root of
  the number of runs and can be estimated.
- The numerical error, which shrinks with refinement.

Reporting the second as if it were the first is the characteristic failure
of simulation work, and it is why a Monte Carlo figure without a stated model
assumption is not a result.

### Time in an event-driven model

The state is constant between events, so a propagation or an interval can be
computed analytically rather than stepped. This makes an event-driven
simulation of an orbital system both faster and more accurate than a fixed
step at the same work, and it is why the choice of when an event fires is a
modelling decision rather than an implementation detail.

## Unit conventions

The units of the model. The failure that matters is a scenario specified in
different units from the implementation, which produces a plausible number
that is wrong by a power of a thousand.

## Applicability

Simulation answers comparative questions well: which of two station keep
strategies costs less propellant over five years, given the same orbit
determination quality. It answers absolute questions badly, because the
result carries the model's error and no amount of runs removes it.

The strongest use is checking that a design does not have a failure the
analysts missed, by simulating the failure rather than by analysing it. The
weakest use is producing a number for a report, which is where the
distinction between the three uncertainties above usually gets lost.

Validation is the part that is skipped and that matters: a simulation is
credible to the extent that it reproduces something already measured. Where
that has not been done, the result is a hypothesis with decimals.

## Standards

- None required. Simulation practice is not standardised, and the reporting
  of uncertainty is where the discipline is usually lost.

## See also

- [[numerical-methods]] for the numerical error that a simulation carries
- [[operations-planning]] for a planning question simulation answers well
- [[maneuvers]] for the model most simulations are built from