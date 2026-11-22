//! Folder ingest, the version 1 input shape one.
//!
//! Every .txt and .md file in the folder is one item, read in
//! sorted file name order, and the item id is the file name
//! without its extension. Bytes are decoded as UTF-8. Invalid
//! sequences are replaced and counted as warnings. Line endings
//! are normalised to LF. The item hash is the SHA-256 of the
//! normalised text bytes, so the hash always names the text the
//! citations are drawn from.

use crate::sha256::sha256_hex;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: String,
    pub text: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Corpus {
    pub items: Vec<Item>,
    pub decode_warnings: u64,
}

/// The corpus hash printed in every report. The SHA-256 of the
/// concatenation of one line per item, id, colon, item hash,
/// newline, in corpus order.
pub fn corpus_hash(items: &[Item]) -> String {
    let mut acc = String::new();
    for item in items {
        acc.push_str(&item.id);
        acc.push(':');
        acc.push_str(&item.sha256);
        acc.push('\n');
    }
    sha256_hex(acc.as_bytes())
}

/// Normalises line endings to LF.
pub fn normalise(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// Loads a corpus from a folder path.
pub fn load_corpus(folder: &Path) -> Result<Corpus, String> {
    let mut names: Vec<String> = Vec::new();
    let entries =
        std::fs::read_dir(folder).map_err(|e| format!("cannot read folder {}: {e}", folder.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("cannot read folder entry: {e}"))?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".txt") || name.ends_with(".md") {
            names.push(name);
        }
    }
    names.sort();
    let mut items = Vec::new();
    let mut warnings = 0u64;
    for name in names {
        let path = folder.join(&name);
        let bytes = std::fs::read(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let (text, file_warnings) = decode(&bytes);
        warnings += file_warnings;
        let text = normalise(&text);
        let id = name
            .rsplit_once('.')
            .map(|(stem, _)| stem.to_string())
            .unwrap_or(name.clone());
        let sha256 = sha256_hex(text.as_bytes());
        items.push(Item { id, text, sha256 });
    }
    Ok(Corpus {
        items,
        decode_warnings: warnings,
    })
}

/// Decodes bytes as UTF-8, counting one warning per replacement
/// character introduced by lossy decoding.
fn decode(bytes: &[u8]) -> (String, u64) {
    match std::str::from_utf8(bytes) {
        Ok(s) => (s.to_string(), 0),
        Err(_) => {
            let lossy = String::from_utf8_lossy(bytes).to_string();
            let count = lossy.chars().filter(|c| *c == '\u{FFFD}').count() as u64;
            (lossy, count)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::normalise;

    #[test]
    fn crlf_and_cr_become_lf() {
        assert_eq!(normalise("a\r\nb\rc\nd"), "a\nb\nc\nd");
    }
}
