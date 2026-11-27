# Contextra – Konzept und Spezifikation der vollständigen Löschkette (Erasure Chain)

*Stand: 27.11.2026 · Rolle: Principal Rust Engineer (Systemprogrammierung, Durability, Concurrency, RAG-Speicherkerne) · Analysebasis: `tfufuz1/contextra`, Branch `main`, Commit `7bd922104` · Status: RFC-Entwurf zur Entscheidung*

> **Hinweis zur Analysegrundlage:** Die Analyse ist rein statisch (Quellcode, Git-Historie, Tests als Dokumentation). In der Analyseumgebung stand keine Rust-Toolchain zur Verfügung; es wurde nichts kompiliert und kein Test ausgeführt. Aussagen mit „nicht belegt“ sind vor Umsetzungsbeginn im Rahmen von WP-0 zu verifizieren.

## 1. Zusammenfassung

**Ausgangslage.** Contextra besitzt heute keine Löschkette, sondern Fragmente. `delete()` schreibt Tombstones. Die vier Purger (LSM, Text, HNSW, Graph) haben **keinen einzigen produktiven Aufrufer**. Alle Ports, die eine Kette tragen müssten (`ErasureLayer`, `ReadGate`, `CommitBarrier`, `PurgeBarrier`, `LedgerSink`, `RecordSealer`, `FileKeyring`), haben **keine Implementierung** im Workspace. Das Orchestrator-Crate `contextra-erasure` wurde am 26./27.11.2026 samt Löschnachweis-Typen entfernt (Commits `f00bc8ff3`, `91e248a64`, `9a6dd9a05`). Der LSM-Purger wurde als Stop-Ship fail-closed gesperrt (`214e679e2`), weil er fremde Datensätze verwerfen, verschlüsselte Speicher übersehen und trotzdem Erfolg melden konnte.

**Kernthese.** Löschen ist in einem Mehrschicht-Speicherkern (LSM + WAL + Textindex + HNSW + Graph + Checkpoints + Caches) **kein Funktionsaufruf, sondern ein verteiltes, crash-sicheres Protokoll**. Es braucht (a) einen dauerhaften Auftrag mit Zustandsautomat, (b) einen synchronen Lese-Gate, der Wiederauferstehung ausschließt, (c) kryptographische Löschung als Primärgarantie, weil physisches Überschreiben auf SSD/CoW nicht belastbar ist, (d) idempotente, wiederaufnehmbare Layer-Purger mit einheitlichem Vertrag, (e) gemessene Nachweise (Residue-Witness) und ein manipulationssicheres Ledger.

**Kernentscheidungen dieses Konzepts**

| Nr. | Entscheidung |
| --- | --- |
| E1 | Löschung ist eine **Saga** mit durablem Journal (`ErasureJournal`) und Zustandsautomat `Requested → Planned → Gated → KeysDestroyed → Purging → Verifying → Sealed → Completed`. Jeder Übergang ist write-ahead und crash-wiederaufnehmbar. |
| E2 | **Zwei Löschklassen:** *Retire* (logisch, Tombstone, zeitreisefähig, wie heute) und *Erase* (Garantie nach Stufen G1–G3, Abschnitt 2.3). `delete()` bleibt Retire; Erase ist ein eigener, expliziter Pfad. |
| E3 | **Crypto-Erase als Primärgarantie (G2)** über die vorhandene DEK-Hierarchie (`derive_dek`, `AncestorSalts` t/c/s/d, `wrap_dek`). Physisches Purging (G3) ist Defense-in-Depth und Speicherfreigabe. Abgeleitete Indexschichten (Postings, HNSW-Vektoren, Kanten) lassen sich nicht pro Dokument verschlüsseln und erhalten G3 als Pflichtgarantie. |
| E4 | **Gate vor Destruktion:** Ein synchroner `GateTable` (RAM, O(log n), lock-arm) deckt alle Lesepfade ab (Punkt, Scan, Text, Vektor, Graph, Hydration, Zeitreise, Restore, Export, Cache). Das Gate ist fail-closed und vor WAL-Replay geladen. |
| E5 | **Ein** Layer-Vertrag: `ErasureLayer` (async, typisierte Fehler, idempotent, resumable). `Purger`/`LayerScanner` werden darin konsolidiert. |
| E6 | **Erase-Compaction:** Der prädikatsgefilterte Rewrite (RFC-WP-4) wird als Filter im bestehenden Compaction-Framework umgesetzt, nicht als zweiter Rewrite-Pfad. |
| E7 | **Nachweis:** `LayerWitness` (nur durch `ScanSession::finish` erzeugbar) je Schicht plus Hash-Ketten-Ledger mit MAC. Nicht gemessene Schichten senken die ausgewiesene Garantiestufe sichtbar. Die Entscheidung **D-06** („Löschung = Tombstone, kein Nachweis“) wird per ADR für Erase ausdrücklich revidiert. |
| E8 | **Nie automatisch entsperren:** Ein Gate wird nur nach vollständigem Witness und abgelaufenem Restore-Horizont in einen kompakten Wasserstand überführt, niemals wegen Fehlern oder Timeouts geöffnet. |

**Umsetzung:** 14 Arbeitspakete (WP-0 bis WP-13), kritischer Pfad WP-1 → WP-2 → WP-3 → WP-4 → (WP-5…WP-8) → WP-9 → WP-10 → WP-11 → WP-12 → WP-13 (Abschnitt 8).

## 2. Anforderungen und Garantiemodell

### 2.1 Ziele

- **Z1 Vollständigkeit:** Jede Kopie und jedes Derivat eines gelöschten Gegenstands wird adressiert: LSM (Memtable, immutable Memtables, SSTables, Bloom-/Indexblöcke, Block-Cache), WAL, Manifest, Textindex (Postings, Forward-Index, Dokumentlängen, Term-Dictionary, BM25-Statistik), HNSW (RAM-Knoten, mmap-Datei), Graph (Entities, Kanten, Hyperedges, Communities), Checkpoints/Hardlink-Klone, Snapshots, Embedding-/KV-/Query-Caches, Commit-Intent und Kompensations-Ledger, Logs und Metriken.
- **Z2 Korrektheit unter Crash und Nebenläufigkeit:** Kein Zustand, in dem gelöschte Daten wieder lesbar werden (kein *Resurrection*), auch nicht durch Flush, Compaction, WAL-Replay, Rebuild, Backfill, Restore oder Konsolidierungs-Worker.
- **Z3 Nachweisbarkeit:** Pro Auftrag gemessene Restmengen je Schicht und ein manipulationssicheres Protokoll.
- **Z4 Betriebssicherheit:** Beschränkte Latenz-Einflüsse auf den Normalbetrieb, Ratenbegrenzung, Wiederaufnahme, Beobachtbarkeit.
- **Z5 Mandantenisolation:** Ein Auftrag trifft nachweislich nur seinen Scope (Mandant, Collection, Subjekt, Dokument).

### 2.2 Nicht-Ziele und ehrliche Grenzen

- Keine Garantie über **externe Kopien**: Backups, Exporte (`export.rs`), Replikate, Kunden-seitige Caches, LLM-Anbieter. Erase meldet Export- und Backup-Bezüge als „außerhalb des Scopes“. Backups müssen unter eigenen, vernichtbaren Schlüsseln stehen, das ist Betriebsvorgabe (Abschnitt 9).
- Keine Garantie gegen **Flash-Remanenz** ohne Verschlüsselung. Deshalb Primärgarantie per Schlüsselvernichtung.
- **Heap/Swap/Core-Dumps:** Klartext kann transient im Prozessspeicher liegen (`Bytes` ist unveränderlich und refcounted, nicht zeroisierbar). Minderung: Klartext nur kurzlebig, `zeroize` für eigene Puffer, Betriebsvorgabe „kein Swap, keine Core-Dumps“, optional versiegelte Memtable (Profil „regulated“).
- Kein juristisch signierter Beweis ohne externe PKI. Das Ledger ist ein manipulationssicheres **Betriebsprotokoll**; eine Signatur für Dritte ist optionales WP-Erweiterungsziel.

### 2.3 Garantiestufen

| Stufe | Aussage | Mechanismus | Zielzeit (Vorschlag) |
| --- | --- | --- | --- |
| **G0 Retire** | Ab Commit-Seq in allen aktuellen Sichten unsichtbar, in Zeitreise innerhalb Retention sichtbar | Tombstone-Commit (heute) | sofort |
| **G1 Gated** | In **allen** Lesepfaden unlesbar, einschließlich Zeitreise, Snapshots, Restore, Export; Schreibzugriffe auf den Scope abgewiesen | `GateTable`, Epoche, Commit-Ablehnung | < 100 ms nach Annahme |
| **G2 Crypto-Erased** | Primärinhalte sind in **allen** physischen Kopien unlesbar, weil der Schlüssel dauerhaft vernichtet ist | Salt-/DEK-Vernichtung, Epoch-Rotation, fsync | typ. < 1 s, Mandant < 30 s |
| **G3 Physically Purged** | Bytes sind aus allen Dateien entfernt, Witness meldet `remaining = 0` für jede Schicht | Erase-Compaction, Index-Purge, Segment-Rewrite, Scrub/Unlink | SLA, Default 24 h, konfigurierbar |

