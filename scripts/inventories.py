#!/usr/bin/env python3
"""Validate the keybinding and settings inventories and (re)generate their Markdown views.

    python3 scripts/inventories.py check     # validate; fail if the .md files are stale (CI)
    python3 scripts/inventories.py generate  # rewrite keybindings.md and settings.md

docs/roadmap/keybindings.json tracks FL Studio's keyboard shortcuts against LimeShot's;
docs/roadmap/settings.json tracks FL Studio's settings against LimeShot's. Both are checked
against the code: a binding LimeShot has must appear in the Help › Keyboard shortcuts list
(crates/limeshot-ui/src/shortcuts.rs), and a setting LimeShot has must name a real field.
`scripts/features.py` runs this check too.
"""
import json
import re
import sys
from collections import Counter, OrderedDict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ROADMAP = ROOT / "docs/roadmap"
KB_JSON, KB_MD = ROADMAP / "keybindings.json", ROADMAP / "keybindings.md"
ST_JSON, ST_MD = ROADMAP / "settings.json", ROADMAP / "settings.md"
SHORTCUTS_RS = ROOT / "crates/limeshot-ui/src/shortcuts.rs"
FIELD_SOURCES = {
    "settings.audio.": ROOT / "crates/limeshot-audio/src/devices.rs",
    "settings.": ROOT / "crates/limeshot-ui/src/settings.rs",
    "ui.": ROOT / "crates/limeshot-ui/src/app.rs",
    "project.": ROOT / "crates/limeshot-core/src/model.rs",
}

CONFIDENCE = ["VERIFIED", "OBSERVED", "INFERRED"]
KB_STATUS = ["IMPLEMENTED", "DIFFERENT", "LIMESHOT_ONLY", "NOT_STARTED", "NOT_PLANNED"]
ST_STATUS = ["IMPLEMENTED", "IN_PROGRESS", "NOT_STARTED", "NOT_PLANNED"]
# Bindings LimeShot has, in some form.
HAS_KEYS = ("IMPLEMENTED", "DIFFERENT", "LIMESHOT_ONLY")


def combos(s):
    return {c.strip() for c in s.split(" / ") if c.strip()}


def help_combos():
    """Every key combination listed in shortcuts.rs's GLOBAL and EDITING tables."""
    text = SHORTCUTS_RS.read_text(encoding="utf-8")
    out = set()
    for table in ("GLOBAL", "EDITING"):
        m = re.search(rf"pub const {table}: &\[\(&str, &str\)\] = &\[(.*?)\n\];", text, re.S)
        if not m:
            raise SystemExit(f"shortcuts.rs: table {table} not found")
        for keys in re.findall(r'\(\s*"([^"]+)"\s*,', m.group(1)):
            out |= combos(keys)
    return out


def field_exists(name):
    for prefix, path in FIELD_SOURCES.items():
        if name.startswith(prefix):
            field = name[len(prefix):]
            return re.search(rf"\bpub {re.escape(field)}:", path.read_text(encoding="utf-8")) is not None
    return False


def validate(kb, st, feature_ids):
    errors = []
    helped = help_combos()
    for kind, items, statuses in (("binding", kb["bindings"], KB_STATUS), ("setting", st["settings"], ST_STATUS)):
        for dup, n in Counter(i.get("id") for i in items).items():
            if n > 1:
                errors.append(f"duplicate {kind} id {dup}")
        for i in items:
            iid = i.get("id", "?")
            if i.get("status") not in statuses:
                errors.append(f"{iid}: bad status {i.get('status')}")
            if i.get("confidence") not in CONFIDENCE:
                errors.append(f"{iid}: bad confidence {i.get('confidence')}")
            if i.get("feature") and i["feature"] not in feature_ids:
                errors.append(f"{iid}: unknown feature {i['feature']}")
    for b in kb["bindings"]:
        keys = b.get("limeshot_keys", "")
        if b["status"] in HAS_KEYS:
            if not keys:
                errors.append(f"{b['id']}: {b['status']} needs limeshot_keys")
            elif not re.search(r"click|drag", keys, re.I):
                missing = combos(keys) - helped
                if missing:
                    errors.append(f"{b['id']}: {', '.join(sorted(missing))} not in the shortcuts help (shortcuts.rs)")
        elif keys:
            errors.append(f"{b['id']}: {b['status']} but limeshot_keys is set")
    for s in st["settings"]:
        name = s.get("limeshot_setting", "")
        if s["status"] == "IMPLEMENTED":
            if not name:
                errors.append(f"{s['id']}: IMPLEMENTED needs limeshot_setting")
            elif not field_exists(name):
                errors.append(f"{s['id']}: {name} is not a field in the code")
    return errors


