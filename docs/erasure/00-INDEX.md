# Contextra Löschkette – Index und Querschnitt (konsolidiert)

*Stand: 28.11.2026 (Revision 2) · Basis: `tfufuz1/contextra` `main`, HEAD `e12873de6` (Vorversion: `e23bcdd`) · Grundlage: `KONZEPT-LOESCHKETTE-FUNDAMENT.md` · **Ersetzt** `00-INDEX-QUERSCHNITT.md`, `00a-INDEX-ERGAENZUNG.md`, `00b-INDEX-ERGAENZUNG-2.md`*

> **Verifikationsgrenze.** Statische Analyse, keine Rust-Toolchain. Nichts wurde kompiliert oder ausgeführt. **✔** = gelesen (Datei/Symbol), **[?]** = offen, mit Prüfauftrag in der jeweiligen Spezifikation.

## 1. Dokumente und Stand

| Nr. | Datei | Crate | Stand am HEAD | Inhalt |
| --- | --- | --- | --- | --- |
| 01 | `01-contextra-types.md` | types | umgesetzt ✔ | Typen, Fehler, Zustände, Garantien |
| 02 | `02-contextra-ports.md` | ports | umgesetzt; **offen:** A-1, A-3, A-4 (`begin_purge`), A-8 (`resume`) | `ReadGate`, `ErasureLayer`, `ErasureKernel`, `LedgerSink` |
| 03 | `03-contextra-store.md` | store | Kernel umgesetzt (≈ 7 000 LOC); **offen:** K-33/A-2, K-45…K-47, K-50, K-51 | Journal, Gate, Filter, Purge, Pending-Delete, Ledger |
| 04 | `04-contextra-mvcc.md` | mvcc | umgesetzt ✔ | Snapshot-Entzug, Pin-Alterung |
| 05 | `05-contextra-crypto.md` | crypto | `ErasureRoot`/`ErasureLabel` umgesetzt ✔, **ohne Verwender** (K-48/K-60); Rückbau, `kv-shredding`, Stufe 2 offen | Rückbau, Schlüsselwurzel, Stufe 2 |
| 06 | `06-contextra-text.md` | text | `TextErasure` vorhanden ✔ (295 LOC), **nicht verdrahtet** (K-48) | Textindex als `ErasureLayer` |
| 07 | `07-contextra-vector.md` | vector | `HnswErasure` vorhanden ✔ (98 LOC), **nicht verdrahtet**, ohne Konformanztest (K-48) | HNSW als `ErasureLayer` |
| 08 | `08-contextra-graph.md` | graph | **nicht umgesetzt** (kein `ErasureLayer`, kein Zeuge) | Provenienz, Graph-Layer |
| 09 | `09-contextra-checkpoint.md` | checkpoint | Restore-Sperre nur auf Meta-Schlüssel ✔ (K-54 offen); Layer, Taint, Floor **nicht umgesetzt** | Restore-Sperre, Taint-Politik |
| 10 | `10-contextra-engine.md` | engine | Treiber, Worker, API, Registry, Resolver vorhanden ✔; **offen:** K-45…K-49, K-52, K-53, K-56…K-58 | Treiber, Resolver, Worker, API |
| 11 | `11-contextra.md` | contextra | Builder-Verdrahtung der Layer **offen** (K-48, K-58) | Builder, Profil |
| 12 | `12-contextra-mcp.md` | mcp | `contextra_erase`, `_erase_status`, `_erase_dry_run` vorhanden ✔ | Erase-Werkzeuge |
| 13 | `13-contextra-testkit.md` | testkit | Mocks, Referenzmodell, Konformanzsuite, Loom vorhanden ✔ | CrashVfs, Konformanz, Referenzmodell |
| 14 | `14-lueckenlosigkeit.md` | quer | **neu (Rev. 2)** | Audit gegen HEAD, Befunde K-45…K-60, Kernel-Vertrag Rev. 2, Datenort-Matrix, CP-19…30, Tests, WPs |

Nicht betroffen: `durable-fs` (kein Abhängiger, Store darf es nicht einbinden), `db`, `rank`, `simd`, `sys`.

## 2. Korrekturen gegenüber Konzept und Annahmen (verbindlich)

