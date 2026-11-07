# AGENTS.md — `contextra-ports`

> **Ring 0** · Klasse `[K]` · Reifegrad `stable` · Unsafe: nein · Rolle: einzige Vertragsschicht des Baukastens. Alle Austausch-Traits (Ports) und deren Test-/Konformitätsmittel.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`. Stand 2026-11-07, statisch gelesen, nicht kompiliert.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-ports --no-default-features --locked` |
| Contextra-Abhängigkeiten (Ist = Soll) | `contextra-types` |
| Externe Abhängigkeiten | `futures-util`, `blake3`, `bytes`, `serde`, `serde_json`, `ahash`, `thiserror` (dev: `proptest`, `tokio`) |
| Features | `contract-tests`, `testkit`; `default = []` |
| Lints | `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]` |
| Re-Export | `pub use contextra_types::*;` (alle `types` erscheinen auch als `ports::…`) |

## 2. Abhängigkeiten

Nur `contextra-types`. Ports kennen **keine** konkrete Implementierung und **kein** Crate oberhalb von Ring 0. Ein Trait mit Austauschrolle außerhalb dieses Crates ist unzulässig (SPEC §4.4 Regel 9).

## 3. Modul-Karte (am Code verifiziert)

| Modul | Port / Inhalt | Anmerkung |
|---|---|---|
| `storage.rs` | `StorageEngine` (27 `BoxFuture`-Methoden), `StorageStats` | async-Grenze (siehe §6) |
| `vector_index.rs`, `text_index.rs`, `graph_index.rs`, `graph.rs` | `VectorIndex`, `TextIndex`, `GraphIndex`, Graph-Mutation | |
| `vfs.rs` | `Vfs`, `VfsFile`, `VfsOpenOptions`, `VerifiedVia` | **synchron** (siehe Root-§10 K1) |
| `clock.rs`, `rng.rs`, `id_gen.rs` | `Clock`/`SystemClock`, `Rng`/`SeededRng`, `IdGen`/`SequentialIdGen` | Umgebungs-Ports (M4) |
| `metrics.rs`, `observability.rs`, `observer.rs`, `lifecycle.rs` | `MetricsSink`, `NoopMetricsSink`, `InMemoryMetricsSink` | |
| `scanner/` (`layer`, `session`, `witness`, `mod`) | `Purger`, `LayerScanner`, `ScanSession`, `LayerWitness`, `ScanTarget`, `WitnessLayer`, `PurgeStats` | Bereinigungs-/Mess-Ports (M7) |
| `erasure.rs`, `gate.rs`, `ledger.rs`, `keys.rs`, `report_signer.rs` | `ErasureLayer`, `StorageBackend`, `ErasureTarget`, `ReadGate`, `LedgerSink`, `RecordSealer`, `FileKeyring`, `ReportSigner` | Löschdomäne: Dopplungen siehe §7 |
| `embedding.rs`, `reranker.rs`, `query_rewriter.rs`, `attention.rs`, `kv.rs`, `kv_bridge_port.rs` | Embedder, Reranker, Query-Rewriter, KV-Ports | Rand-Fähigkeiten, Zuordnung zu Archiv-/Randcrates |
| `checkpoint.rs`, `barrier.rs` | `Checkpoint`, `CheckpointCoordinator`, Barrieren | |
| `license.rs`, `plugin.rs` | `LicenseGate`; `plugin` ist laut SPEC `TEST_ONLY` | |
| `conformance.rs`, `contract_tests.rs` | Verdrahtungsmatrix; Konformitätssuite hinter `contract-tests` | Basis für Fremdadapter |

## 4. Verträge & Invarianten

1. **Nur Verträge.** Keine konkreten Implementierungen im Produktionscode. Ausnahmen mit Begründung: `SystemClock`, `SeededRng`, `SequentialIdGen`, `NoopMetricsSink`, `InMemoryMetricsSink` als Referenz-/Null-Objekte.
2. **`SystemClock` ist die einzige zulässige Stelle für `Instant::now`/`SystemTime::now`.** Sie trägt lokal `#[allow(clippy::disallowed_methods)]` (`clock.rs`); `governance/determinism-allow.toml` führt nur `thread_rng`-Ausnahmen. Jede weitere Nutzung in diesem Crate ist ein Defekt.
3. **Neue Trait-Methoden brauchen eine Default-Implementierung**, die `CapabilityUnsupported` liefert, und einen Test, der beweist, dass der Default **nicht** still greift. Sonst bricht jeder Fremdimplementierer (M3, REQ-PORT-4).
4. **Keine internen Fremdtypen in Signaturen.** Nur `types`, eigene Typen oder generische Parameter (G2: „keine Lecks interner Crate-Typen in Port-Signaturen“).
5. **`LayerWitness` hat keinen öffentlichen Konstruktor und kein `Default`** (INV-WITNESS-MINT-1). Er entsteht nur in `ScanSession::finish` nach bestandener Positivkontrolle. `compile_fail`-Doctests sichern das; sie dürfen nicht entfernt werden.
6. **Sicherheits-Ports** (`governance/security_ports.toml`: u. a. `Vfs`, `LayerScanner`, `Purger`, `KmsProvider`) dürfen in Produktion nie Null-/Accept-All-Implementierungen haben (INV-WIRING-2). **Achtung:** die dortigen `trait_path`-Angaben (`contextra_ports::kms::…`, `::purger::Purger`, `::log::DurableLog`, `::journal::ErasureJournal`) und `prod_impl`-Pfade (`contextra_store::vfs::StdVfs`, `…::sidecar::*`) existieren im Code **nicht**; reale Pfade z. B. `contextra_ports::scanner::Purger`. Die Datei ist zu korrigieren (WP 2.2/7).
7. **Semver-Disziplin (REQ-PORT-4).** Port-Signaturen sind eingefroren. Eine inkompatible Änderung verlangt neue Port-Version, Migrationsregel und ADR. Prüfe mit `cargo public-api`.
8. **Objekt-Sicherheit.** Austausch-Traits, die als `dyn` genutzt werden, bleiben dyn-kompatibel (Rückgabe `BoxFuture`, keine generischen Methoden ohne `where Self: Sized`).

