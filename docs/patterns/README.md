---
title: Patterns Library
---

Patterns Library
================

Currently, the patterns library contains only the patterns ported from
the [SuperCollider](https://supercollider.github.io/) programming language.




## List Patterns

- pfsm - Finite State Machine
- pindex - pattern that indexes into an array
- pswitch - embed values in a list according to a pattern of indices
- pclump - a pattern that takes another pattern and groups its values into
  arrays
- pgeom - geometric series pattern
- place - interlaced embedding of subarrays
- ppatlace - interlace streams
- prand - embed values randomly chosen from a list
- pseq - sequentially embed values in a list
- pser - sequentially embed values in a list
- pseries - arithmetic series pattern
- pshuf - sequentially embed values in a list in constant, but random order
- pslide - slide over a list of values and embed them
- ptuple - combine a list of streams to a stream of lists
- pwalk - a one-dimensional random walk over a list of values that are embedded
- pwrand - embed values randomly chosen from a list
- pxrand - embed values randomly chosen from a list




## Random Patterns

- pbeta - random values that follow a Eulerian Beta Distribution
- pbrown - brownian motion pattern
- pcauchy - random values that follow a Cauchy Distribution
- pexprand - random values that follow a Exponential Distribution
- pgauss - random values that follow a Gaussian Distribution
- pgbrown - geometric brownian motion pattern
- phprand - random values that tend toward hi
- plprand - random values that tend toward lo
- pmeanrand - random values that tend toward ((lo + hi) / 2)
- ppoisson - random values that follow a Poisson Distribution (positive integer
  values)
- pprob - random values with arbitrary probability distribution
- pwhite - random values with uniform distribution




## Repetition Patterns

- pclutch - sample and hold a pattern
- pconst - constrain the sum of a value pattern
- pdup - repeat input stream values
- pn - repeatedly embed a pattern
- pstutter - repeat input stream values
- psubdivide - partition a value into n equal subdivisions
