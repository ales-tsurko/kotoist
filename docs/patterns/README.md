---
title: Patterns Library
---

Patterns Library
================

Currently, the patterns library contains only the patterns ported from
the [SuperCollider](https://supercollider.github.io/) programming language.




## List Patterns

- [pfsm](list#pfsm-2) - Finite State Machine
- [pindex](list#pindex-3) - pattern that indexes into an array
- [pswitch](list#pswitch-4) - embed values in a list according to a
  pattern of indices
- [pclump](list#pclump-5) - a pattern that takes another pattern and
  groups its values into arrays
- [pgeom](list#pgeom-6) - geometric series pattern
- [place](list#place-7) - interlaced embedding of subarrays
- [prand](list#prand-9) - embed values randomly chosen from a list
- [pseq](list#pseq-10) - sequentially embed values in a list
- [pser](list#pser-11) - sequentially embed values in a list
- [pseries](list#pseries-12) - arithmetic series pattern
- [pshuf](list#pshuf-13) - sequentially embed values in a list in
  constant, but random order
- [pslide](list#pslide-14) - slide over a list of values and embed them
- [ptuple](list#ptuple-15) - combine a list of streams to a stream of lists
- [pwalk](list#pwalk-16) - a one-dimensional random walk over a list of
  values that are embedded
- [pwrand](list#pwrand-17) - embed values randomly chosen from a list
- [pxrand](list#pxrand-18) - embed values randomly chosen from a list




## Random Patterns

- [pbeta](random#pbeta-2) - random values that follow a Eulerian Beta Distribution
- [pbrown](random#pbrown-3) - brownian motion pattern
- [pcauchy](random#pcauchy-4) - random values that follow a Cauchy Distribution
- [pexprand](random#pexprand-5) - random values that follow a Exponential Distribution
- [pgauss](random#pgauss-6) - random values that follow a Gaussian Distribution
- [pgbrown](random#pgbrown-7) - geometric brownian motion pattern
- [phprand](random#phprand-8) - random values that tend toward hi
- [plprand](random#plprand-9) - random values that tend toward lo
- [pmeanrand](random#pmeanrand-10) - random values that tend toward ((lo + hi) / 2)
- [ppoisson](random#ppoisson-11) - random values that follow a Poisson Distribution (positive integer
  values)
- [pwhite](random#pwhite-12) - random values with uniform distribution




## Repetition Patterns

- [pclutch](repetiotion#pclutch-2) - sample and hold a pattern
- [pconst](repetiotion#pconst-3) - constrain the sum of a value pattern
- [pdup](repetiotion#pdup-4) - repeat input stream values
- [pn](repetiotion#pn-5) - repeatedly embed a pattern
- [psubdivide](repetiotion#psubdivide-7) - partition a value into n
  equal subdivisions
