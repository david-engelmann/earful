//! The counting core and the report.
//!
//! Counting takes a corpus and a taxonomy and produces a report.
//! The same corpus and taxonomy always produce the same report,
//! and the report renders to the same bytes. Themes under the
//! minimum support are listed as suppressed with their counts,
//! never dropped silently. Minimum support is the larger of three
//! items and one half of one percent of the items analysed.

use crate::corpus::{Corpus, Item, corpus_hash};
use crate::json::{self, Value};
use crate::labels::{LabellingFile, Status};
use crate::sha256::sha256_hex;
use crate::taxonomy::Taxonomy;
use crate::text;
use std::collections::{BTreeMap, HashMap, HashSet};

/// The output format identifier carried in every report.
pub const FORMAT: &str = "earful-themes/0.2";

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
    /// SHA-256 of the frozen labelling file counted from, when
    /// the run used one.
    pub labelling_sha256: Option<String>,
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

/// Splits a corpus into canonical items and exact duplicates.
/// First in corpus order is canonical. Returns the canonical
/// items and the duplicate pairs as item id and canonical id.
pub fn canonical_items(corpus: &Corpus) -> (Vec<&Item>, Vec<(String, String)>) {
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
    (canonical, exact_duplicates)
}

/// Runs the count without a labelling file.
pub fn count(corpus: &Corpus, taxonomy: &Taxonomy) -> Report {
    count_inner(corpus, taxonomy, None).expect("count without labels cannot fail")
}

/// Runs the count, optionally from a frozen labelling file
/// given as the parsed file and its raw text. A draft file is
/// a hard error. A frozen file whose header hashes do not
/// match this corpus and taxonomy is a hard error. Labels add
/// their themes to the items they name, one count per item per
/// theme as always, and a label's span is the citation when a
/// label exists for the item and theme.
pub fn count_with_labels(
    corpus: &Corpus,
    taxonomy: &Taxonomy,
    labelling: Option<(&LabellingFile, &str)>,
) -> Result<Report, String> {
    count_inner(corpus, taxonomy, labelling)
}

