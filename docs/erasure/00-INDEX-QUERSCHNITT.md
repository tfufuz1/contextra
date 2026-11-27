# Contextra Löschkette – Implementierungsspezifikationen: Index und Querschnitt

*Stand: 27.11.2026 · Rolle: Principal Rust Engineer · Basis: `tfufuz1/contextra` `main`, HEAD `17cecf57c` · Grundlage: `KONZEPT-LOESCHKETTE-FUNDAMENT.md` (Analysecommit `3b6a19953`)*

> **Verifikationsgrenze.** Statische Analyse. Im Arbeitsumfeld existiert keine Rust-Toolchain (`cargo`/`rustc` fehlen). Nichts wurde kompiliert oder ausgeführt. **✔** = am Code gelesen (Datei:Zeile), **[?]** = offen, mit konkretem Prüfauftrag in WP-0. Jede Spezifikation nennt am Ende ihre offenen Punkte.
>
> **HEAD gegenüber dem Konzept.** `git diff --stat 3b6a19953 HEAD` zeigt ausschließlich das Löschen von `docs/SPEC-ERASURE.md` und `docs/IMPL-SPEC-ERASURE.md` ✔. Das Konzept verweist für WP-1 auf `IMPL-SPEC-ERASURE.md`. Diese Datei existiert am HEAD nicht mehr (nur in der Historie: `git show 3b6a19953:docs/IMPL-SPEC-ERASURE.md`). Die Crypto-Spezifikation (05) ist deshalb in sich vollständig.

---

## 1. Dokumente

| Nr. | Datei | Crate | Ring / Klasse | Konzept-WP | Inhalt in einem Satz |
| --- | --- | --- | --- | --- | --- |
| 01 | `01-contextra-types.md` | `contextra-types` | Ring 0 | 2 | Typen, Fehler, Zustände, Garantiemodell |
| 02 | `02-contextra-ports.md` | `contextra-ports` | Ring 0 | 2 | Verträge v2: `ReadGate`, `ErasureLayer`, `ErasureKernel`, `LedgerSink`; Rückbau der Platzhalter |
| 03 | `03-contextra-store.md` | `contextra-store` | Ring 1 | 3, 4, 10 | Erasure-Kernel: Journal, Gate, Filter, Purge, Pending-Delete, Ledger |
| 04 | `04-contextra-mvcc.md` | `contextra-mvcc` | Ring 0 | 3, 9 | Snapshot-Entzug und Pin-Alterung |
| 05 | `05-contextra-crypto.md` | `contextra-crypto` | Ring 0 | 1, 11 | Rückbau `legacy-erasure`, SaltVault (Stufe 2) |
| 06 | `06-contextra-text.md` | `contextra-text` | Ring 0/1 | 6 | Async-Purge, Posting-Kollaps, Kandidatenfilter |
| 07 | `07-contextra-vector.md` | `contextra-vector` | Ring 0/1 | 7 | Maskierung (G1), Segment-Rewrite (G3), `Vfs`-I/O |
| 08 | `08-contextra-graph.md` | `contextra-graph` | Ring 0 | 8 | Pflicht-Provenienz, Purge, `NeedsRecompute` |
| 09 | `09-contextra-checkpoint.md` | `contextra-checkpoint` | Ring 1 | 9 | Rollback-Sperre, Hardlink-Klone, Taint-Politik |
| 10 | `10-contextra-engine.md` | `contextra-engine` | Ring 3 | 5, 9, 10 | Treiber, Scope-Auflösung, Worker, öffentliche Erase-API |
| 11 | `11-contextra.md` | `contextra` | Ring 4 | 5, 11 | Builder-Verdrahtung, `ErasureProfile` |
| 12 | `12-contextra-mcp.md` | `contextra-mcp` | Ring 4 | 5 | `ctx_erase`, `ctx_erase_status`, Rechte |
| 13 | `13-contextra-testkit.md` | `contextra-testkit` | Klasse T | 12 | Konformanzsuite, Crash-Harness, Referenzmodell |

