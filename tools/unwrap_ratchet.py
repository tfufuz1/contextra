#!/usr/bin/env python3
"""G-13 / R6 (SPEC §11.6, §13.6 SYNC-5b): Ratchet für Panik-Stellen im Produktcode.

Gezählt werden `unwrap()`, `expect(`, `panic!(`, `todo!(`, `unimplemented!(`, `unreachable!(`
je Crate, nur in `src/`, außerhalb von `#[cfg(test)]`-Blöcken und ohne Kommentarzeilen.

  unwrap_ratchet.py                 Ratchet: Anstieg je Crate ist ein Fehler (Exit 1)
  unwrap_ratchet.py --detail        zusätzlich jede Fundstelle ausgeben
  unwrap_ratchet.py --update        Baseline .unwrap-baseline.json auf den Ist-Stand setzen
                                    (nur nach bewusstem Abbau)

Einschränkung: Die Klammerzählung für `#[cfg(test)]` ignoriert Klammern in Zeichenketten
und Blockkommentaren. Das reicht für den Bestand; Abweichungen zeigt `--detail`.
"""
import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / ".unwrap-baseline.json"
PATTERN = re.compile(r"\.unwrap\(\)|\.expect\(|\bpanic!\(|\btodo!\(|\bunimplemented!\(|\bunreachable!\(")


def strip_line_comment(line: str) -> str:
    idx = line.find("//")
    return line if idx < 0 else line[:idx]


def is_cfg_test(stripped: str) -> bool:
    """`#[cfg(test)]`, `#[cfg(all(test, ...))]`; nicht `#[cfg(not(test))]` und nicht Feature-Namen wie "test-support"."""
    if not stripped.startswith("#[cfg("):
        return False
    body = re.sub(r'"[^"]*"', '""', stripped)  # Zeichenketten entfernen
    if re.search(r"\bnot\s*\(", body):
        return False
    return re.search(r"\btest\b", body) is not None


def production_hits(path: Path):
    """Liefert (zeilennummer, zeilentext) für Panik-Stellen außerhalb von #[cfg(test)]."""
    hits = []
    depth = 0
    test_exit_depth = None  # Tiefe, auf die zurückgefallen werden muss, um den Testblock zu verlassen
    pending_cfg_test = False
    for lineno, raw in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
        code = strip_line_comment(raw)
        stripped = code.strip()
        if is_cfg_test(stripped):
            pending_cfg_test = True
        opens = code.count("{")
        closes = code.count("}")
        if pending_cfg_test and "{" in code and test_exit_depth is None:
            test_exit_depth = depth  # Block beginnt in dieser Zeile
            pending_cfg_test = False
        in_test = test_exit_depth is not None
        if not in_test and not stripped.startswith("//") and PATTERN.search(code):
            hits.append((lineno, raw.strip()))
        depth += opens - closes
        if test_exit_depth is not None and depth <= test_exit_depth:
            test_exit_depth = None
    return hits


def crate_dirs():
    dirs = sorted(p for p in (ROOT / "crates").iterdir() if (p / "src").is_dir())
    bench = ROOT / "benchmarks" / "contextra-bench" / "src"
    if bench.is_dir():
        dirs.append(bench.parent)
    return dirs


def is_test_file(f: Path) -> bool:
    """Testdateien, die über `#[cfg(test)] mod tests;` eingebunden werden."""
    return "tests" in f.parts or f.name in ("tests.rs", "test.rs") or f.name.endswith("_tests.rs")


def count_all(detail: bool):
    counts, details = {}, {}
    for d in crate_dirs():
        name = d.name
        total = 0
        found = []
        for f in sorted((d / "src").rglob("*.rs")):
            if is_test_file(f):
                continue
            for lineno, text in production_hits(f):
                total += 1
                if detail:
                    found.append(f"{f.relative_to(ROOT)}:{lineno}: {text}")
        counts[name] = total
        details[name] = found
    return counts, details


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--detail", action="store_true")
    p.add_argument("--update", action="store_true")
    args = p.parse_args()

    counts, details = count_all(args.detail)

    if args.update:
        BASELINE.write_text(json.dumps(counts, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"Baseline geschrieben: {sum(counts.values())} Stellen in {len(counts)} Crates -> {BASELINE.name}")
        return 0

    base = json.loads(BASELINE.read_text(encoding="utf-8")) if BASELINE.exists() else {}
    increased = []
    for name in sorted(counts):
        now = counts[name]
        was = base.get(name, 0)
        if now > was:
            increased.append(f"{name}: {was} -> {now}")
        if args.detail:
            for line in details[name]:
                print(f"  {line}")
    for name in sorted(base):
        if name not in counts:
            print(f"hinweis: Baseline-Crate {name} existiert nicht mehr (--update)")

    total_now, total_base = sum(counts.values()), sum(base.values())
    print("unwrap/expect/panic in Produktcode je Crate (Baseline -> Ist):")
    for name in sorted(counts):
        print(f"  {name:24} {base.get(name, 0):6} -> {counts[name]}")
    print(f"gesamt: {total_base} -> {total_now}")
    if increased:
        print("ANSTIEG (Ratchet verletzt):")
        for line in increased:
            print(f"  {line}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
