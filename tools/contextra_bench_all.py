#!/usr/bin/env python3
"""
contextra_bench_all.py - autonomes Setup, Benchmarking und Belastungstest fuer Contextra.

Ein Aufruf richtet alles ein (Rust-Toolchain laut rust-toolchain.toml, C-Compiler, Abhaengigkeiten),
klont das Repo falls noetig, baut, misst, belastet und speichert alle Ergebnisse maschinenlesbar.

  python3 contextra_bench_all.py                       # alles, Profil standard (Stunden)
  python3 contextra_bench_all.py --profil smoke        # Funktionstest der Pipeline (Minuten bis ~1 h)
  python3 contextra_bench_all.py --profil max --deep   # maximale Tiefe inkl. Loom und Fuzzing
  python3 contextra_bench_all.py --list                # Plan zeigen, nichts ausfuehren
  python3 contextra_bench_all.py --gruppen setup,bench # nur bestimmte Gruppen
  python3 contextra_bench_all.py --only store,vector   # nur bestimmte Crates (Kurzname)
  python3 contextra_bench_all.py --resume evidence/bench-...   # abgeschlossene Schritte uebernehmen
  python3 contextra_bench_all.py --vergleich evidence/bench-A evidence/bench-B

Gruppen: setup, build, bench, harness, stress, deep   (Reihenfolge = Ausfuehrung)
  setup    Rust via rustup, Pin-Toolchain, C-Compiler, cargo fetch, Messhygiene
  build    jedes Bench-Ziel einzeln bauen (Zeit, CPU, Spitzen-RSS); Fehler bleiben isoliert
  bench    jedes Criterion-Ziel `--repeat` mal mit eigenem CRITERION_HOME; Median, CV, Durchsatz
  harness  contextra-bench (synthetisch, --ann) und scale-100k (Einfuegerate, Latenz-Perzentile, RSS)
  stress   Tests im Release-Modus je Crate, abwechselnd parallel/--test-threads=1/2xCPU, plus
           --include-ignored; Test-Ziele, die nicht kompilieren, werden isoliert; Flake- und Hang-Erkennung
  deep     (nur mit --deep) Loom-Tests und cargo-fuzz (nightly)

Ergebnisse: evidence/bench-<UTC>-<commit>/
  summary.md  results.json/.csv  bench_runs.csv  bench_results.csv  scale_curve.csv  metrics.csv
  flake_report.csv  tests.csv  env.txt  logs/  criterion/  harness/

Grenzen (ehrlich): Der Quellcode wird nie veraendert. Es gibt keinen Kill-9-Durability-Zyklus, weil das
Repo dafuer kein Binary bereitstellt; die vorhandenen Crash-Tests laufen im Gruppen-Schritt `stress`.
Nur Python-Standardbibliothek (3.9+), Linux/macOS.
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
import statistics
import subprocess
import sys
import time
import traceback
import urllib.request
from collections import defaultdict
from pathlib import Path
from typing import Callable, Optional

REPO_URL = "https://github.com/tfufuz1/contextra"
GROUPS = ["setup", "build", "bench", "harness", "stress", "deep"]

# crit = Criterion-Argumente; stress_runs = Anzahl Testlaeufe je Crate (zyklisch par/t1/t2n)
PROFILES = {
    "smoke": dict(crit=["--warm-up-time", "0.3", "--measurement-time", "0.5", "--sample-size", "10"], repeat=1,
                  scale=(10_000, 5_000), stress_runs=1, ignored=False, fuzz=30, bench_to=900),
    "standard": dict(crit=["--warm-up-time", "1", "--measurement-time", "3", "--sample-size", "50"], repeat=3,
                     scale=(100_000, 10_000), stress_runs=3, ignored=True, fuzz=120, bench_to=3600),
    "max": dict(crit=["--warm-up-time", "3", "--measurement-time", "10", "--sample-size", "100"], repeat=5,
                scale=(1_000_000, 100_000), stress_runs=6, ignored=True, fuzz=600, bench_to=7200),
}
STRESS_FEATURES = ("fault-injection", "test-hooks", "inspect")  # werden aktiviert, wenn das Crate sie definiert
MIN_FREE_GB = 8.0
TEST_TIMEOUT = 3600


# ---------------------------------------------------------------------------------------------
# Hilfen
# ---------------------------------------------------------------------------------------------

def short(n: str) -> str:
    return n.removeprefix("contextra-") or n


def sh(cmd: str, cwd: Optional[Path] = None, timeout: int = 30) -> str:
    try:
        return subprocess.run(cmd, shell=True, cwd=cwd, capture_output=True, text=True, timeout=timeout).stdout.strip()
    except (subprocess.SubprocessError, OSError):
        return ""


def read(p: Path) -> str:
    try:
        return p.read_text(errors="replace")
    except OSError:
        return ""


def write_csv(path: Path, header: list, rows: list) -> None:
    with open(path, "w", newline="") as fh:
        w = csv.writer(fh)
        w.writerow(header)
        w.writerows(rows)


def fmt_ns(ns: float) -> str:
    for unit, div in (("s", 1e9), ("ms", 1e6), ("us", 1e3)):
        if abs(ns) >= div:
            return f"{ns / div:.3g} {unit}"
    return f"{ns:.3g} ns"


def cell(s) -> str:
    return str(s).replace("|", "\\|").replace("\n", " ")


def table(header: list, rows: list, align: Optional[list] = None) -> list:
    out = ["| " + " | ".join(header) + " |", "|" + "|".join(align or ["---"] * len(header)) + "|"]
    return out + ["| " + " | ".join(cell(c) for c in r) + " |" for r in rows]


def put_data(outdir: Path, kind: str, step: str, payload) -> None:
    d = outdir / "data"
    d.mkdir(parents=True, exist_ok=True)
    (d / f"{kind}__{re.sub(r'[^A-Za-z0-9_.-]', '_', step)}.json").write_text(json.dumps(payload))


def get_data(outdir: Path, kind: str) -> list:
    out = []
    for f in sorted((outdir / "data").glob(f"{kind}__*.json")):
        try:
            out.append((f.stem.split("__", 1)[1], json.loads(f.read_text())))
        except (OSError, ValueError):
            pass
    return out


def flatten(obj, prefix="", out=None, limit=300):
    out = {} if out is None else out
    if len(out) >= limit:
        return out
    if isinstance(obj, bool):
        out[prefix] = int(obj)
    elif isinstance(obj, (int, float)):
        out[prefix] = obj
    elif isinstance(obj, dict):
        for k, v in obj.items():
            flatten(v, f"{prefix}.{k}" if prefix else str(k), out, limit)
    elif isinstance(obj, list) and len(obj) <= 32:
        for i, v in enumerate(obj):
            flatten(v, f"{prefix}[{i}]", out, limit)
    return out


# ---------------------------------------------------------------------------------------------
# Prozesse: Prozessgruppe beenden bei Timeout, CPU und Spitzen-RSS per wait4
# ---------------------------------------------------------------------------------------------

class Res:
    def __init__(self):
        self.rc: Optional[int] = None
        self.timeout = False
        self.wall = self.user = self.sys = self.rss_mb = 0.0


def kill_group(p: subprocess.Popen) -> None:
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


def run_proc(cmd: list, cwd: Path, env: dict, fh, timeout: int) -> Res:
    r = Res()
    t0 = time.monotonic()
    try:
        p = subprocess.Popen(cmd, cwd=cwd, env=env, stdout=fh, stderr=subprocess.STDOUT, start_new_session=True)
    except FileNotFoundError as e:
        fh.write(f"Befehl nicht gefunden: {e}\n")
        r.rc = 127
        return r
    try:
        while True:
            pid, status, ru = os.wait4(p.pid, os.WNOHANG)
            if pid == p.pid:
                r.rc = os.waitstatus_to_exitcode(status)
                r.user, r.sys = ru.ru_utime, ru.ru_stime
                r.rss_mb = ru.ru_maxrss / (1048576 if sys.platform == "darwin" else 1024)
                break
            if time.monotonic() - t0 > timeout:
                kill_group(p)
                r.timeout = True
                break
            time.sleep(0.1)
    except KeyboardInterrupt:
        kill_group(p)
        raise
    r.wall = time.monotonic() - t0
    return r


def capture(cmd: list, cwd: Path, env: dict, timeout: int):
    try:
        p = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, text=True, timeout=timeout)
        return p.returncode, p.stdout, p.stderr
    except subprocess.TimeoutExpired:
        return None, "", "Zeitlimit"
    except OSError as e:
        return 127, "", str(e)


# ---------------------------------------------------------------------------------------------
# Kontext und Schritte
# ---------------------------------------------------------------------------------------------

class Ctx:
    def __init__(self, root: Path, outdir: Path, args):
        self.root, self.outdir, self.args = root, outdir, args
        self.prof = PROFILES[args.profil]
        self.ncpu = os.cpu_count() or 1
        self.pkgs: dict = {}      # name -> {dir, features, targets}
        self.target_dir = Path(args.target_dir).resolve() if args.target_dir else root / "target"
        self.cargo_ok: Optional[bool] = None
        self.tuned: list = []
        self.env = os.environ.copy()
        self.env.update({"CARGO_TERM_COLOR": "never", "RUST_BACKTRACE": "1", "CARGO_TARGET_DIR": str(self.target_dir)})
        # Debug-Info aus: spart Platz und Linkzeit (frueherer Lauf scheiterte an voller Platte); Messwerte bleiben gleich
        for prof in ("RELEASE", "BENCH", "TEST", "DEV"):
            self.env.setdefault(f"CARGO_PROFILE_{prof}_DEBUG", "false")
        rf = self.env.get("RUSTFLAGS", "")
        if args.native:
            rf = (rf + " -C target-cpu=native").strip()
        if rf:
            self.env["RUSTFLAGS"] = rf
        if args.no_overflow_checks:
            self.env["CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS"] = "false"
            self.env["CARGO_PROFILE_BENCH_OVERFLOW_CHECKS"] = "false"
        for kv in args.env or []:
            k, _, v = kv.partition("=")
            if not _:
                raise SystemExit(f"--env erwartet KEY=VALUE: {kv!r}")
            self.env[k] = v
        if args.jobs:
            self.env["CARGO_BUILD_JOBS"] = str(args.jobs)


class Step:
    def __init__(self, group: str, name: str, cmd: Optional[list] = None, *, py: Optional[Callable] = None,
                 timeout: int = 3600, env: Optional[dict] = None, cwd: Optional[Path] = None, repeat: int = 1,
                 post: Optional[Callable] = None, dep: str = "", cargo: bool = True, info: bool = False,
                 crit_base: Optional[Path] = None, note: str = ""):
        self.group, self.name, self.cmd, self.py = group, name, cmd or [], py
        self.timeout, self.env, self.cwd, self.repeat = timeout, env or {}, cwd, max(1, repeat)
        self.post, self.dep, self.cargo, self.info = post, dep, cargo, info
        self.crit_base, self.note = crit_base, note
        self.status, self.rc, self.seconds, self.detail, self.log = "PENDING", None, 0.0, "", ""
        self.cpu_user = self.cpu_sys = self.rss_mb = 0.0
        self.metrics: dict = {}
        self.acc: list = []
        self.force_fail = False


# ---------------------------------------------------------------------------------------------
# Parser
# ---------------------------------------------------------------------------------------------

TEST_LINE = re.compile(r"^test (\S+) \.\.\. (ok|FAILED|ignored(?:, .*)?)\s*$")
TEST_OPEN = re.compile(r"^test (\S+) \.\.\. $")
RUNNING = re.compile(r"^\s+Running (?:unittests )?(.+?) \(([^)]+)\)\s*$")
RESULT = re.compile(r"^test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored;.*?finished in ([\d.]+)s")


def parse_tests(text: str):
    tests, bins, cur, open_ = [], [], "?", None
    for line in text.splitlines():
        m = RUNNING.match(line)
        if m:
            cur, open_ = m.group(1), None
            continue
        m = TEST_LINE.match(line)
        if m:
            tests.append((cur, m.group(1), "ignored" if m.group(2).startswith("ignored") else m.group(2)))
            open_ = None
            continue
        m = TEST_OPEN.match(line)
        if m:
            open_ = (cur, m.group(1))
            continue
        m = RESULT.match(line)
        if m:
            bins.append((cur, int(m.group(1)), int(m.group(2)), int(m.group(3)), float(m.group(4))))
            open_ = None
    return tests, bins, open_


def first_errors(text: str, n: int = 3) -> str:
    errs = re.findall(r"^error(?:\[\w+\])?: .+$", text, re.M)
    return " | ".join(errs[:n])


def parse_kv_lines(text: str) -> dict:
    """'  Max Recall@10    : 0.9870' und '... : 12.3 ms' -> {name: zahl}."""
    out = {}
    for m in re.finditer(r"^\s*([A-Za-z][^:\n=]{1,40}?)\s*:\s*(-?\d+(?:\.\d+)?)\s*(ms|ns|us|s)?\s*$", text, re.M):
        key = re.sub(r"[^A-Za-z0-9]+", "_", m.group(1).replace("@", " at ")).strip("_").lower()
        out[key + (f"_{m.group(3)}" if m.group(3) else "")] = float(m.group(2))
    return out


def criterion_collect(home: Path, target: str) -> list:
    rows = []
    for est in home.rglob("new/estimates.json"):
        try:
            d = json.loads(est.read_text())
            name = "/".join(est.relative_to(home).parts[:-2])
            row = dict(benchmark=name, target=target, mean_ns=d["mean"]["point_estimate"],
                       median_ns=d["median"]["point_estimate"], std_ns=d["std_dev"]["point_estimate"],
                       ci_low_ns=d["mean"]["confidence_interval"]["lower_bound"],
                       ci_high_ns=d["mean"]["confidence_interval"]["upper_bound"], tp_elems=None, tp_bytes=None)
            bm = est.parent / "benchmark.json"
            if bm.exists():
                tp = json.loads(bm.read_text()).get("throughput") or {}
                row["tp_elems"], row["tp_bytes"] = tp.get("Elements"), tp.get("Bytes")
            rows.append(row)
        except (OSError, KeyError, ValueError):
            continue
    return rows


# ---------------------------------------------------------------------------------------------
# Setup-Schritte
# ---------------------------------------------------------------------------------------------

def add_cargo_to_path(ctx: Ctx) -> None:
    cb = str(Path.home() / ".cargo" / "bin")
    if cb not in ctx.env["PATH"].split(os.pathsep):
        ctx.env["PATH"] = cb + os.pathsep + ctx.env["PATH"]
        os.environ["PATH"] = ctx.env["PATH"]


def toolchain_pin(root: Path) -> str:
    m = re.search(r'channel\s*=\s*"([^"]+)"', read(root / "rust-toolchain.toml"))
    return m.group(1) if m else "stable"


def privileged(cmd: list) -> Optional[list]:
    if hasattr(os, "geteuid") and os.geteuid() == 0:
        return cmd
    if shutil.which("sudo") and subprocess.run(["sudo", "-n", "true"], capture_output=True).returncode == 0:
        return ["sudo", "-n"] + cmd
    return None


def step_system_deps(ctx: Ctx, step: Step):
    """C-Compiler (Crate `cc` ist im Lockfile), git, curl."""
    missing = [t for t, alts in (("cc", ("cc", "gcc", "clang")), ("git", ("git",)), ("curl", ("curl",)),
                                 ("make", ("make",))) if not any(shutil.which(a) for a in alts)]
    if not missing:
        return "PASS", "cc, git, curl, make vorhanden"
    if ctx.args.no_system_install:
        return "FAIL", f"fehlt: {missing} (--no-system-install gesetzt)"
    pm = next(((n, c) for n, c in (
        ("apt-get", ["apt-get", "install", "-y", "build-essential", "git", "curl", "pkg-config"]),
        ("dnf", ["dnf", "install", "-y", "gcc", "gcc-c++", "make", "git", "curl", "pkgconf-pkg-config"]),
        ("pacman", ["pacman", "-S", "--noconfirm", "base-devel", "git", "curl"]),
        ("zypper", ["zypper", "-n", "install", "-t", "pattern", "devel_basis"])) if shutil.which(n)), None)
    if sys.platform == "darwin":
        return "FAIL", f"fehlt: {missing}. Bitte `xcode-select --install` ausfuehren."
    if not pm:
        return "FAIL", f"fehlt: {missing}; kein bekannter Paketmanager"
    cmd = privileged(pm[1])
    if cmd is None:
        return "FAIL", f"fehlt: {missing}. Kein root/sudo ohne Passwort. Manuell: sudo {' '.join(pm[1])}"
    with open(step.log, "w") as fh:
        if pm[0] == "apt-get":
            run_proc(privileged(["apt-get", "update"]) or cmd, ctx.root, os.environ.copy(), fh, 900)
        r = run_proc(cmd, ctx.root, os.environ.copy(), fh, 1800)
    ok = all(any(shutil.which(a) for a in alts) for alts in (("cc", "gcc", "clang"), ("git",), ("curl",)))
    return ("PASS", f"installiert via {pm[0]}") if ok else ("FAIL", f"Installation ergebnislos (rc={r.rc}), Log pruefen")


def step_rust(ctx: Ctx, step: Step):
    add_cargo_to_path(ctx)
    pin = toolchain_pin(ctx.root)
    log = []
    if not shutil.which("rustup"):
        if ctx.args.no_install:
            return "FAIL", "rustup fehlt und --no-install gesetzt"
        script = ctx.outdir / "rustup-init.sh"
        try:
            urllib.request.urlretrieve("https://sh.rustup.rs", script)
        except OSError as e:
            return "FAIL", f"rustup-Download fehlgeschlagen: {e} (Netz/Proxy pruefen)"
        rc, out, err = capture(["sh", str(script), "-y", "--profile", "minimal", "--default-toolchain", "none"],
                               ctx.root, os.environ.copy(), 900)
        log.append(out + err)
        add_cargo_to_path(ctx)
        if rc != 0 or not shutil.which("rustup"):
            Path(step.log).write_text("\n".join(log))
            return "FAIL", "rustup-Installation fehlgeschlagen"
    rc, out, err = capture(["rustup", "toolchain", "install", pin, "--profile", "minimal",
                            "-c", "rustfmt", "-c", "clippy"], ctx.root, ctx.env, 1800)
    log.append(out + err)
    rc2, v, e2 = capture(["cargo", "-V"], ctx.root, ctx.env, 600)
    log.append(v + e2)
    Path(step.log).write_text("\n".join(log))
    if rc2 != 0:
        return "FAIL", f"cargo mit Pin {pin} nicht nutzbar: {(e2 or err).strip().splitlines()[-1:]}"
    return "PASS", f"{v.strip()} | Pin {pin}"


def step_metadata(ctx: Ctx, step: Step):
    rc, out, err = capture(["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"], ctx.root, ctx.env, 600)
    if rc != 0:
        Path(step.log).write_text(err)
        return "FAIL", "cargo metadata fehlgeschlagen: " + (err.strip().splitlines() or [""])[-1][:160]
    meta = json.loads(out)
    members = set(meta["workspace_members"])
    ctx.target_dir = Path(meta["target_directory"])
    ctx.env["CARGO_TARGET_DIR"] = str(ctx.target_dir)
    for p in meta["packages"]:
        if p["id"] in members:
            ctx.pkgs[p["name"]] = dict(dir=Path(p["manifest_path"]).parent, features=set(p["features"]),
                                       targets=[dict(name=t["name"], kind=t["kind"], req=t.get("required-features") or [],
                                                     src=t["src_path"]) for t in p["targets"]])
    sel = selected_crates(ctx)
    if step.log:
        Path(step.log).write_text("\n".join(f"{n}: {len(p['targets'])} Ziele" for n, p in sorted(ctx.pkgs.items())))
    return "PASS", f"{len(ctx.pkgs)} Workspace-Crates, {len(sel)} ausgewaehlt, target={ctx.target_dir}"


def selected_crates(ctx: Ctx) -> list:
    names = sorted(ctx.pkgs)
    if ctx.args.only:
        want = {short(w.strip()) for w in ctx.args.only.split(",") if w.strip()}
        unknown = want - {short(n) for n in names}
        if unknown:
            raise SystemExit(f"--only: unbekannte Crates {sorted(unknown)}")
        names = [n for n in names if short(n) in want]
    return names


def step_fetch(ctx: Ctx, step: Step):
    with open(step.log, "w") as fh:
        r = run_proc(["cargo", "fetch", "--locked"], ctx.root, ctx.env, fh, 1800)
    return ("PASS", "Abhaengigkeiten geladen") if r.rc == 0 else ("FAIL", f"cargo fetch rc={r.rc} (Netz? Lockfile?)")


def step_hygiene(ctx: Ctx, step: Step):
    warn, info = [], []
    load1 = os.getloadavg()[0]
    if load1 > 0.3 * ctx.ncpu:
        warn.append(f"Systemlast {load1:.2f} bei {ctx.ncpu} CPUs: Messungen verrauscht")
    gov = sh("cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null")
    if gov and gov != "performance":
        warn.append(f"CPU-Governor `{gov}` (mit --tune wird auf `performance` gestellt)")
    if sh("cat /sys/devices/system/cpu/intel_pstate/no_turbo 2>/dev/null") == "0":
        info.append("Turbo aktiv (Taktschwankungen moeglich)")
    sw = re.findall(r"(\d+)", sh("grep -E 'SwapTotal|SwapFree' /proc/meminfo 2>/dev/null"))
    if len(sw) == 2 and int(sw[0]) - int(sw[1]) > 100 * 1024:
        warn.append(f"Swap in Benutzung ({(int(sw[0]) - int(sw[1])) // 1024} MiB)")
    mem = re.search(r"MemTotal:\s+(\d+)", read(Path("/proc/meminfo")))
    if mem and int(mem.group(1)) < 6 * 1024 * 1024:
        warn.append("unter 6 GiB RAM: lto=fat-Builds und scale-Laeufe koennen scheitern (OOM); --jobs 1 empfohlen")
    free = shutil.disk_usage(ctx.root).free / 2**30
    if free < 25:
        warn.append(f"nur {free:.1f} GiB frei: Release-Builds aller Benches brauchen leicht 15-25 GiB")
    if ctx.ncpu == 1:
        warn.append("nur 1 CPU: Parallelitaets-Benches sind nicht aussagekraeftig")
    if ctx.args.tune:
        warn.extend(tune_system(ctx))
    put_data(ctx.outdir, "hygiene", "messhygiene", {"warnungen": warn, "info": info})
    Path(step.log).write_text("\n".join(info + warn) + "\n")
    return "INFO", "; ".join(warn) or "keine Warnungen"


def tune_system(ctx: Ctx) -> list:
    """Optional: Governor und Turbo setzen, beim Ende zuruecksetzen. Braucht root/sudo ohne Passwort."""
    msgs = []
    gov_files = sorted(Path("/sys/devices/system/cpu").glob("cpu[0-9]*/cpufreq/scaling_governor"))
    for f in gov_files:
        old = read(f).strip()
        cmd = privileged(["sh", "-c", f"echo performance > {f}"])
        if cmd and old and old != "performance" and subprocess.run(cmd, capture_output=True).returncode == 0:
            ctx.tuned.append(("sh", f"echo {old} > {f}"))
    nt = Path("/sys/devices/system/cpu/intel_pstate/no_turbo")
    if nt.exists() and read(nt).strip() == "0":
        cmd = privileged(["sh", "-c", f"echo 1 > {nt}"])
        if cmd and subprocess.run(cmd, capture_output=True).returncode == 0:
            ctx.tuned.append(("sh", f"echo 0 > {nt}"))
    if not ctx.tuned and gov_files:
        msgs.append("--tune ohne Wirkung (kein root/sudo ohne Passwort oder schon performance)")
    return msgs


def untune(ctx: Ctx) -> None:
    for _, c in ctx.tuned:
        cmd = privileged(["sh", "-c", c])
        if cmd:
            subprocess.run(cmd, capture_output=True)
    ctx.tuned = []


# ---------------------------------------------------------------------------------------------
# Post-Hooks
# ---------------------------------------------------------------------------------------------

def post_bench(step: Step, text: str, ctx: Ctx, run: int) -> None:
    home = step.crit_base / f"run{run + 1}"
    for r in criterion_collect(home, step.name):
        r["run"], r["step"] = run + 1, step.name
        step.acc.append(r)
    put_data(ctx.outdir, "bench", step.name, step.acc)


def post_build_size(step: Step, text: str, ctx: Ctx, run: int) -> None:
    for b in ("contextra-bench", "scale-100k"):
        p = ctx.target_dir / "release" / b
        if p.exists():
            step.metrics[f"bin_mb_{b}"] = round(p.stat().st_size / 1048576, 2)


def post_ann(step: Step, text: str, ctx: Ctx, run: int) -> None:
    step.metrics.update(parse_kv_lines(text))
    keep = [f"{k}={v}" for k, v in list(step.metrics.items())[:6]]
    step.detail = " ".join(keep)


def post_synthetic(step: Step, text: str, ctx: Ctx, run: int) -> None:
    src = step.cwd / "benchmarks" / "results"
    dest = ctx.outdir / "harness" / "synthetic"
    n = 0
    if src.is_dir():
        dest.mkdir(parents=True, exist_ok=True)
        for f in src.rglob("*"):
            if f.is_file():
                shutil.copy2(f, dest / f.name)
                n += 1
                if f.suffix == ".json":
                    try:
                        step.metrics.update(flatten(json.loads(f.read_text())))
                    except (OSError, ValueError):
                        pass
    step.detail = f"{n} Ergebnisdateien, {len(step.metrics)} Kennzahlen"


def post_scale(step: Step, text: str, ctx: Ctx, run: int) -> None:
    out = Path(step.env["_SCALE_OUT"])
    try:
        d = json.loads(out.read_text())
    except (OSError, ValueError):
        step.detail = "keine Ausgabedatei"
        step.force_fail = True
        return
    iv = d.get("intervals", [])
    put_data(ctx.outdir, "scale", step.name, iv)
    if iv:
        last = iv[-1]
        step.metrics.update(docs_reached=last["docs_inserted"], insert_rate_last=last["insert_rate_docs_per_sec"],
                            search_p50_ns_last=last["search_p50_ns"], search_p99_ns_last=last["search_p99_ns"],
                            rss_mb_last=round(last["rss_bytes"] / 1048576, 1))
    step.detail = f"{len(iv)} Intervalle, erreicht={step.metrics.get('docs_reached')}"
    if d.get("aborted_at_docs") or d.get("error"):
        step.force_fail = True
        step.detail += f" | ABBRUCH bei {d.get('aborted_at_docs')}: {d.get('error')}"


def post_fuzz(step: Step, text: str, ctx: Ctx, run: int) -> None:
    last = None
    for m in re.finditer(r"^#(\d+)\s+\w+\s+cov: (\d+) ft: (\d+) corp: (\d+)/\S+.*?exec/s: (\d+)", text, re.M):
        last = m
    if last:
        step.metrics.update(execs=int(last.group(1)), cov=int(last.group(2)), exec_per_s=int(last.group(5)))
        step.detail = f"execs={last.group(1)} cov={last.group(2)} exec/s={last.group(5)}"
    if re.search(r"Test unit written to|SUMMARY: (libFuzzer|AddressSanitizer)", text):
        step.force_fail = True
        step.detail += " | ABSTURZ GEFUNDEN"


# ---------------------------------------------------------------------------------------------
# Stress: Tests im Release-Modus mit Flake- und Hang-Erkennung
# ---------------------------------------------------------------------------------------------

def test_targets(pkg: dict, excluded: set) -> list:
    sel = []
    for t in pkg["targets"]:
        k = t["kind"]
        if t["name"] in excluded and "test" in k:
            continue
        if "lib" in k or "rlib" in k or "proc-macro" in k:
            sel += ["--lib"]
        elif "bin" in k:
            sel += ["--bin", t["name"]]
        elif "test" in k:
            sel += ["--test", t["name"]]
    return sel


def make_stress(crate: str) -> Callable:
    def fn(ctx: Ctx, step: Step):
        pkg = ctx.pkgs[crate]
        feats = [f for f in STRESS_FEATURES if f in pkg["features"]]
        base = ["cargo", "test", "--release", "-p", crate, "--locked", "--no-fail-fast"]
        if feats:
            base += ["--features", ",".join(feats)]
        excluded: set = set()
        acc = defaultdict(list)
        hangs, timeouts, bins_all, rc_bad = [], 0, [], 0
        with open(step.log, "w") as fh:
            # 1) Kompilierung mit Isolierung kaputter Test-Ziele
            for _ in range(8):
                sel = test_targets(pkg, excluded)
                if not sel:
                    return "SKIPPED", "keine Testziele (oder alle nicht kompilierbar)"
                fh.write(f"\n$ {' '.join(base + ['--no-run'] + sel)}\n")
                fh.flush()
                pos = fh.tell()
                r = run_proc(base + ["--no-run"] + sel, ctx.root, ctx.env, fh, int(TEST_TIMEOUT * ctx.args.timeout_faktor))
                step.cpu_user += r.user
                step.cpu_sys += r.sys
                step.rss_mb = max(step.rss_mb, r.rss_mb)
                if r.rc == 0:
                    break
                fh.flush()
                chunk = read(Path(step.log))[pos:]
                bad = set(re.findall(r'could not compile `[^`]+` \(test "([^"]+)"\)', chunk))
                if r.timeout or not (bad - excluded):
                    return "FAIL", "Build der Tests fehlgeschlagen: " + (first_errors(chunk) or "Timeout/unbekannt")
                excluded |= bad
            runs = ctx.prof["stress_runs"]
            modes = [("par", []), ("t1", ["--test-threads=1"]), ("t2n", [f"--test-threads={2 * ctx.ncpu}"])]
            variants = [modes[i % 3] for i in range(runs)]
            variants = [(f"{m}#{i + 1}", a) for i, (m, a) in enumerate(variants)]
            if ctx.prof["ignored"]:
                variants.append(("ign", ["--include-ignored", "--test-threads=1"]))
            labels_main = [v[0] for v in variants if v[0] != "ign"]
            for label, margs in variants:
                cmd = base + test_targets(pkg, excluded) + ["--"] + margs
                fh.write(f"\n##### {label}: {' '.join(cmd)}\n")
                fh.flush()
                pos = fh.tell()
                r = run_proc(cmd, ctx.root, ctx.env, fh, int(TEST_TIMEOUT * ctx.args.timeout_faktor))
                fh.flush()
                step.cpu_user += r.user
                step.cpu_sys += r.sys
                step.rss_mb = max(step.rss_mb, r.rss_mb)
                chunk = read(Path(step.log))[pos:]
                tests, bins, open_ = parse_tests(chunk)
                for b, t, s in tests:
                    acc[(b, t)].append((label, s))
                bins_all += [(label,) + b for b in bins]
                if r.timeout:
                    timeouts += 1
                    hangs.append(f"{label}: " + (f"{open_[0]}::{open_[1]}" if open_ else "unbekannter Test"))
                elif r.rc != 0:
                    rc_bad += 1
        rows, flaky, red = [], 0, 0
        for (b, t), res in sorted(acc.items()):
            main = [s for l, s in res if l in labels_main and s != "ignored"]
            states = set(main)
            if len(states) > 1 or (main and len(main) < len(labels_main)):
                flaky += 1
                rows.append([crate, b, t, ";".join(f"{l}:{s}" for l, s in res), "flaky"])
            elif states == {"FAILED"} or ({s for l, s in res} == {"FAILED"}):
                red += 1
                rows.append([crate, b, t, ";".join(f"{l}:{s}" for l, s in res), "rot"])
        rows += [[crate, "", "", h, "hang"] for h in hangs]
        put_data(ctx.outdir, "flake", step.name, rows)
        put_data(ctx.outdir, "tests", step.name, [[crate, b, t, ";".join(f"{l}:{s}" for l, s in res)]
                                                  for (b, t), res in sorted(acc.items())])
        put_data(ctx.outdir, "testbins", step.name, [[crate] + list(b) for b in bins_all])
        step.metrics.update(tests=len(acc), rot=red, flaky=flaky, hangs=len(hangs), timeouts=timeouts,
                            ausgeschlossene_ziele=",".join(sorted(excluded)))
        detail = (f"{len(acc)} Tests, {red} rot, {flaky} flaky, {len(hangs)} Hang/Timeout"
                  + (f", nicht kompilierbare Ziele: {sorted(excluded)}" if excluded else "")
                  + (f", Features {feats}" if feats else ""))
        bad = red or flaky or hangs or rc_bad or excluded
        return ("FAIL" if bad else "PASS"), detail
    return fn


def make_loom(crate: str) -> Callable:
    def fn(ctx: Ctx, step: Step):
        env = dict(ctx.env)
        env["RUSTFLAGS"] = (env.get("RUSTFLAGS", "") + " --cfg loom").strip()
        env["CARGO_TARGET_DIR"] = str(ctx.target_dir / "loom")
        env["LOOM_MAX_PREEMPTIONS"] = "3"
        with open(step.log, "w") as fh:
            r = run_proc(["cargo", "test", "--release", "-p", crate, "--locked", "--", "loom"], ctx.root, env, fh, 7200)
        text = read(Path(step.log))
        step.cpu_user, step.cpu_sys, step.rss_mb = r.user, r.sys, r.rss_mb
        if r.timeout:
            return "TIMEOUT", "Zeitlimit"
        tests, bins, _ = parse_tests(text)
        n = sum(b[1] for b in bins)
        if r.rc != 0:
            return "FAIL", (first_errors(text) or f"rc={r.rc}") + f" ({n} ok)"
        return ("INFO", "keine Loom-Tests gefunden") if n == 0 else ("PASS", f"{n} Loom-Tests ok")
    return fn


# ---------------------------------------------------------------------------------------------
# Plan
# ---------------------------------------------------------------------------------------------

def is_criterion(src: str) -> bool:
    return "criterion" in read(Path(src))


def build_plan(ctx: Ctx, groups: set) -> list:
    a, prof = ctx.args, ctx.prof
    S: list = []
    O = ctx.outdir
    S.append(Step("setup", "system_deps", py=step_system_deps, cargo=False))
    S.append(Step("setup", "rust_toolchain", py=step_rust, cargo=False, timeout=3600))
    S.append(Step("setup", "workspace_metadata", py=step_metadata, dep="rust_toolchain"))
    S.append(Step("setup", "cargo_fetch", py=step_fetch, dep="workspace_metadata"))
    S.append(Step("setup", "messhygiene", py=step_hygiene, cargo=False, info=True))
    return S  # der restliche Plan entsteht nach dem Setup (expand_plan), weil er cargo metadata braucht


def expand_plan(ctx: Ctx, groups: set) -> list:
    """Wird nach den Setup-Schritten aufgerufen, wenn cargo metadata vorliegt."""
    a, prof, O = ctx.args, ctx.prof, ctx.outdir
    crates = selected_crates(ctx)
    S: list = []
    crit_flags = list(prof["crit"]) + ["--noplot"]
    if a.baseline:
        crit_flags += ["--baseline", a.baseline]
    if a.save_baseline:
        crit_flags += ["--save-baseline", a.save_baseline]
    repeat = a.repeat if a.repeat is not None else prof["repeat"]
    for c in crates:
        for t in ctx.pkgs[c]["targets"]:
            if "bench" not in t["kind"]:
                continue
            feat = ["--features", ",".join(t["req"])] if t["req"] else []
            base = ["cargo", "bench", "-p", c, "--bench", t["name"], "--locked"] + feat
            bname = f"build__{short(c)}__{t['name']}"
            S.append(Step("build", bname, base + ["--no-run"], timeout=7200, dep="cargo_fetch",
                          note="requires " + ",".join(t["req"]) if t["req"] else ""))
            crit = is_criterion(t["src"])
            cmd = base + (["--"] + crit_flags if crit else [])
            nm = f"bench__{short(c)}__{t['name']}"
            S.append(Step("bench", nm, cmd, timeout=prof["bench_to"] * (repeat if crit else 1), dep=bname,
                          post=post_bench if crit else None, repeat=repeat if crit else 1,
                          crit_base=O / "criterion" / nm,
                          note="Criterion" if crit else "eigener Harness ohne Criterion: Ausgabe nur im Log"))
    if "contextra-bench" in crates or not a.only:
        hroot = O / "harness"
        rel = ctx.target_dir / "release"
        S.append(Step("harness", "harness_build",
                      ["cargo", "build", "--release", "-p", "contextra-bench", "--bins", "--features",
                       "external-benchmarks", "--locked"], timeout=7200, dep="cargo_fetch", post=post_build_size))
        S.append(Step("harness", "harness_synthetisch", [str(rel / "contextra-bench")], cwd=hroot / "work_syn",
                      timeout=3600, dep="harness_build", cargo=False, post=post_synthetic,
                      note="Szenario A/B; schreibt benchmarks/results im Arbeitsverzeichnis"))
        S.append(Step("harness", "harness_ann_synthetisch", [str(rel / "contextra-bench"), "--ann"],
                      cwd=hroot / "work_ann", timeout=1800, dep="harness_build", cargo=False, post=post_ann,
                      note="Recall@10, QPS bei Recall 0.90/0.95/0.99, p50/p99"))
        docs, stp = prof["scale"]
        sj = hroot / f"scale_{docs}.json"
        S.append(Step("harness", f"harness_scale__{docs}",
                      [str(rel / "scale-100k"), "--docs", str(docs), "--step", str(stp), "--dim", "128", "--seed", "42",
                       "--out", str(sj)], cwd=hroot / "work_scale", timeout=max(3600, docs // 100_000 * 7200),
                      env={"_SCALE_OUT": str(sj)}, dep="harness_build", cargo=False, post=post_scale,
                      note="Einfuegerate, Such-Perzentile und RSS ueber wachsende Datenmenge"))
    for c in crates:
        if any(("test" in t["kind"] or "lib" in t["kind"] or "bin" in t["kind"]) for t in ctx.pkgs[c]["targets"]):
            S.append(Step("stress", f"stress__{short(c)}", py=make_stress(c), dep="cargo_fetch", timeout=10**6,
                          note="par/threads=1/2xCPU + --include-ignored, Release"))
    if a.deep:
        for c in crates:
            if "loom" in read(ctx.pkgs[c]["dir"] / "Cargo.toml"):
                S.append(Step("deep", f"loom__{short(c)}", py=make_loom(c), dep="cargo_fetch", timeout=10**6))
        for c in crates:
            ft = ctx.pkgs[c]["dir"] / "fuzz" / "fuzz_targets"
            if ft.is_dir():
                for tgt in sorted(p.stem for p in ft.glob("*.rs")):
                    S.append(Step("deep", f"fuzz__{short(c)}__{tgt}",
                                  ["cargo", "+nightly", "fuzz", "run", tgt, "--", f"-max_total_time={prof['fuzz']}"],
                                  cwd=ctx.pkgs[c]["dir"], timeout=prof["fuzz"] + 1800, post=post_fuzz,
                                  dep="fuzz_tooling"))
    S = [s for s in S if s.group in groups]
    return S


def step_fuzz_tooling(ctx: Ctx, step: Step):
    with open(step.log, "w") as fh:
        r1 = run_proc(["rustup", "toolchain", "install", "nightly", "--profile", "minimal"], ctx.root, ctx.env, fh, 1800)
        r2 = run_proc(["cargo", "install", "cargo-fuzz", "--locked"], ctx.root, ctx.env, fh, 3600)
    ok = r1.rc == 0 and r2.rc == 0
    return ("PASS", "nightly + cargo-fuzz bereit") if ok else ("FAIL", f"nightly rc={r1.rc}, cargo-fuzz rc={r2.rc}")


# ---------------------------------------------------------------------------------------------
# Ausfuehrung
# ---------------------------------------------------------------------------------------------

def free_gb(path: Path) -> float:
    return shutil.disk_usage(path).free / 2**30


def ensure_disk(ctx: Ctx) -> bool:
    if free_gb(ctx.root) >= ctx.args.min_free_gb:
        return True
    for sub in ("debug", "loom"):  # werden im Messlauf nicht gebraucht bzw. sind neu erzeugbar
        shutil.rmtree(ctx.target_dir / sub, ignore_errors=True)
    return free_gb(ctx.root) >= ctx.args.min_free_gb


def run_step(step: Step, idx: int, ctx: Ctx, by_name: dict) -> None:
    safe = re.sub(r"[^A-Za-z0-9_.-]", "_", step.name)
    step.log = str(ctx.outdir / "logs" / f"{idx:03d}_{safe}.log")
    skip = None
    if step.dep:
        d = by_name.get(step.dep)
        if d is not None and d.status not in ("PASS", "INFO"):
            skip = f"Voraussetzung `{step.dep}` ist {d.status}"
    if step.cargo and ctx.cargo_ok is False:
        skip = "Toolchain nicht verfuegbar"
    if not skip and step.cargo and not ensure_disk(ctx):
        skip = f"weniger als {ctx.args.min_free_gb} GiB frei ({free_gb(ctx.root):.1f}); Platz schaffen oder --target-dir waehlen"
    if skip:
        step.status, step.detail = "SKIPPED", skip
        Path(step.log).write_text(f"SKIPPED: {skip}\n")
        return
    t0 = time.time()
    if step.py:
        try:
            status, detail = step.py(ctx, step)
        except Exception as e:  # Schritt selbst defekt: sichtbar machen
            status, detail = "FAIL", f"Schritt abgestuerzt: {type(e).__name__}: {e}"
            Path(step.log).write_text(traceback.format_exc())
        if not Path(step.log).exists():
            Path(step.log).write_text(detail + "\n")
        step.status, step.detail = ("INFO" if step.info and status == "PASS" else status), detail
        step.seconds = round(time.time() - t0, 1)
        step.cpu_user, step.cpu_sys, step.rss_mb = round(step.cpu_user, 1), round(step.cpu_sys, 1), round(step.rss_mb, 1)
        return
    env = dict(ctx.env)
    env.update({k: v for k, v in step.env.items() if not k.startswith("_")})
    cwd = step.cwd or ctx.root
    cwd.mkdir(parents=True, exist_ok=True)
    timeout = int(step.timeout * ctx.args.timeout_faktor)
    u = s = rss = 0.0
    timed_out, rc = False, 0
    with open(step.log, "w") as fh:
        fh.write(f"$ {' '.join(step.cmd)}\n# cwd={cwd} wiederholungen={step.repeat}\n\n")
        for run in range(step.repeat):
            if step.crit_base:
                env["CRITERION_HOME"] = str(step.crit_base / f"run{run + 1}")
            if step.repeat > 1:
                fh.write(f"\n##### Lauf {run + 1}/{step.repeat}\n")
            fh.flush()
            pos = fh.tell()
            r = run_proc(step.cmd, cwd, env, fh, timeout)
            fh.flush()
            u, s, rss = u + r.user, s + r.sys, max(rss, r.rss_mb)
            rc, timed_out = r.rc, r.timeout
            if step.post:
                try:
                    step.post(step, read(Path(step.log))[pos:], ctx, run)
                except Exception as e:
                    step.detail += f" | Auswertung fehlgeschlagen: {type(e).__name__}: {e}"
            if timed_out or rc != 0:
                break
    step.seconds = round(time.time() - t0, 1)
    step.cpu_user, step.cpu_sys, step.rss_mb = round(u, 1), round(s, 1), round(rss, 1)
    text = read(Path(step.log))
    if timed_out:
        step.status, step.detail = "TIMEOUT", f">{timeout}s je Lauf. " + step.detail
    else:
        step.rc = rc
        step.status = "PASS" if rc == 0 else "FAIL"
        if rc != 0:
            step.detail = (first_errors(text) or f"rc={rc}") + " " + step.detail
    if step.force_fail and step.status == "PASS":
        step.status = "FAIL"
    if step.name.startswith("bench__") and step.acc:
        step.detail = f"{len({r['benchmark'] for r in step.acc})} Benchmarks x {step.repeat} Lauf/Laeufe " + step.detail
    if step.name.startswith("bench__") and step.status == "PASS" and not step.acc and not step.post:
        step.status, step.detail = "INFO", "ohne Criterion; Rohausgabe im Log"
    step.detail = step.detail.strip()


# ---------------------------------------------------------------------------------------------
# Auswertung
# ---------------------------------------------------------------------------------------------

def aggregate_bench(outdir: Path) -> list:
    raw = [r for _, rows in get_data(outdir, "bench") for r in rows]
    if not raw:
        return []
    write_csv(outdir / "bench_runs.csv",
              ["step", "benchmark", "lauf", "mean_ns", "median_ns", "std_ns", "ci_low_ns", "ci_high_ns", "tp_elems", "tp_bytes"],
              [[r["step"], r["benchmark"], r["run"], r["mean_ns"], r["median_ns"], r["std_ns"], r["ci_low_ns"], r["ci_high_ns"],
                r["tp_elems"], r["tp_bytes"]] for r in sorted(raw, key=lambda x: (x["step"], x["benchmark"], x["run"]))])
    groups = defaultdict(list)
    for r in raw:
        groups[(r["step"], r["benchmark"])].append(r)
    agg = []
    for (step, name), rs in sorted(groups.items()):
        meds = [x["median_ns"] for x in rs]
        med = statistics.median(meds)
        mean = statistics.mean(meds)
        cv = statistics.stdev(meds) / mean * 100 if len(meds) > 1 and mean > 0 else None
        last = rs[-1]
        agg.append(dict(crate=step.split("__")[1], step=step, benchmark=name, runs=len(rs), median_ns=med,
                        min_ns=min(meds), max_ns=max(meds), cv_pct=cv,
                        tp_elems_s=(last["tp_elems"] / (med * 1e-9)) if last["tp_elems"] and med > 0 else None,
                        tp_bytes_s=(last["tp_bytes"] / (med * 1e-9)) if last["tp_bytes"] and med > 0 else None))
    write_csv(outdir / "bench_results.csv",
              ["crate", "step", "benchmark", "laeufe", "median_ns", "min_ns", "max_ns", "cv_pct", "durchsatz_elem_s", "durchsatz_byte_s"],
              [[x["crate"], x["step"], x["benchmark"], x["runs"], f"{x['median_ns']:.3f}", f"{x['min_ns']:.3f}", f"{x['max_ns']:.3f}",
                "" if x["cv_pct"] is None else f"{x['cv_pct']:.2f}",
                "" if x["tp_elems_s"] is None else f"{x['tp_elems_s']:.1f}",
                "" if x["tp_bytes_s"] is None else f"{x['tp_bytes_s']:.1f}"] for x in agg])
    return agg


def write_reports(ctx: Ctx, steps: list, meta: dict, final: bool = False) -> None:
    O = ctx.outdir
    rows = [dict(nr=i, gruppe=s.group, schritt=s.name, status=s.status, rc=s.rc, sekunden=s.seconds, cpu_user_s=s.cpu_user,
                 cpu_sys_s=s.cpu_sys, rss_mb=s.rss_mb, detail=s.detail, log=os.path.relpath(s.log, O) if s.log else "",
                 befehl=" ".join(s.cmd), hinweis=s.note, metrics=s.metrics) for i, s in enumerate(steps)]
    (O / "results.json").write_text(json.dumps({"meta": meta, "steps": rows}, indent=2, ensure_ascii=False, default=str))
    if rows:
        with open(O / "results.csv", "w", newline="") as fh:
            w = csv.DictWriter(fh, fieldnames=[k for k in rows[0] if k != "metrics"])
            w.writeheader()
            w.writerows([{k: v for k, v in r.items() if k != "metrics"} for r in rows])
    order = {"FAIL": 0, "TIMEOUT": 1, "SKIPPED": 2, "INFO": 3, "PASS": 4, "PENDING": 5}
    cnt = defaultdict(int)
    for s in steps:
        cnt[s.status] += 1
    L = ["# Contextra Benchmark und Belastungstest", "",
         f"- Commit `{meta['commit']}` ({meta['commit_date']}), Branch `{meta['branch']}`, ungespeicherte Aenderungen: {meta['dirty']}",
         f"- Start {meta['start']}, Dauer {meta['duration_min']} min, Profil `{ctx.args.profil}`, Wiederholungen je Bench "
         f"{ctx.args.repeat if ctx.args.repeat is not None else ctx.prof['repeat']}",
         f"- Host: {meta['host']}", f"- CPU: {meta['cpu']}", f"- Toolchain: {meta['rustc']}",
         f"- native: {ctx.args.native}, Overflow-Checks aus: {ctx.args.no_overflow_checks}, tune: {ctx.args.tune}", "",
         "## Gesamt", ""]
    L += [f"- **{k}**: {v}" for k, v in sorted(cnt.items(), key=lambda kv: order.get(kv[0], 9))]
    hyg = get_data(O, "hygiene")
    if hyg and hyg[0][1].get("warnungen"):
        L += ["", "**Warnungen zur Messumgebung (Ergebnisse entsprechend vorsichtig lesen):**"] + [f"- {w}" for w in hyg[0][1]["warnungen"]]
    L += ["", "## Rote und uebersprungene Schritte", ""]
    red = [[f"`{s.name}`", s.status, s.seconds, s.detail[:300], f"`{os.path.relpath(s.log, O) if s.log else ''}`"]
           for s in sorted(steps, key=lambda x: order.get(x.status, 9)) if s.status in ("FAIL", "TIMEOUT", "SKIPPED")]
    L += table(["Schritt", "Status", "Sek", "Detail", "Log"], red) if red else ["Keine."]
    if final:
        agg = aggregate_bench(O)
        if agg:
            noisy = [x for x in agg if x["cv_pct"] is not None and x["cv_pct"] > 10]
            L += ["", f"## Criterion: {len(agg)} Benchmarks, {len(noisy)} mit CV > 10 % ueber die Laeufe", "",
                  "Die 30 langsamsten nach Median. Vollstaendig in `bench_results.csv`, jede Wiederholung in `bench_runs.csv`.", ""]
            L += table(["Crate", "Benchmark", "Median", "min..max", "CV %", "Durchsatz"],
                       [[short(x["crate"]), x["benchmark"], fmt_ns(x["median_ns"]),
                         f"{fmt_ns(x['min_ns'])}..{fmt_ns(x['max_ns'])}" if x["runs"] > 1 else "-",
                         "-" if x["cv_pct"] is None else f"{x['cv_pct']:.1f}",
                         f"{x['tp_elems_s']:.3g} Elem/s" if x["tp_elems_s"] else
                         (f"{x['tp_bytes_s'] / 1048576:.3g} MiB/s" if x["tp_bytes_s"] else "-")]
                        for x in sorted(agg, key=lambda x: -x["median_ns"])[:30]], ["---", "---", "---:", "---:", "---:", "---:"])
            if noisy:
                L += ["", "Unruhigste Benchmarks (Unterschiede unterhalb dieses Rauschens sind nicht belastbar):", ""]
                L += table(["Crate", "Benchmark", "CV %"], [[short(x["crate"]), x["benchmark"], f"{x['cv_pct']:.1f}"]
                                                            for x in sorted(noisy, key=lambda x: -x["cv_pct"])[:15]], ["---", "---", "---:"])
        hs = [s for s in steps if s.group == "harness" and s.metrics and not s.name.endswith("_build")]
        if hs:
            L += ["", "## Harness-Kennzahlen", ""]
            for s in hs:
                L += [f"**`{s.name}`** ({s.status})", ""] + table(["Kennzahl", "Wert"], [[k, v] for k, v in list(s.metrics.items())[:30]],
                                                                  ["---", "---:"]) + [""]
        sc = []
        for st, iv in get_data(O, "scale"):
            sc += [[st, x["docs_inserted"], round(x["insert_rate_docs_per_sec"], 1), x["search_p50_ns"], x["search_p95_ns"],
                    x["search_p99_ns"], round(x["rss_bytes"] / 1048576, 1)] for x in iv]
        if sc:
            write_csv(O / "scale_curve.csv", ["schritt", "dokumente", "einfuegerate_docs_s", "p50_ns", "p95_ns", "p99_ns", "rss_mb"], sc)
            L += ["## Skalierung (scale-100k)", ""] + table(["Dokumente", "Einfuegen Docs/s", "p50", "p95", "p99", "RSS MiB"],
                  [[r[1], r[2], fmt_ns(r[3]), fmt_ns(r[4]), fmt_ns(r[5]), r[6]] for r in sc], ["---:"] * 6)
        st_steps = [s for s in steps if s.group == "stress"]
        if st_steps:
            L += ["", "## Stress / Belastungstests (Release, mehrere Laeufe)", ""]
            L += table(["Crate", "Status", "Tests", "rot", "flaky", "Hang", "Detail"],
                       [[s.name.split("__", 1)[1], s.status, s.metrics.get("tests", "-"), s.metrics.get("rot", "-"),
                         s.metrics.get("flaky", "-"), s.metrics.get("hangs", "-"), s.detail[:160]] for s in st_steps])
        fl = [r for _, rr in get_data(O, "flake") for r in rr]
        if fl:
            write_csv(O / "flake_report.csv", ["crate", "binary", "test", "ergebnisse_je_lauf", "art"], fl)
            L += ["", "### Rote, flaky und haengende Tests", ""] + table(["Crate", "Test", "Art", "Ergebnisse"],
                  [[short(r[0]), r[2] or "-", r[4], r[3][:140]] for r in fl[:60]])
        tr = [[st] + r for st, rr in get_data(O, "tests") for r in rr]
        if tr:
            write_csv(O / "tests.csv", ["schritt", "crate", "binary", "test", "ergebnisse_je_lauf"], tr)
        mrows = [[s.name, k, v] for s in steps for k, v in sorted(s.metrics.items())]
        if mrows:
            write_csv(O / "metrics.csv", ["schritt", "schluessel", "wert"], mrows)
        heavy = sorted([s for s in steps if s.status in ("PASS", "INFO", "FAIL") and (s.cpu_user + s.cpu_sys) > 0],
                       key=lambda x: -(x.cpu_user + x.cpu_sys))[:12]
        if heavy:
            L += ["", "## Ressourcenverbrauch (Top 12 nach CPU-Zeit)", ""] + table(
                ["Schritt", "Wand s", "CPU user s", "CPU sys s", "Spitzen-RSS MiB"],
                [[f"`{s.name}`", s.seconds, s.cpu_user, s.cpu_sys, s.rss_mb] for s in heavy], ["---", "---:", "---:", "---:", "---:"])
    L += ["", "## Alle Schritte", ""] + table(["Gruppe", "Schritt", "Status", "Sek", "CPU s", "RSS MiB", "Detail"],
          [[s.group, f"`{s.name}`", s.status, s.seconds, round(s.cpu_user + s.cpu_sys, 1), s.rss_mb, s.detail[:140]] for s in steps])
    (O / "summary.md").write_text("\n".join(L) + "\n")


def collect_meta(ctx: Ctx, start: dt.datetime) -> dict:
    r = ctx.root
    mem = re.search(r"MemTotal:\s+(\d+)", read(Path("/proc/meminfo")))
    return dict(commit=sh("git rev-parse --short HEAD", r) or "unknown", commit_date=sh("git log -1 --format=%ci", r),
                branch=sh("git rev-parse --abbrev-ref HEAD", r), dirty=bool(sh("git status --porcelain", r)),
                host=f"{platform.platform()} | {ctx.ncpu} CPUs | {int(mem.group(1)) // 1024 // 1024 if mem else '?'} GiB RAM",
                cpu=sh("lscpu 2>/dev/null | grep 'Model name' | head -1 | cut -d: -f2") or platform.processor(),
                rustc=sh("rustc -V", r), start=start.isoformat(timespec="seconds"), duration_min=0.0, argv=" ".join(sys.argv),
                lock_sha256=sh("sha256sum Cargo.lock | cut -d' ' -f1", r))


def compare_runs(a: Path, b: Path) -> int:
    def load(p: Path) -> dict:
        f = p / "bench_results.csv"
        if not f.exists():
            return {}
        with open(f) as fh:
            return {f"{r['crate']}/{r['benchmark']}": (float(r["median_ns"]), float(r["cv_pct"]) if r["cv_pct"] else 0.0)
                    for r in csv.DictReader(fh)}
    ba, bb = load(a), load(b)
    out = ["# Vergleich", "", f"A: `{a}`", f"B: `{b}`", "",
           "Nur Abweichungen > 5 % und ueber dem Rauschen (max. CV beider Laeufe). Negativ = B schneller.", "",
           "| Benchmark | A | B | Delta | Rauschen |", "|---|---:|---:|---:|---:|"]
    for k in sorted(set(ba) & set(bb)):
        d = (bb[k][0] - ba[k][0]) / ba[k][0] * 100 if ba[k][0] else 0
        noise = max(ba[k][1], bb[k][1])
        if abs(d) > 5 and abs(d) > noise:
            out.append(f"| `{k}` | {fmt_ns(ba[k][0])} | {fmt_ns(bb[k][0])} | {d:+.1f} % | {noise:.1f} % |")
    text = "\n".join(out) + "\n"
    (b / "vergleich.md").write_text(text)
    print(text)
    return 0


# ---------------------------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------------------------

def find_root(args) -> Path:
    root = Path(args.root).resolve()
    if "[workspace]" in read(root / "Cargo.toml"):
        return root
    target = root / "contextra"
    if "[workspace]" not in read(target / "Cargo.toml"):
        if not shutil.which("git"):
            raise SystemExit("git fehlt und kein Repo gefunden. Bitte git installieren oder im Repo-Root starten.")
        print(f"Klone {args.repo_url} -> {target}")
        if subprocess.run(["git", "clone", args.repo_url, str(target)]).returncode != 0:
            raise SystemExit("git clone fehlgeschlagen (Netz/Proxy?)")
    return target


def main() -> int:
    ap = argparse.ArgumentParser(description="Contextra: autonomes Setup, Benchmarking, Belastungstest")
    ap.add_argument("--profil", choices=sorted(PROFILES), default="standard")
    ap.add_argument("--gruppen", help=f"kommagetrennt aus {','.join(GROUPS)} (Standard: alle ausser deep)")
    ap.add_argument("--deep", action="store_true", help="zusaetzlich Gruppe deep (Loom, cargo-fuzz)")
    ap.add_argument("--only", help="Crates (Kurzname), z. B. store,vector")
    ap.add_argument("--repeat", type=int, help="Wiederholungen je Criterion-Bench")
    ap.add_argument("--native", action="store_true", help="-C target-cpu=native")
    ap.add_argument("--no-overflow-checks", action="store_true", help="overflow-checks in release/bench aus (A/B-Vergleich)")
    ap.add_argument("--tune", action="store_true", help="Governor=performance, Turbo aus (braucht root/sudo ohne Passwort), danach Reset")
    ap.add_argument("--env", action="append", metavar="K=V")
    ap.add_argument("--baseline"), ap.add_argument("--save-baseline")
    ap.add_argument("--target-dir"), ap.add_argument("--jobs", type=int)
    ap.add_argument("--min-free-gb", type=float, default=MIN_FREE_GB)
    ap.add_argument("--timeout-faktor", type=float, default=1.0)
    ap.add_argument("--no-install", action="store_true", help="rustup nicht installieren")
    ap.add_argument("--no-system-install", action="store_true", help="keine Systempakete installieren")
    ap.add_argument("--fail-fast", action="store_true")
    ap.add_argument("--list", action="store_true", help="Plan zeigen (benoetigt vorhandene Toolchain fuer die Bench-Liste)")
    ap.add_argument("--resume")
    ap.add_argument("--vergleich", nargs=2, metavar=("A", "B"))
    ap.add_argument("--root", default=".")
    ap.add_argument("--repo-url", default=REPO_URL)
    args = ap.parse_args()
    if args.vergleich:
        return compare_runs(Path(args.vergleich[0]), Path(args.vergleich[1]))
    if sys.version_info < (3, 9):
        sys.exit("Python 3.9+ erforderlich")

    groups = {g.strip() for g in args.gruppen.split(",")} if args.gruppen else set(GROUPS) - {"deep"}
    if args.deep:
        groups.add("deep")
    bad = groups - set(GROUPS)
    if bad:
        sys.exit(f"unbekannte Gruppen: {sorted(bad)}")
    groups.add("setup")  # alles andere haengt davon ab

    root = find_root(args)
    start = dt.datetime.now(dt.timezone.utc)
    commit = sh("git rev-parse --short HEAD", root) or "nogit"
    outdir = Path(args.resume).resolve() if args.resume else root / "evidence" / f"bench-{start.strftime('%Y%m%dT%H%M%SZ')}-{commit}"
    if not args.list:
        (outdir / "logs").mkdir(parents=True, exist_ok=True)
    ctx = Ctx(root, outdir, args)
    add_cargo_to_path(ctx)
    meta = collect_meta(ctx, start)
    t_start = time.time()
    previous = {}
    if args.resume and (outdir / "results.json").exists():
        previous = {s["schritt"]: s for s in json.loads((outdir / "results.json").read_text())["steps"]}

    steps: list = build_plan(ctx, groups)
    by_name = {s.name: s for s in steps}
    if args.deep and "deep" in groups:
        steps.append(Step("deep", "fuzz_tooling", py=step_fuzz_tooling, dep="cargo_fetch", timeout=10**6))
        by_name["fuzz_tooling"] = steps[-1]

    def expand() -> None:
        extra = expand_plan(ctx, groups)
        steps.extend(extra)
        by_name.update({e.name: e for e in extra})

    if args.list:  # nur Plan: einzig cargo metadata (lesend) wird ausgefuehrt
        if shutil.which("cargo") and step_metadata(ctx, Step("setup", "m", py=None))[0] == "PASS":
            expand()
        else:
            print("Hinweis: ohne cargo/Toolchain ist nur der Setup-Teil des Plans bekannt.")
        for n, s in enumerate(steps):
            print(f"{n:03d} [{s.group:7s}] {s.name:55s} {' '.join(s.cmd) if s.cmd else '(Python-Schritt)'}")
        print(f"\n{len(steps)} Schritte geplant.")
        return 0

    print(f"Ausgabe: {outdir}\nPlatz frei: {free_gb(root):.1f} GiB\n")
    expanded = False
    i = 0
    try:
        while i < len(steps):
            s = steps[i]
            prev = previous.get(s.name)
            if prev and prev["status"] in ("PASS", "INFO") and s.name not in ("rust_toolchain", "workspace_metadata"):
                s.status, s.seconds, s.detail = prev["status"], prev["sekunden"], (prev["detail"] + " (uebernommen)").strip()
                s.log = str(outdir / prev["log"]) if prev["log"] else ""
                s.metrics = prev.get("metrics") or {}
                print(f"[{i + 1:03d}] {s.group:8s} {s.name} ... uebernommen")
            else:
                print(f"[{i + 1:03d}/{len(steps)}] {s.group:8s} {s.name} ...", end=" ", flush=True)
                run_step(s, i, ctx, by_name)
                if s.name == "rust_toolchain":
                    ctx.cargo_ok = s.status == "PASS"
                print(f"{s.status} ({s.seconds}s) {s.detail[:110]}")
            if s.name == "messhygiene" and not expanded:
                expanded = True
                if ctx.pkgs:
                    expand()
                else:
                    print("Hinweis: ohne Workspace-Metadaten gibt es keine Bench-/Stress-Schritte.")
            meta["duration_min"] = round((time.time() - t_start) / 60, 1)
            write_reports(ctx, steps, meta)
            if args.fail_fast and s.status in ("FAIL", "TIMEOUT") and s.group != "setup":
                print("Abbruch (--fail-fast)")
                break
            i += 1
    except KeyboardInterrupt:
        print(f"\nAbgebrochen. Zwischenstand gespeichert. Fortsetzen: --resume {outdir}")
    finally:
        untune(ctx)
    meta["duration_min"] = round((time.time() - t_start) / 60, 1)
    write_reports(ctx, steps, meta, final=True)
    red = [s for s in steps if s.status in ("FAIL", "TIMEOUT")]
    print(f"\nFertig in {meta['duration_min']} min. FAIL/TIMEOUT: {len(red)}  -> {outdir}/summary.md")
    return 1 if red else 0


if __name__ == "__main__":
    sys.exit(main())
