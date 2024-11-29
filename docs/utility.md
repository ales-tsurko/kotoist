---
title: Utility API
---

Utility API
===========




## midiout

Plays the pattern.

```coffee
midiout {}, 4
```
| Argument     | Description                                   | Default |
| --------     | -----------                                   | ------- |
| pattern      | A pattern or an array of patterns to play.    |         |
| quantization | The quantization in beats.                    |         |

The **pattern** is a map with optional values:

- `dur` - note duration
- `length` - note length
- `degree` - step in the scale
- `scale` - case-insensitive, to view available scales execute `print_scales`
  function
- `root` - root note
- `transpose` - simple transpose
- `mtranspose` - transpose relating to the scale
- `octave` - octave number (from 0)
- `channel` - MIDI channel number
- `amp` - amplitude (from 0.0 to 1.0)




## print_scales

Prints all available scales to the console.




## as_iter

Converts any value into an iterator.


| Argument     | Description                | Default |
| --------     | -----------                | ------- |
| value        | Any value.                 |         |




## rrand

Returns a random number between two values.


| Argument     | Description                | Default |
| --------     | -----------                | ------- |
| low          | Low value (a number).      |         |
| high         | High value (a number).     |         |




## exprand

Returns a random number between two values exponentially.


| Argument     | Description                | Default |
| --------     | -----------                | ------- |
| low          | Low value (a number).      |         |
| high         | High value (a number).     |         |




## number.rand

Returns a random value between 0 and the number.




## number.rand2

Returns a random number between the negative number and the number.




## number.fold

Folds the number between two values.

| Argument     | Description                | Default |
| --------     | -----------                | ------- |
| low          | Low value (a number).      |         |
| high         | High value (a number).     |         |




## number.round

Round the number regarding to given quantization.

| Argument     | Description                | Default |
| --------     | -----------                | ------- |
| quantization | A number.                  |         |




## number.sign

Returns the sign of the number.




## number.wrap

Wraps the number between two values.

| Argument     | Description                | Default |
| --------     | -----------                | ------- |
| low          | Low value (a number).      |         |
| high         | High value (a number).     |         |




## list.scramble

Scrambles the list.




## list.windex

Interprets the list's values as normalized probabilities and randomly returns an
index with the given probabilities.




## list.normalize_sum

Recalculates the values in the list and returns the list with the values sum
equal to 1.0.




## iterator.add

Perform addition between the iterator and another iterator or a number.

| Argument     | Description                | Default |
| --------     | -----------                | ------- |
| value        | Iterator or number.        |         |




## iterator.sub

Perform subtraction between the iterator and another iterator or a number.

| Argument     | Description                | Default |
| --------     | -----------                | ------- |
| value        | Iterator or number.        |         |




## iterator.mul

Perform multiplication between the iterator and another iterator or a number.

| Argument     | Description                | Default |
| --------     | -----------                | ------- |
| value        | Iterator or number.        |         |




## iterator.div

Perform division between the iterator and another iterator or a number.

| Argument     | Description                | Default |
| --------     | -----------                | ------- |
| value        | Iterator or number.        |         |




## iterator.mod

Perform the modulo operation between the iterator and another iterator or a
number.

| Argument     | Description                | Default |
| --------     | -----------                | ------- |
| value        | Iterator or number.        |         |
