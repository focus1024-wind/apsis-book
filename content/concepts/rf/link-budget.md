---
title: Link budget
tags: link-budget, eirp, g-t, fspl, margin, rain
summary: The arithmetic of whether a signal arrives strong enough to be used, and every term that turns a comfortable design into an impossible one.
---

# Link budget

A link budget is the addition of every gain and loss between two antennas,
compared with the power needed to be heard. It is arithmetic, but it is
arithmetic on quantities that are all uncertain, and the margin left over is
the only thing standing between a design and an outage.

## Definition

The received power at the far end is

$$
P_r = \frac{\text{EIRP} \, G_t}{L}
$$

with `L` the total propagation and implementation loss. The link closes when
`P_r` exceeds the sensitivity of the receiver, and the **link margin** is the
difference in decibels.

Every term is a ratio, so the budget is written in decibels throughout:

$$
\begin{aligned}
\text{EIRP} &= P_t + G_t - L_{\text{feeder}} \\
P_r &= \text{EIRP} + G_r - L_{\text{space}} - L_{\text{pol}} - L_{\text{point}} \\
\text{margin} &= P_r - \text{sensitivity}
\end{aligned}
$$

The two that decide the result are free space loss, which is fixed by
geometry, and the losses that are not: rain, pointing, polarisation
mismatch.

## Mathematics

### Free space loss

The loss grows with the fourth power of distance and with the square of
frequency:

$$
L_{\text{fs}} = \left( \frac{4 \pi d f}{c} \right)^2
$$

in linear units, or in decibels,

$$
L_{\text{fs,dB}} = 92.45 + 20 \log_{10} d_{\text{km}} + 20 \log_{10} f_{\text{GHz}}
$$

The fourth power is why doubling the frequency costs six decibels and
doubling the distance costs six. A constellation at 550 km versus one at
1100 km has twelve decibels less margin for the same equipment, which is more
than a doubling of the transmit power recovers, because doubling power is only
three decibels.

### Polarisation loss

An antenna receives a polarisation close to its own. A mismatch of an angle
`psi` costs

$$
L_{\text{pol}} = -10 \log_{10} \left( \cos^2 \psi \right) \ \text{dB}
$$

which is small below a few degrees and large beyond thirty. A tilted
heliostat or a rotating spacecraft body can put a link over this threshold
during a pass, and this is a common cause of an outage that appears only in
part of the orbit.

### Rain attenuation

Attenuation from rain at frequencies above 10 GHz is significant. It scales
roughly with `f^0.75` up to about 30 GHz and is then given by the specific
attenuation `gamma` times the path length through the rain cell. The design
figure is not the average attenuation but the value exceeded for a stated
fraction of the worst hour in a year, because the budget is closed against
the rain, not against the median.

### Pointing and pointing loss

Pointing loss is the amount of beam that misses. It is small when the ground
station tracks a narrow beam well and large when a wide beam is used to reach
a mobile user, where the beam is pointed at the user rather than at a fixed
station.

### Margin

Margin is what is left after every term. What it is spent on depends on what
was left out: a modest margin covers pointing jitter and small fades; it does
not cover a rain cell that was sized from an average.

## Unit conventions

- Power in decibels relative to one milliwatt, abbreviated dBm.
- Gains and losses in decibels.
- Distances in kilometres, frequencies in gigahertz, which is the form the
  free space formula is usually written in.
- EIRP in dBW when the figure is compared with a spacecraft power, dBm when
  with a receiver.

## Applicability

A budget computed with average losses over a terrain and an average rain rate
does not close a link. The two terms that have to be exceeded rather than
averaged are rain and pointing, and they are the two that cause outages in
systems whose budget was signed off.

Above roughly 30 GHz the rain attenuation keeps rising and the fade margin
required becomes large enough to dominate the hardware. This is the reason
the geostationary bands fill the sky below about 10 GHz and the higher bands
are used where rain is rare or where the link can afford a large antenna.

A budget does not account for interference, which is not a loss but a
corruption; it needs [[interference]] treated separately.

## Standards

- Free space loss and the link budget convention: ITU-R recommendations
  on radiowave propagation and on the satellite broadcasting and
  telecommunications services
- Rain attenuation: ITU-R P.838 and P.618
- Regional rain rate data: ITU-R P.837

## See also

- [[doppler-and-rate]] for what is measured once the link closes
- [[atmospheric-propagation]] for the medium the budget travels through
- [[constellations]] for how the budget sets the orbital shell