# Source data audit — October 23, 2026

The entire supplied `/Users/etashj/Downloads/data` directory was copied into
`data/raw`. **90,778 non-DS_Store files, 1,718,136,834 bytes** were checked against
a SHA-256 inventory with zero mismatches. The original Downloads folder is
unchanged. `data/catalog/inventory.json` records every file, including old backups;
`data/catalog/report.json` records the import counts and warnings. Raw and derived
data are local artifacts, not committed or placed in the frontend's static folder.

## Coverage and freshness

| Dataset | Available coverage | Interpretation |
| --- | --- | --- |
| Courses | 10,089 distinct merged course codes | Stellic course info plus legacy course records; not all are currently offered |
| Schedules | 48,796 legacy course/term records; 100,541 merged section records | Legacy export includes 3,355 course records for Fall 2026, 3,315 for Spring 2026, and 114 for Summer 2026 |
| Stellic sections | Dated starts August 30, 2021–June 30, 2025 | Older than the legacy schedule export; dates come from records, not `ly*` filenames |
| Offering metadata | 2015–Fall 2026 | Spring 2027 is absent; metadata alone does not establish a complete section schedule |
| FCE CSV | 90,052 rows, 2012–2026; 3,566 rows in 2026 | This is newer than the Mongo FCE export and is the selected aggregate source |
| Mongo FCEs | 447,411 rows, 2012–2025 | Retained, not combined with CSV; repeated historical records would bias aggregates |
| Gen ed mappings | 1,679 course rows, 35 school/category groups | 758 rows labeled October 2024, 319 May 2025, 602 undated |
| Syllabi | 19,794 links, 2018–2025 | Imported for authenticated users; link availability was not individually tested |
| Instructors | 3,534 Stellic identities | Catalog identity coverage is incomplete relative to names in legacy schedules; legacy professor export is retained |
| Stellic programs | 2,848 audits for 1,291 program IDs | Includes historical entry-year variants; newest explicitly labeled entry year is 2026; no verified retrieval timestamp |
| Current public requirements | 204/204 discovered program pages | Scraped October 23, 2026, all labeled **2026–2027**, zero failed pages; includes majors, additional majors and minors |
| Advanced credit | AP, IB, Cambridge and non-offered-course files | Preserved in raw data; not yet exposed as scheduling equivalency rules |
| Finals | Existing repository `data/exams.pdf` | Not imported; no verified current finals feed in this migration |

File modification/copy timestamps are **not** evidence of scrape freshness.
Fall 2026 schedule coverage is useful, but its exact retrieval date is unknown.
The newer public requirements come from the official
[Programs A–Z](https://coursecatalog.web.cmu.edu/programs/) and are archived in
`data/programs-current` with source URLs, HTML hashes and retrieval timestamps.
This verifies all links discovered by the scraper, not every possible graduate
or unpublished program.

## Import choices and limitations

- Prefer Stellic course details; retain legacy-only course codes and department
  labels. Prefer dated Stellic sections for overlapping course/terms; retain
  newer legacy terms. Keep only explicit fields intended for the app.
- FCE summaries are respondent-weighted across the available CSV history;
  teaching-assistant rows are excluded. They are not “current semester ratings.”
- Two Stellic sections lack dates (`11605/ly3_sm3.json`,
  `36641/ly3_sm3.json`) and are omitted from the normalized schedule, with warnings.
  Their source files are retained.
- Raw exports may include student-specific context. Those fields are not copied
  to the public/member course payloads. All older snapshots, legacy professors,
  raw FCE rows, advanced-credit data and ancillary files remain available locally.
- Variable/unknown-unit courses currently use the numeric source value where
  available, otherwise zero. A future range/variable-units UI should distinguish
  these rather than treating zero as a confirmed fixed unit count.
- The requirements UI shows source text or versioned trees and stores the chosen
  program with a personal schedule. It does not yet certify degree completion,
  elective eligibility, substitutions, or double-counting rules.

For registration-quality planning, refresh sections for the target term,
especially Spring 2027. The existing authenticated Stellic scraper supports
courses and program requirements; commands and access requirements are in
[backend setup](./sqlite-backend.md). No live Stellic session was used here.

## Spring 2027 refresh from CourseWeave

Imported the public [CourseWeave S27 snapshot](https://courseweave.jay3332.tech/data/S27.json): **3,085 courses and 6,760 sections**, across all departments. The snapshot reports generation at **2026-10-24 00:37:00 UTC (October 23, 8:37 p.m. EDT)** and upstream sources `csv` and `stellic`. This is the publisher's generation timestamp, not independently verified collection dates or a guarantee of final CMU offerings. Sections do not include individual start/end dates; term metadata identifies Spring 2027. Existing descriptions, requirements and FCE history retain their older provenance.

Original JSON, semester metadata, retrieval time and SHA-256 are preserved in ignored `data/raw/courseweave/`. The importer automatically includes that snapshot when present; `--courseweave PATH` selects another S27 snapshot. It validates semester, unique course/section IDs, minute-of-day times and Monday-based weekday indices. Missing times remain TBA. Public catalogs still omit instructors, FCE and locations; member catalogs preserve section instructors and rooms. No Stellic credentials are sent to CourseWeave.

Rebuild and verify:

```sh
python3 scripts/import_catalog.py data/raw
python3 scripts/verify_catalog.py
```
