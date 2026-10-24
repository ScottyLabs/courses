#!/usr/bin/env python3
"""Take a consistent online SQLite backup without copying a live WAL file."""

import argparse
import os
from pathlib import Path
import sqlite3

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("source", type=Path)
parser.add_argument("destination", type=Path)
args = parser.parse_args()
if not args.source.is_file():
    parser.error("Source database does not exist")
fd = os.open(args.destination, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
os.close(fd)
with sqlite3.connect(args.source.resolve().as_uri() + "?mode=ro", uri=True) as source:
    with sqlite3.connect(args.destination) as destination:
        source.backup(destination)
        if destination.execute("PRAGMA integrity_check").fetchone()[0] != "ok":
            raise RuntimeError("Backup integrity check failed")
print("Consistent backup written:", args.destination)
