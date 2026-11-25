//! Text handling. Lowercasing, sentence spans, and the whole
//! token phrase matching the determinism boundary rests on.
//!
//! Matching runs on a lowercased copy of the item, produced with
//! the standard library Unicode lowercase mapping. A phrase hits
//! when an occurrence of it in the lowercased text is bounded on
//! both sides by a character that is not alphanumeric, or by the
//! edge of the text. That one rule is what keeps the phrase ads
//! from firing inside the word reads.

/// Lowercases text with the standard Unicode mapping.
pub fn lower(text: &str) -> String {
    text.chars().flat_map(|c| c.to_lowercase()).collect()
}

/// Whether a phrase occurs in already lowercased text under the
/// whole token boundary rule. The phrase must already be
/// lowercased as well.
pub fn phrase_hit(phrase: &str, lower_text: &str) -> bool {
    if phrase.is_empty() {
        return false;
    }
    let bytes = lower_text.as_bytes();
    let mut search_from = 0usize;
    while let Some(rel) = lower_text[search_from..].find(phrase) {
        let start = search_from + rel;
        let end = start + phrase.len();
        let before_ok = start == 0
            || !lower_text[..start]
                .chars()
                .next_back()
                .map(|c| c.is_alphanumeric())
                .unwrap_or(false);
        let after_ok = end == bytes.len()
            || !lower_text[end..]
                .chars()
                .next()
                .map(|c| c.is_alphanumeric())
                .unwrap_or(false);
        if before_ok && after_ok {
            return true;
        }
        // Advance past this occurrence's first character, staying
        // on a char boundary for multi byte text.
        search_from = start
            + lower_text[start..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
    }
    false
}

/// Splits text into verbatim sentence spans. A span ends after a
/// run of sentence punctuation, full stop, exclamation, question
/// mark, or newline. A full stop that ends an abbreviation is
/// not a boundary, so a citation can no longer end mid
/// abbreviation, the Phase 1 defect this rule closes. Spans
/// whose stripped length is under 15 characters are merged into
/// the following span, which repairs splits after short
/// headings.
pub fn sentences(text: &str) -> Vec<String> {
    let mut spans: Vec<String> = Vec::new();
    let mut start = 0usize;
    let mut chars = text.char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        if matches!(ch, '.' | '!' | '?' | '\n') && !(ch == '.' && is_abbreviation_dot(text, i)) {
            // Consume the whole punctuation run.
            let mut end = i + ch.len_utf8();
            while let Some(&(j, c)) = chars.peek() {
                if matches!(c, '.' | '!' | '?' | '\n') {
                    chars.next();
                    end = j + c.len_utf8();
                } else {
                    break;
                }
            }
            spans.push(text[start..end].to_string());
            start = end;
        }
    }
    if start < text.len() {
        spans.push(text[start..].to_string());
    }
    let mut merged: Vec<String> = Vec::new();
    for span in spans {
        if let Some(last) = merged.last_mut()
            && last.trim().chars().count() < 15
        {
            last.push_str(&span);
            continue;
        }
        merged.push(span);
    }
    merged
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .collect()
}

/// Whether the full stop at byte index dot in text ends an
/// abbreviation rather than a sentence. The token before the
/// stop is the run of letters and full stops ending there.
/// A single letter is an initial and counts. A small set of
/// common abbreviations counts. The trade is deliberate, a
/// sentence that ends in an abbreviation merges with the next
/// one, so a citation may run long but never truncates.
fn is_abbreviation_dot(text: &str, dot: usize) -> bool {
    const ABBREVIATIONS: [&str; 15] = [
        "e.g", "i.e", "etc", "vs", "mr", "mrs", "dr", "st", "approx", "fig", "no", "a.m", "p.m",
        "u.s", "u.k",
    ];
    let before = &text[..dot];
    let token: String = before
        .chars()
        .rev()
        .take_while(|c| c.is_alphabetic() || *c == '.')
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    let token = token.to_lowercase();
    token.chars().count() == 1 || ABBREVIATIONS.contains(&token.as_str())
}

/// Finds the citation span for a theme in one item. Phrases are
/// tried in taxonomy order. For the first phrase that hits the
/// item, the first sentence span whose lowercased form the phrase
/// also hits is the citation. Returns None when no phrase hits.
pub fn citation_span(item_text: &str, phrases: &[String]) -> Option<String> {
    let lower_text = lower(item_text);
    let spans = sentences(item_text);
    let lower_spans: Vec<String> = spans.iter().map(|s| lower(s)).collect();
    for phrase in phrases {
        let lower_phrase = lower(phrase);
        if phrase_hit(&lower_phrase, &lower_text) {
            for (span, lower_span) in spans.iter().zip(lower_spans.iter()) {
                if phrase_hit(&lower_phrase, lower_span) {
                    return Some(span.trim().to_string());
                }
            }
            return None;
        }
    }
    None
}

/// Whether any phrase of a theme hits the item.
pub fn theme_hits(item_lower: &str, phrases: &[String]) -> bool {
    phrases.iter().any(|p| phrase_hit(&lower(p), item_lower))
}

#[cfg(test)]
mod tests {
    use super::{citation_span, lower, phrase_hit, sentences};

    #[test]
    fn ads_does_not_fire_inside_reads() {
        assert!(!phrase_hit("ads", "he reads the threads on his headset"));
        assert!(phrase_hit("ads", "too many ads since the update"));
    }

    #[test]
    fn import_does_not_fire_inside_important() {
        assert!(!phrase_hit("import", "this is important to me"));
        assert!(phrase_hit("import", "i cannot import my data"));
    }

    #[test]
    fn apostrophe_phrase_hits() {
        assert!(phrase_hit("don't work", &lower("It don't work at all")));
        assert!(phrase_hit("doesn't work", &lower("It doesn't work.")));
    }

    #[test]
    fn sentences_split_and_keep_punctuation() {
        let spans = sentences("It crashes daily. Please fix it now! Thanks.");
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0], "It crashes daily.");
    }

    #[test]
    fn short_heading_glues_forward() {
        let spans = sentences("Great app. It works well and I use it daily.");
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn citation_is_first_matching_sentence() {
        let text = "I love the design of this program. It crashes every morning though.";
        let phrases = vec!["crash".to_string(), "crashes".to_string()];
        assert_eq!(
            citation_span(text, &phrases).unwrap(),
            "It crashes every morning though."
        );
    }

    #[test]
    fn abbreviation_is_not_a_sentence_boundary() {
        let text = "Also the option to import a list of bookmarks e.g. from a simple properties file would be useful.";
        let spans = sentences(text);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0], text);
    }

    #[test]
    fn citation_never_ends_mid_abbreviation() {
        // The Phase 1 item-0001 defect, verbatim. The citation
        // for the import phrase used to end at "e.".
        let text = "Great app! The new version now works on my Bravia Android TV which is great as it's right by my rooftop aerial cable. The scan feature would be useful...any ETA on when this will be available? Also the option to import a list of bookmarks e.g. from a simple properties file would be useful.";
        let phrases = vec!["import".to_string()];
        let span = citation_span(text, &phrases).unwrap();
        assert_eq!(
            span,
            "Also the option to import a list of bookmarks e.g. from a simple properties file would be useful."
        );
    }
}
