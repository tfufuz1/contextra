# AGENTS.md — `contextra-bench`

> **Werkzeug** · Klasse Werkzeug (`benchmarks/`) · Reifegrad `–` · Unsafe: nein · Rolle: Reproduzierbare Benchmark-Harness: Recall@K, MRR, ANN-, BEIR-, Pfad-RAG-Läufe, Regressions-Gate.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-bench --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | (Workspace-Mitglied) |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | nie Abhängigkeit des Kerns |
| Features | – |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- Wird von keinem Kern-Crate benötigt; **enthält laut SPEC §4.2 `unsafe`** (Angabe prüfen, Crate setzt `forbid`).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `src/` (`ann_benchmarks`, `beir_eval`, `compare`, `path_rag_sweep`, `regression_gate`) | Läufe und Vergleich |
| `baseline_metrics.json`, `baseline_recall.json`, `benches/competitive_kv.rs` | Baselines, Wettbewerbsvergleich |

## 4. Verträge & Invarianten

1. Deterministische, synthetische Korpora mit annotierter Ground Truth; Ausgabe JSON und Markdown.
2. **Jede veröffentlichte Zahl nennt Commit, Modell-Hash, Hardware, Workload, Vergleichsbasis** (EP4, INV-CLAIMS-1). Fremde Baselines (z. B. `redb`) gehören dazu.
3. Benchmarks bleiben von Kernaussagen getrennt (SPEC §17.10).

## 5. Ports

Keine.

## 6. Nebenläufigkeit & Performance

- Messung mit Konfidenzintervall; Warm-up und Wiederholungen dokumentieren.

## 7. Anti-Patterns

- Einzelläufe als Beleg.
- Baseline stillschweigend überschreiben (Ratchet: nur besser).

## 8. Tests & Messung

```bash
cargo test -p contextra-bench --locked
cargo clippy -p contextra-bench --all-targets -- -D warnings
```

- `regression_gate.rs` + Workflow `bench-regression.yml`.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 0.4 | `baseline_20261106.json` einchecken |
