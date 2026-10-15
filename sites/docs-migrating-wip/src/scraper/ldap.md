# Directory (LDAP)

CMU's central directory is reachable at `ldaps://ldap.cmu.edu` and holds one entry per Andrew account. The scraper's `ldap` mode enriches instructors with directory data Stellic never exposes: job title, department names, college, and contact info.

## Auth

The scraper binds to the directory as `uid=scottylabs-svc,ou=andrewperson,dc=andrew,dc=cmu,dc=edu` using the password in `CMU_LDAP_PASSWORD`. This is the org-wide ScottyLabs service account, so its password stays out of the shared dev secrets. Developers keep it in a gitignored `.env` that secretspec reads through the `local` provider, and CI reads it from the OpenBao `ci` profile, which only project leads and this repo's CI jobs (including the shared checks on pull requests) can access.

## Query shape

Search base `ou=andrewperson,dc=andrew,dc=cmu,dc=edu`, scope one, filter `(uid=<andrewid>)`. AndrewIDs are collected from the existing scrape output (every instructor `username` across `info.json` and section files) and resolved in OR-filter batches of 50 (`(|(uid=a)(uid=b)...)`) over a single bound connection. IDs missing from the directory (people who have left CMU) are simply absent from the output; the found/missing counts are logged at the end of the run.

## Attributes

Requested server-side so the response never carries more than we save:

| Attribute | Notes |
| ---------------------------------------------------- | ------------------------------------------------------------------------ |
| `uid` | Lowercased, used as the output key. |
| `cn` / `displayName` / `givenName` / `sn` | Name parts, mostly redundant with Stellic's. |
| `title` | Job title, e.g. "Teaching Professor", "Systems Scientist". |
| `cmuDepartment` | Multi-valued; a person can carry several departments. |
| `eduPersonPrimaryAffiliation` / `eduPersonAffiliation` | Faculty/Staff/Student/Alumni; the latter is multi-valued. |
| `eduPersonSchoolCollegeName` | Multi-valued, e.g. "SCS - School of Computer Science". |
| `mail` | Can be a department alias, not always `<uid>@andrew.cmu.edu`. |
| `telephoneNumber` | Sparse; present for some people, absent for others. |
| `postalAddress` | Office location, e.g. "GHC 7503". |
| `eduPersonOrcid` | ORCID URL, mostly research faculty. |
| `guid` | CMU-internal person GUID, useful only as a join key. |

## Output

`<export_root>/ldap.json`: one JSON object mapping lowercased AndrewID to an attribute map, where every attribute value is a list of strings since LDAP attributes are inherently multi-valued. `courses-index` merges it onto `Professor` by username at load time; a missing file is a no-op so catalogs still build from scrapes that predate directory enrichment.
