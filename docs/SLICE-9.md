# Slice 9. Byte order marks are stripped at every entry point

Exports written by Excel and other Windows tools commonly
open with a UTF-8 byte order mark. Earful read it as
content. On a CSV it corrupted the header's first column
name, so a file whose first column was text was refused
for having no text column. On a JSON input it failed the
parse. On a text file it became part of the first item's
first word.

## What landed

- The JSON parser strips one leading byte order mark
  before parsing. Every JSON input earful reads passes
  through it, taxonomies, labelling files, JSONL exports,
  and the reports verify reads back.
- The corpus decoder strips it the same way, covering
  folder text files and CSV headers. It is not a decode
  warning. It was never content.

## Tests

- The decoder test runs the mark through a CSV header and
  a JSONL first line and asserts both read clean.
- The parser test asserts a marked object parses.
- The golden corpus carries no marks, so its outputs are
  unchanged.
