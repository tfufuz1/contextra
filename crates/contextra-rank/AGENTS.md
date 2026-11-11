# AGENTS.md — `contextra-rank`

> **Ring 0** · Klasse `[K]` · Reifegrad `stable` · Unsafe: nein · Rolle: Rangfusion (RRF), Normalisierung, Explain/Provenienz, Score-Kalibrierung, bi-temporale Gültigkeitsfilter (`temporal.rs`).
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-rank --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports` (optional), `types` (optional) via Feature `types-compat` |
| Contextra-Abhängigkeiten **Kern** (`--no-default-features`) | keine (Kern-Closure leer) |
| Features | `types-compat`, `exp-dibud`, `exp-resonance`, `experimental-all`, `experimental-rank` |
| Lints | `#![forbid(unsafe_code)]`, `warn(missing_docs)` |

## 2. Abhängigkeiten & Architektur

- Kern ist vollständig autark (Ring 0, E0).
- Kompatibilitätsbrücke liegt ausschließlich in `src/compat/` hinter dem Feature `types-compat`.
- Experimentelle Module (`dibud`, `resonance`) liegen hinter Features (`exp-dibud`, `exp-resonance`).
- Bi-temporale Gültigkeitsfilterung (`temporal.rs`) wurde aus `contextra-engine` übernommen.

## 3. Modul-Karte

| Pfad | Verantwortung |
|---|---|
| `fusion/` (`rrf`, `normalized`, `global`, `ext`, `signal`, `topk`, `provenance`, `types`, `resonance`) | Standardfusion; einziger Besitzer von RRF; Resonanz hinter `exp-resonance` |
| `temporal.rs` | Bi-temporale Filterung (`apply_temporal_validity_filter`, `ValidityWindow`, `TxId`) |
| `explain.rs` | `RetrievalExplanation`, Modalität/Score/WAL-Seq |
| `calibration/` (`isotonic`, `platt`, `conformal`) | Score-Kalibrierung (PAVA, Platt, konforme Prädiktion) |
| `drift.rs` | Score-Drift-Erkennung |
| `dibud/` | Streaming-Fusion hinter `exp-dibud` |
| `compat/` | Kompatibilitätsbrücke zu `contextra_types` und `contextra_ports` hinter `types-compat` |

## 4. Verträge & Invarianten

1. **Eine Standardfusion:** `engine` ruft sie auf und implementiert keine zweite.
2. **Determinismus:** Gleicher Input ⇒ gleiche Reihenfolge; Gleichstände mit stabilem Tie-Break (DocId aufsteigend).
3. Leere Ergebnislisten sind gültig; kein Indexzugriff auf `lists[0]`.
4. Provenienz: Jedes Ergebnis nennt Modalität, Rohscore und Quelle.
5. Keine unerlaubten Abhängigkeiten im Kern.