fn count_inner(
    corpus: &Corpus,
    taxonomy: &Taxonomy,
    labelling: Option<(&LabellingFile, &str)>,
) -> Result<Report, String> {
    // Validate the labelling file before any counting.
    let mut label_spans: HashMap<(String, String), String> = HashMap::new();
    let mut labelling_sha256: Option<String> = None;
    if let Some((file, raw_text)) = labelling {
        if file.status != Status::Frozen {
            return Err(
                "labelling file is a draft. Only a frozen labelling file can be counted."
                    .to_string(),
            );
        }
        let header = file
            .header
            .as_ref()
            .ok_or("frozen labelling file has no header")?;
        if header.corpus_sha256 != corpus_hash(&corpus.items) {
            return Err("labelling file was frozen against a different corpus".to_string());
        }
        if header.taxonomy_sha256 != taxonomy.sha256 {
            return Err("labelling file was frozen against a different taxonomy".to_string());
        }
        let failures = crate::labels::validate(&file.labels, corpus, taxonomy);
        if !failures.is_empty() {
            return Err(format!(
                "labelling file fails validation: {}",
                failures.join("; ")
            ));
        }
        for label in &file.labels {
            label_spans.insert(
                (label.item_id.clone(), label.theme_id.clone()),
                label.span.clone(),
            );
        }
        labelling_sha256 = Some(sha256_hex(raw_text.as_bytes()));
    }

    let (canonical, exact_duplicates) = canonical_items(corpus);

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
    let near_pairs = near_duplicate_pairs(&canonical, &token_sets);

    // Whole token matching, multi label.
    let total = canonical.len();
    let min_support = 3usize.max(total.div_ceil(200)); // ceil of 0.5 percent
    let mut theme_items: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut theme_citations: BTreeMap<String, Vec<Citation>> = BTreeMap::new();
    let mut tagged: HashSet<String> = HashSet::new();
    for item in &canonical {
        let item_lower = text::lower(&item.text);
        for theme in &taxonomy.themes {
            let label_span = label_spans.get(&(item.id.clone(), theme.id.clone()));
            let lexicon_hit = text::theme_hits(&item_lower, &theme.phrases);
            if lexicon_hit || label_span.is_some() {
                theme_items
                    .entry(theme.id.clone())
                    .or_default()
                    .push(item.id.clone());
                tagged.insert(item.id.clone());
                let span = match label_span {
                    Some(s) => Some(s.clone()),
                    None => text::citation_span(&item.text, &theme.phrases),
                };
                if let Some(span) = span {
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

    Ok(Report {
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
        labelling_sha256,
    })
}

/// Finds near duplicate pairs among canonical items, token
/// Jaccard at or above 0.90. The comparison is exact. Two
/// prefilters only skip pairs that cannot reach the
/// threshold. A pair sharing no token has similarity zero,
/// so a token index generates the candidates. And similarity
/// at or above the threshold needs the smaller set at least
/// nine tenths the larger, so a size ratio check skips the
/// rest. Every surviving candidate gets its exact Jaccard
/// computed, so the pairs are the same pairs an all pairs
/// pass would find, in the same order.
fn near_duplicate_pairs(
    canonical: &[&Item],
    token_sets: &[HashSet<String>],
) -> Vec<(String, String)> {
    // Token to the indices of items carrying it.
    let mut index: HashMap<&str, Vec<usize>> = HashMap::new();
    for (j, set) in token_sets.iter().enumerate() {
        for token in set {
            index.entry(token.as_str()).or_default().push(j);
        }
    }
    let mut pairs: Vec<(String, String)> = Vec::new();
    for j in 0..canonical.len() {
        let b = &token_sets[j];
        if b.is_empty() {
            continue;
        }
        // Intersection counts against every earlier item that
        // shares at least one token with item j.
        let mut intersections: HashMap<usize, usize> = HashMap::new();
        for token in b {
            if let Some(earlier) = index.get(token.as_str()) {
                for &i in earlier {
                    if i < j {
                        *intersections.entry(i).or_insert(0) += 1;
                    }
                }
            }
        }
        let mut candidates: Vec<usize> = intersections.keys().copied().collect();
        candidates.sort_unstable();
        for i in candidates {
            let a = &token_sets[i];
            let (small, large) = if a.len() <= b.len() {
                (a.len(), b.len())
            } else {
                (b.len(), a.len())
            };
            if (small as f64) < 0.90 * (large as f64) {
                continue;
            }
            let inter = intersections[&i];
            let union = a.len() + b.len() - inter;
            if union > 0 && (inter as f64) / (union as f64) >= 0.90 {
                pairs.push((canonical[i].id.clone(), canonical[j].id.clone()));
            }
        }
    }
    pairs
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
        (
            "labelling_sha256",
            match &report.labelling_sha256 {
                Some(h) => json::s(h),
                None => Value::Null,
            },
        ),
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
            Value::Arr(
                report
                    .untagged_item_ids
                    .iter()
                    .map(|i| json::s(i))
                    .collect(),
            ),
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
    use super::{canonical_items, count, norm_key, render_json, render_markdown};
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
        assert!(text.contains("\"format\": \"earful-themes/0.2\""));
    }

    #[test]
    fn frozen_labels_count_and_drafts_are_refused() {
        use crate::labels::{FreezeHeader, Label, Status, parse_labelling, render};
        use crate::report::count_with_labels;
        let corpus = Corpus {
            decode_warnings: 0,
            items: vec![
                item("a", "It crashes every day."),
                item("b", "Another crash here."),
                item("c", "The app crashes at noon."),
                item("d", "He reads threads on his headset daily."),
            ],
        };
        let tax = taxonomy();
        let labels = vec![Label {
            item_id: "d".to_string(),
            theme_id: "ads".to_string(),
            span: "He reads threads on his headset daily.".to_string(),
            labeler: "test".to_string(),
        }];
        // A draft handed to the counter is a hard error.
        let draft_text = render(Status::Draft, None, &labels);
        let draft = parse_labelling(&draft_text).unwrap();
        assert!(count_with_labels(&corpus, &tax, Some((&draft, &draft_text))).is_err());

        // Frozen against these hashes, the label counts.
        let header = FreezeHeader {
            corpus_sha256: crate::corpus::corpus_hash(&corpus.items),
            taxonomy_sha256: tax.sha256.clone(),
            reviewer: "rev".to_string(),
            precision: vec![("ads".to_string(), None)],
        };
        let frozen_text = render(Status::Frozen, Some(&header), &labels);
        let frozen = parse_labelling(&frozen_text).unwrap();
        let report = count_with_labels(&corpus, &tax, Some((&frozen, &frozen_text))).unwrap();
        let ads = report.themes.iter().find(|t| t.id == "ads");
        assert!(ads.is_none(), "one label is under minimum support");
        assert!(report.suppressed.contains(&("ads".to_string(), 1)));
        assert!(report.labelling_sha256.is_some());
        assert!(!report.untagged_item_ids.contains(&"d".to_string()));
    }

    #[test]
    fn near_duplicate_finder_matches_brute_force() {
        // Ten items over a small vocabulary, with one pair at
        // the threshold and several near misses.
        let texts = [
            "alpha bravo charlie delta echo foxtrot golf hotel india juliet",
            "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo",
            "alpha bravo charlie delta echo foxtrot golf hotel india",
            "one two three four five six seven eight nine ten",
            "alpha bravo charlie delta echo foxtrot golf hotel india juliet",
            "red green blue yellow orange purple pink brown black white",
            "alpha bravo zulu",
            "one two three four five six seven eight nine",
            "lorem ipsum dolor sit amet consectetur adipiscing elit sed do",
            "alpha bravo charlie delta echo foxtrot golf hotel india juliet extra words here now",
        ];
        let corpus = Corpus {
            decode_warnings: 0,
            items: texts
                .iter()
                .enumerate()
                .map(|(i, t)| item(&format!("i{}", i + 1), t))
                .collect(),
        };
        let report = count(&corpus, &taxonomy());
        // Brute force over the same canonical sets.
        let (canonical, _d) = canonical_items(&corpus);
        let sets: Vec<std::collections::HashSet<String>> = canonical
            .iter()
            .map(|it| {
                norm_key(&it.text)
                    .split(' ')
                    .filter(|t| !t.is_empty())
                    .map(|t| t.to_string())
                    .collect()
            })
            .collect();
        let mut expected: Vec<(String, String)> = Vec::new();
        for i in 0..canonical.len() {
            for j in (i + 1)..canonical.len() {
                let (a, b) = (&sets[i], &sets[j]);
                if a.is_empty() || b.is_empty() {
                    continue;
                }
                let inter = a.intersection(b).count();
                let union = a.len() + b.len() - inter;
                if union > 0 && (inter as f64) / (union as f64) >= 0.90 {
                    expected.push((canonical[i].id.clone(), canonical[j].id.clone()));
                }
            }
        }
        assert_eq!(report.near_duplicate_pairs, expected);
        assert!(!expected.is_empty(), "fixture must contain a flagged pair");
    }

    #[test]
    fn repeated_counts_are_byte_identical() {
        // The product promise is byte for byte reproducibility.
        // Hash maps and sets carry random per instance state,
        // so any output path that iterated one would drift
        // between runs. Counting a wide corpus many times with
        // fresh structures each time is the probe that catches
        // such a leak when it is introduced.
        let taxonomy = load_taxonomy(
            br#"{"version": "t2", "themes": [
                {"id": "crashes", "label": "Crashes", "kind": "operational_issue",
                 "phrases": ["crash", "crashes", "freeze"]},
                {"id": "praise", "label": "Praise", "kind": "praise",
                 "phrases": ["love", "great", "excellent"]},
                {"id": "speed", "label": "Speed", "kind": "operational_issue",
                 "phrases": ["slow", "lag", "fast"]},
                {"id": "wishlist", "label": "Wishlist", "kind": "feature_request",
                 "phrases": ["wish", "please add", "missing"]},
                {"id": "battery", "label": "Battery", "kind": "operational_issue",
                 "phrases": ["battery", "drain"]},
                {"id": "ads", "label": "Ads", "kind": "operational_issue",
                 "phrases": ["ads", "advert"]}
            ]}"#,
        )
        .unwrap();
        // A fixed seed LCG builds the corpus, so the test
        // itself is deterministic. The vocabulary is shared
        // across items on purpose, to exercise the near
        // duplicate candidate path as well as the counters.
        let vocab = [
            "crash",
            "love",
            "slow",
            "wish",
            "battery",
            "ads",
            "great",
            "freeze",
            "missing",
            "drain",
            "fast",
            "lag",
            "excellent",
            "advert",
            "screen",
            "button",
            "update",
            "screen",
            "update",
            "daily",
        ];
        let mut state: u64 = 0x2545F4914F6CDD1D;
        let mut pick = move || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) as usize
        };
        let mut items = Vec::new();
        for n in 0..120 {
            let words = 6 + pick() % 14;
            let mut text = String::new();
            for w in 0..words {
                if w > 0 {
                    text.push(' ');
                }
                text.push_str(vocab[pick() % vocab.len()]);
            }
            text.push('.');
            items.push(item(&format!("s{:03}", n + 1), &text));
        }
        let corpus = Corpus {
            decode_warnings: 0,
            items,
        };
        let first = count(&corpus, &taxonomy);
        let first_json = render_json(&first);
        let first_md = render_markdown(&first);
        for _ in 0..24 {
            let again = count(&corpus, &taxonomy);
            assert_eq!(render_json(&again), first_json);
            assert_eq!(render_markdown(&again), first_md);
        }
    }

    #[test]
    fn edge_inputs_produce_well_formed_reports() {
        // An empty corpus, and a corpus of whitespace only
        // items, are the shapes a mistyped folder path or a
        // blank export produce. Both must report cleanly
        // rather than panic or divide by zero.
        let taxonomy = taxonomy();
        let empty = Corpus {
            decode_warnings: 0,
            items: Vec::new(),
        };
        let report = count(&empty, &taxonomy);
        assert_eq!(report.items_total, 0);
        assert!(report.themes.is_empty());
        assert!(render_markdown(&report).contains("Items analysed 0"));

        let blank = Corpus {
            decode_warnings: 0,
            items: vec![item("b1", "   "), item("b2", "Text with no theme words.")],
        };
        let report = count(&blank, &taxonomy);
        assert_eq!(report.items_total, 2);
        assert_eq!(report.untagged_item_ids.len(), 2);
        assert!(report.themes.is_empty());
    }
}
