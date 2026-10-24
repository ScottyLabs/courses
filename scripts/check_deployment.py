#!/usr/bin/env python3
"""Read-only anonymous checks against a deployed browser-facing service."""

import argparse
import hashlib
import json
import urllib.error
import urllib.request

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("origin")
args = parser.parse_args()
origin = args.origin.rstrip("/")


def get(path):
    with urllib.request.urlopen(origin + path, timeout=45) as response:
        return response.read()


assert get("/api/health") == b"ok"
terms = json.loads(get("/api/terms"))
assert any(t["id"] == "S27" for t in terms)
page = json.loads(get("/api/courses?term=S27&page_size=3"))
assert page["total"] == 3085
for course in page["items"]:
    assert "fce" not in course and "instructors" not in course
    for section in course["sections"]:
        assert not section["instructors"]
        assert all("location" not in meeting for meeting in section["meetings"])
for route in [
    "/api/instructors",
    "/api/schedules",
    "/api/profile/friends",
    "/api/finals?term=S27",
]:
    try:
        get(route)
    except urllib.error.HTTPError as error:
        assert error.code == 401, (route, error.code)
    else:
        raise AssertionError("Protected route accessible: " + route)
version = json.loads(get("/api/catalog/version"))
assert version["audience"] == "public" and version["format"] == "sqlite"
import gzip

binary = get("/api/catalog/binary")
if binary[:2] == b"\x1f\x8b":
    binary = gzip.decompress(binary)
assert hashlib.sha256(binary).hexdigest() == version["hash"]
assert binary.startswith(b"SQLite format 3\0")
print(
    "PASS: live anonymous API, S27 coverage, access restrictions and public SQLite hash"
)
