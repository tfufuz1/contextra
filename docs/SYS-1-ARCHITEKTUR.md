# SYS-1 — Architektur des Gesamtsystems

Ersetzt R14 §1.4, §2, §10.1. Abhängigkeiten: keine. Gate: G-0 (`INV-TOPO-1`, `INV-LAYER-1`, `INV-PORT-1` als Tests).

## 1. Topologie

Der Workspace besteht aus **genau 14 Crates** (`INV-TOPO-1`). Ein Crate entsteht oder entfällt nur durch Änderung dieser Tabelle plus Gate. Abhängigkeiten zeigen nur auf strikt niedrigere Ebenen; innerhalb einer Ebene gibt es keine Kanten (`INV-LAYER-1`).

| Ebene | Crate | Rolle | darf kennen (intern) | `unsafe` |
|---|---|---|---|---|
| L3 | `contextra-engine` | Composition Root, Async-Rand, Policy | alle | verboten |
| L2 | `contextra-storage` | Speicherkern, synchron | `ports`(kernel), `types` | verboten |
| L2 | `contextra-crypto` | Cipher/Mac/Kdf/Checksum/KeyProvider-Referenz, Beleg-Signatur | `ports`(kernel), `types` | verboten |
| L2 | `contextra-vector` | HNSW/DiskANN/SQ8/RaBitQ/ACORN | `ports`(retrieval), `types` | verboten |
| L2 | `contextra-text` | BM25/WAND, Analyzer | `ports`(retrieval), `types` | verboten |
| L2 | `contextra-graph` | CSR, Hyperkanten, PPR/PathRAG | `ports`(retrieval), `types` | verboten |
| L2 | `contextra-rank` | Fusion, Kalibrierung, `DiBud` | `types` | verboten |
| L2 | `contextra-mvcc` | `EpochPinRegistry` | `types` | verboten |
| L2 | `contextra-checkpoint` | Snapshot-Set-Register, Restore-Koordination | `ports`(kernel), `types` | verboten |
| L1 | `contextra-ports` | Verträge (Features s. §3) | `types` | verboten |
| L1 | `contextra-durable-fs` | sichere Dateischicht | `sys` | verboten |
| L0 | `contextra-types` | Kerntypen, Fehler, Limits-Konstanten | — | verboten |
| L0 | `contextra-sys` | OS-FFI | — | **Insel** |
| L0 | `contextra-simd` | Runtime-SIMD-Dispatch | — | **Insel** |

**Bindungsregeln** (verbindlich, ergänzt R14 §2.1):
1. Nur `engine` kennt Kern, Krypto, Dateischicht, SIMD und Retrieval-Crates. Der Zusammenschluss geschieht ausschließlich in `engine::projection`.
2. `storage` kennt `crypto` **nie** als Crate, nur über `ports::kernel`-Trait-Objekte. `dev-dependencies` von `storage` dürfen `crypto` und `durable-fs` nutzen, **nur** hinter Feature `testing` und `[dev-dependencies]` (nie im normalen Graphen).
3. Persistenzverträge der Retrieval-Crates (`PartitionSink`/`PartitionSource`) liegen im `contract`-Modul des jeweiligen Crates; `ports::retrieval` enthält nur Lese-Verträge.
4. `checkpoint` definiert `SnapshotParticipant`/`RestoreStep`; `engine` bindet Kern und Index-Generationen als Teilnehmer.
5. `block_on` existiert nur in `engine::kernel`. L2-Crates rufen nie `.await`.
6. Fünf Erweiterungsverträge liegen im öffentlichen API von `storage`: `SegmentOwner`, `ExtensionBlock`, `BlockPruner`, `Consumer`/`ConsumerFeed`, `View::filter_visible` (CYC-7 §3).

