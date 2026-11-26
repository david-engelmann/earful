# Slice 14. A worked example that cannot rot

A first user had no way to run earful in a minute and see
what it produces. The README described the tool, and
nothing in the repository let a reader try it.

## What landed

- examples/getting-started carries eleven short synthetic
  feedback items and a four theme taxonomy, with a short
  README giving the count and verify commands and the
  exact numbers to expect. Three themes at three items
  each, battery below minimum support and listed rather
  than dropped, one item untagged.
- A test in the command line crate runs the example on
  every build and asserts those numbers, so the example's
  README cannot drift from what the tool produces.
