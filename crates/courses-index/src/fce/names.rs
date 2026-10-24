//! Instructor names, terms, course numbers and minis as FCEs write them: a port of Courseweave's
//! `src/lib/fce-names.ts`.
//!
//! FCE instructor names are "LAST, First Middle" (often upper case); the Schedule of Classes uses "First Middle
//! Last". Names are matched on (first given name, full last name), with a fallback on a compatible first name (an
//! initial, a prefix or a known nickname) with the same last name.

use std::collections::HashMap;
use std::sync::LazyLock;

use icu_normalizer::DecomposingNormalizerBorrowed;
use regex::Regex;

use super::js;

/// NFD, without combining diacritics, lowercased, with apostrophes dropped and anything but letters, whitespace,
/// commas, periods and hyphens turned into spaces.
fn normalize_name(s: &str) -> String {
    let nfd = DecomposingNormalizerBorrowed::new_nfd().normalize(s);
    let stripped: String = nfd
        .chars()
        .filter(|c| !('\u{300}'..='\u{36F}').contains(c))
        .collect();
    stripped
        .to_lowercase()
        .chars()
        .filter(|c| !matches!(c, '\u{2019}' | '\'' | '`'))
        .map(|c| {
            if c.is_ascii_lowercase() || js::is_space(c) || matches!(c, ',' | '.' | '-') {
                c
            } else {
                ' '
            }
        })
        .collect()
}

