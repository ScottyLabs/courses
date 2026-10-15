# Running the scraper

## CLI

| Flag | Env | Default | Purpose |
| --------------- | ------------- | --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--mode` | `MODE` | `courses` | One of `courses`, `programs`, `syllabi`, `ldap`, `all` (`all` runs the three scrape pipelines in parallel, then the directory pipeline). |
| `--export-root` | `EXPORT_ROOT` | `exported` | Root for all output. Scrape pipelines write under `<export_root>/<username>/`, syllabi under `<export_root>/syllabi/`, the directory aggregate at `<export_root>/ldap.json`. |
| `--fce-path` | `FCE_PATH` | `data/fces.csv` | SmartEvals FCE CSV (used in `courses` and `all`). |
| `--concurrency` | `CONCURRENCY` | `32` | Worker count for the rayon pool that runs HTTP tasks. |
| `--incremental` | `INCREMENTAL` | off | Skip any per-course/per-audit output file that already exists locally. |

Log filtering is via `RUST_LOG` (tracing `EnvFilter`); the default level is `info`.

The scraper reads credentials from the environment. `COOKIE_HEADER` is the Stellic full `Cookie:` header value, prompted interactively when unset. The cookie's session identifies the user and names the per-student output subdir. `CANVAS_TOKEN` is required for `syllabi` and `all` and can be generated at `https://canvas.cmu.edu/profile/settings`. `CMU_LDAP_PASSWORD` is required for `ldap` and optional for `all` (skipped with a warning when unset). The scrape workflow takes `COOKIE_HEADER` and `CANVAS_TOKEN` as dispatch inputs and resolves `CMU_LDAP_PASSWORD` and the `CDN_*` secrets from the OpenBao `ci` profile, which a project lead sets with `secretspec set -P ci <NAME>`. In dev the three scraper credentials live in a gitignored `.env`, declared in `secretspec.toml`'s dev profile under the `local` provider.

## Output layout

Course pipeline:

```
<export_root>/<username>/courses_history/
  <course_code_no_dash>/
    info.json                 # /catalog/getcourseinfo/ response with user-state stripped
    ly<lyear>_sm<sem_id>.json # /planner/getcoursesections/ response, one per (lyear, sem)
```

The course directory uses the dash-stripped form (e.g. `21122` for 21-122). Re-runs overwrite unless `--incremental` is set, which skips any output file that already exists.

`info.json` and the sections file each strip several user-state fields from the upstream response before writing; see [`stellic.md`](./stellic.md) for exactly which fields and why. When `data_list` is empty, the scraper writes nothing, so the absence of a sections file means Stellic returned no sections for that (course, lyear, sem).

Programs pipeline:

```
<export_root>/<username>/programs/
  <catalog_id>/
    <audit_id>.json   # one audit version of one catalog program
```

Each file wraps the matching `req_tree.programs[]` subtree with audit and program metadata, plus the non-personal scalar fields from the audit response:

```
{
  "catalog_id": <int>,                       // /catalog/getprograms/ id
  "program_name": <str>,                     // catalog program name
  "program_type": <int>,                     // 1=major, 2=minor, 3=add'l major, 4=sub-req bundle, 5=eligibility
  "audit_id": <int>,                         // audit publication id from getauditversions/
  "audit_name": <str>,                       // e.g. "EY2021 Pittsburgh - BS in Mathematical Sciences"
  "requirement_id": <int>,                   // surfaces as req_tree.programs[].id
  "is_combination": <bool>,                  // audit-level flag from the response
  "free_electives_req": <obj|null>,          // free electives spec for this audit version
  "program_reqs": <obj|null>,                // program_reqs[<requirement_id>] only
  "unique_course_parents_mapping": <obj>,    // filtered: only entries that point to this audit version
  "tree": { id, screen_name, constraints, choices, ... }   // the matching req_tree.programs[] node
}
```

