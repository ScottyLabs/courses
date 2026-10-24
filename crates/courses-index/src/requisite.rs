//! Requisite expressions (Courseweave's `src/lib/requisite-format.ts` and the Stellic tree reader in
//! `scripts/build-catalog.ts`).

use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};

use crate::js;
use crate::model::{course_code, is_course_id};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Requisite {
    /// A five-digit course id, with the minimum grade when a passing grade isn't enough.
    Course {
        id: String,
        min_grade: Option<String>,
    },
    All(Vec<Requisite>),
    Any(Vec<Requisite>),
    /// At least `n` of the items, with `1 < n < items.len()`.
    AtLeast(i64, Vec<Requisite>),
    /// A condition that isn't a course (standing, permission).
    Text(String),
}

impl Serialize for Requisite {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        match self {
            Requisite::Course { id, min_grade } => {
                map.serialize_entry("type", "course")?;
                map.serialize_entry("id", id)?;
                if let Some(g) = min_grade {
                    map.serialize_entry("minGrade", g)?;
                }
            }
            Requisite::All(items) => {
                map.serialize_entry("type", "all")?;
                map.serialize_entry("items", items)?;
            }
            Requisite::Any(items) => {
                map.serialize_entry("type", "any")?;
                map.serialize_entry("items", items)?;
            }
            Requisite::AtLeast(n, items) => {
                map.serialize_entry("type", "atLeast")?;
                map.serialize_entry("n", n)?;
                map.serialize_entry("items", items)?;
            }
            Requisite::Text(text) => {
                map.serialize_entry("type", "text")?;
                map.serialize_entry("text", text)?;
            }
        }
        map.end()
    }
}

fn to_id(code: &str) -> String {
    code.chars().filter(char::is_ascii_digit).collect()
}

fn course(id: String, grade: Option<&str>) -> Requisite {
    let min_grade = grade.filter(|g| *g != "D").map(str::to_owned);
    Requisite::Course { id, min_grade }
}

#[derive(Debug, PartialEq)]
enum Token {
    Open,
    Close,
    And,
    Or,
    Course(String),
    Grade(String),
}

fn tokenize(text: &str) -> Option<Vec<Token>> {
    // `[]` separates a course from its grade in Stellic's wording.
    let mut src = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find('[') {
        src.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        let inner = after.trim_start_matches(js::is_space);
        if let Some(tail) = inner.strip_prefix(']') {
            src.push(' ');
            rest = tail;
        } else {
            src.push('[');
            rest = after;
        }
    }
    src.push_str(rest);

    let word_char = |c: Option<char>| c.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
    let mut tokens = Vec::new();
    let mut pos = 0;
    while pos < src.len() {
        let rest = &src[pos..];
        let trimmed = rest.trim_start_matches(js::is_space);
        if trimmed.is_empty() {
            break;
        }
        let start = pos + (rest.len() - trimmed.len());
        let prev = src[..start].chars().next_back();
        let lower = trimmed.to_ascii_lowercase();
        let (token, len) = if trimmed.starts_with('(') {
            (Token::Open, 1)
        } else if trimmed.starts_with(')') {
            (Token::Close, 1)
        } else if trimmed.starts_with(',') {
            (Token::Or, 1)
        } else if (lower.starts_with("and") || lower.starts_with("or"))
            && !word_char(prev)
            && !word_char(
                trimmed[if lower.starts_with("and") { 3 } else { 2 }..]
                    .chars()
                    .next(),
            )
        {
            if lower.starts_with("and") {
                (Token::And, 3)
            } else {
                (Token::Or, 2)
            }
        } else if let Some(len) = course_number(trimmed) {
            (Token::Course(to_id(&trimmed[..len])), len)
        } else {
            let mut chars = lower.strip_prefix("at least ")?.chars();
            let letter = chars.next().filter(|c| ('a'..='d').contains(c))?;
            let modifier = chars.next().filter(|c| matches!(c, '+' | '-'));
            let mut grade = letter.to_ascii_uppercase().to_string();
            grade.extend(modifier);
            let len = "at least ".len() + grade.len();
            (Token::Grade(grade), len)
        };
        tokens.push(token);
        pos = start + len;
    }
    Some(tokens)
}

/// `\d{2}-?\d{3}` at the start.
fn course_number(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let digits =
        |r: std::ops::Range<usize>| b.get(r).is_some_and(|d| d.iter().all(u8::is_ascii_digit));
    if !digits(0..2) {
        return None;
    }
    let skip = usize::from(b.get(2) == Some(&b'-'));
    digits(2 + skip..5 + skip).then_some(5 + skip)
}

