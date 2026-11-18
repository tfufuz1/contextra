# contextra-env — Crate-Spezifikation

**Ebene:** E0 (Kern-Closure leer)  
**Bindung:** `docs/SPEC.md` §2

---

## 1 Zweck

Umgebungsabstraktionen: `Clock`, `MetricsSink`, `Vfs` (virtuelles Dateisystem). Ermöglicht deterministische Tests ohne echtes I/O.

## 2 Verantwortlichkeit

- **Besitzer von `Clock`:** Zeitquelle-Abstraktion. `SystemClock` und `ManualClock` (für Tests).
- **Besitzer von `Vfs`:** Dateisystem-Abstraktion für testbare Persistenz.
- **Besitzer von `MetricsSink`:** Metriken-Abstraktion (Zähler, Histogramme).

## 3 Abhängigkeiten

Keine internen Abhängigkeiten.

## 4 Regeln

- Kein `Instant::now()` außerhalb von `SystemClock`.
- Kein `std::fs::*` außerhalb von `Vfs`-Implementierungen.
