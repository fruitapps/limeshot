#!/usr/bin/env python3
"""Validate docs/roadmap/features.json and (re)generate docs/roadmap/features.md from it.

    python3 scripts/features.py check     # validate; fail if features.md is stale (CI)
    python3 scripts/features.py generate  # rewrite features.md from features.json

features.json is the single source of truth; features.md is generated, never hand-edited.
"""
import json
import sys
from collections import Counter, OrderedDict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
JSON_PATH = ROOT / "docs/roadmap/features.json"
MD_PATH = ROOT / "docs/roadmap/features.md"

STATUSES = ["NOT_STARTED", "RESEARCHED", "DESIGNED", "IN_PROGRESS", "IMPLEMENTED", "VERIFIED", "BLOCKED", "DEFERRED"]
PRIORITIES = ["P0", "P1", "P2", "P3"]
REQUIRED = {
    "id": str, "category": str, "name": str, "description": str, "fl_reference": str,
    "limeshot_behavior": str, "priority": str, "status": str, "mvp": bool, "dependencies": list,
    "modules": list, "acceptance_criteria": list, "tests": list, "reference": dict, "notes": str,
}


def validate(data):
    errors = []
    feats = data.get("features", [])
    ids = [f.get("id") for f in feats]
    for dup, n in Counter(ids).items():
        if n > 1:
            errors.append(f"duplicate id {dup}")
    known = set(ids)
    for f in feats:
        fid = f.get("id", "?")
        for key, typ in REQUIRED.items():
            if key not in f:
                errors.append(f"{fid}: missing {key}")
            elif not isinstance(f[key], typ):
                errors.append(f"{fid}: {key} must be {typ.__name__}")
        if f.get("status") not in STATUSES:
            errors.append(f"{fid}: bad status {f.get('status')}")
        if f.get("priority") not in PRIORITIES:
            errors.append(f"{fid}: bad priority {f.get('priority')}")
        for d in f.get("dependencies", []):
            if d not in known:
                errors.append(f"{fid}: unknown dependency {d}")
        if f.get("status") == "VERIFIED" and not f.get("tests"):
            errors.append(f"{fid}: VERIFIED requires at least one test or recorded verification")
        if f.get("status") in ("IMPLEMENTED", "VERIFIED") and not f.get("modules") and f.get("category") != "Platform":
            errors.append(f"{fid}: {f['status']} requires the modules that implement it")
        if not f.get("acceptance_criteria"):
            errors.append(f"{fid}: acceptance_criteria is empty")
        ref = f.get("reference", {})
        if not ref.get("application") or not ref.get("version"):
            errors.append(f"{fid}: reference needs application and version")
    return errors


def generate(data):
    feats = data["features"]
    by_cat = OrderedDict()
    for f in feats:
        by_cat.setdefault(f["category"], []).append(f)
    total = Counter(f["status"] for f in feats)
    mvp = [f for f in feats if f["mvp"]]
    mvp_done = sum(1 for f in mvp if f["status"] in ("IMPLEMENTED", "VERIFIED"))
    mvp_ver = sum(1 for f in mvp if f["status"] == "VERIFIED")
    lines = [
        "# LimeShot feature inventory",
        "",
        "> Generated from [`features.json`](features.json) by `python3 scripts/features.py generate`. Do not edit by hand.",
        f"> Reference: {data.get('reference')}. Updated {data.get('updated')}.",
        "",
        "**IMPLEMENTED** = the code exists and meets its implementation criteria. **VERIFIED** = it also passed the listed",
        "automated tests or recorded manual verification. Everything else is not done.",
        "",
        "## Summary",
        "",
        f"- Features tracked: **{len(feats)}**",
        f"- MVP features: **{len(mvp)}** — {mvp_done} implemented or verified, {mvp_ver} verified",
        "- By status: " + ", ".join(f"{s} {total[s]}" for s in STATUSES if total[s]),
        "",
        "| Category | Total | Verified | Implemented | In progress | Not started / other |",
        "|---|---|---|---|---|---|",
    ]
    for cat, fs in by_cat.items():
        c = Counter(f["status"] for f in fs)
        other = len(fs) - c["VERIFIED"] - c["IMPLEMENTED"] - c["IN_PROGRESS"]
        lines.append(f"| {cat} | {len(fs)} | {c['VERIFIED']} | {c['IMPLEMENTED']} | {c['IN_PROGRESS']} | {other} |")
    for cat, fs in by_cat.items():
        lines += ["", f"## {cat}", "", "| ID | Feature | Pri | MVP | Status | LimeShot behaviour | Tests |", "|---|---|---|---|---|---|---|"]
        for f in fs:
            tests = "<br>".join(f"`{t}`" for t in f["tests"]) or "—"
            beh = f["limeshot_behavior"].replace("|", "\\|")
            if f["notes"]:
                beh += f"<br>*{f['notes'].replace('|', '/')}*"
            lines.append(f"| {f['id']} | {f['name']} | {f['priority']} | {'✓' if f['mvp'] else ''} | {f['status']} | {beh} | {tests} |")
    return "\n".join(lines) + "\n"


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else "check"
    data = json.loads(JSON_PATH.read_text(encoding="utf-8"))
    errors = validate(data)
    if errors:
        print("\n".join(errors))
        sys.exit(1)
    md = generate(data)
    if mode == "generate":
        MD_PATH.write_text(md, encoding="utf-8")
        print(f"wrote {MD_PATH.relative_to(ROOT)} ({len(data['features'])} features)")
    elif MD_PATH.read_text(encoding="utf-8") != md:
        print("features.md is out of date: run python3 scripts/features.py generate")
        sys.exit(1)
    else:
        print(f"features.json valid ({len(data['features'])} features); features.md up to date")


if __name__ == "__main__":
    main()
