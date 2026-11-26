# Slice 22. A worked example of the labelling path

The getting started example counts what a lexicon can
see. The labelling path, the feature that covers what the
words miss, had no worked example at all.

## What landed

- examples/labelling holds eight synthetic items, three
  reporting crashes in the taxonomy's words and five
  reporting billing problems in none of them, a two theme
  taxonomy, and the filled draft labelling file, labelled
  outside the tool as a real labeller would leave it.
- Its README walks the whole path with exact commands,
  propose, the draft, audit-labels, the verdicts, freeze
  with its typed confirmation, and the labelled count,
  promising billing at five from the labels, crashes at
  three from the lexicon, and nothing untagged.
- A command line test runs the flow on every build
  against the shipped files and asserts those numbers
  and the labelling hash in the report, so the example
  cannot drift from the tool.
- The main README and the changelog point at it, and the
  documents index gains its slice 22 line.
