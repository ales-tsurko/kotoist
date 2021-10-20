---
title: Random Patterns
---

Random Patterns
===============




## pbeta

Random values that follow a Eulerian Beta Distribution.

![pbeta](/graphs/pbeta.png "pbeta plot")

| Argument | Description                                      | Default  |
| -------- | -----------                                      | -------  |
| lo       | Lower boundary of values.                        | 0        |
| hi       | Upper boundary of values.                        | 1        |
| prob1    | The probability that a value will occur near lo. | 1        |
| prob2    | The probability that a value will occur near hi. | 1        |
| length   | Number of values produced.                       | infinity |




## pbrown

Returns a stream that behaves like a brownian motion.

![pbrown](/graphs/pbrown.png "pbrown plot")

| Argument | Description                                      | Default  |
| -------- | -----------                                      | -------  |
| lo       | Lower boundary of values.                        | 0        |
| hi       | Upper boundary of values.                        | 1        |
| step     | Maximum change per step.                         | 0.125    |
| length   | Number of values produced.                       | infinity |




## pcauchy

Random values that follow a Cauchy Distribution.

![pcauchy](/graphs/pcauchy.png "pcauchy plot")

| Argument | Description                                                                    | Default  |
| -------- | -----------                                                                    | -------  |
| mean     | The mean of the distribution.                                                  | 0        |
| spread   | The horizontal dispersion of the random values. The distribution is unbounded. | 1        |
| length   | Number of values produced.                                                     | infinity |




## pexprand

Random values that follow a Exponential Distribution.

> NOTE: lo and hi should both be positive or negative (their range should not cross 0).

![pexprand](/graphs/pexprand.png "pexprand plot")

| Argument | Description                                      | Default  |
| -------- | -----------                                      | -------  |
| lo       | Lower boundary of values.                        | 0.0001   |
| hi       | Upper boundary of values.                        | 1        |
| length   | Number of values produced.                       | infinity |




## pgauss

This pattern uses the Box-Muller transform to generate a gaussian distribution
from uniformly distributed values.

![pgauss](/graphs/pgauss.png "pgauss plot")

| Argument | Description                                                | Default  |
| -------- | -----------                                                | -------  |
| mean     | The mean of the distribution.                              | 0        |
| dev      | The spread of values around the mean (standard deviation). | 1        |
| length   | Number of values produced.                                 | infinity |




## pgbrown

Returns an iterator that behaves like a geometric brownian motion.

![pgbrown](/graphs/pgbrown.png "pgbrown plot")

| Argument | Description                                      | Default  |
| -------- | -----------                                      | -------  |
| lo       | Lower boundary of values.                        | 0        |
| hi       | Upper boundary of values.                        | 1        |
| step     | Maximum multiplication factor per step (omega).  | 0.125    |
| length   | Number of values produced.                       | infinity |




## phprand

Random values that tend toward hi.

![phprand](/graphs/phprand.png "phprand plot")

| Argument | Description                                      | Default  |
| -------- | -----------                                      | -------  |
| lo       | Lower boundary of values.                        | 0        |
| hi       | Upper boundary of values.                        | 1        |
| length   | Number of values produced.                       | infinity |




## plprand

Random values that tend toward lo.

![plprand](/graphs/plprand.png "plprand plot")

| Argument | Description                                      | Default  |
| -------- | -----------                                      | -------  |
| lo       | Lower boundary of values.                        | 0        |
| hi       | Upper boundary of values.                        | 1        |
| length   | Number of values produced.                       | infinity |




## pmeanrand

Random values that tend toward ((lo + hi) / 2).

![pmeanrand](/graphs/pmeanrand.png "pmeanrand plot")

| Argument | Description                                      | Default  |
| -------- | -----------                                      | -------  |
| lo       | Lower boundary of values.                        | 0        |
| hi       | Upper boundary of values.                        | 1        |
| length   | Number of values produced.                       | infinity |




## ppoisson

Random values that follow a Poisson Distribution (positive integer values).

![ppoisson](/graphs/ppoisson.png "ppoisson plot")

| Argument | Description                   | Default  |
| -------- | -----------                   | -------  |
| mean     | The mean of the distribution. | 1        |
| length   | Number of values produced.    | infinity |




## pwhite

Random values with uniform distribution.

![pwhite](/graphs/pwhite.png "pwhite plot")

| Argument | Description                                      | Default  |
| -------- | -----------                                      | -------  |
| lo       | Lower boundary of values.                        | 0        |
| hi       | Upper boundary of values.                        | 1        |
| length   | Number of values produced.                       | infinity |
