#!/usr/bin/env python3
"""
contextra_full_audit.py (v3) - Befund-Pruefung, Produkt-Pruefung, Tests, Lints, Gates, Benchmarks, Fuzz, Mutation.

Aufruf im Repo-Root (dort, wo die Cargo.toml mit [workspace] liegt):

  python3 scripts/contextra_full_audit.py --list                 # Plan zeigen, nichts ausfuehren (kein cargo noetig)
  python3 scripts/contextra_full_audit.py --gruppe befunde       # nur statische Befund-Pruefung (kein Build)
  python3 scripts/contextra_full_audit.py --quick                # Preflight, check, fmt, clippy, tests (ohne Benchmarks)
  python3 scripts/contextra_full_audit.py --produkt              # nur Produktmenge + Huelle + SQLCipher-Probe
  python3 scripts/contextra_full_audit.py                        # Standard: alles inkl. Benchmarks
  python3 scripts/contextra_full_audit.py --all                  # zusaetzlich Fuzz, Mutation, Loom, Miri, ASan, Coverage
  python3 scripts/contextra_full_audit.py --only store,ingest    # nur bestimmte Crates (Kurzname)
  python3 scripts/contextra_full_audit.py --resume evidence/audit-...   # PASS/INFO-Schritte werden uebernommen
  python3 scripts/contextra_full_audit.py --vergleich evidence/audit-A evidence/audit-B

Weitere Optionen: --fail-fast, --jobs N, --timeout-faktor F, --probe, --root PFAD

Aenderungen gegenueber v2 (geprueft am Repo tfufuz1/contextra @ 7a639b7):
  * Workspace-Mitglieder werden direkt aus Cargo.toml gelesen (kein cargo metadata). v2 fiel bei
    fehlender Toolchain still auf Verzeichnisnamen zurueck; das traf z. B. tests/arch falsch.
  * Toolchain-Preflight: laesst sich der gepinnte Rust-Toolchain (rust-toolchain.toml) nicht nutzen,
    werden alle cargo-Schritte SKIPPED mit Grund, statt dutzendfach FAIL zu erzeugen.
  * befund_adr: fehlendes docs/decisions gilt nicht mehr als PASS; ADRs werden in allen Crates gesucht.
  * befund_wal_aead_tail: v2 meldete einen Widerspruch, der keiner ist (AEAD-Entschluesselungsfehler am
    Ende vs. Parse-Fehler nach erfolgreicher Entschluesselung). Jetzt INFO mit Begruendung.
  * befund_safe_fn_mit_unsafe_inhalt: generisch (pub fn ohne unsafe mit unsafe-Block im Rumpf) statt
    hartcodierter Funktionsnamen.
  * nextest-Ergebnisse werden ausgewertet (v2 zaehlte immer 0).
  * Panic-Flaeche: Dateipfade werden ueber Workspace-Mitglieder auf Crates abgebildet (benchmarks/ war falsch).
  * unsafe-Inventar in Python; Widerspruch zu capabilities.toml (unsafe_island = false) wird gemeldet.
  * Lib-only-Schritte (--lib, --doc) nur fuer Crates mit Library-Target; sonst FAIL statt "no library targets".
  * Kindprozesse werden bei Timeout/Abbruch als ganze Prozessgruppe beendet.
  * --only mit unbekanntem Crate bricht ab, statt still den ganzen Workspace zu pruefen.
  * Bench-/Fuzz-Dateien werden als Befund geprueft (fehlende Dateien = FAIL, nicht stillschweigend ausgelassen).

Ergebnisse: evidence/audit-<UTC>-<commit>/
  summary.md          Zusammenfassung (rote Schritte zuerst), Toolchain, Reife-Behauptung vs. Beleg
  results.json/.csv   jeder Schritt mit Status, Dauer, Detail, Log
  panic_flaeche.csv   Panic-/Unwrap-Stellen im Produktivcode je Crate (Clippy, lib, ohne Tests)
  unsafe_inventar.csv unsafe-Stellen je Crate gegen capabilities.toml
  bench_results.csv   Criterion-Messwerte je Benchmark
  logs/               voller Output je Schritt
  criterion/          kopierte Criterion-Berichte
  env.txt             Toolchain, Host, git-Stand

Nur Python-Standardbibliothek (3.11+). Der Quellcode des Projekts wird nie veraendert.
Hinweis: Prozessgruppen (start_new_session) sind POSIX-spezifisch; das Skript ist fuer Linux/macOS gedacht.
"""
from __future__ import annotations

import argparse
import csv
import datetime as dt
import json
import os
import platform
import re
import shutil
import signal
import subprocess
import sys
import time
import traceback
from collections import defaultdict
from pathlib import Path
from typing import Callable, Optional

if sys.version_info < (3, 11):
    sys.exit("Python 3.11+ erforderlich (tomllib).")
import tomllib  # noqa: E402

# ----------------------------------------------------------------------------------------------
# Konfiguration
# ----------------------------------------------------------------------------------------------

BENCHES = {
    "contextra-mvcc": ["mvcc_bench"],
    "contextra-checkpoint": ["checkpoint_bench"],
    "contextra-router": ["router_bench", "bandit_latency_gate", "cost_routing_savings_bench"],
    "contextra-graph": ["csr_traversal_bench", "binary_edge_regression"],
    "contextra-crypto": ["crypto_benchmarks", "deletion_proof_latency_bench"],
    "contextra-infer-candle": ["kv_bridge_bench"],
    "contextra-vector": [
        "distance_bench", "sq8_bench", "hnsw_bench", "hnsw_delete_search_bench", "audit_benchmarks",
        "flush_threshold_amplification", "partial_rebuild_recall_bench", "distance_nan_impact_bench",
        "filtered_search_selectivity_bench",
    ],
    "contextra-store": ["sstable_bench", "memtable_bench", "wal_bench", "lsm_concurrency_bench",
                        "block_cache_bench", "redb_comparative_bench"],
    "contextra-simd": ["simd_bench"],
    "contextra-bench": ["competitive_kv", "locomo", "long_mem_eval"],
}

FUZZ_CRATES = ["ingest", "mcp", "text", "db", "crypto", "vector", "store", "wire"]

# Produktmenge laut Entscheidung (SOURCE OF TRUTH). avv-generator nur, weil privacy daran haengen kann.
PRODUKT = ["contextra-types", "contextra-ports", "contextra-rank", "contextra-ingest", "contextra-privacy",
           "contextra-infer-ollama", "contextra-audit-export", "contextra-testkit", "contextra-avv-generator"]
# Diese duerfen in der Abhaengigkeitshuelle der Produktmenge NICHT vorkommen:
VERBOTEN_IM_PRODUKT = ["contextra-engine", "contextra-db", "contextra-store", "contextra-graph",
                       "contextra-cognition", "contextra-adapt", "contextra-core", "contextra",
                       "contextra-vector", "contextra-text", "contextra-simd", "contextra-mvcc",
                       "contextra-checkpoint", "contextra-sandbox"]

K78_KNOWN = {"mmap_readonly", "reopen_and_dup2"}   # aus K7/K8 (Ausgangsbefund)
SAFE_WRAPPER_SCOPE = "contextra-sys"          # Crate, in dem pub-fn ohne unsafe nicht unsafe-Inhalt haben duerfen
PREFLIGHT = "toolchain_preflight"
SLOW_CRATES = {"contextra-crypto": 3600, "contextra-store": 3600, "contextra-vector": 3600,
               "contextra-graph": 3600, "contextra-engine": 3600}
DEFAULT_TIMEOUT = 1800
BENCH_TIMEOUT = 3600
FUZZ_SECONDS = 120
CRATES_IO_PROBE = ["rusqlite", "axum", "clap", "tokio", "tower-http", "argon2", "hkdf", "serde_json"]

PANIC_KINDS = [("unwrap", "unwrap()"), ("expect", "expect("), ("panic", "panic!"), ("todo", "todo!"),
               ("unimplemented", "unimplemented!"), ("index", "may panic")]


def now_utc() -> dt.datetime:
    return dt.datetime.now(dt.timezone.utc)


def short(name: str) -> str:
    return name.removeprefix("contextra-")


def which(name: str) -> Optional[str]:
    return shutil.which(name)


def _read(p: Path) -> str:
    try:
        return p.read_text(errors="replace")
    except OSError:
        return ""


