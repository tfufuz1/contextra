# AGENTS.md — `contextra-rank`

> **Normative Vorgabe gemäß [docs/SPEC.md §8.13](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md)**

## 1. Rolle & Zweck
`contextra-rank` ist für die deterministische Fusion von Trefferlisten aus Volltext- und Vektorsuche mittels Reciprocal Rank Fusion (RRF) zuständig.

## 2. Ring, Abhängigkeiten & Verbote
- **Ring:** Ring 2 (Ranking & Fusion)
- **Erlaubte Abhängigkeiten:** `contextra-types`, `contextra-ports`
- **Verbotene Abhängigkeiten:** Keine Abhängigkeiten zu `contextra-store`, `contextra-engine` oder externen ML-/Adapt-Crates.
- **Verbotene Muster:**
  - **Strikt kein `unsafe`** (`#![forbid(unsafe_code)]`).
  - Keine Hintergrund-Threads oder asynchronen Tasks.
  - Keine nicht-deterministischen Floating-Point-Operationen oder instabile Sortierungen.

## 3. Funktionsumfang & API (Soll-Zustand laut SPEC §8.13)

- **Kanonische Fusions-API:**
  ```rust
  pub fn rrf_fuse(lists: &[Vec<Hit>], k: u32, top: usize) -> Vec<Hit>;
  ```
  Standardwert für $k = 60$.
- **Deterministischer Tie-Break:** Bei identischen RRF-Scores erfolgt der Tie-Break strikt deterministisch nach aufsteigendem `doc_tag`.

> [!IMPORTANT]
> **Bereinigung veralteter Ranking-Module (SPEC §8.13):**
> Historische Kalibratoren (`IsotonicCalibrator`, Platt Scaling), adaptive Ranking-Strategien und alle Abhängigkeiten zu `adapt` werden vollständig entfernt. `contextra-rank` konzentriert sich ausschließlich auf die RRF-Standardfusion.

## 4. Kritische Invarianten
1. **Permutationsinvarianz:** Die Reihenfolge, in der Trefferlisten an `rrf_fuse` übergeben werden, beeinflusst weder die berechneten Scores noch die Ergebnisrangfolge.
2. **Determinismus:** Identische Eingabelisten liefern auf jeder unterstützten Architektur exakt dieselbe Byte-Ausgabe.

## 5. Crate-Arbeitspakete zum SOLL-Zustand (PLAN.md)

- **[ ] WP-1.2: Komplexe Kalibratoren entfernen**
  - Rückbau von `IsotonicCalibrator`, Conformal Calibration und externen Adaptiv-Abhängigkeiten.
- **[ ] WP-6.1: T-RANK-1 Property-Tests & Benchmarks**
  - Verifikation der RRF-Eigenschaften (Permutationsinvarianz, Tie-Breaker, handberechnete Referenzvektoren).

## 6. Verifikation & Benchmarks
- **T-RANK-1:** Property-Tests mit permutierten Eingabelisten und synthetischen Scores.
- **B-RRF:** Benchmark für Fusionsdurchsatz bei Listen bis 10.000 Einträgen.
