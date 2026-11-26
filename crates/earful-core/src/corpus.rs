//! Folder ingest, the version 1 input shape one, plus the
//! CSV and JSONL file shapes from the amendment.
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

/// Loads a corpus from a single CSV or JSONL file, chosen by
/// the file extension. Any other extension is refused by
/// name, per the v1 input fence.
pub fn load_corpus_file(path: &Path) -> Result<Corpus, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let (text, warnings) = decode(&bytes);
    let name = path.to_string_lossy().to_lowercase();
    let mut corpus = if name.ends_with(".csv") {
        parse_csv(&text)?
    } else if name.ends_with(".jsonl") {
        parse_jsonl(&text)?
    } else {
        return Err(format!(
            "unsupported input file {}. Version 1 takes a folder, a .csv file, or a .jsonl file",
            path.display()
        ));
    };
    corpus.decode_warnings = warnings;
    Ok(corpus)
}

fn item_from(id: String, text: &str) -> Item {
    let text = normalise(text);
    Item {
        sha256: sha256_hex(text.as_bytes()),
        id,
        text,
    }
}

fn check_unique_ids(items: &[Item]) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    for item in items {
        if !seen.insert(item.id.clone()) {
            return Err(format!("duplicate item id {} in input", item.id));
        }
    }
    Ok(())
}

/// Parses the CSV shape. A header row is required and must
/// name a text column. Columns named id, source, date, and
/// rating are accepted and only id is used in v1, as the item
/// id. Rows without an id take their data row number. Quoted
/// fields may carry commas, doubled quotes, and newlines.
pub fn parse_csv(text: &str) -> Result<Corpus, String> {
    let rows = csv_rows(text);
    if rows.is_empty() {
        return Err("csv file is empty".to_string());
    }
    let header: Vec<String> = rows[0].iter().map(|h| h.trim().to_lowercase()).collect();
    let text_col = header
        .iter()
        .position(|h| h == "text")
        .ok_or("csv header has no text column")?;
    let id_col = header.iter().position(|h| h == "id");
    let mut items = Vec::new();
    for (i, row) in rows.iter().enumerate().skip(1) {
        if row.iter().all(|f| f.trim().is_empty()) {
            continue;
        }
        let body = row.get(text_col).cloned().unwrap_or_default();
        let id = match id_col {
            Some(c) => {
                let v = row.get(c).cloned().unwrap_or_default();
                if v.trim().is_empty() {
                    format!("row-{}", i)
                } else {
                    v
                }
            }
            None => format!("row-{}", i),
        };
        items.push(item_from(id, &body));
    }
    check_unique_ids(&items)?;
    Ok(Corpus {
        items,
        decode_warnings: 0,
    })
}

/// Splits CSV text into rows of fields, honouring quotes.
fn csv_rows(text: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = text.chars().peekable();
    let mut field_started = false;
    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                field.push(ch);
            }
        } else if ch == '"' && !field_started {
            in_quotes = true;
            field_started = true;
        } else if ch == ',' {
            row.push(std::mem::take(&mut field));
            field_started = false;
        } else if ch == '\n' || ch == '\r' {
            if ch == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
            }
            row.push(std::mem::take(&mut field));
            field_started = false;
            rows.push(std::mem::take(&mut row));
        } else {
            field.push(ch);
            field_started = true;
        }
    }
    if field_started || !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

/// Parses the JSONL shape. Every line is one object with a
/// required text field and optional id, source, date, and
/// rating fields. Only id is used in v1. Lines without an id
/// take their line number.
pub fn parse_jsonl(text: &str) -> Result<Corpus, String> {
    let mut items = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let value = crate::json::parse(line)
            .map_err(|e| format!("jsonl line {} does not parse: {e}", i + 1))?;
        let body = value
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("jsonl line {} has no text field", i + 1))?;
        let id = value
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("line-{}", i + 1));
        items.push(item_from(id, body));
    }
    check_unique_ids(&items)?;
    Ok(Corpus {
        items,
        decode_warnings: 0,
    })
}