Stripped before writing: `audit_data`, top-level `programs`, `course_plan_info`, `placeholders_info`, `unmatched`, `notcounted`, `unmatched_slots`, `permissions`, `program_permissions`, `student_audit`, `full_gpa`, `student_enrollment_levels`, `plan_diplomas`, `remaining_reqs_details`, `last_computed`, `debug_info`, `uni_req_programs`. Cross-reference fields (`unique_course_parents_mapping`, `program_reqs`) are filtered to keys that match the requested audit version, since unfiltered they would also include the caller's auto-attached gen-ed audits and leak the caller's college affiliation.

Syllabi pipeline:

```
<export_root>/syllabi/
  <term_code>/                   # e.g. S26, F24, M25, N23
    <dept_code>/                 # e.g. ARC, CS, ART
      <course_section>.<ext>     # File items: original PDF/DOC
      <course_section>.url       # Page items: plain-text Canvas page URL
```

The pipeline only saves items in each sub-course's "Available Syllabi" module. `File` items are downloaded with their original filename extension. `Page` items are saved as a single-line `.url` file containing the Canvas page URL where the syllabus is rendered, since the page body itself is just a redirect stub and the target course's `syllabus_body` is empty most of the time and ambiguous when populated. Downstream consumers can either link directly to the URL or follow it for users who can authenticate. "Unavailable Syllabi" and "Individualized Experiences" modules are skipped because their items are placeholders without retrievable content. See [`syllabi.md`](./syllabi.md) for the registry structure and the rationale.

The syllabi pipeline has two skip layers. The cumulative one: before scraping, the scraper lists the objects under `syllabi/` in the CDN bucket using the `CDN_*` secrets, and any (term, dept, course_section) already there is skipped before its download, regardless of extension; the scrape workflow syncs the local tree back up after every run, so the bucket accumulates syllabi across runs and students. Only positive presence is ever recorded; a section with no syllabus is re-checked on every run, since one can be uploaded to Canvas at any time. The local one: with `--incremental`, `save_task` also skips when `<course_section>.*` already exists under `<term>/<dept>/` in the local output tree.

Directory pipeline:

```
<export_root>/ldap.json   # one aggregate map: lowercased AndrewID -> directory attributes
```

`--mode ldap` walks the existing `courses_history` output under every student subdir, collects every instructor AndrewID, and resolves them against the CMU directory in one bound LDAP session (see [`ldap.md`](./ldap.md)). `--mode all` runs it as a final step after the three scrape pipelines. Re-runs overwrite.

## Concurrency

A custom rayon thread pool sized by `--concurrency` runs every pipeline. When `--mode all`, the three pipelines run concurrently within the same pool via nested `rayon::join`, sharing the thread budget; their outputs go to separate directories so they don't collide. Stellic and Canvas each keep their own `ureq::Agent`, which pools connections for that host.

Course pipeline:

1. Startup discovery uses `rayon::join` to overlap the FCE parse (single-threaded CSV read) with the four SOC fetches, which themselves run as a rayon par_iter across the four seasons.
1. Task execution sends one HTTP GET per task (course info or course sections), so `--concurrency` directly bounds in-flight HTTP requests. Progress is logged every 500 completed tasks.

Programs pipeline:

1. Audit-version discovery: one `GET /planner/getauditversions/` per catalog program in parallel, building the task list. Progress is logged every 200 programs.
1. Audit fetch: one `POST /planner/getauditinfo/` (test-apply mode) per task in parallel. Progress is logged every 200 completed tasks.

Step 2 of the programs pipeline is heavier than any course-pipeline call, since the server computes the full audit, so its tolerable concurrency is lower than the course endpoint's.

Syllabi pipeline:

1. Sub-course discovery: walks the master `syllabus-registry` course's term modules, resolves each dept's `sis_course_id`, and lists each sub-course's modules to gather `Available Syllabi` items.
1. Item save: for each `File` item, fetches file metadata then downloads the file; for each `Page` item, writes the page URL to a `.url` file.

Empirical concurrency findings:

- Programs pipeline (`getauditinfo` test-apply): best throughput at concurrency 32. The default is 32.
- Course pipeline (`getcourseinfo`, `getcoursesections`): handles concurrency 64 cleanly. If running the course pipeline alone, raising `--concurrency` to 64 is safe.
- Syllabi pipeline (Canvas file downloads): plateaus around concurrency 32. 64 and 128 do not improve throughput.
