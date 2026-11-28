#!/usr/bin/env python3
"""
contextra_full_audit.py (v4) - Messen, Testen, Befunde pruefen; alle Ergebnisse maschinenlesbar speichern.

Aufruf im Repo-Root (Cargo.toml mit [workspace]):

  python3 tools/contextra_full_audit.py --list                       # Plan zeigen, nichts ausfuehren
  python3 tools/contextra_full_audit.py --gruppe befunde             # nur statische Befunde (kein Build)
  python3 tools/contextra_full_audit.py --messen --profil smoke      # nur Messung, ca. Minuten (Funktionstest)
  python3 tools/contextra_full_audit.py --messen --profil standard   # nur Messung, Standardtiefe
  python3 tools/contextra_full_audit.py --messen --profil max        # maximale Messtiefe (Stunden)
  python3 tools/contextra_full_audit.py --quick                      # Befunde, check, fmt, clippy, Tests (ohne Messung)
  python3 tools/contextra_full_audit.py                              # Standard: alles ausser Fuzz/Mutation/Miri/Coverage
  python3 tools/contextra_full_audit.py --all                        # zusaetzlich Fuzz, Mutation, Loom, Miri, ASan, Coverage
  python3 tools/contextra_full_audit.py --only store,vector          # nur bestimmte Crates (Kurzname)
  python3 tools/contextra_full_audit.py --resume evidence/audit-...  # PASS/INFO-Schritte werden uebernommen
  python3 tools/contextra_full_audit.py --vergleich evidence/audit-A evidence/audit-B

Weitere Optionen: --repeat N, --flake N, --native, --no-overflow-checks, --env K=V (mehrfach),
  --baseline NAME, --save-baseline NAME, --criterion-home PFAD, --target-dir PFAD, --test-json,
  --fail-fast, --jobs N, --timeout-faktor F, --root PFAD

Was gemessen wird (Gruppe 4-messung):
  hygiene             Messumgebung: Last, CPU-Governor, Turbo, Swap, stoerende Prozesse (Warnungen, kein Abbruch)
  bench_build         Kompilieren aller Bench-Ziele (Zeit, CPU, Spitzen-RSS des Builds)
  bench__<crate>__<n> jede Criterion-Bench einzeln, `--repeat` mal; eigenes CRITERION_HOME je Lauf-Verzeichnis
                      -> bench_runs.csv (jede Wiederholung), bench_results.csv (Median ueber Laeufe, CV, Durchsatz)
  harness_*           vorhandene Messwerkzeuge von contextra-bench: synthetische Genauigkeit, ANN-Gate,
                      scale-100k (Einfuegerate, Latenz-Perzentile, RSS) und der additive Bin `contextra-measure`
                      (ANN-Sweep mit frei waehlbarer Groesse; Datensaetze wie benchmarks/data/de_mini)
  Jeder Schritt misst zusaetzlich CPU-Zeit (user/sys) und Spitzen-RSS (wait4-Rusage) -> results.csv.

Ergebnisse: evidence/audit-<UTC>-<commit>/
  summary.md            Zusammenfassung: rote Schritte, Messtabellen, Rauschen, Ressourcen, langsamste Tests
  results.json/.csv     jeder Schritt: Status, Dauer, CPU, RSS, Detail, Log, Metriken
  metrics.csv           alle Kennzahlen der Harness-Laeufe (Schritt, Schluessel, Wert)
  bench_runs.csv        Criterion je Benchmark und Wiederholung
  bench_results.csv     Criterion aggregiert (Median, min, max, CV %, Durchsatz)
  scale_curve.csv       scale-100k: Dokumente vs. Einfuegerate, Latenz, RSS
  tests.csv             jeder Einzeltest mit Ergebnis; test_binaries.csv Laufzeit je Test-Binary
  flake_report.csv      Tests mit uneinheitlichem Ergebnis ueber Wiederholungen, Hang-Kandidaten
  panic_flaeche.csv, unsafe_inventar.csv, loc.csv
  criterion/            Rohdaten (CRITERION_HOME), harness/ JSON- und Textausgaben, logs/, env.txt

Der Quellcode des Projekts wird nie veraendert. Harness-Laeufe starten in einem Arbeitsverzeichnis unter
der Evidenz, damit hartkodierte relative Ausgabepfade (benchmarks/results/) nicht ins Repo schreiben.
Nur Python-Standardbibliothek (3.11+), Linux/macOS (Prozessgruppen, wait4).
"""
from __future__ import annotations

import argparse
import csv
import datetime as dt
import glob
import json
import math
import os
import platform
import re
import shutil
import signal
import statistics
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

# Messtiefe. crit = Criterion-Argumente (leer = Criterion-Standard: 3 s Warmup, 5 s, 100 Samples).
PROFILES = {
    "smoke": dict(crit=["--warm-up-time", "0.5", "--measurement-time", "1", "--sample-size", "10"], repeat=1,
                  ann=[(5_000, 128)], ann_q=100, scale=(10_000, 5_000), flake=0, fuzz=30),
    "standard": dict(crit=[], repeat=3,
                     ann=[(10_000, 128), (100_000, 128)], ann_q=500, scale=(100_000, 10_000), flake=0, fuzz=120),
    "max": dict(crit=["--measurement-time", "10", "--sample-size", "100"], repeat=5,
                ann=[(10_000, 128), (100_000, 128), (100_000, 768), (500_000, 128)], ann_q=1_000,
                scale=(1_000_000, 100_000), flake=3, fuzz=600),
}

BENCH_CRATE = "contextra-bench"
HARNESS_BINS = {"synthetic": "contextra-bench", "scale": "scale-100k", "measure": "contextra-measure"}
DATASET_DIR = Path("benchmarks") / "data"

SLOW_CRATES = {"contextra-crypto": 3600, "contextra-store": 3600, "contextra-vector": 3600,
               "contextra-graph": 3600, "contextra-engine": 3600}
DEFAULT_TIMEOUT = 1800
BENCH_TIMEOUT = 3600
PREFLIGHT = "toolchain_preflight"
SAMPLE_SEC = 0.0  # reserviert

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


def fmt_ns(ns: float) -> str:
    for unit, div in (("s", 1e9), ("ms", 1e6), ("us", 1e3)):
        if abs(ns) >= div:
            return f"{ns / div:.3g} {unit}"
    return f"{ns:.3g} ns"


def md_cell(s: object) -> str:
    return str(s).replace("|", "\\|").replace("\n", " ")


def write_csv(path: Path, header: list[str], rows: list[list]) -> None:
    with open(path, "w", newline="") as fh:
        w = csv.writer(fh)
        w.writerow(header)
        w.writerows(rows)


def flatten_json(obj, prefix: str = "", out: Optional[dict] = None, limit: int = 400) -> dict:
    """Numerische und boolesche Blaetter eines JSON-Dokuments als {pfad: wert}."""
    out = {} if out is None else out
    if len(out) >= limit:
        return out
    if isinstance(obj, bool):
        out[prefix] = int(obj)
    elif isinstance(obj, (int, float)):
        out[prefix] = obj
    elif isinstance(obj, dict):
        for k, v in obj.items():
            flatten_json(v, f"{prefix}.{k}" if prefix else str(k), out, limit)
    elif isinstance(obj, list) and len(obj) <= 64:
        for i, v in enumerate(obj):
            flatten_json(v, f"{prefix}[{i}]", out, limit)
    return out


# ----------------------------------------------------------------------------------------------
# Workspace-Modell (aus Cargo.toml, ohne cargo)
# ----------------------------------------------------------------------------------------------

class Workspace:
    def __init__(self, root: Path):
        self.root = root
        ws = _toml(root / "Cargo.toml").get("workspace", {})
        self.members: dict[str, Path] = {}
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
    def __init__(self, root: Path, args):
        self.root = root
        self.args = args
        self.ws = Workspace(root)
        self.caps: dict = _toml(root / "capabilities.toml").get("crates", {})
        self.pin = (_toml(root / "rust-toolchain.toml").get("toolchain") or {}).get("channel", "(kein Pin)")
        self.profile = PROFILES[args.profil]
        self.target_dir = Path(args.target_dir or os.environ.get("CARGO_TARGET_DIR") or root / "target").resolve()
        self.outdir: Path = root  # wird in main gesetzt
        self.extra_env: dict[str, str] = {}
        for kv in args.env or []:
            k, _, v = kv.partition("=")
            if not k or not _:
                raise SystemExit(f"--env erwartet KEY=VALUE, erhalten: {kv!r}")
            self.extra_env[k] = v


def crate_for_path(path: str, dir_map: dict[str, str]) -> str:
    best: Optional[tuple[str, str]] = None
    for rel, name in dir_map.items():
        if path == rel or path.startswith(rel + "/"):
            if best is None or len(rel) > len(best[0]):
                best = (rel, name)
    return best[1] if best else path.split("/")[0]


# ----------------------------------------------------------------------------------------------
# Prozesse: ganze Gruppe beenden, Ressourcen je Kind per wait4 erfassen
# ----------------------------------------------------------------------------------------------

class Usage:
    """CPU-Sekunden und Spitzen-RSS eines Kindprozesses samt seiner abgewarteten Nachfahren."""

    def __init__(self) -> None:
        self.user = self.sys = self.rss_mb = 0.0

    def add(self, ru) -> None:
        if ru is None:
            return
        self.user += ru.ru_utime
        self.sys += ru.ru_stime
        div = 1024 * 1024 if sys.platform == "darwin" else 1024  # Linux: KiB, macOS: Bytes
        self.rss_mb = max(self.rss_mb, ru.ru_maxrss / div)


def _kill_group(p: subprocess.Popen) -> None:
    for sig in (signal.SIGTERM, signal.SIGKILL):
        try:
            os.killpg(p.pid, sig)
        except ProcessLookupError:
            break
        try:
            p.wait(timeout=15)
            break
        except subprocess.TimeoutExpired:
            continue
    try:
        p.wait(timeout=5)
    except (subprocess.TimeoutExpired, ChildProcessError):
        pass


def run_logged(cmd: list[str], cwd: Path, env: dict, out_fh, timeout: int,
               usage: Optional[Usage] = None) -> tuple[Optional[int], bool]:
    """Ausgabe in out_fh. Rueckgabe: (returncode | None bei Timeout, timed_out)."""
    p = subprocess.Popen(cmd, cwd=cwd, env=env, stdout=out_fh, stderr=subprocess.STDOUT, start_new_session=True)
    deadline = time.monotonic() + timeout
    try:
        while True:
            pid, status, ru = os.wait4(p.pid, os.WNOHANG)
            if pid == p.pid:
                p.returncode = os.waitstatus_to_exitcode(status)
                if usage is not None:
                    usage.add(ru)
                return p.returncode, False
            if time.monotonic() > deadline:
                _kill_group(p)
                return None, True
            time.sleep(0.05)
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

