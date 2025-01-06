Kotoist
=======

**Kotoist** is a VST plugin for 
[live coding](https://en.wikipedia.org/wiki/Live_coding) and 
[algorithmic composition](https://en.wikipedia.org/wiki/Algorithmic_composition).
It allows you to compose music on-the-fly using
[Koto](https://github.com/koto-lang/koto) programming language and a library
of patterns.

You can download it [here](https://github.com/ales-tsurko/kotoist/releases).

The source code is available at
[GitHub](https://github.com/ales-tsurko/kotoist).




## Usage

Write your script in the editor. Then evaluate the code using **"Run"** button
or `Cmd-Enter` on macOS or `Ctrl-Enter` or other systems. Also, you can evaluate
a part of the code by selecting it and pressing `Cmd-`\`Ctrl-Enter`.


### Writing Scripts

The main function, which connects your DAW with **koto** is `midiout`. The
arguments are:

- a pattern or a list of patterns to play
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
any of these keys. The patterns are just **Koto** iterators.

You can split your project into multiple snippets (tabs). When plugin is loaded
it evaluates all snippets from the rightmost to the leftmost.
