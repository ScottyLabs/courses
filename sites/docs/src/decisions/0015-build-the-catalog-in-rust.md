# 15. Build the catalog in Rust, as one SQLite file

Date: 2026-10-24

## Status

Accepted. Amends [decision 10](0010-build-the-catalog-separately-from-scraping.md): the build is still separate from
scraping and deterministic, but it now produces one `catalog.sqlite` instead of a set of files. Supersedes
`scripts/import_catalog.py`, `scripts/verify_catalog.py` and `scripts/courseweave.py`.

## Context

Catalogs have been built by `scripts/import_catalog.py` from a one-off folder of exports (`data/raw/`) that only one
maintainer has. `courses-index` was meant to build the catalog from the scraper's own output, but it only loads the
data; writing the catalog is `unimplemented!()`. Courseweave's TypeScript scripts build the data its features need
from the same scraper output, plus CMU's public Schedule of Classes (SOC), the FCE CSV and the syllabus registry.

## Decision

- `courses-index build` reads the raw inputs and writes `catalog.sqlite`. It ports Courseweave's build scripts, so
  its JSON documents match the files those scripts write, byte for byte where practical.
  - Inputs: the scraper's `exported/` tree (Stellic course history, programs, syllabi), SOC captures from the
    scraper's new `soc` mode, schedule CSVs, `data/fces.csv`, the registrar's final exam schedules, and
    `data/terms.json` (term dates, minis and the sources for each term).
- `catalog.sqlite` holds:
  - `documents(name, audience, etag, bytes, gzip)`: the JSON documents the website loads
    ([decision 16](0016-serve-the-catalog-as-json-documents.md)), each stored gzipped once per audience
  - relational tables (`terms`, `courses`, `offerings`, `sections`, `meetings`) for server-side queries, such as
    the public course API and validating a schedule's sections. These hold public fields only
  - `meta`: the schema version, build time and a summary of the inputs
- Release publishing and installation keep working as before, with one file instead of six.

## Consequences

- One person's local export folder is no longer needed to build a catalog; the scraper's output is enough
- The relational tables and the documents come from the same build, so they can't disagree
- A catalog built from the same inputs is byte-identical, as decision 10 requires: documents are serialized from
  sorted collections and gzipped without timestamps
- The old importer's sources that the scraper doesn't produce (the legacy Mongo exports and gen ed spreadsheets) are
  dropped. Gen ed membership now comes from the degree programs in Stellic, which list gen ed requirements as
  requirements
