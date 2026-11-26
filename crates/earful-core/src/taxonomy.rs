//! Taxonomy loading and validation.
//!
//! A taxonomy is a versioned JSON data file with a version string
//! and a list of themes. Each theme carries a stable id, a plain
//! language label, a kind, and the list of phrases that file an
//! item under it. The taxonomy hash printed in every report is the
//! SHA-256 of the file bytes exactly as read.

use crate::json;
use crate::sha256::sha256_hex;

#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub phrases: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Taxonomy {
    pub version: String,
    pub themes: Vec<Theme>,
    pub sha256: String,
}

/// Loads and validates a taxonomy from its raw file bytes.
pub fn load_taxonomy(bytes: &[u8]) -> Result<Taxonomy, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "taxonomy is not valid UTF-8".to_string())?;
    let value = json::parse(text).map_err(|e| format!("taxonomy JSON error: {e}"))?;
    let version = value
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or("taxonomy has no version string")?
        .to_string();
    let theme_values = value
        .get("themes")
        .and_then(|v| v.as_arr())
        .ok_or("taxonomy has no themes array")?;
    let mut themes = Vec::new();
    for tv in theme_values {
        let id = tv
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or("a theme has no id")?
            .to_string();
        let label = tv
            .get("label")
            .and_then(|v| v.as_str())
            .ok_or("a theme has no label")?
            .to_string();
        let kind = tv
            .get("kind")
            .and_then(|v| v.as_str())
            .ok_or("a theme has no kind")?
            .to_string();
        let phrase_values = tv
            .get("phrases")
            .and_then(|v| v.as_arr())
            .ok_or("a theme has no phrases array")?;
        let mut phrases = Vec::new();
        for pv in phrase_values {
            let p = pv
                .as_str()
                .ok_or("a theme phrase is not a string")?
                .to_string();
            if p.trim().is_empty() {
                return Err(format!("theme {id} has an empty phrase"));
            }
            phrases.push(p);
        }
        if phrases.is_empty() {
            return Err(format!("theme {id} has no phrases"));
        }
        if themes.iter().any(|t: &Theme| t.id == id) {
            return Err(format!("duplicate theme id {id}"));
        }
        themes.push(Theme {
            id,
            label,
            kind,
            phrases,
        });
    }
    if themes.is_empty() {
        return Err("taxonomy has no themes".to_string());
    }
    Ok(Taxonomy {
        version,
        themes,
        sha256: sha256_hex(bytes),
    })
}

#[cfg(test)]
mod tests {
    use super::load_taxonomy;

    const SAMPLE: &[u8] = br#"{"version": "9.9", "themes": [
        {"id": "a", "label": "A", "kind": "operational_issue", "phrases": ["x"]},
        {"id": "b", "label": "B", "kind": "recognition", "phrases": ["y", "z"]}
    ]}"#;

    #[test]
    fn loads_sample() {
        let tax = load_taxonomy(SAMPLE).unwrap();
        assert_eq!(tax.version, "9.9");
        assert_eq!(tax.themes.len(), 2);
        assert_eq!(tax.themes[1].phrases.len(), 2);
        assert_eq!(tax.sha256.len(), 64);
    }

    #[test]
    fn rejects_duplicate_ids() {
        let bad = br#"{"version": "1", "themes": [
            {"id": "a", "label": "A", "kind": "k", "phrases": ["x"]},
            {"id": "a", "label": "A2", "kind": "k", "phrases": ["y"]}
        ]}"#;
        assert!(load_taxonomy(bad).is_err());
    }

    #[test]
    fn rejects_empty_phrase() {
        let bad = br#"{"version": "1", "themes": [
            {"id": "a", "label": "A", "kind": "k", "phrases": ["  "]}
        ]}"#;
        assert!(load_taxonomy(bad).is_err());
    }

    #[test]
    fn shipped_taxonomy_matches_its_audit_record() {
        // The shipped taxonomy 0.2 file is a measured
        // artefact. Its audit record names eight themes, five
        // counted and three suppressed, after two were pulled
        // for sitting under the measurement floor. If the
        // file changes, the record and this test change with
        // it, in the open.
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../taxonomy/v0.2.json");
        let bytes = std::fs::read(&path).expect("shipped taxonomy missing");
        let tax = load_taxonomy(&bytes).expect("shipped taxonomy does not parse");
        assert_eq!(tax.version, "0.2.0");
        let ids: Vec<&str> = tax.themes.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "crashes_and_stability",
                "performance_and_speed",
                "battery_and_resources",
                "interface_and_design",
                "missing_features",
                "sync_and_data",
                "regressions_after_update",
                "praise_and_recognition",
            ]
        );
    }
}
