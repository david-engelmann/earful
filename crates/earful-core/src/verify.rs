//! The verify subcommand's checks.
//!
//! Verify trusts nothing in the themes file it is handed. It
//! parses the file, checks every citation span occurs verbatim
//! in the named item, checks every listed item genuinely carries
//! a phrase of its theme, checks counts against the listed ids,
//! and then recomputes the whole report from the folder and
//! taxonomy and compares the canonical bytes. Any failure is
//! reported and the caller exits nonzero.

use crate::corpus::Corpus;
use crate::json;
use crate::report::{count, render_json, Report};
use crate::taxonomy::Taxonomy;
use crate::text;

/// Runs every check. Returns the list of failures, empty on pass.
pub fn verify(corpus: &Corpus, taxonomy: &Taxonomy, themes_text: &str) -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();
    let parsed = match json::parse(themes_text) {
        Ok(v) => v,
        Err(e) => return vec![format!("themes file does not parse: {e}")],
    };

    let texts: std::collections::HashMap<&str, &str> = corpus
        .items
        .iter()
        .map(|i| (i.id.as_str(), i.text.as_str()))
        .collect();
    let phrases: std::collections::HashMap<&str, &[String]> = taxonomy
        .themes
        .iter()
        .map(|t| (t.id.as_str(), t.phrases.as_slice()))
        .collect();

    if let Some(themes) = parsed.get("themes").and_then(|v| v.as_arr()) {
        for row in themes {
            let id = row.get("id").and_then(|v| v.as_str()).unwrap_or("?");
            let ids: Vec<&str> = row
                .get("item_ids")
                .and_then(|v| v.as_arr())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                .unwrap_or_default();
            let count_val = row.get("count").and_then(|v| v.as_f64()).unwrap_or(-1.0);
            if count_val as usize != ids.len() {
                failures.push(format!("{id}: count {count_val} does not match listed ids {}", ids.len()));
            }
            let mut seen_ids = std::collections::HashSet::new();
            for iid in &ids {
                if !seen_ids.insert(iid) {
                    failures.push(format!("{id}: item {iid} listed twice"));
                }
                match texts.get(iid) {
                    Some(text) => {
                        let lower = text::lower(text);
                        let hit = phrases
                            .get(id)
                            .map(|ps| text::theme_hits(&lower, ps))
                            .unwrap_or(false);
                        if !hit {
                            failures.push(format!("{id}: item {iid} carries no phrase of the theme"));
                        }
                    }
                    None => failures.push(format!("{id}: item {iid} not in corpus")),
                }
            }
            if let Some(citations) = row.get("citations").and_then(|v| v.as_arr()) {
                for cit in citations {
                    let iid = cit.get("item_id").and_then(|v| v.as_str()).unwrap_or("?");
                    let span = cit.get("span").and_then(|v| v.as_str()).unwrap_or("");
                    match texts.get(iid) {
                        Some(text) => {
                            if !text.contains(span) {
                                failures
                                    .push(format!("{id}: citation for {iid} is not verbatim"));
                            }
                        }
                        None => failures.push(format!("{id}: citation item {iid} not in corpus")),
                    }
                }
            }
        }
    } else {
        failures.push("themes file has no themes array".to_string());
    }

    // Full recomputation and canonical byte comparison.
    let report: Report = count(corpus, taxonomy);
    let expected = render_json(&report);
    if expected != themes_text {
        failures.push("themes file differs from a fresh recomputation".to_string());
    }
    failures
}
