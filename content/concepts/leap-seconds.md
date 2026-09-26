---
id: leap-seconds
title: Leap seconds
chapter: time
summary: Why UTC has step discontinuities and how the embedded table handles them.
tags: time, utc, leap-seconds, iers
---

# Leap seconds

UTC is kept close to the rotation of the Earth, which is not uniform, by
inserting or removing whole seconds. Each insertion is a step
discontinuity in the UTC clock. This document explains the mechanism and the
convention the kernels use to avoid the discontinuity in arithmetic.

## Definition

A leap second is a one second adjustment to UTC announced by the
International Earth Rotation and Reference Systems Service in a Bulletin C
notice. The adjustment keeps

```text
| UT1 - UTC | < 0.9 s
```

UT1 is the scale defined by the rotation of the Earth. The Earth's rotation
rate is not constant, so the number of seconds that have elapsed since an
epoch and the angle the Earth has turned do not stay in step. Leap seconds
close the gap.

## Mathematical meaning

The offset between TAI and UTC is a step function of the date:

```text
DTAI(date) = 10 s + N(date)
```

where `N(date)` is the number of leap seconds inserted between 1972-01-01
and the date. The kernel stores an instant as TAI, so the offset is applied
only when an instant is converted to or from UTC:

```text
tai_mjd = utc_mjd + DTAI(utc_mjd) / 86400
utc_mjd = tai_mjd - DTAI(utc_mjd) / 86400
```

The second relation needs the offset at the UTC instant, which is what is
being computed, so the table is searched for the interval that contains the
approximate instant and the result is checked for consistency. The
implementation does this by locating the table entry that applies.

An inserted second makes a UTC calendar day contain 86,401 seconds. A minute
that contains an insertion runs from second 0 to second 60.999, so the
seconds field of a calendar value may exceed 59.

## Unit convention

The table stores, for every insertion, the Modified Julian Date at which the
offset changes and the offset in seconds after the change. Both are exact in
the sense that the Modified Julian Date is the one the notice publishes,
converted to a day number.

## Applicability

The table starts at 1972-01-01, the beginning of the modern UTC system.
Before that date the offset was not an integer number of seconds and the
concept of a leap second did not apply; a conversion of such an instant
returns no value.

The table ends at the most recent insertion it contains. A later instant is
converted with the last known offset, which is correct until the next
insertion. A caller that needs certainty about the far future must treat
that as a forecast.

A leap second may in principle be negative, which would remove a second and
make a minute 59 seconds long, but no negative insertion has ever been
announced. The implementation handles the offset as a signed number, so a
negative insertion would be represented correctly, though it would produce a
UTC minute with no second 59.

## Why an instant is not stored on UTC

Storing an instant on UTC makes subtraction wrong across an insertion. Two
UTC seconds that are one tick apart in the table region can be 1 second or 0
seconds of elapsed time depending on which side of the step they fall. The
kernels avoid the question by storing TAI and converting at the edges. See
[[time-scales]].

## Comparison with the alternatives

| Approach | Elapsed time exact | Matches civil time | Handles insertions | Used here |
|----------|--------------------|--------------------|--------------------|-----------|
| Store TAI, convert at the edges | Yes | On display | Yes, from the table | Yes |
| Store UTC, subtract directly | No | Yes | Incorrectly at a step | No |
| Store TAI seconds since a fixed epoch | Yes | Needs conversion | Yes, from the table | For durations |
| Ignore leap seconds, use a fixed offset | Yes | No, off by up to a minute | No | No |
| Use UT1 throughout | Yes for rotation | No | Not applicable | No |

## Source

- Leap second announcements: IERS Bulletin C.
- Offset history: the table published with the bulletins, embedded by
  `apsis-leap`.

## See also

- [[time-scales]] for the scales and the storage convention
- [[calendar-and-mjd]] for the date conversions