def esc(s):
    return s.replace("|", "\\|")


def generate_kb(kb):
    items = kb["bindings"]
    total = Counter(b["status"] for b in items)
    lines = [
        "# Keyboard shortcuts: FL Studio and LimeShot",
        "",
        "> Generated from [`keybindings.json`](keybindings.json) by `python3 scripts/inventories.py generate`. Do not edit by hand.",
        f"> Reference: {kb['reference']} ({kb['source']}).",
        "",
        kb["notes"],
        "",
        "**DIFFERENT** = LimeShot has the action on another key or with other behaviour; **LIMESHOT_ONLY** = no FL",
        "equivalent. Every binding LimeShot has is checked against the Help › Keyboard shortcuts list.",
        "",
        f"Tracked: **{len(items)}** — " + ", ".join(f"{s} {total[s]}" for s in KB_STATUS if total[s]),
    ]
    by_ctx = OrderedDict()
    for b in items:
        by_ctx.setdefault(b["context"], []).append(b)
    for ctx, bs in by_ctx.items():
        lines += ["", f"## {ctx}", "", "| ID | Action | FL Studio | Conf. | LimeShot | Status | Feature | Notes |", "|---|---|---|---|---|---|---|---|"]
        for b in bs:
            lines.append(
                f"| {b['id']} | {esc(b['action'])} | `{esc(b['fl_keys'])}` | {b['confidence']} | "
                f"{'`' + esc(b['limeshot_keys']) + '`' if b['limeshot_keys'] else '—'} | {b['status']} | {b['feature']} | {esc(b['notes'])} |"
            )
    return "\n".join(lines) + "\n"


def generate_st(st):
    items = st["settings"]
    total = Counter(s["status"] for s in items)
    lines = [
        "# Settings: FL Studio and LimeShot",
        "",
        "> Generated from [`settings.json`](settings.json) by `python3 scripts/inventories.py generate`. Do not edit by hand.",
        f"> Reference: {st['reference']} ({st['source']}).",
        "",
        st["notes"],
        "",
        f"Tracked: **{len(items)}** — " + ", ".join(f"{s} {total[s]}" for s in ST_STATUS if total[s]),
    ]
    by_page = OrderedDict()
    for s in items:
        by_page.setdefault(s["page"], []).append(s)
    for page, ss in by_page.items():
        lines += ["", f"## {page}", "", "| ID | Setting | FL Studio | Conf. | LimeShot | Status | Feature | Notes |", "|---|---|---|---|---|---|---|---|"]
        for s in ss:
            lines.append(
                f"| {s['id']} | {esc(s['setting'])} | {esc(s['fl_reference'])} | {s['confidence']} | "
                f"{'`' + s['limeshot_setting'] + '`' if s['limeshot_setting'] else '—'} | {s['status']} | {s['feature']} | {esc(s['notes'])} |"
            )
    return "\n".join(lines) + "\n"


def run(mode, feature_ids):
    """Validate and check or write the Markdown. Returns error lines (empty when fine)."""
    kb = json.loads(KB_JSON.read_text(encoding="utf-8"))
    st = json.loads(ST_JSON.read_text(encoding="utf-8"))
    errors = validate(kb, st, feature_ids)
    if errors:
        return errors
    out = []
    for path, md in ((KB_MD, generate_kb(kb)), (ST_MD, generate_st(st))):
        if mode == "generate":
            path.write_text(md, encoding="utf-8")
            print(f"wrote {path.relative_to(ROOT)}")
        elif path.read_text(encoding="utf-8") != md if path.exists() else True:
            out.append(f"{path.name} is out of date: run python3 scripts/inventories.py generate")
    if not out and mode != "generate":
        print(f"keybindings.json ({len(kb['bindings'])}) and settings.json ({len(st['settings'])}) valid; Markdown up to date")
    return out


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else "check"
    features = json.loads((ROADMAP / "features.json").read_text(encoding="utf-8"))
    errors = run(mode, {f["id"] for f in features["features"]})
    if errors:
        print("\n".join(errors))
        sys.exit(1)


if __name__ == "__main__":
    main()
