# Catalog data and releases

> Backend reset: product endpoints currently return 501. This document describes retained infrastructure or the intended integration contract; feature behavior must be rebuilt. See [backend development](../backend-development.md) for current status.

## Inputs and provenance

`data/raw/` preserves the supplied exports, including files not currently shown in the app. The inventory records path, byte count, modification time and SHA-256. A filesystem copy time is **not** a scrape date. The [audit](data-audit.md) records known source coverage and omissions.

Import precedence is deliberate:

1. Legacy course records provide the baseline descriptions and department names.
1. Stellic history adds metadata, requirement references and dated sections. Semester IDs are 1=Fall, 2=Spring, 3=Summer; section dates determine calendar year.
1. Explicit legacy schedules provide newer 2026 offerings where the older Stellic history has no corresponding term.
1. `data/raw/courseweave/S27.json`, when present, replaces S27 sections and course titles/units with the public CourseWeave snapshot. `--courseweave PATH` chooses a different snapshot. The converter currently accepts **S27 only**; extending it requires an explicit term validation change and test.
1. `dalmatian_courses/fce_data.csv` supplies respondent-weighted course/teaching/hour aggregates through Spring 2026. These aggregates are not Spring 2027 evaluations.
1. Public program pages from `data/programs-current/manifest.json` supplement the older Stellic requirement trees. Public pages are readable anonymously; Stellic trees require CMU authentication.

CourseWeave's S27 snapshot has 3,085 courses and 6,760 sections. Its claimed generation timestamp is October 23, 2026 at 8:37 p.m. EDT. Individual upstream collection dates are not provided. The merged catalog has 10,223 distinct courses across all terms.

## Database outputs

| File | Reader | Contents |
| --- | --- | --- |
| `public.sqlite` | Backend anonymous queries | Sanitized full course history and public programs |
| `member.sqlite` | Backend CMU queries | Full history plus restricted fields and Stellic program trees |
| `public-browser.sqlite` | Anonymous browser worker | Default rows and three newest terms, no program bodies |
| `member-browser.sqlite` | Authenticated browser worker | Same coverage with permitted member fields |
| `*-browser.sqlite.gz` | Transport | Gzipped versions of the browser databases |
| `inventory.json`, `report.json` | Maintainers | Source inventory, counts, provenance and warnings |

Catalog schema version 1 uses `catalog_metadata`, `catalog_coverage`, `catalog_terms`, `catalog_courses`, `catalog_instructors`, `catalog_geneds`, and `programs`. Indexed scalar columns support filtering; `payload_json` holds the wire object. An empty `term_id` means the general course record, with no section selection. Other rows are per course/term. Stable section IDs include their source and term when supplied by CourseWeave/legacy exports.

Public records omit FCEs, instructor identity, syllabus URLs and meeting locations. Weekly meeting times remain public. Source exports may contain more private fields; the importer uses an allowlist and does not publish student profiles or alerts. Never put raw exports, the member database, personal SQLite, or unencrypted catalog files in the static site or public CDN.

## Refresh procedure

```sh
# Optional: refresh official public degree-program reference pages.
python3 scripts/scrape_programs.py
# Import into a staging generation when updating a running deployment.
python3 scripts/import_catalog.py data/raw --output data/catalog-next
# The verifier defaults to data/catalog; validate the staged generation before switching.
python3 scripts/verify_catalog.py --root data/catalog-next
```

Inspect `report.json`, warnings, course/section counts and representative dates. Preserve the old generation until the replacement is verified. Do not overwrite databases beneath a running server. Point `CATALOG_DIR` to the new generation and restart, or publish a new pinned encrypted release.

The Stellic scraper is optional for future refreshes. Targeted runs preflight the public SOC term before authenticated requests, use serial requests with a delay, and stop on a failed course. A valid login is not a guarantee of permission to bulk collect data. Credentials belong in an ignored, permission-restricted file, never a workflow input or chat.

## Production artifacts

`publish_catalog.py` encrypts each of the six catalog files independently using AES-256-GCM with a fresh random nonce, after gzip compression. The dedicated key comes from `CATALOG_ENCRYPTION_KEY`; it is never committed or placed in browser code. The manifest contains only artifact URLs, sizes and SHA-256 digests, and is written only after all uploads succeed.

The backend embeds `deployments/catalog.json`. Startup downloads that exact release, verifies ciphertext hashes, authenticates/decrypts it, bounds decompression, checks plaintext hashes and atomically renames the complete generation into the persistent catalog directory. Failure prevents startup. A successful build alone is therefore not evidence of a healthy deployment.
