//! Helpers shared by the source adapters (Courseweave's `scripts/lib/common.ts`).

use std::collections::HashMap;
use std::hash::Hash;

use crate::js;
use crate::model::TermMeta;

const DAY_NAMES: [&str; 7] = [
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
];

fn day_letter(c: char) -> Option<u8> {
    match c {
        'M' => Some(0),
        'T' => Some(1),
        'W' => Some(2),
        'R' => Some(3),
        'F' => Some(4),
        'S' => Some(5),
        'U' => Some(6),
        _ => None,
    }
}

pub fn day_name(name: &str) -> Option<u8> {
    DAY_NAMES.iter().position(|d| *d == name).map(|i| i as u8)
}

/// `MWF`, `TR` (SOC letters) or a weekday name.
pub fn parse_days(s: &str) -> Vec<u8> {
    let t = js::trim(s);
    if t.is_empty() || t.eq_ignore_ascii_case("TBA") {
        return Vec::new();
    }
    if let Some(d) = day_name(&t.to_lowercase()) {
        return vec![d];
    }
    t.to_uppercase().chars().filter_map(day_letter).collect()
}

/// `09:30AM`, `9:30 AM`, `07:00 P.M.` to minutes after midnight.
pub fn parse_time(s: &str) -> Option<i32> {
    clock(js::trim(s), true)
}

/// `hh:mm` then optional spaces and `AM`/`PM`, with dots after the letters when `dots`.
pub fn clock(t: &str, dots: bool) -> Option<i32> {
    let (h, rest) = t.split_once(':')?;
    if h.is_empty() || h.len() > 2 || !h.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let (m, rest) = rest.split_at_checked(2)?;
    if !m.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let rest = rest.trim_start_matches(js::is_space);
    let mut chars = rest.chars();
    let pm = match chars.next()? {
        'a' | 'A' => false,
        'p' | 'P' => true,
        _ => return None,
    };
    let mut rest = chars.as_str();
    if dots {
        rest = rest.strip_prefix('.').unwrap_or(rest);
    }
    let mut chars = rest.chars();
    if !matches!(chars.next()?, 'm' | 'M') {
        return None;
    }
    let mut rest = chars.as_str();
    if dots {
        rest = rest.strip_prefix('.').unwrap_or(rest);
    }
    if !rest.is_empty() {
        return None;
    }
    let mut hour: i32 = h.parse().ok()?;
    hour %= 12;
    if pm {
        hour += 12;
    }
    Some(hour * 60 + m.parse::<i32>().ok()?)
}

/// A building or room, or `None` for placeholders.
pub fn location(s: &str) -> Option<String> {
    let t = js::trim(s);
    match t.to_uppercase().as_str() {
        "" | "TBA" | "TBD" | "DNM" => None,
        _ => Some(t.to_owned()),
    }
}

pub fn clean_title(s: &str) -> String {
    let mut t = s;
    // `/\s*Click to Print Page\s*$/i`
    let trimmed = t.trim_end_matches(js::is_space);
    const SUFFIX: &str = "click to print page";
    if trimmed.len() >= SUFFIX.len()
        && trimmed.is_char_boundary(trimmed.len() - SUFFIX.len())
        && trimmed[trimmed.len() - SUFFIX.len()..].eq_ignore_ascii_case(SUFFIX)
    {
        t = trimmed[..trimmed.len() - SUFFIX.len()].trim_end_matches(js::is_space);
    }
    js::squish(t)
}

pub fn split_instructors(s: &str) -> Vec<String> {
    s.split(';')
        .map(|x| js::collapse_space(js::trim(x)))
        .filter(|x| {
            !x.is_empty()
                && !["TBA", "Instructor TBA", "Staff"]
                    .iter()
                    .any(|p| p.eq_ignore_ascii_case(x))
        })
        .collect()
}

