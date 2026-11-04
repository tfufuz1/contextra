"""
contextra_full_audit.py (v7) - Tests, Testbarkeit, Lints, Benchmarks, Fuzz, Mutation, Coverage fuer den aktuellen Repo-Stand.

Der Fokus liegt auf der Testbarkeit der vorhandenen Crates und Funktionen. Es gibt keine Messung gegen
Spezifikationsdokumente mehr (keine Gate-/Rueckbau-/Produktlisten).

Aufruf im Repo-Root (dort, wo die Cargo.toml mit [workspace] liegt):

  python3 scripts/contextra_full_audit.py --list                 # Plan zeigen, nichts ausfuehren (kein cargo noetig)
  python3 scripts/contextra_full_audit.py --gruppe testbarkeit   # Testbarkeit (Python-Teil laeuft ohne Toolchain)
  python3 scripts/contextra_full_audit.py --gruppe befunde       # nur statische Befunde (kein Build)
  python3 scripts/contextra_full_audit.py --quick                # Preflight, check, fmt, clippy, tests, Testbarkeit
  python3 scripts/contextra_full_audit.py                        # Standard: alles inkl. Benchmarks
  python3 scripts/contextra_full_audit.py --coverage             # zusaetzlich cargo llvm-cov + Funktionsabdeckung
  python3 scripts/contextra_full_audit.py --all                  # zusaetzlich Fuzz, Mutation, Loom, Miri, ASan, Coverage
  python3 scripts/contextra_full_audit.py --only store,mvcc      # nur bestimmte Crates (Kurzname)
  python3 scripts/contextra_full_audit.py --resume evidence/audit-...   # PASS/INFO-Schritte werden uebernommen
  python3 scripts/contextra_full_audit.py --vergleich evidence/audit-A evidence/audit-B

Weitere Optionen: --fail-fast, --jobs N, --timeout-faktor F, --root PFAD

Gruppe 3-testbarkeit (Python-Schritte brauchen keinen Build):
  testbarkeit_crates             Tests je Crate (unit/integration/doctest), ignorierte und assertionlose Tests,
                                 Anteil der pub-Funktionen mit Testbezug. FAIL: Crate mit Code, aber ohne jeden Test.
  testbarkeit_funktionen         je Funktion Sichtbarkeit, Komplexitaet, Testnachweis, direkte Umweltzugriffe
                                 (Zeit, Zufall, Dateisystem, Umgebung, Prozess, Netz, sleep, globaler Zustand).
                                 FAIL: pub-Funktion ab Komplexitaet KOMPLEX_SCHWELLE ohne Testbezug.
  testbarkeit_verwaiste_dateien  Test-/Benchdateien, die kein Crate einbindet (laufen nie); fehlende [[test]]-Pfade.
  liste_tests__<crate>           `cargo test -- --list`: was cargo tatsaechlich als Test kennt; der Bericht stellt die
                                 statische Zaehlung daneben (Differenz = nie kompilierte oder Makro-Tests).
  coverage, coverage_funktionen  (--coverage/--all) exakte Funktionsabdeckung je Crate und Liste nie ausgefuehrter Funktionen.
Der Testnachweis ist eine Namensheuristik auf dem Quelltext (Kommentare/Strings maskiert), keine Typaufloesung;
generische Namen (new, get, ...) werden als "mehrdeutig" getrennt gefuehrt. Exakt ist nur die Coverage.

Weitere Schritte: Befunde (cfg(feature) ohne Definition, default-members, Reife-Behauptungen, Bench-/Fuzz-Dateien),
Toolchain-Preflight (schlaegt er fehl, werden alle cargo-Schritte SKIPPED), fmt/check/clippy/doc, deny/audit,
Panic-Flaeche, unsafe-Inventar, Tests (lib, integration, doc, nextest, Debug/Release), Benchmarks, Fuzz, Mutation,
Loom, Miri, ASan.

Ergebnisse: evidence/audit-<UTC>-<commit>/
  summary.md                      Zusammenfassung (rote Schritte zuerst), Toolchain, Testbarkeit, Reife-Behauptung vs. Beleg
  results.json/.csv               jeder Schritt mit Status, Dauer, Detail, Log
  testbarkeit_crates.csv          Kennzahlen je Crate
  testbarkeit_funktionen.csv      je Funktion: Komplexitaet, Nachweis, Hindernisse
  testbarkeit_auffaellige_tests.csv  ignorierte (mit/ohne Grund) und assertionlose Tests
  testbarkeit_verwaist.csv        Dateien, die nie ausgefuehrt werden
  testbarkeit_vergleich.csv       statische Testzahl gegen `cargo test --list`
  coverage_crates.csv, coverage_unbedeckt.csv, coverage.json   (nur mit --coverage/--all)
  panic_flaeche.csv               Panic-/Unwrap-Stellen im Produktivcode je Crate
  unsafe_inventar.csv             unsafe-Stellen je Crate gegen capabilities.toml
  bench_results.csv, criterion/   Criterion-Messwerte
  logs/, env.txt

Nur Python-Standardbibliothek (3.11+). Der Quellcode des Projekts wird nie veraendert.
Hinweis: Prozessgruppen (start_new_session) sind POSIX-spezifisch; das Skript ist fuer Linux/macOS gedacht.
"""
from __future__ import annotations

import argparse
import bisect
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
    "contextra-crypto": ["crypto_benchmarks"],
    "contextra-vector": [
        "distance_bench", "sq8_bench", "hnsw_bench", "hnsw_delete_search_bench", "audit_benchmarks",
        "flush_threshold_amplification", "partial_rebuild_recall_bench", "distance_nan_impact_bench",
        "filtered_search_selectivity_bench",
    ],
    "contextra-store": ["sstable_bench", "memtable_bench", "wal_bench", "lsm_concurrency_bench",
                        "block_cache_bench", "redb_comparative_bench"],
    "contextra-simd": ["simd_bench"],
}

FUZZ_CRATES = ["crypto", "store", "text", "vector"]


PREFLIGHT = "toolchain_preflight"
SLOW_CRATES = {"contextra-crypto": 3600, "contextra-store": 3600, "contextra-vector": 3600,
               "contextra-graph": 3600, "contextra-engine": 3600}
DEFAULT_TIMEOUT = 1800
BENCH_TIMEOUT = 3600
FUZZ_SECONDS = 120

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



# ----------------------------------------------------------------------------------------------
# Workspace-Modell (aus Cargo.toml, ohne cargo)
# ----------------------------------------------------------------------------------------------

