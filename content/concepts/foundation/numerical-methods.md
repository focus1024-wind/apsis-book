---
title: Numerical method and error
tags: numerical, error, iteration, conditioning, convergence
summary: The behaviour of a computed number that no exact answer exists for, and the differences between being wrong, being imprecise, and being unstable.
---

# Numerical method and error

Most quantities in orbital work have no closed form. The period of an
elliptical orbit under a perturbed field, the argument of perigee after a drag
history, the instant a satellite enters a cone of view: these are defined by an
equation that must be solved, not evaluated. Solving it numerically replaces
one question, which has an answer, with another, which does not.

## Definition

Three different things go wrong when a number is computed, and they are worth
keeping apart:

- **Truncation error.** The method computes an approximation to the exact
  answer by stopping early or by replacing a function with a simpler one.
- **Rounding error.** The representation of the number itself is finite. A
  double-precision value holds about fifteen to seventeen significant
  decimal digits; beyond that the digits are noise.
- **Instability.** A small error in the input, or in an intermediate step,
  grows into a large error in the answer.

The first two are bounded and can be estimated. The third is a property of the
problem and cannot be estimated from the arithmetic; it has to be recognised.

## Mathematics

### Iteration

A fixed point is reached by repeated substitution, `x` becoming `g(x)`. It
converges when the derivative of `g` at the root is small in magnitude:

$$
\lvert g'(x) \rvert < 1 \quad \text{(convergence)}, \qquad
\lvert g'(x) \rvert > 1 \quad \text{(divergence)}
$$

Kepler's equation, `M = E - e sin E`, has no closed form solution for `E`. It
is solved by Newton's method,

$$
E_{n+1} = E_n - \frac{E_n - e \sin E_n - M}{1 - e \cos E_n}
$$

which converges quadratically near the root when `e` is not close to one. The
iteration is fast and its rate depends on eccentricity; for a near-parabolic
orbit the denominator approaches zero and a different formulation is needed.

A bisection is slower per step but cannot diverge, because it halves an
interval that is known to bracket the root. Which is preferable depends on
whether divergence or cost dominates.

### Conditioning

The sensitivity of an answer to a perturbation of its input is the
**condition number**. For a function `f`,

$$
\kappa = \frac{\lvert f'(x) \rvert \, \lvert x \rvert}{\lvert f(x) \rvert}
$$

A large condition number means a small input error becomes a large output
error, whatever the method. Near a singular point the condition number
diverges: computing the radius of an orbit from its semi-major axis and
eccentricity is well conditioned, while computing the time to perigee from the
same elements is not, because `sqrt(1 - e^2)` appears in the denominator and
goes to zero as `e` approaches one. The problem is ill conditioned, not the
arithmetic, and more digits do not help.

### Truncation in step size

An integration over a fixed step accumulates a global error proportional to
the step, and a local error proportional to the step squared. Halving the
step therefore improves the answer by a factor of two, not four, which is why
"just halve the step" is an expensive way to buy accuracy.

For a method of order `p`, the global error falls as the step to the power
`p`. A fourth order Runge-Kutta gives `h^4`, which is why it is common: the
cost per step is four function evaluations, and halving the step costs eight
times as much for a factor of sixteen less error.

## Unit conventions

- Tolerance is relative to a scale of the answer, never absolute. An absolute
  tolerance of `1e-9` on a radius of `7000` km is far tighter than the
  representation can hold, and the iteration will not terminate.
- The scale used for the relative tolerance should be the magnitude of the
  quantity, not the quantity's units, so that switching between kilometres and
  metres does not change the answer.

## Applicability

A method that converges is not necessarily accurate. Newton's method on
Kepler's equation converges to the root of the equation as posed; if the
equation omits a perturbation the solution is the root of the wrong problem,
and it will converge confidently.

Iteration also fails silently when it stops on a loose criterion. Reporting
the number of iterations rather than only the answer lets a caller see that a
result was reached in the expected number of steps.

The three failure modes are separable in practice: rounding shows up as a
floor on achievable accuracy, truncation as an error that shrinks when the
method is refined, and instability as an error that grows.

## Standards

- Newton, *De analysi per aequationes numero terminas infinitas*, 1669
- Higham, *Accuracy and Stability of Numerical Algorithms*, 2nd edition, 2002
- Quarteroni, Sacco, Saleri, *Numerical Mathematics*, 2nd edition, 2007

## See also

- [[units-and-dimensions]] for what the numbers mean
- [[ephemeris-interpolation]] for the error of a sampled table
- [[higher-order-gravity]] for a model where the series does not converge