//! The dusk.fm export contract check. The export
//! specification in docs/DUSKFM-EXPORT.md pins one JSONL
//! shape. This module checks a candidate export file against
//! that contract before any count from it is trusted, and
//! reports the export side numbers the first run reconciles
//! against. It judges the file, never the feedback in it.

use crate::json::{self, Value};

/// The source values the export starts with, per the spec.
pub const SOURCES: [&str; 4] = ["support", "review", "in_app", "email"];

#[derive(Debug, Clone, PartialEq)]
pub struct ExportReport {
    /// Non empty lines in the file.
    pub lines: usize,
    /// Items per source value, in first seen order.
    pub sources: Vec<(String, usize)>,
    /// Contract failures, empty on pass.
    pub failures: Vec<String>,
}

/// Checks JSONL export text against the contract. Every line
/// must be one object with a required id, a required source
/// from the starting list, and a required text. Ids must be
/// unique and sorted ascending. A date, when present, must be
/// ISO 8601 UTC. A rating, when present, must be a number.
pub fn check_export(text: &str) -> ExportReport {
    let mut failures: Vec<String> = Vec::new();
    let mut sources: Vec<(String, usize)> = Vec::new();
    let mut lines = 0usize;
    let mut previous_id: Option<String> = None;
    let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        lines += 1;
        let at = i + 1;
        let value = match json::parse(line) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!("line {at} does not parse: {e}"));
                continue;
            }
        };
        let id = match value.get("id").and_then(|v| v.as_str()) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => {
                failures.push(format!("line {at} has no id"));
                continue;
            }
        };
        if !seen_ids.insert(id.clone()) {
            failures.push(format!("line {at} repeats id {id}"));
        }
        if let Some(prev) = &previous_id
            && id < *prev
        {
            failures.push(format!("line {at} is out of order, id {id} follows {prev}"));
        }
        previous_id = Some(id);
        match value.get("text").and_then(|v| v.as_str()) {
            Some(_) => {}
            None => failures.push(format!("line {at} has no text field")),
        }
        match value.get("source").and_then(|v| v.as_str()) {
            Some(s) if SOURCES.contains(&s) => {
                match sources.iter_mut().find(|(name, _)| name == s) {
                    Some(entry) => entry.1 += 1,
                    None => sources.push((s.to_string(), 1)),
                }
            }
            Some(s) => failures.push(format!(
                "line {at} names source {s}, outside the export list"
            )),
            None => failures.push(format!("line {at} has no source")),
        }
        if let Some(date) = value.get("date")
            && !matches!(date, Value::Str(s) if is_iso_utc(s))
        {
            failures.push(format!("line {at} has a date that is not ISO 8601 UTC"));
        }
        if let Some(rating) = value.get("rating")
            && !matches!(rating, Value::Num(_))
        {
            failures.push(format!("line {at} has a rating that is not a number"));
        }
    }
    if lines == 0 {
        failures.push("export file has no lines".to_string());
    }
    ExportReport {
        lines,
        sources,
        failures,
    }
}

/// The receiving side reconciliation from the export
/// specification. The export line count must equal the items
/// the counter read, the analysed count must equal the export
/// count minus the exact duplicates listed, and the decode
/// warning count must be zero. Returns the failures, empty on
/// pass. The fourth reconciliation point, the scored audit
/// worksheet, is a human step the audit subcommand serves.
pub fn reconcile(export_lines: usize, report: &crate::report::Report) -> Vec<String> {
    let mut failures = Vec::new();
    if report.items_in_folder != export_lines {
        failures.push(format!(
            "export lines {} do not equal items read {}",
            export_lines, report.items_in_folder
        ));
    }
    let expected_total = export_lines - report.exact_duplicates.len();
    if report.items_total != expected_total {
        failures.push(format!(
            "analysed items {} do not equal export lines minus duplicates {}",
            report.items_total, expected_total
        ));
    }
    if report.decode_warnings != 0 {
        failures.push(format!(
            "decode warnings {} are not zero",
            report.decode_warnings
        ));
    }
    failures
}