**Nicht betroffen (begründet).** `contextra-durable-fs`: bleibt unverändert und ohne Abhängigen (B-15). Der Store darf es laut `capabilities.toml` nicht einbinden ✔, das Konzept baut das Journal bewusst auf `Vfs`. `contextra-db`: enthält nach Dateiliste (`homeostat`, `multistep`, `volatile_vault`, `pid_latency_controller`) keinen Lösch- oder Collection-Pfad ✔; das `Contextra`-Handle liegt in `contextra-engine/src/lib.rs:361` ✔. **[?]** WP-0 bestätigt per `rg -n "drop_collection|delete_prefix|Purger" crates/contextra-db` (Erwartung: keine Treffer). `contextra-rank`, `contextra-simd`, `contextra-sys`: kein Bezug.

---

## 2. Korrekturen gegenüber dem Konzept

Beim Lesen des Codes ergaben sich Abweichungen von Annahmen des Konzepts. Sie sind verbindlich und in den Spezifikationen eingearbeitet.

| ID | Konzept-Annahme | Befund am Code | Konsequenz |
| --- | --- | --- | --- |
| **K-1** | `gated_seq` wird unter `commit_mutex` gelesen; danach liegt kein Commit „dazwischen“ (6.1) | Der Group-Commit-Leader **gibt `commit_mutex` vor `wal.append_batch` frei** und nimmt ihn für Memtable-Update und Sichtbarkeit wieder (`lsm/mod.rs`, Abschnitt „`commit_mutex` Role“ ✔). Die Sequenz wird in `prepare_commit_ops` vergeben (`ops/write.rs:309`, `fetch_add`) | Gate-Prüfung an **zwei** Stellen: `prepare_commit_ops` (Ablehnen) und `apply_mem_updates` (`lsm/commit.rs:92`, Verwerfen). Beide unter `commit_mutex`. Siehe 03 §6 |
| **K-2** | Hülle über „alle Level“, ein Merge über die Vereinigung | SSTables liegen in **einer flachen Liste**, sortiert nach `max_seq`; Compaction verlangt **zusammenhängende** Eingaben (`compaction/engine.rs:331-345`, sonst `Ok(false)`) ✔. Es gibt keine Level-Struktur | Erase-Rewrite ist **1:1 pro Datei** (Prädikat `DropKeys` ist zustandslos). Leeres Ergebnis → `ManifestEntry::Remove`, nicht `Replace` (`Replace.added` ist ein einzelner Pfad, `manifest/entry.rs:28`) |
| **K-3** | „Gemeinsamer Token-Bucket mit der regulären Compaction“ existiert | Die Drosselung besteht aus **lokalen Variablen** je Merge-Aufruf (`io_token_bytes_written`, `io_token_last_reset_nanos`, `compaction/engine.rs:636-637, 780-805, 864-891`) ✔ | Vorarbeit: verhaltensgleiche Extraktion in `IoBudget` (03 WP-4.0) |
| **K-4** | Memtable-Einträge „unter kurzer `state.write()`-Sperre entfernen“ | `MemTable` hat **keine Entfernen-Operation** (nur `put`, `rollback(tx_id)`, `get*`, `iter*`, `scan*`, `memtable.rs`) ✔. Der Flush materialisiert alle Einträge in einer `BTreeMap` (`lsm/ops/compaction.rs`, `key_map`) | Neue Operation `MemTable::retain(pred)` (03 §7.1). Flush-Filter vor dem Einfügen in `key_map` |
| **K-5** | `Collection{…}` ist als Byte-Präfix darstellbar | Die Default-Collection hat **kein Präfix** (`prefix = b""`, `collection/mod.rs:373-377`); `namespaced_key` liefert für Typ 0 und 4 den Rohschlüssel. Benannte Collections: `__col:{name}:\x00{typ}{key}`. Text: `__txt:{name}:`. Physisch zusätzlich `t:{tenant}:` (`TenantScopedStorage::make_key`, `tenant_codec.rs:519`) ✔. `drop_collection` verweigert `default` (`maintenance.rs:854`) ✔ | `EraseTarget::Collection` verweigert `default` (`InvalidInput`). Scope-Beschreibung als **Regelmenge mit Longest-Prefix-Wins** statt Präfixliste. `DocScope` kennt `Set`, `Collection`, `Tenant` (kein Aufzählen von Millionen `DocId`) |
| **K-6** | `ContextraError::Erased` ergänzen | `ContextraError` ist `#[non_exhaustive]`, neue Varianten **ausschließlich am Ende** (`types/src/error.rs:118-124`) ✔ | Variante am Ende anhängen, DTO-/MCP-Abbildung ergänzen |
| **K-7** | WP-1 „Rückbau `legacy-erasure`“ ist ein Löschen | `kv-encryption = ["legacy-erasure"]` und beide sind **Default-Features** (`crypto/Cargo.toml:39-42`) ✔ | Erst Features entkoppeln, dann löschen. Reihenfolge in 05 §3 |
| **K-8** | Facade `contextra` bekommt `erase()` | `Contextra` ist in `contextra-engine/src/lib.rs:361` definiert, `contextra/src/lib.rs` hat 4 `pub fn` und die Invariante „≤ 20 pub fn, keine direkten Abhängigkeiten auf Ring 0/1“ ✔ | Erase-API als `impl Contextra` in der Engine (`contextra_impl/erasure.rs`); Facade: nur Builder/Profil (11) |
| **K-9** | Checkpoint-Restore „lehnt markierte Checkpoints ab“ (Hardlink-Restore) | `CheckpointStore::restore_checkpoint` ist **Rollback desselben Stores** (`rollback_to_tx`), mit Serialisierungsbarriere „neuere Commits → Fehler“ (`store.rs:756-783`) ✔. Hardlink-Klone sind ein eigener Pfad (`hardlink_cloner.rs`) | `rollback_to_tx` wird gesperrt, solange ein Erase-Auftrag nicht `Completed` ist (I-3). Zwei getrennte Maßnahmen in 09 |
| **K-10** | I/O im Store läuft über `Vfs` | `manifest/rollover.rs:141,177` und `lsm/recovery.rs` (u. a. 449, 462, 656, 725, 860, 878) nutzen `tokio::fs` ✔. Rollover ersetzt `MANIFEST` per `rename` (alte Generation nicht aufbewahrt) ✔ | WP-0-Punkt 3 geklärt: **keine** alten Manifest-Generationen. `tokio::fs` im Erase-relevanten Pfad auf `Vfs` (03 WP-3.6) |
| **K-11** | HNSW-Speicherreihenfolge ungeklärt (Punkt 9) | `save_internal`: `sync_all` → `rename` → Verzeichnis-`sync_all`; Fehler beim **Öffnen** des Verzeichnisses werden verschluckt (`if let Ok`), alles über `std::fs` (`hnsw/types.rs:932-946`) ✔ | Reihenfolge korrekt, Fehlerbehandlung und `Vfs`-Pflicht nicht. Siehe 07 |
| **K-12** | `ErasureId` (128 Bit) neu aus `IdGen` | `ErasureId([u8;16])` **existiert** in `types/src/ids.rs:73`; `IdGen::next_id() -> u64` ✔ | Vorhandenen Typ verwenden, Bytes aus `Rng::fill_bytes` (deterministisch über `SplitMix64`). Duplikate `Seq`/`Timestamp`/`TargetKind` in `ports` konsolidieren (01 §4.1) |
| **K-13** | Replay-Filter vor „Anwendung auf die Memtable“ | Replay in `LsmStorage::new_with_merge_operator` (`recovery.rs:277-340`), Memtable-Anwendung bei 322 und 331; danach **erzwungener Start-Flush** (`recovery.rs:639-645`) ✔ | Filter an genau diesen Stellen. Das Gate muss **vor Zeile 277** im Konstruktor geladen sein, also im `LsmStorage`-Konstruktor, nicht erst in der Engine (03 §5) |
| **K-14** | `LedgerSink` v2 baut auf vorhandenem Port | Port-Typen nutzen `KeyBuf([u8;16])` (`ledger.rs`, `gate.rs`) und `RangeSet` aus `KeyBuf`-Paaren ✔ | Vollständig neu definieren (02 §5), kein Weiterverwenden von `KeyBuf` |
| **K-15** | WAL-Schnitt = Rotation + Flush | Rotation läuft unter `commit_mutex` (`ops/compaction.rs:59`); ein Leader mit **bereits geklontem** `Arc<Wal>` (`ops/write.rs:278`) kann nach der Rotation auf das versiegelte Segment schreiben **[?]** | WP-0-Test `TST-STORE-ROT-1` (03 §10). Bis zur Klärung ist der WAL-Schnitt gesperrt |

