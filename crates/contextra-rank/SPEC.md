# `contextra-rank` — Spezifikation (SPEC v4.0)

> **Klasse:** A (Autarker Rechnerkern) · **Ring:** 0 · **Unsafe:** `#![forbid(unsafe_code)]`

## 1. Zweck und Verhalten
`contextra-rank` ist ein reiner Logik-Baustein zur Signal- und Rangfusion (RRF, mRRF), Score-Normalisierung, Kalibrierung (Isotonic via PAVA, Platt-Scaling, Conformal Prediction) sowie Drift-Erkennung und Explainability.

## 2. Invarianten und Verträge
- **INV-RRF-1 (Gleichstandsauflösung):** Gleichstände werden deterministisch über Score absteigend, gefolgt von stabiler ID-Reihenfolge aufgelöst.
- **INV-RRF-2 (K_RRF Konstante):** Standard-Glättungsparameter ist `k = 60.0`. Formel: `1.0 / (k + rank)`.
- **INV-CAL-1 (Isotonische Monotonie):** Nach dem PAVA-Rebuild ist die kalibrierte Wahrscheinlichkeitsfunktion schwach monoton steigend über die sortierten Roh-Scores.
- **INV-CONF-1 (Marginal Coverage):** Der konforme Prädiktor garantiert finite-sample coverage für gegebene Irrtumswahrscheinlichkeit $\alpha \in (0, 1)$.
- **INV-ZERO-PANIC:** Produktionscode enthält kein `unwrap`, `expect` oder `panic!`. Alle Fehler werden typisiert als `RankError` bzw. `ConformalError` zurückgegeben.

## 3. Konstanten
- `MAX_SEARCH_K = 10_000`: Obergrenze für Top-K-Heap.
- `DEFAULT_K_RRF = 60.0`: Normativer RRF-Glättungsparameter.
- `DEFAULT_WARMUP_REQUIRED = 50`: Minimale Beobachtungsanzahl für isotonische Kalibrierung.

## 4. Fehlermodell (`RankError`)
- `InvalidInput`: Ungültige Eingaben (z. B. leere Budgets, unendliche Scores, NaN).
- `Calibration`: Fehler im Kalibrierungsverlauf.
- `FusionFailed`: Fehler bei der Fusionsberechnung.

## 5. Concurrency & Durability
- `contextra-rank` ist rein funktional und zustandslos bzw. thread-safe über `Send + Sync`.
- Keine Locks über `.await`, keine I/O- oder Persistenzoperationen.
