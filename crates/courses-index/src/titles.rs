//! Whether two course titles name the same course (Courseweave's `sameCourseTitle` in `src/lib/substitutes.ts`).
//! Stellic's equivalents and the SOC's cross-listings mix the same course under another number with courses that
//! only overlap or meet together; the catalog keeps a pair only when the titles name the same course.

use std::sync::LazyLock;

use regex::Regex;

/// `(Undergrad)`, `(Minor Option)`, ...: the same course for another audience. `(Honors)` is a different course.
static AUDIENCE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?i){ws}*\((?:undergrad(?:uate)?|ug|grad(?:uate)?|master'?s|minor option|college in high school)\){ws}*$",
        ws = crate::js::WS
    ))
    .unwrap()
});
static TOPICS_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?i)^(?:special topics?|st){ws}*:{ws}*",
        ws = crate::js::WS
    ))
    .unwrap()
});

fn title_words(title: &str) -> Vec<String> {
    let t = AUDIENCE.replace(title, "");
    let t = TOPICS_PREFIX.replace(&t, "");
    t.to_lowercase()
        .replace('&', " and ")
        .split(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit()))
        .filter(|w| !w.is_empty())
        .map(|w| {
            if w == "intro" {
                "introduction".to_owned()
            } else {
                w.to_owned()
            }
        })
        .collect()
}

/// Equal, a plural of the other, or one letter dropped from a long word (a typo: `Formalisims`).
fn same_word(a: &str, b: &str) -> bool {
    if a == b || format!("{a}s") == b || format!("{b}s") == a {
        return true;
    }
    let (short, long) = if a.len() < b.len() { (a, b) } else { (b, a) };
    if short.len() < 5 || long.len() != short.len() + 1 {
        return false;
    }
    (0..long.len()).any(|i| format!("{}{}", &long[..i], &long[i + 1..]) == short)
}

pub fn same_course_title(a: &str, b: &str) -> bool {
    let x = title_words(a);
    let y = title_words(b);
    !x.is_empty() && x.len() == y.len() && x.iter().zip(&y).all(|(a, b)| same_word(a, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_same_course_under_another_number() {
        for (a, b) in [
            (
                "Machine Learning in Practice",
                "Machine Learning in Practice",
            ),
            (
                "Responsible AI & AI Governance",
                "Responsible AI and AI Governance",
            ),
            (
                "Introduction to Probability Modeling",
                "Intro to Probability Modeling",
            ),
            ("Special Topics: Time Series", "Time Series"),
            (
                "ST: Developing Blockchain Use Cases",
                "Developing Blockchain Use Case",
            ),
            (
                "Machine Learning with Large Datasets (Undergraduate)",
                "Machine Learning with Large Datasets",
            ),
            (
                "Structure of Materials (Minor Option)",
                "Structure of Materials",
            ),
            ("Grammar Formalisims", "Grammar Formalisms"),
        ] {
            assert!(same_course_title(a, b), "{a} / {b}");
        }
    }

    #[test]
    fn rejects_courses_that_only_overlap() {
        for (a, b) in [
            ("Matrices and Linear Transformations", "Linear Algebra"),
            (
                "Differential and Integral Calculus",
                "Calculus in Three Dimensions",
            ),
            ("Intermediate French I", "Intermediate French II"),
            ("Undergraduate Seminar I", "Undergraduate Seminar III"),
            (
                "Principles of Macroeconomics",
                "Principles of Microeconomics",
            ),
            (
                "Physics I for Engineering Students",
                "Physics I for Science Students",
            ),
            ("Molecular Biology", "Graduate Molecular Biology"),
            (
                "Mathematical Statistics (Honors)",
                "Mathematical Statistics",
            ),
            ("", ""),
        ] {
            assert!(!same_course_title(a, b), "{a} / {b}");
        }
    }
}