**Repository-Bereinigung vor G-0a** (Reihenfolge verbindlich; Details `PLAN.md` Wurzel):
1. `crates/contextra-kvcache/` (enthält nur `AGENTS.md`) löschen und aus `members`, `[workspace.dependencies]`, `Cargo.lock`, `deny.toml` entfernen. Ohne diesen Schritt scheitert jeder Cargo-Aufruf.
2. Workspace-Abhängigkeiten für Untrusted-Formate (`zip`, `calamine`, `lopdf`, `cfb`, `mail-parser`, `html5ever`, `markup5ever_rcdom`, `quick-xml`, `csv`, `yaml-rust2`, `encoding_rs`, `toml`) **entfernen**. Sie widersprechen DEC-35/AT-9/NZ-24 (Parsing untrusted Formate außerhalb des Workspace). Extraktion läuft in einem separaten Prozess/Repository; Contextra nimmt nur extrahierten Text an (SYS-5 §6).
3. `capabilities.toml` aus dieser Tabelle neu erzeugen (Schema `name`, `layer`, `group`, `tier`, `deps`, genau 14 Einträge). Die heutige Datei beschreibt `contextra-ids`, `-env`, `-wire`, `-store`, `-vault` und weitere nicht existierende Crates.
4. `deny.toml` aus `capabilities.toml` ableiten (Generator `tools/gen_deny.py`). Die heutige `wrappers`-Liste nennt nicht existierende Crates; die `TRANSITION E29`-Ausnahmen für `tokio` in `crypto`, `checkpoint`, `graph` enden spätestens **2027-04-30** und werden mit G-R1 gelöscht.
5. `CLAUDE.md`: Verweise auf `docs/agent/*`, `tests/layering.rs`, `contextra-erasure`, `contextra-vault`, `contextra-ids` korrigieren oder entfernen; Architektur-Tests liegen in `crates/contextra-engine/tests/architecture.rs` (DEC-52).
6. `clippy.toml`: Verweise auf `governance/*` und `ADR-098` entfernen (Dateien existieren nicht); Verbotsliste bleibt, Ausnahmen nur per `#[allow(clippy::disallowed_methods)]` mit Begründung im Code.
7. `.github/workflows/` anlegen (SYS-6 §3). Es existiert keine CI; damit ist kein Gate gültig.

## 2. Zeit- und Ordnungstypen (DEC-60, `INV-EPOCH-1`)

```rust
// contextra-types
pub struct Epoch(u64);                 // < 2^56, streng monoton je Instanz; einziger Zeit-/Ordnungstyp
pub type Stamp = Epoch;                // Alias: kein zweiter Typ
pub struct InstanceId([u8; 16]);
pub struct Position { pub instance: InstanceId, pub epoch: Epoch }   // alles, was eine Instanzgrenze quert
```
Regeln: (1) Retrieval-Crates und `mvcc` kennen nur `Epoch`, nie `InstanceId`. (2) `Position` entsteht ausschließlich in `engine::projection` und `engine::query` (Anhängen der `InstanceId` an Watermarks, `SnapshotSet`, Receipts). (3) Vergleiche und `max/min` über `Position` mit verschiedenen `InstanceId` sind ein Typfehler (`Ord` ist **nicht** implementiert; nur `try_cmp`). (4) Stamp 0 ist reserviert (Füllrahmen, „kein Stamp“). (5) `Epoch::MAX = 2^56 − 1`; Überlauf ⇒ `Exhausted`. (6) Die in R14 §14 `INV-EPOCH-1` genannte „Stamp↔Epoch-Konversion“ bedeutet das Anhängen der `InstanceId`; es gibt keine numerische Umrechnung.

## 3. Ports

**Features von `contextra-ports`** (DEC-62; Cargo-Features sind additiv, daher ist die Trennung per Feature plus Test zu erzwingen):

| Feature | Inhalt | zieht | Verbraucher |
|---|---|---|---|
| `kernel` (Standard) | `Fs`, `Clock`, `Entropy`, `Anchor`, `KeyProvider`, `Cipher`/`Mac`/`Kdf`/`Checksum`, `Authority`, `Metrics`/`Observer`, `MemoryBudget`, `HostProbe`, `ColdTier`, `limits` | nur `types`, `bytes`, `zeroize`, `thiserror` | `storage`, `crypto`, `checkpoint` |
| `retrieval` | `VectorIndex`/`TextIndex`/`GraphIndex` (lesend), `EmbeddingProvider`, `Reranker` | `futures`, `serde` erlaubt | `vector`, `text`, `graph`, `engine` |
| `policy` | `EgressPolicy`, `PiiClassifier`, `IdentityResolver`, `ContextPreparer`, `GroundingValidator` | `futures`, `serde` | `engine` |
| `system` | `TextGenerator`, `ReportSigner`, `LedgerSink`, `ErasureObserver`, `LicenseGate`, `PluginManifest` | `futures`, `serde` | `engine` |
| `legacy` | Alt-Verträge (`StorageEngine`, `Vfs`, `CommitBarrier`, `Purger`, `ErasureLayer`) | wie bisher | `engine` bis G-CUTOVER |
| `conformance` | Konformanzsuiten | `proptest`-frei, nur `std` | `dev-dependencies` |

