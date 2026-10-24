//! Stellic requirement nodes to Courseweave's `ReqNode` (`src/lib/degree-types.ts`), ported from `normalize` and its
//! helpers in `scripts/build-programs.ts`.
//!
//! Stellic expresses each requirement as a node with `choices` (children) and `constraints`. The constraints the
//! audit can evaluate become fields; the rest are kept as human-readable `notes`.

use serde::Serialize;
use serde_json::Value;

use super::raw::{Js, Raw};
use crate::fce::js::{self, Num};

/// Stellic's `program_type` codes and the kinds they name.
pub fn kind_of(code: &str) -> Option<&'static str> {
    match code {
        "1" => Some("major"),
        "2" => Some("minor"),
        "3" => Some("additional-major"),
        "4" => Some("requirements"),
        "5" => Some("track"),
        _ => None,
    }
}

/// Course-number prefixes per CMU college, for Stellic "schools" course sets.
fn school_prefixes(school: &str) -> &'static [&'static str] {
    match school {
        "Carnegie Institute of Technology" => {
            &["06", "12", "14", "18", "19", "24", "27", "39", "42", "49"]
        }
        "School of Computer Science" => &["02", "05", "07", "08", "10", "11", "15", "16", "17"],
        "David A. Tepper School of Business" => &["45", "46", "47", "70"],
        "Mellon College of Science" => &["03", "09", "21", "33", "38"],
        "Carnegie Mellon University" => &["99"],
        "College of Fine Arts" => &["48", "51", "52", "53", "54", "57", "60", "62"],
        "Dietrich College of H&SS" => &[
            "36", "65", "66", "67", "73", "76", "79", "80", "82", "84", "85", "86", "88",
        ],
        "H. John Heinz III College" => &["90", "91", "92", "93", "94", "95"],
        _ => &[],
    }
}

/// A set of courses, matched against five-digit course ids. Fields are unioned.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolSet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub courses: Option<Vec<String>>,
    /// Inclusive ranges over five-digit ids.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranges: Option<Vec<(String, String)>>,
    /// Ranges over the last three digits, in any department.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_dept_ranges: Option<Vec<(Num, Num)>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depts: Option<Vec<String>>,
    /// Regex sources over five-digit ids, for wildcard codes like "15-3**".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patterns: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

impl PoolSet {
    fn any() -> Self {
        Self {
            any: Some(true),
            ..Self::default()
        }
    }

    fn is_empty(&self) -> bool {
        fn none<T>(v: &Option<Vec<T>>) -> bool {
            v.as_ref().is_none_or(Vec::is_empty)
        }
        self.any != Some(true)
            && none(&self.courses)
            && none(&self.ranges)
            && none(&self.any_dept_ranges)
            && none(&self.depts)
            && none(&self.patterns)
    }
}

/// "Take at least/at most `n` courses (or units) from these sets."
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CountRule {
    pub sets: Vec<PoolSet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inverse: Option<bool>,
    pub units: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<Num>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<Num>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub same_dept: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub different_depts: Option<bool>,
}

