# 21. Sort catalog strings with Unicode collation

Date: 2026-10-24

## Status

Accepted

## Context

Courseweave's build scripts order several arrays with JavaScript's `localeCompare`: FCE rows by section and
instructor, the degree program index by name, and tied concentration matches by name. In Node, `localeCompare`
uses ICU's collation for the default locale, English, which is the Unicode root collation. It orders strings
differently from byte order: case counts only between otherwise equal letters ("abc" before "ABD", and
"ABASCAL, Brooke" before "ABASCAL, BROOKE"), punctuation sorts before digits and digits before letters ("(Music) Solfege" before "1st Grad
Recital"), and accented letters sort next to their base letters.

These arrays are part of the documents' content: the website reads FCE rows and programs in document order, and the
audit uses a concentration's first match. The Rust builder has to produce the same order to produce the same
documents ([decision 15](0015-build-the-catalog-in-rust.md)).

## Decision

Where a Courseweave build script compares strings with `localeCompare`, the builder compares them with ICU4X's
root collator (the `icu_collator` crate with its compiled data) at its default, tertiary strength, the same
settings `localeCompare` uses. The helper is `courses_index::fce::js::locale_compare`; other ported builders use it
too. Other comparisons are ported as they are.

## Consequences

- The FCE and program documents have Courseweave's order, checked against its files by the parity test
- Builds don't depend on the machine's ICU or locale: the collation data is compiled into the builder
- The builder carries ICU4X's compiled collation and normalization data
- Node and ICU4X can ship different CLDR versions, which could order rare scripts differently. Program names and
  FCE fields are Latin text, where the versions agree