Der Auftrag meldet zu jedem Zeitpunkt die **erreichte** Stufe. „Abgeschlossen“ bedeutet: G3 für alle Schichten mit Witness oder explizit dokumentierte Ausnahme mit reduzierter Stufe.

### 2.4 Bedrohungs- und Fehlermodell

- **Prozessabsturz/`kill -9`** an beliebiger Stelle, auch zwischen zwei `fsync`.
- **I/O-Fehler:** `EIO`, `ENOSPC`, `fsync`-Fehler (ab dann ist der Dateizustand unbestimmt; keine Wiederholung auf demselben Deskriptor, Poisoning).
- **Teilweise Schreibvorgänge** (Torn Writes), reordertes Flushing.
- **Nebenläufige Akteure:** Writer, Reader, Compaction, Flush, Checkpoint, Konsolidierung, Rebuild, MCP-Clients.
- **Fehlkonfiguration/Bugs:** Falsches Ziel, Mehrdeutigkeit der Zielidentität, Hash-Kollision. Fail-closed und Scope-Prüfung gegen die gespeicherte Schlüsselidentität.
- Nicht betrachtet: Angreifer mit Root-Zugriff auf laufenden Prozess; kompromittierte Schlüsselverwaltung.

## 3. Ist-Analyse

### 3.1 Zeitachse in der Git-Historie

| Datum | Ereignis |
| --- | --- |
| 11.11.2026 | Entkoppelte Erasure-Familie: WAL-Sink, FenceGate, KEK/DEK, Residue-Scanner, Witness-Hash-Kette (INV-ERA-1…7) |
| 20./22.11.2026 | SPEC v4.1 und R8 mit Löschkette und Implementierungsspezifikation |
| 26.11.2026 | Entscheidung D-06: Löschkette entfernt. Welle 0+1 (`contextra-audit-export`, `contextra-avv-generator`, `f00bc8ff3`), Löschketten-Tests (`b70d316ab`), `drop_collection` als Tombstone-Commit (`91e248a64`) |
| 27.11.2026 | `contextra-erasure`/`-kvcache`/`-privacy` und Erasure-Typen entfernt (`9a6dd9a05`); `LsmPurger` gesperrt (`214e679e2`, „RFC WP-0, K2“) |

Die im Commit `214e679e2` zitierte Löschketten-RFC (WP-0 … WP-4) liegt im Checkout nicht vor. Dieses Dokument ist als Konkretisierung und Ersatz gedacht und hält WP-0 (Stop-Ship) und WP-4 (prädikatsgefilterter Rewrite) kompatibel.

### 3.2 Befunde

Schwere: **K** kritisch (Datenschutz-/Korrektheitsbruch möglich), **H** hoch, **M** mittel.

| ID | Sev. | Befund | Evidenz |
| --- | --- | --- | --- |
| F-01 | K | **Kein Koordinator.** Ports `ErasureLayer`, `ReadGate`, `CommitBarrier`, `PurgeBarrier`, `LedgerSink`, `RecordSealer`, `FileKeyring` ohne Implementierung. Kein produktiver Aufrufer von `text_purger`, `hnsw_purger`, `graph_purger`, `LsmPurger`. | `contextra-ports/src/{erasure,gate,barrier,ledger,keys}.rs`; Workspace-Suche; `f00bc8ff3`, `9a6dd9a05` |
| F-02 | H | **Zwei unvereinbare Port-Familien:** `ErasureLayer` (Fehler als `String`, `Scanner<Armed>` ist ein leerer Platzhalter, `ErasureTarget`/`StorageBackend` leere Marker-Traits) und `Purger`/`LayerScanner` (synchron, `ScanTarget` mit String-Id). | `ports/src/erasure.rs`, `ports/src/scanner/*` |
| F-03 | K | **`delete()` ist nur ein Tombstone**, wird aber als Löschen angeboten: `Collection::delete_op`, Facade `forget`, MCP `contextra_forget`/`contextra_delete` (Antwort `semantics: "tombstone"`). Embedding und Dokumentkörper bleiben physisch erhalten. | `engine/src/collection/crud/delete.rs`; `contextra/src/agent_memory.rs:266`; `mcp/src/server_tools.rs:362` |
| F-04 | K | **Tombstone- und Versions-GC ist nicht erzwingbar.** Tombstones fallen nur in *Full*-Compaction und nur bei `raw_seq < min_snapshot_seq`; die neueste Version unterhalb des Floors bleibt stets erhalten. Kein Trigger im Löschpfad. Ein einziger Snapshot oder Pin hält alles. | `store/src/compaction/engine.rs:855`; `compaction/retention.rs` |
| F-05 | K | **LSM-Purger gesperrt.** Alte Fehler: Ziel ignoriert (nur Tenant-Präfix, d. h. ganzer Mandant), Fehler verschluckt (`unwrap_or_default`, `let _ =`), verschlüsselte SSTables (`key_manager = None`) übersehen, WAL unberührt, Erfolg trotz Fehlern. 3 Tests `#[ignore]`. | `store/src/scanner/purge.rs`; `214e679e2`; `scanner/tests.rs:90,178,204` |
| F-06 | H | **Textindex:** `TextPurger` vorhanden, nicht verdrahtet. Der synchrone Purger ruft `futures::executor::block_on` auf (Stall-/Deadlock-Risiko im Tokio-Kontext). Postings liegen im LSM (`__txt:`-Namespace), ein Index-Purge allein reicht physisch nicht. | `text/src/inverted/purge.rs` |
| F-07 | H | **HNSW:** nur RAM-Knoten werden zeroisiert; mmap-Knoten werden lediglich in `deleted_nodes` markiert. Nachbarschaftsbereinigung scannt O(Layer × N), Fehler von `update_backlink` werden mit `let _ =` verworfen, Entry-Point-Neuwahl naiv, keine Reparatur der Nachbarn (Recall-Verlust). Neuschreiben über `save_internal` (Temp-Datei); `fsync`, Rename-Semantik und Scrub der Vorgängerdatei nicht belegt. Normalpfad ist Soft-Delete. | `vector/src/hnsw/purge.rs`, `hnsw/types.rs:~760` |
| F-08 | H | **Graph:** `GraphPurger` tombstoned nur Kanten mit `source_doc_id` (optional!), nutzt `TxId::MAX`, ignoriert Mandant; Entities, Properties, Labels und Hyperedges bleiben. Doku bestätigt: „nicht automatisch vom Löschpfad aufgerufen“. `insert_hyperedge` im Commit ohne TxId. | `graph/src/purger.rs`; `engine/src/transaction/db_transaction_commit.rs` |
| F-09 | H | **`drop_collection`:** `delete_prefix` materialisiert alle Schlüssel im RAM, ist gegenüber parallelen Writes nicht atomar (TOCTOU); `__col_idx` wird in zweiter Transaktion entfernt (Crash-Fenster); HNSW, Graph, Checkpoints unberührt. | `store/src/lsm/ops/write.rs:190`; `engine/src/collection/maintenance.rs:852`; `contextra_impl/collections.rs:299` |
| F-10 | H | **Mehrdeutige Zielidentität:** HNSW und Graph parsen `target_id` zuerst als numerische `DocId`, sonst per Key-Hash. Key `"42"` und Zahl 42 sind verschiedene Dokumente. `DocId` ist ein Hash (64 Bit Standard, 128 Bit hinter `docid-128`); Kollision führt zu Über-Löschung. Mandantensalz nicht belegt. | `hnsw/purge.rs`, `graph/purger.rs`, `types/src/types/domain/ids.rs:223` |
| F-11 | H | **Checkpoints/Snapshots:** Pins blockieren GC; Hardlink-Klone teilen Inodes (Scrub verweigert bei `nlink > 1`); `CheckpointMeta.metadata` ist freies JSON; `restore_checkpoint` prüft kein Gate; `SnapshotRegistry::revoke_covering` hat keinen Aufrufer; manuell gesetzte Pins laufen nicht ab. | `checkpoint/src/store.rs:515,611,756`; `hardlink_cloner.rs`; `mvcc/src/snapshot.rs:298` |
| F-12 | H | **Schlüsselhierarchie ungenutzt:** `derive_dek`, `AncestorSalts`, `wrap_dek` haben außerhalb von `contextra-crypto` keine Nutzer. Daten sind nicht pro Subjekt/Dokument verschlüsselt, Crypto-Erase daher unmöglich. Das Crypto-Crate trägt `legacy-erasure` als Default-Feature (DeletionProof, Ed25519, KvShredding, RevocationLog), obwohl die Entfernungsspezifikation deren Streichung vorsah: inkonsistenter Zwischenstand. | `crypto/src/dek.rs`, `Cargo.toml:38-43`, `lib.rs` |
| F-13 | M | **durable-fs:** `scrub_if_safe` überschreibt mit Nullen, verweigert aber bei Hardlink, CoW, lebendem Reader, nicht-obsolet. Überschreiben ist auf SSD/CoW/Journaling-FS kein Nachweis. `PendingDeleteJournal` existiert, ist aber an keine Löschkette gebunden. | `durable-fs/src/scrub.rs`, `journal.rs` |
| F-14 | H | **WAL:** HMAC-Kette, Inhalte bleiben bis Flush/Truncate lesbar; kein Konzept für Segmentvernichtung; Replay ohne Gate-Filter (es gibt keinen Gate). | `store/src/wal/{io,mod,hmac}.rs` |
| F-15 | H | **Residuen im Commit-Pfad:** `record_keys_with_old_values` und Kompensations-Ledger halten Altwerte im RAM; `CommitIntent{doc_ids}` wird unter Intent-Schlüsseln persistiert, Lebensdauer/Bereinigung nicht belegt. | `engine/src/transaction/*` |
| F-16 | H | **Abgeleitete Daten ohne Herkunftsnachweis:** Embeddings im `StoredDocument`, extrahierte Entities, Communities, Decay/Konsolidierung; Export-Kopien. | `engine/src/{extraction,decay_controller,export}.rs` |
| F-17 | M | **Nachweislage:** `deletion_e2e::T-DEL-1` (Canary-Bytes) ist im Test selbst als „schlägt auf aktuellem Stand fehl“ dokumentiert; T-DEL-3 prüft nichts mehr; Löschnachweis-Tests entfernt (`b70d316ab`). Keine Crash-, Loom- oder Concurrency-Tests für Löschung. | `contextra/tests/deletion_e2e.rs` |
| F-18 | M | **Fail-open-Defaults** im `ReadGate`-Trait: `check_raw` → `Open`, `gated_seq*` → `None`, `gate_epoch` → `0`. Ein Implementierer, der sie nicht überschreibt, öffnet still. | `ports/src/gate.rs` |
| F-19 | M | **Entscheidungswiderspruch:** D-06 (Löschung = Tombstone, Best-Effort-`purge`) steht der Anforderung „vollständige Löschkette“ entgegen. | Historie `docs/LOESCHKETTE_ENTFERNUNG_SPEC.md` |
| F-20 | M | **Nebenläufigkeit ohne Regeln:** HNSW-Purge hält `write_mutex` (parking\_lot) über die gesamte Laufzeit inklusive synchronem Datei-I/O; kein Lock-Rang für Löschung (`lsm/lock_order.rs` kennt nur Schreib-/WAL-Locks); `PurgeBarrier` fehlt, Compaction und Purge sind nicht koordiniert. | `hnsw/purge.rs`, `store/src/lsm/lock_order.rs` |

