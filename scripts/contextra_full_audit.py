#!/usr/bin/env python3
"""
contextra_full_audit.py (v2) - Tests, Lints, Gates, Benchmarks, Fuzz, Befund-Pruefung, Produkt-Pruefung.

Aufruf im Repo-Root (dort, wo die Cargo.toml mit [workspace] liegt):

  python3 scripts/contextra_full_audit.py --list                 # Plan zeigen, nichts ausfuehren
  python3 scripts/contextra_full_audit.py --gruppe befunde       # nur schnelle statische Befund-Pruefung (kein Build)
  python3 scripts/contextra_full_audit.py --quick                # check, fmt, clippy, tests, gates (ohne Benchmarks)
  python3 scripts/contextra_full_audit.py --produkt              # nur die Produktmenge + SQLCipher-Probe + Tag-1-Pruefliste
  python3 scripts/contextra_full_audit.py                        # Standard: alles inkl. Benchmarks
  python3 scripts/contextra_full_audit.py --all                  # zusaetzlich Fuzz, Mutation, Loom, Miri, ASan, Coverage
  python3 scripts/contextra_full_audit.py --only store,ingest    # nur bestimmte Crates (Kurzname)
  python3 scripts/contextra_full_audit.py --resume evidence/audit-...   # bricht ab/neu starten: PASS-Schritte werden uebersprungen
  python3 scripts/contextra_full_audit.py --vergleich evidence/audit-A evidence/audit-B   # zwei Laeufe vergleichen

Weitere Optionen: --fail-fast, --jobs N, --timeout-faktor F, --gruppe PREFIX, --probe, --root PFAD

Ergebnisse: evidence/audit-<UTC>-<commit>/
  summary.md          Zusammenfassung (rote Schritte zuerst) + "Reife-Behauptung vs. Beleg"
  results.json/.csv   jeder Schritt mit Status, Dauer, Detail, Log
  panic_flaeche.csv   echte Panic-/Unwrap-Stellen im Produktivcode je Crate (Clippy, ohne Tests)
  bench_results.csv   Criterion-Messwerte (Mittel, Median, Streuung) je Benchmark
  logs/               voller Output je Schritt
  criterion/          kopierte Criterion-Berichte
  env.txt             Toolchain, Host, git-Stand

Nur Python-Standardbibliothek (3.11+). Der Quellcode des Projekts wird nie veraendert.
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
from collections import defaultdict
from pathlib import Path
from typing import Callable, Optional

try:
    import tomllib  # Python 3.11+
except ImportError:  # pragma: no cover
    tomllib = None  # type: ignore

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

SLOW_CRATES = {"contextra-crypto": 3600, "contextra-store": 3600, "contextra-vector": 3600,
               "contextra-graph": 3600, "contextra-engine": 3600}
DEFAULT_TIMEOUT = 1800
BENCH_TIMEOUT = 3600
FUZZ_SECONDS = 120


def now_utc() -> dt.datetime:
    return dt.datetime.now(dt.timezone.utc)


def which(name: str) -> Optional[str]:
    return shutil.which(name)


def short(name: str) -> str:
    return name.removeprefix("contextra-")


# ----------------------------------------------------------------------------------------------
# Step
# ----------------------------------------------------------------------------------------------

PyFn = Callable[[Path, Path], "tuple[str, str, str]"]  # (root, outdir) -> (status, detail, logtext)


class Step:
    def __init__(self, group: str, name: str, cmd: Optional[list[str]] = None, *, timeout: int = DEFAULT_TIMEOUT,
                 requires: Optional[list[str]] = None, env: Optional[dict] = None, cwd: Optional[Path] = None,
                 note: str = "", py: Optional[PyFn] = None):
        self.group, self.name, self.cmd = group, name, cmd or []
        self.timeout, self.requires = timeout, requires or []
        self.env, self.cwd, self.note, self.py = env or {}, cwd, note, py
        self.status, self.rc, self.seconds = "PENDING", None, 0.0
        self.log, self.detail = "", ""


# ----------------------------------------------------------------------------------------------
# Parser fuer Cargo-Ausgaben
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


def parse_clippy(text: str) -> str:
    warns = len(re.findall(r"^warning: (?!`)", text, re.M))
    errs = len(re.findall(r"^error(\[\w+\])?: ", text, re.M))
    return f"errors={errs} warnings={warns}"


def parse_criterion(text: str) -> str:
    hits = re.findall(r"^(\S.*?)\s+time:\s+\[(.+?)\]", text, re.M)
    return f"{len(hits)} Messpunkte" if hits else "keine Criterion-Zeilen erkannt"


PANIC_KINDS = [("unwrap", "unwrap()"), ("expect", "expect("), ("panic", "panic!"), ("todo", "todo!"),
               ("unimplemented", "unimplemented!"), ("index", "may panic")]


def parse_panic_surface(text: str) -> dict[str, dict[str, int]]:
    """Clippy-Warnungen (nur lib-Targets, ohne #[cfg(test)]) -> {crate: {art: anzahl}}"""
    out: dict[str, dict[str, int]] = defaultdict(lambda: defaultdict(int))
    for m in re.finditer(r"^warning: (.+)\n\s+--> ([^:\n]+):\d+:\d+", text, re.M):
        msg, path = m.group(1), m.group(2)
        kind = next((k for k, needle in PANIC_KINDS if needle in msg), None)
        if not kind:
            continue
        parts = path.split("/")
        crate = parts[1] if parts and parts[0] == "crates" and len(parts) > 1 else parts[0]
        out[crate][kind] += 1
    return out


# ----------------------------------------------------------------------------------------------
# Statische Befund-Pruefungen (reines Python, kein Build). Status: PASS = Befund liegt NICHT vor.
# ----------------------------------------------------------------------------------------------

def _read(p: Path) -> str:
    try:
        return p.read_text(errors="replace")
    except Exception:
        return ""


def _toml(p: Path) -> dict:
    if tomllib is None:
        return {}
    try:
        return tomllib.loads(_read(p))
    except Exception:
        return {}


def befund_cfg_features(root: Path, outdir: Path):
    """K1/K11: cfg(feature = "x") im Code, aber x ist in keiner Cargo.toml des Crates definiert."""
    lines, bad = [], 0
    for cdir in sorted((root / "crates").iterdir()):
        ct = _toml(cdir / "Cargo.toml")
        if not ct:
            continue
        feats = set((ct.get("features") or {}).keys())
        for dname, dv in (ct.get("dependencies") or {}).items():
            if isinstance(dv, dict) and dv.get("optional"):
                feats.add(dname)
        used: dict[str, int] = defaultdict(int)
        for rs in cdir.rglob("*.rs"):
            if "/target/" in str(rs) or "/fuzz/" in str(rs):
                continue
            for m in re.finditer(r'feature\s*=\s*"([^"]+)"', _read(rs)):
                used[m.group(1)] += 1
        undefined = {f: n for f, n in used.items() if f not in feats}
        if undefined:
            bad += 1
            lines.append(f"{cdir.name}: " + ", ".join(f"{f} ({n}x)" for f, n in sorted(undefined.items())))
        allow = _read(cdir / "src" / "lib.rs")
        if "unexpected_cfgs" in allow and "allow(unexpected_cfgs)" in allow:
            lines.append(f"{cdir.name}: #![allow(unexpected_cfgs)] verdeckt die Luecke")
    status = "FAIL" if bad else "PASS"
    detail = f"{bad} Crates mit undefinierten Features" if bad else "keine undefinierten Features"
    return status, detail, "\n".join(lines) or "ok"


def befund_wal_tail(root: Path, outdir: Path):
    """K3: widerspruechliche AEAD-Fehler-Regeln am WAL-Ende."""
    p = root / "crates/contextra-store/src/wal/replay.rs"
    t = _read(p)
    tol = [i + 1 for i, l in enumerate(t.splitlines()) if "Tolerated as incomplete tail write" in l]
    neg = [i + 1 for i, l in enumerate(t.splitlines()) if "never a tail" in l]
    if tol and neg:
        return "FAIL", f"tolerierend Z.{tol}, ablehnend Z.{neg}", "zwei widerspruechliche Regeln in replay.rs"
    return "PASS", "kein Widerspruch erkannt", ""


def befund_safe_unsafe_fns(root: Path, outdir: Path):
    """K7/K8: safe-Funktionen mit unsafe-Inhalt in sys."""
    hits = []
    for rs in (root / "crates/contextra-sys/src").rglob("*.rs"):
        for i, l in enumerate(_read(rs).splitlines()):
            if re.match(r"\s*pub fn (mmap_readonly|reopen_and_dup2)\b", l):
                hits.append(f"{rs.relative_to(root)}:{i+1}: {l.strip()}")
    return ("FAIL" if hits else "PASS"), f"{len(hits)} safe-Signaturen", "\n".join(hits)


def befund_ingest_wiring(root: Path, outdir: Path):
    """ingest: Abnehmer, Kante zu engine, undeklarierte Module."""
    consumers = []
    for ct in (root / "crates").glob("*/Cargo.toml"):
        if ct.parent.name != "contextra-ingest" and "contextra-ingest" in _read(ct):
            consumers.append(ct.parent.name)
    ing = root / "crates/contextra-ingest"
    deps_engine = "contextra-engine" in _read(ing / "Cargo.toml")
    refs = [str(p.relative_to(root)) for p in (ing / "src").rglob("*.rs") if "paged_impl" in _read(p)]
    declared = any((p.name in ("paged_impl.rs",) or p.name == "paged_impl") for p in (ing / "src").rglob("*"))
    declared = declared or any("mod paged_impl" in _read(p) for p in (ing / "src").rglob("*.rs"))
    problems = []
    if not consumers:
        problems.append("kein Crate haengt von contextra-ingest ab")
    if deps_engine:
        problems.append("ingest haengt an contextra-engine")
    if refs and not declared:
        problems.append(f"paged_impl benutzt aber nirgends deklariert ({', '.join(refs)})")
    return ("FAIL" if problems else "PASS"), "; ".join(problems) or "verdrahtet", f"consumers={consumers}"


def befund_default_members(root: Path, outdir: Path):
    ct = _toml(root / "Cargo.toml")
    ws = ct.get("workspace", {})
    dm = [Path(x).name for x in ws.get("default-members", [])]
    members = [Path(x).name for x in ws.get("members", [])]
    prod_missing = [c for c in PRODUKT if c not in dm]
    frozen_present = [c for c in dm if c in VERBOTEN_IM_PRODUKT]
    detail = f"members={len(members)} default-members={len(dm)}; Produkt fehlt in default-members: {prod_missing}; Kern in default-members: {len(frozen_present)}"
    return ("FAIL" if prod_missing else "PASS"), detail, ""


def befund_adr(root: Path, outdir: Path):
    d = root / "docs" / "decisions"
    nums, proposed = [], []
    for f in sorted(d.glob("[0-9][0-9][0-9][0-9]-*.md")):
        nums.append(f.name[:4])
        m = re.search(r"\*\*Status\*\*:\s*(\w+)", _read(f))
        if m and m.group(1).lower() == "proposed":
            proposed.append(f.name[:4])
    nxt = max(int(n) for n in nums if int(n) < 1000) + 1 if nums else 1
    return ("FAIL" if proposed else "PASS"), f"ADRs={nums}; Proposed={proposed}; naechste freie Nummer<1000: {nxt:04d}", ""


def befund_maturity_claims(root: Path, outdir: Path):
    ct = _toml(root / "capabilities.toml").get("crates", {})
    rows = [(k, v.get("maturity", "?")) for k, v in ct.items()]
    stable = [k for k, m in rows if m == "stable"]
    beta = [k for k, m in rows if m == "beta"]
    return "INFO", f"stable={len(stable)} beta={len(beta)}", f"stable: {stable}\nbeta: {beta}"


def befund_from_entropy(root: Path, outdir: Path):
    hits = sorted({str(p.relative_to(root)) for p in (root / "crates").rglob("*.rs")
                   if "from_entropy" in _read(p) and "/tests/" not in str(p) and "/benches/" not in str(p)})
    return ("FAIL" if hits else "PASS"), f"{len(hits)} Dateien mit from_entropy (K14)", "\n".join(hits)


def befund_windows_flush(root: Path, outdir: Path):
    t = "\n".join(_read(p) for p in (root / "crates/contextra-sys/src").rglob("*.rs"))
    has_cfg = "cfg(windows)" in t or 'target_os = "windows"' in t
    has_err = "compile_error!" in t
    return "INFO", f"windows-Pfad vorhanden={has_cfg}, compile_error!={has_err}", ""


# ----------------------------------------------------------------------------------------------
# Produkt-Pruefliste (Tag 1)
# ----------------------------------------------------------------------------------------------

def produkt_huelle(root: Path, outdir: Path):
    """Verbotene Crates in der transitiven Huelle der Produktmenge (cargo tree)."""
    log, bad = [], 0
    for c in PRODUKT:
        if not (root / "crates" / c).exists():
            log.append(f"{c}: fehlt im Repo"); continue
        p = subprocess.run(["cargo", "tree", "-p", c, "-e", "normal", "--prefix", "none", "--locked"],
                           cwd=root, capture_output=True, text=True, timeout=600)
        if p.returncode != 0:
            log.append(f"{c}: cargo tree fehlgeschlagen: {p.stderr.strip()[:200]}"); bad += 1; continue
        names = {l.split()[0] for l in p.stdout.splitlines() if l.strip()}
        found = sorted(n for n in names if n in VERBOTEN_IM_PRODUKT and n != c)
        if found:
            bad += 1
        log.append(f"{c}: verbotene Abhaengigkeiten={found if found else 'keine'}")
    return ("FAIL" if bad else "PASS"), f"{bad} Produkt-Crates mit verbotener Huelle", "\n".join(log)


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
        let sd: Result<i64, _> = c.query_row("PRAGMA secure_delete=ON", [], |r| r.get(0));
        println!("secure_delete={:?}", sd.ok());
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


def produkt_sqlcipher_probe(root: Path, outdir: Path):
    """P0-Gate: baut SQLCipher (rusqlite, vendored OpenSSL) und prueft Verschluesselung, FTS5, falschen Schluessel."""
    pdir = outdir / "sqlcipher_probe"
    (pdir / "src").mkdir(parents=True, exist_ok=True)
    (pdir / "Cargo.toml").write_text(
        '[package]\nname = "sqlcipher_probe"\nversion = "0.0.0"\nedition = "2024"\nrust-version = "1.89"\n\n'
        '[dependencies]\nrusqlite = { version = "*", features = ["bundled-sqlcipher-vendored-openssl"] }\n\n[workspace]\n')
    (pdir / "src" / "main.rs").write_text(SQLCIPHER_MAIN)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(root / "target" / "sqlcipher_probe_target")
    b = subprocess.run(["cargo", "run", "--release", "--manifest-path", str(pdir / "Cargo.toml"), "--",
                        str(pdir / "probe.db")], capture_output=True, text=True, env=env, timeout=3000)
    log = b.stdout + "\n--- stderr ---\n" + b.stderr[-6000:]
    if b.returncode != 0:
        return "FAIL", "Build oder Lauf der Probe fehlgeschlagen (siehe Log; ggf. perl/make/cc fehlen)", log
    kv = dict(re.findall(r"^(\w+)=(.*)$", b.stdout, re.M))
    ok = kv.get("plaintext_leak") == "false" and kv.get("wrong_key_rejected") == "true" and kv.get("right_key_rows") == "1"
    fts = kv.get("fts5") == "true"
    detail = (f"cipher={kv.get('cipher_version')} fts5={kv.get('fts5')} leak={kv.get('plaintext_leak')} "
              f"wrong_key_rejected={kv.get('wrong_key_rejected')} rows={kv.get('right_key_rows')}")
    if ok and not fts:
        return "FAIL", detail + " | FTS5 fehlt im SQLCipher-Build", log
    return ("PASS" if ok else "FAIL"), detail, log


# ----------------------------------------------------------------------------------------------
# Plan
# ----------------------------------------------------------------------------------------------

def workspace_crates(root: Path) -> list[str]:
    try:
        out = subprocess.run(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=root,
                             capture_output=True, text=True, timeout=300, check=True).stdout
        return sorted(p["name"] for p in json.loads(out)["packages"])
    except Exception as e:
        print(f"[warn] cargo metadata fehlgeschlagen ({e}); nutze Verzeichnisnamen", file=sys.stderr)
        names = sorted(p.name for p in (root / "crates").iterdir() if p.is_dir())
        return names + ["contextra-bench"]


def nextest_ok() -> bool:
    try:
        return subprocess.run(["cargo", "nextest", "--version"], capture_output=True, timeout=30).returncode == 0
    except Exception:
        return False


def build_plan(root: Path, args) -> list[Step]:
    crates = workspace_crates(root)
    if args.produkt:
        crates = [c for c in crates if c in PRODUKT]
    if args.only:
        wanted = {short(w.strip()) for w in args.only.split(",") if w.strip()}
        crates = [c for c in crates if short(c) in wanted]
    selected = set(crates)
    sel_all = not (args.only or args.produkt)

    S: list[Step] = []
    cargo = ["cargo"]

    # --- B: Befund-Pruefung (schnell, kein Build) --------------------------------------------
    S += [
        Step("B-befunde", "befund_cfg_features_undefiniert", py=befund_cfg_features, note="K1/K11"),
        Step("B-befunde", "befund_wal_aead_tail_widerspruch", py=befund_wal_tail, note="K3"),
        Step("B-befunde", "befund_safe_fn_mit_unsafe_inhalt", py=befund_safe_unsafe_fns, note="K7/K8"),
        Step("B-befunde", "befund_ingest_verdrahtung", py=befund_ingest_wiring),
        Step("B-befunde", "befund_default_members", py=befund_default_members),
        Step("B-befunde", "befund_adr_status", py=befund_adr),
        Step("B-befunde", "befund_from_entropy", py=befund_from_entropy, note="K14"),
        Step("B-befunde", "befund_maturity_claims", py=befund_maturity_claims),
        Step("B-befunde", "befund_windows_flush", py=befund_windows_flush, note="K10"),
    ]

    # --- T1: Produkt-Pruefliste --------------------------------------------------------------
    if args.produkt or args.all or args.probe:
        S += [
            Step("T1-produkt", "produkt_huelle_ohne_kern", py=produkt_huelle, timeout=1800,
                 note="Produktmenge darf engine/db/store/graph/... nicht transitiv ziehen"),
            Step("T1-produkt", "produkt_sqlcipher_probe", py=produkt_sqlcipher_probe, timeout=3600,
                 note="P0-Gate: SQLCipher baut, verschluesselt, FTS5 vorhanden, falscher Schluessel scheitert"),
            Step("T1-produkt", "produkt_crates_io_versionen",
                 ["bash", "-c", "for c in rusqlite axum clap tokio tower-http argon2 hkdf serde_json; do "
                  "cargo search $c --limit 1 2>&1 | head -1; done"], timeout=300),
        ]

    # --- 0: Basis ----------------------------------------------------------------------------
    S += [
        Step("0-basis", "toolchain_versionen", ["bash", "-c", "rustc -Vv; cargo -V; cargo clippy -V; cargo fmt -V; "
             "cargo nextest --version; cargo deny --version; cargo audit --version; cargo fuzz --version; "
             "cargo llvm-cov --version; cargo mutants --version; cargo hack --version; true"], timeout=120),
        Step("0-basis", "cargo_fetch", cargo + ["fetch", "--locked"], timeout=900),
        Step("0-basis", "cargo_tree_duplicates", cargo + ["tree", "--workspace", "--duplicates"], timeout=300),
    ]

    # --- 1: Statisch -------------------------------------------------------------------------
    pk = [x for c in crates for x in ("-p", c)] if not sel_all else ["--workspace"]
    S += [
        Step("1-statisch", "fmt_check", cargo + ["fmt", "--all", "--", "--check"], requires=["rustfmt"]),
        Step("1-statisch", "check_alle_targets", cargo + ["check"] + pk + ["--all-targets", "--locked", "--keep-going"]),
        Step("1-statisch", "check_no_default_features",
             cargo + ["check"] + pk + ["--no-default-features", "--locked", "--keep-going"]),
        Step("1-statisch", "doc", cargo + ["doc"] + pk + ["--no-deps", "--locked"], env={"RUSTDOCFLAGS": "-D warnings"}),
    ]
    for c in crates:
        S.append(Step("1-statisch", f"clippy_lib__{c}", cargo + ["clippy", "-p", c, "--lib", "--locked", "--", "-D", "warnings"]))
        S.append(Step("1-statisch", f"clippy_alle_targets__{c}",
                      cargo + ["clippy", "-p", c, "--all-targets", "--locked", "--", "-D", "warnings"]))
    S += [
        Step("1-statisch", "feature_matrix_hack", cargo + ["hack", "check", "--each-feature"] + pk +
             ["--no-dev-deps", "--keep-going"], requires=["cargo-hack"], timeout=3600),
        Step("1-statisch", "deny", cargo + ["deny", "check"], requires=["cargo-deny"]),
        Step("1-statisch", "audit", cargo + ["audit"], requires=["cargo-audit"]),
        Step("1-statisch", "panic_flaeche_lib",
             cargo + ["clippy"] + pk + ["--lib", "--locked", "--keep-going", "--",
                      "-W", "clippy::unwrap_used", "-W", "clippy::expect_used", "-W", "clippy::panic",
                      "-W", "clippy::todo", "-W", "clippy::unimplemented", "-W", "clippy::indexing_slicing"],
             timeout=3600, note="echte Panic-Stellen im Produktivcode (ohne #[cfg(test)]); Auswertung: panic_flaeche.csv"),
        Step("1-statisch", "unsafe_inventar",
             ["bash", "-c", "for d in crates/*/; do printf '%s ' \"$d\"; grep -rn --include=*.rs -E '\\bunsafe\\b' \"$d\" | "
              "grep -vc '^[^:]*:[0-9]*:\\s*//'; done"], timeout=120),
    ]

    # --- 2: Tests ----------------------------------------------------------------------------
    use_nextest = nextest_ok()
    for c in crates:
        t = SLOW_CRATES.get(c, DEFAULT_TIMEOUT)
        S += [
            Step("2-tests", f"test_lib__{c}", cargo + ["test", "-p", c, "--lib", "--locked", "--no-fail-fast"], timeout=t),
            Step("2-tests", f"test_integration__{c}", cargo + ["test", "-p", c, "--tests", "--locked", "--no-fail-fast"],
                 timeout=t, note="umfasst auch lib-Tests; Duplikate gewollt"),
            Step("2-tests", f"test_doc__{c}", cargo + ["test", "-p", c, "--doc", "--locked", "--no-fail-fast"], timeout=t),
        ]
        if use_nextest:
            S.append(Step("2-tests", f"nextest__{c}", cargo + ["nextest", "run", "-p", c, "--locked", "--no-fail-fast",
                     "--retries", "0"], timeout=t, note="je Test eigener Prozess: deckt Test-Interferenz auf"))
    S.append(Step("2-tests", "test_gesamt_debug", cargo + ["test"] + pk + ["--all-targets", "--locked", "--no-fail-fast"], timeout=7200))
    S.append(Step("2-tests", "test_gesamt_release", cargo + ["test"] + pk + ["--release", "--locked", "--no-fail-fast"],
                  timeout=7200, note="Release deckt Overflow-/Optimierungsfehler auf (z. B. simd u8)"))
    py = root / "crates" / "contextra-py"
    if sel_all and (py / "tests").exists():
        S.append(Step("2-tests", "python_bindings_pytest", ["python3", "-m", "pytest", "-q", str(py / "tests")],
                      requires=["python3"], note="setzt installierte Bindings voraus (maturin develop)"))


    # --- 4: Benchmarks -----------------------------------------------------------------------
    if not args.quick and not args.produkt:
        for pkg, names in BENCHES.items():
            if pkg not in selected and not (sel_all and pkg == "contextra-bench"):
                continue
            for b in names:
                S.append(Step("4-bench", f"bench__{pkg}__{b}", cargo + ["bench", "-p", pkg, "--bench", b, "--locked"],
                              timeout=BENCH_TIMEOUT, note="locomo/long_mem_eval brauchen evtl. externe Daten"))

    # --- 5: Tief -----------------------------------------------------------------------------
    if args.all:
        for fc in FUZZ_CRATES:
            if args.only and fc not in {short(w.strip()) for w in args.only.split(",")}:
                continue
            tdir = root / "crates" / f"contextra-{fc}" / "fuzz" / "fuzz_targets"
            if not tdir.exists():
                continue
            for tgt in sorted(p.stem for p in tdir.glob("*.rs")):
                S.append(Step("5-tief", f"fuzz__{fc}__{tgt}",
                              ["cargo", "+nightly", "fuzz", "run", tgt, "--", f"-max_total_time={FUZZ_SECONDS}"],
                              cwd=root / "crates" / f"contextra-{fc}", requires=["cargo-fuzz"], timeout=FUZZ_SECONDS + 1200))
        S += [
            Step("5-tief", "mutation_tier0_1", cargo + ["mutants", "--config", ".cargo/mutants.toml", "--timeout", "120"],
                 requires=["cargo-mutants"], timeout=14400),
            Step("5-tief", "loom_mvcc", cargo + ["test", "-p", "contextra-mvcc", "--release", "--locked"],
                 env={"RUSTFLAGS": "--cfg loom", "LOOM_MAX_PREEMPTIONS": "3"}, timeout=3600),
            Step("5-tief", "miri_vector_scalar", ["cargo", "+nightly", "miri", "test", "-p", "contextra-vector", "--lib"],
                 requires=["cargo-miri"], timeout=7200),
            Step("5-tief", "asan_vector", ["cargo", "+nightly", "test", "-p", "contextra-vector", "--target",
                 "x86_64-unknown-linux-gnu"], env={"RUSTFLAGS": "-Zsanitizer=address"}, timeout=3600),
            Step("5-tief", "coverage", cargo + ["llvm-cov"] + pk + ["--locked", "--json", "--output-path", "coverage.json"],
                 requires=["cargo-llvm-cov"], timeout=10800),
        ]

    if args.gruppe:
        S = [s for s in S if s.group.lower().startswith(args.gruppe.lower()) or s.group.split("-", 1)[-1].startswith(args.gruppe.lower())]
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


def run_step(step: Step, idx: int, logdir: Path, root: Path, outdir: Path, tfactor: float, jobs: Optional[int]) -> None:
    safe = re.sub(r"[^A-Za-z0-9_.-]", "_", step.name)
    step.log = str(logdir / f"{idx:03d}_{safe}.log")
    ok, why = tool_available(step)
    if not ok:
        step.status, step.detail = "SKIPPED", why
        Path(step.log).write_text(f"SKIPPED: {why}\n")
        return
    t0 = time.time()
    if step.py:
        try:
            status, detail, text = step.py(root, outdir)
        except subprocess.TimeoutExpired:
            status, detail, text = "TIMEOUT", "Zeitlimit", ""
        except Exception as e:  # Pruefung selbst defekt: sichtbar machen, nicht verschweigen
            status, detail, text = "FAIL", f"Pruefung abgestuerzt: {type(e).__name__}: {e}", ""
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
    try:
        with open(step.log, "w") as fh:
            fh.write(f"$ {' '.join(step.cmd)}\n# cwd={step.cwd or root}\n# env+={step.env}\n\n")
            fh.flush()
            p = subprocess.run(step.cmd, cwd=step.cwd or root, env=env, stdout=fh, stderr=subprocess.STDOUT,
                               timeout=timeout, text=True)
        step.rc = p.returncode
        step.status = "PASS" if p.returncode == 0 else "FAIL"
    except subprocess.TimeoutExpired:
        step.status, step.detail = "TIMEOUT", f">{timeout}s (zaehlt als rot)"
    except FileNotFoundError as e:
        step.status, step.detail = "SKIPPED", f"Befehl nicht gefunden: {e}"
    step.seconds = round(time.time() - t0, 1)

    text = Path(step.log).read_text(errors="replace").split("\n\n", 1)[-1]  # Kopfzeile mit Befehl nicht mitparsen
    if step.name.startswith(("test_", "nextest")):
        step.detail = (step.detail + " " + parse_cargo_test_summary(text)).strip()
        if "could not compile" in text:
            step.detail += " | COMPILE-FEHLER"
            step.status = "FAIL"
        elif step.status == "PASS" and re.search(r"test result: \w+\. 0 passed; 0 failed", text) and \
                not re.search(r"test result: \w+\. [1-9]\d* passed", text):
            step.detail += " | KEINE TESTS AUSGEFUEHRT"
    elif step.name.startswith("clippy") or step.name == "panic_flaeche_lib":
        step.detail = (step.detail + " " + parse_clippy(text)).strip()
        if step.name == "panic_flaeche_lib":
            step.status = "INFO" if step.status == "PASS" else step.status
    elif step.name.startswith("bench__"):
        step.detail = (step.detail + " " + parse_criterion(text)).strip()
    elif step.name.startswith("check_"):
        step.detail = (step.detail + f" errors={len(re.findall(r'^error', text, re.M))}").strip()


# ----------------------------------------------------------------------------------------------
# Auswertungen
# ----------------------------------------------------------------------------------------------

def export_panic_surface(steps: list[Step], outdir: Path) -> Optional[dict]:
    s = next((x for x in steps if x.name == "panic_flaeche_lib" and x.log and Path(x.log).exists()), None)
    if not s:
        return None
    data = parse_panic_surface(Path(s.log).read_text(errors="replace"))
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
        except Exception:
            continue
    if rows:
        with open(outdir / "bench_results.csv", "w", newline="") as fh:
            w = csv.writer(fh)
            w.writerow(["benchmark", "mean_ns", "median_ns", "stddev_ns", "mean_ci_low_ns", "mean_ci_high_ns"])
            w.writerows(sorted(rows))
    return len(rows)


def maturity_table(root: Path, steps: list[Step]) -> list[str]:
    caps = _toml(root / "capabilities.toml").get("crates", {})
    by = {s.name: s for s in steps}
    lines = ["", "## Reife-Behauptung vs. Beleg (capabilities.toml gegen diesen Lauf)", "",
             "| Crate | behauptet | lib-Tests | Clippy lib | Urteil |", "|---|---|---|---|---|"]
    for name, v in sorted(caps.items()):
        claimed = v.get("maturity", "?")
        t = by.get(f"test_lib__{name}")
        c = by.get(f"clippy_lib__{name}")
        if not t and not c:
            continue
        ts = t.status if t else "-"
        cs = c.status if c else "-"
        if claimed in ("stable", "beta") and (ts != "PASS" or cs != "PASS"):
            verdict = "**Behauptung ohne Beleg**"
        else:
            verdict = "ok" if ts == "PASS" and cs == "PASS" else "-"
        lines.append(f"| `{name}` | {claimed} | {ts} | {cs} | {verdict} |")
    return lines


def write_reports(steps: list[Step], outdir: Path, meta: dict, root: Path, final: bool = False) -> None:
    rows = [{"nr": i, "gruppe": s.group, "schritt": s.name, "status": s.status, "rc": s.rc, "sekunden": s.seconds,
             "detail": s.detail, "log": os.path.relpath(s.log, outdir) if s.log else "",
             "befehl": " ".join(s.cmd), "hinweis": s.note} for i, s in enumerate(steps)]
    (outdir / "results.json").write_text(json.dumps({"meta": meta, "steps": rows}, indent=2, ensure_ascii=False))
    with open(outdir / "results.csv", "w", newline="") as fh:
        w = csv.DictWriter(fh, fieldnames=list(rows[0].keys())) if rows else None
        if w:
            w.writeheader(); w.writerows(rows)

    counts: dict[str, int] = {}
    for s in steps:
        counts[s.status] = counts.get(s.status, 0) + 1
    order = {"FAIL": 0, "TIMEOUT": 1, "SKIPPED": 2, "INFO": 3, "PASS": 4, "PENDING": 5}
    L = ["# Contextra Voll-Audit", "",
         f"- Commit: `{meta['commit']}` ({meta['commit_date']}), Branch `{meta.get('branch','')}`, "
         f"hinter origin: {meta.get('behind','?')}, ungespeicherte Aenderungen: {meta.get('dirty')}",
         f"- Start: {meta['start']}  Dauer: {meta['duration_min']} min",
         f"- Toolchain: {meta['rustc']}", f"- Host: {meta['host']}", f"- Nutzer: {meta['user']}", "",
         "## Gesamt", ""]
    L += [f"- **{k}**: {v}" for k, v in sorted(counts.items(), key=lambda kv: order.get(kv[0], 9))]
    L += ["", "Hinweis: Bei `befund_*`-Schritten bedeutet FAIL, dass der Befund **vorliegt**. "
          "`INFO` ist eine Messung ohne Bestanden/Durchgefallen.", "",
          "## Rote und uebersprungene Schritte", "", "| Schritt | Status | Sek | Detail | Log |", "|---|---|---:|---|---|"]
    for s in sorted(steps, key=lambda x: order.get(x.status, 9)):
        if s.status in ("FAIL", "TIMEOUT", "SKIPPED"):
            L.append(f"| `{s.name}` | {s.status} | {s.seconds} | {s.detail} | `{os.path.relpath(s.log, outdir) if s.log else ''}` |")
    if final:
        data = export_panic_surface(steps, outdir)
        if data:
            L += ["", "## Panic-Flaeche im Produktivcode (Clippy, nur lib, ohne Tests)", "",
                  "| Crate | unwrap | expect | panic | todo | index | Summe |", "|---|---:|---:|---:|---:|---:|---:|"]
            for crate, k in sorted(data.items(), key=lambda kv: -sum(kv[1].values()))[:25]:
                L.append(f"| `{crate}` | {k.get('unwrap',0)} | {k.get('expect',0)} | {k.get('panic',0)} | "
                         f"{k.get('todo',0)} | {k.get('index',0)} | {sum(k.values())} |")
        nb = export_bench_csv(root, outdir, meta["_t0"])
        if nb:
            L += ["", f"## Benchmarks: {nb} Messwerte in `bench_results.csv`"]
        L += maturity_table(root, steps)
    L += ["", "## Alle Schritte", "", "| Gruppe | Schritt | Status | Sek | Detail |", "|---|---|---|---:|---|"]
    for s in steps:
        L.append(f"| {s.group} | `{s.name}` | {s.status} | {s.seconds} | {s.detail} |")
    (outdir / "summary.md").write_text("\n".join(L) + "\n")


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
        "behind": sh("git rev-list --count HEAD..@{u} 2>/dev/null") or "?",
        "dirty": bool(sh("git status --porcelain")),
        "rustc": sh("rustc -V"),
        "host": f"{platform.platform()} | {os.cpu_count()} CPUs | " + sh("grep MemTotal /proc/meminfo 2>/dev/null"),
        "user": sh("id -un") + (" (ROOT: Read-only-Tests sind dann nicht aussagekraeftig!)" if os.geteuid() == 0 else ""),
    }
    (outdir / "env.txt").write_text("\n".join(f"{k}: {v}" for k, v in meta.items()) + "\n\n"
                                    + sh("rustup show 2>/dev/null") + "\n\n" + sh("uname -a"))
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
    out = [f"# Vergleich", "", f"- A: `{ma['commit']}` ({a.name})", f"- B: `{mb['commit']}` ({b.name})", "",
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
        if name.startswith("test_lib__"):
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
    ap = argparse.ArgumentParser(description="Contextra: alles testen, benchmarken, Befunde pruefen, Ergebnisse speichern")
    ap.add_argument("--quick", action="store_true", help="ohne Benchmarks, Fuzz, Mutation")
    ap.add_argument("--all", action="store_true", help="zusaetzlich Fuzz, Mutation, Miri, ASan, Loom, Coverage, Produkt-Probe")
    ap.add_argument("--produkt", action="store_true", help="nur Produktmenge + Tag-1-Pruefliste + SQLCipher-Probe")
    ap.add_argument("--probe", action="store_true", help="Tag-1-Pruefliste (Huelle, SQLCipher, crates.io) zusaetzlich ausfuehren")
    ap.add_argument("--only", help="kommagetrennte Crates (Kurzname), z. B. store,ingest")
    ap.add_argument("--gruppe", help="nur Gruppe, z. B. befunde, statisch, tests, bench, produkt, gates, tief")
    ap.add_argument("--list", action="store_true", help="Plan anzeigen, nichts ausfuehren")
    ap.add_argument("--fail-fast", action="store_true")
    ap.add_argument("--jobs", type=int, help="CARGO_BUILD_JOBS begrenzen (z. B. bei wenig RAM)")
    ap.add_argument("--timeout-faktor", type=float, default=1.0, help="alle Zeitlimits multiplizieren")
    ap.add_argument("--resume", help="bestehendes evidence/audit-... Verzeichnis fortsetzen (PASS-Schritte werden uebersprungen)")
    ap.add_argument("--vergleich", nargs=2, metavar=("A", "B"), help="zwei Lauf-Verzeichnisse vergleichen")
    ap.add_argument("--root", default=".", help="Repo-Root (Standard: aktuelles Verzeichnis)")
    args = ap.parse_args()

    if args.vergleich:
        return compare_runs(Path(args.vergleich[0]), Path(args.vergleich[1]))

    root = Path(args.root).resolve()
    if not (root / "Cargo.toml").exists() or "[workspace]" not in (root / "Cargo.toml").read_text():
        print("Fehler: bitte im Workspace-Root ausfuehren (Cargo.toml mit [workspace]).", file=sys.stderr)
        return 2

    steps = build_plan(root, args)
    if args.list:
        for i, s in enumerate(steps):
            print(f"{i:03d} [{s.group}] {s.name:62s} {' '.join(s.cmd) if s.cmd else '(Python-Pruefung)'}")
        print(f"\n{len(steps)} Schritte")
        return 0
    if not steps:
        print("Keine Schritte ausgewaehlt."); return 2

    start = now_utc()
    t_all = time.time()
    commit = subprocess.run("git rev-parse --short HEAD", shell=True, cwd=root, capture_output=True, text=True).stdout.strip() or "nogit"
    previous: dict[str, dict] = {}
    if args.resume:
        outdir = Path(args.resume).resolve()
        rj = outdir / "results.json"
        if rj.exists():
            previous = {s["schritt"]: s for s in json.loads(rj.read_text())["steps"]}
    else:
        outdir = root / "evidence" / f"audit-{start.strftime('%Y%m%dT%H%M%SZ')}-{commit}"
    (outdir / "logs").mkdir(parents=True, exist_ok=True)
    meta = collect_env(root, outdir)
    meta["start"] = start.isoformat(timespec="seconds")
    meta["_t0"] = t_all - 5
    if meta["behind"] not in ("", "0", "?"):
        print(f"WARNUNG: HEAD liegt {meta['behind']} Commits hinter origin. Erst 'git pull', wenn du den neuesten Stand willst.")
    if meta["dirty"]:
        print("HINWEIS: Arbeitsbaum hat ungespeicherte Aenderungen; sie fliessen in die Ergebnisse ein.")
    if os.geteuid() == 0:
        print("WARNUNG: Lauf als root. Tests zu Dateirechten sind dann nicht aussagekraeftig.")

    print(f"{len(steps)} Schritte -> {outdir}\n")
    try:
        for i, s in enumerate(steps):
            prev = previous.get(s.name)
            if prev and prev["status"] in ("PASS", "INFO"):
                s.status, s.detail, s.seconds = prev["status"], (prev["detail"] + " (uebernommen)").strip(), prev["sekunden"]
                s.log = str(outdir / prev["log"]) if prev["log"] else ""
                print(f"[{i+1:03d}/{len(steps)}] {s.group:10s} {s.name} ... uebersprungen ({s.status}, aus frueherem Lauf)")
                continue
            print(f"[{i+1:03d}/{len(steps)}] {s.group:10s} {s.name} ...", end=" ", flush=True)
            run_step(s, i, outdir / "logs", root, outdir, args.timeout_faktor, args.jobs)
            print(f"{s.status} ({s.seconds}s) {s.detail[:110]}")
            meta["duration_min"] = round((time.time() - t_all) / 60, 1)
            write_reports(steps, outdir, meta, root)  # Zwischenstand: nichts geht bei Abbruch verloren
            if args.fail_fast and s.status in ("FAIL", "TIMEOUT"):
                print("Abbruch wegen --fail-fast"); break
    except KeyboardInterrupt:
        print("\nAbgebrochen. Zwischenergebnisse sind gespeichert. Fortsetzen mit --resume", outdir)

    crit = root / "target" / "criterion"
    if crit.exists() and any(s.group == "4-bench" for s in steps):
        shutil.copytree(crit, outdir / "criterion", dirs_exist_ok=True)
    meta["duration_min"] = round((time.time() - t_all) / 60, 1)
    write_reports(steps, outdir, meta, root, final=True)

    red = [s for s in steps if s.status in ("FAIL", "TIMEOUT")]
    print(f"\nFertig in {meta['duration_min']} min. FAIL/TIMEOUT: {len(red)}  -> {outdir}/summary.md")
    return 1 if red else 0


if __name__ == "__main__":
    sys.exit(main())
