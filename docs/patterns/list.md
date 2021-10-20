---
title: List Patterns
---

List Patterns
=============




## pfsm

Every state consists of an item and an array of integer indices of possible
**next states**. The initial state is chosen at random from the array of 
**entry states**.  That chosen state's item is returned, and the next state is
chosen from its array of possible **next states**. When the end state is chosen,
the stream ends.

```coffee
random.seed(1)
pattern = pfsm([
    [0,1],
    67, [0, 0, 3],
    72, [2],
    73, [0, 2],
    pseq([74, 75, 76, 77]), [2, 3, 3],
    (), ()], inf)
expected = [72,73,73,73,73,73,73,67,67,67,67,74,75,76,77,73,73,73,67,73]

for item in expected
  assert_eq pattern.next(), item
```

| Argument | Description       | Default |
| -------- | -----------       | ------- |
| list     |                   |         |
| repeats  | Number of repeats | 1       |




## pindex

Pattern that indexes into an array.

```coffee
pattern = pindex(
  [7, 13, 12, 2, 2, 2, 5], 
  pseq([0, 0, 2, 0, 4, 6, 7], inf), 
  inf)
expected = [7, 7, 12, 7, 2, 5, 7]
for item in expected
  assert_eq pattern.next(), item
```

| Argument | Description       | Default |
| -------- | -----------       | ------- |
| list     |                   |         |
| index    |                   |         |
| repeats  | Number of repeats | 1       |




## pswitch

pswitch chooses elements from the **list** by a stream of indices (**which**)
and embeds them in the stream. If the element is itself a pattern, it first
completely embeds it before looking for the next index.

```coffee
a = pseq [1, 2, 3], 2
b = pseq [65, 76]
pattern = pswitch [a, b, 800], pseq([2, 2, 0, 1], inf)
expected = [800,800,1,2,3,1,2,3,65,76,800,800]
for item in expected
  assert_eq pattern.next(), item
```

| Argument | Description       | Default |
| -------- | -----------       | ------- |
| list     |                   |         |
| which    |                   | 0       |




## pclump

Groups the source pattern into arrays whose size is given by n.

E.g. If the source pattern has 5 elements and you choose a clump size of 2, the
new pattern will return two arrays containing 2 elements and a final array
containing 1 element.

```coffee
pattern = pclump 2, pseq([1,2,3])
expected = [[1,2], [3], ()]
for item in expected
  assert_eq pattern.next(), item
```

| Argument | Description                                                                                               | Default |
| -------- | -----------                                                                                               | ------- |
| n        | An integer, or a pattern that returns an integer. This integer will determine the size of the next clump. |         |
| pattern  | The pattern to be filtered.                                                                               |         |




## pgeom

Returns an iterator that behaves like a geometric series.

```coffee
pattern = pgeom 1, 1.1
expected = [
  1,
  1.1,
  1.2100000000000002,
  1.3310000000000004,
  1.4641000000000006,
  1.6105100000000008,
  1.771561000000001,
  1.9487171000000014,
  2.1435888100000016,
  2.357947691000002,
  2.5937424601000023,
  2.853116706110003,
  3.1384283767210035,
  3.4522712143931042,
  3.797498335832415,
  4.177248169415656,
  4.594972986357222,
  5.054470284992944,
  5.559917313492239,
  6.115909044841463
]
for item in expected
  assert_eq pattern.next(), item
```

| Argument | Description                | Default  |
| -------- | -----------                | -------  |
| start    | Start value.               | 0        |
| grow     | Multiplication factor.     | 1        |
| length   | Number of values produced. | infinity |




## place

Returns elements in the list. If an element is a list itself, it embeds the
first element when it comes by first time, the second element when it comes by
the second time... The nth when it comes by the nth time.

```coffee
pattern = place [1, [2, 5], [3, 6]], inf
expected = [1,2,3,1,5,6,1,2]
for item in expected
  assert_eq pattern.next(), item

pattern = place(
  [1, pseq([2, 5], inf), pseq([3, 6], inf)], inf)
for item in expected
  assert_eq pattern.next(), item
```

| Argument | Description       | Default |
| -------- | -----------       | ------- |
| list     |                   |         |
| repeats  | Number of repeats | 1       |




## prand

Embed one item from the list at random for each repeat.

```coffee
pattern = prand (0..10).to_list(), inf
expected = [6,1,5,4,9,6,4,6,6,8,2,2,8,4,0,7,6,1,7,5]
for item in expected
  assert_eq pattern.next(), item
```

| Argument | Description       | Default |
| -------- | -----------       | ------- |
| list     |                   |         |
| repeats  | Number of repeats | 1       |