def _toml(p: Path) -> dict:
    try:
        return tomllib.loads(p.read_text(errors="replace"))
    except (OSError, tomllib.TOMLDecodeError):
        return {}


def has_lib(d: Path) -> bool:
    return (d / "src" / "lib.rs").exists() or "lib" in _toml(d / "Cargo.toml")


def locked_version(root: Path, pkg: str) -> Optional[str]:
    for p in _toml(root / "Cargo.lock").get("package", []):
        if p.get("name") == pkg:
            return p.get("version")
    return None


# ----------------------------------------------------------------------------------------------
# Workspace-Modell (aus Cargo.toml, ohne cargo)
# ----------------------------------------------------------------------------------------------

class Workspace:
    def __init__(self, root: Path):
        self.root = root
        ws = _toml(root / "Cargo.toml").get("workspace", {})
        self.members: dict[str, Path] = {}     # Paketname -> absolutes Verzeichnis
        self.problems: list[str] = []
        for pat in ws.get("members", []):
            hits = sorted(root.glob(pat)) if "*" in pat else [root / pat]
            for d in hits:
                name = (_toml(d / "Cargo.toml").get("package") or {}).get("name")
                if not name:
                    self.problems.append(f"Mitglied {pat}: kein [package].name in {d.relative_to(root)}/Cargo.toml")
                    continue
                self.members[name] = d.resolve()
        by_dir = {p: n for n, p in self.members.items()}
        self.default: list[str] = []
        for pat in ws.get("default-members", []):
            d = (root / pat).resolve()
            if d in by_dir:
                self.default.append(by_dir[d])
            else:
                self.problems.append(f"default-member {pat} ist kein Workspace-Mitglied")

    def names(self) -> list[str]:
        return sorted(self.members)

    def dir_map(self) -> dict[str, str]:
        return {str(p.relative_to(self.root)): n for n, p in self.members.items()}


class Ctx:
    def __init__(self, root: Path):
        self.root = root
        self.ws = Workspace(root)
        self.caps: dict = _toml(root / "capabilities.toml").get("crates", {})
        self.pin = (_toml(root / "rust-toolchain.toml").get("toolchain") or {}).get("channel", "(kein Pin)")


def crate_for_path(path: str, dir_map: dict[str, str]) -> str:
    best: Optional[tuple[str, str]] = None
    for rel, name in dir_map.items():
        if path == rel or path.startswith(rel + "/"):
            if best is None or len(rel) > len(best[0]):
                best = (rel, name)
    return best[1] if best else path.split("/")[0]


# ----------------------------------------------------------------------------------------------
# Prozesse (ganze Gruppe beenden, damit keine Test-Binaries verwaist weiterlaufen)
# ----------------------------------------------------------------------------------------------

def _kill_group(p: subprocess.Popen) -> None:
    for sig in (signal.SIGTERM, signal.SIGKILL):
        try:
            os.killpg(p.pid, sig)
        except ProcessLookupError:
            return
        try:
            p.wait(timeout=15)
            return
        except subprocess.TimeoutExpired:
            continue


def run_logged(cmd: list[str], cwd: Path, env: dict, out_fh, timeout: int) -> tuple[Optional[int], bool]:
    """Ausgabe in out_fh. Rueckgabe: (returncode | None bei Timeout, timed_out)."""
    p = subprocess.Popen(cmd, cwd=cwd, env=env, stdout=out_fh, stderr=subprocess.STDOUT, start_new_session=True)
    try:
        return p.wait(timeout=timeout), False
    except subprocess.TimeoutExpired:
        _kill_group(p)
        return None, True
    except KeyboardInterrupt:
        _kill_group(p)
        raise


def run_capture(cmd: list[str], cwd: Path, env: dict, timeout: int) -> tuple[Optional[int], str, str, bool]:
    p = subprocess.Popen(cmd, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                         text=True, start_new_session=True)
    try:
        out, err = p.communicate(timeout=timeout)
        return p.returncode, out or "", err or "", False
    except subprocess.TimeoutExpired:
        _kill_group(p)
        out, err = p.communicate()
        return None, out or "", err or "", True
    except KeyboardInterrupt:
        _kill_group(p)
        raise


# ----------------------------------------------------------------------------------------------
# Step
# ----------------------------------------------------------------------------------------------

PyFn = Callable[[Ctx, Path], "tuple[str, str, str]"]  # (ctx, outdir) -> (status, detail, logtext)


class Step:
    def __init__(self, group: str, name: str, cmd: Optional[list[str]] = None, *, timeout: int = DEFAULT_TIMEOUT,
                 requires: Optional[list[str]] = None, env: Optional[dict] = None, cwd: Optional[Path] = None,
                 note: str = "", py: Optional[PyFn] = None, info: bool = False, cargo: Optional[bool] = None):
        self.group, self.name, self.cmd = group, name, cmd or []
        self.timeout, self.requires = timeout, requires or []
        self.env, self.cwd, self.note, self.py = env or {}, cwd, note, py
        self.info = info  # Messung: Exitcode 0 -> INFO statt PASS
        self.needs_cargo = cargo if cargo is not None else (bool(self.cmd) and self.cmd[0] == "cargo")
        self.status, self.rc, self.seconds = "PENDING", None, 0.0
        self.log, self.detail = "", ""


# ----------------------------------------------------------------------------------------------
# Parser
# ----------------------------------------------------------------------------------------------

def parse_cargo_test_summary(text: str) -> str:
    passed = failed = ignored = 0
    for m in re.finditer(r"test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored", text):
        passed += int(m.group(1)); failed += int(m.group(2)); ignored += int(m.group(3))
    failing = sorted(set(re.findall(r"^test (\S+) \.\.\. FAILED", text, re.M)))
    s = f"passed={passed} failed={failed} ignored={ignored}"
    if failing:
        s += " | FAILED: " + ", ".join(failing[:15]) + (" ..." if len(failing) > 15 else "")
    return s


def parse_nextest_summary(text: str) -> str:
    # nextest-Summary z. B.: "Summary [ 1.2s] 12 tests run: 11 passed (1 slow), 1 failed, 0 skipped"
    lines = re.findall(r"^.*tests? run: .*$", text, re.M)
    if not lines:
        return "nextest: keine Summary-Zeile gefunden"
    last = lines[-1]
    p = re.search(r"(\d+) passed", last)
    f = re.search(r"(\d+) failed", last)
    failing = re.findall(r"^\s*(?:FAIL|TIMEOUT|SIGSEGV|ABORT)\s+\[[^\]]*\]\s+(\S.*?)\s*$", text, re.M)
    s = f"passed={p.group(1) if p else 0} failed={f.group(1) if f else 0}"
    if failing:
        s += " | FAILED: " + ", ".join(sorted(set(failing))[:15])
    return s


def parse_clippy(text: str) -> str:
    warns = len(re.findall(r"^warning: (?!`)", text, re.M))
    errs = len(re.findall(r"^error(\[\w+\])?: ", text, re.M))
    return f"errors={errs} warnings={warns}"


def parse_criterion(text: str) -> str:
    hits = re.findall(r"^(\S.*?)\s+time:\s+\[(.+?)\]", text, re.M)
    return f"{len(hits)} Messpunkte" if hits else "keine Criterion-Zeilen erkannt"


def parse_panic_surface(text: str, dir_map: dict[str, str]) -> dict[str, dict[str, int]]:
    out: dict[str, dict[str, int]] = defaultdict(lambda: defaultdict(int))
    for m in re.finditer(r"^warning: (.+)\n\s+--> ([^:\n]+):\d+:\d+", text, re.M):
        msg, path = m.group(1), m.group(2)
        kind = next((k for k, needle in PANIC_KINDS if needle in msg), None)
        if kind:
            out[crate_for_path(path, dir_map)][kind] += 1
    return out


# ----------------------------------------------------------------------------------------------
# Statische Befund-Pruefungen (reines Python, kein Build).
# FAIL = Befund liegt vor. INFO = Messung/Hinweis ohne Urteil.
# ----------------------------------------------------------------------------------------------