| ID | Befund (Beleg) | Konsequenz |
| --- | --- | --- |
| K-1 | Group-Commit-Leader gibt `commit_mutex` vor `wal.append_batch` frei; Sequenz wird in `prepare_commit_ops` vergeben ✔ | Gate-Prüfung an zwei Stellen (03 §6) |
| K-2 | SSTables liegen in einer flachen Liste, Compaction nur über zusammenhängende Eingaben ✔ | Erase-Rewrite 1:1 pro Datei, leeres Ergebnis ⇒ `Remove` |
| K-3 | Token-Bucket nur lokal je Merge ✔ | `IoBudget` extrahiert (03 §7.6) |
| K-4 | `MemTable` hatte kein Entfernen | `MemTable::retain` |
| K-5 | Default-Collection hat kein Präfix ✔ | `EraseTarget::Collection` verweigert `default`; Scope = Regelmenge, Longest-Match |
| K-6 | `ContextraError` ist `non_exhaustive`, neue Varianten nur am Ende | `Erased` am Ende |
| K-7/K-41 | `legacy-erasure` ist Default-Feature **und** trägt den KV-Cache-Shredding-Pfad (`KeyRegistry`→`RevocationLog`, `KvSegmentCipher`, `EncryptedCommitmentSalt`) ✔ | Nur Beweiscode entfällt, Rest unter neuem Feature `kv-shredding` (05 §3) |
| K-8 | `Contextra` ist in der Engine definiert; Facade hat Invariante ≤ 20 `pub fn` ✔ | Erase-API als `impl Contextra` in der Engine |
| K-9 | `restore_checkpoint` ist Rollback desselben Stores ✔ | Rollback-Sperre bis `Completed` (09) |
| K-10 | Recovery und Manifest nutzen `tokio::fs` ✔ | Umstellung auf `StoreIo` (03 WP-3.6) |
| K-11 | HNSW `save_internal` verschluckt Fehler beim Öffnen des Verzeichnisses, nutzt `std::fs` ✔ | 07 §6 |
| K-12 | `ErasureId([u8;16])` existiert ✔ | Bytes über `Rng::fill_bytes` |
| K-13 | Replay-Filter an `recovery.rs:322/331`, Gate vor Zeile 277 | Gate im `LsmStorage`-Konstruktor |
| K-14 | `KeyBuf` aus Ports entfernt | `KeyScope` |
| K-15 | WAL-Schnitt: Leader mit geklontem `Arc<Wal>` ⇒ `TST-STORE-ROT-1` | WAL-Schnitt gesperrt bis grün |
| K-16/K-32 | Store-Kernel, Journal, Gate, Ledger, MVCC-Entzug sind im Repo | 05–13 zitieren Symbole des Codes |
| K-17 | `Vfs` ist synchron, `rename` Default `Unsupported`; Store kapselt in `StoreIo` ✔ | Erase-I/O nur über `StoreIo` |
| K-18 | Manifest nutzt `tokio::fs` ✔ | „wie Manifest“ meint nur das Framing |
| K-19 | MCP-Werkzeuge heißen `contextra_*` ✔ | `contextra_erase`, `_erase_status`, `_erase_dry_run` |
| K-20 | `FlushPurgeFence` nur Flush-Seite genutzt ✔ | Erase setzt `purge_active` nicht |
| K-21 | `revoke_covering` entzieht nur Mandanten-Snapshots ✔ | G1 stützt sich nie darauf |
| K-22 | `EdgeProvenance`/`DocEdgeIndex` existieren; `Edge.source_doc_id` ist zweites Feld ✔ | 08 baut auf `EdgeProvenance` |
| K-23 | Checkpoint-Metadaten sind Store-Schlüssel `{namespace}:checkpoint:{name}` ✔ | Scope-Abgleich in 09 |
| K-24 | `inner_storage()` liefert `Arc<LsmStorage>` | Kernel über `LsmStorage::erasure_kernel()` |
| K-25 | Legacy-Symbol `DeletionProofKeyPair` nur in `store/tests/campaign_j42_audit.rs` ✔ | kleiner Rückbau |
| K-26 | `TextPurger` committet selbst, arbeitet über `StorageEngine` ✔ | Layer liest über `PrivilegedRead` |
| K-27 | `HnswIndex.deleted_nodes: RwLock<RoaringTreemap>` ✔ | Dokumentsicht zusätzlich, nicht eingetragen |
| K-28 | Lesepfad: `is_active()` ist Acquire-Load | – |
| K-29 | H-2 verwirft nicht, nur `prepare_commit_ops` lehnt ab | Gate-Maske, Flush-Filter, `retain`, Replay-Filter |
| K-30 | Posting-Schlüssel `i:`/`plb:` nicht im Schlüssel-Scope | `CollapseVersions` über A-1 |
| K-31 | `contextra-store` ist unter `cfg(loom)` nicht gebaut | Loom prüft ein Modell |
| **K-33** | **Ledger-/Log-Schlüssel nicht persistent:** `open_early` leitet aus `fill_key_bytes` (Zufall) ab, `recovery.rs:228` nutzt `OsKeyMaterialSource` ✔ | Nach Neustart `LedgerCorrupt`, Idempotenz bricht. **Auslieferungsstopp**, A-2 |
| K-34 | `register_collapse` nur auf `StoreErasureKernel` (`mod.rs:797`), `ProgressSink` ohne Implementierer außer Tests ✔ | A-1 |
| K-35 | `WitnessLayer` ohne Text/Checkpoint-Variante; Text nutzt `Custom("TextIndex")` ✔ | A-3 Konstanten |
| K-36 | Layering: text/vector/graph/checkpoint nur `ports`/`types`; Facade nur `db`/`ports`/`types`; MCP nur `contextra`; testkit nur `ports`/`types` ✔ | Checkpoint nutzt den Kernel-Port |
| K-37 | HNSW wird in Produktion aus dem LSM neu aufgebaut (`load_index`), `save`/`load_mmap` ohne Engine-Aufrufer ✔ | G3 Vektor = RAM-Bereinigung, Datei-Rewrite optional |
| K-38 | Graph-Schlüssel `__graph:entity:/edge:/community:` sind tenantweit (`csr/types.rs:9-12`) ✔ | Nicht im Collection-Schlüssel-Scope; Bereinigung über Provenienz |
| K-39 | Text: Präfix-Scan je Dokument, gebündelte Listen `plb:{term}`, `resident_index`, `meta:stats` ✔ | 06 |
| K-40 | `derive_dek` verwirft `hkdf_expand`-Fehler (`let _ =`) ✔ | fallibel (05) |
| K-42 | `deletion_proof_active` steuert Durability-Prüfung, nicht Nachweis ✔ | Name bleibt, Doku korrigiert (11) |
| K-43 | `#[instrument(skip(self))]` mit Parameter `id` (`delete.rs:14`) ✔ | Gate G-3, Korrektur (10) |
| K-44 | `drop_collection` ist nicht atomar, lässt HNSW/Graph/Checkpoints und `__col_idx` unberührt ✔ | wird `erase(Collection)` |
| **K-45…K-60** | Befunde des Audits am HEAD `e12873de6` (Reihenfolgenwiderspruch `record_layer`/`purge_store`, Fehlerpfad und Quarantäne, nicht nachgearbeitete Layer-Reste, unverdrahtete Layer, verschluckte `on_gate`-Fehler, WAL-Zeuge, Restore nach `Completed`, u. a.) | **Vollständig in `14-lueckenlosigkeit.md` §2**; verbindlich, S1-Befunde blockieren Auslieferung |

