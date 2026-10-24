//! The JavaScript semantics Courseweave's build scripts rely on, so the ported builders write the same JSON: number
//! parsing, formatting and rounding, regular-expression whitespace, and `String.prototype.localeCompare`.
//!
//! Shared by [`crate::fce`] and [`crate::programs`].

use std::cmp::Ordering;
use std::sync::LazyLock;

use icu_collator::{Collator, CollatorBorrowed};
use serde::{Serialize, Serializer};

/// `\s` in a JavaScript regular expression, which is also what `String.prototype.trim` removes.
pub fn is_space(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\u{B}' | '\u{C}' | '\r' | ' ' | '\u{A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

/// The `\s` class as regular-expression source, for patterns that must match JavaScript's.
pub const SPACE_CLASS: &str = r"[\t\n\x0B\x0C\r \x{A0}\x{1680}\x{2000}-\x{200A}\x{2028}\x{2029}\x{202F}\x{205F}\x{3000}\x{FEFF}]";

/// `s.trim()`.
pub fn trim(s: &str) -> &str {
    s.trim_matches(is_space)
}

/// `s.replace(/\s+/g, ' ')`.
pub fn collapse_spaces(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_run = false;
    for c in s.chars() {
        if is_space(c) {
            if !in_run {
                out.push(' ');
                in_run = true;
            }
        } else {
            out.push(c);
            in_run = false;
        }
    }
    out
}

/// `s.split(/\s+/)`: leading or trailing whitespace gives an empty first or last item, and `""` gives `[""]`.
pub fn split_spaces(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut in_run = false;
    for (i, c) in s.char_indices() {
        if is_space(c) {
            if !in_run {
                out.push(&s[start..i]);
                in_run = true;
            }
        } else if in_run {
            start = i;
            in_run = false;
        }
    }
    out.push(if in_run { "" } else { &s[start..] });
    out
}

/// `s.padStart(width, '0')` for ASCII strings.
pub fn pad_zero(s: &str, width: usize) -> String {
    format!("{s:0>width$}")
}

/// `Math.round(x)`: halves round toward positive infinity.
pub fn round(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }
    let floor = x.floor();
    if x - floor >= 0.5 { floor + 1.0 } else { floor }
}

/// Courseweave's `round(n, digits)`: `Math.round(n * 10 ** digits) / 10 ** digits`, keeping null.
pub fn round_to(x: Option<f64>, digits: i32) -> Option<f64> {
    let scale = 10f64.powi(digits);
    x.map(|x| round(x * scale) / scale)
}

/// `Math.max(a, b)`: NaN when either is.
pub fn max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.max(b)
    }
}

/// The comparator `a - b` as an ordering. JavaScript's sort treats a NaN result as equal.
pub fn cmp_num(a: f64, b: f64) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

/// `Number(s)` for a string: whitespace-trimmed decimal, `0x`/`0o`/`0b` integers, `Infinity`; NaN otherwise.
pub fn string_to_number(s: &str) -> f64 {
    let t = trim(s);
    if t.is_empty() {
        return 0.0;
    }
    match t {
        "Infinity" | "+Infinity" => return f64::INFINITY,
        "-Infinity" => return f64::NEG_INFINITY,
        _ => {}
    }
    for (prefix, radix) in [
        ("0x", 16),
        ("0X", 16),
        ("0o", 8),
        ("0O", 8),
        ("0b", 2),
        ("0B", 2),
    ] {
        if let Some(digits) = t.strip_prefix(prefix) {
            if digits.is_empty() {
                return f64::NAN;
            }
            let mut value = 0.0;
            for c in digits.chars() {
                match c.to_digit(radix) {
                    Some(d) => value = value * f64::from(radix) + f64::from(d),
                    None => return f64::NAN,
                }
            }
            return value;
        }
    }
    if is_decimal_literal(t) {
        t.parse().unwrap_or(f64::NAN)
    } else {
        f64::NAN
    }
}

/// `[+-]? (digits [. digits?] | . digits) ([eE] [+-]? digits)?`, which Rust's float parser also accepts, without
/// the `inf` and `nan` spellings JavaScript doesn't.
fn is_decimal_literal(s: &str) -> bool {
    let b = s.as_bytes();
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

/// `String(x)` for a number.
pub fn number_to_string(x: f64) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Infinity" } else { "-Infinity" }.into();
    }
    if x == 0.0 {
        return "0".into();
    }
    let abs = x.abs();
    if (1e-6..1e21).contains(&abs) {
        // Rust's Display is the shortest round-trip decimal without an exponent, as JavaScript writes this range.
        format!("{x}")
    } else {
        let s = format!("{x:e}");
        match s.split_once('e') {
            Some((mantissa, exp)) if !exp.starts_with('-') => format!("{mantissa}e+{exp}"),
            _ => s,
        }
    }
}

