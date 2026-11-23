//! The audit subcommand. Deterministic sampling, the scoring
//! worksheet, and the precision gate computation.
//!
//! The sample is evenly spaced entries across each theme's full
//! citation list in report order, ten per theme or all when
//! fewer, so the same run always yields the same worksheet. A
//! theme needs at least five citations in total to be measured
//! at all. Precision is supports over sampled, with against and
//! unclear both counting in the denominator only. The gate is
//! four in five, applied per theme.

use crate::json::{self, Value};
use crate::report::Report;
use std::collections::BTreeMap;

/// Citations sampled per theme at most.
pub const SAMPLE_SIZE: usize = 10;

/// The minimum total citations for a theme to be measurable.
pub const MIN_MEASURED: usize = 5;

/// The precision gate, four in five.
pub const GATE: f64 = 0.80;

#[derive(Debug, Clone, PartialEq)]
pub struct SampleEntry {
    pub theme_id: String,
    pub theme_total: usize,
    pub item_id: String,
    pub span: String,
}

/// Selects the deterministic audit sample from a report.
pub fn sample(report: &Report) -> Vec<SampleEntry> {
    let pools: Vec<(String, Vec<crate::report::Citation>)> = report
        .themes
        .iter()
        .map(|row| (row.id.clone(), row.citations.clone()))
        .collect();
    sample_pools(&pools)
}

/// Selects the sample from explicit per theme citation pools,
/// in the order the pools are given. Used for report themes and
/// for draft labelling files alike.
pub fn sample_pools(pools: &[(String, Vec<crate::report::Citation>)]) -> Vec<SampleEntry> {
    let mut entries = Vec::new();
    for (theme_id, pool) in pools {
        let take = SAMPLE_SIZE.min(pool.len());
        if take == 0 {
            continue;
        }
        for k in 0..take {
            let idx = (k * pool.len()) / take;
            let cit = &pool[idx];
            entries.push(SampleEntry {
                theme_id: theme_id.clone(),
                theme_total: pool.len(),
                item_id: cit.item_id.clone(),
                span: cit.span.clone(),
            });
        }
    }
    entries
}

/// Renders the sample as audit.jsonl, one object per line with
/// an empty verdict field for the scorer to fill.
pub fn render_sample_jsonl(entries: &[SampleEntry]) -> String {
    let mut out = String::new();
    for e in entries {
        let v = json::obj(vec![
            ("theme_id", json::s(&e.theme_id)),
            ("theme_total", json::n(e.theme_total as u64)),
            ("item_id", json::s(&e.item_id)),
            ("span", json::s(&e.span)),
            ("verdict", json::s("")),
        ]);
        out.push_str(&json::write_compact(&v));
        out.push('\n');
    }
    out
}

