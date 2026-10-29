#!/usr/bin/env python3
"""
contextra_full_audit.py - vollstaendiger Test-, Lint-, Gate-, Fuzz- und Benchmark-Lauf.

Aufruf (im Repo-Root, also dort wo die Cargo.toml mit [workspace] liegt):

    python3 scripts/contextra_full_audit.py                 # Standardlauf (alles ausser Fuzz/Mutation/Sanitizer)
    python3 scripts/contextra_full_audit.py --quick         # nur check + fmt + clippy + tests (ohne Benchmarks)
    python3 scripts/contextra_full_audit.py --all           # wirklich alles inkl. Fuzz, Mutation, Miri, Loom
    python3 scripts/contextra_full_audit.py --only store,ingest   # nur diese Crates (Kurzname ohne "contextra-")
    python3 scripts/contextra_full_audit.py --list          # zeigt alle Schritte, fuehrt nichts aus

Ergebnisse landen in  evidence/audit-<UTC-Zeit>-<commit>/ :
    summary.md        lesbare Zusammenfassung (rote Schritte zuerst)
    results.json      maschinenlesbar, jeder Schritt mit Status, Dauer, Log-Pfad
    results.csv       dasselbe als Tabelle
    logs/NN_name.log  vollstaendiger Output je Schritt
    criterion/        kopierte Criterion-Berichte (Benchmarks)
    env.txt           Toolchain, OS, CPU, RAM, git-Stand

Nur Python-Standardbibliothek noetig. Fehlende Werkzeuge fuehren zu SKIPPED mit Grund, nicht zum Abbruch.
Das Skript aendert keinen Quellcode. Es schreibt nur nach evidence/ und target/.
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
import subprocess
import sys
import time
from pathlib import Path

# ----------------------------------------------------------------------------------------------
# Konfiguration
# ----------------------------------------------------------------------------------------------

# Crates mit Benchmarks laut Cargo.toml: (Paket, [bench-Namen])
BENCHES = {
    "contextra-mvcc": ["mvcc_bench"],
    "contextra-checkpoint": ["checkpoint_bench"],
    "contextra-router": ["router_bench", "bandit_latency_gate", "cost_routing_savings_bench"],
    "contextra-graph": ["csr_traversal_bench", "binary_edge_regression"],
    "contextra-crypto": ["crypto_benchmarks", "deletion_proof_latency_bench"],
    "contextra-infer-candle": ["kv_bridge_bench"],
    "contextra-vector": [
        "distance_bench", "sq8_bench", "hnsw_bench", "hnsw_delete_search_bench",
        "audit_benchmarks", "flush_threshold_amplification", "partial_rebuild_recall_bench",
        "distance_nan_impact_bench", "filtered_search_selectivity_bench",
    ],
    "contextra-store": [
        "sstable_bench", "memtable_bench", "wal_bench", "lsm_concurrency_bench",
        "block_cache_bench", "redb_comparative_bench",
    ],
    "contextra-simd": ["simd_bench"],
    "contextra-bench": ["competitive_kv", "locomo", "long_mem_eval"],
}

# Fuzz-Verzeichnisse: crates/<name>/fuzz/fuzz_targets/*.rs (benoetigt: cargo install cargo-fuzz + nightly)
FUZZ_CRATES = ["ingest", "mcp", "text", "db", "crypto", "vector", "store", "wire"]

# Crates, deren Tests bekanntermassen sehr lange dauern (Debug); bekommen groesseres Timeout.
SLOW_CRATES = {"contextra-crypto": 3600, "contextra-store": 3600, "contextra-vector": 3600,
               "contextra-graph": 3600, "contextra-engine": 3600}

DEFAULT_TIMEOUT = 1800      # Sekunden pro Schritt
BENCH_TIMEOUT = 3600        # Sekunden pro Benchmark-Ziel
FUZZ_SECONDS = 120          # Laufzeit je Fuzz-Ziel


def now_utc() -> dt.datetime:
    return dt.datetime.now(dt.timezone.utc)


def which(name: str) -> str | None:
    return shutil.which(name)


# ----------------------------------------------------------------------------------------------
# Hilfsklassen
# ----------------------------------------------------------------------------------------------

class Step:
    def __init__(self, group: str, name: str, cmd: list[str], *, timeout: int = DEFAULT_TIMEOUT,
                 requires: list[str] | None = None, env: dict | None = None,
                 cwd: Path | None = None, note: str = ""):
        self.group, self.name, self.cmd = group, name, cmd
        self.timeout, self.requires = timeout, requires or []
        self.env, self.cwd, self.note = env or {}, cwd, note
        self.status = "PENDING"
        self.rc: int | None = None
        self.seconds = 0.0
        self.log = ""
        self.detail = ""


def parse_cargo_test_summary(text: str) -> str:
    """Summiert alle 'test result:' Zeilen eines cargo-test-Laufs."""
    passed = failed = ignored = 0
    for m in re.finditer(r"test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored", text):
        passed += int(m.group(1)); failed += int(m.group(2)); ignored += int(m.group(3))
    failing = sorted(set(re.findall(r"^test (\S+) \.\.\. FAILED", text, re.M)))
    s = f"passed={passed} failed={failed} ignored={ignored}"
    if failing:
        s += " | FAILED: " + ", ".join(failing[:15]) + (" ..." if len(failing) > 15 else "")
    return s


def parse_clippy(text: str) -> str:
    warns = len(re.findall(r"^warning: ", text, re.M))
    errs = len(re.findall(r"^error(\[\w+\])?: ", text, re.M))
    return f"errors={errs} warnings={warns}"


def parse_criterion(text: str) -> str:
    # Zeilen wie:  name   time:   [12.3 ns 12.5 ns 12.7 ns]
    hits = re.findall(r"^(\S.*?)\s+time:\s+\[(.+?)\]", text, re.M)
    return f"{len(hits)} Messpunkte" if hits else "keine Criterion-Zeilen erkannt"


# ----------------------------------------------------------------------------------------------
# Plan aufbauen
# ----------------------------------------------------------------------------------------------

def workspace_crates(root: Path) -> list[str]:
    """Liest Paketnamen aus cargo metadata (statisch, ohne Build)."""
    try:
        out = subprocess.run(["cargo", "metadata", "--no-deps", "--format-version", "1"],
                             cwd=root, capture_output=True, text=True, timeout=300, check=True).stdout
        data = json.loads(out)
        return sorted(p["name"] for p in data["packages"])
    except Exception as e:  # Fallback: Verzeichnisnamen
        print(f"[warn] cargo metadata fehlgeschlagen ({e}); nutze Verzeichnisnamen", file=sys.stderr)
        names = sorted(p.name for p in (root / "crates").iterdir() if p.is_dir())
        return names + ["xtask", "contextra-bench"]


def build_plan(root: Path, args) -> list[Step]:
    crates = workspace_crates(root)
    if args.only:
        wanted = {w.strip().removeprefix("contextra-") for w in args.only.split(",") if w.strip()}
        crates = [c for c in crates if c.removeprefix("contextra-") in wanted]

    steps: list[Step] = []
    cargo = ["cargo"]

    # --- 0. Grundlage -------------------------------------------------------------------------
    steps += [
        Step("0-basis", "toolchain_versionen", ["bash", "-c", "rustc -Vv; cargo -V; cargo clippy -V; cargo fmt -V; "
              "cargo nextest --version; cargo deny --version; cargo audit --version; cargo fuzz --version; "
              "cargo llvm-cov --version; cargo mutants --version; cargo hack --version; true"], timeout=120),
        Step("0-basis", "cargo_fetch", cargo + ["fetch", "--locked"], timeout=900),
        Step("0-basis", "cargo_tree_duplicates", cargo + ["tree", "--workspace", "--duplicates"], timeout=300),
    ]

    # --- 1. Statische Pruefungen --------------------------------------------------------------
    steps += [
        Step("1-statisch", "fmt_check", cargo + ["fmt", "--all", "--", "--check"], requires=["rustfmt"]),
        Step("1-statisch", "check_workspace_alle_targets",
             cargo + ["check", "--workspace", "--all-targets", "--locked", "--keep-going"]),
        Step("1-statisch", "check_workspace_no_default_features",
             cargo + ["check", "--workspace", "--no-default-features", "--locked", "--keep-going"]),
        Step("1-statisch", "doc_workspace",
             cargo + ["doc", "--workspace", "--no-deps", "--locked"],
             env={"RUSTDOCFLAGS": "-D warnings"}),
    ]
    for c in crates:
        steps.append(Step("1-statisch", f"clippy_lib__{c}",
                          cargo + ["clippy", "-p", c, "--lib", "--locked", "--", "-D", "warnings"]))
        steps.append(Step("1-statisch", f"clippy_alle_targets__{c}",
                          cargo + ["clippy", "-p", c, "--all-targets", "--locked", "--", "-D", "warnings"]))
    steps += [
        Step("1-statisch", "feature_matrix_hack", cargo + ["hack", "check", "--each-feature", "--workspace",
             "--no-dev-deps", "--keep-going"], requires=["cargo-hack"], timeout=3600),
        Step("1-statisch", "deny", cargo + ["deny", "check"], requires=["cargo-deny"]),
        Step("1-statisch", "audit", cargo + ["audit"], requires=["cargo-audit"]),
        # Panic-/Unwrap-Flaeche des Nicht-Testcodes: Clippy-Lints nur auf lib-Targets
        Step("1-statisch", "panic_flaeche_lib",
             cargo + ["clippy", "--workspace", "--lib", "--locked", "--keep-going", "--",
                      "-W", "clippy::unwrap_used", "-W", "clippy::expect_used", "-W", "clippy::panic",
                      "-W", "clippy::todo", "-W", "clippy::unimplemented", "-W", "clippy::indexing_slicing"],
             timeout=3600, note="zaehlt echte Panic-Stellen im Produktivcode (ohne Tests)"),
        Step("1-statisch", "unsafe_inventar",
             ["bash", "-c", "grep -rn --include=*.rs -E '\\bunsafe\\b' crates xtask | "
              "grep -v '^[^:]*:[0-9]*:\\s*//' | wc -l && "
              "for d in crates/*/; do printf '%s ' \"$d\"; grep -rn --include=*.rs -E '\\bunsafe\\b' \"$d\" | "
              "grep -vc '^[^:]*:[0-9]*:\\s*//'; done"], timeout=120),
    ]

    # --- 2. Tests je Crate, fein aufgeteilt ----------------------------------------------------
    test_runner_nextest = which("cargo-nextest") is not None
    for c in crates:
        t = SLOW_CRATES.get(c, DEFAULT_TIMEOUT)
        steps += [
            Step("2-tests", f"test_lib__{c}", cargo + ["test", "-p", c, "--lib", "--locked", "--no-fail-fast"], timeout=t),
            Step("2-tests", f"test_integration__{c}",
                 cargo + ["test", "-p", c, "--tests", "--locked", "--no-fail-fast"], timeout=t,
                 note="Integrationstests unter tests/ (umfasst auch lib-Tests, Duplikate sind gewollt)"),
            Step("2-tests", f"test_doc__{c}", cargo + ["test", "-p", c, "--doc", "--locked", "--no-fail-fast"], timeout=t),
        ]
        if test_runner_nextest:
            steps.append(Step("2-tests", f"nextest_flaky__{c}",
                              cargo + ["nextest", "run", "-p", c, "--locked", "--no-fail-fast", "--retries", "0"],
                              timeout=t, note="Tests einzeln als Prozess: deckt Test-Interferenz auf"))
    steps.append(Step("2-tests", "test_workspace_gesamt",
                      cargo + ["test", "--workspace", "--all-targets", "--locked", "--no-fail-fast"],
                      timeout=7200, note="Gesamtlauf, falls Einzelschritte gruen sind aber Zusammenspiel bricht"))
    # Release-Modus (anderes Optimierungsverhalten, Integer-Overflow-Semantik!)
    steps.append(Step("2-tests", "test_workspace_release",
                      cargo + ["test", "--workspace", "--release", "--locked", "--no-fail-fast"], timeout=7200,
                      note="Release deckt Overflow-/Optimierungsfehler auf (z. B. simd u8)"))
    # Python-Bindings
    py = root / "crates" / "contextra-py"
    if (py / "tests").exists():
        steps.append(Step("2-tests", "python_bindings_pytest", ["python3", "-m", "pytest", "-q", str(py / "tests")],
                          requires=["python3"], note="nur sinnvoll, wenn Bindings per maturin installiert sind"))

    # --- 3. Gates des Projekts (xtask) ---------------------------------------------------------
    steps += [
        Step("3-gates", "xtask_gate_liste", cargo + ["run", "-q", "-p", "xtask", "--", "--list-gates"], timeout=900),
        Step("3-gates", "xtask_check_compile", cargo + ["run", "-q", "-p", "xtask", "--", "check-compile"], timeout=1800),
        Step("3-gates", "xtask_check_alle", cargo + ["run", "-q", "-p", "xtask", "--", "check", "--json"], timeout=7200,
             note="fuehrt alle Gates des Katalogs governance/gates.toml aus"),
    ]

    # --- 4. Benchmarks (je Ziel einzeln, Criterion) --------------------------------------------
    if not args.quick:
        for pkg, names in BENCHES.items():
            if args.only and pkg.removeprefix("contextra-") not in {w.strip().removeprefix("contextra-") for w in args.only.split(",")}:
                continue
            for b in names:
                steps.append(Step("4-bench", f"bench__{pkg}__{b}",
                                  cargo + ["bench", "-p", pkg, "--bench", b, "--locked"],
                                  timeout=BENCH_TIMEOUT, env={"CRITERION_HOME": str(root / "target" / "criterion")},
                                  note="Benchmarks ohne Referenzen (z. B. LoCoMo) koennen Daten/Netz brauchen"))
        # Vergleich Store vs redb ist der einzige echte Differenzmassstab im Repo (siehe Bench oben).

    # --- 5. Fuzz, Mutation, Sanitizer, Loom (nur mit --all) ------------------------------------
    if args.all:
        for fc in FUZZ_CRATES:
            fdir = root / "crates" / f"contextra-{fc}" / "fuzz"
            tdir = fdir / "fuzz_targets"
            if not tdir.exists():
                continue
            for tgt in sorted(p.stem for p in tdir.glob("*.rs")):
                steps.append(Step("5-tief", f"fuzz__{fc}__{tgt}",
                                  ["cargo", "+nightly", "fuzz", "run", tgt, "--", f"-max_total_time={FUZZ_SECONDS}"],
                                  cwd=root / "crates" / f"contextra-{fc}", requires=["cargo-fuzz"],
                                  timeout=FUZZ_SECONDS + 1200))
        steps += [
            Step("5-tief", "mutation_tier0_1", cargo + ["mutants", "--config", ".cargo/mutants.toml", "--timeout", "120"],
                 requires=["cargo-mutants"], timeout=14400, note="store, crypto, checkpoint (laut .cargo/mutants.toml)"),
            Step("5-tief", "loom_mvcc", cargo + ["test", "-p", "contextra-mvcc", "--release", "--locked"],
                 env={"RUSTFLAGS": "--cfg loom", "LOOM_MAX_PREEMPTIONS": "3"}, timeout=3600),
            Step("5-tief", "miri_vector_scalar", ["cargo", "+nightly", "miri", "test", "-p", "contextra-vector", "--lib"],
                 requires=["cargo-miri"], timeout=7200, note="SIMD-Intrinsics laufen unter Miri nicht"),
            Step("5-tief", "asan_vector", ["cargo", "+nightly", "test", "-p", "contextra-vector",
                                            "--target", "x86_64-unknown-linux-gnu"],
                 env={"RUSTFLAGS": "-Zsanitizer=address"}, timeout=3600),
            Step("5-tief", "coverage_workspace", cargo + ["llvm-cov", "--workspace", "--locked",
                 "--json", "--output-path", "coverage.json"], requires=["cargo-llvm-cov"], timeout=10800),
        ]
    return steps


# ----------------------------------------------------------------------------------------------
# Ausfuehrung
# ----------------------------------------------------------------------------------------------

def tool_available(step: Step) -> tuple[bool, str]:
    for r in step.requires:
        if r.startswith("cargo-"):
            sub = r.removeprefix("cargo-")
            if sub == "fuzz" or sub == "miri":
                if not which(r) and not which("cargo"):
                    return False, f"{r} nicht installiert"
            elif not which(r):
                return False, f"{r} nicht installiert (cargo install {r} --locked)"
        elif r == "rustfmt":
            if not which("rustfmt"):
                return False, "rustfmt fehlt (rustup component add rustfmt)"
        elif not which(r):
            return False, f"{r} nicht gefunden"
    return True, ""


def run_step(step: Step, idx: int, logdir: Path, root: Path) -> None:
    ok, why = tool_available(step)
    safe = re.sub(r"[^A-Za-z0-9_.-]", "_", step.name)
    step.log = str(logdir / f"{idx:03d}_{safe}.log")
    if not ok:
        step.status, step.detail = "SKIPPED", why
        Path(step.log).write_text(f"SKIPPED: {why}\n")
        return
    env = os.environ.copy()
    env.update(step.env)
    env.setdefault("CARGO_TERM_COLOR", "never")
    env.setdefault("RUST_BACKTRACE", "1")
    t0 = time.time()
    try:
        with open(step.log, "w") as fh:
            fh.write(f"$ {' '.join(step.cmd)}\n# cwd={step.cwd or root}\n# env+={step.env}\n\n")
            fh.flush()
            p = subprocess.run(step.cmd, cwd=step.cwd or root, env=env, stdout=fh, stderr=subprocess.STDOUT,
                               timeout=step.timeout, text=True)
        step.rc = p.returncode
        step.status = "PASS" if p.returncode == 0 else "FAIL"
    except subprocess.TimeoutExpired:
        step.status, step.rc, step.detail = "TIMEOUT", None, f">{step.timeout}s (zaehlt als rot)"
    except FileNotFoundError as e:
        step.status, step.detail = "SKIPPED", f"Befehl nicht gefunden: {e}"
    step.seconds = round(time.time() - t0, 1)

    text = Path(step.log).read_text(errors="replace")
    if step.name.startswith(("test_", "nextest_")):
        step.detail = (step.detail + " " + parse_cargo_test_summary(text)).strip()
        # Compilefehler sichtbar machen
        if "could not compile" in text:
            step.detail += " | COMPILE-FEHLER"
            step.status = "FAIL"
    elif step.name.startswith("clippy") or step.name == "panic_flaeche_lib":
        step.detail = (step.detail + " " + parse_clippy(text)).strip()
    elif step.name.startswith("bench__"):
        step.detail = (step.detail + " " + parse_criterion(text)).strip()
    elif step.name.startswith("check_"):
        step.detail = (step.detail + f" errors={len(re.findall(r'^error', text, re.M))}").strip()


def write_reports(steps: list[Step], outdir: Path, meta: dict) -> None:
    rows = [{"nr": i, "gruppe": s.group, "schritt": s.name, "status": s.status, "rc": s.rc,
             "sekunden": s.seconds, "detail": s.detail, "log": os.path.relpath(s.log, outdir),
             "befehl": " ".join(s.cmd), "hinweis": s.note} for i, s in enumerate(steps)]
    (outdir / "results.json").write_text(json.dumps({"meta": meta, "steps": rows}, indent=2, ensure_ascii=False))
    with open(outdir / "results.csv", "w", newline="") as fh:
        w = csv.DictWriter(fh, fieldnames=list(rows[0].keys()))
        w.writeheader(); w.writerows(rows)

    counts: dict[str, int] = {}
    for s in steps:
        counts[s.status] = counts.get(s.status, 0) + 1
    order = {"FAIL": 0, "TIMEOUT": 1, "SKIPPED": 2, "PASS": 3, "PENDING": 4}
    lines = [f"# Contextra Voll-Audit", "",
             f"- Commit: `{meta['commit']}`  ({meta['commit_date']})",
             f"- Start: {meta['start']}  Dauer: {meta['duration_min']} min",
             f"- Toolchain: {meta['rustc']}",
             f"- Host: {meta['host']}", "",
             "## Gesamt", ""]
    lines += [f"- **{k}**: {v}" for k, v in sorted(counts.items(), key=lambda kv: order.get(kv[0], 9))]
    lines += ["", "## Rote und uebersprungene Schritte", "",
              "| Schritt | Status | Sek | Detail | Log |", "|---|---|---:|---|---|"]
    for s in sorted(steps, key=lambda x: order.get(x.status, 9)):
        if s.status in ("FAIL", "TIMEOUT", "SKIPPED"):
            lines.append(f"| `{s.name}` | {s.status} | {s.seconds} | {s.detail} | `{os.path.relpath(s.log, outdir)}` |")
    lines += ["", "## Alle Schritte", "", "| Gruppe | Schritt | Status | Sek | Detail |", "|---|---|---|---:|---|"]
    for s in steps:
        lines.append(f"| {s.group} | `{s.name}` | {s.status} | {s.seconds} | {s.detail} |")
    (outdir / "summary.md").write_text("\n".join(lines) + "\n")


def collect_env(root: Path, outdir: Path) -> dict:
    def sh(cmd: str) -> str:
        try:
            return subprocess.run(cmd, shell=True, cwd=root, capture_output=True, text=True, timeout=60).stdout.strip()
        except Exception as e:
            return f"n/a ({e})"
    meta = {
        "commit": sh("git rev-parse --short HEAD") or "unknown",
        "commit_date": sh("git log -1 --format=%ci"),
        "branch": sh("git rev-parse --abbrev-ref HEAD"),
        "dirty": bool(sh("git status --porcelain")),
        "rustc": sh("rustc -V"),
        "host": f"{platform.platform()} | {os.cpu_count()} CPUs | "
                + sh("grep MemTotal /proc/meminfo 2>/dev/null"),
        "user": sh("id -un") + (" (ROOT: Read-only-Tests koennen falsch rot/gruen sein!)" if os.geteuid() == 0 else ""),
    }
    (outdir / "env.txt").write_text(
        "\n".join(f"{k}: {v}" for k, v in meta.items()) + "\n\n" + sh("rustup show 2>/dev/null") + "\n\n" + sh("uname -a"))
    return meta


def main() -> int:
    ap = argparse.ArgumentParser(description="Contextra: alles testen, benchmarken und Ergebnisse speichern")
    ap.add_argument("--quick", action="store_true", help="ohne Benchmarks, Fuzz, Mutation")
    ap.add_argument("--all", action="store_true", help="zusaetzlich Fuzz, Mutation, Miri, ASan, Loom, Coverage")
    ap.add_argument("--only", help="kommagetrennte Crates (Kurzname), z. B. store,ingest")
    ap.add_argument("--list", action="store_true", help="Plan anzeigen, nichts ausfuehren")
    ap.add_argument("--fail-fast", action="store_true", help="bei erstem FAIL abbrechen")
    ap.add_argument("--root", default=".", help="Repo-Root (Standard: aktuelles Verzeichnis)")
    args = ap.parse_args()

    root = Path(args.root).resolve()
    if not (root / "Cargo.toml").exists() or "[workspace]" not in (root / "Cargo.toml").read_text():
        print("Fehler: bitte im Workspace-Root ausfuehren (Cargo.toml mit [workspace]).", file=sys.stderr)
        return 2

    steps = build_plan(root, args)
    if args.list:
        for i, s in enumerate(steps):
            print(f"{i:03d} [{s.group}] {s.name:60s} {' '.join(s.cmd)}")
        print(f"\n{len(steps)} Schritte")
        return 0

    start = now_utc()
    commit = subprocess.run("git rev-parse --short HEAD", shell=True, cwd=root, capture_output=True, text=True).stdout.strip() or "nogit"
    outdir = root / "evidence" / f"audit-{start.strftime('%Y%m%dT%H%M%SZ')}-{commit}"
    (outdir / "logs").mkdir(parents=True, exist_ok=True)
    meta = collect_env(root, outdir)
    meta["start"] = start.isoformat(timespec="seconds")
    if os.geteuid() == 0:
        print("WARNUNG: Lauf als root. Tests zu Dateirechten sind dann nicht aussagekraeftig. Besser als normaler Nutzer.")

    print(f"{len(steps)} Schritte -> {outdir}\n")
    t_all = time.time()
    try:
        for i, s in enumerate(steps):
            print(f"[{i+1:03d}/{len(steps)}] {s.group:10s} {s.name} ...", end=" ", flush=True)
            run_step(s, i, outdir / "logs", root)
            print(f"{s.status} ({s.seconds}s) {s.detail[:100]}")
            # Zwischenstand sichern: bei Abbruch (Strg+C, Absturz) geht nichts verloren
            meta["duration_min"] = round((time.time() - t_all) / 60, 1)
            write_reports(steps, outdir, meta)
            if args.fail_fast and s.status in ("FAIL", "TIMEOUT"):
                print("Abbruch wegen --fail-fast"); break
    except KeyboardInterrupt:
        print("\nAbgebrochen. Zwischenergebnisse sind gespeichert.")

    # Criterion-Berichte sichern
    crit = root / "target" / "criterion"
    if crit.exists():
        shutil.copytree(crit, outdir / "criterion", dirs_exist_ok=True)
    meta["duration_min"] = round((time.time() - t_all) / 60, 1)
    write_reports(steps, outdir, meta)

    fails = [s for s in steps if s.status in ("FAIL", "TIMEOUT")]
    print(f"\nFertig in {meta['duration_min']} min. FAIL/TIMEOUT: {len(fails)}  -> {outdir}/summary.md")
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())
