# 8. Cache the catalog in OPFS

Date: 2026-10-14

## Status

Amended by [decision 16](0016-serve-the-catalog-as-json-documents.md): the browser's HTTP cache, keyed by ETag, replaces the OPFS copy

## Context

We want users to download the catalog again only when it changes.

## Decision

- The web backend serves the catalog at `/api/catalog/binary`, and its ETag along with an identifier for the catalog format the backend reads at `/api/catalog/version`
- We'll store the catalog in OPFS, keyed by its ETag, and delete older copies
- On each page load, the website fetches the current ETag and only downloads the catalog when it differs from the cached one. If that request fails, it uses the cached copy
- If the format identifier differs from the one compiled into the website code the browser is running, the website updates its service worker and reloads before downloading the catalog, at most once per identifier
- We store the catalog decompressed, so visits after the first don't decompress it again

## Consequences

- A visit with an up-to-date cache makes one small request before searching
- The cached copy takes up the catalog's uncompressed size on disk
- A deploy that changes the catalog format makes each open copy of the website reload once
