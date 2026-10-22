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


def without_code_blocks(text):
    """Exclude fenced and indented Markdown examples from reference checks."""
    lines = []
    fence = None
    paragraph = False
    for line in text.splitlines(keepends=True):
        lines.append("\n")
        if fence is not None:
            if re.fullmatch(rf" {{0,3}}{fence[0]}{{{len(fence)},}}[ \t]*", line.rstrip("\r\n")):
                fence = None
            continue
        match = re.match(r" {0,3}(`{3,}|~{3,})(.*)", line)
        if match and (match[1][0] == "~" or "`" not in match[2]):
            fence = match[1]
            paragraph = False
        elif not paragraph and line.expandtabs(4).startswith("    "):
            continue
        else:
            lines[-1] = line
            # Indented code cannot interrupt a paragraph.
            paragraph = bool(line.strip())
    return "".join(lines)


def link_targets(text):
    """Extract inline Markdown destinations (without titles) and HTML attributes."""
    for match in re.finditer(r'\]\(\s*|(?:src|href)="([^"]+)"', text):
        if match[1] is not None:
            yield match[1].strip()
            continue
        start = end = match.end()
        if text[start:start + 1] == "<":
            end = text.find(">", start + 1)
            if end < 0:
                continue
            target = text[start + 1:end]
            end += 1
        else:
            depth = 0
            while end < len(text):
                char = text[end]
                if char == "\\" and end + 1 < len(text):
                    end += 2
                    continue
                if char == "(":
                    depth += 1
                elif char == ")":
                    if depth == 0:
                        break
                    depth -= 1
                elif char.isspace():
                    break
                end += 1
            if depth:
                continue
            target = text[start:end]
        # A title must be separated from the destination by whitespace.
        tail = text[end:]
        if re.match(r"\s*\)", tail) or re.match(
            r'''\s+(?:"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|\((?:\\.|[^()\\])*\))\s*\)''', tail
        ):
            yield re.sub(r"\\([!\"#$%&'()*+,\-./:;<=>?@\[\]\\^_`{|}~])", r"\1", target)


def check():
    """Print missing local links, embedded assets, and feature evidence in tracked files.

    Return True if any reference is missing, otherwise False. Markdown code blocks,
    anchors, and external URLs are excluded. Git, file access, JSON parsing, and URL
    parsing errors propagate to the caller.
    """
    files = subprocess.check_output(
        ["git", "ls-files", "-z"], cwd=ROOT
    ).decode().split("\0")[:-1]
    errors = []
    links = assets = evidence = 0
    for name in files:
        path = ROOT / name
        if name.endswith(".md"):
            # The repository uses inline Markdown and HTML links. Exclude code blocks, whose
            # examples can contain placeholder paths or HTML that is not a document link.
            text = without_code_blocks(path.read_text(encoding="utf-8"))
            for target in link_targets(text):
                url = urlsplit(target)
                if url.scheme or url.netloc or not url.path:
                    continue
                links += 1
                if not (path.parent / unquote(url.path)).exists():
                    errors.append(f"{name}: missing link target {target}")
        if name.endswith(".rs"):
            for target in re.findall(r'include_(?:bytes|str)!\(\s*"([^"]+)"', path.read_text(encoding="utf-8")):
                assets += 1
                if not (path.parent / target).is_file():
                    errors.append(f"{name}: missing embedded asset {target}")

    features = json.loads((ROOT / "docs/roadmap/features.json").read_text(encoding="utf-8"))["features"]
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
                if not re.search(rf"\bfn\s+{re.escape(symbol)}\b", path.read_text(encoding="utf-8")):
                    errors.append(f"{feature['id']}: missing test function {entry}")
    for error in errors:
        print(error)
    print(f"Repository references: {links} links, {assets} embedded assets, {evidence} evidence paths; {len(errors)} errors")
    return bool(errors)


if __name__ == "__main__":
    raise SystemExit(check())