impl CountRule {
    fn new(sets: Vec<PoolSet>, units: bool) -> Self {
        Self {
            sets,
            inverse: None,
            units,
            min: None,
            max: None,
            same_dept: None,
            different_depts: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind")]
pub enum GroupRule {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "any")]
    Any,
    #[serde(rename = "atLeast")]
    AtLeast { n: Num },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CourseRef {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repeat {
    pub courses: Vec<String>,
    pub times: Num,
    pub same_course: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Window {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ShareWithin {
    pub limit: Num,
    pub units: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ProgramDependent {
    pub programs: Vec<Raw>,
    pub children: Raw,
    pub include: bool,
}

/// A requirement node. Optional fields are left out of the JSON when unset, as in Courseweave.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReqNode {
    /// Stellic's requirement id.
    pub id: Raw,
    pub name: String,
    pub rule: GroupRule,
    pub children: Vec<ReqNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub course: Option<CourseRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pool: Option<Vec<CountRule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<CountRule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caps: Option<Vec<CountRule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<PoolSet>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_units: Option<Num>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_courses: Option<Num>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_units_each: Option<Num>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat: Option<Repeat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_grade: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_pass_fail: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<Window>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_within: Option<ShareWithin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program_dependent: Option<Vec<ProgramDependent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_if_attempted: Option<bool>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Exception {
    pub nodes: Raw,
    pub limit: Num,
    pub units: bool,
}

/// A limit on courses shared between this program and others.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoubleCountRule {
    pub programs: Vec<Raw>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kinds: Option<Vec<&'static str>>,
    pub program_names: Vec<Raw>,
    pub limit: Num,
    pub units: bool,
    pub exceptions: Vec<Exception>,
}

// ---- Course codes, ranges and patterns ----

fn all_digits(s: &str) -> bool {
    s.bytes().all(|b| b.is_ascii_digit())
}

/// "15-122" or "15122" to "15122"; `None` for anything that isn't a plain course code.
pub fn course_id(code: &str) -> Option<String> {
    let t = js::trim(code);
    match t.len() {
        5 if all_digits(t) => Some(t.to_owned()),
        6 if t.as_bytes()[2] == b'-' && all_digits(&t[..2]) && all_digits(&t[3..]) => {
            Some(format!("{}{}", &t[..2], &t[3..]))
        }
        _ => None,
    }
}

fn is_wild(c: u8) -> bool {
    matches!(c, b'*' | b'x' | b'X')
}

fn code_chars(s: &str) -> bool {
    s.bytes().all(|b| b.is_ascii_digit() || is_wild(b))
}

/// `/^([0-9*xX]{2})-([0-9*xX]{1,4})$/` on the trimmed code.
fn split_code(code: &str) -> Option<(&str, &str)> {
    let (dept, num) = js::trim(code).split_once('-')?;
    (dept.len() == 2 && code_chars(dept) && (1..=4).contains(&num.len()) && code_chars(num))
        .then_some((dept, num))
}

/// "15-3**", "XX-4**", "99-x9x" to a regex source over five-digit ids.
pub fn pattern_source(code: &str) -> Option<String> {
    let (dept, num) = split_code(code)?;
    let num = format!("{num}***");
    let chars: String = format!("{dept}{}", &num[..3])
        .chars()
        .map(|c| {
            if is_wild(c as u8) {
                r"\d".to_owned()
            } else {
                c.to_string()
            }
        })
        .collect();
    Some(format!("^{chars}$"))
}

/// A range end: its department (`None` for a wildcard one) and number, with wildcards expanded low or high.
fn range_end(code: &str, high: bool) -> Option<(Option<&str>, u32)> {
    let (dept, num) = split_code(code)?;
    let dept = (!dept.bytes().any(is_wild)).then_some(dept);
    let fill = if high { '9' } else { '0' };
    let digits: String = num
        .chars()
        .chain(std::iter::repeat(fill))
        .take(3)
        .map(|c| if is_wild(c as u8) { fill } else { c })
        .collect();
    Some((dept, digits.parse().ok()?))
}

fn range_to_set(lo: &str, hi: &str, set: &mut PoolSet) -> bool {
    let (Some((a_dept, a)), Some((b_dept, b))) = (range_end(lo, false), range_end(hi, true)) else {
        return false;
    };
    match (a_dept, b_dept) {
        (Some(a_dept), Some(b_dept)) => set
            .ranges
            .get_or_insert_default()
            .push((format!("{a_dept}{a:03}"), format!("{b_dept}{b:03}"))),
        _ => set
            .any_dept_ranges
            .get_or_insert_default()
            .push((Num(f64::from(a)), Num(f64::from(b)))),
    }
    true
}

fn add_codes(set: &mut PoolSet, codes: Js) {
    for code in codes.items() {
        if let Some(id) = course_id(code.str()) {
            let courses = set.courses.get_or_insert_default();
            if !courses.contains(&id) {
                courses.push(id);
            }
        } else if let Some(src) = pattern_source(code.str()) {
            set.patterns.get_or_insert_default().push(src);
        }
    }
}

fn add_patterns(set: &mut PoolSet, patterns: Js) {
    for p in patterns.items() {
        let p = js::trim(p.str());
        // "15-***" is a whole department; keep those as prefixes (cheaper and clearer).
        if let Some((dept, rest)) = p.split_once('-')
            && (3..=4).contains(&rest.len())
            && rest.bytes().all(is_wild)
        {
            if dept.len() == 2 && all_digits(dept) {
                set.depts.get_or_insert_default().push(dept.to_owned());
                continue;
            }
            if dept.len() == 2 && dept.bytes().all(is_wild) {
                set.any = Some(true);
                continue;
            }
        }
        if let Some(src) = pattern_source(p) {
            set.patterns.get_or_insert_default().push(src);
        }
    }
}

fn add_schools(set: &mut PoolSet, schools: Js) {
    for school in schools.items() {
        for p in school_prefixes(&school.get("name").string()) {
            set.depts.get_or_insert_default().push((*p).to_owned());
        }
    }
}

/// Stellic `{ semester: 1 fall | 2 spring | 3 summer, year }` to "F22".
fn usem_term(semester: Js, year: Js) -> Option<String> {
    if !year.truthy() || !semester.truthy() {
        return None;
    }
    let season = match semester.0.and_then(Value::as_f64) {
        Some(1.0) => 'F',
        Some(2.0) => 'S',
        _ => 'M',
    };
    Some(format!(
        "{season}{}",
        js::pad_zero(&js::number_to_string(year.number() % 100.0), 2)
    ))
}

fn usem(u: Js) -> Option<String> {
    if !u.truthy() {
        return None;
    }
    usem_term(u.get("semester"), u.get("year"))
}

/// Stellic `conditional_course_sets` to pool sets, dropping excluded and empty ones.
fn course_sets(raw: Js, ctx: &mut Ctx) -> Vec<PoolSet> {
    let mut out = Vec::new();
    for s in raw.items() {
        let mut set = PoolSet::default();
        add_codes(&mut set, s.get("courses"));
        for pair in s.get("code_ranges").items() {
            if !range_to_set(pair.at(0).str(), pair.at(1).str(), &mut set) {
                ctx.unknown_patterns += 1;
            }
        }
        add_patterns(&mut set, s.get("code_patterns"));
        add_schools(&mut set, s.get("schools"));
        let tf = s.get("timeframe");
        if tf.truthy() {
            set.from = usem_term(tf.get("start_term"), tf.get("start_year"));
            set.to = usem_term(tf.get("end_term"), tf.get("end_year"));
        }
        if s.get("is_excluded").truthy() {
            continue;
        }
        if !set.is_empty() {
            out.push(set);
        }
    }
    out
}

// ---- Node normalization ----

pub struct Ctx<'a> {
    /// The root's limits on sharing courses with other programs.
    pub double_count: Vec<DoubleCountRule>,
    /// A program's name by catalog id, for limits Stellic doesn't name.
    pub program_name: &'a dyn Fn(Js) -> String,
    /// Course ranges that couldn't be parsed, for the build log.
    pub unknown_patterns: usize,
}

/// Constraint types that only affect Stellic's display or GPA bookkeeping.
const SILENT: [&str; 6] = [
    "course",
    "coursesection",
    "customgpa",
    "excludegpa",
    "computationorder",
    "checkfromreference",
];

fn group_rule(c: Js, child_count: usize) -> GroupRule {
    let data = c.get("data");
    let x = data.get("x").number_or(1.0);
    #[expect(clippy::cast_precision_loss, reason = "child counts are small")]
    let child_count = child_count as f64;
    if data.get("is_and").truthy() {
        GroupRule::All
    } else if x <= 1.0 {
        GroupRule::Any
    } else if x >= child_count {
        GroupRule::All
    } else {
        GroupRule::AtLeast { n: Num(x) }
    }
}

fn min_rule(sets: Vec<PoolSet>, units: bool, x: f64) -> CountRule {
    CountRule {
        min: (x > 0.0).then_some(Num(x)),
        ..CountRule::new(sets, units)
    }
}

pub fn normalize(raw: Js, ctx: &mut Ctx, is_root: bool) -> ReqNode {
    let kids: Vec<Js> = raw.get("choices").items().collect();
    let mut node = ReqNode {
        id: raw.get("id").to_raw(),
        name: js::trim(raw.get("screen_name").str()).to_owned(),
        rule: GroupRule::All,
        children: Vec::new(),
        course: None,
        pool: None,
        conditions: None,
        caps: None,
        exclude: None,
        min_units: None,
        min_courses: None,
        min_units_each: None,
        repeat: None,
        min_grade: None,
        allow_pass_fail: None,
        window: None,
        share_within: None,
        program_dependent: None,
        manual: None,
        only_if_attempted: None,
        notes: Vec::new(),
    };
    let note = |node: &mut ReqNode, c: Js| {
        let text = js::trim(&js::collapse_spaces(
            &c.get("type_string").or(c.get("type")).string(),
        ))
        .to_owned();
        if !text.is_empty() && !node.notes.contains(&text) {
            node.notes.push(text);
        }
    };
    let mut pool: Vec<CountRule> = Vec::new();

    for c in raw.get("constraints").items() {
        let d = c.get("data");
        match c.get("type").str() {
            "course" => {
                if let Some(id) = course_id(d.get("course").get("code").str()) {
                    let section = node
                        .course
                        .take()
                        .and_then(|c| c.section)
                        .filter(|s| !s.is_empty());
                    node.course = Some(CourseRef { id, section });
                }
            }
            "coursesection" => {
                if let Some(id) = course_id(d.get("course").get("code").str()) {
                    let section = d.get("section");
                    node.course = Some(CourseRef {
                        id,
                        section: section.truthy().then(|| section.string()),
                    });
                }
            }
            "anyxof" => node.rule = group_rule(c, kids.len()),
            "minxunits" => {
                let units = d.get("units").number();
                if d.get("is_each").truthy() {
                    node.min_units_each = Some(Num(units));
                } else {
                    let current = node.min_units.map_or(0.0, |n| n.0);
                    node.min_units = Some(Num(js::max(current, units)));
                }
            }
            "minxcourses" => {
                let current = node.min_courses.map_or(0.0, |n| n.0);
                node.min_courses = Some(Num(js::max(current, d.get("x").number())));
            }
            "xfromcourseset" => {
                let sets = course_sets(d.get("conditional_course_sets"), ctx);
                let x = d.get("x").number_or(0.0);
                let units = d.get("is_units").truthy();
                let inverse = d.get("inverse").truthy().then_some(true);
                if sets.is_empty() {
                    if x > 0.0 {
                        note(&mut node, c);
                    }
                } else if inverse.is_some() && x == 0.0 {
                    node.exclude.get_or_insert_default().extend(sets);
                } else if d.get("is_atmost").truthy() {
                    node.caps.get_or_insert_default().push(CountRule {
                        max: Some(Num(x)),
                        inverse,
                        ..CountRule::new(sets, units)
                    });
                } else {
                    pool.push(CountRule {
                        max: (d.get("is_exact").truthy() && x > 0.0).then_some(Num(x)),
                        inverse,
                        ..min_rule(sets, units, x)
                    });
                }
            }
            "coderange" => {
                let mut set = PoolSet::default();
                range_to_set(d.get("code_start").str(), d.get("code_end").str(), &mut set);
                add_codes(&mut set, d.get("additional_courses"));
                let x = d.get("x").number_or(0.0);
                if !set.is_empty() {
                    pool.push(min_rule(vec![set], false, x));
                }
            }
            "xfromdepts" => {
                let mut set = PoolSet::default();
                for dep in d.get("depts").items() {
                    let code = dep.get("code").string();
                    if code.len() == 2 && all_digits(&code) {
                        set.depts.get_or_insert_default().push(code);
                    }
                }
                add_schools(&mut set, d.get("schools"));
                add_codes(&mut set, d.get("additional_courses"));
                let x = d.get("x").number_or(0.0);
                if set.is_empty() {
                    note(&mut node, c);
                } else {
                    pool.push(CountRule {
                        same_dept: d.get("all_from_same_choice").truthy().then_some(true),
                        different_depts: d
                            .get("all_from_different_choice")
                            .truthy()
                            .then_some(true),
                        ..min_rule(vec![set], d.get("is_units").truthy(), x)
                    });
                }
            }
            "xfromallcourses" => {
                let x = d.get("x").number_or(0.0);
                pool.push(min_rule(vec![PoolSet::any()], false, x));
            }
            "notcountcourseset" => {
                let mut set = PoolSet::default();
                add_codes(&mut set, d.get("courses"));
                for pair in d.get("code_ranges").items() {
                    range_to_set(pair.at(0).str(), pair.at(1).str(), &mut set);
                }
                add_patterns(&mut set, d.get("code_patterns"));
                if !set.is_empty() {
                    node.exclude.get_or_insert_default().push(set);
                }
            }
            "mingrade" => {
                if d.get("grade").truthy() {
                    node.min_grade = Some(d.get("grade").string());
                }
            }
            "cancountungraded" => node.allow_pass_fail = Some(true),
            "repeatx" => {
                if let Some(id) = course_id(d.get("course").get("code").str()) {
                    node.repeat = Some(Repeat {
                        courses: vec![id],
                        times: Num(d.get("x").number_or(1.0)),
                        same_course: true,
                    });
                }
            }
            "repeatxcourses" => {
                let ids: Vec<String> = d
                    .get("courses")
                    .items()
                    .filter_map(|x| course_id(x.get("code").str()))
                    .collect();
                if !ids.is_empty() {
                    node.repeat = Some(Repeat {
                        courses: ids,
                        times: Num(d.get("x").number_or(1.0)),
                        same_course: d.get("same_course").truthy(),
                    });
                }
            }
            "notcountbeforeusem" => {
                let (from, to) = (usem(d.get("usem")), usem(d.get("end_usem")));
                if from.is_some() || to.is_some() {
                    node.window = Some(Window { from, to });
                } else {
                    note(&mut node, c);
                }
            }
            "noncourse" => {
                node.manual = Some(true);
                note(&mut node, c);
            }
            "staticonly" => {
                if d.get("name").str() != "OTHERS_MAY_COUNT" && kids.is_empty() {
                    node.manual = Some(true);
                }
                note(&mut node, c);
            }
            "alwayshide" => {
                if d.get("hide_unless_attempted").truthy() {
                    node.only_if_attempted = Some(true);
                }
            }
            "dc" => {
                let limit = d.get("limit").number_or(0.0);
                let units = d.get("is_units").truthy();
                if d.get("for_subrequirement").truthy() || !is_root {
                    // "... with other requirements" / "... within the following subrequirements": sharing inside
                    // this program.
                    if limit > 0.0 {
                        node.share_within = Some(ShareWithin {
                            limit: Num(limit),
                            units,
                        });
                    }
                } else {
                    let scope = d.get("scope_programs");
                    let program_type = d.get("scope_program_type");
                    let kind = if program_type.is_nullish() {
                        None
                    } else {
                        kind_of(&js::number_to_string(program_type.number()))
                    };
                    let program_names = scope
                        .items()
                        .map(|p| {
                            let name = p.get("name");
                            if name.is_nullish() {
                                Raw(Value::String((ctx.program_name)(p.get("id"))))
                            } else {
                                name.to_raw()
                            }
                        })
                        .collect();
                    let exceptions = d
                        .get("specific_subrequirements")
                        .items()
                        .map(|s| Exception {
                            nodes: raw_or_empty(s.get("requirements")),
                            limit: Num(s.get("limit").number_or(0.0)),
                            units: s.get("is_units").truthy(),
                        })
                        .collect();
                    ctx.double_count.push(DoubleCountRule {
                        programs: scope.items().map(|p| p.get("id").to_raw()).collect(),
                        kinds: kind.map(|k| vec![k]),
                        program_names,
                        limit: Num(limit),
                        units,
                        exceptions,
                    });
                }
            }
            "programdependentanyxof" => {
                let programs: Vec<Raw> = d
                    .get("scope_programs")
                    .items()
                    .map(|p| p.get("id").to_raw())
                    .collect();
                if !programs.is_empty()
                    && !d.get("is_inverse").truthy()
                    && !d.get("concentrations").has_length()
                    && !d.get("student_tags").has_length()
                {
                    node.program_dependent
                        .get_or_insert_default()
                        .push(ProgramDependent {
                            programs,
                            children: raw_or_empty(d.get("subrequirements")),
                            include: d.get("is_include").truthy(),
                        });
                }
                note(&mut node, c);
            }
            other => {
                if !SILENT.contains(&other) {
                    note(&mut node, c);
                }
            }
        }
    }

    if !kids.is_empty() {
        node.children = kids.into_iter().map(|k| normalize(k, ctx, false)).collect();
        if !pool.is_empty() {
            node.conditions = Some(pool);
        }
    } else if node.course.is_none() {
        let truthy = |n: Option<Num>| n.is_some_and(|n| n.0 != 0.0 && !n.0.is_nan());
        if !pool.is_empty() {
            node.pool = Some(pool);
        } else if (truthy(node.min_units) || truthy(node.min_courses)) && node.repeat.is_none() {
            node.pool = Some(vec![CountRule::new(vec![PoolSet::any()], false)]);
        } else if node.repeat.is_none() && node.manual.is_none() {
            // An empty placeholder: nothing to evaluate. Keep it visible as a manual check.
            node.manual = Some(true);
        }
    }
    node
}

/// `x ?? []`, copied.
fn raw_or_empty(x: Js) -> Raw {
    if x.is_nullish() {
        Raw(Value::Array(Vec::new()))
    } else {
        x.to_raw()
    }
}

#[cfg(test)]
mod tests;
