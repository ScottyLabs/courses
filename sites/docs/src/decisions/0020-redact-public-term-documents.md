# 20. Redact public term documents

Date: 2026-10-24

## Status

Accepted

## Context

Each catalog document has a public version and may have a member version for signed-in CMU users
([decision 16](0016-serve-the-catalog-as-json-documents.md)). The semester documents carry who teaches each section
and where it meets. Those come from Stellic, which needs a CMU login, and the existing public API already leaves them
out. The website's ported modules read the documents with Courseweave's types, which expect every field to be there.

## Decision

- `term/<id>`'s public version is the member version with `instructors` emptied on every course and section and
  `building` and `room` set to `null` on every meeting. Everything else is the same, including meeting times,
  capacities, reservations, cross-listings and the term's notes, which are computed from the full data.
- `finals/<id>`'s public version empties `instructors`.
- Documents built from member-only sources have no public version: `teaching` (who teaches what) and `syllabi/<NN>`.
  The catalog (`catalog/index`, `catalog/details/<NN>`), `offerings` and `terms` are public.
- The relational tables in `catalog.sqlite` hold public fields only.
- The builder writes both versions; the member version is stored only when it differs.

## Consequences

- Public and member documents have the same shape, so the same code reads both
- A signed-out visitor gets empty instructor lists and no rooms, not a different format, so the website should say
  that signing in shows them
- Cross-listings found from shared rooms are public, though the rooms aren't
- New fields default to public, so adding a member-only field means adding it to the redaction and its test
