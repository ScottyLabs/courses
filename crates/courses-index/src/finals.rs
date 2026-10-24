//! Final exam schedules from the registrar's PDFs: `data/finals/<term>.pdf`, plus `data/finals/<term>-<part>.pdf`
//! for separately published parts (such as a mini's exams), become `finals/<term>`.
//!
//! The PDFs are spreadsheets printed to PDF. Their text is rows like
//! `48240 A HIS WLD ARC Tuesday, April 28, 2026 01:00pm - 04:00pm In Person TBD SHAW DIANE (dshaw)`, with department
//! headers, column headers, page footers and notices in between. Each row is found by its date and time range, so
//! rows survive line breaks wherever the text extractor puts them.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::LazyLock;

use anyhow::{Context, Result, bail, ensure};
use rayon::prelude::*;
use regex::Regex;
use serde::Serialize;
use tracing::info;

use crate::js;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Exam {
    pub course_id: String,
    pub section: String,
    /// The registrar's abbreviated title.
    pub title: String,
    /// `YYYY-MM-DD`.
    pub date: String,
    /// Minutes after midnight, Pittsburgh time.
    pub start: i32,
    pub end: i32,
    /// `In Person`, `Remote` or `Cancelled`.
    pub mode: String,
    /// Room, or the registrar's note about it (`To Be Announced`).
    pub location: Option<String>,
    /// As printed, last name first, such as `SHAW DIANE`.
    pub instructors: Vec<String>,
}

const WEEKDAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// A row's date, times and delivery mode, which every row has.
static ANCHOR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?x)
        (?P<weekday>{weekdays}),\s*(?P<month>{months})\s+(?P<day>[0-9]{{1,2}})(?:,?\s*(?P<year>[0-9]{{4}}))?\s+
        (?P<start>[0-9]{{1,2}}:[0-9]{{2}}\s*[AaPp]\.?\s*[Mm]\.?)\s*[-\x{{2013}}]\s*
        (?P<end>[0-9]{{1,2}}:[0-9]{{2}}\s*[AaPp]\.?\s*[Mm]\.?)\s*
        (?P<mode>(?i:in[\s-]*person|remote|online|hybrid|cancell?ed))(?-u:\b)",
        weekdays = WEEKDAYS.join("|"),
        months = MONTHS.join("|"),
    ))
    .unwrap()
});

/// A row's course number, section and title, ending where its date starts.
static HEAD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|[\s])(?P<course>[0-9]{5})[ \t]+(?P<section>[A-Z0-9]{1,3})[ \t]+").unwrap()
});

static ANDREW_ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\(([a-z][a-z0-9]*)\)").unwrap());

/// Text that follows a row on the same line in some PDFs and is never part of it.
const JUNK: [&str; 8] = [
    "Course Sec Title",
    "Notice:",
    "Note:",
    "University Registrar",
    "Questions:",
    "The End",
    "Before making travel",
    "Teaching Assistants who grade",
];

/// Words in all capitals that are a location, not part of a name.
const PLACEHOLDERS: [&str; 5] = ["TBA", "TBD", "CANCELED", "CANCELLED", "REMOTE"];

fn minutes(time: &str) -> Option<i32> {
    let compact: String = time
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '.')
        .collect();
    crate::text::clock(&compact, false)
}

fn is_name_token(token: &str) -> bool {
    let t = token.strip_suffix(',').unwrap_or(token);
    t.starts_with(|c: char| c.is_ascii_uppercase())
        && t.chars()
            .all(|c| c.is_ascii_uppercase() || matches!(c, '.' | '\'' | '-'))
        && !PLACEHOLDERS.contains(&t)
}

fn cut_junk(s: &str) -> &str {
    let end = JUNK
        .iter()
        .filter_map(|j| s.find(j))
        .chain(s.find('\n'))
        .min()
        .unwrap_or(s.len());
    &s[..end]
}