def befund_cfg_features(ctx: Ctx, outdir: Path):
    """K1/K11: cfg(feature = "x") im Code, aber x ist in keiner Cargo.toml des Crates definiert."""
    lines, bad = [], 0
    for name, d in sorted(ctx.ws.members.items()):
        ct = _toml(d / "Cargo.toml")
        feats = set((ct.get("features") or {}).keys())
        for dname, dv in (ct.get("dependencies") or {}).items():
            if isinstance(dv, dict) and dv.get("optional"):
                feats.add(dname)
        used: dict[str, set[str]] = defaultdict(set)
        for rs in d.rglob("*.rs"):
            if {"target", "fuzz"} & set(rs.parts):
                continue
            for m in re.finditer(r'feature\s*=\s*"([^"]+)"', _read(rs)):
                used[m.group(1)].add(str(rs.relative_to(ctx.root)))
        und = {f: sorted(v) for f, v in used.items() if f not in feats}
        if und:
            bad += 1
            for f, files in sorted(und.items()):
                lines.append(f"{name}: feature `{f}` ({len(files)} Dateien, z. B. {files[0]})")
        if "allow(unexpected_cfgs)" in _read(d / "src" / "lib.rs"):
            lines.append(f"{name}: #![allow(unexpected_cfgs)] verdeckt moegliche Luecken")
    detail = f"{bad} Crates mit undefinierten Features" if bad else "keine undefinierten Features"
    return ("FAIL" if bad else "PASS"), detail, "\n".join(lines) or "ok"


def befund_wal_tail(ctx: Ctx, outdir: Path):
    """K3: AEAD-Toleranz am WAL-Ende. Kein Widerspruch: zwei verschiedene Fehlerklassen."""
    p = ctx.root / "crates/contextra-store/src/wal/replay.rs"
    t = _read(p)
    tol = [i + 1 for i, l in enumerate(t.splitlines()) if "Tolerated as incomplete tail write" in l]
    neg = [i + 1 for i, l in enumerate(t.splitlines()) if "never a tail" in l]
    if not tol and not neg:
        return "INFO", "Marker nicht gefunden (Datei umbenannt?)", ""
    text = ("Toleranz (AEAD-Entschluesselungsfehler am Dateiende) Z.%s; Ablehnung (Parsefehler nach erfolgreicher "
            "Entschluesselung) Z.%s. Kein Widerspruch, da verschiedene Fehlerklassen; Policy bitte bestaetigen."
            % (tol, neg))
    return "INFO", text, ""


PUB_FN = re.compile(r"(?m)^[ \t]*pub(?:\([^)]*\))?[ \t]+fn[ \t]+(\w+)[^{;]*\{")


def _fn_bodies_with_unsafe(text: str) -> list[tuple[str, int]]:
    hits = []
    for m in PUB_FN.finditer(text):
        start = m.end() - 1
        depth, j = 0, start
        while j < len(text):
            c = text[j]
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        if re.search(r"\bunsafe\s*\{", text[start:j]):
            hits.append((m.group(1), text.count("\n", 0, m.start()) + 1))
    return hits


def befund_safe_unsafe_fns(ctx: Ctx, outdir: Path):
    """K7/K8: `pub fn` ohne `unsafe` mit `unsafe {}`-Block im Rumpf (Heuristik: Klammerzaehlung ohne Strings)."""
    d = ctx.ws.members.get(SAFE_WRAPPER_SCOPE)
    if not d:
        return "INFO", f"{SAFE_WRAPPER_SCOPE} nicht im Workspace", ""
    hits = []
    for rs in (d / "src").rglob("*.rs"):
        for fn, line in _fn_bodies_with_unsafe(_read(rs)):
            hits.append((fn, f"{rs.relative_to(ctx.root)}:{line}: pub fn {fn} (safe Signatur, unsafe Rumpf)"))
    known = [h for fn, h in hits if fn in K78_KNOWN]
    text = "\n".join(h for _, h in hits)
    if known:
        return "FAIL", f"{len(known)} bekannte K7/K8-Stellen ({', '.join(sorted(K78_KNOWN))}) | gesamt {len(hits)} Kandidaten", text
    return "INFO", f"{len(hits)} Kandidaten zur Pruefung (cfg-Varianten und dokumentierte Wrapper moeglich)", text


def befund_ingest_wiring(ctx: Ctx, outdir: Path):
    """contextra-ingest: Abnehmer vorhanden, keine Abhaengigkeit zum Kern, Module deklariert."""
    name = "contextra-ingest"
    d = ctx.ws.members.get(name)
    if not d:
        return "FAIL", f"{name} ist kein Workspace-Mitglied", ""
    consumers = [n for n, p in ctx.ws.members.items() if n != name and name in _read(p / "Cargo.toml")]
    deps = _toml(d / "Cargo.toml").get("dependencies") or {}
    problems = []
    if not consumers:
        problems.append("kein Workspace-Crate haengt von contextra-ingest ab")
    if "contextra-engine" in deps:
        problems.append("ingest haengt direkt an contextra-engine (Produktmenge darf den Kern nicht ziehen)")
    files = list((d / "src").rglob("*.rs"))
    refs = [str(p.relative_to(ctx.root)) for p in files if "paged_impl" in _read(p)]
    declared = any(p.stem == "paged_impl" for p in files) or \
        any(re.search(r"\bmod\s+paged_impl\b", _read(p)) for p in files)
    if refs and not declared:
        problems.append(f"paged_impl wird benutzt, aber nirgends deklariert ({', '.join(refs)})")
    return ("FAIL" if problems else "PASS"), "; ".join(problems) or "verdrahtet", f"consumers={consumers}"


def befund_default_members(ctx: Ctx, outdir: Path):
    ws = ctx.ws
    missing = [c for c in PRODUKT if c not in ws.members]
    notdef = [c for c in PRODUKT if c in ws.members and c not in ws.default]
    detail = (f"members={len(ws.members)} default-members={len(ws.default)}; "
              f"Produkt nicht Mitglied: {missing}; Produkt nicht in default-members: {notdef}")
    bad = bool(missing or notdef or ws.problems)
    return ("FAIL" if bad else "PASS"), detail, "\n".join(ws.problems)


def befund_adr(ctx: Ctx, outdir: Path):
    """ADR-Verzeichnisse im Root und in allen Crates. Fehlt alles, ist das Ergebnis INFO, nie PASS."""
    dirs = [ctx.root / "docs" / "decisions"] + [p / "docs" / "decisions" for p in ctx.ws.members.values()]
    found = [d for d in dirs if d.is_dir()]
    if not found:
        return "INFO", "kein docs/decisions im Root und in keinem Crate gefunden", ""
    files, proposed = [], []
    for d in found:
        for f in sorted(d.glob("*.md")):
            files.append(str(f.relative_to(ctx.root)))
            m = re.search(r"\*\*Status\*\*:\s*(\w+)", _read(f))
            if m and m.group(1).lower() == "proposed":
                proposed.append(f.name)
    detail = f"{len(files)} ADR-Dateien in {len(found)} Verzeichnissen; Proposed: {proposed or 'keine'}"
    return ("FAIL" if proposed else "PASS"), detail, "\n".join(files)


def befund_maturity_claims(ctx: Ctx, outdir: Path):
    caps = ctx.caps
    stable = [k for k, v in caps.items() if v.get("maturity") == "stable"]
    beta = [k for k, v in caps.items() if v.get("maturity") == "beta"]
    fehlend = [k for k in caps if k not in ctx.ws.members]
    no_cap = [k for k in ctx.ws.members if k not in caps]
    text = f"stable: {stable}\nbeta: {beta}\nin capabilities.toml, aber kein Mitglied: {fehlend}\nMitglied ohne Eintrag: {no_cap}"
    detail = f"stable={len(stable)} beta={len(beta)} Eintraege ohne Crate={len(fehlend)}"
    return ("FAIL" if fehlend else "INFO"), detail, text


def befund_from_entropy(ctx: Ctx, outdir: Path):
    hits = sorted({str(p.relative_to(ctx.root)) for d in ctx.ws.members.values()
                   for p in (d / "src").rglob("*.rs") if "from_entropy" in _read(p)})
    return ("FAIL" if hits else "PASS"), f"{len(hits)} Produktivdateien mit from_entropy (K14)", "\n".join(hits)


def befund_windows_flush(ctx: Ctx, outdir: Path):
    d = ctx.ws.members.get(SAFE_WRAPPER_SCOPE)
    t = "\n".join(_read(p) for p in (d / "src").rglob("*.rs")) if d else ""
    has_cfg = "cfg(windows)" in t or 'target_os = "windows"' in t
    has_err = "compile_error!" in t
    return "INFO", f"windows-Pfad vorhanden={has_cfg}, compile_error!={has_err}", ""


