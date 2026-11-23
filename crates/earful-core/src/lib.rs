//! earful-core holds the deterministic counting core.
//!
//! Ingest, normalisation, whole token phrase matching, counting,
//! deduplication, canonical rendering, verification, and the
//! audit sampling and scoring live here. This crate uses only
//! the Rust standard library. It makes no network calls, reads
//! no clock, and uses no randomness, so a run over the same
//! folder, taxonomy, and optional frozen labelling file produces
//! byte identical output on any machine.
//!
//! The design record in docs/DESIGN.md states the determinism
//! boundary in full. Model assisted labelling happens outside
//! this crate, once, and enters runs only as a frozen data file.

pub mod audit;
pub mod corpus;
pub mod export;
pub mod json;
pub mod labels;
pub mod propose;
pub mod report;
pub mod sha256;
pub mod taxonomy;
pub mod text;
pub mod verify;

pub use audit::{ThemeScore, render_sample_jsonl, render_worksheet, sample, score};
pub use corpus::{Corpus, Item, load_corpus};
pub use labels::{FreezeHeader, Label, LabellingFile, Status};
pub use report::{FORMAT, Report, count, count_with_labels, render_json, render_markdown};
pub use taxonomy::{Taxonomy, load_taxonomy};
pub use verify::verify;