Test `feature_closure` (in `engine/tests/architecture.rs`): für jedes L2-Crate `cargo tree -p <crate> -e normal,features --prefix none` darf keines der Verbotspakete der Ebene enthalten (§9). Verbraucher binden `contextra-ports` immer mit `default-features = false` und expliziter Featureliste. Der Workspace-Feature-Union ändert nichts, weil der Test je Paket prüft.

**Kernel-Ports (Vertragsinhalt, `INV-PORT-1`: synchron, ohne `futures`/`serde`/`std::fs`):**

| Port | Verbindlicher Vertrag |
|---|---|
| `Fs` / `FsFile` | `open(path, mode, FileRole, IoCtx)`, `read_at`, `read_ranges`, `write_at`, `append_prealloc`, `preallocate`, `set_len`, `sync_data`, `sync_all`, `sync_dir`, `rename`, `unlink`, `list`, `create_dir_all`, `lock(Exclusive\|Shared)`, `free_space`, `len`; optional `link`, `reflink`, `punch_hole`. **`caps() -> FsCaps`** mit: `atomic_rename`, `dir_sync_durable`, `prealloc_reserves_space` (auch unter CoW/Quota), `punch_hole`, `reflink`, `hardlink`, `overlapped_sync_beneficial`, `sector_bytes`, `torn_write_unit`, `device_id`. Kein `mmap`. Ein Fehler von `sync_*` vergiftet das Handle (`INV-FSYNC-1`) und ist nie wiederholbar. `FileRole ∈ {Redo, Run, Blob, Catalog, Ledger, Segment, Keys, Anchor, Format, Reserve, Tmp}`. |
| `Clock` | `utc() -> UtcNanos`, `mono() -> MonoNanos`, `clock_floor() -> UtcNanos` (höchster durabel bekannter UTC-Wert), `skew_state() -> {Ok, Uncertain}`. Nie für Stamps. |
| `Entropy` | `fill(&mut [u8]) -> Result<(), EntropyError>`; kryptografisch stark; Fehler ⇒ fail-closed (keine Schlüssel-, Nonce-, ID-Erzeugung). |
| `Anchor` | `read()`, `advance(expect, next)` (CAS, monoton, durabel), `strength() -> Strong\|Weak`. Wert `AnchorValue{erase_epoch u64, ledger_head [u8;32], catalog_epoch u64}`. |
| `KeyProvider` | CYC-4 §3 (vollständige Schnittstelle). |
| `Cipher`/`Mac`/`Kdf`/`Checksum` | XChaCha20-Poly1305 (24-B-Nonce, 16-B-Tag); HMAC-SHA256 (Ausgabe 32 B, Kürzung auf 16 B nur wo SYS-2 es nennt); HKDF-SHA256; CRC32C. |
| `Authority` | `verify(principal, request_digest, signature) -> bool` für `LiftPolicy::FourEyes`. |
| `Metrics`/`Observer` | nicht blockierend, nie unter Sperren, geschlossener Katalog (SYS-4 §7). |
| `MemoryBudget` | `try_reserve(ResourceClass, bytes) -> Result<ResourcePermit, Limit>`; RAII-Freigabe; atomar. |
| `HostProbe` | `swap() , core_dumps(), memlock(), fs_type(), device_ids()`; Wert `Known\|Declared\|Unknown`; nur Diagnose. |
| `ColdTier` | optional; erhält Purge-Garantien. |

## 4. Threading