## pseq

Cycles over a list of values. The repeats variable gives the number of times to
repeat the entire list.

```coffee
pattern = pseq [0,1,2], inf
expected = [0,1,2,0,1]
for item in expected
  assert_eq pattern.next(), item
```

| Argument | Description       | Default |
| -------- | -----------       | ------- |
| list     |                   |         |
| repeats  | Number of repeats | 1       |
| offset   |                   | 0       |




## pser

is like pseq, however the repeats variable gives the number of items returned
instead of the number of complete cycles.

```coffee
pattern = pser [1, 2, 3], 5
expected = [1, 2, 3, 1, 2, ()]
for item in expected
  assert_eq pattern.next(), item
```

| Argument | Description       | Default |
| -------- | -----------       | ------- |
| list     |                   |         |
| repeats  | Number of repeats | 1       |
| offset   |                   | 0       |




## pseries

Returns a stream that behaves like an arithmetic series.

```coffee
pattern = pseries 300, 20, 20
expected = [
  300,
  320,
  340,
  360,
  380,
  400,
  420,
  440,
  460,
  480,
  500,
  520,
  540,
  560,
  580,
  600,
  620,
  640,
  660,
  680
]
for item in expected
  assert_eq pattern.next(), item
```

| Argument | Description                | Default  |
| -------- | -----------                | -------  |
| start    | Start value.               | 0        |
| step     | Addition factor.           | 1        |
| length   | Number of values produced. | infinity |




## pshuf

Returns a shuffled version of the **list** item by item, with n **repeats**.

```coffee
pattern = pshuf (1..5).to_list(), 3
expected = [3,1,4,2,3,1,4,2,3,1,4,2,()]
for item in expected
  assert_eq pattern.next(), item
```

| Argument | Description                | Default  |
| -------- | -----------                | -------  |
| list     |                            |          |
| repeats  | Number of repeats.         | 1        |




## pslide

Slide over a list of values and embed them.

```coffee
pattern = pslide [1, 2, 3, 4, 5], inf
expected = [1, 2, 3, 2, 3, 4, 3, 4, 5, 4, 5, 1, 5, 1, 2, 1, 2, 3, 2, 3]
for item in expected
  assert_eq pattern.next(), item
```

| Argument    | Description                                                                                                                                                   | Default |
| --------    | -----------                                                                                                                                                   | ------- |
| list        |                                                                                                                                                               |         |
| repeats     | Number of segments.                                                                                                                                           | 1       |
| step        | How far to step the start of each segment from previous. step can be negative.                                                                                | 3       |
| start       | What index to start at.                                                                                                                                       | 0       |
| wrap_at_end | If true (default), indexing wraps around if goes past beginning or end. If false, the pattern stops if it hits a nil element or goes outside the list bounds. | true    |




## ptuple

At each iteration, ptuple returns a tuple (array) combining the output of each
of the patterns in the list. When any of the patterns returns a nil, ptuple ends
that 'repeat' and restarts all of the streams.

| Argument | Description                | Default  |
| -------- | -----------                | -------  |
| list     |                            |          |
| repeats  | Number of repeats.         | 1        |




## pwalk

A one-dimensional random walk over a list of values that are embedded.

```coffee
pattern = pwalk(
  [1, 2, 3, 4, 5, 6, 7], 
  pseq([1], inf), 
  pseq([1, -1], inf) 
)
expected = [1, 2, 3, 4, 5, 6, 7, 6, 5, 4, 3, 2, 1, 2, 3, 4, 5, 6, 7, 6]
for item in expected
  assert_eq pattern.next(), item
```

| Argument          | Description                                                                       | Default |
| --------          | -----------                                                                       | ------- |
| list              | The items to be walked over.                                                      |         |
| step_pattern      | Returns integers that will be used to increment the index into list.              | 1       |
| direction_pattern | Used to determine the behavior at boundaries: 1 means forward, -1 means backward. | 1       |
| start_pos         | Where to start in the list.                                                       | 0       |




## pwrand

Returns one item from the list at random for each repeat, the probability for
each item is determined by a list of weights which should sum to 1.0.

| Argument | Description                | Default  |
| -------- | -----------                | -------  |
| list     |                            |          |
| weights  |                            |          |
| repeats  | Number of repeats.         | 1        |




## pxrand

Like prand, returns one item from the list at random for each repeat, but pxrand
never repeats the same element twice in a row.

| Argument | Description                | Default  |
| -------- | -----------                | -------  |
| list     |                            |          |
| repeats  | Number of repeats.         | 1        |
