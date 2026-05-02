//! Tokenization for text fields. Splits on whitespace and ASCII punctuation
//! while preserving course-code shapes like `15-122` as a single token, so
//! exact code searches don't have to special-case anything downstream.
//!
//! Output is lowercased ASCII. Terms shorter than 2 chars are dropped.

const MIN_TERM_LEN: usize = 2;

pub fn tokenize_general(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut buf = String::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_alphanumeric() {
            buf.push(c.to_ascii_lowercase() as char);
            i += 1;
        } else if c == b'-'
            && !buf.is_empty()
            && buf.chars().all(|ch| ch.is_ascii_digit())
            && i + 1 < bytes.len()
            && bytes[i + 1].is_ascii_digit()
        {
            buf.push('-');
            i += 1;
        } else {
            flush(&mut buf, &mut out);
            i += 1;
        }
    }
    flush(&mut buf, &mut out);
    out
}

/// Tokenize a long-form text field (course description, etc) but drop any
/// course-code-shaped tokens. Descriptions routinely reprint a course's
/// prereqs verbatim ("e.g. 15-213") which would otherwise let those codes
/// score on description weight (1.0) on top of their proper prereqs_text
/// weight (0.3), inflating courses that simply mention a code over the
/// courses that genuinely depend on it.
pub fn tokenize_description(text: &str) -> Vec<String> {
    tokenize_general(text)
        .into_iter()
        .filter(|t| !is_course_code(t))
        .collect()
}

fn is_course_code(t: &str) -> bool {
    let bytes = t.as_bytes();
    bytes.len() == 6
        && bytes[2] == b'-'
        && bytes[..2].iter().all(|c| c.is_ascii_digit())
        && bytes[3..].iter().all(|c| c.is_ascii_digit())
}

/// True when `t` is a non-empty prefix of a course code shorter than the
/// full `XX-YYY` shape: `21-`, `21-2`, `21-24` all qualify, while `21-241`
/// (already a full code) does not. Lets the search front-end run a
/// dedicated prefix walk on the code FST as the user types.
pub fn is_partial_code(t: &str) -> bool {
    let bytes = t.as_bytes();
    if !(3..=5).contains(&bytes.len()) {
        return false;
    }
    if bytes[2] != b'-' {
        return false;
    }
    bytes[..2].iter().all(|c| c.is_ascii_digit()) && bytes[3..].iter().all(|c| c.is_ascii_digit())
}

pub fn tokenize_code(code: &str) -> Vec<String> {
    let lower = code.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return Vec::new();
    }
    let mut out = vec![lower.clone()];
    if let Some((dept, num)) = lower.split_once('-') {
        if !dept.is_empty() {
            out.push(dept.to_string());
        }
        if !num.is_empty() {
            out.push(num.to_string());
        }
    }
    out
}

fn flush(buf: &mut String, out: &mut Vec<String>) {
    if buf.len() >= MIN_TERM_LEN {
        out.push(std::mem::take(buf));
    } else {
        buf.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_split() {
        assert_eq!(
            tokenize_general("Linear Algebra is hard"),
            vec!["linear", "algebra", "is", "hard"]
        );
    }

    #[test]
    fn keeps_course_codes() {
        let toks = tokenize_general("requires 15-122 or 21-127");
        assert!(toks.contains(&"15-122".to_string()));
        assert!(toks.contains(&"21-127".to_string()));
    }

    #[test]
    fn drops_short_terms() {
        let toks = tokenize_general("a b cd ef ghij");
        assert_eq!(toks, vec!["cd", "ef", "ghij"]);
    }

    #[test]
    fn code_field_emits_halves() {
        let toks = tokenize_code("15-122");
        assert_eq!(toks, vec!["15-122", "15", "122"]);
    }
}
