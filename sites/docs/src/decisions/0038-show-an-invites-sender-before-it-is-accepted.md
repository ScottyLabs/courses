# 38. Show an invite's sender before it's accepted

Date: 2026-10-24

## Status

Accepted

## Context

Friend invite links (`POST /api/friends/invites`) are single-use tokens that make two students friends as soon as the
second one redeems them. The web API could only redeem a token: nothing told the invite page who had made it, so the
page could only ask a student to accept an invite from an unknown person, and say who it was afterwards. Accepting a
stranger's invite is harmless on its own (friendship shares nothing until each student turns sharing on), but asking
someone to agree without saying to what is poor form, and it hides mistakes such as opening one's own link.

## Decision

- `GET /api/friends/invites/{token}` answers a member with `{ name, andrewId, expiresAt, own }` for a live invite:
  the sender's name and Andrew ID, when it expires, and whether the caller made it. Looking doesn't use the invite up.
  An expired, used or unknown token is 404 with the same message as redeeming one.
- Only signed-in CMU members can look, like every other friends route. Signed out, the invite page says that a CMU
  student invited them and asks them to sign in, without naming anyone.
- The invite page shows the sender, explains what friends can and can't see, and offers Accept. It recognizes the
  student's own link, an existing friendship and pending requests either way from the friends list.

## Consequences

- Anyone holding a live token can learn the sender's name and Andrew ID by signing in. That is who the sender gave the
  link to, and the same details the sender's friends and anyone they've sent a request to already see.
- The mock API answers the route with 404, as it has no invites to look up.
