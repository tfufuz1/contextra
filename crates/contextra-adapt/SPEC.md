# `contextra-adapt` – Spezifikation (SPEC v4.0 §9.2)

> **Klasse:** A (Autarker Baustein)
> **Ring:** 0 (Mathematischer Kontroll- & Adaptionskern)
> **Unsafe-Disziplin:** `#![forbid(unsafe_code)]`

---

## 1. Zweck und Baukasten-Architektur

`contextra-adapt` ist ein vollständig autarker mathematischer Baustein zur Laufzeit-Adaption und -Regelung:
1. **Latenzregelung:** PID-Latency-Controller und Rerank-Deadline-Manager zum Einhalten von Dienstgütevereinbarungen (SLAs) und dynamischen K-Pool-Anpassungen.
2. **Bandits & Exploration:** LinUCB (Sherman-Morrison $O(d^2)$, Sketched Projection $O(d \cdot k + k^2)$, Diagonal-Approximation $O(d)$), Flow-Corrected Thompson Sampling (FC-TS) mit Drift-Kompensation und RIE-Greedy-Personalisierung.
3. **Konzeptdrift & Stabilität:** Catoni M-Estimator für robuste Mittelwertschätzung unter Ausreißern, Lyapunov-Drift-Wächter und Ensemble-Drift-Watcher.
4. **Off-Policy & Schattenmodus:** Inverse Propensity Scoring (IPS) mit Positivitätsprüfung und Diskrepanz-Logging.
5. **Decay-Mechanismus:** Adaptive exponentielle Abklingregelung für Relevanz-Scores.

---

## 2. Invarianten und Garantien

- **INV-ADAPT-DETERMINISM (P28 / B11):** Alle Berechnungen und Zufallsprozesse sind streng deterministisch. Zufall wird über deterministische PRNGs (`SplitMix64`, `SeededRng`) abgewickelt, Zeit wird als Parameter oder über den lokalen `Clock`-Trait injiziert. Es existieren keine verdeckten Aufrufe von `SystemTime::now()` oder `thread_rng()` im Produktivcode.
- **INV-ADAPT-STABILITY (B9):** Sämtliche Algorithmen sind geschützt gegen Division durch 0, NaN, Unendlich und negative Varianzen. Alle Regler nutzen Anti-Windup Clamping (z. B. `k_min >= 50` Hard Floor, Integrator-Clamping `[-MAX_INTEGRAL, MAX_INTEGRAL]`).
- **INV-ADAPT-ISOLATION (B1, B3, B7):** Keine internen Abhängigkeiten im Kern (`may_depend_on = []`). Keine Fremdfachlogik (kein Wissen über Mandanten-IDs, Vektorindizes, Speichermodelle oder Löschprotokolle).
- **INV-ADAPT-ERROR (B4):** Einheitlicher, typisierter Fehler `AdaptError` (`#[non_exhaustive]`) ohne globale Abhängigkeit.

---

## 3. Konstanten und Parameter

| Konstante | Standardwert | Bedeutung |
|---|---|---|
| `DEFAULT_TARGET_LATENCY_MS` | `100.0` | Ziel-P95-Latenz für den PID-Latency-Controller |
| `MIN_SCALING_FACTOR` | `0.3` | Minimaler Skalierungsfaktor für K-Pool |
| `MAX_SCALING_FACTOR` | `1.0` | Maximaler Skalierungsfaktor für K-Pool |
| `MAX_INTEGRAL` | `10.0` | Anti-Windup Begrenzung des Integrator-Anteils |
| `SHERMAN_MORRISON_REFACTORIZATION_INTERVAL` | `1000` | Maximales Aktualisierungsintervall vor Drift-Meldung |
| `MIN_BANDIT_SHADOW_SAMPLES` | `10_000` | Schwellwert für Shadow-Mode Default-Flip |
| `MAX_CUMULATIVE_REGRET_THRESHOLD` | `0.0` | Regret-Schwelle für Default-Flip |

---

## 4. Fehlermodell

Das Fehlermodell wird über [`AdaptError`](crate::error::AdaptError) abgebildet:
- `DimensionMismatch`: Vektorlänge weicht vom Zustand ab.
- `InvalidConfig`: Parameter außerhalb des Definitionsbereichs.
- `NonFinite`: Auftreten von NaN oder Unendlich in numerischen Zwischenergebnissen.
- `PrecisionMatrixDriftDetected`: Numerischer Präzisionsverlust der Sherman-Morrison Inversen.
- `ZeroPropensityViolation`: Verstoß gegen die Positivitätsannahme im Off-Policy-Sampling.
- `InvalidInput`: Ungültige Eingabewerte.

---

## 5. Benchmarks & Performance

Messung auf isoliertem System (Criterion):
- `pid_update`: O(1) Aktualisierung, Sub-Mikrosekunden-Latenz.
- `bandit_sherman_morrison_score`: O(d²) Matrix-Vektor Skalarprodukt mit SIMD-freundlichem Unrolling.
- `fcts_sample_score`: O(d) allokationsfreies Thompson-Sampling.
