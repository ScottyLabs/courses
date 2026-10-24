//! Matching a major's concentrations to the requirement bundles that audit them, ported from Courseweave's
//! `scripts/lib/concentrations.ts`.
//!
//! Stellic lists a major's concentrations by name only ("Machine Learning"); the requirements that audit each one are a
//! separate "requirements" program. Several programs can contain the name: other campuses' versions ("CMU Africa -
//! Applied Machine Learning Concentration"), other majors' concentrations ("StatML Machine Learning Concentration
//! Requirements") and the major's own ("Machine Learning SCS Concentration"). Candidates are scored to pick the last.

use std::collections::BTreeSet;

use crate::fce::js;

pub struct Parent<'a> {
    pub name: &'a str,
    /// Campus of the major's version (`None` is Pittsburgh).
    pub campus: Option<&'a str>,
    /// Empty when Stellic gives an empty string.
    pub department: Option<&'a str>,
}

pub trait Candidate {
    fn name(&self) -> &str;
    /// Campuses of the candidate's versions (`None` entries are ignored).
    fn campuses(&self) -> Vec<Option<&str>>;
    fn department(&self) -> Option<&str>;
}

const STOP_WORDS: [&str; 6] = [
    "and",
    "the",
    "for",
    "concentration",
    "requirements",
    "track",
];

/// Words that name a requirements version ("EY2018", "2025 and Beyond", "(OLD)"), not a different program.
fn is_version_word(t: &str) -> bool {
    let digits4 = |s: &str| s.len() == 4 && s.bytes().all(|b| b.is_ascii_digit());
    digits4(t) || t.strip_prefix("ey").is_some_and(digits4) || matches!(t, "beyond" | "old" | "new")
}

/// Campus names in program names; the campus is scored on its own.
fn is_campus_word(t: &str) -> bool {
    matches!(
        t,
        "cmu" | "pittsburgh" | "qatar" | "doha" | "africa" | "rwanda" | "kigali"
    )
}

/// Distinct lowercase words of 3+ letters or digits, without stop words, in first-seen order.
pub fn name_tokens(s: &str) -> Vec<String> {
    let lower = s.to_lowercase();
    let mut seen = BTreeSet::new();
    lower
        .split(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit()))
        .filter(|t| t.len() > 2 && !STOP_WORDS.contains(t))
        .filter(|t| seen.insert(*t))
        .map(str::to_owned)
        .collect()
}

/// CMU-Africa programs are named "CMU Africa", "Rwanda" or "Kigali"; Qatar's "Qatar" or "Doha".
fn campus_group(campus: Option<&str>) -> Option<&str> {
    match campus {
        Some("Kigali" | "Rwanda") => Some("Africa"),
        Some("Qatar") => Some("Doha"),
        other => other,
    }
}

/// `/concentration|track/i` and the like: an ASCII case-insensitive search.
pub fn contains_ascii_ci(haystack: &str, needle: &str) -> bool {
    haystack.to_ascii_lowercase().contains(needle)
}

pub fn concentration_score(
    concentration: &str,
    parent: &Parent,
    candidate: &impl Candidate,
) -> f64 {
    let parent_tokens = name_tokens(parent.name);
    let wanted = name_tokens(concentration);
    let words: Vec<String> = name_tokens(candidate.name())
        .into_iter()
        .filter(|t| !is_version_word(t) && !is_campus_word(t))
        .collect();
    let mut score = if contains_ascii_ci(candidate.name(), "concentration")
        || contains_ascii_ci(candidate.name(), "track")
    {
        3.0
    } else {
        0.0
    };
    score += words.iter().filter(|t| parent_tokens.contains(t)).count() as f64;
    // The major's own department suggests its own concentration; another department, another program's.
    if let (Some(c), Some(p)) = (
        candidate.department().filter(|d| !d.is_empty()),
        parent.department.filter(|d| !d.is_empty()),
    ) {
        score += if c == p { 2.0 } else { -1.0 };
    }
    // Another campus's version is a last resort; a matching non-Pittsburgh campus is a hint.
    let parent_campus = campus_group(parent.campus).unwrap_or("Pittsburgh");
    let campuses: BTreeSet<&str> = candidate
        .campuses()
        .into_iter()
        .filter_map(campus_group)
        .collect();
    if !campuses.is_empty() && !campuses.contains(parent_campus) {
        score -= 6.0;
    } else if campuses.contains(parent_campus) && parent_campus != "Pittsburgh" {
        score += 1.0;
    }
    // Words beyond the concentration's and the major's own names point at some other program.
    let other = words
        .iter()
        .filter(|t| !wanted.contains(t) && !parent_tokens.contains(t))
        .count();
    score - 0.5 * other as f64
}