PyFn = Callable[["Ctx", Path], "tuple[str, str, str]"]          # (ctx, outdir) -> (status, detail, logtext)
PostFn = Callable[["Step", str, "Ctx", Path, int], None]         # (step, logtext_des_laufs, ctx, outdir, lauf_index)


class Step:
    def __init__(self, group: str, name: str, cmd: Optional[list[str]] = None, *, timeout: int = DEFAULT_TIMEOUT,
                 requires: Optional[list[str]] = None, env: Optional[dict] = None, cwd: Optional[Path] = None,
                 note: str = "", py: Optional[PyFn] = None, info: bool = False, cargo: Optional[bool] = None,
                 post: Optional[PostFn] = None, repeat: int = 1, measure: bool = False,
                 skip_if: Optional[Callable[[], str]] = None):
        self.group, self.name, self.cmd = group, name, cmd or []
        self.timeout, self.requires = timeout, requires or []
        self.env, self.cwd, self.note, self.py = env or {}, cwd, note, py
        self.info, self.post, self.repeat, self.measure = info, post, max(1, repeat), measure
        self.skip_if = skip_if
        self.needs_cargo = cargo if cargo is not None else (bool(self.cmd) and self.cmd[0] == "cargo")
        self.status, self.rc, self.seconds = "PENDING", None, 0.0
        self.log, self.detail = "", ""
        self.cpu_user = self.cpu_sys = self.rss_mb = 0.0
        self.load_before = self.load_after = 0.0
        self.metrics: dict[str, object] = {}
        # ----------------------------------------------------------------------------------------------
# Datenablage je Schritt (eine Datei je Schritt und Art: bei erneutem Lauf wird ueberschrieben)
# ----------------------------------------------------------------------------------------------

def put_data(outdir: Path, kind: str, step: str, payload) -> None:
    d = outdir / "data"
    d.mkdir(parents=True, exist_ok=True)
    safe = re.sub(r"[^A-Za-z0-9_.-]", "_", step)
    (d / f"{kind}__{safe}.json").write_text(json.dumps(payload, ensure_ascii=False))


def get_data(outdir: Path, kind: str) -> list[tuple[str, object]]:
    out = []
    for f in sorted(glob.glob(str(outdir / "data" / f"{kind}__*.json"))):
        try:
            out.append((Path(f).stem.split("__", 1)[1], json.loads(Path(f).read_text())))
        except (OSError, ValueError):
            continue
    return out


# ----------------------------------------------------------------------------------------------
# Parser
# ----------------------------------------------------------------------------------------------

TEST_LINE = re.compile(r"^test (\S+) \.\.\. (ok|FAILED|ignored(?:, .*)?)\s*$")
TEST_START = re.compile(r"^test (\S+) \.\.\. $")
RUNNING = re.compile(r"^\s+Running (?:unittests )?(.+?) \(([^)]+)\)\s*$")
DOCTESTS = re.compile(r"^\s+Doc-tests (\S+)")
RESULT = re.compile(r"^test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored;.*?finished in ([\d.]+)s")


def parse_tests(text: str) -> tuple[list[tuple[str, str, str]], list[tuple[str, int, int, int, float]], Optional[tuple[str, str]]]:
    """-> (Einzeltests [(binary, test, status)], Binaries [(binary, ok, rot, ign, sek)], Hang-Kandidat)."""
    tests: list[tuple[str, str, str]] = []
    bins: list[tuple[str, int, int, int, float]] = []
    cur, last_open = "?", None
    for line in text.splitlines():
        m = RUNNING.match(line)
        if m:
            cur, last_open = m.group(1), None
            continue
        m = DOCTESTS.match(line)
        if m:
            cur, last_open = f"doc-tests {m.group(1)}", None
            continue
        m = TEST_LINE.match(line)
        if m:
            st = "ignored" if m.group(2).startswith("ignored") else m.group(2)
            tests.append((cur, m.group(1), st))
            last_open = None
            continue
        m = TEST_START.match(line)
        if m:
            last_open = (cur, m.group(1))
            continue
        m = RESULT.match(line)
        if m:
            bins.append((cur, int(m.group(1)), int(m.group(2)), int(m.group(3)), float(m.group(4))))
            last_open = None
    return tests, bins, last_open


def parse_cargo_test_summary(text: str) -> str:
    tests, bins, hang = parse_tests(text)
    passed = sum(b[1] for b in bins)
    failed = sum(b[2] for b in bins)
    ignored = sum(b[3] for b in bins)
    failing = sorted({t for _, t, s in tests if s == "FAILED"})
    s = f"passed={passed} failed={failed} ignored={ignored} binaries={len(bins)}"
    if failing:
        s += " | FAILED: " + ", ".join(failing[:12]) + (" ..." if len(failing) > 12 else "")
    if hang:
        s += f" | HANG? zuletzt begonnen: {hang[1]}"
    return s


def parse_nextest_summary(text: str) -> str:
    lines = re.findall(r"^.*tests? run: .*$", text, re.M)
    if not lines:
        return "nextest: keine Summary-Zeile gefunden"
    last = lines[-1]
    p = re.search(r"(\d+) passed", last)
    f = re.search(r"(\d+) failed", last)
    failing = re.findall(r"^\s*(?:FAIL|TIMEOUT|SIGSEGV|ABORT)\s+\[[^\]]*\]\s+(\S.*?)\s*$", text, re.M)
    s = f"passed={p.group(1) if p else 0} failed={f.group(1) if f else 0}"
    if failing:
        s += " | FAILED: " + ", ".join(sorted(set(failing))[:12])
    return s


def parse_clippy(text: str) -> str:
    warns = len(re.findall(r"^warning: (?!`)", text, re.M))
    errs = len(re.findall(r"^error(\[\w+\])?: ", text, re.M))
    return f"errors={errs} warnings={warns}"


def parse_kv(text: str) -> dict[str, str]:
    """Zeilen der Form schluessel=wert (Ausgabe von contextra-measure)."""
    return dict(re.findall(r"^([A-Za-z0-9_@.\-]+)=(\S+)\s*$", text, re.M))


def to_num(v: str):
    try:
        return int(v)
    except ValueError:
        try:
            return float(v)
        except ValueError:
            return v


def parse_panic_surface(text: str, dir_map: dict[str, str]) -> dict[str, dict[str, int]]:
    out: dict[str, dict[str, int]] = defaultdict(lambda: defaultdict(int))
    for m in re.finditer(r"^warning: (.+)\n\s+--> ([^:\n]+):\d+:\d+", text, re.M):
        msg, path = m.group(1), m.group(2)
        kind = next((k for k, needle in PANIC_KINDS if needle in msg), None)
        if kind:
            out[crate_for_path(path, dir_map)][kind] += 1
    return out


# ----------------------------------------------------------------------------------------------
# Bench-Ziele (aus Cargo.toml, nicht hartkodiert)
# ----------------------------------------------------------------------------------------------

def bench_targets(d: Path) -> list[dict]:
    """Je Bench-Ziel: name, harness, criterion, features, path, exists."""
    ct = _toml(d / "Cargo.toml")
    declared = {b.get("name"): b for b in ct.get("bench", []) if b.get("name")}
    out = []
    for name, b in declared.items():
        p = d / b.get("path", f"benches/{name}.rs")
        out.append(dict(name=name, harness=b.get("harness", True), features=b.get("required-features", []),
                        path=p, exists=p.exists(), criterion="criterion" in _read(p)))
    bdir = d / "benches"
    if bdir.is_dir() and ct.get("package", {}).get("autobenches", True):
        for f in sorted(bdir.glob("*.rs")):
            if f.stem not in declared:
                out.append(dict(name=f.stem, harness=True, features=[], path=f, exists=True,
                                criterion="criterion" in _read(f)))
    return out


def criterion_snapshot(home: Path, since: float, target: str) -> list[dict]:
    """Criterion-Schaetzungen, die seit `since` geschrieben wurden."""
    rows = []
    for est in home.rglob("new/estimates.json"):
        try:
            if est.stat().st_mtime < since - 1:
                continue
            d = json.loads(est.read_text())
            name = "/".join(est.relative_to(home).parts[:-2])
            row = dict(benchmark=name, target=target, mean_ns=d["mean"]["point_estimate"],
                       median_ns=d["median"]["point_estimate"], std_ns=d["std_dev"]["point_estimate"],
                       ci_low_ns=d["mean"]["confidence_interval"]["lower_bound"],
                       ci_high_ns=d["mean"]["confidence_interval"]["upper_bound"],
                       tp_elems=None, tp_bytes=None)
            bm = est.parent / "benchmark.json"
            if bm.exists():
                tp = (json.loads(bm.read_text()).get("throughput") or {})
                row["tp_elems"], row["tp_bytes"] = tp.get("Elements"), tp.get("Bytes")
            rows.append(row)
        except (OSError, KeyError, ValueError):
            continue
    return rows


# ----------------------------------------------------------------------------------------------
# Statische Befund-Pruefungen (reines Python, kein Build). FAIL = Befund liegt vor, INFO = Messung/Hinweis.
# ----------------------------------------------------------------------------------------------

def befund_cfg_features(ctx: Ctx, outdir: Path):
    """cfg(feature = "x") im Code, aber x ist in der Cargo.toml des Crates nicht definiert."""
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


def befund_default_members(ctx: Ctx, outdir: Path):
    ws = ctx.ws
    not_def = sorted(set(ws.members) - set(ws.default))
    detail = f"members={len(ws.members)} default-members={len(ws.default)}; nicht in default-members: {not_def or 'keine'}"
    return ("FAIL" if ws.problems else "INFO"), detail, "\n".join(ws.problems)


def befund_repo_huelle(ctx: Ctx, outdir: Path):
    """Vertrauensminimum: Lizenzdatei, README, CI, die test/clippy/fmt tatsaechlich ausfuehrt."""
    r = ctx.root
    probs = []
    if not any(r.glob("LICENSE*")) and not any(r.glob("COPYING*")):
        probs.append("keine LICENSE-Datei im Root (Cargo.toml deklariert: "
                     f"{(_toml(r / 'Cargo.toml').get('workspace', {}).get('package', {}) or {}).get('license', '?')})")
    if not any(r.glob("README*")):
        probs.append("kein README im Root")
    wf = r / ".github" / "workflows"
    text = "\n".join(_read(p) for p in sorted(wf.glob("*.y*ml"))) if wf.is_dir() else ""
    if not text:
        probs.append("keine CI-Workflows (.github/workflows)")
    else:
        for needle, label in (("cargo test", "cargo test"), ("clippy", "clippy"), ("fmt", "cargo fmt")):
            if needle not in text:
                probs.append(f"CI fuehrt `{label}` nirgends aus")
    return ("FAIL" if probs else "PASS"), f"{len(probs)} Befunde", "\n".join(probs) or "ok"


