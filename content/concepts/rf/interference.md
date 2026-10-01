---
title: Interference
tags: interference, jamming, spoofing, agc, cn0
summary: The difference between noise, which degrades a measurement gradually, and a coherent source, which replaces the signal with a false one.
---

# Interference

Interference is anything in the band that is not the wanted signal. The
engineering consequences depend entirely on whether it is uncorrelated noise,
in which case the measurement degrades smoothly, or a coherent source, in
which case the receiver may be reporting something that was never sent.

## Definition

- **Noise** is uncorrelated with the wanted signal. It raises the noise floor
  and reduces the carrier-to-noise density ratio, which shortens the usable
  arc and increases the ranging error.
- **Jamming** is a high-power source that drives the front end out of its
  linear range, so that the wanted signal is no longer recoverable at all.
- **Spoofing** is a coherent source structured to look like the wanted
  signal, at a level below the jamming threshold, so that the receiver
  tracks the wrong position and the link appears healthy.

The distinction is not about power. It is about coherence, and it is what
separates a system that fails loudly from one that fails quietly.

## Mathematics

### Carrier to noise density

The usable quantity is the carrier to noise density ratio, written as
`C/N0`, which is the carrier power in a one hertz bandwidth over the noise
power in the same bandwidth.

```text
C/N0 = P_r * G / (k * T_system * F)
```

A pointing loss, a rain fade or a jammer each reduces it, and the range error
scales with the square root of it. A jammer that costs six decibels of
`C/N0` does considerably more than six decibels of ranging accuracy.

### The threshold

A receiver has a usable range of input power, between a floor set by its
noise figure and a ceiling set by where the analogue chain saturates. Above
the ceiling, gain compression makes the output a function of the total power
rather than of the wanted signal, and the receiver is blind to whether the
wanted signal is present.

This is why jamming at a moderate level can be more effective than jamming
at a high level: a strong jammer is detected and a moderate one is not.

### Coherence

A spread spectrum receiver correlates the incoming signal against a known
code. Uncorrelated noise does not correlate, so it is rejected in
proportion to the processing gain. A coherent jammer that knows the code, or
that replays it, is not rejected, because it satisfies the correlation. This
is the property that makes the Doppler observable attractive in this
environment: a jammer must reproduce the motion to corrupt it.

## Unit conventions

- Power in decibels relative to one milliwatt; `C/N0` in decibels hertz or
  dBHz.
- Noise figure dimensionless, in decibels.
- Signal strength on an unknown source cannot be converted without a
  reference, which is why interference reporting is inherently relative.

## Applicability

The mitigation available determines the risk. Frequency agility and adaptive
coding help against noise and against a wideband jammer, and do nothing
against a jammer that follows the agility. Direction finding and nulling
help against a jammer whose position is not co-located with the wanted
source, and do nothing against one that is.

An automatic gain control is a liability in this context: it is what makes a
receiver report a constant output power, which is exactly the behaviour that
lets a jammer hide in a nominal-looking reading.

Reporting interference is hard in the absence of a reference, which is why
practically it is done by observing that the expected signal is absent rather
than by characterising what replaced it.

## Standards

- Interference reporting: the Radio Regulations of the International
  Telecommunication Union
- Signal quality reporting: the CCSDS recommendations on tracking data
  messages

## See also

- [[link-budget]] for the margin a jammer consumes
- [[doppler-and-rate]] for why the measurement can be corrupted rather than
  degraded
- [[operations-planning]] for the operational response