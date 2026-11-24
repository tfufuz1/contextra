# Schnittstellenspezifikation: `contextra-erasure`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-erasure` |
| Version | workspace |
| Pfad | `crates/contextra-erasure` |
| Rolle | Löschkern alt (Orchestrator-Ziel laut R10) |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> Löschkern: Saga, Gate, Journal, Tresor, Messung, signierter Bericht.
>
> Ring 3 (SPEC §5.1, §5.2, §11). Abhängigkeitsregel AR-3 (SPEC §5.3): Erlaubte Abhängigkeiten
> sind ausschließlich `contextra-types`, `contextra-ports` und `contextra-crypto`.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `aes-gcm-siv` | normal | registry | ja |  |
| `ahash` | normal | workspace | nein |  |
| `bincode` | normal | registry | nein |  |
| `blake3` | normal | registry | nein |  |
| `bytes` | normal | workspace | nein |  |
| `ed25519-dalek` | normal | registry | ja |  |
| `futures-util` | normal | registry | ja |  |
| `hmac` | normal | registry | nein |  |
| `loom` | dev | registry | nein |  |
| `parking_lot` | normal | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `rand` | normal | registry | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `sha2` | normal | registry | nein |  |
| `subtle` | normal | registry | nein |  |
| `tempfile` | dev | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | dev | workspace | nein |  |
| `tracing` | normal | workspace | nein |  |
| `zeroize` | normal | registry | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-erasure-fuzz`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `async` | `dep:tokio`, `dep:futures-util` |
| `default` | `signing`, `vault`, `async` |
| `signing` | `dep:ed25519-dalek` |
| `vault` | `dep:aes-gcm-siv` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 83 |
| Quellzeilen | 22910 |
| Öffentliche Items (Modul-Ebene) | 559 |
| davon `fn` | 97 |
| davon `struct` | 204 |
| davon `enum` | 99 |
| davon `trait` | 39 |
| davon `type` | 14 |
| davon `const` | 25 |
| davon `mod` | 81 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 64 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 119 |
| `.expect(` (Produktionscode) | 1 |
| `panic!` (Produktionscode) | 2 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 49 |
| `#[deprecated]` | 37 |
| `#[cfg(feature…)]`-Gates | 43 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 1155 / 1155 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `backend` | `crates/contextra-erasure/src/backend.rs` | 32 |
| `crypto_local` | `crates/contextra-erasure/src/crypto_local.rs` | 274 |
| `deletion_proof` | `crates/contextra-erasure/src/deletion_proof.rs` | 2743 |
| `ed25519_proof` | `crates/contextra-erasure/src/ed25519_proof.rs` | 166 |
| `erasure_domain` | `crates/contextra-erasure/src/erasure_domain.rs` | 633 |
| `error` | `crates/contextra-erasure/src/error.rs` | 125 |
| `gate::batch` | `crates/contextra-erasure/src/gate/batch.rs` | 262 |
| `gate::erased_set` | `crates/contextra-erasure/src/gate/erased_set.rs` | 341 |
| `gate::fence` | `crates/contextra-erasure/src/gate/fence.rs` | 442 |
| `gate::log` | `crates/contextra-erasure/src/gate/log.rs` | 202 |
| `gate::read_gate` | `crates/contextra-erasure/src/gate/read_gate.rs` | 120 |
| `gate` | `crates/contextra-erasure/src/gate.rs` | 14 |
| `io` | `crates/contextra-erasure/src/io.rs` | 64 |
| `journal::appendlog` | `crates/contextra-erasure/src/journal/appendlog.rs` | 385 |
| `journal::epoch` | `crates/contextra-erasure/src/journal/epoch.rs` | 53 |
| `journal::record` | `crates/contextra-erasure/src/journal/record.rs` | 405 |
| `journal::reduce` | `crates/contextra-erasure/src/journal/reduce.rs` | 23 |
| `journal::scope_purge` | `crates/contextra-erasure/src/journal/scope_purge.rs` | 78 |
| `journal::wal` | `crates/contextra-erasure/src/journal/wal.rs` | 85 |
| `journal::wal_sink` | `crates/contextra-erasure/src/journal/wal_sink.rs` | 406 |
| `journal` | `crates/contextra-erasure/src/journal.rs` | 75 |
| `kv_shredding` | `crates/contextra-erasure/src/kv_shredding.rs` | 982 |
| `ledger::durable` | `crates/contextra-erasure/src/ledger/durable.rs` | 291 |
| `ledger::entry` | `crates/contextra-erasure/src/ledger/entry.rs` | 76 |
| `ledger::error` | `crates/contextra-erasure/src/ledger/error.rs` | 48 |
| `ledger::persistent` | `crates/contextra-erasure/src/ledger/persistent.rs` | 184 |
| `ledger::record` | `crates/contextra-erasure/src/ledger/record.rs` | 318 |
| `ledger::sink` | `crates/contextra-erasure/src/ledger/sink.rs` | 228 |
| `ledger::storage` | `crates/contextra-erasure/src/ledger/storage.rs` | 184 |
| `ledger` | `crates/contextra-erasure/src/ledger.rs` | 209 |
| `crate` | `crates/contextra-erasure/src/lib.rs` | 44 |
| `measure::field` | `crates/contextra-erasure/src/measure/field.rs` | 326 |
| `measure::positive_control` | `crates/contextra-erasure/src/measure/positive_control.rs` | 176 |
| `measure::residue` | `crates/contextra-erasure/src/measure/residue.rs` | 566 |
| `measure::scanner_flow` | `crates/contextra-erasure/src/measure/scanner_flow.rs` | 196 |
| `measure::witness` | `crates/contextra-erasure/src/measure/witness.rs` | 61 |
| `measure` | `crates/contextra-erasure/src/measure.rs` | 141 |
| `orchestrator` | `crates/contextra-erasure/src/orchestrator.rs` | 147 |
| `purge::cx` | `crates/contextra-erasure/src/purge/cx.rs` | 47 |
| `purge::order` | `crates/contextra-erasure/src/purge/order.rs` | 53 |
| `purge::phase` | `crates/contextra-erasure/src/purge/phase.rs` | 309 |
| `purge::reserve` | `crates/contextra-erasure/src/purge/reserve.rs` | 40 |
| `purge::resume` | `crates/contextra-erasure/src/purge/resume.rs` | 87 |
| `purge::window` | `crates/contextra-erasure/src/purge/window.rs` | 75 |
| `purge` | `crates/contextra-erasure/src/purge.rs` | 26 |
| `replace` | `crates/contextra-erasure/src/replace.rs` | 138 |
| `report::binding` | `crates/contextra-erasure/src/report/binding.rs` | 46 |
| `report::build` | `crates/contextra-erasure/src/report/build.rs` | 651 |
| `report::canonical` | `crates/contextra-erasure/src/report/canonical.rs` | 92 |
| `report::risk` | `crates/contextra-erasure/src/report/risk.rs` | 341 |
| `report::signing` | `crates/contextra-erasure/src/report/signing.rs` | 398 |
| `report` | `crates/contextra-erasure/src/report.rs` | 27 |
| `restore::guard` | `crates/contextra-erasure/src/restore/guard.rs` | 67 |
| `restore::replay` | `crates/contextra-erasure/src/restore/replay.rs` | 48 |
| `restore` | `crates/contextra-erasure/src/restore.rs` | 29 |
| `restricted` | `crates/contextra-erasure/src/restricted.rs` | 211 |
| `revocation_log` | `crates/contextra-erasure/src/revocation_log.rs` | 466 |
| `saga::core` | `crates/contextra-erasure/src/saga/core.rs` | 932 |
| `saga::driver` | `crates/contextra-erasure/src/saga/driver.rs` | 145 |
| `saga::idempotency` | `crates/contextra-erasure/src/saga/idempotency.rs` | 19 |
| `saga::machine` | `crates/contextra-erasure/src/saga/machine.rs` | 191 |
| `saga::retention` | `crates/contextra-erasure/src/saga/retention.rs` | 22 |
| `saga::seams` | `crates/contextra-erasure/src/saga/seams.rs` | 128 |
| `saga` | `crates/contextra-erasure/src/saga.rs` | 25 |
| `scope::codec` | `crates/contextra-erasure/src/scope/codec.rs` | 163 |
| `scope::file` | `crates/contextra-erasure/src/scope/file.rs` | 186 |
| `scope::flight` | `crates/contextra-erasure/src/scope/flight.rs` | 118 |
| `scope::set` | `crates/contextra-erasure/src/scope/set.rs` | 77 |
| `scope::single_flight` | `crates/contextra-erasure/src/scope/single_flight.rs` | 361 |
| `scope` | `crates/contextra-erasure/src/scope.rs` | 17 |
| `supersede` | `crates/contextra-erasure/src/supersede.rs` | 174 |
| `test_mocks` | `crates/contextra-erasure/src/test_mocks.rs` | 80 |
| `types_local` | `crates/contextra-erasure/src/types_local.rs` | 2439 |
| `vault::durable` | `crates/contextra-erasure/src/vault/durable.rs` | 651 |
| `vault::key_store` | `crates/contextra-erasure/src/vault/key_store.rs` | 218 |
| `vault::keyring` | `crates/contextra-erasure/src/vault/keyring.rs` | 159 |
| `vault::open` | `crates/contextra-erasure/src/vault/open.rs` | 236 |
| `vault::rehash` | `crates/contextra-erasure/src/vault/rehash.rs` | 169 |
| `vault::scrub` | `crates/contextra-erasure/src/vault/scrub.rs` | 130 |
| `vault::sealer` | `crates/contextra-erasure/src/vault/sealer.rs` | 825 |
| `vault::slot` | `crates/contextra-erasure/src/vault/slot.rs` | 431 |
| `vault::table` | `crates/contextra-erasure/src/vault/table.rs` | 375 |
| `vault` | `crates/contextra-erasure/src/vault.rs` | 383 |

## 6. Re-Exports (`pub use`)

- `pub use erased_set::ErasedSet` — `crates/contextra-erasure/src/gate.rs:9` (Modul `gate`)
- `pub use fence::` — `crates/contextra-erasure/src/gate.rs:10` (Modul `gate`)
- `pub use log::` — `crates/contextra-erasure/src/gate.rs:13` (Modul `gate`)
- `pub use read_gate::GateGuard` — `crates/contextra-erasure/src/gate.rs:14` (Modul `gate`)
- `pub use crate::saga::SagaState` — `crates/contextra-erasure/src/journal.rs:9` (Modul `journal`)
- `pub use wal::MemoryWalSink` — `crates/contextra-erasure/src/journal.rs:10` (Modul `journal`)
- `pub use appendlog::` — `crates/contextra-erasure/src/journal.rs:11` (Modul `journal`)
- `pub use epoch::` — `crates/contextra-erasure/src/journal.rs:12` (Modul `journal`)
- `pub use record::` — `crates/contextra-erasure/src/journal.rs:13` (Modul `journal`)
- `pub use reduce::reduce` — `crates/contextra-erasure/src/journal.rs:14` (Modul `journal`)
- `pub use crate::saga::SagaState` — `crates/contextra-erasure/src/journal/record.rs:6` (Modul `journal::record`)
- `pub use durable::` — `crates/contextra-erasure/src/ledger.rs:13` (Modul `ledger`)
- `pub use entry::` — `crates/contextra-erasure/src/ledger.rs:14` (Modul `ledger`)
- `pub use error::LedgerError` — `crates/contextra-erasure/src/ledger.rs:15` (Modul `ledger`)
- `pub use persistent::ErasureLedger` — `crates/contextra-erasure/src/ledger.rs:16` (Modul `ledger`)
- `pub use durable::ErasureLedger as DurableErasureLedger` — `crates/contextra-erasure/src/ledger.rs:17` (Modul `ledger`)
- `pub use record::` — `crates/contextra-erasure/src/ledger.rs:18` (Modul `ledger`)
- `pub use sink::InMemoryLedger as FileLedger` — `crates/contextra-erasure/src/ledger.rs:19` (Modul `ledger`)
- `pub use storage::` — `crates/contextra-erasure/src/ledger.rs:20` (Modul `ledger`)
- `pub use field::` — `crates/contextra-erasure/src/measure.rs:14` (Modul `measure`)
- `pub use scanner_flow::` — `crates/contextra-erasure/src/measure.rs:19` (Modul `measure`)
- `pub use residue::` — `crates/contextra-erasure/src/measure.rs:20` (Modul `measure`)
- `pub use witness::` — `crates/contextra-erasure/src/measure.rs:26` (Modul `measure`)
- `pub use crate::types_local::` — `crates/contextra-erasure/src/measure.rs:29` (Modul `measure`)
- `pub use cx::PurgeCx` — `crates/contextra-erasure/src/purge.rs:12` (Modul `purge`)
- `pub use order::` — `crates/contextra-erasure/src/purge.rs:13` (Modul `purge`)
- `pub use phase::` — `crates/contextra-erasure/src/purge.rs:14` (Modul `purge`)
- `pub use reserve::` — `crates/contextra-erasure/src/purge.rs:18` (Modul `purge`)
- `pub use resume::PurgeProgressTracker` — `crates/contextra-erasure/src/purge.rs:19` (Modul `purge`)
- `pub use window::run_core_window` — `crates/contextra-erasure/src/purge.rs:21` (Modul `purge`)
- `pub use window::` — `crates/contextra-erasure/src/purge.rs:22` (Modul `purge`)
- `pub use crate::types_local::` — `crates/contextra-erasure/src/purge.rs:24` (Modul `purge`)
- `pub use binding::` — `crates/contextra-erasure/src/report.rs:12` (Modul `report`)
- `pub use build::` — `crates/contextra-erasure/src/report.rs:13` (Modul `report`)
- `pub use canonical::` — `crates/contextra-erasure/src/report.rs:18` (Modul `report`)
- `pub use risk::` — `crates/contextra-erasure/src/report.rs:22` (Modul `report`)
- `pub use signing::` — `crates/contextra-erasure/src/report.rs:27` (Modul `report`)
- `pub use guard::` — `crates/contextra-erasure/src/restore.rs:8` (Modul `restore`)
- `pub use replay::` — `crates/contextra-erasure/src/restore.rs:9` (Modul `restore`)
- `pub use core::` — `crates/contextra-erasure/src/saga.rs:14` (Modul `saga`)
- `pub use driver::` — `crates/contextra-erasure/src/saga.rs:15` (Modul `saga`)
- `pub use idempotency::` — `crates/contextra-erasure/src/saga.rs:16` (Modul `saga`)
- `pub use machine::` — `crates/contextra-erasure/src/saga.rs:17` (Modul `saga`)
- `pub use retention::` — `crates/contextra-erasure/src/saga.rs:18` (Modul `saga`)
- `pub use seams::` — `crates/contextra-erasure/src/saga.rs:19` (Modul `saga`)
- `pub use crate::types_local::` — `crates/contextra-erasure/src/saga.rs:23` (Modul `saga`)
- `pub use self::EraseRequest as SagaEraseRequest` — `crates/contextra-erasure/src/saga/machine.rs:13` (Modul `saga::machine`)
- `pub use crate::types_local::` — `crates/contextra-erasure/src/saga/machine.rs:14` (Modul `saga::machine`)
- `pub use codec::` — `crates/contextra-erasure/src/scope.rs:11` (Modul `scope`)
- `pub use file::` — `crates/contextra-erasure/src/scope.rs:12` (Modul `scope`)
- `pub use flight::` — `crates/contextra-erasure/src/scope.rs:13` (Modul `scope`)
- `pub use set::` — `crates/contextra-erasure/src/scope.rs:14` (Modul `scope`)
- `pub use single_flight::` — `crates/contextra-erasure/src/scope.rs:15` (Modul `scope`)
- `pub use futures_util::future::BoxFuture` — `crates/contextra-erasure/src/types_local.rs:8` (Modul `types_local`)
- `pub use durable::VAULT_SLOT_BASE` — `crates/contextra-erasure/src/vault.rs:21` (Modul `vault`)
- `pub use keyring::` — `crates/contextra-erasure/src/vault.rs:22` (Modul `vault`)
- `pub use open::` — `crates/contextra-erasure/src/vault.rs:23` (Modul `vault`)
- `pub use rehash::` — `crates/contextra-erasure/src/vault.rs:27` (Modul `vault`)
- `pub use scrub::` — `crates/contextra-erasure/src/vault.rs:28` (Modul `vault`)
- `pub use sealer::VaultSealer` — `crates/contextra-erasure/src/vault.rs:29` (Modul `vault`)
- `pub use slot::` — `crates/contextra-erasure/src/vault.rs:30` (Modul `vault`)
- `pub use table::` — `crates/contextra-erasure/src/vault.rs:37` (Modul `vault`)
- `pub use crate::types_local::VerifiedVia` — `crates/contextra-erasure/src/vault/durable.rs:4` (Modul `vault::durable`)
- `pub use key_store::` — `crates/contextra-erasure/src/vault/keyring.rs:159` (Modul `vault::keyring`)

## 8. Öffentliche Schnittstelle je Item

### Modul `backend`

#### `trait ErasureTarget`
- Ort: `crates/contextra-erasure/src/backend.rs:26`
```rust
pub trait ErasureTarget: Send + Sync + Debug + 'static
```
Dokumentation: Generisches Löschziel (ErasureTarget) für Mandanten, Collections oder Dokumente.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn target_id(&self) -> String` | `crates/contextra-erasure/src/backend.rs:28` | Eindeutiger Identifikator des Targets. |
| `fn target_type(&self) -> &'static str` | `crates/contextra-erasure/src/backend.rs:31` | Art des Löschziels. |

#### `trait StorageBackend`
- Ort: `crates/contextra-erasure/src/backend.rs:17`
```rust
pub trait StorageBackend: Send + Sync + 'static
```
Dokumentation: Generisches Storage-Backend für modulare Datenzugriffe im Löschkern.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn purge<'a>(&'a self, cx: &'a PurgeCx) -> BoxFuture<'a, Result<PurgeOutcome, ErasureError>>` | `crates/contextra-erasure/src/backend.rs:19` | Führt physische Bereinigung auf dem Backend aus. |
| `fn layer_id(&self) -> LayerId` | `crates/contextra-erasure/src/backend.rs:22` | Liefert den Schicht-Identifikator des Backends. |

#### `type BoxFuture`
- Ort: `crates/contextra-erasure/src/backend.rs:13`
```rust
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>
```
Dokumentation: Type-Alias für gepinnte ge-boxte Futures ohne Tokio/Futures-Util.

### Modul `crate`

#### `mod backend`
- Ort: `crates/contextra-erasure/src/lib.rs:13`
```rust
pub mod backend
```

#### `mod crypto_local`
- Ort: `crates/contextra-erasure/src/lib.rs:14`
```rust
pub mod crypto_local
```

#### `mod deletion_proof`
- Ort: `crates/contextra-erasure/src/lib.rs:16`
```rust
pub mod deletion_proof
```
- Gate: `#[cfg(feature = "signing")]`

#### `mod ed25519_proof`
- Ort: `crates/contextra-erasure/src/lib.rs:38`
```rust
pub mod ed25519_proof
```
- Gate: `#[cfg(feature = "signing")]`

#### `mod erasure_domain`
- Ort: `crates/contextra-erasure/src/lib.rs:17`
```rust
pub mod erasure_domain
```

#### `mod error`
- Ort: `crates/contextra-erasure/src/lib.rs:18`
```rust
pub mod error
```

#### `mod gate`
- Ort: `crates/contextra-erasure/src/lib.rs:19`
```rust
pub mod gate
```

#### `mod io`
- Ort: `crates/contextra-erasure/src/lib.rs:20`
```rust
pub mod io
```

#### `mod journal`
- Ort: `crates/contextra-erasure/src/lib.rs:21`
```rust
pub mod journal
```

#### `mod kv_shredding`
- Ort: `crates/contextra-erasure/src/lib.rs:40`
```rust
pub mod kv_shredding
```
- Gate: `#[cfg(feature = "vault")]`

#### `mod ledger`
- Ort: `crates/contextra-erasure/src/lib.rs:22`
```rust
pub mod ledger
```

#### `mod measure`
- Ort: `crates/contextra-erasure/src/lib.rs:23`
```rust
pub mod measure
```

#### `mod orchestrator`
- Ort: `crates/contextra-erasure/src/lib.rs:42`
```rust
pub mod orchestrator
```
- Gate: `#[cfg(feature = "signing")]`

#### `mod purge`
- Ort: `crates/contextra-erasure/src/lib.rs:24`
```rust
pub mod purge
```

#### `mod replace`
- Ort: `crates/contextra-erasure/src/lib.rs:25`
```rust
pub mod replace
```

#### `mod report`
- Ort: `crates/contextra-erasure/src/lib.rs:26`
```rust
pub mod report
```

#### `mod restore`
- Ort: `crates/contextra-erasure/src/lib.rs:27`
```rust
pub mod restore
```

#### `mod restricted`
- Ort: `crates/contextra-erasure/src/lib.rs:28`
```rust
pub mod restricted
```

#### `mod revocation_log`
- Ort: `crates/contextra-erasure/src/lib.rs:44`
```rust
pub mod revocation_log
```
- Gate: `#[cfg(feature = "signing")]`

#### `mod saga`
- Ort: `crates/contextra-erasure/src/lib.rs:29`
```rust
pub mod saga
```

#### `mod scope`
- Ort: `crates/contextra-erasure/src/lib.rs:30`
```rust
pub mod scope
```

#### `mod supersede`
- Ort: `crates/contextra-erasure/src/lib.rs:31`
```rust
pub mod supersede
```

#### `mod types_local`
- Ort: `crates/contextra-erasure/src/lib.rs:34`
```rust
pub mod types_local
```

#### `mod vault`
- Ort: `crates/contextra-erasure/src/lib.rs:35`
```rust
pub mod vault
```

### Modul `crypto_local`

#### `struct AncestorSalts`
- Ort: `crates/contextra-erasure/src/crypto_local.rs:47`
```rust
pub struct AncestorSalts
```
Dokumentation: Ahnen-Salts für die DEK-Ableitung (Tenant, Collection, Subject, Document).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `t` | `[u8; 32]` | `crates/contextra-erasure/src/crypto_local.rs:49` |
| `c` | `[u8; 32]` | `crates/contextra-erasure/src/crypto_local.rs:51` |
| `s` | `Option<[u8; 32]>` | `crates/contextra-erasure/src/crypto_local.rs:53` |
| `d` | `[u8; 32]` | `crates/contextra-erasure/src/crypto_local.rs:55` |

#### `struct Dek`
- Ort: `crates/contextra-erasure/src/crypto_local.rs:39`
```rust
pub struct Dek(pub SecretKey<32>)
```
Dokumentation: Data Encryption Key (DEK).

#### `struct SecretKey`
- Ort: `crates/contextra-erasure/src/crypto_local.rs:15`
```rust
pub struct SecretKey<const N: usize>
```
Dokumentation: 32-Byte SecretKey mit Zeroize-on-Drop.
- Impl-Block `crates/contextra-erasure/src/crypto_local.rs:19`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-erasure/src/crypto_local.rs:31`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(bytes: [u8` | `crates/contextra-erasure/src/crypto_local.rs:21` | Erzeugt einen neuen SecretKey aus einem Byte-Array. |
| `pub fn as_bytes(&self) -> &[u8` | `crates/contextra-erasure/src/crypto_local.rs:26` | Gibt eine Referenz auf die geschützten Bytes zurück. |

#### `struct UnitKey`
- Ort: `crates/contextra-erasure/src/crypto_local.rs:43`
```rust
pub struct UnitKey(pub SecretKey<32>)
```
Dokumentation: UnitKey Wrapper.

#### `enum HkdfError`
- Ort: `crates/contextra-erasure/src/crypto_local.rs:61`
```rust
pub enum HkdfError
```
Dokumentation: Führt HKDF-Expand gemäß RFC 5869 auf Basis von HMAC-SHA256 durch. Fehler bei HKDF-Operationen.

Varianten:

- `OutputTooLarge` — `crates/contextra-erasure/src/crypto_local.rs:64`
- `InvalidKey` — `crates/contextra-erasure/src/crypto_local.rs:67`
- `CounterOverflow` — `crates/contextra-erasure/src/crypto_local.rs:70`

#### `fn derive_dek`
- Ort: `crates/contextra-erasure/src/crypto_local.rs:105`
```rust
pub fn derive_dek(unit: &UnitKey, a: &AncestorSalts, doc: [u8
```
Dokumentation: Leitet den DEK nach SPEC D-79 / §9.4 / §10.1 ab: `DEK = HKDF-SHA256(ikm = K_unit, salt = SHA-256("ctx/dek-salt/v2" || salt_T || salt_C || salt_S || salt_D), info = "ctx/dek/v2" || doc_tag)`

#### `fn hkdf_expand`
- Ort: `crates/contextra-erasure/src/crypto_local.rs:74`
```rust
pub fn hkdf_expand(prk: &[u8], info: &[u8], out: &mut [u8]) -> Result<(), HkdfError>
```
Dokumentation: Fuehrt HKDF-Expand gemaess RFC 5869 auf Basis von HMAC-SHA256 durch.

#### `fn open_record`
- Ort: `crates/contextra-erasure/src/crypto_local.rs:195`
```rust
pub fn open_record( dek: &Dek, key_bytes: &[u8], env: &[u8], ) -> Result<zeroize::Zeroizing<Vec<u8>>, crate::error::ErasureError>
```
Dokumentation: Öffnet einen versiegelten Datensatz gemäß F.6.

#### `fn seal_record`
- Ort: `crates/contextra-erasure/src/crypto_local.rs:136`
```rust
pub fn seal_record(dek: &Dek, doc: [u8
```
Dokumentation: Verschlüsselt und versiegelt einen Datensatz gemäß F.6 (Envelope-Format). Format: `version(1) || alg_id(1) || doc(16) || nonce(12) || ciphertext || tag(16)` Für autarke Modularität verwenden wir BLAKE3-Keyed MAC Streaming Verschlüsselung (CTR-Modus über BLAKE3-KDF).

### Modul `deletion_proof`

#### `struct DeletionProof`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:433`
```rust
pub struct DeletionProof
```
- Impl-Block `crates/contextra-erasure/src/deletion_proof.rs:498`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `signature_version` | `u8` | `crates/contextra-erasure/src/deletion_proof.rs:450` |
| `scope` | `DeletionScope` | `crates/contextra-erasure/src/deletion_proof.rs:452` |
| `deleted_keys_hash` | `[u8; 32]` | `crates/contextra-erasure/src/deletion_proof.rs:454` |
| `deleted_after_tx` | `TxId` | `crates/contextra-erasure/src/deletion_proof.rs:457` |
| `timestamp` | `u64` | `crates/contextra-erasure/src/deletion_proof.rs:460` |
| `signature` | `Vec<u8>` | `crates/contextra-erasure/src/deletion_proof.rs:462` |
| `covered_layers` | `Vec<DeletionLayer>` | `crates/contextra-erasure/src/deletion_proof.rs:464` |
| `excluded_scopes` | `Vec<ExcludedScope>` | `crates/contextra-erasure/src/deletion_proof.rs:466` |
| `graph_repair` | `Vec<GraphRepairAttestation>` | `crates/contextra-erasure/src/deletion_proof.rs:469` |
| `wal_chain_receipt` | `Option<[u8; 32]>` | `crates/contextra-erasure/src/deletion_proof.rs:472` |
| `audit_chain_position` | `Option<u64>` | `crates/contextra-erasure/src/deletion_proof.rs:475` |
| `custody` | `CustodyLevel` | `crates/contextra-erasure/src/deletion_proof.rs:478` |
| `measurements` | `Vec<LayerWitnessMeasurement>` | `crates/contextra-erasure/src/deletion_proof.rs:481` |
| `uncovered_layers` | `Vec<UncoveredLayer>` | `crates/contextra-erasure/src/deletion_proof.rs:484` |
| `integrity_warning` | `Option<String>` | `crates/contextra-erasure/src/deletion_proof.rs:487` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn issue<'a>( ctx: &crate::types_local::RequestContext, saga: &crate::types_local::SagaRecord, witnesses: &[LayerWitness], excluded: &[ExcludedScope], custody: CustodyLevel, key: impl Into<SigningKeyHandle<'a>>, clock: &dyn crate::types_local::Clock, ) -> Result<Self>` | `crates/contextra-erasure/src/deletion_proof.rs:505` | Issues a new `DeletionProof` based on verified `LayerWitness` measurements. # Fail-Closed Validation (INV-WITNESS-MINT-1 & REQ-PRF-2) Returns an error if any pr |
| `pub fn create( scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, proof_key: &[u8], ) -> Result<Self>` | `crates/contextra-erasure/src/deletion_proof.rs:713` | Erstellt und signiert einen DeletionProof nach Layer-Bereinigung. AUFRUFREIHENFOLGE (INV-DELETION-1): 1. Alle covered_layers physisch bereinigen 2. WAL-Commit m |
| `pub fn create_with_wal_receipt( scope: DeletionScope, mut deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, wal_chain_receipt: Option<[u8` | `crates/contextra-erasure/src/deletion_proof.rs:733` | Erstellt und signiert einen DeletionProof (Version 2) inklusive optionaler WAL-HMAC-Kettenquittung. |
| `pub fn create_v3( scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, attested_at: i64, graph_repair: &[GraphRepairAttestation], signing_key: &ed25519_dalek::SigningKey, ) -> Result<Self>` | `crates/contextra-erasure/src/deletion_proof.rs:799` | Erstellt und signiert einen DeletionProof (Version 3) mit Ed25519. |
| `pub fn create_v3_with_audit_position( scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, audit_chain_position: Option<u64>, attested_at: i64, graph_repair: &[GraphRepairAttestation], signing_key: &ed25519_dalek::SigningKey, ) -> Result<Self>` | `crates/contextra-erasure/src/deletion_proof.rs:824` | Erstellt und signiert einen DeletionProof (Version 3, Ed25519) inklusive Verweis auf den Audit-Chain-Eintrag. |
| `pub fn create_with_wal_receipt_v3( scope: DeletionScope, deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, wal_chain_receipt: Option<[u8` | `crates/contextra-erasure/src/deletion_proof.rs:851` | Erstellt und signiert einen DeletionProof (Version 3, Ed25519) inklusive optionaler WAL-HMAC-Kettenquittung. |
| `pub fn create_full_v3( scope: DeletionScope, mut deleted_keys: Vec<Vec<u8>>, deleted_after_tx: TxId, covered_layers: Vec<LayerCleanupProof>, excluded_scopes: Vec<ExcludedScope>, wal_chain_receipt: Option<[u8` | `crates/contextra-erasure/src/deletion_proof.rs:878` | Erstellt und signiert einen DeletionProof (Version 3, Ed25519) mit allen optionalen Erweiterungen. |
| `pub fn signature_version_typed( &self, ) -> std::result::Result<SignatureVersion, DeletionProofError>` | `crates/contextra-erasure/src/deletion_proof.rs:939` | Parst und liefert die typsichere Signaturversion des Beweises. # Errors Gibt [`DeletionProofError::UnsupportedVersion`] zurück, wenn `signature_version` einen u |
| `pub fn verify<'a>(&self, key: impl Into<VerificationKey<'a>>) -> Result<bool>` | `crates/contextra-erasure/src/deletion_proof.rs:948` | Verifiziert Signatur. Referenz: Befund v17 Teil 10.1 & INVARIANTE INV-DELETION-1. NICHT-GARANTIE: Prüft nur Signatur, nicht ob Storage tatsächlich bereinigt ist |
| `pub fn verify_external( &self, public_key: &ed25519_dalek::VerifyingKey, ) -> std::result::Result<(), CryptoError>` | `crates/contextra-erasure/src/deletion_proof.rs:1178` | Verifies this proof using ONLY the provided Ed25519 public key. Does NOT require access to any `KeyManager` state — this is the verification path intended for t |
| `pub fn export_for_audit(&self) -> Result<String>` | `crates/contextra-erasure/src/deletion_proof.rs:1205` | Exportiert Proof als JSON für Compliance-Dokumentation. ExcludedScope-Liste ist maschinenlesbar enthalten. |
| `pub fn tenant_id(&self) -> TenantId` | `crates/contextra-erasure/src/deletion_proof.rs:1232` | Gibt die Tenant-ID aus dem Scope zurück. |

#### `struct DeletionProofKeyPair`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:127`
```rust
pub struct DeletionProofKeyPair
```
Dokumentation: KeyPair for Ed25519 signing and verification of DeletionProofs (version 3).
- Impl-Block `crates/contextra-erasure/src/deletion_proof.rs:133`
- Impl-Block `crates/contextra-erasure/src/ed25519_proof.rs:65`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `verifying_key` | `ed25519_dalek::VerifyingKey` | `crates/contextra-erasure/src/deletion_proof.rs:130` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn generate() -> Self` | `crates/contextra-erasure/src/deletion_proof.rs:135` | Generates a new random Ed25519 keypair using OsRng. |
| `pub fn verifying_key_bytes(&self) -> [u8` | `crates/contextra-erasure/src/deletion_proof.rs:146` | Returns the 32-byte representation of the verifying public key. |
| `pub fn signing_key(&self) -> &ed25519_dalek::SigningKey` | `crates/contextra-erasure/src/deletion_proof.rs:151` | Returns a reference to the signing key. |
| `pub fn generate() -> Self` | `crates/contextra-erasure/src/ed25519_proof.rs:67` | Generiert ein neues Ed25519-Schlüsselpaar unter Nutzung des System-RNGs (`OsRng`). |
| `pub fn generate_with_rng<R: RngCore + CryptoRng>(rng: &mut R) -> Self` | `crates/contextra-erasure/src/ed25519_proof.rs:73` | Generiert ein neues Ed25519-Schlüsselpaar unter Nutzung eines beliebigen RNGs. |
| `pub fn from_signing_key(signing_key: ed25519_dalek::SigningKey) -> Self` | `crates/contextra-erasure/src/ed25519_proof.rs:85` | Erzeugt ein `DeletionProofKeyPair` aus einem bestehenden Signierschlüssel. |
| `pub fn verifying_key_bytes(&self) -> [u8` | `crates/contextra-erasure/src/ed25519_proof.rs:94` | Gibt die Bytes des öffentlichen Verifikationsschlüssels (32 Bytes) zurück. |
| `pub fn signing_key(&self) -> &ed25519_dalek::SigningKey` | `crates/contextra-erasure/src/ed25519_proof.rs:99` | Gibt eine Referenz auf den Signierschlüssel zurück. |

#### `struct DeletionProofVerifier`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:2697`
```rust
pub struct DeletionProofVerifier
```
Dokumentation: Public facade for standalone third-party verification of deletion proofs.
- Impl-Block `crates/contextra-erasure/src/deletion_proof.rs:2699`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn verify_standalone( proof: &DeletionProof, verifying_key_bytes: &[u8` | `crates/contextra-erasure/src/deletion_proof.rs:2701` | Verifies a single standalone `DeletionProof` using a public key. |
| `pub fn verify_standalone_for_tenant( proof: &DeletionProof, expected_tenant: TenantId, verifying_key_bytes: &[u8` | `crates/contextra-erasure/src/deletion_proof.rs:2730` | Verifies a `DeletionProof` and checks tenant binding. |

#### `struct GraphRepairAttestation`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:98`
```rust
pub struct GraphRepairAttestation
```
Dokumentation: Attestation confirming synchronous neighborhood graph repair for vector deletions.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_id` | `DocId` | `crates/contextra-erasure/src/deletion_proof.rs:100` |
| `verified_no_ghost_pointers` | `bool` | `crates/contextra-erasure/src/deletion_proof.rs:102` |
| `attested_at` | `i64` | `crates/contextra-erasure/src/deletion_proof.rs:104` |

#### `struct LayerCleanupProof`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:232`
```rust
pub struct LayerCleanupProof
```
Dokumentation: Beweis, dass ein bestimmter DeletionLayer physisch bereinigt wurde. Kann NUR von den jeweiligen Bereinigungsfunktionen der Storage-Layer erzeugt werden (siehe `new_after_physical_cleanup`), niemals direkt frei durch Aufrufer von `DeletionProof::create()`.
- Impl-Block `crates/contextra-erasure/src/deletion_proof.rs:237`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new_after_verified_empty( layer: DeletionLayer, remaining_live_entries: usize, ) -> Result<Self>` | `crates/contextra-erasure/src/deletion_proof.rs:248` | Erzeugt einen Proof NUR, wenn `remaining_live_entries == 0` — also nur dann, wenn der Aufrufer nachweislich (durch einen Re-Scan des betroffenen Storage-Bereich |
| `pub fn verify_and_create<F>(layer: DeletionLayer, verification: F) -> Result<Self> where F: FnOnce() -> Result<bool>,` | `crates/contextra-erasure/src/deletion_proof.rs:276` | Erzeugt einen Proof für `layer` NUR, wenn `verification` bestätigt, dass die physische Bereinigung tatsächlich abgeschlossen ist. `verification` MUSS eine echte |
| `pub fn layer(&self) -> &DeletionLayer` | `crates/contextra-erasure/src/deletion_proof.rs:295` | Gibt den zugrundeliegenden DeletionLayer zurück. |

#### `struct LayerWitnessMeasurement`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:381`
```rust
pub struct LayerWitnessMeasurement
```
Dokumentation: Measurement details for a single witnessed layer in V5 proof.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `layer` | `DeletionLayer` | `crates/contextra-erasure/src/deletion_proof.rs:383` |
| `technique` | `ScanTechnique` | `crates/contextra-erasure/src/deletion_proof.rs:386` |
| `scanned` | `u64` | `crates/contextra-erasure/src/deletion_proof.rs:388` |
| `remaining` | `u64` | `crates/contextra-erasure/src/deletion_proof.rs:390` |
| `control_matches` | `u64` | `crates/contextra-erasure/src/deletion_proof.rs:392` |
| `digest` | `[u8; 32]` | `crates/contextra-erasure/src/deletion_proof.rs:394` |

#### `struct UncoveredLayer`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:327`
```rust
pub struct UncoveredLayer
```
Dokumentation: Uncovered layer explanation for scope.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `layer` | `DeletionLayer` | `crates/contextra-erasure/src/deletion_proof.rs:329` |
| `reason` | `String` | `crates/contextra-erasure/src/deletion_proof.rs:331` |

#### `struct VerificationResult`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:2670`
```rust
pub struct VerificationResult
```
Dokumentation: Structured outcome of third-party deletion proof verification.
- Impl-Block `crates/contextra-erasure/src/deletion_proof.rs:2677`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `is_valid` | `bool` | `crates/contextra-erasure/src/deletion_proof.rs:2672` |
| `error_reason` | `Option<VerificationErrorReason>` | `crates/contextra-erasure/src/deletion_proof.rs:2674` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn success() -> Self` | `crates/contextra-erasure/src/deletion_proof.rs:2679` | Creates a successful verification result. |
| `pub fn failure(reason: VerificationErrorReason) -> Self` | `crates/contextra-erasure/src/deletion_proof.rs:2687` | Creates a failed verification result with an explicit reason. |

#### `enum CustodyLevel`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:158`
```rust
pub enum CustodyLevel
```
Dokumentation: Custody level for cryptographic key management during deletion proof generation.

Varianten:

- `Local` — `crates/contextra-erasure/src/deletion_proof.rs:160`
- `Sovereign` — `crates/contextra-erasure/src/deletion_proof.rs:162`
- `Escrow` — `crates/contextra-erasure/src/deletion_proof.rs:164`

#### `enum DeletionLayer`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:303`
```rust
pub enum DeletionLayer
```
Dokumentation: Layer-explizite Coverage-Deklaration. Jeder Layer MUSS physisch bereinigt sein bevor er hier deklariert wird.

Varianten:

- `LsmMemtable` — `crates/contextra-erasure/src/deletion_proof.rs:305`
- `SsTableAllLevels` — `crates/contextra-erasure/src/deletion_proof.rs:307`
- `HnswIndex` — `crates/contextra-erasure/src/deletion_proof.rs:309`
- `WalAllSegments{` — `crates/contextra-erasure/src/deletion_proof.rs:311`
- `CsrGraph` — `crates/contextra-erasure/src/deletion_proof.rs:316`
- `KvCacheSegments` — `crates/contextra-erasure/src/deletion_proof.rs:318`
- `EmbeddingCache` — `crates/contextra-erasure/src/deletion_proof.rs:320`
- `TextIndex` — `crates/contextra-erasure/src/deletion_proof.rs:322`

#### `enum DeletionScope`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:408`
```rust
pub enum DeletionScope
```
Dokumentation: Target scope for physical deletion.

Varianten:

- `Document{` — `crates/contextra-erasure/src/deletion_proof.rs:410`
- `Collection{` — `crates/contextra-erasure/src/deletion_proof.rs:417`
- `Tenant{` — `crates/contextra-erasure/src/deletion_proof.rs:424`

#### `enum ExcludedScope`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:399`
```rust
pub enum ExcludedScope
```
Dokumentation: Explizite Nicht-Abdeckung — maschinenlesbar für Audit-Systeme.

Varianten:

- `ConsolidatedAndDistilled` — `crates/contextra-erasure/src/deletion_proof.rs:401`
- `LlmParameterMemory` — `crates/contextra-erasure/src/deletion_proof.rs:403`

#### `enum SigningKeyHandle`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:169`
```rust
pub enum SigningKeyHandle<'a>
```
Dokumentation: Handle to the key used for signing deletion proofs.
- Impl-Block (Trait `From<&'a [u8]>`) `crates/contextra-erasure/src/deletion_proof.rs:176`
- Impl-Block (Trait `From<&'a [u8; N]>`) `crates/contextra-erasure/src/deletion_proof.rs:182`
- Impl-Block (Trait `From<&'a ed25519_dalek::SigningKey>`) `crates/contextra-erasure/src/deletion_proof.rs:188`

Varianten:

- `Hmac(&'a [u8])` — `crates/contextra-erasure/src/deletion_proof.rs:171`
- `Ed25519(&'a ed25519_dalek::SigningKey)` — `crates/contextra-erasure/src/deletion_proof.rs:173`

#### `enum VerificationErrorReason`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:2629`
```rust
pub enum VerificationErrorReason
```
Dokumentation: Detailed failure reasons for third-party verification of deletion proofs.

Varianten:

- `InvalidSignature(String)` — `crates/contextra-erasure/src/deletion_proof.rs:2631`
- `ChainBroken{` — `crates/contextra-erasure/src/deletion_proof.rs:2633`
- `HeadMismatch{` — `crates/contextra-erasure/src/deletion_proof.rs:2640`
- `UnsupportedVersion(u8)` — `crates/contextra-erasure/src/deletion_proof.rs:2647`
- `TruncatedChain{` — `crates/contextra-erasure/src/deletion_proof.rs:2649`
- `TenantMismatch{` — `crates/contextra-erasure/src/deletion_proof.rs:2656`
- `EmptyChain` — `crates/contextra-erasure/src/deletion_proof.rs:2663`
- `InvalidKey(String)` — `crates/contextra-erasure/src/deletion_proof.rs:2665`

#### `enum VerificationKey`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:196`
```rust
pub enum VerificationKey<'a>
```
Dokumentation: Key parameter for verification (either HMAC-SHA256 byte slice or Ed25519 VerifyingKey).
- Impl-Block (Trait `From<&'a [u8]>`) `crates/contextra-erasure/src/deletion_proof.rs:203`
- Impl-Block (Trait `From<&'a [u8; N]>`) `crates/contextra-erasure/src/deletion_proof.rs:209`
- Impl-Block (Trait `From<&'a Vec<u8>>`) `crates/contextra-erasure/src/deletion_proof.rs:215`
- Impl-Block (Trait `From<&'a ed25519_dalek::VerifyingKey>`) `crates/contextra-erasure/src/deletion_proof.rs:221`

Varianten:

- `Hmac(&'a [u8])` — `crates/contextra-erasure/src/deletion_proof.rs:198`
- `Ed25519(&'a ed25519_dalek::VerifyingKey)` — `crates/contextra-erasure/src/deletion_proof.rs:200`

#### `trait LayerWitnessExt`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:38`
```rust
pub trait LayerWitnessExt
```
Dokumentation: Extension trait providing accessors for `LayerWitness` without mutating `contextra-ports`.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn wal_seq_after(&self) -> Option<u64>` | `crates/contextra-erasure/src/deletion_proof.rs:40` | Extracts the `wal_seq_after` if layer is `WalAllSegments`, or `None`. |
| `fn control_matches(&self) -> u64` | `crates/contextra-erasure/src/deletion_proof.rs:42` | Extracts the positive control check match count. |

#### `fn compute_wal_delete_receipt`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:1243`
```rust
pub fn compute_wal_delete_receipt( prev_hmac: &[u8
```
Dokumentation: Erzeugt eine WAL-Löschquittung H(hmac_prev || delete_event) für den Nachweis auf WAL-Ebene gemäß DSGVO Art. 17.

#### `fn hash_deleted_keys_length_prefixed`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:115`
```rust
pub fn hash_deleted_keys_length_prefixed(deleted_keys: &[Vec<u8>]) -> [u8
```
Dokumentation: Berechnet den Blake3-Hash einer deterministisch sortierten Liste gelöschter Schlüssel mit Längenpräfix. DOKUMENTATION ZUM KOLLISIONSRISIKO: Ohne Längenpräfix pro Element (z. B. bloße Konkatenation) erzeugen unterschiedliche Key-Listen wie `["ab", "c"]` und `["a", "bc"]` denselben Hash-Wert, da die Elementgrenzen im Byte-Strom nicht kodiert sind. Durch das Voranstellen der Schlüssellänge als 4-Byte Little-Endian integer (`(key.len() as u32).to_le_bytes()`) vor jedem Schlüssel-Byte-Array wird eine eindeutige, kollisionsfreie Kodierung für jede Sequenz von Schlüssel-Bytes.

#### `fn map_witness_layer_to_deletion_layer`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:67`
```rust
pub fn map_witness_layer_to_deletion_layer( layer: &WitnessLayer, witness: &LayerWitness, ) -> Result<DeletionLayer>
```
Dokumentation: Mappt `WitnessLayer` erschöpfend auf `DeletionLayer` ohne `_`-Zweig.

#### `fn verify_wal_delete_receipt`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:1255`
```rust
pub fn verify_wal_delete_receipt( receipt: &[u8
```
Dokumentation: Verifiziert eine WAL-Löschquittung in O(1) konstanter Zeit ohne Klartextzugang.

#### `type CryptoError`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:28`
```rust
pub type CryptoError = crate::error::ErasureError
```
Dokumentation: Typalias für CryptoError.

#### `type Result`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:34`
```rust
pub type Result<T> = std::result::Result<T, crate::error::ErasureError>
```
Dokumentation: Typalias für Result.

### Modul `deletion_proof::scan_technique_serde`

#### `fn deserialize`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:363`
```rust
pub fn deserialize<'de, D>(deserializer: D) -> Result<ScanTechnique, D::Error> where D: Deserializer<'de>,
```

#### `fn serialize`
- Ort: `crates/contextra-erasure/src/deletion_proof.rs:348`
```rust
pub fn serialize<S>(tech: &ScanTechnique, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer,
```

### Modul `ed25519_proof`

#### `struct DeletionProofKeyPair`
- Ort: `crates/contextra-erasure/src/ed25519_proof.rs:57`
```rust
pub struct DeletionProofKeyPair
```
Dokumentation: Schlüsselpaar für Ed25519-Attestationsbeweise. Signierschlüssel wird bei Drop automatisch zeroisiert (`ZeroizeOnDrop`).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `signing_key` | `ed25519_dalek::SigningKey` | `crates/contextra-erasure/src/ed25519_proof.rs:59` |
| `verifying_key` | `ed25519_dalek::VerifyingKey` | `crates/contextra-erasure/src/ed25519_proof.rs:62` |

#### `enum DeletionProofError`
- Ort: `crates/contextra-erasure/src/ed25519_proof.rs:39`
```rust
pub enum DeletionProofError
```
Dokumentation: Fehlerzustände bei der Verifikation von Ed25519-Attestationsbeweisen.

Varianten:

- `InvalidVerifyingKey(String)` — `crates/contextra-erasure/src/ed25519_proof.rs:42`
- `InvalidSignature(String)` — `crates/contextra-erasure/src/ed25519_proof.rs:46`
- `UnsupportedVersion(u8)` — `crates/contextra-erasure/src/ed25519_proof.rs:50`

#### `enum SignatureVersion`
- Ort: `crates/contextra-erasure/src/ed25519_proof.rs:130`
```rust
pub enum SignatureVersion
```
Dokumentation: `SignatureVersion`-Enum für typsichere Versionierung von DeletionProof-Signaturen.
- Impl-Block `crates/contextra-erasure/src/ed25519_proof.rs:141`
- Impl-Block (Trait `TryFrom<u8>`) `crates/contextra-erasure/src/ed25519_proof.rs:154`

Varianten:

- `V1= 1` — `crates/contextra-erasure/src/ed25519_proof.rs:132`
- `V2= 2` — `crates/contextra-erasure/src/ed25519_proof.rs:134`
- `V3= 3` — `crates/contextra-erasure/src/ed25519_proof.rs:136`
- `V5= 5` — `crates/contextra-erasure/src/ed25519_proof.rs:138`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn as_u8(self) -> u8` | `crates/contextra-erasure/src/ed25519_proof.rs:143` | Liefert den numerischen Wert als `u8`. |

#### `fn hash_deleted_keys_length_prefixed`
- Ort: `crates/contextra-erasure/src/ed25519_proof.rs:27`
```rust
pub fn hash_deleted_keys_length_prefixed(keys: &[Vec<u8>]) -> [u8
```
Dokumentation: Berechnet den BLAKE3-Hash über längenpräfixierte gelöschte Schlüssel.

#### `fn sign_deletion_proof_v3`
- Ort: `crates/contextra-erasure/src/ed25519_proof.rs:105`
```rust
pub fn sign_deletion_proof_v3(keypair: &DeletionProofKeyPair, proof_payload: &[u8]) -> [u8
```
Dokumentation: Signiert den Payload eines DeletionProofs unter Nutzung von Ed25519 (Version 3).

#### `fn verify_deletion_proof_v3`
- Ort: `crates/contextra-erasure/src/ed25519_proof.rs:112`
```rust
pub fn verify_deletion_proof_v3( verifying_key_bytes: &[u8
```
Dokumentation: Verifiziert die Ed25519-Signatur eines DeletionProofs unabhängig von internem Contextra-State (Version 3).

### Modul `erasure_domain`

#### `struct Armed`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:454`
```rust
pub struct Armed(())
```

#### `struct CollectionRef`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:16`
```rust
pub struct CollectionRef(pub Vec<u8>)
```
Dokumentation: Collection-Referenz (SPEC 7.3).

#### `struct ControlEvidence`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:462`
```rust
pub struct ControlEvidence(())
```
- Impl-Block `crates/contextra-erasure/src/erasure_domain.rs:464`
- Impl-Block `crates/contextra-erasure/src/types_local.rs:741`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn observed() -> Self` | `crates/contextra-erasure/src/erasure_domain.rs:467` | Bezeugt das Auffinden des Kontrollmusters (nur in autorisierten Modulen aufgerufen). |
| `pub const fn observed() -> Self` | `crates/contextra-erasure/src/types_local.rs:744` | Bezeugt das Auffinden des Kontrollmusters. |

#### `struct DocKey`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:24`
```rust
pub struct DocKey(pub Vec<u8>)
```
Dokumentation: Dokumentenschlüssel (SPEC 7.2).

#### `struct DocRef`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:28`
```rust
pub struct DocRef
```
Dokumentation: Eindeutige Dokumentenadresse.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `subject` | `Option<SubjectRef>` | `crates/contextra-erasure/src/erasure_domain.rs:30` |
| `key` | `DocKey` | `crates/contextra-erasure/src/erasure_domain.rs:32` |

#### `struct EraseReceipt`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:153`
```rust
pub struct EraseReceipt
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `ErasureId` | `crates/contextra-erasure/src/erasure_domain.rs:155` |
| `status` | `ErasureStatus` | `crates/contextra-erasure/src/erasure_domain.rs:157` |
| `gated_at` | `Timestamp` | `crates/contextra-erasure/src/erasure_domain.rs:159` |

#### `struct EraseRequest`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:136`
```rust
pub struct EraseRequest
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `target` | `EraseTarget` | `crates/contextra-erasure/src/erasure_domain.rs:138` |
| `timing` | `Timing` | `crates/contextra-erasure/src/erasure_domain.rs:140` |
| `retention` | `RetentionAttestation` | `crates/contextra-erasure/src/erasure_domain.rs:142` |
| `mode` | `EraseMode` | `crates/contextra-erasure/src/erasure_domain.rs:144` |

#### `struct LayerId`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:168`
```rust
pub struct LayerId(pub [u8
```
- Impl-Block `crates/contextra-erasure/src/erasure_domain.rs:170`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/erasure_domain.rs:182`
- Impl-Block `crates/contextra-erasure/src/types_local.rs:681`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:692`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-erasure/src/types_local.rs:698`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(bytes: [u8` | `crates/contextra-erasure/src/erasure_domain.rs:172` | Erzeugt eine neue `LayerId`. |
| `pub const fn as_bytes(&self) -> &[u8` | `crates/contextra-erasure/src/erasure_domain.rs:177` | Liefert die rohen Bytes der Schicht-ID. |
| `pub const fn new(raw: [u8` | `crates/contextra-erasure/src/types_local.rs:683` | Erzeugt eine neue `LayerId`. |
| `pub fn as_bytes(&self) -> &[u8` | `crates/contextra-erasure/src/types_local.rs:687` | Liefert die Bytes als Slice. |

#### `struct LayerWitness`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:575`
```rust
pub struct LayerWitness
```
- Impl-Block (Trait `LayerWitnessExt`) `crates/contextra-erasure/src/deletion_proof.rs:45`
- Impl-Block `crates/contextra-erasure/src/erasure_domain.rs:585`
- Impl-Block `crates/contextra-erasure/src/types_local.rs:965`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:1038`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn layer(&self) -> LayerId` | `crates/contextra-erasure/src/erasure_domain.rs:588` | Liefert die Schicht-ID. |
| `pub fn class(&self) -> MeasureClass` | `crates/contextra-erasure/src/erasure_domain.rs:594` | Liefert die Messklasse (MK1-MK3). |
| `pub fn method_id(&self) -> u16` | `crates/contextra-erasure/src/erasure_domain.rs:600` | Liefert die Methoden-ID. |
| `pub fn files(&self) -> u64` | `crates/contextra-erasure/src/erasure_domain.rs:606` | Liefert die Anzahl geprüfter Dateien. |
| `pub fn bytes(&self) -> u64` | `crates/contextra-erasure/src/erasure_domain.rs:612` | Liefert die Anzahl geprüfter Bytes. |
| `pub fn duration(&self) -> Duration` | `crates/contextra-erasure/src/erasure_domain.rs:618` | Liefert die Scandauer. |
| `pub fn hash(&self) -> &[u8` | `crates/contextra-erasure/src/erasure_domain.rs:624` | Liefert den kryptografischen Hash des Prüflaufs. |
| `pub const fn matches(&self) -> u64` | `crates/contextra-erasure/src/erasure_domain.rs:630` | matches() ist per Konstruktion immer 0 (I-19). |
| `pub const fn empty() -> Self` | `crates/contextra-erasure/src/types_local.rs:967` | Erzeugt einen leeren Zeugen. |
| `pub const fn from_hash(witness_hash: [u8` | `crates/contextra-erasure/src/types_local.rs:980` | Erzeugt einen Zeugen aus einem Hash. |
| `pub fn layer(&self) -> LayerId` | `crates/contextra-erasure/src/types_local.rs:993` | Liefert die Schicht-ID. |
| `pub fn class(&self) -> MeasureClass` | `crates/contextra-erasure/src/types_local.rs:998` | Liefert die Messklasse. |
| `pub fn method_id(&self) -> u16` | `crates/contextra-erasure/src/types_local.rs:1003` | Liefert die Methoden-ID. |
| `pub fn files(&self) -> u64` | `crates/contextra-erasure/src/types_local.rs:1008` | Liefert geprüfte Dateien. |
| `pub fn bytes(&self) -> u64` | `crates/contextra-erasure/src/types_local.rs:1013` | Liefert geprüfte Bytes. |
| `pub fn duration(&self) -> Duration` | `crates/contextra-erasure/src/types_local.rs:1018` | Liefert Scandauer. |
| `pub fn witness_hash(&self) -> &[u8` | `crates/contextra-erasure/src/types_local.rs:1023` | Liefert den Hash. |
| `pub fn hash(&self) -> &[u8` | `crates/contextra-erasure/src/types_local.rs:1028` | Alias für witness_hash. |
| `pub const fn matches(&self) -> u64` | `crates/contextra-erasure/src/types_local.rs:1033` | matches() ist per Konstruktion immer 0. |

#### `struct Residue`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:478`
```rust
pub struct Residue
```
- Impl-Block (Trait `fmt::Display`) `crates/contextra-erasure/src/erasure_domain.rs:483`
- Impl-Block (Trait `std::error::Error`) `crates/contextra-erasure/src/erasure_domain.rs:493`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-erasure/src/types_local.rs:756`
- Impl-Block (Trait `std::error::Error`) `crates/contextra-erasure/src/types_local.rs:766`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `matches` | `u64` | `crates/contextra-erasure/src/erasure_domain.rs:480` |

#### `struct RetentionAttestation`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:121`
```rust
pub struct RetentionAttestation
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `code` | `RetentionCode` | `crates/contextra-erasure/src/erasure_domain.rs:123` |
| `attested_by` | `OperatorRef` | `crates/contextra-erasure/src/erasure_domain.rs:125` |
| `attested_at` | `Timestamp` | `crates/contextra-erasure/src/erasure_domain.rs:127` |

#### `struct Scanner`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:501`
```rust
pub struct Scanner<S>
```
- Impl-Block `crates/contextra-erasure/src/erasure_domain.rs:510`
- Impl-Block `crates/contextra-erasure/src/erasure_domain.rs:536`
- Impl-Block `crates/contextra-erasure/src/types_local.rs:876`
- Impl-Block `crates/contextra-erasure/src/types_local.rs:902`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:943`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(layer: LayerId, class: MeasureClass, method_id: u16) -> Self` | `crates/contextra-erasure/src/erasure_domain.rs:512` | Erzeugt einen neuen unbewaffneten Scanner. |
| `pub fn arm(self, _ev: ControlEvidence) -> Scanner<Armed>` | `crates/contextra-erasure/src/erasure_domain.rs:524` | Einzige Brücke nach Armed; erfordert Positivkontrollbeleg (TY-10). |
| `pub fn record(&mut self, files: u64, bytes: u64)` | `crates/contextra-erasure/src/erasure_domain.rs:538` | Zeichnet geprüfte Dateien und Bytes auf. |
| `pub fn finish( self, matches: u64, duration: Duration, hash: [u8` | `crates/contextra-erasure/src/erasure_domain.rs:546` | Schließt die Messung ab. matches != 0 => Err(Residue). LayerWitness wird ausschließlich bei matches == 0 ausgegeben (TY-10, I-19, D-72). |
| `pub fn new(layer: LayerId, class: MeasureClass, method_id: u16) -> Self` | `crates/contextra-erasure/src/types_local.rs:878` | Erzeugt einen unbewaffneten Scanner. |
| `pub fn arm(self, _ev: ControlEvidence) -> Scanner<Armed>` | `crates/contextra-erasure/src/types_local.rs:890` | Schaltet Scanner über Positivkontrolle scharf. |
| `pub fn armed(layer: LayerId, class: MeasureClass, method_id: u16) -> Self` | `crates/contextra-erasure/src/types_local.rs:904` | Erzeugt einen direkt scharfen Scanner (z.B. für Tests/Mocks). |
| `pub fn record(&mut self, files: u64, bytes: u64)` | `crates/contextra-erasure/src/types_local.rs:916` | Zeichnet geprüfte Dateien und Bytes auf. |
| `pub fn finish( self, matches: u64, duration: Duration, hash: [u8` | `crates/contextra-erasure/src/types_local.rs:922` | Schließt Messung ab. |

#### `struct SubjectRef`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:20`
```rust
pub struct SubjectRef(pub Vec<u8>)
```
Dokumentation: Betroffenen-Referenz (SPEC 7.2, 7.3).

#### `struct TenantRef`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:12`
```rust
pub struct TenantRef(pub Vec<u8>)
```
Dokumentation: Mandanten-Referenz (SPEC 7.3).

#### `struct Unarmed`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:447`
```rust
pub struct Unarmed
```

#### `enum EraseMode`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:87`
```rust
pub enum EraseMode
```

Varianten:

- `Erase` — `crates/contextra-erasure/src/erasure_domain.rs:89`
- `Restrict` — `crates/contextra-erasure/src/erasure_domain.rs:91`

#### `enum EraseTarget`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:37`
```rust
pub enum EraseTarget
```
Dokumentation: Löschziel (SPEC 7.3).
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:824`

Varianten:

- `Document{` — `crates/contextra-erasure/src/erasure_domain.rs:39`
- `Subject{` — `crates/contextra-erasure/src/erasure_domain.rs:48`
- `Collection{` — `crates/contextra-erasure/src/erasure_domain.rs:55`
- `Tenant{` — `crates/contextra-erasure/src/erasure_domain.rs:62`

#### `enum ErasureStatus`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:255`
```rust
pub enum ErasureStatus
```

Varianten:

- `Requested` — `crates/contextra-erasure/src/erasure_domain.rs:257`
- `Gated` — `crates/contextra-erasure/src/erasure_domain.rs:259`
- `KeyDestroyed` — `crates/contextra-erasure/src/erasure_domain.rs:261`
- `Purging` — `crates/contextra-erasure/src/erasure_domain.rs:263`
- `Verifying` — `crates/contextra-erasure/src/erasure_domain.rs:265`
- `Completed` — `crates/contextra-erasure/src/erasure_domain.rs:267`
- `Restricted` — `crates/contextra-erasure/src/erasure_domain.rs:269`
- `Lifted` — `crates/contextra-erasure/src/erasure_domain.rs:271`
- `Pending{` — `crates/contextra-erasure/src/erasure_domain.rs:273`
- `Refused(RefusalReason)` — `crates/contextra-erasure/src/erasure_domain.rs:280`

#### `enum GateVariant`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:304`
```rust
pub enum GateVariant
```

Varianten:

- `Erased` — `crates/contextra-erasure/src/erasure_domain.rs:306`
- `Restricted` — `crates/contextra-erasure/src/erasure_domain.rs:308`

#### `enum KeyCoverage`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:289`
```rust
pub enum KeyCoverage
```

Varianten:

- `Full` — `crates/contextra-erasure/src/erasure_domain.rs:291`
- `Unlinked` — `crates/contextra-erasure/src/erasure_domain.rs:293`
- `None` — `crates/contextra-erasure/src/erasure_domain.rs:295`

#### `enum MeasureClass`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:317`
```rust
pub enum MeasureClass
```

Varianten:

- `Mk1` — `crates/contextra-erasure/src/erasure_domain.rs:319`
- `Mk2` — `crates/contextra-erasure/src/erasure_domain.rs:321`
- `Mk3` — `crates/contextra-erasure/src/erasure_domain.rs:323`

#### `enum PendingReason`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:198`
```rust
pub enum PendingReason
```

Varianten:

- `Io` — `crates/contextra-erasure/src/erasure_domain.rs:200`
- `SlaExceeded` — `crates/contextra-erasure/src/erasure_domain.rs:202`
- `LayerFailed(LayerId)` — `crates/contextra-erasure/src/erasure_domain.rs:204`
- `WitnessMissing` — `crates/contextra-erasure/src/erasure_domain.rs:206`
- `DiskReserve` — `crates/contextra-erasure/src/erasure_domain.rs:208`

#### `enum Phase`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:230`
```rust
pub enum Phase
```

Varianten:

- `Intent` — `crates/contextra-erasure/src/erasure_domain.rs:232`
- `Gate` — `crates/contextra-erasure/src/erasure_domain.rs:234`
- `KeyDestroy` — `crates/contextra-erasure/src/erasure_domain.rs:236`
- `Purge` — `crates/contextra-erasure/src/erasure_domain.rs:238`
- `Verify` — `crates/contextra-erasure/src/erasure_domain.rs:240`
- `Replace` — `crates/contextra-erasure/src/erasure_domain.rs:242`
- `Supersede` — `crates/contextra-erasure/src/erasure_domain.rs:244`
- `Restrict` — `crates/contextra-erasure/src/erasure_domain.rs:246`

#### `enum RefusalReason`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:217`
```rust
pub enum RefusalReason
```

Varianten:

- `RetentionObligation` — `crates/contextra-erasure/src/erasure_domain.rs:219`
- `InvalidParameters` — `crates/contextra-erasure/src/erasure_domain.rs:221`

#### `enum ResidualRisk`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:373`
```rust
pub enum ResidualRisk
```
- Impl-Block `crates/contextra-erasure/src/erasure_domain.rs:406`

Varianten:

- `FlashRemanence` — `crates/contextra-erasure/src/erasure_domain.rs:375`
- `FsJournalOrCow` — `crates/contextra-erasure/src/erasure_domain.rs:377`
- `WrappedKeyRemanence` — `crates/contextra-erasure/src/erasure_domain.rs:379`
- `FreedBlockRemanence` — `crates/contextra-erasure/src/erasure_domain.rs:381`
- `OldKeyslotRemanence` — `crates/contextra-erasure/src/erasure_domain.rs:383`
- `ProcessMemory` — `crates/contextra-erasure/src/erasure_domain.rs:385`
- `IndexTopologyInference` — `crates/contextra-erasure/src/erasure_domain.rs:387`
- `TextIndexUntilPurge` — `crates/contextra-erasure/src/erasure_domain.rs:389`
- `ReplicaWindow` — `crates/contextra-erasure/src/erasure_domain.rs:391`
- `ExternalCopies` — `crates/contextra-erasure/src/erasure_domain.rs:393`
- `ExternalProcessors` — `crates/contextra-erasure/src/erasure_domain.rs:395`
- `PseudonymCommitment` — `crates/contextra-erasure/src/erasure_domain.rs:397`
- `PurgeScalesWithDb` — `crates/contextra-erasure/src/erasure_domain.rs:399`
- `PurgeDeferred` — `crates/contextra-erasure/src/erasure_domain.rs:401`
- `KeyCompromise` — `crates/contextra-erasure/src/erasure_domain.rs:403`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn id(&self) -> &'static str` | `crates/contextra-erasure/src/erasure_domain.rs:408` | Feste Identifikator-Zeichenkette nach SPEC Anhang D. |
| `pub fn condition(&self) -> RiskCondition` | `crates/contextra-erasure/src/erasure_domain.rs:429` | Statische Bedingung für Audit- und Verifikationswerkzeuge (AR-4). |

#### `enum RetentionCode`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:100`
```rust
pub enum RetentionCode
```

Varianten:

- `NoObligation` — `crates/contextra-erasure/src/erasure_domain.rs:102`
- `ObligationExpired` — `crates/contextra-erasure/src/erasure_domain.rs:104`
- `ObligationActive` — `crates/contextra-erasure/src/erasure_domain.rs:106`
- `LegalClaimsPending` — `crates/contextra-erasure/src/erasure_domain.rs:108`
- `DataSubjectRequestedRestriction` — `crates/contextra-erasure/src/erasure_domain.rs:110`
- `OperatorOverride` — `crates/contextra-erasure/src/erasure_domain.rs:112`

#### `enum RiskCondition`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:358`
```rust
pub enum RiskCondition
```

Varianten:

- `Always` — `crates/contextra-erasure/src/erasure_domain.rs:360`
- `CowOrJournaling` — `crates/contextra-erasure/src/erasure_domain.rs:362`
- `UntilPurgeComplete` — `crates/contextra-erasure/src/erasure_domain.rs:364`

#### `enum ScanMethod`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:332`
```rust
pub enum ScanMethod
```
- Impl-Block `crates/contextra-erasure/src/erasure_domain.rs:341`

Varianten:

- `Mk1` — `crates/contextra-erasure/src/erasure_domain.rs:334`
- `Mk2` — `crates/contextra-erasure/src/erasure_domain.rs:336`
- `Mk3` — `crates/contextra-erasure/src/erasure_domain.rs:338`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn class(&self) -> MeasureClass` | `crates/contextra-erasure/src/erasure_domain.rs:343` | Liefert die zugehörige Messklasse (`MeasureClass`). |

#### `enum Timing`
- Ort: `crates/contextra-erasure/src/erasure_domain.rs:74`
```rust
pub enum Timing
```

Varianten:

- `Now` — `crates/contextra-erasure/src/erasure_domain.rs:76`
- `Batched` — `crates/contextra-erasure/src/erasure_domain.rs:78`

### Modul `error`

#### `enum ErasureError`
- Ort: `crates/contextra-erasure/src/error.rs:7`
```rust
pub enum ErasureError
```
Dokumentation: Oberster Fehlertyp des Löschkerns.

Varianten:

- `UnsupportedProofVersion(u8)` — `crates/contextra-erasure/src/error.rs:10`
- `InvalidProofSignature` — `crates/contextra-erasure/src/error.rs:13`
- `KeyRevoked(String)` — `crates/contextra-erasure/src/error.rs:16`
- `Io(#[from] std::io::Error)` — `crates/contextra-erasure/src/error.rs:20`
- `Storage(String)` — `crates/contextra-erasure/src/error.rs:23`
- `IntegrityViolation` — `crates/contextra-erasure/src/error.rs:26`
- `CryptoError(String)` — `crates/contextra-erasure/src/error.rs:29`
- `Crypto(String)` — `crates/contextra-erasure/src/error.rs:33`
- `Corrupt` — `crates/contextra-erasure/src/error.rs:37`
- `Capacity` — `crates/contextra-erasure/src/error.rs:41`
- `SlotMissing` — `crates/contextra-erasure/src/error.rs:45`
- `InvalidTransition{` — `crates/contextra-erasure/src/error.rs:49`
- `LimitExceeded(String)` — `crates/contextra-erasure/src/error.rs:58`
- `DiskReserve` — `crates/contextra-erasure/src/error.rs:62`
- `Residue(u64)` — `crates/contextra-erasure/src/error.rs:66`
- `Json(String)` — `crates/contextra-erasure/src/error.rs:70`
- `Ledger(String)` — `crates/contextra-erasure/src/error.rs:74`
- `Refused` — `crates/contextra-erasure/src/error.rs:78`
- `NotImplemented(String)` — `crates/contextra-erasure/src/error.rs:82`
- `Internal(String)` — `crates/contextra-erasure/src/error.rs:86`
- `ReplacementMissing` — `crates/contextra-erasure/src/error.rs:90`
- `UpdateMissing` — `crates/contextra-erasure/src/error.rs:94`
- `SameIdentity` — `crates/contextra-erasure/src/error.rs:98`
- `RestrictionViolated(String)` — `crates/contextra-erasure/src/error.rs:102`
- `RestrictedReadAuditRequired` — `crates/contextra-erasure/src/error.rs:106`
- `PurgePhaseTransition{` — `crates/contextra-erasure/src/error.rs:110`
- `PurgePhaseError{` — `crates/contextra-erasure/src/error.rs:119`

### Modul `gate`

#### `mod batch`
- Ort: `crates/contextra-erasure/src/gate.rs:3`
```rust
pub mod batch
```

#### `mod erased_set`
- Ort: `crates/contextra-erasure/src/gate.rs:4`
```rust
pub mod erased_set
```

#### `mod fence`
- Ort: `crates/contextra-erasure/src/gate.rs:5`
```rust
pub mod fence
```

#### `mod log`
- Ort: `crates/contextra-erasure/src/gate.rs:6`
```rust
pub mod log
```

#### `mod read_gate`
- Ort: `crates/contextra-erasure/src/gate.rs:7`
```rust
pub mod read_gate
```

### Modul `gate::batch`

#### `struct BatchGateOutcome`
- Ort: `crates/contextra-erasure/src/gate/batch.rs:85`
```rust
pub struct BatchGateOutcome
```
Dokumentation: Ergebnis eines Gruppen-Commits.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `gated_seq` | `Seq` | `crates/contextra-erasure/src/gate/batch.rs:87` |
| `applied_to_set` | `bool` | `crates/contextra-erasure/src/gate/batch.rs:89` |
| `gates_count` | `usize` | `crates/contextra-erasure/src/gate/batch.rs:91` |

#### `struct BatchGateRequest`
- Ort: `crates/contextra-erasure/src/gate/batch.rs:25`
```rust
pub struct BatchGateRequest
```
Dokumentation: Anfrage für einen Gruppen-Commit.
- Impl-Block `crates/contextra-erasure/src/gate/batch.rs:40`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `gate_id` | `[u8; 16]` | `crates/contextra-erasure/src/gate/batch.rs:27` |
| `lo` | `KeyBuf` | `crates/contextra-erasure/src/gate/batch.rs:29` |
| `hi` | `KeyBuf` | `crates/contextra-erasure/src/gate/batch.rs:31` |
| `is_erased` | `bool` | `crates/contextra-erasure/src/gate/batch.rs:35` |
| `erasure_id` | `[u8; 16]` | `crates/contextra-erasure/src/gate/batch.rs:37` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( gate_id: [u8` | `crates/contextra-erasure/src/gate/batch.rs:42` | Erstellt eine Standard-Gate-Anfrage aus 16-Byte-Schlüsseln. |
| `pub fn with_bounds_65( gate_id: [u8` | `crates/contextra-erasure/src/gate/batch.rs:60` | Erstellt eine Gate-Anfrage mit exakten 65-Byte Grenzen (SPEC 9.1, D-123). |

#### `struct GateBatchCoordinator`
- Ort: `crates/contextra-erasure/src/gate/batch.rs:95`
```rust
pub struct GateBatchCoordinator
```
Dokumentation: Kontext für die koordinierte Ausführung von Gate-Gruppen-Commits.
- Impl-Block `crates/contextra-erasure/src/gate/batch.rs:101`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( barrier: Arc<dyn CommitBarrier>, erased_set: Arc<RwLock<ErasedSet>>, gate_log: Arc<RwLock<GateLog>>, ) -> Self` | `crates/contextra-erasure/src/gate/batch.rs:103` | Erstellt einen neuen Koordinator für Gate-Gruppen-Commits. |
| `pub async fn execute_batch( &self, requests: Vec<BatchGateRequest>, gated_seq: Seq, ) -> Result<BatchGateOutcome, ErasureError>` | `crates/contextra-erasure/src/gate/batch.rs:123` | Führt den Gruppen-Commit für die übergebenen Anfragen unter `CommitBarrier::exclusive` aus. Invarianten und Zusicherungen: - `gate_epoch += 1` vor Schlüsselvern |

#### `fn add_batch`
- Ort: `crates/contextra-erasure/src/gate/batch.rs:210`
```rust
pub fn add_batch( requests: Vec<BatchGateRequest>, current_seq: Seq, ) -> Result<BatchGateOutcome, ErasureError>
```
Dokumentation: Freistehende `add_batch`-Funktion (SPEC §11.3, D-57, D-78). Fügt Gate-Anfragen im Batch mit atomarer Inkrementierung von `gate_epoch` und Aktualisierung des gebundenen `ErasedSet` hinzu.

#### `fn bind_global_erased_set`
- Ort: `crates/contextra-erasure/src/gate/batch.rs:201`
```rust
pub fn bind_global_erased_set(set: Arc<RwLock<ErasedSet>>)
```
Dokumentation: Initialisiert oder bindet das globale ErasedSet für freistehende `add_batch`-Aufrufe.

#### `fn evaluate_restricted_access`
- Ort: `crates/contextra-erasure/src/gate/batch.rs:242`
```rust
pub fn evaluate_restricted_access( purpose: &str, is_restricted_only: bool, batch_size: usize, ) -> Result<bool, ErasureError>
```
Dokumentation: Prüft den Zugriff auf `Restricted`-Daten nach Art. 18 Abs. 2 (SPEC D-116, T-RESTR-2..4). Stapelgrenze höchstens 1 000 Elemente (T-RESTR-4).

### Modul `gate::erased_set`

#### `struct ErasedSet`
- Ort: `crates/contextra-erasure/src/gate/erased_set.rs:72`
```rust
pub struct ErasedSet
```
Dokumentation: Sortierte Bereichsmenge aktiver Gates (ER-11, D-120).
- Impl-Block `crates/contextra-erasure/src/gate/erased_set.rs:79`
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/gate/erased_set.rs:299`
- Impl-Block (Trait `ReadGate`) `crates/contextra-erasure/src/gate/erased_set.rs:305`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/gate/erased_set.rs:81` | Erstellt einen leeren Gate-Satz. |
| `pub fn gate_epoch(&self) -> u64` | `crates/contextra-erasure/src/gate/erased_set.rs:92` | Liefert die aktuelle Gate-Epoche. |
| `pub fn increment_epoch(&self) -> u64` | `crates/contextra-erasure/src/gate/erased_set.rs:98` | Erhöht die Gate-Epoche atomar (vor der Schlüsselvernichtung, I-21). |
| `pub fn add_erased(&mut self, lo: KeyBuf, hi: KeyBuf, gated_seq: Seq)` | `crates/contextra-erasure/src/gate/erased_set.rs:103` | Fügt einen `Erased`-Bereich hinzu. |
| `pub fn add_restricted(&mut self, lo: KeyBuf, hi: KeyBuf, gated_seq: Seq)` | `crates/contextra-erasure/src/gate/erased_set.rs:116` | Fügt einen `Restricted`-Bereich hinzu. |
| `pub fn add_erased_with_bounds( &mut self, gate_id: [u8` | `crates/contextra-erasure/src/gate/erased_set.rs:129` | Fügt einen `Erased`-Bereich mit 65-Byte-Grenzen hinzu. |
| `pub fn add_restricted_with_bounds( &mut self, gate_id: [u8` | `crates/contextra-erasure/src/gate/erased_set.rs:153` | Fügt einen `Restricted`-Bereich mit 65-Byte-Grenzen hinzu. |
| `pub fn add_prefix_seq(&mut self, prefix: [u8` | `crates/contextra-erasure/src/gate/erased_set.rs:177` | Fügt ein PrefixSeq-Gate hinzu (SPEC D-117). |
| `pub fn remove_gate(&mut self, gate_id: &[u8` | `crates/contextra-erasure/src/gate/erased_set.rs:187` | Entfernt ein Gate über dessen ID. |
| `pub fn lift_restriction(&mut self, lo: &KeyBuf, hi: &KeyBuf)` | `crates/contextra-erasure/src/gate/erased_set.rs:194` | Hebt eine Restriktion auf (ER-15). |
| `pub fn remove_erased(&mut self, lo: &KeyBuf, hi: &KeyBuf)` | `crates/contextra-erasure/src/gate/erased_set.rs:200` | Entfernt ein Erased-Gate (bei Completed, ER-12 / L-24). |
| `pub fn check(&self, k: &KeyBuf) -> GateDecision` | `crates/contextra-erasure/src/gate/erased_set.rs:206` | Prüft einen 16-Byte-Schlüssel (O(log n) mit binärer Suche auf sortierten Einträgen). |
| `pub fn check_raw(&self, raw: &[u8` | `crates/contextra-erasure/src/gate/erased_set.rs:218` | Prüft einen kanonischen 78-Byte-Schlüssel (SPEC 9.1, D-123). |
| `pub fn covers(&self, k: &KeyBuf) -> bool` | `crates/contextra-erasure/src/gate/erased_set.rs:232` | Prüft, ob ein 16-Byte KeyBuf gegated ist (Erased oder Restricted, für Replay/Commit, I-2). |
| `pub fn covers_raw(&self, raw: &[u8` | `crates/contextra-erasure/src/gate/erased_set.rs:240` | Prüft, ob ein 78-Byte Schlüssel gegated ist (Erased oder Restricted, für Replay/Commit, I-2). |
| `pub fn gated_seq(&self, k: &KeyBuf) -> Option<Seq>` | `crates/contextra-erasure/src/gate/erased_set.rs:248` | Liefert die Sequenznummer, bei der ein Schlüssel eingesprrt wurde. |
| `pub fn gated_seq_raw(&self, raw: &[u8` | `crates/contextra-erasure/src/gate/erased_set.rs:258` | Liefert die Sequenznummer, bei der ein 78-Byte Schlüssel eingesprrt wurde. |
| `pub fn check_bound(&self, b: &KeyBound) -> GateDecision` | `crates/contextra-erasure/src/gate/erased_set.rs:273` | Prüft eine Grenze. |
| `pub fn variant_for_range(&self, r: &KeyRange) -> RangeGate` | `crates/contextra-erasure/src/gate/erased_set.rs:281` | Bereichsrichtlinie für Kompaktion (I-18): Restricted > Erased > Open. |

### Modul `gate::fence`

#### `struct FenceToken`
- Ort: `crates/contextra-erasure/src/gate/fence.rs:28`
```rust
pub struct FenceToken(pub [u8
```
Dokumentation: Eindeutiger Autorisierungs-Token für einen aktiven Fence. Nur der Inhaber dieses Tokens ist berechtigt, den Fence über [`FenceGate::release_fence`] aufzuheben.
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/gate/fence.rs:30`
- Impl-Block `crates/contextra-erasure/src/gate/fence.rs:36`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn from_bytes(bytes: [u8` | `crates/contextra-erasure/src/gate/fence.rs:39` | Erzeugt einen neuen Token aus Roh-Bytes. |
| `pub const fn as_bytes(&self) -> &[u8` | `crates/contextra-erasure/src/gate/fence.rs:45` | Gibt die zugrundeliegenden Bytes zurück. |

#### `struct InMemoryFenceGate`
- Ort: `crates/contextra-erasure/src/gate/fence.rs:213`
```rust
pub struct InMemoryFenceGate
```
Dokumentation: In-Memory-Referenzimplementierung des [`FenceGate`]-Ports (Klasse A, autark, synchron). Arbeitet synchron ohne externe Runtimes oder Threads.
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/gate/fence.rs:221`
- Impl-Block `crates/contextra-erasure/src/gate/fence.rs:227`
- Impl-Block (Trait `crate::types_local::FenceGate`) `crates/contextra-erasure/src/gate/fence.rs:289`
- Impl-Block (Trait `FenceGate`) `crates/contextra-erasure/src/gate/fence.rs:304`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/gate/fence.rs:230` | Erzeugt eine neue, unbeschädigte Instanz von `InMemoryFenceGate`. |
| `pub fn mark_corrupted(&self)` | `crates/contextra-erasure/src/gate/fence.rs:241` | Markiert den Fence-Zustand explizit als korrumpiert (Fail-Closed-Simulation und -Verifikation). |
| `pub fn inflight_count(&self) -> u32` | `crates/contextra-erasure/src/gate/fence.rs:247` | Gibt die Anzahl aktuell registrierter Inflight-Schreiber zurück. |
| `pub fn is_fenced(&self, key: impl AsRef<[u8]>) -> bool` | `crates/contextra-erasure/src/gate/fence.rs:279` | Synchroner Check auf einen Schlüssel (inherente Methode für Abwärtskompatibilität). |
| `pub fn add_fenced_key(&self, key: [u8` | `crates/contextra-erasure/src/gate/fence.rs:284` | Fügt einen Einzelschlüssel zum Fence hinzu (inherente Hilfsmethode). |

#### `struct WriterGuard`
- Ort: `crates/contextra-erasure/src/gate/fence.rs:153`
```rust
pub struct WriterGuard
```
Dokumentation: RAII-Wächter für einen aktiven Schreiber im System. Solange dieser Wächter existiert, wird der Zähler aktiver Schreiber (`inflight_writers`) hochgehalten. Bei Drop wird der Zähler dekrementiert.
- Impl-Block (Trait `Drop`) `crates/contextra-erasure/src/gate/fence.rs:157`

#### `enum FenceGateError`
- Ort: `crates/contextra-erasure/src/gate/fence.rs:118`
```rust
pub enum FenceGateError
```
Dokumentation: Fehler im Lebenszyklus eines Fences oder bei Schreibzugriffen.

Varianten:

- `Fenced{` — `crates/contextra-erasure/src/gate/fence.rs:121`
- `OverlapConflict{` — `crates/contextra-erasure/src/gate/fence.rs:128`
- `InvalidToken` — `crates/contextra-erasure/src/gate/fence.rs:137`
- `CorruptedState` — `crates/contextra-erasure/src/gate/fence.rs:141`
- `DrainTimeout` — `crates/contextra-erasure/src/gate/fence.rs:145`

#### `enum FenceRange`
- Ort: `crates/contextra-erasure/src/gate/fence.rs:52`
```rust
pub enum FenceRange
```
Dokumentation: Spezifikation des gefencten Schlüsselbereichs.
- Impl-Block `crates/contextra-erasure/src/gate/fence.rs:68`

Varianten:

- `Exact(Vec<u8>)` — `crates/contextra-erasure/src/gate/fence.rs:54`
- `Prefix(Vec<u8>)` — `crates/contextra-erasure/src/gate/fence.rs:56`
- `Range{` — `crates/contextra-erasure/src/gate/fence.rs:58`
- `Global` — `crates/contextra-erasure/src/gate/fence.rs:65`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn contains_key(&self, key: &[u8]) -> bool` | `crates/contextra-erasure/src/gate/fence.rs:71` | Prüft, ob ein gegebener Schlüssel in diesen Bereich fällt. |
| `pub fn overlaps(&self, other: &Self) -> bool` | `crates/contextra-erasure/src/gate/fence.rs:82` | Prüft, ob sich zwei Bereiche überschneiden. |

#### `trait FenceGate`
- Ort: `crates/contextra-erasure/src/gate/fence.rs:164`
```rust
pub trait FenceGate: Send + Sync
```
Dokumentation: Port-Trait für das Fencing von Schlüsselbereichen während des physischen Purge (INV-ERA-5).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn acquire_fence(&self, range: FenceRange) -> Result<FenceToken, FenceGateError>` | `crates/contextra-erasure/src/gate/fence.rs:171` | Blockiert einen Schlüsselbereich für Schreibzugriffe und liefert einen Autorisierungs-Token. # Fehler - [`FenceGateError::OverlapConflict`]: Falls `range` mit e |
| `fn release_fence(&self, token: &FenceToken) -> Result<(), FenceGateError>` | `crates/contextra-erasure/src/gate/fence.rs:178` | Hebt einen aktiven Fence auf. Dies gelingt ausschließlich mit dem ursprünglichen Besitzer-Token. # Fehler - [`FenceGateError::InvalidToken`]: Falls der Token un |
| `fn is_fenced(&self, key: &[u8]) -> bool` | `crates/contextra-erasure/src/gate/fence.rs:181` | Prüft, ob für einen Einzelschlüssel ein aktiver Fence vorliegt. |
| `fn check_write(&self, key: &[u8]) -> Result<WriterGuard, FenceGateError>` | `crates/contextra-erasure/src/gate/fence.rs:190` | Prüft, ob ein Schreibzugriff auf einen Einzelschlüssel zulässig ist. Ist der Schlüssel frei, wird ein RAII-[`WriterGuard`] zurückgegeben. # Fehler - [`FenceGate |
| `fn check_write_batch<'a, I>(&self, keys: I) -> Result<WriterGuard, FenceGateError> where I: IntoIterator<Item = &'a [u8]>` | `crates/contextra-erasure/src/gate/fence.rs:195` | Prüft eine Menge von Schlüsseln atomar auf Schreibzulässigkeit. Wird mindestens ein Schlüssel blockiert, schlägt der gesamte Batch fehl. |
| `fn is_active(&self) -> bool` | `crates/contextra-erasure/src/gate/fence.rs:200` | Gibt an, ob aktuell mindestens ein Fence aktiv ist. |

### Modul `gate::log`

#### `struct GateLog`
- Ort: `crates/contextra-erasure/src/gate/log.rs:57`
```rust
pub struct GateLog
```
Dokumentation: In-Memory- und Disk-Append-Log für Gates (SPEC §11.3, D-58).
- Impl-Block `crates/contextra-erasure/src/gate/log.rs:61`
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/gate/log.rs:198`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/gate/log.rs:63` | Erstellt ein leeres Gate-Log. |
| `pub fn from_entries(entries: Vec<GateLogEntry>) -> Self` | `crates/contextra-erasure/src/gate/log.rs:70` | Lädt Einträge aus einer Liste. |
| `pub fn append(&mut self, entry: GateLogEntry)` | `crates/contextra-erasure/src/gate/log.rs:75` | Fügt einen Eintrag hinzu. |
| `pub fn append_batch(&mut self, new_entries: Vec<GateLogEntry>)` | `crates/contextra-erasure/src/gate/log.rs:80` | Fügt mehrere Einträge im Batch hinzu. |
| `pub fn entries(&self) -> &[GateLogEntry]` | `crates/contextra-erasure/src/gate/log.rs:85` | Liefert alle Einträge. |
| `pub fn len(&self) -> usize` | `crates/contextra-erasure/src/gate/log.rs:90` | Anzahl der Einträge. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-erasure/src/gate/log.rs:95` | Prüft ob das Log leer ist. |
| `pub fn compact(&self) -> Self` | `crates/contextra-erasure/src/gate/log.rs:102` | Kompaktiert das Gate-Log: Entfernt alle `Add`-Einträge, deren `gate_id` durch ein späteres `Remove` gelöscht wurde, und verwirft die `Remove`-Einträge (SPEC 11. |
| `pub fn replay_into(&self, set: &mut crate::gate::erased_set::ErasedSet)` | `crates/contextra-erasure/src/gate/log.rs:127` | Wendet die Einträge des Gate-Logs auf ein `ErasedSet` an (Recovery I-2). |

#### `enum GateLogEntry`
- Ort: `crates/contextra-erasure/src/gate/log.rs:30`
```rust
pub enum GateLogEntry
```
Dokumentation: Mögliche Eintragsarten im Gate-Log (SPEC §11.3, Anhang F.9).

Varianten:

- `Add{` — `crates/contextra-erasure/src/gate/log.rs:32`
- `Remove{` — `crates/contextra-erasure/src/gate/log.rs:49`

#### `enum GatePredicate`
- Ort: `crates/contextra-erasure/src/gate/log.rs:11`
```rust
pub enum GatePredicate
```
Dokumentation: Prädikat eines GateLog-Eintrags nach SPEC F.9.

Varianten:

- `Range{` — `crates/contextra-erasure/src/gate/log.rs:13`
- `PrefixSeq{` — `crates/contextra-erasure/src/gate/log.rs:20`

### Modul `gate::read_gate`

#### `struct GateGuard`
- Ort: `crates/contextra-erasure/src/gate/read_gate.rs:12`
```rust
pub struct GateGuard
```
Dokumentation: Thread-sichere Implementierung des `ReadGate`-Ports (ER-11, D-120).
- Impl-Block `crates/contextra-erasure/src/gate/read_gate.rs:16`
- Impl-Block (Trait `ReadGate`) `crates/contextra-erasure/src/gate/read_gate.rs:84`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(set: Arc<RwLock<ErasedSet>>) -> Self` | `crates/contextra-erasure/src/gate/read_gate.rs:18` | Erstellt einen neuen Gate-Wächter. |
| `pub fn linearized_read<T, F>(&self, k: &KeyBuf, mut f: F) -> Result<T, GateDecision> where F: FnMut() -> Result<T, GateDecision>,` | `crates/contextra-erasure/src/gate/read_gate.rs:28` | Führt einen linearisierten Lesevorgang aus (SPEC D-120, I-21). Prüft das Gate vor und nach der übergebenen Lese-/Entschlüsselungs-Operation `f`. Hat sich `gate_ |
| `pub fn linearized_read_raw<T, F>(&self, raw: &[u8` | `crates/contextra-erasure/src/gate/read_gate.rs:57` | Führt einen linearisierten Lesevorgang auf einem 78-Byte-Schlüssel aus (SPEC D-120, D-123, I-21). |

### Modul `io`

#### `fn read_exact_at`
- Ort: `crates/contextra-erasure/src/io.rs:29`
```rust
pub fn read_exact_at(f: &dyn VfsFile, buf: &mut [u8], offset: u64) -> Result<(), ErasureError>
```
Dokumentation: Liest einen vollständigen Puffer von einer Dateiposition.

#### `fn write_all_at`
- Ort: `crates/contextra-erasure/src/io.rs:11`
```rust
pub fn write_all_at(f: &dyn VfsFile, buf: &[u8], offset: u64) -> Result<(), ErasureError>
```
Dokumentation: Schreibt einen vollständigen Puffer an eine Dateiposition und liefert einen Fehler, wenn weniger als `buf.len()` Bytes geschrieben wurden.

#### `fn zero_at`
- Ort: `crates/contextra-erasure/src/io.rs:54`
```rust
pub fn zero_at(f: &dyn VfsFile, offset: u64, len: usize) -> Result<(), ErasureError>
```
Dokumentation: Schreibt Nullen an eine Dateiposition ohne Heap-Allokation in einer Block-Schleife. Alle I/O-Fehler sind typisiert und enthalten keine Dateipfade oder Kennungen (P13).

### Modul `journal`

#### `mod appendlog`
- Ort: `crates/contextra-erasure/src/journal.rs:3`
```rust
pub mod appendlog
```

#### `mod epoch`
- Ort: `crates/contextra-erasure/src/journal.rs:4`
```rust
pub mod epoch
```

#### `mod record`
- Ort: `crates/contextra-erasure/src/journal.rs:5`
```rust
pub mod record
```

#### `mod reduce`
- Ort: `crates/contextra-erasure/src/journal.rs:6`
```rust
pub mod reduce
```

#### `mod wal`
- Ort: `crates/contextra-erasure/src/journal.rs:7`
```rust
pub mod wal
```

#### `struct ErasureJournal`
- Ort: `crates/contextra-erasure/src/journal.rs:26`
```rust
pub struct ErasureJournal
```
Dokumentation: In-Memory- und Disk-Abstraktion des Journals (Appendlog nach F.9).
- Impl-Block `crates/contextra-erasure/src/journal.rs:30`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/journal.rs:32` | Erstellt ein neues leeres In-Memory-Journal. |
| `pub fn open_durable( vfs: Arc<dyn Vfs>, path: &Path, derived_key: &[u8` | `crates/contextra-erasure/src/journal.rs:39` | Öffnet ein dauerhaftes Journal auf Vfs. |
| `pub fn append(&mut self, entry: JournalEntry)` | `crates/contextra-erasure/src/journal.rs:48` | Fügt einen neuen Eintrag an. |
| `pub fn entries(&self) -> &[JournalEntry]` | `crates/contextra-erasure/src/journal.rs:53` | Gibt alle Einträge zurück. |
| `pub fn recover_open_sagas(&self) -> Vec<EraseSaga>` | `crates/contextra-erasure/src/journal.rs:61` | Rekonstruiert den Zustand aller noch nicht abgeschlossenen Sagas aus dem Journal (SPEC §11.2, §11.6). Jede nicht-vollendete Löschung (nicht `Completed` oder `Re |

### Modul `journal::appendlog`

#### `struct AppendLog`
- Ort: `crates/contextra-erasure/src/journal/appendlog.rs:332`
```rust
pub struct AppendLog
```
Dokumentation: In-Memory-Journal-Implementierung (SPEC §11.6, ER-29).
- Impl-Block `crates/contextra-erasure/src/journal/appendlog.rs:336`
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/journal/appendlog.rs:381`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/journal/appendlog.rs:338` | Erstellt ein leeres Log. |
| `pub fn append(&mut self, entry: JournalEntry)` | `crates/contextra-erasure/src/journal/appendlog.rs:345` | Fügt einen Eintrag hinzu. |
| `pub fn entries(&self) -> &[JournalEntry]` | `crates/contextra-erasure/src/journal/appendlog.rs:350` | Liefert alle Einträge. |
| `pub fn len(&self) -> usize` | `crates/contextra-erasure/src/journal/appendlog.rs:355` | Anzahl der Einträge. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-erasure/src/journal/appendlog.rs:360` | Prüft ob das Log leer ist. |
| `pub fn recover_open_sagas(&self) -> Vec<EraseSaga>` | `crates/contextra-erasure/src/journal/appendlog.rs:365` | Rekonstruiert den Zustand aller noch nicht abgeschlossenen Sagas aus dem Journal (SPEC §11.2, §11.6). |

#### `struct DurableJournal`
- Ort: `crates/contextra-erasure/src/journal/appendlog.rs:25`
```rust
pub struct DurableJournal
```
Dokumentation: Dauerhaftes Journal auf Basis von Vfs (SPEC §11.6, Anhang F.1, F.9).
- Impl-Block `crates/contextra-erasure/src/journal/appendlog.rs:33`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn open_or_create( vfs: Arc<dyn Vfs>, path: &Path, derived_key: &[u8` | `crates/contextra-erasure/src/journal/appendlog.rs:35` | Öffnet ein existierendes Journal oder legt ein neues an. |
| `pub fn append_record(&mut self, rec: &JournalRecord) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/journal/appendlog.rs:181` | Fügt einen neuen Record an und synchronisiert (`fsync`). |
| `pub fn records(&self) -> &[JournalRecord]` | `crates/contextra-erasure/src/journal/appendlog.rs:224` | Liefert alle Records. |
| `pub fn reduce_completed(&mut self) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/journal/appendlog.rs:229` | Führt die Reduktion nach Completed (R13) via atomarem Dateiaustausch durch. |
| `pub fn recover_open_sagas(&self) -> Vec<EraseSaga>` | `crates/contextra-erasure/src/journal/appendlog.rs:314` | Rekonstruiert den Zustand aller noch nicht abgeschlossenen Sagas aus dem Journal (SPEC §11.2, §11.6). |

### Modul `journal::epoch`

#### `struct Epoch`
- Ort: `crates/contextra-erasure/src/journal/epoch.rs:30`
```rust
pub struct Epoch
```
Dokumentation: Zustand einer Journal-Epoche.
- Impl-Block `crates/contextra-erasure/src/journal/epoch.rs:37`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `index` | `u32` | `crates/contextra-erasure/src/journal/epoch.rs:32` |
| `started_at` | `u64` | `crates/contextra-erasure/src/journal/epoch.rs:34` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn first(started_at: u64) -> Self` | `crates/contextra-erasure/src/journal/epoch.rs:39` | Erstellt die erste Epoche. |
| `pub fn next(&self, now: u64) -> Self` | `crates/contextra-erasure/src/journal/epoch.rs:47` | Rotiert zur nächsten Epoche. |

#### `trait EpochKeys`
- Ort: `crates/contextra-erasure/src/journal/epoch.rs:9`
```rust
pub trait EpochKeys: Send + Sync
```
Dokumentation: Trait zur Bereitstellung von Epochenschlüsseln (SPEC D-125). Rotation vernichtet den alten Epochenschlüssel (Slot E im Tresor). Danach sind alte Commitments nicht mehr verknüpfbar.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn commit_key(&self, epoch_id: u32) -> [u8` | `crates/contextra-erasure/src/journal/epoch.rs:11` | Liefert den 32-Byte-Epochenschlüssel `K_commit(epoch)`. |

#### `fn compute_commitment`
- Ort: `crates/contextra-erasure/src/journal/epoch.rs:16`
```rust
pub fn compute_commitment(commit_key: &[u8
```
Dokumentation: Berechnet das kryptografische Commitment für Journal und Bericht nach SPEC §11.6 und D-125: `BLAKE3-keyed(K_commit(epoch), "ctx/commit/v1" ‖ subject ‖ tenant)`

### Modul `journal::record`

#### `struct JournalEntry`
- Ort: `crates/contextra-erasure/src/journal/record.rs:315`
```rust
pub struct JournalEntry
```
Dokumentation: Ein abwärtskompatibler Journal-Eintrag (SPEC §11.6, ER-29).
- Impl-Block `crates/contextra-erasure/src/journal/record.rs:326`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `erasure_id` | `[u8; 16]` | `crates/contextra-erasure/src/journal/record.rs:317` |
| `state` | `SagaState` | `crates/contextra-erasure/src/journal/record.rs:319` |
| `timestamp` | `u64` | `crates/contextra-erasure/src/journal/record.rs:321` |
| `commit_hash` | `[u8; 32]` | `crates/contextra-erasure/src/journal/record.rs:323` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new_v2( erasure_id: [u8` | `crates/contextra-erasure/src/journal/record.rs:329` | Additiver Konstruktor `new_v2` für erweiterte Initialisierung. |
| `pub fn from_record(rec: &JournalRecord) -> Self` | `crates/contextra-erasure/src/journal/record.rs:358` | Konvertiert einen `JournalRecord` in einen `JournalEntry`. |
| `pub fn to_record(&self) -> JournalRecord` | `crates/contextra-erasure/src/journal/record.rs:372` | Konvertiert `self` in einen `JournalRecord`. |

#### `struct JournalRecord`
- Ort: `crates/contextra-erasure/src/journal/record.rs:124`
```rust
pub struct JournalRecord
```
Dokumentation: Vollständiger Journal-Eintrag nach SPEC Anhang F.9. Klartext: `erasure_id(16) || record_code(u8) || time(u64) || commitment(16) || target_type(u8) || retention_code(u8) || phase(u8) || reason(u8) || scope_tags(Liste lo/hi) || counters || witness_hashes`
- Impl-Block `crates/contextra-erasure/src/journal/record.rs:149`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `erasure_id` | `[u8; 16]` | `crates/contextra-erasure/src/journal/record.rs:126` |
| `record_code` | `RecordCode` | `crates/contextra-erasure/src/journal/record.rs:128` |
| `timestamp` | `u64` | `crates/contextra-erasure/src/journal/record.rs:130` |
| `commitment` | `[u8; 16]` | `crates/contextra-erasure/src/journal/record.rs:132` |
| `target_type` | `TargetType` | `crates/contextra-erasure/src/journal/record.rs:134` |
| `retention_code` | `u8` | `crates/contextra-erasure/src/journal/record.rs:136` |
| `phase` | `u8` | `crates/contextra-erasure/src/journal/record.rs:138` |
| `reason` | `u8` | `crates/contextra-erasure/src/journal/record.rs:140` |
| `scope_tags` | `Vec<u8>` | `crates/contextra-erasure/src/journal/record.rs:142` |
| `counters` | `u64` | `crates/contextra-erasure/src/journal/record.rs:144` |
| `witness_hashes` | `Vec<[u8; 32]>` | `crates/contextra-erasure/src/journal/record.rs:146` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new_v2( erasure_id: [u8` | `crates/contextra-erasure/src/journal/record.rs:152` | Erzeugt einen neuen `JournalRecord`. |
| `pub fn encode(&self) -> Vec<u8>` | `crates/contextra-erasure/src/journal/record.rs:182` | Serialisiert den Klartext nach SPEC F.9 in Bytes: `erasure_id(16) ‖ record_code(u8) ‖ time(u64) ‖ commitment(16) ‖ target_type(u8) ‖ retention_code(u8) ‖ phase( |
| `pub fn decode(bytes: &[u8]) -> Result<Self, ErasureError>` | `crates/contextra-erasure/src/journal/record.rs:209` | Deserialisiert den Klartext aus Bytes nach SPEC F.9. |

#### `enum RecordCode`
- Ort: `crates/contextra-erasure/src/journal/record.rs:11`
```rust
pub enum RecordCode
```
Dokumentation: Record-Codes nach SPEC Anhang F.9.
- Impl-Block `crates/contextra-erasure/src/journal/record.rs:38`

Varianten:

- `RequestedIntent= 1` — `crates/contextra-erasure/src/journal/record.rs:13`
- `Gated= 2` — `crates/contextra-erasure/src/journal/record.rs:15`
- `KeyDestroyed= 3` — `crates/contextra-erasure/src/journal/record.rs:17`
- `Purging= 4` — `crates/contextra-erasure/src/journal/record.rs:19`
- `Verifying= 5` — `crates/contextra-erasure/src/journal/record.rs:21`
- `Completed= 6` — `crates/contextra-erasure/src/journal/record.rs:23`
- `Pending= 7` — `crates/contextra-erasure/src/journal/record.rs:25`
- `Refused= 8` — `crates/contextra-erasure/src/journal/record.rs:27`
- `RestrictedRead= 9` — `crates/contextra-erasure/src/journal/record.rs:29`
- `ObserverAck= 10` — `crates/contextra-erasure/src/journal/record.rs:31`
- `Lifted= 11` — `crates/contextra-erasure/src/journal/record.rs:33`
- `Restricted= 12` — `crates/contextra-erasure/src/journal/record.rs:35`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_u8(val: u8) -> Result<Self, ErasureError>` | `crates/contextra-erasure/src/journal/record.rs:40` | Konvertiert ein Byte in `RecordCode`. |
| `pub fn as_u8(self) -> u8` | `crates/contextra-erasure/src/journal/record.rs:59` | Gibt den numerischen Code zurück. |
| `pub fn to_saga_state(self) -> SagaState` | `crates/contextra-erasure/src/journal/record.rs:64` | Mappt RecordCode auf den entsprechenden SagaState. |

#### `enum TargetType`
- Ort: `crates/contextra-erasure/src/journal/record.rs:84`
```rust
pub enum TargetType
```
Dokumentation: Zieltypen (target_type) nach SPEC Anhang F.9.
- Impl-Block `crates/contextra-erasure/src/journal/record.rs:99`

Varianten:

- `Document= 1` — `crates/contextra-erasure/src/journal/record.rs:86`
- `Subject= 2` — `crates/contextra-erasure/src/journal/record.rs:88`
- `Collection= 3` — `crates/contextra-erasure/src/journal/record.rs:90`
- `Tenant= 4` — `crates/contextra-erasure/src/journal/record.rs:92`
- `Superseded= 5` — `crates/contextra-erasure/src/journal/record.rs:94`
- `DocumentReplace= 6` — `crates/contextra-erasure/src/journal/record.rs:96`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_u8(val: u8) -> Result<Self, ErasureError>` | `crates/contextra-erasure/src/journal/record.rs:101` | Konvertiert ein Byte in `TargetType`. |
| `pub fn as_u8(self) -> u8` | `crates/contextra-erasure/src/journal/record.rs:114` | Gibt den numerischen Code zurück. |

### Modul `journal::reduce`

#### `fn reduce`
- Ort: `crates/contextra-erasure/src/journal/reduce.rs:9`
```rust
pub fn reduce(entries: &[JournalEntry]) -> Vec<JournalEntry>
```
Dokumentation: Reduziert Journal-Einträge: Behält nur den letzten Eintrag je `erasure_id` nach `Completed` (R13).

### Modul `journal::scope_purge`

#### `fn dispose_scope_wal`
- Ort: `crates/contextra-erasure/src/journal/scope_purge.rs:30`
```rust
pub fn dispose_scope_wal( sink: &dyn WalSink, scope: &WalScope, clock: &dyn Clock, ) -> Result<WalDisposalReceipt, WalSinkError>
```
Dokumentation: Führt die WAL-Entsorgung für den angegebenen Scope synchron durch. Gemäß INV-ERA-6 werden WAL-Einträge eines gelöschten Scopes entsorgt oder unlesbar gemacht. - **Wiederaufnehmbarkeit & Idempotenz (INV-ERA-2):** Wurde der Scope bereits entsorgt, wird dies sofort erkannt und quittiert (`WalDisposalMethod::AlreadyDisposed`). - **Priorität:** Zuerst werden alle auffindbaren Einträge physisch getilgt (`dispose_entries`). - **Sicherheitsfall:** Anschließend bzw. zur vollständigen Entwertung wird der scope-spezifische Integritäts-/Kryptoschlüssel über `discard_scope_integrity_key` vernichtet, sodass

### Modul `journal::wal`

#### `struct MemoryWalSink`
- Ort: `crates/contextra-erasure/src/journal/wal.rs:13`
```rust
pub struct MemoryWalSink
```
Dokumentation: In-Memory Referenzimplementierung von [`WalSink`].
- Impl-Block `crates/contextra-erasure/src/journal/wal.rs:18`
- Impl-Block (Trait `WalSink`) `crates/contextra-erasure/src/journal/wal.rs:39`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/journal/wal.rs:20` | Erzeugt eine neue `MemoryWalSink`. |
| `pub fn is_scope_purged(&self, scope: &KeyScope) -> bool` | `crates/contextra-erasure/src/journal/wal.rs:28` | Prüft, ob ein Scope bereits als im WAL bereinigt markiert ist. |
| `pub fn markers_count(&self) -> usize` | `crates/contextra-erasure/src/journal/wal.rs:34` | Gibt alle registrierten Marker zurück. |

### Modul `journal::wal_sink`

#### `struct InMemoryWalEntry`
- Ort: `crates/contextra-erasure/src/journal/wal_sink.rs:173`
```rust
pub struct InMemoryWalEntry
```
Dokumentation: In-Memory-Eintrag für die Test- und Referenz-Implementierung.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `WalEntryId` | `crates/contextra-erasure/src/journal/wal_sink.rs:175` |
| `scope` | `WalScope` | `crates/contextra-erasure/src/journal/wal_sink.rs:177` |
| `payload` | `Vec<u8>` | `crates/contextra-erasure/src/journal/wal_sink.rs:179` |
| `is_purged` | `bool` | `crates/contextra-erasure/src/journal/wal_sink.rs:181` |

#### `struct InMemoryWalSink`
- Ort: `crates/contextra-erasure/src/journal/wal_sink.rs:185`
```rust
pub struct InMemoryWalSink
```
Dokumentation: Referenzimplementierung des `WalSink`-Ports im Speicher für Tests und Verifikation.
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/journal/wal_sink.rs:194`
- Impl-Block `crates/contextra-erasure/src/journal/wal_sink.rs:200`
- Impl-Block (Trait `WalSink`) `crates/contextra-erasure/src/journal/wal_sink.rs:299`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/journal/wal_sink.rs:202` | Erzeugt eine neue leere Instanz. |
| `pub fn add_entry(&self, id: u64, scope: WalScope, payload: Vec<u8>)` | `crates/contextra-erasure/src/journal/wal_sink.rs:214` | Fügt einen Test-WAL-Eintrag hinzu. |
| `pub fn set_integrity_key(&self, scope_id: [u8` | `crates/contextra-erasure/src/journal/wal_sink.rs:229` | Hinterlegt einen Integritätsschlüssel für einen Scope. |
| `pub fn get_integrity_key(&self, scope_id: &[u8` | `crates/contextra-erasure/src/journal/wal_sink.rs:236` | Liest den Integritätsschlüssel eines Scopes, sofern noch vorhanden. |
| `pub fn read_entries_for_scope(&self, scope: &WalScope) -> Vec<Vec<u8>>` | `crates/contextra-erasure/src/journal/wal_sink.rs:247` | Liest lesbare Einträge eines Scopes. Wenn der Schlüssel verworfen wurde oder Einträge getilgt wurden, ist kein Klartext verfügbar. |
| `pub fn set_fail_on_enumerate(&self, fail: bool)` | `crates/contextra-erasure/src/journal/wal_sink.rs:278` | Simuliert einen Fehler bei `enumerate_scope_entries`. |
| `pub fn set_fail_on_dispose(&self, fail: bool)` | `crates/contextra-erasure/src/journal/wal_sink.rs:285` | Simuliert einen Fehler bei `dispose_entries`. |
| `pub fn set_fail_on_key_discard(&self, fail: bool)` | `crates/contextra-erasure/src/journal/wal_sink.rs:292` | Simuliert einen Fehler bei `discard_scope_integrity_key`. |

#### `struct WalDisposalReceipt`
- Ort: `crates/contextra-erasure/src/journal/wal_sink.rs:134`
```rust
pub struct WalDisposalReceipt
```
Dokumentation: Beleg über die durchgeführte Entsorgung von WAL-Einträgen.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `scope` | `WalScope` | `crates/contextra-erasure/src/journal/wal_sink.rs:136` |
| `disposed_entries_count` | `usize` | `crates/contextra-erasure/src/journal/wal_sink.rs:138` |
| `method` | `WalDisposalMethod` | `crates/contextra-erasure/src/journal/wal_sink.rs:140` |
| `timestamp_nanos` | `u64` | `crates/contextra-erasure/src/journal/wal_sink.rs:142` |

#### `struct WalEntryId`
- Ort: `crates/contextra-erasure/src/journal/wal_sink.rs:50`
```rust
pub struct WalEntryId(pub u64)
```
Dokumentation: Identifikator eines einzelnen Eintrags oder Blocks im WAL.
- Impl-Block `crates/contextra-erasure/src/journal/wal_sink.rs:52`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-erasure/src/journal/wal_sink.rs:64`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-erasure/src/journal/wal_sink.rs:54` | Erzeugt eine neue `WalEntryId`. |
| `pub const fn as_u64(self) -> u64` | `crates/contextra-erasure/src/journal/wal_sink.rs:59` | Liefert den numerischen Wert zurück. |

#### `struct WalScope`
- Ort: `crates/contextra-erasure/src/journal/wal_sink.rs:16`
```rust
pub struct WalScope
```
Dokumentation: Eindeutiger Identifikator für einen logischen Bereich (Scope) im WAL.
- Impl-Block `crates/contextra-erasure/src/journal/wal_sink.rs:23`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(scope_id: [u8` | `crates/contextra-erasure/src/journal/wal_sink.rs:25` | Erzeugt einen neuen `WalScope` mit 16-Byte Kennung und leerem Präfix. |
| `pub fn with_prefix(scope_id: [u8` | `crates/contextra-erasure/src/journal/wal_sink.rs:33` | Erzeugt einen neuen `WalScope` mit Kennung und Schlüsselpräfix. |
| `pub fn scope_id(&self) -> &[u8` | `crates/contextra-erasure/src/journal/wal_sink.rs:38` | Gibt die 16-Byte Scope-ID zurück. |
| `pub fn prefix(&self) -> &[u8]` | `crates/contextra-erasure/src/journal/wal_sink.rs:43` | Gibt das optionale Präfix zurück. |

#### `enum WalDisposalMethod`
- Ort: `crates/contextra-erasure/src/journal/wal_sink.rs:72`
```rust
pub enum WalDisposalMethod
```
Dokumentation: Methode, mit der Einträge eines Scopes unlesbar oder getilgt wurden.
- Impl-Block (Trait `fmt::Display`) `crates/contextra-erasure/src/journal/wal_sink.rs:81`

Varianten:

- `PhysicalPurge` — `crates/contextra-erasure/src/journal/wal_sink.rs:74`
- `CryptoShredding` — `crates/contextra-erasure/src/journal/wal_sink.rs:76`
- `AlreadyDisposed` — `crates/contextra-erasure/src/journal/wal_sink.rs:78`

#### `enum WalSinkError`
- Ort: `crates/contextra-erasure/src/journal/wal_sink.rs:93`
```rust
pub enum WalSinkError
```
Dokumentation: Fehlertyp für den WAL-Sink-Port mit Phasenbezug (fail-closed, kein reiner String-Fehler).

Varianten:

- `Io{` — `crates/contextra-erasure/src/journal/wal_sink.rs:96`
- `Corrupt{` — `crates/contextra-erasure/src/journal/wal_sink.rs:104`
- `KeyDiscardFailed{` — `crates/contextra-erasure/src/journal/wal_sink.rs:112`
- `SinkUnavailable{` — `crates/contextra-erasure/src/journal/wal_sink.rs:118`
- `StateConflict{` — `crates/contextra-erasure/src/journal/wal_sink.rs:124`

#### `trait WalSink`
- Ort: `crates/contextra-erasure/src/journal/wal_sink.rs:148`
```rust
pub trait WalSink: Send + Sync
```
Dokumentation: Synchroner Port zur physischen oder kryptographischen Entsorgung von WAL-Einträgen eines Scopes. Gemäß der synchronen Kernregel existieren im Löschpfad keine Threads und keine Async-Aufrufe.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn enumerate_scope_entries(&self, scope: &WalScope) -> Result<Vec<WalEntryId>, WalSinkError>` | `crates/contextra-erasure/src/journal/wal_sink.rs:150` | Enumeriert alle bekannten WAL-Einträge des angegebenen Scopes. |
| `fn dispose_entries( &self, scope: &WalScope, entries: &[WalEntryId], ) -> Result<usize, WalSinkError>` | `crates/contextra-erasure/src/journal/wal_sink.rs:155` | Führt die physische Entsorgung (Truncate/Überschreiben) der angegebenen Einträge durch. Gibt die tatsächliche Anzahl der entsorgten Einträge zurück. |
| `fn discard_scope_integrity_key(&self, scope: &WalScope) -> Result<(), WalSinkError>` | `crates/contextra-erasure/src/journal/wal_sink.rs:165` | Verwirft und nullt den scope-spezifischen Integritäts- oder Verschlüsselungsschlüssel. Nach diesem Aufruf können verbliebene Einträge des Scopes kryptographisch |
| `fn is_scope_disposed(&self, scope: &WalScope) -> Result<bool, WalSinkError>` | `crates/contextra-erasure/src/journal/wal_sink.rs:168` | Prüft, ob der Scope bereits als vollständig entsorgt gilt. |

### Modul `kv_shredding`

#### `struct EncryptedEpochRecordPayload`
- Ort: `crates/contextra-erasure/src/kv_shredding.rs:202`
```rust
pub struct EncryptedEpochRecordPayload
```
Dokumentation: Serialisierbarer Container für einen mit Envelope Encryption und Epochenbindung geschützten Datensatz.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `record_id` | `u64` | `crates/contextra-erasure/src/kv_shredding.rs:204` |
| `group_id` | `u64` | `crates/contextra-erasure/src/kv_shredding.rs:206` |
| `epoch` | `u64` | `crates/contextra-erasure/src/kv_shredding.rs:208` |
| `ciphertext` | `Vec<u8>` | `crates/contextra-erasure/src/kv_shredding.rs:210` |
| `nonce` | `[u8; 12]` | `crates/contextra-erasure/src/kv_shredding.rs:212` |
| `wrapped_dek` | `Vec<u8>` | `crates/contextra-erasure/src/kv_shredding.rs:214` |
| `dek_nonce` | `[u8; 12]` | `crates/contextra-erasure/src/kv_shredding.rs:216` |

#### `struct EncryptedRecordPayload`
- Ort: `crates/contextra-erasure/src/kv_shredding.rs:94`
```rust
pub struct EncryptedRecordPayload
```
Dokumentation: Serialisierbarer Container für einen mit Envelope Encryption geschützten Datensatz.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `record_id` | `u64` | `crates/contextra-erasure/src/kv_shredding.rs:96` |
| `group_id` | `u64` | `crates/contextra-erasure/src/kv_shredding.rs:98` |
| `ciphertext` | `Vec<u8>` | `crates/contextra-erasure/src/kv_shredding.rs:100` |
| `nonce` | `[u8; 12]` | `crates/contextra-erasure/src/kv_shredding.rs:102` |
| `wrapped_dek` | `Vec<u8>` | `crates/contextra-erasure/src/kv_shredding.rs:104` |
| `dek_nonce` | `[u8; 12]` | `crates/contextra-erasure/src/kv_shredding.rs:106` |

#### `struct GroupEpochKey`
- Ort: `crates/contextra-erasure/src/kv_shredding.rs:138`
```rust
pub struct GroupEpochKey
```
Dokumentation: Thread-sichere Schlüssel-Registry für Envelope Encryption und O(1) Krypto-Shredding. Schlüsselbereich für Gruppen und Epochen (Entscheidung E5).
- Impl-Block `crates/contextra-erasure/src/kv_shredding.rs:145`
- Impl-Block `crates/contextra-erasure/src/vault/key_store.rs:30`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `group_id` | `u64` | `crates/contextra-erasure/src/kv_shredding.rs:140` |
| `epoch` | `u64` | `crates/contextra-erasure/src/kv_shredding.rs:142` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(group_id: u64, epoch: u64) -> Self` | `crates/contextra-erasure/src/kv_shredding.rs:147` | Erzeugt einen neuen `GroupEpochKey`. |
| `pub const fn new(group_id: u64, epoch: u64) -> Self` | `crates/contextra-erasure/src/vault/key_store.rs:32` | Erzeugt einen neuen `GroupEpochKey`. |

#### `struct GroupKek`
- Ort: `crates/contextra-erasure/src/kv_shredding.rs:57`
```rust
pub struct GroupKek(pub [u8
```
Dokumentation: Group Key Encryption Key (KEK) - 256 Bit, mit automatischer Nullung beim Verwerfen (`ZeroizeOnDrop`).
- Impl-Block (Trait `Clone`) `crates/contextra-erasure/src/kv_shredding.rs:59`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-erasure/src/kv_shredding.rs:65`

#### `struct KeyPurgeReceipt`
- Ort: `crates/contextra-erasure/src/kv_shredding.rs:155`
```rust
pub struct KeyPurgeReceipt
```
Dokumentation: Vernichtungsergebnis nach `purge_keys` zur Verankerung im Ledger durch den Orchestrator. Herkunft: ffeb79555:crates/contextra-crypto/src/scope_keyring.rs:220 (DestructionReceipt)

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `group_id` | `u64` | `crates/contextra-erasure/src/kv_shredding.rs:157` |
| `epoch` | `u64` | `crates/contextra-erasure/src/kv_shredding.rs:159` |
| `affected_deks` | `usize` | `crates/contextra-erasure/src/kv_shredding.rs:161` |
| `purged_at_nanos` | `u64` | `crates/contextra-erasure/src/kv_shredding.rs:163` |

#### `struct KeyRegistry`
- Ort: `crates/contextra-erasure/src/kv_shredding.rs:220`
```rust
pub struct KeyRegistry
```
Dokumentation: Thread-sichere Schlüssel-Registry für Envelope Encryption und O(1) Krypto-Shredding.
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/kv_shredding.rs:235`
- Impl-Block `crates/contextra-erasure/src/kv_shredding.rs:241`
- Impl-Block (Trait `KeyStore`) `crates/contextra-erasure/src/vault/key_store.rs:145`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/kv_shredding.rs:243` | Erzeugt eine neue `KeyRegistry` mit zufälligem Master-KEK. |
| `pub fn with_revocation_log(mut self, log: Arc<RevocationLog>) -> Self` | `crates/contextra-erasure/src/kv_shredding.rs:263` | Bindet ein `RevocationLog` an und synchronisiert bereits protokollierte Widerrufe. |
| `pub fn get_wrapped_kek(&self, group_id: u64) -> Option<(Vec<u8>, [u8` | `crates/contextra-erasure/src/kv_shredding.rs:285` | Liefert den gewrappten KEK und die Nonce einer Gruppe, falls vorhanden und aktiv. |
| `pub fn get_wrapped_dek(&self, group_id: u64, record_id: u64) -> Option<(Vec<u8>, [u8` | `crates/contextra-erasure/src/kv_shredding.rs:298` | Liefert den gewrappten DEK und die Nonce eines Records, falls vorhanden und aktiv. |
| `pub fn is_group_active(&self, group_id: u64) -> bool` | `crates/contextra-erasure/src/kv_shredding.rs:315` | Prüft ob eine Gruppe aktiv ist. |
| `pub fn is_key_active(&self, group_id: u64) -> bool` | `crates/contextra-erasure/src/kv_shredding.rs:320` | Alias für `is_group_active`. |
| `pub fn is_group_revoked(&self, group_id: u64) -> bool` | `crates/contextra-erasure/src/kv_shredding.rs:325` | Prüft ob eine Gruppe widerrufen ist. |
| `pub fn get_or_derive(&self, group_id: u64) -> Result<SubKey, ErasureError>` | `crates/contextra-erasure/src/kv_shredding.rs:340` | Ermittelt oder erzeugt den Group KEK für `group_id`. |
| `pub fn revoke_group(&self, group_id: u64) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/kv_shredding.rs:392` | Widerruft eine gesamte Shredding-Gruppe in O(1) und nullt das Schlüsselmaterial im RAM. |
| `pub fn revoke_subkey(&self, group_id: u64) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/kv_shredding.rs:438` | Alias für `revoke_group`. |
| `pub fn revoke_record_num(&self, group_id: u64, record_id: u64) -> bool` | `crates/contextra-erasure/src/kv_shredding.rs:443` | Widerruft einen einzelnen Datensatz anhand seiner numerischen ID. |
| `pub fn revoke_record(&self, record_id: &str) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/kv_shredding.rs:466` | Widerruft einen einzelnen Datensatz anhand seines String-Schlüssels. |
| `pub fn is_record_num_active(&self, group_id: u64, record_id: u64) -> bool` | `crates/contextra-erasure/src/kv_shredding.rs:491` | Prüft ob ein numerischer Datensatz aktiv ist. |
| `pub fn is_record_active(&self, record_id: &str) -> bool` | `crates/contextra-erasure/src/kv_shredding.rs:508` | Prüft ob ein Datensatz anhand seines String-Schlüssels aktiv ist. |
| `pub fn encrypt_envelope_record( &self, group_id: u64, record_id: u64, plaintext: &[u8], ) -> Result<EncryptedRecordPayload, ErasureError>` | `crates/contextra-erasure/src/kv_shredding.rs:526` | Verschlüsselt einen Datensatz mittels Envelope Encryption. |
| `pub fn decrypt_envelope_record( &self, payload: &EncryptedRecordPayload, ) -> Result<Vec<u8>, ErasureError>` | `crates/contextra-erasure/src/kv_shredding.rs:581` | Entschlüsselt einen Datensatz mittels Envelope Encryption. |
| `pub fn encrypt_with_group( &self, group_id: u64, plaintext: &[u8], ) -> Result<(Vec<u8>, [u8` | `crates/contextra-erasure/src/kv_shredding.rs:625` | Verschlüsselt direkt unter dem Gruppen-KEK für `group_id`. |
| `pub fn decrypt_with_group( &self, group_id: u64, ciphertext: &[u8], nonce_bytes: &[u8` | `crates/contextra-erasure/src/kv_shredding.rs:649` | Entschlüsselt Chiffretext direkt unter dem Gruppen-KEK für `group_id`. |
| `pub fn encrypt_record( &self, group_id: u64, record_id: &str, plaintext: &[u8], ) -> Result<Vec<u8>, ErasureError>` | `crates/contextra-erasure/src/kv_shredding.rs:675` | Verschlüsselt Daten für einen Record mit AES-256-GCM-SIV. |
| `pub fn decrypt_record( &self, group_id: u64, record_id: &str, dek_bytes: &[u8` | `crates/contextra-erasure/src/kv_shredding.rs:707` | Entschlüsselt Daten für einen Record mit AES-256-GCM-SIV. |
| `pub fn is_group_epoch_revoked(&self, group_id: u64, epoch: u64) -> bool` | `crates/contextra-erasure/src/kv_shredding.rs:734` | Prüft ob eine Gruppe in einer bestimmten Epoche widerrufen ist. |
| `pub fn is_group_epoch_active(&self, group_id: u64, epoch: u64) -> bool` | `crates/contextra-erasure/src/kv_shredding.rs:746` | Prüft ob eine Gruppe in einer bestimmten Epoche aktiv ist. |
| `pub fn get_or_derive_epoch(&self, group_id: u64, epoch: u64) -> Result<SubKey, ErasureError>` | `crates/contextra-erasure/src/kv_shredding.rs:751` | Ermittelt oder erzeugt den Group-KEK für ein spezifisches `(group_id, epoch)`-Paar. |
| `pub fn encrypt_envelope_record_epoch( &self, group_id: u64, epoch: u64, record_id: u64, plaintext: &[u8], ) -> Result<EncryptedEpochRecordPayload, ErasureError>` | `crates/contextra-erasure/src/kv_shredding.rs:804` | Verschlüsselt einen Datensatz unter Envelope Encryption gebunden an Gruppe und Epoche. |
| `pub fn decrypt_envelope_record_epoch( &self, payload: &EncryptedEpochRecordPayload, ) -> Result<Vec<u8>, ErasureError>` | `crates/contextra-erasure/src/kv_shredding.rs:865` | Entschlüsselt einen Datensatz unter Envelope Encryption mit Epochenprüfung. |
| `pub fn purge_keys( &self, group_id: u64, epoch: u64, clock: Option<&dyn crate::types_local::Clock>, ) -> Result<KeyPurgeReceipt, KeyShredError>` | `crates/contextra-erasure/src/kv_shredding.rs:919` | Physische und kryptographische Schlüsselvernichtung (`purge_keys`) für eine Gruppe und Epoche. Herkunft: ffeb79555:crates/contextra-crypto/src/scope_keyring.rs: |

#### `struct RecordDek`
- Ort: `crates/contextra-erasure/src/kv_shredding.rs:75`
```rust
pub struct RecordDek(pub [u8
```
Dokumentation: Record Data Encryption Key (DEK) - 256 Bit, mit automatischer Nullung beim Verwerfen (`ZeroizeOnDrop`).
- Impl-Block (Trait `Clone`) `crates/contextra-erasure/src/kv_shredding.rs:77`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-erasure/src/kv_shredding.rs:83`

#### `struct SubKey`
- Ort: `crates/contextra-erasure/src/kv_shredding.rs:39`
```rust
pub struct SubKey(pub [u8
```
Dokumentation: Sub-Key bzw. KEK-Bytes (256 Bit) mit automatischer Nullung beim Verwerfen (`ZeroizeOnDrop`).
- Impl-Block (Trait `Clone`) `crates/contextra-erasure/src/kv_shredding.rs:41`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-erasure/src/kv_shredding.rs:47`

#### `enum KeyShredError`
- Ort: `crates/contextra-erasure/src/kv_shredding.rs:168`
```rust
pub enum KeyShredError
```
Dokumentation: Lokale Fehlertaxonomie für Schlüsseloperationen und Krypto-Shredding.

Varianten:

- `GroupNotFound{` — `crates/contextra-erasure/src/kv_shredding.rs:171`
- `EpochNotFound{` — `crates/contextra-erasure/src/kv_shredding.rs:177`
- `KekAlreadyDestroyed{` — `crates/contextra-erasure/src/kv_shredding.rs:185`
- `Crypto(String)` — `crates/contextra-erasure/src/kv_shredding.rs:193`
- `InvalidInput(String)` — `crates/contextra-erasure/src/kv_shredding.rs:196`

#### `fn derive_subkey`
- Ort: `crates/contextra-erasure/src/kv_shredding.rs:125`
```rust
pub fn derive_subkey(master_key: &[u8
```
Dokumentation: Leitet einen Sub-Key aus einem Masterschlüssel deterministisch via BLAKE3 KDF ab.

#### `const DEFAULT_SHRED_KEY_GROUP_SIZE`
- Ort: `crates/contextra-erasure/src/kv_shredding.rs:35`
```rust
pub const DEFAULT_SHRED_KEY_GROUP_SIZE: usize = 64
```
Dokumentation: Standardgröße einer Shredding-Schlüsselgruppe (Anzahl Records je KEK).

### Modul `ledger`

#### `mod durable`
- Ort: `crates/contextra-erasure/src/ledger.rs:5`
```rust
pub mod durable
```

#### `mod entry`
- Ort: `crates/contextra-erasure/src/ledger.rs:6`
```rust
pub mod entry
```

#### `mod error`
- Ort: `crates/contextra-erasure/src/ledger.rs:7`
```rust
pub mod error
```

#### `mod persistent`
- Ort: `crates/contextra-erasure/src/ledger.rs:8`
```rust
pub mod persistent
```

#### `mod record`
- Ort: `crates/contextra-erasure/src/ledger.rs:9`
```rust
pub mod record
```

#### `mod sink`
- Ort: `crates/contextra-erasure/src/ledger.rs:10`
```rust
pub mod sink
```

#### `mod storage`
- Ort: `crates/contextra-erasure/src/ledger.rs:11`
```rust
pub mod storage
```

#### `struct FinalizeOutcome`
- Ort: `crates/contextra-erasure/src/ledger.rs:65`
```rust
pub struct FinalizeOutcome
```
Dokumentation: Ergebnis des Abschlusses.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `report` | `ErasureReport` | `crates/contextra-erasure/src/ledger.rs:67` |
| `report_path` | `PathBuf` | `crates/contextra-erasure/src/ledger.rs:69` |
| `completed_seq` | `u64` | `crates/contextra-erasure/src/ledger.rs:71` |

#### `struct FinalizeParams`
- Ort: `crates/contextra-erasure/src/ledger.rs:39`
```rust
pub struct FinalizeParams
```
Dokumentation: Parameter für den Abschluss eines Löschvorgangs (SPEC §11.8).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `erasure_id` | `ErasureId` | `crates/contextra-erasure/src/ledger.rs:41` |
| `gated_seq` | `u64` | `crates/contextra-erasure/src/ledger.rs:43` |
| `completed_seq` | `u64` | `crates/contextra-erasure/src/ledger.rs:45` |
| `gated_at` | `u64` | `crates/contextra-erasure/src/ledger.rs:47` |
| `completed_at` | `u64` | `crates/contextra-erasure/src/ledger.rs:49` |
| `commitment` | `[u8; 32]` | `crates/contextra-erasure/src/ledger.rs:51` |
| `range_tags` | `Vec<u8>` | `crates/contextra-erasure/src/ledger.rs:53` |
| `witnesses` | `Vec<LayerWitness>` | `crates/contextra-erasure/src/ledger.rs:55` |
| `signing_key` | `SigningKey` | `crates/contextra-erasure/src/ledger.rs:58` |
| `report_dir` | `PathBuf` | `crates/contextra-erasure/src/ledger.rs:60` |

#### `struct InMemoryLedger`
- Ort: `crates/contextra-erasure/src/ledger.rs:178`
```rust
pub struct InMemoryLedger
```
Dokumentation: In-Memory-Implementierung von `LedgerSink` (für Ports-Kompatibilität).

#### `fn finalize_erasure`
- Ort: `crates/contextra-erasure/src/ledger.rs:81`
```rust
pub fn finalize_erasure( vfs: &dyn Vfs, ledger_sink: Option<&dyn LedgerSink>, params: &FinalizeParams, ) -> Result<FinalizeOutcome, ErasureError>
```
Dokumentation: Führt die Abschlussreihenfolge nach SPEC §11.8 idempotent durch: 1. Zeugen prüfen / Bericht kanonisch serialisieren und signieren. 2. `erasure/report/<id>.json` schreiben und synchronisieren. 3. Ledger-Eintrag über `LedgerSink` anhängen und synchronisieren. - Fehlt das Ledger-Ziel oder schlägt die Ablage fehl: Abbruch als `ErasureError::Pending(PendingReason::Io)`. `Completed` ohne Ledger-Eintrag ist verboten! 4. Idempotent: Wiederholung führt bei bereits existierenden Einträgen/Dateien zum gleichen konsistenten Ergebnis.

### Modul `ledger::durable`

#### `struct BatchRecordItem`
- Ort: `crates/contextra-erasure/src/ledger/durable.rs:23`
```rust
pub struct BatchRecordItem
```
Dokumentation: Eingabedaten für einen Batch-Append im Ledger.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `record_type` | `RecordType` | `crates/contextra-erasure/src/ledger/durable.rs:25` |
| `commitment` | `[u8; 32]` | `crates/contextra-erasure/src/ledger/durable.rs:27` |
| `range_tags` | `Vec<u8>` | `crates/contextra-erasure/src/ledger/durable.rs:29` |
| `gated_seq` | `u64` | `crates/contextra-erasure/src/ledger/durable.rs:31` |
| `completed_at` | `u64` | `crates/contextra-erasure/src/ledger/durable.rs:33` |

#### `struct ErasureLedger`
- Ort: `crates/contextra-erasure/src/ledger/durable.rs:37`
```rust
pub struct ErasureLedger
```
Dokumentation: Persistentes, hash-verkettetes Ledger (WP-8, CTRALEDG v1).
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/ledger/durable.rs:45`
- Impl-Block `crates/contextra-erasure/src/ledger/durable.rs:55`
- Impl-Block `crates/contextra-erasure/src/ledger/persistent.rs:24`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn open( storage: Box<dyn LedgerStorage>, ledger_key: [u8` | `crates/contextra-erasure/src/ledger/durable.rs:63` | Öffnet oder erstellt ein ErasureLedger über den angegebenen Speicher. Fail-Closed Verhalten: - `create_if_missing == false` und Speicher leer/nicht vorhanden: ` |
| `pub fn append_batch( &mut self, items: &[BatchRecordItem], ) -> Result<LedgerAnchorProof, LedgerError>` | `crates/contextra-erasure/src/ledger/durable.rs:169` | Führt einen atomaren Batch-Append von Einträgen durch (INV-ERA-4). Synchronisiert vor Rückgabe (`sync`) und liefert einen `LedgerAnchorProof`. Bei Fehlern vor d |
| `pub fn verify_chain(&self) -> Result<(), LedgerError>` | `crates/contextra-erasure/src/ledger/durable.rs:232` | Verifiziert die gesamte Hash-Kette aller Records im Ledger. |
| `pub fn last_hash(&self) -> &[u8` | `crates/contextra-erasure/src/ledger/durable.rs:256` | Liefert den aktuellen Ketten-Hash. |
| `pub fn ledger_floor(&self) -> u64` | `crates/contextra-erasure/src/ledger/durable.rs:261` | Liefert den ledger_floor: `max(gated_seq)` (ER-30). |
| `pub fn detect_time_regression(&self, store_current_seq: u64) -> bool` | `crates/contextra-erasure/src/ledger/durable.rs:266` | Prüft auf Zeitregression: `store_current_seq < max(gated_seq)` (ER-30). |
| `pub fn records(&self) -> &[FramedRecord]` | `crates/contextra-erasure/src/ledger/durable.rs:271` | Alle geladenen Records. |
| `pub fn open_or_create( vfs: Arc<dyn Vfs>, path: &Path, ledger_key: &[u8` | `crates/contextra-erasure/src/ledger/persistent.rs:29` | Öffnet oder erstellt ein persistentes `ErasureLedger`. Fail-Closed: Wenn die Datei existiert, aber beschädigt ist (z. B. MAC-Mismatch oder Ketten-Unterbrechung) |
| `pub fn record_key_destruction( &self, commitment: [u8` | `crates/contextra-erasure/src/ledger/persistent.rs:73` | Verankert eine Schlüsselvernichtung dauerhaft vor dem Abschluss der Saga (INV-ERA-4). |
| `pub fn entries(&self) -> Vec<LedgerEntry>` | `crates/contextra-erasure/src/ledger/persistent.rs:111` | Liefert alle Einträge. |
| `pub fn verify_integrity(&self) -> bool` | `crates/contextra-erasure/src/ledger/persistent.rs:116` | Prüft die Kette auf Integrität. |

#### `struct LedgerAnchorProof`
- Ort: `crates/contextra-erasure/src/ledger/durable.rs:12`
```rust
pub struct LedgerAnchorProof
```
Dokumentation: Bestätigung für einen atomaren Batch-Append als Verankerungsnachweis (WP-8, INV-ERA-4).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `last_seq` | `u64` | `crates/contextra-erasure/src/ledger/durable.rs:14` |
| `chain_hash` | `[u8; 32]` | `crates/contextra-erasure/src/ledger/durable.rs:16` |
| `count` | `usize` | `crates/contextra-erasure/src/ledger/durable.rs:18` |

### Modul `ledger::entry`

#### `struct LedgerEntry`
- Ort: `crates/contextra-erasure/src/ledger/entry.rs:10`
```rust
pub struct LedgerEntry
```
Dokumentation: Ein Ledger-Eintrag (SPEC §11.8, ER-30).
- Impl-Block `crates/contextra-erasure/src/ledger/entry.rs:25`
- Impl-Block `crates/contextra-erasure/src/types_local.rs:588`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:653`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `commitment` | `[u8; 32]` | `crates/contextra-erasure/src/ledger/entry.rs:12` |
| `range_tags` | `Vec<u8>` | `crates/contextra-erasure/src/ledger/entry.rs:14` |
| `gated_seq` | `u64` | `crates/contextra-erasure/src/ledger/entry.rs:16` |
| `completed_at` | `u64` | `crates/contextra-erasure/src/ledger/entry.rs:18` |
| `chain_hash` | `[u8; 32]` | `crates/contextra-erasure/src/ledger/entry.rs:20` |
| `ledger_mac` | `[u8; 16]` | `crates/contextra-erasure/src/ledger/entry.rs:22` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( commitment: [u8` | `crates/contextra-erasure/src/ledger/entry.rs:27` | Erstellt einen neuen Ledger-Eintrag mit Hash-Kette. |
| `pub fn verify(&self, prev_hash: &[u8` | `crates/contextra-erasure/src/ledger/entry.rs:48` | Prüft die Integrität dieses Eintrags (MAC-Verifikation). |
| `pub const fn new( commitment: [u8` | `crates/contextra-erasure/src/types_local.rs:590` | Erzeugt einen neuen Ledger-Eintrag. |
| `pub fn compute_chained( commitment: [u8` | `crates/contextra-erasure/src/types_local.rs:609` | Berechnet die verkettete Hash-Kette. |
| `pub fn verify_chain_hash(&self, prev_hash: &[u8` | `crates/contextra-erasure/src/types_local.rs:639` | Verifiziert den Ketten-Hash. |

#### `const GENESIS_HASH`
- Ort: `crates/contextra-erasure/src/ledger/entry.rs:76`
```rust
pub const GENESIS_HASH: [u8
```
Dokumentation: Genesis-Hash für die erste Kette (Nullvektor).

### Modul `ledger::error`

#### `enum LedgerError`
- Ort: `crates/contextra-erasure/src/ledger/error.rs:7`
```rust
pub enum LedgerError
```
Dokumentation: Fehlerzustände des Ledger-Subsystems.
- Impl-Block (Trait `From<std::io::Error>`) `crates/contextra-erasure/src/ledger/error.rs:44`

Varianten:

- `NotFound` — `crates/contextra-erasure/src/ledger/error.rs:10`
- `Corrupt(String)` — `crates/contextra-erasure/src/ledger/error.rs:14`
- `BrokenChain{` — `crates/contextra-erasure/src/ledger/error.rs:18`
- `UnsupportedVersion(u32)` — `crates/contextra-erasure/src/ledger/error.rs:29`
- `Storage(std::io::ErrorKind, String)` — `crates/contextra-erasure/src/ledger/error.rs:33`
- `InvalidRecord(String)` — `crates/contextra-erasure/src/ledger/error.rs:37`
- `EmptyBatch` — `crates/contextra-erasure/src/ledger/error.rs:41`

### Modul `ledger::persistent`

#### `struct ErasureLedger`
- Ort: `crates/contextra-erasure/src/ledger/persistent.rs:15`
```rust
pub struct ErasureLedger
```
Dokumentation: Persistentes, fehlertolerantes und fail-closed Ledger für Erasure-Nachweise.

### Modul `ledger::record`

#### `struct FramedRecord`
- Ort: `crates/contextra-erasure/src/ledger/record.rs:132`
```rust
pub struct FramedRecord
```
Dokumentation: Strukturierter Record im CTRALEDG-Format (Framing).
- Impl-Block `crates/contextra-erasure/src/ledger/record.rs:151`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `record_type` | `RecordType` | `crates/contextra-erasure/src/ledger/record.rs:134` |
| `seq` | `u64` | `crates/contextra-erasure/src/ledger/record.rs:136` |
| `timestamp` | `u64` | `crates/contextra-erasure/src/ledger/record.rs:138` |
| `commitment` | `[u8; 32]` | `crates/contextra-erasure/src/ledger/record.rs:140` |
| `range_tags` | `Vec<u8>` | `crates/contextra-erasure/src/ledger/record.rs:142` |
| `prev_hash` | `[u8; 32]` | `crates/contextra-erasure/src/ledger/record.rs:144` |
| `chain_hash` | `[u8; 32]` | `crates/contextra-erasure/src/ledger/record.rs:146` |
| `ledger_mac` | `[u8; 16]` | `crates/contextra-erasure/src/ledger/record.rs:148` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn serialize(&self) -> Vec<u8>` | `crates/contextra-erasure/src/ledger/record.rs:162` | Serialisiert den Record inklusive CRC32C Prüfsumme. |
| `pub fn parse(buf: &[u8]) -> Result<(Self, usize), LedgerError>` | `crates/contextra-erasure/src/ledger/record.rs:183` | Parst einen Record aus einem Slice. Gibt den geparsten Record und die gelesene Bytezahl zurück. |

Assoziierte Konstanten/Typen:

- `pub const FIXED_HEADER_LEN: usize = 1 + 8 + 8 + 32 + 4 + 32 + 32 + 16` — `crates/contextra-erasure/src/ledger/record.rs:157`
- `pub const MIN_FRAME_LEN: usize = Self::FIXED_HEADER_LEN + 4` — `crates/contextra-erasure/src/ledger/record.rs:159`

#### `struct LedgerFileHeader`
- Ort: `crates/contextra-erasure/src/ledger/record.rs:14`
```rust
pub struct LedgerFileHeader
```
Dokumentation: Dateikopf (Header) eines CTRALEDG v1 Ledgers (16 Bytes).
- Impl-Block `crates/contextra-erasure/src/ledger/record.rs:23`
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/ledger/record.rs:97`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `magic` | `[u8; 8]` | `crates/contextra-erasure/src/ledger/record.rs:16` |
| `version` | `u32` | `crates/contextra-erasure/src/ledger/record.rs:18` |
| `flags` | `u32` | `crates/contextra-erasure/src/ledger/record.rs:20` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/ledger/record.rs:28` | Erstellt einen Standard-Header. |
| `pub fn to_bytes(&self) -> [u8` | `crates/contextra-erasure/src/ledger/record.rs:37` | Serialisiert den Header in 16 Bytes. |
| `pub fn parse(buf: &[u8]) -> Result<Self, LedgerError>` | `crates/contextra-erasure/src/ledger/record.rs:52` | Parst und validiert den Header aus einem Byte-Slice. |

Assoziierte Konstanten/Typen:

- `pub const SIZE: usize = 16` — `crates/contextra-erasure/src/ledger/record.rs:25`

#### `enum RecordType`
- Ort: `crates/contextra-erasure/src/ledger/record.rs:106`
```rust
pub enum RecordType
```
Dokumentation: Typ eines Ledger-Records im CTRALEDG-Format.
- Impl-Block `crates/contextra-erasure/src/ledger/record.rs:115`

Varianten:

- `Intent= 0x01` — `crates/contextra-erasure/src/ledger/record.rs:108`
- `Completed= 0x02` — `crates/contextra-erasure/src/ledger/record.rs:110`
- `KeyDestroyed= 0x03` — `crates/contextra-erasure/src/ledger/record.rs:112`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_u8(v: u8) -> Result<Self, LedgerError>` | `crates/contextra-erasure/src/ledger/record.rs:117` | Parst den Record-Typ aus einem u8. |

#### `const CTRALEDG_MAGIC`
- Ort: `crates/contextra-erasure/src/ledger/record.rs:7`
```rust
pub const CTRALEDG_MAGIC: &[u8
```
Dokumentation: CTRALEDG Magic (8 Bytes): "CTRALEDG"

#### `const CTRALEDG_VERSION`
- Ort: `crates/contextra-erasure/src/ledger/record.rs:10`
```rust
pub const CTRALEDG_VERSION: u32 = 1
```
Dokumentation: Aktuelle Version des Ledger-Formats

### Modul `ledger::sink`

#### `struct InMemoryLedger`
- Ort: `crates/contextra-erasure/src/ledger/sink.rs:18`
```rust
pub struct InMemoryLedger
```
Dokumentation: In-Memory-Ledger-Implementierung (für Tests und WP-4.3 als Basis). Ausgebaut um optionales persistentes Backend (`ErasureLedger`), behält aber alle bestehenden öffentlichen Signaturen exakt bei (Entscheidung E3).
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/ledger/sink.rs:28`
- Impl-Block `crates/contextra-erasure/src/ledger/sink.rs:39`
- Impl-Block `crates/contextra-erasure/src/ledger.rs:182`
- Impl-Block (Trait `LedgerSink`) `crates/contextra-erasure/src/ledger.rs:191`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(ledger_key: [u8` | `crates/contextra-erasure/src/ledger/sink.rs:41` | Erstellt ein leeres Ledger mit dem gegebenen MAC-Schlüssel. |
| `pub fn with_storage( storage: Box<dyn LedgerStorage>, ledger_key: [u8` | `crates/contextra-erasure/src/ledger/sink.rs:52` | Erstellt ein Ledger mit persistentem `LedgerStorage`-Backend (WP-8). |
| `pub fn append( &mut self, commitment: [u8` | `crates/contextra-erasure/src/ledger/sink.rs:82` | Fügt einen Eintrag hinzu (ER-30). |
| `pub fn append_batch( &mut self, items: &[BatchRecordItem], ) -> Result<LedgerAnchorProof, LedgerError>` | `crates/contextra-erasure/src/ledger/sink.rs:118` | Führt einen atomaren Batch-Append mit Verankerungsnachweis durch (WP-8, INV-ERA-4). |
| `pub fn verify_chain(&self) -> bool` | `crates/contextra-erasure/src/ledger/sink.rs:180` | Prüft alle Einträge auf Integrität (Ketten-Verifikation). |
| `pub fn detect_time_regression(&self, store_current_seq: u64) -> bool` | `crates/contextra-erasure/src/ledger/sink.rs:197` | Prüft auf Zeitregression: `store_seq < max(gated_seq)` (ER-30). |
| `pub fn ledger_floor(&self) -> u64` | `crates/contextra-erasure/src/ledger/sink.rs:202` | Gibt den ledger_floor zurück: `max(gated_seq)` (ER-30). |
| `pub fn entries(&self) -> &[LedgerEntry]` | `crates/contextra-erasure/src/ledger/sink.rs:207` | Alle Einträge. |
| `pub fn prune(&mut self, cutoff_us: u64)` | `crates/contextra-erasure/src/ledger/sink.rs:214` | Entfernt Einträge jenseits von `backup_retention` (ER-30). `cutoff_us`: Mikrosekunden UTC, Einträge mit `completed_at < cutoff_us` werden entfernt. |
| `pub fn max_gated_seq(&self) -> u64` | `crates/contextra-erasure/src/ledger/sink.rs:220` | Gibt die höchste gesehene gate_seq zurück. |
| `pub fn last_hash(&self) -> &[u8` | `crates/contextra-erasure/src/ledger/sink.rs:225` | Gibt den aktuellen Ketten-Hash zurück. |
| `pub fn new() -> Self` | `crates/contextra-erasure/src/ledger.rs:184` | Erstellt ein neues leeres In-Memory-Ledger. |

### Modul `ledger::storage`

#### `struct FileLedgerStorage`
- Ort: `crates/contextra-erasure/src/ledger/storage.rs:89`
```rust
pub struct FileLedgerStorage
```
Dokumentation: Dateibasierte Implementierung von `LedgerStorage` im sicheren Dateisystem.
- Impl-Block `crates/contextra-erasure/src/ledger/storage.rs:93`
- Impl-Block (Trait `LedgerStorage`) `crates/contextra-erasure/src/ledger/storage.rs:107`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new<P: AsRef<Path>>(path: P) -> Self` | `crates/contextra-erasure/src/ledger/storage.rs:95` | Erstellt einen neuen dateibasierten Storage am angegebenen Pfad. |
| `pub fn path(&self) -> &Path` | `crates/contextra-erasure/src/ledger/storage.rs:102` | Gibt den Dateipfad zurück. |

#### `struct MemLedgerStorage`
- Ort: `crates/contextra-erasure/src/ledger/storage.rs:38`
```rust
pub struct MemLedgerStorage
```
Dokumentation: In-Memory-Implementierung von `LedgerStorage` für Unit-Tests und In-Memory-Szenarien.
- Impl-Block `crates/contextra-erasure/src/ledger/storage.rs:42`
- Impl-Block (Trait `LedgerStorage`) `crates/contextra-erasure/src/ledger/storage.rs:58`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/ledger/storage.rs:44` | Erstellt einen leeren In-Memory-Speicher. |
| `pub fn with_bytes(bytes: Vec<u8>) -> Self` | `crates/contextra-erasure/src/ledger/storage.rs:51` | Erstellt einen In-Memory-Speicher mit initialen Bytes. |

#### `trait LedgerStorage`
- Ort: `crates/contextra-erasure/src/ledger/storage.rs:11`
```rust
pub trait LedgerStorage: Send + Sync
```
Dokumentation: Abstrakter synchroner Speicher-Port für Append-Only Ledgers.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn exists(&self) -> bool` | `crates/contextra-erasure/src/ledger/storage.rs:13` | Prüft, ob das Ledger bereits existiert. |
| `fn is_empty(&self) -> Result<bool, LedgerError>` | `crates/contextra-erasure/src/ledger/storage.rs:16` | Prüft, ob der Speicher leer ist. |
| `fn read_all(&self) -> Result<Vec<u8>, LedgerError>` | `crates/contextra-erasure/src/ledger/storage.rs:21` | Liest den gesamten Speicherinhalt. |
| `fn append_atomic(&self, data: &[u8]) -> Result<(), LedgerError>` | `crates/contextra-erasure/src/ledger/storage.rs:24` | Hängt Daten atomar an und garantiert, dass bei einem Fehler vor dem Sync keine Teil-Daten verbleiben. |
| `fn sync(&self) -> Result<(), LedgerError>` | `crates/contextra-erasure/src/ledger/storage.rs:27` | Synchronisiert die geschriebenen Daten dauerhaft auf das Speichermedium (fsync). |
| `fn truncate(&self, len: usize) -> Result<(), LedgerError>` | `crates/contextra-erasure/src/ledger/storage.rs:30` | Setzt den Speicherinhalt zurück oder kürzt ihn (z. B. für Rollback oder Prune). |
| `fn len(&self) -> Result<usize, LedgerError>` | `crates/contextra-erasure/src/ledger/storage.rs:33` | Liefert die aktuelle Bytegröße des Speichers. |

### Modul `measure`

#### `mod field`
- Ort: `crates/contextra-erasure/src/measure.rs:8`
```rust
pub mod field
```

#### `mod positive_control`
- Ort: `crates/contextra-erasure/src/measure.rs:9`
```rust
pub mod positive_control
```

#### `mod residue`
- Ort: `crates/contextra-erasure/src/measure.rs:10`
```rust
pub mod residue
```

#### `mod scanner_flow`
- Ort: `crates/contextra-erasure/src/measure.rs:11`
```rust
pub mod scanner_flow
```

#### `mod witness`
- Ort: `crates/contextra-erasure/src/measure.rs:12`
```rust
pub mod witness
```

#### `struct ArmedScanner`
- Ort: `crates/contextra-erasure/src/measure.rs:98`
```rust
pub struct ArmedScanner
```
Dokumentation: Scanner im Zustand `Armed` (scharfgeschaltet für Residuenprüfung).
- **deprecated**
- Impl-Block `crates/contextra-erasure/src/measure.rs:101`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn finish(&self, matches: u64) -> Result<TestWitness, ErasureError>` | `crates/contextra-erasure/src/measure.rs:103` | Schließt den Scan ab. Gibt `TestWitness` NUR zurück, wenn `matches == 0` (ER-25). |

#### `struct LocalControlEvidence`
- Ort: `crates/contextra-erasure/src/measure.rs:48`
```rust
pub struct LocalControlEvidence
```
Dokumentation: Nachweis über erfolgreiche Positivkontrolle für Test-Scanner.
- **deprecated**
- Impl-Block `crates/contextra-erasure/src/measure.rs:54`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn kind(&self) -> EvidenceKind` | `crates/contextra-erasure/src/measure.rs:56` | Gibt die Art des Nachweises zurück. |

#### `struct MeasurementCoordinator`
- Ort: `crates/contextra-erasure/src/measure.rs:132`
```rust
pub struct MeasurementCoordinator
```
Dokumentation: Koordinator für Feldmessungen (MK1, MK2, MK3, ER-26).
- Impl-Block `crates/contextra-erasure/src/measure.rs:136`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/measure.rs:138` | Erstellt einen neuen Koordinator. |

#### `struct TestWitness`
- Ort: `crates/contextra-erasure/src/measure.rs:118`
```rust
pub struct TestWitness
```
Dokumentation: Hilfstyp für Zeugen-Prüfung in Tests.
- **deprecated**
- Impl-Block `crates/contextra-erasure/src/measure.rs:123`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn witness_hash(&self) -> &[u8` | `crates/contextra-erasure/src/measure.rs:125` | Liefert den Zeugen-Hash. |

#### `struct UnarmedScanner`
- Ort: `crates/contextra-erasure/src/measure.rs:64`
```rust
pub struct UnarmedScanner
```
Dokumentation: Scanner im Zustand `Unarmed` (noch nicht scharfgeschaltet).
- **deprecated**
- Impl-Block `crates/contextra-erasure/src/measure.rs:67`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/measure.rs:69` | Erstellt einen neuen unbewaffneten Scanner. |
| `pub fn run_positive_control( &self, pattern: &[u8], ) -> Result<LocalControlEvidence, ErasureError>` | `crates/contextra-erasure/src/measure.rs:74` | Führt die Positivkontrolle auf einem bekannten Muster aus (ER-25). |
| `pub fn arm(self, _evidence: LocalControlEvidence) -> ArmedScanner` | `crates/contextra-erasure/src/measure.rs:90` | Schaltet den Scanner mit dem Positivkontrollnachweis scharf (`arm`, L-5, ER-25). |

#### `enum EvidenceKind`
- Ort: `crates/contextra-erasure/src/measure.rs:39`
```rust
pub enum EvidenceKind
```
Dokumentation: Art des Positivkontroll-Nachweises (ER-25).
- **deprecated**

Varianten:

- `Observed` — `crates/contextra-erasure/src/measure.rs:41`

### Modul `measure::field`

#### `struct Mk1Result`
- Ort: `crates/contextra-erasure/src/measure/field.rs:145`
```rust
pub struct Mk1Result
```
Dokumentation: Ergebnis einer MK1-Feldmessung (logische Prüfung).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `checked_structures` | `u64` | `crates/contextra-erasure/src/measure/field.rs:147` |
| `matches` | `u64` | `crates/contextra-erasure/src/measure/field.rs:149` |
| `verified_via` | `VerifiedVia` | `crates/contextra-erasure/src/measure/field.rs:151` |

#### `struct Mk2Result`
- Ort: `crates/contextra-erasure/src/measure/field.rs:156`
```rust
pub struct Mk2Result
```
Dokumentation: Ergebnis einer MK2-Feldmessung (Rohscan).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `checked_files` | `u64` | `crates/contextra-erasure/src/measure/field.rs:158` |
| `checked_bytes` | `u64` | `crates/contextra-erasure/src/measure/field.rs:160` |
| `matches` | `u64` | `crates/contextra-erasure/src/measure/field.rs:162` |
| `verified_via` | `VerifiedVia` | `crates/contextra-erasure/src/measure/field.rs:164` |

#### `struct Mk3Result`
- Ort: `crates/contextra-erasure/src/measure/field.rs:169`
```rust
pub struct Mk3Result
```
Dokumentation: Ergebnis einer MK3-Feldmessung (Schlüsselabwesenheit).
- Impl-Block `crates/contextra-erasure/src/measure/field.rs:180`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `top_slot_zeroed` | `bool` | `crates/contextra-erasure/src/measure/field.rs:171` |
| `dek_derivation_fails` | `bool` | `crates/contextra-erasure/src/measure/field.rs:173` |
| `file_slots_absent` | `bool` | `crates/contextra-erasure/src/measure/field.rs:175` |
| `verified_via` | `VerifiedVia` | `crates/contextra-erasure/src/measure/field.rs:177` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn is_clean(&self) -> bool` | `crates/contextra-erasure/src/measure/field.rs:182` | Prüft ob MK3 vollständig bestanden (kein Schlüssel mehr ableitbar). |

#### `struct Mk3StatusInput`
- Ort: `crates/contextra-erasure/src/measure/field.rs:81`
```rust
pub struct Mk3StatusInput
```
Dokumentation: Eingabewerte für MK3-Schlüsselprüfung.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `top_slot_zeroed` | `bool` | `crates/contextra-erasure/src/measure/field.rs:83` |
| `dek_derivation_fails` | `bool` | `crates/contextra-erasure/src/measure/field.rs:85` |
| `file_slots_absent` | `bool` | `crates/contextra-erasure/src/measure/field.rs:87` |

#### `struct ScanPlan`
- Ort: `crates/contextra-erasure/src/measure/field.rs:58`
```rust
pub struct ScanPlan
```
Dokumentation: Konfiguration für einen verifizierten Mess-Scan mit Positivkontrolle (SPEC 12.1, 12.2, D-72).
- Impl-Block `crates/contextra-erasure/src/measure/field.rs:90`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `layer` | `LayerId` | `crates/contextra-erasure/src/measure/field.rs:60` |
| `class` | `MeasureClass` | `crates/contextra-erasure/src/measure/field.rs:62` |
| `method_id` | `u16` | `crates/contextra-erasure/src/measure/field.rs:64` |
| `root_dir` | `PathBuf` | `crates/contextra-erasure/src/measure/field.rs:66` |
| `control_pattern` | `Vec<u8>` | `crates/contextra-erasure/src/measure/field.rs:68` |
| `control_file` | `Option<PathBuf>` | `crates/contextra-erasure/src/measure/field.rs:70` |
| `target_patterns` | `Vec<Vec<u8>>` | `crates/contextra-erasure/src/measure/field.rs:72` |
| `mk3_status` | `Option<Mk3StatusInput>` | `crates/contextra-erasure/src/measure/field.rs:74` |
| `decoders` | `Vec<std::sync::Arc<dyn StructureDecoder>>` | `crates/contextra-erasure/src/measure/field.rs:76` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( layer: LayerId, class: MeasureClass, method_id: u16, root_dir: impl Into<PathBuf>, control_pattern: impl Into<Vec<u8>>, ) -> Self` | `crates/contextra-erasure/src/measure/field.rs:92` | Erzeugt einen neuen ScanPlan. |
| `pub fn with_control_file(mut self, path: impl Into<PathBuf>) -> Self` | `crates/contextra-erasure/src/measure/field.rs:113` | Setzt einen expliziten Pfad zu einer Kontrolldatei. |
| `pub fn with_target_pattern(mut self, pattern: impl Into<Vec<u8>>) -> Self` | `crates/contextra-erasure/src/measure/field.rs:119` | Fügt ein Zielmuster hinzu. |
| `pub fn with_target_patterns(mut self, patterns: impl IntoIterator<Item = Vec<u8>>) -> Self` | `crates/contextra-erasure/src/measure/field.rs:125` | Fügt mehrere Zielmuster hinzu. |
| `pub fn with_mk3_status(mut self, status: Mk3StatusInput) -> Self` | `crates/contextra-erasure/src/measure/field.rs:131` | Setzt MK3 Statuswerte. |
| `pub fn with_decoder(mut self, decoder: std::sync::Arc<dyn StructureDecoder>) -> Self` | `crates/contextra-erasure/src/measure/field.rs:137` | Fügt einen Strukturdecoder hinzu. |

#### `enum VerifiedVia`
- Ort: `crates/contextra-erasure/src/measure/field.rs:14`
```rust
pub enum VerifiedVia
```
Dokumentation: Art der Leseverifikation gemäß SPEC D-113.
- Impl-Block `crates/contextra-erasure/src/measure/field.rs:23`

Varianten:

- `Direct` — `crates/contextra-erasure/src/measure/field.rs:16`
- `CacheDropped` — `crates/contextra-erasure/src/measure/field.rs:18`
- `Cached` — `crates/contextra-erasure/src/measure/field.rs:20`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn as_str(&self) -> &'static str` | `crates/contextra-erasure/src/measure/field.rs:25` | Identifikator-String für den Bericht. |

#### `trait StructureDecoder`
- Ort: `crates/contextra-erasure/src/measure/field.rs:38`
```rust
pub trait StructureDecoder: Send + Sync
```
Dokumentation: Strukturdecoder-Schnittstelle zur Prüfung schichtspezifischer Strukturen (SPEC 12.1, AR-3). Ermöglicht die Injektion von Decodern für Manifeste, SSTable-Indizes, Segmentwörterbücher und Slot-Tabellen, ohne dass `contextra-erasure` von `store`, `text` oder `vector` abhängt.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn name(&self) -> &str` | `crates/contextra-erasure/src/measure/field.rs:40` | Name oder Typ des Decoders für Berichte/Traceability. |
| `fn can_decode(&self, path: &Path) -> bool` | `crates/contextra-erasure/src/measure/field.rs:43` | Prüft, ob dieser Decoder für die angegebene Datei zuständig ist. |
| `fn decode_and_check( &self, path: &Path, content: &[u8], target_pattern: &[u8], ) -> Result<u64, ErasureError>` | `crates/contextra-erasure/src/measure/field.rs:48` | Decodiert und prüft den Dateiinhalt auf Einträge des gesuchten Ziels. Gibt die Anzahl der gefundenen Treffer zurück (0 im fehlerfreien Zustand). |

#### `fn collect_all_files`
- Ort: `crates/contextra-erasure/src/measure/field.rs:213`
```rust
pub fn collect_all_files(vfs: &dyn Vfs, dir: &Path) -> Result<Vec<PathBuf>, ErasureError>
```
Dokumentation: Rekursives Sammeln aller Dateipfade über Vfs (ohne Verzeichnisse selbst als Dateien einzufügen).

#### `fn read_file_verified`
- Ort: `crates/contextra-erasure/src/measure/field.rs:202`
```rust
pub fn read_file_verified( vfs: &dyn Vfs, path: &Path, ) -> Result<(Vec<u8>, VerifiedVia), ErasureError>
```
Dokumentation: Liest Bytes einer Datei verifiziert über Vfs (SPEC D-113 Brücke).

#### `fn scan_mk1_structures`
- Ort: `crates/contextra-erasure/src/measure/field.rs:241`
```rust
pub fn scan_mk1_structures( vfs: &dyn Vfs, root: &Path, pattern: &[u8], ) -> Result<Mk1Result, ErasureError>
```
Dokumentation: Scannt MK1-Strukturen (Manifeste, SSTable-Indizes, WAL-Köpfe) auf ein Schlüsselmuster.

#### `fn scan_mk2_raw`
- Ort: `crates/contextra-erasure/src/measure/field.rs:289`
```rust
pub fn scan_mk2_raw(vfs: &dyn Vfs, root: &Path, pattern: &[u8]) -> Result<Mk2Result, ErasureError>
```
Dokumentation: Scannt MK2-Rohdaten aller Dateien (inklusive tmp/, alte Segmente, WAL, scope/, Berichte).

#### `fn scan_mk3_keys`
- Ort: `crates/contextra-erasure/src/measure/field.rs:188`
```rust
pub fn scan_mk3_keys( top_slot_zeroed: bool, dek_derivation_fails: bool, file_slots_absent: bool, ) -> Mk3Result
```
Dokumentation: Führt MK3-Schlüsselprüfung durch (SPEC §12.3, Anhang F.3, D-79, D-113).

### Modul `measure::positive_control`

#### `fn run_positive_control`
- Ort: `crates/contextra-erasure/src/measure/positive_control.rs:18`
```rust
pub fn run_positive_control(pattern: &[u8]) -> Option<ControlEvidence>
```
Dokumentation: Führt eine echte Positivkontrolle im Speicher durch: 1. Schreibt ein Kontrollmuster eingebettet in einen Scratch-Puffer mit Zufalls-/Paddingdaten. 2. Führt einen echten Scan-Durchlauf über den Puffer aus. 3. Erzwingt und prüft die exakte Übereinstimmung des Musters und dessen Position/Trefferanzahl. 4. Verifiziert, dass ein mutiertes oder fehlendes Muster fehlschlägt. Gibt `Some(ControlEvidence::observed())` nur dann zurück, wenn das Muster durch den Suchdurchlauf tatsächlich aufgefunden und kryptografisch bestätigt wurde.

#### `fn run_positive_control_vfs`
- Ort: `crates/contextra-erasure/src/measure/positive_control.rs:71`
```rust
pub fn run_positive_control_vfs( vfs: &dyn Vfs, control_file: Option<&Path>, scratch_dir: &Path, pattern: &[u8], ) -> Option<ControlEvidence>
```
Dokumentation: Führt eine echte Positivkontrolle über das Vfs durch (SPEC 12.1, HR-10, T-RAW-2). Wenn `control_file` angegeben ist, wird diese Datei gelesen und geprüft, ob `pattern` darin enthalten ist. Ein Fehlschlag oder Fehlen der Datei führt ausnahmslos zu `None` (fail-closed). Falls keine Datei angegeben ist, wird temporär eine Kontrolldatei im Scratch-/Zielverzeichnis angelegt, gescannt und nach Nachweis wieder entfernt.

#### `fn verify_scanner_integrity`
- Ort: `crates/contextra-erasure/src/measure/positive_control.rs:148`
```rust
pub fn verify_scanner_integrity(pattern: &[u8], mutator: impl Fn(&[u8]) -> Vec<u8>) -> bool
```
Dokumentation: Kalibrierungs- und Validierungsfunktion für Scanner-Implementierungen. Verifiziert, dass ein Scanner, der blind arbeitet oder manipuliert ist, durchfällt.

### Modul `measure::residue`

#### `struct InMemoryVectorIndexStub`
- Ort: `crates/contextra-erasure/src/measure/residue.rs:448`
```rust
pub struct InMemoryVectorIndexStub
```
Dokumentation: In-Memory Testdouble für Indexstrukturen zur Verifikation der ResidueScanner-Logik.
- Impl-Block `crates/contextra-erasure/src/measure/residue.rs:458`
- Impl-Block (Trait `ResidueScanner`) `crates/contextra-erasure/src/measure/residue.rs:504`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(layer: LayerId) -> Self` | `crates/contextra-erasure/src/measure/residue.rs:460` | Erstellt einen neuen leeren Stub-Index. |
| `pub fn insert_document(&mut self, doc_id: DocId, backlink_targets: Vec<u64>)` | `crates/contextra-erasure/src/measure/residue.rs:473` | Fügt ein Dokument mit Vektorknoten und Backlinks hinzu. |
| `pub fn register_canary(&mut self, canary: Vec<u8>)` | `crates/contextra-erasure/src/measure/residue.rs:481` | Registriert ein Positivkontroll-Muster (Canary). |
| `pub fn physical_erase(&mut self, doc_id: DocId)` | `crates/contextra-erasure/src/measure/residue.rs:486` | Führt physische Bereinigung/Nullung des Dokuments durch. |
| `pub fn set_simulate_fault(&mut self, fault: Option<ScannerFaultKind>)` | `crates/contextra-erasure/src/measure/residue.rs:494` | Konfiguriert die Simulation eines Scanner-Fehlers. |
| `pub fn set_simulate_corrupt(&mut self, corrupt: Option<IndexCorruptionKind>)` | `crates/contextra-erasure/src/measure/residue.rs:499` | Konfiguriert die Simulation von Korruption. |

#### `struct ResidueFinding`
- Ort: `crates/contextra-erasure/src/measure/residue.rs:189`
```rust
pub struct ResidueFinding
```
Dokumentation: Fundstelle einer Restspur im Index. Enthält niemals vertrauliche Rohdaten oder Klartext (Datensparsamkeit).
- Impl-Block `crates/contextra-erasure/src/measure/residue.rs:198`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `layer` | `LayerId` | `crates/contextra-erasure/src/measure/residue.rs:191` |
| `offset_class` | `OffsetClass` | `crates/contextra-erasure/src/measure/residue.rs:193` |
| `count` | `u64` | `crates/contextra-erasure/src/measure/residue.rs:195` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(layer: LayerId, offset_class: OffsetClass, count: u64) -> Self` | `crates/contextra-erasure/src/measure/residue.rs:200` | Erstellt eine neue Fundstellenmeldung. |

#### `struct ResidueMeasurementCoordinator`
- Ort: `crates/contextra-erasure/src/measure/residue.rs:332`
```rust
pub struct ResidueMeasurementCoordinator<'a>
```
Dokumentation: Koordiniert den normativen Messablauf für Index-Schichten. Ablauf: 1. Positivkontrolle: Canary-Prüfung (`canary_probe`). Scheitert dies -> `PositiveControlFailed`. 2. Post-Erase Scan: Restspurensuche nach Löschung (`scan_residue`). 3. Validierung: Bei `total_matches > 0` -> `ResidueDetected`. 4. Ausstellung: Zeugen-Digest wird nur berechnet, wenn die Positivkontrolle erfolgreich war UND keine Restspuren vorhanden sind.
- Impl-Block `crates/contextra-erasure/src/measure/residue.rs:337`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(scanner: &'a dyn ResidueScanner, clock: &'a dyn Clock) -> Self` | `crates/contextra-erasure/src/measure/residue.rs:339` | Erzeugt einen neuen Koordinator mit injiziertem Scanner und Clock. |
| `pub fn verify_positive_control(&self, canary: &[u8]) -> Result<(), ResidueMeasurementError>` | `crates/contextra-erasure/src/measure/residue.rs:344` | Führt die Positivkontrolle auf dem Scanner aus. |
| `pub fn measure_after_erasure( &self, query: &ResidueQuery, canary: &[u8], pre_erase_matches: u64, ) -> Result<ResidueMeasurementReport, ResidueMeasurementError>` | `crates/contextra-erasure/src/measure/residue.rs:379` | Führt die vollständige Verifikationsmessung nach physischer Löschung durch. # Parameter - `query`: Suchkriterium des gelöschten Datensatzes. - `canary`: Kontrol |

#### `struct ResidueMeasurementReport`
- Ort: `crates/contextra-erasure/src/measure/residue.rs:313`
```rust
pub struct ResidueMeasurementReport
```
Dokumentation: Statusbericht eines vollständigen Restspuren-Messablaufs.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `pre_erase_matches` | `u64` | `crates/contextra-erasure/src/measure/residue.rs:315` |
| `post_erase_matches` | `u64` | `crates/contextra-erasure/src/measure/residue.rs:317` |
| `positive_control_passed` | `bool` | `crates/contextra-erasure/src/measure/residue.rs:319` |
| `witness` | `ResidueWitnessDigest` | `crates/contextra-erasure/src/measure/residue.rs:321` |

#### `struct ResidueQuery`
- Ort: `crates/contextra-erasure/src/measure/residue.rs:244`
```rust
pub struct ResidueQuery
```
Dokumentation: Zielanfrage für einen Restspuren-Scan.
- Impl-Block `crates/contextra-erasure/src/measure/residue.rs:251`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_id` | `DocId` | `crates/contextra-erasure/src/measure/residue.rs:246` |
| `target_fingerprint` | `[u8; 32]` | `crates/contextra-erasure/src/measure/residue.rs:248` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(doc_id: DocId, target_fingerprint: [u8` | `crates/contextra-erasure/src/measure/residue.rs:253` | Erstellt eine neue Suchanfrage. |

#### `struct ResidueScanResult`
- Ort: `crates/contextra-erasure/src/measure/residue.rs:211`
```rust
pub struct ResidueScanResult
```
Dokumentation: Gesamtergebnis eines Restspuren-Scans.
- Impl-Block `crates/contextra-erasure/src/measure/residue.rs:220`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `findings` | `Vec<ResidueFinding>` | `crates/contextra-erasure/src/measure/residue.rs:213` |
| `total_matches` | `u64` | `crates/contextra-erasure/src/measure/residue.rs:215` |
| `units_checked` | `u64` | `crates/contextra-erasure/src/measure/residue.rs:217` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn clean(units_checked: u64) -> Self` | `crates/contextra-erasure/src/measure/residue.rs:222` | Erzeugt ein leeres, sauberes Ergebnis (0 Treffer). |
| `pub fn add_finding(&mut self, finding: ResidueFinding)` | `crates/contextra-erasure/src/measure/residue.rs:231` | Fügt einen Fund hinzu. |
| `pub fn is_clean(&self) -> bool` | `crates/contextra-erasure/src/measure/residue.rs:237` | Prüft, ob keine Spuren gefunden wurden. |

#### `struct ResidueWitnessDigest`
- Ort: `crates/contextra-erasure/src/measure/residue.rs:265`
```rust
pub struct ResidueWitnessDigest
```
Dokumentation: Kryptographischer Zeugen-Digest einer erfolgreichen Messung. Kann an `contextra-erasure::report` oder `contextra-crypto` übergeben werden.
- Impl-Block `crates/contextra-erasure/src/measure/residue.rs:276`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `layer` | `LayerId` | `crates/contextra-erasure/src/measure/residue.rs:267` |
| `duration` | `Duration` | `crates/contextra-erasure/src/measure/residue.rs:269` |
| `units_checked` | `u64` | `crates/contextra-erasure/src/measure/residue.rs:271` |
| `digest` | `[u8; 32]` | `crates/contextra-erasure/src/measure/residue.rs:273` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn as_bytes(&self) -> &[u8` | `crates/contextra-erasure/src/measure/residue.rs:278` | Liefert den 32-Byte Digest als Slice. |

#### `enum IndexCorruptionKind`
- Ort: `crates/contextra-erasure/src/measure/residue.rs:129`
```rust
pub enum IndexCorruptionKind
```
Dokumentation: Klassifikation festgestellter Index-Korruptionen.
- Impl-Block (Trait `fmt::Display`) `crates/contextra-erasure/src/measure/residue.rs:140`

Varianten:

- `DanglingBacklink` — `crates/contextra-erasure/src/measure/residue.rs:131`
- `BadChecksum` — `crates/contextra-erasure/src/measure/residue.rs:133`
- `PersistedSnapshotResidue` — `crates/contextra-erasure/src/measure/residue.rs:135`
- `InvalidOverlayState` — `crates/contextra-erasure/src/measure/residue.rs:137`

#### `enum OffsetClass`
- Ort: `crates/contextra-erasure/src/measure/residue.rs:157`
```rust
pub enum OffsetClass
```
Dokumentation: Klassifikation der Fundstelle/Speicherstruktur innerhalb der Indexschicht.
- Impl-Block (Trait `fmt::Display`) `crates/contextra-erasure/src/measure/residue.rs:172`

Varianten:

- `VectorNode` — `crates/contextra-erasure/src/measure/residue.rs:159`
- `BacklinkEntry` — `crates/contextra-erasure/src/measure/residue.rs:161`
- `NeighborList` — `crates/contextra-erasure/src/measure/residue.rs:163`
- `QuantizedData` — `crates/contextra-erasure/src/measure/residue.rs:165`
- `AuxiliaryIndex` — `crates/contextra-erasure/src/measure/residue.rs:167`
- `EntryPoint` — `crates/contextra-erasure/src/measure/residue.rs:169`

#### `enum ResidueMeasurementError`
- Ort: `crates/contextra-erasure/src/measure/residue.rs:53`
```rust
pub enum ResidueMeasurementError
```
Dokumentation: Spezifische Fehlerklasse für Restspuren-Messungen. Folgt Regel 2 (keine Stringfehler, Phasenbezug) und Regel 4 (Fail-Closed).

Varianten:

- `PositiveControlFailed{` — `crates/contextra-erasure/src/measure/residue.rs:56`
- `ResidueDetected{` — `crates/contextra-erasure/src/measure/residue.rs:63`
- `ScannerFault{` — `crates/contextra-erasure/src/measure/residue.rs:72`
- `InvalidCanary{` — `crates/contextra-erasure/src/measure/residue.rs:81`
- `ArithmeticOverflow{` — `crates/contextra-erasure/src/measure/residue.rs:88`
- `CorruptIndexState{` — `crates/contextra-erasure/src/measure/residue.rs:95`

#### `enum ResiduePhase`
- Ort: `crates/contextra-erasure/src/measure/residue.rs:24`
```rust
pub enum ResiduePhase
```
Dokumentation: Spezifische Phase einer Restspuren-Messung.
- Impl-Block (Trait `fmt::Display`) `crates/contextra-erasure/src/measure/residue.rs:37`

Varianten:

- `Init` — `crates/contextra-erasure/src/measure/residue.rs:26`
- `PreEraseScan` — `crates/contextra-erasure/src/measure/residue.rs:28`
- `PositiveControl` — `crates/contextra-erasure/src/measure/residue.rs:30`
- `PostEraseScan` — `crates/contextra-erasure/src/measure/residue.rs:32`
- `WitnessFinalization` — `crates/contextra-erasure/src/measure/residue.rs:34`

#### `enum ScannerFaultKind`
- Ort: `crates/contextra-erasure/src/measure/residue.rs:105`
```rust
pub enum ScannerFaultKind
```
Dokumentation: Art des Fehlers im zugrundeliegenden Index-Scanner.
- Impl-Block (Trait `fmt::Display`) `crates/contextra-erasure/src/measure/residue.rs:116`

Varianten:

- `IoFailure` — `crates/contextra-erasure/src/measure/residue.rs:107`
- `AccessDenied` — `crates/contextra-erasure/src/measure/residue.rs:109`
- `SegmentUnavailable` — `crates/contextra-erasure/src/measure/residue.rs:111`
- `AdapterFailure` — `crates/contextra-erasure/src/measure/residue.rs:113`

#### `trait ResidueScanner`
- Ort: `crates/contextra-erasure/src/measure/residue.rs:291`
```rust
pub trait ResidueScanner: Send + Sync
```
Dokumentation: Synchroner Port für Index-spezifische Restspuren-Scanner (SPEC §12, WP-12). Implementiert durch Adapter in Vektor-, Text- oder Graph-Index-Crates. Arbeitet rein lesend ohne Modifikationen am Index.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn layer(&self) -> LayerId` | `crates/contextra-erasure/src/measure/residue.rs:293` | Liefert die Schichtkennung des Scanners. |
| `fn canary_probe(&self, canary: &[u8]) -> Result<bool, ResidueMeasurementError>` | `crates/contextra-erasure/src/measure/residue.rs:299` | Prüft, ob das injizierte Positivkontroll-Muster auffindbar ist. Gibt `Ok(true)` zurück, wenn das Kontrollmuster nachweisbar vorhanden ist. Gibt `Ok(false)` zurü |
| `fn scan_residue(&self, query: &ResidueQuery) -> Result<ResidueScanResult, ResidueMeasurementError>` | `crates/contextra-erasure/src/measure/residue.rs:304` | Durchsucht den Index nach Spuren der spezifizierten Abfrage. Gibt ein `ResidueScanResult` zurück. Fehler führen fail-closed zum Abbruch. |

### Modul `measure::scanner_flow`

#### `fn run_scanner_flow`
- Ort: `crates/contextra-erasure/src/measure/scanner_flow.rs:187`
```rust
pub fn run_scanner_flow( _layer: LayerId, _class: MeasureClass, _method_id: u16, _control_pattern: &[u8], ) -> Result<LayerWitness, ErasureError>
```
Dokumentation: Führt einen delegierenden Scan-Fluss für Schichten aus (SPEC §12.1, §12.2, D-72, ER-25). Veraltet: Für echte Messungen muss `scan_with_control(vfs, plan)` verwendet werden. Gibt `ErasureError::NotImplemented` zurück, um Scheinerfolge zu verhindern (P1, HR-10).
- **deprecated**

#### `fn scan_with_control`
- Ort: `crates/contextra-erasure/src/measure/scanner_flow.rs:29`
```rust
pub fn scan_with_control(vfs: &dyn Vfs, plan: &ScanPlan) -> Result<LayerWitness, ErasureError>
```
Dokumentation: Führt einen echten Scan über alle Dateien mit vorangehender Positivkontrolle durch. 1. Positivkontrolle über Vfs. 2. Scanner scharfschalten (`arm`). 3. Durchsuchen aller Dateien im Zielverzeichnis (inkl. tmp/, alte Segmente, WAL, Snapshots, scope/, Berichte). 4. Je nach Messklasse: - MK1: Strukturscan über Decoder und Rohbytes von Steuerdateien. - MK2: Rohscan nach Klartext-Needles und Mustern. - MK3: Prüfung der Schlüsselabwesenheit (Status zwingend erforderlich). 5. Bei matches > 0 liefert die Funktion `ErasureError::Residue(matches)`. 6. Bei matches == 0 wird ein `LayerWitness` ausgestellt.

### Modul `measure::witness`

#### `fn arm_scanner`
- Ort: `crates/contextra-erasure/src/measure/witness.rs:11`
```rust
pub fn arm_scanner(scanner: Scanner<Unarmed>, evidence: ControlEvidence) -> Scanner<Armed>
```
Dokumentation: Erstellt einen bewaffneten Scanner nach Positivkontrolle (ER-25). Nur aus `src/measure` aufzurufen (L-5).

#### `fn compute_witness_digest`
- Ort: `crates/contextra-erasure/src/measure/witness.rs:49`
```rust
pub fn compute_witness_digest(witness: &LayerWitness) -> [u8
```
Dokumentation: Berechnet einen kryptografischen BLAKE3-Digest eines `LayerWitness` (SPEC §12.2, INV-ERA-7). Bindet Schicht-ID, Messklasse, Methoden-ID, Dateien, Bytes, Scandauer und den Scan-Hash. // Herkunft: d835c682d:crates/contextra-crypto/src/deletion_proof/issuer.rs:112

#### `fn finish_scan`
- Ort: `crates/contextra-erasure/src/measure/witness.rs:19`
```rust
pub fn finish_scan( scanner: Scanner<Armed>, matches: u64, duration: Duration, scan_hash: [u8
```
Dokumentation: Erstellt einen Zeugen nach erfolgreichem Scan (ER-25). `matches == 0` → Ok(LayerWitness) `matches != 0` → Err(Residue)

#### `fn validate_witness`
- Ort: `crates/contextra-erasure/src/measure/witness.rs:33`
```rust
pub fn validate_witness(witness: &LayerWitness) -> Result<(), crate::error::ErasureError>
```
Dokumentation: Prüft, ob ein `LayerWitness` die strengen Invarianten für die Berichterstellung erfüllt (INV-CLASS-MEASURED-1). Ein Zeuge ist nur dann gültig, wenn: 1. `matches == 0` (keine Restspuren) 2. `witness_hash` nicht nur aus Nullen besteht (Scanlauf tatsächlich ausgeführt).

### Modul `orchestrator`

#### `struct ErasureCoordinator`
- Ort: `crates/contextra-erasure/src/orchestrator.rs:122`
```rust
pub struct ErasureCoordinator
```
Dokumentation: Kanonischer einheitlicher Koordinator der Löschkette (INV-ERA-1 / WP-10). Stellt sicher, dass es genau einen koordinierten Löschpfad im Crate gibt.
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/orchestrator.rs:126`
- Impl-Block `crates/contextra-erasure/src/orchestrator.rs:132`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/orchestrator.rs:134` | Erstellt eine neue Instanz des einheitlichen Koordinators. |
| `pub fn coordinate_erasure( &self, scope: crate::types_local::KeyScope, ) -> Result<crate::scope::ActiveScopeFlight, ErasureError>` | `crates/contextra-erasure/src/orchestrator.rs:141` | Einheitlicher Einstiegspunkt für die Ausführung einer Lösch-Pipeline (INV-ERA-1). |

#### `struct ErasureFailure`
- Ort: `crates/contextra-erasure/src/orchestrator.rs:55`
```rust
pub struct ErasureFailure
```
Dokumentation: Detailed failure reason when erasure verification fails.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `layer` | `String` | `crates/contextra-erasure/src/orchestrator.rs:57` |
| `reason` | `String` | `crates/contextra-erasure/src/orchestrator.rs:59` |

#### `struct ErasureReport`
- Ort: `crates/contextra-erasure/src/orchestrator.rs:90`
```rust
pub struct ErasureReport
```
Dokumentation: Comprehensive erasure verification report.
- Impl-Block `crates/contextra-erasure/src/report/build.rs:313`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `verified` | `bool` | `crates/contextra-erasure/src/orchestrator.rs:92` |
| `proof` | `Option<DeletionProof>` | `crates/contextra-erasure/src/orchestrator.rs:94` |
| `layers` | `Vec<LayerSummary>` | `crates/contextra-erasure/src/orchestrator.rs:96` |
| `failure` | `Option<ErasureFailure>` | `crates/contextra-erasure/src/orchestrator.rs:98` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(schema_version: &str, timestamp_micros: u64) -> Self` | `crates/contextra-erasure/src/report/build.rs:315` | Abwärtskompatibler Konstruktor für bestehende Tests. |
| `pub fn set_field(&mut self, key: &str, val: &str)` | `crates/contextra-erasure/src/report/build.rs:332` | Abwärtskompatible Hilfsmethode. |

#### `struct LayerSummary`
- Ort: `crates/contextra-erasure/src/orchestrator.rs:40`
```rust
pub struct LayerSummary
```
Dokumentation: Summary of a single storage layer's scan result.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `layer` | `String` | `crates/contextra-erasure/src/orchestrator.rs:42` |
| `technique` | `String` | `crates/contextra-erasure/src/orchestrator.rs:44` |
| `scanned` | `u64` | `crates/contextra-erasure/src/orchestrator.rs:46` |
| `remaining` | `u64` | `crates/contextra-erasure/src/orchestrator.rs:48` |
| `control_matches` | `u64` | `crates/contextra-erasure/src/orchestrator.rs:50` |

#### `enum CollectionStatus`
- Ort: `crates/contextra-erasure/src/orchestrator.rs:18`
```rust
pub enum CollectionStatus
```
Dokumentation: Status einer Collection im System.

Varianten:

- `Active` — `crates/contextra-erasure/src/orchestrator.rs:21`
- `PendingPurge` — `crates/contextra-erasure/src/orchestrator.rs:23`

#### `trait ProofKeyProvider`
- Ort: `crates/contextra-erasure/src/orchestrator.rs:27`
```rust
pub trait ProofKeyProvider: Send + Sync
```
Dokumentation: Injected provider interface for deletion proof signing keys.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn resolve_proof_key(&self, tenant_id: TenantId) -> Result<Vec<u8>, ErasureError>` | `crates/contextra-erasure/src/orchestrator.rs:29` | Resolves the deletion proof key for the specified tenant. |

#### `fn compute_collection_id`
- Ort: `crates/contextra-erasure/src/orchestrator.rs:108`
```rust
pub fn compute_collection_id(name: &str) -> CollectionId
```
Dokumentation: Erzeugt eine deterministische, kollisionsfreie [`CollectionId`] aus einem Namen.

#### `fn summary_from_witness`
- Ort: `crates/contextra-erasure/src/orchestrator.rs:74`
```rust
pub fn summary_from_witness(layer_name: String, witness: &ScanWitness) -> LayerSummary
```
Dokumentation: Erstellt ein [`LayerSummary`] aus einem [`ScanWitness`].

#### `const DEFAULT_EXCLUDED_SCOPES`
- Ort: `crates/contextra-erasure/src/orchestrator.rs:102`
```rust
pub const DEFAULT_EXCLUDED_SCOPES: &[ExcludedScope] = &[ ExcludedScope::LlmParameterMemory, ExcludedScope::ConsolidatedAndDistilled, ]
```
Dokumentation: Default excluded scopes for Contextra deletion proofs (§20.4).

#### `const UNMEASURED_ERASURE_LAYERS`
- Ort: `crates/contextra-erasure/src/orchestrator.rs:63`
```rust
pub const UNMEASURED_ERASURE_LAYERS: &[&str] = &[ "", "", "", "", "", "", "", ]
```
Dokumentation: Schichten, die in standardmäßigen Löschläufen derzeit ungemessen bleiben können (SPEC §9.4).

### Modul `purge`

#### `mod cx`
- Ort: `crates/contextra-erasure/src/purge.rs:5`
```rust
pub mod cx
```

#### `mod order`
- Ort: `crates/contextra-erasure/src/purge.rs:6`
```rust
pub mod order
```

#### `mod phase`
- Ort: `crates/contextra-erasure/src/purge.rs:7`
```rust
pub mod phase
```

#### `mod reserve`
- Ort: `crates/contextra-erasure/src/purge.rs:8`
```rust
pub mod reserve
```

#### `mod resume`
- Ort: `crates/contextra-erasure/src/purge.rs:9`
```rust
pub mod resume
```

#### `mod window`
- Ort: `crates/contextra-erasure/src/purge.rs:10`
```rust
pub mod window
```

### Modul `purge::cx`

#### `struct PurgeCx`
- Ort: `crates/contextra-erasure/src/purge/cx.rs:10`
```rust
pub struct PurgeCx
```
Dokumentation: Ausführungskontext eines Purge-Fensters (ER-20, ER-22).
- Impl-Block `crates/contextra-erasure/src/purge/cx.rs:19`
- Impl-Block `crates/contextra-erasure/src/types_local.rs:859`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `deadline_nanos` | `Option<u64>` | `crates/contextra-erasure/src/purge/cx.rs:12` |
| `min_free_bytes` | `u64` | `crates/contextra-erasure/src/purge/cx.rs:14` |
| `batch_window` | `Duration` | `crates/contextra-erasure/src/purge/cx.rs:16` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(min_free_bytes: u64, batch_window: Duration) -> Self` | `crates/contextra-erasure/src/purge/cx.rs:21` | Erstellt einen neuen Purge-Kontext. |
| `pub fn with_deadline_nanos(mut self, deadline_nanos: u64) -> Self` | `crates/contextra-erasure/src/purge/cx.rs:30` | Erstellt einen Kontext mit Deadline (Monotonic Nanos). |
| `pub fn is_expired_against(&self, clock: &dyn Clock) -> bool` | `crates/contextra-erasure/src/purge/cx.rs:36` | Prüft ob die Deadline überschritten ist bezüglich einer gegebenen Uhr. |
| `pub fn is_expired(&self) -> bool` | `crates/contextra-erasure/src/purge/cx.rs:43` | Prüft ob die Deadline mit der Systemuhr überschritten ist. |
| `pub const fn new(deadline_ms: u64) -> Self` | `crates/contextra-erasure/src/types_local.rs:861` | Erzeugt einen neuen Kontext. |

### Modul `purge::order`

#### `struct LayerPurgeOrder`
- Ort: `crates/contextra-erasure/src/purge/order.rs:35`
```rust
pub struct LayerPurgeOrder
```
Dokumentation: Zuordnung einer Schicht zu ihrer Purge-Priorität.
- Impl-Block `crates/contextra-erasure/src/purge/order.rs:42`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `layer_id` | `[u8; 16]` | `crates/contextra-erasure/src/purge/order.rs:37` |
| `priority` | `PurgePriority` | `crates/contextra-erasure/src/purge/order.rs:39` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(layer_id: [u8` | `crates/contextra-erasure/src/purge/order.rs:44` | Erstellt eine neue Zuordnung. |

#### `struct PurgePriority`
- Ort: `crates/contextra-erasure/src/purge/order.rs:14`
```rust
pub struct PurgePriority(pub u8)
```
Dokumentation: Priorität einer Schicht beim Purge (niedrigere Zahl = früher ausgeführt).
- Impl-Block `crates/contextra-erasure/src/purge/order.rs:16`

Assoziierte Konstanten/Typen:

- `pub const SNAPSHOT_REVOKE: Self = Self(0)` — `crates/contextra-erasure/src/purge/order.rs:18`
- `pub const TEXT_REWRITE: Self = Self(1)` — `crates/contextra-erasure/src/purge/order.rs:20`
- `pub const WAL_FLUSH: Self = Self(2)` — `crates/contextra-erasure/src/purge/order.rs:22`
- `pub const SSTABLE_REWRITE: Self = Self(3)` — `crates/contextra-erasure/src/purge/order.rs:24`
- `pub const VECTOR_REPAIR: Self = Self(4)` — `crates/contextra-erasure/src/purge/order.rs:26`
- `pub const SLOT_CLEANUP: Self = Self(5)` — `crates/contextra-erasure/src/purge/order.rs:28`
- `pub const CACHE_EVICT: Self = Self(6)` — `crates/contextra-erasure/src/purge/order.rs:30`

#### `fn sort_by_priority`
- Ort: `crates/contextra-erasure/src/purge/order.rs:50`
```rust
pub fn sort_by_priority(mut layers: Vec<LayerPurgeOrder>) -> Vec<LayerPurgeOrder>
```
Dokumentation: Sortiert Schichten nach Purge-Priorität (niedrigste Zahl zuerst).

### Modul `purge::phase`

#### `enum AdmissionFailure`
- Ort: `crates/contextra-erasure/src/purge/phase.rs:71`
```rust
pub enum AdmissionFailure
```
Dokumentation: Detaillierte Gründe für das Scheitern von `Admitted`.

Varianten:

- `EmptyScope` — `crates/contextra-erasure/src/purge/phase.rs:73`
- `InsufficientSpace{` — `crates/contextra-erasure/src/purge/phase.rs:75`
- `ScopeIoError` — `crates/contextra-erasure/src/purge/phase.rs:82`

#### `enum DisposeFailure`
- Ort: `crates/contextra-erasure/src/purge/phase.rs:121`
```rust
pub enum DisposeFailure
```
Dokumentation: Detaillierte Gründe für das Scheitern von `Disposing`.

Varianten:

- `ScrubFailed` — `crates/contextra-erasure/src/purge/phase.rs:123`
- `UnlinkRefused` — `crates/contextra-erasure/src/purge/phase.rs:125`

#### `enum FenceFailure`
- Ort: `crates/contextra-erasure/src/purge/phase.rs:87`
```rust
pub enum FenceFailure
```
Dokumentation: Detaillierte Gründe für das Scheitern von `Fenced`.

Varianten:

- `ScopeAlreadyLocked` — `crates/contextra-erasure/src/purge/phase.rs:89`
- `CompactionBusy` — `crates/contextra-erasure/src/purge/phase.rs:91`
- `ActiveSnapshotPinned{` — `crates/contextra-erasure/src/purge/phase.rs:93`

#### `enum FinalizeFailure`
- Ort: `crates/contextra-erasure/src/purge/phase.rs:153`
```rust
pub enum FinalizeFailure
```
Dokumentation: Detaillierte Gründe für das Scheitern von `Finalizing`.

Varianten:

- `LedgerAppendFailed` — `crates/contextra-erasure/src/purge/phase.rs:155`
- `LedgerSyncFailed` — `crates/contextra-erasure/src/purge/phase.rs:157`

#### `enum FlushFailure`
- Ort: `crates/contextra-erasure/src/purge/phase.rs:101`
```rust
pub enum FlushFailure
```
Dokumentation: Detaillierte Gründe für das Scheitern von `Flushed`.

Varianten:

- `IoError` — `crates/contextra-erasure/src/purge/phase.rs:103`
- `Timeout` — `crates/contextra-erasure/src/purge/phase.rs:105`

#### `enum PurgePhase`
- Ort: `crates/contextra-erasure/src/purge/phase.rs:174`
```rust
pub enum PurgePhase
```
Dokumentation: Einheitliche Enum für den gesamten Lebenszyklus des physischen Purge (WP-2). Repräsentiert als `u8`-Codes (1..=9) zur sicheren Persistenz und Wiederherstellung.
- Impl-Block `crates/contextra-erasure/src/purge/phase.rs:195`
- Impl-Block (Trait `TryFrom<u8>`) `crates/contextra-erasure/src/purge/phase.rs:276`
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-erasure/src/purge/phase.rs:295`

Varianten:

- `Admitted= 1` — `crates/contextra-erasure/src/purge/phase.rs:176`
- `Fenced= 2` — `crates/contextra-erasure/src/purge/phase.rs:178`
- `Flushed= 3` — `crates/contextra-erasure/src/purge/phase.rs:180`
- `Rewriting= 4` — `crates/contextra-erasure/src/purge/phase.rs:182`
- `Disposing= 5` — `crates/contextra-erasure/src/purge/phase.rs:184`
- `WalRetiring= 6` — `crates/contextra-erasure/src/purge/phase.rs:186`
- `Witnessing= 7` — `crates/contextra-erasure/src/purge/phase.rs:188`
- `Finalizing= 8` — `crates/contextra-erasure/src/purge/phase.rs:190`
- `Released= 9` — `crates/contextra-erasure/src/purge/phase.rs:192`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn code(self) -> u8` | `crates/contextra-erasure/src/purge/phase.rs:199` | Liefert den numerischen `u8`-Code der Phase (1..=9). |
| `pub const fn is_terminal(self) -> bool` | `crates/contextra-erasure/src/purge/phase.rs:206` | Gibt an, ob dieser Zustand der finale Abschlusszustand des Purge-Ablaufes ist. |
| `pub const fn can_transition_to(self, next: PurgePhase) -> bool` | `crates/contextra-erasure/src/purge/phase.rs:218` | Prüft, ob ein Übergang von `self` zu `next` zulässig ist. Zulässige Übergänge: - Genau der nächste lineare Schritt: `Admitted → Fenced → Flushed → Rewriting → D |
| `pub const fn next_phase(self) -> Option<PurgePhase>` | `crates/contextra-erasure/src/purge/phase.rs:235` | Ermittelt die deterministische Folgephase im regulären Lebenszyklus. Liefert `None`, falls sich die Phase bereits im Endzustand `Released` befindet. |
| `pub fn must_run(self, target: PurgePhase) -> bool` | `crates/contextra-erasure/src/purge/phase.rs:252` | Prüft, ob die angegebene Zielphase `target` noch ausgeführt werden muss (`self <= target`). |
| `pub fn transition_to(&mut self, next: PurgePhase) -> Result<(), PurgePhaseError>` | `crates/contextra-erasure/src/purge/phase.rs:259` | Führt einen überprüften Phasenübergang durch. Schlägt mit [`PurgePhaseError::InvalidTransition`] fehl, wenn `can_transition_to` `false` ist. |
| `pub fn from_code(code: u8) -> Result<Self, PurgePhaseError>` | `crates/contextra-erasure/src/purge/phase.rs:271` | Lädt eine Phase aus ihrem persistierten `u8`-Code (1..=9). |

#### `enum PurgePhaseError`
- Ort: `crates/contextra-erasure/src/purge/phase.rs:18`
```rust
pub enum PurgePhaseError
```
Dokumentation: Fehlerzustände während der einzelnen Purge-Phasen (WP-1, WP-2). Jeder Fehler besitzt einen klaren Phasenbezug; Freitext-Stringfehler sind verboten.

Varianten:

- `InvalidTransition{` — `crates/contextra-erasure/src/purge/phase.rs:21`
- `InvalidCode(u8)` — `crates/contextra-erasure/src/purge/phase.rs:30`
- `AdmissionFailed(AdmissionFailure)` — `crates/contextra-erasure/src/purge/phase.rs:34`
- `FenceFailed(FenceFailure)` — `crates/contextra-erasure/src/purge/phase.rs:38`
- `FlushFailed(FlushFailure)` — `crates/contextra-erasure/src/purge/phase.rs:42`
- `RewriteFailed(RewriteFailure)` — `crates/contextra-erasure/src/purge/phase.rs:46`
- `DisposeFailed(DisposeFailure)` — `crates/contextra-erasure/src/purge/phase.rs:50`
- `WalRetireFailed(WalRetireFailure)` — `crates/contextra-erasure/src/purge/phase.rs:54`
- `WitnessFailed(WitnessFailure)` — `crates/contextra-erasure/src/purge/phase.rs:58`
- `FinalizeFailed(FinalizeFailure)` — `crates/contextra-erasure/src/purge/phase.rs:62`
- `ReleaseFailed(ReleaseFailure)` — `crates/contextra-erasure/src/purge/phase.rs:66`

#### `enum ReleaseFailure`
- Ort: `crates/contextra-erasure/src/purge/phase.rs:162`
```rust
pub enum ReleaseFailure
```
Dokumentation: Detaillierte Gründe für das Scheitern von `Released`.

Varianten:

- `FenceReleaseFailed` — `crates/contextra-erasure/src/purge/phase.rs:164`
- `IntentCleanupFailed` — `crates/contextra-erasure/src/purge/phase.rs:166`

#### `enum RewriteFailure`
- Ort: `crates/contextra-erasure/src/purge/phase.rs:110`
```rust
pub enum RewriteFailure
```
Dokumentation: Detaillierte Gründe für das Scheitern von `Rewriting`.

Varianten:

- `DiskReserveExceeded` — `crates/contextra-erasure/src/purge/phase.rs:112`
- `SegmentIoError` — `crates/contextra-erasure/src/purge/phase.rs:114`
- `CorruptSegment` — `crates/contextra-erasure/src/purge/phase.rs:116`

#### `enum WalRetireFailure`
- Ort: `crates/contextra-erasure/src/purge/phase.rs:130`
```rust
pub enum WalRetireFailure
```
Dokumentation: Detaillierte Gründe für das Scheitern von `WalRetiring`.

Varianten:

- `WalSyncError` — `crates/contextra-erasure/src/purge/phase.rs:132`
- `WalCorrupt` — `crates/contextra-erasure/src/purge/phase.rs:134`

#### `enum WitnessFailure`
- Ort: `crates/contextra-erasure/src/purge/phase.rs:139`
```rust
pub enum WitnessFailure
```
Dokumentation: Detaillierte Gründe für das Scheitern von `Witnessing`.

Varianten:

- `PositiveControlFailed` — `crates/contextra-erasure/src/purge/phase.rs:141`
- `ResidueDetected{` — `crates/contextra-erasure/src/purge/phase.rs:143`
- `WitnessGenerationFailed` — `crates/contextra-erasure/src/purge/phase.rs:148`

### Modul `purge::reserve`

#### `enum ReserveCheck`
- Ort: `crates/contextra-erasure/src/purge/reserve.rs:10`
```rust
pub enum ReserveCheck
```
Dokumentation: Ergebnis der Speicherplatzprüfung.

Varianten:

- `Sufficient{` — `crates/contextra-erasure/src/purge/reserve.rs:12`
- `Insufficient{` — `crates/contextra-erasure/src/purge/reserve.rs:17`

#### `fn check_disk_reserve`
- Ort: `crates/contextra-erasure/src/purge/reserve.rs:26`
```rust
pub fn check_disk_reserve( available_bytes: u64, required_bytes: u64, ) -> Result<ReserveCheck, ErasureError>
```
Dokumentation: Prüft ob genug Speicherplatz für den Purge-Rewrite vorhanden ist.

### Modul `purge::resume`

#### `struct PurgeProgressTracker`
- Ort: `crates/contextra-erasure/src/purge/resume.rs:13`
```rust
pub struct PurgeProgressTracker
```
Dokumentation: Persistierbarer Fortschritts-Zustand eines Purge-Laufes für Wiederanlauf (P9).
- Impl-Block `crates/contextra-erasure/src/purge/resume.rs:24`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `purge_id` | `[u8; 16]` | `crates/contextra-erasure/src/purge/resume.rs:15` |
| `current_phase` | `PurgePhase` | `crates/contextra-erasure/src/purge/resume.rs:17` |
| `attempts` | `u32` | `crates/contextra-erasure/src/purge/resume.rs:19` |
| `last_phase_error` | `Option<PurgePhaseError>` | `crates/contextra-erasure/src/purge/resume.rs:21` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(purge_id: [u8` | `crates/contextra-erasure/src/purge/resume.rs:26` | Initialisiert einen neuen Fortschrittstracker im Zustand `Admitted`. |
| `pub fn recover( purge_id: [u8` | `crates/contextra-erasure/src/purge/resume.rs:38` | Stellt einen Tracker aus persistierten Rohdaten wieder her (P9 Resume). Validiert den Phasencode fail-closed. |
| `pub fn current_phase(&self) -> PurgePhase` | `crates/contextra-erasure/src/purge/resume.rs:53` | Gibt die aktuell erreichte Phase zurück. |
| `pub fn next_phase(&self) -> Option<PurgePhase>` | `crates/contextra-erasure/src/purge/resume.rs:60` | Ermittelt die als nächstes auszuführende Phase. Liefert `None`, falls der Purge bereits vollständig im Zustand `Released` abgeschlossen ist. |
| `pub fn is_completed(&self) -> bool` | `crates/contextra-erasure/src/purge/resume.rs:65` | Prüft, ob der Purge bereits terminal beendet ist. |
| `pub fn advance_to_next(&mut self) -> Result<PurgePhase, PurgePhaseError>` | `crates/contextra-erasure/src/purge/resume.rs:72` | Schließt die aktuelle Phase erfolgreich ab und wechselt auf die Folgephase. Schlägt fail-closed fehl, wenn keine Folgephase existiert oder der Übergang unzuläss |
| `pub fn record_failure(&mut self, err: PurgePhaseError)` | `crates/contextra-erasure/src/purge/resume.rs:84` | Zeichnet einen Phasenfehler auf. |

### Modul `purge::window`

#### `struct WindowOutcome`
- Ort: `crates/contextra-erasure/src/purge/window.rs:21`
```rust
pub struct WindowOutcome
```
Dokumentation: Ergebnis eines Purge-Fensters.
- Impl-Block `crates/contextra-erasure/src/purge/window.rs:32`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `purged_units` | `u64` | `crates/contextra-erasure/src/purge/window.rs:23` |
| `bytes_freed` | `u64` | `crates/contextra-erasure/src/purge/window.rs:25` |
| `pending_count` | `u64` | `crates/contextra-erasure/src/purge/window.rs:27` |
| `phase` | `PurgePhase` | `crates/contextra-erasure/src/purge/window.rs:29` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn empty() -> Self` | `crates/contextra-erasure/src/purge/window.rs:34` | Erstellt ein leeres Ergebnis im Initialzustand `Admitted`. |
| `pub fn merge(&mut self, other: &WindowOutcome)` | `crates/contextra-erasure/src/purge/window.rs:44` | Addiert ein weiteres Ergebnis. |

#### `fn run_core_window`
- Ort: `crates/contextra-erasure/src/purge/window.rs:73`
```rust
pub async fn run_core_window(core: &ErasureCore) -> Result<WindowOutcome, ErasureError>
```
Dokumentation: Führt ein vollständiges Purge-Fenster über den `ErasureCore` aus (SPEC §11.2, §11.4).
- Gate: `#[cfg(feature = "async")]`

#### `fn run_window`
- Ort: `crates/contextra-erasure/src/purge/window.rs:60`
```rust
pub fn run_window(cx: &PurgeCx) -> Result<WindowOutcome, ErasureError>
```
Dokumentation: Führt ein Purge-Fenster durch (ER-20). Quellkompatible Signatur für bestehende Aufrufer: prüft Plattenreserve. Schlägt fehl mit `DiskReserve`, falls kein ausreichender Platz vorhanden ist, oder liefert `NotImplemented`, falls kein `ErasureCore` konfiguriert ist, um Scheinerfolge zu verhindern (P1, HR-10).

### Modul `replace`

#### `struct ReplacePlan`
- Ort: `crates/contextra-erasure/src/replace.rs:35`
```rust
pub struct ReplacePlan
```
Dokumentation: Plan und Zustandsverwaltung für die Ersetzung eines Mehr-Betroffenen-Dokuments (SPEC D-115).
- Impl-Block `crates/contextra-erasure/src/replace.rs:42`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( erasure_id: ErasureId, old_doc_tag: [u8` | `crates/contextra-erasure/src/replace.rs:47` | Erstellt einen neuen Ersetzungsplan. # Fehler Liefert `ErasureError::SameIdentity`, wenn altes und neues Dokument denselben Tag tragen. |
| `pub fn erasure_id(&self) -> ErasureId` | `crates/contextra-erasure/src/replace.rs:65` | Liefert die zugehörige Lösch-ID. |
| `pub fn old_doc_tag(&self) -> &[u8` | `crates/contextra-erasure/src/replace.rs:71` | Liefert den Tag des alten Dokuments. |
| `pub fn replacement_tag(&self) -> &[u8` | `crates/contextra-erasure/src/replace.rs:77` | Liefert den Ersatz-Tag (`replacement_tag`, SPEC D-115) für den Löschbericht. |
| `pub fn step(&self) -> ReplaceStep` | `crates/contextra-erasure/src/replace.rs:83` | Liefert den aktuellen Protokollschritt (R1 bis R4). |
| `pub fn advance_to_r2_intent(&mut self) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/replace.rs:88` | Schaltet idempotent auf R2 (PreGate Intent) vor (D-115 R2). |
| `pub fn verify_replacement_present(&self, is_present: bool) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/replace.rs:104` | Prüft vor dem Gate-Schritt (R3), ob das Ersatzdokument dauerhaft vorhanden ist (I-25). Liefert `ErasureError::ReplacementMissing` (FM-19), falls das Ersatzdokum |
| `pub fn advance_to_r3_gate_old(&mut self) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/replace.rs:112` | Schaltet nach Bestätigung des Neudokuments auf R3 (Gate altes Dokument) vor (D-115 R3). |
| `pub fn advance_to_r4_key_destroyed(&mut self) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/replace.rs:126` | Schaltet auf R4 (D-Slot des Altdokuments vernichtet) vor (D-115 R4). |

#### `enum ReplaceStep`
- Ort: `crates/contextra-erasure/src/replace.rs:22`
```rust
pub enum ReplaceStep
```
Dokumentation: Einzelschritte des Ersetzungsprotokolls (SPEC D-115).

Varianten:

- `R1Requested` — `crates/contextra-erasure/src/replace.rs:24`
- `R2PreGate` — `crates/contextra-erasure/src/replace.rs:26`
- `R3GatedOld` — `crates/contextra-erasure/src/replace.rs:28`
- `R4KeyDestroyed` — `crates/contextra-erasure/src/replace.rs:30`

### Modul `report`

#### `mod binding`
- Ort: `crates/contextra-erasure/src/report.rs:5`
```rust
pub mod binding
```

#### `mod build`
- Ort: `crates/contextra-erasure/src/report.rs:6`
```rust
pub mod build
```

#### `mod canonical`
- Ort: `crates/contextra-erasure/src/report.rs:7`
```rust
pub mod canonical
```

#### `mod risk`
- Ort: `crates/contextra-erasure/src/report.rs:8`
```rust
pub mod risk
```

#### `mod signing`
- Ort: `crates/contextra-erasure/src/report.rs:10`
```rust
pub mod signing
```
- Gate: `#[cfg(feature = "signing")]`

### Modul `report::binding`

#### `fn compute_witnesses_chain_hash`
- Ort: `crates/contextra-erasure/src/report/binding.rs:9`
```rust
pub fn compute_witnesses_chain_hash(witnesses: &[LayerWitness]) -> [u8
```
Dokumentation: Berechnet den verketteten Hash über eine Liste von `LayerWitness`-Objekten.

#### `fn verify_witnesses_binding`
- Ort: `crates/contextra-erasure/src/report/binding.rs:22`
```rust
pub fn verify_witnesses_binding(witnesses: &[LayerWitness], expected_hash: &[u8
```
Dokumentation: Verifiziert, ob die gegebenen Zeugen mit dem gebundenen Hash übereinstimmen.

### Modul `report::build`

#### `struct ErasureReport`
- Ort: `crates/contextra-erasure/src/report/build.rs:231`
```rust
pub struct ErasureReport
```
Dokumentation: Kanonischer Löschbericht (ER-27, SPEC §11.7, Anhang D). Keine Inhalte, Schlüssel oder Klartext-Kennungen (P13).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `report_version` | `u32` | `crates/contextra-erasure/src/report/build.rs:233` |
| `store_id` | `String` | `crates/contextra-erasure/src/report/build.rs:235` |
| `erasure_id` | `String` | `crates/contextra-erasure/src/report/build.rs:237` |
| `target_type` | `String` | `crates/contextra-erasure/src/report/build.rs:239` |
| `commitment` | `String` | `crates/contextra-erasure/src/report/build.rs:241` |
| `requested_at` | `u64` | `crates/contextra-erasure/src/report/build.rs:243` |
| `gated_at` | `u64` | `crates/contextra-erasure/src/report/build.rs:245` |
| `key_destroyed_at` | `u64` | `crates/contextra-erasure/src/report/build.rs:247` |
| `completed_at` | `u64` | `crates/contextra-erasure/src/report/build.rs:249` |
| `sla_us` | `u64` | `crates/contextra-erasure/src/report/build.rs:251` |
| `sla_violation` | `bool` | `crates/contextra-erasure/src/report/build.rs:253` |
| `retention_code` | `String` | `crates/contextra-erasure/src/report/build.rs:255` |
| `field_measurement` | `FieldMeasurementSection` | `crates/contextra-erasure/src/report/build.rs:257` |
| `lab_evidence` | `String` | `crates/contextra-erasure/src/report/build.rs:259` |
| `segments_rewritten` | `u64` | `crates/contextra-erasure/src/report/build.rs:265` |
| `bytes_rewritten` | `u64` | `crates/contextra-erasure/src/report/build.rs:267` |
| `vault_medium` | `String` | `crates/contextra-erasure/src/report/build.rs:269` |
| `vault_rehash_count` | `u64` | `crates/contextra-erasure/src/report/build.rs:271` |
| `vault_old_overwritten` | `bool` | `crates/contextra-erasure/src/report/build.rs:273` |
| `vault_backup_scope` | `String` | `crates/contextra-erasure/src/report/build.rs:275` |
| `verified_via` | `String` | `crates/contextra-erasure/src/report/build.rs:277` |
| `epoch_id` | `String` | `crates/contextra-erasure/src/report/build.rs:279` |
| `replaced` | `bool` | `crates/contextra-erasure/src/report/build.rs:281` |
| `replacement_tag` | `Option<String>` | `crates/contextra-erasure/src/report/build.rs:284` |
| `signer_id` | `u8` | `crates/contextra-erasure/src/report/build.rs:286` |
| `process_hardening` | `String` | `crates/contextra-erasure/src/report/build.rs:288` |
| `residual_risks` | `Vec<ResidualRiskEntry>` | `crates/contextra-erasure/src/report/build.rs:290` |
| `witnesses` | `Vec<LayerWitnessReport>` | `crates/contextra-erasure/src/report/build.rs:292` |
| `witness_chain` | `WitnessChainReport` | `crates/contextra-erasure/src/report/build.rs:295` |
| `ledger_confirmation` | `Option<LedgerConfirmation>` | `crates/contextra-erasure/src/report/build.rs:298` |
| `signature_scope_notice` | `String` | `crates/contextra-erasure/src/report/build.rs:300` |
| `legal_notice` | `String` | `crates/contextra-erasure/src/report/build.rs:302` |
| `scope_notice` | `String` | `crates/contextra-erasure/src/report/build.rs:304` |
| `superseded_purge_notice` | `Option<String>` | `crates/contextra-erasure/src/report/build.rs:307` |

#### `struct FieldMeasurementSection`
- Ort: `crates/contextra-erasure/src/report/build.rs:112`
```rust
pub struct FieldMeasurementSection
```
Dokumentation: Feldmessungs-Abschnitt (MK1, MK2, MK3) für Abwärtskompatibilität.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `mk1` | `ReachStatus` | `crates/contextra-erasure/src/report/build.rs:114` |
| `mk2` | `ReachStatus` | `crates/contextra-erasure/src/report/build.rs:116` |
| `mk3` | `ReachStatus` | `crates/contextra-erasure/src/report/build.rs:118` |

#### `struct LayerKeyCoverage`
- Ort: `crates/contextra-erasure/src/report/build.rs:202`
```rust
pub struct LayerKeyCoverage
```
Dokumentation: Schlüsselabdeckung einer Schicht im Bericht.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `layer_id_hex` | `String` | `crates/contextra-erasure/src/report/build.rs:204` |
| `coverage` | `String` | `crates/contextra-erasure/src/report/build.rs:206` |

#### `struct LayerWitnessReport`
- Ort: `crates/contextra-erasure/src/report/build.rs:52`
```rust
pub struct LayerWitnessReport
```
Dokumentation: Strukturierter Zeugeneintrag für eine Schicht (SPEC §11.7, §12.2).
- Impl-Block `crates/contextra-erasure/src/report/build.rs:73`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `layer` | `String` | `crates/contextra-erasure/src/report/build.rs:54` |
| `class` | `String` | `crates/contextra-erasure/src/report/build.rs:56` |
| `method_id` | `String` | `crates/contextra-erasure/src/report/build.rs:58` |
| `scanned_files` | `u64` | `crates/contextra-erasure/src/report/build.rs:60` |
| `scanned_bytes` | `u64` | `crates/contextra-erasure/src/report/build.rs:62` |
| `matches` | `u64` | `crates/contextra-erasure/src/report/build.rs:64` |
| `control_ok` | `bool` | `crates/contextra-erasure/src/report/build.rs:66` |
| `duration_us` | `u64` | `crates/contextra-erasure/src/report/build.rs:68` |
| `hash` | `String` | `crates/contextra-erasure/src/report/build.rs:70` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_witness(w: &LayerWitness) -> Self` | `crates/contextra-erasure/src/report/build.rs:75` | Erstellt einen Report-Eintrag aus einem `LayerWitness`. |

#### `struct LedgerConfirmation`
- Ort: `crates/contextra-erasure/src/report/build.rs:183`
```rust
pub struct LedgerConfirmation
```
Dokumentation: Ledger-Verankerungsbestätigung für den Löschbericht (SPEC §11.8, ER-30). Opaker lokaler Typ, der die Bestätigung der Ledger-Verankerung festhält, ohne direkt vom Ledger-Modul abzuhängen.
- Impl-Block `crates/contextra-erasure/src/report/build.rs:190`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `seq` | `u64` | `crates/contextra-erasure/src/report/build.rs:185` |
| `chain_hash` | `String` | `crates/contextra-erasure/src/report/build.rs:187` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(seq: u64, chain_hash: [u8` | `crates/contextra-erasure/src/report/build.rs:192` | Erstellt eine neue Ledger-Verankerungsbestätigung. |

#### `struct ProcessHardeningStatus`
- Ort: `crates/contextra-erasure/src/report/build.rs:211`
```rust
pub struct ProcessHardeningStatus
```
Dokumentation: Prozesshärtungsstatus (SPEC §11.7, Pflichtfeld).
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/report/build.rs:218`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `encryption_mandatory` | `bool` | `crates/contextra-erasure/src/report/build.rs:213` |
| `wal_integrity_mandatory` | `bool` | `crates/contextra-erasure/src/report/build.rs:215` |

#### `struct ReportBuildOptions`
- Ort: `crates/contextra-erasure/src/report/build.rs:339`
```rust
pub struct ReportBuildOptions
```
Dokumentation: Optionen für die Berichterstellung.
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/report/build.rs:388`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `store_id` | `String` | `crates/contextra-erasure/src/report/build.rs:341` |
| `target_type` | `String` | `crates/contextra-erasure/src/report/build.rs:343` |
| `commitment` | `String` | `crates/contextra-erasure/src/report/build.rs:345` |
| `requested_at` | `u64` | `crates/contextra-erasure/src/report/build.rs:347` |
| `gated_at` | `u64` | `crates/contextra-erasure/src/report/build.rs:349` |
| `key_destroyed_at` | `u64` | `crates/contextra-erasure/src/report/build.rs:351` |
| `completed_at` | `u64` | `crates/contextra-erasure/src/report/build.rs:353` |
| `sla_us` | `u64` | `crates/contextra-erasure/src/report/build.rs:355` |
| `sla_violation` | `bool` | `crates/contextra-erasure/src/report/build.rs:357` |
| `retention_code` | `String` | `crates/contextra-erasure/src/report/build.rs:359` |
| `lab_evidence` | `String` | `crates/contextra-erasure/src/report/build.rs:361` |
| `vault_medium` | `String` | `crates/contextra-erasure/src/report/build.rs:363` |
| `vault_rehash_count` | `u64` | `crates/contextra-erasure/src/report/build.rs:365` |
| `vault_old_overwritten` | `bool` | `crates/contextra-erasure/src/report/build.rs:367` |
| `vault_backup_scope` | `String` | `crates/contextra-erasure/src/report/build.rs:369` |
| `verified_via` | `String` | `crates/contextra-erasure/src/report/build.rs:371` |
| `epoch_id` | `String` | `crates/contextra-erasure/src/report/build.rs:373` |
| `replaced` | `bool` | `crates/contextra-erasure/src/report/build.rs:375` |
| `replacement_tag` | `Option<String>` | `crates/contextra-erasure/src/report/build.rs:377` |
| `signer_id` | `u8` | `crates/contextra-erasure/src/report/build.rs:379` |
| `process_hardening` | `String` | `crates/contextra-erasure/src/report/build.rs:381` |
| `risk_context` | `ResidualRiskContext` | `crates/contextra-erasure/src/report/build.rs:383` |
| `ledger_confirmation` | `Option<LedgerConfirmation>` | `crates/contextra-erasure/src/report/build.rs:385` |

#### `struct WitnessChainReport`
- Ort: `crates/contextra-erasure/src/report/build.rs:126`
```rust
pub struct WitnessChainReport
```
Dokumentation: Kryptografische Zeugenkette und Wurzel-Digest (SPEC §11.7, §12.2, INV-ERA-7). Jeder Zeuge wird über seinen deterministischen BLAKE3-Digest gebunden. Die Kette aggregiert die Einzel-Digests zu einem gemeinsamen Wurzel-Digest (`chain_root`).
- Impl-Block `crates/contextra-erasure/src/report/build.rs:133`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `witness_digests` | `Vec<String>` | `crates/contextra-erasure/src/report/build.rs:128` |
| `chain_root` | `String` | `crates/contextra-erasure/src/report/build.rs:130` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn compute_from_digests(digests: &[[u8` | `crates/contextra-erasure/src/report/build.rs:139` | Berechnet die Zeugenkette aus einer geordneten Liste von Zeugendigests (INV-ERA-7). Bei leerer Liste ist der Wurzel-Digest 32 Nullbytes. Bei n Zeugen wird die K |

#### `enum ReachStatus`
- Ort: `crates/contextra-erasure/src/report/build.rs:33`
```rust
pub enum ReachStatus
```
Dokumentation: Pflichtstatus einer Nachweisklasse (G, K, P laut SPEC D-36, §12.5).
- Impl-Block `crates/contextra-erasure/src/report/build.rs:40`

Varianten:

- `Reached` — `crates/contextra-erasure/src/report/build.rs:35`
- `NotReached` — `crates/contextra-erasure/src/report/build.rs:37`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn as_str(&self) -> &'static str` | `crates/contextra-erasure/src/report/build.rs:42` | Liefert den kanonischen String "reached" oder "not_reached". |

#### `fn build_report`
- Ort: `crates/contextra-erasure/src/report/build.rs:614`
```rust
pub fn build_report( erasure_id: ErasureId, gated_at: u64, completed_at: u64, witnesses: Vec<LayerWitness>, segments_rewritten: u64, bytes_rewritten: u64, purge_complete: bool, ) -> ErasureReport
```
Dokumentation: Erstellt einen Löschbericht mit Standardoptionen (ER-27).

#### `fn build_report_with_options`
- Ort: `crates/contextra-erasure/src/report/build.rs:421`
```rust
pub fn build_report_with_options( erasure_id: ErasureId, witnesses: Vec<LayerWitness>, segments_rewritten: u64, bytes_rewritten: u64, opts: ReportBuildOptions, ) -> ErasureReport
```
Dokumentation: Erstellt einen vollständigen, normativen Löschbericht nach SPEC §11.7.

#### `fn canonicalize_report`
- Ort: `crates/contextra-erasure/src/report/build.rs:645`
```rust
pub fn canonicalize_report(report: &ErasureReport) -> Result<Vec<u8>, ErasureError>
```
Dokumentation: Kanonisiert einen Bericht als Bytes (für die Signatur).

#### `fn try_build_report_with_options`
- Ort: `crates/contextra-erasure/src/report/build.rs:586`
```rust
pub fn try_build_report_with_options( erasure_id: ErasureId, witnesses: Vec<LayerWitness>, segments_rewritten: u64, bytes_rewritten: u64, opts: ReportBuildOptions, ) -> Result<ErasureReport, ErasureError>
```
Dokumentation: Erstellt einen normativen Löschbericht mit strenger Validierung (fail-closed, INV-CLASS-MEASURED-1). Schlägt fehl (`Err`), wenn: - keine Zeugen übergeben wurden (`witnesses.is_empty()`) - ein Zeuge unbewaffnet ist oder `matches > 0` hat - ein Zeuge ungültige Null-Hashes aufweist.

#### `type WitnessEntry`
- Ort: `crates/contextra-erasure/src/report/build.rs:108`
```rust
pub type WitnessEntry = LayerWitnessReport
```
Dokumentation: Zeuge einer einzelnen Schicht im Bericht (Abwärtskompatibilitäts-Alias).

### Modul `report::canonical`

#### `fn canonicalize_json`
- Ort: `crates/contextra-erasure/src/report/canonical.rs:90`
```rust
pub fn canonicalize_json<T: Serialize>(val: &T) -> Result<String, ErasureError>
```
Dokumentation: Alias für `to_canonical_json_string` für Abwärtskompatibilität.

#### `fn canonicalize_value`
- Ort: `crates/contextra-erasure/src/report/canonical.rs:40`
```rust
pub fn canonicalize_value(v: &Value) -> Result<Value, ErasureError>
```
Dokumentation: Kanonisiert einen `serde_json::Value` nach F.10.

#### `fn to_canonical_json`
- Ort: `crates/contextra-erasure/src/report/canonical.rs:77`
```rust
pub fn to_canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, ErasureError>
```
Dokumentation: Serialisiert einen Wert als kanonisches JSON (keine Leerzeichen, sortierte Schlüssel).

#### `fn to_canonical_json_string`
- Ort: `crates/contextra-erasure/src/report/canonical.rs:84`
```rust
pub fn to_canonical_json_string<T: Serialize>(value: &T) -> Result<String, ErasureError>
```
Dokumentation: Serialisiert und kanonisiert einen Wert als String.

#### `fn verify_canonical_nfc`
- Ort: `crates/contextra-erasure/src/report/canonical.rs:21`
```rust
pub fn verify_canonical_nfc(s: &str) -> Result<(), ErasureError>
```
Dokumentation: Prüft, ob ein UTF-8-String die Anforderungen an Normalform C (NFC) erfüllt (SPEC F.10). Zur Vermeidung neuer Abhängigkeiten (AR-3, Regel 4) wird eine deterministische Prüfung auf nicht zusammengesetzte Kombinationszeichen durchgeführt: Dekomponierte Folgen (Combining Diacritical Marks U+0300..=U+036F u. a.) werden erkannt und abgelehnt.

### Modul `report::risk`

#### `struct ResidualRiskContext`
- Ort: `crates/contextra-erasure/src/report/risk.rs:163`
```rust
pub struct ResidualRiskContext
```
Dokumentation: Kontext zur Auswertung der dynamischen Bedingungen für Restrisiken (SPEC Anhang D).
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/report/risk.rs:192`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `purge_complete` | `bool` | `crates/contextra-erasure/src/report/risk.rs:165` |
| `is_rotational` | `bool` | `crates/contextra-erasure/src/report/risk.rs:167` |
| `is_cow_or_journaling` | `bool` | `crates/contextra-erasure/src/report/risk.rs:169` |
| `is_file_vault` | `bool` | `crates/contextra-erasure/src/report/risk.rs:171` |
| `passphrase_changed` | `bool` | `crates/contextra-erasure/src/report/risk.rs:173` |
| `process_hardening_off_or_partial` | `bool` | `crates/contextra-erasure/src/report/risk.rs:175` |
| `vector_topology_dirty` | `bool` | `crates/contextra-erasure/src/report/risk.rs:177` |
| `fallback_stage_4` | `bool` | `crates/contextra-erasure/src/report/risk.rs:179` |
| `fallback_stage_3` | `bool` | `crates/contextra-erasure/src/report/risk.rs:181` |
| `verified_via_cache` | `bool` | `crates/contextra-erasure/src/report/risk.rs:183` |
| `vault_backup_scope_same_or_unknown` | `bool` | `crates/contextra-erasure/src/report/risk.rs:185` |
| `is_replace_erasing_pre_gate` | `bool` | `crates/contextra-erasure/src/report/risk.rs:187` |
| `is_superseded_with_same_key` | `bool` | `crates/contextra-erasure/src/report/risk.rs:189` |

#### `struct ResidualRiskEntry`
- Ort: `crates/contextra-erasure/src/report/risk.rs:315`
```rust
pub struct ResidualRiskEntry
```
Dokumentation: Serialisiert ein Restrisiko als maschinenlesbaren Eintrag.
- Impl-Block `crates/contextra-erasure/src/report/risk.rs:326`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra-erasure/src/report/risk.rs:317` |
| `name` | `String` | `crates/contextra-erasure/src/report/risk.rs:319` |
| `description` | `String` | `crates/contextra-erasure/src/report/risk.rs:321` |
| `operator_guidance` | `String` | `crates/contextra-erasure/src/report/risk.rs:323` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_id(risk: ResidualRiskId) -> Self` | `crates/contextra-erasure/src/report/risk.rs:328` | Erstellt einen Eintrag aus einer Risiko-ID. |
| `pub fn from_risk(risk: &ResidualRiskId) -> Self` | `crates/contextra-erasure/src/report/risk.rs:338` | Abwärtskompatibler Alias. |

#### `enum ResidualRiskId`
- Ort: `crates/contextra-erasure/src/report/risk.rs:10`
```rust
pub enum ResidualRiskId
```
Dokumentation: Identifikatoren für Restrisiken nach SPEC Anhang D.
- Impl-Block `crates/contextra-erasure/src/report/risk.rs:51`

Varianten:

- `FlashRemanence` — `crates/contextra-erasure/src/report/risk.rs:12`
- `FsJournalOrCow` — `crates/contextra-erasure/src/report/risk.rs:14`
- `WrappedKeyRemanence` — `crates/contextra-erasure/src/report/risk.rs:16`
- `FreedBlockRemanence` — `crates/contextra-erasure/src/report/risk.rs:18`
- `OldKeyslotRemanence` — `crates/contextra-erasure/src/report/risk.rs:20`
- `ProcessMemory` — `crates/contextra-erasure/src/report/risk.rs:22`
- `IndexTopologyInference` — `crates/contextra-erasure/src/report/risk.rs:24`
- `TextIndexUntilPurge` — `crates/contextra-erasure/src/report/risk.rs:26`
- `ReplicaWindow` — `crates/contextra-erasure/src/report/risk.rs:28`
- `ExternalCopies` — `crates/contextra-erasure/src/report/risk.rs:30`
- `ExternalProcessors` — `crates/contextra-erasure/src/report/risk.rs:32`
- `PseudonymCommitment` — `crates/contextra-erasure/src/report/risk.rs:34`
- `PurgeScalesWithDb` — `crates/contextra-erasure/src/report/risk.rs:36`
- `PurgeDeferred` — `crates/contextra-erasure/src/report/risk.rs:38`
- `KeyCompromise` — `crates/contextra-erasure/src/report/risk.rs:40`
- `VerifiedViaCache` — `crates/contextra-erasure/src/report/risk.rs:42`
- `BackupKeyMaterial` — `crates/contextra-erasure/src/report/risk.rs:44`
- `ReplacementOverlap` — `crates/contextra-erasure/src/report/risk.rs:46`
- `SupersededWithSameKey` — `crates/contextra-erasure/src/report/risk.rs:48`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn as_str(&self) -> &'static str` | `crates/contextra-erasure/src/report/risk.rs:53` | Gibt den kanonischen Bezeichner nach Anhang D und `ctx-verify` zurück. |
| `pub const fn id_token(&self) -> &'static str` | `crates/contextra-erasure/src/report/risk.rs:78` | Gibt den Token nach SPEC Anhang D zurück. |
| `pub const fn description(&self) -> &'static str` | `crates/contextra-erasure/src/report/risk.rs:103` | Gibt die Kurzbezeichnung / Text nach SPEC Anhang D zurück. |
| `pub const fn operator_guidance(&self) -> &'static str` | `crates/contextra-erasure/src/report/risk.rs:128` | Gibt den Betreiberhinweis nach SPEC Anhang D zurück. |

#### `fn evaluate_residual_risks`
- Ort: `crates/contextra-erasure/src/report/risk.rs:305`
```rust
pub fn evaluate_residual_risks(purge_complete: bool) -> Vec<ResidualRiskId>
```
Dokumentation: Abwärtskompatible Auswertung aller Restrisiken (ER-28). Beinhaltet standardmäßig alle bedingungslosen und die Purge-abhängigen Risiken.

#### `fn evaluate_residual_risks_with_context`
- Ort: `crates/contextra-erasure/src/report/risk.rs:213`
```rust
pub fn evaluate_residual_risks_with_context(ctx: &ResidualRiskContext) -> Vec<ResidualRiskId>
```
Dokumentation: Bewertet alle Restrisiken mit vollständigem Kontext gemäß Anhang D.

### Modul `report::signing`

#### `struct DerivedReportSigner`
- Ort: `crates/contextra-erasure/src/report/signing.rs:268`
```rust
pub struct DerivedReportSigner
```
Dokumentation: Implementierung des `ReportSigner`-Ports für intern abgeleitete Schlüssel (signer_id = 0, D-130).
- Impl-Block `crates/contextra-erasure/src/report/signing.rs:273`
- Impl-Block (Trait `ReportSignerPort`) `crates/contextra-erasure/src/report/signing.rs:285`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(key: SigningKey) -> Self` | `crates/contextra-erasure/src/report/signing.rs:275` | Erzeugt einen neuen internen Signierer (signer_id = 0). |
| `pub fn with_signer_id(key: SigningKey, signer_id: SignerId) -> Self` | `crates/contextra-erasure/src/report/signing.rs:280` | Erzeugt einen Signierer mit benutzerdefinierter Signer-ID (z. B. 1 für Betreiber). |

#### `struct ReportSigner`
- Ort: `crates/contextra-erasure/src/report/signing.rs:322`
```rust
pub struct ReportSigner
```
Dokumentation: Statische Hilfsfunktionen für Berichtssignierung und VFS-Persistierung (Abwärtskompatibilität).
- Impl-Block `crates/contextra-erasure/src/report/signing.rs:324`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn sign_bytes( report_bytes: &[u8], signature: String, pub_fingerprint: String, ) -> SignedReport` | `crates/contextra-erasure/src/report/signing.rs:326` | Erstellt einen signierten Bericht aus kanonischen Bytes. |
| `pub fn sign_report( report: &ErasureReport, key: &SigningKey, ) -> Result<SignedReport, ErasureError>` | `crates/contextra-erasure/src/report/signing.rs:340` | Signiert einen kanonischen Bericht mit dem Ed25519-Schlüssel (SPEC §11.7, F.10, D-130). |
| `pub fn persist_signed_report( vfs: &dyn Vfs, target_path: &Path, signed: &SignedReport, ) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/report/signing.rs:375` | Schreibt und synchronisiert den signierten Bericht atomar über Vfs (SPEC §11.8). |

#### `struct SignedReport`
- Ort: `crates/contextra-erasure/src/report/signing.rs:26`
```rust
pub struct SignedReport
```
Dokumentation: Signierter Löschbericht mit Base64url-Signatur und Prüffingerabdruck (ER-27, SPEC §11.7, F.10).
- Impl-Block `crates/contextra-erasure/src/report/signing.rs:37`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `canonical_body` | `String` | `crates/contextra-erasure/src/report/signing.rs:28` |
| `signature_base64` | `String` | `crates/contextra-erasure/src/report/signing.rs:30` |
| `pub_fingerprint` | `String` | `crates/contextra-erasure/src/report/signing.rs:32` |
| `signer_id` | `u8` | `crates/contextra-erasure/src/report/signing.rs:34` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn sign(report: ErasureReport, key: &SigningKey) -> Result<Self, ErasureError>` | `crates/contextra-erasure/src/report/signing.rs:39` | Signiert einen kanonischen Bericht mit einem `SigningKey`. |
| `pub fn signature_base64(&self) -> &str` | `crates/contextra-erasure/src/report/signing.rs:44` | Liefert die Base64url-Signatur. |
| `pub fn verify_with_key(&self, verifying_key: &VerifyingKey) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/report/signing.rs:49` | Verifiziert die Integrität und Signatur des Berichts gegen den VerifyingKey. |
| `pub fn verify_report_with_chain( &self, verifying_key: &VerifyingKey, ) -> Result<ErasureReport, ErasureError>` | `crates/contextra-erasure/src/report/signing.rs:87` | Verifiziert sowohl die Ed25519-Signatur als auch die kryptografische Zeugenkette (INV-ERA-7, INV-CLASS-MEASURED-1). Validiert: 1. Ed25519 verify_strict über Sig |
| `pub fn verify(&self) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/report/signing.rs:144` | Verifiziert die Integrität (erfordert bekannten Verifying-Key oder decodierten Fingerprint). |

#### `fn base64url_encode`
- Ort: `crates/contextra-erasure/src/report/signing.rs:230`
```rust
pub fn base64url_encode(data: &[u8]) -> String
```
Dokumentation: Hilfsfunktion für Base64url-Kodierung ohne Padding (F.10, RFC 4648 §5).

#### `type BoxFuture`
- Ort: `crates/contextra-erasure/src/report/signing.rs:19`
```rust
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>
```
Dokumentation: Type-Alias für BoxFuture ohne tokio/futures-util.

### Modul `restore`

#### `mod guard`
- Ort: `crates/contextra-erasure/src/restore.rs:5`
```rust
pub mod guard
```

#### `mod replay`
- Ort: `crates/contextra-erasure/src/restore.rs:6`
```rust
pub mod replay
```

#### `struct RestoreGuard`
- Ort: `crates/contextra-erasure/src/restore.rs:15`
```rust
pub struct RestoreGuard
```
Dokumentation: Restore-Guard prüft Sequenznummern gegen den Ledger-Floor.
- Impl-Block `crates/contextra-erasure/src/restore.rs:19`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(ledger_floor: Seq) -> Self` | `crates/contextra-erasure/src/restore.rs:21` | Erstellt einen neuen RestoreGuard mit einem Ledger-Floor. |
| `pub fn is_replay(&self, seq: Seq) -> bool` | `crates/contextra-erasure/src/restore.rs:26` | Prüft, ob eine Sequenznummer kleiner oder gleich dem Floor ist (Replay / Altbestand). |

### Modul `restore::guard`

#### `struct FileRestoreChecker`
- Ort: `crates/contextra-erasure/src/restore/guard.rs:33`
```rust
pub struct FileRestoreChecker<'a>
```
Dokumentation: Dateibasierter Restore-Checker: prüft auf Zeitregression und Wiederauftauchen (ER-30).
- Impl-Block `crates/contextra-erasure/src/restore/guard.rs:37`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(ledger: &'a InMemoryLedger) -> Self` | `crates/contextra-erasure/src/restore/guard.rs:39` | Erstellt einen neuen Checker mit Ledger-Referenz. |
| `pub fn check(&self, store_current_seq: u64) -> GuardOutcome` | `crates/contextra-erasure/src/restore/guard.rs:46` | Führt die Integritätsprüfung durch (ER-30). `store_current_seq`: aktuelle Sequenznummer des Stores. |
| `pub fn next_safe_seq(&self, store_current_seq: u64) -> u64` | `crates/contextra-erasure/src/restore/guard.rs:62` | Berechnet die nächste sichere Sequenznummer: `max(current_seq, ledger_floor) + 1` (ER-30). |

#### `enum GuardOutcome`
- Ort: `crates/contextra-erasure/src/restore/guard.rs:13`
```rust
pub enum GuardOutcome
```
Dokumentation: Ergebnis des Restore-Guard-Checks.

Varianten:

- `Clean` — `crates/contextra-erasure/src/restore/guard.rs:15`
- `TimeRegression{` — `crates/contextra-erasure/src/restore/guard.rs:17`
- `Reappearance{` — `crates/contextra-erasure/src/restore/guard.rs:24`
- `LedgerMissing` — `crates/contextra-erasure/src/restore/guard.rs:29`

### Modul `restore::replay`

#### `struct ReplayOutcome`
- Ort: `crates/contextra-erasure/src/restore/replay.rs:11`
```rust
pub struct ReplayOutcome
```
Dokumentation: Ergebnis eines Replay-Vorgangs.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `replayed_count` | `u64` | `crates/contextra-erasure/src/restore/replay.rs:13` |
| `store_cleared` | `bool` | `crates/contextra-erasure/src/restore/replay.rs:15` |

#### `fn replay_erasures`
- Ort: `crates/contextra-erasure/src/restore/replay.rs:22`
```rust
pub fn replay_erasures(affected_ids: &[ErasureId]) -> Result<ReplayOutcome, ErasureError>
```
Dokumentation: Führt einen Replay-Vorgang für betroffene Löschungen durch (ER-30 / INV-ERA-2). Ein Wiederanlauf (Resume) führt den Purge für jede nicht abgeschlossene Saga aktiv fort, anstatt nur den Zustand auszulesen.

#### `fn resume_and_execute_purge`
- Ort: `crates/contextra-erasure/src/restore/replay.rs:35`
```rust
pub fn resume_and_execute_purge( saga: &mut crate::saga::machine::EraseSaga, driver: &crate::saga::driver::SagaDriver, ) -> Result<crate::saga::driver::StepOutcome, ErasureError>
```
Dokumentation: Führt die aktive Wiederaufnahme (Resume) einer unterbrochenen Saga aus (INV-ERA-2).

### Modul `restricted`

#### `struct OperatorAttestation`
- Ort: `crates/contextra-erasure/src/restricted.rs:32`
```rust
pub struct OperatorAttestation
```
Dokumentation: Attestierung des beauftragten Betreibers für Art. 18 Abs. 2 Zugriff (SPEC D-116).
- Impl-Block `crates/contextra-erasure/src/restricted.rs:38`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(operator: &str, case_ref: &[u8], attested_at: u64, epoch_key: &[u8` | `crates/contextra-erasure/src/restricted.rs:43` | Erstellt eine neue Betreiber-Attestierung mit gehashtem Fallbezug. `case_ref` wird kryptografisch mit dem Epochenschlüssel gehasht (`case_commitment`), sodass k |
| `pub fn operator(&self) -> &str` | `crates/contextra-erasure/src/restricted.rs:54` | Liefert den Betreiber-Identifikator. |
| `pub fn case_commitment(&self) -> &[u8` | `crates/contextra-erasure/src/restricted.rs:60` | Liefert das kryptografische Commitment des Fallbezugs. |
| `pub fn attested_at(&self) -> u64` | `crates/contextra-erasure/src/restricted.rs:66` | Liefert den Zeitstempel der Attestierung. |

#### `struct RestrictedAccess`
- Ort: `crates/contextra-erasure/src/restricted.rs:73`
```rust
pub struct RestrictedAccess
```
Dokumentation: Autorisierungskontext für den Zugriff auf eingeschränkte Daten (SPEC D-116).
- Impl-Block `crates/contextra-erasure/src/restricted.rs:78`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(purpose: RestrictedPurpose, attestation: OperatorAttestation) -> Self` | `crates/contextra-erasure/src/restricted.rs:80` | Erstellt ein neues Zugriffsgesuch nach Art. 18 Abs. 2 DSGVO. |
| `pub fn purpose(&self) -> RestrictedPurpose` | `crates/contextra-erasure/src/restricted.rs:89` | Liefert den rechtlichen Zweck. |
| `pub fn attestation(&self) -> &OperatorAttestation` | `crates/contextra-erasure/src/restricted.rs:95` | Liefert die zugehörige Betreiber-Attestierung. |
| `pub fn allows_access_to_restricted(&self, is_target_restricted: bool) -> bool` | `crates/contextra-erasure/src/restricted.rs:102` | Prüft, ob der Pfad ausschließlich auf `Restricted`-Bereiche angewandt wird (SPEC D-116, T-RESTR-3). Gegen `Erased`-Bereiche ist kein Zugriff möglich; liefert `f |
| `pub fn validate_batch_size(&self, batch_size: usize) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/restricted.rs:113` | Validiert die Stapelgröße (SPEC D-116: höchstens 1 000 Dokumente). |

#### `struct RestrictedReadAudit`
- Ort: `crates/contextra-erasure/src/restricted.rs:126`
```rust
pub struct RestrictedReadAudit
```
Dokumentation: Audit-Datensatz für Journal Code 9 `RestrictedRead` (SPEC D-116, F.9).
- Impl-Block `crates/contextra-erasure/src/restricted.rs:141`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `erasure_id` | `ErasureId` | `crates/contextra-erasure/src/restricted.rs:128` |
| `record_code` | `u8` | `crates/contextra-erasure/src/restricted.rs:130` |
| `purpose` | `RestrictedPurpose` | `crates/contextra-erasure/src/restricted.rs:132` |
| `timestamp` | `u64` | `crates/contextra-erasure/src/restricted.rs:134` |
| `doc_count` | `u32` | `crates/contextra-erasure/src/restricted.rs:136` |
| `case_commitment` | `[u8; 32]` | `crates/contextra-erasure/src/restricted.rs:138` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn create_entry( erasure_id: ErasureId, access: &RestrictedAccess, doc_count: u32, timestamp: u64, ) -> Result<Self, ErasureError>` | `crates/contextra-erasure/src/restricted.rs:143` | Erstellt einen auditierbaren Journal-Eintrag für Art. 18 Abs. 2 Zugriff. |

#### `enum RestrictedPurpose`
- Ort: `crates/contextra-erasure/src/restricted.rs:23`
```rust
pub enum RestrictedPurpose
```
Dokumentation: Gesetzlicher Ausnahmezweck nach Art. 18 Abs. 2 DSGVO (SPEC D-116).

Varianten:

- `LegalClaims` — `crates/contextra-erasure/src/restricted.rs:25`
- `DataSubjectConsent` — `crates/contextra-erasure/src/restricted.rs:27`

#### `enum RestrictedState`
- Ort: `crates/contextra-erasure/src/restricted.rs:164`
```rust
pub enum RestrictedState
```
Dokumentation: Zustandsmodell für eingeschränkte Daten (Art. 18 DSGVO, SPEC §11.2, D-116, D-119).
- Impl-Block `crates/contextra-erasure/src/restricted.rs:173`

Varianten:

- `Restricted` — `crates/contextra-erasure/src/restricted.rs:166`
- `Lifted` — `crates/contextra-erasure/src/restricted.rs:168`
- `SupersededByErasure` — `crates/contextra-erasure/src/restricted.rs:170`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn is_lifted(&self) -> bool` | `crates/contextra-erasure/src/restricted.rs:175` | Prüft, ob die Einschränkung aufgehoben ist. |
| `pub fn is_terminal(&self) -> bool` | `crates/contextra-erasure/src/restricted.rs:180` | Prüft, ob es sich um einen Endzustand handelt. |
| `pub fn lift(&mut self, _reason: &str) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/restricted.rs:185` | Hebt die Restriktion mit einer Begründung auf (Journal-Eintrag erforderlich). |
| `pub fn supersede_by_erasure(&mut self) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/restricted.rs:199` | Ersetzt die Restriktion durch einen nachfolgenden regulären Erase-Vorgang (D-119). |

### Modul `revocation_log`

#### `struct RevocationEntry`
- Ort: `crates/contextra-erasure/src/revocation_log.rs:73`
```rust
pub struct RevocationEntry
```
Dokumentation: Ein einzelner Eintrag im manipulationssicheren Widerrufslog.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `sequence_number` | `u64` | `crates/contextra-erasure/src/revocation_log.rs:75` |
| `target` | `RevocationTarget` | `crates/contextra-erasure/src/revocation_log.rs:77` |
| `revoked_at_nanos` | `u64` | `crates/contextra-erasure/src/revocation_log.rs:79` |
| `prev_hash` | `[u8; 32]` | `crates/contextra-erasure/src/revocation_log.rs:81` |
| `entry_hash` | `[u8; 32]` | `crates/contextra-erasure/src/revocation_log.rs:83` |
| `signature` | `[u8; 64]` | `crates/contextra-erasure/src/revocation_log.rs:86` |

#### `struct RevocationLog`
- Ort: `crates/contextra-erasure/src/revocation_log.rs:128`
```rust
pub struct RevocationLog
```
Dokumentation: Persistentes Protokoll über widerrufene Schlüsselgruppen.
- Impl-Block `crates/contextra-erasure/src/revocation_log.rs:137`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-erasure/src/revocation_log.rs:321`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new_in_memory( clock: Arc<dyn Clock>, signing_key: Option<SigningKey>, verifying_key: VerifyingKey, ) -> Self` | `crates/contextra-erasure/src/revocation_log.rs:139` | Erstellt einen neuen In-Memory-Widerrufslog ohne Dateipersistenz. |
| `pub fn open_or_create( vfs: Arc<dyn Vfs>, path: impl AsRef<Path>, clock: Arc<dyn Clock>, signing_key: Option<SigningKey>, verifying_key: VerifyingKey, ) -> Result<Self>` | `crates/contextra-erasure/src/revocation_log.rs:159` | Öffnet einen bestehenden Widerrufslog oder erzeugt einen neuen unter `path` über den Vfs-Port. Beim Öffnen einer bestehenden Datei wird die komplette Hash-Kette |
| `pub fn verify_integrity(&self) -> Result<()>` | `crates/contextra-erasure/src/revocation_log.rs:203` | Verifiziert die Integrität der gesamten Logeinträge-Kette (Sequenz, Hashes, Ed25519-Signaturen). |
| `pub fn append(&self, target: RevocationTarget) -> Result<RevocationEntry>` | `crates/contextra-erasure/src/revocation_log.rs:250` | Fügt einen neuen Widerrufs-Eintrag für `target` an den Log an. Nutzt den injizierten Clock-Port zur Generierung des Zeitstempels und den Signierschlüssel zur Er |
| `pub fn revoked_targets(&self) -> AHashSet<RevocationTarget>` | `crates/contextra-erasure/src/revocation_log.rs:300` | Gibt eine Kopie aller aktuell im Log widerrufenen Targets zurück. |
| `pub fn is_revoked(&self, target: &RevocationTarget) -> bool` | `crates/contextra-erasure/src/revocation_log.rs:305` | Prüft, ob ein gegebenes Target (KEK, DEK, Group oder Record) widerrufen wurde. |
| `pub fn len(&self) -> usize` | `crates/contextra-erasure/src/revocation_log.rs:311` | Gibt die Anzahl der Widerrufe im Log zurück. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-erasure/src/revocation_log.rs:316` | Prüft, ob der Log leer ist. |

#### `enum RevocationTarget`
- Ort: `crates/contextra-erasure/src/revocation_log.rs:48`
```rust
pub enum RevocationTarget
```
Dokumentation: Das Ziel eines Widerruf-Eintrags im Log (Group ID, KEK ID, DEK ID oder Record ID).
- Impl-Block `crates/contextra-erasure/src/revocation_log.rs:59`

Varianten:

- `Group(u64)` — `crates/contextra-erasure/src/revocation_log.rs:50`
- `Kek(String)` — `crates/contextra-erasure/src/revocation_log.rs:52`
- `Dek(String)` — `crates/contextra-erasure/src/revocation_log.rs:54`
- `Record(String)` — `crates/contextra-erasure/src/revocation_log.rs:56`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn canonical_bytes(&self) -> Vec<u8>` | `crates/contextra-erasure/src/revocation_log.rs:61` | Gibt eine kanonische Byte-Repräsentation des Targets für das Hashing zurück. |

#### `fn compute_entry_hash`
- Ort: `crates/contextra-erasure/src/revocation_log.rs:111`
```rust
pub fn compute_entry_hash( sequence_number: u64, target: &RevocationTarget, revoked_at_nanos: u64, prev_hash: &[u8
```
Dokumentation: Berechnet den SHA-256-Hash eines Eintrags aus seinen kanonischen Feldern.

#### `type Result`
- Ort: `crates/contextra-erasure/src/revocation_log.rs:35`
```rust
pub type Result<T> = std::result::Result<T, ErasureError>
```
Dokumentation: Typalias für Result.

### Modul `revocation_log::serde_64_bytes`

#### `fn deserialize`
- Ort: `crates/contextra-erasure/src/revocation_log.rs:99`
```rust
pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8
```

#### `fn serialize`
- Ort: `crates/contextra-erasure/src/revocation_log.rs:92`
```rust
pub fn serialize<S>(bytes: &[u8
```

### Modul `saga`

#### `mod core`
- Ort: `crates/contextra-erasure/src/saga.rs:6`
```rust
pub mod core
```
- Gate: `#[cfg(feature = "async")]`

#### `mod driver`
- Ort: `crates/contextra-erasure/src/saga.rs:7`
```rust
pub mod driver
```

#### `mod idempotency`
- Ort: `crates/contextra-erasure/src/saga.rs:8`
```rust
pub mod idempotency
```

#### `mod machine`
- Ort: `crates/contextra-erasure/src/saga.rs:9`
```rust
pub mod machine
```

#### `mod retention`
- Ort: `crates/contextra-erasure/src/saga.rs:10`
```rust
pub mod retention
```

#### `mod seams`
- Ort: `crates/contextra-erasure/src/saga.rs:11`
```rust
pub mod seams
```

### Modul `saga::core`

#### `struct EraseExecutionRequest`
- Ort: `crates/contextra-erasure/src/saga/core.rs:111`
```rust
pub struct EraseExecutionRequest
```
Dokumentation: Antrag an den Löschkern.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `erasure_id` | `ErasureId` | `crates/contextra-erasure/src/saga/core.rs:113` |
| `target` | `EraseTarget` | `crates/contextra-erasure/src/saga/core.rs:115` |
| `mode` | `EraseMode` | `crates/contextra-erasure/src/saga/core.rs:117` |
| `attestation` | `RetentionAttestation` | `crates/contextra-erasure/src/saga/core.rs:119` |
| `timing` | `Timing` | `crates/contextra-erasure/src/saga/core.rs:121` |

#### `struct ErasureConfig`
- Ort: `crates/contextra-erasure/src/saga/core.rs:52`
```rust
pub struct ErasureConfig
```
Dokumentation: Konfiguration für den Löschkern (SPEC §4, §11.2).
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/saga/core.rs:65`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `erase_return_timeout` | `Duration` | `crates/contextra-erasure/src/saga/core.rs:54` |
| `erase_sla` | `Duration` | `crates/contextra-erasure/src/saga/core.rs:56` |
| `batch_window` | `Duration` | `crates/contextra-erasure/src/saga/core.rs:58` |
| `min_free_bytes` | `u64` | `crates/contextra-erasure/src/saga/core.rs:60` |
| `k_idx` | `[u8; 32]` | `crates/contextra-erasure/src/saga/core.rs:62` |

#### `struct ErasureCore`
- Ort: `crates/contextra-erasure/src/saga/core.rs:194`
```rust
pub struct ErasureCore
```
Dokumentation: Haupt-Orchestrator des Löschkerns (SPEC §11.2, AUFTRAG J-07).
- Impl-Block `crates/contextra-erasure/src/saga/core.rs:167`
- Impl-Block `crates/contextra-erasure/src/saga/core.rs:201`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(cfg: ErasureConfig, deps: ErasureDeps) -> Self` | `crates/contextra-erasure/src/saga/core.rs:203` | Erstellt eine neue Instanz des Orchestrators. |
| `pub fn config(&self) -> &ErasureConfig` | `crates/contextra-erasure/src/saga/core.rs:213` | Gibt die Konfiguration zurück. |
| `pub fn deps(&self) -> &Arc<ErasureDeps>` | `crates/contextra-erasure/src/saga/core.rs:218` | Gibt die Abhängigkeiten zurück. |
| `pub fn evaluate_retention( mode: EraseMode, attestation: &RetentionAttestation, ) -> Result<RetentionDecision, ErasureError>` | `crates/contextra-erasure/src/saga/core.rs:255` | Führt die Validierung der Retention-Attestierung gemäß Matrix D-124 Abs. 6 durch. |
| `pub async fn erase(&self, req: EraseExecutionRequest) -> Result<EraseReceipt, ErasureError>` | `crates/contextra-erasure/src/saga/core.rs:283` | Nimmt eine Löschung an und führt sie bis mindestens `Gated` aus (SPEC §11.2, ER-23). Cancel-Safety (ER-24): Die Ausführung läuft in einer unabhängigen Tokio-Tas |
| `pub async fn run_purge_window( &self, ) -> Result<crate::purge::window::WindowOutcome, ErasureError>` | `crates/contextra-erasure/src/saga/core.rs:786` | Führt ein Purge-Fenster über alle offenen Sagas im Zustand `KeyDestroyed` oder `Purging` aus. |
| `pub async fn recover_and_resume(&self) -> Result<Vec<ErasureId>, ErasureError>` | `crates/contextra-erasure/src/saga/core.rs:846` | Wiederanlauf und Recovery unfertiger Sagas (SPEC §11.2 Recovery, I-15, FM-1..14). |
| `pub async fn status(&self, id: ErasureId) -> Result<ErasureCoreStatus, ErasureError>` | `crates/contextra-erasure/src/saga/core.rs:888` | Fragt den Status einer Löschung ab. |

#### `struct ErasureCoreStatus`
- Ort: `crates/contextra-erasure/src/saga/core.rs:126`
```rust
pub struct ErasureCoreStatus
```
Dokumentation: Status eines laufenden oder abgeschlossenen Löschauftrags.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `ErasureId` | `crates/contextra-erasure/src/saga/core.rs:128` |
| `state` | `SagaState` | `crates/contextra-erasure/src/saga/core.rs:130` |
| `status` | `ErasureStatus` | `crates/contextra-erasure/src/saga/core.rs:132` |
| `gated_seq` | `Option<u64>` | `crates/contextra-erasure/src/saga/core.rs:134` |

#### `struct ErasureDeps`
- Ort: `crates/contextra-erasure/src/saga/core.rs:78`
```rust
pub struct ErasureDeps
```
Dokumentation: Abhängigkeiten des Löschkerns (SPEC §4 Schnittstellen, Ports & Seams).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `journal` | `Arc<dyn SagaJournal>` | `crates/contextra-erasure/src/saga/core.rs:80` |
| `gate` | `Arc<dyn SagaGate>` | `crates/contextra-erasure/src/saga/core.rs:82` |
| `vault` | `Arc<dyn SagaVault>` | `crates/contextra-erasure/src/saga/core.rs:84` |
| `measure` | `Arc<dyn SagaMeasure>` | `crates/contextra-erasure/src/saga/core.rs:86` |
| `report` | `Arc<dyn SagaReport>` | `crates/contextra-erasure/src/saga/core.rs:88` |
| `ledger` | `Arc<dyn SagaLedger>` | `crates/contextra-erasure/src/saga/core.rs:90` |
| `snapshots` | `Arc<dyn SnapshotRevoker>` | `crates/contextra-erasure/src/saga/core.rs:92` |
| `layers` | `Vec<Arc<dyn ErasureLayer>>` | `crates/contextra-erasure/src/saga/core.rs:94` |
| `commit_barrier` | `Arc<dyn CommitBarrier>` | `crates/contextra-erasure/src/saga/core.rs:98` |
| `purge_barrier` | `Arc<dyn PurgeBarrier>` | `crates/contextra-erasure/src/saga/core.rs:100` |
| `clock` | `Arc<dyn Clock>` | `crates/contextra-erasure/src/saga/core.rs:102` |
| `ids` | `Arc<dyn IdGen>` | `crates/contextra-erasure/src/saga/core.rs:104` |
| `observer` | `Option<Arc<dyn ErasureObserver>>` | `crates/contextra-erasure/src/saga/core.rs:106` |

#### `enum RetentionDecision`
- Ort: `crates/contextra-erasure/src/saga/core.rs:925`
```rust
pub enum RetentionDecision
```
Dokumentation: Ergebnis der Retention-Prüfung.

Varianten:

- `ProceedErase` — `crates/contextra-erasure/src/saga/core.rs:927`
- `ProceedRestrict` — `crates/contextra-erasure/src/saga/core.rs:929`
- `Refuse(RefusalReason)` — `crates/contextra-erasure/src/saga/core.rs:931`

### Modul `saga::driver`

#### `struct SagaDriver`
- Ort: `crates/contextra-erasure/src/saga/driver.rs:37`
```rust
pub struct SagaDriver
```
Dokumentation: Treiber-Kontext für einen einzelnen Saga-Schritt.
- Impl-Block `crates/contextra-erasure/src/saga/driver.rs:42`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/saga/driver.rs:44` | Erstellt einen neuen Treiber. |
| `pub fn with_core(core: Arc<super::core::ErasureCore>) -> Self` | `crates/contextra-erasure/src/saga/driver.rs:57` | Erstellt einen Treiber mit Anbindung an den `ErasureCore`. |
| `pub fn core(&self) -> Option<&Arc<super::core::ErasureCore>>` | `crates/contextra-erasure/src/saga/driver.rs:63` | Liefert die Core-Referenz, falls vorhanden. |
| `pub fn step(&self, saga: &mut EraseSaga) -> Result<StepOutcome, ErasureError>` | `crates/contextra-erasure/src/saga/driver.rs:70` | Führt den nächsten Schritt der Saga durch (idempotent, ER-18). Validiert den Übergang und vollzieht den Schritt in der Zustandsmaschine. |
| `pub async fn step_async(&self, saga: &mut EraseSaga) -> Result<StepOutcome, ErasureError>` | `crates/contextra-erasure/src/saga/driver.rs:102` | Asynchrone Ausführung des nächsten Treiberschritts unter Nutzung des `ErasureCore`. |
| `pub fn process_event( &self, saga: &mut EraseSaga, event: SagaEvent, ) -> Result<StepOutcome, ErasureError>` | `crates/contextra-erasure/src/saga/driver.rs:117` | Verarbeitet ein Ereignis aus der Treiber-Pipeline (SPEC §11.2). |

#### `enum SagaEvent`
- Ort: `crates/contextra-erasure/src/saga/driver.rs:15`
```rust
pub enum SagaEvent
```
Dokumentation: Ereignis der Treiber-Pipeline (SPEC §11.2).

Varianten:

- `Advance(ErasureId)` — `crates/contextra-erasure/src/saga/driver.rs:17`
- `SetPending(ErasureId, PendingReason)` — `crates/contextra-erasure/src/saga/driver.rs:19`
- `Complete(ErasureId)` — `crates/contextra-erasure/src/saga/driver.rs:21`

#### `enum StepOutcome`
- Ort: `crates/contextra-erasure/src/saga/driver.rs:26`
```rust
pub enum StepOutcome
```
Dokumentation: Ergebnis eines Treiberschritts.

Varianten:

- `Advanced(SagaState)` — `crates/contextra-erasure/src/saga/driver.rs:28`
- `Pending(PendingReason)` — `crates/contextra-erasure/src/saga/driver.rs:30`
- `Terminal(SagaState)` — `crates/contextra-erasure/src/saga/driver.rs:32`

### Modul `saga::idempotency`

#### `fn derive_open_key`
- Ort: `crates/contextra-erasure/src/saga/idempotency.rs:7`
```rust
pub fn derive_open_key(k_idx: &[u8
```
Dokumentation: Berechnet den Open-Lookup-Key nach D-119: `open_key = BLAKE3-keyed(K_idx, "ctx/erasure-open/v1" || kanonisches_Ziel || mode)`

#### `fn is_same_request`
- Ort: `crates/contextra-erasure/src/saga/idempotency.rs:17`
```rust
pub fn is_same_request(a_erasure_id: [u8
```
Dokumentation: Prüft ob zwei `EraseRequest`-Anfragen dieselbe Löschung darstellen. Gleiches Ziel, gleicher Modus → gleiche ErasureId.

### Modul `saga::machine`

#### `struct EraseRequest`
- Ort: `crates/contextra-erasure/src/saga/machine.rs:18`
```rust
pub struct EraseRequest
```
Dokumentation: Anfrage zur Löschung (Kompatibilität mit Ports & Tests).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `target` | `PortEraseTarget` | `crates/contextra-erasure/src/saga/machine.rs:20` |
| `erasure_id` | `[u8; 16]` | `crates/contextra-erasure/src/saga/machine.rs:22` |

#### `struct EraseSaga`
- Ort: `crates/contextra-erasure/src/saga/machine.rs:109`
```rust
pub struct EraseSaga
```
Dokumentation: Zustandsmaschine für einen einzelnen Löschvorgang.
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/saga/machine.rs:118`
- Impl-Block `crates/contextra-erasure/src/saga/machine.rs:124`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(erasure_id: ErasureId) -> Self` | `crates/contextra-erasure/src/saga/machine.rs:126` | Erstellt eine neue Saga mit gegebener ErasureId im Zustand `Requested`. |
| `pub fn with_id(erasure_id: ErasureId) -> Self` | `crates/contextra-erasure/src/saga/machine.rs:135` | Erstellt eine neue Saga mit gegebener ErasureId im Zustand `Requested`. |
| `pub fn with_state(erasure_id: ErasureId, state: SagaState) -> Self` | `crates/contextra-erasure/src/saga/machine.rs:140` | Erstellt eine Saga mit einem vorgegebenen Zustand (Recovery-Pfad). |
| `pub fn erasure_id(&self) -> ErasureId` | `crates/contextra-erasure/src/saga/machine.rs:149` | Gibt die Lösch-ID zurück. |
| `pub fn state(&self) -> SagaState` | `crates/contextra-erasure/src/saga/machine.rs:154` | Gibt den aktuellen Zustand zurück. |
| `pub fn pending_reason(&self) -> Option<&PendingReason>` | `crates/contextra-erasure/src/saga/machine.rs:159` | Gibt den letzten Pending-Grund zurück (falls vorhanden). |
| `pub fn transition_to(&mut self, next: SagaState) -> Result<(), crate::error::ErasureError>` | `crates/contextra-erasure/src/saga/machine.rs:164` | Führt einen einfachen Zustandsübergang durch (idempotent). |
| `pub fn transition_with_reason( &mut self, next: SagaState, reason: Option<PendingReason>, ) -> Result<(), crate::error::ErasureError>` | `crates/contextra-erasure/src/saga/machine.rs:171` | Führt einen Zustandsübergang mit Begründung durch (ER-18: idempotent). Gibt `Err` zurück wenn der Übergang nicht erlaubt ist. |
| `pub fn is_terminal(&self) -> bool` | `crates/contextra-erasure/src/saga/machine.rs:188` | Prüft ob die Saga abgeschlossen ist. |

#### `enum SagaState`
- Ort: `crates/contextra-erasure/src/saga/machine.rs:27`
```rust
pub enum SagaState
```
Dokumentation: Saga-Zustand (SPEC §11.2, §7.4, D-124 Abs. 5, ER-29).
- Impl-Block `crates/contextra-erasure/src/saga/machine.rs:50`
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-erasure/src/saga/machine.rs:90`

Varianten:

- `Requested` — `crates/contextra-erasure/src/saga/machine.rs:29`
- `Gated` — `crates/contextra-erasure/src/saga/machine.rs:31`
- `KeyDestroyed` — `crates/contextra-erasure/src/saga/machine.rs:33`
- `Purging` — `crates/contextra-erasure/src/saga/machine.rs:35`
- `Verifying` — `crates/contextra-erasure/src/saga/machine.rs:37`
- `Completed` — `crates/contextra-erasure/src/saga/machine.rs:39`
- `Restricted` — `crates/contextra-erasure/src/saga/machine.rs:41`
- `Lifted` — `crates/contextra-erasure/src/saga/machine.rs:43`
- `Pending` — `crates/contextra-erasure/src/saga/machine.rs:45`
- `Refused` — `crates/contextra-erasure/src/saga/machine.rs:47`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn is_terminal(self) -> bool` | `crates/contextra-erasure/src/saga/machine.rs:52` | Prüft ob dieser Zustand ein Endzustand ist. |
| `pub fn can_transition_to(self, next: SagaState) -> bool` | `crates/contextra-erasure/src/saga/machine.rs:57` | Prüft ob ein Übergang zu `next` erlaubt ist (SPEC §11.2, I-4, FW-9). |

### Modul `saga::retention`

#### `fn is_restrict_allowed`
- Ort: `crates/contextra-erasure/src/saga/retention.rs:14`
```rust
pub fn is_restrict_allowed(att: &RetentionAttestation) -> bool
```
Dokumentation: Prüft ob der Modus `Restrict` zulässig ist.

#### `fn is_retention_active`
- Ort: `crates/contextra-erasure/src/saga/retention.rs:6`
```rust
pub fn is_retention_active(att: &RetentionAttestation) -> bool
```
Dokumentation: Prüft ob die Attestierung eine aktive Aufbewahrungspflicht anzeigt.

### Modul `saga::seams`

#### `trait SagaGate`
- Ort: `crates/contextra-erasure/src/saga/seams.rs:43`
```rust
pub trait SagaGate: Send + Sync
```
Dokumentation: Naht für das Gate-Subsystem (SPEC §11.3, ER-11, ER-12, ER-13).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn add_gate<'a>( &'a self, erasure_id: ErasureId, target: &'a EraseTarget, ) -> BoxFuture<'a, Result<u64, ErasureError>>` | `crates/contextra-erasure/src/saga/seams.rs:46` | Trägt eine Lesesperre ein und persistiert das Gate-Log (fsync #2). Liefert die `gated_seq`. |
| `fn compact_gate<'a>(&'a self, erasure_id: ErasureId) -> BoxFuture<'a, Result<(), ErasureError>>` | `crates/contextra-erasure/src/saga/seams.rs:53` | Kompaktiert das Gate nach Abschluss der Löschung (SPEC §11.2 Schritt 6). |
| `fn decision_for_target<'a>( &'a self, target: &'a EraseTarget, ) -> BoxFuture<'a, Result<GateDecision, ErasureError>>` | `crates/contextra-erasure/src/saga/seams.rs:57` | Liefert die Gate-Entscheidung (`Erased`, `Restricted`, `Open`) für ein Ziel. |

#### `trait SagaJournal`
- Ort: `crates/contextra-erasure/src/saga/seams.rs:21`
```rust
pub trait SagaJournal: Send + Sync
```
Dokumentation: Naht für das Intent-Journal (SPEC §11.6, ER-29).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn append_intent<'a>( &'a self, erasure_id: ErasureId, target: &'a EraseTarget, attestation: &'a RetentionAttestation, ) -> BoxFuture<'a, Result<(), ErasureError>>` | `crates/contextra-erasure/src/saga/seams.rs:23` | Hängt den initialen `Requested+Intent`-Eintrag mit eingebettetem Umfang an (fsync #1). |
| `fn append_state<'a>( &'a self, erasure_id: ErasureId, state: SagaState, reason: Option<&'a PendingReason>, ) -> BoxFuture<'a, Result<(), ErasureError>>` | `crates/contextra-erasure/src/saga/seams.rs:31` | Hängt einen Zustandsübergang an. |
| `fn recover_open_sagas<'a>(&'a self) -> BoxFuture<'a, Result<Vec<EraseSaga>, ErasureError>>` | `crates/contextra-erasure/src/saga/seams.rs:39` | Rekonstruiert noch offene Sagas aus dem Journal. |

#### `trait SagaLedger`
- Ort: `crates/contextra-erasure/src/saga/seams.rs:109`
```rust
pub trait SagaLedger: Send + Sync
```
Dokumentation: Naht für das Ledger und Restore-Absicherung (SPEC §11.8, ER-30).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn record_completed<'a>( &'a self, erasure_id: ErasureId, target: &'a EraseTarget, gated_seq: u64, completed_at: u64, ) -> BoxFuture<'a, Result<(), ErasureError>>` | `crates/contextra-erasure/src/saga/seams.rs:111` | Schreibt den Eintrag nach `Completed` in das Ledger außerhalb des Datenverzeichnisses. |

#### `trait SagaMeasure`
- Ort: `crates/contextra-erasure/src/saga/seams.rs:87`
```rust
pub trait SagaMeasure: Send + Sync
```
Dokumentation: Naht für die Messung und Zeugenprüfung (SPEC §12, ER-25, ER-26).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn witness_layer<'a>( &'a self, layer_id: LayerId, target: &'a EraseTarget, ) -> BoxFuture<'a, Result<LayerWitness, ErasureError>>` | `crates/contextra-erasure/src/saga/seams.rs:89` | Führt die Positivkontrolle durch und verifiziert Residuen für eine Schicht. |

#### `trait SagaReport`
- Ort: `crates/contextra-erasure/src/saga/seams.rs:97`
```rust
pub trait SagaReport: Send + Sync
```
Dokumentation: Naht für die Berichterstellung und kryptografische Signatur (SPEC §11.7, ER-27).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn sign_report<'a>( &'a self, erasure_id: ErasureId, target: &'a EraseTarget, attestation: &'a RetentionAttestation, witnesses: &'a [LayerWitness], ) -> BoxFuture<'a, Result<Vec<u8>, ErasureError>>` | `crates/contextra-erasure/src/saga/seams.rs:99` | Erstellt den kanonischen, signierten Löschbericht. |

#### `trait SagaVault`
- Ort: `crates/contextra-erasure/src/saga/seams.rs:72`
```rust
pub trait SagaVault: Send + Sync
```
Dokumentation: Naht für den Schlüsseltresor (SPEC §11.5, ER-05, ER-06).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn destroy_highest_key<'a>( &'a self, target: &'a EraseTarget, ) -> BoxFuture<'a, Result<(), ErasureError>>` | `crates/contextra-erasure/src/saga/seams.rs:74` | Vernichtet den Slot der höchsten Ebene (`T`/`C`/`S`/`D`) und verifiziert das Nullen (fsync #3). |
| `fn verify_key_absent<'a>( &'a self, target: &'a EraseTarget, ) -> BoxFuture<'a, Result<bool, ErasureError>>` | `crates/contextra-erasure/src/saga/seams.rs:80` | Prüft lesend, dass der Schlüssel nicht mehr existiert. |

#### `trait SnapshotRevoker`
- Ort: `crates/contextra-erasure/src/saga/seams.rs:121`
```rust
pub trait SnapshotRevoker: Send + Sync
```
Dokumentation: Naht für den Snapshot-Widerruf (SPEC §11.2 Schritt 5a, D-61, L-21).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn revoke_covering<'a>( &'a self, target: &'a EraseTarget, gated_seq: u64, ) -> BoxFuture<'a, Result<u64, ErasureError>>` | `crates/contextra-erasure/src/saga/seams.rs:123` | Widerruft Snapshots des Mandanten für den betroffenen Bereich bis `gated_seq`. |

#### `fn query_target_gate`
- Ort: `crates/contextra-erasure/src/saga/seams.rs:64`
```rust
pub async fn query_target_gate( gate: &dyn SagaGate, target: &EraseTarget, ) -> Result<GateDecision, ErasureError>
```
Dokumentation: Öffentlicher Delegator für `decision_for_target`.

#### `type BoxFuture`
- Ort: `crates/contextra-erasure/src/saga/seams.rs:18`
```rust
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>
```
Dokumentation: Type-Alias für gepinnte ge-boxte Futures ohne Tokio/Futures-Util.

### Modul `scope`

#### `mod codec`
- Ort: `crates/contextra-erasure/src/scope.rs:5`
```rust
pub mod codec
```

#### `mod file`
- Ort: `crates/contextra-erasure/src/scope.rs:6`
```rust
pub mod file
```

#### `mod flight`
- Ort: `crates/contextra-erasure/src/scope.rs:7`
```rust
pub mod flight
```

#### `mod set`
- Ort: `crates/contextra-erasure/src/scope.rs:8`
```rust
pub mod set
```

#### `mod single_flight`
- Ort: `crates/contextra-erasure/src/scope.rs:9`
```rust
pub mod single_flight
```

### Modul `scope::codec`

#### `fn decode_scope`
- Ort: `crates/contextra-erasure/src/scope/codec.rs:62`
```rust
pub fn decode_scope(data: &[u8]) -> Result<LayerScopeSet, ErasureError>
```
Dokumentation: Deserialisiert einen `LayerScopeSet` aus dem festen Binärformat. Fail-Closed: Bei falschem Magic, unbekannter Version, Trunkierung, ungültigen Zählern oder Prüfsummen-Diskrepanz schlägt die Deserialisierung zwingend mit `ErasureError::Corrupt` fehl und gibt niemals still leere Daten zurück.

#### `fn encode_scope`
- Ort: `crates/contextra-erasure/src/scope/codec.rs:29`
```rust
pub fn encode_scope(scope: &LayerScopeSet) -> Result<Vec<u8>, ErasureError>
```
Dokumentation: Serialisiert einen `LayerScopeSet` in ein festes Binärformat mit Versionsfeld und Prüfsumme (Big-Endian).

### Modul `scope::file`

#### `fn load_scope_file`
- Ort: `crates/contextra-erasure/src/scope/file.rs:125`
```rust
pub async fn load_scope_file( vfs: &dyn Vfs, keyring: &dyn FileKeyring, erasure_id: &[u8
```
Dokumentation: Lädt einen `LayerScopeSet` aus einer persistenten Scope-Datei (kind=9). Fail-Closed: Jede Beschädigung (Header-MAC, Record-AEAD, Magic, Version) führt sofort zu `Err(ErasureError::Corrupt)` und niemals zu einer leeren Menge.

#### `fn scope_file_path`
- Ort: `crates/contextra-erasure/src/scope/file.rs:25`
```rust
pub fn scope_file_path(erasure_id: &[u8
```
Dokumentation: Ermittelt den relativen Pfad der Scope-Datei nach SPEC D-75: `scope/<erasure_id_hex>.scp`

#### `fn store_scope_file`
- Ort: `crates/contextra-erasure/src/scope/file.rs:36`
```rust
pub async fn store_scope_file( vfs: &dyn Vfs, keyring: &dyn FileKeyring, erasure_id: &[u8
```
Dokumentation: Speichert einen `LayerScopeSet` als persistente, verschlüsselte Scope-Datei (kind=9). Schreibt atomar und synchronisiert das übergeordnete Verzeichnis.

### Modul `scope::flight`

#### `struct ActiveScopeFlight`
- Ort: `crates/contextra-erasure/src/scope/flight.rs:14`
```rust
pub struct ActiveScopeFlight
```
Dokumentation: RAII-Token für einen aktiven Flug eines Scopes.
- Impl-Block (Trait `Drop`) `crates/contextra-erasure/src/scope/flight.rs:19`

#### `struct SingleFlightCoordinator`
- Ort: `crates/contextra-erasure/src/scope/flight.rs:43`
```rust
pub struct SingleFlightCoordinator
```
Dokumentation: Koordinator zur Durchsetzung von INV-ERA-3 (Single-Flight je Scope).
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/scope/flight.rs:47`
- Impl-Block `crates/contextra-erasure/src/scope/flight.rs:53`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/scope/flight.rs:55` | Erzeugt einen neuen `SingleFlightCoordinator`. |
| `pub fn try_acquire(&self, scope: KeyScope) -> Result<ActiveScopeFlight, ErasureError>` | `crates/contextra-erasure/src/scope/flight.rs:65` | Versucht, den exklusiven Flug für den gegebenen Scope zu reservieren. Liefert `Err(ErasureError::LimitExceeded(...))` falls bereits ein Flug aktiv ist. |
| `pub fn is_active(&self, scope: &KeyScope) -> bool` | `crates/contextra-erasure/src/scope/flight.rs:81` | Prüft, ob für den Scope derzeit ein Purge aktiv ist. |

### Modul `scope::set`

#### `struct LayerScopeSet`
- Ort: `crates/contextra-erasure/src/scope/set.rs:52`
```rust
pub struct LayerScopeSet
```
Dokumentation: Gesamtmenge der Scope-Auflösungen aller Schichten für eine Löschung.
- Impl-Block `crates/contextra-erasure/src/scope/set.rs:57`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `scopes` | `Vec<ResolvedLayerScope>` | `crates/contextra-erasure/src/scope/set.rs:54` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/scope/set.rs:59` | Erstellt eine leere Scope-Menge. |
| `pub fn push(&mut self, scope: ResolvedLayerScope)` | `crates/contextra-erasure/src/scope/set.rs:64` | Fügt einen Scope hinzu. |
| `pub fn total_units(&self) -> u64` | `crates/contextra-erasure/src/scope/set.rs:69` | Gesamtanzahl betroffener Einheiten über alle Schichten. |
| `pub fn is_embeddable(&self) -> bool` | `crates/contextra-erasure/src/scope/set.rs:74` | Prüft ob der gesamte Scope einbettbar ist (ER-17). |

#### `struct ResolvedLayerScope`
- Ort: `crates/contextra-erasure/src/scope/set.rs:13`
```rust
pub struct ResolvedLayerScope
```
Dokumentation: Auflösung einer einzelnen Schicht für eine Löschung (ER-17).
- Impl-Block `crates/contextra-erasure/src/scope/set.rs:22`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `layer_id` | `ScopeLayerId` | `crates/contextra-erasure/src/scope/set.rs:15` |
| `unit_count` | `u64` | `crates/contextra-erasure/src/scope/set.rs:17` |
| `units` | `Vec<u64>` | `crates/contextra-erasure/src/scope/set.rs:19` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn empty(layer_id: ScopeLayerId) -> Self` | `crates/contextra-erasure/src/scope/set.rs:24` | Erstellt einen leeren Scope für eine Schicht. |
| `pub fn with_units(layer_id: ScopeLayerId, units: Vec<u64>) -> Self` | `crates/contextra-erasure/src/scope/set.rs:33` | Erstellt einen Scope mit Einheiten. |
| `pub fn is_embeddable(&self) -> bool` | `crates/contextra-erasure/src/scope/set.rs:45` | Prüft ob der Scope klein genug ist um eingebettet zu werden (ER-17). Schwellwert: ≤ 16 Einheiten → Einbettung in Journal-Eintrag. |

#### `type ScopeLayerId`
- Ort: `crates/contextra-erasure/src/scope/set.rs:9`
```rust
pub type ScopeLayerId = [u8
```
Dokumentation: Kennung einer Schicht (für Scope-Zuordnung).

### Modul `scope::single_flight`

#### `struct ScopeLockGuard`
- Ort: `crates/contextra-erasure/src/scope/single_flight.rs:179`
```rust
pub struct ScopeLockGuard<'a>
```
Dokumentation: Single-Flight Guard für einen Scope (INV-ERA-3). Hält das Lock auf einen Scope solange aktiv, bis der Guard gedroppt oder explizit via [`ScopeLockGuard::release`] freigegeben wird.
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-erasure/src/scope/single_flight.rs:185`
- Impl-Block `crates/contextra-erasure/src/scope/single_flight.rs:194`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn epoch(&self) -> u64` | `crates/contextra-erasure/src/scope/single_flight.rs:196` | Gibt die aktuelle Sperr-Epoche zurück. |
| `pub fn owner_id(&self) -> &[u8` | `crates/contextra-erasure/src/scope/single_flight.rs:201` | Gibt die Besitzer-ID zurück. |
| `pub fn scope_id(&self) -> &[u8` | `crates/contextra-erasure/src/scope/single_flight.rs:206` | Gibt die Scope-ID zurück. |
| `pub fn release(self) -> Result<(), ScopeLockError>` | `crates/contextra-erasure/src/scope/single_flight.rs:211` | Gibt das Scope-Lock explizit frei. |

#### `struct ScopeLockManager`
- Ort: `crates/contextra-erasure/src/scope/single_flight.rs:238`
```rust
pub struct ScopeLockManager<'a>
```
Dokumentation: Verwaltet Single-Flight Scope-Locks (INV-ERA-3).
- Impl-Block `crates/contextra-erasure/src/scope/single_flight.rs:244`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(vfs: &'a dyn Vfs, clock: &'a dyn Clock, default_lease_nanos: u64) -> Self` | `crates/contextra-erasure/src/scope/single_flight.rs:246` | Erstellt einen neuen `ScopeLockManager`. |
| `pub fn acquire( &self, scope_id: [u8` | `crates/contextra-erasure/src/scope/single_flight.rs:262` | Versucht, ein exklusives Single-Flight Lock auf den angegebenen Scope zu erwerben. - Falls keine Sperre existiert: Erwirbt Lock mit Epoche 1. - Falls eine Sperr |
| `pub fn acquire_with_lease( &self, scope_id: [u8` | `crates/contextra-erasure/src/scope/single_flight.rs:271` | Versucht, ein exklusives Single-Flight Lock mit benutzerdefinierter Lease-Dauer zu erwerben. |
| `pub fn is_locked(&self, scope_id: &[u8` | `crates/contextra-erasure/src/scope/single_flight.rs:343` | Prüft, ob ein Scope aktuell gesperrt ist. |

#### `struct ScopeLockRecord`
- Ort: `crates/contextra-erasure/src/scope/single_flight.rs:60`
```rust
pub struct ScopeLockRecord
```
Dokumentation: Zustand eines Scope-Locks auf Platte oder im VFS.
- Impl-Block `crates/contextra-erasure/src/scope/single_flight.rs:73`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `scope_id` | `[u8; 16]` | `crates/contextra-erasure/src/scope/single_flight.rs:62` |
| `owner_id` | `[u8; 16]` | `crates/contextra-erasure/src/scope/single_flight.rs:64` |
| `epoch` | `u64` | `crates/contextra-erasure/src/scope/single_flight.rs:66` |
| `acquired_monotonic_nanos` | `u64` | `crates/contextra-erasure/src/scope/single_flight.rs:68` |
| `lease_nanos` | `u64` | `crates/contextra-erasure/src/scope/single_flight.rs:70` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn to_bytes(&self) -> Vec<u8>` | `crates/contextra-erasure/src/scope/single_flight.rs:77` | Kodiert den Lock-Record in ein festes Binärformat mit BLAKE3-Prüfsumme. |
| `pub fn from_bytes(bytes: &[u8]) -> Result<Self, ScopeLockError>` | `crates/contextra-erasure/src/scope/single_flight.rs:95` | Dekodiert und validiert den Lock-Record fail-closed. |
| `pub fn is_stale(&self, current_monotonic_nanos: u64) -> bool` | `crates/contextra-erasure/src/scope/single_flight.rs:161` | Prüft, ob das Lock relativ zur übergebenen Monotonic-Zeit abgelaufen (stale) ist. |

#### `enum ScopeLockError`
- Ort: `crates/contextra-erasure/src/scope/single_flight.rs:19`
```rust
pub enum ScopeLockError
```
Dokumentation: Fehler beim Erwerb oder der Freigabe eines Scope-Locks.
- Impl-Block (Trait `From<ErasureError>`) `crates/contextra-erasure/src/scope/single_flight.rs:49`

Varianten:

- `ScopeLocked{` — `crates/contextra-erasure/src/scope/single_flight.rs:22`
- `CorruptedLock` — `crates/contextra-erasure/src/scope/single_flight.rs:33`
- `Io(String)` — `crates/contextra-erasure/src/scope/single_flight.rs:37`
- `EpochMismatch{` — `crates/contextra-erasure/src/scope/single_flight.rs:41`

#### `fn scope_lock_path`
- Ort: `crates/contextra-erasure/src/scope/single_flight.rs:167`
```rust
pub fn scope_lock_path(scope_id: &[u8
```
Dokumentation: Pfad der Sperrdatei für einen Scope.

### Modul `supersede`

#### `struct PrefixSeqMatcher`
- Ort: `crates/contextra-erasure/src/supersede.rs:59`
```rust
pub struct PrefixSeqMatcher
```
Dokumentation: Matcher für das Prädikat `PrefixSeq` (SPEC D-117, F.9).
- Impl-Block `crates/contextra-erasure/src/supersede.rs:65`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(prefix: [u8` | `crates/contextra-erasure/src/supersede.rs:67` | Erstellt einen neuen Matcher mit 65-Byte-Präfix und Sequenzobergrenze (exklusiv). |
| `pub const fn prefix(&self) -> &[u8` | `crates/contextra-erasure/src/supersede.rs:73` | Liefert das 65-Byte-Präfix. |
| `pub const fn seq_lt(&self) -> u64` | `crates/contextra-erasure/src/supersede.rs:79` | Liefert die exklusive Sequenz-Obergrenze. |
| `pub fn matches(&self, candidate_prefix: &[u8` | `crates/contextra-erasure/src/supersede.rs:85` | Prüft, ob ein gegebener Schlüssel mit dem Präfix übereinstimmt und die Sequenz unter `seq_lt` liegt. |

#### `struct SupersedePlan`
- Ort: `crates/contextra-erasure/src/supersede.rs:92`
```rust
pub struct SupersedePlan
```
Dokumentation: Plan für die Berichtigung mit Purge alter Versionen (SPEC D-117).
- Impl-Block `crates/contextra-erasure/src/supersede.rs:99`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( erasure_id: ErasureId, prefix: [u8` | `crates/contextra-erasure/src/supersede.rs:101` | Erstellt einen neuen Plan für `Superseded`. |
| `pub fn erasure_id(&self) -> ErasureId` | `crates/contextra-erasure/src/supersede.rs:117` | Liefert die zugehörige Lösch-ID. |
| `pub fn intent_seq(&self) -> u64` | `crates/contextra-erasure/src/supersede.rs:123` | Liefert die Intent-Sequenz vor dem Update-Commit. |
| `pub fn matcher(&self) -> &PrefixSeqMatcher` | `crates/contextra-erasure/src/supersede.rs:129` | Liefert den Matcher für Gate und Purge. |
| `pub fn mode(&self) -> SupersedeMode` | `crates/contextra-erasure/src/supersede.rs:135` | Liefert den Berichtigungsmodus. |
| `pub fn verify_updated_seq(&self, latest_committed_seq: u64) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/supersede.rs:142` | Validiert, dass eine neue Version mit `seq > intent_seq` committet wurde (D-117). Liefert `ErasureError::UpdateMissing`, wenn die neue Version fehlt oder `seq < |
| `pub const fn is_key_destroyed(&self) -> bool` | `crates/contextra-erasure/src/supersede.rs:153` | Zeigt an, ob auf Schlüsselebene etwas vernichtet wurde. SPEC D-117: Bei Supersede bleibt der DEK gleich, kein Schlüssel wird vernichtet. |
| `pub const fn key_coverage_store(&self) -> KeyCoverage` | `crates/contextra-erasure/src/supersede.rs:159` | Schlüsselabdeckung der Speicherschicht (SPEC D-117: None / not_reached). |
| `pub const fn key_coverage_text(&self) -> KeyCoverage` | `crates/contextra-erasure/src/supersede.rs:165` | Schlüsselabdeckung des Textindex (SPEC D-117: None / not_reached). |
| `pub const fn key_coverage_vector(&self) -> KeyCoverage` | `crates/contextra-erasure/src/supersede.rs:171` | Schlüsselabdeckung des Vektorindex (SPEC D-117: None / not_reached). |

#### `enum SupersedeMode`
- Ort: `crates/contextra-erasure/src/supersede.rs:23`
```rust
pub enum SupersedeMode
```
Dokumentation: Modus der Berichtigung (SPEC D-117).

Varianten:

- `Keep` — `crates/contextra-erasure/src/supersede.rs:25`
- `PurgeOldVersions` — `crates/contextra-erasure/src/supersede.rs:27`

### Modul `supersede::prefix_serde`

#### `fn deserialize`
- Ort: `crates/contextra-erasure/src/supersede.rs:40`
```rust
pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8
```

#### `fn serialize`
- Ort: `crates/contextra-erasure/src/supersede.rs:33`
```rust
pub fn serialize<S>(prefix: &[u8
```

### Modul `test_mocks`

#### `struct MockErasureTarget`
- Ort: `crates/contextra-erasure/src/test_mocks.rs:55`
```rust
pub struct MockErasureTarget
```
Dokumentation: Mock für ein konkretes ErasureTarget.
- Impl-Block `crates/contextra-erasure/src/test_mocks.rs:62`
- Impl-Block (Trait `ErasureTarget`) `crates/contextra-erasure/src/test_mocks.rs:72`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra-erasure/src/test_mocks.rs:57` |
| `kind` | `&'static str` | `crates/contextra-erasure/src/test_mocks.rs:59` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(id: impl Into<String>, kind: &'static str) -> Self` | `crates/contextra-erasure/src/test_mocks.rs:64` | Erzeugt ein neues MockErasureTarget. |

#### `struct MockStorageBackend`
- Ort: `crates/contextra-erasure/src/test_mocks.rs:17`
```rust
pub struct MockStorageBackend
```
Dokumentation: In-Memory Mock für ein generisches Storage-Backend.
- Impl-Block `crates/contextra-erasure/src/test_mocks.rs:22`
- Impl-Block (Trait `StorageBackend`) `crates/contextra-erasure/src/test_mocks.rs:37`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(layer: LayerId) -> Self` | `crates/contextra-erasure/src/test_mocks.rs:24` | Erzeugt ein neues Mock-Backend für den gegebenen Layer. |
| `pub fn purge_count(&self) -> usize` | `crates/contextra-erasure/src/test_mocks.rs:32` | Liefert die Anzahl der aufgerufenen Purge-Operationen. |

#### `type BoxFuture`
- Ort: `crates/contextra-erasure/src/test_mocks.rs:11`
```rust
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>
```
Dokumentation: Type-Alias für BoxFuture ohne tokio/futures-util.

### Modul `types_local`

#### `struct Armed`
- Ort: `crates/contextra-erasure/src/types_local.rs:735`
```rust
pub struct Armed(pub ())
```
Dokumentation: Typzustand: Scanner ist scharfgeschaltet nach erfolgreicher Positivkontrolle.

#### `struct BarrierExclusive`
- Ort: `crates/contextra-erasure/src/types_local.rs:297`
```rust
pub struct BarrierExclusive
```
Dokumentation: RAII-Token für exklusive Gate-Barriere.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:301`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:308`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(token: Option<Arc<()>>) -> Self` | `crates/contextra-erasure/src/types_local.rs:303` | Erzeugt ein neues Exclusive-Token. |

#### `struct BarrierShared`
- Ort: `crates/contextra-erasure/src/types_local.rs:279`
```rust
pub struct BarrierShared
```
Dokumentation: RAII-Token für geteilte Commit-Barriere.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:283`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:290`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(token: Option<Arc<()>>) -> Self` | `crates/contextra-erasure/src/types_local.rs:285` | Erzeugt ein neues Shared-Token. |

#### `struct CollectionId`
- Ort: `crates/contextra-erasure/src/types_local.rs:1615`
```rust
pub struct CollectionId(pub u64)
```
- Impl-Block `crates/contextra-erasure/src/types_local.rs:1617`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-erasure/src/types_local.rs:1629`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-erasure/src/types_local.rs:1619` | Erzeugt eine neue `CollectionId`. |
| `pub const fn inner(self) -> u64` | `crates/contextra-erasure/src/types_local.rs:1624` | Liefert den internen `u64`-Wert. |

#### `struct CompactionPermit`
- Ort: `crates/contextra-erasure/src/types_local.rs:315`
```rust
pub struct CompactionPermit
```
Dokumentation: RAII-Token für Kompaktions-Erlaubnis.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:319`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:326`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(token: Option<Arc<()>>) -> Self` | `crates/contextra-erasure/src/types_local.rs:321` | Erzeugt ein neues CompactionPermit-Token. |

#### `struct ControlEvidence`
- Ort: `crates/contextra-erasure/src/types_local.rs:739`
```rust
pub struct ControlEvidence(pub ())
```
Dokumentation: Beleg für eine beobachtete Positivkontrolle (D-72).

#### `struct DocId`
- Ort: `crates/contextra-erasure/src/types_local.rs:1639`
```rust
pub struct DocId(pub u64)
```
- Impl-Block `crates/contextra-erasure/src/types_local.rs:1641`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-erasure/src/types_local.rs:1666`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-erasure/src/types_local.rs:1643` | Erzeugt eine neue `DocId`. |
| `pub const fn inner(self) -> u64` | `crates/contextra-erasure/src/types_local.rs:1648` | Liefert den internen `u64`-Wert. |
| `pub fn from_key(key: &str) -> Result<Self, crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:1653` | Erzeugt eine `DocId` aus einem Schlüssel mittels BLAKE3. |

#### `struct Envelope`
- Ort: `crates/contextra-erasure/src/types_local.rs:485`
```rust
pub struct Envelope
```
Dokumentation: Versiegelter Datensatz-Umschlag.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:489`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:501`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(payload: Vec<u8>) -> Self` | `crates/contextra-erasure/src/types_local.rs:491` | Erzeugt einen neuen Envelope. |
| `pub fn payload(&self) -> &[u8]` | `crates/contextra-erasure/src/types_local.rs:496` | Gibt eine Referenz auf die Payload zurück. |

#### `struct EraseReceipt`
- Ort: `crates/contextra-erasure/src/types_local.rs:1240`
```rust
pub struct EraseReceipt
```
Dokumentation: Löschquittung.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `ErasureId` | `crates/contextra-erasure/src/types_local.rs:1242` |
| `status` | `ErasureStatus` | `crates/contextra-erasure/src/types_local.rs:1244` |
| `gated_at` | `Timestamp` | `crates/contextra-erasure/src/types_local.rs:1246` |

#### `struct ErasureEvent`
- Ort: `crates/contextra-erasure/src/types_local.rs:1151`
```rust
pub struct ErasureEvent
```
Dokumentation: Ereignis für den ErasureObserver.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `ErasureId` | `crates/contextra-erasure/src/types_local.rs:1153` |
| `target_kind` | `TargetKind` | `crates/contextra-erasure/src/types_local.rs:1155` |
| `phase` | `ObservedPhase` | `crates/contextra-erasure/src/types_local.rs:1157` |
| `at` | `u64` | `crates/contextra-erasure/src/types_local.rs:1159` |

#### `struct ErasureId`
- Ort: `crates/contextra-erasure/src/types_local.rs:24`
```rust
pub struct ErasureId(pub [u8
```
Dokumentation: 16-Byte Lösch-Identifikator (SPEC §7.4, TY-09).
- Impl-Block `crates/contextra-erasure/src/types_local.rs:26`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:44`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-erasure/src/types_local.rs:54`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(bytes: [u8` | `crates/contextra-erasure/src/types_local.rs:28` | Erzeugt eine `ErasureId` aus rohen 16 Bytes. |
| `pub const fn as_bytes(&self) -> &[u8` | `crates/contextra-erasure/src/types_local.rs:33` | Liefert die 16 Bytes als Slice. |

#### `struct FencePermit`
- Ort: `crates/contextra-erasure/src/types_local.rs:2309`
```rust
pub struct FencePermit
```
Dokumentation: RAII-Token für einen aktiven Lese-/Schreib-Fence (INV-ERA-5).
- Impl-Block `crates/contextra-erasure/src/types_local.rs:2313`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:2320`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(token: Option<Arc<()>>) -> Self` | `crates/contextra-erasure/src/types_local.rs:2315` | Erzeugt ein neues FencePermit. |

#### `struct FileId`
- Ort: `crates/contextra-erasure/src/types_local.rs:395`
```rust
pub struct FileId(pub u64)
```
Dokumentation: Eindeutige Dateikennung.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:397`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-erasure/src/types_local.rs:399` | Erzeugt eine neue `FileId`. |
| `pub const fn as_u64(&self) -> u64` | `crates/contextra-erasure/src/types_local.rs:404` | Liefert die numerische Kennung. |

#### `struct FileKey`
- Ort: `crates/contextra-erasure/src/types_local.rs:425`
```rust
pub struct FileKey
```
Dokumentation: Flüchtiger Dateischlüssel mit Zeroize-on-Drop.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:430`
- Impl-Block (Trait `Drop`) `crates/contextra-erasure/src/types_local.rs:447`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:453`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(raw: Vec<u8>, file_id: FileId) -> Self` | `crates/contextra-erasure/src/types_local.rs:432` | Erzeugt einen neuen `FileKey`. |
| `pub fn as_bytes(&self) -> &[u8]` | `crates/contextra-erasure/src/types_local.rs:437` | Liefert Referenz auf die geschützten Bytes. |
| `pub fn file_id(&self) -> FileId` | `crates/contextra-erasure/src/types_local.rs:442` | Liefert die Dateikennung. |

#### `struct KeyBuf`
- Ort: `crates/contextra-erasure/src/types_local.rs:139`
```rust
pub struct KeyBuf(pub [u8
```
Dokumentation: Fester 16-Byte Schlüsselpuffer für Speicher- und Gate-Operationen (SPEC §8.1).
- Impl-Block (Trait `AsRef<[u8]>`) `crates/contextra-erasure/src/types_local.rs:38`
- Impl-Block `crates/contextra-erasure/src/types_local.rs:141`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:153`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(raw: [u8` | `crates/contextra-erasure/src/types_local.rs:143` | Erzeugt einen neuen `KeyBuf` aus rohen 16 Bytes. |
| `pub fn as_bytes(&self) -> &[u8` | `crates/contextra-erasure/src/types_local.rs:148` | Gibt eine Referenz auf die Bytes zurück. |

#### `struct KeyRange`
- Ort: `crates/contextra-erasure/src/types_local.rs:189`
```rust
pub struct KeyRange
```
Dokumentation: Halboffenes Schlüsselintervall `[start, end)`.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:196`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:208`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `start` | `KeyBuf` | `crates/contextra-erasure/src/types_local.rs:191` |
| `end` | `KeyBuf` | `crates/contextra-erasure/src/types_local.rs:193` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(start: KeyBuf, end: KeyBuf) -> Self` | `crates/contextra-erasure/src/types_local.rs:198` | Erzeugt einen neuen Schlüsselbereich. |
| `pub fn contains(&self, key: &KeyBuf) -> bool` | `crates/contextra-erasure/src/types_local.rs:203` | Prüft ob ein Schlüssel im Intervall liegt. |

#### `struct LayerId`
- Ort: `crates/contextra-erasure/src/types_local.rs:679`
```rust
pub struct LayerId(pub [u8
```
Dokumentation: Schicht-Kennung.

#### `struct LayerScope`
- Ort: `crates/contextra-erasure/src/types_local.rs:770`
```rust
pub struct LayerScope
```
Dokumentation: Schicht-Geltungsbereich vor dem Gating.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:775`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:800`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn empty() -> Self` | `crates/contextra-erasure/src/types_local.rs:777` | Erzeugt einen leeren Geltungsbereich. |
| `pub fn new(scope_id: [u8` | `crates/contextra-erasure/src/types_local.rs:784` | Erzeugt einen neuen Geltungsbereich. |
| `pub fn scope_id(&self) -> &[u8` | `crates/contextra-erasure/src/types_local.rs:791` | Liefert die Scope-ID. |
| `pub fn affected_units(&self) -> &[u64]` | `crates/contextra-erasure/src/types_local.rs:795` | Liefert die betroffenen Einheiten. |

#### `struct LayerWitness`
- Ort: `crates/contextra-erasure/src/types_local.rs:955`
```rust
pub struct LayerWitness
```
Dokumentation: Zeuge einer überprüften physischen Löschung.

#### `struct LedgerEntry`
- Ort: `crates/contextra-erasure/src/types_local.rs:573`
```rust
pub struct LedgerEntry
```
Dokumentation: Revisionssicherer Ledger-Eintrag.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `commitment` | `[u8; 32]` | `crates/contextra-erasure/src/types_local.rs:575` |
| `range_set` | `RangeSet` | `crates/contextra-erasure/src/types_local.rs:577` |
| `gated_seq` | `Seq` | `crates/contextra-erasure/src/types_local.rs:579` |
| `completed_at` | `Timestamp` | `crates/contextra-erasure/src/types_local.rs:581` |
| `chain_hash` | `[u8; 32]` | `crates/contextra-erasure/src/types_local.rs:583` |
| `ledger_mac` | `[u8; 32]` | `crates/contextra-erasure/src/types_local.rs:585` |

#### `struct MemoryVfsFile`
- Ort: `crates/contextra-erasure/src/types_local.rs:1418`
```rust
pub struct MemoryVfsFile
```
Dokumentation: In-Memory-VfsFile Adapter für In-Memory Vfs.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:1424`
- Impl-Block (Trait `VfsFile`) `crates/contextra-erasure/src/types_local.rs:1439`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( data: Vec<u8>, vfs_raw: std::sync::Arc<dyn crate::types_local::Vfs>, path: std::path::PathBuf, ) -> Self` | `crates/contextra-erasure/src/types_local.rs:1426` | Erzeugt ein MemoryVfsFile aus vorhandenen Bytes. |

#### `struct OperatorRef`
- Ort: `crates/contextra-erasure/src/types_local.rs:107`
```rust
pub struct OperatorRef(Vec<u8>)
```
Dokumentation: Opake Betreiberkennung für Attestierungen (SPEC §7.3).
- Impl-Block `crates/contextra-erasure/src/types_local.rs:109`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:127`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn try_new(bytes: Vec<u8>) -> Result<Self, String>` | `crates/contextra-erasure/src/types_local.rs:111` | Erzeugt eine geprüfte Instanz. |
| `pub fn as_bytes(&self) -> &[u8]` | `crates/contextra-erasure/src/types_local.rs:122` | Liefert die Bytes als Slice. |

#### `struct PurgeCx`
- Ort: `crates/contextra-erasure/src/types_local.rs:854`
```rust
pub struct PurgeCx
```
Dokumentation: Bereinigungs-Kontext mit Deadline.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `deadline_ms` | `u64` | `crates/contextra-erasure/src/types_local.rs:856` |

#### `struct PurgeOutcome`
- Ort: `crates/contextra-erasure/src/types_local.rs:835`
```rust
pub struct PurgeOutcome
```
Dokumentation: Bereinigungs-Ergebnis.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:842`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `purged_count` | `u64` | `crates/contextra-erasure/src/types_local.rs:837` |
| `bytes_reclaimed` | `u64` | `crates/contextra-erasure/src/types_local.rs:839` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(purged_count: u64, bytes_reclaimed: u64) -> Self` | `crates/contextra-erasure/src/types_local.rs:844` | Erzeugt ein neues PurgeOutcome. |

#### `struct PurgePermit`
- Ort: `crates/contextra-erasure/src/types_local.rs:333`
```rust
pub struct PurgePermit
```
Dokumentation: RAII-Token für exklusive Purge-Erlaubnis.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:337`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:344`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(token: Option<Arc<()>>) -> Self` | `crates/contextra-erasure/src/types_local.rs:339` | Erzeugt ein neues PurgePermit-Token. |

#### `struct PurgeStats`
- Ort: `crates/contextra-erasure/src/types_local.rs:1833`
```rust
pub struct PurgeStats
```
Dokumentation: Statistiken über bereinigte Einheiten.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `purged_units` | `u64` | `crates/contextra-erasure/src/types_local.rs:1835` |
| `bytes_freed` | `u64` | `crates/contextra-erasure/src/types_local.rs:1837` |

#### `struct RangeSet`
- Ort: `crates/contextra-erasure/src/types_local.rs:544`
```rust
pub struct RangeSet
```
Dokumentation: Intervall-Menge für Ledger-Einträge.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:548`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:565`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn empty() -> Self` | `crates/contextra-erasure/src/types_local.rs:550` | Erzeugt eine leere Bereichsmenge. |
| `pub fn new(ranges: Vec<(KeyBuf, KeyBuf)>) -> Self` | `crates/contextra-erasure/src/types_local.rs:555` | Erzeugt eine Bereichsmenge aus Intervallen. |
| `pub fn ranges(&self) -> &[(KeyBuf, KeyBuf)]` | `crates/contextra-erasure/src/types_local.rs:560` | Liefert die Bereiche. |

#### `struct ReportSignature`
- Ort: `crates/contextra-erasure/src/types_local.rs:1073`
```rust
pub struct ReportSignature
```
Dokumentation: Signatur eines DeletionReports (SPEC F.10).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `signer_id` | `SignerId` | `crates/contextra-erasure/src/types_local.rs:1075` |
| `signature_base64` | `String` | `crates/contextra-erasure/src/types_local.rs:1077` |
| `pub_fingerprint` | `String` | `crates/contextra-erasure/src/types_local.rs:1079` |

#### `struct RequestContext`
- Ort: `crates/contextra-erasure/src/types_local.rs:1698`
```rust
pub struct RequestContext
```
Dokumentation: RequestContext für Löschoperationen.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:1705`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tenant_id` | `TenantId` | `crates/contextra-erasure/src/types_local.rs:1700` |
| `tx_id` | `TxId` | `crates/contextra-erasure/src/types_local.rs:1702` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(tenant_id: TenantId, tx_id: TxId) -> Self` | `crates/contextra-erasure/src/types_local.rs:1707` | Erzeugt einen neuen `RequestContext`. |

#### `struct Residue`
- Ort: `crates/contextra-erasure/src/types_local.rs:751`
```rust
pub struct Residue
```
Dokumentation: Rückstandsbefund bei fehlgeschlagener Verifikation.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `matches` | `u64` | `crates/contextra-erasure/src/types_local.rs:753` |

#### `struct RetentionAttestation`
- Ort: `crates/contextra-erasure/src/types_local.rs:1229`
```rust
pub struct RetentionAttestation
```
Dokumentation: Attestierung zur Aufbewahrung.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `code` | `RetentionCode` | `crates/contextra-erasure/src/types_local.rs:1231` |
| `attested_by` | `OperatorRef` | `crates/contextra-erasure/src/types_local.rs:1233` |
| `attested_at` | `Timestamp` | `crates/contextra-erasure/src/types_local.rs:1235` |

#### `struct SagaRecord`
- Ort: `crates/contextra-erasure/src/types_local.rs:1714`
```rust
pub struct SagaRecord
```
Dokumentation: Saga record containing deletion tracking state and digests.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:1719`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `deleted_keys_hash` | `[u8; 32]` | `crates/contextra-erasure/src/types_local.rs:1716` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(deleted_keys_hash: [u8` | `crates/contextra-erasure/src/types_local.rs:1721` | Erzeugt einen neuen `SagaRecord`. |

#### `struct ScanSession`
- Ort: `crates/contextra-erasure/src/types_local.rs:1875`
```rust
pub struct ScanSession
```
Dokumentation: Laufende Scan-Session zur kontrollierten Erzeugung eines `ScanWitness`.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:1885`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn begin(layer: WitnessLayer, target: ScanTarget, technique: ScanTechnique) -> Self` | `crates/contextra-erasure/src/types_local.rs:1887` | Startet eine neue ScanSession. |
| `pub fn record_unit(&mut self, unit_hash: [u8` | `crates/contextra-erasure/src/types_local.rs:1900` | Registriert eine gescannte Einheit. |
| `pub fn record_control_match(&mut self)` | `crates/contextra-erasure/src/types_local.rs:1909` | Registriert eine bestandene Positivkontrolle (Control Match). |
| `pub fn finish(mut self, clock: &dyn Clock) -> Result<ScanWitness, crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:1914` | Schließt die Session ab und erzeugt den `ScanWitness` (gesperrter Mint-Pfad). |

#### `struct ScanTarget`
- Ort: `crates/contextra-erasure/src/types_local.rs:1814`
```rust
pub struct ScanTarget
```
Dokumentation: Zielbeschreibung des Scans inklusive Hash.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:1821`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `target_id` | `String` | `crates/contextra-erasure/src/types_local.rs:1816` |
| `target_hash` | `[u8; 32]` | `crates/contextra-erasure/src/types_local.rs:1818` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(target_id: impl Into<String>, target_hash: [u8` | `crates/contextra-erasure/src/types_local.rs:1823` | Erzeugt ein neues ScanTarget. |

#### `struct ScanWitness`
- Ort: `crates/contextra-erasure/src/types_local.rs:1963`
```rust
pub struct ScanWitness
```
Dokumentation: Verifiziertes Messergebnis eines Storage-Scanners (SPEC §12.7.4).
- Impl-Block `crates/contextra-erasure/src/types_local.rs:1973`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( layer: WitnessLayer, scanned: u64, remaining: u64, technique: ScanTechnique, digest: [u8` | `crates/contextra-erasure/src/types_local.rs:1975` | Erzeugt einen neuen `ScanWitness`. |
| `pub fn layer(&self) -> &str` | `crates/contextra-erasure/src/types_local.rs:2003` | Gibt die gescannte Storage-Schicht als &str zurück. |
| `pub fn witness_layer(&self) -> &WitnessLayer` | `crates/contextra-erasure/src/types_local.rs:2008` | Gibt die gescannte Storage-Schicht als &WitnessLayer zurück. |
| `pub fn scanned(&self) -> u64` | `crates/contextra-erasure/src/types_local.rs:2013` | Gibt die Anzahl der während der Verifikation gescannten Einheiten zurück. |
| `pub fn remaining(&self) -> u64` | `crates/contextra-erasure/src/types_local.rs:2018` | Gibt die Anzahl der verbleibenden Lebendeinträge zurück. |
| `pub fn technique(&self) -> &ScanTechnique` | `crates/contextra-erasure/src/types_local.rs:2023` | Gibt die angewandte Scantechnik zurück. |
| `pub fn method(&self) -> &ScanMethod` | `crates/contextra-erasure/src/types_local.rs:2028` | Gibt die ScanMethod-Entsprechung zurück. |
| `pub fn digest(&self) -> &[u8` | `crates/contextra-erasure/src/types_local.rs:2033` | Gibt den BLAKE3-Digest der Messung zurück. |
| `pub fn measured_at(&self) -> u64` | `crates/contextra-erasure/src/types_local.rs:2038` | Gibt den Zeitstempel der Messung (Unix Nanos) zurück. |

#### `struct Scanner`
- Ort: `crates/contextra-erasure/src/types_local.rs:867`
```rust
pub struct Scanner<State>
```
Dokumentation: Typisierter Scanner.

#### `struct SealCtx`
- Ort: `crates/contextra-erasure/src/types_local.rs:465`
```rust
pub struct SealCtx
```
Dokumentation: Versiegelungskontext.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:470`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:477`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `epoch` | `u64` | `crates/contextra-erasure/src/types_local.rs:467` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(epoch: u64) -> Self` | `crates/contextra-erasure/src/types_local.rs:472` | Erzeugt einen neuen Versiegelungskontext. |

#### `struct Seq`
- Ort: `crates/contextra-erasure/src/types_local.rs:65`
```rust
pub struct Seq(pub u64)
```
Dokumentation: Streng monoton steigende Sequenznummer (SPEC §8.7, I-12).
- Impl-Block `crates/contextra-erasure/src/types_local.rs:67`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(val: u64) -> Self` | `crates/contextra-erasure/src/types_local.rs:69` | Erzeugt eine neue Sequenznummer. |
| `pub const fn as_u64(&self) -> u64` | `crates/contextra-erasure/src/types_local.rs:74` | Liefert den numerischen Wert als u64. |

#### `struct SequentialIdGen`
- Ort: `crates/contextra-erasure/src/types_local.rs:2163`
```rust
pub struct SequentialIdGen
```
Dokumentation: A thread-safe sequential ID generator.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:2167`
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/types_local.rs:2176`
- Impl-Block (Trait `IdGen`) `crates/contextra-erasure/src/types_local.rs:2182`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(start: u64) -> Self` | `crates/contextra-erasure/src/types_local.rs:2169` | Creates a new [`SequentialIdGen`]. |

#### `struct SystemClock`
- Ort: `crates/contextra-erasure/src/types_local.rs:2104`
```rust
pub struct SystemClock
```
Dokumentation: Standard implementation of [`Clock`].
- Impl-Block `crates/contextra-erasure/src/types_local.rs:2108`
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/types_local.rs:2119`
- Impl-Block (Trait `Clock`) `crates/contextra-erasure/src/types_local.rs:2125`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/types_local.rs:2111` | Creates a new [`SystemClock`]. |

#### `struct TenantId`
- Ort: `crates/contextra-erasure/src/types_local.rs:1591`
```rust
pub struct TenantId(pub u64)
```
- Impl-Block `crates/contextra-erasure/src/types_local.rs:1593`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-erasure/src/types_local.rs:1605`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-erasure/src/types_local.rs:1595` | Erzeugt eine neue `TenantId`. |
| `pub const fn inner(self) -> u64` | `crates/contextra-erasure/src/types_local.rs:1600` | Liefert den internen `u64`-Wert. |

#### `struct Timestamp`
- Ort: `crates/contextra-erasure/src/types_local.rs:81`
```rust
pub struct Timestamp(pub u64)
```
Dokumentation: Diskreter Zeitstempel in Mikrosekunden oder Millisekunden seit Epoche.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:83`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(val: u64) -> Self` | `crates/contextra-erasure/src/types_local.rs:85` | Erzeugt einen neuen Zeitstempel. |
| `pub const fn from_micros(micros: u64) -> Self` | `crates/contextra-erasure/src/types_local.rs:90` | Erzeugt Zeitstempel aus Mikrosekunden. |
| `pub const fn as_u64(&self) -> u64` | `crates/contextra-erasure/src/types_local.rs:95` | Liefert den numerischen Wert. |
| `pub const fn as_micros(&self) -> u64` | `crates/contextra-erasure/src/types_local.rs:100` | Liefert Mikrosekunden. |

#### `struct TxId`
- Ort: `crates/contextra-erasure/src/types_local.rs:1676`
```rust
pub struct TxId(pub u64)
```
- Impl-Block `crates/contextra-erasure/src/types_local.rs:1678`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-erasure/src/types_local.rs:1690`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-erasure/src/types_local.rs:1680` | Erzeugt eine neue `TxId`. |
| `pub const fn inner(self) -> u64` | `crates/contextra-erasure/src/types_local.rs:1685` | Liefert den internen `u64`-Wert. |

#### `struct Unarmed`
- Ort: `crates/contextra-erasure/src/types_local.rs:731`
```rust
pub struct Unarmed
```
Dokumentation: Typzustand: Scanner ist noch nicht scharfgeschaltet.

#### `struct VfsOpenOptions`
- Ort: `crates/contextra-erasure/src/types_local.rs:1343`
```rust
pub struct VfsOpenOptions
```
Dokumentation: Optionen zum Öffnen von Dateien im VFS.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:1358`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `read` | `bool` | `crates/contextra-erasure/src/types_local.rs:1345` |
| `write` | `bool` | `crates/contextra-erasure/src/types_local.rs:1347` |
| `create` | `bool` | `crates/contextra-erasure/src/types_local.rs:1349` |
| `create_new` | `bool` | `crates/contextra-erasure/src/types_local.rs:1351` |
| `truncate` | `bool` | `crates/contextra-erasure/src/types_local.rs:1353` |
| `append` | `bool` | `crates/contextra-erasure/src/types_local.rs:1355` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new() -> Self` | `crates/contextra-erasure/src/types_local.rs:1360` | Erzeugt neue leere Optionen. |
| `pub fn read_only() -> Self` | `crates/contextra-erasure/src/types_local.rs:1371` | Konfiguriert Optionen für reinen Lesezugriff. |
| `pub fn write_create() -> Self` | `crates/contextra-erasure/src/types_local.rs:1378` | Konfiguriert Optionen für Schreibzugriff mit Dateierzeugung. |
| `pub const fn read(mut self, val: bool) -> Self` | `crates/contextra-erasure/src/types_local.rs:1386` | Setzt Leseberechtigung. |
| `pub const fn write(mut self, val: bool) -> Self` | `crates/contextra-erasure/src/types_local.rs:1391` | Setzt Schreibberechtigung. |
| `pub const fn create(mut self, val: bool) -> Self` | `crates/contextra-erasure/src/types_local.rs:1396` | Setzt Erstellungsflag. |
| `pub const fn create_new(mut self, val: bool) -> Self` | `crates/contextra-erasure/src/types_local.rs:1401` | Setzt Create-New-Flag. |

#### `enum EraseMode`
- Ort: `crates/contextra-erasure/src/types_local.rs:1194`
```rust
pub enum EraseMode
```
Dokumentation: Löschmodus.

Varianten:

- `Erase` — `crates/contextra-erasure/src/types_local.rs:1196`
- `Restrict` — `crates/contextra-erasure/src/types_local.rs:1198`

#### `enum EraseTarget`
- Ort: `crates/contextra-erasure/src/types_local.rs:812`
```rust
pub enum EraseTarget
```
Dokumentation: Lösch-Zielangabe.

Varianten:

- `KeyRange{` — `crates/contextra-erasure/src/types_local.rs:814`
- `ExactKeys(Vec<Vec<u8>>)` — `crates/contextra-erasure/src/types_local.rs:821`

#### `enum ErasureStatus`
- Ort: `crates/contextra-erasure/src/types_local.rs:1298`
```rust
pub enum ErasureStatus
```
Dokumentation: Status einer Lösch-Saga (SPEC §11.2, D-124).

Varianten:

- `Requested` — `crates/contextra-erasure/src/types_local.rs:1300`
- `Gated` — `crates/contextra-erasure/src/types_local.rs:1302`
- `KeyDestroyed` — `crates/contextra-erasure/src/types_local.rs:1304`
- `Purging` — `crates/contextra-erasure/src/types_local.rs:1306`
- `Verifying` — `crates/contextra-erasure/src/types_local.rs:1308`
- `Completed` — `crates/contextra-erasure/src/types_local.rs:1310`
- `Restricted` — `crates/contextra-erasure/src/types_local.rs:1312`
- `Lifted` — `crates/contextra-erasure/src/types_local.rs:1314`
- `Pending{` — `crates/contextra-erasure/src/types_local.rs:1316`
- `Refused(RefusalReason)` — `crates/contextra-erasure/src/types_local.rs:1323`

#### `enum FileKind`
- Ort: `crates/contextra-erasure/src/types_local.rs:411`
```rust
pub enum FileKind
```
Dokumentation: Dateityp für den FileKeyring.

Varianten:

- `SSTable` — `crates/contextra-erasure/src/types_local.rs:413`
- `Wal` — `crates/contextra-erasure/src/types_local.rs:415`
- `TextSegment` — `crates/contextra-erasure/src/types_local.rs:417`
- `VectorSegment` — `crates/contextra-erasure/src/types_local.rs:419`
- `Auxiliary` — `crates/contextra-erasure/src/types_local.rs:421`

#### `enum GateDecision`
- Ort: `crates/contextra-erasure/src/types_local.rs:216`
```rust
pub enum GateDecision
```
Dokumentation: Gate-Entscheidung für Punktabfragen oder Dateigrenzen (SPEC §4.2, §11.3).

Varianten:

- `Open` — `crates/contextra-erasure/src/types_local.rs:218`
- `Erased` — `crates/contextra-erasure/src/types_local.rs:220`
- `Restricted` — `crates/contextra-erasure/src/types_local.rs:222`

#### `enum KeyBound`
- Ort: `crates/contextra-erasure/src/types_local.rs:161`
```rust
pub enum KeyBound
```
Dokumentation: Begrenzungs-Schlüssel für Scans und Kompaktion.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:170`
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-erasure/src/types_local.rs:177`

Varianten:

- `Exact(KeyBuf)` — `crates/contextra-erasure/src/types_local.rs:163`
- `UnboundedLower` — `crates/contextra-erasure/src/types_local.rs:165`
- `UnboundedUpper` — `crates/contextra-erasure/src/types_local.rs:167`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn exact(key: KeyBuf) -> Self` | `crates/contextra-erasure/src/types_local.rs:172` | Erzeugt eine exakte Grenze. |

#### `enum KeyCoverage`
- Ort: `crates/contextra-erasure/src/types_local.rs:709`
```rust
pub enum KeyCoverage
```
Dokumentation: Reichweite der Schlüsselabdeckung einer Schicht (SPEC §8.2, D-69).

Varianten:

- `Full` — `crates/contextra-erasure/src/types_local.rs:711`
- `Unlinked` — `crates/contextra-erasure/src/types_local.rs:713`
- `None` — `crates/contextra-erasure/src/types_local.rs:715`

#### `enum KeyScope`
- Ort: `crates/contextra-erasure/src/types_local.rs:1842`
```rust
pub enum KeyScope
```
Dokumentation: Abstrakter Bezeichner für den zu bereinigenden Schlüsselbereich (SPEC v4.0 §6.2, B3).

Varianten:

- `Tenant(TenantId)` — `crates/contextra-erasure/src/types_local.rs:1844`
- `Collection{` — `crates/contextra-erasure/src/types_local.rs:1846`
- `Document{` — `crates/contextra-erasure/src/types_local.rs:1853`

#### `enum MeasureClass`
- Ort: `crates/contextra-erasure/src/types_local.rs:720`
```rust
pub enum MeasureClass
```
Dokumentation: Messklasse für Kontrollscans (SPEC §12.1).

Varianten:

- `Mk1` — `crates/contextra-erasure/src/types_local.rs:722`
- `Mk2` — `crates/contextra-erasure/src/types_local.rs:724`
- `Mk3` — `crates/contextra-erasure/src/types_local.rs:726`

#### `enum ObservedPhase`
- Ort: `crates/contextra-erasure/src/types_local.rs:1138`
```rust
pub enum ObservedPhase
```
Dokumentation: Beobachtete Lebenszyklus-Phase.

Varianten:

- `Gated` — `crates/contextra-erasure/src/types_local.rs:1140`
- `Completed` — `crates/contextra-erasure/src/types_local.rs:1142`
- `Restricted` — `crates/contextra-erasure/src/types_local.rs:1144`
- `Lifted` — `crates/contextra-erasure/src/types_local.rs:1146`

#### `enum ObserverError`
- Ort: `crates/contextra-erasure/src/types_local.rs:1164`
```rust
pub enum ObserverError
```
Dokumentation: Fehlerklasse für den Observer.

Varianten:

- `Transient(String)` — `crates/contextra-erasure/src/types_local.rs:1167`
- `Permanent(String)` — `crates/contextra-erasure/src/types_local.rs:1170`
- `Timeout` — `crates/contextra-erasure/src/types_local.rs:1173`

#### `enum PendingReason`
- Ort: `crates/contextra-erasure/src/types_local.rs:1251`
```rust
pub enum PendingReason
```
Dokumentation: Grund für Schwebezustand.

Varianten:

- `Io` — `crates/contextra-erasure/src/types_local.rs:1253`
- `SlaExceeded` — `crates/contextra-erasure/src/types_local.rs:1255`
- `LayerFailed(LayerId)` — `crates/contextra-erasure/src/types_local.rs:1257`
- `WitnessMissing` — `crates/contextra-erasure/src/types_local.rs:1259`
- `DiskReserve` — `crates/contextra-erasure/src/types_local.rs:1261`
- `Other` — `crates/contextra-erasure/src/types_local.rs:1263`

#### `enum Phase`
- Ort: `crates/contextra-erasure/src/types_local.rs:1277`
```rust
pub enum Phase
```
Dokumentation: Phase eines Löschablaufs (SPEC §7.4, D-124 Abs. 5).

Varianten:

- `Intent` — `crates/contextra-erasure/src/types_local.rs:1279`
- `Gate` — `crates/contextra-erasure/src/types_local.rs:1281`
- `KeyDestroy` — `crates/contextra-erasure/src/types_local.rs:1283`
- `Purge` — `crates/contextra-erasure/src/types_local.rs:1285`
- `Verify` — `crates/contextra-erasure/src/types_local.rs:1287`
- `Replace` — `crates/contextra-erasure/src/types_local.rs:1289`
- `Supersede` — `crates/contextra-erasure/src/types_local.rs:1291`
- `Restrict` — `crates/contextra-erasure/src/types_local.rs:1293`

#### `enum RangeGate`
- Ort: `crates/contextra-erasure/src/types_local.rs:227`
```rust
pub enum RangeGate
```
Dokumentation: Gate-Entscheidungsvariante für Bereichsoperationen (SPEC §4.2, I-18).

Varianten:

- `ContainsErased` — `crates/contextra-erasure/src/types_local.rs:229`
- `RestrictedNeverDiscard` — `crates/contextra-erasure/src/types_local.rs:231`
- `EntirelyOpen` — `crates/contextra-erasure/src/types_local.rs:233`

#### `enum RefusalReason`
- Ort: `crates/contextra-erasure/src/types_local.rs:1268`
```rust
pub enum RefusalReason
```
Dokumentation: Ablehnungsgrund vor Gate.

Varianten:

- `RetentionObligation` — `crates/contextra-erasure/src/types_local.rs:1270`
- `InvalidParameters` — `crates/contextra-erasure/src/types_local.rs:1272`

#### `enum RetentionCode`
- Ort: `crates/contextra-erasure/src/types_local.rs:1212`
```rust
pub enum RetentionCode
```
Dokumentation: Aufbewahrungscode.

Varianten:

- `NoObligation` — `crates/contextra-erasure/src/types_local.rs:1214`
- `ObligationExpired` — `crates/contextra-erasure/src/types_local.rs:1216`
- `ObligationActive` — `crates/contextra-erasure/src/types_local.rs:1218`
- `LegalClaimsPending` — `crates/contextra-erasure/src/types_local.rs:1220`
- `DataSubjectRequestedRestriction` — `crates/contextra-erasure/src/types_local.rs:1222`
- `OperatorOverride` — `crates/contextra-erasure/src/types_local.rs:1224`

#### `enum ScanMethod`
- Ort: `crates/contextra-erasure/src/types_local.rs:1803`
```rust
pub enum ScanMethod
```
Dokumentation: Scan-Methode für Berichts- und Nachweiszwecke.

Varianten:

- `Structural` — `crates/contextra-erasure/src/types_local.rs:1805`
- `NotMeasured(String)` — `crates/contextra-erasure/src/types_local.rs:1807`
- `FeatureDisabled` — `crates/contextra-erasure/src/types_local.rs:1809`

#### `enum ScanTechnique`
- Ort: `crates/contextra-erasure/src/types_local.rs:1786`
```rust
pub enum ScanTechnique
```
Dokumentation: Angewandte Methode/Technik während der Verifikation.

Varianten:

- `Structural` — `crates/contextra-erasure/src/types_local.rs:1788`
- `DecryptedView` — `crates/contextra-erasure/src/types_local.rs:1790`
- `IndexIntrospection` — `crates/contextra-erasure/src/types_local.rs:1792`
- `NeedleScan` — `crates/contextra-erasure/src/types_local.rs:1794`
- `NotMeasured(String)` — `crates/contextra-erasure/src/types_local.rs:1796`
- `FeatureDisabled` — `crates/contextra-erasure/src/types_local.rs:1798`

#### `enum SignerError`
- Ort: `crates/contextra-erasure/src/types_local.rs:1084`
```rust
pub enum SignerError
```
Dokumentation: Signierer-Fehlerklasse.

Varianten:

- `KeyUnavailable(String)` — `crates/contextra-erasure/src/types_local.rs:1087`
- `ExternalSignerFailed(String)` — `crates/contextra-erasure/src/types_local.rs:1090`
- `InvalidPayload(String)` — `crates/contextra-erasure/src/types_local.rs:1093`

#### `enum TargetKind`
- Ort: `crates/contextra-erasure/src/types_local.rs:1121`
```rust
pub enum TargetKind
```
Dokumentation: Ziel-Ebene für Observer-Ereignisse.

Varianten:

- `Document` — `crates/contextra-erasure/src/types_local.rs:1123`
- `Subject` — `crates/contextra-erasure/src/types_local.rs:1125`
- `Collection` — `crates/contextra-erasure/src/types_local.rs:1127`
- `Tenant` — `crates/contextra-erasure/src/types_local.rs:1129`
- `DocumentReplace` — `crates/contextra-erasure/src/types_local.rs:1131`
- `Superseded` — `crates/contextra-erasure/src/types_local.rs:1133`

#### `enum Timing`
- Ort: `crates/contextra-erasure/src/types_local.rs:1203`
```rust
pub enum Timing
```
Dokumentation: Timing der Löschung.

Varianten:

- `Now` — `crates/contextra-erasure/src/types_local.rs:1205`
- `Batched` — `crates/contextra-erasure/src/types_local.rs:1207`

#### `enum VerifiedVia`
- Ort: `crates/contextra-erasure/src/types_local.rs:1332`
```rust
pub enum VerifiedVia
```
Dokumentation: Verifikationsmodus beim Lesen von Medien (SPEC D-113).

Varianten:

- `Direct` — `crates/contextra-erasure/src/types_local.rs:1334`
- `CacheDropped` — `crates/contextra-erasure/src/types_local.rs:1336`
- `Cached` — `crates/contextra-erasure/src/types_local.rs:1338`

#### `enum WitnessLayer`
- Ort: `crates/contextra-erasure/src/types_local.rs:1728`
```rust
pub enum WitnessLayer
```
Dokumentation: Identifikator für die physische Storage-Schicht.
- Impl-Block `crates/contextra-erasure/src/types_local.rs:1747`
- Impl-Block (Trait `From<&str>`) `crates/contextra-erasure/src/types_local.rs:1763`
- Impl-Block (Trait `From<String>`) `crates/contextra-erasure/src/types_local.rs:1778`

Varianten:

- `LsmMemtable` — `crates/contextra-erasure/src/types_local.rs:1730`
- `SsTableAllLevels` — `crates/contextra-erasure/src/types_local.rs:1732`
- `HnswIndex` — `crates/contextra-erasure/src/types_local.rs:1734`
- `WalAllSegments` — `crates/contextra-erasure/src/types_local.rs:1736`
- `CsrGraph` — `crates/contextra-erasure/src/types_local.rs:1738`
- `KvCacheSegments` — `crates/contextra-erasure/src/types_local.rs:1740`
- `EmbeddingCache` — `crates/contextra-erasure/src/types_local.rs:1742`
- `Custom(String)` — `crates/contextra-erasure/src/types_local.rs:1744`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn as_str(&self) -> &str` | `crates/contextra-erasure/src/types_local.rs:1749` | Gibt den String-Namen der Schicht zurück. |

#### `trait Clock`
- Ort: `crates/contextra-erasure/src/types_local.rs:2094`
```rust
pub trait Clock: Send + Sync + 'static
```
Dokumentation: Port trait for time operations.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn now_unix_nanos(&self) -> u64` | `crates/contextra-erasure/src/types_local.rs:2096` | Returns the current wall-clock time as nanoseconds since the Unix epoch. |
| `fn monotonic_nanos(&self) -> u64` | `crates/contextra-erasure/src/types_local.rs:2099` | Returns nanoseconds elapsed since a monotonic baseline reference point. |

#### `trait CommitBarrier`
- Ort: `crates/contextra-erasure/src/types_local.rs:351`
```rust
pub trait CommitBarrier: Send + Sync
```
Dokumentation: CommitBarrier-Port (SPEC §11.3, D-57, D-78).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn shared(&self) -> BoxFuture<'_, BarrierShared>` | `crates/contextra-erasure/src/types_local.rs:353` | Bezieht geteilten Commit-Zugriff. |
| `fn exclusive(&self) -> BoxFuture<'_, BarrierExclusive>` | `crates/contextra-erasure/src/types_local.rs:355` | Bezieht exklusiven Gate-Zugriff. |

#### `trait ErasureLayer`
- Ort: `crates/contextra-erasure/src/types_local.rs:1045`
```rust
pub trait ErasureLayer: Send + Sync
```
Dokumentation: Schichtvertrag für Speicher- und Index-Teilnehmer (SPEC §8.2).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn id(&self) -> LayerId` | `crates/contextra-erasure/src/types_local.rs:1047` | Schichtkennung. |
| `fn resolve(&self, t: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>>` | `crates/contextra-erasure/src/types_local.rs:1049` | Löst Ziel vor dem Gating in den konkreten Bereich auf. |
| `fn on_gate(&self, s: &LayerScope) -> BoxFuture<'_, Result<(), String>>` | `crates/contextra-erasure/src/types_local.rs:1051` | Vorbereitungsmaßnahmen vor dem Gating. |
| `fn purge(&self, s: &LayerScope, cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, String>>` | `crates/contextra-erasure/src/types_local.rs:1053` | Führt physische Bereinigung durch. |
| `fn witness( &self, s: &LayerScope, sc: &Scanner<Armed>, ) -> BoxFuture<'_, Result<LayerWitness, String>>` | `crates/contextra-erasure/src/types_local.rs:1055` | Prüft Rückstände und erzeugt Zeugen. |
| `fn key_coverage(&self) -> KeyCoverage` | `crates/contextra-erasure/src/types_local.rs:1061` | Meldet die Schlüsselabdeckung. |

#### `trait ErasureObserver`
- Ort: `crates/contextra-erasure/src/types_local.rs:1177`
```rust
pub trait ErasureObserver: Send + Sync + 'static
```
Dokumentation: Port-Vertrag für Lösch-Ereignis-Benachrichtigungen.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn on_erased(&self, ev: ErasureEvent) -> BoxFuture<'static, Result<(), ObserverError>>` | `crates/contextra-erasure/src/types_local.rs:1179` | Benachrichtigt den Observer über ein Ereignis. |

#### `trait FenceGate`
- Ort: `crates/contextra-erasure/src/types_local.rs:2328`
```rust
pub trait FenceGate: Send + Sync
```
Dokumentation: Port-Vertrag für FenceGate (INV-ERA-5). Blockiert konkurrierende Schreiber und Lesezugriffe während eines physischen Purge.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn acquire_fence<'a>( &'a self, scope: &'a KeyScope, ) -> BoxFuture<'a, Result<FencePermit, crate::error::ErasureError>>` | `crates/contextra-erasure/src/types_local.rs:2330` | Aktiviert den Fence für einen bestimmten KeyScope und gibt ein RAII-Token zurück. |
| `fn is_fenced(&self, key: &KeyBuf) -> bool` | `crates/contextra-erasure/src/types_local.rs:2336` | Prüft synchron, ob ein Fence für den gegebenen Schlüssel oder Bereich aktiv ist. |

#### `trait FileKeyring`
- Ort: `crates/contextra-erasure/src/types_local.rs:529`
```rust
pub trait FileKeyring: Send + Sync
```
Dokumentation: Port-Vertrag für Dateischlüsselverwaltung.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn create(&self, kind: FileKind, id: FileId) -> BoxFuture<'_, Result<FileKey, String>>` | `crates/contextra-erasure/src/types_local.rs:531` | Erzeugt einen neuen Dateischlüssel. |
| `fn open(&self, id: FileId) -> BoxFuture<'_, Result<FileKey, String>>` | `crates/contextra-erasure/src/types_local.rs:533` | Öffnet einen existierenden Dateischlüssel. |
| `fn destroy(&self, id: FileId) -> BoxFuture<'_, Result<(), String>>` | `crates/contextra-erasure/src/types_local.rs:535` | Vernichtet den Dateischlüssel im Tresor dauerhaft. |

#### `trait IdGen`
- Ort: `crates/contextra-erasure/src/types_local.rs:2156`
```rust
pub trait IdGen: Send + Sync + 'static
```
Dokumentation: Port trait for unique ID generation.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn next_id(&self) -> u64` | `crates/contextra-erasure/src/types_local.rs:2158` | Generates and returns the next unique 64-bit unsigned identifier. |

#### `trait KeyStore`
- Ort: `crates/contextra-erasure/src/types_local.rs:2392`
```rust
pub trait KeyStore: Send + Sync
```
Dokumentation: Port-Vertrag für KeyStore (WP-K). Verwaltet Schlüsselhierarchien (KEK/DEK), Wrapping und unwiderrufliche Schlüsselvernichtung.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn destroy_scope_keys( &self, ctx: &RequestContext, scope: &KeyScope, ) -> Result<usize, crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:2394` | Vernichtet alle Schlüssel (DEKs / SubKeys) eines Scopes unwiderruflich. |
| `fn key_exists(&self, scope: &KeyScope) -> Result<bool, crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:2401` | Prüft, ob ein Schlüssel für die Entschlüsselung noch existiert. |

#### `trait LayerScanner`
- Ort: `crates/contextra-erasure/src/types_local.rs:2044`
```rust
pub trait LayerScanner: Send + Sync
```
Dokumentation: Trait für Storage-Scanner zur Durchführung von Restmengen-Messungen.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn layer(&self) -> WitnessLayer` | `crates/contextra-erasure/src/types_local.rs:2046` | Gibt die von diesem Scanner bediente Schicht zurück. |
| `fn scan_units( &self, session: &mut ScanSession, ctx: &RequestContext, ) -> Result<(), crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:2049` | Führt das Scannen von Einheiten auf der übergebenen `ScanSession` aus. |
| `fn scan( &self, ctx: &RequestContext, target: ScanTarget, technique: ScanTechnique, clock: &dyn Clock, ) -> Result<ScanWitness, crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:2056` | Default-Implementierung von `scan`. |

#### `trait LedgerSink`
- Ort: `crates/contextra-erasure/src/types_local.rs:664`
```rust
pub trait LedgerSink: Send + Sync
```
Dokumentation: Port-Vertrag für das Lösch-Ledger.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn append(&self, e: &LedgerEntry) -> Result<(), String>` | `crates/contextra-erasure/src/types_local.rs:666` | Hängt einen Eintrag an das Ledger an. |
| `fn read_all(&self) -> Result<Vec<LedgerEntry>, String>` | `crates/contextra-erasure/src/types_local.rs:668` | Liest alle Einträge aus. |
| `fn prune(&self, older_than: Timestamp) -> Result<usize, String>` | `crates/contextra-erasure/src/types_local.rs:670` | Bereinigt alte Einträge nach Aufbewahrungsfrist. |

#### `trait PurgeBarrier`
- Ort: `crates/contextra-erasure/src/types_local.rs:359`
```rust
pub trait PurgeBarrier: Send + Sync
```
Dokumentation: PurgeBarrier-Port (SPEC §11.3, §11.4).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn compaction(&self) -> BoxFuture<'_, CompactionPermit>` | `crates/contextra-erasure/src/types_local.rs:361` | Bezieht geteilte Kompaktions-Erlaubnis. |
| `fn purge(&self) -> BoxFuture<'_, PurgePermit>` | `crates/contextra-erasure/src/types_local.rs:363` | Bezieht exklusive Purge-Erlaubnis. |
| `fn purge_waiting(&self) -> bool` | `crates/contextra-erasure/src/types_local.rs:365` | Prüft ob ein exklusiver Purge wartet. |

#### `trait Purger`
- Ort: `crates/contextra-erasure/src/types_local.rs:1864`
```rust
pub trait Purger: Send + Sync
```
Dokumentation: Trait für physische Bereinigungsaktionen (Purger) einer Schicht.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn purge( &self, ctx: &RequestContext, target: &ScanTarget, ) -> Result<PurgeStats, crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:1866` | Führt die physische Bereinigung für das gegebene Ziel aus. |

#### `trait ReadGate`
- Ort: `crates/contextra-erasure/src/types_local.rs:237`
```rust
pub trait ReadGate: Send + Sync
```
Dokumentation: Synchroner ReadGate-Port (SPEC §4.2, §11.3, PO-07).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn check(&self, k: &KeyBuf) -> GateDecision` | `crates/contextra-erasure/src/types_local.rs:239` | Prüft den Status eines einzelnen Schlüssels. |
| `fn check_bound(&self, b: &KeyBound) -> GateDecision` | `crates/contextra-erasure/src/types_local.rs:241` | Prüft den Status einer Bereichsgrenze. |
| `fn variant_for_range(&self, r: &KeyRange) -> RangeGate` | `crates/contextra-erasure/src/types_local.rs:243` | Liefert die Bereichsentscheidung für Kompaktionen. |
| `fn gated_seq(&self, k: &KeyBuf) -> Option<Seq>` | `crates/contextra-erasure/src/types_local.rs:245` | Liefert die Gating-Sequenznummer, falls vorhanden. |
| `fn gate_epoch(&self) -> u64` | `crates/contextra-erasure/src/types_local.rs:247` | Liefert den monotonen Gate-Epochenzähler. |
| `fn covers(&self, k: &KeyBuf) -> bool` | `crates/contextra-erasure/src/types_local.rs:251` | Prüft ob ein Schlüssel durch ein Gate abgedeckt ist. |
| `fn check_raw(&self, _raw: &[u8` | `crates/contextra-erasure/src/types_local.rs:258` | Prüft Status eines kanonischen 78-Byte Schlüssels. |
| `fn covers_raw(&self, raw: &[u8` | `crates/contextra-erasure/src/types_local.rs:262` | Prüft ob ein 78-Byte Schlüssel abgedeckt ist. |
| `fn gated_seq_raw(&self, _raw: &[u8` | `crates/contextra-erasure/src/types_local.rs:269` | Liefert Sequenznummer eines 78-Byte Schlüssels. |

#### `trait RecordSealer`
- Ort: `crates/contextra-erasure/src/types_local.rs:508`
```rust
pub trait RecordSealer: Send + Sync
```
Dokumentation: Port-Vertrag für RecordSealer.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn seal<'a>( &'a self, ctx: &'a SealCtx, key: &'a KeyBuf, plain: &'a [u8], ) -> BoxFuture<'a, Result<Envelope, String>>` | `crates/contextra-erasure/src/types_local.rs:510` | Versiegelt Klartext. |
| `fn open<'a>(&'a self, key: &'a KeyBuf, env: &'a [u8]) -> BoxFuture<'a, Result<Vec<u8>, String>>` | `crates/contextra-erasure/src/types_local.rs:518` | Öffnet einen Envelope. |
| `fn evict(&self, scope: &KeyRange)` | `crates/contextra-erasure/src/types_local.rs:522` | Invalidiert Schlüssel aus dem Cache. |
| `fn make_slots_durable(&self) -> BoxFuture<'_, Result<(), String>>` | `crates/contextra-erasure/src/types_local.rs:525` | Erzwingt Slot-Dauerhaftigkeit. |

#### `trait ReportSigner`
- Ort: `crates/contextra-erasure/src/types_local.rs:1097`
```rust
pub trait ReportSigner: Send + Sync
```
Dokumentation: Port-Vertrag für Berichtssignierung.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn signer_id(&self) -> SignerId` | `crates/contextra-erasure/src/types_local.rs:1099` | Liefert die Signer-ID. |
| `fn sign_report<'a>( &'a self, canonical_report_bytes: &'a [u8], ) -> BoxFuture<'a, Result<ReportSignature, SignerError>>` | `crates/contextra-erasure/src/types_local.rs:1101` | Signiert kanonische Berichts-Bytes. |

#### `trait ResidueScanner`
- Ort: `crates/contextra-erasure/src/types_local.rs:2420`
```rust
pub trait ResidueScanner: Send + Sync
```
Dokumentation: Port-Vertrag für ResidueScanner (WP-12 / INV-CLASS-MEASURED-1). Untersucht Index-Artefakte (z.B. Vektorindex / Graph) auf unzulässige Restspuren.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn scan_residues( &self, ctx: &RequestContext, scope: &KeyScope, target: &ScanTarget, ) -> Result<u64, crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:2422` | Führt den Restspuren-Scan für einen Scope auf einem Ziel-Artefakt durch. |

#### `trait Vfs`
- Ort: `crates/contextra-erasure/src/types_local.rs:2195`
```rust
pub trait Vfs: Send + Sync + 'static
```
Dokumentation: Abstract object-safe Virtual File System interface for `contextra-erasure`.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn open( &self, _path: &std::path::Path, _opts: &VfsOpenOptions, ) -> Result<Box<dyn VfsFile>, crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:2197` | Opens a file according to the specified options. |
| `fn read_at( &self, path: &std::path::Path, offset: u64, buf: &mut [u8], ) -> Result<usize, crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:2209` | Reads data from byte offset `offset` into `buf` for file at `path`. |
| `fn read(&self, path: &std::path::Path) -> Result<Vec<u8>, crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:2221` | Reads the entire contents of a file at `path`. |
| `fn write_atomic( &self, path: &std::path::Path, data: &[u8], ) -> Result<(), crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:2224` | Atomically writes `data` to `path` with crash-consistency guarantees. |
| `fn create_dir_all(&self, path: &std::path::Path) -> Result<(), crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:2231` | Creates a directory and all parent directories along `path` if missing. |
| `fn remove_file(&self, path: &std::path::Path) -> Result<(), crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:2234` | Removes a file at `path`. |
| `fn exists(&self, path: &std::path::Path) -> Result<bool, crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:2237` | Returns `true` if a file or directory exists at `path`. |
| `fn len(&self, path: &std::path::Path) -> Result<u64, crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:2240` | Returns the length in bytes of the file at `path`. |
| `fn list_dir(&self, path: &std::path::Path) -> Result<Vec<String>, crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:2243` | Lists entry names inside directory `path`. |
| `fn sync_dir(&self, path: &std::path::Path) -> Result<(), crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:2246` | Flushes directory metadata to persistent storage. |

#### `trait VfsExt`
- Ort: `crates/contextra-erasure/src/types_local.rs:1408`
```rust
pub trait VfsExt
```
Dokumentation: Erweiterungstraits für `Vfs` zum Öffnen von positionellen `VfsFile` Handles.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn open_file( &self, path: &std::path::Path, opts: &VfsOpenOptions, ) -> Result<Box<dyn VfsFile>, crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:1410` | Öffnet eine Datei als VfsFile. |

#### `trait VfsFile`
- Ort: `crates/contextra-erasure/src/types_local.rs:1511`
```rust
pub trait VfsFile: Send + Sync
```
Dokumentation: Port-Vertrag für geöffnete VFS-Dateihandles.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>` | `crates/contextra-erasure/src/types_local.rs:1513` | Liest an einer bestimmten Position. |
| `fn read_at_verified( &self, buf: &mut [u8], offset: u64, ) -> std::io::Result<(usize, VerifiedVia)>` | `crates/contextra-erasure/src/types_local.rs:1515` | Verifiziertes Lesen unter Umgehung des Caches. |
| `fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize>` | `crates/contextra-erasure/src/types_local.rs:1524` | Schreibt an einer bestimmten Position. |
| `fn read_verified(&self, offset: u64, len: usize) -> std::io::Result<(Vec<u8>, VerifiedVia)>` | `crates/contextra-erasure/src/types_local.rs:1526` | Verifiziertes Lesen (SPEC D-113). |
| `fn sync_data(&self) -> std::io::Result<()>` | `crates/contextra-erasure/src/types_local.rs:1533` | Schreibt Daten dauerhaft auf Festplatte. |
| `fn sync_all(&self) -> std::io::Result<()>` | `crates/contextra-erasure/src/types_local.rs:1535` | Schreibt alle Daten und Metadaten dauerhaft. |
| `fn set_len(&self, _len: u64) -> std::io::Result<()>` | `crates/contextra-erasure/src/types_local.rs:1539` | Kürzt oder erweitert die Dateilänge. |
| `fn len(&self) -> std::io::Result<u64>` | `crates/contextra-erasure/src/types_local.rs:1543` | Ermittelt die Dateigröße. |
| `fn is_empty(&self) -> std::io::Result<bool>` | `crates/contextra-erasure/src/types_local.rs:1545` | Prüft ob die Datei leer ist. |

#### `trait WalSink`
- Ort: `crates/contextra-erasure/src/types_local.rs:2354`
```rust
pub trait WalSink: Send + Sync
```
Dokumentation: Port-Vertrag für WalSink (INV-ERA-6). Verwaltet WAL-Entsorgung, Verankerung und Unlesbarmachung von WAL-Einträgen eines gelöschten Scopes.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn purge_scope_wal( &self, ctx: &RequestContext, scope: &KeyScope, ) -> Result<u64, crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:2356` | Entsorgt oder überschreibt alle WAL-Segmente/Einträge, die zum Scope gehören. |
| `fn append_erasure_marker( &self, ctx: &RequestContext, erasure_id: &ErasureId, scope: &KeyScope, ) -> Result<(), crate::error::ErasureError>` | `crates/contextra-erasure/src/types_local.rs:2363` | Schreibt einen Commit-Marker zur dauerhaften Verankerung der WAL-Bereinigung. |

#### `fn open_vfs_file`
- Ort: `crates/contextra-erasure/src/types_local.rs:1551`
```rust
pub fn open_vfs_file( vfs: std::sync::Arc<dyn crate::types_local::Vfs>, path: &std::path::Path, opts: &VfsOpenOptions, ) -> Result<Box<dyn VfsFile>, crate::error::ErasureError>
```
Dokumentation: Öffnet eine Datei im VFS als positionelles `VfsFile` Handle (In-Memory-Simulation).

#### `type BoxFuture`
- Ort: `crates/contextra-erasure/src/types_local.rs:12`
```rust
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>
```
Dokumentation: Type-Alias für gepinnte ge-boxte Futures ohne Tokio/Futures-Util.

#### `type SignerId`
- Ort: `crates/contextra-erasure/src/types_local.rs:1069`
```rust
pub type SignerId = u8
```
Dokumentation: Signer-ID-Typalias.

### Modul `vault`

#### `mod durable`
- Ort: `crates/contextra-erasure/src/vault.rs:5`
```rust
pub mod durable
```

#### `mod keyring`
- Ort: `crates/contextra-erasure/src/vault.rs:6`
```rust
pub mod keyring
```

#### `mod open`
- Ort: `crates/contextra-erasure/src/vault.rs:7`
```rust
pub mod open
```

#### `mod rehash`
- Ort: `crates/contextra-erasure/src/vault.rs:8`
```rust
pub mod rehash
```

#### `mod scrub`
- Ort: `crates/contextra-erasure/src/vault.rs:9`
```rust
pub mod scrub
```

#### `mod sealer`
- Ort: `crates/contextra-erasure/src/vault.rs:10`
```rust
pub mod sealer
```

#### `mod slot`
- Ort: `crates/contextra-erasure/src/vault.rs:11`
```rust
pub mod slot
```

#### `mod table`
- Ort: `crates/contextra-erasure/src/vault.rs:12`
```rust
pub mod table
```

#### `struct FileVault`
- Ort: `crates/contextra-erasure/src/vault.rs:51`
```rust
pub struct FileVault
```
Dokumentation: 16-Shard Tresor nach SPEC D-111.
- Impl-Block `crates/contextra-erasure/src/vault.rs:59`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new_in_memory(total_capacity: usize) -> Self` | `crates/contextra-erasure/src/vault.rs:61` | Erstellt einen leeren In-Memory-Tresor mit 16 Shards und gegebener Gesamtkapazität. |
| `pub fn new_in_memory_with_key(total_capacity: usize, mac_key: [u8` | `crates/contextra-erasure/src/vault.rs:68` | Erstellt einen In-Memory-Tresor mit einem spezifischen MAC-Schlüssel. |
| `pub fn new_in_memory_with_id_and_key( total_capacity: usize, vault_id: [u8` | `crates/contextra-erasure/src/vault.rs:74` | Erstellt einen In-Memory-Tresor mit vault_id und MAC-Schlüssel. |
| `pub fn shard_file_name(shard_no: u8) -> String` | `crates/contextra-erasure/src/vault.rs:101` | Berechnet den Shard-Dateinamen `<nn>.vlt` für Shard-Index `0..15`. |
| `pub fn initial_shard_capacity(planned_docs: usize) -> usize` | `crates/contextra-erasure/src/vault.rs:107` | Berechnet die Anfangskapazität je Shard nach SPEC D-111 Abs. 7: `nextpow2(planned_docs * 1.3 / 16 / 0.5).max(1024)` |
| `pub fn create_dir( vfs: &Arc<dyn crate::types_local::Vfs>, vault_dir: &Path, vault_id: [u8` | `crates/contextra-erasure/src/vault.rs:120` | Erstellt einen neuen dauerhaften Tresor mit 16 Shards in `vault_dir` über VFS (SPEC D-111, D-112). |
| `pub fn open_dir( vfs: &Arc<dyn crate::types_local::Vfs>, vault_dir: &Path, expected_vault_id: &[u8` | `crates/contextra-erasure/src/vault.rs:168` | Öffnet einen bestehenden dauerhaften Tresor über VFS nach SPEC D-111 Abs. 8 / D-112: |
| `pub fn shard_index(&self, slot_type: SlotType, tag: &[u8` | `crates/contextra-erasure/src/vault.rs:201` | Ermittelt den zuständigen Shard nach SPEC D-111 Abs. 1. |
| `pub fn put_slot( &mut self, slot_type: SlotType, tag: [u8` | `crates/contextra-erasure/src/vault.rs:207` | Speichert einen Slot (ER-04, D-111, I-9). |
| `pub fn put_slot_with_gen( &mut self, slot_type: SlotType, tag: [u8` | `crates/contextra-erasure/src/vault.rs:217` | Speichert einen Slot mit explizitem Generationszähler (u48). |
| `pub fn get_slot( &self, slot_type: SlotType, tag: [u8` | `crates/contextra-erasure/src/vault.rs:239` | Liest einen Slot (ER-02, D-111). |
| `pub fn destroy_slot( &mut self, slot_type: SlotType, tag: [u8` | `crates/contextra-erasure/src/vault.rs:253` | Vernichtet einen Slot (D-111 Abs. 3, D-113, I-23): |
| `pub fn sync_all_shards(&self) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/vault.rs:267` | Führt Gruppen-Sync für alle Shards aus (D-111 Abs. 9, I-9). |
| `pub fn load_exceeded(&self) -> bool` | `crates/contextra-erasure/src/vault.rs:277` | Prüft ob die Lastgrenze in mindestens einem Shard überschritten ist (λ > 0,90). |
| `pub fn live_count(&self) -> usize` | `crates/contextra-erasure/src/vault.rs:285` | Gesamtzahl aktiver Slots über alle 16 Shards. |
| `pub fn tombstone_count(&self) -> usize` | `crates/contextra-erasure/src/vault.rs:296` | Gesamtzahl Grabsteine über alle 16 Shards. |
| `pub fn capacity(&self) -> usize` | `crates/contextra-erasure/src/vault.rs:307` | Gesamtkapazität über alle 16 Shards. |
| `pub fn is_poisoned(&self) -> bool` | `crates/contextra-erasure/src/vault.rs:318` | Prüft ob mindestens ein Shard vergiftet ist (D-55, I-10). |
| `pub fn shard(&self, index: usize) -> Option<&VaultTable>` | `crates/contextra-erasure/src/vault.rs:326` | Zugriff auf einzelnen Shard (In-Memory). |
| `pub fn shard_mut(&mut self, index: usize) -> Option<&mut VaultTable>` | `crates/contextra-erasure/src/vault.rs:334` | Veränderlicher Zugriff auf einzelnen Shard (In-Memory). |
| `pub fn durable_shard(&self, index: usize) -> Option<&DurableVaultShard>` | `crates/contextra-erasure/src/vault.rs:342` | Zugriff auf dauerhaften Shard. |
| `pub fn durable_shard_mut(&mut self, index: usize) -> Option<&mut DurableVaultShard>` | `crates/contextra-erasure/src/vault.rs:350` | Veränderlicher Zugriff auf dauerhaften Shard. |
| `pub fn replace_durable_shard(&mut self, index: usize, shard: DurableVaultShard)` | `crates/contextra-erasure/src/vault.rs:358` | Ersetzt einen dauerhaften Shard (z.B. nach Rehash). |
| `pub fn vault_id(&self) -> &[u8` | `crates/contextra-erasure/src/vault.rs:365` | Vault-ID. |
| `pub fn mac_key(&self) -> &[u8` | `crates/contextra-erasure/src/vault.rs:370` | MAC-Schlüssel. |
| `pub fn vault_dir(&self) -> Option<&Path>` | `crates/contextra-erasure/src/vault.rs:375` | Vault-Verzeichnis (falls dauerhaft). |
| `pub fn vfs(&self) -> Option<&Arc<dyn crate::types_local::Vfs>>` | `crates/contextra-erasure/src/vault.rs:380` | Vfs-Instanz (falls dauerhaft). |

#### `enum ShardBackend`
- Ort: `crates/contextra-erasure/src/vault.rs:43`
```rust
pub enum ShardBackend
```
Dokumentation: Backend-Speicher eines Shards (entweder persistent über VFS oder im Arbeitsspeicher).

Varianten:

- `InMemory(VaultTable)` — `crates/contextra-erasure/src/vault.rs:45`
- `Durable(DurableVaultShard)` — `crates/contextra-erasure/src/vault.rs:47`

#### `const VAULT_NUM_SHARDS`
- Ort: `crates/contextra-erasure/src/vault.rs:40`
```rust
pub const VAULT_NUM_SHARDS: usize = 16
```
Dokumentation: Anzahl Shards im Tresor nach SPEC D-111 Abs. 1.

### Modul `vault::durable`

#### `struct DurableVaultShard`
- Ort: `crates/contextra-erasure/src/vault/durable.rs:22`
```rust
pub struct DurableVaultShard
```
Dokumentation: Dauerhafter Tresor-Shard über VFS.
- Impl-Block `crates/contextra-erasure/src/vault/durable.rs:37`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn create_file( vfs: &Arc<dyn Vfs>, path: PathBuf, shard_no: u8, capacity: usize, vault_id: [u8` | `crates/contextra-erasure/src/vault/durable.rs:39` | Erstellt eine neue dauerhafte Shard-Datei über VFS. |
| `pub fn open_file( vfs: &Arc<dyn Vfs>, path: PathBuf, expected_shard_no: u8, expected_vault_id: &[u8` | `crates/contextra-erasure/src/vault/durable.rs:112` | Öffnet eine bestehende dauerhafte Shard-Datei über VFS. |
| `pub fn is_poisoned(&self) -> bool` | `crates/contextra-erasure/src/vault/durable.rs:239` | Prüft ob der Shard vergiftet ist (D-55, I-10). |
| `pub fn poison(&self)` | `crates/contextra-erasure/src/vault/durable.rs:244` | Vergiftet den Shard irreversibel (D-55, I-10). |
| `pub fn path(&self) -> &Path` | `crates/contextra-erasure/src/vault/durable.rs:260` | Shard-Pfad. |
| `pub fn set_path(&mut self, path: PathBuf)` | `crates/contextra-erasure/src/vault/durable.rs:265` | Setzt den Pfad. |
| `pub fn shard_no(&self) -> u8` | `crates/contextra-erasure/src/vault/durable.rs:270` | Shard-Nummer (0..15). |
| `pub fn capacity(&self) -> usize` | `crates/contextra-erasure/src/vault/durable.rs:275` | Shard-Kapazität. |
| `pub fn live_count(&self) -> usize` | `crates/contextra-erasure/src/vault/durable.rs:280` | Geschätzte Live-Anzahl laut Metablock. |
| `pub fn tombstone_count(&self) -> usize` | `crates/contextra-erasure/src/vault/durable.rs:285` | Geschätzte Tombstone-Anzahl laut Metablock. |
| `pub fn generation(&self) -> u64` | `crates/contextra-erasure/src/vault/durable.rs:290` | Shard-Generation. |
| `pub fn set_generation(&mut self, gen: u64)` | `crates/contextra-erasure/src/vault/durable.rs:295` | Setzt die Shard-Generation. |
| `pub fn rsv_cursor(&self) -> u64` | `crates/contextra-erasure/src/vault/durable.rs:300` | Scrub-Cursor (`rsv`). |
| `pub fn set_rsv_cursor(&mut self, cursor: u64)` | `crates/contextra-erasure/src/vault/durable.rs:305` | Setzt den Scrub-Cursor (`rsv`). |
| `pub fn vault_id(&self) -> &[u8` | `crates/contextra-erasure/src/vault/durable.rs:310` | Vault-ID Referenz. |
| `pub fn mac_key_ref(&self) -> &[u8` | `crates/contextra-erasure/src/vault/durable.rs:315` | MAC-Schlüssel Referenz. |
| `pub fn load_factor(&self) -> f64` | `crates/contextra-erasure/src/vault/durable.rs:320` | Aktueller Lastfaktor λ = (live + tombstones) / capacity. |
| `pub fn should_background_rehash(&self) -> bool` | `crates/contextra-erasure/src/vault/durable.rs:328` | Prüft ob Rehash im Hintergrund ausgelöst werden soll (λ > 0,60). |
| `pub fn must_force_rehash(&self) -> bool` | `crates/contextra-erasure/src/vault/durable.rs:333` | Prüft ob Rehash erzwungen werden muss (λ > 0,80). |
| `pub fn load_exceeded(&self) -> bool` | `crates/contextra-erasure/src/vault/durable.rs:338` | Prüft ob Lastfaktor mit nächstem Insert > 0,90 wäre (D-111 Abs. 4). |
| `pub fn read_slot_raw_pub(&self, idx: usize) -> Result<[u8` | `crates/contextra-erasure/src/vault/durable.rs:347` | Liest rohe 64-Byte Slot-Bytes an Index `idx`. |
| `pub fn find(&self, slot_type: SlotType, tag: &[u8` | `crates/contextra-erasure/src/vault/durable.rs:372` | Findet den Slot-Index für (slot_type, tag) durch lineare Sondierung. |
| `pub fn get( &self, slot_type: SlotType, tag: &[u8` | `crates/contextra-erasure/src/vault/durable.rs:401` | Liest und decodiert einen Slot. |
| `pub fn put(&mut self, sd: SlotData) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/vault/durable.rs:417` | Schreibt einen Slot dauerhaft via VFS (genau ein 64-Byte-Schreibvorgang, I-23). |
| `pub fn destroy( &mut self, slot_type: SlotType, tag: &[u8` | `crates/contextra-erasure/src/vault/durable.rs:480` | Vernichtet einen Slot nach SPEC D-111 Abs. 3, D-113 und I-23: 1. Schreibt in EINEM 64-Byte-Schreibvorgang das Grabstein-Abbild (tombstone_image) 2. sync_data 3. |
| `pub fn flush_meta(&self) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/vault/durable.rs:582` | Aktualisiert den Metablock auf Platte. |
| `pub fn sync_data(&self) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/vault/durable.rs:603` | Synchronisiert die Datenblöcke der Datei (`sync_data`). Bei Fehler: Shard wird vergiftet (D-55, I-10). |
| `pub fn collect_active_slots(&self) -> Result<Vec<SlotData>, ErasureError>` | `crates/contextra-erasure/src/vault/durable.rs:613` | Sammelt alle aktiven Slots für den Rehash (Grabsteine werden nicht übernommen, D-111 Abs. 5). |
| `pub fn overwrite_with_zeros(&mut self) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/vault/durable.rs:628` | Überschreibt die Shard-Datei über das gehaltene Handle vollständig mit Nullen (D-111 Abs. 5). |

#### `const VAULT_SLOT_BASE`
- Ort: `crates/contextra-erasure/src/vault/durable.rs:19`
```rust
pub const VAULT_SLOT_BASE: usize = 128
```
Dokumentation: Basis-Offset der Slots in einer Shard-Datei: 64 Bytes F.1-Kopf + 64 Bytes Metablock = Offset 128 (SPEC F.3).

### Modul `vault::key_store`

#### `struct GroupEpochKey`
- Ort: `crates/contextra-erasure/src/vault/key_store.rs:22`
```rust
pub struct GroupEpochKey
```
Dokumentation: Schlüsselbereich für Gruppen und Epochen (Entscheidung E5), falls Feature vault nicht aktiv ist.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `group_id` | `u64` | `crates/contextra-erasure/src/vault/key_store.rs:24` |
| `epoch` | `u64` | `crates/contextra-erasure/src/vault/key_store.rs:26` |

#### `struct KeyPurgeReceipt`
- Ort: `crates/contextra-erasure/src/vault/key_store.rs:41`
```rust
pub struct KeyPurgeReceipt
```
Dokumentation: Vernichtungsergebnis nach `purge_keys` zur Verankerung im Ledger durch den Orchestrator. Herkunft: ffeb79555:crates/contextra-crypto/src/scope_keyring.rs:220 (DestructionReceipt)

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `group_id` | `u64` | `crates/contextra-erasure/src/vault/key_store.rs:43` |
| `epoch` | `u64` | `crates/contextra-erasure/src/vault/key_store.rs:45` |
| `affected_deks` | `usize` | `crates/contextra-erasure/src/vault/key_store.rs:47` |
| `purged_at_nanos` | `u64` | `crates/contextra-erasure/src/vault/key_store.rs:49` |

#### `struct LocalDek`
- Ort: `crates/contextra-erasure/src/vault/key_store.rs:55`
```rust
pub struct LocalDek(pub [u8
```
Dokumentation: Stark typisierter Daten-Schlüssel (DEK) mit automatischer Nullung im Speicher. Herkunft: 1364a54d5:crates/contextra-crypto/src/hierarchy.rs:16 (MasterKey)
- Impl-Block `crates/contextra-erasure/src/vault/key_store.rs:57`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-erasure/src/vault/key_store.rs:69`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(bytes: [u8` | `crates/contextra-erasure/src/vault/key_store.rs:59` | Erzeugt einen neuen `LocalDek` aus 32 Byte Schlüsselmaterial. |
| `pub fn as_bytes(&self) -> &[u8` | `crates/contextra-erasure/src/vault/key_store.rs:64` | Zugriff auf die rohen Schlüssel-Bytes als Referenz. |

#### `enum KeyStoreError`
- Ort: `crates/contextra-erasure/src/vault/key_store.rs:79`
```rust
pub enum KeyStoreError
```
Dokumentation: Lokale Fehlertaxonomie für den synchronen `KeyStore`-Port.

Varianten:

- `NotFound{` — `crates/contextra-erasure/src/vault/key_store.rs:82`
- `KeyDestroyed{` — `crates/contextra-erasure/src/vault/key_store.rs:90`
- `Crypto(String)` — `crates/contextra-erasure/src/vault/key_store.rs:98`
- `InvalidInput(String)` — `crates/contextra-erasure/src/vault/key_store.rs:101`

#### `trait KeyStore`
- Ort: `crates/contextra-erasure/src/vault/key_store.rs:108`
```rust
pub trait KeyStore: Send + Sync
```
Dokumentation: Synchroner Port-Trait für Schlüsselverwaltung und Vernichtung (WP-K). Folgt der synchronen Kernregel: Keine Async-Runtime im Löschpfad. Zeit und Zufall werden ausschließlich über Ports (`Clock`) injiziert.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn get_or_derive_kek(&self, scope: GroupEpochKey) -> Result<[u8` | `crates/contextra-erasure/src/vault/key_store.rs:110` | Erzeugt oder lädt den Gruppen-KEK für einen Bereich `(group_id, epoch)`. |
| `fn encrypt_record( &self, scope: GroupEpochKey, record_id: u64, plaintext: &[u8], ) -> Result<EncryptedEpochRecordPayload, KeyStoreError>` | `crates/contextra-erasure/src/vault/key_store.rs:114` | Verschlüsselt und wrappt einen Datensatz unter Envelope Encryption. |
| `fn decrypt_record( &self, payload: &EncryptedEpochRecordPayload, ) -> Result<Vec<u8>, KeyStoreError>` | `crates/contextra-erasure/src/vault/key_store.rs:123` | Entschlüsselt einen Datensatz unter Envelope Encryption. |
| `fn is_active(&self, scope: GroupEpochKey) -> bool` | `crates/contextra-erasure/src/vault/key_store.rs:129` | Prüft ob ein Schlüsselbereich aktiv (nicht widerrufen) ist. |
| `fn purge_keys( &self, scope: GroupEpochKey, clock: Option<&dyn Clock>, ) -> Result<KeyPurgeReceipt, KeyStoreError>` | `crates/contextra-erasure/src/vault/key_store.rs:136` | Vernichtet alle Schlüssel eines Bereichs unwiderruflich (`purge_keys`). Nullt das Schlüsselmaterial im RAM mit Zeroize, markiert die Epoche als vernichtet und l |

### Modul `vault::keyring`

#### `mod key_store`
- Ort: `crates/contextra-erasure/src/vault/keyring.rs:157`
```rust
pub mod key_store
```

#### `struct VaultKeyring`
- Ort: `crates/contextra-erasure/src/vault/keyring.rs:59`
```rust
pub struct VaultKeyring
```
Dokumentation: Tresor-gestützte `FileKeyring`-Implementierung nach SPEC D-85 / PO-06.
- Impl-Block `crates/contextra-erasure/src/vault/keyring.rs:64`
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/vault/keyring.rs:83`
- Impl-Block (Trait `FileKeyring`) `crates/contextra-erasure/src/vault/keyring.rs:89`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/vault/keyring.rs:66` | Erstellt einen neuen `VaultKeyring` mit eigener In-Memory-Tresor-Instanz. |
| `pub fn with_vault(vault: Arc<RwLock<FileVault>>, k_enc: [u8` | `crates/contextra-erasure/src/vault/keyring.rs:71` | Erstellt einen neuen `VaultKeyring` über einen bestehenden Tresor. |
| `pub fn new_in_memory() -> Self` | `crates/contextra-erasure/src/vault/keyring.rs:76` | Erstellt einen neuen In-Memory-Keyring mit eigener Tresor-Instanz. |

#### `fn file_id_to_tag`
- Ort: `crates/contextra-erasure/src/vault/keyring.rs:23`
```rust
pub fn file_id_to_tag(id: FileId) -> [u8
```
Dokumentation: Erzeugt das 16-Byte-Tag für eine `FileId` nach SPEC F.3.

#### `fn mask_file_key`
- Ort: `crates/contextra-erasure/src/vault/keyring.rs:33`
```rust
pub fn mask_file_key(raw_key: &[u8
```
Dokumentation: Maskiert oder demaskiert den Dateischlüssel mit HKDF-Expand nach SPEC F.3 / D-85.

#### `type BoxFuture`
- Ort: `crates/contextra-erasure/src/vault/keyring.rs:18`
```rust
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>
```
Dokumentation: Type-Alias für gepinnte ge-boxte Futures ohne Tokio/Futures-Util.

#### `type InMemoryKeyring`
- Ort: `crates/contextra-erasure/src/vault/keyring.rs:151`
```rust
pub type InMemoryKeyring = VaultKeyring
```
Dokumentation: In-Memory-Keyring für Tests und abwärtskompatible Validierung.

### Modul `vault::open`

#### `struct MetablockParams`
- Ort: `crates/contextra-erasure/src/vault/open.rs:62`
```rust
pub struct MetablockParams<'a>
```
Dokumentation: Parameter für einen Metablock (64 Bytes) nach SPEC F.3.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `vault_id` | `&'a [u8; 16]` | `crates/contextra-erasure/src/vault/open.rs:64` |
| `shard_no` | `u8` | `crates/contextra-erasure/src/vault/open.rs:66` |
| `cap_log2` | `u8` | `crates/contextra-erasure/src/vault/open.rs:68` |
| `live` | `u64` | `crates/contextra-erasure/src/vault/open.rs:70` |
| `tombstones` | `u64` | `crates/contextra-erasure/src/vault/open.rs:72` |
| `generation` | `u64` | `crates/contextra-erasure/src/vault/open.rs:74` |
| `scrub_cursor` | `u64` | `crates/contextra-erasure/src/vault/open.rs:76` |
| `mac_key` | `&'a [u8; 32]` | `crates/contextra-erasure/src/vault/open.rs:78` |

#### `struct VaultOpenOutcome`
- Ort: `crates/contextra-erasure/src/vault/open.rs:29`
```rust
pub struct VaultOpenOutcome
```
Dokumentation: Ergebnis der Open-Prüfung für einen Tresor-Shard (D-111 Abs. 8).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `vault_id` | `[u8; 16]` | `crates/contextra-erasure/src/vault/open.rs:31` |
| `shard_no` | `u8` | `crates/contextra-erasure/src/vault/open.rs:33` |
| `capacity` | `usize` | `crates/contextra-erasure/src/vault/open.rs:35` |
| `live_estimate` | `u64` | `crates/contextra-erasure/src/vault/open.rs:37` |
| `tombstone_estimate` | `u64` | `crates/contextra-erasure/src/vault/open.rs:39` |
| `generation` | `u64` | `crates/contextra-erasure/src/vault/open.rs:41` |
| `scrub_cursor` | `u64` | `crates/contextra-erasure/src/vault/open.rs:43` |

#### `fn compute_metablock_mac`
- Ort: `crates/contextra-erasure/src/vault/open.rs:48`
```rust
pub fn compute_metablock_mac(mac_key: &[u8
```
Dokumentation: Berechnet den Metablock-MAC nach SPEC F.3: `HMAC-SHA256(K_vm, Bytes[0..50))[0..8]`.

#### `fn encode_metablock`
- Ort: `crates/contextra-erasure/src/vault/open.rs:82`
```rust
pub fn encode_metablock(params: MetablockParams<'_>) -> [u8
```
Dokumentation: Encodiert einen Metablock (64 Bytes) nach SPEC F.3.

#### `fn open_vault`
- Ort: `crates/contextra-erasure/src/vault/open.rs:220`
```rust
pub fn open_vault(header: &[u8]) -> Result<VaultOpenOutcome, ErasureError>
```
Dokumentation: Abwärtskompatible Hilfsfunktion für Header-Öffnung.

#### `fn open_vault_shard`
- Ort: `crates/contextra-erasure/src/vault/open.rs:119`
```rust
pub fn open_vault_shard( raw_shard: &[u8], expected_vault_id: &[u8
```
Dokumentation: Öffnet und prüft einen Tresor-Shard (Kopf + Metablock + Stichprobe, kein Vollscan, D-111 Abs. 8).

#### `const VAULT_METABLOCK_SIZE`
- Ort: `crates/contextra-erasure/src/vault/open.rs:25`
```rust
pub const VAULT_METABLOCK_SIZE: usize = 64
```
Dokumentation: Größe des Metablocks nach SPEC F.3.

### Modul `vault::rehash`

#### `struct RehashOutcome`
- Ort: `crates/contextra-erasure/src/vault/rehash.rs:20`
```rust
pub struct RehashOutcome
```
Dokumentation: Ergebnis einer Rehash-Operation (D-111).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `new_capacity` | `usize` | `crates/contextra-erasure/src/vault/rehash.rs:22` |
| `slots_copied` | `usize` | `crates/contextra-erasure/src/vault/rehash.rs:24` |
| `old_overwritten` | `bool` | `crates/contextra-erasure/src/vault/rehash.rs:26` |

#### `fn rehash`
- Ort: `crates/contextra-erasure/src/vault/rehash.rs:158`
```rust
pub fn rehash( old: &VaultTable, vault_id: [u8
```
Dokumentation: Rückwärtskompatible Hilfsfunktion für Rehash einer Tabelle.

#### `fn rehash_durable_shard`
- Ort: `crates/contextra-erasure/src/vault/rehash.rs:72`
```rust
pub fn rehash_durable_shard( old: &mut DurableVaultShard, vfs: &Arc<dyn Vfs>, erase_pending: Option<&AtomicBool>, ) -> Result<(DurableVaultShard, RehashOutcome), ErasureError>
```
Dokumentation: Führt Rehash für einen dauerhaften Shard über VFS durch nach SPEC D-111 Abs. 4 bis 6: - Prüft ob Abbruch durch wartendes erase angefordert wurde (`erase_pending`), falls λ <= 0.80. - Schreibt temporäre Datei `<nn>.vlt.new` mit F.1-Kopf und Metablock, kopiert alle aktiven Slots, sync_data. - Rename `<nn>.vlt.new` -> `<nn>.vlt`, sync_dir. - Überschreibt Altdatei über das gehaltene Handle vollständig mit Nullen, sync_data, schließen.

#### `fn rehash_shard`
- Ort: `crates/contextra-erasure/src/vault/rehash.rs:30`
```rust
pub fn rehash_shard( old: &mut VaultTable, vault_id: [u8
```
Dokumentation: Führt Rehash für einen In-Memory-Shard durch.

### Modul `vault::scrub`

#### `struct ScrubOutcome`
- Ort: `crates/contextra-erasure/src/vault/scrub.rs:27`
```rust
pub struct ScrubOutcome
```
Dokumentation: Ergebnis eines Scrub-Schritts.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `slots_scanned` | `usize` | `crates/contextra-erasure/src/vault/scrub.rs:29` |
| `f_orphans_cleaned` | `usize` | `crates/contextra-erasure/src/vault/scrub.rs:31` |
| `d_orphans_cleaned` | `usize` | `crates/contextra-erasure/src/vault/scrub.rs:33` |
| `next_cursor` | `u64` | `crates/contextra-erasure/src/vault/scrub.rs:35` |

#### `trait VaultScrubContext`
- Ort: `crates/contextra-erasure/src/vault/scrub.rs:14`
```rust
pub trait VaultScrubContext
```
Dokumentation: Kontext oder Callback-Prüfer für Manifest- und Dateiexistenz.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn file_exists(&self, file_id: &[u8` | `crates/contextra-erasure/src/vault/scrub.rs:16` | Prüft ob die gegebene Datei existiert (für F-Slots). |
| `fn document_in_manifest(&self, doc_tag: &[u8` | `crates/contextra-erasure/src/vault/scrub.rs:19` | Prüft ob ein Dokument noch im Manifest aktiv ist (für D-Slots). |
| `fn next_seq(&self) -> u64` | `crates/contextra-erasure/src/vault/scrub.rs:22` | Aktuelle nächste Sequenznummer des Stores (next_seq). |

#### `fn scrub_shard_step`
- Ort: `crates/contextra-erasure/src/vault/scrub.rs:39`
```rust
pub fn scrub_shard_step<C: VaultScrubContext>( vault: &mut FileVault, shard_idx: usize, batch_size: usize, ctx: &C, ) -> Result<ScrubOutcome, ErasureError>
```
Dokumentation: Führt einen Scrub-Schritt über bis zu `batch_size` Slots eines Shards durch.

### Modul `vault::sealer`

#### `mod rewrite`
- Ort: `crates/contextra-erasure/src/vault/sealer.rs:378`
```rust
pub mod rewrite
```
Dokumentation: Modul für Tresor-Blob-Rewrite mit Journal und Manifest (WP-5, INV-VAULT-FAILCLOSED-1).

#### `struct DocumentHierarchyTags`
- Ort: `crates/contextra-erasure/src/vault/sealer.rs:76`
```rust
pub struct DocumentHierarchyTags
```
Dokumentation: Kontext für Hierarchie-Tags eines Dokuments (T, C, S, D).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tenant_tag` | `[u8; 16]` | `crates/contextra-erasure/src/vault/sealer.rs:78` |
| `coll_tag` | `[u8; 16]` | `crates/contextra-erasure/src/vault/sealer.rs:80` |
| `subject_tag` | `Option<[u8; 16]>` | `crates/contextra-erasure/src/vault/sealer.rs:82` |
| `doc_tag` | `[u8; 16]` | `crates/contextra-erasure/src/vault/sealer.rs:84` |

#### `struct ProtectedDek`
- Ort: `crates/contextra-erasure/src/vault/sealer.rs:58`
```rust
pub struct ProtectedDek
```
Dokumentation: Sicher geschützter 32-Byte DEK-Puffer mit automatischem Zeroize-on-Drop.
- Impl-Block `crates/contextra-erasure/src/vault/sealer.rs:62`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(bytes: [u8` | `crates/contextra-erasure/src/vault/sealer.rs:64` | Erstellt einen neuen ProtectedDek. |
| `pub fn as_bytes(&self) -> &[u8` | `crates/contextra-erasure/src/vault/sealer.rs:69` | Gibt eine Referenz auf die geschützten Bytes zurück. |

#### `struct VaultSealer`
- Ort: `crates/contextra-erasure/src/vault/sealer.rs:89`
```rust
pub struct VaultSealer
```
Dokumentation: RecordSealer zur Kapselung von Datensätzen in Envelopes mit DEK-Cache und Gruppen-Sync.
- Impl-Block `crates/contextra-erasure/src/vault/sealer.rs:96`
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/vault/sealer.rs:292`
- Impl-Block (Trait `RecordSealer`) `crates/contextra-erasure/src/vault/sealer.rs:298`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/vault/sealer.rs:98` | Erstellt einen neuen `VaultSealer` im Speicher. |
| `pub fn new_in_memory() -> Self` | `crates/contextra-erasure/src/vault/sealer.rs:108` | Erstellt einen neuen `VaultSealer` im Speicher. |
| `pub fn with_vault(vault: Arc<RwLock<FileVault>>, k_unit: [u8` | `crates/contextra-erasure/src/vault/sealer.rs:113` | Erstellt einen `VaultSealer` über einen gegebenen Tresor mit K_unit-Schlüssel. |
| `pub fn is_poisoned(&self) -> bool` | `crates/contextra-erasure/src/vault/sealer.rs:123` | Prüft ob das Sealer-Handle vergiftet ist (INV-VAULT-FAILCLOSED-1). |
| `pub fn poison(&self)` | `crates/contextra-erasure/src/vault/sealer.rs:128` | Vergiftet das Sealer-Handle dauerhaft (Fail-Closed). |
| `pub fn safe_swap_buffers( &self, dest: &mut [u8], src: &mut [u8], fail_at_step: Option<u8>, ) -> Result<(), SealerError>` | `crates/contextra-erasure/src/vault/sealer.rs:138` | Sicherer Speicher-Swap mit Fail-Closed Garantie (WP-5a, INV-VAULT-FAILCLOSED-1). Tauscht die Inhalte von `dest` und `src` byteweise aus. Falls `fail_at_step` ak |
| `pub fn derive_dek_from_vault( &self, hierarchy: &DocumentHierarchyTags, ) -> Result<[u8` | `crates/contextra-erasure/src/vault/sealer.rs:193` | Leitet den DEK aus den Ahnen-Salts im Tresor ab (D-79). |

#### `enum SealerError`
- Ort: `crates/contextra-erasure/src/vault/sealer.rs:30`
```rust
pub enum SealerError
```
Dokumentation: Fehlerarten für Sealer- und Swap-Operationen mit Phasenbezug (WP-5a).

Varianten:

- `Poisoned` — `crates/contextra-erasure/src/vault/sealer.rs:33`
- `LengthMismatch{` — `crates/contextra-erasure/src/vault/sealer.rs:36`
- `InjectedSwapFault{` — `crates/contextra-erasure/src/vault/sealer.rs:44`
- `Durability(String)` — `crates/contextra-erasure/src/vault/sealer.rs:50`
- `Vault(String)` — `crates/contextra-erasure/src/vault/sealer.rs:53`

#### `type BoxFuture`
- Ort: `crates/contextra-erasure/src/vault/sealer.rs:21`
```rust
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>
```
Dokumentation: Type-Alias für gepinnte ge-boxte Futures ohne Tokio/Futures-Util.

### Modul `vault::sealer::rewrite`

#### `struct BlobRewriter`
- Ort: `crates/contextra-erasure/src/vault/sealer.rs:652`
```rust
pub struct BlobRewriter
```
Dokumentation: Absturzsicherer, fail-closed BlobRewriter über `StorageBackend`, `RewriteJournal` und `RewriteManifest`.
- Impl-Block `crates/contextra-erasure/src/vault/sealer.rs:660`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( journal: RewriteJournal, manifest: RewriteManifest, backend: Arc<dyn StorageBackend>, clock: Arc<dyn Clock>, ) -> Self` | `crates/contextra-erasure/src/vault/sealer.rs:662` | Erstellt einen neuen BlobRewriter. |
| `pub fn is_poisoned(&self) -> bool` | `crates/contextra-erasure/src/vault/sealer.rs:678` | Prüft ob der Rewriter dauerhaft vergiftet ist (INV-VAULT-FAILCLOSED-1). |
| `pub fn poison(&self)` | `crates/contextra-erasure/src/vault/sealer.rs:683` | Vergiftet das Rewriter-Handle dauerhaft. |
| `pub fn journal(&self) -> &RewriteJournal` | `crates/contextra-erasure/src/vault/sealer.rs:688` | Liefert eine Referenz auf das Journal. |
| `pub fn manifest(&self) -> &RewriteManifest` | `crates/contextra-erasure/src/vault/sealer.rs:693` | Liefert eine Referenz auf das Manifest. |
| `pub async fn execute_rewrite( &self, rewrite_id: u64, target_id: &str, pre_digest: [u8` | `crates/contextra-erasure/src/vault/sealer.rs:706` | Führt die 5-Phasen-Rewrite-Pipeline absturzsicher und fail-closed aus. 1. Phase 1: Intent ins Journal schreiben (`Planned`). 2. Phase 2: Überschreiben der Ziel- |
| `pub async fn recover_and_reconcile(&self) -> Result<usize, RewriteError>` | `crates/contextra-erasure/src/vault/sealer.rs:794` | Wiederanlauf nach Absturz (Crash Recovery): Prüft alle uncommitted Intents im Journal: - Wurde das Manifest vor dem Absturz bereits aktualisiert (`ManifestUpdat |

#### `struct ManifestEntry`
- Ort: `crates/contextra-erasure/src/vault/sealer.rs:542`
```rust
pub struct ManifestEntry
```
Dokumentation: Eintrag im Tresor-Manifest.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `target_id` | `String` | `crates/contextra-erasure/src/vault/sealer.rs:544` |
| `is_erased` | `bool` | `crates/contextra-erasure/src/vault/sealer.rs:546` |
| `updated_at_nanos` | `u64` | `crates/contextra-erasure/src/vault/sealer.rs:548` |
| `rewrite_id` | `u64` | `crates/contextra-erasure/src/vault/sealer.rs:550` |

#### `struct RewriteIntent`
- Ort: `crates/contextra-erasure/src/vault/sealer.rs:435`
```rust
pub struct RewriteIntent
```
Dokumentation: Intent-Datenstruktur für einen Blob-Rewrite.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `rewrite_id` | `u64` | `crates/contextra-erasure/src/vault/sealer.rs:437` |
| `target_id` | `String` | `crates/contextra-erasure/src/vault/sealer.rs:439` |
| `timestamp_nanos` | `u64` | `crates/contextra-erasure/src/vault/sealer.rs:441` |
| `state` | `RewriteState` | `crates/contextra-erasure/src/vault/sealer.rs:443` |
| `pre_digest` | `[u8; 32]` | `crates/contextra-erasure/src/vault/sealer.rs:445` |

#### `struct RewriteJournal`
- Ort: `crates/contextra-erasure/src/vault/sealer.rs:450`
```rust
pub struct RewriteJournal
```
Dokumentation: In-Memory- und VFS-fähiges Journal für Rewrite-Transaktionen.
- Impl-Block `crates/contextra-erasure/src/vault/sealer.rs:456`
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/vault/sealer.rs:534`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/vault/sealer.rs:458` | Erstellt ein neues In-Memory RewriteJournal. |
| `pub fn with_vfs(vfs: Arc<dyn Vfs>, journal_path: std::path::PathBuf) -> Self` | `crates/contextra-erasure/src/vault/sealer.rs:467` | Erstellt ein neues VFS-gestütztes RewriteJournal. |
| `pub fn record_intent(&self, mut intent: RewriteIntent) -> Result<(), RewriteError>` | `crates/contextra-erasure/src/vault/sealer.rs:476` | Erfasst den initialen RewriteIntent (Phase 1: Planned). |
| `pub fn update_state( &self, rewrite_id: u64, new_state: RewriteState, ) -> Result<(), RewriteError>` | `crates/contextra-erasure/src/vault/sealer.rs:485` | Aktualisiert den Zustand eines Intents im Journal. |
| `pub fn get_intent(&self, rewrite_id: u64) -> Option<RewriteIntent>` | `crates/contextra-erasure/src/vault/sealer.rs:503` | Liest einen Intent anhand seiner ID. |
| `pub fn uncommitted_intents(&self) -> Vec<RewriteIntent>` | `crates/contextra-erasure/src/vault/sealer.rs:509` | Liefert alle nicht abgeschlossenen Intents (zur Wiederanlaufprüfung). |

#### `struct RewriteManifest`
- Ort: `crates/contextra-erasure/src/vault/sealer.rs:555`
```rust
pub struct RewriteManifest
```
Dokumentation: Manifest des Tresors zur Verankerung aktiver und bereinigter Blobs.
- Impl-Block `crates/contextra-erasure/src/vault/sealer.rs:562`
- Impl-Block (Trait `Default`) `crates/contextra-erasure/src/vault/sealer.rs:631`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-erasure/src/vault/sealer.rs:564` | Erstellt ein neues leeres In-Memory RewriteManifest. |
| `pub fn with_vfs(vfs: Arc<dyn Vfs>, manifest_path: std::path::PathBuf) -> Self` | `crates/contextra-erasure/src/vault/sealer.rs:574` | Erstellt ein neues VFS-gestütztes RewriteManifest. |
| `pub fn version(&self) -> u64` | `crates/contextra-erasure/src/vault/sealer.rs:584` | Aktuelle Manifest-Version. |
| `pub fn get_entry(&self, target_id: &str) -> Option<ManifestEntry>` | `crates/contextra-erasure/src/vault/sealer.rs:589` | Liest einen Manifest-Eintrag. |
| `pub fn update_entry( &self, target_id: String, is_erased: bool, updated_at_nanos: u64, rewrite_id: u64, ) -> Result<u64, RewriteError>` | `crates/contextra-erasure/src/vault/sealer.rs:595` | Aktualisiert einen Eintrag atomar im Manifest (Phase 4). |

#### `struct RewriteOutcome`
- Ort: `crates/contextra-erasure/src/vault/sealer.rs:639`
```rust
pub struct RewriteOutcome
```
Dokumentation: Ergebnis eines Blob-Rewrite-Vorgangs.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `rewrite_id` | `u64` | `crates/contextra-erasure/src/vault/sealer.rs:641` |
| `target_id` | `String` | `crates/contextra-erasure/src/vault/sealer.rs:643` |
| `manifest_updated` | `bool` | `crates/contextra-erasure/src/vault/sealer.rs:645` |
| `journal_committed` | `bool` | `crates/contextra-erasure/src/vault/sealer.rs:647` |

#### `enum RewriteError`
- Ort: `crates/contextra-erasure/src/vault/sealer.rs:389`
```rust
pub enum RewriteError
```
Dokumentation: Fehlerarten für Rewrite-Operationen mit Phasenbezug (WP-5).

Varianten:

- `Poisoned` — `crates/contextra-erasure/src/vault/sealer.rs:392`
- `JournalWrite(u8, String)` — `crates/contextra-erasure/src/vault/sealer.rs:395`
- `BlobOverwrite(String)` — `crates/contextra-erasure/src/vault/sealer.rs:398`
- `SyncFailure(String)` — `crates/contextra-erasure/src/vault/sealer.rs:401`
- `ManifestUpdate(String)` — `crates/contextra-erasure/src/vault/sealer.rs:404`
- `JournalFinalize(String)` — `crates/contextra-erasure/src/vault/sealer.rs:407`
- `Recovery(String)` — `crates/contextra-erasure/src/vault/sealer.rs:410`
- `InjectedCrash(u8)` — `crates/contextra-erasure/src/vault/sealer.rs:413`

#### `enum RewriteState`
- Ort: `crates/contextra-erasure/src/vault/sealer.rs:418`
```rust
pub enum RewriteState
```
Dokumentation: Zustand eines Rewrite-Eintrags im Journal.

Varianten:

- `Planned` — `crates/contextra-erasure/src/vault/sealer.rs:420`
- `Overwritten` — `crates/contextra-erasure/src/vault/sealer.rs:422`
- `Synced` — `crates/contextra-erasure/src/vault/sealer.rs:424`
- `ManifestUpdated` — `crates/contextra-erasure/src/vault/sealer.rs:426`
- `Committed` — `crates/contextra-erasure/src/vault/sealer.rs:428`
- `RolledBack` — `crates/contextra-erasure/src/vault/sealer.rs:430`

### Modul `vault::slot`

#### `mod off`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:52`
```rust
pub mod off
```
Dokumentation: Byte-Offsets innerhalb eines 64-Byte-Slots (F.3).

#### `struct SlotData`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:102`
```rust
pub struct SlotData
```
Dokumentation: Entpackter Slot-Inhalt (in-memory).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `slot_type` | `SlotType` | `crates/contextra-erasure/src/vault/slot.rs:104` |
| `tag` | `[u8; 16]` | `crates/contextra-erasure/src/vault/slot.rs:106` |
| `payload` | `[u8; 32]` | `crates/contextra-erasure/src/vault/slot.rs:108` |
| `gen` | `u64` | `crates/contextra-erasure/src/vault/slot.rs:110` |

#### `enum SlotType`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:70`
```rust
pub enum SlotType
```
Dokumentation: Typen von Tresor-Slots (F.3, D-33, D-79, D-85, D-111, D-125).
- Impl-Block `crates/contextra-erasure/src/vault/slot.rs:85`

Varianten:

- `Tenant= 0x01` — `crates/contextra-erasure/src/vault/slot.rs:72`
- `Collection= 0x02` — `crates/contextra-erasure/src/vault/slot.rs:74`
- `Subject= 0x03` — `crates/contextra-erasure/src/vault/slot.rs:76`
- `Document= 0x04` — `crates/contextra-erasure/src/vault/slot.rs:78`
- `File= 0x05` — `crates/contextra-erasure/src/vault/slot.rs:80`
- `Epoch= 0x06` — `crates/contextra-erasure/src/vault/slot.rs:82`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_u8(b: u8) -> Option<Self>` | `crates/contextra-erasure/src/vault/slot.rs:87` | Konvertiert u8 in SlotType. |

#### `fn compute_header_tag`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:115`
```rust
pub fn compute_header_tag(mac_key: &[u8
```
Dokumentation: Berechnet den 16-Byte Header-Tag für einen Tresor-Shard-Kopf nach SPEC F.1 / F.3: `HMAC-SHA256(K_vm, Bytes[0..48))[0..16]`.

#### `fn compute_slot_mac`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:281`
```rust
pub fn compute_slot_mac(mac_key: &[u8
```
Dokumentation: Berechnet den 8-Byte Slot-MAC nach SPEC F.3: `HMAC-SHA256(K_vm, vault_id ‖ Bytes[0..56))[0..8]`.

#### `fn decode_slot`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:311`
```rust
pub fn decode_slot( raw: &[u8
```
Dokumentation: Deserialisiert einen Slot aus 64 Bytes nach SPEC F.3. Gibt `Ok(None)` bei leeren oder Grabstein-Slots zurück, `Err(Corrupt)` bei ungültigem Status oder ungültigem MAC.

#### `fn encode_slot`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:231`
```rust
pub fn encode_slot( vault_id: &[u8
```
Dokumentation: Serialisiert einen aktiven Slot in 64 Bytes inkl. MAC (F.3, D-111).

#### `fn encode_slot_body`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:261`
```rust
pub fn encode_slot_body(sd: &SlotData) -> [u8
```
Dokumentation: Serialisiert die Nutzlast eines aktiven Slots (ohne MAC).

#### `fn encode_vault_header`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:128`
```rust
pub fn encode_vault_header( file_id: &[u8
```
Dokumentation: Encodiert einen F.1-Dateikopf (64 Bytes) für einen Tresor-Shard (kind = 10, key_mode = 1) nach SPEC F.1 / F.3.

#### `fn mask_epoch_key`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:208`
```rust
pub fn mask_epoch_key(raw_key: &[u8
```
Dokumentation: Maskiert oder demaskiert einen Epochenschlüssel (Slot-Typ E) nach SPEC D-125 / F.3: `payload = K_epoch ⊕ HKDF-Expand(K_enc, "ctx/emask/v1" ‖ epoch_id, 32)`

#### `fn slot_probe_start`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:408`
```rust
pub fn slot_probe_start(slot_type: SlotType, tag: &[u8
```
Dokumentation: Bestimmt Shard und Startindex für lineare Sondierung nach SPEC D-111 Abs. 1: `h = BLAKE3(slot_type ‖ tag)` `shard = h[0] >> 4` (0..15) `i = u64_be(h[1..9]) mod 2^cap_log2`

#### `fn tombstone_image`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:296`
```rust
pub fn tombstone_image(vault_id: &[u8
```
Dokumentation: Erstellt das kanonische Grabstein-Abbild (tombstone_image) nach SPEC D-111 Abs. 3 / F.3: Alle Bytes 0 außer `status = 0x02`, MAC über das Abbild mit `vault_id`.

#### `fn verify_vault_header`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:161`
```rust
pub fn verify_vault_header( header_bytes: &[u8], mac_key: &[u8
```
Dokumentation: Prüft einen F.1-Dateikopf (64 Bytes) für einen Tresor-Shard.

#### `const CTX_FILE_MAGIC`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:30`
```rust
pub const CTX_FILE_MAGIC: &[u8
```
Dokumentation: F.1 Magic-Bytes für Contextra-Dateien.

#### `const CTX_FORMAT_VERSION`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:33`
```rust
pub const CTX_FORMAT_VERSION: u16 = 1
```
Dokumentation: F.1 Format-Version (v1 = 1).

#### `const STATUS_ACTIVE`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:47`
```rust
pub const STATUS_ACTIVE: u8 = 0x01
```
Dokumentation: Slot-Status: aktiv.

#### `const STATUS_EMPTY`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:45`
```rust
pub const STATUS_EMPTY: u8 = 0x00
```
Dokumentation: Slot-Status-Werte (F.3).

#### `const STATUS_TOMBSTONE`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:49`
```rust
pub const STATUS_TOMBSTONE: u8 = 0x02
```
Dokumentation: Slot-Status: Grabstein (für lineare Sondierung).

#### `const VAULT_ALG_IDS`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:39`
```rust
pub const VAULT_ALG_IDS: [u8
```
Dokumentation: Standard-Algorithmen-Satz v1: [aead=1, kdf=0, mac=1, hash=1].

#### `const VAULT_FILE_KIND`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:36`
```rust
pub const VAULT_FILE_KIND: u8 = 10
```
Dokumentation: Dateityp (kind) für Tresor nach SPEC F.1: 10.

#### `const VAULT_HEADER_SIZE`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:27`
```rust
pub const VAULT_HEADER_SIZE: usize = 64
```
Dokumentation: Größe des F.1-Dateikopfs in Bytes.

#### `const VAULT_SLOT_SIZE`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:42`
```rust
pub const VAULT_SLOT_SIZE: usize = 64
```
Dokumentation: Größe eines Tresor-Slots in Bytes (F.3, D-111, I-23).

### Modul `vault::slot::off`

#### `const GEN`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:58`
```rust
pub const GEN: usize = 2
```
Dokumentation: Offset des gen-Feldes (Bytes 2..8, u48 big-endian, 6 B).

#### `const MAC`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:64`
```rust
pub const MAC: usize = 56
```
Dokumentation: Offset des mac-Feldes (Bytes 56..64, 8 B).

#### `const PAYLOAD`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:62`
```rust
pub const PAYLOAD: usize = 24
```
Dokumentation: Offset des payload-Feldes (Bytes 24..56, 32 B).

#### `const SLOT_TYPE`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:54`
```rust
pub const SLOT_TYPE: usize = 0
```
Dokumentation: Offset des slot_type-Feldes (Byte 0, 1 B).

#### `const STATUS`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:56`
```rust
pub const STATUS: usize = 1
```
Dokumentation: Offset des status-Feldes (Byte 1, 1 B).

#### `const TAG`
- Ort: `crates/contextra-erasure/src/vault/slot.rs:60`
```rust
pub const TAG: usize = 8
```
Dokumentation: Offset des tag-Feldes (Bytes 8..24, 16 B).

### Modul `vault::table`

#### `struct Destroyed`
- Ort: `crates/contextra-erasure/src/vault/table.rs:36`
```rust
pub struct Destroyed
```
Dokumentation: Nachweis einer erfolgreichen Slot-Vernichtung (SPEC D-111, D-113).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `verified_via` | `VerifiedVia` | `crates/contextra-erasure/src/vault/table.rs:38` |

#### `struct VaultTable`
- Ort: `crates/contextra-erasure/src/vault/table.rs:42`
```rust
pub struct VaultTable
```
Dokumentation: In-Memory-Tresor-Shard (oder Tabelle) für Tests und als Kern-Implementierungsbasis.
- Impl-Block `crates/contextra-erasure/src/vault/table.rs:65`
- Impl-Block `crates/contextra-erasure/src/vault/table.rs:365`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(capacity: usize, vault_id: [u8` | `crates/contextra-erasure/src/vault/table.rs:67` | Erstellt eine neue leere Tabelle mit gegebener Kapazität (muss Zweierpotenz sein). |
| `pub fn new_shard(shard_no: u8, capacity: usize, vault_id: [u8` | `crates/contextra-erasure/src/vault/table.rs:72` | Erstellt eine neue leere Shard-Tabelle mit gegebener Shard-Nummer. |
| `pub fn is_poisoned(&self) -> bool` | `crates/contextra-erasure/src/vault/table.rs:89` | Gibt zurück, ob der Shard vergiftet ist (D-55, I-10). |
| `pub fn poison(&mut self)` | `crates/contextra-erasure/src/vault/table.rs:94` | Vergiftet den Shard irreversibel (D-55, I-10). |
| `pub fn load_factor(&self) -> f64` | `crates/contextra-erasure/src/vault/table.rs:110` | Aktueller Lastfaktor λ = (live + tombstones) / capacity. |
| `pub fn should_background_rehash(&self) -> bool` | `crates/contextra-erasure/src/vault/table.rs:118` | Prüft, ob Rehash im Hintergrund ausgelöst werden soll (λ > 0,60). |
| `pub fn must_force_rehash(&self) -> bool` | `crates/contextra-erasure/src/vault/table.rs:123` | Prüft, ob Rehash erzwungen werden muss (λ > 0,80). |
| `pub fn load_exceeded(&self) -> bool` | `crates/contextra-erasure/src/vault/table.rs:128` | Prüft, ob die Kapazität mit dem nächsten Einfügen erschöpft wäre (λ > 0,90, D-111 Abs. 4). |
| `pub fn find(&self, slot_type: SlotType, tag: &[u8` | `crates/contextra-erasure/src/vault/table.rs:145` | Findet den Slot-Index für (slot_type, tag). Gibt `None` zurück wenn nicht gefunden. Stoppt bei `STATUS_EMPTY` (Sondierung endet dort, D-79, F.3). |
| `pub fn get( &self, slot_type: SlotType, tag: &[u8` | `crates/contextra-erasure/src/vault/table.rs:173` | Liest einen Slot (dekodiert mit MAC-Prüfung). |
| `pub fn put(&mut self, sd: SlotData) -> Result<(), ErasureError>` | `crates/contextra-erasure/src/vault/table.rs:189` | Schreibt einen Slot (ER-04, D-111). Gibt Fehler zurück wenn Kapazität voll oder vergiftet. |
| `pub fn destroy( &mut self, slot_type: SlotType, tag: &[u8` | `crates/contextra-erasure/src/vault/table.rs:241` | Vernichtet einen Slot nach SPEC D-111 Abs. 3 und I-23: 1. Schreibt in EINEM 64-Byte-Schreibvorgang das Grabstein-Abbild (tombstone_image) 2. sync_data (bei in-m |
| `pub fn collect_active_slots(&self) -> Vec<SlotData>` | `crates/contextra-erasure/src/vault/table.rs:298` | Liefert alle aktiven Slots für den Rehash (Grabsteine werden ausgelassen, D-111 Abs. 5). |
| `pub fn overwrite_with_zeros(&mut self)` | `crates/contextra-erasure/src/vault/table.rs:313` | Überschreibt die gesamte Tabelle vollständig mit Nullen (D-111 Abs. 5). |
| `pub fn live_count(&self) -> usize` | `crates/contextra-erasure/src/vault/table.rs:320` | Anzahl aktiver Slots. |
| `pub fn tombstone_count(&self) -> usize` | `crates/contextra-erasure/src/vault/table.rs:325` | Anzahl Grabsteine. |
| `pub fn capacity(&self) -> usize` | `crates/contextra-erasure/src/vault/table.rs:330` | Kapazität. |
| `pub fn shard_no(&self) -> u8` | `crates/contextra-erasure/src/vault/table.rs:335` | Shard-Nummer. |
| `pub fn generation(&self) -> u64` | `crates/contextra-erasure/src/vault/table.rs:340` | Generation. |
| `pub fn set_generation(&mut self, gen: u64)` | `crates/contextra-erasure/src/vault/table.rs:345` | Setzt die Generation. |
| `pub fn rsv_cursor(&self) -> u64` | `crates/contextra-erasure/src/vault/table.rs:350` | Scrub-Cursor (rsv) für vault_scrub (D-111 Abs. 8). |
| `pub fn set_rsv_cursor(&mut self, cursor: u64)` | `crates/contextra-erasure/src/vault/table.rs:355` | Aktualisiert den Scrub-Cursor (rsv). |
| `pub fn vault_id(&self) -> &[u8` | `crates/contextra-erasure/src/vault/table.rs:360` | Vault-ID. |
| `pub fn slot_bytes_pub(&self, idx: usize) -> Option<&[u8` | `crates/contextra-erasure/src/vault/table.rs:367` | Öffentlicher Zugriff auf rohe Slot-Bytes für Scrubbing. |
| `pub fn mac_key_ref(&self) -> &[u8` | `crates/contextra-erasure/src/vault/table.rs:372` | Referenz auf MAC-Schlüssel. |

#### `enum VerifiedVia`
- Ort: `crates/contextra-erasure/src/vault/table.rs:25`
```rust
pub enum VerifiedVia
```
Dokumentation: Modus für Verifikation am Medium nach SPEC D-113.

Varianten:

- `Direct` — `crates/contextra-erasure/src/vault/table.rs:27`
- `CacheDropped` — `crates/contextra-erasure/src/vault/table.rs:29`
- `Cached` — `crates/contextra-erasure/src/vault/table.rs:31`

#### `const VAULT_SLOT_BASE`
- Ort: `crates/contextra-erasure/src/vault/table.rs:21`
```rust
pub const VAULT_SLOT_BASE: usize = 128
```
Dokumentation: Byte-Offset des ersten Slots in der Vault-Datei (F.3, ER-03).

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl From<SignatureVersion> for u8` — `crates/contextra-erasure/src/ed25519_proof.rs:148` (0 pub Methoden)
- `impl<T: ProofKeyProvider + ?Sized> ProofKeyProvider for std::sync::Arc<T>` — `crates/contextra-erasure/src/orchestrator.rs:32` (0 pub Methoden)
- `impl<T: CommitBarrier + ?Sized> CommitBarrier for Arc<T>` — `crates/contextra-erasure/src/types_local.rs:368` (0 pub Methoden)
- `impl<T: PurgeBarrier + ?Sized> PurgeBarrier for Arc<T>` — `crates/contextra-erasure/src/types_local.rs:377` (0 pub Methoden)
- `impl<T: ReportSigner + ?Sized> ReportSigner for Arc<T>` — `crates/contextra-erasure/src/types_local.rs:1107` (0 pub Methoden)
- `impl<T: ErasureObserver + ?Sized> ErasureObserver for Arc<T>` — `crates/contextra-erasure/src/types_local.rs:1182` (0 pub Methoden)
- `impl<T: LayerScanner + ?Sized> LayerScanner for Arc<T>` — `crates/contextra-erasure/src/types_local.rs:2069` (0 pub Methoden)
- `impl<T: Clock + ?Sized> Clock for Arc<T>` — `crates/contextra-erasure/src/types_local.rs:2145` (0 pub Methoden)
- `impl<T: IdGen + ?Sized> IdGen for Arc<T>` — `crates/contextra-erasure/src/types_local.rs:2188` (0 pub Methoden)
- `impl<T: Vfs + ?Sized> Vfs for Arc<T>` — `crates/contextra-erasure/src/types_local.rs:2249` (0 pub Methoden)
- `impl<T: FenceGate + ?Sized> FenceGate for Arc<T>` — `crates/contextra-erasure/src/types_local.rs:2339` (0 pub Methoden)
- `impl<T: WalSink + ?Sized> WalSink for Arc<T>` — `crates/contextra-erasure/src/types_local.rs:2371` (0 pub Methoden)
- `impl<T: KeyStore + ?Sized> KeyStore for Arc<T>` — `crates/contextra-erasure/src/types_local.rs:2404` (0 pub Methoden)
- `impl<T: ResidueScanner + ?Sized> ResidueScanner for Arc<T>` — `crates/contextra-erasure/src/types_local.rs:2430` (0 pub Methoden)