def befund_maturity_claims(ctx: Ctx, outdir: Path):
    caps = ctx.caps
    stable = [k for k, v in caps.items() if v.get("maturity") == "stable"]
    beta = [k for k, v in caps.items() if v.get("maturity") == "beta"]
    fehlend = [k for k in caps if k not in ctx.ws.members]
    no_cap = [k for k in ctx.ws.members if k not in caps]
    text = f"stable: {stable}\nbeta: {beta}\nin capabilities.toml, aber kein Mitglied: {fehlend}\nMitglied ohne Eintrag: {no_cap}"
    detail = f"stable={len(stable)} beta={len(beta)} Eintraege ohne Crate={len(fehlend)}"
    return ("FAIL" if fehlend else "INFO"), detail, text


def befund_zufall_zeit(ctx: Ctx, outdir: Path):
    """Nichtdeterminismus im Produktivcode (macht Messungen schwer reproduzierbar)."""
    pats = {"from_entropy": re.compile(r"\bfrom_entropy\b"), "thread_rng": re.compile(r"\bthread_rng\b"),
            "SystemTime::now": re.compile(r"\bSystemTime::now\b"), "Instant::now": re.compile(r"\bInstant::now\b")}
    rows = []
    for name, d in sorted(ctx.ws.members.items()):
        counts = {k: 0 for k in pats}
        for rs in (d / "src").rglob("*.rs"):
            t = _read(rs)
            for k, rx in pats.items():
                counts[k] += len(rx.findall(t))
        if any(counts.values()):
            rows.append(f"{name}: " + ", ".join(f"{k}={v}" for k, v in counts.items() if v))
    return "INFO", f"{len(rows)} Crates mit Zufalls-/Zeitquellen (inkl. Test-Module)", "\n".join(rows)


def befund_bench_inventar(ctx: Ctx, outdir: Path):
    """Bench-/Fuzz-Ziele aus den Cargo.toml: fehlende Dateien sind ein Befund, Nicht-Criterion-Benches ein Hinweis."""
    missing, notes, n_bench, n_fuzz = [], [], 0, 0
    for name, d in sorted(ctx.ws.members.items()):
        for b in bench_targets(d):
            n_bench += 1
            if not b["exists"]:
                missing.append(f"{name}: Bench `{b['name']}` -> Datei fehlt ({b['path'].relative_to(ctx.root)})")
            elif b["harness"]:
                notes.append(f"{name}/{b['name']}: Standard-Harness (libtest-bench, braucht nightly) wird nicht ausgefuehrt")
            elif not b["criterion"]:
                notes.append(f"{name}/{b['name']}: eigener Harness ohne Criterion (keine Criterion-Argumente)")
        ft = d / "fuzz" / "fuzz_targets"
        if ft.is_dir():
            n_fuzz += len(list(ft.glob("*.rs")))
    detail = f"{n_bench} Bench-Ziele, {n_fuzz} Fuzz-Ziele, {len(missing)} fehlende Dateien"
    return ("FAIL" if missing else "INFO"), detail, "\n".join(missing + notes) or "ok"


def befund_daten_verdrahtung(ctx: Ctx, outdir: Path):
    """Datensaetze in benchmarks/data, die im Code/Skripten nirgends referenziert werden, messen nichts."""
    base = ctx.root / DATASET_DIR
    if not base.is_dir():
        return "INFO", f"{DATASET_DIR} nicht vorhanden", ""
    haystack = ""
    for pat in ("*.rs", "*.toml", "*.py", "*.sh", "*.yml", "*.yaml"):
        for f in ctx.root.rglob(pat):
            if {"target", "evidence", ".git"} & set(f.parts) or f.name == Path(__file__).name:
                continue
            haystack += _read(f)
    unwired = [d.name for d in sorted(base.iterdir()) if d.is_dir() and d.name not in haystack]
    detail = f"{len(unwired)} nicht verdrahtete Datensaetze: {unwired}" if unwired else "alle Datensaetze referenziert"
    return ("FAIL" if unwired else "PASS"), detail, ""


def befund_profil(ctx: Ctx, outdir: Path):
    """Build-Profile und RUSTFLAGS: bestimmen, was die Benchmarks tatsaechlich messen."""
    prof = _toml(ctx.root / "Cargo.toml").get("profile", {})
    cfg = _toml(ctx.root / ".cargo" / "config.toml")
    lines = []
    for name in ("release", "bench"):
        if name in prof:
            lines.append(f"[profile.{name}] " + ", ".join(f"{k}={v}" for k, v in prof[name].items()))
    rf = (cfg.get("build") or {}).get("rustflags")
    lines.append(f"[build].rustflags = {rf}" if rf else "keine rustflags in .cargo/config.toml (kein target-cpu=native)")
    warn = []
    rel = {**prof.get("release", {}), **prof.get("bench", {})}
    if rel.get("overflow-checks") is True:
        warn.append("overflow-checks=true im Release/Bench: Benchmarks messen mit Ueberlaufpruefung (Option --no-overflow-checks vergleicht)")
    if rel.get("lto") in ("fat", True):
        warn.append("lto=fat: Bench-Builds sind langsam und speicherhungrig")
    return "INFO", "; ".join(warn) or "Profile ohne Auffaelligkeit", "\n".join(lines)


# ----------------------------------------------------------------------------------------------
# Inventare: unsafe gegen capabilities.toml, Zeilen je Crate
# ----------------------------------------------------------------------------------------------

UNSAFE_WORD = re.compile(r"\bunsafe\b")


def _unsafe_lines(path: Path) -> int:
    n = 0
    for line in _read(path).splitlines():
        s = line.strip()
        if s.startswith("//") or not UNSAFE_WORD.search(s):
            continue
        if re.search(r'"[^"]*\bunsafe\b[^"]*"', s):
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
    write_csv(outdir / "unsafe_inventar.csv", ["crate", "unsafe_zeilen", "unsafe_island", "urteil"], rows)
    detail = f"{len(verstoss)} Verstoesse gegen unsafe_island" if verstoss else "keine Verstoesse"
    return ("FAIL" if verstoss else "PASS"), detail, "\n".join(map(str, rows))


def _count_lines(path: Path) -> tuple[int, int]:
    """(Code-Zeilen, Kommentar-/Leerzeilen) grob: Zeilenkommentare und Leerzeilen zaehlen nicht als Code."""
    code = other = 0
    for line in _read(path).splitlines():
        s = line.strip()
        if not s or s.startswith("//"):
            other += 1
        else:
            code += 1
    return code, other


def inventar_loc(ctx: Ctx, outdir: Path):
    rows = []
    tot = [0, 0, 0, 0]
    for name, d in sorted(ctx.ws.members.items()):
        src = [p for p in (d / "src").rglob("*.rs")] if (d / "src").is_dir() else []
        tst = [p for sub in ("tests", "benches", "fuzz") if (d / sub).is_dir() for p in (d / sub).rglob("*.rs")
               if "target" not in p.parts]
        sc = sum(_count_lines(p)[0] for p in src)
        tc = sum(_count_lines(p)[0] for p in tst)
        biggest = max(((_count_lines(p)[0], p) for p in src), default=(0, None), key=lambda x: x[0])
        rows.append([name, len(src), sc, len(tst), tc, biggest[0], str(biggest[1].relative_to(ctx.root)) if biggest[1] else ""])
        tot = [tot[0] + len(src), tot[1] + sc, tot[2] + len(tst), tot[3] + tc]
    write_csv(outdir / "loc.csv", ["crate", "src_dateien", "src_zeilen", "test_bench_fuzz_dateien",
                                   "test_bench_fuzz_zeilen", "groesste_datei_zeilen", "groesste_datei"], rows)
    over = sum(1 for r in rows if r[5] > 1500)
    return "INFO", (f"src={tot[1]} Zeilen in {tot[0]} Dateien; test/bench/fuzz={tot[3]} Zeilen in {tot[2]} Dateien; "
                    f"{over} Crates mit Datei > 1500 Zeilen"), "\n".join(map(str, rows))


# ----------------------------------------------------------------------------------------------
# Toolchain-Preflight und Messhygiene
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


def _sh(cmd: str, timeout: int = 20) -> str:
    try:
        return subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=timeout).stdout.strip()
    except (subprocess.SubprocessError, OSError):
        return ""


def messhygiene(ctx: Ctx, outdir: Path):
    """Warnungen zur Messumgebung. Rauschen vor der Messung ist guenstiger als Rauschen in den Zahlen."""
    ncpu = os.cpu_count() or 1
    load1, load5, _ = os.getloadavg()
    warn, info = [], []
    if load1 > 0.3 * ncpu:
        warn.append(f"Systemlast {load1:.2f} bei {ncpu} CPUs (>30 %): Messungen verrauscht")
    gov = _sh("cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null")
    if gov and gov != "performance":
        warn.append(f"CPU-Governor `{gov}` statt `performance` (cpupower frequency-set -g performance)")
    elif not gov:
        info.append("CPU-Governor nicht lesbar (VM/Container/macOS)")
    nt = _sh("cat /sys/devices/system/cpu/intel_pstate/no_turbo 2>/dev/null")
    if nt == "0":
        info.append("Turbo aktiv (Taktschwankungen moeglich; fuer Vergleiche no_turbo=1 setzen)")
    swap = _sh("grep -E 'SwapTotal|SwapFree' /proc/meminfo 2>/dev/null")
    m = re.findall(r"(\d+)", swap)
    if len(m) == 2 and int(m[0]) - int(m[1]) > 100 * 1024:
        warn.append(f"Swap in Benutzung ({(int(m[0]) - int(m[1])) // 1024} MiB)")
    top = _sh("ps -eo pcpu,comm --sort=-pcpu 2>/dev/null | sed -n '2,4p'")
    if top:
        info.append("Top-Prozesse (CPU %): " + " | ".join(l.strip() for l in top.splitlines()))
    if ncpu == 1:
        warn.append("nur 1 CPU: Mess- und Compilerlast konkurrieren; Parallelitaets-Benches sind nicht aussagekraeftig")
    cpu = _sh("lscpu 2>/dev/null | grep -E 'Model name|^CPU\\(s\\)|MHz' | head -4") or platform.processor()
    text = f"CPUs={ncpu} load1={load1:.2f} load5={load5:.2f}\n{cpu}\n" + "\n".join(info + warn)
    ctx_hyg = {"ncpu": ncpu, "load1": load1, "governor": gov, "warnungen": warn}
    put_data(outdir, "hygiene", "messhygiene", ctx_hyg)
    return "INFO", ("; ".join(warn) if warn else "keine Warnungen"), text
    # ----------------------------------------------------------------------------------------------