/// Normalises line endings to LF.
pub fn normalise(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// Loads a corpus from a folder path.
pub fn load_corpus(folder: &Path) -> Result<Corpus, String> {
    let mut names: Vec<String> = Vec::new();
    let entries = std::fs::read_dir(folder)
        .map_err(|e| format!("cannot read folder {}: {e}", folder.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("cannot read folder entry: {e}"))?;
        // Directories are never items, whatever their name.
        if !entry
            .file_type()
            .map(|t| t.is_file())
            .map_err(|e| format!("cannot stat folder entry: {e}"))?
        {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        // The extension match is case insensitive. A folder
        // assembled on a case insensitive filesystem holds
        // ITEM.TXT as readily as item.txt, and silently
        // dropping it would lose feedback without a word.
        let lower_name = name.to_lowercase();
        if lower_name.ends_with(".txt") || lower_name.ends_with(".md") {
            names.push(name);
        }
    }
    names.sort();
    let mut items = Vec::new();
    let mut warnings = 0u64;
    for name in names {
        let path = folder.join(&name);
        let bytes =
            std::fs::read(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
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
    // Two files can share a stem, a.txt and a.md, and so
    // share an item id. The adapters already refuse duplicate
    // ids, because an id is how citations are attributed.
    // The folder shape now refuses them too.
    check_unique_ids(&items)?;
    Ok(Corpus {
        items,
        decode_warnings: warnings,
    })
}

/// Decodes bytes as UTF-8, counting one warning per replacement
/// character introduced by lossy decoding.
pub fn decode(bytes: &[u8]) -> (String, u64) {
    let (text, warnings) = match std::str::from_utf8(bytes) {
        Ok(s) => (s.to_string(), 0),
        Err(_) => {
            let lossy = String::from_utf8_lossy(bytes).to_string();
            let count = lossy.chars().filter(|c| *c == '\u{FFFD}').count() as u64;
            (lossy, count)
        }
    };
    // A leading byte order mark is an artefact of the writing
    // tool, never feedback content. Left in, it corrupts a CSV
    // header's first column name and the first word of a text
    // file. It is stripped without a decode warning, the same
    // way the JSON parser strips it.
    let text = text.strip_prefix('\u{FEFF}').unwrap_or(&text).to_string();
    (text, warnings)
}

#[cfg(test)]
mod tests {
    use super::{decode, load_corpus, normalise, parse_csv, parse_jsonl};

    #[test]
    fn crlf_and_cr_become_lf() {
        assert_eq!(normalise("a\r\nb\rc\nd"), "a\nb\nc\nd");
    }

    #[test]
    fn folder_ingest_case_directories_and_duplicate_stems() {
        let base = std::env::temp_dir().join(format!("earful-corpus-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("archive.txt")).unwrap();
        std::fs::write(base.join("one.txt"), "First item text here.").unwrap();
        std::fs::write(base.join("TWO.TXT"), "Second item text here.").unwrap();
        std::fs::write(base.join("three.Md"), "Third item text here.").unwrap();
        std::fs::write(base.join("skip.csv"), "not,a,text,item").unwrap();
        let corpus = load_corpus(&base).unwrap();
        let ids: Vec<&str> = corpus.items.iter().map(|i| i.id.as_str()).collect();
        assert_eq!(ids, ["TWO", "one", "three"], "sorted, case kept in ids");
        // A second file sharing a stem is a duplicate id and
        // the folder is refused, like the file shapes refuse.
        std::fs::write(base.join("one.md"), "Same stem, other shape.").unwrap();
        let err = load_corpus(&base).unwrap_err();
        assert!(err.contains("duplicate item id one"), "got {err}");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn byte_order_mark_is_stripped_not_counted() {
        // The shape an Excel or Windows tool export takes. The
        // mark must not corrupt the CSV header's first column,
        // the first JSONL line, or a text item's first word.
        let (text, warnings) = decode("\u{FEFF}id,text\n".as_bytes());
        assert_eq!(text, "id,text\n");
        assert_eq!(warnings, 0);
        let corpus = parse_csv(&decode("\u{FEFF}text\nIt crashes daily\n".as_bytes()).0).unwrap();
        assert_eq!(corpus.items[0].text, "It crashes daily");
        let corpus = parse_jsonl(&decode("\u{FEFF}{\"text\": \"One\"}\n".as_bytes()).0).unwrap();
        assert_eq!(corpus.items[0].text, "One");
    }

    #[test]
    fn csv_quotes_ids_and_row_numbers() {
        let text = "id,text,rating\nx1,\"It crashes, badly\",1\n,Plain note,5\n";
        let corpus = parse_csv(text).unwrap();
        assert_eq!(corpus.items.len(), 2);
        assert_eq!(corpus.items[0].id, "x1");
        assert_eq!(corpus.items[0].text, "It crashes, badly");
        assert_eq!(corpus.items[1].id, "row-2");
        assert!(parse_csv("a,b\n1,2\n").is_err());
    }

    #[test]
    fn jsonl_text_required_and_ids_checked() {
        let text = "{\"id\": \"a\", \"text\": \"One\"}\n{\"text\": \"Two\"}\n";
        let corpus = parse_jsonl(text).unwrap();
        assert_eq!(corpus.items[0].id, "a");
        assert_eq!(corpus.items[1].id, "line-2");
        assert!(parse_jsonl("{\"id\": \"a\"}\n").is_err());
        let dup = "{\"id\": \"a\", \"text\": \"One\"}\n{\"id\": \"a\", \"text\": \"Two\"}\n";
        assert!(parse_jsonl(dup).is_err());
    }
}