/// A number serialized as `JSON.stringify` writes it: integral values without a fraction, NaN and infinities as
/// `null`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Num(pub f64);

impl Serialize for Num {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let x = self.0;
        if !x.is_finite() {
            s.serialize_none()
        } else if x.fract() == 0.0 && x.abs() < 9_007_199_254_740_992.0 {
            #[expect(clippy::cast_possible_truncation, reason = "integral and within 2^53")]
            s.serialize_i64(x as i64)
        } else {
            s.serialize_f64(x)
        }
    }
}

/// `Option<f64>` with `None` as `null`.
pub fn opt(x: Option<f64>) -> Option<Num> {
    x.map(Num)
}

static COLLATOR: LazyLock<CollatorBorrowed<'static>> = LazyLock::new(|| {
    Collator::try_new(Default::default(), Default::default())
        .expect("the root collation is compiled in")
});

/// `a.localeCompare(b)` in Node's default English locale: Unicode root collation at tertiary strength.
pub fn locale_compare(a: &str, b: &str) -> Ordering {
    if a == b {
        Ordering::Equal
    } else {
        COLLATOR.compare(a, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_like_math_round() {
        assert_eq!(round(2.5), 3.0);
        assert_eq!(round(-2.5), -2.0);
        assert_eq!(round(0.499_999_999_999_999_94), 0.0);
        assert_eq!(round_to(Some(4.565), 2), Some(4.57));
        assert_eq!(round_to(Some(-0.25), 1), Some(-0.2));
        assert_eq!(round_to(None, 2), None);
    }

    #[test]
    fn parses_numbers_like_number() {
        assert_eq!(string_to_number(" 4.5 "), 4.5);
        assert_eq!(string_to_number(""), 0.0);
        assert_eq!(string_to_number("0x10"), 16.0);
        assert_eq!(string_to_number(".5"), 0.5);
        assert_eq!(string_to_number("5."), 5.0);
        assert_eq!(string_to_number("1e3"), 1000.0);
        assert!(string_to_number("inf").is_nan());
        assert!(string_to_number("NaN").is_nan());
        assert!(string_to_number("1_000").is_nan());
        assert!(string_to_number("e5").is_nan());
        assert_eq!(string_to_number("-Infinity"), f64::NEG_INFINITY);
    }

    #[test]
    fn formats_numbers_like_string() {
        assert_eq!(number_to_string(22.0), "22");
        assert_eq!(number_to_string(-5.0), "-5");
        assert_eq!(number_to_string(0.1), "0.1");
        assert_eq!(number_to_string(1e-7), "1e-7");
        assert_eq!(number_to_string(1e21), "1e+21");
        assert_eq!(serde_json::to_string(&Num(4.0)).unwrap(), "4");
        assert_eq!(serde_json::to_string(&Num(-0.0)).unwrap(), "0");
        assert_eq!(serde_json::to_string(&Num(4.57)).unwrap(), "4.57");
        assert_eq!(serde_json::to_string(&Num(f64::NAN)).unwrap(), "null");
    }

    #[test]
    fn splits_whitespace_like_regexes() {
        assert_eq!(split_spaces("a  b"), ["a", "b"]);
        assert_eq!(split_spaces(" a"), ["", "a"]);
        assert_eq!(split_spaces(""), [""]);
        assert_eq!(collapse_spaces("a \u{A0}\tb"), "a b");
        assert_eq!(trim("\u{FEFF} x \n"), "x");
    }

    #[test]
    fn compares_like_locale_compare() {
        // Case is a tertiary difference: letters sort alphabetically first, lowercase before uppercase on ties.
        assert_eq!(locale_compare("abc", "ABD"), Ordering::Less);
        assert_eq!(
            locale_compare("ABASCAL, Brooke", "ABASCAL, BROOKE"),
            Ordering::Less
        );
        // Punctuation sorts before digits, digits before letters.
        assert_eq!(
            locale_compare("(Music) Solfege", "1st Grad Recital"),
            Ordering::Less
        );
        assert_eq!(locale_compare("1st", "Art"), Ordering::Less);
        assert_eq!(
            locale_compare("Economics - A", "Economics (SHS)"),
            Ordering::Less
        );
        assert_eq!(locale_compare("Martinez", "Mart\u{ed}nez"), Ordering::Less);
        assert_eq!(locale_compare("Mart\u{ed}nez", "Martinf"), Ordering::Less);
    }
}