# Post-Hooks (werten die Ausgabe eines Laufs aus und legen Daten ab)
# ----------------------------------------------------------------------------------------------

def parse_tests_json(text: str):
    """libtest-JSON (--test-json): Einzeltests mit exec_time. Binary-Namen kommen aus den Cargo-Running-Zeilen."""
    tests, bins, cur = [], [], "?"
    for line in text.splitlines():
        m = RUNNING.match(line)
        if m:
            cur = m.group(1)
            continue
        if not line.startswith("{"):
            continue
        try:
            ev = json.loads(line)
        except ValueError:
            continue
        if ev.get("type") == "test" and ev.get("event") in ("ok", "failed", "ignored", "timeout"):
            tests.append((cur, ev.get("name", "?"), "FAILED" if ev["event"] in ("failed", "timeout") else ev["event"],
                          ev.get("exec_time")))
        elif ev.get("type") == "suite" and ev.get("event") in ("ok", "failed"):
            bins.append((cur, ev.get("passed", 0), ev.get("failed", 0), ev.get("ignored", 0), ev.get("exec_time", 0.0)))
    return tests, bins


def post_test(step: Step, text: str, ctx: Ctx, outdir: Path, run: int) -> None:
    crate = step.name.split("__", 1)[-1]
    if ctx.args.test_json and not step.name.startswith("test_doc__"):
        t4, bins = parse_tests_json(text)
        _, _, hang = parse_tests(text)
        rows = [[b, t, s, e] for b, t, s, e in t4]
    else:
        t3, bins, hang = parse_tests(text)
        rows = [[b, t, s, None] for b, t, s in t3]
    put_data(outdir, "tests", step.name, {"crate": crate, "rows": rows})
    put_data(outdir, "testbins", step.name, {"crate": crate, "rows": [list(b) for b in bins]})
    if hang:
        step.metrics["hang_kandidat"] = f"{hang[0]}::{hang[1]}"


def post_bench(step: Step, text: str, ctx: Ctx, outdir: Path, run: int) -> None:
    home = Path(step.env.get("CRITERION_HOME") or ctx.outdir / "criterion")
    rows = criterion_snapshot(home, step.t_run0, step.name.split("__")[2] if step.name.count("__") >= 2 else step.name)
    for r in rows:
        r["run"] = run + 1
        r["step"] = step.name
    step.acc.extend(rows)
    put_data(outdir, "bench", step.name, step.acc)


def _kv_metrics(step: Step, text: str) -> dict:
    kv = {k: to_num(v) for k, v in parse_kv(text).items()}
    step.metrics.update(kv)
    return kv


def post_kv(step: Step, text: str, ctx: Ctx, outdir: Path, run: int) -> None:
    kv = _kv_metrics(step, text)
    if kv:
        step.detail = " ".join(f"{k}={v}" for k, v in list(kv.items())[:8])


def post_harness_build(step: Step, text: str, ctx: Ctx, outdir: Path, run: int) -> None:
    sizes = {}
    for key, b in HARNESS_BINS.items():
        p = ctx.target_dir / "release" / b
        if p.exists():
            sizes[f"bin_mb_{b}"] = round(p.stat().st_size / 1048576, 2)
    step.metrics.update(sizes)
    step.detail = ", ".join(f"{k.removeprefix('bin_mb_')}={v} MB" for k, v in sizes.items()) or "keine Binaries gefunden"


def _copy_results(workdir: Path, dest: Path) -> list[str]:
    copied = []
    src = workdir / "benchmarks" / "results"
    if src.is_dir():
        dest.mkdir(parents=True, exist_ok=True)
        for f in src.rglob("*"):
            if f.is_file():
                shutil.copy2(f, dest / f.name)
                copied.append(f.name)
    return copied


def post_synthetic(step: Step, text: str, ctx: Ctx, outdir: Path, run: int) -> None:
    files = _copy_results(outdir / "harness" / "work_synthetic", outdir / "harness" / "synthetic")
    for name in files:
        if name.endswith(".json"):
            try:
                flat = flatten_json(json.loads((outdir / "harness" / "synthetic" / name).read_text()))
            except (OSError, ValueError):
                continue
            step.metrics.update({k: v for k, v in flat.items()})
    step.detail = f"{len(files)} Ergebnisdateien, {len(step.metrics)} Kennzahlen; " + (
        "Szenario B uebersprungen" if "Scenario B skipped" in text else "")


def post_dataset(step: Step, text: str, ctx: Ctx, outdir: Path, run: int) -> None:
    post_kv(step, text, ctx, outdir, run)


def post_scale(step: Step, text: str, ctx: Ctx, outdir: Path, run: int) -> None:
    out = Path(step.env.get("_SCALE_OUT", ""))
    try:
        d = json.loads(out.read_text())
    except (OSError, ValueError):
        step.detail = "keine Ausgabedatei"
        return
    iv = d.get("intervals", [])
    put_data(outdir, "scale", step.name, iv)
    step.metrics["docs_requested"] = d.get("total_docs_requested")
    step.metrics["aborted_at_docs"] = d.get("aborted_at_docs")
    step.metrics["error"] = d.get("error")
    if iv:
        last = iv[-1]
        step.metrics.update({"docs_reached": last["docs_inserted"], "insert_rate_last": last["insert_rate_docs_per_sec"],
                             "search_p50_ns_last": last["search_p50_ns"], "search_p99_ns_last": last["search_p99_ns"],
                             "rss_mb_last": round(last["rss_bytes"] / 1048576, 1)})
    step.detail = (f"{len(iv)} Intervalle, erreicht={step.metrics.get('docs_reached')} von {step.metrics['docs_requested']}"
                   + (f", ABBRUCH bei {d['aborted_at_docs']}: {d.get('error')}" if d.get("aborted_at_docs") else ""))
    if d.get("aborted_at_docs") or d.get("error"):
        step.force_fail = True


def post_fuzz(step: Step, text: str, ctx: Ctx, outdir: Path, run: int) -> None:
    last = None
    for m in re.finditer(r"^#(\d+)\s+(\w+)\s+cov: (\d+) ft: (\d+) corp: (\d+)/\S+.*?exec/s: (\d+)", text, re.M):
        last = m
    if last:
        step.metrics.update({"execs": int(last.group(1)), "cov": int(last.group(3)), "ft": int(last.group(4)),
                             "corpus": int(last.group(5)), "exec_per_s": int(last.group(6))})
        step.detail = f"execs={last.group(1)} cov={last.group(3)} exec/s={last.group(6)}"
    if re.search(r"Test unit written to|SUMMARY: (libFuzzer|AddressSanitizer)", text):
        step.force_fail = True
        step.detail += " | ABSTURZ GEFUNDEN"


# ----------------------------------------------------------------------------------------------
# Flake-Scan: dieselben Tests mehrfach, abwechselnd parallel und mit --test-threads=1
# ----------------------------------------------------------------------------------------------

def make_flake(crate: str, n: int, timeout: int) -> PyFn:
    def fn(ctx: Ctx, outdir: Path):
        env = os.environ.copy()
        env.update(ctx.extra_env)
        seen: dict[tuple[str, str], list[tuple[str, str]]] = defaultdict(list)
        hangs, log, timeouts = [], [], 0
        for i in range(n):
            mode = "parallel" if i % 2 == 0 else "threads=1"
            cmd = ["cargo", "test", "-p", crate, "--tests", "--no-fail-fast", "--locked"]
            if mode == "threads=1":
                cmd += ["--", "--test-threads=1"]
            rc, out, err, to = run_capture(cmd, ctx.root, env, timeout)
            text = out + "\n" + err
            tests, bins, hang = parse_tests(text)
            for b, t, s in tests:
                seen[(b, t)].append((mode, s))
            if to:
                timeouts += 1
            if hang and (to or rc != 0):
                hangs.append(f"lauf {i + 1} ({mode}): {hang[0]}::{hang[1]}" + (" [Timeout]" if to else ""))
            log.append(f"lauf {i + 1}/{n} {mode}: rc={rc} timeout={to} tests={len(tests)} {parse_cargo_test_summary(text)}")
        rows, flaky = [], []
        for (b, t), res in sorted(seen.items()):
            states = {s for _, s in res}
            incomplete = len(res) < n
            if len(states) > 1 or incomplete:
                flaky.append((b, t))
                rows.append([crate, b, t, ";".join(f"{m}:{s}" for m, s in res) + (";fehlt-in-Laeufen" if incomplete else ""), "flaky"])
        for h in hangs:
            rows.append([crate, "", "", h, "hang"])
        put_data(outdir, "flake", f"flake__{crate}", rows)
        detail = f"{n} Laeufe, {len(flaky)} uneinheitliche Tests, {len(hangs)} Hang-Kandidaten, {timeouts} Timeouts"
        return ("FAIL" if flaky or hangs else "PASS"), detail, "\n".join(log + [r[3] for r in rows])
    return fn


# ----------------------------------------------------------------------------------------------
# Plan
# ----------------------------------------------------------------------------------------------

def nextest_ok() -> bool:
    return which("cargo-nextest") is not None


def build_flags_env(ctx: Ctx) -> dict:
    env: dict[str, str] = {}
    rf = ctx.extra_env.get("RUSTFLAGS", os.environ.get("RUSTFLAGS", ""))
    if ctx.args.native:
        rf = (rf + " -C target-cpu=native").strip()
    if rf:
        env["RUSTFLAGS"] = rf
    if ctx.args.no_overflow_checks:
        env["CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS"] = "false"
        env["CARGO_PROFILE_BENCH_OVERFLOW_CHECKS"] = "false"
    return env


def _bin_skip(ctx: Ctx, binname: str, hint: str = "") -> Callable[[], str]:
    p = ctx.target_dir / "release" / binname
    return lambda: "" if p.exists() else f"{p} fehlt{hint}"


