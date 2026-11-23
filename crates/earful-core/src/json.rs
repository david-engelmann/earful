//! A minimal JSON parser and a canonical JSON writer.
//!
//! The parser covers the full JSON grammar and keeps numbers as
//! their raw source text so nothing is ever reformatted through a
//! float. The canonical writer sorts object keys, uses two space
//! indentation, emits non ASCII text as raw UTF-8, and escapes only
//! the quote, the backslash, and control characters below U+0020,
//! the named ones with their short forms and the rest as unicode
//! escapes with lowercase hex. Canonical output ends with exactly
//! one trailing newline when written through `write_canonical`.

/// A parsed JSON value. Numbers keep their raw source text.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    Arr(Vec<Value>),
    Obj(Vec<(String, Value)>),
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Obj(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_arr(&self) -> Option<&[Value]> {
        match self {
            Value::Arr(items) => Some(items),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Num(raw) => raw.parse().ok(),
            _ => None,
        }
    }
}

/// Parses a complete JSON document. Errors carry a byte position.
pub fn parse(input: &str) -> Result<Value, String> {
    let bytes = input.as_bytes();
    let mut pos = 0usize;
    let value = parse_value(bytes, &mut pos)?;
    skip_ws(bytes, &mut pos);
    if pos != bytes.len() {
        return Err(format!("trailing characters at byte {pos}"));
    }
    Ok(value)
}

fn skip_ws(b: &[u8], pos: &mut usize) {
    while *pos < b.len() && matches!(b[*pos], b' ' | b'\t' | b'\n' | b'\r') {
        *pos += 1;
    }
}

fn parse_value(b: &[u8], pos: &mut usize) -> Result<Value, String> {
    skip_ws(b, pos);
    if *pos >= b.len() {
        return Err("unexpected end of input".to_string());
    }
    match b[*pos] {
        b'{' => parse_obj(b, pos),
        b'[' => parse_arr(b, pos),
        b'"' => Ok(Value::Str(parse_string(b, pos)?)),
        b't' => parse_lit(b, pos, "true", Value::Bool(true)),
        b'f' => parse_lit(b, pos, "false", Value::Bool(false)),
        b'n' => parse_lit(b, pos, "null", Value::Null),
        b'-' | b'0'..=b'9' => parse_num(b, pos),
        other => Err(format!("unexpected byte {other} at byte {pos}", pos = *pos)),
    }
}

fn parse_lit(b: &[u8], pos: &mut usize, lit: &str, val: Value) -> Result<Value, String> {
    if b.len() - *pos >= lit.len() && &b[*pos..*pos + lit.len()] == lit.as_bytes() {
        *pos += lit.len();
        Ok(val)
    } else {
        Err(format!("invalid literal at byte {}", *pos))
    }
}

fn parse_num(b: &[u8], pos: &mut usize) -> Result<Value, String> {
    let start = *pos;
    if *pos < b.len() && b[*pos] == b'-' {
        *pos += 1;
    }
    while *pos < b.len() && matches!(b[*pos], b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-') {
        *pos += 1;
    }
    if *pos == start {
        return Err(format!("invalid number at byte {start}"));
    }
    let raw = std::str::from_utf8(&b[start..*pos]).map_err(|_| "bad number".to_string())?;
    Ok(Value::Num(raw.to_string()))
}

fn parse_string(b: &[u8], pos: &mut usize) -> Result<String, String> {
    if b[*pos] != b'"' {
        return Err(format!("expected string at byte {}", *pos));
    }
    *pos += 1;
    let mut out = String::new();
    loop {
        if *pos >= b.len() {
            return Err("unterminated string".to_string());
        }
        match b[*pos] {
            b'"' => {
                *pos += 1;
                return Ok(out);
            }
            b'\\' => {
                *pos += 1;
                if *pos >= b.len() {
                    return Err("unterminated escape".to_string());
                }
                match b[*pos] {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'/' => out.push('/'),
                    b'b' => out.push('\u{0008}'),
                    b'f' => out.push('\u{000C}'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    b'u' => {
                        let hi = parse_hex4(b, pos)?;
                        if (0xD800..0xDC00).contains(&hi) {
                            if *pos + 2 < b.len() && b[*pos + 1] == b'\\' && b[*pos + 2] == b'u' {
                                *pos += 2;
                                let lo = parse_hex4(b, pos)?;
                                let c = 0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00);
                                out.push(char::from_u32(c).unwrap_or('\u{FFFD}'));
                            } else {
                                out.push('\u{FFFD}');
                            }
                        } else {
                            out.push(char::from_u32(hi).unwrap_or('\u{FFFD}'));
                        }
                    }
                    other => return Err(format!("bad escape {other} at byte {}", *pos)),
                }
                *pos += 1;
            }
            _ => {
                let s = std::str::from_utf8(&b[*pos..]).map_err(|_| "bad utf8".to_string())?;
                let ch = s.chars().next().ok_or("bad utf8")?;
                out.push(ch);
                *pos += ch.len_utf8();
            }
        }
    }
}

/// Parses the four hex digits of a unicode escape. On entry `pos`
/// points at the `u`; on exit it points at the last hex digit.
fn parse_hex4(b: &[u8], pos: &mut usize) -> Result<u32, String> {
    if *pos + 4 >= b.len() {
        return Err("truncated unicode escape".to_string());
    }
    let s = std::str::from_utf8(&b[*pos + 1..*pos + 5]).map_err(|_| "bad hex".to_string())?;
    let v = u32::from_str_radix(s, 16).map_err(|_| "bad hex".to_string())?;
    *pos += 4;
    Ok(v)
}

