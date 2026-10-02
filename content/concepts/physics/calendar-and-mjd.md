---
title: Calendars and Modified Julian Date
summary: The day numbering systems, the proleptic Gregorian calendar, and the conversion algorithms.
tags: time, calendar, julian-date, mjd
---

# Calendars and Modified Julian Date

A day number is a count of days from a fixed origin. A calendar is a rule
for naming a day by a year, a month, and a day of the month. The kernels use
day numbers for arithmetic and calendars for input and display, and this
document gives both.

## Definition

- **Julian Date (JD)** counts days from noon on 4713-01-01 in the proleptic
  Julian calendar.
- **Modified Julian Date (MJD)** counts days from midnight on 1858-11-17.
- The two differ by a fixed offset:

$$
\text{JD} = \text{MJD} + 2400000.5
$$

The half day is the difference between the noon start of the Julian Date and
the midnight start of the Modified Julian Date. The Modified Julian Date is
used here because it keeps the day number in a range where the fraction of a
day is a manageable number.

## The calendars

The proleptic Gregorian calendar is the Gregorian calendar extended backward
before its introduction in 1582. The kernels use it throughout, including
for dates before 1582, so that every day number has one calendar
representation.

A year is a leap year when it is divisible by four, except that a year
divisible by 100 is not a leap year unless it is also divisible by 400:

$$
\text{leap}(y) = (y \bmod 4 = 0) \land (y \bmod 100 \ne 0 \lor y \bmod 400 = 0)
$$

So 2000 is a leap year and 1900 is not.

## Mathematical meaning

The conversion from a calendar date to a day number and back uses integer
arithmetic so that it is exact for every representable date. The algorithm
shifts the year so that it begins in March, which places the leap day at the
end of the shifted year and removes the special case from the day count.

$$
\begin{aligned}
a &= \left\lfloor \frac{14 - \text{month}}{12} \right\rfloor \\
y &= \text{year} + 4800 - a \\
m &= \text{month} + 12a - 3 \\
\text{JDN} &= \text{day} + \left\lfloor \frac{153m + 2}{5} \right\rfloor + 365y
  + \left\lfloor \frac{y}{4} \right\rfloor - \left\lfloor \frac{y}{100} \right\rfloor
  + \left\lfloor \frac{y}{400} \right\rfloor - 32045
\end{aligned}
$$

The inverse uses the same structure with a shift of 32044. Both directions
are implemented once and tested against each other over a wide range of
dates.

## Unit convention

- A day number is in days, with the fraction of a day carrying the time of
  day.
- The fraction is zero at midnight and 0.5 at noon.
- A calendar value carries a year, a month, a day, an hour, a minute, and a
  second with a fraction. The second may be up to 60.999 in a minute that
  contains an inserted leap second.

## Applicability

The Gregorian calendar is a civil convention and the day number is a
counting convention. Neither has a physical meaning beyond that. A day
number is a convenient way to do arithmetic; it is not an observable.

The leap year rule is exact for the proleptic Gregorian calendar but does
not describe any calendar actually in use before 1582. For a date before
1582, the conversion is the proleptic one, and a user comparing against a
historical document may find a difference of days.

## Comparison of day numbering systems

| System | Origin | Unit | Fraction at midnight | Used here |
|--------|--------|------|----------------------|-----------|
| Julian Date | 4713-01-01 12:00 (proleptic Julian) | day | 0.5 | Converted to MJD only |
| Modified Julian Date | 1858-11-17 00:00 | day | 0.0 | Yes |
| Reduced Julian Date | 1858-11-16 12:00 | day | 0.5 | No |
| Truncated Julian Date | 1968-05-24 00:00 | day | 0.0 | No |
| Days since J2000.0 | 2000-01-01 12:00 TT | day | 0.5 | For the Earth Rotation Angle |
| Unix time | 1970-01-01 00:00 UTC | second | 0 at 00:00 | No |

## Source

- The day number conventions follow IAU usage as described in the
  *Explanatory Supplement to the Astronomical Almanac*.
- The conversion algorithm is the one described by Howard Hinnant,
  *chrono-Compatible Low-Level Date Algorithms*, 2015.

## See also

- [[time-scales]] for the scales a day number may be interpreted on
- [[leap-seconds]] for the inserted seconds a calendar may contain
- The elapsed days since J2000.0 appear in [[earth-rotation-angle]]
