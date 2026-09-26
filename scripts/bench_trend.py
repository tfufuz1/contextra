#!/usr/bin/env python3
"""
bench_trend.py
===============

Liest die von `cargo bench` (Criterion.rs) erzeugten `estimates.json`-Dateien
unter `target/criterion/**` aus, führt eine lesbare Trend-Historie in
`.jules/bench_history/history.jsonl` und meldet Regressionen gegenüber dem
letzten gespeicherten Lauf in einer für Menschen und Agenten gut lesbaren
Tabelle direkt im Terminal — ohne HTML-Report öffnen zu müssen.

Abgrenzung zu `xtask bench-gate`
---------------------------------
`xtask/src/bench_gate.rs` ist das **CI-Gate**: es vergleicht eine
`current_metrics.json` gegen eine eingecheckte `baseline_metrics.json` mit
hartem Pass/Fail für die Pipeline. Dieses Skript ist der **lokale,
interaktive Begleiter** dafür: es
  1. funktioniert direkt mit den rohen Criterion-Outputs (kein eigener
     Metrik-Export-Schritt nötig),
  2. hält eine fortlaufende Historie (nicht nur "aktuell vs. eine Baseline"),
     sodass ein Agent Performance-Trends über mehrere Commits sieht,
  3. gibt sofort eine kompakte Terminal-Tabelle statt einer CI-Fehlermeldung.

Workflow
--------
    cargo bench -p contextra-vector                     # Criterion läuft, schreibt target/criterion/
    python3 scripts/bench_trend.py --record              # Snapshot in Historie aufnehmen + Vergleich anzeigen
    python3 scripts/bench_trend.py --show                # Nur letzten Vergleich/Trend anzeigen (kein neuer Snapshot)
    python3 scripts/bench_trend.py --record --threshold 0.10   # Regressions-Schwelle auf 10% setzen (Default 5%)
    python3 scripts/bench_trend.py --record --fail-on-regression  # Exit-Code 1 bei Regression (für CI-Vorstufe)

Exit-Codes: 0 = ok / keine (relevanten) Regressionen, 1 = Regression(en)
            gefunden UND --fail-on-regression gesetzt, 2 = Ausführungsfehler.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
from pathlib import Path


def find_estimates(criterion_dir: Path) -> dict[str, float]:
    """Sammelt {benchmark_id: mean_point_estimate_nanoseconds} aus allen estimates.json."""
    results: dict[str, float] = {}
    if not criterion_dir.is_dir():
        return results
    for estimates_path in criterion_dir.rglob("new/estimates.json"):
        try:
            with open(estimates_path) as f:
                data = json.load(f)
            mean_ns = data["mean"]["point_estimate"]
        except (KeyError, json.JSONDecodeError, OSError):
            continue
        # benchmark_id aus Pfad ableiten: target/criterion/<group>/<bench>/new/estimates.json
        rel = estimates_path.relative_to(criterion_dir)
        bench_id = "/".join(rel.parts[:-2])  # alles vor 'new/estimates.json'
        results[bench_id] = mean_ns
    return results


def fmt_ns(ns: float) -> str:
    if ns >= 1_000_000_000:
        return f"{ns / 1_000_000_000:.3f} s"
    if ns >= 1_000_000:
        return f"{ns / 1_000_000:.3f} ms"
    if ns >= 1_000:
        return f"{ns / 1_000:.3f} µs"
    return f"{ns:.1f} ns"


def load_history(history_path: Path) -> list[dict]:
    if not history_path.is_file():
        return []
    entries = []
    with open(history_path) as f:
        for line in f:
            line = line.strip()
            if line:
                try:
                    entries.append(json.loads(line))
                except json.JSONDecodeError:
                    continue
    return entries


def append_history(history_path: Path, entry: dict) -> None:
    history_path.parent.mkdir(parents=True, exist_ok=True)
    with open(history_path, "a") as f:
        f.write(json.dumps(entry) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser(description="Criterion-Benchmark-Trend-Tracker für Contextra.")
    parser.add_argument("--criterion-dir", default="target/criterion",
                         help="Pfad zu den Criterion-Ausgaben (Standard: target/criterion).")
    parser.add_argument("--history-file", default=".jules/bench_history/history.jsonl",
                         help="Pfad zur JSONL-Historiendatei (Standard: .jules/bench_history/history.jsonl).")
    parser.add_argument("--record", action="store_true",
                         help="Aktuellen Criterion-Stand als neuen Snapshot in die Historie aufnehmen.")
    parser.add_argument("--show", action="store_true",
                         help="Nur den letzten gespeicherten Vergleich anzeigen, ohne neuen Snapshot.")
    parser.add_argument("--threshold", type=float, default=0.05,
                         help="Regressions-Schwellenwert als Anteil, z.B. 0.05 = 5%% (Standard: 0.05).")
    parser.add_argument("--fail-on-regression", action="store_true",
                         help="Exit-Code 1 setzen, wenn Regressionen über der Schwelle gefunden werden.")
    parser.add_argument("--label", default=None,
                         help="Optionales Label für diesen Snapshot (z.B. Commit-Hash oder Branch-Name).")
    args = parser.parse_args()

    criterion_dir = Path(args.criterion_dir)
    history_path = Path(args.history_file)
    history = load_history(history_path)

    if args.show:
        if not history:
            print("Keine Historie vorhanden. Zuerst mit --record einen Snapshot aufnehmen.")
            return 0
        last = history[-1]
        print(f"Letzter Snapshot: {last.get('label', '(kein Label)')} @ {last.get('timestamp', '?')}")
        print(f"{'Benchmark':<50} {'Mittelwert':>15}")
        for bench_id, ns in sorted(last["results"].items()):
            print(f"{bench_id:<50} {fmt_ns(ns):>15}")
        return 0

    current = find_estimates(criterion_dir)
    if not current:
        print(f"[error] Keine Criterion-Estimates unter '{criterion_dir}' gefunden. "
              f"Zuerst `cargo bench` ausführen.", file=sys.stderr)
        return 2

    previous = history[-1]["results"] if history else {}

    print(f"{'Benchmark':<50} {'Vorher':>15} {'Jetzt':>15} {'Δ':>10}  Status")
    print("-" * 100)

    regressions = []
    improvements = []
    new_benches = []

    for bench_id in sorted(set(current) | set(previous)):
        now_ns = current.get(bench_id)
        prev_ns = previous.get(bench_id)

        if now_ns is None:
            print(f"{bench_id:<50} {fmt_ns(prev_ns):>15} {'(entfernt)':>15} {'':>10}  ⚪ entfernt")
            continue
        if prev_ns is None:
            print(f"{bench_id:<50} {'(neu)':>15} {fmt_ns(now_ns):>15} {'':>10}  🆕 neu")
            new_benches.append(bench_id)
            continue

        delta = (now_ns - prev_ns) / prev_ns if prev_ns else 0.0
        sign = "+" if delta >= 0 else ""
        status = "✅ ok"
        if delta > args.threshold:
            status = "🔴 REGRESSION"
            regressions.append((bench_id, delta))
        elif delta < -args.threshold:
            status = "🟢 Verbesserung"
            improvements.append((bench_id, delta))

        print(f"{bench_id:<50} {fmt_ns(prev_ns):>15} {fmt_ns(now_ns):>15} {sign}{delta*100:>8.1f}%  {status}")

    print("-" * 100)
    print(f"Zusammenfassung: {len(regressions)} Regression(en) > {args.threshold*100:.0f}%, "
          f"{len(improvements)} Verbesserung(en), {len(new_benches)} neue Benchmark(s).")

    if regressions:
        print("\n🔴 Regressions-Details:")
        for bench_id, delta in regressions:
            print(f"   - {bench_id}: +{delta*100:.1f}% langsamer")

    if args.record:
        entry = {
            "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S"),
            "label": args.label or os.environ.get("GIT_COMMIT", "unlabeled"),
            "results": current,
        }
        append_history(history_path, entry)
        print(f"\n[gespeichert] Snapshot in {history_path} aufgenommen ({len(current)} Benchmarks).")

    if regressions and args.fail_on_regression:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
