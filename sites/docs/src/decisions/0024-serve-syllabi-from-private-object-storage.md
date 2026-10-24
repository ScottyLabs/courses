# 24. Serve syllabi from private object storage

Date: 2026-10-24

## Status

Accepted

## Context

Past syllabi from CMU's Canvas Syllabus Registry are CMU-internal, like FCEs: Courseweave serves them only to signed-in
CMU users, from a private R2 bucket, through its Worker. Here the scraper uploads them to ScottyLabs' Garage
(`s3.scottylabs.org`), into the `cdn-courses` bucket under `syllabi/`, which the CDN serves publicly. The files are
uploads by many instructors: mostly PDFs, but also Word files and HTML that must never run on this site.

## Decision

- `GET /api/syllabi/<term>/<dept>/<file>` serves a syllabus to signed-in CMU users only, reading it from an
  S3-compatible bucket (`SYLLABI_BUCKET`, `SYLLABI_PREFIX`, `SYLLABI_S3_ENDPOINT` and read-only keys) with `aws-sdk-s3`,
  or from a local directory (`SYLLABI_DIR`) in development.
- Keys must follow the registry layout, ported from Courseweave's `isSyllabusKey`: a term, a department code and a file
  name with a known type. Anything else is 404, which also rules out path traversal in the directory store.
- PDFs are shown inline, framed only by this site (`X-Frame-Options: SAMEORIGIN`,
  `Content-Security-Policy: default-src 'none'; frame-ancestors 'self'`). Every other type is an attachment with a
  sandboxing, deny-all policy. The saved name is `<term>-<file>`.
- Syllabi move to a private bucket, or a prefix the CDN doesn't serve, before the gate is relied on.

## Consequences

- Until the files leave the public `cdn-courses` bucket, anyone who knows a key can still download them from the CDN;
  the member check protects nothing until then
- The web API needs read-only keys for the syllabus bucket in its secret store
- Files pass through the service, which buffers each one (up to 50 MB) rather than redirecting to a signed URL
- Local development can serve the scraper's `exported/syllabi` directory without any credentials