## 3. Nachträge

- **A-1 (02 §4.3):** `ProgressSink::register_collapse(&self, keys: &[(Vec<u8>, Seq)])` und `ErasureKernel::register_collapse(id, keys)`; Implementierung ist die vorhandene Methode des Kernels. Der Treiber liefert `KernelProgress { kernel, id }`.
- **A-2 (03, behebt K-33):** neuer Port `ErasureRootProvider { root() -> ErasureRoot; is_external() -> bool }` in `store/src/key_source.rs`. Implementierungen: `FileErasureRoot` (`<erasure_dir>/root.key`, `write_atomic` + `sync_dir`, Modus 0600, nur beim allerersten Start erzeugt), `DerivedErasureRoot` (aus `KeyManager`, wenn Passphrase gesetzt), `StaticErasureRoot` (Tests). `open_early` nimmt `&dyn ErasureRootProvider`; Schlüssel nur aus `ErasureRoot::derive()` (05 §4). Existiert ein Journal oder Ledger, aber keine Wurzel: Fail-Closed, **keine** Neuerzeugung. Regulated + nicht externe Wurzel ⇒ Start verweigert. Tests `TST-STORE-ROOT-1…5` (Neustart besteht `verify`; stabile `erasure_id_hmac`; fehlende Wurzel; Regulated-Verweigerung; Regression rot auf heutigem Code).
- **A-3 (02):** `LAYER_TEXT = "TextIndex"`, `LAYER_CHECKPOINTS = "Checkpoints"` und `WitnessLayer::text()/checkpoints()` (liefern `Custom`). Journal-Format unverändert.

