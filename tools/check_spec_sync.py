#!/usr/bin/env python3
"""SYNC-1 und SYNC-2 aus docs/SPEC.md §13.6.

  check_spec_sync.py check                       Anhang D/E gegen die TOML-Dateien vergleichen
  check_spec_sync.py sync                        TOML-Dateien 1:1 aus Anhang D/E schreiben
  check_spec_sync.py layering                    Schichtregeln (Anhang E) prüfen
  check_spec_sync.py layering --ratchet          nur neue Befunde gegen layering_baseline.txt sind Fehler
  check_spec_sync.py layering --update-baseline  Baseline auf den Ist-Stand setzen (nur nach bewusstem Abbau)

Exit 1 bei Abweichung. Nur Standardbibliothek (Python 3.11+, tomllib).
"""
import argparse
import json
import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPEC = ROOT / "docs" / "SPEC.md"
BUDGETS = ROOT / "benchmarks" / "budgets.toml"
CAPS = ROOT / "capabilities.toml"
BASELINE = ROOT / "layering_baseline.txt"

ANCHOR_D = "## Anhang D: `benchmarks/budgets.toml`"
ANCHOR_E = "## Anhang E: Schichtregeln für G-03"


def spec_block(anchor: str) -> str:
    """Erster ```toml-Block nach der Überschrift. Inhalt ohne Fences."""
    text = SPEC.read_text(encoding="utf-8")
    start = text.find(anchor)
    if start < 0:
        raise SystemExit(f"Überschrift nicht gefunden: {anchor}")
    m = re.compile(r"^```toml\n(.*?)^```", re.S | re.M).search(text, start)
    if not m:
        raise SystemExit(f"Kein ```toml-Block nach: {anchor}")
    return m.group(1)


def cmd_check() -> int:
    ok = True
    for label, path, anchor in (("budgets.toml", BUDGETS, ANCHOR_D), ("capabilities.toml", CAPS, ANCHOR_E)):
        want = spec_block(anchor)
        have = path.read_text(encoding="utf-8") if path.exists() else None
        if have is None:
            print(f"FEHLT: {path.relative_to(ROOT)}")
            ok = False
        elif have != want:
            print(f"ABWEICHUNG: {label} weicht von Anhang {'D' if anchor == ANCHOR_D else 'E'} ab (`sync` ausführen)")
            ok = False
        else:
            print(f"ok: {label} synchron zu Anhang {'D' if anchor == ANCHOR_D else 'E'}")
    return 0 if ok else 1


def cmd_sync() -> int:
    BUDGETS.parent.mkdir(parents=True, exist_ok=True)
    BUDGETS.write_text(spec_block(ANCHOR_D), encoding="utf-8")
    CAPS.write_text(spec_block(ANCHOR_E), encoding="utf-8")
    print(f"geschrieben: {BUDGETS.relative_to(ROOT)}, {CAPS.relative_to(ROOT)}")
    return 0


def cargo_metadata() -> dict:
    env = dict(os.environ)
    env.setdefault("RUSTUP_TOOLCHAIN", "stable")
    out = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=ROOT, env=env, capture_output=True, text=True, check=True,
    ).stdout
    return json.loads(out)


def layering_findings() -> list[str]:
    caps = tomllib.loads(CAPS.read_text(encoding="utf-8"))
    allowed: dict[str, list[str]] = caps["allowed"]
    rules = caps["rules"]
    e1 = {f"contextra-{c}" for c in rules["e1_crates"]}
    forbidden = set(rules["forbidden_crate_names"])

    md = cargo_metadata()
    members = {p["name"]: p for p in md["packages"]}  # --no-deps: nur Workspace-Mitglieder
    findings: set[str] = set()

    for name, pkg in members.items():
        if name in forbidden:
            findings.add(f"{name}: verbotener Crate-Name (R-Liste, Anhang E [rules])")
        if rules.get("compat_forbidden"):
            for feat in pkg.get("features", {}):
                if "compat" in feat:
                    findings.add(f"{name}: Feature `{feat}` (R7, CMP-10)")
            if (Path(pkg["manifest_path"]).parent / "src" / "compat").is_dir():
                findings.add(f"{name}: Verzeichnis src/compat/ (R7, G-11)")
        # Nur normale Abhängigkeiten (kind None). Dev-Abhängigkeiten sind ausgenommen (REM-27).
        for dep in pkg["dependencies"]:
            if dep["kind"] is not None or dep["name"] not in members:
                continue
            dname = dep["name"]
            if dname in forbidden:
                findings.add(f"{name} -> {dname}: verbotener Crate-Name")
            if name in allowed and dname not in allowed[name]:
                findings.add(f"{name} -> {dname}: nicht in [allowed]")
            if rules.get("e1_no_e1_dependency") and name in e1 and dname in e1:
                findings.add(f"{name} -> {dname}: E1 -> E1 (R2)")
    return sorted(findings)


def cmd_layering(ratchet: bool, update: bool) -> int:
    findings = layering_findings()
    if update:
        header = "# Schichtbefunde-Baseline (G-03, SYNC-5). Nur nach bewusstem Abbau mit --update-baseline neu schreiben.\n"
        BASELINE.write_text(header + "\n".join(findings) + ("\n" if findings else ""), encoding="utf-8")
        print(f"Baseline geschrieben: {len(findings)} Befunde -> {BASELINE.relative_to(ROOT)}")
        return 0

    if not ratchet:
        for f in findings:
            print(f"BEFUND: {f}")
        print(f"Schichtbefunde: {len(findings)}")
        return 1 if findings else 0

    known = set()
    if BASELINE.exists():
        known = {l.strip() for l in BASELINE.read_text(encoding="utf-8").splitlines() if l.strip() and not l.startswith("#")}
    new = [f for f in findings if f not in known]
    resolved = sorted(known - set(findings))
    for f in new:
        print(f"NEUER BEFUND: {f}")
    if resolved:
        print(f"Abgebaut seit Baseline: {len(resolved)} (mit --update-baseline übernehmen)")
    print(f"Schichtbefunde gesamt: {len(findings)}, bekannt: {len(findings) - len(new)}, neu: {len(new)}")
    return 1 if new else 0


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("check")
    sub.add_parser("sync")
    lp = sub.add_parser("layering")
    lp.add_argument("--ratchet", action="store_true")
    lp.add_argument("--update-baseline", action="store_true")
    args = p.parse_args()
    if args.cmd == "check":
        return cmd_check()
    if args.cmd == "sync":
        return cmd_sync()
    return cmd_layering(args.ratchet, args.update_baseline)


if __name__ == "__main__":
    sys.exit(main())