### 3.3 Was tragfähig ist (wiederverwenden)

- **Ports mit durchdachten Verträgen:** `ReadGate` (O(log n), keine I/O, Epoche), `RangeGate` (Erased darf verworfen werden, Restricted nie), owned Barrier-Token ohne Lock-Lifetimes, `LedgerEntry` mit BLAKE3-Kette und MAC.
- **Mint-geschützter `LayerWitness`:** nur über `ScanSession::finish` erzeugbar, kein `Default`, kein `Deserialize` (Compile-Fail-Doctests).
- **Compaction-Framework** mit Token-Bucket-Rate-Limit, Manifest-Swap und `retain_key_versions`.
- **`HnswPurger`** mit Zeroize und Arena-Slot-Freigabe, **`TextPurger`** mit `PurgeReport` (Teilfehler sichtbar).
- **`SnapshotRegistry::revoke_covering(tenant, range, gated_seq)`** als Mechanismus, um blockierende Snapshots mandantengenau zu entziehen.
- **durable-fs:** `AppendOnlyJournal` (CRC, Recovery-Policies), `PendingDeleteJournal`, `FaultInjector`, `HealthState`.
- **Testwerkzeug:** `contextra-testkit` (Referenzmodell, `fault_vfs`, Conformance-Makros, `ManualClock`), Crash-Matrix-Muster in `checkpoint_systematic_crash`.

### 3.4 Ursachen

1. **Kein Eigentümer:** Löschen war pro Schicht ein Nachgedanke ohne Koordinator, daher laufen Purger ins Leere.
2. **Keine Identität und Herkunft:** Ziel ist ein String; Derivate (Kanten, Entities, Embeddings) tragen keine verpflichtende Herkunft.
3. **Kein Crash-Protokoll:** Kein dauerhafter Auftrag, keine Wiederaufnahme, keine definierte Reihenfolge.
4. **Kein Nachweis:** Fehler werden verschluckt, Erfolg wird gemeldet, Messung fehlt oder ist Platzhalter.

## 4. Zielarchitektur

### 4.1 Komponenten und Schichtung

Neues schlankes Crate **`contextra-erasure`** (Ring 2, zwischen `contextra-ports`/`-types`/`-crypto`/`-durable-fs` und `contextra-engine`). Es hängt **nicht** von Store, Text, Vektor oder Graph ab. Die Schichten implementieren `ErasureLayer` in ihren eigenen Crates (Dependency Inversion); die Verdrahtung erfolgt im Builder der Facade (`wiring_matrix`-Tests). Dadurch bleiben `tests/layering.rs` und `unsafe_islands.rs` gültig.

| Komponente | Verantwortung | Nutzt / implementiert |
| --- | --- | --- |
| `ErasureCoordinator` | Annahme, Idempotenz, Planung, Saga-Treiber, Wiederaufnahme beim Start | Journal, Gate, Barriers, Layer-Registry |
| `ErasureJournal` | Durables Write-Ahead-Protokoll der Aufträge (Zustandsübergänge, Layer-Fortschritt) | `durable-fs::AppendOnlyJournal` (CRC, fsync, Recovery-Policy) |
| `GateTable` | Synchroner Lese-Gate, Epoche, Wasserstände | implementiert `ReadGate` |
| `ErasureBarriers` | Koordination Commit/Compaction/Purge | implementiert `CommitBarrier`, `PurgeBarrier` |
| `KeyService` + `SaltVault` | Schlüsselableitung, Vernichtung, Epoch-Rotation | `contextra-crypto` (`derive_dek`, `wrap_dek`), implementiert `FileKeyring`, `RecordSealer` |
| Layer-Adapter | Auflösen, Gate-Vorbereitung, Purge, Witness je Schicht | `ErasureLayer` in store, text, vector, graph, checkpoint, caches |
| `WitnessAggregator` | Sammelt `LayerWitness`, bildet Wurzel-Digest, bestimmt erreichte Stufe | `ScanSession::finish` |
| `ErasureLedger` | Hash-Ketten-Protokoll außerhalb des Datenverzeichnisses | implementiert `LedgerSink` |
| API-Oberfläche | `erase`, `erase_status`, `drop_collection` als Saga, MCP-Tools | Facade, `contextra-mcp` |

```text
 erase(req) ─▶ Coordinator ─▶ [Journal: Requested, Planned]
                 │
   Barrier.exclusive (kurz) ▶ Gate publish + Epoche++ ▶ revoke_covering ▶ Barrier frei     => G1
                 │
   KeyService: Salt-Vault neu schreiben, Epoch-Key rotieren, Caches evict           => G2
                 │
   Purge:  Caches ▶ {Text | Vektor | Graph} parallel ▶ Store (Memtable, Erase-Compaction, WAL)
                 ▶ Checkpoints/Pins ▶ Datei-Unlink/Scrub (PendingDeleteJournal)
                 │
   Verify: Witness je Schicht (Structural / NeedleScan) ▶ Ledger-Append ▶ Sealed      => G3
```

### 4.2 Identität, Scope und Provenienz

**Auftragsidentität.** `ErasureId` (128 Bit, aus dem `IdGen`-Port, damit Tests deterministisch sind) ist zugleich Idempotenzschlüssel: Dieselbe `ErasureId` mit demselben `plan_hash` liefert den bestehenden Auftrag, mit abweichendem `plan_hash` den Fehler `IdempotencyConflict`.

**Typisiertes Ziel statt String.** `EraseTarget` wird ersetzt (siehe F-02, F-10):

```rust
pub enum EraseTarget {
    Document   { tenant: TenantId, collection: CollectionName, key: DocKey },
    Subject    { tenant: TenantId, subject: SubjectId },       // alle Dokumente mit diesem Subjekt
    Collection { tenant: TenantId, name: CollectionName },
    Tenant     { tenant: TenantId },
}
```

