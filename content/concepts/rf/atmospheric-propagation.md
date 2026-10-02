---
title: Atmospheric propagation
tags: troposphere, ionosphere, scintillation, attenuation, fade
summary: What the air between a satellite and a receiver does to the signal, which is small in one band and decisive in another.
---

# Atmospheric propagation

The signal path from a satellite to a receiver crosses atmosphere. Below
about 10 GHz the atmosphere is close to transparent and the only loss is the
geometry. Above it the same air becomes the largest single term in the
budget, and above about 30 GHz it can be the reason a link does not close at
all.

## Definition

**Attenuation** is the reduction in signal power along the path. **Scintillation**
is the rapid fluctuation in amplitude and phase caused by refractive
irregularities. **Fade** is the interval during which the received signal falls
below what the link can use, whether because of attenuation, scintillation, or
the geometry.

The three are separate mechanisms with separate frequencies and separate
mitigations, and a design that handles one does not handle the others.

## Mathematics

### Tropospheric attenuation

Below about 10 GHz the clear-sky attenuation is the sum of contributions
from oxygen and water vapour, each with its own mass frequency absorption
curve. It is small, a fraction of a decibel at sea level, and nearly constant
over a pass.

Above about 10 GHz water vapour becomes significant, and above about 30 GHz
it dominates. The specific attenuation for a given rain rate is

$$
\gamma = \frac{k R^{\alpha}}{\cos\theta}
$$

where `R` is the rain rate, `theta` the zenith angle, and `k` and `alpha`
frequency-dependent. The `1 / cos theta` is why a low-elevation pass is
severe: at ten degrees from the horizon the path is about six times longer
through the layer.

### Rain fading

The fade duration is a function of how fast the terminal moves relative to a
rain cell. A high-elevation pass through a small cell gives a short, deep
fade; a low-elevation pass gives a long, shallow one. Design mitigation
therefore depends on the pass geometry, not only on the rain rate.

### Ionospheric scintillation

Above about 1 GHz the ionosphere becomes a refractive medium. The free
electron content produces a phase shift proportional to the inverse square of
frequency, and irregularities along the path produce rapid fluctuations. The
scintillation index is the conventional measure.

The effect is worst near the magnetic equator and near the plasma
frequency peak around 10 to 20 UTC, and it is the reason a link that works
at one station fails at another at the same moment.

## Unit conventions

- Attenuation and scintillation in decibels, index dimensionless.
- Rain rate in millimetres per hour; the models are tabulated for that unit.
- Free electron content in electrons per square metre, or in TEC units.
- Frequency in gigahertz; the losses scale with its square or higher power.

## Applicability

The band decides everything. At 1 to 10 GHz, which covers a large part of
the useful spectrum, the atmosphere is a detail in the budget. At Ku band it
is a design figure. At Ka and above it is the design.

The atmosphere also fails in a direction that a link budget does not show:
scintillation is a fade that depends on the path and the time, not on the
accumulated loss, so a link with adequate margin still degrades.

This is why a modern constellation uses different bands and different
mitigation for different shells, and why a handover that moves a user into
a scintillation-active region is scheduled rather than opportunistic.

## Standards

- Specific attenuation: ITU-R P.838
- Cross-polarisation and rain: ITU-R P.618
- Scintillation: ITU-R P.1546
- Local rain rate data: ITU-R P.837

## See also

- [[link-budget]] for the arithmetic these terms enter
- [[access-and-availability]] for whether the geometry allows a pass at all
- [[doppler-and-rate]] for what the measurement suffers