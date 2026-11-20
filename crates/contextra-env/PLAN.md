# contextra-env — Implementierungsspezifikation

**Ebene:** E0 (Fundament)
**Status:** Normativ, abgeleitet aus `docs/SPEC.md` v4.1
**Version:** 4.1
**Geltungsbereich:** `crates/contextra-env`
**Normative Quellen:** SPEC 3.1, 3.2 (R3, R10), 3.3, 8.6 (REQ-TIM-1 bis REQ-TIM-3), 15, ADR-02, ADR-19, ADR-37, ADR-43, G-01, G-03, G-06, G-12, G-13, REM-14.

---

## 1. Zweck und Ebene

`contextra-env` stellt die drei Umgebungsabhängigkeiten bereit, über die alle anderen Crates Zeit, Zufall und Messwerte beziehen: `Clock`, Zufall für Tests, `MetricsSink`. Das Crate hat keine Verbindung zu Daten, Schlüsseln oder Persistenz.

Die Regeln R10 und G-12 machen `env` zur einzigen Stelle mit Zugriff auf die Systemuhr. Die Testwerkzeuge (`ManualClock`, `SeededRng`) gehören nicht in den Produktbau.

### 1.1 Ist-Befund (gemessen am HEAD `f8b44b2`)

| Messgröße | Wert | Befund |
|---|---|---|
| Quelldateien | 11 `.rs`, 2.658 Zeilen | 1.142 Zeilen Produktcode (`clock`, `metrics`, `rng`, `seq_log`, `id_gen`, `env`, `manual_clock`) |
| Abhängigkeiten | `contextra-ids`, `tracing` | `tracing` verletzt die Schichtung für ein Umgebungs-Crate (kein Logging hier). `ids` wird nur von `IdGen` gebraucht |
| Features | `std` (leer), `test-utils` | `std` ist ohne Wirkung und entfällt. `default = []` ist eingehalten |
| Aggregator | `env.rs` (39 Zeilen) | Reine Re-Exports mit Verweisen wie `ANCHOR[id=env-root,...]` und `LINK docs/crates/...`. Verstößt gegen ADR-19 und REM-14 |
| Zufall | `SeededRng::from_entropy()` | Nicht deterministisch im Produktcode möglich. Verletzt R10 (Zufall für Schlüssel nur über `OsRng`) |
| Uhr | `SystemClock::new()` speichert einen eigenen `Instant`-Nullpunkt je Instanz | Zwei Instanzen zählen unterschiedlich. `Instant::now` und `SystemTime::now` stehen nur in `clock.rs` (korrekt für G-12) |
| Metriken | `record_counter(&str, u64, &[(&str, &str)])`, `record_gauge(f64)`, `record_histogram(f64)`, `MetricKey`, `HistogramSnapshot`, `InMemoryMetricsSink` | Labels und freie Strings. ADR-37 erlaubt nur Zähler und Dauern. Die Histogramm-Logik ist nicht Aufgabe von `env` (ADR-43 liegt in `db`) |
| Sequenzen | `seq_log.rs` (`SequenceLog`, `pin_snapshot`, `max_pin_duration`) | Snapshot-Logik gehört nach SPEC 4.5 (REQ-VER-2) zu `contextra-version`. 13 Dateien außerhalb nutzen sie |
| IDs | `id_gen.rs` (`IdGen`, `SequentialIdGen`) | Nicht zufällig, nicht sicher. Ordinale vergibt der Schreib-Task (SPEC 5.9). 8 Dateien nutzen es |
| Verbraucher | `SystemClock` 30, `SeededRng` 21, `ManualClock` 16, `MetricsSink`-Varianten 24, `SequenceLog` 13, `IdGen` 8, `unix_ms` 2 | Zählung über Bezeichner (grep -w), ohne eigenes Crate |
| Tests | 47 bestanden | Grün. `tests/legacy_reference_unix_ms.rs` testet ein Altverhalten, `conformance_env.rs` ist teilweise veraltet |

