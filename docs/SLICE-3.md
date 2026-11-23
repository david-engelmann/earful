# Slice 3. The CSV and JSONL adapters

The remaining two input shapes from the amendment, so all
three version 1 shapes are now live.

## What landed

- The core gains load_corpus_file, which dispatches on the
  file extension and refuses anything else by name. Both
  parsers live in crates/earful-core/src/corpus.rs on the
  standard library only.
- The CSV shape takes a header row with a required text
  column and optional id, source, date, and rating columns.
  Only id is used in version 1, as the item id. Rows without
  an id take their data row number. Quoted fields may carry
  commas, doubled quotes, and newlines.
- The JSONL shape takes one object per line with a required
  text field and the same optional fields. Lines without an
  id take their line number.
- Both shapes normalise line endings and hash items exactly
  as the folder shape does, so the corpus hash means the same
  thing across shapes. Duplicate item ids in one file are a
  hard error, because citations must name items unambiguously.
- Every command that reads a corpus now takes either the
  folder flag or an input flag naming a CSV or JSONL file.

## Tests

- Core unit tests for CSV quoting and id fallback, the
  missing text column error, JSONL id fallback, the missing
  text field error, and duplicate id refusal.
- One CLI integration test counts a JSONL file and checks
  the totals, the counted theme, and the untagged id.
