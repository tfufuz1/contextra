# AGENTS.md — contextra-env — Umgebungs- & Systemabstraktionen

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra-env.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra-env.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §2 (Architektur), §5 (Löschkette), §13 (INV-5 Determinismus)  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E0 (Autarkes Fundament / Ring 0)  
**Kern-Closure:** Leer (0 interne Workspace-Abhängigkeiten)  
**Unsafe-Status:** Verboten (`#![forbid(unsafe_code)]`)  

---

## 1. Zweck & Systemrolle

Abstraktion von Zeitquellen (`Clock`), deterministischen Zufallsquellen (`Rng`), Sequenz-IDs (`IdGen`) und Metriken (`MetricsSink`) für deterministische Tests. Das virtuelle Dateisystem gehört **nicht** in dieses Crate (Entscheidung D-04, siehe `contextra-durable-fs`).

---

## 2. Kern-Richtlinien für Agenten

- **Zeitabstraktion (`Clock`):** In Nicht-Test-Code darf niemals direkt `Instant::now()` oder `SystemTime::now()` aufgerufen werden. Immer `Clock` (z.B. `SystemClock` in Produktion, `ManualClock` in Tests) injizieren.
- **Kein Dateisystem-Port:** `contextra-env` enthält keinen `Vfs`-Trait und keine I/O-Adapter. Crash-, Flush- und Recovery-Abstraktionen liegen ausschließlich in `contextra-durable-fs` (D-04).
- **Metriken (`MetricsSink`):** Keine externen Netzwerkaufrufe für Telemetrie; rein lokale Senke.
- **Keine globalen statischen Zustände:** Uhren, RNGs und ID-Generatoren werden stets als injizierbare `Arc<dyn Clock>` bzw. generische Parameter übergeben. Auch Zähler als `static` sind unzulässig.
- **Golden-Vektoren sind eingefroren:** `SeededRng` darf seine Ausgabefolge nicht ändern (`tests/golden/`, INV-1). Die Formel aus Spec §2.2 ist um einen Schritt versetzt und daher NICHT bit-kompatibel; maßgeblich ist die Implementierung, die die Golden-Datei erzeugt hat.

---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-env` · **Ring:** – · **Version:** 0.1.0
- **Beschreibung:** Injectable environment (clock, rng, id generation, metrics) for Contextra
- **src:** 7 Dateien, 607 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 0):** –
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 0):** –
- **Reverse-Abhängigkeiten (8):** `contextra-checkpoint`, `contextra-crypto`, `contextra-graph`, `contextra-mvcc`, `contextra-ports`, `contextra-store`, `contextra-text`, `contextra-vector`
- **Kopplung an Hub-Crates:** ausgehend –; eingehend `contextra-ports`
- **Features (3):** `default`, `std`, `test-utils`

### Öffentliche API (grob, aus src/)
- struct (9): `HistogramSnapshot`, `InMemoryMetricsSink`, `ManualClock`, `MetricKey`, `MetricSnapshot`, `NoopMetricsSink`, `SeededRng`, `SequentialIdGen`, `SystemClock`
- trait (4): `Clock`, `IdGen`, `MetricsSink`, `Rng`
- fn (7): `advance_nanos`, `from_entropy`, `new`, `set_monotonic_nanos`, `set_unix_nanos`, `snapshot`, `unix_ms`
- mod (7): `clock`, `env`, `id_gen`, `manual_clock`, `metrics`, `prelude`, `rng`
- pub use: `clock::{Clock, SystemClock}`, `crate::clock::{Clock, SystemClock}`, `crate::id_gen::{IdGen, SequentialIdGen}`, `crate::metrics::{ HistogramSnapshot, InMemoryMetricsSink, MetricKey, MetricSnapshot, MetricsSink, NoopMetricsSink, }`, `crate::rng::{Rng, SeededRng}`, `env::*`, `id_gen::{IdGen, SequentialIdGen}`, `manual_clock::ManualClock`, `metrics::{ HistogramSnapshot, InMemoryMetricsSink, MetricKey, MetricSnapshot, MetricsSink, NoopMetricsSink, }`, `rng::{Rng, SeededRng}` … (+1)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
