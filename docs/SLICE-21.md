# Slice 21. Edge input tests for the counter

Nothing in the suite counted an empty corpus or a corpus
of blank items, the two shapes a mistyped folder path or
a blank export actually produce.

## What landed

- A core test counts an empty corpus and asserts a well
  formed report, zero analysed, no themes, and a Markdown
  rendering that states the zero.
- A second case counts a whitespace only item beside a
  plain untagged one and asserts both land in the untagged
  list with no theme counted.
- The documents index gains its slice 21 line here, and
  its slice 20 line in slice 20's own commit.