```text
Async-Rand (tokio, nur engine) ─ spawn_blocking (engine::kernel) ─►
Kern (sync):   Aufruferthread (Stufe-A-Guards, Hülle, Krypto, Vorkodierung)
               Sequencer: Flat-Combining-Leader, Rang 1, nur RAM, kein eigener Thread
               Syncer: 1 Thread (write + sync_data, Publikation)
               Wartung: Worker-Pool, Lanes urgent > flush > compact > blobgc > idle
               Ledger-Writer: 1 Thread, Kanal, gebündelt; Anker-Writer: 1 Thread, Kanal
Retrieval:     ComputePool (engine), Distanz/Quantisierung; nie unter Kernsperre
```

| Thread | Anzahl (Standard) | Blockieren erlaubt | Anmerkung |
|---|---|---|---|
| Aufrufer | beliebig | ja (Credit, Provider, Platte) | `Warten` nur an Credit (CYC-1 §3 Schritt 4) |
| Syncer | 1 | ja (E/A) | keine Sperre während `write`/`sync_data` |
| Wartungsworker | `max(1, min(cpus, 4))` | ja | Lane-Quoten CYC-5 §2 |
| Ledger-Writer | 1 | ja (E/A) | eigene Failure Domain |
| Anker-Writer | 1 | ja | koalesziert (`anchor_max_age`) |
| Blocking-Pool (`engine::kernel`) | `min(8, 2 × cpus)` | ja | einzige Brücke Async→Sync |
| ComputePool (`engine`) | `cpus` | nein (CPU) | Kanalparallelität QP-4 |

Provider-, Cipher-, KDF-, Graph- und Guard-Arbeit läuft ausschließlich **vor** Rang 1 oder auf Wartungs-Lanes (`INV-PIPE-1`). Auf der Entwicklungsmaschine (1 vCPU) laufen alle Rollen als Threads desselben Prozesses; die Rangdisziplin bleibt unverändert.

## 5. Sperrränge (`INV-LOCK-1`)

| Rang | Sperre | Über E/A | Bemerkung |
|---|---|---|---|
| 1 | `sequencer` | nein | Stamp-Vergabe, `Live→Retiring`, Link/Restrict/Lift, Branch-Ops |
| 2 | `intake` | nein | Leader nimmt Rang 2 nur unter Rang 1 |
| 3 | `catalog_edit` | ja | Gruppen-Commit; nie zugleich mit Rang 1 |
| 4 | `state` (`RwLock<Arc<Version>>`) | nein | Veröffentlichung |
| 5 | `registry` | nein | Holds, Consumer, Claims, Restrictions, `ViewInner` |
| 6 | `cluster_table` (sharded) | nein | Lesen aus Rang 1 zulässig |
| 7 | `maint` (je Lane) | nein | Blatt |
| 8 | `notify` | nein | Blatt |

Implementierung: Typ `RankedMutex<const R: u8, T>` / `RankedRwLock`. Rangverletzung ist ein Logikfehler (F-8): in Testbuilds `panic`-frei durch `Result` aus `lock()`, in Produktion Poison plus Observer-Ereignis. Der Rangstack ist ein `thread_local` und **in allen Builds aktiv**, wenn Feature `rank-check` gesetzt ist; CI baut Release-Tests mit `rank-check` (Kosten < 1 % bei ≤ 8 Sperren je Pfad). Kein `std::sync::Mutex` außerhalb `sync::ranked`. Kein Rückruf (`Observer`, `SegmentOwner`, `KeyProvider`, `Metrics`) unter Sperren.

## 6. Fehlermodell

| Klasse | Beispiele | Verhalten |
|---|---|---|
| F-1 transient | `EINTR` | Wiederholung mit Backoff (≤ 3, 1/2/4 ms) |
| F-2 Ressource | `ENOSPC` beim Vorallokieren, `DiskFull`, `Limit{..}`, `Backpressure` | kein Poison |
| F-3 E/A-hart | `EIO`, `sync_*`-Fehler | Handle `Poisoned`; nie auf demselben Handle wiederholt |
| F-4 Integrität | CRC/MAC/AEAD | kleinste Einheit vergiften (Block→Abschnitt→Datei→Segment), `Corrupt{..}` |
| F-5 Format | `Unsupported` | Öffnen/Dekodieren verweigert |
| F-6 Schlüssel | `KeyUnavailable{retry_after}`, `KeyLost` | Nichterreichbarkeit ≠ Vernichtung |
| F-7 Aufrufer/Index | s. Tabelle | typisiert mit `hint()` |
| F-8 Logik | Invariante verletzt | Poison + Observer; **kein `panic`** |
| F-9 Rollback | Anker höher als Catalog/Ledger | schreibend verweigert, `ReadOnly` mit Diagnose |