- **A-4 (02 §4.6, 14 §3.2):** `ErasureKernel::begin_purge(id)`, idempotent, extrahiert aus `purge_store`; setzt `Purging` (`PurgeStart`) und `Restricted → Erased`. Der Treiber ruft es vor dem ersten Layer. Behebt K-45. Kein Formatwechsel.
- **A-5 (09, 14 §3.9):** persistenter *Erasure-Floor* aus `completed_summaries`; `restore_checkpoint` verweigert Checkpoints mit `meta.seq_no < gated_seq` eines abgeschlossenen Auftrags, sofern nicht `Rewritten`. Behebt K-54.
- **A-6 (10/11, 14 §3.6):** `ErasureConfig.required_layers` je Profil; `ErasureError::LayerMissing{layer}` (am Ende). Builder registriert Text → Graph → Vektor → Checkpoint. Behebt K-48.
- **A-7 (10, 14 K-52):** `ErasureId`-Bytes aus `KeyMaterialSource` (OS-CSPRNG), `SeededRng` nur in Tests.
- **A-8 (02/10, 14 §3.3):** `ErasureKernel::resume(id)` und `Contextra::erase_resume(id)`; einziger Weg aus `Quarantined`, nie automatisch, kein MCP-Werkzeug.
- **Fehlerpräzisierung:** Quarantäne-Schwelle liegt allein im Kernel (`StoreErasureConfig.quarantine_after_attempts`); `Failed{layer: Some(l)}` löscht den Layer-Cursor (kein neues Record-Tag).

## 4. Querschnittsentscheidungen

**Schlüsselmodell.** Das Gate arbeitet auf Rohschlüssel-Bytes (`t:{tenant}:` + Collection-Schlüssel). `KeyScope` = Regeln `(Prefix|Exact, In|Out)`, längste Regel gewinnt, `Exact` vor `Prefix`, kein Treffer = nicht im Scope. `DocScope` = `Set | Collection | Tenant`. `Collection("default")` wird abgelehnt. Mandantenpräfix immer mit abschließendem `:`. `Document` verwendet nur exakte Schlüssel.

**Gate (fail-closed).** Lesen gesperrter Schlüssel wirkt wie „nicht vorhanden“, Schreiben ⇒ `Erased`. Compaction: `Restricted` behält, `Erased` darf verwerfen. Nicht ladbarer Zustand wirkt für alles wie `Restricted` (I-7).

**Lock-Ränge.** Bestehend 100 Intent, 200 Commit, 300 Flush, 400 State, 500 WAL-Truncate, 600 SSTables, 700 Memtable-Shards, 800 Manifest, 900 Observer, 950 Block-Cache. Neu 150 `RANK_ERASURE_PLAN`, 960 `RANK_ERASURE_GATE_SNAPSHOT`. Rang 150 wird nie über einen Layer-Aufruf gehalten.

**Zeit, Zufall, I/O, Logging.** Zeit nur über `Clock`, Zufall nur über `Rng`, keine `HashMap`/`HashSet`. I/O nur über `Vfs`/`StoreIo`. Journal, Ledger, Metriken, Tracing: IDs nur als gekürzter HMAC. `tracing::instrument` ohne `skip` auf Funktionen mit `id`/`key`/`target` ist verboten.

**Fehlerklassen.** `Retryable | Permanent | Poisoned | ScopeViolation | Cancelled`. Teilfehler sind `Err`. Kein `let _ =`, kein `unwrap_or_default()` in Erase-/Purge-Pfaden.

**Zustandsautomat.** `Requested → Planned → Gating → Gated → [KeysDestroying] → Purging → Verifying → Sealed → Completed`, Seitenzustände `Failed`, `Quarantined`. Ab `Gated` kein Rücksprung vor `Gated`. Layer-Arbeit beginnt erst nach `begin_purge` (Zustand `Purging`, A-4); Fehler führen nach `Failed`, der nächste Versuch beginnt wieder mit `begin_purge`; `Quarantined` verlässt nur `resume` (A-8).

**Schichtreihenfolge im Treiber.** (1) RAM-Caches über `ErasureObserver`, (2) Text → Graph → Vektor, (3) Store (`purge_store`), (4) Checkpoints. `expected_layers` enthält genau die registrierten Schichten; eine registrierte, aber leere Schicht meldet `remaining = 0`.