Ergebnis: `env` ist nicht leer, sondern enthält einen Sequenz-Snapshot und einen Ort für Ordinale, die dort nicht hingehören. Der Testzufall ist im Produktbau erreichbar.

---

## 2. Vertrag (öffentliche API)

### 2.1 Produktbau (ohne Features)

```rust
/// Zeitquelle. Einzige Quelle der Systemuhr im Workspace (G-12).
pub trait Clock: Send + Sync + 'static {
    /// Unix-Zeit in Nanosekunden. Nicht monoton.
    fn unix_nanos(&self) -> u64;
    /// Monotone Zeit in Nanosekunden seit einem prozessweiten Nullpunkt.
    fn monotonic_nanos(&self) -> u64;
}

/// Die echte Systemuhr. Unit-Struct, kein Zustand.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

/// Name einer Metrik. Konstruierbar nur aus `&'static str`, damit zur Laufzeit keine Daten als Metrikname auftauchen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetricName(&'static str);
impl MetricName {
    pub const fn new(name: &'static str) -> Self;
    pub const fn as_str(self) -> &'static str;
}

/// Empfänger von Zählern und Dauern (ADR-37). Keine Labels, keine Gleitkommawerte, keine Abfragetexte.
pub trait MetricsSink: Send + Sync + 'static {
    fn counter(&self, name: MetricName, value: u64);
    fn duration_ns(&self, name: MetricName, nanos: u64);
}

/// Verwirft alles.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopMetricsSink;
```

Im Produktbau gibt es keinen `Rng`-Trait, keinen `SeededRng`, keine Sequenz- und keine ID-Typen.

### 2.2 Testbau (Feature `test-utils`)

```rust
/// Uhr mit manuell gesetzter Zeit. Für Scrub-Fristen, Review-Fristen und Latenzfenster (SPEC 11, T-ER-18, T-ER-19).
#[derive(Debug)]
pub struct ManualClock { /* Atomare Nanosekunden, nicht öffentlich */ }
impl ManualClock {
    pub fn new(unix_nanos: u64) -> Self;
    pub fn advance_nanos(&self, delta: u64);
    pub fn set_unix_nanos(&self, t: u64);
}
impl Clock for ManualClock {}

/// Deterministischer Zufall (SplitMix64). Nur Tests und Benchmarks (R10).
#[derive(Debug, Clone)]
pub struct SeededRng { /* u64-Zustand */ }
impl SeededRng {
    pub fn new(seed: u64) -> Self;
    pub fn next_u64(&mut self) -> u64;
    pub fn fill_bytes(&mut self, out: &mut [u8]);
}

/// Sammelt Zähler und Dauern im Speicher. Für Assertions in Tests.
#[derive(Debug, Default)]
pub struct RecordingMetricsSink { /* Mutex<BTreeMap<&'static str, u64>> */ }
impl RecordingMetricsSink {
    pub fn counter_total(&self, name: MetricName) -> u64;
    pub fn duration_samples(&self, name: MetricName) -> usize;
}
impl MetricsSink for RecordingMetricsSink {}
```

`SeededRng` hat keinen Konstruktor aus Systemzufall. `from_entropy` entfällt vollständig.

---

## 3. Abhängigkeiten

| Crate | Status |
|---|---|
| `contextra-ids` | **entfernen.** `env` braucht keine Identitätstypen mehr. Die Zufalls-ID-Erzeugung liegt bei `ids` selbst |
| `tracing` | **entfernen** (E-ENV-09) |
| sonst keine | `env` hat nach dem Umbau keine Laufzeitabhängigkeit |

Dev-Abhängigkeiten: keine außer `proptest` für T-ENV-03 (Seeded-Sequenzen), nur wenn der Testplan es verlangt.

Die Schichtregel „`env` ↛ alles“ folgt aus SPEC 3.3, wo `env` nur `ids` erlaubt. Der Umbau macht die Erlaubnis überflüssig. Ein Crate, das nichts braucht, bekommt keine Abhängigkeit.

---

## 4. Module

```
src/
  lib.rs              # Re-Exports, #![forbid(unsafe_code)], Modul-Dokumentation
  clock.rs            # Clock, SystemClock (Instant- und SystemTime-Aufrufe, nur hier)
  metrics.rs          # MetricName, MetricsSink, NoopMetricsSink
  test_utils/         # nur mit Feature test-utils
    mod.rs
    manual_clock.rs   # ManualClock
    rng.rs            # SeededRng (SplitMix64)
    recording.rs      # RecordingMetricsSink