/// Renders the human worksheet, audit.md. Each sampled citation
/// is quoted with the full text of its item and a blank verdict
/// line, per the audit design.
pub fn render_worksheet(
    entries: &[SampleEntry],
    item_texts: &std::collections::HashMap<String, String>,
) -> String {
    let mut out = String::new();
    out.push_str("# Audit worksheet\n\n");
    let mut current_theme = String::new();
    for e in entries {
        if e.theme_id != current_theme {
            current_theme = e.theme_id.clone();
            out.push_str(&format!(
                "## {} ({} citations in total)\n\n",
                e.theme_id, e.theme_total
            ));
        }
        let full = item_texts.get(&e.item_id).cloned().unwrap_or_default();
        out.push_str(&format!("- [{}] \"{}\"\n", e.item_id, e.span));
        out.push_str(&format!("  Item in full: {}\n", full.replace('\n', " ")));
        out.push_str("  Verdict: \n\n");
    }
    out
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeScore {
    pub theme_id: String,
    pub theme_total: usize,
    pub sampled: usize,
    pub supports: usize,
    /// Precision, or None when the theme is unmeasured.
    pub precision: Option<f64>,
    pub passed: Option<bool>,
}

/// Scores a completed audit. The sample text is the audit.jsonl
/// the tool produced. The verdicts text is the same lines with
/// the verdict field filled as support, against, or unclear.
/// A sampled line missing from the verdicts counts as unclear.
pub fn score(sample_text: &str, verdicts_text: &str) -> Result<Vec<ThemeScore>, String> {
    let sample = parse_jsonl(sample_text)?;
    let verdicts = parse_jsonl(verdicts_text)?;
    let mut verdict_map: BTreeMap<(String, String), String> = BTreeMap::new();
    for v in &verdicts {
        let theme = v.get("theme_id").and_then(|x| x.as_str()).unwrap_or("");
        let item = v.get("item_id").and_then(|x| x.as_str()).unwrap_or("");
        let verdict = v.get("verdict").and_then(|x| x.as_str()).unwrap_or("");
        verdict_map.insert((theme.to_string(), item.to_string()), verdict.to_string());
    }
    let mut by_theme: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
    for entry in &sample {
        let theme = entry
            .get("theme_id")
            .and_then(|x| x.as_str())
            .ok_or("sample line has no theme_id")?
            .to_string();
        let item = entry
            .get("item_id")
            .and_then(|x| x.as_str())
            .ok_or("sample line has no item_id")?
            .to_string();
        let total = entry
            .get("theme_total")
            .and_then(|x| x.as_f64())
            .unwrap_or(0.0) as usize;
        let acc = by_theme.entry(theme.clone()).or_insert((total, 0, 0));
        acc.1 += 1;
        let verdict = verdict_map
            .get(&(theme, item))
            .map(|s| s.as_str())
            .unwrap_or("");
        match verdict {
            "support" => acc.2 += 1,
            "against" | "unclear" | "" => {}
            other => return Err(format!("unknown verdict {other}")),
        }
    }
    let mut scores = Vec::new();
    for (theme_id, (total, sampled, supports)) in by_theme {
        let (precision, passed) = if total < MIN_MEASURED {
            (None, None)
        } else {
            let p = (supports as f64) / (sampled as f64);
            (Some(p), Some(p >= GATE))
        };
        scores.push(ThemeScore {
            theme_id,
            theme_total: total,
            sampled,
            supports,
            precision,
            passed,
        });
    }
    Ok(scores)
}

fn parse_jsonl(text: &str) -> Result<Vec<Value>, String> {
    let mut values = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let v = json::parse(line).map_err(|e| format!("line {} does not parse: {e}", i + 1))?;
        values.push(v);
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::{SampleEntry, render_sample_jsonl, sample, score};
    use crate::report::{Citation, Report, ThemeRow};

    fn report_with(pool_sizes: Vec<(&str, usize)>) -> Report {
        let themes = pool_sizes
            .into_iter()
            .map(|(id, n)| ThemeRow {
                id: id.to_string(),
                label: id.to_string(),
                kind: "operational_issue".to_string(),
                item_ids: (0..n).map(|i| format!("item-{i}")).collect(),
                citations: (0..n)
                    .map(|i| Citation {
                        item_id: format!("item-{i}"),
                        span: format!("span {i}"),
                    })
                    .collect(),
            })
            .collect();
        Report {
            taxonomy_version: "t".to_string(),
            taxonomy_sha256: "x".to_string(),
            corpus_sha256: "y".to_string(),
            items_in_folder: 0,
            items_total: 0,
            decode_warnings: 0,
            exact_duplicates: vec![],
            near_duplicate_pairs: vec![],
            untagged_item_ids: vec![],
            themes,
            suppressed: vec![],
            labelling_sha256: None,
        }
    }

    #[test]
    fn sample_is_evenly_spaced_and_deterministic() {
        let report = report_with(vec![("t", 40)]);
        let a: Vec<SampleEntry> = sample(&report);
        let b = sample(&report);
        assert_eq!(a, b);
        assert_eq!(a.len(), 10);
        assert_eq!(a[0].item_id, "item-0");
        assert_eq!(a[1].item_id, "item-4");
        assert_eq!(a[9].item_id, "item-36");
    }

    #[test]
    fn scoring_applies_gate_and_unmeasured() {
        let report = report_with(vec![("big", 10), ("small", 4)]);
        let entries = sample(&report);
        let sample_text = render_sample_jsonl(&entries);
        // Score big at 9 of 10 support, small fully support.
        let mut verdicts = String::new();
        for (i, line) in sample_text.lines().enumerate() {
            let verdict = if i == 0 { "against" } else { "support" };
            verdicts
                .push_str(&line.replace("\"verdict\":\"\"", &format!("\"verdict\":\"{verdict}\"")));
            verdicts.push('\n');
        }
        let scores = score(&sample_text, &verdicts).unwrap();
        let big = scores.iter().find(|s| s.theme_id == "big").unwrap();
        assert_eq!(big.precision, Some(0.9));
        assert_eq!(big.passed, Some(true));
        let small = scores.iter().find(|s| s.theme_id == "small").unwrap();
        assert_eq!(small.precision, None);
        assert_eq!(small.passed, None);
    }
}
