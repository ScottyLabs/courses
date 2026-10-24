//! Courseweave's CSV reader (`parseCsv` and `readCsvRecords` in `scripts/lib/common.ts`), ported so quirks such as
//! skipped blank lines and trimmed cells match.

use std::collections::HashMap;

use super::js;

/// Splits CSV text into rows of fields. Quoted fields may contain commas, newlines and doubled quotes; `\r\n`, `\r`
/// and `\n` end a row; rows that are a single empty field (blank lines) are skipped.
pub fn parse(text: &str) -> Vec<Vec<String>> {
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

/// Rows keyed by their trimmed header, with trimmed cells, like `readCsvRecords`.
pub struct Records {
    columns: HashMap<String, usize>,
    rows: Vec<Vec<String>>,
}

impl Records {
    pub fn parse(text: &str) -> Self {
        let mut rows = parse(text).into_iter();
        let header = rows.next().unwrap_or_default();
        // `Object.fromEntries` keeps the last column of a repeated name.
        let columns = header
            .iter()
            .enumerate()
            .map(|(i, h)| (js::trim(h).to_owned(), i))
            .collect();
        let width = header.len();
        let rows = rows
            .map(|cells| {
                (0..width)
                    .map(|i| cells.get(i).map_or("", |c| js::trim(c)).to_owned())
                    .collect()
            })
            .collect();
        Self { columns, rows }
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// The column's index, or `None` when the header doesn't have it (the record's value is `undefined`).
    pub fn column(&self, name: &str) -> Option<usize> {
        self.columns.get(name).copied()
    }

    pub fn rows(&self) -> impl Iterator<Item = Record<'_>> {
        self.rows.iter().map(|cells| Record { cells })
    }
}

pub struct Record<'a> {
    cells: &'a [String],
}

impl<'a> Record<'a> {
    /// The cell in a column from [`Records::column`]; `None` stands for `undefined`.
    pub fn get(&self, column: Option<usize>) -> Option<&'a str> {
        column.map(|i| self.cells[i].as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quotes_newlines_and_blank_lines() {
        let rows = parse("a,b\r\n\"x, \"\"y\"\"\",\"line\nbreak\"\n\n1,2");
        assert_eq!(
            rows,
            vec![
                vec!["a", "b"],
                vec!["x, \"y\"", "line\nbreak"],
                vec!["1", "2"]
            ]
        );
    }

    #[test]
    fn keys_records_by_trimmed_headers() {
        let records = Records::parse("\u{FEFF}Year , Num\n 2026 ,15122\n2025\n");
        let year = records.column("Year");
        let num = records.column("Num");
        let rows: Vec<_> = records.rows().map(|r| (r.get(year), r.get(num))).collect();
        assert_eq!(
            rows,
            vec![(Some("2026"), Some("15122")), (Some("2025"), Some(""))]
        );
        assert_eq!(
            records.rows().next().unwrap().get(records.column("Sem")),
            None
        );
    }
}
