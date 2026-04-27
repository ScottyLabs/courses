# Running the scraper

## CLI

| Flag | Env | Default | Purpose |
|---|---|---|---|
| `--andrew-id` | `ANDREW_ID` | (required) | Andrew ID sent to `getstudentprofile/`. |
| `--cookie-header` | `COOKIE_HEADER` | (prompt) | Full `Cookie:` header value. If absent, the scraper prompts on stderr with instructions. |
| `--mode` | `MODE` | `both` | One of `courses`, `programs`, `both`. Selects which pipeline runs. |
| `--fce-path` | `FCE_PATH` | `data/fces.csv` | SmartEvals FCE CSV (used in `courses` and `both`). |
| `--out-dir` | `OUT_DIR` | `data/courses_history` | Course pipeline output root. |
| `--programs-dir` | `PROGRAMS_DIR` | `data/programs` | Programs pipeline output root. |
| `--concurrency` | `CONCURRENCY` | `32` | Worker count for the rayon pool that runs Stellic tasks. |
| `--limit` | `LIMIT` | (no limit) | Cap on number of tasks, for smoke tests. Applied to both pipelines when `--mode both`. |

Log filtering is via `RUST_LOG` (tracing `EnvFilter`); the default level is `info`.

## Output layout

Course pipeline:

```
<out_dir>/
  <course_code_no_dash>/
    info.json                 # /catalog/getcourseinfo/ response with user-state stripped
    ly<lyear>_sm<sem_id>.json # /planner/getcoursesections/ response, one per (lyear, sem)
```

The course directory uses the dash-stripped form (e.g. `21122` for 21-122). Re-runs overwrite, and there is no incremental skip.

`info.json` strips `student_context`, `enrollment_action_windows`, and `alerts` from the upstream response before writing, since those reflect user state rather than catalog data. The sections file strips the `current` field from each `data_list` entry. When `data_list` is empty, the scraper writes nothing, so the absence of a sections file means Stellic returned no sections for that (course, lyear, sem).

Programs pipeline:

```
<programs_dir>/
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

## Concurrency

A custom rayon thread pool sized by `--concurrency` runs both pipelines. When `--mode both`, the two pipelines run concurrently within the same pool via `rayon::join`, sharing the thread budget; their outputs go to separate directories so they don't collide. A shared `ureq::Agent` provides the HTTP connection pool internally.

Course pipeline:

1. Startup discovery uses `rayon::join` to overlap the FCE parse (single-threaded CSV read) with the four SOC fetches, which themselves run as a rayon par_iter across the four seasons.
2. Task execution sends one HTTP GET per task (course info or course sections), so `--concurrency` directly bounds in-flight HTTP requests. Progress is logged every 500 completed tasks.

Programs pipeline:

1. Audit-version discovery: one `GET /planner/getauditversions/` per catalog program in parallel, building the task list. Progress is logged every 200 programs.
2. Audit fetch: one `POST /planner/getauditinfo/` (test-apply mode) per task in parallel. Progress is logged every 200 completed tasks.

Step 2 of the programs pipeline is heavier than any course-pipeline call: each `getauditinfo` returns ~80 KB and the server takes several seconds to compute the audit, so its tolerable concurrency is lower than the course endpoint's.

Empirical concurrency findings on this Stellic deployment:

- Programs pipeline (`getauditinfo` test-apply): best throughput at concurrency 32 (~1.2 s per task on a 512-task run). Concurrency 64 dropped to ~3 s per task with sporadic failures, and 128 stalled. The default is 32.
- Course pipeline (`getcourseinfo`, `getcoursesections`): handles concurrency 64 cleanly during full-history scrapes; the per-call cost is small enough that the bottleneck is HTTP latency rather than server compute. If running the course pipeline alone, raising `--concurrency` to 64 is safe.
