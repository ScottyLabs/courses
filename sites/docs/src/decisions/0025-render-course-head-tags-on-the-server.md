# 25. Render course head tags on the server

Date: 2026-10-24

## Status

Accepted

## Context

The website is client-rendered: every page is the same `index.html` until scripts run, so search engines and link
previews see "CMU Courses" for every course. Courseweave fixes this at build time with `scripts/build-seo.ts`, writing
one HTML file per course with its title, description, canonical link, social tags and schema.org data. Here the catalog
is installed when the service starts, not when the website is built, and the same website build serves whichever
catalog release is pinned.

## Decision

- When `STATIC_DIR` is set, the web API answers `GET /course/{code}` itself: the website's `index.html`, with that
  course's title, description, canonical link (`APP_URL/course/15-122`), Open Graph tags and schema.org `Course` JSON-LD
  added to the head, and a plain `<noscript>` summary. The body is otherwise the website's own, so visitors get the same
  app.
- The page is built from the catalog's relational tables, which hold public fields only, so nothing for signed-in users
  can leak into it. Text is HTML-escaped and `<` in the JSON-LD is escaped.
- Unknown codes, courses titled with just their number, and a template that doesn't have the expected shape get the
  plain website.
- The service also serves `/sitemap.xml` (the home page and every course with a description or an offering) and
  `/robots.txt` (keeping crawlers out of `/api/`, `/auth/` and the signed-in pages).
- The logic is a port of Courseweave's `build-seo.ts`.

## Consequences

- Course pages and the sitemap always match the catalog being served, with no extra build step
- Each course page request does a small catalog query and string edits on `index.html`
- The website must keep `<title>`, `</head>` and `<body>` in its `index.html`; if it doesn't, course pages quietly fall
  back to the plain template
- `/robots.txt` comes from the service, not the website's static file
