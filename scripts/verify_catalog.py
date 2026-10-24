#!/usr/bin/env python3
"""Check generated catalog integrity, audience isolation, counts and references."""

import argparse
import json, sqlite3, pathlib

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--root", type=pathlib.Path, default=pathlib.Path("data/catalog"))
root = parser.parse_args().root
report = json.loads((root / "report.json").read_text())
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
            assert "fce" not in c and "instructors" not in c
            for section in c["sections"]:
                assert not section["instructors"]
                assert all("location" not in m for m in section["meetings"])
    if audience == "public":
        assert db.execute("SELECT count(*) FROM catalog_instructors").fetchone()[0] == 0
    if report.get("courseweave"):
        snapshot = report["courseweave"]
        rows = [
            json.loads(r[0])
            for r in db.execute(
                "SELECT payload_json FROM catalog_courses WHERE term_id=?",
                (snapshot["term"],),
            )
        ]
        assert len(rows) == snapshot["courses"]
        assert sum(len(c["sections"]) for c in rows) == snapshot["sections"]
        with sqlite3.connect(root / f"{audience}-browser.sqlite") as browser:
            assert browser.execute(
                "SELECT count(*) FROM catalog_courses WHERE term_id=?",
                (snapshot["term"],),
            ).fetchone()[0] == len(rows)
    print(audience, "integrity and disclosure checks passed")
print(
    "Imported",
    report["courses"],
    "courses;",
    report["program_audits"],
    "program audits",
)
