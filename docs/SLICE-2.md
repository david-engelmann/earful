# Slice 2. The propose path

Built on Main's order of 2026-11-22, which brings propose
forward ahead of the input adapters. The adapters move to
slice 3 in this numbering. The slice implements the propose
design in docs/PROPOSE.md end to end.

## What landed

- The labelling file in the core, crates/earful-core/src/labels.rs.
  One JSONL format for human and model labellers. Label lines
  carry item id, theme id, verbatim span, labeler identity,
  and a status of draft or frozen. A frozen file opens with a
  header line naming the corpus hash, the taxonomy hash, the
  reviewer, and the per theme precision recorded at freeze
  time. Validation runs on every load. Item must be canonical,
  theme must exist, span must be verbatim, no item carries a
  theme twice.
- The counter consumes a frozen labelling file through an
  optional labels argument. A draft file is a hard error. A
  frozen file whose header hashes do not match the run is a
  hard error. Labels add their themes beside the lexicon hits,
  one count per item per theme as always, and a label's span
  is the citation when a label exists for the item and theme.
  The report format moves to earful-themes/0.2 and always
  carries the labelling file hash, null when no file was used.
  Verify takes the same labels argument and checks a labelled
  report against the same rules.
- The propose subcommand writes the worksheet, one header
  line with the taxonomy themes and one line per untagged
  canonical item in full, ready for a person or a chat tool
  to label outside the tool. No model client is embedded and
  no key is held.
- The audit-labels subcommand samples a draft file's labels
  with the same deterministic sampling as the audit
  subcommand, so the freeze measurement uses the standing
  audit contract.
- The freeze subcommand validates the draft, scores the
  recorded verdicts, refuses when any theme with at least
  five labels falls under four in five, and writes the frozen
  file only after the reviewer confirms at the prompt. No
  flag skips the confirmation.

## Tests

- Core unit tests for labelling parse and validation, frozen
  header roundtrip, draft refusal at the counter, frozen
  labels counting through the support rules, worksheet shape,
  and freeze passing on support verdicts and refusing on
  against verdicts for a measured theme.
- One CLI integration test walks the whole flow. Propose,
  label outside the tool, draft refused at count, audit the
  labels, freeze with the typed confirmation, count from the
  frozen file, verify the labelled report.

## Notes

- Theme creation stays a human taxonomy edit, as designed.
- The sentence splitting abbreviation defect from Phase 1 is
  unchanged and still recorded in the text module.
