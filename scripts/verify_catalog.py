#!/usr/bin/env python3
"""Check generated catalog integrity, audience isolation, counts and references."""

import argparse
import gzip
import hashlib
import json
import pathlib
import sqlite3

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--root", type=pathlib.Path, default=pathlib.Path("data/catalog"))
root = parser.parse_args().root
report = json.loads((root / "report.json").read_text())
assert report["courses"] > 0 and report["terms"]
for audience in ["public", "member"]:
    db = sqlite3.connect(root / f"{audience}.sqlite")
    assert db.execute("PRAGMA integrity_check").fetchone()[0] == "ok"
    assert (
        db.execute("SELECT count(*) FROM catalog_courses WHERE term_id=''").fetchone()[
            0
        ]
        == report["courses"]
    )
    for term, raw in db.execute("SELECT term_id,payload_json FROM catalog_courses"):
        c = json.loads(raw)
        if term:
            assert term in c["offered_terms"]
        if audience == "public":
            assert not {"fce", "instructors", "syllabi"} & c.keys()
            for section in c["sections"]:
                assert not section["instructors"]
                assert all("location" not in m for m in section["meetings"])
    if audience == "public":
        assert db.execute("SELECT count(*) FROM catalog_instructors").fetchone()[0] == 0
        assert db.execute("SELECT count(*) FROM programs").fetchone()[0] == 0
    browser_path = root / f"{audience}-browser.sqlite"
    with sqlite3.connect(browser_path) as browser:
        assert browser.execute("PRAGMA integrity_check").fetchone()[0] == "ok"
        expected = {"", *report["terms"][:3]}
        covered = {
            term
            for (term,) in browser.execute(
                "SELECT term_id FROM catalog_coverage WHERE dataset='courses'"
            )
        }
        assert covered == expected
        for term in expected:
            query = "SELECT count(*) FROM catalog_courses WHERE term_id=?"
            assert browser.execute(query, (term,)).fetchone() == db.execute(
                query, (term,)
            ).fetchone()
        assert browser.execute("SELECT count(*) FROM programs").fetchone()[0] == 0
    with browser_path.open("rb") as plain, gzip.open(
        browser_path.with_suffix(".sqlite.gz"), "rb"
    ) as compressed:
        assert hashlib.file_digest(plain, "sha256").digest() == hashlib.file_digest(
            compressed, "sha256"
        ).digest()
    print(audience, "integrity and disclosure checks passed")
print(
    "Imported",
    report["courses"],
    "courses;",
    report["program_audits"],
    "program audits",
)
