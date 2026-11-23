//! The labelling file. One JSONL format serves the human
//! labeller, the model labeller, the draft state, and the
//! frozen state, exactly as the propose design specifies.
//!
//! Each label line carries an item id, a theme id, a verbatim
//! span from the item, the labeler identity, and a status of
//! draft or frozen. A frozen file opens with a header line
//! naming the corpus hash it was frozen against, the taxonomy
//! hash, the reviewer who froze it, and the audit precision
//! recorded per theme at freeze time. Validation is mechanical
//! and runs on every load. The counting core accepts only a
//! frozen file whose header hashes match the run, and a draft
//! file handed to the counter is a hard error.

use crate::corpus::Corpus;
use crate::json::{self, Value};
use crate::report::canonical_items;
use crate::taxonomy::Taxonomy;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Draft,
    Frozen,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub item_id: String,
    pub theme_id: String,
    pub span: String,
    pub labeler: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeHeader {
    pub corpus_sha256: String,
    pub taxonomy_sha256: String,
    pub reviewer: String,
    /// Per theme precision at freeze time. None is unmeasured.
    pub precision: Vec<(String, Option<f64>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabellingFile {
    pub status: Status,
    pub header: Option<FreezeHeader>,
    pub labels: Vec<Label>,
}

const HEADER_KIND: &str = "earful-labelling-header";

/// Parses a labelling file. Syntactic checks only. Call
/// validate for the checks against a corpus and taxonomy.
pub fn parse_labelling(text: &str) -> Result<LabellingFile, String> {
    let mut header: Option<FreezeHeader> = None;
    let mut labels: Vec<Label> = Vec::new();
    let mut status: Option<Status> = None;
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let value = json::parse(line)
            .map_err(|e| format!("labelling line {} does not parse: {e}", i + 1))?;
        if value.get("kind").and_then(|v| v.as_str()) == Some(HEADER_KIND) {
            if i != 0 || header.is_some() {
                return Err("labelling header must be the first line".to_string());
            }
            header = Some(parse_header(&value)?);
            continue;
        }
        let line_status = match value.get("status").and_then(|v| v.as_str()) {
            Some("draft") => Status::Draft,
            Some("frozen") => Status::Frozen,
            _ => return Err(format!("labelling line {} has no valid status", i + 1)),
        };
        match status {
            None => status = Some(line_status),
            Some(s) if s != line_status => {
                return Err("labelling file mixes draft and frozen lines".to_string());
            }
            _ => {}
        }
        let get = |key: &str| -> Result<String, String> {
            value
                .get(key)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| format!("labelling line {} has no {key}", i + 1))
        };
        let labeler = get("labeler")?;
        if labeler.trim().is_empty() {
            return Err(format!("labelling line {} has an empty labeler", i + 1));
        }
        labels.push(Label {
            item_id: get("item_id")?,
            theme_id: get("theme_id")?,
            span: get("span")?,
            labeler,
        });
    }
    let status = status.ok_or("labelling file has no label lines")?;
    match (status, &header) {
        (Status::Frozen, None) => {
            return Err("frozen labelling file has no header line".to_string());
        }
        (Status::Draft, Some(_)) => {
            return Err("draft labelling file must not carry a freeze header".to_string());
        }
        _ => {}
    }
    Ok(LabellingFile {
        status,
        header,
        labels,
    })
}

fn parse_header(value: &Value) -> Result<FreezeHeader, String> {
    let get = |key: &str| -> Result<String, String> {
        value
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| format!("labelling header has no {key}"))
    };
    let mut precision: Vec<(String, Option<f64>)> = Vec::new();
    if let Some(obj) = value.get("precision") {
        if let Value::Obj(pairs) = obj {
            for (theme, p) in pairs {
                let val = match p {
                    Value::Null => None,
                    other => Some(
                        other
                            .as_f64()
                            .ok_or("header precision is not a number or null")?,
                    ),
                };
                precision.push((theme.clone(), val));
            }
        } else {
            return Err("header precision is not an object".to_string());
        }
    }
    precision.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(FreezeHeader {
        corpus_sha256: get("corpus_sha256")?,
        taxonomy_sha256: get("taxonomy_sha256")?,
        reviewer: get("reviewer")?,
        precision,
    })
}

