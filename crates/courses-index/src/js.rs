//! JavaScript semantics that Courseweave's scripts rely on, reproduced so the documents match the files they write.
//! Whitespace, `Number()`, number formatting and `localeCompare` are shared with the FCE and program builders
//! ([`crate::fce::js`]); this module adds object key order, numeric collation and dates.

use std::cmp::Ordering;
use std::fmt;
use std::marker::PhantomData;
use std::sync::LazyLock;

use icu_collator::options::CollatorOptions;
use icu_collator::preferences::CollationNumericOrdering;
use icu_collator::{Collator, CollatorBorrowed, CollatorPreferences};
use serde::de::{Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};

use crate::fce::js::Num;
pub use crate::fce::js::{
    SPACE_CLASS as WS, collapse_spaces as collapse_space, is_space, locale_compare as locale_cmp,
    string_to_number as number, trim,
};

/// `s.replace(/\s+/g, ' ').trim()`.
pub fn squish(s: &str) -> String {
    trim(&collapse_space(s)).to_owned()
}

/// Serializes a number as `JSON.stringify` does: integers without a fraction, non-finite numbers as `null`.
pub fn ser_num<S: Serializer>(value: &f64, s: S) -> Result<S::Ok, S::Error> {
    Num(*value).serialize(s)
}

pub fn ser_opt_num<S: Serializer>(value: &Option<f64>, s: S) -> Result<S::Ok, S::Error> {
    value.map(Num).serialize(s)
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

impl<'de, V: Deserialize<'de>> Deserialize<'de> for Object<V> {
    /// Keeps the keys in document order, as `JSON.parse` does: a repeated key keeps its first position and its last
    /// value.
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Entries<V>(PhantomData<V>);
        impl<'de, V: Deserialize<'de>> Visitor<'de> for Entries<V> {
            type Value = Object<V>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a JSON object")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Object<V>, A::Error> {
                let mut out: Vec<(String, V)> = Vec::new();
                while let Some((k, v)) = map.next_entry::<String, V>()? {
                    match out.iter_mut().find(|(x, _)| *x == k) {
                        Some(entry) => entry.1 = v,
                        None => out.push((k, v)),
                    }
                }
                Ok(Object(out))
            }
        }
        d.deserialize_map(Entries(PhantomData))
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

static NUMERIC: LazyLock<CollatorBorrowed<'static>> = LazyLock::new(|| {
    let mut prefs = CollatorPreferences::default();
    prefs.numeric_ordering = Some(CollationNumericOrdering::True);
    Collator::try_new(prefs, CollatorOptions::default()).expect("the root collation is compiled in")
});

/// `a.localeCompare(b, undefined, { numeric: true })`: digit runs compare by value.
pub fn numeric_cmp(a: &str, b: &str) -> Ordering {
    if a == b {
        Ordering::Equal
    } else {
        NUMERIC.compare(a, b)
    }
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
    iso_from_unix_millis(seconds * 1000)
}

/// `new Date(millis).toISOString()`.
pub fn iso_from_unix_millis(millis: i64) -> String {
    let seconds = millis.div_euclid(1000);
    let (y, m, d) = civil_from_days(seconds.div_euclid(86_400));
    let rem = seconds.rem_euclid(86_400);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.{:03}Z",
        rem / 3600,
        rem / 60 % 60,
        rem % 60,
        millis.rem_euclid(1000)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let parsed: Object<u8> =
            serde_json::from_str(r#"{"b":1,"10":2,"a":3,"2":4,"b":5}"#).unwrap();
        assert_eq!(
            serde_json::to_string(&parsed).unwrap(),
            r#"{"2":4,"10":2,"b":5,"a":3}"#
        );
        assert!(is_array_index("0") && is_array_index("4294967294"));
        assert!(!is_array_index("4294967295") && !is_array_index("01") && !is_array_index("-1"));
    }

    #[test]
    fn collation_matches_icu() {
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
        assert_eq!(
            iso_from_unix_millis(1_787_529_601_042),
            "2026-08-24T00:00:01.042Z"
        );
        assert_eq!(civil_from_days(days_from_civil(2024, 2, 29)), (2024, 2, 29));
    }
}
