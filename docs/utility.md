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




## on_load

Executes a callback function when plugin initialized.

```coffee
on_load ||
    print "plugin loaded"
```
| Argument     | Description                                   | Default |
| --------     | -----------                                   | ------- |
| callback      | A function, which will be called, when plugin initialized.    |         |




## on_midiin

Executes a callback function when host sends MIDI note-on/off events.

```coffee
on_midiin |note, velocity, channel|
    if velocity == 0
        print "note off: ", note
    else
        print "note on: ", note
```
| Argument     | Description                                                                                                                    | Default |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------ | ------- |
| callback      | A function, receiving `note_number` (integer `[0, 127]`), `velocity` (float `[0.0, 1.0]`) and `channel` (integer `[0, 127]`). |         |





## on_midiincc

Executes a callback function when host sends MIDI control change message.

```coffee
on_midiincc |cc, value, channel|
    print "control: ", cc, ", value: ", value
```
| Argument     | Description                                                                                                                       | Default |
| ------------ | --------------------------------------------------------------------------------------------------------------------------------- | ------- |
| callback      | A function, receiving `control_number` (integer `[0, 127]`), `value` (float `[0.0, 1.0]`) and `channel` (integer `[0, 127]`).    |         |




## on_play

Executes a callback function when playback is started.

```coffee
on_play ||
    print "playing"
```
| Argument | Description                                                     | Default |
| -------- | --------------------------------------------------------------- | ------- |
| callback | A function, which will be called, when playback is started.     |         |




## on_pause

Executes a callback function when playback is paused.

```coffee
on_pause ||
    print "paused"
```
| Argument | Description                                                     | Default |
| -------- | --------------------------------------------------------------- | ------- |
| callback | A function, which will be called, when playback is paused.      |         |




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