def befund_bench_fuzz_dateien(ctx: Ctx, outdir: Path):
    """Fehlende Bench-/Fuzz-Dateien sind ein Befund; sonst wuerden Schritte still ausbleiben."""
    missing = []
    for pkg, names in BENCHES.items():
        d = ctx.ws.members.get(pkg)
        if not d:
            missing.append(f"Paket {pkg} fehlt im Workspace")
            continue
        for b in names:
            if not (d / "benches" / f"{b}.rs").exists():
                missing.append(f"{pkg}/benches/{b}.rs")
    for fc in FUZZ_CRATES:
        name = f"contextra-{fc}"
        d = ctx.ws.members.get(name)
        if not d or not (d / "fuzz" / "fuzz_targets").is_dir():
            missing.append(f"{name}/fuzz/fuzz_targets")
    return ("FAIL" if missing else "PASS"), f"{len(missing)} fehlende Dateien/Verzeichnisse", "\n".join(missing)


# ----------------------------------------------------------------------------------------------
# unsafe-Inventar gegen capabilities.toml
# ----------------------------------------------------------------------------------------------

UNSAFE_WORD = re.compile(r"\bunsafe\b")


def _unsafe_lines(path: Path) -> int:
    n = 0
    for line in _read(path).splitlines():
        s = line.strip()
        if s.startswith("//") or not UNSAFE_WORD.search(s):
            continue
        if re.search(r'"[^"]*\bunsafe\b[^"]*"', s):   # Treffer nur im String-Literal
            continue
        n += 1
    return n


def unsafe_inventar(ctx: Ctx, outdir: Path):
    rows, verstoss = [], []
    for name, d in sorted(ctx.ws.members.items()):
        n = sum(_unsafe_lines(p) for p in (d / "src").rglob("*.rs"))
        cap = ctx.caps.get(name)
        island = bool((cap or {}).get("unsafe_island", False))
        if cap is None:
            verdict = "nicht in capabilities.toml"
        elif n > 0 and not island:
            verdict = "VERSTOSS: unsafe ohne unsafe_island=true"
            verstoss.append(name)
        else:
            verdict = "ok"
        rows.append([name, n, island, verdict])
    with open(outdir / "unsafe_inventar.csv", "w", newline="") as fh:
        w = csv.writer(fh)
        w.writerow(["crate", "unsafe_zeilen", "unsafe_island", "urteil"])
        w.writerows(rows)
    detail = f"{len(verstoss)} Verstoesse gegen unsafe_island" if verstoss else "keine Verstoesse"
    return ("FAIL" if verstoss else "PASS"), detail, "\n".join(map(str, rows))


# ----------------------------------------------------------------------------------------------
# Toolchain-Preflight und Produkt-Pruefliste (Tag 1)
# ----------------------------------------------------------------------------------------------

def toolchain_preflight(ctx: Ctx, outdir: Path):
    """Gate fuer alle cargo-Schritte: der Pin aus rust-toolchain.toml muss nutzbar sein."""
    if not which("cargo"):
        return "FAIL", "cargo nicht im PATH", ""
    rc, out, err, to = run_capture(["cargo", "-V"], ctx.root, os.environ.copy(), 900)
    text = out + "\n--- stderr ---\n" + err
    if to:
        return "FAIL", f"cargo -V Zeitlimit (Toolchain-Download haengt?), Pin {ctx.pin}", text
    if rc != 0:
        first = next((l for l in err.splitlines() if l.strip()), "")
        return "FAIL", f"Pin {ctx.pin} nicht nutzbar: {first[:200]}", text
    return "PASS", f"{out.strip()} | Pin {ctx.pin}", text


def produkt_huelle(ctx: Ctx, outdir: Path):
    """Verbotene Crates in der transitiven Normal-Huelle der Produktmenge (cargo tree)."""
    log, bad = [], 0
    for c in PRODUKT:
        if c not in ctx.ws.members:
            log.append(f"{c}: kein Workspace-Mitglied")
            bad += 1
            continue
        rc, out, err, to = run_capture(["cargo", "tree", "-p", c, "-e", "normal", "--prefix", "none", "--locked"],
                                       ctx.root, os.environ.copy(), 600)
        if to or rc != 0:
            log.append(f"{c}: cargo tree fehlgeschlagen: {'Zeitlimit' if to else err.strip()[:200]}")
            bad += 1
            continue
        names = {l.split()[0] for l in out.splitlines() if l.strip()}
        found = sorted(n for n in names if n in VERBOTEN_IM_PRODUKT and n != c)
        if found:
            bad += 1
        log.append(f"{c}: verbotene Abhaengigkeiten={found if found else 'keine'}")
    return ("FAIL" if bad else "PASS"), f"{bad} Produkt-Crates mit Befund", "\n".join(log)


SQLCIPHER_MAIN = r'''
use rusqlite::Connection;
fn main() {
    let path = std::env::args().nth(1).expect("path");
    let _ = std::fs::remove_file(&path);
    {
        let c = Connection::open(&path).unwrap();
        c.execute_batch("PRAGMA key='probe-key-1';").unwrap();
        c.execute_batch("CREATE TABLE t(id INTEGER PRIMARY KEY, body TEXT); INSERT INTO t(body) VALUES ('KLARTEXT_MARKER_4711');").unwrap();
        let v: Result<String, _> = c.query_row("PRAGMA cipher_version", [], |r| r.get(0));
        println!("cipher_version={:?}", v.ok());
        let fts = c.execute_batch("CREATE VIRTUAL TABLE d USING fts5(x); INSERT INTO d(x) VALUES ('Rechnung Mueller');");
        println!("fts5={}", fts.is_ok());
    }
    let bytes = std::fs::read(&path).unwrap();
    let leak = bytes.windows(20).any(|w| w == b"KLARTEXT_MARKER_4711");
    println!("plaintext_leak={}", leak);
    {
        let c = Connection::open(&path).unwrap();
        c.execute_batch("PRAGMA key='falsch';").unwrap();
        let wrong = c.query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get::<_, i64>(0));
        println!("wrong_key_rejected={}", wrong.is_err());
    }
    let c = Connection::open(&path).unwrap();
    c.execute_batch("PRAGMA key='probe-key-1';").unwrap();
    let rows: i64 = c.query_row("SELECT count(*) FROM t", [], |r| r.get(0)).unwrap_or(-1);
    println!("right_key_rows={}", rows);
}
'''


def produkt_sqlcipher_probe(ctx: Ctx, outdir: Path):
    """P0-Gate: SQLCipher baut, verschluesselt, FTS5 vorhanden, falscher Schluessel scheitert.
    rusqlite wird auf die Version aus Cargo.lock gepinnt (Reproduzierbarkeit wie im Projekt)."""
    pdir = outdir / "sqlcipher_probe"
    (pdir / "src").mkdir(parents=True, exist_ok=True)
    ver = locked_version(ctx.root, "rusqlite")
    req = f'"={ver}"' if ver else '"*"'
    (pdir / "Cargo.toml").write_text(
        '[package]\nname = "sqlcipher_probe"\nversion = "0.0.0"\nedition = "2024"\nrust-version = "1.89"\n\n'
        f'[dependencies]\nrusqlite = {{ version = {req}, features = ["bundled-sqlcipher-vendored-openssl"] }}\n\n'
        '[workspace]\n')
    (pdir / "src" / "main.rs").write_text(SQLCIPHER_MAIN)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(ctx.root / "target" / "sqlcipher_probe_target")
    rc, out, err, to = run_capture(["cargo", "run", "--release", "--manifest-path", str(pdir / "Cargo.toml"), "--",
                                    str(pdir / "probe.db")], ctx.root, env, 3000)
    log = out + "\n--- stderr ---\n" + err[-6000:]
    if to:
        return "FAIL", "Zeitlimit", log
    if rc != 0:
        return "FAIL", "Build oder Lauf der Probe fehlgeschlagen (siehe Log; ggf. perl/make/cc fehlen)", log
    kv = dict(re.findall(r"^(\w+)=(.*)$", out, re.M))
    ok = kv.get("plaintext_leak") == "false" and kv.get("wrong_key_rejected") == "true" and kv.get("right_key_rows") == "1"
    fts = kv.get("fts5") == "true"
    detail = (f"rusqlite={ver or '*'} cipher={kv.get('cipher_version')} fts5={kv.get('fts5')} "
              f"leak={kv.get('plaintext_leak')} wrong_key_rejected={kv.get('wrong_key_rejected')} "
              f"rows={kv.get('right_key_rows')}")
    if ok and not fts:
        return "FAIL", detail + " | FTS5 fehlt im SQLCipher-Build", log
    return ("PASS" if ok else "FAIL"), detail, log