/// `s.split(/[\s.-]+/).filter(Boolean)`.
fn tokens(s: &str) -> Vec<&str> {
    s.split(|c| js::is_space(c) || c == '.' || c == '-')
        .filter(|t| !t.is_empty())
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameKeys {
    /// "first|last name", such as "iliano|cervesato" or "claire|le goues".
    pub full: String,
    /// "f|last name", for initials and nicknames.
    pub initial: String,
}

fn keys_of(first: &[&str], last: &[&str]) -> NameKeys {
    let f = first.first().copied().unwrap_or("");
    let l = last.join(" ");
    let initial: String = f.chars().take(1).collect();
    NameKeys {
        full: format!("{f}|{l}"),
        initial: format!("{initial}|{l}"),
    }
}

/// Keys for an FCE-style name ("CERVESATO, ILIANO"). Names without a comma are read as "First Last".
pub fn fce_name_keys(name: &str) -> NameKeys {
    let n = normalize_name(name);
    match n.find(',') {
        None => schedule_name_candidates(name)
            .into_iter()
            .next()
            .unwrap_or_else(|| NameKeys {
                full: "|".into(),
                initial: "|".into(),
            }),
        Some(comma) => keys_of(&tokens(&n[comma + 1..]), &tokens(&n[..comma])),
    }
}

/// Candidate keys for a schedule-style name ("Claire Le Goues"), one per way of splitting it into first and last
/// name, shortest last name first.
pub fn schedule_name_candidates(name: &str) -> Vec<NameKeys> {
    let normalized = normalize_name(name).replace(',', " ");
    let t = tokens(&normalized);
    if t.len() < 2 {
        return if t.is_empty() {
            Vec::new()
        } else {
            vec![keys_of(&[], &t)]
        };
    }
    (1..t.len())
        .rev()
        .map(|k| keys_of(&t[..1], &t[k..]))
        .collect()
}

fn first_of(full: &str) -> &str {
    &full[..full.find('|').unwrap_or(0)]
}

fn last_of(full: &str) -> &str {
    &full[full.find('|').map_or(0, |i| i + 1)..]
}

/// Whether an FCE-style name and a schedule-style name refer to the same person.
pub fn same_instructor(fce_name: &str, schedule_name: &str) -> bool {
    let k = fce_name_keys(fce_name);
    let candidates = schedule_name_candidates(schedule_name);
    candidates.iter().any(|c| c.full == k.full)
        || candidates.iter().any(|c| {
            last_of(&c.full) == last_of(&k.full)
                && first_names_compatible(first_of(&c.full), first_of(&k.full))
        })
}

/// Common English nicknames and the formal names they stand for. Only consulted when the last names already match.
const NICKNAMES: &[(&str, &[&str])] = &[
    ("al", &["albert", "alan", "alfred", "alexander"]),
    ("bert", &["robert", "albert", "herbert"]),
    ("beth", &["elizabeth"]),
    ("betsy", &["elizabeth"]),
    ("bill", &["william"]),
    ("billy", &["william"]),
    ("bob", &["robert"]),
    ("bobby", &["robert"]),
    ("cathy", &["catherine", "cathleen"]),
    ("charlie", &["charles"]),
    ("chuck", &["charles"]),
    ("cindy", &["cynthia"]),
    ("dan", &["daniel"]),
    ("danny", &["daniel"]),
    ("dave", &["david"]),
    ("dick", &["richard"]),
    ("don", &["donald"]),
    ("drew", &["andrew"]),
    ("ed", &["edward", "edwin"]),
    ("eddie", &["edward"]),
    ("hank", &["henry"]),
    ("harry", &["harold", "henry"]),
    ("jack", &["john"]),
    ("jen", &["jennifer"]),
    ("jenny", &["jennifer"]),
    ("jerry", &["gerald", "jerome", "jeremy"]),
    ("jim", &["james"]),
    ("jimmy", &["james"]),
    ("joe", &["joseph"]),
    ("kate", &["katherine", "kathleen", "catherine", "kathryn"]),
    ("katie", &["katherine", "kathleen", "catherine", "kathryn"]),
    ("kathy", &["katherine", "kathleen", "kathryn"]),
    ("larry", &["lawrence"]),
    ("len", &["leonard"]),
    ("lenny", &["leonard"]),
    ("liz", &["elizabeth"]),
    ("maggie", &["margaret"]),
    ("meg", &["margaret"]),
    ("mike", &["michael"]),
    ("nick", &["nicholas", "dominick", "dominic"]),
    ("peggy", &["margaret"]),
    ("rich", &["richard"]),
    ("rick", &["richard", "frederick"]),
    ("rob", &["robert"]),
    ("ron", &["ronald"]),
    ("sandy", &["sandra", "alexandra"]),
    ("steve", &["steven", "stephen"]),
    ("sue", &["susan", "suzanne"]),
    ("ted", &["edward", "theodore"]),
    ("tom", &["thomas"]),
    ("tony", &["anthony"]),
    ("vicky", &["victoria"]),
    ("zach", &["zachary"]),
    ("zack", &["zachary"]),
];

fn nickname_of(nick: &str, formal: &str) -> bool {
    NICKNAMES
        .iter()
        .any(|(n, formals)| *n == nick && formals.contains(&formal))
}

/// Same person by first name: equal, an initial, a prefix of 3+ letters (Ben / Benjamin), or a known nickname
/// (Liz / Elizabeth).
fn first_names_compatible(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    if a.is_empty() || b.is_empty() {
        return false;
    }
    if a.chars().count() == 1 || b.chars().count() == 1 {
        return a.chars().next() == b.chars().next();
    }
    let (short, long) = if a.chars().count() <= b.chars().count() {
        (a, b)
    } else {
        (b, a)
    };
    if short.chars().count() >= 3 && long.starts_with(short) {
        return true;
    }
    nickname_of(a, b) || nickname_of(b, a)
}

/// Values keyed by FCE-style names, looked up with schedule-style names. Exact (first, last) matches win; otherwise a
/// unique compatible first name with the same last name.
pub struct NameIndex<T> {
    by_full: HashMap<String, usize>,
    by_last: HashMap<String, Vec<(String, usize)>>,
    values: Vec<T>,
}

impl<T> Default for NameIndex<T> {
    fn default() -> Self {
        Self {
            by_full: HashMap::new(),
            by_last: HashMap::new(),
            values: Vec::new(),
        }
    }
}

impl<T> NameIndex<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, fce_name: &str, value: T) {
        self.add_key(&fce_name_keys(fce_name).full, value);
    }

    /// Adds under a precomputed full key ("first|last"), as `fce/instructors` is keyed. The first value for a key
    /// wins.
    pub fn add_key(&mut self, full: &str, value: T) {
        if self.by_full.contains_key(full) {
            return;
        }
        let index = self.values.len();
        self.values.push(value);
        self.by_full.insert(full.to_owned(), index);
        self.by_last
            .entry(last_of(full).to_owned())
            .or_default()
            .push((first_of(full).to_owned(), index));
    }

    /// Every value, in the order added.
    pub fn values(&self) -> &[T] {
        &self.values
    }

    pub fn get(&self, schedule_name: &str) -> Option<&T> {
        let candidates = schedule_name_candidates(schedule_name);
        for c in &candidates {
            if let Some(&i) = self.by_full.get(&c.full) {
                return Some(&self.values[i]);
            }
        }
        for c in &candidates {
            let hits: Vec<usize> = self
                .by_last
                .get(last_of(&c.full))
                .into_iter()
                .flatten()
                .filter(|(first, _)| first_names_compatible(first, first_of(&c.full)))
                .map(|&(_, i)| i)
                .collect();
            if let [i] = hits[..] {
                return Some(&self.values[i]);
            }
        }
        None
    }
}

