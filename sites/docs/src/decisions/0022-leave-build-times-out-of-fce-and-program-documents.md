# 22. Leave build times out of FCE and program documents

Date: 2026-10-24

## Status

Accepted

## Context

Courseweave's FCE summary, FCE instructor file and program index each carry `generatedAt`, the wall-clock time of
the build. Nothing in Courseweave reads it.

A catalog built from the same inputs must be byte-identical ([decision 15](0015-build-the-catalog-in-rust.md)), and
each document's ETag is the SHA-256 of its JSON ([decision 16](0016-serve-the-catalog-as-json-documents.md)). A
timestamp would give every FCE and program document a new ETag on every build, so browsers would download about
7 MB (gzipped) of unchanged FCE and program data again after each release.

## Decision

`fce/summary`, `fce/instructors` and `programs/index` leave out `generatedAt`. A build's time is recorded once, in
the catalog's `meta` and its `build` document. The ported TypeScript types drop the field or make it optional
(`ProgramsIndex.generatedAt` in `plan-types.ts`).

## Consequences

- A document changes only when its content does, so revalidation gets 304 for unchanged FCEs and programs
- The documents' contents otherwise match Courseweave's files, which the parity test checks with `generatedAt`
  ignored
- Readers that want to know how fresh FCEs are use `latest` in `fce/summary`, the newest term in the export
