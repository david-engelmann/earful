# Slice 10. Verify checks the Markdown as well

Count writes two artefacts, the canonical JSON and the
Markdown rendering. Verify recomputed and compared the
JSON byte for byte and never looked at the Markdown, so a
hand edited themes.md, the artefact people actually read
and quote, passed verification silently.

## What landed

- Verify takes the Markdown text when it is available and
  compares it byte for byte against a fresh rendering of
  the recomputed report, failing with a named message on
  any difference.
- The verify subcommand reads the themes.md sitting beside
  the themes file it was given and hands it over. Its
  absence is not a failure, because verify's named input
  is the themes file itself.
- The import path's self verification passes the rendering
  it just wrote, so the pair is checked there too.

## Tests

- A CLI test counts, edits a count in the Markdown only,
  and asserts verify exits nonzero naming the Markdown.
  The JSON is untouched in that test, so only the new
  check can catch it.
