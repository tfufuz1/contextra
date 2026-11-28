# Contextra – Master-Spezifikation für Agenten (MSA v2.0)

*Stand 28.11.2026 · Basis: `tfufuz1/contextra` `main`, HEAD `d9ebd1192` · Ersetzt MSA v1.1 · Status: normativ*

> **Verifikationsgrenze.** Statische Lesung des Klons (Dateien, `grep`, `Cargo.toml`-Auswertung mit `tomllib`). **Nichts wurde kompiliert, getestet oder ausgeführt.** Aussagen von v1.1 zu `cargo check` und Testläufen sind nicht übernommen. **✔** = am HEAD per Datei/Symbol belegt · **◐** = teilweise · **✖** = offen · **[?]** = statisch nicht entscheidbar, Prüfauftrag in Anhang C.

---

## 0. Lesekarte (kontextschonend)

**Prinzip.** Dieses Dokument enthält Entscheidungen, Status, Reihenfolge und Abnahme, **nicht** die Detailentwürfe. Ein Agent liest immer §1–§4 und §10, dazu **genau** den Abschnitt seiner Aufgabe, danach nur die genannte Stelle der Detailquelle (per `sed -n`/`rg`, nie die ganze Datei). Die Quellen im Repo umfassen rund 1,3 MB; diese Datei ersetzt das Durchlesen.

| Aufgabe | MSA-Abschnitt | Detailquelle (nur diese Stelle) |
| --- | --- | --- |
| Jede Aufgabe | §1–§4, §10 | – |
| Was ist als Nächstes zu tun? | §9 | – |
| Erase-Kernel, Treiber, Gate, Journal | §5.1–§5.6 | `docs/erasure/14` §3; `10` und `03` jeweils ab „Revision 2“ bzw. §4–§7 |
| Layer (text, vector, graph, checkpoint) | §5.3, §5.7 | `06`/`07`/`08`/`09`: „Revision 2“, dann „Layer-Verhalten“ |
| Löschtests, Crash-/Loom-/Canary-Tests | §5.8 | `14` §5–§6, `13` §4–§7 |
| Store (WAL, Manifest, Compaction, Katalog) | §6 | `docs/contextra-store Implementierungsspezifikation.md`, nur das WP |
| Vector (Persistenz, Concurrency, Format) | §7 | `docs/contextra-vector – Implementierungsspezifikation.md`, nur das WP |
| CI, Gates, Werkzeuge | §8 | `tools/erasure_gates.py`, `.github/workflows/erasure-gates.yml` |
| Warum gilt X? Abweichung von einer Quelle | Anhang A, B | – |
| Prüfauftrag ausführen | Anhang C | – |

**Nie vollständig laden:** `docs/CONTEXTRA_INTERFACE_SPECIFICATION.md` (806 KB, generiert, Stand veraltet), `docs/crate-coupling-report.md` (191 KB, generiert), `docs/erasure/03-contextra-store.md` (58 KB). Zugriff nur per `rg`/`sed -n`.

**Abschnitt herauslösen:** `awk '/^## /{p=($2=="5.")} p' Contextra_MSA_v2_0.md` (Ziffer anpassen; Anhänge: `($2=="Anhang"&&$3=="A")`).

**Größe der Abschnitte** (Token grob als Bytes/3, deutsch/technisch; „Immer lesen“ = §0–§4 + §10):

| Abschnitt | Bytes | ≈ Tokens |
| --- | --- | --- |
| 0. Lesekarte (kontextschonend) | 2724 | 908 |
| 1. Systemüberblick | 712 | 237 |
| 2. Quellen und Rangfolge | 1618 | 539 |
| 3. Crates und Abhängigkeiten | 3797 | 1265 |
| 4. Globale Regeln (gelten in jedem Crate) | 4853 | 1617 |
| 5. Löschkette (Erasure) | 16611 | 5537 |
| 6. Store (`contextra-store`, Ring 1) | 8225 | 2741 |
| 7. Vector (`contextra-vector`, Ring 0) | 6933 | 2311 |
| 8. Gates, CI, Werkzeuge | 3461 | 1153 |
| 9. Arbeitsplan (global, abhängigkeitsgeordnet) | 3104 | 1034 |
| 10. Arbeitsanweisungen für Agenten | 3289 | 1096 |
| Anhang A – Entscheidungsregister und Errata | 4311 | 1437 |
| Anhang B – Änderungen gegenüber MSA v1.1 | 2132 | 710 |
| Anhang C – Prüfaufträge mit festgelegten Folgen | 3052 | 1017 |
| **Gesamt** | **65400** | **21800** |


---

## 1. Systemüberblick

Contextra ist eine eingebettete Speicher- und Abrufmaschine für RAG-Agenten. Rust, Edition 2021, MSRV 1.89, `MIT OR Apache-2.0`, Workspace-Version 0.1.0, 18 Crates plus `benchmarks/contextra-bench` (19 Mitglieder, `default-members` ohne Bench), rund 173 000 Zeilen in `crates/*/src`.

**Produktziele (messbar, gelten für jedes WP).**

- (a) Kein Datenverlust bei Crash an irgendeiner Stelle der Crash-Matrix (§6.6, §5.8).
- (b) Gelöschte Daten sind nach `Gated` in allen Schichten nicht lesbar (G1) und nach `Sealed` physisch nicht auffindbar (G3, Canary `T-DEL-1`).
- (c) Kein Panic im Produktionspfad.
- (d) Deterministische Testbarkeit: Zeit, Zufall und I/O sind injiziert.

---

## 2. Quellen und Rangfolge

**Bei Konflikt gewinnt die niedrigere Zahl.**

| Rang | Quelle | Rolle |
| --- | --- | --- |
| 1 | Dieses Dokument | Norm |
| 2 | `capabilities.toml` | Ring, Klasse, Abhängigkeitsobergrenze. **Muss** mit §3 übereinstimmen (E-12, derzeit ✖) |
| 3 | Code am HEAD samt grünen Tests | Ist-Zustand. Abweichung von Rang 1 ist ein Defekt, außer Anhang A nennt ein Erratum |
| 4 | `docs/erasure/00–14` (Stand Rev. 2, geschrieben gegen `e12873de6`), die zwei Implementierungsspezifikationen | Detailentwurf. `14` geht `00`–`13` vor, `00` §2/§3 gehen den Einzeldokumenten vor |
| 5 | Interface-Spezifikation, Koppelungsbericht | Generiert, nur informativ |