---

## 3. Verbindliche Querschnittsentscheidungen

Diese Entscheidungen gelten für alle Spezifikationen.

### 3.1 Schlüsselmodell und Scope

- Das Gate arbeitet auf **Rohschlüssel-Bytes** (`&[u8]`), so wie `LsmStorage` sie sieht: `t:{tenant}:` + Collection-Schlüssel.
- Ein **Scope** ist `KeyScope`: eine Menge von Regeln `(Muster, Urteil)` mit Muster ∈ {`Prefix(bytes)`, `Exact(bytes)`} und Urteil ∈ {`In`, `Out`}. Ein Schlüssel gehört zum Scope, wenn die **längste passende Regel** `In` ist; `Exact` schlägt `Prefix`. Kein Treffer = nicht im Scope. Damit sind Komplemente ausdrückbar (Tenant ohne fremde Collections) ohne Aufzählung.
- **DocScope** (Dokumentsicht der Indizes): `Set(BTreeSet<DocId>)` | `Collection{tenant, collection}` | `Tenant{tenant}`. `DocId` ist `u64` (Feature `docid-128`: `u128`) ✔ `types/src/types/domain/ids.rs:171-178`; der Code verwendet nur den Typ.
- `EraseTarget::Collection` für `"default"` wird abgelehnt (K-5). Die Default-Collection wird nur per `Document` oder `Tenant` gelöscht.
- **Reservierte Präfixe** in der Default-Collection (`__docid:`, `__col:`, `__txt:`, `__rel:`, `__tx_intent:`) sind nicht von Nutzerschlüsseln unterscheidbar. Erase von `Document` verwendet ausschließlich **exakte** Schlüssel, nie Präfixe. Ein Nutzerschlüssel, der mit `__` beginnt, kann deshalb kein anderes Dokument treffen. Erase von `Tenant` ist per Definition vollständig.

