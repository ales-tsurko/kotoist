---
title: Utility API
---

Utility API
===========




## midi_out

### Args

- the pattern to play
- the quantization in beats.

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


### Example

```coffee
midi_out {}, 4
```




## print_scales

Prints all available scales to the console.



## as_iter

Converts any value into an iterator.


### Args

- any value




## rrand

Returns a random number between two values.


### Args

- low value (a number)
- high value (a number)




## exprand

Returns a random number between two values exponentially.


### Args

- low value (a number)
- high value (a number)




## number.rand

Returns a random value between 0 and the number.




## number.rand2

Returns a random number between the negative number and the number.




## number.fold

Folds the number between two values.


### Args

- low value (a number)
- high value (a number)




## number.round

Round the number regarding to given quantization.


### Args

- quantization




## number.sign

Returns the sign of the number.




## number.wrap

Wraps the number between two values.


### Args

- low value (a number)
- high value (a number)




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


### Args

- iterator or number




## iterator.sub

Perform subtraction between the iterator and another iterator or a number.


### Args

- iterator or number




## iterator.mul

Perform multiplication between the iterator and another iterator or a number.


### Args

- iterator or number




## iterator.div

Perform division between the iterator and another iterator or a number.


### Args

- iterator or number




## iterator.mod

Perform the modulo operation between the iterator and another iterator or a
number.


### Args

- iterator or number
