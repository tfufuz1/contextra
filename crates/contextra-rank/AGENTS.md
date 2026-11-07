# AGENTS.md — `contextra-rank`

> **Ring 0** · Klasse `[K]` · Reifegrad `stable` · Unsafe: nein · Rolle: Rangfusion (RRF), Normalisierung, Explain/Provenienz, Score-Kalibrierung.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-rank --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` |
| Features | `dibud` |
| Lints | `#![forbid(unsafe_code)]`, `warn(missing_docs)` |

## 2. Abhängigkeiten & Schulden

- Ist = Soll. Dev-Kante auf `adapt` (M6-Kandidat: nach `testkit`/`tests` verlagern).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `fusion/` (`rrf`, `normalized`, `global`, `resonance`, `signal`, `topk`, `provenance`, `types`) | Standardfusion; **einziger Besitzer** von RRF (SPEC §4.4 Regel 10) |
| `explain.rs` | `RetrievalExplanation`, Modalität/Score/WAL-Seq |
| `calibration/` (`isotonic`, `platt`, `conformal`) | Kalibrierung (SPEC: Rand/Archiv) |
| `dibud/`, `drift.rs` | Feature `dibud`; Drift-Erkennung |

## 4. Verträge & Invarianten

1. **Eine Standardfusion:** `engine` ruft sie auf und implementiert keine zweite.
2. **Determinismus:** gleicher Input ⇒ gleiche Reihenfolge; Gleichstände mit stabilem, dokumentiertem Tie-Break (z. B. `DocId`).
3. Leere Ergebnislisten sind gültig; kein Indexzugriff auf `lists[0]` (bekannte Fehlerklasse).
4. Provenienz: jedes Ergebnis nennt Modalität, Rohscore und Quelle.
5. Keine Abhängigkeit auf `graph`/`adapt` (SPEC §4.4).

## 5. Ports

Konsumiert `ports`-Typen; stellt `Calibrator`-Implementierungen bereit.

## 6. Nebenläufigkeit & Performance

- Rein funktional und zustandslos; `O(Σ nᵢ log k)` für Top-k-Fusion (Komplexität im Doc-Kommentar belegen).
- Parallelisierung nur über deterministische Partitionierung.

## 7. Anti-Patterns

- `HashMap`-Iterationsreihenfolge als Ranking.
- Gleitkomma-Summation in wechselnder Reihenfolge.
- Fusion in `engine` duplizieren.

## 8. Tests & Messung

```bash
cargo test -p contextra-rank --locked
cargo clippy -p contextra-rank --all-targets -- -D warnings
```

- Property-Tests (Monotonie, Permutationsinvarianz), Referenzwerte, Fuzz-Ziel `rrf_fusion` (liegt unter `contextra-db`, Zuordnung prüfen).

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 5.3 | `infer-*` → `rank`-Kante prüfen |
| 6.1 | Beispiel `example-rag` nutzt `rank` isoliert |