**Auflösung (Planung)** ist read-only, deterministisch und erzeugt `ResolvedScope`: je Dokument `(DocKey, DocId, Collection)` plus abgeleitete Objekte (Kanten, Entities, Hyperedges, Communities, Summaries) aus dem Provenienzindex. Pflichtprüfungen:

1. **Schlüsselverifikation:** Der im Record gespeicherte `DocKey` muss dem angefragten Schlüssel byte-gleich entsprechen. Abweichung (Hash-Kollision, F-10) → Abbruch `ScopeAmbiguous`, kein Überlöschen.
2. **Mandantenprüfung:** Jeder aufgelöste Schlüssel trägt das Mandantenpräfix des Auftrags; Verletzung ist ein Invariantenbruch (Abbruch, Alarm).
3. **Keine Mehrdeutigkeit:** Numerische DocId-Literale sind im Auftrag nicht zulässig; Layer erhalten ausschließlich `ResolvedScope`, nie freie Strings.
4. `plan_hash = BLAKE3(kanonische Serialisierung des Scopes)`; Dry-Run nutzt denselben Code und liefert Mengen, keine Inhalte.

**Subjektbezug.** Bei Schreibzugriffen mit `subject` wird transaktional ein Index `__subj:{tenant}:{subject_hmac}:{doc_id}` gepflegt (Profil *regulated*: Pflichtfeld). Ohne Subjektindex ist nur `Document`-Erase möglich.

**Pflicht-Provenienz für Derivate.** Jedes abgeleitete Objekt trägt `derived_from: SmallVec<[DocId; 2]>` (nicht `Option`). Beim Erase gilt:

- Leere Restmenge → Objekt wird gepurged.
- Mehrere Quellen → gelöschte Quelle aus der Menge entfernen; inhaltsabhängige Felder (Summary, Kantengewicht, Community-Zuordnung) erhalten `NeedsRecompute` und sind bis zur Neuberechnung nicht lesbar.
- Altbestand ohne Provenienz (`ProvenanceUnknown`) wird im Plan als **nicht verifizierbar** ausgewiesen und senkt die ausgewiesene Garantie (Migration in WP-13).

### 4.3 Zustandsautomat der Saga

Jeder Übergang wird **vor** seiner Wirkung in das Journal geschrieben und per `fsync` gesichert (Write-Ahead). Das Journal ist die einzige Wahrheit für die Wiederaufnahme.

| Zustand | Aktionen | Durabler Marker | Stufe |
| --- | --- | --- | --- |
| `Requested` | Validierung, Berechtigungs- und Mandantenprüfung, Idempotenzprüfung | R1 `Requested{id, target_hash}` | – |
| `Planned` | Scope auflösen (Snapshot `s0`), Layer-Menge festlegen, Schlüssel-/Salt-Inventar bestimmen | R2 `Planned{plan_hash, layers, salts}` | – |
| `Gating` | Scope-Sperre, `CommitBarrier::exclusive()`, `gated_seq := commit_seq`, Gate veröffentlichen (Restricted → Erased), Epoche++ (Release), `revoke_covering`, laufende Transaktionen mit Scope-Bezug abbrechen, Barrier freigeben | R3 `GateIntent{gated_seq}` vor Veröffentlichung | → G1 |
| `Gated` | Bereitschaft für Vernichtung; ab hier ist **kein Rücksprung** mehr möglich | R4 `Gated` | G1 |
| `KeysDestroying` | Salt-Vault als neue Generation schreiben (ohne die Salts), `fsync` Datei, `rename`, `fsync` Verzeichnis; Epoch-Key rotieren, alten vernichten; `RecordSealer::evict(scope)` | R5 `KeysDestroyed{vault_gen}` nach erfolgreichem fsync | → G2 |
| `Purging` | Layer in fester Reihenfolge, je Layer idempotent und mit Cursor (`LayerProgress{layer, cursor, attempt}`) | R6.n je Layer | G2 |
| `Verifying` | `LayerScanner::scan` je Schicht; Restmenge > 0 → zurück nach `Purging` (`attempt++`, begrenzt) | R7 `Verified{witness_root}` | G2 |
| `Sealed` | Ledger-Eintrag (fsync), dann Journal | R8 `Sealed{ledger_index}` | → G3 |
| `Completed` | Gate-Lebenszyklus (Abschnitt 4.4) | R9 `Completed` | G3 |
| `Failed(retryable)` | Backoff mit Jitter (Clock-Port), nach Grenzwert `Quarantined(layer)` und Alarm | R `Failed{layer, kind}` | hält letzte Stufe |

**Reihenfolge der Layer in `Purging` und Begründung:** (1) RAM-Caches, (2) Text, Vektor, Graph parallel, (3) Store (Memtable, Erase-Compaction, WAL-Schnitt), (4) Checkpoints und Pins, (5) Datei-Unlink und Scrub. Die abgeleiteten Indizes laufen **vor** dem Store, weil `TextPurger` das Forward-Index-Dokument `fw:{doc}` lesen muss, um die betroffenen Terme zu kennen, und der Graph über die Provenienz im Store navigiert.

**Invarianten**

| ID | Invariante |
| --- | --- |
| INV-ERA-1 | Reihenfolge Gate → Schlüsselvernichtung → Purge → Witness → Seal wird nie vertauscht oder übersprungen. |
| INV-ERA-2 | **Write-Ahead:** Jede sichtbare oder destruktive Wirkung folgt einem durablen Journal-Eintrag. |
| INV-ERA-3 | **Kein Wiederaufleben:** Flush, Compaction, WAL-Replay, Rebuild, Backfill, Restore und Konsolidierung wenden das Gate an. |
| INV-ERA-4 | **Idempotenz:** Jeder Schritt und jeder Layer-Purge ist beliebig oft wiederholbar, ohne Doppelfreigabe oder Zustandsbruch. |
| INV-ERA-5 | **Monotonie:** `gate_epoch` steigt streng monoton und wird vor der Schlüsselvernichtung veröffentlicht (Release/Acquire). |
| INV-ERA-6 | **Witness-Ehrlichkeit:** `LayerWitness` entsteht nur in `ScanSession::finish`; `NotMeasured` senkt die ausgewiesene Stufe. |
| INV-ERA-7 | **Fail-closed:** Unbekannter oder nicht ladbarer Gate-Zustand wirkt wie `Restricted`; Gates werden bei Fehlern nie geöffnet. |
| INV-ERA-8 | **Scope-Treue:** Ein Layer berührt nur Objekte aus `ResolvedScope`; Mandantenpräfix wird bei jedem Zugriff geprüft. |
| INV-ERA-9 | **Kein Klartext in Betriebsdaten:** Journal, Ledger, Logs und Metriken enthalten nur Hashes mit Schlüsselbindung (HMAC), keine Schlüssel, keine Inhalte. |
| INV-ERA-10 | **Eine Lock-Ordnung** (Abschnitt 6.1) und keine Sperre über fremde `.await`-Punkte; Barrier-Token sind owned und cancel-safe. |
| INV-ERA-11 | **Forward-only ab G1:** Ein Auftrag kann nach `Gated` nicht abgebrochen, nur beendet oder in Quarantäne überführt werden. |
| INV-ERA-12 | **Keine Blockierung durch Dritte:** Pins, Snapshots und Checkpoints dürfen G1 und G2 nie verhindern, nur G3-Zeit beeinflussen (mit Alarm). |

Die Bezeichnung folgt dem früheren INV-ERA-Schema der Repo-Historie; die Inhalte sind hier neu gefasst.

### 4.4 Gate

**Struktur.** `GateTable` hält einen unveränderlichen `GateSnapshot` hinter einem lesefreundlichen Zeiger (`arc_swap::ArcSwap`, neue Abhängigkeit mit `cargo deny`-Prüfung; Alternative `parking_lot::RwLock<Arc<_>>`, Read-Lock nur für den Arc-Clone). Writer (Erase-Aufträge) bauen einen neuen Snapshot copy-on-write und veröffentlichen ihn atomar; Leser blockieren nie.

```rust
struct GateSnapshot { tenants: HashMap<TenantId, TenantGates>, epoch: u64 }
struct TenantGates {
    docs:   RoaringTreemap,          // DocId (u64) -> Erased/Restricted
    docs128: BTreeSet<u128>,         // bei Feature docid-128
    ranges: IntervalMap<KeyBuf, GateEntry>,   // Collection/Subjekt/Tenant
    floor:  Seq,                     // Wasserstand: alle Versionen <= floor gesperrt
}
enum GateEntry { Restricted{ since: Seq }, Erased{ up_to: Seq } }
```