struct Parser {
    tokens: Vec<Token>,
    i: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.i)
    }

    fn primary(&mut self) -> Option<Requisite> {
        let t = self.tokens.get(self.i)?;
        self.i += 1;
        match t {
            Token::Open => {
                let inner = self.or_expr()?;
                (self.tokens.get(self.i) == Some(&Token::Close)).then_some(())?;
                self.i += 1;
                Some(inner)
            }
            Token::Course(id) => {
                let id = id.clone();
                if let Some(Token::Grade(g)) = self.peek() {
                    let g = g.clone();
                    self.i += 1;
                    Some(course(id, Some(&g)))
                } else {
                    Some(course(id, None))
                }
            }
            _ => None,
        }
    }

    fn and_expr(&mut self) -> Option<Requisite> {
        let mut items = vec![self.primary()?];
        while self.peek() == Some(&Token::And) {
            self.i += 1;
            items.push(self.primary()?);
        }
        Some(if items.len() == 1 {
            items.pop().unwrap()
        } else {
            Requisite::All(items)
        })
    }

    fn or_expr(&mut self) -> Option<Requisite> {
        let mut items = vec![self.and_expr()?];
        while self.peek() == Some(&Token::Or) {
            self.i += 1;
            items.push(self.and_expr()?);
        }
        Some(if items.len() == 1 {
            items.pop().unwrap()
        } else {
            Requisite::Any(items)
        })
    }
}

/// Best-effort parser for requisite wording: `15112 or 02120`, `(15-122 [] at least C) and (21-127 or 15-151)`.
/// `and` binds tighter than `or`; commas read as `or`. `None` for anything else.
pub fn parse_text(text: &str) -> Option<Requisite> {
    let tokens = tokenize(text)?;
    if tokens.is_empty() {
        return None;
    }
    let mut p = Parser { tokens, i: 0 };
    let result = p.or_expr()?;
    (p.i == p.tokens.len()).then(|| simplify(result))
}

/// Flattens same-kind nesting, collapses single-item groups and removes duplicate alternatives.
pub fn simplify(r: Requisite) -> Requisite {
    let (kind, n, items) = match r {
        Requisite::All(items) => (0, 0, items),
        Requisite::Any(items) => (1, 0, items),
        Requisite::AtLeast(n, items) => (2, n, items),
        other => return other,
    };
    let mut flat = Vec::new();
    for x in items.into_iter().map(simplify) {
        match (kind, x) {
            (0, Requisite::All(inner)) | (1, Requisite::Any(inner)) => flat.extend(inner),
            (_, x) => flat.push(x),
        }
    }
    let mut seen = Vec::new();
    for x in flat {
        if !seen.contains(&x) {
            seen.push(x);
        }
    }
    let mut items = seen;
    if items.len() == 1 {
        return items.pop().unwrap();
    }
    match kind {
        0 => Requisite::All(items),
        1 => Requisite::Any(items),
        _ if n >= items.len() as i64 => Requisite::All(items),
        _ if n <= 1 => Requisite::Any(items),
        _ => Requisite::AtLeast(n, items),
    }
}

