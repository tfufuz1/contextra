# contextra-env — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2 / v4.1, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und dokumentiert den entkoppelten Bausteinzustand.
> **Faktenbasis:** Statische Analyse und Kompilationsprüfungen (`tools/export_crate.py`, `tools/decouple_inspector.py`, `tools/zero_panic_analyzer.py`).
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-env/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Injizierbare Umwelt: `Clock`, `Rng`, `IdGen`, `MetricsSink` und deren Standardimplementierungen (`SystemClock`, `SeededRng`, `SequentialIdGen`, `NoopMetricsSink`, `InMemoryMetricsSink`, `ManualClock`). *[S]*

Als autarker Ebene-0-Mikrokern (E0 / Foundation) besitzt `contextra-env` **null interne Abhängigkeiten** im Workspace und fungiert als primärer Baustein für deterministische Ausführung, Simulationen, Zeiterfassung und Metriksammlung in allen höheren Schichten.

## 2. Zielzustand (docs/SPEC.md §3.1 & §4.2) [S]

| Merkmal | Ist / Ziel | Status |
|---|---|---|
| Gruppe | foundation / core | Erfüllt [V] |
| Ebene (Kern-Closure) | 0 (E0) | Erfüllt [V] (0 interne Abhängigkeiten) |
| Kern-Closure | Leer | Erfüllt [V] |
| Default-Features | `["std"]` (K13 behoben) | Erfüllt [V] |
| Optionale Features | `test-utils` (beinhaltet `ManualClock`) | Erfüllt [V] |
| Fehlertyp (R3) | — (alle Methoden unfehlbar / infallible) | Erfüllt [V] |
| Unsafe-Status | `#![forbid(unsafe_code)]` | Erfüllt [V] (0 unsafe) |
| Modularitäts-Score | 100/100 (`tools/decouple_inspector.py`) | Erfüllt [T] |

## 3. Ist-Zustand (Verifiziert) [V]

- Crate ist vollständig implementiert in `src/` (`lib.rs`, `clock.rs`, `rng.rs`, `id_gen.rs`, `metrics.rs`, `manual_clock.rs`, `env.rs`).
- `test-utils`-Feature kapselt `ManualClock` für Zeitmanipulation in Unittests und Simulationen.
- `Cargo.toml` definiert:
  ```toml
  [features]
  default = ["std"]
  std = []
  test-utils = []
  ```
- Standalone-Exportfähigkeit (Gate G3) via `tools/export_crate.py` verifiziert.
- Zero-Panic-Audit: 0 Verletzungen in Produktionspfaden (`tools/zero_panic_analyzer.py`).
- 0 DSGVO-/Löschketten-Leaks (`tools/decouple_inspector.py`).

## 4. Bindender Vertrag (§4.2) [S]

### 4.1 Clock
```rust
pub trait Clock: Send + Sync + 'static {
    fn now_unix_nanos(&self) -> u64;   // 0 vor der Epoche, nie Panic
    fn monotonic_nanos(&self) -> u64;  // Streng monoton nicht fallend
}
pub fn unix_ms(c: &dyn Clock) -> u64 { c.now_unix_nanos() / 1_000_000 }   // Abrunden
```
- `SystemClock`: Standard-Produktionsimplementierung basierend auf `std::time::SystemTime` und `std::time::Instant`.
- `ManualClock` (unter `test-utils`): Atomar steuerbare Uhrzeit mit `advance_nanos()`, `set_unix_nanos()` und `set_monotonic_nanos()`.

### 4.2 Rng
```rust
pub trait Rng: Send + Sync + 'static {
    fn next_u64(&self) -> u64;
    fn fill_bytes(&self, dest: &mut [u8]);
    fn next_unit_f64(&self) -> f64;
}
```
- `SeededRng`: Bitgleicher SplitMix64-Algorithmus mit threadsicherem atomarem Zustand (`AtomicU64`). Verifiziert gegen Golden-Folgen (Seeds 0, 1, 42, `u64::MAX`).

### 4.3 IdGen
```rust
pub trait IdGen: Send + Sync + 'static {
    fn next_id(&self) -> u64;
}
```
- `SequentialIdGen`: Monoton steigender atomarer Generator (`AtomicU64` mit `fetch_add(1, Ordering::SeqCst)`).

### 4.4 MetricsSink
```rust
pub trait MetricsSink: Send + Sync + 'static {
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]);
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);
}
```
- `NoopMetricsSink`: Null-Kosten Discard-Sink.
- `InMemoryMetricsSink`: Aggregierende In-Memory-Metriksenke für Tests und Telemetrie mit Snapshots (`counters()`, `gauges()`, `histograms()`).

## 5. Invarianten

- **INV-ENV-TIME-1 (Fail-Closed Zeit):** `now_unix_nanos` gibt bei Zeiten vor der Unix-Epoche `0` zurück, niemals einen Panic oder Overflow.
- **INV-ENV-TIME-2 (Monotonie):** `monotonic_nanos` fällt innerhalb eines Prozesses niemals ab.
- **INV-ENV-RNG-1 (SplitMix64-Determinismus):** Gleicher Seed erzeugt auf allen Architekturen die identische 64-Bit-Zufallssequenz (bitgleich mit Golden-Folgen).
- **INV-ENV-RNG-2 (Float-Grenzen):** `next_unit_f64` liefert Werte im halboffenen Intervall `[0.0, 1.0)`.
- **INV-ENV-ID-1 (Monotonie & Eindeutigkeit):** `SequentialIdGen` garantiert threadsichere und eindeutige IDs.
- **INV-ENV-FORBID-UNSAFE:** Vollständiges Verbot von `unsafe` im gesamten Crate (`#![forbid(unsafe_code)]`).

## 6. Kommentierungs- und Dokumentationsstandard

Alle Quellcodedateien folgen dem standardisierten Comment-Anchors-Schema:
- Standard-Anker (`ANCHOR`, `NOTE`, `TODO`, `FIXME`, `STUB`, `REVIEW`)
- Hierarchische Regionen (`// SECTION ...` und `// !SECTION ...`)
- Navigationslinks (`// LINK ...`)
- Attribute (`[id=...,epic=...,seq=...]`)