def build_plan(ctx: Ctx, args, outdir: Path) -> list[Step]:
    ws, prof = ctx.ws, ctx.profile
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
    flags = build_flags_env(ctx)
    S: list[Step] = []
    measure_only = args.messen
    do_static = not measure_only
    do_measure = not (args.quick)
    do_deep = args.all

    # --- 1: Befunde (kein Build) -------------------------------------------------------------
    if do_static:
        S += [
            Step("1-befunde", "befund_repo_huelle", py=befund_repo_huelle, note="LICENSE, README, CI-Inhalt"),
            Step("1-befunde", "befund_cfg_features_undefiniert", py=befund_cfg_features),
            Step("1-befunde", "befund_default_members", py=befund_default_members),
            Step("1-befunde", "befund_maturity_claims", py=befund_maturity_claims),
            Step("1-befunde", "befund_zufall_zeit", py=befund_zufall_zeit, note="Reproduzierbarkeit der Messungen"),
            Step("1-befunde", "befund_bench_inventar", py=befund_bench_inventar),
            Step("1-befunde", "befund_daten_verdrahtung", py=befund_daten_verdrahtung,
                 note="Datensaetze in benchmarks/data ohne Referenz messen nichts"),
            Step("1-befunde", "befund_profil", py=befund_profil, note="Build-Profile, die die Benchmarks pr\u00e4gen"),
            Step("1-befunde", "inventar_loc", py=inventar_loc, timeout=300),
            Step("1-befunde", "unsafe_inventar", py=unsafe_inventar, timeout=300, note="gegen capabilities.toml"),
        ]

    # --- 0: Basis ----------------------------------------------------------------------------
    S += [
        Step("0-basis", PREFLIGHT, py=toolchain_preflight, timeout=1000, cargo=False,
             note="Gate: schlaegt er fehl, werden alle cargo-Schritte SKIPPED"),
        Step("0-basis", "toolchain_versionen", ["bash", "-c",
             "rustc -Vv; cargo -V; cargo clippy -V; cargo fmt -V; rustup show active-toolchain; true"],
             timeout=300, info=True),
        Step("0-basis", "cargo_fetch", cargo + ["fetch", "--locked"], timeout=900),
    ]
    if do_static:
        S.append(Step("0-basis", "cargo_tree_duplicates", cargo + ["tree", "--workspace", "--duplicates"],
                      timeout=300, info=True))

    # --- 2: Statisch -------------------------------------------------------------------------
    if do_static:
        S += [
            Step("2-statisch", "fmt_check", cargo + ["fmt", "--all", "--", "--check"], requires=["rustfmt"]),
            Step("2-statisch", "check_alle_targets", cargo + ["check"] + pk + ["--all-targets", "--locked", "--keep-going"]),
            Step("2-statisch", "check_no_default_features",
                 cargo + ["check"] + pk + ["--no-default-features", "--locked", "--keep-going"]),
            Step("2-statisch", "doc", cargo + ["doc"] + pk + ["--no-deps", "--locked"], env={"RUSTDOCFLAGS": "-D warnings"}),
        ]
        for c in crates:
            S.append(Step("2-statisch", f"clippy__{c}", cargo + ["clippy", "-p", c, "--all-targets", "--locked", "--",
                                                                "-D", "warnings"]))
        S += [
            Step("2-statisch", "deny", cargo + ["deny", "check"], requires=["cargo-deny"]),
            Step("2-statisch", "audit", cargo + ["audit"], requires=["cargo-audit"]),
            Step("2-statisch", "panic_flaeche_lib",
                 cargo + ["clippy"] + pk_lib + ["--lib", "--locked", "--keep-going", "--",
                          "-W", "clippy::unwrap_used", "-W", "clippy::expect_used", "-W", "clippy::panic",
                          "-W", "clippy::todo", "-W", "clippy::unimplemented", "-W", "clippy::indexing_slicing"],
                 timeout=3600, info=True, note="Panic-Stellen im Produktivcode; Auswertung: panic_flaeche.csv"),
        ]

    # --- 3: Tests ----------------------------------------------------------------------------
    if do_static:
        tj_env: dict[str, str] = {}
        tj_args: list[str] = []
        if args.test_json:
            tj_env = {"RUSTC_BOOTSTRAP": "1", "CARGO_TARGET_DIR": str(ctx.target_dir / "audit-testjson")}
            tj_args = ["--", "-Z", "unstable-options", "--format", "json", "--report-time"]
        for c in crates:
            t = SLOW_CRATES.get(c, DEFAULT_TIMEOUT)
            S.append(Step("3-tests", f"test__{c}", cargo + ["test", "-p", c, "--tests", "--no-fail-fast", "--locked"] + tj_args,
                          timeout=t, env=tj_env, post=post_test, note="lib-, Bin- und Integrationstests"))
            if c in libs:
                S.append(Step("3-tests", f"test_doc__{c}", cargo + ["test", "-p", c, "--doc", "--locked", "--no-fail-fast"],
                              timeout=t, post=post_test))
            if args.nextest and nextest_ok():
                S.append(Step("3-tests", f"nextest__{c}",
                              cargo + ["nextest", "run", "-p", c, "--locked", "--no-fail-fast", "--retries", "0"], timeout=t,
                              note="je Test eigener Prozess: deckt Test-Interferenz auf"))
        flake_n = args.flake if args.flake is not None else prof["flake"]
        if flake_n > 0:
            for c in crates:
                S.append(Step("3-tests", f"flake_scan__{c}", py=make_flake(c, flake_n, SLOW_CRATES.get(c, DEFAULT_TIMEOUT)),
                              timeout=SLOW_CRATES.get(c, DEFAULT_TIMEOUT) * flake_n,
                              note=f"{flake_n} Laeufe, abwechselnd parallel / threads=1"))

    # --- 4: Messung --------------------------------------------------------------------------
    if do_measure:
        crit_home = Path(args.criterion_home).resolve() if args.criterion_home else outdir / "criterion"
        repeat = args.repeat if args.repeat is not None else prof["repeat"]
        crit_flags = list(prof["crit"])
        if args.baseline:
            crit_flags += ["--baseline", args.baseline]
        if args.save_baseline:
            crit_flags += ["--save-baseline", args.save_baseline]
        benv = dict(flags)
        benv["CRITERION_HOME"] = str(crit_home)

        S.append(Step("4-messung", "messhygiene", py=messhygiene, info=True, cargo=False, timeout=120,
                      note="Last, Governor, Turbo, Swap: Warnungen vor der Messung"))
        bench_crates = [c for c in crates if any(b["exists"] and not b["harness"] for b in bench_targets(ws.members[c]))]
        if bench_crates:
            S.append(Step("4-messung", "bench_build",
                          cargo + ["bench", "--no-run", "--locked"] + [x for c in bench_crates for x in ("-p", c)],
                          env=flags, timeout=7200, measure=True,
                          note="Kompilierzeit/Speicher der Bench-Ziele (bench-Profil = release)"))
        for c in bench_crates:
            for b in bench_targets(ws.members[c]):
                if not b["exists"] or b["harness"]:
                    continue
                cmd = cargo + ["bench", "-p", c, "--bench", b["name"], "--locked"]
                if b["features"]:
                    cmd += ["--features", ",".join(b["features"])]
                if b["criterion"]:
                    cmd += ["--"] + crit_flags
                S.append(Step("4-messung", f"bench__{c}__{b['name']}", cmd, env=benv, timeout=BENCH_TIMEOUT,
                              post=post_bench, repeat=repeat if b["criterion"] else 1, measure=True,
                              note="" if b["criterion"] else "eigener Harness"))

        if BENCH_CRATE in crates:
            hroot = outdir / "harness"
            benv2 = dict(flags)
            S.append(Step("4-messung", "harness_build",
                          cargo + ["build", "--release", "-p", BENCH_CRATE, "--bins", "--features", "external-benchmarks",
                                   "--locked"], env=benv2, timeout=7200, post=post_harness_build, measure=True,
                          note="Messwerkzeuge von contextra-bench (Release-Profil)"))
            tdir = ctx.target_dir
            syn, scl, mea = (tdir / "release" / HARNESS_BINS[k] for k in ("synthetic", "scale", "measure"))
            S.append(Step("4-messung", "harness_synthetic_genauigkeit", [str(syn)], cwd=hroot / "work_synthetic",
                          timeout=1800, post=post_synthetic, cargo=False, measure=True,
                          skip_if=_bin_skip(ctx, HARNESS_BINS["synthetic"]),
                          note="Szenario A/B (Kontext-Praefix, Reranking); schreibt benchmarks/results in das Arbeitsverzeichnis"))
            S.append(Step("4-messung", "harness_ann_ci_gate", [str(syn), "--ann"], cwd=hroot / "work_ann", timeout=900,
                          cargo=False, measure=True, skip_if=_bin_skip(ctx, HARNESS_BINS["synthetic"]),
                          note="1k Vektoren, 50 Queries: Schwellen des Projekts (nur Smoke-Test)"))
            hint = " (additiver Bin: benchmarks/contextra-bench/src/bin/measure.rs + [[bin]]-Eintrag, siehe Anleitung)"
            for n, dim in prof["ann"]:
                outj = hroot / f"ann_{n}_{dim}d.json"
                S.append(Step("4-messung", f"harness_ann__{n}x{dim}",
                              [str(mea), "ann", "--n", str(n), "--dim", str(dim), "--queries", str(prof["ann_q"]),
                               "--out", str(outj)], cwd=hroot / "work_ann", timeout=BENCH_TIMEOUT * (2 if n >= 100_000 else 1),
                              post=post_kv, cargo=False, measure=True, skip_if=_bin_skip(ctx, HARNESS_BINS["measure"], hint),
                              note="HNSW-Sweep: Recall@10, QPS bei Recall 0.90/0.95/0.99, Latenz"))
            data_root = ctx.root / DATASET_DIR
            if data_root.is_dir():
                for dsd in sorted(p for p in data_root.iterdir() if p.is_dir()):
                    if (dsd / "corpus.jsonl").exists() and (dsd / "queries.jsonl").exists():
                        S.append(Step("4-messung", f"harness_dataset__{dsd.name}",
                                      [str(mea), "dataset", "--dir", str(dsd), "--out", str(hroot / f"dataset_{dsd.name}.json")],
                                      cwd=hroot / f"work_dataset_{dsd.name}", timeout=1800, post=post_dataset, cargo=False,
                                      measure=True, skip_if=_bin_skip(ctx, HARNESS_BINS["measure"], hint),
                                      note="NDCG@10, Recall@10, MAP@100, Latenz (Textsuche, siehe disclaimer in der JSON)"))
            docs, step_sz = prof["scale"]
            sj = hroot / f"scale_{docs}.json"
            S.append(Step("4-messung", f"harness_scale__{docs}",
                          [str(scl), "--docs", str(docs), "--step", str(step_sz), "--dim", "128", "--seed", "42",
                           "--out", str(sj)], cwd=hroot / "work_scale", timeout=max(3600, docs // 100_000 * 7200),
                          env={"_SCALE_OUT": str(sj)}, post=post_scale, cargo=False, measure=True,
                          skip_if=_bin_skip(ctx, HARNESS_BINS["scale"]),
                          note="Einfuegerate, Such-Perzentile und RSS ueber wachsende Datenmenge"))

    # --- 5: Tief -----------------------------------------------------------------------------
    if do_deep and not measure_only:
        for c in crates:
            d = ws.members[c]
            ft = d / "fuzz" / "fuzz_targets"
            if ft.is_dir():
                for tgt in sorted(p.stem for p in ft.glob("*.rs")):
                    S.append(Step("5-tief", f"fuzz__{short(c)}__{tgt}",
                                  ["cargo", "+nightly", "fuzz", "run", tgt, "--", f"-max_total_time={prof['fuzz']}"],
                                  cwd=d, requires=["cargo-fuzz"], timeout=prof["fuzz"] + 1200, post=post_fuzz, measure=True))
        if (ctx.root / ".cargo" / "mutants.toml").exists():
            S.append(Step("5-tief", "mutation", cargo + ["mutants", "--config", ".cargo/mutants.toml", "--timeout", "120"],
                          requires=["cargo-mutants"], timeout=14400))
        for c in crates:
            if "loom" in _read(ws.members[c] / "Cargo.toml"):
                S.append(Step("5-tief", f"loom__{short(c)}", cargo + ["test", "-p", c, "--release", "--locked"],
                              env={"RUSTFLAGS": "--cfg loom", "LOOM_MAX_PREEMPTIONS": "3"}, timeout=3600))
        islands = [c for c in crates if (ctx.caps.get(c) or {}).get("unsafe_island")]
        for c in islands:
            S.append(Step("5-tief", f"miri__{short(c)}", ["cargo", "+nightly", "miri", "test", "-p", c, "--lib"],
                          requires=["cargo-miri"], timeout=7200))
            S.append(Step("5-tief", f"asan__{short(c)}", ["cargo", "+nightly", "test", "-p", c, "--target",
                                                          "x86_64-unknown-linux-gnu"],
                          env={"RUSTFLAGS": "-Zsanitizer=address"}, timeout=3600))
        S.append(Step("5-tief", "test_release_workspace", cargo + ["test"] + pk + ["--release", "--locked", "--no-fail-fast"],
                      timeout=7200, post=post_test, note="Release deckt Overflow-/Optimierungsfehler auf"))
        S.append(Step("5-tief", "coverage", cargo + ["llvm-cov"] + pk + ["--locked", "--json", "--output-path",
                                                                       str(outdir / "coverage.json")],
                      requires=["cargo-llvm-cov"], timeout=10800))

    if args.gruppe:
        g = args.gruppe.lower()
        g = {"bench": "messung", "mess": "messung", "test": "tests"}.get(g, g)
        S = [s for s in S if s.group.lower().startswith(g) or s.group.split("-", 1)[-1].lower().startswith(g)]
    S.sort(key=lambda s: s.group)  # stabil: Gruppen-Praefix 0..5 ordnet die Phasen
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
    if step.skip_if:
        why = step.skip_if()
        if why:
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
    env.update(ctx.extra_env)
    env.update({k: v for k, v in step.env.items() if not k.startswith("_")})
    env.setdefault("CARGO_TERM_COLOR", "never")
    env.setdefault("RUST_BACKTRACE", "1")
    if jobs:
        env["CARGO_BUILD_JOBS"] = str(jobs)
    timeout = int(step.timeout * tfactor)
    cwd = step.cwd or ctx.root
    cwd.mkdir(parents=True, exist_ok=True)
    header = (f"$ {' '.join(step.cmd)}\n# cwd={cwd}\n# env+={ {k: v for k, v in step.env.items() if not k.startswith('_')} }\n"
              f"# wiederholungen={step.repeat}\n\n")
    usage = Usage()
    step.load_before = round(os.getloadavg()[0], 2)
    rc: Optional[int] = 0
    timed_out = False
    try:
        with open(step.log, "w") as fh:
            fh.write(header)
            fh.flush()
            for run in range(step.repeat):
                if step.repeat > 1:
                    fh.write(f"\n##### Lauf {run + 1}/{step.repeat}\n")
                fh.flush()
                start = fh.tell()
                step.t_run0 = time.time()
                rc, timed_out = run_logged(step.cmd, cwd, env, fh, timeout, usage)
                fh.flush()
                if step.post:
                    with open(step.log, "rb") as rf:
                        rf.seek(start)
                        chunk = rf.read().decode(errors="replace")
                    try:
                        step.post(step, chunk, ctx, outdir, run)
                    except Exception as e:  # Auswertung darf den Lauf nicht verlieren
                        step.detail += f" | Auswertung fehlgeschlagen: {type(e).__name__}: {e}"
                if timed_out or rc != 0:
                    break
    except FileNotFoundError as e:
        step.status, step.detail = "SKIPPED", f"Befehl nicht gefunden: {e}"
        step.seconds = round(time.time() - t0, 1)
        return
    step.seconds = round(time.time() - t0, 1)
    step.cpu_user, step.cpu_sys, step.rss_mb = round(usage.user, 1), round(usage.sys, 1), round(usage.rss_mb, 1)
    step.load_after = round(os.getloadavg()[0], 2)
    if timed_out:
        step.status, step.detail = "TIMEOUT", f">{timeout}s je Lauf (zaehlt als rot) " + step.detail
    else:
        step.rc = rc
        step.status = "PASS" if rc == 0 else "FAIL"
        if step.info and step.status == "PASS":
            step.status = "INFO"
    if step.force_fail and step.status in ("PASS", "INFO"):
        step.status = "FAIL"

    text = Path(step.log).read_text(errors="replace").split("\n\n", 1)[-1]
    if step.name.startswith("nextest__"):
        step.detail = (step.detail + " " + parse_nextest_summary(text)).strip()
    elif step.name.startswith("test"):
        step.detail = (step.detail + " " + parse_cargo_test_summary(text)).strip()
        if "could not compile" in text:
            step.detail += " | COMPILE-FEHLER"
            step.status = "FAIL"
        elif step.status == "PASS" and not re.search(r"test result: \w+\. [1-9]\d* passed", text):
            step.detail += " | KEINE TESTS AUSGEFUEHRT"
    elif step.name.startswith("clippy") or step.name == "panic_flaeche_lib":
        step.detail = (step.detail + " " + parse_clippy(text)).strip()
    elif step.name.startswith("bench__"):
        names = {r["benchmark"] for r in step.acc}
        step.detail = (step.detail + f" {len(names)} Benchmarks x {step.repeat} Lauf/Laeufe").strip()
    elif step.name.startswith("check_"):
        step.detail = (step.detail + f" errors={len(re.findall(r'^error', text, re.M))}").strip()
    if step.metrics.get("hang_kandidat"):
        step.detail += f" | HANG-KANDIDAT {step.metrics['hang_kandidat']}"
    # ----------------------------------------------------------------------------------------------
# Auswertungen (lesen die Datendateien; funktionieren auch nach --resume)
# ----------------------------------------------------------------------------------------------

def export_panic_surface(ctx: Ctx, steps: list[Step], outdir: Path) -> Optional[dict]:
    s = next((x for x in steps if x.name == "panic_flaeche_lib" and x.log and Path(x.log).exists()), None)
    if not s:
        return None
    data = parse_panic_surface(Path(s.log).read_text(errors="replace"), ctx.ws.dir_map())
    rows = []
    for crate, k in sorted(data.items(), key=lambda kv: -sum(kv[1].values())):
        r = [k.get(x, 0) for x in ("unwrap", "expect", "panic", "todo", "unimplemented", "index")]
        rows.append([crate] + r + [sum(r)])
    write_csv(outdir / "panic_flaeche.csv", ["crate", "unwrap", "expect", "panic", "todo", "unimplemented", "index", "summe"], rows)
    return data


def aggregate_bench(outdir: Path) -> list[dict]:
    raw = []
    for step, rows in get_data(outdir, "bench"):
        for r in rows:  # type: ignore[union-attr]
            raw.append(r)
    if not raw:
        return []
    write_csv(outdir / "bench_runs.csv",
              ["step", "benchmark", "lauf", "mean_ns", "median_ns", "std_ns", "ci_low_ns", "ci_high_ns", "tp_elems", "tp_bytes"],
              [[r["step"], r["benchmark"], r["run"], r["mean_ns"], r["median_ns"], r["std_ns"], r["ci_low_ns"],
                r["ci_high_ns"], r.get("tp_elems"), r.get("tp_bytes")] for r in sorted(raw, key=lambda x: (x["step"], x["benchmark"], x["run"]))])
    groups: dict[tuple[str, str], list[dict]] = defaultdict(list)
    for r in raw:
        groups[(r["step"], r["benchmark"])].append(r)
    agg = []
    for (step, name), rs in sorted(groups.items()):
        rs.sort(key=lambda x: x["run"])
        meds = [x["median_ns"] for x in rs]
        med = statistics.median(meds)
        cv = (statistics.stdev(meds) / statistics.mean(meds) * 100) if len(meds) > 1 and statistics.mean(meds) > 0 else None
        last = rs[-1]
        tp_e = (last["tp_elems"] / (med * 1e-9)) if last.get("tp_elems") and med > 0 else None
        tp_b = (last["tp_bytes"] / (med * 1e-9)) if last.get("tp_bytes") and med > 0 else None
        crate = step.split("__")[1] if step.count("__") >= 1 else ""
        agg.append(dict(crate=crate, step=step, benchmark=name, runs=len(rs), median_ns=med, min_ns=min(meds), max_ns=max(meds),
                        cv_pct=cv, mean_ns=statistics.mean(x["mean_ns"] for x in rs), std_ns=last["std_ns"],
                        ci_low_ns=last["ci_low_ns"], ci_high_ns=last["ci_high_ns"], tp_elems_s=tp_e, tp_bytes_s=tp_b))
    write_csv(outdir / "bench_results.csv",
              ["crate", "step", "benchmark", "laeufe", "median_ns", "min_ns", "max_ns", "cv_pct", "mean_ns", "std_ns_letzter_lauf",
               "ci_low_ns", "ci_high_ns", "durchsatz_elem_s", "durchsatz_byte_s"],
              [[a["crate"], a["step"], a["benchmark"], a["runs"], f"{a['median_ns']:.3f}", f"{a['min_ns']:.3f}", f"{a['max_ns']:.3f}",
                "" if a["cv_pct"] is None else f"{a['cv_pct']:.2f}", f"{a['mean_ns']:.3f}", f"{a['std_ns']:.3f}",
                f"{a['ci_low_ns']:.3f}", f"{a['ci_high_ns']:.3f}",
                "" if a["tp_elems_s"] is None else f"{a['tp_elems_s']:.1f}", "" if a["tp_bytes_s"] is None else f"{a['tp_bytes_s']:.1f}"]
               for a in agg])
    return agg


def export_tests(outdir: Path) -> dict:
    rows, bins = [], []
    for step, p in get_data(outdir, "tests"):
        for b, t, s, e in p["rows"]:  # type: ignore[index]
            rows.append([step, p["crate"], b, t, s, "" if e is None else e])  # type: ignore[index]
    for step, p in get_data(outdir, "testbins"):
        for b, ok, bad, ign, secs in p["rows"]:  # type: ignore[index]
            bins.append([step, p["crate"], b, ok, bad, ign, secs])  # type: ignore[index]
    if rows:
        write_csv(outdir / "tests.csv", ["schritt", "crate", "binary", "test", "status", "sekunden"], rows)
    if bins:
        write_csv(outdir / "test_binaries.csv", ["schritt", "crate", "binary", "ok", "rot", "ignoriert", "sekunden"], bins)
    flake = []
    for _, r in get_data(outdir, "flake"):
        flake.extend(r)  # type: ignore[arg-type]
    if flake:
        write_csv(outdir / "flake_report.csv", ["crate", "binary", "test", "ergebnisse_je_lauf", "art"], flake)
    return {"tests": rows, "bins": bins, "flake": flake}


def export_scale(outdir: Path) -> list[list]:
    rows = []
    for step, iv in get_data(outdir, "scale"):
        for x in iv:  # type: ignore[union-attr]
            rows.append([step, x["docs_inserted"], round(x["insert_rate_docs_per_sec"], 1), x["search_p50_ns"], x["search_p95_ns"],
                         x["search_p99_ns"], round(x["rss_bytes"] / 1048576, 1)])
    if rows:
        write_csv(outdir / "scale_curve.csv", ["schritt", "dokumente", "einfuegerate_docs_s", "suche_p50_ns", "suche_p95_ns",
                                               "suche_p99_ns", "rss_mb"], rows)
    return rows


def export_metrics(steps: list[Step], outdir: Path) -> list[list]:
    rows = [[s.name, k, v] for s in steps for k, v in sorted(s.metrics.items())]
    if rows:
        write_csv(outdir / "metrics.csv", ["schritt", "schluessel", "wert"], rows)
    return rows


def maturity_table(ctx: Ctx, steps: list[Step]) -> list[str]:
    by = {s.name: s for s in steps}
    lines = ["", "## Reife-Behauptung vs. Beleg (capabilities.toml gegen diesen Lauf)", "",
             "| Crate | behauptet | Tests | Clippy | Urteil |", "|---|---|---|---|---|"]
    for name, v in sorted(ctx.caps.items()):
        claimed = v.get("maturity", "?")
        t, c = by.get(f"test__{name}"), by.get(f"clippy__{name}")
        if not t and not c:
            continue
        ts, cs = (t.status if t else "-"), (c.status if c else "-")
        if claimed in ("stable", "beta") and (ts not in ("PASS", "-") or cs not in ("PASS", "-")):
            verdict = "**Behauptung ohne Beleg**"
        else:
            verdict = "ok" if ts == "PASS" and cs == "PASS" else "-"
        lines.append(f"| `{name}` | {claimed} | {ts} | {cs} | {verdict} |")
    return lines


def table(header: list[str], rows: list[list], aligns: Optional[list[str]] = None) -> list[str]:
    al = aligns or ["---"] * len(header)
    out = ["| " + " | ".join(header) + " |", "|" + "|".join(al) + "|"]
    out += ["| " + " | ".join(md_cell(c) for c in r) + " |" for r in rows]
    return out


def measurement_sections(ctx: Ctx, steps: list[Step], outdir: Path, agg: list[dict], scale_rows: list[list]) -> list[str]:
    L: list[str] = []
    args = ctx.args
    L += ["", "## Messung", "",
          f"- Profil `{args.profil}`, Wiederholungen je Criterion-Bench: "
          f"{args.repeat if args.repeat is not None else ctx.profile['repeat']}, "
          f"Criterion-Argumente: `{' '.join(ctx.profile['crit']) or '(Standard)'}`",
          f"- native CPU-Optimierung: {args.native}; Overflow-Checks aus: {args.no_overflow_checks}; "
          f"zusaetzliche Umgebung: {ctx.extra_env or '-'}"]
    hyg = get_data(outdir, "hygiene")
    if hyg and hyg[0][1].get("warnungen"):  # type: ignore[union-attr]
        L += ["", "**Warnungen zur Messumgebung:**"] + [f"- {w}" for w in hyg[0][1]["warnungen"]]  # type: ignore[index]

    if agg:
        noisy = [a for a in agg if a["cv_pct"] is not None and a["cv_pct"] > 10]
        multi = any(a["runs"] > 1 for a in agg)
        L += ["", f"### Criterion: {len(agg)} Benchmarks" + (f", {len(noisy)} mit Streuung > 10 % ueber die Laeufe" if multi else ""), "",
              "Die 25 langsamsten (Median je Lauf, dann Median ueber Laeufe). Vollstaendig: `bench_results.csv`.", ""]
        top = sorted(agg, key=lambda a: -a["median_ns"])[:25]
        L += table(["Crate", "Benchmark", "Median", "min..max", "CV %", "Durchsatz"],
                   [[short(a["crate"]), a["benchmark"], fmt_ns(a["median_ns"]),
                     f"{fmt_ns(a['min_ns'])}..{fmt_ns(a['max_ns'])}" if a["runs"] > 1 else "-",
                     "-" if a["cv_pct"] is None else f"{a['cv_pct']:.1f}",
                     (f"{a['tp_elems_s']:.3g} Elem/s" if a["tp_elems_s"] else (f"{a['tp_bytes_s'] / 1048576:.3g} MiB/s" if a["tp_bytes_s"] else "-"))]
                    for a in top], ["---", "---", "---:", "---:", "---:", "---:"])
        if noisy:
            L += ["", "Unruhigste Benchmarks (CV ueber Laeufe): Ergebnisse unter ca. diesem Rauschen sind nicht belastbar.", ""]
            L += table(["Crate", "Benchmark", "CV %", "min..max"],
                       [[short(a["crate"]), a["benchmark"], f"{a['cv_pct']:.1f}", f"{fmt_ns(a['min_ns'])}..{fmt_ns(a['max_ns'])}"]
                        for a in sorted(noisy, key=lambda a: -a["cv_pct"])[:15]], ["---", "---", "---:", "---:"])

    harness = [s for s in steps if s.name.startswith("harness_") and s.metrics]
    if harness:
        L += ["", "### Harness-Kennzahlen", ""]
        for s in harness:
            kv = [(k, v) for k, v in s.metrics.items() if not k.startswith("bin_mb_")][:24]
            if kv:
                L += [f"**`{s.name}`** ({s.status})", ""] + table(["Kennzahl", "Wert"], [[k, v] for k, v in kv], ["---", "---:"]) + [""]
    if scale_rows:
        L += ["", "### Skalierung (scale-100k)", ""]
        L += table(["Dokumente", "Einfuegerate Docs/s", "Suche p50", "Suche p95", "Suche p99", "RSS MiB"],
                   [[r[1], r[2], fmt_ns(r[3]), fmt_ns(r[4]), fmt_ns(r[5]), r[6]] for r in scale_rows],
                   ["---:", "---:", "---:", "---:", "---:", "---:"])

    timed = [s for s in steps if s.status in ("PASS", "INFO", "FAIL") and (s.cpu_user + s.cpu_sys) > 0]
    if timed:
        L += ["", "### Ressourcenverbrauch je Schritt (Top 12 nach CPU-Zeit)", ""]
        L += table(["Schritt", "Wand s", "CPU user s", "CPU sys s", "Spitzen-RSS MiB", "Last vorher/nachher"],
                   [[f"`{s.name}`", s.seconds, s.cpu_user, s.cpu_sys, s.rss_mb, f"{s.load_before}/{s.load_after}"]
                    for s in sorted(timed, key=lambda x: -(x.cpu_user + x.cpu_sys))[:12]],
                   ["---", "---:", "---:", "---:", "---:", "---:"])
    return L


def write_reports(ctx: Ctx, steps: list[Step], outdir: Path, meta: dict, final: bool = False) -> None:
    rows = [{"nr": i, "gruppe": s.group, "schritt": s.name, "status": s.status, "rc": s.rc, "sekunden": s.seconds,
             "cpu_user_s": s.cpu_user, "cpu_sys_s": s.cpu_sys, "rss_mb": s.rss_mb, "load_vorher": s.load_before,
             "load_nachher": s.load_after, "detail": s.detail, "log": os.path.relpath(s.log, outdir) if s.log else "",
             "befehl": " ".join(s.cmd), "hinweis": s.note, "metrics": s.metrics} for i, s in enumerate(steps)]
    (outdir / "results.json").write_text(json.dumps({"meta": meta, "steps": rows}, indent=2, ensure_ascii=False, default=str))
    if rows:
        csv_rows = [{k: v for k, v in r.items() if k != "metrics"} for r in rows]
        with open(outdir / "results.csv", "w", newline="") as fh:
            w = csv.DictWriter(fh, fieldnames=list(csv_rows[0].keys()))
            w.writeheader()
            w.writerows(csv_rows)

    counts: dict[str, int] = defaultdict(int)
    for s in steps:
        counts[s.status] += 1
    order = {"FAIL": 0, "TIMEOUT": 1, "SKIPPED": 2, "INFO": 3, "PASS": 4, "PENDING": 5}
    pre = next((s for s in steps if s.name == PREFLIGHT), None)
    L = ["# Contextra Audit und Messung", "",
         f"- Commit: `{meta['commit']}` ({meta['commit_date']}), Branch `{meta.get('branch', '')}`, "
         f"hinter origin: {meta.get('behind', '?')}, ungespeicherte Aenderungen: {meta.get('dirty')}",
         f"- Start: {meta['start']}  Dauer: {meta['duration_min']} min  Profil: `{ctx.args.profil}`",
         f"- Toolchain-Pin: `{ctx.pin}`; Preflight: **{pre.status if pre else 'nicht gelaufen'}** ({pre.detail if pre else ''})",
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
            L.append(f"| `{s.name}` | {s.status} | {s.seconds} | {md_cell(s.detail)} | `{os.path.relpath(s.log, outdir) if s.log else ''}` |")
    if final:
        agg = aggregate_bench(outdir)
        scale_rows = export_scale(outdir)
        export_metrics(steps, outdir)
        tdata = export_tests(outdir)
        L += measurement_sections(ctx, steps, outdir, agg, scale_rows) if any(s.group == "4-messung" for s in steps) else []
        if tdata["bins"]:
            slow = sorted(tdata["bins"], key=lambda r: -float(r[6]))[:15]
            L += ["", "## Tests", "", f"{len(tdata['tests'])} Einzeltests erfasst (`tests.csv`). Langsamste Test-Binaries:", ""]
            L += table(["Crate", "Binary", "ok", "rot", "ignoriert", "Sek"], [[short(r[1]), r[2], r[3], r[4], r[5], r[6]] for r in slow],
                       ["---", "---", "---:", "---:", "---:", "---:"])
            ign = sum(r[5] for r in tdata["bins"])
            L += ["", f"Ignorierte Tests insgesamt: {ign}."]
        if tdata["flake"]:
            L += ["", "## Flake-Scan", ""] + table(["Crate", "Test / Hang", "Ergebnisse"],
                  [[short(r[0]), r[2] or r[3], r[3] if r[2] else ""] for r in tdata["flake"][:40]])
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
        L += maturity_table(ctx, steps)
    L += ["", "## Alle Schritte", "", "| Gruppe | Schritt | Status | Sek | CPU s | RSS MiB | Detail |", "|---|---|---|---:|---:|---:|---|"]
    for s in steps:
        L.append(f"| {s.group} | `{s.name}` | {s.status} | {s.seconds} | {round(s.cpu_user + s.cpu_sys, 1)} | {s.rss_mb} | {md_cell(s.detail)} |")
    (outdir / "summary.md").write_text("\n".join(L) + "\n")


def collect_env(root: Path, outdir: Path, ctx: Ctx) -> dict:
    root_user = hasattr(os, "geteuid") and os.geteuid() == 0

    def sh(cmd: str) -> str:
        try:
            return subprocess.run(cmd, shell=True, cwd=root, capture_output=True, text=True, timeout=60).stdout.strip()
        except (subprocess.SubprocessError, OSError) as e:
            return f"n/a ({e})"
    meta = {
        "commit": sh("git rev-parse --short HEAD") or "unknown",
        "commit_date": sh("git log -1 --format=%ci"),
        "branch": sh("git rev-parse --abbrev-ref HEAD"),
        "behind": sh("git rev-list --count HEAD..@{u} 2>/dev/null") or "?",
        "dirty": bool(sh("git status --porcelain")),
        "host": f"{platform.platform()} | {os.cpu_count()} CPUs | " + sh("grep MemTotal /proc/meminfo 2>/dev/null"),
        "user": sh("id -un") + (" (ROOT: Dateirechte-Tests nicht aussagekraeftig!)" if root_user else ""),
        "rustc": sh("rustc -V"),
        "cpu_modell": sh("lscpu 2>/dev/null | grep 'Model name' | head -1"),
        "argv": " ".join(sys.argv),
    }
    (outdir / "env.txt").write_text("\n".join(f"{k}: {v}" for k, v in meta.items()) + "\n\n"
                                    + sh("rustup show active-toolchain 2>&1") + "\n\n" + sh("uname -a") + "\n\n"
                                    + f"profil={ctx.args.profil} native={ctx.args.native} no_overflow_checks={ctx.args.no_overflow_checks}\n"
                                    + f"extra_env={ctx.extra_env}\n" + f"Cargo.lock sha256: {sh('sha256sum Cargo.lock | cut -d' ' -f1')}\n")
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
        if name.startswith(("test__", "nextest__")):
            ca, cb = counts(sa[name]), counts(sb[name])
            if ca != cb:
                out.append(f"| `{name}` | {ca} | {cb} |")

    def bench(d: Path) -> dict:
        p = d / "bench_results.csv"
        if not p.exists():
            return {}
        with open(p) as fh:
            return {f"{r['crate']}/{r['benchmark']}": (float(r["median_ns"]), float(r["cv_pct"]) if r["cv_pct"] else 0.0)
                    for r in csv.DictReader(fh)}
    ba, bb = bench(a), bench(b)
    if ba and bb:
        out += ["", "## Criterion (Median, Abweichung > 5 % und ueber dem Rauschen beider Laeufe)", "",
                "| Benchmark | A | B | Delta | Rauschen (max CV) |", "|---|---:|---:|---:|---:|"]
        for k in sorted(set(ba) & set(bb)):
            if ba[k][0] > 0:
                d = (bb[k][0] - ba[k][0]) / ba[k][0] * 100
                noise = max(ba[k][1], bb[k][1])
                if abs(d) > 5 and abs(d) > noise:
                    out.append(f"| `{k}` | {fmt_ns(ba[k][0])} | {fmt_ns(bb[k][0])} | {d:+.1f} % | {noise:.1f} % |")

    def metr(d: Path) -> dict:
        p = d / "metrics.csv"
        if not p.exists():
            return {}
        with open(p) as fh:
            return {f"{r['schritt']}:{r['schluessel']}": r["wert"] for r in csv.DictReader(fh)}
    ma_, mb_ = metr(a), metr(b)
    if ma_ and mb_:
        out += ["", "## Harness-Kennzahlen (Abweichung > 5 %)", "", "| Kennzahl | A | B | Delta |", "|---|---:|---:|---:|"]
        for k in sorted(set(ma_) & set(mb_)):
            try:
                x, y = float(ma_[k]), float(mb_[k])
            except ValueError:
                continue
            if x != 0 and abs((y - x) / x * 100) > 5:
                out.append(f"| `{k}` | {x:g} | {y:g} | {(y - x) / x * 100:+.1f} % |")
    text = "\n".join(out) + "\n"
    (b / "vergleich.md").write_text(text)
    print(text)
    return 0


# ----------------------------------------------------------------------------------------------
# main
# ----------------------------------------------------------------------------------------------

def main() -> int:
    ap = argparse.ArgumentParser(description="Contextra: messen, testen, Befunde pruefen, Ergebnisse speichern")
    ap.add_argument("--profil", choices=sorted(PROFILES), default="standard", help="Messtiefe (smoke|standard|max)")
    ap.add_argument("--messen", action="store_true", help="nur Messung (Gruppen 0 und 4)")
    ap.add_argument("--quick", action="store_true", help="ohne Messung, Fuzz, Mutation")
    ap.add_argument("--all", action="store_true", help="zusaetzlich Fuzz, Mutation, Loom, Miri, ASan, Release-Tests, Coverage")
    ap.add_argument("--only", help="kommagetrennte Crates (Kurzname), z. B. store,vector")
    ap.add_argument("--gruppe", help="nur Gruppe: basis, befunde, statisch, tests, messung, tief")
    ap.add_argument("--list", action="store_true", help="Plan anzeigen, nichts ausfuehren")
    ap.add_argument("--repeat", type=int, help="Wiederholungen je Criterion-Bench (Standard je Profil)")
    ap.add_argument("--flake", type=int, help="Testwiederholungen je Crate fuer den Flake-Scan (Standard je Profil, 0 = aus)")
    ap.add_argument("--nextest", action="store_true", help="zusaetzlich cargo-nextest je Crate (je Test eigener Prozess)")
    ap.add_argument("--test-json", action="store_true",
                    help="Einzeltest-Laufzeiten ueber libtest-JSON (RUSTC_BOOTSTRAP=1, eigenes Target-Verzeichnis -> voller Neubau)")
    ap.add_argument("--native", action="store_true", help="Bench-/Harness-Builds mit -C target-cpu=native")
    ap.add_argument("--no-overflow-checks", action="store_true", help="overflow-checks in release/bench abschalten (A/B-Vergleich)")
    ap.add_argument("--env", action="append", metavar="KEY=VALUE", help="zusaetzliche Umgebungsvariable fuer alle Schritte (mehrfach)")
    ap.add_argument("--baseline", help="Criterion: gegen gespeicherte Baseline vergleichen")
    ap.add_argument("--save-baseline", help="Criterion: Ergebnis als Baseline speichern")
    ap.add_argument("--criterion-home", help="persistentes CRITERION_HOME (fuer Baselines ueber mehrere Laeufe)")
    ap.add_argument("--target-dir", help="Cargo-Target-Verzeichnis (Standard: CARGO_TARGET_DIR oder <root>/target)")
    ap.add_argument("--fail-fast", action="store_true")
    ap.add_argument("--jobs", type=int, help="CARGO_BUILD_JOBS begrenzen")
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
    ctx = Ctx(root, args)
    if ctx.ws.problems:
        print("Workspace-Hinweise:", *ctx.ws.problems, sep="\n  ")

    start = now_utc()
    t_start = time.time()
    commit = subprocess.run("git rev-parse --short HEAD", shell=True, cwd=root, capture_output=True, text=True).stdout.strip() or "nogit"
    outdir = Path(args.resume).resolve() if args.resume else root / "evidence" / f"audit-{start.strftime('%Y%m%dT%H%M%SZ')}-{commit}"
    ctx.outdir = outdir
    previous: dict[str, dict] = {}
    if args.resume and (outdir / "results.json").exists():
        previous = {s["schritt"]: s for s in json.loads((outdir / "results.json").read_text())["steps"]}

    steps = build_plan(ctx, args, outdir)
    if args.list:
        for i, s in enumerate(steps):
            rep = f" x{s.repeat}" if s.repeat > 1 else ""
            print(f"{i:03d} [{s.group}] {s.name:58s}{rep:4s} {' '.join(s.cmd) if s.cmd else '(Python-Pruefung)'}")
        print(f"\n{len(steps)} Schritte, Profil {args.profil}")
        return 0

    (outdir / "logs").mkdir(parents=True, exist_ok=True)
    meta = collect_env(root, outdir, ctx)
    meta.update({"start": start.isoformat(timespec="seconds"), "t_start_epoch": t_start, "duration_min": 0.0})
    if meta["behind"] not in ("", "0", "?"):
        print(f"WARNUNG: HEAD liegt {meta['behind']} Commits hinter origin.")
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
                s.metrics = prev.get("metrics") or {}
                s.cpu_user, s.cpu_sys, s.rss_mb = prev.get("cpu_user_s", 0.0), prev.get("cpu_sys_s", 0.0), prev.get("rss_mb", 0.0)
                print(f"[{i + 1:03d}/{len(steps)}] {s.group:10s} {s.name} ... uebersprungen ({s.status}, aus frueherem Lauf)")
                continue
            print(f"[{i + 1:03d}/{len(steps)}] {s.group:10s} {s.name} ...", end=" ", flush=True)
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

    meta["duration_min"] = round((time.time() - t_start) / 60, 1)
    write_reports(ctx, steps, outdir, meta, final=True)
    red = [s for s in steps if s.status in ("FAIL", "TIMEOUT")]
    print(f"\nFertig in {meta['duration_min']} min. FAIL/TIMEOUT: {len(red)}  -> {outdir}/summary.md")
    return 1 if red else 0


if __name__ == "__main__":
    sys.exit(main())
