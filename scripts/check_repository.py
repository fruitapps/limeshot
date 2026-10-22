#!/usr/bin/env python3
"""Check tracked local documentation links, feature evidence and embedded asset paths offline.

Run from any directory: python3 scripts/check_repository.py
This checks file targets, not Markdown anchors, external URLs or arbitrary prose references.
"""
import json
import re
import subprocess
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parent.parent


def check():
    files = subprocess.check_output(
        ["git", "ls-files", "-z"], cwd=ROOT
    ).decode().split("\0")[:-1]
    errors = []
    links = assets = evidence = 0
    for name in files:
        path = ROOT / name
        if name.endswith(".md"):
            # The repository uses inline Markdown and HTML links. Exclude code fences, whose
            # examples can contain placeholder paths or HTML that is not a document link.
            text = re.sub(r"^```.*?^```[^\n]*", "", path.read_text(), flags=re.M | re.S)
            for md, html in re.findall(r'\]\(([^)]+)\)|(?:src|href)="([^"]+)"', text):
                target = (md or html).strip().removeprefix("<").removesuffix(">")
                url = urlsplit(target)
                if url.scheme or url.netloc or not url.path:
                    continue
                links += 1
                if not (path.parent / unquote(url.path)).exists():
                    errors.append(f"{name}: missing link target {target}")
        if name.endswith(".rs"):
            for target in re.findall(r'include_(?:bytes|str)!\(\s*"([^"]+)"', path.read_text()):
                assets += 1
                if not (path.parent / target).is_file():
                    errors.append(f"{name}: missing embedded asset {target}")

    features = json.loads((ROOT / "docs/roadmap/features.json").read_text())["features"]
    for feature in features:
        for entry in feature["tests"]:
            file, _, symbol = entry.partition("::")
            if not file.startswith(("crates/", "docs/", "scripts/")):
                continue  # Manual evidence descriptions are allowed by the feature schema.
            evidence += 1
            path = ROOT / file
            if not path.exists():
                errors.append(f"{feature['id']}: missing evidence {entry}")
            elif path.suffix == ".rs" and re.fullmatch(r"\w+", symbol):
                if not re.search(rf"\bfn\s+{re.escape(symbol)}\b", path.read_text()):
                    errors.append(f"{feature['id']}: missing test function {entry}")
    for error in errors:
        print(error)
    print(f"Repository references: {links} links, {assets} embedded assets, {evidence} evidence paths; {len(errors)} errors")
    return bool(errors)


if __name__ == "__main__":
    raise SystemExit(check())