/// Validates labels against a corpus and taxonomy. Every item
/// must be a canonical corpus item, every theme must exist in
/// the taxonomy, every span must occur verbatim in its item,
/// and no item may carry the same theme twice. Returns the
/// list of failures, empty on pass.
pub fn validate(labels: &[Label], corpus: &Corpus, taxonomy: &Taxonomy) -> Vec<String> {
    let (canonical, _dups) = canonical_items(corpus);
    let texts: HashMap<&str, &str> = canonical
        .iter()
        .map(|item| (item.id.as_str(), item.text.as_str()))
        .collect();
    let theme_ids: HashSet<&str> = taxonomy.themes.iter().map(|t| t.id.as_str()).collect();
    let mut failures: Vec<String> = Vec::new();
    let mut seen: HashSet<(&str, &str)> = HashSet::new();
    for label in labels {
        if !seen.insert((&label.item_id, &label.theme_id)) {
            failures.push(format!(
                "duplicate label for item {} theme {}",
                label.item_id, label.theme_id
            ));
        }
        if !theme_ids.contains(label.theme_id.as_str()) {
            failures.push(format!(
                "label for item {} names unknown theme {}",
                label.item_id, label.theme_id
            ));
        }
        match texts.get(label.item_id.as_str()) {
            None => failures.push(format!(
                "label names item {} which is not a canonical corpus item",
                label.item_id
            )),
            Some(text) => {
                if !text.contains(&label.span) {
                    failures.push(format!(
                        "label span for item {} theme {} is not verbatim",
                        label.item_id, label.theme_id
                    ));
                }
            }
        }
    }
    failures
}

/// Renders a labelling file. The header line comes first when
/// present, then one compact line per label in the order given.
pub fn render(status: Status, header: Option<&FreezeHeader>, labels: &[Label]) -> String {
    let status_text = match status {
        Status::Draft => "draft",
        Status::Frozen => "frozen",
    };
    let mut out = String::new();
    if let Some(h) = header {
        let precision = Value::Obj(
            h.precision
                .iter()
                .map(|(theme, p)| {
                    let val = match p {
                        Some(v) => Value::Num(format!("{v:.4}")),
                        None => Value::Null,
                    };
                    (theme.clone(), val)
                })
                .collect(),
        );
        let v = json::obj(vec![
            ("kind", json::s(HEADER_KIND)),
            ("corpus_sha256", json::s(&h.corpus_sha256)),
            ("taxonomy_sha256", json::s(&h.taxonomy_sha256)),
            ("reviewer", json::s(&h.reviewer)),
            ("precision", precision),
        ]);
        out.push_str(&json::write_compact(&v));
        out.push('\n');
    }
    for label in labels {
        let v = json::obj(vec![
            ("item_id", json::s(&label.item_id)),
            ("theme_id", json::s(&label.theme_id)),
            ("span", json::s(&label.span)),
            ("labeler", json::s(&label.labeler)),
            ("status", json::s(status_text)),
        ]);
        out.push_str(&json::write_compact(&v));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{FreezeHeader, Label, Status, parse_labelling, render, validate};
    use crate::corpus::{Corpus, Item};
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
                mk("b", "A perfectly fine note."),
            ],
        }
    }

    fn taxonomy() -> crate::taxonomy::Taxonomy {
        load_taxonomy(
            br#"{"version": "t1", "themes": [
                {"id": "crashes", "label": "Crashes", "kind": "operational_issue",
                 "phrases": ["crash"]}
            ]}"#,
        )
        .unwrap()
    }

    #[test]
    fn draft_roundtrip_and_validate() {
        let labels = vec![Label {
            item_id: "a".to_string(),
            theme_id: "crashes".to_string(),
            span: "It crashes every day.".to_string(),
            labeler: "test".to_string(),
        }];
        let text = render(Status::Draft, None, &labels);
        let parsed = parse_labelling(&text).unwrap();
        assert_eq!(parsed.status, Status::Draft);
        assert_eq!(parsed.labels, labels);
        assert!(validate(&parsed.labels, &corpus(), &taxonomy()).is_empty());
    }

    #[test]
    fn frozen_needs_header_and_roundtrips() {
        let labels = vec![Label {
            item_id: "a".to_string(),
            theme_id: "crashes".to_string(),
            span: "It crashes every day.".to_string(),
            labeler: "model-x".to_string(),
        }];
        let bare = render(Status::Frozen, None, &labels);
        assert!(parse_labelling(&bare).is_err());
        let header = FreezeHeader {
            corpus_sha256: "c".repeat(64),
            taxonomy_sha256: "d".repeat(64),
            reviewer: "rev".to_string(),
            precision: vec![("crashes".to_string(), Some(0.9))],
        };
        let text = render(Status::Frozen, Some(&header), &labels);
        let parsed = parse_labelling(&text).unwrap();
        assert_eq!(parsed.status, Status::Frozen);
        assert_eq!(parsed.header.unwrap().reviewer, "rev");
    }

    #[test]
    fn validation_catches_bad_span_and_unknown_theme() {
        let labels = vec![
            Label {
                item_id: "a".to_string(),
                theme_id: "crashes".to_string(),
                span: "not in the item".to_string(),
                labeler: "t".to_string(),
            },
            Label {
                item_id: "b".to_string(),
                theme_id: "nope".to_string(),
                span: "A perfectly fine note.".to_string(),
                labeler: "t".to_string(),
            },
        ];
        let failures = validate(&labels, &corpus(), &taxonomy());
        assert_eq!(failures.len(), 2);
    }
}