**Quellenstand (ersetzt E-13).** HEAD `d9ebd1192` ist normativ. Die Erasure-Dokumente beziehen sich auf `e12873de6`; danach kamen u. a. A-3 (Layer-Konstanten, `71c7a9135`) und der Rückbau des KV-Cache-Subsystems (`f3c2c01e2`, PR #6094) hinzu. Ein Befund aus einem älteren Stand gilt erst als offen, wenn §5.3/§5.4/§6.2/§7.1 ihn für den HEAD bestätigen.

**Im Repo nicht vorhanden (obwohl referenziert):** `KONZEPT-LOESCHKETTE-FUNDAMENT.md` (Quelle der ADR A–G und der Invarianten I-1…I-12, siehe E-33), `PLAN.md`, `VETOES.md`, `governance/` (E-30). Aussagen daraus gelten nur in der hier und in `docs/erasure/00`, `13` wiedergegebenen Form.

**Größen der Quellen im Repo** (Orientierung für die Lesekarte): `erasure/00` 16 KB, `01` 17, `02` 19, `03` 58, `04` 12, `05` 15, `06` 15, `07` 11, `08` 12, `09` 14, `10` 27, `11` 9, `12` 10, `13` 22, `14` 30; Store-Spezifikation 24 KB; Vector-Spezifikation 27 KB.

---

## 3. Crates und Abhängigkeiten

### 3.1 Crate-Karte (Ist = Soll, ✔ per `Cargo.toml` am HEAD geprüft)

„Obergrenze“ = erlaubte interne `[dependencies]` (auch optionale). `[dev-dependencies]`: Regel 1.

| Crate | Ring/Klasse | LOC | Zweck | Obergrenze (normal) | Unsafe |
| --- | --- | --- | --- | --- | --- |
| `types` | 0/B | 8,3k | Gemeinsame Typen, `ContextraError`, Erase-Zustände | – | verboten |
| `ports` | 0/B | 7,2k | Traits: `Clock`, `Rng`, `Vfs`, `ReadGate`, `ErasureLayer`, `ErasureKernel`, `LedgerSink` | types | verboten |
| `mvcc` | 0/B | 5,3k | Snapshot-Isolation, Sequenzlog, SSI, Pins | ports, types | verboten |
| `sys` | 0/A | 2,2k | OS-Zugriff (mmap, ACL) | – | **Insel** |
| `simd` | 0/A | 3,3k | Distanzkerne | types (optional) | **Insel** |
| `durable-fs` | 0/A | 1,9k | Durable-Write-Referenz, Scrub; **Blatt ohne Abhängige** (E-04) | – (Cap.: ports, sys, types) | verboten |
| `crypto` | 0/A | 6,5k | AES-256-GCM-SIV, HMAC, KDF, `ErasureRoot` | ports, types | verboten |
| `text` | 0/A | 9,8k | BM25/Invertierter Index, `TextErasure` | ports, types (optional) | verboten |
| `vector` | 0/A | 16,9k | HNSW, DiskANN/RaBitQ (experimentell), `HnswErasure` | mvcc, ports, simd, sys, types (alle optional, hinter `ports-compat`) | verboten |
| `graph` | 0/A | 22,4k | CSR-Graph, GraphRAG, `EdgeProvenance` | ports, types | verboten |
| `rank` | 0/A | 5,1k | RRF, Kalibrierung, konforme Vorhersage | ports, types | verboten |
| `store` | 1/B | 38,4k | LSM, WAL, Manifest, SSTable, Löschkernel | crypto, mvcc, ports, sys, types | verboten |
| `checkpoint` | 1/B | 5,0k | Blake3-Checkpoints, Restore | ports, types | verboten |
| `engine` | 3/C | 23,1k | Collections, Hybrid-Suche, Erase-Treiber und -Worker | checkpoint, graph, mvcc, ports, rank, store, text, types, vector | verboten |
| `db` | 3/C | 2,1k | Ring-3-Schicht über `engine` | engine, ports, types | verboten |
| `contextra` | 4/C | 2,1k | Fassade, Builder | db, ports, types | verboten |
| `mcp` | 4/C | 5,5k | MCP-Server `contextra-mcp-server`, Python-Launcher | contextra, crypto, ports, rank, types | verboten |
| `testkit` | Tooling/T | 8,0k | Mocks, `CrashVfs`, Referenzmodell, Konformanzsuite | ports, types | verboten |
| `bench` | Tooling/T | – | Benchmarks | db, ports, rank, store, text, types, vector | verboten |

Ring 2 ist leer und reserviert (E-01).

### 3.2 Regeln

1. `[dev-dependencies]` dürfen zusätzlich `testkit` und niedrigere Ringe enthalten, nie höhere.
2. `testkit` steht in keinem `[dependencies]` eines Produktions-Crates (INV-EMBED-1). CI-Job `release-feature-boundary` (✔ vorhanden) prüft, dass `test-hooks`, `fault-injection`, `memory-only-storage` des Stores nicht im Release-Baum liegen.
3. Eine neue interne Kante ändert gleichzeitig §3.1, `capabilities.toml` und Anhang A.
4. `contextra-index` in `[workspace.dependencies]` ist ein Alias auf `vector`; kein Code nutzt ihn (nur Kommentare und eine `println!`-Zeichenkette in `vector/benches/flush_threshold_amplification.rs:140`). Entfernen in WP-0.4.
5. Gemeinsame Versionen nur über `[workspace.dependencies]`; `cargo deny check` (`deny.toml`) ist Pflicht.
6. Profile: `release` mit `panic = "unwind"`, `lto = "fat"`, `overflow-checks = true`; `release-abort` (`panic = "abort"`) nur für Binaries ohne FFI, nie für Bibliotheken. Der MCP-Server darf `release-abort` nutzen.

### 3.3 Abweichung `capabilities.toml` ↔ §3.1 (E-12, ✖ zu schließen in WP-0.3)

`vector` erlaubt zusätzlich `crypto`; `db` erlaubt zusätzlich checkpoint, crypto, graph, rank, store, sys, text, vector (Ist: nur dev); `engine` zusätzlich crypto, sys; `bench` zusätzlich `sys` und es fehlt `rank` (E-06); `durable-fs` erlaubt ports, sys, types (Ist: keine). `capabilities.toml` bekommt die Obergrenze aus §3.1.

---

## 4. Globale Regeln (gelten in jedem Crate)

**R-1 Lints.** Jeder Crate setzt `[lints] workspace = true` (✔ alle). `unsafe_code`, `unsafe_op_in_unsafe_fn` = deny. Clippy deny: `undocumented_unsafe_blocks`, `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `await_holding_lock`, `await_holding_refcell_ref`, `indexing_slicing`, `exit`, `dbg_macro`. Warn: `cast_possible_truncation`, `cast_sign_loss` (im Store deny, WP-S6). Erlaubte `cfg`-Werte im Workspace-`check-cfg`: `loom`, Feature `test-utils`, Feature `encryption-at-rest` (v1.1 nannte zusätzlich `onnx`; steht nicht mehr drin). Tests dürfen `unwrap`/`expect`, Produktionscode nie.

**R-2 Unsafe.** Alle Crates außer `sys` und `simd` tragen `#![forbid(unsafe_code)]` (✔ geprüft). Die Inseln: nur in benannten Modulen, `// SAFETY:` je Block. `#[allow(dead_code)]`/`allow(unexpected_cfgs)` im Produktionscode nur mit `// allow: <ADR/Issue>` (Gate L2, Soll).

**R-3 Zeit, Zufall, Umgebung, Collections.** `clippy.toml` verbietet **workspace-weit** `Instant::now`, `SystemTime::now`, `rand::thread_rng`, `std::env::var(_os)` sowie die Typen `std::collections::HashMap`/`HashSet` (P28). Zeit nur über `Clock`, Zufall über `Rng`, IDs über `IdGen`; Ausnahme Schlüsselmaterial über `OsKeyMaterialSource`/`KeyMaterialSource`. `ahash` nur, wo die Iterationsreihenfolge nie beobachtbar ist. Journal-/Ledger-Formate: `BTreeMap`/`BTreeSet`.

**R-4 I/O.** Dateizugriff nur über `Vfs` (Port, synchron) bzw. im Store über `StoreIo`. `std::fs`/`tokio::fs` verboten in `store/src` außer `io_seam.rs` und `cfg(test)`, in `vector/src/persistence` außer `durable.rs`, und in Löschpfaden (Gate G-4). `Vfs::rename` hat Default `Unsupported`; produktive Implementierungen müssen `rename` und `sync_dir` liefern.

**R-5 Nebenläufigkeit.** Ring 0 ohne `tokio`; `async` erst in Store-Rändern, Engine, Fassade. Kein `.await` unter Sync-Lock. Kein `block_on` in Erase-/Purge-Pfaden (G-1). Blockierende Datei-I/O in async nur über `spawn_blocking`. Schreibende Locks nie über unbeschränkt lange I/O.

Lock-Rangordnung des Stores (✔ `lsm/lock_order.rs`), Erwerb strikt aufsteigend: 100 `DOCUMENT_INTENT` · 150 `ERASURE_KERNEL` · 200 `COMMIT_MUTEX` · 300 `FLUSH_MUTEX` · 350 `CATALOG_MUTEX` · 400 `STATE_RWLOCK` · 500 `WAL_TRUNCATE` · 600 `SSTABLES_RWLOCK` · 700 `MEMTABLE_SHARDS` · 800 `MANIFEST_MUTEX` · 900 `OBSERVER_REGISTRY` · 950 `BLOCK_CACHE` · 960 `ERASURE_GATE`. Rang 150 wird nie über einen Layer-Aufruf gehalten. Außerhalb des Stores: `collections (RwLock) → kv_locks → embedder (RwLock)`. Laufzeitprüfung offen (WP-S7.3, ✖ kein `RankedMutex`).

**R-6 Fehler.** `ContextraError` ist `#[non_exhaustive]`; neue Varianten nur am Ende (Ist-Ende ✔ `Erased`, geplant `NotReady`). Neuer Code nutzt typisierte Crate-Fehler (`VecError`, `CryptoError`, `IoFault`, …) mit `From` an der Crate-Grenze. Löschkette: Fehlerklassen `Retryable | Permanent | Poisoned | ScopeViolation | Cancelled`. Teilfehler sind `Err`; in Erase-/Purge-Pfaden sind `let _ =`, `unwrap_or_default()`, `.ok();` verboten (G-2). **Fail-closed:** nach fehlgeschlagenem `fsync` ist der Pfad vergiftet; ein zweiter `fsync` gilt nie als Bestätigung; Wiederholung nur durch Neuschreiben in eine neue Datei.

**R-7 Logging.** IDs, Schlüssel, Ziele erscheinen nie im Klartext in Journal, Ledger, Metriken, Spans, nur als gekürzter keyed HMAC. `#[tracing::instrument]` ohne `skip(...)` auf Funktionen mit `id`/`key`/`target` ist verboten (G-3; 22 Altlasten in der Baseline).

**R-8 Durable-Write-Protokoll (alle Primitiven).** (1) Temp-Datei im Zielverzeichnis mit `create_new`. (2) `write_all`, `sync_all`; jeder Fehler ⇒ Temp löschen, `Err`, Handle vergiftet. (3) `rename`, dann `sync_dir` des Elternverzeichnisses; Fehler beim Öffnen/Synchronisieren des Verzeichnisses werden nie verschluckt; schlägt `sync_dir` nach erfolgreichem `rename` fehl ⇒ `Poisoned`, Aufrufer öffnet neu und validiert. (4) Beim Start räumt jeder Besitzer verwaiste Temp-Dateien seines Verzeichnisses (über `Clock`/`Vfs`). (5) Windows ist Tier 2: kein Verzeichnis-Sync, `Guarantee`-Abschlag `DirSyncUnavailable`, `ErasureProfile::Regulated` wird verweigert (E-16, [?]). **Temp-Name (E-09, Soll):** `.{ziel}.{pid}.{zähler}.tmp`, Zähler = prozessweiter `AtomicU64`, kein Thread, kein Zufall. **Ist ✖:** `store/io_seam.rs:48` nutzt `{datei}.tmp.{pid}`, `durable-fs/vfs.rs:139` nutzt `.{stem}.tmp.{pid}.{zähler}`, Vector hat Zufall im Namen laut Spezifikation. Umstellung in WP-0.5.

**R-9 Prüfsummen.** Alle Prüfsummen in Store, WAL, Manifest, Ledger, Vektorformat: CRC-32 IEEE über `crc32fast` (E-11; ✔ von `store` und `durable-fs` genutzt). Vollintegrität großer Dateien: `blake3`. Die falsche Angabe „CRC32C“ steht am HEAD nur noch in `crates/contextra-store/README.md`.

---

## 5. Löschkette (Erasure)

### 5.1 Modell in Kürze

- **Stufen.** G0 Retire (`delete` = Tombstone) · G1 Gate (Lesesperre, RAM) · G2 Schlüsselvernichtung (Stufe 2, opt-in, Entwurf) · G3 physisches Entfernen. `purged = true` erst bei G3 mit erfüllten Pflicht-Layern.
- **Zustandsautomat.** `Requested → Planned → Gating → Gated → [KeysDestroying] → Purging → Verifying → Sealed → Completed`; Seitenzustände `Failed{layer,attempt}`, `Quarantined{layer}`. Ab `Gated` kein Rücksprung. `Quarantined` verlässt nur `resume` (A-8), nie automatisch.
- **Scope.** `EraseTarget::{Document, Collection, Tenant}` (Subject nur *Regulated*, ADR E). `KeyScope` = Regeln `(Prefix|Exact, In|Out)`, längste Regel gewinnt, `Exact` vor `Prefix`; `Collection("default")` wird abgelehnt (K-5); Mandantenpräfix immer mit `:`; leerer Scope ⇒ `InvalidTarget` (K-56).
- **Gate (fail-closed).** Lesen gesperrter Schlüssel = „nicht vorhanden“; Schreiben ⇒ `Erased`. Compaction: `Restricted` behält, `Erased` darf verwerfen. Nicht ladbarer Zustand wirkt wie `Restricted` (I-7). Gate endet nach Witness 0 (I-9); der Erasure-Floor (A-5) bleibt.
- **Schichtreihenfolge im Treiber.** RAM-Caches (`ErasureObserver`) → Text → Graph → Vektor → Store (`purge_store`) → Checkpoints → Verifikation. `expected_layers` = genau die registrierten Layer.
- **Profile.** *Regulated* (Pflicht-Layer, externe Wurzel, externes Ledger, `StaticKeyMaterialSource` verboten), *Standard*, *Embedded* (Löschung läuft, fehlende Layer werden als `exceptions += FeatureDisabled{layer}` ausgewiesen, Garantie bleibt `gated`).
- **Invarianten** (Texte nur in `13` §11, Quelle fehlt im Repo, E-33): I-1 Reihenfolge · I-2 Write-Ahead · I-3 Kein Wiederaufleben · I-4 Idempotenz · I-5 Epoche monoton · I-6 Witness-Ehrlichkeit · I-7 Fail-closed · I-8 Scope-Treue · I-9 Gate-Lebensdauer · I-10 Lock-Ordnung · I-11 Pins blockieren G1 nie · I-12 Kein Klartext. Jede hat mindestens einen Test, der bei Verletzung rot wird.

### 5.2 Entscheidungen ADR A–G (übernommen aus v1.1, Quelle: Konzept, E-33)

| ADR | Entscheidung |
| --- | --- |
| A | Erase revidiert D-06; `delete` bleibt Retire (Tombstone), Erase ist eigener Pfad |
| B | Scrubbing nicht in G3 im Default; optional später über Kante `store → durable-fs`, nur nach §3.2 Regel 3 und Anhang-A-Eintrag |
| C | Gate-Zeiger `RwLock<Arc<_>>`, kein neues Crate |
| D | SLA 24 h, Koaleszenzfenster 5 min (Änderung nur per Anhang A nach Lastmessung) |
| E | Subjekt-Erase (`__subj:`) nicht im Default, nur *Regulated* nach WP-E5 |
| F | `delete` löst nicht automatisch `erase(Document)` aus, nur *Regulated* |
| G | Schlüsselableitung `PerDocument` Default, `PerSubject` in *Regulated* |

### 5.3 Umsetzungsstand je Crate (HEAD `d9ebd1192`)

| Crate | Stand | Offen |
| --- | --- | --- |
| `types` | ✔ Zustände, Garantien, Fehler | `ErasureError::LayerMissing`, `ContextraError::NotReady` (jeweils am Ende) |
| `ports` | ◐ `erasure.rs` (`KeyScope`, `ReadGate`, `ErasureLayer`, `ErasureKernel`, …) ✔; **A-3 ✔** (`LAYER_TEXT`, `LAYER_CHECKPOINTS`, `WitnessLayer::text()/checkpoints()`) | A-1 `register_collapse`, A-4 `begin_purge`, A-8 `resume` |
| `store` | ◐ Kernel, Gate, Filter, Journal, Ledger, Pending-Delete, `purge_lsm` in `store/src/erasure/` (7 Dateien) ✔ | K-33/A-2, K-45–K-47 (Kernel-Teil), K-50, K-51, Manifest-Zeuge |
| `mvcc` | ✔ Snapshot-Entzug, Pin-Alterung (04) | K-55 (Ergebnis von `reap_expired` erfassen) |
| `crypto` | ◐ `ErasureRoot`/`ErasureLabel` ✔, **ohne Verwender**; K-40 ✔ behoben | Wurzelquelle (A-2), Rückbau neu zuzuschneiden (E-32), Stufe 2 |
| `text` | ◐ `TextErasure` ✔, **nicht verdrahtet** | Registrierung, Konformanzsuite voll, C8-Filter, `LAYER_TEXT` nutzen |
| `vector` | ◐ `HnswErasure` ✔, **nicht verdrahtet**, kein Konformanztest | Registrierung, `TST-VEC-CB-1`, Datei-I/O (E-31) |
| `graph` | ✖ kein Layer, kein Zeuge (`EdgeProvenance` vorhanden) | `GraphErasure`, `LAYER_GRAPH` (S1) |
| `checkpoint` | ◐ Restore-Sperre nur auf Meta-Schlüssel ✔ | Layer, Taint, Floor (A-5), Zeuge `Checkpoints` (S1) |
| `engine` | ◐ `erasure/` (10 Dateien), Worker ✔; `register_erasure_layer` (`api.rs:218`) **ohne Produktivaufrufer** | K-45–K-49, K-52, K-53, K-56–K-58, `required_layers`, `erase_resume` |
| `contextra` | ◐ Builder kennt `erasure_config`/`erasure_profile` ✔ | Layer-Verdrahtung, Startreihenfolge (K-48, K-58) |
| `mcp` | ✔ `contextra_erase`, `_erase_status`, `_erase_dry_run` | Ergänzungen aus `12` Rev. 2 |
| `testkit` | ✔ Mocks, Modell, Konformanz, Loom, `CrashVfs` (`crash_vfs.rs`) | `MockLayer`-Fehlerinjektion, C8/IDEM, CP-19…30 |

### 5.4 Offene Befunde (verbindlich; Details und Belege in `14` §2)

Schwere: **S1** Löschzusage falsch oder Job hängt (blockiert Auslieferung) · **S2** Härtung · **S3** Hygiene. Tests zu S1-Befunden müssen vor der Korrektur rot sein.

| ID | Sev | Kurzbefund | Korrektur | WP |
| --- | --- | --- | --- | --- |
| K-33 | S1 | Ledger-/Log-Schlüssel nicht persistent: `open_early` zieht aus `fill_key_bytes` (`mod.rs:344`), Recovery nutzt `OsKeyMaterialSource` ⇒ nach Neustart `LedgerCorrupt`. **Auslieferungsstopp** | A-2 | E1 |
| K-45 | S1 | Treiber ruft `record_layer` vor `purge_store`; `record_layer` verlangt `Purging`, das nur `purge_store` setzt ⇒ `InvalidTransition` | A-4 `begin_purge` | E0 |
| K-46 | S1 | `fail()` lehnt `Failed→Failed` ab, Treiber verwirft das Ergebnis (`let _ =`); Quarantäneschwelle doppelt (Kernel hart 10, Engine konfigurierbar) | eine Schwelle im Kernel (E-34), Ergebnis auswerten | E0 |
| K-47 | S1 | Verifikationsreste werden nie nachgearbeitet: Layer-Cursor bleibt `None` | `fail(.., Some(l), ..)` löscht `layer_cursor[l]` (live und Replay) | E0 |
| K-48 | S1 | Layer unverdrahtet ⇒ `Sealed` mit leerem Lauf, Garantie wirkt zu hoch (I-6) | `required_layers`, Builder (A-6) | E3 |
| K-49 | S1 | `on_gate`-Fehler werden nur geloggt | Layer „unmasked“ ⇒ Retry, kein `Verifying`; C8-Ergebnisfilter | E2 |
| K-51 | S1 | WAL-Zeuge liest nur aktives WAL, ignoriert Lesefehler, Rohbytesuche versagt bei Verschlüsselung | alle Segmente, Lesefehler ⇒ `ScanFailed`, Technik `Structural` | E4 |
| K-54 | S1 | `restore_checkpoint` prüft nur aktive Gates; nach `Completed` stellt ein alter Checkpoint Gelöschtes wieder her | A-5 Erasure-Floor, Pflicht-Layer Checkpoint | E5 |
| K-57 | S1 | Ein Job monopolisiert den Worker | ein Tick je Job und Pass, `DriveOutcome::Progress`, SLA-Jobs zuerst | E6 |
| K-58 | S1 | Startfenster: Layer-Lesepfade vor `mask_layers` unmaskiert | Startreihenfolge §5.6, `NotReady` | E2 |
| K-50 | S2 | Encoder in `journal.rs` mit `_ =>`-Fallbacks (falsche Persistierung bei neuen Enum-Werten) | fallibel, G-9 | E6 |
| K-52 | S2 | `ErasureId` aus Uhr-gesätem PRNG | A-7 (`KeyMaterialSource`) | E6 |
| K-53 | S2 | Observer-Zustellung nicht durabel, untracked `tokio::spawn` | `TaskTracker`, Replay `Gated` beim Start | E2 |
| K-56 | S2 | `tenant_of` fällt auf `SYSTEM` zurück | leerer Scope ⇒ `InvalidTarget` | E7 |
| K-59 | S2 | G-2-/G-3-Baseline wächst (Engine-Löschkette, CRUD) | Verstöße beheben, Baseline schrumpft | E7 |
| K-55 | S3 | `reap_expired`-Ergebnis verworfen | erfassen, in `exceptions`/Metrik | E7 |

**Weitere Altbefunde aus `00` §2, am HEAD nicht neu geprüft:** K-7/K-41 (Rückbau `legacy-erasure`, siehe E-32), K-10 (Manifest/Recovery auf `StoreIo`, hängt an WP-S1), K-11 (HNSW `save_internal` verschluckt Fehler, bestätigt durch `persistence/atomic.rs`), K-15 (WAL-Schnitt gesperrt bis `TST-STORE-ROT-1` grün), K-34 (= A-1), K-43 (`instrument` ohne `skip`, `delete.rs:14`), K-44 (`drop_collection` nicht atomar ⇒ wird `erase(Collection)`), K-25 (Legacy-Symbol nur in einem Store-Test).

### 5.5 Nachträge A-1 … A-8

| ID | Inhalt | HEAD | WP |
| --- | --- | --- | --- |
| A-1 | `ProgressSink::register_collapse(&self, keys: &[(Vec<u8>, Seq)])` und `ErasureKernel::register_collapse(id, keys)`; Treiber liefert `KernelProgress{kernel,id}` | ✖ nur `StoreErasureKernel::register_collapse` (`erasure/mod.rs:834`) | E1 |
| A-2 | `ErasureRootProvider{ root(); is_external() }` in `store/src/key_source.rs`; `FileErasureRoot` (`<erasure_dir>/root.key`, 0600, `write_atomic`+`sync_dir`, nur beim allerersten Start erzeugt), `DerivedErasureRoot` (aus `KeyManager`), `StaticErasureRoot` (Tests); `open_early(&dyn ErasureRootProvider)`; Journal/Ledger ohne Wurzel ⇒ fail-closed, **nie** Neuerzeugung; *Regulated* + nicht externe Wurzel ⇒ Start verweigert; Schlüssel nur über `ErasureRoot::derive()` | ✖ (`ErasureRoot` in `crypto` ✔, keine Verwender) | E1 |
| A-3 | Layer-Konstanten `LAYER_TEXT="TextIndex"`, `LAYER_CHECKPOINTS="Checkpoints"` | **✔** (`ports/src/scanner/layer.rs`) | – |
| A-4 | `ErasureKernel::begin_purge(id)`, idempotent, aus `purge_store` extrahiert; setzt `Purging` (`PurgeStart`), Gate `Restricted → Erased`; Treiber ruft es vor dem ersten Layer jeder Runde; Journal-Format unverändert | ✖ | E0 |
| A-5 | Persistenter Erasure-Floor aus `completed_summaries` `(erasure_id_hmac, plan_hash, gated_seq, scope_digest)`; `restore_checkpoint` ⇒ `Erased`, wenn `gated_seq > meta.seq_no` und Checkpoint nicht `Rewritten`; Vergleich über `gated_seq` (Erase erzeugt keine `TxId`) | ✖ | E5 |
| A-6 | `ErasureConfig.required_layers: BTreeSet<WitnessLayer>`; `accept` prüft `⊆ registriert`; *Regulated* ⇒ `{Text, Vector, Graph, Checkpoints}` (soweit gebaut), sonst `LayerMissing`; andere Profile: Ausweis als Ausnahme | ✖ | E3 |
| A-7 | `ErasureId`-Bytes über `KeyMaterialSource` (OS-CSPRNG); `SeededRng` nur Tests; Jitter darf PRNG bleiben | ✖ (`engine/erasure/mod.rs:93-98`) | E6 |
| A-8 | `ErasureKernel::resume(id)`, `Contextra::erase_resume(id)`; einziger Weg aus `Quarantined`; **kein** MCP-Werkzeug | ✖ | E6 |

### 5.6 Treiberablauf und Startreihenfolge (normativ)

```text
drive_job(job):
 1 Requested|Planned|Gating → publish_gate → mask_layers (Fehler ⇒ unmasked, Retry) → Gated
 2 Quarantined              → return Quarantined         (nur der Kernel-Zustand zählt)
 3 Gated|KeysDestroying|Failed|Purging:
      kernel.begin_purge(id)                               (A-4)
      EIN Tick: nächster fälliger Layer (Text → Graph → Vektor), sonst Store, sonst Checkpoint
      record_layer / StoreProgress                         → return Progress
 4 alle erledigt → Verifying: Zeugen sammeln
      Rest > 0 → fail(layer, Retryable)  (Kernel verwirft Layer-Cursor)
      sonst    → seal
 5 Sealed → complete → Observer(Completed)
Jeder Fehler: fail(...) auswerten (Quarantined | Failed | Err), nie verwerfen.
```

**Fehler- und Quarantäneregeln.** Eine Schwelle (`StoreErasureConfig.quarantine_after_attempts`, Default 10, neues Feld); `fail` erreicht sie ⇒ `Quarantined{layer}` (bei `layer=None`: `"job"`). `Ok(Quarantined)` ⇒ `DriveOutcome::Quarantined`; `Ok(Failed)` ⇒ `Retry(backoff)`; `Err(JournalPoisoned)` und jeder andere `Err` ⇒ `Poisoned` + `error!`, Gate bleibt. `Failed→Failed` bleibt verboten (Treiber ruft vorher `begin_purge`).

**Layer-Pflichten.** Jede `ErasureLayer::purge` ist idempotent (Neustart ab Cursor `None`, `CONF-LAYER-IDEM-1`); jede Trefferliste wird vor Rückgabe über `ReadGate` gefiltert, unabhängig von `on_gate` (C8, `CONF-LAYER-READ-1`); `on_gate` ist Beschleunigung, nicht Sicherheitsgrenze. Lesepfade liefern `NotReady`, bis Kernel offen, alle Layer registriert und `mask_layers` für alle offenen Jobs erfolgreich war.

**Zeugen.** `LayerScanner::scan`: nicht lesbare Einheit ⇒ `Err(ScanFailed)` ⇒ `Retryable` am Layer, nie `remaining = 0`. WAL: aktives WAL, `pending_sealed_wals`, jedes `wal-*.log`; verschlüsselt ⇒ Technik `Structural`. `NotMeasured` senkt die Garantie (I-6), blockiert `Sealed` nicht, wenn `required_layers` erfüllt sind.

**Observer.** Idempotent, auf `TaskTracker`, `stop()` wartet bis `shutdown_grace`; beim Start erhält jeder offene Job ab `Gated` erneut `ObservedPhase::Gated`. Zustellfehler zählen (`observer_failures`), blockieren nur *Regulated* mit `required_observers`.

**Planung.** Ein Tick je Job und Pass, `cfg.max_jobs_per_pass`, SLA-gefährdete Jobs (≥ 80 %) zuerst, dann ältestes `first_seen`; Leerlauf-CPU 0. SLA-Überschreitung ⇒ `error!` und Laufzeitfeld `sla_breached_at` (kein Journal-Formatwechsel).

**Startreihenfolge (ersetzt `00` §5, Satz 1).**

```text
1 ErasureRootProvider.root()        fehlt Wurzel bei vorhandenem Journal/Ledger ⇒ Start verweigert
2 StoreErasureKernel::open_early    Journal, Ledger, Pending-Delete, Gate fail-closed
3 LsmStorage-Konstruktor            Replay-Filter und Gate VOR WAL-Replay (K-13)
4 Layer registrieren                Text → Graph → Vektor → Checkpoint
5 mask_layers + Observer-Replay     alle offenen Jobs
6 Lesepfade frei                    NotReady endet
7 Worker starten
```

### 5.7 Datenort-Matrix (Prüfmaßstab für „lückenlos“)

Eine Zeile ist erst **geschlossen**, wenn G1, G3, Zeuge und Crash-Test vorhanden sind. `purged = true` setzt Zeilen 1–16 geschlossen voraus; 17–20 sind „verhindert“ oder in `ErasureStatus.exceptions` ausgewiesen.

| # | Ort | Lücke am HEAD |
| --- | --- | --- |
| 1 | Commit-Pfad (Group-Commit in flight) | Gate in `prepare_commit_ops` **und** vor `wal.append_batch` (K-1); Test `TST-STORE-ROT-1` Pflicht |
| 2 | WAL aktiv | Zeuge defekt (K-51) |
| 3 | WAL versiegelt / `pending_sealed_wals` | Zeuge sieht es nicht (K-51) |
| 4 | MemTable (aktiv, immutable) | ✔ (`MemTable::retain`) |
| 5 | SSTables inkl. Index-/Bloom-Blöcke | ✔; [?] `sstable_overlaps_filter` (`purge_lsm.rs:85`) darf nicht nur Min/Max nutzen (`TST-STORE-SST-3`) |
| 6 | Block-Cache | ✔ (`evict_file`) |
| 7 | Manifest (Min/Max-Schlüssel je SST, alte Generationen) | **kein Zeuge `Manifest`**; [?] `TST-STORE-MAN-2` (E4) |
| 8 | MVCC-Versionen, Snapshots, Pins | ✔; K-55 |
| 9 | Journal-Plan-Blob, Pending-Delete, `*.erase.tmp` | ✔ (Zielbytes nach `Completed` entfernen, I-12) |
| 10 | Ledger | K-33 |
| 11 | Text-Index (`i:`, `plb:`, `resident_index`, `meta:stats`) | nicht verdrahtet (K-48) |
| 12 | HNSW (RAM, Quantisierung, `save`/mmap, `deleted_nodes`) | nicht verdrahtet; [?] `TST-VEC-CB-1` Codebook |
| 13 | Graph (`__graph:entity/edge/community`, tenantweit K-38) | **Layer fehlt (S1)**, Bereinigung über Provenienz |
| 14 | Checkpoints, Hardlink-Klone, Orphans | **Layer fehlt (S1)**, K-54 |
| 15 | ~~KV-Cache / Salt-Tresor~~ | **entfällt als Datenort** (KV-Cache-Subsystem entfernt, E-32); Stufe 2 bleibt Entwurf |
| 16 | Engine-Caches (Epoch-, Antwort-, Embedding-, Query-Plan-Cache) | ✔ `caches.rs`; Replay K-53 |
| 17 | Logs, Traces, Metriken | nicht entfernbar ⇒ **verhindern** (G-3, K-59) |
| 18 | Antworten in Flug (MCP-Cursor, Streams) | [?] `TST-MCP-CUR-1`, Lücke dokumentieren |
| 19 | Prozessspeicher (Heap, Swap, Core-Dumps) | dauerhafte Garantiegrenze, in Doku und Ledger-Scope-Text |
| 20 | FS-Snapshots, Backups, SSD-Wear-Leveling | `exceptions += OutOfScopeExternalCopies`; nur Stufe 2 wirkt |

### 5.8 Abnahme und Tests der Löschkette

**Globale DoD.** (1) Jede Invariante I-1…I-12 hat einen rot werdenden Test; Mutationstests über `mutants-guarantees.toml`. (2) Crash-Matrix CP-1…CP-18 (`03` §9) und CP-19…CP-30 (`14` §5) über `CrashVfs`: nach `accept`, Gate-Intent, Layer-Seiteneffekt ohne Fortschritt, Ledger ohne `Sealed`, Floor-Aufbau, WAL-Schnitt, Verifikation, Ledger-Torn-Tail, Wurzel-Erstanlage, Layer-Datei-Rewrite, Checkpoint-Halbzustand, zweiter Crash in Recovery. (3) Loom (`loom_erasure`): Schreiber ∥ Gate-Veröffentlichung, `complete` ∥ Lese-Epoche, Layer-`purge` ∥ `restore_checkpoint`. (4) Canary `T-DEL-1` (`crates/contextra/tests/deletion_e2e.rs`): eindeutige Bytefolge schreiben, `erase(Document)`, alle Dateien durchsuchen, 0 Treffer, in **Klartext- und verschlüsseltem WAL-Modus**. (5) Leerlauf-CPU des Workers 0; Lesepfad ohne aktive Gates = ein Acquire-Load. (6) Matrix §5.7 Zeilen 1–16 geschlossen. (7) Gates G-1…G-9 grün, Baseline ohne Eintrag für `engine/src/erasure/`. (8) Ohne Pflicht-Layer nie `purged = true`.

**Neue Rot-Grün-Tests (`14` §6):** `TST-DRV-ORDER-1`, `-FAIL-1`, `-FAIL-2`, `-RESET-1`, `-GATEERR-1`, `-SCHED-1` · `TST-ENG-WIRE-1`, `-NOTREADY-1`, `-ID-1` · `TST-JRN-ENC-1` · `TST-STORE-WIT-1`, `-WIT-2` · `TST-STORE-ROOT-1…5` · `TST-CKPT-RESTORE-3` · `TST-OBS-REPLAY-1` · `TST-GATES-ENC-1`. Treibertests laufen gegen `MockErasureKernel` **und** den echten `StoreErasureKernel` (Differential-Test `TST-MODEL-DIFF-1`).

---

## 6. Store (`contextra-store`, Ring 1)

### 6.1 Invarianten (normativ)

- **INV-M1** Der durable Manifest-Zustand ist die Reduktion aller bestätigten Einträge; Rollover verliert nichts (Live-Set, Dead-Set, Rollback-Marker, WAL-HWM).
- **INV-M2** Nur `Manifest` bildet seinen Zustand (`ManifestState`); kein Aufrufer liefert Snapshots.
- **INV-M3** Ein `append_batch` ist ein Frame mit einer Prüfsumme, ganz oder gar nicht sichtbar.
- **INV-M4** Fehler nach Teilschreibung ⇒ bewiesene Rücknahme (Truncate auf `committed_len`, `sync_data`) oder Poisoning. Kein Retry nach `fsync`-Fehler.
- **INV-M5** Unbekannte `.sst` werden nie gelöscht und nie still ignoriert (Quarantäne oder Boot-Fehler).
- **INV-IO1** Jede Dateioperation im Store-Produktionscode läuft über `StoreIo`/`Vfs` (Ausnahme `io_seam.rs::RealFs`, `cfg(test)`).
- **INV-C1** Alle SSTable-Katalogänderungen (Flush-Publish, Compaction-Swap, Rollback, Erase-Purge, Rollover) laufen unter `catalog_mutex` (350). **INV-C2** `sstables.write()` nur für den RAM-Swap (< 100 µs), nie über I/O.

### 6.2 Befundstatus am HEAD

| ID | Sev | Befund | HEAD | Beleg / Folge |
| --- | --- | --- | --- | --- |
| F1 | P0 | Rollover verliert `WalCheckpoint`, Dead-Records | ◐ | `ManifestState` (`manifest/state.rs`) mit `wal_hwm`, `rollover(&self)` ohne Parameter ✔; Regressionstest PA-02 |
| F2 | P0 | Snapshot-Race Rollover ∥ Flush | ◐ | `catalog_mutex` in Flush/Compaction/Erase ✔; Loom-Abdeckung PA-03 |
| F3 | P0/P1 | VFS-Umgehung | ◐ | `wal/file.rs` auf `VfsFile` ✔. **Noch `tokio::fs`:** Shim `wal/mod.rs:66`, `lsm/engine.rs:398,463,479`, `scanner/wal.rs:71`, `scanner/sstable.rs` ⇒ WP-S1 offen (PA-09 beantwortet) |
| F4 | P0 | Fehler nach `rename` hinterlässt toten Handle | PA | `poisoned` in `manifest/rollover.rs`; Reihenfolge PA-04 |
| F5 | P0 | Append ohne Torn-Frame-Schutz | ◐ | `AppendFile::append_durable`, typisiertes `IoFault` ✔; Rollback bei Teilschreibung PA-05 |
| F6 | P1 | `load` maskiert I/O-Fehler als „leer“ | PA | PA-06 |
| F7 | P0 | Orphan-SST still übersprungen | ✔ | `RecoveryPolicy{Quarantine (Default), Fail, Adopt}` (`lsm/config.rs:147`); der Feldname `unmanifested_sst` aus v1.1 kommt im Code nicht vor |
| F8 | P1 | Fehlerklassifikation, globale Fault-Injection | ◐ | `FaultVfs` vorhanden; **globale `static`-Schalter** `FAIL_APPEND_FOR_TX` u. a. in `wal/mod.rs:345ff` ⇒ WP-S5 |
| F9 | P1 | `fsync` unter `sstables.write()` | PA | PA-07 |
| F10 | P2 | Doku-/Lint-Drift | ◐ | `#![deny(dead_code)]` ✔ (`lib.rs:31`); 2× `allow(dead_code)` im Produktionscode; „CRC32C“ nur noch im README |
| F11 | P2 | Verlustbehaftete Pfade/`as u32` | ◐ | `SstName` ✔; `as`-Konversionen im Format PA |
| F12 | P2 | Lock-Rang nicht erzwungen | ✖ | kein `RankedMutex` |

### 6.3 Manifest-Format v2 (Soll; Version 2 ist Ist: `CURRENT_MANIFEST_VERSION = 2`, `manifest/core.rs:27`)

```text
Header := Magic "CMAN" | Version 2
frame  := len:u32le | crc:u32le | tag:u8 | body      (crc = CRC-32 IEEE über tag||body)
tag 1  Batch    { n:u32, entries:[Entry] }           (ein Frame = atomarer Batch)
tag 2  Snapshot { live:[(SstName,max_tx)], dead:[SstName], rollbacks:[u64], wal_hwm:Option<[u8;32]> }  (erster Frame nach dem Header)
Entry  := Add{name,max_tx} | Replace{removed,added,added_max_tx} | Remove{name} | RollbackComplete{target_tx} | WalCheckpoint{hmac}
SstName:= u16 len | utf8 (nur Dateiname, kein Verzeichnis)
```

Längen über `u32::try_from`; `Replace.rank` entfällt. Abweichungen des Codes davon sind Defekte oder Errata (Abgleich: `rg "tag" crates/contextra-store/src/manifest/entry.rs`).

**Protokolle.** *Append:* gegen Kopie des Zustands validieren → Frame kodieren → `append_durable` → erst danach Zustand übernehmen. *Rollover:* Temp nach R-8 mit Header+Snapshot, `sync_all`; Fehler bis hier: Temp löschen, Handle gültig; ab `rename` jeder Fehler ⇒ `poisoned`. Auslöser ist `maybe_rollover` im Manifest selbst. *Laden:* `io.exists(path)?` (Fehler propagieren); Torn Creation (`len < 5`, Präfix passt) ⇒ atomar neu initialisieren; Torn Tail ⇒ vor dem ersten Append auf `last_valid_offset` kürzen; CRC-Fehler vor dem letzten Frame ⇒ `ManifestCorrupt{offset}`. *Migration v1→v2:* Reduktion in `ManifestState`, Snapshot über Rollover, `MANIFEST.v1.bak` bis zum nächsten erfolgreichen Flush, kein Downgrade. Alte Generationen werden sonst nicht aufbewahrt (E-22).

**Orphans.** `Quarantine`: nach `<data>/quarantine/<name>.<ts>` verschieben (`rename`, `sync_dir`, Zeit über `Clock`), `store_orphan_sst_total`, `BootReport.orphans`. `Fail` (Regulated): `UnmanifestedSstable{names}`. `Adopt`: nur explizit. Dead-Set-Dateien dürfen automatisch gelöscht werden.

**Katalog-Publish (WP-S4).** Unter `catalog_mutex`: (a) Kontiguität prüfen, (b) `manifest.append(Replace)` ohne `sstables`-Sperre, (c) `sstables.write()` nur für `retain`/`insert`/`sort`, (d) loslassen, (e) Dateien über `pending_delete` entsorgen. `OutputGuard` entfernt Ausgabe im `Drop`.

### 6.4 Arbeitspakete Store (Reihenfolge S1 → S2 → S3 → S4, dann S5–S7; S8 begleitet)

| WP | Inhalt | Akzeptanz |
| --- | --- | --- |
| S1 | `VfsFile` additiv um `write_at`, `append`, `read_at`, `set_len`, `sync_data`, `sync_all`, `len` (PA-08); WAL auf `StoreIo`/`Box<dyn VfsFile>`; Shim `wal/mod.rs:66` weg, Loom nutzt `MemVfs`; Lint `disallowed_methods` für `tokio::fs::*`, `std::fs::*` | `rg "tokio::fs\|std::fs" crates/contextra-store/src` nur `io_seam.rs` und `cfg(test)`; `CrashVfs` fängt jeden Write/Sync/Rename von WAL und Manifest |
| S2 | Manifest v2 nach §6.3 verifizieren, Lücken schließen; `extract_high_water_mark`/`reconstruct_valid_sstables` durch `ManifestState` ersetzen, falls noch vorhanden | Regressionstests F1 (HWM überlebt Rollover), F2 (Barrier), F4 (Fault nach `rename` ⇒ Poison, Neustart findet alles), F5 (ENOSPC im Frame), F6 (EACCES ⇒ Fehler) |
| S3 | Orphan-Politik gegen §6.3 prüfen | SST ohne Eintrag: Default Quarantäne+Report, `Fail` ⇒ Fehler, Datei bleibt in jedem Fall erhalten |
| S4 | Katalog-Serialisierung nach §6.3, Haltedauer messen | `loom_catalog_publish` um Flush ∥ Compaction ∥ Rollover erweitern; Manifest-Live-Set == RAM-Katalog nach jedem Interleaving |
| S5 | `PoisonCause` aus `IoFault`; Fault-Injection instanzgebunden (`FaultVfs{plan}`, Seed) statt globaler `static`; `let _ = remove_file(tmp)` in `wal/hmac.rs` ⇒ `cleanup_tmp` mit `warn` | keine globale Fault-`static` mehr |
| S6 | `SstName`/`SstId` validiert; `as` ⇒ `try_from`; `cast_possible_truncation` im Store `deny`; Pfade nur an der `StoreIo`-Grenze | Clippy grün |
| S7 | 2× `allow(dead_code)` löschen oder `// allow:` begründen; `RankedMutex`/`RankedRwLock` (nur `debug_assertions`, `task_local`-Stapel); CRC-Doku (README); Release-Feature-Prüfung | Gate L2, Lock-Order-Tests |
| S8 | Crash-Matrix (§6.6), Referenzmodell, Fuzz, Mutation | siehe §6.6 |

### 6.5 Feature `encryption-at-rest` (E-14, ✔ deklariert)

`encryption-at-rest` und `legacy-wal-key` sind in `store/Cargo.toml` deklariert, `default = []`. Helper `load_or_create_wal_uuid` (`wal/hmac.rs`) und `parse_mmap_slice` (`wal/replay.rs`) hängen an derselben `cfg`-Bedingung wie ihre Aufrufer, nie an `cfg(test)` allein. **Store-Feature-Matrix (blockierend, Soll):** `default` · `encryption-at-rest` · `encryption-at-rest,legacy-wal-key` · `fault-injection` (Tests) · `memory-only-storage` (Tests). PA-01 (`test_wal_rotate_and_seal_readonly_guarantee` als Nicht-Root) bleibt offen.

### 6.6 Crash-Matrix, Referenzmodell, Fuzz (Store)

Mit `CrashVfs` für WAL und Manifest: für jede Operationsnummer N einer festen Sequenz (Puts, Flush, Compaction, Rollover, Rollback) bricht der VFS nach der N-ten Mutation ab, danach Recovery. Prüfung: (a) jeder bestätigte Commit vorhanden (Präfix), (b) kein gelöschter Schlüssel taucht auf, (c) Store öffnet immer, (d) `BootReport.orphans` leer, außer bewusst ungelistet. `proptest_model_based.rs` um `Rollover`, `CompactNow`, `FlushNow`, `Crash+Recover` erweitern (Orakel `BTreeMap`). `cargo-fuzz`: `Manifest::load` und WAL-Frame-Decoder (kein Panic, kein OOM). `mutants-guarantees.toml` um INV-M1…M5, INV-C1, INV-VEC-* ergänzen.

---

## 7. Vector (`contextra-vector`, Ring 0)

### 7.1 Befundstatus am HEAD (alle ✖, per Grep belegt)

| ID | Befund | Beleg | WP |
| --- | --- | --- | --- |
| F-01 | Feature-Modell vs. Klasse A | `default=["std","ports-compat"]`; Feature `simd` fehlt; leere Features `inspect`, `graph` | V1 |
| F-02 | Mehrere Atomic-Write-Pfade, ignorierte Fehler | `persistence/atomic.rs`: `if let Ok(dir)`, `let _ = dir.sync_all()`; kein `AtomicFileWriter` | V2 |
| F-03 | Dateiformat ohne Integrität | keine `header_crc`/`blake3_full` | V3 |
| F-04 | mmap-Lebenszyklus, öffentliche Felder | `persistence/mmap.rs` `pub mmap/header/path` | V4 |
| F-05 | Lock-Hierarchie nur Kommentar | kein `Ordered<..>` | V5 |
| F-06 | Panic-Safety des Index-Zustands | kein `MutationGuard` | V6 |
| F-07 | Stringly-typed Fehler | kein `enum VecError` | V6 |
| F-08 | Persistierte Snapshots enthalten erased Vektoren | Lücke dokumentiert | V7 |
| F-09 | Testnamen, Dubletten, keine Crash-Tests für HNSW | `campaign_j*` u. a. | V8 |
| F-10 | Doku-Drift | README `simd`, `#[allow(deprecated)]` | V1 |

### 7.2 Entscheidungen

- **V-D1 (E-04, E-31)** Keine Einbindung von `durable-fs`. Interne Primitive `persistence::durable::AtomicFileWriter` (R-8); Crash-Test über `FsOps`-Trait mit `RealFs`/`FaultFs` im Vector-Crate sowie die testkit-Konformanzsuite (§8.3).
- **V-D2** `VecError` bleibt crate-eigen; `From<VecError> for ContextraError` in `compat/`.
- **V-D3 (E-16)** Windows Tier 2, Verzeichnis-Sync dort No-op mit Abschlag `DirSyncUnavailable`.
- **V-D4 (E-17)** DocId-Breite per Compile-Feature `docid-128` (✔ Feature vorhanden), im Header gespeichert (✖), Abweichung ⇒ `VecError::Format(DocIdWidth)`.
- **V-D5** Suche standardmäßig über RCU (`Arc<CoreSnapshot>`, keine Read-Guards in der Traversierung). Contention-Benchmark ist **Abnahmekriterium**: p99 nach Änderung ≤ 1,5× Einzel-Writer-Wert bei 1/4/8 Writern; verfehlt ⇒ V5 nicht abgeschlossen, Ursache im Bericht, keine Lock-Variante als Ersatz.
- **V-D6 (E-05)** `simd` standardmäßig aus; Default nur, wenn Paritätstest Skalar↔SIMD (Cosine, L2, Dot; Dimensionen 8, 64, 384, 768, 1536; rel. Abweichung ≤ 1e-5) besteht **und** AVX2-Benchmark ≥ 1,5× bei Dimension ≥ 384. Negativergebnis ist ein gültiger Endzustand.

### 7.3 Invarianten

INV-VEC-ATOMIC-1 (nach Crash alte **oder** neue valide Generation) · -FSYNC-1 (jeder Fehler aus `fsync`/`rename`/Dir-Sync propagiert, danach `DurabilityPoisoned`) · -INTEGRITY-1 (jedes Byte durch Prüfsumme gedeckt, jeder Einzelbit-Flip in `Verify::Full` erkannt) · -NOPANIC-1 · -LOCK-1 (Reihenfolge im Code, in Debug geprüft) · -NOAWAIT-1 · -AUTARK-1 (`--no-default-features --features std` baut ohne Workspace-Crates) · -REPAIR-1 (unterbrochene Mutation ⇒ `needs_repair`, `Poisoned` bis `repair()`) · -ZEROIZE-1 (gelöschte Vektoren im RAM sofort genullt, ✔).

### 7.4 Dateiformat v3 (WP-V3)

```text
[Header 128 B] magic u32 | version u16 | min_reader_version u16 | flags u32 | required_flags u32
   dimension u32 | m u16 | metric u8 | quant u8 | node_count u64 | entry_point i64 | last_tx_id u64
   section_count u16 | section_table[offset,len,kind,hash16] (≤ 8) | header_crc u32 (CRC-32 IEEE)
[NODES] [VECTORS 64-B-aligned] [CONNECTIONS mit Layer-Offset-Tabelle u32×(layers+1)] [CALIBRATION]
[Trailer 48 B] commit_magic u64 | file_len u64 | blake3_full[32]
```

`flags`: bit0 quantized, bit1 docid128, bit2 has_sq8_bias; unbekannte `required_flags` ⇒ `UnsupportedFlags`. Writer schreibt nur v3; Reader liest v1/v2 über getrennte Parser mit festen Größen (bis 0.3, mit `warn`). `Verify::Header` (O(1), Default mmap) prüft Magic, Version, Header-CRC, Commit-Magic, `file_len`, Section-Table (Bounds, Überlappung, Alignment), DocId-Breite. `Verify::Full` (gestreamt, Default beim RAM-Laden, `repair()`, Inspect) prüft zusätzlich `blake3_full`, Section-Hashes, `id < node_count`. Goldene Fixtures v1/v2/v3 (quantisiert/unquantisiert, 64/128-Bit-DocId) unter `tests/fixtures/`.

### 7.5 Arbeitspakete Vector (Reihenfolge V1 → V2 → V6 → V3 → V4 → V5 → V7 → V8; V8-Gates ab dem ersten PR)

| WP | Inhalt | Akzeptanz |
| --- | --- | --- |
| V1 | Alle `use contextra_*` in `src/compat/` (hinter `ports-compat`); `default=["std"]`; Features `std, mmap, serde, tracing, ports-compat, docid-128, simd, experimental-*, partial-index-rebuild`; leere Features implementieren oder entfernen; `#![doc = include_str!("../README.md")]` | `cargo hack check --each-feature`; `rg "contextra_" crates/contextra-vector/src \| rg -v compat/` leer; README-Doctest mit `default-features=false` |
| V2 | `persistence/durable.rs`: `AtomicFileWriter{create, commit}`, `sweep_stale_tmp`, `DurabilityState`; `write_atomic` wird `#[deprecated]`-Wrapper; `clippy.toml` verbietet `rename`/`sync_all`/`sync_data` außerhalb | Fault-Injection an jedem Schritt ⇒ `Err`, alte Datei oder kein Temp; zwei Threads, ein Pfad ⇒ eine der beiden Generationen |
| V6 | `VecError` (Io, Format, Corrupt{section,offset,kind}, Config, Limit, DurabilityPoisoned, Poisoned); `is_recoverable_by_rebuild()`; `MutationGuard` (Drop ohne `finish()` ⇒ `needs_repair`); Lint-Hygiene | Panic mitten im Insert ⇒ `Err(Poisoned)`; `repair()` stellt Recall-Gleichheit zum Referenz-Rebuild her |
| V3 | Format v3 nach §7.4 | Einzelbit-Flip ⇒ `Err` in `Verify::Full` (CI 10 000 Stichproben, nightly vollständig); Truncation an jeder Byte-Grenze ⇒ `Corrupt::Truncated`; Fuzz `fuzz_hnsw_persistence` 0 Crashes in 10 min |
| V4 | `MmapIndex`-Felder privat (`header()`, `node(i)`, `vector(i)`, `connections(i,layer)`); `open_async` ⇒ deprecated Shim; `OpenMode::{Mmap, ReadToHeap}`; optionaler Shared-Advisory-Lock über `sys` | per `rename` ersetzte Datei: Reader liest alte, konsistente Generation |
| V5 | `Ordered<const LEVEL,T>` mit Ordnung `L0 rebuild → L1 save → L2 write → L3 nodes → L4 doc_to_node → L5 deleted_nodes → L6 erased_docs → L7 quantizer/sq8_bias/mmap_index → L8 seq_log → L9 clock/metrics`; Save außerhalb des Schreib-Mutex; RCU-Suche; synchroner Kern (`insert_sync`, …); Loom: insert∥search, rebuild-swap∥search, save∥insert, delete∥search | Contention-Benchmark (V-D5); `allow(await_holding_lock)` in Tests entfernt |
| V7 | `erase` nullt RAM, `save` schreibt ohne erased Einträge, Vorgängergeneration wird entlinkt (`secure_wipe_old_generation` Best Effort); `erase_and_persist` ⇒ `ErasureReceipt{generation, ram_zeroized, on_disk_rewritten}`; `wait_readers_drained(timeout)`; keyed Blake3 pro Abschnitt | Vektormuster im Dateibytestrom nach `erase_and_persist` nicht mehr vorhanden |
| V8 | Testnamen nach Verhalten (`campaign_j*`, `k02_*` umbenennen, Dubletten zusammenführen, `// Invariante: …`); Crash-Harness; Bitflip-/Truncation-Matrix; Determinismus mit Seed; CI-Matrix (hack, clippy, test, loom, miri auf Parser, deny+mutants, Fuzz-Smoke, Doctests) | alle blockierend |

---

## 8. Gates, CI, Werkzeuge

### 8.1 Ist gegen Soll

| Gate | Prüft | Ist am HEAD |
| --- | --- | --- |
| G-1 … G-6 | Löschkette (§8.2) | ✔ `tools/erasure_gates.py` mit Selbsttest (`tools/tests/erasure_gates/g1…g6_red/green.rs`), CI-Job `gates` |
| G-7 … G-9 | `tokio::spawn` ohne `TaskTracker`, verworfene `fail`/`record_layer`/`on_gate`-Ergebnisse, `_ =>` in `journal.rs`-Encodern | ✖ nicht implementiert (WP-E7) |
| Release-Grenze | `cargo tree -e features -p contextra` ohne Test-Features | ✔ CI-Job `release-feature-boundary` |
| Testkit, Loom | `cargo test -p contextra-testkit`, `--test loom_erasure` | ✔ CI-Job `testkit` |
| **L1 Layering** | Ist-Kanten gegen §3.1 | **✖ wirkungslos:** `tests/layering.rs` ruft `tools/gate_g1_layering.py` (Datei fehlt) und prüft nur „Ausgabe nicht leer“ |
| **U Unsafe** | Inseln `sys`, `simd` | **✖ wirkungslos** (`tests/unsafe_islands.rs`, gleiche Ursache); `tools/unsafe_boundary_auditor.py` nur manuell |
| P Panic | `tools/zero_panic_analyzer.py` gegen `tools/baseline` | manuell, nicht in CI |
| B Budget | `tools/abhaengigkeits_budget.py` | manuell, nicht in CI |
| L2 Lint-Hygiene | `allow(dead_code)`/`allow(unexpected_cfgs)` ohne `// allow:` | ✖ nicht vorhanden |
| D Doc-Wahrheit | CRC-, Ring-, Crate-Angaben gegen Code | ✖ nicht vorhanden (kein Skript, E-30) |
| Build/Test/Clippy/Deny | `cargo build/test/clippy --workspace`, `cargo deny check` | **✖ kein CI-Job** (einzige Workflow-Datei: `erasure-gates.yml`) |

**Ursache L1/U (E-29).** Das Root-`Cargo.toml` ist ein *virtuelles* Workspace (kein `[package]`, kein `[[test]]`); `tests/*.rs` im Root werden von Cargo nie kompiliert. `tests/arch` ist ein eigenes Package (`contextra-arch-tests`), steht aber nicht in `members`. Konsequenz: Das in den Erasure-Dokumenten als Pflicht genannte „`tests/layering.rs` grün“ ist heute nicht erfüllbar im Sinne einer Prüfung.

### 8.2 Erasure-Gates G-1 … G-9

G-1 kein `block_on` in Erase/Purge · G-2 kein `let _ =`, `unwrap_or_default()`, `.ok();` (Ausnahme nur `// gate-allow: G-2 <Begründung>`; Pfade: Löschpfade plus `crypto/src/dek.rs`) · G-3 `instrument` ohne `skip` bei `id`/`key`/`target` (`crates/*/src/*`) · G-4 kein `std::fs`/`tokio::fs` in `store/src/erasure/`, `engine/src/erasure/` und den neuen Layer-Dateien (Ausnahme E-31) · G-5 kein `Instant::now`/`SystemTime::now`/`HashMap`/`HashSet` in Löschpfaden · G-6 `ReadGate` ohne Default-Methoden · **G-7** kein `tokio::spawn` ohne `TaskTracker` in `engine/src/erasure/` · **G-8** `Result` von `fail`/`record_layer`/`on_gate` nicht verworfen · **G-9** kein `_ =>` in den Encodern von `store/src/erasure/journal.rs`. Jedes Gate hat einen Rot- und einen Grün-Test.

**Ratsche** `tools/erasure_gates_baseline.txt` (Stand HEAD, 9 Einträge): G-2 `crypto/dek.rs` 1, `engine/erasure/driver.rs` 5, `engine/erasure/mod.rs` 1; G-3 `insert.rs` 5, `kv.rs` 3, `read.rs` 2, `update.rs` 1, `contextra_impl/crud.rs` 10, `text/inverted/index_struct.rs` 1 (zusammen 22). Sie darf **nur schrumpfen**: mehr Verstöße ⇒ Fehler, weniger ⇒ Eintrag senken. [?] `dek.rs` ist laut `14` K-40 behoben, steht aber noch in der Baseline: `python3 tools/erasure_gates.py` ausführen und angleichen.

### 8.3 Konformanzsuite Durable-Semantik (E-04)

Im `testkit`: eine Suite prüft `StoreIo`/`AppendFile`, `vector::persistence::durable` und `contextra-durable-fs` gegen dieselbe R-8-Semantik; läuft in CI für alle drei.

---

## 9. Arbeitsplan (global, abhängigkeitsgeordnet)

**Regel.** Je Zeile ein Branch. „Nach“ nennt harte Vorgänger. Pakete ohne gegenseitige Abhängigkeit laufen parallel. Jedes Paket erfüllt §10.2.

| WP | Inhalt | Crates | Nach | Blockiert |
| --- | --- | --- | --- | --- |
| **0.1** | Baseline: `cargo build/test/clippy --workspace --all-targets --locked`, `cargo deny check`; CI-Job dafür anlegen (fehlt); PA-19, PA-01 | alle | – | alles |
| **0.2** | Gate-Verdrahtung (E-29): Layering-/Unsafe-Prüfung als Test in einem Workspace-Mitglied; `tools/gate_l1_layering.py` anlegen, Exit-Code ≠ 0 bei Überschreitung von §3.1, Dev-Kanten ausgenommen; CI-Job | root, tools | 0.1 | 0.3 |
| **0.3** | `capabilities.toml` auf §3.1 (E-12, E-06) | root | 0.2 | – |
| **0.4** | Doku-Wahrheit: `engine/src/lib.rs` Ring 3 (E-01), Generator-Docstring `tools/` (E-02), Store-README CRC (E-11), `contextra-index`-Alias und -Kommentare (§3.2.4), `governance/`-Verweise in `clippy.toml` (E-30) | engine, store, tools, root | – | – |
| **0.5** | Temp-Namen vereinheitlichen (E-09) in `store/io_seam`, `durable-fs`, später `vector` (V2); Test `tmp_file_naming_determinism.rs` anpassen | store, durable-fs | 0.1 | S2 |
| **E0** | K-45/46/47 + A-4: `begin_purge`, eine Quarantäneschwelle, Cursor-Reset | ports, store, engine | 0.1 | alle E |
| **E1** | A-1, A-2 (K-33, **Auslieferungsstopp**), `TST-STORE-ROOT-1…5` | ports, store, crypto | 0.1 | E3 |
| **E2** | Observer-Replay, `TaskTracker`, `NotReady`, `on_gate`-Fehlerpfad (K-49, K-53, K-58) | engine, types | E0 | E3 |
| **E3** | Pflicht-Layer + Builder-Verdrahtung (K-48, A-6), `TST-ENG-WIRE-1` | engine, contextra, types | E0, E1, E2 | E5 |
| **E4** | Zeugen (K-51), Manifest-Zeuge (Matrix Zeile 7), `TST-STORE-WIT-1/2` | store | E0, S1 | – |
| **E5** | `GraphErasure` (`08`) · `CheckpointErasure` + Floor (`09`, K-54, A-5); Layer-Konformanz C1–C8 für alle vier Layer | graph, checkpoint, engine | E3 | – |
| **E6** | Scheduler (K-57), `ErasureId`-Quelle (K-52, A-7), Encoder fallibel (K-50), A-8 `resume` | engine, store, ports | E0 | – |
| **E7** | Baseline-Abbau (K-55, K-56, K-59), Gates G-7…G-9 | tools, engine | E0, E6 | – |
| **E8** | Testkit: `MockLayer`-Fehlerinjektion, C8/IDEM, CP-19…30, Differential-Test; begleitet jedes E-Paket | testkit | – | – |
| **E9** | Krypto-Rückbau neu zuschneiden (E-32), `kv-shredding`-Entscheidung, danach Stufe 2 (opt-in, zuletzt) | crypto | E1 | – |
| **S1** | Einheitlicher I/O-Port (§6.4) | store, ports | 0.1 | S2, E4 |
| **S2 → S3 → S4** | Manifest, Orphan, Katalog (§6.4) | store | S1, 0.5 | S5 |
| **S5, S6, S7** | WAL-Fehlerpfade, Typisierung, Wartbarkeit | store | S4 | – |
| **S8** | Crash-Matrix, Modell, Fuzz; begleitet S1–S7 | store, testkit | – | – |
| **V1 → V2 → V6 → V3 → V4 → V5 → V7 → V8** | Vector (§7.5) | vector | 0.1 | V7 liefert Matrix Zeile 12 (G3 Datei) |

**Kritischer Pfad der Löschkette:** E0 → E1 → E2 → E3 → E5. Ein praktischer Lauf der Kette ist erst nach E0 möglich; eine Auslieferung erst nach E1.

---

## 10. Arbeitsanweisungen für Agenten

### 10.1 Arbeitsregeln (aus `AGENTS.md`/`CLAUDE.md`, verbindlich)

1. Kontext sparen: gezielt mit `rg`/`grep`/`sed -n` suchen, Dateien nicht vollständig laden, wenn ein Symbol genügt.
2. Immer nur das aktuelle WP bearbeiten und vollständig abschließen, bevor das nächste beginnt. Arbeitsschritte und Kontingent sparen.
3. Toolchain Rust **1.89.0** (`rust-toolchain.toml`), Installation über conda-forge, da `conda.anaconda.org` in der Allowlist steht:

```bash
mkdir -p /tmp/mm && curl -sSfL https://conda.anaconda.org/conda-forge/linux-64/micromamba-2.3.0-0.tar.bz2 | tar -xj -C /tmp/mm bin/micromamba
MAMBA_ROOT_PREFIX=$HOME/mamba /tmp/mm/bin/micromamba create -y -q -p $HOME/rust189 -c conda-forge rust=1.89
export PATH=$HOME/rust189/bin:$PATH && cargo --version
```

4. Kein Agent ändert diese Spezifikation stillschweigend. Ein Erratum ist ein neuer Eintrag in Anhang A mit Begründung und Beleg (Datei:Zeile).

### 10.2 Ablauf je WP

1. **Vorher.** MSA-Abschnitt des WP lesen, dann die Detailquelle nur an der genannten Stelle. `git rev-parse HEAD` (erwartet `d9ebd1192` oder neuer; bei Abweichung die Status-Tabellen §5.3, §5.4, §6.2, §7.1 per `rg` neu validieren). Baseline `cargo check --workspace --all-targets` muss grün sein.
2. **Währenddessen.** Ein WP je Branch; jeder Commit kompiliert; `cargo clippy -p <crate> --all-targets -- -D warnings` grün. Formatänderungen (Manifest, WAL, Journal, Vector) nur mit Migrationstest und Fixture. Neue interne Kante nur nach §3.2 Regel 3. Bei S1-Befunden den Test **zuerst** schreiben und rot sehen.
3. **Definition of Done.** (1) `cargo build --workspace --locked`, `cargo test --workspace --locked`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check` grün; (2) Feature-Matrix des Crates (§6.5); (3) relevante Gates aus §8, Ratsche nicht gewachsen; (4) Regressionstest je Befund, benannt nach Verhalten, mit `// Invariante: …`; (5) Doku und Anhang A aktualisiert (CRC, Ring, Crate-Liste); (6) Crash-/Loom-Satz des Pakets grün; (7) kein `#[ignore]` ohne Verweis auf Anhang A oder Issue; (8) `capabilities.toml` unverändert, außer das WP ändert §3.1.

### 10.3 Eskalation (keine Rückfragen)

1. Ist die Frage in Anhang A oder C geregelt: danach handeln.
2. Sonst, wenn ein Prüfauftrag existiert: ausführen und nach der festgelegten Folge handeln.
3. Sonst: Befund mit Beleg als Erratum-Entwurf in Anhang A eintragen, das WP auf „blockiert“ setzen, mit dem nächsten unabhängigen WP fortfahren.

### 10.4 Risiken

| Risiko | Gegenmaßnahme |
| --- | --- |
| Manifest-Formatwechsel verliert Daten | Migrationstest, `MANIFEST.v1.bak`, Fixture |
| `#![deny(dead_code)]` im Store bricht Teilauslieferung | Pakete so schneiden, dass jedes verdrahtet ist |
| Erase-Compaction erzeugt hohe I/O-Last | `IoBudget`, Koaleszenz, Fortschritt im Journal |
| Gate-Tabelle wächst, Lesepfad wird langsamer | Gate-Ende nach Witness 0 (I-9), Intervalle |
| Vektor-Rewrite senkt Recall | Recall-Guard, Teil-Rebuild |
| Altbestand ohne Provenienz | Ausweis als Ausnahme im `SealReceipt` |
| Tests als Root ausgeführt (Berechtigungstests) | CI als Nicht-Root (PA-01) |
| Löschzusage wirkt höher als belegt (K-48) | `required_layers`, I-6 erzwingen |

---

## Anhang A – Entscheidungsregister und Errata

**Status** am HEAD: ✔ eingehalten/umgesetzt · ◐ teilweise · ✖ offen (Folgepaket in §9).

| ID | Typ | Kurzfassung | Status |
| --- | --- | --- | --- |
| E-01 | Entscheidung | Engine = Ring 3, Ring 2 leer; Doc-Kommentar in `engine/src/lib.rs:3,5` sagt noch „Ring 2“ | ✖ 0.4 |
| E-02 | Entscheidung | `contextra-core` existiert nicht; Generator-Docstring nennt noch `scripts/` statt `tools/` | ✖ 0.4 |
| E-03 | Entscheidung | Layering-Gate heißt **L1**, Datei `tools/gate_l1_layering.py` (fehlt); Exit-Code prüfen | ✖ 0.2 |
| E-04 | Entscheidung | `durable-fs` nicht eingebunden; eine Primitive je Crate; Konformanzsuite (§8.3) | ✔ |
| E-05 | Entscheidung | Vector `simd` aus, Kriterien V-D6 | ✖ V1 |
| E-06 | Entscheidung | `bench` darf `rank` (Ist-Kante vorhanden, `capabilities.toml` fehlt) | ✖ 0.3 |
| E-07 | Erratum | K-36 „MCP nur contextra“ korrigiert auf §3.1; **Erase-Code im MCP nur über die Fassade** | ✔ |
| E-08 | Erratum | Rangnamen im Code maßgeblich: `RANK_ERASURE_KERNEL` (150), `RANK_ERASURE_GATE` (960), `RANK_CATALOG_MUTEX` (350); `00` §4 nennt noch `…_PLAN`/`…_GATE_SNAPSHOT` | ✔ |
| E-09 | Erratum | Temp-Muster `.{ziel}.{pid}.{zähler}.tmp` (R-8) | ✖ 0.5 |
| E-10 | Erratum | `catalog_mutex` (350) existiert bereits | ✔ |
| E-11 | Entscheidung | CRC-32 IEEE über `crc32fast`, Blake3 für Vollintegrität | ◐ README |
| E-12 | Entscheidung | `capabilities.toml` auf §3.1 | ✖ 0.3 |
| E-13 | Entscheidung | HEAD `d9ebd1192` normativ, ältere Stände historisch (§2) | ✔ |
| E-14 | Erratum | Features `encryption-at-rest`, `legacy-wal-key` deklariert; Helper mit Aufrufer-`cfg` | ✔ |
| E-15 | Entscheidung | Ringverteilung wie §3.1 (✔ stimmt mit `capabilities.toml`) | ✔ |
| E-16 | Entscheidung | Windows Tier 2, `DirSyncUnavailable`, *Regulated* verweigert | [?] |
| E-17 | Entscheidung | DocId-Breite per Compile-Feature, im Vector-Header gespeichert | ◐ V3 |
| E-18 | Entscheidung | Vector-Primitive `persistence::durable`, kein Adapter-Feature | ✖ V2 |
| E-19 | Entscheidung | ADR A–G gemäß §5.2 | ✔ (übernommen, E-33) |
| E-20 | Entscheidung | Erasure-Gates G-1…G-9, Ratsche nur schrumpfend (erweitert um G-7…G-9) | ◐ E7 |
| E-21 | Entscheidung | Manifest-Rollover ist interner Auslöser, Snapshot aus eigenem Zustand | ◐ S2 |
| E-22 | Entscheidung | Alte Manifest-Generationen nicht aufbewahrt (außer `.v1.bak`) | [?] |
| E-23–E-28 | Errata | Vector-`rand64`, Store-„CRC32C“, „neuer Rang 350“, `allow(unexpected_cfgs)`, doppelte Interface-Spezifikation, „keine Toolchain“ | erledigt oder gegenstandslos |
| **E-29** | neu | Root-Tests (`tests/layering.rs`, `tests/unsafe_islands.rs`) laufen nie (virtuelles Workspace, `tests/arch` kein Mitglied). **Entscheidung:** Prüfungen laufen als Test eines Workspace-Mitglieds (z. B. `tests/arch` wird Mitglied), Aufruf von L1 mit Exit-Code | ✖ 0.2 |
| **E-30** | neu | `clippy.toml` verweist auf `governance/clippy.toml` und `governance/determinism-allow.toml`; das Verzeichnis fehlt. **Entscheidung:** Root-`clippy.toml` ist die einzige Quelle; Verweise entfernen. Gate D bleibt Soll, bis dahin Checkliste per `rg` (§10.2) | ✖ 0.4 |
| **E-31** | neu | `07` §6 verlangt Vector-Datei-I/O „über `StoreIo`“, Vector (Ring 0) darf `store` nicht kennen. **Entscheidung:** über `persistence::durable` (V-D1, R-8), Crash-getestet; G-4 nimmt `durable.rs` aus | ✖ V2 |
| **E-32** | neu | KV-Cache-Subsystem entfernt (PR #6094): `KvSegmentCipher` und `ports/src/kv.rs` sind weg, Matrix-Zeile 15 entfällt. **Weiter vorhanden:** `KeyRegistry`, `RevocationLog`, `EncryptedCommitmentSalt`, `kv_shredding.rs`, `deletion_proof`, Default-Feature `legacy-erasure`. **Entscheidung:** `05` §3 (Rückbau, Feature `kv-shredding`) ist vor E9 gegen den HEAD neu zu schneiden; bis dahin bleibt `legacy-erasure` Default | ✖ E9 |
| **E-33** | neu | `KONZEPT-LOESCHKETTE-FUNDAMENT.md` fehlt im Repo. ADR A–G (§5.2) und I-1…I-12 (§5.1) gelten in der hier wiedergegebenen Form; Änderung nur über Anhang A | ✔ |
| **E-34** | neu | Quarantäneschwelle liegt allein im Kernel (`StoreErasureConfig.quarantine_after_attempts`); `engine::ErasureConfig.quarantine_after_attempts` entfällt | ✖ E0 |

---

## Anhang B – Änderungen gegenüber MSA v1.1

| Nr | Korrektur | Beleg |
| --- | --- | --- |
| B-1 | HEAD `4bce39fc4` ⇒ `d9ebd1192`; Erasure-Dokumente Rev. 2 (`00`–`14`, Befunde K-1…K-60) eingearbeitet. v1.1 kannte nur K-1…K-30, A-1…A-3 | `docs/erasure/00`, `14` |
| B-2 | v1.1 §6.1 „übrige Layer … noch nicht gebaut“ ist falsch: Store-Kernel, Engine-Treiber, MCP-Werkzeuge, Testkit, `TextErasure`, `HnswErasure` existieren. Ersetzt durch §5.3 | `store/src/erasure/`, `engine/src/erasure/` |
| B-3 | A-3 ist erledigt (`LAYER_TEXT`, `LAYER_CHECKPOINTS`). A-1, A-2, A-4…A-8 neu aufgenommen | `ports/src/scanner/layer.rs`, Commit `71c7a9135` |
| B-4 | v1.1 nannte Gates L1/U/P/D/B „bestehend“. Tatsächlich laufen nur Erasure-Gates, Testkit/Loom und Release-Grenze in CI; L1/U sind wirkungslos, D und L2 existieren nicht, es gibt keinen Build/Test/Clippy/Deny-CI | §8.1, E-29 |
| B-5 | HashMap/HashSet sind **workspace-weit** verboten (`clippy.toml`), nicht nur in der Löschkette | `clippy.toml` |
| B-6 | `onnx` aus der Liste erlaubter `cfg`-Werte entfernt | Root-`Cargo.toml` `check-cfg` |
| B-7 | WP-S1/S2/S3 präzisiert: `wal/file.rs` auf `VfsFile`, `ManifestState`, `CURRENT_MANIFEST_VERSION = 2`, `rollover(&self)`, `RecoveryPolicy` existieren; verbleibend sind `tokio::fs` in `wal/mod.rs:66`, `lsm/engine.rs`, `scanner/*` und globale Fault-`static`s | §6.2 |
| B-8 | CRC32C-Doku im Code behoben, Rest nur `store/README.md`; Temp-Namen (E-09) sind in keinem der drei Writer umgesetzt | R-8, R-9 |
| B-9 | KV-Cache-Rückbau (#6094) berücksichtigt: Matrix-Zeile 15, `05` §3 | E-32 |
| B-10 | Neu: fehlende Quellen (Konzept, `governance/`, `PLAN.md`, `VETOES.md`), Facade-Zählung unklar (PA-20) | §2, E-30, E-33 |
| B-11 | Struktur: Lesekarte, Crate-Karte, ein globaler Arbeitsplan (§9) statt WP-Listen je Abschnitt; Anhang D (Widersprüche W-1…W-25) entfällt, da jede Zeile bereits in Anhang A steht; Rollout-/Hotfix-Tabelle v1.1 §10.2 durch §9 ersetzt | – |
| B-12 | Aussagen zu `cargo check`/Testläufen (v1.1 §0.5) entfernt, da hier nicht reproduzierbar | Verifikationsgrenze |

---

## Anhang C – Prüfaufträge mit festgelegten Folgen

**Beantwortet durch statische Lesung:** PA-09 ⇒ Treffer > 0 ⇒ **WP-S1 offen** (§6.2 F3) · PA-10 ⇒ K-1 (Gate-Prüfung an zwei Stellen) · PA-11 ⇒ K-13 (Gate im `LsmStorage`-Konstruktor) · PA-12 ⇒ E-22 · PA-13 ⇒ K-38 (Graph tenantweit, `__graph:`-Schlüssel im Store) · PA-18 ⇒ abweichend (`persistence/atomic.rs` verschluckt Dir-Sync-Fehler) ⇒ **V2**.

**Offen (Ausführung oder Lesen nötig):**

| ID | Prüfauftrag | Ergebnis A | Ergebnis B |
| --- | --- | --- | --- |
| PA-01 | `cargo test -p contextra-store --features encryption-at-rest,legacy-wal-key --lib -- wal::tests::io_tests::test_wal_rotate_and_seal_readonly_guarantee` als Nicht-Root | grün ⇒ Root-Effekt, kein Fix | rot ⇒ Seal erzwingt keine Schreibsperre, Fix in S1 |
| PA-02 | Test F1 „Rollover erhält `wal_hwm`“ (`manifest/tests.rs`) | grün ⇒ F1 erledigt | rot ⇒ S2 vorziehen |
| PA-03 | Loom Flush ∥ Rollover in `loom_catalog_publish` | vorhanden ⇒ F2 erledigt bis Erweiterung | fehlt ⇒ Erweiterung in S4 |
| PA-04 | Reihenfolge `rename → sync_dir → Reopen → poisoned` in `manifest/rollover.rs` | Poison nach jedem Fehler ab `rename` ⇒ F4 erledigt | Lücke ⇒ S2 |
| PA-05 | Torn-Tail-Kürzung (`set_len`) vor erstem Append | vorhanden ⇒ F5 erledigt | fehlt ⇒ S2 |
| PA-06 | `load` bei 1–4 Byte großer Datei | Neu-Init ⇒ F6 erledigt | Fehler ⇒ S2 |
| PA-07 | `manifest.append` relativ zu `sstables.write()` im Compaction-Swap | außerhalb ⇒ F9 erledigt | innerhalb ⇒ S4 (b) |
| PA-08 | `rg "trait VfsFile" -A40 crates/contextra-ports` | alle Methoden aus S1 vorhanden ⇒ Port unverändert | fehlende additiv ergänzen |
| PA-14 | Bereinigung der Intent-Dateien beim Start (`lsm/recovery.rs`) | vorhanden ⇒ erledigt | fehlt ⇒ nach R-8 (4) ergänzen |
| PA-15 | Alle Engine-Pfade durch `TenantScopedStorage` | ja ⇒ Gate sieht Rohschlüssel | nein ⇒ Pfade nachziehen, Test je Pfad |
| PA-16 | `CheckpointMeta.metadata` im Produktivcode genutzt | ja ⇒ Feld bleibt | nein ⇒ streichen (Anhang A) |
| PA-17 | Aufrufer von `Purger`/`ScanTarget`/`ScanSession::finish` | vorhanden ⇒ Migration nach `ResolvedScope` | keine ⇒ Typ/Trait löschen |
| PA-19 | `cargo build/test/clippy/deny` und die drei `#[ignore]`-Tests | grün ⇒ Baseline bestätigt | rot ⇒ Befunde in Anhang A, Ignore mit Verweis |
| PA-20 | Facade-Invariante „≤ 20 `pub fn`“ (Kopfkommentar `contextra/src/lib.rs:3`): am HEAD 45 `pub fn` im Crate, davon 3 in `lib.rs` | Messregel = nur `lib.rs`/`impl Contextra` ⇒ Regel präzisieren (Anhang A) | Gesamtzahl gemeint ⇒ Invariante verletzt, WP anlegen |
| PA-21 | `python3 tools/erasure_gates.py` ausführen, Baseline angleichen (Eintrag `dek.rs`) | Ist = Baseline ⇒ nichts | Abweichung ⇒ Baseline senken (nie erhöhen) |
| PA-22 | Offene Punkte `14` §8: `TST-STORE-SST-3`, `TST-VEC-CB-1`, `TST-MCP-CUR-1`, `TST-STORE-LOCK-1` (exklusives Verzeichnis-Lock) | grün ⇒ Matrix-Zeile schließen | rot ⇒ WP in §9 ergänzen |
