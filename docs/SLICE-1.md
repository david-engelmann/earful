# First build slice, scoped

Slice 1 builds the deterministic counting core end to end over one input shape, with the byte for byte claim under test from the first commit. Scoped by the earful seat on 2026-11-20 after the Phase 1 gate confirmation.

## In scope

Shape one ingest only. A folder of .txt or .md files becomes the normalised item stream with stable ids in sorted file name order, UTF-8 decoding with a recorded warning count for replaced bytes, LF line ending normalisation, and SHA-256 hashes per item and for the corpus.

Taxonomy loading from a versioned JSON file, with the taxonomy hash computed and carried into the output.

Whole token phrase matching, multi label, one count per item per theme. Exact duplicate exclusion on normalised text with duplicates listed against their canonical item. Near duplicate flagging at token Jaccard 0.90 or above, flagged and still counted.

Canonical rendering of themes.json and themes.md exactly as the design record specifies, including minimum support suppression with suppressed themes listed rather than dropped.

A verify subcommand that re-derives every count from the folder and the taxonomy and fails with a nonzero exit when any citation span is not found verbatim in its source item or any count disagrees.

Taxonomy version 0.2, carried from the Phase 1 test taxonomy with two gate driven changes. The updates theme is redesigned to regression only phrasing, because its Phase 1 form failed the precision gate at about two in five. The ease theme is renamed to interface and design, because its genuine citations were design remarks rather than ease remarks.

A golden test that runs the Phase 1 corpus, all 600 items, through the Rust core and compares both output files byte for byte against frozen expected files. A rerun test that runs the counter twice and compares outputs byte for byte.

## Acceptance

Cargo test, cargo clippy with warnings denied, and cargo fmt check all pass on the pinned toolchain. The golden test and the rerun test pass. The audit sample for taxonomy 0.2 prints, and the per theme precision scoring of that sample is recorded in the slice report, with any theme under four in five pulled from the shipped taxonomy before the slice closes. The three review checkpoint rule runs before the slice is called done.

## Out of scope for slice 1

CSV and JSONL adapters land in slice 2. The frozen labelling path lands in slice 3. The propose subcommand lands in slice 4, gated on its precision measurement. The dusk.fm export and first real run wait on the export existing and are not part of any slice above.