def produkt_crates_io(ctx: Ctx, outdir: Path):
    """Informativ: aktuelle Versionen. Netzabhaengig, daher nie FAIL."""
    lines = []
    for c in CRATES_IO_PROBE:
        rc, out, err, to = run_capture(["cargo", "search", c, "--limit", "1"], ctx.root, os.environ.copy(), 120)
        lines.append(f"{c}: " + ("Zeitlimit" if to else (out.splitlines()[0] if out.strip() else err.strip()[:120] or "keine Antwort")))
    return "INFO", f"{len(CRATES_IO_PROBE)} Abfragen", "\n".join(lines)


# ----------------------------------------------------------------------------------------------
# Plan
# ----------------------------------------------------------------------------------------------

def nextest_ok() -> bool:
    return which("cargo-nextest") is not None


def build_plan(ctx: Ctx, args, outdir: Path) -> list[Step]:
    ws = ctx.ws
    crates = ws.names()
    if args.produkt:
        crates = [c for c in crates if c in PRODUKT]
    if args.only:
        wanted = {short(w.strip()) for w in args.only.split(",") if w.strip()}
        unknown = sorted(wanted - {short(c) for c in ws.names()})
        if unknown:
            raise SystemExit(f"--only: unbekannte Crates {unknown} (Kurzname ohne 'contextra-')")
        crates = [c for c in crates if short(c) in wanted]
    if not crates:
        raise SystemExit("Keine Crates ausgewaehlt (--only/--produkt pruefen).")

    sel_all = not (args.only or args.produkt)
    libs = [c for c in crates if has_lib(ws.members[c])]
    pk = ["--workspace"] if sel_all else [x for c in crates for x in ("-p", c)]
    pk_lib = ["--workspace"] if sel_all else [x for c in libs for x in ("-p", c)]
    cargo = ["cargo"]
    S: list[Step] = []

    # --- B: Befund-Pruefung (kein Build) -----------------------------------------------------
    S += [
        Step("B-befunde", "befund_cfg_features_undefiniert", py=befund_cfg_features, note="K1/K11"),
        Step("B-befunde", "befund_wal_aead_tail_hinweis", py=befund_wal_tail, note="K3 (v2-Falschpositiv korrigiert)"),
        Step("B-befunde", "befund_safe_fn_mit_unsafe_inhalt", py=befund_safe_unsafe_fns, note="K7/K8"),
        Step("B-befunde", "befund_ingest_verdrahtung", py=befund_ingest_wiring),
        Step("B-befunde", "befund_default_members", py=befund_default_members),
        Step("B-befunde", "befund_adr_status", py=befund_adr),
        Step("B-befunde", "befund_from_entropy", py=befund_from_entropy, note="K14"),
        Step("B-befunde", "befund_maturity_claims", py=befund_maturity_claims),
        Step("B-befunde", "befund_windows_flush", py=befund_windows_flush, note="K10"),
        Step("B-befunde", "befund_bench_fuzz_dateien", py=befund_bench_fuzz_dateien),
    ]

    # --- T1: Produkt-Pruefliste --------------------------------------------------------------
    if args.produkt or args.all or args.probe:
        S += [
            Step("T1-produkt", "produkt_huelle_ohne_kern", py=produkt_huelle, timeout=1800,
                 note="Produktmenge darf engine/db/store/graph/... nicht transitiv ziehen"),
            Step("T1-produkt", "produkt_sqlcipher_probe", py=produkt_sqlcipher_probe, timeout=3600,
                 note="P0-Gate: SQLCipher baut, verschluesselt, FTS5 vorhanden, falscher Schluessel scheitert"),
            Step("T1-produkt", "produkt_crates_io_versionen", py=produkt_crates_io, info=True, timeout=900),
        ]

    # --- 0: Basis ----------------------------------------------------------------------------
    S += [
        Step("0-basis", PREFLIGHT, py=toolchain_preflight, timeout=1000, cargo=False,
             note="Gate: schlaegt er fehl, werden alle cargo-Schritte SKIPPED"),
        Step("0-basis", "toolchain_versionen", ["bash", "-c",
             "rustc -Vv; cargo -V; cargo clippy -V; cargo fmt -V; rustup show active-toolchain; true"],
             timeout=300, info=True),
        Step("0-basis", "cargo_fetch", cargo + ["fetch", "--locked"], timeout=900),
        Step("0-basis", "cargo_tree_duplicates", cargo + ["tree", "--workspace", "--duplicates"], timeout=300, info=True),
    ]

    # --- 1: Statisch -------------------------------------------------------------------------
    S += [
        Step("1-statisch", "fmt_check", cargo + ["fmt", "--all", "--", "--check"], requires=["rustfmt"]),
        Step("1-statisch", "check_alle_targets", cargo + ["check"] + pk + ["--all-targets", "--locked", "--keep-going"]),
        Step("1-statisch", "check_no_default_features",
             cargo + ["check"] + pk + ["--no-default-features", "--locked", "--keep-going"]),
        Step("1-statisch", "doc", cargo + ["doc"] + pk + ["--no-deps", "--locked"], env={"RUSTDOCFLAGS": "-D warnings"}),
    ]
    for c in crates:
        if c in libs:
            S.append(Step("1-statisch", f"clippy_lib__{c}",
                          cargo + ["clippy", "-p", c, "--lib", "--locked", "--", "-D", "warnings"]))
        S.append(Step("1-statisch", f"clippy_alle_targets__{c}",
                      cargo + ["clippy", "-p", c, "--all-targets", "--locked", "--", "-D", "warnings"]))
    S += [
        Step("1-statisch", "feature_matrix_hack", cargo + ["hack", "check", "--each-feature"] + pk +
             ["--no-dev-deps", "--keep-going"], requires=["cargo-hack"], timeout=3600),
        Step("1-statisch", "deny", cargo + ["deny", "check"], requires=["cargo-deny"]),
        Step("1-statisch", "audit", cargo + ["audit"], requires=["cargo-audit"]),
        Step("1-statisch", "panic_flaeche_lib",
             cargo + ["clippy"] + pk_lib + ["--lib", "--locked", "--keep-going", "--",
                      "-W", "clippy::unwrap_used", "-W", "clippy::expect_used", "-W", "clippy::panic",
                      "-W", "clippy::todo", "-W", "clippy::unimplemented", "-W", "clippy::indexing_slicing"],
             timeout=3600, info=True,
             note="Panic-Stellen im Produktivcode (ohne #[cfg(test)]); Auswertung: panic_flaeche.csv"),
        Step("1-statisch", "unsafe_inventar", py=unsafe_inventar, timeout=300,
             note="gegen capabilities.toml (unsafe_island)"),
    ]

    # --- 2: Tests ----------------------------------------------------------------------------
    use_nextest = nextest_ok()
    for c in crates:
        t = SLOW_CRATES.get(c, DEFAULT_TIMEOUT)
        if c in libs:
            S.append(Step("2-tests", f"test_lib__{c}",
                          cargo + ["test", "-p", c, "--lib", "--locked", "--no-fail-fast"], timeout=t))
        S.append(Step("2-tests", f"test_integration__{c}",
                      cargo + ["test", "-p", c, "--tests", "--locked", "--no-fail-fast"], timeout=t,
                      note="umfasst auch lib-Tests; Duplikate gewollt"))
        if c in libs:
            S.append(Step("2-tests", f"test_doc__{c}",
                          cargo + ["test", "-p", c, "--doc", "--locked", "--no-fail-fast"], timeout=t))
        if use_nextest:
            S.append(Step("2-tests", f"nextest__{c}",
                          cargo + ["nextest", "run", "-p", c, "--locked", "--no-fail-fast", "--retries", "0"],
                          timeout=t, note="je Test eigener Prozess: deckt Test-Interferenz auf"))
    S.append(Step("2-tests", "test_gesamt_debug",
                  cargo + ["test"] + pk + ["--all-targets", "--locked", "--no-fail-fast"], timeout=7200))
    S.append(Step("2-tests", "test_gesamt_release",
                  cargo + ["test"] + pk + ["--release", "--locked", "--no-fail-fast"], timeout=7200,
                  note="Release deckt Overflow-/Optimierungsfehler auf (z. B. simd u8)"))
    py_dir = ws.members.get("contextra-py")
    if sel_all and py_dir and (py_dir / "tests").exists():
        S.append(Step("2-tests", "python_bindings_pytest", ["python3", "-m", "pytest", "-q", str(py_dir / "tests")],
                      requires=["python3"], note="setzt installierte Bindings voraus (maturin develop)"))

    # --- 4: Benchmarks -----------------------------------------------------------------------
    if not args.quick and not args.produkt:
        for pkg, names in BENCHES.items():
            if pkg not in crates:
                continue
            for b in names:
                S.append(Step("4-bench", f"bench__{pkg}__{b}", cargo + ["bench", "-p", pkg, "--bench", b, "--locked"],
                              timeout=BENCH_TIMEOUT, note="locomo/long_mem_eval brauchen evtl. externe Daten"))

    # --- 5: Tief -----------------------------------------------------------------------------
    if args.all:
        for fc in FUZZ_CRATES:
            name = f"contextra-{fc}"
            if name not in crates:
                continue
            d = ws.members[name]
            for tgt in sorted(p.stem for p in (d / "fuzz" / "fuzz_targets").glob("*.rs")):
                S.append(Step("5-tief", f"fuzz__{fc}__{tgt}",
                              ["cargo", "+nightly", "fuzz", "run", tgt, "--", f"-max_total_time={FUZZ_SECONDS}"],
                              cwd=d, requires=["cargo-fuzz"], timeout=FUZZ_SECONDS + 1200))
        S += [
            Step("5-tief", "mutation_tier0_1", cargo + ["mutants", "--config", ".cargo/mutants.toml", "--timeout", "120"],
                 requires=["cargo-mutants"], timeout=14400),
            Step("5-tief", "loom_mvcc", cargo + ["test", "-p", "contextra-mvcc", "--release", "--locked"],
                 env={"RUSTFLAGS": "--cfg loom", "LOOM_MAX_PREEMPTIONS": "3"}, timeout=3600),
            Step("5-tief", "miri_vector_scalar", ["cargo", "+nightly", "miri", "test", "-p", "contextra-vector", "--lib"],
                 requires=["cargo-miri"], timeout=7200),
            Step("5-tief", "asan_vector", ["cargo", "+nightly", "test", "-p", "contextra-vector", "--target",
                 "x86_64-unknown-linux-gnu"], env={"RUSTFLAGS": "-Zsanitizer=address"}, timeout=3600),
            Step("5-tief", "coverage", cargo + ["llvm-cov"] + pk + ["--locked", "--json", "--output-path",
                 str(outdir / "coverage.json")], requires=["cargo-llvm-cov"], timeout=10800),
        ]

    if args.gruppe:
        g = args.gruppe.lower()
        S = [s for s in S if s.group.lower().startswith(g) or s.group.split("-", 1)[-1].lower().startswith(g)]
    return S


