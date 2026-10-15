# 4. Publish the catalog to object storage

Date: 2026-10-14

## Status

Accepted

## Context

The course data changes independently of our code, and the web backend and the public course API should pick up a new catalog without being redeployed.

## Decision

- We'll store the catalog gzipped in an S3 bucket
- The web backend and the public course API check the object's ETag periodically and reload the catalog when it changes

## Consequences

The catalog is published separately from the web backend and the public course API, so either might end up reading a catalog it isn't compatible with.
