# Changelog

All notable changes to earful, in the order they landed. The current version is 0.1.0 and nothing has been published to a package index yet.

## Unreleased

- A getting started example with eleven synthetic feedback items and a four theme taxonomy, run by the test suite on every build so its promised numbers cannot drift.
- A worked example of the labelling path with eight synthetic items and a filled draft labelling file, run end to end by the test suite.
- A worked example of the export path with a nine line synthetic export in the contract shape, reconciled by the test suite on every build.
- A guide to writing a taxonomy in the documents folder, covering matching, phrase choice, support, and the measurement gate.
- A reference for the file formats in the documents folder, the report, the labelling file, the worksheet, and the audit sample.
- A security note at the root stating the local only posture and the reporting route.
- Release metadata on both crates, and a README brought current with all nine commands.

## 0.1.0, built across November 2026

- Deterministic theme counting over three input shapes, a folder of text or Markdown files, a CSV file with a required text column, and a JSONL file with a required text field. Every theme carries verbatim cited examples, and a rerun on the same input reproduces the output byte for byte.
- A verify command that re-derives every count, checks every citation as verbatim, and compares both the JSON record and the Markdown rendering against a fresh computation.
- An audit command that draws a deterministic citation sample per theme for human scoring, under a per theme precision gate of four in five, with the shipped taxonomy 0.2 measured at precision 1.00 on every counted theme.
- A frozen labelling path. Propose drafts a labelling worksheet, audit-labels samples it, and freeze admits it only after the precision measurement and a typed human confirmation. The counter accepts frozen files only and prints the file hash in the report.
- An export contract check and an import command that gates a JSONL export on the contract, counts it, verifies its own output, and reconciles the export lines against the counted items before reporting success.
- Hardening. Citations no longer end mid abbreviation. Near duplicate detection runs at export scale without comparing every pair. The JSON parser refuses nesting deeper than 128 levels. Byte order marks in exports written by other tools are stripped at every entry point. A determinism stress test counts a wide corpus 25 times in the suite and asserts byte identical output on every run.
