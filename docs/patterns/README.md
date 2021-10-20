---
title: Patterns Library
---

Patterns Library
================

Currently, the patterns library contains only the patterns ported from
the [SuperCollider](https://supercollider.github.io/) programming language.




## List Patterns

- [pfsm](patterns/list#pfsm-2) - Finite State Machine
- [pindex](patterns/list#pindex-3) - pattern that indexes into an array
- [pswitch](patterns/list#pswitch-4) - embed values in a list according to a
  pattern of indices
- [pclump](patterns/list#pclump-5) - a pattern that takes another pattern and
  groups its values into arrays
- [pgeom](patterns/list#pgeom-6) - geometric series pattern
- [place](patterns/list#place-7) - interlaced embedding of subarrays
- [prand](patterns/list#prand-9) - embed values randomly chosen from a list
- [pseq](patterns/list#pseq-10) - sequentially embed values in a list
- [pser](patterns/list#pser-11) - sequentially embed values in a list
- [pseries](patterns/list#pseries-12) - arithmetic series pattern
- [pshuf](patterns/list#pshuf-13) - sequentially embed values in a list in
  constant, but random order
- [pslide](patterns/list#pslide-14) - slide over a list of values and embed them
- [ptuple](patterns/list#ptuple-15) - combine a list of streams to a stream of lists
- [pwalk](patterns/list#pwalk-16) - a one-dimensional random walk over a list of
  values that are embedded
- [pwrand](patterns/list#pwrand-17) - embed values randomly chosen from a list
- [pxrand](patterns/list#pxrand-18) - embed values randomly chosen from a list




## Random Patterns

- [pbeta](patterns/random#pbeta-2) - random values that follow a Eulerian Beta Distribution
- [pbrown](patterns/random#pbrown-3) - brownian motion pattern
- [pcauchy](patterns/random#pcauchy-4) - random values that follow a Cauchy Distribution
- [pexprand](patterns/random#pexprand-5) - random values that follow a Exponential Distribution
- [pgauss](patterns/random#pgauss-6) - random values that follow a Gaussian Distribution
- [pgbrown](patterns/random#pgbrown-7) - geometric brownian motion pattern
- [phprand](patterns/random#phprand-8) - random values that tend toward hi
- [plprand](patterns/random#plprand-9) - random values that tend toward lo
- [pmeanrand](patterns/random#pmeanrand-10) - random values that tend toward ((lo + hi) / 2)
- [ppoisson](patterns/random#ppoisson-11) - random values that follow a Poisson Distribution (positive integer
  values)
- [pwhite](patterns/random#pwhite-12) - random values with uniform distribution




## Repetition Patterns

- [pclutch](patterns/repetiotion#pclutch-2) - sample and hold a pattern
- [pconst](patterns/repetiotion#pconst-3) - constrain the sum of a value pattern
- [pdup](patterns/repetiotion#pdup-4) - repeat input stream values
- [pn](patterns/repetiotion#pn-5) - repeatedly embed a pattern
- [psubdivide](patterns/repetiotion#psubdivide-7) - partition a value into n
  equal subdivisions