**CI-Gates** (`tools/erasure_gates.py`, je mit Rot-Grün-Test): G-1 kein `block_on` in Erase-/Purge-Pfaden; G-2 kein `let _ =`/`unwrap_or_default()`/`.ok();`; G-3 `instrument` ohne `skip` mit `id`/`key`/`target`; G-4 kein `std::fs`/`tokio::fs` in `store/src/erasure/`, `engine/src/erasure/`, den neuen Layer-Dateien in `text`, `vector`, `graph`, `checkpoint`; G-5 keine `Instant::now`/`SystemTime::now`/`HashMap` in neuen Dateien; G-6 `ReadGate` ohne Default-Methoden. **Neu (14 §6):** G-7 kein `tokio::spawn` ohne `TaskTracker` in `engine/src/erasure/`; G-8 `Result` von `fail`/`record_layer`/`on_gate` nicht verworfen; G-9 kein `_ =>` in den Encodern von `store/src/erasure/journal.rs`.

## 5. Reihenfolge

**Laufzeit-Startreihenfolge** (normativ, 14 §3.11): Wurzel → Kernel (`open_early`) → `LsmStorage` (Gate/Filter vor Replay) → Layer registrieren → `mask_layers` + Observer-Replay → Lesepfade frei (`NotReady` endet) → Worker.

**Arbeitsreihenfolge:**

```text
WP-14.0  K-45/46/47 + A-4   (ohne diese läuft keine Layer-Kette durch)
A-1, A-2, A-3  (vor allem anderen; K-33 ist ein Auslieferungsstopp)
 └─ 05 §3 Rückbau, §4 Wurzel
     ├─ 13 §4 CrashVfs (blockiert Crash-Tests von 03)
     ├─ 10 Treiber/Worker/API ── 11 Facade ── 12 MCP     ← erste Auslieferung (LSM + Dokumentsicht)
     ├─ 06 Text ┐
     ├─ 07 Vektor ├ parallel
     ├─ 08 Graph ┘
     ├─ 09 Checkpoint + 04 §4
     ├─ 14 WP-14.2…14.7 (Observer/Start, Pflicht-Layer, Zeugen, Scheduler, Gates)
     └─ 05 §5 Stufe 2 (opt-in)
```

Jedes Paket endet mit `cargo build --workspace --locked`, `cargo test --workspace --locked`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`, `tests/layering.rs` und den Gates G-1…G-6. `capabilities.toml` bleibt unverändert (keine neue Kante).

## 6. Globale Definition of Done

1. Invarianten I-1…I-12 (Konzept 5.3) haben je einen Test, der bei Verletzung rot wird; Mutationstests über `mutants-guarantees.toml`.
2. Crash-Matrix CP-1…CP-18 läuft über `CrashVfs` (13 §4).
3. Canary `T-DEL-1`: eindeutige Bytefolge schreiben, `erase(Document)`, alle Dateien byteweise durchsuchen, 0 Treffer.
4. Leerlauf-CPU des Workers 0; Lesepfad ohne aktive Gates ein Acquire-Load.
5. `NotMeasured`/`FeatureDisabled` senkt die ausgewiesene Garantie sichtbar (I-6).
6. `tests/layering.rs` grün.
7. **Datenort-Matrix** (14 §4): Zeilen 1–16 „geschlossen“ (G1, G3, Zeuge, Crash-Test), Zeilen 17–20 verhindert oder im `ErasureStatus.exceptions` ausgewiesen.
8. Crash-Matrix CP-1…CP-30 und Loom-Modelle grün; Gates G-1…G-9 grün, `tools/erasure_gates_baseline.txt` ohne Eintrag für `engine/src/erasure/`.
9. Ohne registrierte Pflicht-Layer wird nie `purged = true` ausgewiesen (I-6).

## 7. Glossar

**Retire** logisches Löschen (G0). **Erase** Auftrag mit Gate (G1), physischem Entfernen (G3), optional Schlüsselvernichtung (G2). **Gate** RAM-Snapshot gesperrter Mengen. **Epoche** monotoner Zähler der Gate-Veröffentlichungen. **Hülle** alle Dateien, die Scope-Schlüssel enthalten können. **Witness** gemessene Restmenge je Schicht. **Pending-Delete** durable Liste noch zu entfernender Dateien.