# ----------------------------------------------------------------------------------------------
# Ausfuehrung
# ----------------------------------------------------------------------------------------------

def tool_available(step: Step) -> tuple[bool, str]:
    for r in step.requires:
        if r == "cargo-miri":
            if not which("cargo-miri"):
                return False, "cargo-miri fehlt (rustup +nightly component add miri)"
        elif not which(r):
            hint = f" (cargo install {r} --locked)" if r.startswith("cargo-") else ""
            return False, f"{r} nicht gefunden{hint}"
    return True, ""


def run_step(step: Step, idx: int, logdir: Path, ctx: Ctx, outdir: Path, tfactor: float,
             jobs: Optional[int], cargo_ok: Optional[bool]) -> None:
    safe = re.sub(r"[^A-Za-z0-9_.-]", "_", step.name)
    step.log = str(logdir / f"{idx:03d}_{safe}.log")
    if step.needs_cargo and cargo_ok is False:
        step.status, step.detail = "SKIPPED", "Toolchain-Preflight fehlgeschlagen (siehe Schritt " + PREFLIGHT + ")"
        Path(step.log).write_text(f"SKIPPED: {step.detail}\n")
        return
    ok, why = tool_available(step)
    if not ok:
        step.status, step.detail = "SKIPPED", why
        Path(step.log).write_text(f"SKIPPED: {why}\n")
        return
    t0 = time.time()

    if step.py:
        try:
            status, detail, text = step.py(ctx, outdir)
        except Exception as e:  # Pruefung selbst defekt: sichtbar machen, nicht verschweigen
            status, detail, text = "FAIL", f"Pruefung abgestuerzt: {type(e).__name__}: {e}", traceback.format_exc()
        if step.info and status == "PASS":
            status = "INFO"
        step.status, step.detail = status, detail
        Path(step.log).write_text(f"# {step.name}\n{detail}\n\n{text}\n")
        step.seconds = round(time.time() - t0, 1)
        return

    env = os.environ.copy()
    env.update(step.env)
    env.setdefault("CARGO_TERM_COLOR", "never")
    env.setdefault("RUST_BACKTRACE", "1")
    if jobs:
        env["CARGO_BUILD_JOBS"] = str(jobs)
    timeout = int(step.timeout * tfactor)
    header = f"$ {' '.join(step.cmd)}\n# cwd={step.cwd or ctx.root}\n# env+={step.env}\n\n"
    try:
        with open(step.log, "w") as fh:
            fh.write(header)
            fh.flush()
            rc, timed_out = run_logged(step.cmd, step.cwd or ctx.root, env, fh, timeout)
    except FileNotFoundError as e:
        step.status, step.detail = "SKIPPED", f"Befehl nicht gefunden: {e}"
        step.seconds = round(time.time() - t0, 1)
        return
    step.seconds = round(time.time() - t0, 1)
    if timed_out:
        step.status, step.detail = "TIMEOUT", f">{timeout}s (zaehlt als rot)"
    else:
        step.rc = rc
        step.status = "PASS" if rc == 0 else "FAIL"
        if step.info and step.status == "PASS":
            step.status = "INFO"

    text = Path(step.log).read_text(errors="replace").split("\n\n", 1)[-1]  # Kopfzeile nicht mitparsen
    if step.name.startswith("nextest__"):
        step.detail = (step.detail + " " + parse_nextest_summary(text)).strip()
    elif step.name.startswith("test_"):
        step.detail = (step.detail + " " + parse_cargo_test_summary(text)).strip()
        if "could not compile" in text:
            step.detail += " | COMPILE-FEHLER"
            step.status = "FAIL"
        elif step.status == "PASS" and re.search(r"test result: \w+\. 0 passed; 0 failed", text) and \
                not re.search(r"test result: \w+\. [1-9]\d* passed", text):
            step.detail += " | KEINE TESTS AUSGEFUEHRT"
    elif step.name.startswith("clippy") or step.name == "panic_flaeche_lib":
        step.detail = (step.detail + " " + parse_clippy(text)).strip()
    elif step.name.startswith("bench__"):
        step.detail = (step.detail + " " + parse_criterion(text)).strip()
    elif step.name.startswith("check_"):
        step.detail = (step.detail + f" errors={len(re.findall(r'^error', text, re.M))}").strip()


# ----------------------------------------------------------------------------------------------
# Auswertungen
# ----------------------------------------------------------------------------------------------

def export_panic_surface(ctx: Ctx, steps: list[Step], outdir: Path) -> Optional[dict]:
    s = next((x for x in steps if x.name == "panic_flaeche_lib" and x.log and Path(x.log).exists()), None)
    if not s:
        return None
    data = parse_panic_surface(Path(s.log).read_text(errors="replace"), ctx.ws.dir_map())
    with open(outdir / "panic_flaeche.csv", "w", newline="") as fh:
        w = csv.writer(fh)
        w.writerow(["crate", "unwrap", "expect", "panic", "todo", "unimplemented", "index", "summe"])
        for crate, k in sorted(data.items(), key=lambda kv: -sum(kv[1].values())):
            row = [k.get(x, 0) for x in ("unwrap", "expect", "panic", "todo", "unimplemented", "index")]
            w.writerow([crate] + row + [sum(row)])
    return data