**Semantik.** `check(k)` ist O(log n) und führt keine I/O aus. `Restricted` bedeutet „darf weder gelesen noch verworfen werden“ (Compaction behält), `Erased` bedeutet „unlesbar, darf verworfen werden“ (`RangeGate::ContainsErased`). Der Port wird um `check_version(k, version_seq)` erweitert, damit eine Wiederanlage desselben Schlüssels nach `Sealed` (neue Salt-Generation, `seq > up_to`) lesbar bleibt und Zeitreise alte Versionen weiter sperrt. Die fail-open-Defaults (F-18) entfallen: `check_raw`, `gated_seq*`, `gate_epoch` haben keine Default-Implementierung mehr.

**Linearisierbarer Lesepfad.** Lesezugriffe, die über mehrere Schritte laufen (Kandidaten → Hydration), folgen dem Muster: `e1 := gate_epoch (Acquire)`, Gate prüfen, lesen, `e2 := gate_epoch`; bei `e1 != e2` Prüfung wiederholen. Linearisierungspunkt eines Erase ist die Veröffentlichung des Snapshots. Was danach beginnt, sieht nichts Gesperrtes.

**Durchsetzungspunkte (alle verpflichtend, je einer mit Regressionstest):**

| Pfad | Einhängung | Maßnahme |
| --- | --- | --- |
| Punkt-Get, MVCC-Reads | `GatedStorage<S>` als Decorator um `StorageEngine` (neben `TenantScopedStorage`) | `check_raw` vor und nach dem Lesen |
| Scan, Prefix, Range | `scan`, `scan_prefix_bounded` | `variant_for_range`; Erased überspringen, Restricted → Fehler oder Auslassung nach Policy |
| Textsuche | Kandidatenstrom (BM25, WAND) | Postings vor Scoring und Top-k filtern; Hydration erneut prüfen |
| Vektorsuche | HNSW-Traversierung | `deleted_nodes` ∪ Gate maskieren, Über-Fetch, Hydration erneut prüfen |
| Graph | Nachbarn, Traversierung, Communities | Entity-/Kantenprovenienz gegen Gate |
| Zeitreise (`AS OF`), `get_at_seq` | `temporal_filter`, Storage | Gate gilt für **alle** Sequenzen |
| Checkpoint-Restore | `restore_checkpoint` | Ablehnung bei `tainted` (Abschnitt 4.8) |
| Export/Import | `export_memories` | Filter; Import prüft Gate |
| Caches | Embedding-, KV-, Query-Cache | Eviction beim Gate, Cache-Key enthält Epoche |
| WAL-Replay | `lsm/recovery.rs` | `covers_raw` **vor** Anwendung (SPEC I-2) |
| Flush und Compaction | `lsm/flush.rs`, `compaction/engine.rs` | Filter beim SST-Schreiben (Abschnitt 4.6) |
| Commit | Group-Commit-Leader | Ablehnung von Writes auf gegatete Schlüssel (`gated_seq`), Re-Check unter `CommitBarrier::shared` |

**Lebenszyklus eines Gate-Eintrags.** `Restricted` → `Erased(up_to)` → nach `Sealed` und Ablauf des Restore-Horizonts (kein tainted Checkpoint, kein Snapshot < `up_to`, `max_snapshot_age` abgelaufen) Verdichtung zu `floor` je Bereich. Ohne Verdichtung wüchse die Tabelle unbegrenzt; mit Verdichtung bleibt die Wiederaufleben-Sperre erhalten.

**Startreihenfolge (Boot).** (1) durable-fs-Health prüfen; (2) `ErasureJournal` öffnen, Gates laden (nicht ladbar → Store startet ohne Mandantendaten im Zustand `Restricted`, kein Fail-open); (3) `SaltVault` und Keyring öffnen; (4) **WAL-Replay mit Gate-Filter**; (5) Layer öffnen; (6) offene Aufträge wiederaufnehmen; (7) Verkehr annehmen.

### 4.5 Kryptographische Löschung (Primärgarantie G2)

**Warum Kryptographie.** Nullen überschreiben ist auf SSD (Wear-Leveling), CoW-Dateisystemen und Journaling-FS kein Nachweis; `scrub_if_safe` verweigert zudem bei Hardlinks, CoW und lebenden Readern (F-13). Nur ein vernichteter Schlüssel macht **alle** Kopien gleichzeitig unlesbar, auch Hardlink-Klone, Checkpoints und Reste im Flash.

**Hierarchie** (vorhandene Primitive in `contextra-crypto/src/dek.rs`):

```text
Root (KMS / OS-Keystore / HSM, Port KeyProvider)
  └─ UnitKey je Mandant (KEK)
       └─ AncestorSalts { t: Mandant, c: Collection, s: Subjekt (optional), d: Dokument }
            └─ derive_dek(unit, salts, doc_tag)  -> Dek   (HKDF; ohne Salt nicht rekonstruierbar)
                 └─ AES-256-GCM-SIV Seal-Rahmen (AAD: Mandant, Collection, DocKey-Hash, Typ, Formatversion)
```

Die Vernichtung des Salts auf der passenden Ebene (`d`, `s`, `c`, `t`) macht alle darunter liegenden DEKs unableitbar, auch wenn der `UnitKey` erhalten bleibt.

**SaltVault.** Pro Mandant eine Vault-Datei mit Salts, jeweils mit einem **Vault-Epoch-Key (VEK)** umhüllt (`wrap_dek`). Vernichtung = (1) neue Vault-Generation ohne den Salt schreiben (copy-on-write, `fsync` Datei, `rename`, `fsync` Verzeichnis), (2) **VEK rotieren**: verbleibende Salts mit `VEK(n+1)` neu umhüllen, `VEK(n)` im `FileKeyring` per `destroy` vernichten. Dadurch sind auch alte Vault-Blöcke im Flash ohne `VEK(n)` wertlos. Mehrere Aufträge werden in einem Fenster gebündelt (Batch, Default 1 s oder 1 000 Aufträge), weil jede Rotation O(Vault-Größe) kostet. Große Vaults werden segmentiert (je Segment eigener VEK).

**Granularitätsprofil** (Speicher- und Betriebskosten gegen Feinheit):

| Profil | Salt-Ebene | Kosten | Erase-Feinheit mit G2 |
| --- | --- | --- | --- |
| `PerDocument` | `d` je Dokument | 32 Byte je Dokument (10⁸ Dok. ≈ 3,2 GB Vault) | Dokument |
| `PerSubject` (Default *regulated*) | `s` je Subjekt | gering | Subjekt (und damit dessen Dokumente) |
| `PerCollection` | `c` | minimal | Collection |
| `PerTenant` | `t` | minimal | Mandant |

Wird ein feineres Ziel angefordert, als das Profil kryptographisch trägt, führt der Auftrag G1 und G3 aus und weist im Ergebnis ausdrücklich „G2 nicht verfügbar für diesen Scope“ aus. Er verschweigt es nie.

**Was versiegelt wird (Pflicht im Profil *regulated*):**

| Objekt | Versiegelt | Begründung |
| --- | --- | --- |
| `StoredDocument` (Körper, Metadaten, Embedding) | ja | Primärinhalt; Embeddings gelten wegen Inversionsrisiko als personenbezogen |
| WAL-Payloads | ja | WAL wird nach Schlüsselvernichtung unlesbar, ohne die HMAC-Kette anzutasten |
| Forward-Index `fw:{doc}` | ja | enthält Terme im Klartext |
| Graph-Entity-Payload (Name, Properties) | ja | Klartext-Personenbezug |
| `CheckpointMeta.metadata` | ja oder Allowlist | freies JSON ist ein Residuenkanal |
| Postings, Term-Dictionary, HNSW-Vektoren, CSR-Struktur | **nein** | Suche braucht Klartext; **G3 ist hier Pflicht** |

**Migration.** Records tragen eine Formatversion im Header; Lesen unterstützt versiegelt und unversiegelt (Dual-Read). Unversiegelter Altbestand wird **bei Compaction versiegelt** (seal-on-rewrite) und per Backfill-Job nachgezogen. Bis dahin ist für Altbestand nur G1+G3 möglich (Ausweis im Plan).

**Aufräumen im Crypto-Crate.** `legacy-erasure` wird nicht mehr Default-Feature. `deletion_proof`, `ed25519_proof`, `kv_shredding`, `revocation_log` werden entfernt (konsistent mit der Entfernungsspezifikation). `kv_segment` (`TenantIsolatedKvStore`, Eviction) bleibt als Layer `KvCacheSegments`. Zu prüfen: Wiederverwendbarkeit von `audit_chain` als Ledger-Baustein statt Duplikat.

### 4.6 Store: LSM, WAL und Erase-Compaction

**Grundsatz:** Ein einziger Rewrite-Pfad. Der prädikatsgefilterte Rewrite (RFC-WP-4) ist ein **Filter im Compaction-Framework**, kein zweiter, handgeschriebener SSTable-Rewriter wie der gesperrte `LsmPurger` (F-05).