tests/
  clock.rs            # T-ENV-01, T-ENV-08
  rng_golden.rs       # T-ENV-03 (Vektoren für Seeds 0, 1, 42)
  metrics.rs          # T-ENV-06, T-ENV-07
  manual_clock.rs     # T-ENV-02
  fixtures/rng_splitmix64.json
```

Entfallen: `src/env.rs`, `src/id_gen.rs`, `src/seq_log.rs`, `src/rng.rs` (Trait und Produktzugang), `src/manual_clock.rs` (wandert in `test_utils`), Teile von `src/metrics.rs` (Labels, Histogramme, Snapshots), `tests/legacy_reference_unix_ms.rs`, `tests/conformance_env.rs` (wird durch `tests/clock.rs` und `tests/metrics.rs` ersetzt), `tests/generate_golden_env.rs` (falls vorhanden, nicht mehr gebraucht).

---

## 5. Entfernungen

| ID | Entfernen | Grund | Ersatz | Verbraucher (Dateien) |
|---|---|---|---|---|
| E-ENV-01 | `src/env.rs` (Aggregator, ANCHOR-Kommentare, LINK auf `docs/crates/`) | ADR-19, REM-14. Reiner Re-Export ohne Funktion | direkter Import | 0 |
| E-ENV-02 | `unix_ms(&dyn Clock)` in `lib.rs` | Umrechnung ist eine Division, kein Konzept. `Clock::unix_nanos() / 1_000_000` im Aufrufer | Division im Aufrufer | 2 |
| E-ENV-03 | `SequenceLog`, `SeqLogEntry`, `SeqLogChange`, `pin_snapshot`, `max_pin_duration` | SPEC 4.5, REQ-VER-2: Snapshots gehören `contextra-version` | Verschiebung nach `contextra-mvcc` (heutiger Stand des Version-Crates), später `contextra-version` | 13 |
| E-ENV-04 | `IdGen`, `SequentialIdGen` | R10: Zufall nur `OsRng`. Ordinale vergibt der Schreib-Task (SPEC 5.9) | `SealOrd`/`SegOrd` aus `contextra-ids` mit Zähler im Schreib-Task. Tests: `SeededRng` | 8 |
| E-ENV-05 | `Rng`-Trait und `SeededRng::from_entropy` im Produktbau | R10. Der deterministische Zufall gehört in `test-utils` | `SeededRng` (nur Feature) | 21 |
| E-ENV-06 | `SystemClock::new()` mit eigenem Nullpunkt je Instanz | Zwei Instanzen liefern verschiedene monotone Zeiten. Ersetzt durch prozessweiten `OnceLock<Instant>` | `SystemClock` als Unit-Struct | 30 (Konstruktoraufruf) |
| E-ENV-07 | `Clock::monotonic_nanos` mit f64 oder Zeitstempel-Structs | Vereinheitlicht auf `u64` | `u64` | prüfen beim Umbau |
| E-ENV-08 | `MetricKey`, `HistogramSnapshot`, `MetricSnapshot`, `record_gauge`, `record_histogram` | ADR-37 erlaubt nur Zähler und Dauern. Histogramme des Latenzfensters liegen in `db` (ADR-43) | `counter`, `duration_ns` | 24 (Methodennamen) |
| E-ENV-09 | Abhängigkeit `tracing` | Kein Logging in `env`. T-LOG-01 verlangt, dass Metriken nie Nutzdaten tragen, Logs bleiben im Aufrufer | keine | 0 |
| E-ENV-10 | Labels `&[(&str, &str)]` in allen Metrikmethoden | ADR-37: Labels könnten Suchtexte oder Kennungen tragen | keine | 24 |
| E-ENV-11 | `InMemoryMetricsSink` (Produktmodul) | Testwerkzeug | `RecordingMetricsSink` in `test_utils` | 4 |
| E-ENV-12 | Feature `std` | ohne Wirkung. R3 erlaubt `std` nur in `durable-fs`, `simd`, `rank` | keine | 0 |
| E-ENV-13 | `tests/legacy_reference_unix_ms.rs` | testet die Division `unix_ms` (E-ENV-02) | keine | 1 |

---

## 6. Umbau (je Schritt ein Commit, Workspace grün)

| Schritt | Inhalt | Verbraucher-Anpassung | Prüfung |
|---|---|---|---|
| U-ENV-1 | `env.rs`, `unix_ms` und `legacy_reference_unix_ms.rs` löschen | 2 Aufrufe im Aufrufer ersetzen | `cargo check --workspace` |
| U-ENV-2 | Produktbau von `rng.rs` trennen. `SeededRng`, `Rng`-Trait, `ManualClock`, `RecordingMetricsSink` nach `test_utils` verschieben. `from_entropy` löschen | **Vor dem Löschen prüfen**: jede Stelle mit `SeededRng` außerhalb von `#[cfg(test)]`, `tests/`, `benches/`, `examples/` ist ein Fehler nach R10 und wird auf `OsRng` (`rand::rngs::OsRng`) umgestellt. Verbraucher in `ports` (`contract_tests.rs`) entfallen mit `ports`-PLAN | `cargo check -p contextra-env` mit und ohne Feature |
| U-ENV-3 | **Zeit-API festlegen:** `Clock::unix_nanos() -> u64`, `monotonic_nanos() -> u64`. `SystemClock` als Unit-Struct mit prozessweitem `OnceLock<Instant>`. `SystemClock::new()` entfernt | 30 Aufrufe `SystemClock::new()` → `SystemClock` (mechanisch). Dieser Schritt ist Voraussetzung für `contextra-ids` U-IDS-5 | `grep -rn "SystemClock::new" crates` leer |
| U-ENV-4 | `SequenceLog` nach `contextra-mvcc/src/snapshot_log.rs` verschieben. `env` verliert den Typ | 13 Verbraucher importieren aus `contextra-mvcc` (Crate-Name bleibt bis zur Umbenennung in `version`) | `grep -rn "contextra_env::SequenceLog" crates` leer |
| U-ENV-5 | `id_gen.rs` löschen. Ordinal-Vergabe im Schreib-Task des Verbrauchers (zählt in `db` nach dem Schema von SPEC 5.9). Testhelfer in `test_utils` ergänzt, falls nötig | 8 Dateien | `grep -rn "IdGen\|SequentialIdGen" crates` leer |
| U-ENV-6 | Metrik-API auf `MetricName`, `counter`, `duration_ns`. Labels, Gauge, Histogramm, `MetricKey`, `MetricSnapshot`, `HistogramSnapshot` löschen | 24 Dateien. Labels werden beim Aufruf gestrichen, keine Umwandlung in Namen | `grep -rn "record_gauge\|record_histogram\|MetricKey" crates` leer |
| U-ENV-7 | `tracing` und Feature `std` aus `Cargo.toml` entfernen. `contextra-ids` aus `[dependencies]` entfernen | keine | `cargo tree -p contextra-env` zeigt keine Laufzeitabhängigkeit |
| U-ENV-8 | Tests auf `tests/clock.rs`, `tests/rng_golden.rs`, `tests/metrics.rs`, `tests/manual_clock.rs` umstellen | keine | `cargo test -p contextra-env` und `cargo test -p contextra-env --features test-utils` |

