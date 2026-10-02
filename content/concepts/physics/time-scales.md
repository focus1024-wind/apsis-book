---
title: Time scales
summary: UTC, TAI, TT, and GPS, and why an instant is stored on one scale.
tags: time, utc, tai, tt, gps
---

# Time scales

A time scale is a rule for assigning a number to an instant. Several are in
use at once, they do not agree, and the differences between them are large
enough to matter for pointing and ranging. This document defines the scales
the kernels support and the storage convention that removes the ambiguity
from the arithmetic.

## Definition

| Scale | Full name | Rate | Reference |
|-------|-----------|------|-----------|
| UTC | Coordinated Universal Time | Not uniform | Civil time, steered to UT1 by leap seconds |
| TAI | International Atomic Time | Uniform | Weighted mean of atomic clocks |
| TT | Terrestrial Time | Uniform | Geocentric coordinate time |
| GPS | GPS time | Uniform | Maintained by the GPS constellation |

UTC is the only scale here that is not uniform. It is kept within 0.9
seconds of UT1, the scale defined by the rotation of the Earth, by inserting
or removing whole seconds. A UTC minute that contains an inserted second has
61 seconds.

## Mathematical meaning

The storage model is one number: a **Modified Julian Date on the TAI
scale**. Every instant is converted to TAI when it is created and converted
back to whatever scale is asked for when it is displayed.

The reason is exactness. Two instants stored on TAI differ by

$$
\Delta t = \left( \text{tai}_{\text{later}} - \text{tai}_{\text{earlier}} \right) \times 86400
$$

and that difference is the elapsed SI time between them, whatever scales
they were created in. If an instant were stored on UTC, the same subtraction
would be wrong across a leap second insertion, because the UTC clock does
not have a single well defined numeric value at that point.

The conversions between scales are offsets except for UTC:

$$
\begin{aligned}
\text{TT} - \text{TAI} &= 32.184\,\text{s} && \text{(constant, IAU 2000)} \\
\text{TAI} - \text{GPS} &= 19\,\text{s} && \text{(constant since 1980)} \\
\text{TAI} - \text{UTC} &= 10\,\text{s} + N && \text{(N leap seconds since 1972)}
\end{aligned}
$$

The first two are constants. The third is a step function of the date, and
its value comes from the leap second table in `apsis-leap`, which is the
table published in IERS Bulletin C.

## Unit convention

- Instants are Modified Julian Dates, in days, counted from midnight on
  1858-11-17.
- A duration is in seconds, and is SI seconds of uniform elapsed time.
- A calendar date carries a year, a month, a day, an hour, a minute, and a
  second with a fraction. The second may be up to 60.999 in a minute that
  contains an inserted leap second.

## Applicability

The leap second table starts at the beginning of 1972, which is where the
modern UTC system begins. Before that date a UTC instant has no exact
representation here, and the conversion returns no value rather than a
guess. The table ends at the most recent entry it contains; a date after
that is converted using the last known offset, which is correct until the
next insertion and is therefore a forecast rather than a fact.

A date far in the future is a forecast for the same reason. The kernels make
this visible rather than hiding it: a conversion that is not covered returns
an absent value, and the caller decides what to do.

## Duration against calendar arithmetic

Adding seconds to an instant moves it by exactly that much elapsed time.
Adding a day in the calendar sense is a different operation: across a leap
second insertion, one UTC calendar day is 86,401 SI seconds. The kernels
expose only the first operation on the instant, and require a conversion to
a calendar for the second. This is deliberate, because the two are easy to
confuse and the error is small enough to go unnoticed until it matters.

## Comparison of representations

| Representation | Uniform rate | Leap seconds | Range | Used here for |
|----------------|--------------|--------------|-------|---------------|
| TAI MJD | Yes | Not represented | Wide | Storage and arithmetic |
| UTC MJD | No | Represented as a step | 1972 onward | Input and output |
| TAI seconds since an epoch | Yes | Not represented | Depends on epoch | Duration |
| Calendar fields | No | Represented | Any | Human input and display |
| GPS week and seconds | Yes | Not represented | 1980 onward | GNSS interfaces |

## Source

- Julian Date and Modified Julian Date conventions: IAU, as described in the
  *Explanatory Supplement to the Astronomical Almanac*.
- Terrestrial Time offset from TAI: IAU 2000 Resolution B1.9.
- Leap seconds: IERS Bulletin C.
- Calendar algorithms: Howard Hinnant, *chrono-Compatible Low-Level Date
  Algorithms*, 2015.

## See also

- [[leap-seconds]] for the table and the insertion rule
- [[calendar-and-mjd]] for the date conversions
- Try the conversion in the online tool: *Convert a time scale*
