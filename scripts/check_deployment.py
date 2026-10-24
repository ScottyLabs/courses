#!/usr/bin/env python3
"""Read-only anonymous checks against a deployed CMU Courses: health, public catalog documents and their redaction,
member-only documents, personal routes and search-engine files. Usage: check_deployment.py https://cmu.courses"""

import argparse
import json
import urllib.error
import urllib.request

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("origin")
args = parser.parse_args()
origin = args.origin.rstrip("/")


def get(path):
    request = urllib.request.Request(origin + path, headers={"Accept-Encoding": "identity"})
    with urllib.request.urlopen(request, timeout=45) as response:
        return response.read()


def status(path):
    try:
        get(path)
    except urllib.error.HTTPError as error:
        return error.code
    return 200


health = json.loads(get("/api/health"))
assert health["status"] == "ok", health

terms = json.loads(get("/api/catalog/terms"))
assert terms, "no semesters in the catalog"
newest = terms[0]["id"]

# Public term documents leave out instructors and rooms (decision 20).
term = json.loads(get(f"/api/catalog/term/{newest}"))
assert term["courses"], f"{newest} has no courses"
for course in term["courses"]:
    assert not course["instructors"], course["id"]
    for section in course["sections"]:
        assert not section["instructors"], (course["id"], section["name"])
        for meeting in section["meetings"]:
            assert meeting["building"] is None and meeting["room"] is None, (course["id"], section["name"])

# Member-only documents and personal routes reject anonymous callers.
for path in ["/api/catalog/fce/summary", "/api/catalog/programs/index", "/api/catalog/teaching"]:
    assert status(path) == 401, path
for path in [
    "/api/me",
    "/api/schedules",
    "/api/plans",
    "/api/record",
    "/api/preferences",
    "/api/saved",
    "/api/friends",
    f"/api/finals?term={newest}",
]:
    assert status(path) == 401, path

# The public course API and search-engine files.
page = json.loads(get(f"/api/courses?term={newest}&pageSize=3"))
assert page["total"] > 0 and len(page["items"]) <= 3
assert b"<urlset" in get("/sitemap.xml")
assert b"Disallow: /api/" in get("/robots.txt")
assert b'<link rel="canonical"' in get("/course/15-122")

print(f"PASS: health, {len(terms)} semesters ({newest} newest), redaction, access checks, course API and SEO files")