fn parse_obj(b: &[u8], pos: &mut usize) -> Result<Value, String> {
    *pos += 1;
    let mut pairs = Vec::new();
    skip_ws(b, pos);
    if *pos < b.len() && b[*pos] == b'}' {
        *pos += 1;
        return Ok(Value::Obj(pairs));
    }
    loop {
        skip_ws(b, pos);
        let key = parse_string(b, pos)?;
        skip_ws(b, pos);
        if *pos >= b.len() || b[*pos] != b':' {
            return Err(format!("expected colon at byte {}", *pos));
        }
        *pos += 1;
        let val = parse_value(b, pos)?;
        pairs.push((key, val));
        skip_ws(b, pos);
        match b.get(*pos) {
            Some(b',') => *pos += 1,
            Some(b'}') => {
                *pos += 1;
                return Ok(Value::Obj(pairs));
            }
            _ => return Err(format!("expected comma or brace at byte {}", *pos)),
        }
    }
}

fn parse_arr(b: &[u8], pos: &mut usize) -> Result<Value, String> {
    *pos += 1;
    let mut items = Vec::new();
    skip_ws(b, pos);
    if *pos < b.len() && b[*pos] == b']' {
        *pos += 1;
        return Ok(Value::Arr(items));
    }
    loop {
        let val = parse_value(b, pos)?;
        items.push(val);
        skip_ws(b, pos);
        match b.get(*pos) {
            Some(b',') => *pos += 1,
            Some(b']') => {
                *pos += 1;
                return Ok(Value::Arr(items));
            }
            _ => return Err(format!("expected comma or bracket at byte {}", *pos)),
        }
    }
}

/// Writes a value in canonical form, without a trailing newline.
pub fn write_canonical(value: &Value) -> String {
    let mut out = String::new();
    write_value(value, 0, &mut out);
    out.push('\n');
    out
}

fn escape_into(s: &str, out: &mut String) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn write_value(value: &Value, indent: usize, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Num(raw) => out.push_str(raw),
        Value::Str(s) => escape_into(s, out),
        Value::Arr(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                out.push_str(&"  ".repeat(indent + 1));
                write_value(item, indent + 1, out);
                if i + 1 < items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&"  ".repeat(indent));
            out.push(']');
        }
        Value::Obj(pairs) => {
            if pairs.is_empty() {
                out.push_str("{}");
                return;
            }
            let mut sorted: Vec<&(String, Value)> = pairs.iter().collect();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            out.push_str("{\n");
            for (i, (key, val)) in sorted.iter().enumerate() {
                out.push_str(&"  ".repeat(indent + 1));
                escape_into(key, out);
                out.push_str(": ");
                write_value(val, indent + 1, out);
                if i + 1 < sorted.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&"  ".repeat(indent));
            out.push('}');
        }
    }
}

/// Builds an object value from key and value pairs.
pub fn obj(pairs: Vec<(&str, Value)>) -> Value {
    Value::Obj(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}

/// Builds a string value.
pub fn s(text: &str) -> Value {
    Value::Str(text.to_string())
}

/// Writes a value on a single line with sorted object keys and
/// the same escaping rules as the canonical writer. Used for
/// JSONL records, labelling files, and worksheets.
pub fn write_compact(value: &Value) -> String {
    let mut out = String::new();
    write_compact_into(value, &mut out);
    out
}

fn write_compact_into(value: &Value, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Num(raw) => out.push_str(raw),
        Value::Str(s) => escape_into_compact(s, out),
        Value::Arr(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_compact_into(item, out);
            }
            out.push(']');
        }
        Value::Obj(pairs) => {
            let mut sorted: Vec<&(String, Value)> = pairs.iter().collect();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            out.push('{');
            for (i, (key, val)) in sorted.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                escape_into_compact(key, out);
                out.push(':');
                write_compact_into(val, out);
            }
            out.push('}');
        }
    }
}

fn escape_into_compact(s: &str, out: &mut String) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Builds an unsigned integer value.
pub fn n(value: u64) -> Value {
    Value::Num(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::{Value, obj, parse, s, write_canonical};

    #[test]
    fn canonical_sorts_keys() {
        let v = obj(vec![("b", s("2")), ("a", s("1"))]);
        let text = write_canonical(&v);
        assert!(text.find("\"a\"").unwrap() < text.find("\"b\"").unwrap());
        assert!(text.ends_with("}\n"));
    }

    #[test]
    fn roundtrip_nested() {
        let src = r#"{"x": [1, 2.5, "a\nb", true, null], "y": {"z": "é"}}"#;
        let v = parse(src).unwrap();
        let text = write_canonical(&v);
        let v2 = parse(&text).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn raw_number_preserved() {
        let v = parse("{\"share\": 0.2540}").unwrap();
        match v.get("share").unwrap() {
            Value::Num(raw) => assert_eq!(raw, "0.2540"),
            other => panic!("expected number, got {other:?}"),
        }
    }

    #[test]
    fn unicode_escape_and_surrogate_pair() {
        let v = parse(r#""Aé😀""#).unwrap();
        assert_eq!(v, Value::Str("Aé😀".to_string()));
    }
}
