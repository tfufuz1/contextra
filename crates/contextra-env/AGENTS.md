# AGENTS.md — `contextra-env`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-env` · **Ring:** – · **Version:** 0.1.0
- **Beschreibung:** Injectable environment (clock, rng, id generation, metrics) for Contextra
- **src:** 7 Dateien, 607 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 0):** –
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 0):** –
- **Reverse-Abhängigkeiten (10):** `contextra-checkpoint`, `contextra-crypto`, `contextra-graph`, `contextra-infer-candle`, `contextra-kvcache`, `contextra-mvcc`, `contextra-ports`, `contextra-store`, `contextra-text`, `contextra-vector`
- **Kopplung an Hub-Crates:** ausgehend –; eingehend `contextra-ports`
- **Features (3):** `default`, `std`, `test-utils`

### Öffentliche API (grob, aus src/)
- struct (9): `HistogramSnapshot`, `InMemoryMetricsSink`, `ManualClock`, `MetricKey`, `MetricSnapshot`, `NoopMetricsSink`, `SeededRng`, `SequentialIdGen`, `SystemClock`
- trait (4): `Clock`, `IdGen`, `MetricsSink`, `Rng`
- fn (7): `advance_nanos`, `from_entropy`, `new`, `set_monotonic_nanos`, `set_unix_nanos`, `snapshot`, `unix_ms`
- mod (7): `clock`, `env`, `id_gen`, `manual_clock`, `metrics`, `prelude`, `rng`
- pub use: `clock::{Clock, SystemClock}`, `crate::clock::{Clock, SystemClock}`, `crate::id_gen::{IdGen, SequentialIdGen}`, `crate::metrics::{ HistogramSnapshot, InMemoryMetricsSink, MetricKey, MetricSnapshot, MetricsSink, NoopMetricsSink, }`, `crate::rng::{Rng, SeededRng}`, `env::*`, `id_gen::{IdGen, SequentialIdGen}`, `manual_clock::ManualClock`, `metrics::{ HistogramSnapshot, InMemoryMetricsSink, MetricKey, MetricSnapshot, MetricsSink, NoopMetricsSink, }`, `rng::{Rng, SeededRng}` … (+1)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->`.

<!-- END GENERATED: crate_inventory.py -->
