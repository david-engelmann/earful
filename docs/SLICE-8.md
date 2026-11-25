# Slice 8. A nesting limit on the JSON parser

Earful reads files other tools produced. Taxonomies,
labelling files, JSONL exports, and the reports verify
reads back all come through one hand rolled recursive
parser, and an unbounded recursive descent turns a hostile
or broken file into a stack overflow instead of an error.

## What landed

- The parser in crates/earful-core/src/json.rs carries a
  depth counter and refuses nesting deeper than 128 levels
  with a plain error naming the limit.
- The limit sits far above anything the product reads.
  Taxonomies, labelling files, and exports nest a handful
  of levels. It sits far below any stack risk.

## Tests

- A hostile input of one hundred thousand opening brackets
  returns the depth error instead of crashing.
- An honestly nested input one hundred levels deep still
  parses.