/// The most frequent item; ties go to the first seen.
pub fn most_common<T: Eq + Hash + Clone>(items: impl IntoIterator<Item = T>) -> Option<T> {
    let mut order = Vec::new();
    let mut counts = HashMap::new();
    for x in items {
        let n = counts.entry(x.clone()).or_insert(0usize);
        if *n == 0 {
            order.push(x);
        }
        *n += 1;
    }
    let max = counts.values().copied().max()?;
    order.into_iter().find(|x| counts[x] == max)
}

/// Mini number from a section name's last digit (A3 is mini 3), when it's one of the term's minis.
pub fn mini_of(section: &str, term: &TermMeta) -> Option<u8> {
    if section.encode_utf16().count() < 2 {
        return None;
    }
    let last = section.chars().last()?.to_digit(10)? as u8;
    term.minis.iter().any(|m| m.number == last).then_some(last)
}

pub fn add_instructors(target: &mut Vec<String>, names: impl IntoIterator<Item = String>) {
    for n in names {
        if !target.contains(&n) {
            target.push(n);
        }
    }
}

/// CSV rows, as Courseweave's `parseCsv`: quoted fields, `""` escapes, blank lines skipped.
pub fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if quoted {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    quoted = false;
                }
            } else {
                field.push(ch);
            }
        } else if ch == '"' {
            quoted = true;
        } else if ch == ',' {
            row.push(std::mem::take(&mut field));
        } else if ch == '\n' || ch == '\r' {
            if ch == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
            }
            row.push(std::mem::take(&mut field));
            if row.len() > 1 || !row[0].is_empty() {
                rows.push(std::mem::take(&mut row));
            } else {
                row.clear();
            }
        } else {
            field.push(ch);
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

/// Rows as header-keyed records, trimmed. Missing cells are empty.
pub fn csv_records(text: &str) -> Vec<HashMap<String, String>> {
    let mut rows = parse_csv(text).into_iter();
    let Some(header) = rows.next() else {
        return Vec::new();
    };
    let header: Vec<String> = header.iter().map(|h| js::trim(h).to_owned()).collect();
    rows.map(|cells| {
        header
            .iter()
            .enumerate()
            .map(|(i, h)| {
                (
                    h.clone(),
                    js::trim(cells.get(i).map_or("", String::as_str)).to_owned(),
                )
            })
            .collect()
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_and_times() {
        assert_eq!(parse_days("MWF"), [0, 2, 4]);
        assert_eq!(parse_days("TR"), [1, 3]);
        assert_eq!(parse_days("tuesday"), [1]);
        assert!(parse_days(" TBA ").is_empty());
        assert_eq!(parse_time("09:30AM"), Some(570));
        assert_eq!(parse_time("9:30 AM"), Some(570));
        assert_eq!(parse_time("07:00 P.M."), Some(1140));
        assert_eq!(parse_time("12:00PM"), Some(720));
        assert_eq!(parse_time("12:10AM"), Some(10));
        assert_eq!(parse_time("TBA"), None);
        assert_eq!(parse_time("9:3AM"), None);
        assert_eq!(clock("07:00 P.M.", false), None);
    }

    #[test]
    fn titles_and_people() {
        assert_eq!(
            clean_title("  Intro   to  CS Click to Print Page "),
            "Intro to CS"
        );
        assert_eq!(
            split_instructors("Kosbie, David; TBA ; Staff;  Jane   Doe"),
            ["Kosbie, David", "Jane Doe"]
        );
        assert_eq!(location(" tba "), None);
        assert_eq!(location("GHC"), Some("GHC".to_owned()));
        assert_eq!(most_common(["a", "b", "b", "a"]), Some("a"));
        assert_eq!(most_common(["a", "b", "b"]), Some("b"));
    }

    #[test]
    fn csv() {
        let rows = parse_csv("a,b\r\n\"x, \"\"y\"\"\",2\n\n3,4");
        assert_eq!(
            rows,
            [vec!["a", "b"], vec!["x, \"y\"", "2"], vec!["3", "4"]]
        );
        let records = csv_records("\u{FEFF}num , title\n15122, Imperative \n");
        assert_eq!(records[0]["num"], "15122");
        assert_eq!(records[0]["title"], "Imperative");
    }
}