def export_bench_csv(root: Path, outdir: Path, since: float) -> int:
    crit = root / "target" / "criterion"
    if not crit.exists():
        return 0
    rows = []
    for est in crit.rglob("new/estimates.json"):
        if est.stat().st_mtime < since:
            continue
        try:
            d = json.loads(est.read_text())
            name = "/".join(est.relative_to(crit).parts[:-2])
            rows.append([name, d["mean"]["point_estimate"], d["median"]["point_estimate"],
                         d["std_dev"]["point_estimate"], d["mean"]["confidence_interval"]["lower_bound"],
                         d["mean"]["confidence_interval"]["upper_bound"]])
        except (OSError, KeyError, ValueError):
            continue
    if rows:
        with open(outdir / "bench_results.csv", "w", newline="") as fh:
            w = csv.writer(fh)
            w.writerow(["benchmark", "mean_ns", "median_ns", "stddev_ns", "mean_ci_low_ns", "mean_ci_high_ns"])
            w.writerows(sorted(rows))
    return len(rows)


def maturity_table(ctx: Ctx, steps: list[Step]) -> list[str]:
    by = {s.name: s for s in steps}
    lines = ["", "## Reife-Behauptung vs. Beleg (capabilities.toml gegen diesen Lauf)", "",
             "| Crate | behauptet | lib-Tests | Clippy lib | Urteil |", "|---|---|---|---|---|"]
    for name, v in sorted(ctx.caps.items()):
        claimed = v.get("maturity", "?")
        t, c = by.get(f"test_lib__{name}"), by.get(f"clippy_lib__{name}")
        if not t and not c:
            continue
        ts = t.status if t else "-"
        cs = c.status if c else "-"
        if claimed in ("stable", "beta") and (ts not in ("PASS", "-") or cs not in ("PASS", "-")):
            verdict = "**Behauptung ohne Beleg**"
        else:
            verdict = "ok" if ts == "PASS" and cs == "PASS" else "-"
        lines.append(f"| `{name}` | {claimed} | {ts} | {cs} | {verdict} |")
    return lines


def write_reports(ctx: Ctx, steps: list[Step], outdir: Path, meta: dict, final: bool = False) -> None:
    rows = [{"nr": i, "gruppe": s.group, "schritt": s.name, "status": s.status, "rc": s.rc, "sekunden": s.seconds,
             "detail": s.detail, "log": os.path.relpath(s.log, outdir) if s.log else "",
             "befehl": " ".join(s.cmd), "hinweis": s.note} for i, s in enumerate(steps)]
    (outdir / "results.json").write_text(json.dumps({"meta": meta, "steps": rows}, indent=2, ensure_ascii=False))
    if rows:
        with open(outdir / "results.csv", "w", newline="") as fh:
            w = csv.DictWriter(fh, fieldnames=list(rows[0].keys()))
            w.writeheader()
            w.writerows(rows)

    counts: dict[str, int] = defaultdict(int)
    for s in steps:
        counts[s.status] += 1
    order = {"FAIL": 0, "TIMEOUT": 1, "SKIPPED": 2, "INFO": 3, "PASS": 4, "PENDING": 5}
    pre = next((s for s in steps if s.name == PREFLIGHT), None)
    L = ["# Contextra Voll-Audit", "",
         f"- Commit: `{meta['commit']}` ({meta['commit_date']}), Branch `{meta.get('branch', '')}`, "
         f"hinter origin: {meta.get('behind', '?')}, ungespeicherte Aenderungen: {meta.get('dirty')}",
         f"- Start: {meta['start']}  Dauer: {meta['duration_min']} min",
         f"- Toolchain-Pin: `{ctx.pin}`; Preflight: **{pre.status if pre else 'nicht gelaufen'}** "
         f"({pre.detail if pre else ''})",
         f"- Host: {meta['host']}", f"- Nutzer: {meta['user']}", "", "## Gesamt", ""]
    L += [f"- **{k}**: {v}" for k, v in sorted(counts.items(), key=lambda kv: order.get(kv[0], 9))]
    if pre and pre.status != "PASS":
        L += ["", "> **Kein gueltiges Audit:** der Toolchain-Preflight ist nicht bestanden. "
              "Alle cargo-Schritte sind SKIPPED und zaehlen nicht als gruen."]
    L += ["", "Hinweis: Bei `befund_*`-Schritten bedeutet FAIL, dass der Befund **vorliegt**. "
          "`INFO` ist eine Messung oder ein Hinweis ohne Urteil.", "",
          "## Rote und uebersprungene Schritte", "", "| Schritt | Status | Sek | Detail | Log |", "|---|---|---:|---|---|"]
    for s in sorted(steps, key=lambda x: order.get(x.status, 9)):
        if s.status in ("FAIL", "TIMEOUT", "SKIPPED"):
            L.append(f"| `{s.name}` | {s.status} | {s.seconds} | {s.detail} | "
                     f"`{os.path.relpath(s.log, outdir) if s.log else ''}` |")
    if final:
        data = export_panic_surface(ctx, steps, outdir)
        if data:
            L += ["", "## Panic-Flaeche im Produktivcode (Clippy, lib, ohne Tests)", "",
                  "| Crate | unwrap | expect | panic | todo | index | Summe |", "|---|---:|---:|---:|---:|---:|---:|"]
            for crate, k in sorted(data.items(), key=lambda kv: -sum(kv[1].values()))[:25]:
                L.append(f"| `{crate}` | {k.get('unwrap', 0)} | {k.get('expect', 0)} | {k.get('panic', 0)} | "
                         f"{k.get('todo', 0)} | {k.get('index', 0)} | {sum(k.values())} |")
        ui = outdir / "unsafe_inventar.csv"
        if ui.exists():
            L += ["", "## unsafe-Inventar gegen capabilities.toml", ""]
            with open(ui) as fh:
                for r in csv.DictReader(fh):
                    if r["urteil"] != "ok" or int(r["unsafe_zeilen"]) > 0:
                        L.append(f"- `{r['crate']}`: {r['unsafe_zeilen']} Zeilen, island={r['unsafe_island']}, {r['urteil']}")
        nb = export_bench_csv(ctx.root, outdir, meta["t_start_epoch"])
        if nb:
            L += ["", f"## Benchmarks: {nb} Messwerte in `bench_results.csv`"]
        L += maturity_table(ctx, steps)
    L += ["", "## Alle Schritte", "", "| Gruppe | Schritt | Status | Sek | Detail |", "|---|---|---|---:|---|"]
    for s in steps:
        L.append(f"| {s.group} | `{s.name}` | {s.status} | {s.seconds} | {s.detail} |")
    (outdir / "summary.md").write_text("\n".join(L) + "\n")


def collect_env(root: Path, outdir: Path) -> dict:
    def sh(cmd: str) -> str:
        try:
            return subprocess.run(cmd, shell=True, cwd=root, capture_output=True, text=True, timeout=60).stdout.strip()
        except (subprocess.SubprocessError, OSError) as e:
            return f"n/a ({e})"
    root_user = hasattr(os, "geteuid") and os.geteuid() == 0
    meta = {
        "commit": sh("git rev-parse --short HEAD") or "unknown",
        "commit_date": sh("git log -1 --format=%ci"),
        "branch": sh("git rev-parse --abbrev-ref HEAD"),
        "behind": sh("git rev-list --count HEAD..@{u} 2>/dev/null") or "?",
        "dirty": bool(sh("git status --porcelain")),
        "host": f"{platform.platform()} | {os.cpu_count()} CPUs | " + sh("grep MemTotal /proc/meminfo 2>/dev/null"),
        "user": sh("id -un") + (" (ROOT: Dateirechte-Tests nicht aussagekraeftig!)" if root_user else ""),
    }
    meta["rustc"] = sh("rustc -V")
    (outdir / "env.txt").write_text("\n".join(f"{k}: {v}" for k, v in meta.items()) + "\n\n"
                                    + sh("rustup show active-toolchain 2>&1") + "\n\n" + sh("uname -a"))
    return meta


# ----------------------------------------------------------------------------------------------
# Vergleich zweier Laeufe
# ----------------------------------------------------------------------------------------------