**Hinweis zur Reihenfolge.** U-ENV-3 gehört vor `contextra-ids` U-IDS-5. U-ENV-4 setzt voraus, dass `contextra-mvcc` noch existiert. `version` ist ein späterer Umbenennungsschritt in M1, kein Vorgriff.

---

## 7. Tests

| ID | Prüfung | Ort |
|---|---|---|
| T-ENV-01 | `monotonic_nanos` ist über 10.000 Aufrufe nicht fallend. `unix_nanos` liegt zwischen zwei `SystemTime::now`-Messungen | `tests/clock.rs` |
| T-ENV-02 | `ManualClock::advance_nanos` und `set_unix_nanos` wirken auf alle Klone derselben Instanz (über `Arc`) | `tests/manual_clock.rs` |
| T-ENV-03 | `SeededRng::new(seed)` liefert die Golden-Sequenz aus `tests/fixtures/rng_splitmix64.json` für Seeds 0, 1, 42 (je 64 Werte) | `tests/rng_golden.rs` |
| T-ENV-04 | Im Produktbau sind `SeededRng`, `ManualClock`, `RecordingMetricsSink` nicht exportiert (Build ohne Feature, Symbolprüfung über `cargo doc` oder `nm`) | CI-Skript |
| T-ENV-05 | `MetricName` lässt sich nur aus `&'static str` bauen. Kein Konstruktor nimmt `String` (Signatur-Test über Trait-Bounds) | `tests/metrics.rs` |
| T-ENV-06 | `RecordingMetricsSink` summiert Zähler über mehrere Aufrufe. Dauern werden als Stichproben gezählt | `tests/metrics.rs` |
| T-ENV-07 | `NoopMetricsSink` implementiert `MetricsSink` ohne Seiteneffekt | `tests/metrics.rs` |
| T-ENV-08 | `dyn Clock` ist objektsicher. `Box<dyn Clock>` mit `SystemClock` und `ManualClock` | `tests/clock.rs` |
| T-ENV-09 | `cargo tree -p contextra-env --edges normal` enthält keine Laufzeitabhängigkeit (leer außer `contextra-env`) | CI-Skript |
| T-ENV-10 | `Instant::now` und `SystemTime::now` kommen im Workspace nur in `crates/contextra-env/src/clock.rs` vor (G-12) | CI-Skript |

---

## 8. Abnahme

| Gate | Prüfung für `contextra-env` |
|---|---|
| G-01 | `cargo check`, `cargo test`, `cargo test --features test-utils`, `cargo clippy -- -D warnings` grün (Toolchain 1.89.0) |
| G-02 | `#![forbid(unsafe_code)]` in `src/lib.rs` |
| G-03 | `env` hat keine Workspace-Abhängigkeit |
| G-06 | `default = []`, Feature `std` entfällt |
| G-12 | `Instant::now`/`SystemTime::now` nur in `clock.rs` (T-ENV-10) |
| G-13 | keine `unwrap`/`expect`/`panic!` im Produktcode |
| R10 | `SeededRng::from_entropy` und `OsRng` nur an dokumentierten Stellen. Produktcode-Verbraucher von `SeededRng` sind 0 (U-ENV-2) |
| ADR-37 | `MetricsSink` hat keine Labels und keine Gleitkommawerte |

**Reihenfolge im Repository:** Dieses PLAN wird nach `contextra-ids` als zweiter Commit der Fundamentreihe eingecheckt. Die Umsetzung beginnt mit U-ENV-1 und endet mit U-ENV-8. U-ENV-3 ist Voraussetzung für `contextra-ids` U-IDS-5.
