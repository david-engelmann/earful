//! The counting core and the report.
//!
//! Counting takes a corpus and a taxonomy and produces a report.
//! The same corpus and taxonomy always produce the same report,
//! and the report renders to the same bytes. Themes under the
//! minimum support are listed as suppressed with their counts,
//! never dropped silently. Minimum support is the larger of three
//! items and one half of one percent of the items analysed.

use crate::corpus::{corpus_hash, Corpus, Item};
use crate::json::{self, Value};
use crate::taxonomy::Taxonomy;
use crate::text;
use std::collections::{BTreeMap, HashMap, HashSet};

/// The output format identifier carried in every report.
pub const FORMAT: &str = "earful-themes/0.1";

/// The maximum number of citations printed per theme.
pub const MAX_CITATIONS: usize = 5;

#[derive(Debug, Clone, PartialEq)]
pub struct Citation {
    pub item_id: String,
    pub span: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeRow {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub item_ids: Vec<String>,
    /// Every citation for the theme, in item order. The report
    /// prints only the first MAX_CITATIONS. Audit samples from
    /// the full list.
    pub citations: Vec<Citation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub taxonomy_version: String,
    pub taxonomy_sha256: String,
    pub corpus_sha256: String,
    pub items_in_folder: usize,
    pub items_total: usize,
    pub decode_warnings: u64,
    pub exact_duplicates: Vec<(String, String)>,
    pub near_duplicate_pairs: Vec<(String, String)>,
    pub untagged_item_ids: Vec<String>,
    pub themes: Vec<ThemeRow>,
    pub suppressed: Vec<(String, usize)>,
}

/// The token key used for duplicate detection. Lowercased, every
/// non alphanumeric character a space, whitespace collapsed.
pub fn norm_key(text: &str) -> String {
    let lowered = text::lower(text);
    let mut key = String::with_capacity(lowered.len());
    let mut last_space = true;
    for ch in lowered.chars() {
        if ch.is_alphanumeric() {
            key.push(ch);
            last_space = false;
        } else if !last_space {
            key.push(' ');
            last_space = true;
        }
    }
    key.trim_end().to_string()
}

/// Runs the count.
pub fn count(corpus: &Corpus, taxonomy: &Taxonomy) -> Report {
    // Exact duplicates on the normalised key. First in corpus
    // order is canonical.
    let mut seen: HashMap<String, String> = HashMap::new();
    let mut canonical: Vec<&Item> = Vec::new();
    let mut exact_duplicates: Vec<(String, String)> = Vec::new();
    for item in &corpus.items {
        let key = norm_key(&item.text);
        if let Some(first) = seen.get(&key) {
            exact_duplicates.push((item.id.clone(), first.clone()));
        } else {
            seen.insert(key, item.id.clone());
            canonical.push(item);
        }
    }

    // Near duplicates among canonical items, token Jaccard at
    // or above 0.90. Flagged, still counted.
    let token_sets: Vec<HashSet<String>> = canonical
        .iter()
        .map(|item| {
            norm_key(&item.text)
                .split(' ')
                .filter(|t| !t.is_empty())
                .map(|t| t.to_string())
                .collect()
        })
        .collect();
    let mut near_pairs: Vec<(String, String)> = Vec::new();
    for i in 0..canonical.len() {
        for j in (i + 1)..canonical.len() {
            let a = &token_sets[i];
            let b = &token_sets[j];
            if a.is_empty() || b.is_empty() {
                continue;
            }
            let inter = a.intersection(b).count();
            let union = a.len() + b.len() - inter;
            if union > 0 && (inter as f64) / (union as f64) >= 0.90 {
                near_pairs.push((canonical[i].id.clone(), canonical[j].id.clone()));
            }
        }
    }

    // Whole token matching, multi label.
    let total = canonical.len();
    let min_support = 3usize.max((total + 199) / 200); // ceil of 0.5 percent
    let mut theme_items: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut theme_citations: BTreeMap<String, Vec<Citation>> = BTreeMap::new();
    let mut tagged: HashSet<String> = HashSet::new();
    for item in &canonical {
        let item_lower = text::lower(&item.text);
        for theme in &taxonomy.themes {
            if text::theme_hits(&item_lower, &theme.phrases) {
                theme_items
                    .entry(theme.id.clone())
                    .or_default()
                    .push(item.id.clone());
                tagged.insert(item.id.clone());
                if let Some(span) = text::citation_span(&item.text, &theme.phrases) {
                    theme_citations
                        .entry(theme.id.clone())
                        .or_default()
                        .push(Citation {
                            item_id: item.id.clone(),
                            span,
                        });
                }
            }
        }
    }

    let mut themes: Vec<ThemeRow> = Vec::new();
    let mut suppressed: Vec<(String, usize)> = Vec::new();
    for theme in &taxonomy.themes {
        let ids = theme_items.get(&theme.id).cloned().unwrap_or_default();
        if ids.len() < min_support {
            if !ids.is_empty() {
                suppressed.push((theme.id.clone(), ids.len()));
            }
            continue;
        }
        themes.push(ThemeRow {
            id: theme.id.clone(),
            label: theme.label.clone(),
            kind: theme.kind.clone(),
            item_ids: ids,
            citations: theme_citations.get(&theme.id).cloned().unwrap_or_default(),
        });
    }
    themes.sort_by(|a, b| {
        b.item_ids
            .len()
            .cmp(&a.item_ids.len())
            .then_with(|| a.id.cmp(&b.id))
    });
    suppressed.sort();

    let untagged: Vec<String> = canonical
        .iter()
        .filter(|item| !tagged.contains(&item.id))
        .map(|item| item.id.clone())
        .collect();

    Report {
        taxonomy_version: taxonomy.version.clone(),
        taxonomy_sha256: taxonomy.sha256.clone(),
        corpus_sha256: corpus_hash(&corpus.items),
        items_in_folder: corpus.items.len(),
        items_total: total,
        decode_warnings: corpus.decode_warnings,
        exact_duplicates,
        near_duplicate_pairs: near_pairs,
        untagged_item_ids: untagged,
        themes,
        suppressed,
    }
}

/// Share of analysed items, rendered with exactly four decimal
/// places by integer arithmetic, half up.
fn share_text(count: usize, total: usize) -> String {
    if total == 0 {
        return "0.0000".to_string();
    }
    let scaled = (count as u128) * 100_000u128;
    let units = (scaled / (total as u128) + 5) / 10; // four places, half up
    format!("{}.{:04}", units / 10_000, units % 10_000)
}

/// Renders the report as canonical JSON text.
pub fn render_json(report: &Report) -> String {
    let themes: Vec<Value> = report
        .themes
        .iter()
        .map(|row| {
            json::obj(vec![
                ("id", json::s(&row.id)),
                ("label", json::s(&row.label)),
                ("kind", json::s(&row.kind)),
                ("count", json::n(row.item_ids.len() as u64)),
                (
                    "share",
                    Value::Num(share_text(row.item_ids.len(), report.items_total)),
                ),
                (
                    "item_ids",
                    Value::Arr(row.item_ids.iter().map(|i| json::s(i)).collect()),
                ),
                (
                    "citations",
                    Value::Arr(
                        row.citations
                            .iter()
                            .take(MAX_CITATIONS)
                            .map(|c| {
                                json::obj(vec![
                                    ("item_id", json::s(&c.item_id)),
                                    ("span", json::s(&c.span)),
                                ])
                            })
                            .collect(),
                    ),
                ),
            ])
        })
        .collect();
    let root = json::obj(vec![
        ("format", json::s(FORMAT)),
        ("taxonomy_version", json::s(&report.taxonomy_version)),
        ("taxonomy_sha256", json::s(&report.taxonomy_sha256)),
        ("corpus_sha256", json::s(&report.corpus_sha256)),
        ("items_in_folder", json::n(report.items_in_folder as u64)),
        ("items_total", json::n(report.items_total as u64)),
        ("decode_warnings", json::n(report.decode_warnings)),
        (
            "exact_duplicates",
            Value::Arr(
                report
                    .exact_duplicates
                    .iter()
                    .map(|(dup, first)| {
                        json::obj(vec![
                            ("item_id", json::s(dup)),
                            ("duplicate_of", json::s(first)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "near_duplicate_pairs",
            Value::Arr(
                report
                    .near_duplicate_pairs
                    .iter()
                    .map(|(a, b)| Value::Arr(vec![json::s(a), json::s(b)]))
                    .collect(),
            ),
        ),
        (
            "items_untagged",
            json::n(report.untagged_item_ids.len() as u64),
        ),
        (
            "untagged_item_ids",
            Value::Arr(report.untagged_item_ids.iter().map(|i| json::s(i)).collect()),
        ),
        ("themes", Value::Arr(themes)),
        (
            "suppressed_themes",
            Value::Arr(
                report
                    .suppressed
                    .iter()
                    .map(|(id, count)| {
                        json::obj(vec![("id", json::s(id)), ("count", json::n(*count as u64))])
                    })
                    .collect(),
            ),
        ),
    ]);
    json::write_canonical(&root)
}

/// Renders the report as the human readable markdown summary.
pub fn render_markdown(report: &Report) -> String {
    let mut out = String::new();
    out.push_str("# Feedback themes\n\n");
    out.push_str(&format!(
        "Items analysed {}. Items with no theme {}. Exact duplicates excluded {}. Near duplicate pairs flagged {}.\n\n",
        report.items_total,
        report.untagged_item_ids.len(),
        report.exact_duplicates.len(),
        report.near_duplicate_pairs.len()
    ));
    for row in &report.themes {
        let share_pct = if report.items_total == 0 {
            0.0
        } else {
            (row.item_ids.len() as f64) * 100.0 / (report.items_total as f64)
        };
        out.push_str(&format!(
            "## {} ({} items, {:.1}%)\n\n",
            row.label,
            row.item_ids.len(),
            share_pct
        ));
        for cit in row.citations.iter().take(MAX_CITATIONS) {
            out.push_str(&format!("- [{}] \"{}\"\n", cit.item_id, cit.span));
        }
        out.push('\n');
    }
    if !report.suppressed.is_empty() {
        out.push_str("## Below minimum support\n\n");
        for (id, count) in &report.suppressed {
            out.push_str(&format!("- {id} ({count} items)\n"));
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{count, norm_key, render_json};
    use crate::corpus::{Corpus, Item};
    use crate::sha256::sha256_hex;
    use crate::taxonomy::load_taxonomy;

    fn item(id: &str, text: &str) -> Item {
        Item {
            id: id.to_string(),
            text: text.to_string(),
            sha256: sha256_hex(text.as_bytes()),
        }
    }

    fn taxonomy() -> crate::taxonomy::Taxonomy {
        load_taxonomy(
            br#"{"version": "t1", "themes": [
                {"id": "crashes", "label": "Crashes", "kind": "operational_issue",
                 "phrases": ["crash", "crashes"]},
                {"id": "ads", "label": "Ads", "kind": "operational_issue",
                 "phrases": ["ads"]}
            ]}"#,
        )
        .unwrap()
    }

    #[test]
    fn norm_key_collapses() {
        assert_eq!(norm_key("Hello,  WORLD! It's 2.0"), "hello world it s 2 0");
    }

    #[test]
    fn counts_and_suppresses() {
        let corpus = Corpus {
            decode_warnings: 0,
            items: vec![
                item("a", "It crashes every day."),
                item("b", "Another crash here."),
                item("c", "The app crashes at noon."),
                item("d", "He reads threads on his headset daily."),
                item("e", "It crashes every day."),
            ],
        };
        let report = count(&corpus, &taxonomy());
        assert_eq!(report.items_total, 4);
        assert_eq!(report.exact_duplicates.len(), 1);
        assert_eq!(report.themes.len(), 1);
        assert_eq!(report.themes[0].id, "crashes");
        assert_eq!(report.themes[0].item_ids.len(), 3);
        // The ads phrase stayed silent inside reads and headset.
        assert_eq!(report.suppressed.len(), 0);
        let text = render_json(&report);
        assert!(text.contains("\"format\": \"earful-themes/0.1\""));
    }
}
