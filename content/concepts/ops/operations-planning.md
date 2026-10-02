---
title: Operations planning
tags: operations, planning, scheduling, fuel-budget, collision-avoidance
summary: The day-to-day problem of keeping a spacecraft where it should be, and the three budgets that decide it.
---

# Operations planning

Once a spacecraft is flying, someone has to decide what it should do. That
is a scheduling problem with three hard budgets, and the whole of operations
is the negotiation between them.

## Definition

- The **fuel budget** bounds the total manoeuvre and station-keeping propellant
  available over the mission life.
- The **contact budget** bounds the number of passes available from the
  ground segment, which bounds how much commanding and how much orbit
  determination are possible.
- The **safety budget** bounds the number of collision avoidance manoeuvres
  that can be executed, because each one consumes fuel and contact time.

Planning is the allocation of these. A spacecraft that is short on contact
cannot be flown to its accuracy targets; one that is short on fuel will end
early.

## Mathematics

### Station keeping cost

For a low Earth orbit in a plane that is not exactly sun-synchronous, the
plane drifts because the oblateness and the solar and lunar gravity produce
a nodal rate the design did not ask for. Holding the plane costs a
station-keeping manoeuvre at some interval,

$$
\Delta v_{\text{keeping}} \sim \left\lvert \dot\Omega \right\rvert \Delta t \, a
$$

so the cost is linear in the drift rate and in the interval. Halving the
allowed drift doubles the rate, and halving the allowed drift interval
doubles the number of manoeuvres: the two multiply.

### Collision avoidance

With many satellites in similar shells, conjunction assessment produces
candidate approaches, and each confirmed one needs a manoeuvre. The cost per
manoeuvre is small and the number is not controlled, which is why the
modern practice is a few centimetres of uncertainty per satellite rather
than tens of metres: an approach that cannot be confirmed does not need a
manoeuvre.

### Orbit determination as a budget

The fuel budget and the contact budget meet here. Accuracy requires arc
length, arc length requires contact, and contact is the scarce resource. A
spacecraft that cannot be observed often enough will drift, will need
manoeuvres it cannot afford, and the failure appears as a budget rather than
as an orbit error.

## Unit conventions

- Propellant in kilograms, or in delta-v which is what it is spent for.
- Contact time in minutes per day per spacecraft.
- Drift rates in degrees per day.
- Conjunction screening volumes in kilometres; the uncertainty that
  determines them in the same.

## Applicability

The budgets are not independent. Spending fuel to buy accuracy because the
contact budget allowed more tracking is a legitimate trade, and it is the
trade a plan makes explicitly or implicitly. A plan that does not state the
allocation has made one by accident.

The hardest case is the crowded shell, where the collision avoidance budget is
consumed by events nobody planned for, and the response is a rule rather than
a decision: the manoeuvre is executed automatically when the probability
exceeds a threshold, because the decision cannot be made in the time
available.

Operating a spacecraft in a crowded shell is therefore partly a matter of
the uncertainty with which it is catalogued, which is a design decision made
years before launch.

## Standards

- Conjunction assessment and avoidance practice: the standards of the
  international coordination bodies for the relevant regime, plus mission
  practice
- Ground segment scheduling interfaces: the CCSDS mission operations
  standards

## See also

- [[maneuvers]] for what a manoeuvre costs
- [[tt-and-c]] for the contact budget
- [[scenario-simulation]] for rehearsing a plan before flying it