/// Location and instructors from the text after a row's mode.
fn location_and_instructors(rest: &str) -> (Option<String>, Vec<String>) {
    let ids: Vec<_> = ANDREW_ID.find_iter(rest).collect();
    let nonempty = |s: &str| Some(js::squish(s)).filter(|s| !s.is_empty());
    let Some(first) = ids.first() else {
        return (nonempty(cut_junk(rest)), Vec::new());
    };
    // The first name follows the last cell boundary, when the extractor saw one, or is the run of capitalized words
    // before the first id.
    let before = &rest[..first.start()];
    let after_tab = before.rfind('\t').map(|i| i + 1);
    let name_start = match after_tab {
        Some(i)
            if before[i..].split_whitespace().all(is_name_token)
                && !js::trim(&before[i..]).is_empty() =>
        {
            i
        }
        _ => {
            let mut start = before.len();
            for token in before.split_whitespace().rev() {
                if !is_name_token(token) {
                    break;
                }
                start = token.as_ptr() as usize - before.as_ptr() as usize;
            }
            start
        }
    };
    let location = nonempty(cut_junk(&before[..name_start]));
    let mut instructors = Vec::new();
    let mut from = name_start;
    for id in &ids {
        let raw = rest[from..id.start()].trim_start_matches([',', ' ', '\t']);
        let name = js::squish(raw);
        if name.is_empty() || raw.contains('\n') {
            break;
        }
        if !instructors.contains(&name) {
            instructors.push(name);
        }
        from = id.end();
        // Only a comma separates consecutive instructors.
        let next = rest[from..].trim_start_matches([' ', '\t']);
        if !next.starts_with(',') {
            break;
        }
    }
    (location, instructors)
}

static NO_INSTRUCTOR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?-u:\b)INSTRUCTOR\s+TBA\s*\(instructor\)").unwrap());

/// Rows of a final exam schedule's text. `year` is used for dates printed without one.
pub fn parse(text: &str, year: i64) -> Result<Vec<Exam>> {
    let text = text.replace('\u{A0}', " ");
    let text = NO_INSTRUCTOR.replace_all(&text, "");
    let anchors: Vec<_> = ANCHOR.captures_iter(&text).collect();
    let mut exams = Vec::with_capacity(anchors.len());
    let mut prev_end = 0;
    for (i, a) in anchors.iter().enumerate() {
        let whole = a.get(0).unwrap();
        // The head is the last course number between the previous row and this date.
        let segment = &text[prev_end..whole.start()];
        let Some(head) = HEAD
            .captures_iter(segment)
            .filter(|h| {
                let title = &segment[h.get(0).unwrap().end()..];
                !title.contains('\n') && !ANDREW_ID.is_match(title)
            })
            .last()
        else {
            bail!("no course number before {:?}", whole.as_str());
        };
        let title = js::squish(&segment[head.get(0).unwrap().end()..]);
        let next_start = anchors
            .get(i + 1)
            .map_or(text.len(), |n| n.get(0).unwrap().start());
        let rest = &text[whole.end()..next_start];
        let (location, instructors) = location_and_instructors(rest);

        let month = MONTHS.iter().position(|m| *m == &a["month"]).unwrap() as u32 + 1;
        let day: u32 = a["day"].parse()?;
        let y: i64 = a.name("year").map_or(Ok(year), |y| y.as_str().parse())?;
        let days = js::days_from_civil(y, month, day);
        let weekday = WEEKDAYS.iter().position(|d| *d == &a["weekday"]).unwrap() as i64;
        // 1970-01-01 was a Thursday.
        ensure!(
            (days + 3).rem_euclid(7) == weekday,
            "{} {} {day}, {y} is not a {}",
            &head["course"],
            &a["month"],
            &a["weekday"]
        );
        let (Some(start), Some(end)) = (minutes(&a["start"]), minutes(&a["end"])) else {
            bail!("bad times in {:?}", whole.as_str());
        };
        let mode = a["mode"].to_lowercase();
        let mode = if mode.starts_with("cancel") {
            "Cancelled".to_owned()
        } else if mode.starts_with("in") {
            "In Person".to_owned()
        } else {
            let mut m = mode;
            m[..1].make_ascii_uppercase();
            m
        };
        exams.push(Exam {
            course_id: head["course"].to_owned(),
            section: head["section"].to_owned(),
            title,
            date: format!("{y:04}-{month:02}-{day:02}"),
            start,
            end,
            mode,
            location,
            instructors,
        });
        prev_end = whole.end();
    }
    validate(&text, &exams)?;
    exams.sort();
    exams.dedup();
    Ok(exams)
}

