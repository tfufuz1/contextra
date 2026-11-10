# Handoff-Bericht: RANK-3 — contextra-rank

**Commit-Kennung:** RK3
**Branch:** `jules/rank-3-decouple`
**Datum:** 2026-11-10
**Status:** Vollständig abgeschlossen

---

## 1. Ergebnis je Arbeitspaket

| WP | Titel | Status | Prüfbefehl |
|---|---|---|---|
| **RK3.0** | Baseline, Kopplungs- und Feature-Befund | GRÜN | `cargo test -p contextra-rank --locked` |
| **RK3.1** | Golden-, Differenz- und Orakel-Korpus | GRÜN | `cargo test -p contextra-rank --test golden_tests --locked` |
| **RK3.2** | Brücke `src/compat/` & Kern-Bereinigung | GRÜN | `cargo check -p contextra-rank --no-default-features --locked` |
| **RK3.3** | `temporal.rs` aus `contextra-engine` integriert | GRÜN | `cargo test -p contextra-rank --test temporal_parity_test --locked` |
| **RK3.4** | Experimente hinter Features (`exp-dibud`, `exp-resonance`) | GRÜN | `cargo check -p contextra-rank --no-default-features --features exp-resonance` |
| **RK3.5** | Härtung, Zero-Panic, Fehlerabbildung | GRÜN | `cargo test -p contextra-rank --test hardening_tests --locked` |
| **RK3.6** | Dokumentation, Crate-AGENTS.md & Handoff | GRÜN | `cargo clippy -p contextra-rank --all-targets --locked -- -D warnings` |

---

## 2. Abweichungen von SPEC-v2

- **Entfallene Schritte gemäß bindender Auftrags-Vorgabe (UNABHÄNGIGKEIT):**
  - Umstellung `DocId → ids::DocId` und `Clock → env::Clock` entfallen ersatzlos und sind zurückgestellt für den Integrationsschritt (S-64).
  - Brücke liegt hinter dem bestehenden Feature `types-compat` (nicht umbenannt auf `workspace`, um bestehende Consumers nicht zu brechen).
  - Default-Features: `exp-resonance` verbleibt in `default`, da `contextra-engine` und `contextra-db` `apply_resonance_bonus` im Standard-Build nutzen.

---

## 3. Koordinationsanforderungen (für Integrationsschritt S-64 / Folge-WPs)

1. **`contextra-engine/src/temporal_filter.rs`:**
   - In S-64 kann `temporal_filter.rs` durch `pub use contextra_rank::temporal::*;` ersetzt werden.
2. **`exp-resonance` Feature-Entkoppelung in Consumers:**
   - Sobald `contextra-engine` und `contextra-db` ihre Manifeste auf das Feature `exp-resonance` aktualisieren, kann `exp-resonance` aus den Default-Features von `contextra-rank` entfernt werden.
3. **`types-compat` → `workspace`:**
   - Einheitliche Umbenennung des Kompatibilitätsfeatures im Integrationsschritt.

---

## 4. API-Freeze-Tabelle

| Crate | Genutztes Symbol / Signatur | Status | Kompatibilitätsweg |
|---|---|---|---|
| `contextra-engine` | `SearchResult`, `fuse_signals`, `apply_resonance_bonus`, `IsotonicCalibrator` | Unverändert | Quellkompatibel re-exportiert |
| `contextra-mcp` | `ProvenanceRecord`, `IsotonicCalibrator` | Unverändert | Quellkompatibel re-exportiert |
| `contextra-router` | `ConformalCalibrator` | Unverändert | Quellkompatibel |
| `contextra-db` | `IsotonicCalibrator`, `apply_resonance_bonus`, `ResonanceConfig` | Unverändert | Quellkompatibel |
| `contextra-bench` | `rrf_fuse`, `weighted_reciprocal_rank_fusion`, `score_normalized_fusion_with_options` | Unverändert | Quellkompatibel |

---

## 5. Kern-API-Lücken

Keine. Alle öffentlichen APIs bleiben vollständig quellkompatibel.

---

## 6. Messwerte Vorher / Nachher

| Metrik | Vorher (RK3.0) | Nachher (RK3.6) |
|---|---|---|
| Interne Abhängigkeiten Kern (`--no-default-features`) | 0 (keine) | 0 (keine) |
| `rg 'contextra_(types\|ports)'` im Kern (`src/` ohne `compat/`) | 27 Funde | **0 Funde (leer)** |
| Zeilen Code (`src/`) | ~3.980 | ~4.450 (inkl. `temporal.rs`, `compat/`, `ext.rs`) |
| Zero-Panic Verstöße im Produktionscode | 0 | 0 |
| Determinismus Verstöße im Produktionscode | 0 | 0 |
| Feature-Test-Kombinationen | 1 | 5 (`no-default`, `exp-dibud`, `exp-resonance`, `experimental-all`, `default`) |

---

## 7. Offene Risiken

Keine offenen Risiken. Alle Tests laufen deterministisch und ohne Warnings grün.