**a) Memtable und immutable Memtables.** In-Place-Entfernung der Schlüssel im Scope unter kurzer `state.write()`-Sperre. Immutable Memtables, die noch auf Flush warten, werden **beim Flush gefiltert** (Gate, INV-ERA-3). Bei versiegelten Werten liegt nur Chiffretext im Heap. Klartext-`Bytes` können nicht zeroisiert werden (Grenze aus 2.2).

**b) Erase-Compaction** (`CompactionJobKind::Erase`):

1. **Kandidaten:** Alle SSTables, die Schlüssel im Scope enthalten können (Min/Max-Schlüssel, Bloom-Probe), über **alle Level**. Diese *Hülle* ist zwingend: Wird ein Tombstone verworfen, während ältere Werte in nicht ausgewählten Dateien liegen, leben sie wieder auf. Ist die Hülle nicht vollständig verarbeitbar, bleibt ein `EraseMarker`-Eintrag bestehen, bis sie es ist.
2. **Prädikat:** `ErasePredicate = DropKeys(scope) ∪ CollapseVersions(index_keys)`. `DropKeys` verwirft jede Version mit `seq ≤ gated_seq` unabhängig vom Snapshot-Floor (Snapshots sind durch das Gate geschützt, Abschnitt 4.4). `CollapseVersions` gilt für umgeschriebene Index-Schlüssel (z. B. Posting-Listen `__txt:`): alle älteren Versionen fallen, nur die neueste bleibt, weil ältere Postings die Dokument-ID und Term-Zuordnung noch enthalten.
3. **Ausgabe:** Bestehender `SstableBuilder` mit demselben `key_manager` (verschlüsselte Speicher werden nicht übersehen), neue Datei-IDs, **Manifest-Swap als Commit-Punkt** (atomar, fsync), alte Dateien in das `PendingDeleteJournal`.
4. **Streaming:** Keine Materialisierung ganzer Dateien oder Schlüsselmengen (Gegenbeispiel: `reader.iter().await.unwrap_or_default()` im Altpurger, `delete_prefix` mit `Vec<Vec<u8>>`).
5. **Fehler:** Jeder I/O-Fehler beendet den Job typisiert; `PurgeOutcome` zählt nur, was durch Manifest-Swap committet ist. Keine `let _ =`, kein `unwrap_or_default` im Pfad.
6. **Betrieb:** Token-Bucket (`max_io_bytes_per_second`), Erase-Jobs haben höhere Priorität als reguläre Compaction und fordern Compaction über `PurgeBarrier::purge_waiting()` zum kooperativen Abbruch auf.
7. **Crash:** Temporäre Dateien `*.erase.tmp` sind im Journal als `LayerProgress` registriert und werden beim Start entfernt. Der Swap ist der Commit-Punkt; Wiederholung findet keine Schlüssel im Scope mehr und endet als No-op (INV-ERA-4).

**c) `delete_prefix` und `drop_collection` (F-09).** Mit dem Gate ist die Atomarität der Tombstones nicht mehr nötig: Der atomare, durable Fakt ist der Gate-Eintrag im Journal. `drop_collection` wird zu `erase(Collection)`; Tombstones und `__col_idx` werden danach in begrenzten Batches (Scope-Sperre, kein Vec aller Schlüssel) geschrieben und durch Erase-Compaction physisch entfernt. Optional (WP-4b): Range-Tombstone `RangeDelete{start,end,seq}` in Memtable, SST und WAL.

**d) WAL.** (1) Payloads sind versiegelt (4.5). (2) **WAL-Schnitt:** Memtables mit Scope-Schlüsseln erzwungen flushen (Flush-Filter verhindert, dass gelöschte Schlüssel in die SST gelangen), WAL danach auf Segmentgrenze kürzen, Altsegment über `PendingDeleteJournal` entfernen und scrubben (Best Effort). (3) Es wird **nie** ein einzelner WAL-Record entfernt; die HMAC-Kette bleibt dadurch intakt. Der WAL-Lebenszyklus nach Flush (Rotation gegen reines `truncate`) ist nicht belegt und in WP-4 zu verifizieren; fehlt Segmentrotation, gehört sie zu WP-4.

**e) Caches, Blöcke, Manifest.** Block-Cache-Einträge der ersetzten Dateien werden beim Swap geändert (`RecordSealer::evict`). Bloom- und Indexblöcke verschwinden mit der Datei. Ältere Manifest-Generationen und Min/Max-Schlüssel in Footern können Schlüsselbytes enthalten; ob alte Generationen aufbewahrt werden, ist nicht belegt und Teil des Witness-Scans.

**f) Datei-Entsorgung.** `PendingDeleteJournal` → `scrub_if_safe` (Best Effort, verweigert bei Hardlink/CoW/Reader) → `unlink` → `fsync` Verzeichnis. Verweigerter Scrub ist **kein** Fehler für G3, weil G2 greift, wird aber im Witness als `ScrubRefused(reason)` protokolliert. `fsync`-Fehler setzen `HealthState` auf degradiert (Poisoning, keine Wiederholung auf demselben Deskriptor).

### 4.7 Text, Vektor und Graph

**Layer-privater Scope.** Der Textindex muss die Terme eines Dokuments kennen (`fw:{doc}`), diese sind aber nach der Schlüsselvernichtung versiegelt unlesbar. Lösung: `resolve` in der Planung liest `fw`, bildet die Term-Menge und legt sie als **Job-Blob** ab, versiegelt mit einem Job-Schlüssel `JK`. `JK` liegt im `JobVault` des Journals und wird erst bei `Sealed` vernichtet. Gleiches gilt für andere layerprivate Zwischenstände. So ist die Reihenfolge Gate → Schlüsselvernichtung → Purge unabhängig davon, was die Layer vorab lesen müssen.

**Textindex (`contextra-text`).**

- `TextPurger` wird **async** (Entfernung von `block_on_async`/`futures::executor::block_on`, F-06). CPU-lastige Teile via `spawn_blocking` mit Cancel-Prüfung.
- Pro Dokument: Postings aller Terme der Job-Menge, `dl:`, `fw:`, `tbs`, leere Term-Dictionary-Einträge (df = 0), BM25-Statistik (N, avgdl) atomar, Morphologie-/Trie-Caches (`BM25MorphIndex`) für entfernte Terme.
- Posting-Listen sind neue Versionen im LSM; die alten Versionen entfernt der Store per `CollapseVersions` (4.6). Ohne diesen Schritt bliebe die Dokument-Term-Zuordnung in älteren SSTables erhalten.
- Teilfehler werden nicht zu Erfolg verdichtet: `PurgeReport.units_failed > 0` ergibt `Err(Retryable)`.

**Vektorindex (`contextra-vector`).**

- **Phase A (G1):** Maskierung über `deleted_nodes`/Gate, Suche überspringt, Über-Fetch gleicht Top-k aus.
- **Phase B (G3):** Segment-Rewrite statt Markieren. Ziel ist die Architektur „immutable Segmente + Liveness-Bitmap“: neues Segment ohne die Knoten bauen, Nachbarn über den Reverse-Adjazenzindex (statt O(Layer × N)-Scan) bereinigen und mit der Nachbarschaftsheuristik neu verbinden, damit der Recall nicht fällt. Temp-Datei, `fsync`, `rename`, `fsync` Verzeichnis, neu mappen, Altmapping nach Reader-Drain (epochenbasierte Freigabe) freigeben, Altdatei über `PendingDeleteJournal` entfernen.
- RAM-Knoten: Zeroize von Vektor und Arena-Slots bleibt (vorhanden). **mmap-Knoten werden nie nur markiert**, sondern im Rewrite entfernt.
- Entry-Point: höchster Layer unter lebenden Knoten, je Layer gepflegt (nicht „erster nicht gelöschter“).
- Fehler (`update_backlink`) werden propagiert, nicht verworfen. `write_mutex` wird **nicht** über Datei-I/O gehalten, nur für den Zeiger-Swap (Shadow-Aufbau, F-20).
- **Recall-Guard:** Nach dem Purge Stichprobe (Self-Queries) gegen den Vorzustand; Fällt der Recall um mehr als 2 Prozentpunkte (konfigurierbar), wird ein Partial-Rebuild aus dem LSM-Quellbestand **mit Gate-Filter** geplant (R7).
- Kosten: Rewrite ist O(Indexgröße); deshalb **Koaleszenzfenster** für G3 (G1 sofort), Intervall `max_rewrite_interval` ist Teil des SLA.

**Graph (`contextra-graph`).**