def compare_runs(a: Path, b: Path) -> int:
    def load(d: Path):
        j = json.loads((d / "results.json").read_text())
        return j["meta"], {s["schritt"]: s for s in j["steps"]}
    ma, sa = load(a)
    mb, sb = load(b)
    out = ["# Vergleich", "", f"- A: `{ma['commit']}` ({a.name})", f"- B: `{mb['commit']}` ({b.name})", "",
           "## Statuswechsel", "", "| Schritt | A | B |", "|---|---|---|"]
    changed = 0
    for name in sorted(set(sa) | set(sb)):
        x, y = sa.get(name, {}).get("status", "-"), sb.get(name, {}).get("status", "-")
        if x != y:
            changed += 1
            out.append(f"| `{name}` | {x} | {y} |")
    if not changed:
        out.append("| (keine) | | |")

    def counts(s):
        m = re.search(r"passed=(\d+) failed=(\d+)", s.get("detail", ""))
        return (int(m.group(1)), int(m.group(2))) if m else None
    out += ["", "## Testzahlen (nur Aenderungen)", "", "| Schritt | A (ok/rot) | B (ok/rot) |", "|---|---|---|"]
    for name in sorted(set(sa) & set(sb)):
        if name.startswith(("test_lib__", "nextest__")):
            ca, cb = counts(sa[name]), counts(sb[name])
            if ca != cb:
                out.append(f"| `{name}` | {ca} | {cb} |")

    def bench(d: Path) -> dict:
        p = d / "bench_results.csv"
        if not p.exists():
            return {}
        with open(p) as fh:
            return {r["benchmark"]: float(r["median_ns"]) for r in csv.DictReader(fh)}
    ba, bb = bench(a), bench(b)
    if ba and bb:
        out += ["", "## Benchmarks (Median, > 5 % Abweichung)", "", "| Benchmark | A ns | B ns | Delta |", "|---|---:|---:|---:|"]
        for k in sorted(set(ba) & set(bb)):
            if ba[k] > 0:
                d = (bb[k] - ba[k]) / ba[k] * 100
                if abs(d) > 5:
                    out.append(f"| `{k}` | {ba[k]:.0f} | {bb[k]:.0f} | {d:+.1f} % |")
    text = "\n".join(out) + "\n"
    (b / "vergleich.md").write_text(text)
    print(text)
    return 0


# ----------------------------------------------------------------------------------------------
# main
# ----------------------------------------------------------------------------------------------

def main() -> int:
    ap = argparse.ArgumentParser(description="Contextra: testen, benchmarken, Befunde pruefen, Ergebnisse speichern")
    ap.add_argument("--quick", action="store_true", help="ohne Benchmarks, Fuzz, Mutation")
    ap.add_argument("--all", action="store_true", help="zusaetzlich Fuzz, Mutation, Miri, ASan, Loom, Coverage, Produkt-Probe")
    ap.add_argument("--produkt", action="store_true", help="nur Produktmenge + Tag-1-Pruefliste + SQLCipher-Probe")
    ap.add_argument("--probe", action="store_true", help="Tag-1-Pruefliste zusaetzlich ausfuehren")
    ap.add_argument("--only", help="kommagetrennte Crates (Kurzname), z. B. store,ingest")
    ap.add_argument("--gruppe", help="nur Gruppe, z. B. befunde, statisch, tests, bench, produkt, tief")
    ap.add_argument("--list", action="store_true", help="Plan anzeigen, nichts ausfuehren")
    ap.add_argument("--fail-fast", action="store_true")
    ap.add_argument("--jobs", type=int, help="CARGO_BUILD_JOBS begrenzen (z. B. bei wenig RAM)")
    ap.add_argument("--timeout-faktor", type=float, default=1.0, help="alle Zeitlimits multiplizieren")
    ap.add_argument("--resume", help="bestehendes evidence/audit-... Verzeichnis fortsetzen")
    ap.add_argument("--vergleich", nargs=2, metavar=("A", "B"), help="zwei Lauf-Verzeichnisse vergleichen")
    ap.add_argument("--root", default=".", help="Repo-Root (Standard: aktuelles Verzeichnis)")
    args = ap.parse_args()

    if args.vergleich:
        return compare_runs(Path(args.vergleich[0]), Path(args.vergleich[1]))

    root = Path(args.root).resolve()
    manifest = root / "Cargo.toml"
    if not manifest.exists() or "[workspace]" not in _read(manifest):
        print("Fehler: bitte im Workspace-Root ausfuehren (Cargo.toml mit [workspace]).", file=sys.stderr)
        return 2
    ctx = Ctx(root)
    if ctx.ws.problems:
        print("Workspace-Hinweise:", *ctx.ws.problems, sep="\n  ")

    start = now_utc()
    t_start = time.time()
    commit = subprocess.run("git rev-parse --short HEAD", shell=True, cwd=root,
                            capture_output=True, text=True).stdout.strip() or "nogit"
    if args.resume:
        outdir = Path(args.resume).resolve()
    else:
        outdir = root / "evidence" / f"audit-{start.strftime('%Y%m%dT%H%M%SZ')}-{commit}"
    previous: dict[str, dict] = {}
    if args.resume and (outdir / "results.json").exists():
        previous = {s["schritt"]: s for s in json.loads((outdir / "results.json").read_text())["steps"]}

    steps = build_plan(ctx, args, outdir)
    if args.list:
        for i, s in enumerate(steps):
            print(f"{i:03d} [{s.group}] {s.name:62s} {' '.join(s.cmd) if s.cmd else '(Python-Pruefung)'}")
        print(f"\n{len(steps)} Schritte")
        return 0

    (outdir / "logs").mkdir(parents=True, exist_ok=True)
    meta = collect_env(root, outdir)
    meta.update({"start": start.isoformat(timespec="seconds"), "t_start_epoch": t_start, "duration_min": 0.0})
    if meta["behind"] not in ("", "0", "?"):
        print(f"WARNUNG: HEAD liegt {meta['behind']} Commits hinter origin. Erst 'git pull', wenn du den neuesten Stand willst.")
    if meta["dirty"]:
        print("HINWEIS: Arbeitsbaum hat ungespeicherte Aenderungen; sie fliessen in die Ergebnisse ein.")
    if "ROOT" in meta["user"]:
        print("WARNUNG: Lauf als root. Dateirechte-Tests sind dann nicht aussagekraeftig.")

    print(f"{len(steps)} Schritte -> {outdir}\n")
    cargo_ok: Optional[bool] = None
    try:
        for i, s in enumerate(steps):
            prev = previous.get(s.name)
            if prev and prev["status"] in ("PASS", "INFO") and s.name != PREFLIGHT:
                s.status, s.seconds = prev["status"], prev["sekunden"]
                s.detail = (prev["detail"] + " (uebernommen)").strip()
                s.log = str(outdir / prev["log"]) if prev["log"] else ""
                print(f"[{i+1:03d}/{len(steps)}] {s.group:10s} {s.name} ... uebersprungen ({s.status}, aus frueherem Lauf)")
                continue
            print(f"[{i+1:03d}/{len(steps)}] {s.group:10s} {s.name} ...", end=" ", flush=True)
            run_step(s, i, outdir / "logs", ctx, outdir, args.timeout_faktor, args.jobs, cargo_ok)
            if s.name == PREFLIGHT:
                cargo_ok = s.status == "PASS"
            print(f"{s.status} ({s.seconds}s) {s.detail[:110]}")
            meta["duration_min"] = round((time.time() - t_start) / 60, 1)
            write_reports(ctx, steps, outdir, meta)  # Zwischenstand: nichts geht bei Abbruch verloren
            if args.fail_fast and s.status in ("FAIL", "TIMEOUT"):
                print("Abbruch wegen --fail-fast")
                break
    except KeyboardInterrupt:
        print("\nAbgebrochen. Zwischenergebnisse sind gespeichert. Fortsetzen mit --resume", outdir)

    crit = root / "target" / "criterion"
    if crit.exists() and any(s.group == "4-bench" for s in steps):
        shutil.copytree(crit, outdir / "criterion", dirs_exist_ok=True)
    meta["duration_min"] = round((time.time() - t_start) / 60, 1)
    write_reports(ctx, steps, outdir, meta, final=True)

    red = [s for s in steps if s.status in ("FAIL", "TIMEOUT")]
    print(f"\nFertig in {meta['duration_min']} min. FAIL/TIMEOUT: {len(red)}  -> {outdir}/summary.md")
    return 1 if red else 0


if __name__ == "__main__":
    sys.exit(main())
