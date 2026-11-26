# Decisions record for earful

Locked at the Phase 1 design gate, confirmed by Main Muse on 2026-11-20. This record is banked from docs/DESIGN.md and docs/AMENDMENT.md. Changes to a locked decision are escalated to Main Muse rather than made inside this repository.

## Identity and licensing

The product name is earful. The repository is david-engelmann/earful, public. The license is MIT OR Apache-2.0 at the user's option, with both texts at the repository root.

Rename recorded on 2026-11-20. The product was named refrain from its Phase 1 design gate until David settled the final name earful later the same day, and the Phase 1 record was produced and banked under the earlier name. The GitHub repository was created as david-engelmann/refrain on 2026-11-20 and renamed to david-engelmann/earful on 2026-11-22, with the repository id unchanged.

## Stack

Rust, edition 2024, toolchain pinned in rust-toolchain.toml. Cargo.lock is committed. The workspace holds two crates. The crate earful-core is standard library only and holds all counting behaviour. The crate earful-cli holds argument handling and input adapters and may use pinned external crates.

## The determinism boundary

The counting and citation core is byte for byte reproducible. It makes no network calls, reads no clock, uses no randomness, and consumes no model output at run time. Model assisted labelling happens once, outside the core, and enters runs only as a frozen labelling file whose hash is printed in the report. The core never calls a model and never consumes an unfrozen label file.

## Product framing

Earful does theme accounting, not theme discovery. That sentence stands on the face of the product, in the README and in the crate descriptions.

## Precision gate

The shipped default taxonomy is under a per theme precision contract of at least four in five sampled citations genuinely supporting their theme. An audit subcommand prints a deterministic citation sample per theme for human scoring. A theme that fails the gate ships redesigned or not at all.

## The propose subcommand

Propose is a version 1 component, ruled in at the gate with the precision measurement as its binding condition. It drafts candidate labels for human freezing only, in the same labelling file format a human would write. If its measured precision fails, propose comes out and the README says known themes only.

## Version 1 input fence

Version 1 reads a folder of .txt or .md files with one item per file, a CSV file with a required text column and optional id, source, date, and rating columns, and a JSONL file with a required text field and the same optional fields. Mail archives, PDF and Office documents, direct connectors, and live database connections are refused in version 1.

## Access

This repository is not added to the existing GitHub App installation. It carries its own per repo SSH deploy key, generated on the build machine, with git transport over SSH as the push route.

Access state recorded on 2026-11-25. A repository ruleset that arrived with the fleet Trunk enrollment refuses direct pushes to the main branch, so work lands on branches and reaches main through pull requests opened from Main Muse's side, because the fine grained token covering this repository carries no pull requests grant. When the build machine's intermittent write fault refuses local git object writes, slices are staged through the Git Data API under the same credential, with every staged blob hash checked against the worktree before the branch is reported.

## Evidence discipline

The public review corpus result from Phase 1 stands as method evidence only. It is never cited as evidence about dusk.fm feedback. The first dusk.fm run requires an export in the JSONL shape, which is a build phase integration task.