**Retry-Semantik der F-7-Fehler (verbindlich für Aufrufer und Fassade):**

| Fehler | Retry? | Voraussetzung / Wirkung |
|---|---|---|
| `Conflict{contended}` | ja, nach erneutem Lesen | neue Guards; Zähler `guard_retry_max` ist intern |
| `Backpressure(Retry{after})` | ja, nach `after` | gleicher Request, gleiche Guards zulässig |
| `DiskFull` | ja, nach Platzgewinn | Löschung/Lesen laufen weiter |
| `Limit{kind}` | nein ohne Änderung | Request oder Konfiguration ändern |
| `ClusterRetired` / `Erased{at}` | nie | terminal; Cluster nie wieder `Live` |
| `Restricted{effect}` | nein | erst nach `lift` (CYC-3 §7) |
| `UnknownCluster` | nein | nach `cluster_trim` oder nie existent |
| `BelowFloor{floor}` / `SnapshotTooOld` | neue View | `view_at(≥ floor)` bzw. neue View |
| `ErasePending` | später | nach Neustart und Gate (`INV-ERA-13`) |
| `LedgerUnavailable` / `LedgerSameDevice` / `AnchorUnavailable` | betriebsabhängig | CYC-3 §9 |
| `BackupBeyondHorizon` | nie | Backup zu alt |
| `ProviderAssuranceInsufficient` / `DeadlineBeforeKeyDestroy` | nein | Konfiguration ändern |
| `IndexLag` | ja | später oder `min_stamp` senken |
| `IdemConflict` / `IdemCapacity` | nein / später | anderer Token / nach TTL |
| `InvalidInput` | nein | |
| `Indeterminate{first,last}` | nur mit `IdemToken` | sonst lesen und prüfen (CYC-1 §5) |
| `Poisoned{..}` | nein | Neustart; Löschabsicht bleibt aufnehmbar (CYC-3 §9) |

`Error::hint()` ist stabil (Texte versioniert, nie Schlüssel, Nutzerschlüssel, Inhalte, Queries, Embeddings). `Debug` für `ClusterId`, Schlüssel, Nutzdaten, Query-Texte ist `Redacted<T>`. `ErrorClass` (für Metriken): `Transient, Resource, Io, Integrity, Format, Key, Caller, Logic, Rollback`.

## 7. Lints, Unsafe, verbotene APIs