/// Every line that starts like a row must have become one.
fn validate(text: &str, exams: &[Exam]) -> Result<()> {
    static LINE_HEAD: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?m)^[ \t]*[0-9]{5}[ \t]+[A-Z0-9]{1,3}[ \t]").unwrap());
    let lines = LINE_HEAD.find_iter(text).count();
    let times = Regex::new(
        r"[0-9]{1,2}:[0-9]{2}\s*[AaPp]\.?\s*[Mm]\.?\s*[-\x{2013}]\s*[0-9]{1,2}:[0-9]{2}",
    )
    .unwrap()
    .find_iter(text)
    .count();
    ensure!(!exams.is_empty(), "no exams found");
    ensure!(
        times == exams.len(),
        "{times} time ranges but {} rows parsed",
        exams.len()
    );
    ensure!(
        lines <= exams.len(),
        "{lines} lines start like a row but only {} rows parsed",
        exams.len()
    );
    Ok(())
}

/// Replaces ICC color spaces with a device space. pdf-extract decompresses an ICC profile every
/// time a page selects its color space, which made the registrar's PDFs take seconds; colors don't affect the text.
fn drop_icc_profiles(obj: &mut pdf_extract::Object) {
    use pdf_extract::Object;
    match obj {
        Object::Array(items) => {
            if items.first().and_then(|o| o.as_name().ok()) == Some(b"ICCBased".as_slice()) {
                // pdf-extract keeps color operands as given, so the component count needn't match.
                *items = vec![Object::Name(b"DeviceRGB".to_vec())];
            } else {
                items.iter_mut().for_each(drop_icc_profiles);
            }
        }
        Object::Dictionary(dict) => dict.iter_mut().for_each(|(_, v)| drop_icc_profiles(v)),
        Object::Stream(stream) => stream
            .dict
            .iter_mut()
            .for_each(|(_, v)| drop_icc_profiles(v)),
        _ => {}
    }
}

/// Plain text like pdf-extract's `PlainTextOutput`, except that a gap wider than a column gap becomes a tab. The
/// schedules are spreadsheets, so a tab marks a cell boundary, such as between a room and the instructors.
struct ColumnText {
    out: String,
    page_height: f64,
    last_end: f64,
    last_y: f64,
    first_char: bool,
}

/// Gaps between cells are at least this many font sizes; spaces between words are about a quarter.
const COLUMN_GAP: f64 = 1.0;

impl pdf_extract::OutputDev for ColumnText {
    fn begin_page(
        &mut self,
        _page: u32,
        media_box: &pdf_extract::MediaBox,
        _art_box: Option<(f64, f64, f64, f64)>,
    ) -> Result<(), pdf_extract::OutputError> {
        self.page_height = media_box.ury - media_box.lly;
        Ok(())
    }

    fn end_page(&mut self) -> Result<(), pdf_extract::OutputError> {
        Ok(())
    }

    fn output_character(
        &mut self,
        trm: &pdf_extract::Transform,
        width: f64,
        _spacing: f64,
        font_size: f64,
        ch: &str,
    ) -> Result<(), pdf_extract::OutputError> {
        let (vx, vy) = (
            font_size * trm.m11 + font_size * trm.m21,
            font_size * trm.m12 + font_size * trm.m22,
        );
        let size = (vx * vy).sqrt();
        let (x, y) = (trm.m31, self.page_height - trm.m32);
        if self.first_char {
            let dy = (y - self.last_y).abs();
            let mut newline = false;
            if dy > size * 1.5 || (x < self.last_end && dy > size * 0.5) {
                self.out.push('\n');
                newline = true;
            }
            if x > self.last_end + size * COLUMN_GAP && !newline {
                self.out.push('\t');
            } else if x > self.last_end + size * 0.1 {
                self.out.push(' ');
            }
        }
        self.out.push_str(ch);
        self.first_char = false;
        self.last_y = y;
        self.last_end = x + width * size;
        Ok(())
    }

    fn begin_word(&mut self) -> Result<(), pdf_extract::OutputError> {
        self.first_char = true;
        Ok(())
    }

    fn end_word(&mut self) -> Result<(), pdf_extract::OutputError> {
        Ok(())
    }

    fn end_line(&mut self) -> Result<(), pdf_extract::OutputError> {
        Ok(())
    }
}

/// The text of a PDF, with tabs between table cells.
pub fn extract_text(bytes: &[u8]) -> Result<String> {
    let mut doc = pdf_extract::Document::load_mem(bytes).context("reading the PDF")?;
    if doc.is_encrypted() {
        bail!("the PDF is encrypted");
    }
    for obj in doc.objects.values_mut() {
        drop_icc_profiles(obj);
    }
    let mut text = ColumnText {
        out: String::new(),
        page_height: 0.0,
        last_end: 100_000.0,
        last_y: 0.0,
        first_char: false,
    };
    // pdf-extract panics on some malformed files rather than returning an error.
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pdf_extract::output_doc(&doc, &mut text)
    }))
    .map_err(|_| anyhow::anyhow!("the PDF text extractor panicked"))?
    .context("extracting text")?;
    Ok(text.out)
}