/// Candidates whose name contains the concentration's, best first; ties in name order, then in the given order.
pub fn rank_concentration_programs<'c, C: Candidate>(
    concentration: &str,
    parent: &Parent,
    candidates: &'c [C],
) -> Vec<&'c C> {
    let want = concentration.to_lowercase();
    let mut scored: Vec<(&C, f64)> = candidates
        .iter()
        .filter(|c| c.name().to_lowercase().contains(&want))
        .map(|c| (c, concentration_score(concentration, parent, c)))
        .collect();
    scored.sort_by(|a, b| {
        js::cmp_num(b.1, a.1).then_with(|| js::locale_compare(a.0.name(), b.0.name()))
    });
    scored.into_iter().map(|(c, _)| c).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct C {
        name: &'static str,
        campuses: Vec<Option<&'static str>>,
        department: Option<&'static str>,
    }

    impl Candidate for C {
        fn name(&self) -> &str {
            self.name
        }
        fn campuses(&self) -> Vec<Option<&str>> {
            self.campuses.clone()
        }
        fn department(&self) -> Option<&str> {
            self.department
        }
    }

    fn candidate(
        name: &'static str,
        campus: Option<&'static str>,
        department: Option<&'static str>,
    ) -> C {
        C {
            name,
            campuses: vec![campus],
            department,
        }
    }

    fn top(
        concentration: &str,
        parent: (&str, Option<&str>, Option<&str>),
        candidates: &[C],
    ) -> Option<&'static str> {
        let parent = Parent {
            name: parent.0,
            campus: parent.1,
            department: parent.2,
        };
        rank_concentration_programs(concentration, &parent, candidates)
            .first()
            .map(|c| c.name)
    }

    fn ml() -> Vec<C> {
        vec![
            candidate(
                "CMU Africa - Applied Machine Learning Concentration",
                Some("Africa"),
                None,
            ),
            candidate("Machine Learning SCS Concentration", None, None),
            candidate(
                "StatML Machine Learning Concentration Requirements",
                None,
                Some("Statistics"),
            ),
            candidate(
                "Machine Learning Advanced Elective",
                None,
                Some("Statistics"),
            ),
        ]
    }

    #[test]
    fn prefers_the_majors_own_concentration_over_other_campuses_and_majors() {
        assert_eq!(
            top(
                "Machine Learning",
                (
                    "BS in Computer Science",
                    Some("Pittsburgh"),
                    Some("Computer Science")
                ),
                &ml()
            ),
            Some("Machine Learning SCS Concentration")
        );
        assert_eq!(
            top(
                "Machine Learning",
                (
                    "BS in Statistics and Machine Learning",
                    None,
                    Some("Statistics")
                ),
                &ml()
            ),
            Some("StatML Machine Learning Concentration Requirements")
        );
    }

    #[test]
    fn matches_campuses_treating_kigali_and_rwanda_as_cmu_africa() {
        assert_eq!(
            top(
                "Applied Machine Learning",
                ("MS in Information Technology", Some("Kigali"), None),
                &ml()
            ),
            Some("CMU Africa - Applied Machine Learning Concentration")
        );
        // Another campus's version is still used when it's the only one.
        assert_eq!(
            top(
                "Information Security",
                ("BS in Information Systems", Some("Pittsburgh"), None),
                &[candidate(
                    "Information Security Concentration - Qatar",
                    Some("Doha"),
                    None
                )]
            ),
            Some("Information Security Concentration - Qatar")
        );
    }

    #[test]
    fn prefers_exact_names_and_ignores_version_words() {
        let analytics = [
            candidate(
                "Analytics Concentration",
                None,
                Some("Social and Decision Sciences"),
            ),
            candidate(
                "Data Analytics Concentration - BS in Information Systems",
                None,
                None,
            ),
        ];
        assert_eq!(
            top(
                "Analytics Concentration",
                (
                    "BS in Policy and Management",
                    None,
                    Some("Engineering and Public Policy")
                ),
                &analytics
            ),
            Some("Analytics Concentration")
        );
        let econ = [
            candidate(
                "EY2017 Economics (SHS) - Strategy and Markets Concentration",
                None,
                None,
            ),
            candidate(
                "EY2017 Economics - Strategy and Markets Concentration",
                None,
                None,
            ),
        ];
        assert_eq!(
            top(
                "Strategy and Markets",
                ("BS in Economics", None, None),
                &econ
            ),
            Some("EY2017 Economics - Strategy and Markets Concentration")
        );
    }

    #[test]
    fn tokenizes_names() {
        assert_eq!(
            name_tokens("EY2017 Economics (SHS) - and the Track"),
            ["ey2017", "economics", "shs"]
        );
        assert!(is_version_word("ey2017") && is_version_word("2025") && !is_version_word("ey20"));
    }
}