### 3.2 Gate-Semantik (fail-closed)

| Operation | Gate `Open` | `Restricted` / `Erased` |
| --- | --- | --- |
| Punkt-Lesen, Snapshot-Lesen, Zeitreise (jede Sequenz) | normal | wie **nicht vorhanden** (`Ok(None)`, im Scan übersprungen). Kein Fehler, kein Hinweis auf Existenz |
| Schreiben/Löschen auf gesperrten Schlüssel | normal | `ContextraError::Erased` (damit Clients nicht wiederholen) |
| Compaction/Flush/Replay | normal | `Restricted`: Version bleibt erhalten und wird nicht gelesen. `Erased`: darf verworfen werden |
| Wiederanlage nach `Sealed` | – | nur mit `seq > up_to`; Altversionen bleiben gesperrt |

Unbekannter oder nicht ladbarer Gate-Zustand wirkt wie `Restricted` für **alle** Schlüssel (I-7).

### 3.3 Lock-Ränge

Bestehend ✔ (`lsm/lock_order.rs`): 100 Intent · 200 Commit · 300 Flush · 400 State · 500 WAL-Truncate · 600 SSTables · 700 Memtable-Shards · 800 Manifest · 900 Observer · 950 Block-Cache. Neu:

| Rang | Konstante | Primitiv | Zweck |
| --- | --- | --- | --- |
| 150 | `RANK_ERASURE_PLAN` | `tokio::sync::Mutex<()>` | Serialisiert Annahme, Gate-Veröffentlichung und Journal-Übergänge je Store. Genommen **vor** `commit_mutex` |
| 960 | `RANK_ERASURE_GATE_SNAPSHOT` | `parking_lot::RwLock<Arc<GateSnapshot>>` | Nur Arc-Clone/Austausch, nie über `.await` |

Regel: Rang 150 nimmt nie einen Rang ≥ 300 und hält ihn über keinen Layer-Aufruf. `erase_plan`-Sperre und Journal-`fsync` zusammen < 5 ms Haltedauer.

### 3.4 Zeit, Zufall, I/O, Logging