/// Plain text, such as `15-122 (min C) and (21-127 or 15-151)`.
pub fn format(req: &Requisite, nested: bool) -> String {
    let group = |items: &[Requisite], sep: &str| {
        let s = items
            .iter()
            .map(|x| format(x, true))
            .collect::<Vec<_>>()
            .join(sep);
        if nested { format!("({s})") } else { s }
    };
    match req {
        Requisite::Course {
            id,
            min_grade: Some(g),
        } => format!("{} (min {g})", course_code(id)),
        Requisite::Course {
            id,
            min_grade: None,
        } => course_code(id),
        Requisite::Text(t) => t.clone(),
        Requisite::All(items) => group(items, " and "),
        Requisite::Any(items) => group(items, " or "),
        Requisite::AtLeast(n, items) => {
            let s = format!(
                "{n} of {}",
                items
                    .iter()
                    .map(|x| format(x, true))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            if nested { format!("({s})") } else { s }
        }
    }
}

/// A node of a Stellic requirement tree (`info.json`'s `prereqs.req_obj`).
#[derive(Clone, Debug, Default, Deserialize)]
pub struct StellicNode {
    #[serde(default)]
    pub constraints: Option<Vec<StellicConstraint>>,
    #[serde(default)]
    pub choices: Option<Vec<StellicNode>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StellicConstraint {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub data: Option<ConstraintData>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ConstraintData {
    #[serde(default)]
    pub is_and: Option<bool>,
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub grade: Option<String>,
    #[serde(default)]
    pub course: Option<StellicCourseRef>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct StellicCourseRef {
    pub code: String,
    #[serde(default)]
    pub name: Option<String>,
}

/// A Stellic requirement tree as a requisite. Leaves are `course` constraints; groups carry `anyxof` and `mingrade`.
pub fn from_stellic(node: &StellicNode, inherited_grade: Option<&str>) -> Option<Requisite> {
    let cons = node.constraints.as_deref().unwrap_or_default();
    let data = |kind: &str| {
        cons.iter()
            .find(|c| c.kind == kind)
            .and_then(|c| c.data.as_ref())
    };
    let grade = data("mingrade")
        .and_then(|d| d.grade.as_deref())
        .or(inherited_grade);
    if let Some(c) = data("course").and_then(|d| d.course.as_ref()) {
        let id = to_id(&c.code);
        if !is_course_id(&id) {
            return Some(Requisite::Text(c.code.clone()));
        }
        return Some(course(id, grade.filter(|g| !g.is_empty())));
    }
    let items: Vec<Requisite> = node
        .choices
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter_map(|c| from_stellic(c, grade))
        .collect();
    if items.is_empty() {
        return None;
    }
    let anyx = data("anyxof");
    let n = anyx.and_then(|d| d.x).unwrap_or(items.len() as f64);
    Some(
        if anyx.and_then(|d| d.is_and) == Some(true) || n >= items.len() as f64 {
            Requisite::All(items)
        } else if n <= 1.0 {
            Requisite::Any(items)
        } else {
            Requisite::AtLeast(n as i64, items)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(id: &str) -> Requisite {
        course(id.to_owned(), None)
    }

    fn cg(id: &str, g: &str) -> Requisite {
        course(id.to_owned(), Some(g))
    }

    #[test]
    fn reads_soc_wording() {
        assert_eq!(
            parse_text("15112 or 02120"),
            Some(Requisite::Any(vec![c("15112"), c("02120")]))
        );
    }

    #[test]
    fn reads_stellic_wording_with_grades() {
        assert_eq!(
            parse_text("(15-150 [] at least C) and (15-122 [] at least D)"),
            Some(Requisite::All(vec![cg("15150", "C"), c("15122")]))
        );
        assert_eq!(parse_text("15-213 at least B+"), Some(cg("15213", "B+")));
    }

    #[test]
    fn and_binds_tighter_than_or() {
        assert_eq!(
            parse_text("15122 and 21127 or 15151"),
            Some(Requisite::Any(vec![
                Requisite::All(vec![c("15122"), c("21127")]),
                c("15151")
            ]))
        );
        assert_eq!(
            parse_text("15122, 15150 AND 21127"),
            Some(Requisite::Any(vec![
                c("15122"),
                Requisite::All(vec![c("15150"), c("21127")])
            ]))
        );
    }

    #[test]
    fn rejects_what_it_cannot_read() {
        assert_eq!(parse_text("Junior standing"), None);
        assert_eq!(parse_text("15122 and"), None);
        assert_eq!(parse_text("(15122"), None);
        assert_eq!(parse_text(""), None);
        assert_eq!(parse_text("   "), None);
        assert_eq!(parse_text("15122andor"), None);
    }

    #[test]
    fn simplifies() {
        assert_eq!(
            simplify(Requisite::All(vec![
                Requisite::All(vec![c("1"), c("2")]),
                c("1")
            ])),
            Requisite::All(vec![c("1"), c("2")])
        );
        assert_eq!(simplify(Requisite::Any(vec![c("1")])), c("1"));
        assert_eq!(
            simplify(Requisite::AtLeast(2, vec![c("1"), c("2")])),
            Requisite::All(vec![c("1"), c("2")])
        );
    }

    #[test]
    fn formats_and_serializes() {
        let req = Requisite::All(vec![
            cg("15122", "C"),
            Requisite::Any(vec![c("21127"), c("15151")]),
        ]);
        assert_eq!(format(&req, false), "15-122 (min C) and (21-127 or 15-151)");
        assert_eq!(
            serde_json::to_string(&Requisite::AtLeast(
                2,
                vec![c("15122"), Requisite::Text("x".into())]
            ))
            .unwrap(),
            r#"{"type":"atLeast","n":2,"items":[{"type":"course","id":"15122"},{"type":"text","text":"x"}]}"#
        );
    }

    #[test]
    fn reads_stellic_trees() {
        let tree: StellicNode = serde_json::from_str(
            r#"{"constraints":[{"type":"anyxof","data":{"x":1,"is_and":true}},{"type":"mingrade","data":{"grade":"C"}}],
            "choices":[
              {"constraints":[{"type":"course","data":{"course":{"code":"21-111"}}}]},
              {"constraints":[{"type":"anyxof","data":{"x":1,"is_and":false}}],"choices":[
                {"constraints":[{"type":"mingrade","data":{"grade":"D"}},{"type":"course","data":{"course":{"code":"15-112"}}}]},
                {"constraints":[{"type":"course","data":{"course":{"code":"AP CS"}}}]}]},
              {"constraints":[{"type":"anyxof","data":{"x":2}}],"choices":[
                {"constraints":[{"type":"course","data":{"course":{"code":"15-150"}}}]},
                {"constraints":[{"type":"course","data":{"course":{"code":"15-151"}}}]},
                {"constraints":[{"type":"course","data":{"course":{"code":"15-210"}}}]}]},
              {"choices":[]}]}"#,
        )
        .unwrap();
        assert_eq!(
            from_stellic(&tree, None),
            Some(Requisite::All(vec![
                cg("21111", "C"),
                Requisite::Any(vec![c("15112"), Requisite::Text("AP CS".into())]),
                Requisite::AtLeast(
                    2,
                    vec![cg("15150", "C"), cg("15151", "C"), cg("15210", "C")]
                ),
            ]))
        );
    }
}
