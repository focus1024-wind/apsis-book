---
title: Units, dimensions and conversion
tags: units, dimensions, si, conversion
summary: How physical quantities carry their unit, why a unit is a type rather than a label, and the conventions that make an arithmetic answer meaningful.
---

# Units, dimensions and conversion

A number without a unit is not a physical quantity. "3" might be three metres,
three seconds, or three kilometres per hour, and the three do not describe the
same thing. The purpose of a unit system is to make an arithmetic result
interpretable, which means the unit is not a note attached to a value but part
of the value itself.

## Definition

A **quantity** is a magnitude together with a **unit**. The **dimension** is
the kind of thing measured, such as length, time, or mass; a unit is a
particular choice of magnitude within that dimension.

The base dimensions of the International System of Units, and the units chosen
for them, are:

| Dimension | Unit | Defined by |
|-----------|------|-----------|
| Length | metre | Speed of light in vacuum |
| Mass | kilogram | Planck constant |
| Time | second | Cesium hyperfine transition |
| Electric current | ampere | Elementary charge |
| Thermodynamic temperature | kelvin | Boltzmann constant |
| Amount of substance | mole | Avogadro constant |
| Luminous intensity | candela | Defined by the other six |

Everything else is **derived**: newton from kilogram, metre and second; joule
from newton and metre; watt from joule and second.

## Mathematics

A unit is a conversion factor to a base unit. Length may be expressed in
metres, kilometres or feet; each is a number multiplied by a factor to give
metres. Dimensionally aware arithmetic keeps track of that factor, so

$$
1\,\text{km} + 500\,\text{m} = 1500\,\text{m}, \quad \text{never } 1 + 500 = 501
$$

The rule is that the conversion factor travels with the value. A quantity of
type length carries the factor to metres with it, so adding it to another
length applies both factors at once.

A dimension behaves like a type in a type system. Adding a length to a time is
a dimension error, in the same way that adding a string to an integer would be
one in a compiler that tracked those types.

### Scaling laws

Dimension reasoning earns its keep in scaling arguments, where an error shows
up as an impossible exponent. The gravitational perturbation at distance `r`
from a body of mass `M` must scale as

$$
a \propto \frac{G M}{r^2}
$$

because `G` has units of `m^3 kg^-1 s^-2`; any other combination either does
not have the dimensions of an acceleration or does have them only for a
particular case, which is the signal that the argument is incomplete.

## Unit conventions

- **Angles.** An angle is dimensionless in principle, because a full turn is
  one. It is written in radians in most formulae, and in degrees, arcminutes
  and arcseconds where it is read by a person. The relationship is
  `180 / pi` degrees per radian. A rotation described in axis and angle uses
  the angle in a plane and the axis through the pole; the two conventions
  differ, which is why a rotation is usually written explicitly as
  right-hand or left-hand.
- **Prefixes.** SI prefixes are powers of ten: kilo is `10^3`, mega `10^6`,
  micro `10^-6`. A kilometre is a thousand metres, never a thousand miles.
- **Binary prefixes.** A mebibyte is `2^20` bytes, not `10^6`. The distinction
  matters wherever a size is compared against a capacity.
- **Units in the field.** Orbital work is written in kilometres and seconds,
  not metres, because the quantities are large and the models are fitted to
  those units. A propagation model expressed in metres and a catalogue
  expressed in kilometres will disagree by a factor of a thousand, silently.

## Applicability

Unit arithmetic prevents a class of error that is otherwise easy to make and
hard to see, because the numbers stay plausible. It does not prevent a
different class: the same unit used with two different conventions. A height
above the equatorial radius and a height above the ellipsoid differ by tens of
kilometres; both are lengths, and neither is wrong.

The measure a quantity is taken against matters as much as its unit. An epoch
expressed in seconds from an unspecified origin is not comparable with one
expressed in a Modified Julian Date, even though both are times.

## Standards

- SI: *The International System of Units (SI)*, BIPM, most recent edition
- Non-SI units accepted for use with SI: the same document, section 4
- Earth reference models: NIMA TR8350.2 for WGS 84

## See also

- [[numerical-methods]] for what the numbers are computed with
- [[time-scales]] for a quantity whose unit hides an entire convention
- [[geodesy-wgs84]] for the two measures a height is taken against