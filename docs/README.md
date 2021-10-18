Kotoist
=======

**Kotoist** is a VST plugin for 
[live coding](https://en.wikipedia.org/wiki/Live_coding) and 
[algorithmic composition](https://en.wikipedia.org/wiki/Algorithmic_composition).
It allows you to compose music on-the-fly using
[Koto](https://github.com/koto-lang/koto) programming language and the library
of patterns.

The source code is available at
[GitHub](https://github.com/ales-tsurko/kotoist).




## Usage

Write your script in the editor. Then evaluate the code using **"Build"** (the
hammer) button.

You can also evaluate only the part of the code by selecting the code you want
to evaluate and pressing the **Build** button.

![screenshot 1](/screenshots/editor.png "Kotoist editor")

To open snippets chooser view, press **"Snippets"** (the table) button.

![screenshot 2](/screenshots/snippets.png "Kotoist snippets")

You can have a code snippet per each MIDI note.

To choose another snippet, click on the snippet label. The currently chosen
snippet is indicated by yellow color.

To run the snippet, you can either play a corresponding note, click on a pad in
the **"Snippets"** view or evaluate the code using **"Build"** button.

The latest ran snippet is indicated by green color.


### Writing Scripts

The main function, which connects your DAW with **koto** is `midi_out`. The
arguments are:

- the pattern to play
- the quantization in beats.

The **pattern** is a map with optional values:

- `dur` - note duration
- `length` - note length
- `degree` - step in the scale
- `scale` - to view available scales execute `print_scales` function
- `root` - root note
- `transpose` - simple transpose
- `mtranspose` - transpose relating to the scale
- `octave` - octave number (from 0)
- `channel` - MIDI channel number
- `amp` - amplitude (from 0.0 to 1.0)

You can apply any pattern or combination of them, or write your own patterns to
any of these keys. The patterns are just **koto** iterators.
