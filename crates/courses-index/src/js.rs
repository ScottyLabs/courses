//! JavaScript semantics that Courseweave's scripts rely on, reproduced so the documents match the files they write:
//! whitespace as `\s` and `trim()` see it, `Number()`, `JSON.stringify` of numbers and of objects with integer-like
//! keys, and `localeCompare`.

use std::cmp::Ordering;

use serde::Serialize;
use serde::ser::{SerializeMap, Serializer};

/// The characters JavaScript's `\s` and `String.prototype.trim` treat as whitespace. Unlike Rust's, it has U+FEFF and
/// lacks U+0085.
pub fn is_space(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\u{0B}' | '\u{0C}' | '\r' | ' ' | '\u{A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

/// JavaScript's `\s` as a regex character class.
pub const WS: &str = r"[\t\n\x0B\x0C\r \x{A0}\x{1680}\x{2000}-\x{200A}\x{2028}\x{2029}\x{202F}\x{205F}\x{3000}\x{FEFF}]";

pub fn trim(s: &str) -> &str {
    s.trim_matches(is_space)
}

/// `s.replace(/\s+/g, ' ')`.
pub fn collapse_space(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_space = false;
    for c in s.chars() {
        if is_space(c) {
            if !in_space {
                out.push(' ');
            }
            in_space = true;
        } else {
            out.push(c);
            in_space = false;
        }
    }
    out
}

/// `s.replace(/\s+/g, ' ').trim()`.
pub fn squish(s: &str) -> String {
    trim(&collapse_space(s)).to_owned()
}

/// `Number(s)`: the empty string is 0, anything that isn't a whole numeric literal is NaN.
pub fn number(s: &str) -> f64 {
    let t = trim(s);
    if t.is_empty() {
        return 0.0;
    }
    let radix = match t.get(..2) {
        Some("0x" | "0X") => 16,
        Some("0o" | "0O") => 8,
        Some("0b" | "0B") => 2,
        _ => 10,
    };
    if radix != 10 {
        let digits = &t[2..];
        if digits.is_empty() {
            return f64::NAN;
        }
        let mut value = 0.0;
        for c in digits.chars() {
            match c.to_digit(radix) {
                Some(d) => value = value * radix as f64 + d as f64,
                None => return f64::NAN,
            }
        }
        return value;
    }
    match t {
        "Infinity" | "+Infinity" => return f64::INFINITY,
        "-Infinity" => return f64::NEG_INFINITY,
        _ => {}
    }
    if is_decimal_literal(t) {
        t.parse().unwrap_or(f64::NAN)
    } else {
        f64::NAN
    }
}

/// `[+-]? (digits [. digits?] | . digits) ([eE] [+-]? digits)?`
fn is_decimal_literal(t: &str) -> bool {
    let b = t.as_bytes();
    let mut i = 0;
    if matches!(b.first(), Some(b'+' | b'-')) {
        i += 1;
    }
    let int_start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let mut digits = i - int_start;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let frac_start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        digits += i - frac_start;
    }
    if digits == 0 {
        return false;
    }
    if i < b.len() && matches!(b[i], b'e' | b'E') {
        i += 1;
        if matches!(b.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let exp_start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == exp_start {
            return false;
        }
    }
    i == b.len()
}

/// Serializes a number as `JSON.stringify` does: integers without a fraction, non-finite numbers as `null`.
pub fn ser_num<S: Serializer>(value: &f64, s: S) -> Result<S::Ok, S::Error> {
    let v = *value;
    if !v.is_finite() {
        s.serialize_none()
    } else if v.fract() == 0.0 && v.abs() < 9.007_199_254_740_992e15 {
        s.serialize_i64(v as i64)
    } else {
        s.serialize_f64(v)
    }
}

pub fn ser_opt_num<S: Serializer>(value: &Option<f64>, s: S) -> Result<S::Ok, S::Error> {
    match value {
        Some(v) => ser_num(v, s),
        None => s.serialize_none(),
    }
}

/// Whether JavaScript orders this property key as an array index: before every other key, in numeric order.
pub fn is_array_index(key: &str) -> bool {
    key == "0"
        || (!key.is_empty()
            && !key.starts_with('0')
            && key.len() <= 10
            && key.bytes().all(|b| b.is_ascii_digit())
            && key.parse::<u64>().is_ok_and(|n| n < u32::MAX as u64))
}

/// An object whose keys serialize in JavaScript's property order: array-index keys (such as course ids not starting
/// with 0) ascending, then the other keys in insertion order.
#[derive(Clone, Debug, PartialEq)]
pub struct Object<V>(pub Vec<(String, V)>);

impl<V> Default for Object<V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<V> Object<V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, key: impl Into<String>, value: V) {
        self.0.push((key.into(), value));
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Entries in serialization order.
    pub fn ordered(&self) -> Vec<&(String, V)> {
        let mut indices: Vec<&(String, V)> =
            self.0.iter().filter(|(k, _)| is_array_index(k)).collect();
        indices.sort_by_key(|(k, _)| k.parse::<u64>().unwrap_or(0));
        indices.extend(self.0.iter().filter(|(k, _)| !is_array_index(k)));
        indices
    }
}

impl<V> FromIterator<(String, V)> for Object<V> {
    fn from_iter<I: IntoIterator<Item = (String, V)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<V: Serialize> Serialize for Object<V> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(Some(self.0.len()))?;
        for (k, v) in self.ordered() {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

/// Primary collation weight of a character, approximating ICU's root collation (Node's `localeCompare`): spaces,
/// then punctuation and symbols in CLDR order, then digits, then letters ignoring case.
fn primary(c: char) -> (u8, u32) {
    const PUNCTUATION: &str = "_-,;:!?.'\"()[]{}@*/\\&#%`^+<=>|~$";
    if is_space(c) {
        (0, 0)
    } else if let Some(i) = PUNCTUATION.find(c) {
        (1, i as u32)
    } else if c.is_ascii_digit() {
        (3, c as u32)
    } else if c.is_alphabetic() {
        (4, c.to_lowercase().next().unwrap_or(c) as u32)
    } else if c.is_ascii() {
        (2, c as u32)
    } else {
        (5, c as u32)
    }
}

/// Lowercase before uppercase, as ICU's tertiary level orders them.
fn tertiary(a: &str, b: &str) -> Ordering {
    for (x, y) in a.chars().zip(b.chars()) {
        if x != y {
            match (x.is_lowercase(), y.is_lowercase()) {
                (true, false) => return Ordering::Less,
                (false, true) => return Ordering::Greater,
                _ => {}
            }
        }
    }
    a.cmp(b)
}

/// `a.localeCompare(b)`.
pub fn locale_cmp(a: &str, b: &str) -> Ordering {
    a.chars()
        .map(primary)
        .cmp(b.chars().map(primary))
        .then_with(|| tertiary(a, b))
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Part<'a> {
    // Leading zeros stripped: shorter runs are smaller numbers.
    Number(usize, &'a str),
    Char((u8, u32)),
}

fn parts(s: &str) -> Vec<Part<'_>> {
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(c) = rest.chars().next() {
        if c.is_ascii_digit() {
            let end = rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(rest.len());
            let digits = rest[..end].trim_start_matches('0');
            out.push(Part::Number(digits.len(), digits));
            rest = &rest[end..];
        } else {
            out.push(Part::Char(primary(c)));
            rest = &rest[c.len_utf8()..];
        }
    }
    out
}

/// `a.localeCompare(b, undefined, { numeric: true })`: digit runs compare by value, before letters.
pub fn numeric_cmp(a: &str, b: &str) -> Ordering {
    // Digits sort before letters, so a number against a letter compares by class.
    let key = |p: &Part| match p {
        Part::Number(..) => 3,
        Part::Char((class, _)) => *class,
    };
    let (pa, pb) = (parts(a), parts(b));
    for (x, y) in pa.iter().zip(&pb) {
        let ord = key(x).cmp(&key(y)).then_with(|| x.cmp(y));
        if ord != Ordering::Equal {
            return ord;
        }
    }
    pa.len().cmp(&pb.len()).then_with(|| {
        // Equal apart from case (lowercase first) or leading zeros (equal).
        match a.chars().zip(b.chars()).find(|(x, y)| x != y) {
            Some((x, y)) if x.to_lowercase().eq(y.to_lowercase()) => {
                y.is_lowercase().cmp(&x.is_lowercase())
            }
            _ => Ordering::Equal,
        }
    })
}

/// Days since 1970-01-01 of a proleptic Gregorian date.
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// `Date.parse` of a `YYYY-MM-DD` date (UTC midnight), in milliseconds; `None` where JavaScript gives NaN for the
/// formats the inputs use.
pub fn parse_date(s: &str) -> Option<f64> {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let y: i64 = s[..4].parse().ok()?;
    let m: u32 = s[5..7].parse().ok()?;
    let d: u32 = s[8..].parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    Some(days_from_civil(y, m, d) as f64 * 86_400_000.0)
}

/// `new Date(seconds * 1000).toISOString()`.
pub fn iso_from_unix(seconds: i64) -> String {
    let (y, m, d) = civil_from_days(seconds.div_euclid(86_400));
    let rem = seconds.rem_euclid(86_400);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.000Z",
        rem / 3600,
        rem / 60 % 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_matches_javascript() {
        assert_eq!(trim("\u{FEFF} a b \u{A0}"), "a b");
        assert_eq!(trim("\u{85}x"), "\u{85}x");
        assert_eq!(squish("  a \n\t b  "), "a b");
    }

    #[test]
    fn number_matches_javascript() {
        assert_eq!(number(""), 0.0);
        assert_eq!(number(" 12 "), 12.0);
        assert_eq!(number("12.0"), 12.0);
        assert_eq!(number("-1"), -1.0);
        assert_eq!(number("4.5"), 4.5);
        assert_eq!(number(".5"), 0.5);
        assert_eq!(number("0x10"), 16.0);
        assert_eq!(number("1e3"), 1000.0);
        assert!(number("VAR").is_nan());
        assert!(number("inf").is_nan());
        assert!(number("1-3").is_nan());
        assert!(number("Infinity").is_infinite());
    }

    #[test]
    fn numbers_serialize_like_json_stringify() {
        #[derive(Serialize)]
        struct N {
            #[serde(serialize_with = "ser_num")]
            a: f64,
            #[serde(serialize_with = "ser_num")]
            b: f64,
            #[serde(serialize_with = "ser_opt_num")]
            c: Option<f64>,
            #[serde(serialize_with = "ser_num")]
            d: f64,
        }
        let json = serde_json::to_string(&N {
            a: 12.0,
            b: 4.5,
            c: None,
            d: f64::NAN,
        })
        .unwrap();
        assert_eq!(json, r#"{"a":12,"b":4.5,"c":null,"d":null}"#);
    }

    #[test]
    fn integer_keys_come_first() {
        let o: Object<u8> = [
            ("09931", 1),
            ("15122", 2),
            ("02100", 3),
            ("10202", 4),
            ("x", 5),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect();
        assert_eq!(
            serde_json::to_string(&o).unwrap(),
            r#"{"10202":4,"15122":2,"09931":1,"02100":3,"x":5}"#
        );
        assert!(is_array_index("0") && is_array_index("4294967294"));
        assert!(!is_array_index("4294967295") && !is_array_index("01") && !is_array_index("-1"));
    }

    #[test]
    fn collation_approximates_icu() {
        let mut names = vec![
            "Garnhart-Bushakra",
            "Garnhart-bushakra",
            "Aaron Johnson",
            "aaron Blum",
            "O'Connor",
            "Oconnor",
        ];
        names.sort_by(|a, b| locale_cmp(a, b));
        assert_eq!(
            names,
            [
                "aaron Blum",
                "Aaron Johnson",
                "Garnhart-bushakra",
                "Garnhart-Bushakra",
                "O'Connor",
                "Oconnor"
            ]
        );
        let mut sections = vec!["10", "A", "2", "", "A1", "1", "AA", "W"];
        sections.sort_by(|a, b| numeric_cmp(a, b));
        assert_eq!(sections, ["", "1", "2", "10", "A", "A1", "AA", "W"]);
    }

    #[test]
    fn dates() {
        assert_eq!(parse_date("1970-01-02"), Some(86_400_000.0));
        assert_eq!(parse_date("2026-08-24"), Some(1_787_529_600_000.0));
        assert_eq!(parse_date("2026-8-24"), None);
        assert_eq!(iso_from_unix(1_787_529_601), "2026-08-24T00:00:01.000Z");
        assert_eq!(civil_from_days(days_from_civil(2024, 2, 29)), (2024, 2, 29));
    }
}