- Zeit **nur** über `Clock` (`now_unix_nanos`, `monotonic_nanos`) ✔. Zufall nur über `Rng`. Verboten per `clippy.toml`: `Instant::now`, `SystemTime::now`, `std::env::var`, `rand::thread_rng` (Schlüsselmaterial ausgenommen), `std::collections::HashMap/HashSet` ✔ (Konzept 3.3).
- I/O **nur** über `Vfs` (`write_atomic`, `rename`, `sync_dir`, `remove_file`, `open` → `VfsFile::{write_at, sync_data, sync_all}`) ✔ `ports/src/vfs.rs`.
- Journal, Ledger, Metriken, Tracing: Schlüssel- und Dokument-IDs nur als `HMAC-BLAKE3(erasure_log_key, id)` (8 Byte Hex). `tracing::instrument` ohne `skip` auf Funktionen mit `id`/`key`/`target`-Parametern ist verboten (CI-Gate G-3).
- Store: `#![forbid(unsafe_code)]`, `deny(dead_code, unwrap_used, expect_used, panic, todo, unimplemented, unreachable, indexing_slicing)` ✔. Neuer Code muss im selben Paket verdrahtet sein.

### 3.5 Fehlerklassen (einheitlich)

`LayerErrorKind::{Retryable, Permanent, Poisoned, ScopeViolation, Cancelled}` (02 §4). Teilfehler sind `Err`, nie `Ok` mit Zähler. Kein `let _ =` und kein `unwrap_or_default()` in Erase-/Purge-Pfaden (CI-Gate G-2).

### 3.6 Zustandsautomat

`Requested → Planned → Gating → Gated → [KeysDestroying] → Purging → Verifying → Sealed → Completed`; Seitenzustände `Failed(retryable)`, `Quarantined(layer)`. Ab `Gated` kein Rücksprung. Vollständige Tabelle und Übergangsbedingungen: 01 §5 und 03 §4.

---

## 4. Reihenfolge und Abhängigkeiten

```text
WP-0 (Baseline, CI-Gates, offene Punkte)
  └─ WP-1  05 §3  Rückbau legacy-erasure
       └─ WP-2  01 Typen + 02 Ports
            └─ WP-3  03 §4-§6 Journal, Gate, Hooks   + 04 §3 (mvcc)
                 └─ WP-4  03 §7-§9 Purge, Filter, Pending-Delete
                      └─ WP-5  10 Treiber/Worker + 11 + 12      ← erste Auslieferung (Canary grün)
                           ├─ WP-6  06 Text      ┐
                           ├─ WP-7  07 Vektor    ├ parallel
                           ├─ WP-8  08 Graph     ┘
                           ├─ WP-9  09 Checkpoint + 04 §4
                           └─ WP-11 05 §4 SaltVault (opt-in, parallel)
                      WP-10  03 §11 Ledger (nach 6–9)
                      WP-12  13 Testkit (fortlaufend)
```

