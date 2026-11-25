# Implementierungsplan: contextra-simd

**Ebene:** L0 · **Ziel-Zustand:** Runtime-SIMD-Dispatch für Distanz- und Quantisierungskerne, bitidentisch zur Skalar-Referenz (Z-20).
**Basis:** SPEC R14 §2.1, §2.6, Z-20, §16 (Messung); R15 §2.3.

## 1. Ist-Befund (gemessen)

- 9 Dateien, 3 283 Zeilen. Module: `dispatch` (1187), `error`, `kernels`, `metric`.
- 269 `unsafe`-Vorkommen, davon die meisten in `kernels`.
- `Cargo.toml`: `bytemuck` (nicht in der SPEC-Liste §2.7), `tracing` und `serde` optional.
- `lib.rs` hat **kein** `forbid`, korrekt als Unsafe-Insel.
- **Abweichung:** `bytemuck` ist nicht in §2.7 genannt. Entweder SPEC ergänzen oder entfernen.

## 2. Entfernen

- `serde` als optionale Abhängigkeit, falls nur für Benchmark-Reports genutzt. Prüfen mit `grep -rn serde contextra-simd/src`.
- Kernel-Varianten, die keine Skalar-Referenz besitzen. Jede Architektur-Variante braucht einen Gegentest; ohne ihn raus.

## 3. Ändern

- **`dispatch.rs` (1187 Zeilen):** trennen in
  - `dispatch/table.rs`: Funktionszeiger-Tabelle, einmalig initialisiert.
  - `dispatch/detect.rs`: Feature-Erkennung (`is_x86_feature_detected!`, aarch64-Pfade).
  - `dispatch/select.rs`: Auswahl nach Metrik und Dimension.
- **`kernels`:** jede Architektur in eigenes Modul (`x86_64`, `aarch64`, `scalar`). Scalar ist Referenz und darf nie entfallen.
- **`metric`:** Distanzdefinitionen (L2², Cosinus, Dot) mit fester Reduktionsreihenfolge, damit Ergebnisse reproduzierbar sind.
- **SAFETY-Kommentare:** jeder `unsafe`-Block mit konkreter Begründung (welches Feature geprüft wurde, welche Längen garantiert sind).

## 4. Implementieren

1. **Skalar-Referenz** für: L2², Dot, Cosinus, SQ8-Dequantisierung, RaBitQ-Distanzschätzung. Diese Funktionen sind die Spezifikation.
2. **Feste Reduktionsreihenfolge:** Summation in Blöcken konstanter Breite, immer gleiche Reihenfolge, unabhängig von der Vektorbreite. Sonst ist Bitidentität nicht erreichbar.
3. **Dispatch:** einmalige Auswahl beim ersten Aufruf (`OnceLock`), danach nur Funktionszeiger-Aufruf.
4. **x86_64:** AVX2 und AVX-512 (wenn verfügbar), SSE2 als Basis.
5. **aarch64:** NEON.
6. **Bitidentitätstest:** Zufallsvektoren beliebiger Länge (inkl. Länge 0, 1, nicht-Vielfache der Blockbreite), Ergebnis jeder Variante muss `to_bits()`-gleich der Skalar-Referenz sein.
7. **Benchmarks** mit `criterion` (ohne async), Ergebnis in den Messbericht (§16).

## 5. Tests

- Bitidentität je Architektur auf CI-Runnern mit unterschiedlicher Feature-Unterstützung.
- Fuzz-Ziel für Distanzfunktionen (Pflicht laut §13.1).
- Miri für die Skalar-Pfade; für SIMD-Pfade nur Dispatch-Logik.
- Test: Ohne Feature-Erkennung fällt Dispatch auf Skalar zurück (kein Panic).

## 6. Abhängigkeiten nach Umbau

Erlaubt: `thiserror`, `tracing` (optional), `bytemuck` (nach Entscheidung, siehe §1). Verboten: Allokation im Hot-Path der Kernel.

## 7. Gate und Reihenfolge

1. `dispatch.rs` aufteilen (Verhalten unverändert, Tests grün).
2. Skalar-Referenz und Bitidentitätstest.
3. Architekturvarianten einzeln hinzufügen, je mit Test.
4. G-R1: keine verbotenen APIs. G-R2: Spike H-36 (Distanzkosten).

**Risiko:** Float-Reduktion ist ohne feste Reihenfolge nicht bitidentisch. Das ist die häufigste Fehlerquelle; der Bitidentitätstest muss vor den SIMD-Varianten stehen.
