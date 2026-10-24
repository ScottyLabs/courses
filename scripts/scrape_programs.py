#!/usr/bin/env python3
"""Archive current public CMU program requirements with provenance.
Preserves requirement text and course references, not an inferred degree-audit rule set.
Only follows links discovered on the official program index; serial requests.
"""

import subprocess
import socket

socket.setdefaulttimeout(30)
import argparse, datetime, hashlib, json, pathlib, re, time
from html.parser import HTMLParser
from urllib.parse import urljoin, urlparse, urldefrag
from urllib.request import Request, urlopen
from urllib.robotparser import RobotFileParser

BASE = "https://coursecatalog.web.cmu.edu/"
AGENT = "CMUCoursesCatalogImporter/1.0"


class Page(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []
        self.parts = []
        self.active = 0
        self.depth = 0
        self.skip = 0
        self.link = None
        self.title = []
        self.in_title = False

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if tag == "title":
            self.in_title = True
        if tag == "main" and a.get("id") == "contentarea":
            self.active = -1
        if tag == "a":
            self.link = [a.get("href", ""), ""]
        if tag == "div":
            self.depth += 1
            if a.get("id") == "content":
                self.active = self.depth
        if tag in ("script", "style"):
            self.skip += 1
        if self.active and tag in ("p", "br", "h1", "h2", "h3", "h4", "tr", "li"):
            self.parts.append("\n")
        if self.active and tag in ("td", "th"):
            self.parts.append(" | ")

    def handle_endtag(self, tag):
        if tag == "a" and self.link:
            self.links.append(tuple(self.link))
            self.link = None
        if tag == "title":
            self.in_title = False
        if tag == "main":
            self.active = 0
        if tag in ("script", "style"):
            self.skip = max(0, self.skip - 1)
        if tag == "div":
            if self.depth == self.active:
                self.active = 0
            self.depth -= 1

    def handle_data(self, data):
        if self.link:
            self.link[1] += data
        if self.in_title:
            self.title.append(data)
        if self.active and not self.skip:
            self.parts.append(data)

    def text(self):
        return "\n".join(
            " ".join(l.split()) for l in "".join(self.parts).splitlines() if l.strip()
        )


def main():
    p = argparse.ArgumentParser()
    p.add_argument(
        "--output", type=pathlib.Path, default=pathlib.Path("data/programs-current")
    )
    p.add_argument("--limit", type=int, default=0)
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=True)

    def download(url):
        return subprocess.check_output(
            [
                "curl",
                "--fail",
                "--silent",
                "--show-error",
                "--max-time",
                "45",
                "--user-agent",
                AGENT,
                url,
            ],
            text=True,
        )

    robots = RobotFileParser()
    robots.parse(download(BASE + "robots.txt").splitlines())

    def fetch(url):
        if urlparse(url).netloc != urlparse(BASE).netloc:
            raise ValueError("External URL refused")
        if not robots.can_fetch(AGENT, url):
            raise ValueError("Disallowed by robots.txt: " + url)
        return download(url)

    html = fetch(BASE + "programs/")
    index = Page()
    index.feed(html)
    links = {
        urldefrag(urljoin(BASE, h))[0]: name.strip()
        for h, name in index.links
        if re.search(r",\s*(B\.|Minor|Additional Major)|Major Program", name)
        and urlparse(urljoin(BASE, h)).netloc == urlparse(BASE).netloc
    }
    if not links:
        raise RuntimeError("No program links discovered; source layout changed")
    results = []
    failures = []
    for url, name in sorted(links.items())[: a.limit or None]:
        try:
            time.sleep(0.5)
            html = fetch(url)
            page = Page()
            page.feed(html)
            text = page.text()
            if len(text) < 100:
                raise ValueError("Missing content; source layout changed")
            edition = re.search(r"20\d{2}-20\d{2}", html)
            ident = hashlib.sha256(url.encode()).hexdigest()[:20]
            row = {
                "id": "catalog:" + ident,
                "name": name,
                "source_url": url,
                "catalog_year": edition.group() if edition else None,
                "retrieved_at": datetime.datetime.now(
                    datetime.timezone.utc
                ).isoformat(),
                "sha256": hashlib.sha256(html.encode()).hexdigest(),
                "requirements_text": text,
                "course_references": sorted(set(re.findall(r"\b\d{2}-\d{3}\b", text))),
                "validation_status": "reference_only",
                "tree": None,
            }
            (a.output / (ident + ".html")).write_text(html)
            (a.output / (ident + ".json")).write_text(json.dumps(row, indent=2))
            results.append(row)
            print(name, flush=True)
        except Exception as e:
            failures.append({"url": url, "error": str(e)})
    manifest = {
        "discovered": len(links),
        "scraped": len(results),
        "partial": bool(a.limit),
        "failures": failures,
        "programs": results,
    }
    (a.output / "manifest.json").write_text(json.dumps(manifest, indent=2))
    print(json.dumps({k: v for k, v in manifest.items() if k != "programs"}, indent=2))
    if failures:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