Jedes Paket endet mit: `cargo build --workspace --locked`, `cargo test --workspace --locked`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`, `tests/layering.rs` (Workspace-Root, Aufruf wie in der Repo-CI) und den CI-Gates aus 3.7. Wegen `deny(dead_code)` im Store wird neuer Code im selben Paket verdrahtet.

### 3.7 CI-Gates (neu, `tools/erasure_gates.py`)

| ID | Prüfung | Muster |
| --- | --- | --- |
| G-1 | Kein `block_on` in Erase-/Purge-Pfaden | `rg "block_on" crates/*/src/{erasure,purge}*` und `crates/contextra-text/src/inverted/purge.rs` |
| G-2 | Kein `let _ =`, `unwrap_or_default()`, `.ok();` in `store/src/erasure/`, `*purge*.rs`, `engine/src/erasure/` | `rg` |
| G-3 | `#[tracing::instrument]` ohne `skip` auf Funktionen mit Parameter `id`, `key`, `target` | AST-Heuristik |
| G-4 | Kein `tokio::fs`/`std::fs` in `store/src/erasure/`, `engine/src/erasure/`, `vector/src/hnsw/` | `rg` |
| G-5 | Kein `Instant::now`/`SystemTime::now`/`HashMap` in neuen Dateien | durch `clippy.toml` bereits erzwungen; Gate prüft Dateiliste |
| G-6 | `ReadGate`-Implementierungen ohne Default-Methoden | Compile-Fail-Test in `ports` |

Rot-grün-Test jedes Gates in WP-0 (Muster aus `tests/test_gate_g8_negative.py` ✔).

---

## 5. Auflösung der WP-0-Punkte aus dem Konzept (12.1)

| Nr. | Frage | Stand | Beleg / Prüfauftrag |
| --- | --- | --- | --- |
| 1 | Serialisiert `commit_mutex` alle Commits? | **Teilweise.** Sequenzvergabe ja, physische I/O nein (K-1) | `lsm/mod.rs` „`commit_mutex` Role“, `ops/write.rs:254,276,309`. **[?]** `single_commit_path` (`write.rs:368`, `group_commit_window_micros == 0`) hält `commit_mutex` über die gesamte Dauer? Test `TST-STORE-COMMIT-1` |
| 2 | Replay- und Commit-Hook | **Geklärt** | Replay: `recovery.rs:287-338` (Anwendung 322/331). Commit: `prepare_commit_ops` (`write.rs:300-365`) und `apply_mem_updates` (`commit.rs:92`, aufgerufen `write.rs:469, 816`) |
| 3 | Alte Manifest-Generationen? | **Geklärt: keine** | `rollover.rs:18,141` (`rename` auf `MANIFEST`). Footer-Min/Max der SSTables bleiben Schlüsselträger (Witness) |
| 4 | Graph-Persistenzpfad | **Teilweise** | `graph/src/csr/graph_write/mod.rs` nutzt `StorageEngine` ✔; kein `std::fs` im Produktivcode. **[?]** Schlüsselfamilie: `TenantKeyCodec::encode_graph_key` (`t:{tenant}:{collection}:graph:{entity}`) ist ein anderes Format als `namespaced_key`. Inventar in 08 §2 |
| 5 | Intent-Dateien | **[?]** | `__tx_intent:` existiert als **Schlüssel** (`namespaced_key` Typ 3) und liegt damit im Scope. Dateien: `recovery.rs` ~449–725, 860–878 prüfen (03 §10, `TST-STORE-INTENT-1`) |
| 6 | Alle Engine-Pfade über `TenantScopedStorage`? | **Weitgehend** | `contextra_impl/collections.rs:60,273,324` ✔; `tests/tenant_wrapper_coverage.rs` existiert. **[?]** `rg "LsmStorage::" crates/contextra-engine/src` auf direkte Zugriffe |
| 7 | Nutzung `CheckpointMeta.metadata` | **[?]** | `meta.rs:69,79` (`into_workflow_state`). Produktivnutzer per `rg` (09 §9) |
| 8 | Aufrufer `Purger`/`ScanTarget` | **Geklärt** | Nur eigene Crates (B-02 ✔) |
| 9 | `fsync`/`rename`-Reihenfolge HNSW | **Geklärt** | K-11 |
| 10 | Workspace-Lauf | **Offen** | Keine Toolchain. WP-0: `cargo build/test/clippy/deny`, die drei `#[ignore]`-Tests (`store/src/scanner/tests.rs:90,178,204`) |

---

## 6. Globale Definition of Done

1. **Korrektheit:** Alle Invarianten I-1…I-12 (Konzept 5.3) haben je mindestens einen automatisierten Test, der bei Verletzung rot wird (Mutationstest über `mutants-guarantees.toml` ✔ für Gate und Filter).
2. **Crash:** Matrix CP-1…CP-12 und die Ergänzungen CP-13…CP-18 (03 §9) laufen systematisch über `fault_vfs` ✔.
3. **Nachweis:** Canary-Test `T-DEL-1` (`contextra/tests/deletion_e2e.rs`) grün: eindeutige Bytefolge schreiben, per `erase(Document)` löschen, alle Dateien im Datenverzeichnis byteweise durchsuchen: 0 Treffer.
4. **Ressourcen:** Leerlauf-CPU des Workers 0; Lesepfad ohne aktive Gates ein Relaxed-Load; Benchmarks in `benchmarks/contextra-bench` ✔.
5. **Ehrlichkeit:** `NotMeasured`/`FeatureDisabled` senkt die ausgewiesene Garantie sichtbar (I-6).
6. **Layering:** `capabilities.toml` unverändert (keine neue Kante) ✔; `tests/layering.rs` grün.

## 7. Glossar

**Retire** logisches Löschen (Tombstone, G0). **Erase** Löschauftrag mit Gate (G1), physischem Entfernen (G3), optional Schlüsselvernichtung (G2). **Gate** RAM-Snapshot gesperrter Schlüssel- und Dokumentmengen. **Epoche** monotoner Zähler der Gate-Veröffentlichungen. **Hülle** alle Dateien, die Scope-Schlüssel enthalten können. **Witness** gemessene Restmenge je Schicht (`LayerWitness`). **Pending-Delete** durable Liste noch zu entfernender Dateien.
