//! The propose subcommand's core. The worksheet writer and
//! the freeze transform, per the propose design.
//!
//! Propose embeds no model client and holds no key. It writes
//! a worksheet of the items the lexicon left untagged. A person
//! or the user's own chat tool fills in labels outside the
//! tool. Freeze then runs the audit measurement over the draft
//! labels and, when every measured theme clears the gate,
//! writes the frozen labelling file a count can consume. The
//! human confirmation itself lives in the command line front
//! end. Nothing here can freeze anything on its own.

use crate::audit;
use crate::corpus::{Corpus, corpus_hash};
use crate::json::{self, Value};
use crate::labels::{self, FreezeHeader, Label, LabellingFile, Status};
use crate::report::{Citation, Report};
use crate::taxonomy::Taxonomy;
use std::collections::BTreeMap;

/// Renders the worksheet for a count report. The first line is
/// a header naming the taxonomy themes with their labels and
/// kinds. Each following line carries one untagged canonical
/// item in full, with the labelling fields blank for the
/// labeller to complete. Order is the report's untagged order,
/// which is corpus order, so the worksheet is reproducible.
pub fn worksheet(report: &Report, corpus: &Corpus, taxonomy: &Taxonomy) -> String {
    let texts: std::collections::HashMap<&str, &str> = corpus
        .items
        .iter()
        .map(|i| (i.id.as_str(), i.text.as_str()))
        .collect();
    let mut out = String::new();
    let header = json::obj(vec![
        ("kind", json::s("earful-worksheet")),
        ("taxonomy_version", json::s(&taxonomy.version)),
        (
            "themes",
            Value::Arr(
                taxonomy
                    .themes
                    .iter()
                    .map(|t| {
                        json::obj(vec![
                            ("id", json::s(&t.id)),
                            ("label", json::s(&t.label)),
                            ("kind", json::s(&t.kind)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ]);
    out.push_str(&json::write_compact(&header));
    out.push('\n');
    for item_id in &report.untagged_item_ids {
        let text = texts.get(item_id.as_str()).copied().unwrap_or("");
        let line = json::obj(vec![
            ("item_id", json::s(item_id)),
            ("text", json::s(text)),
            ("theme_id", json::s("")),
            ("span", json::s("")),
            ("labeler", json::s("")),
            ("status", json::s("draft")),
        ]);
        out.push_str(&json::write_compact(&line));
        out.push('\n');
    }
    out
}

/// Groups a labelling file's labels into per theme citation
/// pools, in taxonomy order, for audit sampling.
pub fn label_pools(file: &LabellingFile, taxonomy: &Taxonomy) -> Vec<(String, Vec<Citation>)> {
    let mut by_theme: BTreeMap<String, Vec<Citation>> = BTreeMap::new();
    for label in &file.labels {
        by_theme
            .entry(label.theme_id.clone())
            .or_default()
            .push(Citation {
                item_id: label.item_id.clone(),
                span: label.span.clone(),
            });
    }
    taxonomy
        .themes
        .iter()
        .filter_map(|t| by_theme.remove(&t.id).map(|pool| (t.id.clone(), pool)))
        .collect()
}

/// Freezes a draft labelling file. Validates the draft,
/// samples its labels, scores the sample against the verdicts
/// the human recorded, and refuses when any measured theme
/// falls under the gate. On success returns the frozen file
/// text with its header. The labels keep the draft's order.
pub fn freeze(
    draft: &LabellingFile,
    corpus: &Corpus,
    taxonomy: &Taxonomy,
    verdicts_text: &str,
    reviewer: &str,
) -> Result<String, String> {
    if draft.status != Status::Draft {
        return Err("only a draft labelling file can be frozen".to_string());
    }
    if reviewer.trim().is_empty() {
        return Err("freeze needs a reviewer name".to_string());
    }
    let failures = labels::validate(&draft.labels, corpus, taxonomy);
    if !failures.is_empty() {
        return Err(format!("draft fails validation: {}", failures.join("; ")));
    }
    let pools = label_pools(draft, taxonomy);
    let entries = audit::sample_pools(&pools);
    let sample_text = audit::render_sample_jsonl(&entries);
    let scores = audit::score(&sample_text, verdicts_text)?;
    let mut precision: Vec<(String, Option<f64>)> = Vec::new();
    for s in &scores {
        if let Some(false) = s.passed {
            return Err(format!(
                "theme {} fails the precision gate at {:.2} on {} sampled labels",
                s.theme_id,
                s.precision.unwrap_or(0.0),
                s.sampled
            ));
        }
        precision.push((s.theme_id.clone(), s.precision));
    }
    let header = FreezeHeader {
        corpus_sha256: corpus_hash(&corpus.items),
        taxonomy_sha256: taxonomy.sha256.clone(),
        reviewer: reviewer.to_string(),
        precision,
    };
    let sorted_labels: Vec<Label> = draft.labels.clone();
    Ok(labels::render(
        Status::Frozen,
        Some(&header),
        &sorted_labels,
    ))
}

#[cfg(test)]
mod tests {
    use super::{freeze, worksheet};
    use crate::corpus::{Corpus, Item};
    use crate::labels::{Status, parse_labelling};
    use crate::report::count;
    use crate::sha256::sha256_hex;
    use crate::taxonomy::load_taxonomy;

    fn corpus() -> Corpus {
        let mk = |id: &str, text: &str| Item {
            id: id.to_string(),
            text: text.to_string(),
            sha256: sha256_hex(text.as_bytes()),
        };
        Corpus {
            decode_warnings: 0,
            items: vec![
                mk("a", "It crashes every day."),
                mk("b", "The widget never syncs my data."),
                mk("c", "Sync is broken for me."),
                mk("d", "My data will not sync at all."),
                mk("e", "Nothing syncs since Tuesday."),
                mk("f", "Sync never works here."),
            ],
        }
    }

    fn taxonomy() -> crate::taxonomy::Taxonomy {
        load_taxonomy(
            br#"{"version": "t1", "themes": [
                {"id": "crashes", "label": "Crashes", "kind": "operational_issue",
                 "phrases": ["crash", "crashes"]},
                {"id": "sync", "label": "Sync", "kind": "operational_issue",
                 "phrases": ["sync", "syncs"]}
            ]}"#,
        )
        .unwrap()
    }

    fn draft_text() -> String {
        // Labels for the four sync items the lexicon phrases
        // partly miss, plus a crash label, all verbatim.
        let lines = [
            ("b", "sync", "The widget never syncs my data."),
            ("c", "sync", "Sync is broken for me."),
            ("d", "sync", "My data will not sync at all."),
            ("e", "sync", "Nothing syncs since Tuesday."),
            ("f", "sync", "Sync never works here."),
            ("a", "crashes", "It crashes every day."),
        ];
        let mut out = String::new();
        for (item, theme, span) in lines {
            out.push_str(&format!(
                "{{\"item_id\":\"{item}\",\"theme_id\":\"{theme}\",\"span\":\"{span}\",\"labeler\":\"test\",\"status\":\"draft\"}}\n"
            ));
        }
        out
    }

    #[test]
    fn worksheet_lists_only_untagged() {
        let report = count(&corpus(), &taxonomy());
        let sheet = worksheet(&report, &corpus(), &taxonomy());
        // One header line plus one line per untagged item.
        assert_eq!(sheet.lines().count(), 1 + report.untagged_item_ids.len());
    }

    #[test]
    fn freeze_passes_and_refuses() {
        let draft = parse_labelling(&draft_text()).unwrap();
        // Verdicts support everything.
        let pools = super::label_pools(&draft, &taxonomy());
        let entries = crate::audit::sample_pools(&pools);
        let sample = crate::audit::render_sample_jsonl(&entries);
        let verdicts = sample.replace("\"verdict\":\"\"", "\"verdict\":\"support\"");
        let frozen = freeze(&draft, &corpus(), &taxonomy(), &verdicts, "rev").unwrap();
        let parsed = parse_labelling(&frozen).unwrap();
        assert_eq!(parsed.status, Status::Frozen);
        assert_eq!(parsed.header.unwrap().reviewer, "rev");

        // Verdicts against every label fail the freeze,
        // because sync carries five labels and is measured.
        let bad = sample.replace("\"verdict\":\"\"", "\"verdict\":\"against\"");
        let err = freeze(&draft, &corpus(), &taxonomy(), &bad, "rev").unwrap_err();
        assert!(err.contains("fails the precision gate"), "got: {err}");
    }
}
