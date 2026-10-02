---
title: Doppler shift and range rate
tags: doppler, frequency-offset, clock, ranging, tracking
summary: How the motion of a satellite changes the frequency it appears to be received at, and how that shift is turned into a position.
---

# Doppler shift and range rate

A satellite is observed at a frequency that differs from the one it
transmits, because it is moving. That difference is the principal observable
in a tracking system, and turning it into a position is the whole of
ranging.

## Definition

**Doppler shift** is the change in observed frequency caused by relative
motion along the line of sight. **Range rate** is the rate of change of
range, and it is the same physical quantity: one is a frequency, the other a
velocity.

For a satellite moving at $v_r$ radially, at range $R$, the received
frequency shift is

$$
\frac{\Delta f}{f} = -\frac{v_r}{c}
$$

approaching raises the frequency. The non-relativistic form is adequate for
orbital speeds; the relativistic correction is about one part in $10^{13}$ and
matters only for a clock discipline experiment.

## Mathematics

### The two-way shift

A signal that travels to the satellite and back is shifted twice, so the
observed fractional shift is

$$
\frac{\Delta f}{f} = -\frac{2 v_r}{c}
$$

with $v_r$ the range rate. The factor of two is why the same satellite
appears to shift twice as much when tracked in two-way mode, and a range
model that ignores it has a factor-of-two error that is easy to miss.

### Range from accumulated shift

Integrating the two-way shift gives the range,

$$
R(t) = R_0 - \int \frac{2 c \, v_r}{f_0} \, dt
$$

which is why a tracking system measures range as the integral of a
frequency rather than as a timing. The accumulator drifts with any error in
the reference frequency, which is why the reference clock is the limiting
element in the whole chain.

### Range rate from the carrier

Alternatively the carrier phase is differenced directly, giving range rate
without an accumulator. This is the common approach and it is noisy, because
phase noise differentiates into range rate noise amplified by the carrier
frequency. The trade is a clean velocity against a degraded one.

### Clock offset

Separately from the motion, the transmitted frequency may not equal the
nominal one. A **frequency offset** between the spacecraft clock and the
reference is indistinguishable from a range rate over a short arc, because
both move the phase. Separating them needs a second observable, typically a
range measurement from a different station, or an upload that is
self-referenced.

## Unit conventions

- Frequencies in hertz; a shift is often expressed in parts per billion.
- Range rate in metres per second or kilometres per second.
- Range in kilometres, with the carrier frequency in hertz, so the factor
  $2c/f$ carries its units.
- The factor of two applies to two-way; one-way is the single case.

## Applicability

Doppler is insensitive to noise that is common to the uplink and the
downlink, which is why it is the observable of choice in a jamming
environment: a jammer must be coherent to spoof it.

The shift is large enough to measure and small enough to alias. If the
estimate of range rate is off by more than half a carrier cycle per sample
interval, the tracking wraps and the recovered position is wrong by an
arbitrary amount. This is a real failure mode in practice and it is why the
sample rate is bounded relative to the dynamics.

A one-way system needs a stable reference on the spacecraft; a two-way system
needs a stable reference on the ground. Either way the clock, not the
antenna, is usually the limit.

## Standards

- Range and Doppler conventions: the tracking and data link messages
  specified by the CCSDS Space Link Protocol
- Reference oscillator discipline: the same documents, in the time and
  frequency parts

## See also

- [[link-budget]] for whether there is enough signal to measure at all
- [[interference]] for what a coherent jammer does to this measurement
- [[tt-and-c]] for the full measurement chain