- Pflicht-Provenienz (4.2): Kanten, Entities, Hyperedges, Community-Zuordnungen tragen `derived_from`.
- Primäre Entities (aus dem Dokument selbst) werden entfernt, Payload ist versiegelt. Abgeleitete Objekte mit leerer Restmenge werden gepurged, andere erhalten `NeedsRecompute`.
- **Hyperedges** sind transaktionsgebunden (`insert_hyperedge(tx, …)`, heute ohne TxId) und werden entfernt, wenn weniger als 2 Teilnehmer bleiben.
- Kein `TxId::MAX`, kein Umgehen der MVCC-Sicht; Mandantenprüfung bei jedem Zugriff (F-08).
- **Physisch:** Tombstone-Kanten bleiben in den CSR-Arrays bis zur Compaction; `CsrGraph::compact` schreibt Arrays neu und zeroisiert Property-Arenen. Der Persistenzpfad des Graphen (Snapshots, Dateien) ist in WP-8 zu inventarisieren (nicht belegt).
- Communities: betroffene Communities mit Gate-Filter neu berechnen; Summaries aus gelöschten Quellen verwerfen.

### 4.8 Checkpoints, Snapshots, Caches und Commit-Residuen

**Checkpoints.** Ein Checkpoint ist ein Pin einer `seq_no` plus Hardlink-Klon der SSTables und freies `metadata`-JSON (F-11). Er hält Daten am Leben, die der Store sonst verwerfen würde.

- **`tainted`:** Ein Checkpoint mit `seq_no ≥ scope.min_write_seq` enthält potenziell Scope-Daten. Der Plan bestimmt `min_write_seq` je Scope.
- **G1/G2 sind unabhängig von Checkpoints** (INV-ERA-12): `restore_checkpoint` lehnt tainted Checkpoints ab (Gate-Prüfung), und die Primärinhalte sind nach der Schlüsselvernichtung auch in Klonen unlesbar.
- **G3-Politik `CheckpointTaintPolicy`:** `Drop` (Default *regulated*: tainted Checkpoints nach Karenz `checkpoint_taint_grace`, Default 24 h, löschen), `Rewrite` (gefilterter Klon durch Kopie statt Hardlink, dann Original entpinnen), `LegalHold` (ausdrückliche Ausnahme, Auftrag endet bei G2 und weist `LegalHold` als Ausnahme aus).
- **Hardlink-Schluss:** Obsolete SSTables werden mit **allen** Namen (Checkpoint-Verzeichnisse) entfernt; erst bei `nlink == 1` ist `scrub_if_safe` zulässig, sonst wird der Scrub zurückgestellt (F-13).
- **`drop_checkpoint`** propagiert Entpin-Fehler (heute nur `warn`): Wiederholung plus Alarm.
- **`CheckpointMeta.metadata`:** Allowlist oder Versiegelung (4.5). Freies JSON ist ein Residuenkanal.

**Snapshots und Pins.** Beim Gate wird `SnapshotRegistry::revoke_covering(tenant, 0..gated_seq, gated_seq)` aufgerufen (heute ohne Aufrufer, F-11), damit die Compaction-Untergrenze `min_active_seqno` voranschreiten kann. Sicherheit für Leser gibt das Gate, nicht der Entzug. Manuell gesetzte Pins erhalten ein Ablaufdatum (`max_pin_age`, Default 7 Tage, konfigurierbar) und einen Alarm bei Überschreitung.

**Caches (nur RAM).** Embedding-Cache, KV-Cache-Segmente (`KvCacheSegments`), Query-Ergebnis-Cache, Block-Cache und entschlüsselte Schlüssel-Caches werden beim Gate nach Scope geräumt (`on_gate`). Cache-Schlüssel enthalten die Gate-Epoche, damit ein Eintrag aus einer älteren Epoche nie als Treffer gilt.

**Commit-Residuen (F-15).**

- `record_keys_with_old_values` und das Kompensations-Ledger speichern Altwerte nur **versiegelt** oder als Referenz und werden nach Commit/Rollback zeroisiert.
- `CommitIntent` (Schlüsseltyp 3) wird nach Abschluss entfernt; der Intent-Namespace gehört zum Store-Scope (`CollapseVersions`).
- Laufende Transaktionen mit Scope-Bezug werden beim Gate abgebrochen (`RollbackStagedAction`); der Commit-Leader lehnt Schlüssel unter Gate ab.

**Logs, Spans, Metriken.** `#[tracing::instrument(skip(self))]` ohne `skip` des Parameters erfasst Argumente: `Collection::delete(&self, id: &str)` schreibt `id` in den Span. Regel: Schlüssel, Subjekt- und Dokument-IDs gelangen nur als HMAC in Telemetrie (INV-ERA-9); ein CI-Gate (`rg`) prüft `instrument`-Attribute auf Funktionen mit `id`/`key`/`subject`-Parametern ohne `skip`.

### 4.9 Layer-Vertrag, Witness und Ledger

**Konsolidierter Vertrag.** `ErasureLayer` (heute ohne Implementierung, F-01/F-02) ersetzt `Purger` und `Scanner<Armed>`. `LayerScanner`/`ScanSession`/`LayerWitness` bleiben unverändert die einzige Messkette (INV-WITNESS-MINT-1).

```rust
pub trait ErasureLayer: Send + Sync {
    fn id(&self) -> LayerId;
    fn key_coverage(&self) -> KeyCoverage;                       // Full | Unlinked | None
    fn resolve<'a>(&'a self, scope: &'a ResolvedScope, cx: &'a PlanCx)
        -> BoxFuture<'a, Result<LayerScope, LayerError>>;        // read-only, deterministisch
    fn on_gate<'a>(&'a self, s: &'a LayerScope)
        -> BoxFuture<'a, Result<(), LayerError>>;                // RAM-Maskierung, Cache-Evict, idempotent
    fn purge<'a>(&'a self, s: &'a LayerScope, cx: &'a PurgeCx)
        -> BoxFuture<'a, Result<PurgeOutcome, LayerError>>;      // wiederaufnehmbar über cx.cursor
    fn scanner(&self) -> &dyn LayerScanner;                      // Witness nur über ScanSession
}

pub struct LayerError { pub kind: LayerErrorKind, pub layer: LayerId, pub source: Box<dyn Error + Send + Sync> }
pub enum LayerErrorKind { Retryable, Permanent, Poisoned, ScopeViolation, Cancelled }

pub struct PurgeCx<'a> {
    pub erasure_id: ErasureId, pub clock: &'a dyn Clock, pub deadline: Instant,
    pub io_budget: &'a IoBudget, pub cancel: &'a CancellationToken,
    pub cursor: Option<Cursor>, pub progress: &'a dyn ProgressSink,
    pub gate: &'a dyn ReadGate, pub job_keys: &'a JobKeys,
}
pub struct PurgeOutcome { pub purged_count: u64, pub bytes_reclaimed: u64, pub cursor: Option<Cursor> } // None = fertig
```

Regeln: `purge` meldet nur committete Arbeit; Teilfehler sind `Err`, nie `Ok` mit Zähler. Es gibt keine blockierenden Aufrufe aus async-Kontext (`block_on` ist verboten, Clippy-/`rg`-Gate). `LayerScope` darf ein **versiegeltes privates Blob** (JK) tragen. Fehler sind typisiert statt `String` (F-02).

**Conformance-Suite** (neu in `contextra-testkit/src/conformance/erasure.rs`, für jede Layer-Implementierung verpflichtend):

| ID | Prüfung |
| --- | --- |
| C1 | Idempotenz: zweimal `purge` ergibt denselben Endzustand, zweites Ergebnis ist No-op |
| C2 | Wiederaufnahme: Abbruch an jedem Cursor-Punkt, Fortsetzung erreicht denselben Endzustand |
| C3 | Scope-Isolation: Nachbarmandant und Nachbardokument bleiben bit-identisch |
| C4 | Witness-Ehrlichkeit: Wird ein Canary absichtlich nicht entfernt, meldet der Scanner `remaining > 0` |
| C5 | Fehlerinjektion: jeder injizierte I/O-Fehler führt zu `Err`, nie zu `Ok` |
| C6 | Cancel-Safety: Abbruch zu jedem `.await` hinterlässt konsistenten Zustand |
| C7 | Gate-Respekt: Nach `on_gate` liefert keine Leseoperation der Schicht Scope-Daten |

**Witness-Technik.** `Structural` (Strukturiteration nach Scope-IDs), `DecryptedView` (für verschlüsselte Dateien), `NeedleScan` (Byte-Scan über Dateien, WAL, mmap, Checkpoints, Logs nach Nadeln aus dem Scope). **Nadeln** (DocKey-Bytes, DocId als LE/BE, Dezimal-/Hex-Form, Terme aus dem Job-Blob) existieren nur im RAM und im JK-versiegelten Blob. Sie sind kein Persistenzmaterial. Versiegelter Inhalt ist absichtlich nicht auffindbar (das ist G2).