## 5. Ports (Übersicht der Verantwortung)

**Definiert:** alle Ports aus §3. **Implementiert:** nur die Referenzobjekte aus §4.1. **Konsumiert:** `contextra-types`.

**Fehlend laut PLAN/SPEC (Soll, noch nicht im Code):** `EgressPolicy` (WP 2.4; `privacy` kennt bisher nur `BlockReason::EgressPolicyDenied`). `KmsProvider` liegt noch in `crypto/src/wal_crypto.rs:32` und soll hierher (SPEC §5.2), `security_ports.toml` setzt das bereits voraus. `DurableLog`, `ErasureJournal`, `EffaceableKeySlot`, `ErasedSetMirror`, `GroupRevocationListener` sind hier **nicht** als Traits zu finden (gegen `grep -rn "trait <Name>"` prüfen, bevor du sie benutzt).

## 6. Nebenläufigkeit & Performance

- `StorageEngine` ist die **async-Grenze** (`BoxFuture`). Zielbild (SPEC §6): `StorageRead` synchron, `StorageWrite` async. Diese zwei Traits existieren **noch nicht** (`.agents/rules/async-io.md` setzt sie fälschlich voraus). Neue Ports sind **synchron**, es sei denn, ein Netz- oder Laufzeit-Bezug ist belegt (M8).
- `Vfs` bleibt synchron; blockierende Aufrufer in async-Kontext nutzen `spawn_blocking`.
- Hot-Path-Ports (`VectorIndex::search`, `StorageRead::get`) werden von Konsumenten **generisch** gebunden (Monomorphisierung), nicht als `dyn`. Cold-Path-Ports (`Purger`, `LayerScanner`, `ReportSigner`) dürfen `dyn` sein.
- Ratchet: die 4 `async fn` und die `BoxFuture`-Nutzung in `ports` dürfen nicht steigen (REQ-ASYNC-1).

## 7. Anti-Patterns

- **Zweite Typfamilie für ein Konzept.** Verifiziert doppelt: `LayerWitness` (`erasure.rs:183`, `scanner/witness.rs:17`), `LayerId` und `EraseTarget` (auch in `types`). Neue Nutzung nur über `scanner::*`; Vereinigung per ADR (WP 2.2).
- **Trait ohne Contract-Test.** Jeder Port in `contract_tests.rs` (aktuell `Clock`, `Rng`, `Vfs`, `MetricsSink`) bekommt eine generische Konformitätsfunktion; Lücken: `StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`, `Purger`, `LayerScanner` (WP 2.5).
- **`Vfs` zu `async_trait` umbauen** (PLAN-Snippet ist veraltet, Root-§10 K1).
- **Konkrete Engine-Typen in Signaturen** (`LsmStorage`, `HnswIndex`).
- **`Default`-Implementierung, die still `Ok(())` liefert** für sicherheitsrelevante Methoden.

## 8. Tests & Messung

```bash
cargo test -p contextra-ports --locked
cargo test -p contextra-ports --locked --features contract-tests
cargo test -p contextra-ports --doc            # compile_fail-Beweise für Witness-Mint
cargo clippy -p contextra-ports --all-targets -- -D warnings
```

- Fremdadapter (`testkit`, `store`, `vector`, `text`, `graph`) rufen die Konformitätsfunktionen aus `contract_tests` und `conformance` auf.
- Property-Tests für `SeededRng`-Determinismus und `ManualClock`-Verhalten liegen in `contextra-testkit`.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 2.1 | `Vfs` auf Streaming/Handle-Operationen konsolidieren (`read_at`, `write_at`, `sync_data`, `sync_all` sind in `VfsFile` vorhanden, Vereinheitlichung der zwei historischen Vfs-Konzepte, D7) |
| 2.2 | `Purger`/`LayerScanner`-Signaturen entscheiden (Root-§10 K2), doppelte Löschtypen vereinen (K3) |
| 2.3 | `StorageEngine`/`VectorIndex`/`TextIndex`/`GraphIndex` bereinigen, `StorageRead`/`StorageWrite` trennen |
| 2.4 | Port `EgressPolicy` einführen |
| 2.5 | Contract-Tests auf alle Ports erweitern |
