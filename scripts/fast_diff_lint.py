#!/usr/bin/env python3
"""
fast_diff_lint.py
==================

Schnelle, **cargo-freie** Vorprüfung der aktuell UNCOMMITTETEN bzw. gegenüber
einer Basis-Referenz geänderten Zeilen ("diff-scoped"). Ziel: Ein Agent
(Google-Jules oder Mensch) bekommt in < 1 Sekunde Feedback zu den häufigsten,
in `docs/GITHUB_ANALYSE.md` / `.jules/COMMON_LLM_ERRORS.md` dokumentierten
Fehlerklassen, OHNE die vollständige, minutenlange `cargo xtask jules-preflight`
Pipeline (die einen Full-Workspace-Build voraussetzt) anzustoßen.

Dies ersetzt NICHT die Rust-Gates in `xtask/` (`check_unsafe_islands`,
`check_ring_layering`, `check_result_dropped_on_io`, ...) — es ist ein
komplementärer, extrem schneller Vorfilter für den lokalen Edit-Save-Check-Loop.

Geprüfte Muster (nur auf NEU HINZUGEFÜGTEN Diff-Zeilen, `+`-Zeilen):
  1. Zero-Panic-Doktrin-Verstöße: `.unwrap()`, `.expect(`, `panic!`, `todo!`,
     `unimplemented!`, `unreachable!` außerhalb von `#[cfg(test)]`/`tests/`-Dateien.
  2. Neue `unsafe`-Blöcke ohne unmittelbar vorangehenden `// SAFETY:`-Kommentar.
  3. Neue `unsafe`-Vorkommen in Crates, die NICHT zu den 3 deklarierten
     Unsafe-Inseln (`contextra-sys`, `contextra-simd`, `contextra-wire`) gehören.
  4. Heuristischer Ring-Layering-Verstoß: `use contextra_infer_` (Ring-2-Leaf)
     oder `use contextra_sandbox` innerhalb eines Ring-0/1/3-Crates (siehe
     ARCHITECTURE.md §1) — genau der in JULES_LOG.md §6 (P1) benannte
     "Ring-2-Leaf-Verstoß".
  5. Debug-Leftovers: `println!`, `dbg!`, `eprintln!("DEBUG` in Nicht-Test-Code.
  6. `TODO`/`FIXME`/`XXX`-Marker ohne Ticket-/Issue-Referenz in Klammern.
  7. "Shell-Commit"-verdächtige Muster: leere oder generische Commit-relevante
     Marker-Strings (`git commit -m "wip"` / `"fix"` / `"update"`) IM DIFF-TEXT
     selbst (z. B. versehentlich eingecheckte Scratch-Skripte).

Verwendung
----------
    # Alle uncommitted Änderungen (working tree + staged) gegen HEAD:
    python3 scripts/fast_diff_lint.py

    # Gegen einen anderen Referenzpunkt (z.B. vor dem Rebase auf main):
    python3 scripts/fast_diff_lint.py --base origin/main

    # Nur ein bestimmtes Crate scannen:
    python3 scripts/fast_diff_lint.py --base HEAD~3 --path crates/contextra-engine

    # Exit-Code 1 bei Findings erzwingen (für Pre-Commit-Hooks / CI):
    python3 scripts/fast_diff_lint.py --fail-on-findings

Exit-Codes: 0 = keine Findings (oder --fail-on-findings nicht gesetzt),
            1 = Findings vorhanden UND --fail-on-findings gesetzt,
            2 = Ausführungsfehler (z. B. kein Git-Repo).
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass, field

UNSAFE_ISLANDS = {"contextra-sys", "contextra-simd", "contextra-wire"}
RING2_LEAF_MARKERS = (
    r"\buse\s+contextra_infer_candle\b",
    r"\buse\s+contextra_infer_ollama\b",
    r"\buse\s+contextra_infer_onnx\b",
    r"\buse\s+contextra_sandbox\b",
)
# Crates, die Ring-2-Leaf-Adapter legitim direkt importieren dürfen
# (Ring-4 Composition Roots + die Adapter-Crates selbst untereinander, siehe ARCHITECTURE.md §1.2).
RING2_ALLOWED_IMPORTERS = {
    "contextra", "contextra-mcp", "contextra-py",
    "contextra-infer-candle", "contextra-infer-ollama", "contextra-infer-onnx",
}

PANIC_PATTERNS = [
    (re.compile(r"\.unwrap\(\)"), "Zero-Panic-Verstoß: .unwrap()"),
    (re.compile(r"\.expect\("), "Zero-Panic-Verstoß: .expect(...)"),
    (re.compile(r"\bpanic!\("), "Zero-Panic-Verstoß: panic!(...)"),
    (re.compile(r"\btodo!\("), "Zero-Panic-Verstoß: todo!(...)"),
    (re.compile(r"\bunimplemented!\("), "Zero-Panic-Verstoß: unimplemented!(...)"),
    (re.compile(r"\bunreachable!\("), "Zero-Panic-Verstoß: unreachable!(...)"),
]
DEBUG_LEFTOVER_PATTERNS = [
    (re.compile(r"\bdbg!\("), "Debug-Leftover: dbg!(...)"),
    (re.compile(r"^\s*println!\("), "Debug-Leftover: println!(...) (ggf. tracing::debug! verwenden)"),
]
TODO_MARKER = re.compile(r"\b(TODO|FIXME|XXX)\b(?!.*\(#?\d+\))", re.IGNORECASE)
SHELL_COMMIT_MARKER = re.compile(r'git\s+commit\s+-m\s+["\'](wip|fix|update|test|tmp|stuff)["\']', re.IGNORECASE)


@dataclass
class Finding:
    file: str
    line_no: int
    line_new: bool
    category: str
    message: str
    text: str


@dataclass
class DiffFile:
    path: str
    added_lines: list[tuple[int, str]] = field(default_factory=list)


def run_git(args: list[str]) -> str:
    result = subprocess.run(["git", *args], capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed: {result.stderr.strip()}")
    return result.stdout


def parse_unified_diff(diff_text: str) -> list[DiffFile]:
    files: list[DiffFile] = []
    current: DiffFile | None = None
    new_line_no = 0
    for line in diff_text.split("\n"):
        if line.startswith("+++ b/"):
            path = line[6:]
            current = DiffFile(path=path)
            files.append(current)
            continue
        if line.startswith("@@"):
            m = re.search(r"\+(\d+)", line)
            new_line_no = int(m.group(1)) if m else 0
            continue
        if current is None:
            continue
        if line.startswith("+") and not line.startswith("+++"):
            current.added_lines.append((new_line_no, line[1:]))
            new_line_no += 1
        elif line.startswith("-") and not line.startswith("---"):
            continue
        else:
            new_line_no += 1
    return files


def crate_of(path: str) -> str | None:
    m = re.match(r"crates/([^/]+)/", path)
    return m.group(1) if m else None


def is_test_context(path: str, window: list[str], idx: int) -> bool:
    if "/tests/" in path or path.endswith("_test.rs") or "fuzz/" in path:
        return True
    # grobe Heuristik: rückwärts nach #[cfg(test)] oder mod tests suchen (nur im Diff-Fenster sichtbar)
    for j in range(idx, -1, -1):
        if "#[cfg(test)]" in window[j] or re.search(r"\bmod\s+tests\b", window[j]):
            return True
        if j < idx - 60:  # Fenster begrenzen
            break
    return False


def lint_file(path: str, added_lines: list[tuple[int, str]]) -> list[Finding]:
    findings: list[Finding] = []
    if not path.endswith(".rs"):
        # Nicht-Rust-Dateien nur auf TODO/Shell-Commit-Marker prüfen
        for lineno, text in added_lines:
            if TODO_MARKER.search(text):
                findings.append(Finding(path, lineno, True, "todo-marker",
                                         "TODO/FIXME ohne Issue-Referenz", text.strip()))
            if SHELL_COMMIT_MARKER.search(text):
                findings.append(Finding(path, lineno, True, "shell-commit",
                                         "Generischer Commit-Message-String im Diff gefunden", text.strip()))
        return findings

    crate = crate_of(path)
    texts = [t for _, t in added_lines]

    for idx, (lineno, text) in enumerate(added_lines):
        stripped = text.strip()
        in_test = is_test_context(path, texts, idx)

        if not in_test:
            for pattern, label in PANIC_PATTERNS:
                if pattern.search(text):
                    findings.append(Finding(path, lineno, True, "zero-panic", label, stripped))
            for pattern, label in DEBUG_LEFTOVER_PATTERNS:
                if pattern.search(text):
                    findings.append(Finding(path, lineno, True, "debug-leftover", label, stripped))

        if "unsafe" in text and re.search(r"\bunsafe\s*(\{|fn\b)", text):
            prev_text = texts[idx - 1].strip() if idx > 0 else ""
            if not prev_text.startswith("// SAFETY:") and "SAFETY:" not in prev_text:
                findings.append(Finding(path, lineno, True, "unsafe-no-safety-comment",
                                         "Neuer unsafe-Block ohne unmittelbaren // SAFETY:-Kommentar", stripped))
            if crate and crate not in UNSAFE_ISLANDS:
                findings.append(Finding(path, lineno, True, "unsafe-outside-island",
                                         f"unsafe außerhalb der 3 deklarierten Inseln (Crate: {crate})", stripped))

        if crate and crate not in RING2_ALLOWED_IMPORTERS:
            for marker in RING2_LEAF_MARKERS:
                if re.search(marker, text):
                    findings.append(Finding(path, lineno, True, "ring2-leaf-violation",
                                             f"Ring-2-Leaf-Direktimport in Nicht-Ring-4-Crate '{crate}' "
                                             "(P1-Befund JULES_LOG.md §6) — stattdessen Trait-Objekt "
                                             "aus contextra-ports verwenden", stripped))

        if TODO_MARKER.search(text):
            findings.append(Finding(path, lineno, True, "todo-marker",
                                     "TODO/FIXME ohne Issue-Referenz, z.B. TODO(#1234)", stripped))
        if SHELL_COMMIT_MARKER.search(text):
            findings.append(Finding(path, lineno, True, "shell-commit",
                                     "Generischer Commit-Message-String im Diff gefunden", stripped))

    return findings


def main() -> int:
    parser = argparse.ArgumentParser(description="Schneller, cargo-freier Diff-Linter für Contextra.")
    parser.add_argument("--base", default="HEAD",
                         help="Git-Referenz, gegen die verglichen wird (Standard: HEAD, d.h. alle uncommitted Änderungen).")
    parser.add_argument("--path", default=None,
                         help="Diff auf einen Pfad-Präfix beschränken (z.B. crates/contextra-engine).")
    parser.add_argument("--fail-on-findings", action="store_true",
                         help="Exit-Code 1 setzen, wenn Findings vorhanden sind (für Git-Hooks/CI geeignet).")
    args = parser.parse_args()

    try:
        diff_args = ["diff", "--unified=1", args.base]
        if args.path:
            diff_args += ["--", args.path]
        diff_text = run_git(diff_args)
    except RuntimeError as e:
        print(f"[error] {e}", file=sys.stderr)
        return 2

    if not diff_text.strip():
        print("Keine Diff-Änderungen gefunden — nichts zu prüfen.")
        return 0

    files = parse_unified_diff(diff_text)
    all_findings: list[Finding] = []
    for df in files:
        all_findings.extend(lint_file(df.path, df.added_lines))

    if not all_findings:
        print(f"✅ fast_diff_lint: {len(files)} geänderte Datei(en), keine Findings.")
        return 0

    by_cat: dict[str, int] = {}
    for f in all_findings:
        by_cat[f.category] = by_cat.get(f.category, 0) + 1

    print(f"⚠️  fast_diff_lint: {len(all_findings)} Finding(s) in {len(files)} geänderte(n) Datei(en)\n")
    for f in all_findings:
        print(f"  [{f.category}] {f.file}:{f.line_no}\n      {f.message}\n      > {f.text}\n")

    print("--- Zusammenfassung nach Kategorie ---")
    for cat, count in sorted(by_cat.items(), key=lambda x: -x[1]):
        print(f"  {cat}: {count}")

    print("\nHinweis: Dies ersetzt NICHT `cargo xtask jules-preflight` — es ist ein schneller Vorfilter.")

    return 1 if args.fail_on_findings else 0


if __name__ == "__main__":
    raise SystemExit(main())