**Stufenermittlung.** G3 genau dann, wenn jede Schicht mit `Structural`/`DecryptedView`/`NeedleScan` `remaining = 0` meldet. Jede Schicht mit `NotMeasured(reason)` oder `FeatureDisabled` reduziert auf G2 und erscheint als `exception` im Ergebnis. Die `witness_root` ist ein BLAKE3-Merkle-Digest über die Schicht-Digests in `LayerId`-Reihenfolge.

**Ledger.** Aufbauend auf `LedgerEntry` (Commitment, RangeSet, `gated_seq`, `completed_at`, `chain_hash`, `ledger_mac`) und `LedgerSink`:

- **Felder neu:** `erasure_id_hmac`, `guarantee` (G1–G3), `layers_root`, `exceptions`, `plan_hash`, `vault_gen`, `policy_version`.
- **Commitment** `= HMAC(ledger_key, plan_hash ‖ erasure_id)`: Ein Prüfer mit Schlüssel kann nachrechnen, dass ein bestimmter Schlüssel gelöscht wurde, ein Dritter kann nichts aufzählen (INV-ERA-9).
- **Ablage** außerhalb des Datenverzeichnisses (Start im Profil *regulated* verweigert, wenn der Ledger-Pfad unter dem Datenpfad liegt); `append` = Schreiben, `fsync`; Rotation mit `fsync` Verzeichnis.
- **Manipulationsschutz:** Kette plus MAC erkennen Veränderung, nicht Kürzung am Ende. Dagegen veröffentlicht ein `LedgerAnchor`-Port periodisch den Kopf-Hash (Datei im WORM-Speicher, Syslog oder externes Log).
- **Prüfung:** `verify_ledger()` prüft Kette und MAC; bei Fehler `LedgerCorrupt`: Erase-Aufträge werden abgelehnt (fail-closed), Lesen bleibt möglich.
- **Retention:** `LedgerSink::prune(older_than)` schreibt einen `PruneMarker` mit dem Hash des abgeschnittenen Teils, damit die Kette fortsetzbar bleibt.
- **Optional (WP-12\*):** Ed25519-Signatur des Kopf-Hashs für Dritte. Nicht Teil des Kernumfangs, weil D-06 die Signaturkette bewusst entfernt hat.

### 4.10 API-Oberfläche und Semantik

```rust
impl Contextra {
    pub async fn erase(&self, req: EraseRequest) -> Result<ErasureHandle>;          // kehrt nach G1 zurück, await_level optional
    pub async fn erase_dry_run(&self, t: EraseTarget) -> Result<ErasurePlanSummary>; // Mengen, keine Inhalte
    pub async fn erase_status(&self, id: ErasureId) -> Result<ErasureStatus>;
    pub async fn erase_resume(&self, id: ErasureId) -> Result<()>;                   // Operator, nie ungaten
    pub async fn verify_erasure_ledger(&self) -> Result<LedgerVerification>;
}
pub struct EraseRequest { pub id: ErasureId, pub target: EraseTarget, pub reason: ReasonCode,
                          pub policy: ErasurePolicy, pub await_level: Option<Guarantee> }
pub struct ErasurePolicy { pub checkpoint: CheckpointTaintPolicy, pub reinsert: ReinsertPolicy,
                           pub sla: Duration, pub allow_g2_unavailable: bool }
pub struct ErasureStatus  { pub state: ErasureState, pub reached: Guarantee,
                            pub layers: Vec<LayerStatus>, pub exceptions: Vec<Exception>, pub ledger_index: Option<u64> }
```

| Operation | Klasse | Garantie |
| --- | --- | --- |
| `delete(id)`, `forget`, `ctx_retire` | Retire | G0 (Dokumentation und MCP-Antwort weisen „tombstone“ ausdrücklich aus) |
| `erase(Document \| Subject)` | Erase | G1 sofort, G2 typisch < 1 s, G3 im SLA |
| `drop_collection`, `contextra_drop_collection` | Erase(Collection) | wie `erase`, ersetzt den heutigen Tombstone-Pfad |
| Profil *regulated* mit `delete_implies_erase = true` | `delete` löst zusätzlich `erase(Document)` aus | wie `erase` |

- **Fehler im Normalbetrieb:** Zugriff auf gegatete Inhalte liefert `ContextraError::Erased` (statt „not found“), damit Clients nicht in Wiederholungen oder falsche Annahmen laufen. Details enthalten keine Inhalte.
- **MCP:** `ctx_erase` (Schreib-Tool, `confirm_required`, `reason` Pflicht, `dry_run` Default `true`) und `ctx_erase_status`; Klassifikation in `tool_registry_classification`. `contextra_drop_collection` ruft `erase` auf; die Antwort enthält `semantics: "erase"`, `reached` und `erasure_id`.
- **Berechtigung:** Neue Capability `erase` in `capabilities.toml`; `Tenant`-Scope erfordert Administratorrolle.
- **Idempotenz:** Wiederholung mit gleicher `ErasureId` liefert den bestehenden Status.

## 5. Fehler-, Crash- und Konsistenzmodell

### 5.1 Crash-Matrix

Die Matrix ist zugleich Testspezifikation (Abschnitt 7): Jeder Crash-Punkt wird mit `FaultInjector`/`fault_vfs` systematisch erzwungen.

| ID | Crash-Punkt | Zustand nach Neustart | Wiederaufnahme |
| --- | --- | --- | --- |
| CP-1 | nach R1, vor R2 | `Requested` | neu planen (idempotent) |
| CP-2 | nach R2, vor R3 | `Planned`, nichts sichtbar | Gating wiederholen |
| CP-3 | nach R3, vor RAM-Veröffentlichung | Gate steht im Journal | Gate beim Boot **vor** dem Verkehr laden: G1 gilt ohne Lücke |
| CP-4 | nach Veröffentlichung, vor R4 | wie CP-3 | R4 nachtragen |
| CP-5 | Vault-Rewrite, Temp-Datei, vor `rename` | alte Vault-Generation gültig | Temp löschen, Generation neu schreiben |
| CP-6 | nach `rename`, vor `fsync` Verzeichnis | Generation unbestimmt | neu schreiben (idempotent: Generation n+1 ohne Salt) |
| CP-7 | Vault fertig, vor R5 | Salts fehlen im Vault | Vault prüfen, R5 nachtragen |
| CP-8 | Layer L mitten im Purge | `LayerProgress{cursor}` | ab Cursor fortsetzen, `*.erase.tmp` entfernen |
| CP-9 | nach Manifest-Swap, vor Progress-Eintrag | Daten entfernt | Wiederholung findet nichts, No-op, Progress nachtragen |
| CP-10 | in `Verifying` | `Purging` abgeschlossen | Scan wiederholen |
| CP-11 | nach Ledger-Append, vor R8 | Ledger-Eintrag vorhanden | Idempotenz über `commitment`, kein Doppeleintrag |
| CP-12 | Gate-Verdichtung | alter oder neuer Snapshot | beide sicher: Verdichtung nur monoton |

### 5.2 Fehlerklassen und Politik

| Klasse | Beispiele | Reaktion |
| --- | --- | --- |
| `Retryable` | transientes `EIO`, Lock-Timeout, `ENOSPC` | Backoff 1 s bis 5 min mit Jitter (Clock-Port), nach 10 Versuchen `Quarantined(layer)` mit Alarm, stündliche Wiederholung läuft weiter |
| `Permanent` | ungültiger Scope, `ScopeAmbiguous`, Mandantenverletzung | Auftrag beenden (nur vor G1) oder Quarantäne mit Alarm |
| `Poisoned` | `fsync`-Fehler | Layer fencen, `HealthState` degradiert, keine Wiederholung auf demselben Deskriptor; Datei neu öffnen und per Prüfsumme validieren. Auftrag bleibt `Gated` |
| `ScopeViolation` | Layer wollte Nachbardaten berühren | Abbruch, Sicherheitsalarm |

**Platzbedarf.** G1 und G2 benötigen nur kleine Schreibvorgänge (Journal, Vault). Dazu werden `reserve_bytes` für Journal und Vault vorab reserviert, damit beide Stufen auch auf einem vollen Datenträger gelingen. Vor G3 prüft ein Preflight den Temp-Bedarf der Erase-Compaction (Größe der Kandidaten-SSTables); fehlt Platz, bleibt der Auftrag bei G2 mit `InsufficientSpace` und Alarm.

### 5.3 Konsistenz zwischen Schichten

Der Mehrschicht-Commit (`CommitIntent`, Kompensation) ist nicht atomar über Indizes (siehe `split_brain_vector_orphan_test`). Erase hängt davon **nicht** ab: Sichtbarkeit ist durch das Gate atomar, die Schichten konvergieren idempotent. Die physische Konsistenz ist „eventual“, begrenzt durch das SLA und überwacht durch Witness.
