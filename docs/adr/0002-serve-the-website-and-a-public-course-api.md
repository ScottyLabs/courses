# 2. Serve the website and a public course API

Date: 2026-10-14

## Status

Accepted

## Context

Aside from the website [cmu.courses](https://cmu.courses), we want to expose a public course API for student developers to use in their own projects.

## Decision

We'll run the website's backend and the public course API as separate services.

## Consequences

We need one interface around the course data that both the website and the public course API use.
