# The taxonomy 0.2 rerun proof

Run 2026-11-24 with the Rust binary built from head e4b18fe0,
in the first local write window since the build landed. The
corpus is the Phase 1 corpus of 600 public app review files,
598 canonical items after two exact duplicates. The taxonomy
is taxonomy/v0.2.json, SHA-256
764c0004b151f1ff01131dad77ef17eb4ba76363b2081955ec64332000ac1d42.

## Byte identity

Two consecutive count runs over the same folder and taxonomy
produced byte identical outputs, compared with cmp and with
SHA-256.

- themes.json, SHA-256
  496eb007b357ce15f8512aff6ca4efdd35bf0c835813e57450196fa992e673b0
- themes.md, SHA-256
  4ac6ccb933439c87762a34d89d2d9a96440eeb39313d42efedf19bc6e11defca

The verify subcommand on the first run's themes.json reports
ALL CHECKS PASS.

## The numbers against the audit record

Every counted and suppressed number in docs/AUDIT-RECORD-0.2.md
reproduces exactly.

- praise_and_recognition 180
- crashes_and_stability 122
- missing_features 34
- interface_and_design 15
- battery_and_resources 5
- Suppressed, performance_and_speed 1, sync_and_data 1,
  regressions_after_update 2
- Items in folder 600, items analysed 598, exact duplicates 2,
  decode warnings 0

One number does not reproduce, and it is the record that is
wrong. The record says 284 items untagged. The Rust build
says 283. To settle which, the Phase 1 prototype was rerun
today on its own corpus copy, which a recursive diff shows
identical to the corpus the Rust build read. The prototype
also produces 283 untagged, and its untagged item set is
identical to the Rust build's, item for item, with no item
in either set alone. The record's 284 was written during the
taxonomy redesign rounds, before the final battery narrowing
settled the shipped taxonomy bytes, and was never recomputed
against the final file. The audit record carries a dated
correction. The percentage moves from 47.5 to 47.3.

## The audit sample cross check

The audit record owed a cross check that the Rust audit
subcommand reproduces the scored sample's shape. It samples
45 citations here, ten each for praise_and_recognition,
crashes_and_stability, missing_features, and
interface_and_design, and five for battery_and_resources,
which is its whole pool. That matches the record's rounds,
ten per theme with battery measured on its full pool of
five. The prototype sample file itself no longer exists on
disk, so a line by line sample diff is not possible. The
sampling algorithm is the deterministic one both
implementations share, and the counts above pin the pools
the sample is drawn from.

## Local gates in the same window

- cargo test across the workspace, 32 core unit tests and 5
  command line integration tests, all passing. This is the
  first full local test pass for this repository.
- cargo clippy across the workspace and all targets with
  warnings denied, clean.
- cargo fmt check, clean.
