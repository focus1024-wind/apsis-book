---
title: The two-line element format
summary: The fixed width columns of a two-line element set, the checksum rule, and the two conventions that are easy to misread.
tags: tle, elements, format, checksum
---

# The two-line element format

A two-line element set, or TLE, is a fixed width text record that describes
the mean orbit of a catalogued satellite. It is what a catalogue publishes
and it is the input the SGP4 propagator is fitted to. This document gives
the layout, the checksum rule, and the units.

## Definition

A two-line element set has one optional name line followed by two data
lines, conventionally called line 1 and line 2. Every field occupies a fixed
range of columns, numbered from one. Each data line ends with a single
checksum digit.

```text
ISS (ZARYA)
1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927
2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537
```

## Field layout

Line 1:

| Columns | Field | Unit |
|---------|-------|------|
| 1 | Line number, always `1` | |
| 3-7 | Satellite catalog number | |
| 8 | Classification, `U`, `C`, or `S` | |
| 10-17 | International designator | |
| 19-32 | Epoch, two digit year and fractional day | year, day of year |
| 34-43 | First derivative of mean motion divided by two | rev/day^2 |
| 45-52 | Second derivative of mean motion divided by six | rev/day^3 |
| 54-61 | `BSTAR` drag term | 1 / Earth radius |
| 63 | Ephemeris type | |
| 65-68 | Element set number | |
| 69 | Checksum | |

Line 2:

| Columns | Field | Unit |
|---------|-------|------|
| 1 | Line number, always `2` | |
| 3-7 | Satellite catalog number | |
| 9-16 | Inclination | deg |
| 18-25 | Right ascension of the ascending node | deg |
| 27-33 | Eccentricity, assumed decimal point | dimensionless |
| 35-42 | Argument of perigee | deg |
| 44-51 | Mean anomaly | deg |
| 53-63 | Mean motion | rev/day |
| 64-68 | Revolution number at epoch | |
| 69 | Checksum | |

## Two conventions that are easy to misread

**The exponent fields carry an assumed decimal point.** An exponent field is
written as a sign, a mantissa without a decimal point, a sign for the
exponent, and a single exponent digit. The mantissa is read with a decimal
point in front of its first digit. So

```text
 12345-3  means  0.12345e-3
-11606-4  means -0.11606e-4
```

**The eccentricity has an assumed decimal point.** The field `0006703` means
`0.0006703`, not `6703`.

A field that is entirely blank is zero, which is how the format writes an
absent term.

## Mathematical meaning

The checksum of a line is the sum of the decimal digits on the line, plus
one for each minus sign, reduced modulo ten:

```text
checksum = (sum of digits + number of minus signs) mod 10
```

Only the first 68 columns contribute; the checksum occupies column 69 and is
not part of its own sum. For the example line 1 the sum is 167 and the
checksum is 7.

The epoch is written as a two digit year and a fractional day of that year.
The two digit year is expanded with the convention that years `57` through
`99` are in the twentieth century and years `00` through `56` are in the
twenty-first, which places the boundary before the start of the satellite
catalogue.

## Unit convention

The format is not self describing and its units are fixed:

- Orientation angles are in **degrees**.
- Mean motion is in **revolutions per day**.
- Derivatives of mean motion are per **day** to the matching power.
- `BSTAR` is in units of **inverse Earth radii**, the Earth radius being the
  equatorial radius in kilometers used by the propagator.
- Eccentricity is **dimensionless**.
- The epoch is a **UTC** instant.

The parser keeps the units of the format. The conversion to radians and to
radians per minute happens in the propagator, not in the parser, so a value
read from a parsed element set is exactly what the file says.

## Applicability

A two-line element set describes **mean** elements, not instantaneous ones.
The numbers are the input to the SGP4 model and are only meaningful together
with it. Feeding them to a two-body propagator gives a different and wrong
answer.

The format has no field for the accuracy of the element set and no field for
the propagation model to use. The convention that SGP4 is the model is
external to the format. Both facts are reasons to keep the element set
together with its source and the model that was used.

A two-line element set is valid for a limited interval around its epoch. The
propagator includes a drag term that models the decay of the orbit over that
interval, and the interval over which the model is useful is measured in
days to weeks depending on the altitude and the activity of the Sun.

## Comparison of element formats

| Format | Content | Model it is fitted to | Precision | Used here |
|--------|---------|-----------------------|-----------|-----------|
| Two-line element set | Mean elements, drag term | SGP4 and SDP4 | Kilometers over days | Yes |
| CCSDS OEM | Position and velocity or elements at a time grid | Any | Meters | No |
| CCSDS OPM | Position and velocity at one epoch | Any | Meters | No |
| SP3 | Satellite position at a regular grid | Any | Centimeters to meters | No |
| Broadcast ephemeris | Keplerian elements plus corrections | A GNSS orbit model | Meters | No |
| Precise ephemeris | Position and clock at a grid | Post-processed | Centimeters | No |

The two-line format is the least precise of these and the most widely
available, which is why it matters.

## Source

- The field layout follows the format description published by CelesTrak,
  *FAQs: Two-Line Element Set Format*, which documents the columns defined
  by the original NORAD specification. The checksum rule is the one stated
  there and used by Space-Track.

## See also

- [[sgp4]] for the propagator the elements are fitted to
- [[keplerian-elements]] for the elements of the orbit itself
- Try the parser in the online tool: *Parse a two-line element set*