class Workspace:
    def __init__(self, root: Path):
        self.root = root
        ws = _toml(root / "Cargo.toml").get("workspace", {})
        self.members: dict[str, Path] = {}     # Paketname -> absolutes Verzeichnis
        self.problems: list[str] = []
        self.fehlend: list[str] = []            # Mitglied in Cargo.toml, Verzeichnis fehlt
        self.fehlend_default: list[str] = []    # default-member ohne Verzeichnis
        for pat in ws.get("members", []):
            if "*" not in pat and not (root / pat).exists():
                self.fehlend.append(pat)
                continue
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
            elif not (root / pat).exists():
                self.fehlend_default.append(pat)
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
















def befund_maturity_claims(ctx: Ctx, outdir: Path):
    caps = ctx.caps
    stable = [k for k, v in caps.items() if v.get("maturity") == "stable"]
    beta = [k for k, v in caps.items() if v.get("maturity") == "beta"]
    fehlend = [k for k in caps if k not in ctx.ws.members]
    no_cap = [k for k in ctx.ws.members if k not in caps]
    text = f"stable: {stable}\nbeta: {beta}\nin capabilities.toml, aber kein Mitglied: {fehlend}\nMitglied ohne Eintrag: {no_cap}"
    detail = f"stable={len(stable)} beta={len(beta)} Eintraege ohne Crate={len(fehlend)}"
    return ("FAIL" if fehlend else "INFO"), detail, text






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










# ----------------------------------------------------------------------------------------------
# Plan
# ----------------------------------------------------------------------------------------------

def befund_default_members(ctx: Ctx, outdir: Path):
    """Was `cargo test` ohne --workspace prueft: Mitglieder ausserhalb von default-members werden nicht getestet."""
    ws = ctx.ws
    nicht_default = [c for c in ws.names() if c not in ws.default]
    detail = (f"members={len(ws.members)} default-members={len(ws.default)}; "
              f"nicht in default-members (von `cargo test` ohne --workspace nicht erfasst): {nicht_default or 'keine'}")
    bad = bool(ws.fehlend or ws.fehlend_default or ws.problems)
    extra = [f"Mitglieder ohne Verzeichnis ({len(ws.fehlend)}): {', '.join(ws.fehlend)}",
             f"default-members ohne Verzeichnis ({len(ws.fehlend_default)}): {', '.join(ws.fehlend_default)}"]
    return ("FAIL" if bad else "INFO"), detail, "\n".join(extra + ws.problems)


# ----------------------------------------------------------------------------------------------
# Testbarkeit (statisch): Tests je Crate, Testnachweis je Funktion, Hindernisse, verwaiste Testdateien
#
# Alles hier ist eine Heuristik auf dem Quelltext (kein Compiler). Kommentare und String-Inhalte
# werden maskiert, damit Klammern und Namen darin nichts verfaelschen. Der Nachweis "Funktion wird
# von einem Test aufgerufen" beruht auf Namensabgleich, nicht auf Typaufloesung; mehrdeutige Namen
# (z. B. `new`, `get`) werden getrennt ausgewiesen. Exakte Funktionsabdeckung liefert nur die
# Coverage (--coverage, Schritt coverage_funktionen).
# ----------------------------------------------------------------------------------------------

KOMPLEX_SCHWELLE = 25     # pub-Funktion ab dieser Komplexitaet ohne Testbezug => FAIL
GENERISCHE_NAMEN = {
    "new", "default", "get", "set", "len", "is_empty", "from", "into", "iter", "insert", "remove", "push", "pop",
    "clear", "open", "close", "build", "run", "start", "stop", "next", "read", "write", "flush", "sync", "apply",
    "load", "save", "name", "id", "key", "value", "size", "contains", "with", "finish", "drop", "clone", "init",
    "reset", "update", "delete", "put", "take", "peek", "add", "sub", "min", "max", "map", "call", "send", "recv",
}
HINDERNISSE = [   # (Kennung, Muster): direkte Zugriffe auf Umwelt ohne erkennbare Abstraktion
    ("zeit", re.compile(r"\b(?:SystemTime|Instant|Utc|Local)::now\s*\(|\bchrono::\w+::now")),
    ("zufall", re.compile(r"\bthread_rng\s*\(|\brand::random\b|\bOsRng\b|\bfrom_entropy\b|\bgetrandom\b")),
    ("dateisystem", re.compile(r"\bstd::fs::|\bfs::(?:read|write|File|create_dir|remove|rename|metadata|copy)\w*"
                               r"|\bFile::(?:open|create)\b|\bOpenOptions::new\b|\btokio::fs::")),
    ("umgebung", re.compile(r"\benv::(?:var|vars|args|current_dir|set_var|remove_var)\b")),
    ("prozess", re.compile(r"\bCommand::new\b|\bprocess::(?:exit|abort)\b")),
    ("netz", re.compile(r"\b(?:TcpStream|TcpListener|UdpSocket)::\w+")),
    ("schlafen", re.compile(r"\b(?:thread|tokio::time)::sleep\b")),
    ("globaler_zustand", re.compile(r"\bstatic\s+mut\b|\bOnceLock\b|\bOnceCell\b|\blazy_static!|\bthread_local!"
                                    r"|\bstatic\s+\w+\s*:\s*(?:Mutex|RwLock|Atomic)")),
]

_TOK = re.compile(r'//|/\*|(?<!\w)b?r(#*)"|"|\'')
_CHAR = re.compile(r"'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^\\'\n])'")
_BRACE = re.compile(r"[{}]")
_CFG_TEST = re.compile(r"#\[cfg\(\s*(?:all\(\s*)?test\b")
_TEST_ATTR = re.compile(r"#\[\s*(?:[A-Za-z_]\w*::)*(?:test|rstest|test_case|ctx_test)\b[^\]]*\]")
_IGNORE_ATTR = re.compile(r"#\[\s*ignore\b")
_IGNORE_GRUND = re.compile(r"#\[\s*ignore\s*=")
_SHOULD_PANIC = re.compile(r"#\[\s*should_panic\b")
_FN_NAME = re.compile(r"\bfn\s+(\w+)")
_FN_DEF = re.compile(r'(?m)^[ \t]*(pub(?:\([^)]*\))?[ \t]+)?(?:(?:const|async|unsafe|default)[ \t]+)*'
                     r'(?:extern[ \t]+(?:"[^"]*"[ \t]+)?)?fn[ \t]+(\w+)')
