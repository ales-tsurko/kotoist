---
title: Repetition Patterns
---

Repetition Patterns
===================




## pclutch

Sample and hold a pattern.

```coffee
pattern = pclutch pseq([0,1,2,3], 2), pseq([true, true, false], 3)
expected = [0, 1, 1, 2, 3, 3, 0, 1, 1, ()]
for item in expected
  assert_eq pattern.next(), item
```

| Argument  | Description                                                                    | Default |
| --------  | -----------                                                                    | ------- |
| pattern   |                                                                                |         |
| connected | If `true`, the pattern plays as usual, if `false`, the previous value is kept. | true    |



## pconst

Embeds elements of the pattern into the stream until the sum comes close enough
to sum. At that point, the difference between the specified sum and the actual
running sum is embedded.

```coffee
pattern = pconst 5, pseq([1,2,0.5,0.1], 2)
expected = [1, 2, 0.5, 0.1, 1, 0.40000000000000036, ()]
for item in expected
  assert_eq pattern.next(), item
```

| Argument   | Description | Default |
| --------   | ----------- | ------- |
| sum        |             |         |
| pattern    |             |         |
| tollerance |             | 0.001   |




## pdup

Repeat each element n times.

```coffee
pattern = pdup 5, pseq([42,27], inf)
expected = [42,42,42,42,42]

for item in expected
  assert_eq pattern.next(), item
```

| Argument   | Description | Default |
| --------   | ----------- | ------- |
| n          |             |         |
| pattern    |             |         |




## pn

Repeatedly embed a pattern.

| Argument | Description                                 | Default  |
| -------- | -----------                                 | -------  |
| pattern  | The pattern to repeat.                      |          |
| repeats  | Repeats the enclosed pattern repeats times. | infinity |




## psubdivide

Subdivides each duration by each subdivision and yields that value n times. A
subdivision of 0 will skip the duration value, a subdivision of 1 yields the
duration value unaffected.

```coffee
pattern = psubdivide(
  pseq([1,1,1,1,1,2,2,2,2,2,0,1,3,4,0], inf),
  pseq([0.5, 1, 2, 0.25,0.25],inf)
)
expected = [
  0.5,
  1,
  2,
  0.25,
  0.25,
  0.25,
  0.25,
  0.5,
  0.5,
  1.0,
  1.0,
  0.125,
  0.125,
  0.125,
  0.125,
  1,
  0.6666666666666666,
  0.6666666666666666,
  0.6666666666666666,
  0.0625
]
for item in expected
  assert_near pattern.next(), item, 0.00001
```

| Argument   | Description | Default |
| --------   | ----------- | ------- |
| n          |             |         |
| pattern    |             |         |
