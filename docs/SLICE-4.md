# Slice 4. The dusk.fm export contract check

The earful side of the dusk.fm integration, against the
banked specification in docs/DUSKFM-EXPORT.md. The export
itself is produced on the dusk.fm side. What this slice
builds is the gate the first real export passes through
before any count from it is trusted.

## What landed

- A core export module that checks JSONL export text
  against the contract. Every line must be one object with
  a required id, a required source drawn from the starting
  list of support, review, in_app, and email, and a required
  text. Ids must be unique and sorted ascending, because
  sorted order is what makes citation order stable across
  exports. A date, when present, must be ISO 8601 UTC,
  checked strictly. A rating, when present, must be a number.
- A check-export subcommand that prints the export line
  count, the per source counts, and the decode warning
  count, then passes or fails the file with every contract
  break named. These are the export side numbers the first
  run reconciles against the themes report, per the spec.
- The corpus decode helper is public now, so the checker
  counts decode warnings the same way ingest does.

## Tests

- Core unit tests run the specification's two synthetic
  example lines through the checker clean, name four
  contract breaks in a bad file, and pin the ISO timestamp
  shape.
- One CLI integration test accepts the spec examples and
  refuses a file with an unlisted source.

## Still owed for the integration

- The export itself, produced on the dusk.fm side.
- The first run reconciliation, when the export exists.