static WORD_START: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(^|[-'.])([a-z])").unwrap());

/// "CERVESATO, ILIANO" to "Iliano Cervesato". Leaves mixed-case words alone.
pub fn display_fce_name(name: &str) -> String {
    let ordered = match name.find(',') {
        None => name.to_owned(),
        Some(comma) => format!("{} {}", &name[comma + 1..], &name[..comma]),
    };
    js::split_spaces(js::trim(&ordered))
        .into_iter()
        .map(|w| {
            if w == w.to_uppercase() {
                WORD_START
                    .replace_all(&w.to_lowercase(), |c: &regex::Captures| {
                        format!("{}{}", &c[1], c[2].to_uppercase())
                    })
                    .into_owned()
            } else {
                w.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ---- Terms ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Season {
    Spring,
    Summer,
    Fall,
}

impl Season {
    /// "Spring", "Summer" or "Fall", as the FCE export's `Sem` column has them.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "Spring" => Some(Self::Spring),
            "Summer" => Some(Self::Summer),
            "Fall" => Some(Self::Fall),
            _ => None,
        }
    }

    fn letter(self) -> char {
        match self {
            Self::Spring => 'S',
            Self::Summer => 'M',
            Self::Fall => 'F',
        }
    }

    /// Mini numbers by season: fall minis 1/2, spring 3/4, summer 5/6 (odd is the first half, even the second).
    fn minis(self) -> [u8; 2] {
        match self {
            Self::Fall => [1, 2],
            Self::Spring => [3, 4],
            Self::Summer => [5, 6],
        }
    }
}

/// (2026, Spring) to "S26".
pub fn fce_term_code(year: f64, season: Season) -> String {
    format!(
        "{}{}",
        season.letter(),
        js::pad_zero(&js::number_to_string(year % 100.0), 2)
    )
}

fn season_order(letter: Option<char>) -> f64 {
    match letter {
        Some('M') => 1.0,
        Some('F') => 2.0,
        _ => 0.0,
    }
}

/// Sortable index: later terms are larger.
pub fn term_order(code: &str) -> f64 {
    let mut chars = code.chars();
    let letter = chars.next();
    let year = 2000.0 + js::string_to_number(chars.as_str());
    year * 3.0 + season_order(letter)
}

/// "S26" to "Spring 2026".
pub fn term_name(code: &str) -> String {
    let mut chars = code.chars();
    let letter = chars.next();
    let season = match letter {
        Some('S') => "Spring".to_owned(),
        Some('M') => "Summer".to_owned(),
        Some('F') => "Fall".to_owned(),
        Some(c) => c.to_string(),
        None => "undefined".to_owned(),
    };
    let year = 2000.0 + js::string_to_number(chars.as_str());
    format!("{season} {}", js::number_to_string(year))
}

/// FCE course numbers come as "15122", "15-122", "6222" (leading zero dropped) or "FA14-12-100".
pub fn normalize_fce_course(num: &str) -> Option<String> {
    let digits: String = num.chars().filter(char::is_ascii_digit).collect();
    match digits.len() {
        n if n >= 5 => Some(digits[n - 5..].to_owned()),
        4 => Some(format!("0{digits}")),
        _ => None,
    }
}

// ---- Minis ----

static MINI_SECTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:[A-Za-z]{1,3}|[0-9])([1-6])$").unwrap());

/// Mini number of an FCE section name, or `None` for full-semester sections. Minis end in the mini digit after a
/// letter code ("A3", "B4", "M1") or a lecture number ("13" is lecture 1 in mini 3, as the Schedule of Classes names
/// them). Single digits ("1") are full-semester lectures.
pub fn fce_section_mini(section: &str, season: Season) -> Option<u8> {
    let caps = MINI_SECTION.captures(js::trim(section))?;
    let n: u8 = caps[1].parse().ok()?;
    season.minis().contains(&n).then_some(n)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Half {
    First,
    Second,
}

/// Which half of its semester a mini runs in.
pub fn mini_half(mini: u8) -> Half {
    if mini % 2 == 1 {
        Half::First
    } else {
        Half::Second
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_fce_names_on_first_given_name_and_full_last_name() {
        assert_eq!(fce_name_keys("CERVESATO, ILIANO").full, "iliano|cervesato");
        assert_eq!(fce_name_keys("LE GOUES, CLAIRE").full, "claire|le goues");
        assert_eq!(fce_name_keys("KAYNAR, ZELIHA DILSUN").full, "zeliha|kaynar");
        assert_eq!(
            fce_name_keys("COLE-RHODES, ARLENE").full,
            "arlene|cole rhodes"
        );
        assert_eq!(fce_name_keys("SEVER, S. DUYGU").full, "s|sever");
        assert_eq!(fce_name_keys(" SLAUGH,  VINCENT").full, "vincent|slaugh");
        assert_eq!(fce_name_keys("SEVER, S. DUYGU").initial, "s|sever");
        assert_eq!(fce_name_keys("O'NEIL, Pat").full, "pat|oneil");
        // Without a comma, the name reads as "First Last"; usernames have no first name.
        assert_eq!(fce_name_keys("Dennis Epple").full, "dennis|epple");
        assert_eq!(fce_name_keys("rtursky").full, "|rtursky");
        assert_eq!(fce_name_keys("").full, "|");
    }

    #[test]
    fn tries_every_first_last_split_of_schedule_names() {
        let full: Vec<_> = schedule_name_candidates("Claire Le Goues")
            .into_iter()
            .map(|c| c.full)
            .collect();
        assert_eq!(full, ["claire|goues", "claire|le goues"]);
        assert_eq!(
            schedule_name_candidates("Jos\u{e9} Mart\u{ed}nez")[0].full,
            "jose|martinez"
        );
    }

    #[test]
    fn matches_across_formats_accents_hyphens_and_middle_names() {
        let mut index = NameIndex::new();
        for n in [
            "CERVESATO, ILIANO",
            "LE GOUES, CLAIRE",
            "KAYNAR, ZELIHA DILSUN",
            "COLE-RHODES, ARLENE",
            "COLLIER, BENJAMIN",
            "RAVI, R.",
            "SMITH, JANE",
            "MARTINEZ, JOSE",
        ] {
            index.add(n, n);
        }
        assert_eq!(index.get("Iliano Cervesato"), Some(&"CERVESATO, ILIANO"));
        assert_eq!(index.get("Claire Le Goues"), Some(&"LE GOUES, CLAIRE"));
        assert_eq!(
            index.get("Zeliha Dilsun Kaynar"),
            Some(&"KAYNAR, ZELIHA DILSUN")
        );
        assert_eq!(
            index.get("Arlene Cole-Rhodes"),
            Some(&"COLE-RHODES, ARLENE")
        );
        assert_eq!(index.get("Ben Collier"), Some(&"COLLIER, BENJAMIN"));
        index.add("GARROD, CHARLES", "GARROD, CHARLES");
        index.add("RANSEY, ELIZABETH", "RANSEY, ELIZABETH");
        assert_eq!(index.get("Charlie Garrod"), Some(&"GARROD, CHARLES"));
        assert_eq!(index.get("Liz Ransey"), Some(&"RANSEY, ELIZABETH"));
        assert_eq!(index.get("R Ravi"), Some(&"RAVI, R."));
        assert_eq!(
            index.get("Jos\u{e9} Mart\u{ed}nez"),
            Some(&"MARTINEZ, JOSE")
        );
        // A different person with the same last name.
        assert_eq!(index.get("John Smith"), None);
        assert_eq!(index.values().len(), 10);
    }

    #[test]
    fn compares_one_fce_name_with_one_schedule_name() {
        assert!(same_instructor("CERVESATO, ILIANO", "Iliano Cervesato"));
        assert!(same_instructor("Epple, Dennis", "Dennis Epple"));
        assert!(!same_instructor("SMITH, JANE", "John Smith"));
    }

    #[test]
    fn formats_fce_names_for_display() {
        assert_eq!(display_fce_name("CERVESATO, ILIANO"), "Iliano Cervesato");
        assert_eq!(display_fce_name("ABASCAL, Brooke"), "Brooke Abascal");
        assert_eq!(
            display_fce_name("COLE-RHODES, ARLENE"),
            "Arlene Cole-Rhodes"
        );
        assert_eq!(display_fce_name("O'NEIL, J.R."), "J.R. O'Neil");
        assert_eq!(display_fce_name("rtursky"), "rtursky");
    }

    #[test]
    fn codes_and_orders_terms() {
        assert_eq!(fce_term_code(2026.0, Season::Spring), "S26");
        assert_eq!(fce_term_code(2025.0, Season::Summer), "M25");
        assert_eq!(fce_term_code(2005.0, Season::Fall), "F05");
        assert!(term_order("F25") > term_order("M25"));
        assert!(term_order("S26") > term_order("F25"));
        assert_eq!(term_name("F25"), "Fall 2025");
    }

    #[test]
    fn normalizes_fce_course_numbers() {
        assert_eq!(normalize_fce_course("15122").as_deref(), Some("15122"));
        assert_eq!(normalize_fce_course("15-451").as_deref(), Some("15451"));
        assert_eq!(normalize_fce_course("6222").as_deref(), Some("06222"));
        assert_eq!(
            normalize_fce_course("FA14-12-100").as_deref(),
            Some("12100")
        );
        assert_eq!(normalize_fce_course("Email Testing"), None);
    }

    #[test]
    fn reads_minis_from_fce_section_names_by_season() {
        assert_eq!(fce_section_mini("A3", Season::Spring), Some(3));
        assert_eq!(fce_section_mini("B4", Season::Spring), Some(4));
        assert_eq!(fce_section_mini("M1", Season::Fall), Some(1));
        assert_eq!(fce_section_mini("Z6", Season::Summer), Some(6));
        assert_eq!(fce_section_mini("a3", Season::Spring), Some(3));
        // The wrong season's digit, single-digit lectures and plain letters are full-semester.
        assert_eq!(fce_section_mini("A3", Season::Fall), None);
        assert_eq!(fce_section_mini("1", Season::Fall), None);
        assert_eq!(fce_section_mini("13", Season::Spring), Some(3));
        assert_eq!(fce_section_mini("22", Season::Fall), Some(2));
        assert_eq!(fce_section_mini("13", Season::Fall), None);
        assert_eq!(fce_section_mini("A", Season::Spring), None);
        assert_eq!(fce_section_mini("ABCD3", Season::Spring), None);
        assert_eq!(mini_half(3), Half::First);
        assert_eq!(mini_half(4), Half::Second);
    }
}