/// `<term>.pdf` or `<term>-<part>.pdf`: the term id.
fn term_of(file: &str) -> Option<&str> {
    let stem = file.strip_suffix(".pdf")?;
    let id = stem.split_once('-').map_or(stem, |(id, _)| id);
    let b = id.as_bytes();
    (b.len() == 3 && matches!(b[0], b'S' | b'M' | b'F') && b[1..].iter().all(u8::is_ascii_digit))
        .then_some(id)
}

/// Every term's exams, from the PDFs in `dir`.
pub fn read_dir(dir: &Path) -> Result<BTreeMap<String, Vec<Exam>>> {
    let mut files: Vec<(String, String)> = Vec::new();
    if dir.exists() {
        for e in std::fs::read_dir(dir).with_context(|| format!("listing {}", dir.display()))? {
            let name = e?.file_name().to_string_lossy().into_owned();
            if let Some(term) = term_of(&name) {
                files.push((term.to_owned(), name));
            }
        }
    }
    files.sort();
    let parsed = files
        .par_iter()
        .map(|(term, name)| -> Result<(String, Vec<Exam>)> {
            let path = dir.join(name);
            let bytes =
                std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
            let year = 2000 + term[1..].parse::<i64>()?;
            let exams = parse(&extract_text(&bytes)?, year)
                .with_context(|| format!("parsing {}", path.display()))?;
            info!(file = %name, exams = exams.len(), "final exams");
            Ok((term.clone(), exams))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut out: BTreeMap<String, Vec<Exam>> = BTreeMap::new();
    for (term, exams) in parsed {
        let list = out.entry(term).or_default();
        list.extend(exams);
        list.sort();
        list.dedup();
    }
    Ok(out)
}

/// The public version: no instructors.
pub fn redacted(exams: &[Exam]) -> Vec<Exam> {
    exams
        .iter()
        .cloned()
        .map(|mut e| {
            e.instructors.clear();
            e
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINES: &str = "
Spring 2026 Semester & Mini-4 Final Examination Schedule
April 27, 28 , 30 May 1 & 4 (Make Up Day: May 5)

Architecture

Course Sec Title Date Time (EST USA) Delivery Mode Exam Assigned Space(s) Instructor(s)

48240 A HIS WLD ARC Tuesday, April 28, 2026 01:00pm - 04:00pm In Person TBD -assigned after mini-4 add deadline SHAW DIANE (dshaw)

48721 A BCD Thursday, April 30, 2026 01:00pm - 04:00pm In Person TBD BETZ FREDERIK (fbetz), O'NEIL T. ANN-MARIE (taon)

70340 D BUSINESS COMMUNCTNS Tuesday, April 28, 2026 01:00pm - 04:00pm Cancelled Cancelled 3/2/2026 DEJEU EMILY (edejeu)
82343 A CLTRS LATIN AMRCA Friday, May 1, 2026 05:30pm - 08:30pm CANCELED CANCELED GOMEZ FELIPE (fgomez)
24609 A ST: ENTERTAIN ENG. Friday, May 1, 2026 08:30am - 11:30am Remote Remote BEDILLION MARK (capn), HINES KEVIN (hines), ...
University Registrar's Office Page 1 of 16 As of: 3/24/2026
";

    #[test]
    fn reads_rows_on_lines() {
        let exams = parse(LINES, 2026).unwrap();
        assert_eq!(exams.len(), 5);
        let e = &exams[1];
        assert_eq!(
            (
                e.course_id.as_str(),
                e.section.as_str(),
                e.title.as_str(),
                e.date.as_str()
            ),
            ("48240", "A", "HIS WLD ARC", "2026-04-28")
        );
        assert_eq!((e.start, e.end, e.mode.as_str()), (780, 960, "In Person"));
        assert_eq!(
            e.location.as_deref(),
            Some("TBD -assigned after mini-4 add deadline")
        );
        assert_eq!(e.instructors, ["SHAW DIANE"]);
        assert_eq!(
            exams[2].instructors,
            ["BETZ FREDERIK", "O'NEIL T. ANN-MARIE"]
        );
        assert_eq!(exams[2].location.as_deref(), Some("TBD"));
        let cancelled = &exams[4];
        assert_eq!(
            (cancelled.course_id.as_str(), cancelled.mode.as_str()),
            ("82343", "Cancelled")
        );
        assert_eq!(cancelled.location.as_deref(), Some("CANCELED"));
        assert_eq!(cancelled.instructors, ["GOMEZ FELIPE"]);
        assert_eq!(exams[0].title, "ST: ENTERTAIN ENG.");
        assert_eq!(exams[0].instructors, ["BEDILLION MARK", "HINES KEVIN"]);
        assert_eq!(exams[3].location.as_deref(), Some("Cancelled 3/2/2026"));
    }

    #[test]
    fn reads_rows_without_line_breaks_or_years() {
        let text = "Fall 2026 Mini-1 Final Examination Schedule Saturday, October 10, 2026 Architecture  Course Sec Title Date Time (EST USA) Delivery Mode Assigned Exam Space(s) Instructor(s) 48628 A1 GEOAI Saturday, October 10, 2026 08:30AM - 11:30AM In Person WEH 5202 LI SUZY (zekunl) Civil & Environmental Engineering  Course Sec Title Date Time 12746 A1 ST: PYTHON PROTOTYP Saturday, October 10 01:00 p.m. - 04:00 p.m. Remote To Be Announced ROSEN, RACHEL (rrosen2)Questions: Joy";
        let exams = parse(text, 2026).unwrap();
        assert_eq!(exams.len(), 2);
        assert_eq!(exams[0].course_id, "12746");
        assert_eq!(
            (exams[0].start, exams[0].end, exams[0].mode.as_str()),
            (780, 960, "Remote")
        );
        assert_eq!(exams[0].location.as_deref(), Some("To Be Announced"));
        assert_eq!(exams[0].instructors, ["ROSEN, RACHEL"]);
        assert_eq!(exams[1].location.as_deref(), Some("WEH 5202"));
        assert_eq!(exams[1].instructors, ["LI SUZY"]);
        assert_eq!(exams[1].date, "2026-10-10");
    }

    #[test]
    fn splits_location_from_instructors_at_cell_boundaries() {
        let tabbed = "14741\t1\tINTRO INFO SECURITY\tFriday, May 1, 2026\t01:00pm - 04:00pm\tIn Person\tCIC DEC\tKHAN TAHA (mtk2)
15210\t2\tPRL SEQ DATA STR ALG\tFriday, May 1, 2026\t01:00pm - 04:00pm\tIn Person\tTBD\tINSTRUCTOR TBA (instructor)";
        let exams = parse(tabbed, 2026).unwrap();
        assert_eq!(exams[0].location.as_deref(), Some("CIC DEC"));
        assert_eq!(exams[0].instructors, ["KHAN TAHA"]);
        assert_eq!(exams[1].location.as_deref(), Some("TBD"));
        assert!(exams[1].instructors.is_empty());
    }

    #[test]
    fn rejects_wrong_weekdays_and_unknown_rows() {
        assert!(
            parse(
                "15122 A X Monday, April 28, 2026 01:00pm - 04:00pm In Person TBD",
                2026
            )
            .is_err()
        );
        assert!(
            parse(
                "15122 A X Tuesday, April 28, 2026 01:00pm - 04:00pm Somewhere TBD",
                2026
            )
            .is_err()
        );
        assert!(parse("nothing here", 2026).is_err());
    }

    /// The committed schedules. Slow in debug builds: `cargo test --release -p courses-index -- --ignored`.
    #[test]
    #[ignore]
    fn reads_the_committed_pdfs() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/finals");
        let exams = read_dir(&dir).unwrap();
        assert_eq!(exams["S26"].len(), 583);
        assert_eq!(exams["F26"].len(), 592 + 12);
        let e = exams["S26"]
            .iter()
            .find(|e| e.course_id == "48240")
            .unwrap();
        assert_eq!((e.date.as_str(), e.start, e.end), ("2026-04-28", 780, 960));
        assert_eq!(e.instructors, ["SHAW DIANE"]);
    }

    #[test]
    fn file_names() {
        assert_eq!(term_of("F26.pdf"), Some("F26"));
        assert_eq!(term_of("F26-mini1.pdf"), Some("F26"));
        assert_eq!(term_of("exams.pdf"), None);
        assert_eq!(term_of("F26.txt"), None);
    }
}