1. `[workspace.lints]` bleibt wie im Repository (`deny` für `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `indexing_slicing`, `await_holding_lock`, `await_holding_refcell_ref`, `exit`, `dbg_macro`, `undocumented_unsafe_blocks`; `unsafe_op_in_unsafe_fn`). Zusätzlich: `let _ =` auf `sync_*`/`rename`/`unlink`/`append_durable` ist per `clippy::let_underscore_must_use` und Verbotsscript untersagt.
2. `#![forbid(unsafe_code)]` in allen Crates außer `sys` und `simd`. Der Workspace-Wert `unsafe_code = "deny"` bleibt; `sys` und `simd` setzen `#![allow(unsafe_code)]` modulweise mit `SAFETY`-Kommentar. Test `unsafe_islands` (cargo metadata + Quelltextscan) erzwingt genau diese beiden.
3. **`scripts/check_forbidden_apis.sh` (DEC-53), Soll-Verhalten:**
   - Pfade werden aus `[workspace].members` + `capabilities.toml` abgeleitet; ein fehlender Pfad ⇒ `exit 1` (kein `|| true` auf dem Pfadtest).
   - Musterklassen je Ebene: **Kern (`storage`)**: `std::fs::`, `SystemTime`, `Instant::now`, `rand::`, `getrandom`, `.unwrap(`, `.expect(`, `panic!(`, `unreachable!(`, `todo!(`, `tokio`, `futures::`, `block_on`, `parking_lot`, `memmap2`, `bincode`, `blake3`, `sha2`, `crc32fast`, `scc::`, `dashmap`, `static [A-Z_]*KEY`, `contextra_store`, `contextra_storage_ports`. **L2-Retrieval**: wie Kern ohne `futures::` in `contract`-Lesemodulen, plus `memmap2`, `DISKANN_INTEGRITY_KEY`. **Global (INV-SCOPE-1)**: `kv_cache`, `KvCache`, `kvcache`, `AdaptiveDecay`, `SessionBranchTree`.
   - Ausgenommen: `src/testing/`, `#[cfg(test)]`-Module, Kommentarzeilen.
   - **Selbsttest**: Fixture-Datei je Musterklasse unter `scripts/selftest/fixtures/` (liegt außerhalb `crates/`); der Test kopiert sie in ein temporäres Crate-Verzeichnis und erwartet Exit ≠ 0. Ein Lauf ohne gefundenes Fixture ⇒ Fehler „Script blind“.
4. `INV-LAYER-1`-Test liest `cargo metadata --locked` und vergleicht mit §1 (`layer`, `deps`); jede Kante in der falschen Richtung oder innerhalb einer Ebene bricht den Test.

## 8. Abhängigkeitsrichtlinie (normativ für jede `Cargo.toml`)

Auswahl in Rangfolge: (1) `std` genügt; (2) `forbid(unsafe)`-kompatibel; (3) kein Async im Kern und in L2-Algorithmuskernen; (4) ein Crate je Aufgabe, eine Workspace-Version je Abhängigkeit. Jede neue externe Abhängigkeit braucht: Eintrag in dieser Tabelle, `cargo deny check` grün, Lizenz in der Positivliste, `cargo geiger`-Befund im PR.

| Crate | erlaubt (extern) | verboten |
|---|---|---|
| `storage` | `thiserror ^2`, `bytes ^1`, `zeroize ^1`, `crossbeam-skiplist ^0.1` (nur hinter Trait `MemIndex`; Fallback sharded `BTreeMap` für `loom`), `siphasher ^1` (nur flüchtige Guard-Fingerabdrücke) | `tokio`, `async-std`, `futures`, `rayon`, `parking_lot`, `scc`, `dashmap`, `memmap2`, `serde*`, `bincode`, `blake3`, `sha2`, `rand`, `getrandom`, `crc32fast`, `anyhow`, `tracing` |
| `crypto` | `chacha20poly1305 ^0.10`, `hmac ^0.12`, `sha2 ^0.10`, `hkdf ^0.12`, `argon2 ^0.5` (nur Passphrase-Provider), `crc32c ^0.6` (zwingend CRC-32**C**), `subtle ^2`, `getrandom ^0.3` (nur `Entropy`-Adapter), `ed25519-dalek ^2` (`verify_strict`), `zeroize` | `blake3`, `aes-gcm-siv`, `bincode`, `tokio`, `kv_*`-Module |
| `durable-fs` | `sys`, `crc32c` | `libc`, `memmap2`, `crc32fast` |
| `sys` | `libc` (einziger Ort für OS-FFI); `mmap` nur hinter Feature `mmap` (Standard aus) | — |
| `types` | `thiserror 2`, `bytes`, `ahash`; `serde`/`bincode`/`blake3`/`sha2` nur hinter Feature `wire-compat` | `tokio` |
| `vector` | `ahash`, `roaring`, `subtle`, `zeroize` | `memmap2`, `std::fs`, `bincode`, `blake3`, `tokio`, `scc` |
| `text` | `bytes`, `itoa`, `regex`, `unicode-segmentation` | wie `vector` |
| `graph` | `bytes`, `ahash`, `arc-swap` | wie `vector` |
| `rank` | `ahash`; `serde` nur für Config/Reports | `tokio` |
| `engine` | `tokio`, `tokio-util` (nur Rand), `serde`, `serde_json` | Untrusted-Format-Parser |
| Werkzeuge (nur dev) | `loom ^0.7`, `shuttle ^0.8`, `proptest ^1`, `criterion`, `tempfile`; extern installiert: `cargo-fuzz`, `miri`, `cargo-mutants`, `cargo-llvm-cov`, `cargo-nextest`, `cargo-deny`, `cargo-geiger`, TLA+/TLC | — |

`deny.toml`: Positivliste der Lizenzen, `multiple-versions = "deny"` ab G-0 (heute `warn`), jede `ignore`-Zeile trägt `reason` und `expires` (Format `# expires: YYYY-MM-DD`; Script `scripts/check_deny_expiry.sh` bricht nach Ablauf). Die heutigen `ignore`-Einträge (`RUSTSEC-2024-0384`, `-0436`, `2026-0176`, `-0177`, `-0285`, `-0205`) haben kein Ablaufdatum und gehören zu Paketen (`pyo3`, `rustls`, `scc`, `paste`, `number_prefix`), die im 14-Crate-Graphen nicht vorkommen sollen: nach der Bereinigung entfallen sie; verbleibende erhalten Ablaufdatum.

## 9. Toolchain und Edition (DEC-79)

Rust **1.89.0** (`rust-toolchain.toml`), Edition **2021**, `rust-version = "1.89"`, identisch im Workspace. Kein Edition-Wechsel vor G-P (Risiko ohne Nutzen für den Kern). Toolchain-Anhebung nur als eigener PR mit vollem CI plus Nachtlauf, höchstens einmal je Phase, nie innerhalb eines Gate-Zeitraums. Installation über conda-forge (`tools/env/bootstrap.sh`); CI nutzt dieselbe Version. Plattformen v1: Linux (ext4, xfs), macOS (`F_FULLFSYNC`); Windows erst nach bestandenem Fs-Konformanzlauf (NZ-8).

## 10. Modulpläne

**`contextra-storage`** (nur abwärts, Test `layering`): `engine` (sequencer, syncer, memtable, version, views, branches, registry, extension, feed, page, cursor, cache, idem, guard_window, graveyard, volatile) · `maint` (flush, compaction, blob_gc, purge, scheduler, scrub, governor) · `erase` (gate, cluster_table, links, protocol, ledger, tickets, receipt, restrictions) · `recovery` · `checkpoint` · `catalog` · `store` (run, blob, redo) · `keys` · `codec` · `env` (Clock-/Entropy-Adapter-Typen, `MemFs`, `FaultFs`, `Sim`, nur `testing`) · `sync` (Ränge) · `facade` (`apply`, `view`, `erase`, `branch`, `checkpoint`, `space`; `lib.rs` nur Re-Exports) · `testing`. `MemFs`/`FaultFs`/`Sim*` sind **nicht** Produktionsstandard (DEC-43): `open` ohne explizit übergebene Ports ⇒ `InvalidInput`.

**`contextra-engine`:** `platform` (Adapter `Fs` ≤ ≈ 400 LOC, `Clock`, `Entropy`, `HostProbe`, `FileAnchor`) · `kernel` (Verdrahtung, Blocking-Pool, Fehlerabbildung, `ComplianceProfile`-Prüfung) · `projection` · `collection` · `query` · `ingest` · `compliance` · `workers`. Aufspaltung nur bei > 40 kLOC oder zyklischer Modulabhängigkeit nach G-R3; `INV-TOPO-1` bleibt 14.

## 11. Entscheidungen und geschlossene Lücken

DEC-60 (Epoch/Stamp/Position), DEC-62 (Ports-Features), DEC-79 (Edition/MSRV), DEC-52/53 (Architekturtests, Verbotsscript, hier präzisiert), GAP-03, GAP-09, GAP-10, GAP-11, GAP-12, GAP-13, GAP-14 (SYS-7 §3).

## 12. Abnahme

Tests `layering`, `unsafe_islands`, `feature_closure`, `capabilities_match_members` (INV-TOPO-1), `forbidden_selftest`, `deny_expiry`, `rank_violation_detected`. Gate G-0a: jedes der 14 Crates baut einzeln, `cargo metadata --locked` fehlerfrei.

**Offene Punkte:** keine (extern: zweiter Reviewer).