_IMPL_FOR = re.compile(r"(?m)^[ \t]*(?:unsafe[ \t]+)?impl\b[^{};]*?\bfor\b[^{};]*\{")
_TRAIT_DEF = re.compile(r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?(?:unsafe[ \t]+)?trait\b[^{};]*\{")
_COMPLEX = re.compile(r"\b(?:if|while|for|loop)\b|=>|&&|\|\||\?(?!\w)")
_ASSERT = re.compile(r"\bassert\w*\s*!|\bdebug_assert\w*\s*!|\bpanic\s*!|\bunreachable\s*!|\bprop_assert\w*\s*!"
                     r"|\bmatches\s*!|\.unwrap(?:_err)?\s*\(|\.expect(?:_err)?\s*\(|\bassert\w*\s*\(|\?(?!\w)")
_T_CALL = re.compile(r"(\w+)\s*(?:\(|::<)")
_T_PATH = re.compile(r"::(\w+)")
_T_METH = re.compile(r"\.(\w+)")
_RUST_FENCE = {"", "rust", "no_run", "should_panic", "ignore", "compile_fail", "edition2018", "edition2021",
               "edition2024"}


def _mask(text: str) -> str:
    """Kommentare und Inhalt von String-/Zeichenliteralen durch Leerzeichen ersetzen (Laenge und Zeilen bleiben)."""
    out = list(text)
    n, i = len(text), 0

    def blank(a: int, b: int) -> None:
        for k in range(a, min(b, n)):
            if out[k] != "\n":
                out[k] = " "

    while True:
        m = _TOK.search(text, i)
        if not m:
            break
        s, a = m.group(0), m.start()
        if s == "//":
            j = text.find("\n", a)
            j = n if j < 0 else j
            blank(a, j)
            i = j
        elif s == "/*":
            depth, j = 1, a + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif text.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            blank(a, j)
            i = j
        elif s == '"':
            j = a + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            blank(a + 1, j)
            i = j + 1
        elif s == "'":
            c = _CHAR.match(text, a)
            if c:
                blank(a + 1, c.end() - 1)
                i = c.end()
            else:      # Lifetime
                i = a + 1
        else:          # Raw-String r#"..."#
            close = '"' + m.group(1)
            e = text.find(close, m.end())
            e = n if e < 0 else e
            blank(m.end(), e)
            i = min(n, e + len(close))
    return "".join(out)


def _brace_end(m: str, start: int) -> int:
    """Index hinter der schliessenden Klammer zu m[start] == '{' (len(m), wenn unbalanciert)."""
    depth = 0
    for b in _BRACE.finditer(m, start):
        depth += 1 if b.group(0) == "{" else -1
        if depth == 0:
            return b.end()
    return len(m)


def _fn_span(m: str, pos: int) -> Optional[tuple[int, int]]:
    """(Index von '{', Ende des Rumpfs) ab Funktionsname; None bei Deklaration ohne Rumpf."""
    depth, k, n = 0, pos, len(m)
    while k < n:
        c = m[k]
        if c in "([":
            depth += 1
        elif c in ")]":
            depth -= 1
        elif depth <= 0 and c == ";":
            return None
        elif depth <= 0 and c == "{":
            return k, _brace_end(m, k)
        k += 1
    return None


def _skip_attrs(m: str, pos: int) -> int:
    n = len(m)
    while True:
        while pos < n and m[pos].isspace():
            pos += 1
        if not m.startswith("#[", pos):
            return pos
        depth, k = 0, pos + 1
        while k < n:
            if m[k] == "[":
                depth += 1
            elif m[k] == "]":
                depth -= 1
                if depth == 0:
                    break
            k += 1
        pos = k + 1


def _item_end(m: str, pos: int) -> int:
    """Ende des Items, das hinter einem Attribut ab `pos` beginnt."""
    pos = _skip_attrs(m, pos)
    head = m[pos:pos + 24]
    if re.match(r"(?:pub(?:\([^)]*\))?\s+)?(?:use|static|type|extern\s+crate)\b|(?:pub\s+)?const\s+(?!fn|unsafe)", head):
        d, k, n = 0, pos, len(m)
        while k < n:
            c = m[k]
            if c in "([{":
                d += 1
            elif c in ")]}":
                d -= 1
            elif c == ";" and d <= 0:
                return k + 1
            k += 1
        return n
    depth, k, n = 0, pos, len(m)
    while k < n:
        c = m[k]
        if c in "([":
            depth += 1
        elif c in ")]":
            depth -= 1
        elif depth <= 0 and c == ";":
            return k + 1
        elif depth <= 0 and c == "{":
            return _brace_end(m, k)
        k += 1
    return n


def _test_ranges(m: str) -> list[tuple[int, int]]:
    res, i = [], 0
    while True:
        x = _CFG_TEST.search(m, i)
        if not x:
            return res
        e = _item_end(m, m.find("]", x.start()) + 1)
        res.append((x.start(), e))
        i = max(e, x.end())


def _blank_ranges(m: str, ranges: list[tuple[int, int]]) -> str:
    if not ranges:
        return m
    parts, last = [], 0
    for a, b in ranges:
        parts.append(m[last:a])
        parts.append(re.sub(r"[^\n]", " ", m[a:b]))
        last = b
    parts.append(m[last:])
    return "".join(parts)


def _ranges_of(m: str, rx: re.Pattern) -> list[tuple[int, int]]:
    out = []
    for x in rx.finditer(m):
        out.append((x.start(), _brace_end(m, x.end() - 1)))
    return out


def _is_test_path(p: Path) -> bool:
    return "tests" in p.parts or p.name.startswith("tests") or p.name.endswith(("_tests.rs", "_test.rs"))


def _tokens(masked_test_text: str) -> set[str]:
    t = _FN_NAME.sub("fn", masked_test_text)
    toks = set(_T_CALL.findall(t))
    toks.update(_T_PATH.findall(t))
    toks.update(_T_METH.findall(t))
    return toks


def _doc_tests(orig: str) -> tuple[int, int, str]:
    """(Doctests, davon ignoriert, Code aller Doctest-Bloecke)."""
    n = ign = 0
    code: list[str] = []
    in_code = False
    for ln in orig.splitlines():
        m = re.match(r"\s*(?://[/!])\s?(.*)$", ln)
        if not m:
            in_code = False
            continue
        body = m.group(1)
        if body.strip().startswith("```"):
            if not in_code:
                info = {t.strip().lower() for t in body.strip()[3:].split(",") if t.strip()}
                if info <= _RUST_FENCE:
                    n += 1
                    ign += 1 if "ignore" in info else 0
                    in_code = True
                else:
                    in_code = None   # fremder Block (text, toml, ...): nicht zaehlen
            else:
                in_code = False
            continue
        if in_code:
            code.append(re.sub(r"^#\s", "", body))
    return n, ign, "\n".join(code)


class CrateScan:
    def __init__(self, name: str, d: Path):
        self.name, self.dir = name, d
        ct = _toml(d / "Cargo.toml")
        self.proc_macro = bool((ct.get("lib") or {}).get("proc-macro"))
        self.has_lib = has_lib(d)
        self.has_bin = (d / "src" / "main.rs").exists() or bool(ct.get("bin"))
        self.fns: list[dict] = []
        self.tests: list[dict] = []
        self.doctests = self.doctests_ignoriert = self.proptests = self.prod_zeilen = 0
        self.tok_unit: set[str] = set()
        self.tok_integ: set[str] = set()
        self.tok_doc: set[str] = set()
        self.integ_dateien: dict[str, int] = {}
        fz = d / "fuzz" / "fuzz_targets"
        self.fuzz_targets = len(list(fz.glob("*.rs"))) if fz.is_dir() else 0
        bd = d / "benches"
        self.benches = len(list(bd.glob("*.rs"))) if bd.is_dir() else 0


def _extract_tests(orig: str, masked: str, nl: list[int], rel: str, art: str) -> list[dict]:
    res, seen = [], set()
    for m in _TEST_ATTR.finditer(masked):
        fm = _FN_NAME.search(masked, m.end())
        if not fm or fm.start() in seen:
            continue
        between = masked[m.end():fm.start()]
        if "{" in between or "}" in between:
            continue
        seen.add(fm.start())
        span = _fn_span(masked, fm.end())
        body = masked[span[0]:span[1]] if span else ""
        lo = max(0, m.start() - 300)
        w = masked[lo:fm.start()]
        cut = max(w.rfind("}"), w.rfind(";")) + 1
        w = w[cut:]
        grund = bool(_IGNORE_GRUND.search(orig[lo + cut:fm.start()]))
        res.append(dict(datei=rel, zeile=bisect.bisect_left(nl, fm.start()) + 1, name=fm.group(1), art=art,
                        ignoriert=bool(_IGNORE_ATTR.search(w)), ignoriert_ohne_grund=bool(_IGNORE_ATTR.search(w)) and not grund,
                        ohne_assertion=not (_SHOULD_PANIC.search(w) or _ASSERT.search(body))))
    return res


def _scan_crate(root: Path, name: str, d: Path, extra_tests: list[Path]) -> CrateScan:
    sc = CrateScan(name, d)
    src = d / "src"
    for p in sorted(src.rglob("*.rs")) if src.is_dir() else []:
        if "target" in p.parts:
            continue
        orig = _read(p)
        masked = _mask(orig)
        nl = [x.start() for x in re.finditer("\n", masked)]
        rel = str(p.relative_to(root))
        sc.tests += _extract_tests(orig, masked, nl, rel, "unit")
        sc.proptests += len(re.findall(r"\bproptest\s*!", masked))
        if _is_test_path(p.relative_to(d)):
            sc.tok_unit |= _tokens(masked)
            continue
        ranges = _test_ranges(masked)
        prodm = _blank_ranges(masked, ranges)
        sc.tok_unit |= _tokens("".join(masked[a:b] for a, b in ranges))
        sc.prod_zeilen += sum(1 for ln in prodm.splitlines() if ln.strip())
        n_doc, n_ign, code = _doc_tests(orig)
        sc.doctests += n_doc
        sc.doctests_ignoriert += n_ign
        if code:
            sc.tok_doc |= _tokens(_mask(code))
        trait_like = _ranges_of(prodm, _IMPL_FOR) + _ranges_of(prodm, _TRAIT_DEF)
        for f in _FN_DEF.finditer(prodm):
            span = _fn_span(prodm, f.end())
            if not span:
                continue
            body = prodm[span[0]:span[1]]
            vis_raw = (f.group(1) or "").strip()
            vis = "pub" if vis_raw == "pub" else ("crate" if vis_raw else "privat")
            im_trait = any(a <= f.start() < b for a, b in trait_like)
            hind = [k for k, rx in HINDERNISSE if rx.search(body)]
            sc.fns.append(dict(datei=rel, zeile=bisect.bisect_left(nl, f.start()) + 1, name=f.group(2), vis=vis,
                               art="trait" if im_trait else "frei", zeilen=body.count("\n") + 1,
                               komplexitaet=1 + len(_COMPLEX.findall(body)), hindernisse=hind))
    seen_files: set[Path] = set()
    tdir = d / "tests"
    files = (sorted(tdir.rglob("*.rs")) if tdir.is_dir() else []) + [p for p in extra_tests if p.exists()]
    for p in files:
        rp = p.resolve()
        if rp in seen_files:
            continue
        if tdir.is_dir() and tdir in p.parents and "ui" in p.relative_to(tdir).parts[:-1]:
            continue   # trybuild-Fixtures (tests/ui/*.rs) sind Compile-Eingaben, keine Tests
        seen_files.add(rp)
        orig = _read(p)
        masked = _mask(orig)
        nl = [x.start() for x in re.finditer("\n", masked)]
        try:
            rel = str(p.relative_to(root))
        except ValueError:
            rel = str(p)
        t = _extract_tests(orig, masked, nl, rel, "integration")
        sc.tests += t
        sc.integ_dateien[rel] = len(t)
        sc.proptests += len(re.findall(r"\bproptest\s*!", masked))
        sc.tok_integ |= _tokens(masked)
    return sc


def _declared(ctx: Ctx) -> tuple[dict[str, list[Path]], dict[str, list[Path]], list[str]]:
    """[[test]]- und [[bench]]-Pfade je Crate (aufgeloest), plus Fehlerliste fuer fehlende Dateien."""
    tests: dict[str, list[Path]] = {}
    benches: dict[str, list[Path]] = {}
    problems: list[str] = []
    for name, d in ctx.ws.members.items():
        ct = _toml(d / "Cargo.toml")
        for kind, store, default in (("test", tests, "tests"), ("bench", benches, "benches")):
            for e in ct.get(kind, []) or []:
                path = e.get("path") or f"{default}/{e.get('name')}.rs"
                rp = (d / path).resolve()
                store.setdefault(name, []).append(rp)
                if not rp.exists():
                    problems.append(f"{name}: [[{kind}]] `{e.get('name')}` verweist auf fehlende Datei {path}")
    return tests, benches, problems


def get_scan(ctx: Ctx) -> dict[str, CrateScan]:
    if getattr(ctx, "_scan", None) is None:
        decl_tests, _, _ = _declared(ctx)
        ctx._scan = {name: _scan_crate(ctx.root, name, d, decl_tests.get(name, []))
                     for name, d in sorted(ctx.ws.members.items())}
    return ctx._scan


def _selected(ctx: Ctx) -> list[str]:
    sel = getattr(ctx, "selected", None)
    return list(sel) if sel else ctx.ws.names()


def _nachweis(sc: CrateScan, fn: dict, scans: dict[str, CrateScan]) -> str:
    """unit | integration | doctest | andere_crates | keiner (+ Zusatz ' (mehrdeutig)')."""
    nm = fn["name"]
    if fn["art"] == "trait":
        return "trait_impl"
    hit = ""
    if nm in sc.tok_unit:
        hit = "unit"
    elif fn["vis"] != "privat":
        if nm in sc.tok_integ:
            hit = "integration"
        elif nm in sc.tok_doc:
            hit = "doctest"
        elif any(nm in o.tok_unit or nm in o.tok_integ for k, o in scans.items() if k != sc.name):
            hit = "andere_crates"
    if not hit:
        return "keiner"
    if nm in GENERISCHE_NAMEN or len(nm) <= 3:
        return hit + " (mehrdeutig)"
    return hit


def _fn_rows(ctx: Ctx) -> dict[str, list[dict]]:
    scans = get_scan(ctx)
    rows: dict[str, list[dict]] = {}
    for c in _selected(ctx):
        sc = scans[c]
        rows[c] = [dict(fn, crate=c, nachweis=_nachweis(sc, fn, scans)) for fn in sc.fns]
    return rows


def _crate_summary(ctx: Ctx) -> list[dict]:
    scans = get_scan(ctx)
    fr = _fn_rows(ctx)
    out = []
    for c in _selected(ctx):
        sc = scans[c]
        unit = sum(1 for t in sc.tests if t["art"] == "unit")
        integ = sum(1 for t in sc.tests if t["art"] == "integration")
        pubs = [f for f in fr[c] if f["vis"] == "pub" and f["art"] == "frei"]
        belegt = [f for f in pubs if f["nachweis"] not in ("keiner",) and "mehrdeutig" not in f["nachweis"]]
        mehrd = [f for f in pubs if "mehrdeutig" in f["nachweis"]]
        quote = round(len(belegt) / len(pubs), 2) if pubs else None
        if sc.proc_macro:
            urteil = "ausnahme_proc_macro" if unit + integ == 0 else "ok"
        elif unit + integ == 0 and sc.doctests == 0 and fr[c]:
            urteil = "kein_test"
        elif unit + integ == 0:
            urteil = "nur_doctests"
        elif quote is not None and len(pubs) >= 5 and quote < 0.5:
            urteil = "schwach"
        else:
            urteil = "ok"
        out.append(dict(
            crate=c, lib=sc.has_lib, prod_zeilen=sc.prod_zeilen, tests_unit=unit, tests_integration=integ,
            tests_statisch=unit + integ, integrationsdateien=len(sc.integ_dateien), doctests=sc.doctests,
            ignoriert=sum(1 for t in sc.tests if t["ignoriert"]),
            ignoriert_ohne_grund=sum(1 for t in sc.tests if t["ignoriert_ohne_grund"]),
            proptest_bloecke=sc.proptests, fuzz_targets=sc.fuzz_targets, benches=sc.benches,
            tests_ohne_assertion=sum(1 for t in sc.tests if t["ohne_assertion"]),
            fns=len(fr[c]), pub_fns=len(pubs), pub_mit_nachweis=len(belegt), pub_mehrdeutig=len(mehrd),
            pub_ohne_nachweis=len(pubs) - len(belegt) - len(mehrd), nachweis_quote=quote if quote is not None else "",
            urteil=urteil))
    return out


def _write_csv(path: Path, rows: list[dict]) -> None:
    if not rows:
        return
    with open(path, "w", newline="") as fh:
        w = csv.DictWriter(fh, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)


def _md_table(rows: list[dict], cols: list[str]) -> list[str]:
    L = ["| " + " | ".join(cols) + " |", "|" + "|".join("---" for _ in cols) + "|"]
    for r in rows:
        L.append("| " + " | ".join(str(r.get(c, "")) for c in cols) + " |")
    return L


def testbarkeit_crates(ctx: Ctx, outdir: Path):
    """Tests je Crate: Anzahl, Art, ignorierte und assertionlose Tests, Nachweisquote der pub-Funktionen."""
    rows = _crate_summary(ctx)
    _write_csv(outdir / "testbarkeit_crates.csv", rows)
    scans = get_scan(ctx)
    detail_rows = []
    for c in _selected(ctx):
        for t in scans[c].tests:
            if t["ignoriert"] or t["ohne_assertion"]:
                detail_rows.append(dict(crate=c, **t))
    _write_csv(outdir / "testbarkeit_auffaellige_tests.csv", detail_rows)
    kein = [r["crate"] for r in rows if r["urteil"] == "kein_test"]
    schwach = [r["crate"] for r in rows if r["urteil"] == "schwach"]
    text = "\n".join(_md_table(rows, ["crate", "tests_unit", "tests_integration", "doctests", "ignoriert",
                                      "tests_ohne_assertion", "pub_fns", "pub_mit_nachweis", "pub_mehrdeutig",
                                      "pub_ohne_nachweis", "nachweis_quote", "urteil"]))
    text += ("\n\nHeuristik: Nachweis = Funktionsname wird in Test-Code (unit, integration, doctest) aufgerufen. "
             "`tests_ohne_assertion` = Test ohne assert/unwrap/expect/?/should_panic (kann nie fehlschlagen oder "
             "prueft ueber Hilfsfunktionen).\nDetails: testbarkeit_auffaellige_tests.csv")
    gesamt = sum(r["tests_statisch"] for r in rows)
    detail = f"{len(rows)} Crates, {gesamt} Tests (statisch); ohne Tests: {kein or 'keine'}; schwach: {schwach or 'keine'}"
    return ("FAIL" if kein else "PASS"), detail, text


def testbarkeit_funktionen(ctx: Ctx, outdir: Path):
    """Je Funktion: Sichtbarkeit, Komplexitaet, Testnachweis, Hindernisse (direkte Umweltzugriffe)."""
    fr = _fn_rows(ctx)
    allrows = [r for c in fr for r in fr[c]]
    out = [dict(crate=r["crate"], datei=r["datei"], zeile=r["zeile"], funktion=r["name"], sichtbarkeit=r["vis"],
                art=r["art"], zeilen=r["zeilen"], komplexitaet=r["komplexitaet"], nachweis=r["nachweis"],
                hindernisse=",".join(r["hindernisse"])) for r in allrows]
    _write_csv(outdir / "testbarkeit_funktionen.csv", out)
    kritisch = sorted((r for r in allrows if r["vis"] == "pub" and r["art"] == "frei" and r["nachweis"] == "keiner"
                       and r["komplexitaet"] >= KOMPLEX_SCHWELLE), key=lambda r: -r["komplexitaet"])
    hind_cnt: dict[str, dict[str, int]] = defaultdict(lambda: defaultdict(int))
    for r in allrows:
        for h in r["hindernisse"]:
            hind_cnt[r["crate"]][h] += 1
    L = [f"## pub-Funktionen ohne Testbezug mit Komplexitaet >= {KOMPLEX_SCHWELLE} ({len(kritisch)})", ""]
    L += [f"- `{r['crate']}` {r['datei']}:{r['zeile']} `{r['name']}` komplexitaet={r['komplexitaet']} "
          f"zeilen={r['zeilen']}" + (f" hindernisse={','.join(r['hindernisse'])}" if r["hindernisse"] else "")
          for r in kritisch[:60]] or ["keine"]
    L += ["", "## Direkte Umweltzugriffe im Produktivcode (schwer deterministisch testbar), Funktionen je Crate", ""]
    kinds = [k for k, _ in HINDERNISSE]
    L += _md_table([dict(crate=c, **{k: v.get(k, 0) for k in kinds}) for c, v in sorted(hind_cnt.items())],
                   ["crate"] + kinds) if hind_cnt else ["keine"]
    big = sorted((r for r in allrows if r["art"] == "frei"), key=lambda r: -r["komplexitaet"])[:20]
    L += ["", "## Komplexeste Funktionen (Top 20)", ""]
    L += [f"- `{r['crate']}` {r['datei']}:{r['zeile']} `{r['name']}` komplexitaet={r['komplexitaet']} "
          f"nachweis={r['nachweis']}" for r in big]
    nh = sum(1 for r in allrows if r["hindernisse"])
    detail = (f"{len(allrows)} Funktionen, {nh} mit direktem Umweltzugriff; "
              f"{len(kritisch)} komplexe pub-Funktionen ohne Testbezug")
    return ("FAIL" if kritisch else "PASS"), detail, "\n".join(L)


def testbarkeit_verwaiste(ctx: Ctx, outdir: Path):
    """Test-/Benchdateien, die kein Crate einbindet (laufen nie), und leere Testdateien."""
    root, members = ctx.root, ctx.ws.members
    decl_t, decl_b, problems = _declared(ctx)
    declared_t = {p for v in decl_t.values() for p in v}
    declared_b = {p for v in decl_b.values() for p in v}
    member_dirs = {d.resolve(): n for n, d in members.items()}
    skip = {"target", ".git", "evidence", "node_modules", "fuzz", ".cargo"}
    rows: list[dict] = []
    info: list[str] = []

    def owner(p: Path) -> Optional[str]:
        best = None
        for d, n in member_dirs.items():
            if d == p or d in p.parents:
                if best is None or len(str(d)) > len(str(best[0])):
                    best = (d, n)
        return best[1] if best else None

    def top_level(dirp: Path) -> list[Path]:
        res = sorted(dirp.glob("*.rs"))
        for sub in sorted(x for x in dirp.iterdir() if x.is_dir()):
            if (sub / "main.rs").exists() and sub.resolve() not in member_dirs:
                res.append(sub / "main.rs")
        return res

    for dirpath, dirnames, _ in os.walk(root):
        dirnames[:] = [x for x in dirnames if x not in skip and not x.startswith(".")]
        base = Path(dirpath)
        if base.name not in ("tests", "benches"):
            continue
        kind = "test" if base.name == "tests" else "bench"
        own = owner(base.parent.resolve())
        declared = declared_t if kind == "test" else declared_b
        for f in top_level(base):
            rp = f.resolve()
            if rp in declared:
                continue
            if own is None:
                grund = f"gehoert zu keinem Crate (Verzeichnis {base.relative_to(root)} ist kein Package) und ist nirgends per path deklariert"
            else:
                pkg = (_toml(members[own] / "Cargo.toml").get("package") or {})
                if pkg.get("autotests" if kind == "test" else "autobenches", True) is not False:
                    continue
                grund = f"{own}: Auto-Erkennung abgeschaltet, Datei nicht als [[{kind}]] deklariert"
            orig = _read(f)
            masked = _mask(orig)
            n = len(_extract_tests(orig, masked, [x.start() for x in re.finditer("\n", masked)], "", "x"))
            rows.append(dict(datei=str(f.relative_to(root)), art=kind, tests=n if kind == "test" else "", grund=grund))

    scans = get_scan(ctx)
    for c in _selected(ctx):
        d = members[c]
        for rel, n in scans[c].integ_dateien.items():
            p = root / rel
            if n == 0 and p.parent == d / "tests" and "harness = false" not in _read(d / "Cargo.toml") \
                    and not re.search(r"\bfn\s+main\b", _mask(_read(p))):
                info.append(f"{c}: {rel} enthaelt 0 Tests")
        for e in (_toml(d / "Cargo.toml").get("test") or []):
            if e.get("required-features"):
                info.append(f"{c}: [[test]] `{e.get('name')}` laeuft nur mit Features {e['required-features']}")
    _write_csv(outdir / "testbarkeit_verwaist.csv", rows)
    nt = sum(r["tests"] for r in rows if r["art"] == "test" and isinstance(r["tests"], int))
    text = "## Dateien, die nie ausgefuehrt werden\n\n"
    text += "\n".join(f"- `{r['datei']}` ({r['art']}, Tests: {r['tests'] or '-'}): {r['grund']}" for r in rows) or "keine"
    text += "\n\n## Fehlende deklarierte Dateien\n\n" + ("\n".join(f"- {p}" for p in problems) or "keine")
    text += "\n\n## Hinweise (leer / Feature-abhaengig)\n\n" + ("\n".join(f"- {x}" for x in info) or "keine")
    detail = f"{len(rows)} verwaiste Dateien ({nt} Tests laufen nie), {len(problems)} fehlende deklarierte Dateien"
    return ("FAIL" if rows or problems else "PASS"), detail, text


def _demangle(n: str) -> str:
    if not n.startswith("_ZN"):
        return re.sub(r"::h[0-9a-f]{16}$", "", n)
    parts, i = [], 3
    while i < len(n) and n[i].isdigit():
        j = i
        while j < len(n) and n[j].isdigit():
            j += 1
        ln = int(n[i:j])
        parts.append(n[j:j + ln])
        i = j + ln
    parts = [p for p in parts if not re.fullmatch(r"h[0-9a-f]{16}", p)]
    s = "::".join(parts)
    for a, b in (("$LT$", "<"), ("$GT$", ">"), ("$u20$", " "), ("$C$", ","), ("$RF$", "&"), ("$LP$", "("),
                 ("$RP$", ")"), ("$BP$", "*"), ("$SP$", "@"), ("..", "::")):
        s = s.replace(a, b)
    return re.sub(r"\$u([0-9a-f]{2})\$", lambda m: chr(int(m.group(1), 16)), s)


def coverage_funktionen(ctx: Ctx, outdir: Path):
    """Exakte Funktionsabdeckung aus coverage.json (cargo llvm-cov): je Crate und nie ausgefuehrte Funktionen."""
    cj = outdir / "coverage.json"
    if not cj.exists():
        return "SKIPPED", "coverage.json fehlt (Schritt `coverage` nicht gelaufen: --coverage oder --all)", ""
    data = json.loads(cj.read_text())["data"][0]
    dmap = ctx.ws.dir_map()
    per: dict[str, dict[str, float]] = defaultdict(lambda: defaultdict(float))
    for f in data.get("files", []):
        rel = os.path.relpath(f["filename"], ctx.root)
        if "/src/" not in "/" + rel or _is_test_path(Path(rel)):
            continue
        c = crate_for_path(rel, dmap)
        fs, ls = f["summary"]["functions"], f["summary"]["lines"]
        per[c]["fn_gesamt"] += fs["count"]
        per[c]["fn_belegt"] += fs["covered"]
        per[c]["zeilen_gesamt"] += ls["count"]
        per[c]["zeilen_belegt"] += ls["covered"]
    groups: dict[tuple[str, int], dict] = {}
    for fn in data.get("functions", []):
        fname = fn["filenames"][0] if fn.get("filenames") else ""
        rel = os.path.relpath(fname, ctx.root) if fname else ""
        if "/src/" not in "/" + rel or _is_test_path(Path(rel)):
            continue
        line = fn["regions"][0][0] if fn.get("regions") else 0
        g = groups.setdefault((rel, line), dict(datei=rel, zeile=line, funktion=_demangle(fn["name"]), aufrufe=0))
        g["aufrufe"] += fn.get("count", 0)
    unb = sorted((dict(crate=crate_for_path(g["datei"], dmap), **g) for g in groups.values() if g["aufrufe"] == 0),
                 key=lambda r: (r["crate"], r["datei"], r["zeile"]))
    _write_csv(outdir / "coverage_unbedeckt.csv", unb)
    rows = []
    for c, v in sorted(per.items()):
        rows.append(dict(crate=c, funktionen=int(v["fn_gesamt"]), funktionen_belegt=int(v["fn_belegt"]),
                         funktionen_prozent=round(100 * v["fn_belegt"] / v["fn_gesamt"], 1) if v["fn_gesamt"] else "",
                         zeilen_prozent=round(100 * v["zeilen_belegt"] / v["zeilen_gesamt"], 1) if v["zeilen_gesamt"] else ""))
    _write_csv(outdir / "coverage_crates.csv", rows)
    tf = sum(r["funktionen"] for r in rows)
    tb = sum(r["funktionen_belegt"] for r in rows)
    text = "\n".join(_md_table(rows, ["crate", "funktionen", "funktionen_belegt", "funktionen_prozent", "zeilen_prozent"]))
    text += f"\n\nNie ausgefuehrte Funktionen: {len(unb)} (coverage_unbedeckt.csv)"
    return "PASS", (f"{tb}/{tf} Funktionen ausgefuehrt ({round(100 * tb / tf, 1) if tf else 0} %), "
                    f"{len(unb)} nie ausgefuehrt"), text


def export_testliste(ctx: Ctx, steps: list["Step"], outdir: Path) -> list[dict]:
    """Vergleicht statisch gezaehlte Tests mit dem, was `cargo test -- --list` tatsaechlich kennt."""
    csvp = outdir / "testbarkeit_crates.csv"
    if not csvp.exists():
        return []
    with open(csvp) as fh:
        stat = {r["crate"]: int(r["tests_statisch"]) for r in csv.DictReader(fh)}
    rows = []
    for s in steps:
        if not s.name.startswith("liste_tests__") or s.status not in ("PASS", "INFO") or not s.log:
            continue
        c = s.name.split("__", 1)[1]
        text = Path(s.log).read_text(errors="replace")
        gelistet = len(re.findall(r"^\S.*: test$", text, re.M))
        if c in stat:
            rows.append(dict(crate=c, statisch=stat[c], gelistet=gelistet, differenz=stat[c] - gelistet))
    _write_csv(outdir / "testbarkeit_vergleich.csv", rows)
    return rows


def nextest_ok() -> bool:
    return which("cargo-nextest") is not None


def build_plan(ctx: Ctx, args, outdir: Path) -> list[Step]:
    ws = ctx.ws
    crates = ws.names()
    if args.only:
        wanted = {short(w.strip()) for w in args.only.split(",") if w.strip()}
        unknown = sorted(wanted - {short(c) for c in ws.names()})
        if unknown:
            raise SystemExit(f"--only: unbekannte Crates {unknown} (Kurzname ohne 'contextra-')")
        crates = [c for c in crates if short(c) in wanted]
    if not crates:
        raise SystemExit("Keine Crates ausgewaehlt (--only pruefen).")

    sel_all = not args.only
    libs = [c for c in crates if has_lib(ws.members[c])]
    pk = ["--workspace"] if sel_all else [x for c in crates for x in ("-p", c)]
    pk_lib = ["--workspace"] if sel_all else [x for c in libs for x in ("-p", c)]
    cargo = ["cargo"]
    S: list[Step] = []

    # --- B: Befund-Pruefung (kein Build) -----------------------------------------------------
    S += [
        Step("B-befunde", "befund_cfg_features_undefiniert", py=befund_cfg_features,
             note="cfg(feature) ohne Definition: Code und Tests werden still nie kompiliert"),
        Step("B-befunde", "befund_default_members", py=befund_default_members,
             note="welche Crates `cargo test` ohne --workspace ueberspringt"),
        Step("B-befunde", "befund_maturity_claims", py=befund_maturity_claims),
        Step("B-befunde", "befund_bench_fuzz_dateien", py=befund_bench_fuzz_dateien),
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

    # --- 3: Testbarkeit ----------------------------------------------------------------------
    ctx.selected = crates
    S += [
        Step("3-testbarkeit", "testbarkeit_crates", py=testbarkeit_crates, timeout=900,
             note="Tests je Crate, ignorierte/assertionlose Tests, Testnachweis der pub-Funktionen"),
        Step("3-testbarkeit", "testbarkeit_funktionen", py=testbarkeit_funktionen, timeout=900,
             note=f"Komplexitaet, Testnachweis, direkte Umweltzugriffe je Funktion (FAIL ab Komplexitaet {KOMPLEX_SCHWELLE})"),
        Step("3-testbarkeit", "testbarkeit_verwaiste_dateien", py=testbarkeit_verwaiste, timeout=600,
             note="Test-/Benchdateien ohne Crate-Zuordnung laufen nie"),
    ]
    for c in crates:
        S.append(Step("3-testbarkeit", f"liste_tests__{c}",
                      cargo + ["test", "-p", c, "--tests", "--locked", "--", "--list"],
                      timeout=SLOW_CRATES.get(c, DEFAULT_TIMEOUT), info=True,
                      note="was cargo tatsaechlich als Test kennt (gegen statische Zaehlung)"))
    if args.all or args.coverage:
        S += [
            Step("3-testbarkeit", "coverage", cargo + ["llvm-cov"] + pk + ["--locked", "--no-fail-fast", "--json",
                 "--ignore-filename-regex", "/(tests|benches|fuzz|examples)/", "--output-path",
                 str(outdir / "coverage.json")], requires=["cargo-llvm-cov"], timeout=10800),
            Step("3-testbarkeit", "coverage_funktionen", py=coverage_funktionen, timeout=600,
                 note="Funktionsabdeckung je Crate, nie ausgefuehrte Funktionen"),
        ]

    # --- 4: Benchmarks -----------------------------------------------------------------------
    if not args.quick:
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
        ]

    if args.gruppe:
        g = args.gruppe.lower()
        def _passt(group: str) -> bool:
            # exakt: Kennung ("s", "1", "t1") oder Gruppenname ("befunde", "statisch") ; kein Praefix-Treffer
            kennung, _, name = group.lower().partition("-")
            return g in (kennung, name, group.lower())
        S = [s for s in S if _passt(s.group)]
    # Gruppenfilter darf den Preflight nicht wegfiltern, sonst laufen cargo-Schritte ohne Toolchain-Pruefung
    if any(s.needs_cargo for s in S) and not any(s.name == PREFLIGHT for s in S):
        S.insert(0, Step("0-basis", PREFLIGHT, py=toolchain_preflight, timeout=1000, cargo=False,
                         note="Gate (automatisch ergaenzt)"))
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
    elif step.name.startswith("liste_tests__"):
        step.detail = (step.detail + f" {len(re.findall(r'^\\S.*: test$', text, re.M))} Tests gelistet").strip()
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


def testbarkeit_report(ctx: Ctx, steps: list[Step], outdir: Path) -> list[str]:
    L: list[str] = []
    p = outdir / "testbarkeit_crates.csv"
    if p.exists():
        with open(p) as fh:
            rows = list(csv.DictReader(fh))
        L += ["", "## Testbarkeit je Crate (statisch)", ""]
        L += _md_table(rows, ["crate", "tests_unit", "tests_integration", "doctests", "ignoriert",
                              "tests_ohne_assertion", "pub_fns", "pub_mit_nachweis", "pub_ohne_nachweis",
                              "nachweis_quote", "urteil"])
    vr = export_testliste(ctx, steps, outdir)
    if vr:
        L += ["", "## Statisch gezaehlte Tests gegen `cargo test -- --list`", "",
              "Differenz > 0: Tests im Quelltext, die cargo nicht kennt (Feature/Plattform/cfg). "
              "Differenz < 0: Tests aus Makros oder nicht erkannten Attributen.", ""]
        L += _md_table(vr, ["crate", "statisch", "gelistet", "differenz"])
    p = outdir / "coverage_crates.csv"
    if p.exists():
        with open(p) as fh:
            rows = list(csv.DictReader(fh))
        L += ["", "## Funktionsabdeckung je Crate (cargo llvm-cov)", ""]
        L += _md_table(rows, ["crate", "funktionen", "funktionen_belegt", "funktionen_prozent", "zeilen_prozent"])
    return L


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
        L += testbarkeit_report(ctx, steps, outdir)
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

START_EPOCH: float = 0.0


def main() -> int:
    ap = argparse.ArgumentParser(description="Contextra: testen, benchmarken, Befunde pruefen, Ergebnisse speichern")
    ap.add_argument("--quick", action="store_true", help="ohne Benchmarks, Fuzz, Mutation")
    ap.add_argument("--all", action="store_true", help="zusaetzlich Fuzz, Mutation, Miri, ASan, Loom, Coverage")
    ap.add_argument("--coverage", action="store_true", help="zusaetzlich cargo llvm-cov und Funktionsabdeckung")
    ap.add_argument("--only", help="kommagetrennte Crates (Kurzname), z. B. store,mvcc")
    ap.add_argument("--gruppe", help="nur Gruppe (Name oder Kennung), z. B. befunde, statisch, tests, testbarkeit, bench, tief")
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
    if ctx.ws.fehlend or ctx.ws.fehlend_default or ctx.ws.problems:
        print(f"Workspace: {len(ctx.ws.fehlend)} Mitglieder ohne Verzeichnis, "
              f"{len(ctx.ws.fehlend_default)} default-members ohne Verzeichnis, "
              f"{len(ctx.ws.problems)} sonstige Hinweise (Details im Befund 'default_members')")

    start = now_utc()
    global START_EPOCH
    START_EPOCH = time.time()
    t_start = START_EPOCH
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