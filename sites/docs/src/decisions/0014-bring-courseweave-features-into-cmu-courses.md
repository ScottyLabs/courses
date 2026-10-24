# 14. Bring Courseweave's features into CMU Courses

Date: 2026-10-24

## Status

Accepted

## Context

[Courseweave](https://courseweave.jay3332.tech) is a separate CMU course explorer, schedule planner and degree
planner. It already does most of what this project's specification asks for: requisite trees checked against a
student's record, FCE charts, every section with its meetings, schedules with conflict detection and calendar export,
a requirement audit over Stellic's degree programs, a semester-by-semester plan, imports of a student's SIO and
Stellic records, past syllabi, and course pages for search engines.

Its stack doesn't match ours. It is a React app with a Cloudflare Worker backend, its data pipeline is TypeScript
scripts, and it keeps personal data in the browser and syncs it to an account. CMU Courses is a Svelte app with a Rust
backend, deployed on Kennel, and follows the [Figma redesign](https://www.figma.com/design/vsvdhGC2QP5fsvG0Pl7W6v/CMUCourses-Redesign--summer-25---Copy-?node-id=502-10393).

## Decision

We'll port Courseweave's features into this repository, so that CMU Courses does everything Courseweave does and
more. CMU Courses adds friends and the official final exam schedule, which Courseweave doesn't have.

- The frontend stays SvelteKit and follows the Figma design. Courseweave's React components are rewritten in
  Svelte. Its UI-free TypeScript modules (audit, plan, requisites, search, sections, conflicts, calendar export,
  record parsers) are copied into `sites/web/src/lib/core` with their tests.
- The backend is Rust. Courseweave's data scripts become the catalog builder in `courses-index`
  ([decision 15](0015-build-the-catalog-in-rust.md)), and its Worker routes become `courses-web-api` handlers.
- We keep Courseweave's data formats wherever the frontend reads them, so ported modules work unchanged
  ([decision 16](0016-serve-the-catalog-as-json-documents.md)).
- Personal data lives in the web API's database ([decision 17](0017-store-personal-data-on-the-server.md)).
- Where the Figma design has no screen for a Courseweave feature, such as the degree plan, we design one in the same
  visual language. Where the two disagree, these changes apply:
  - There's no separate gen eds page. Requirements, gen eds included, are a filter on the course list.
  - Saved courses are a filter on the course and instructor lists, not a separate tab.
  - Every list of courses or instructors can switch between the Figma cards and a compact one-column list, like
    Courseweave's.

[Courseweave features in CMU Courses](../courseweave.md) maps each feature to where it lives here.

## Consequences

- Two codebases implement the same features for a while; fixes in either may need porting
- Courseweave's tests come along with its modules, so the ported logic keeps its coverage
- The ported TypeScript modules don't follow this repository's Prettier style until reformatted
