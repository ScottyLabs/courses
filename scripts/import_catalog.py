#!/usr/bin/env python3
"""Compatibility entry point for the courses-index SQLite catalog builder."""

import os
import pathlib
import sys

root = pathlib.Path(__file__).resolve().parents[1]
os.execvp("cargo", [
    "cargo", "run", "--release", "--manifest-path", str(root / "Cargo.toml"),
    "-p", "courses-index", "--", "--import-catalog", *sys.argv[1:],
])