/// Strict shape check for an ISO 8601 UTC timestamp of the
/// form YYYY-MM-DDTHH:MM:SSZ, with optional fractional seconds.
pub fn is_iso_utc(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() < 20 || !s.ends_with('Z') {
        return false;
    }
    let digit = |i: usize| bytes[i].is_ascii_digit();
    let shape = digit(0)
        && digit(1)
        && digit(2)
        && digit(3)
        && bytes[4] == b'-'
        && digit(5)
        && digit(6)
        && bytes[7] == b'-'
        && digit(8)
        && digit(9)
        && bytes[10] == b'T'
        && digit(11)
        && digit(12)
        && bytes[13] == b':'
        && digit(14)
        && digit(15)
        && bytes[16] == b':'
        && digit(17)
        && digit(18);
    if !shape {
        return false;
    }
    let num = |from: usize, to: usize| s[from..to].parse::<u32>().unwrap_or(99);
    let rest_ok = bytes.len() == 20
        || (bytes[19] == b'.'
            && bytes[20..bytes.len() - 1]
                .iter()
                .all(|b| b.is_ascii_digit())
            && bytes.len() > 21);
    rest_ok
        && (1..=12).contains(&num(5, 7))
        && (1..=31).contains(&num(8, 10))
        && num(11, 13) <= 23
        && num(14, 16) <= 59
        && num(17, 19) <= 59
}

#[cfg(test)]
mod tests {
    use super::{check_export, is_iso_utc};

    const GOOD: &str = "{\"id\": \"fb-0001\", \"source\": \"in_app\", \"date\": \"2026-11-01T14:23:00Z\", \"text\": \"The player stops playing when the screen locks and I have to reopen the app.\"}\n{\"id\": \"fb-0002\", \"source\": \"support\", \"date\": \"2026-11-02T09:05:00Z\", \"rating\": 2, \"text\": \"Sync between my phone and desktop has not worked since the last update.\"}\n";

    #[test]
    fn spec_examples_pass() {
        // The two synthetic example lines from the spec.
        let report = check_export(GOOD);
        assert!(
            report.failures.is_empty(),
            "failures: {:?}",
            report.failures
        );
        assert_eq!(report.lines, 2);
        assert_eq!(report.sources.len(), 2);
    }

    #[test]
    fn contract_breaks_are_named() {
        let bad = "{\"id\": \"fb-0002\", \"source\": \"chat\", \"text\": \"x\"}\n{\"id\": \"fb-0001\", \"source\": \"support\", \"date\": \"Nov 2\", \"text\": \"y\"}\n{\"source\": \"support\", \"text\": \"z\"}\n";
        let report = check_export(bad);
        assert_eq!(report.failures.len(), 4, "failures: {:?}", report.failures);
    }

    #[test]
    fn iso_shape() {
        assert!(is_iso_utc("2026-11-01T14:23:00Z"));
        assert!(is_iso_utc("2026-11-01T14:23:00.5Z"));
        assert!(!is_iso_utc("2026-13-01T14:23:00Z"));
        assert!(!is_iso_utc("2026-11-01 14:23:00Z"));
        assert!(!is_iso_utc("2026-11-01T14:23:00+01:00"));
    }

    #[test]
    fn reconcile_accepts_and_names() {
        use crate::corpus::{Corpus, Item};
        use crate::sha256::sha256_hex;
        let mk = |id: &str, text: &str| Item {
            id: id.to_string(),
            text: text.to_string(),
            sha256: sha256_hex(text.as_bytes()),
        };
        let corpus = Corpus {
            decode_warnings: 0,
            items: vec![
                mk("a", "It crashes daily."),
                mk("b", "It crashes daily."),
                mk("c", "A fine note."),
            ],
        };
        let taxonomy = crate::taxonomy::load_taxonomy(
            br#"{"version": "t", "themes": [
                {"id": "crashes", "label": "Crashes", "kind": "operational_issue",
                 "phrases": ["crashes"]}
            ]}"#,
        )
        .unwrap();
        let report = crate::report::count(&corpus, &taxonomy);
        // Three lines in, one exact duplicate, two analysed.
        assert!(super::reconcile(3, &report).is_empty());
        assert_eq!(super::reconcile(4, &report).len(), 2);
    }
}
