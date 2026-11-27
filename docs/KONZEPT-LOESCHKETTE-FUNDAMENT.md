# Contextra – Konzept der vollständigen Löschkette als Fundamentfunktion (ohne neues Crate)

*Stand: 27.11.2026 · Rolle: Principal Rust Engineer (Systemprogrammierung, Durability, Concurrency, RAG-Speicherkerne) · Analysebasis: `tfufuz1/contextra`, `main`, Commit `3b6a19953` · Status: Entwurf zur Entscheidung*

> **Verifikationsgrenze.** Die Analyse ist statisch. In der Analyseumgebung gibt es keine Rust-Toolchain (`cargo`/`rustc` fehlen, `rust-toolchain.toml` verlangt 1.89). Es wurde nichts kompiliert und kein Test ausgeführt. Alle Zeilenangaben beziehen sich auf den genannten Commit.
>
> **Kennzeichnung der Belege.** **✔** = in dieser Analyse am Code gelesen und bestätigt. **◐** = aus dem im Repo liegenden Entwurf `docs/SPEC-ERASURE.md` übernommen, nicht erneut geprüft. **[?]** = offen, in WP-0 zu klären, bevor das betroffene Arbeitspaket beginnt.

---

## 1. Zusammenfassung

**Ausgangslage.** Contextra hat keine Löschkette, sondern Fragmente. `Collection::delete` schreibt zwei Tombstones und markiert Indexeinträge ✔. Die vier Purger (LSM, Text, HNSW, Graph) haben keinen produktiven Aufrufer ✔. Die Ports `ReadGate`, `CommitBarrier`, `PurgeBarrier`, `LedgerSink`, `RecordSealer`, `FileKeyring` und `ErasureLayer` haben keine einzige Implementierung ✔ (`CommitBarrier`/`PurgeBarrier` nur den `Arc<T>`-Weiterleiter). Der `LsmPurger` ist seit Commit `214e679e2` fail-closed gesperrt und liefert immer `Err` ✔.

**Vorgabe dieses Auftrags.** Keine neuen Crates. Die Löschung wird tief im Fundament implementiert, sauber gekapselt, als Hintergrundfunktion, die Ressourcen schont.

**Kernthese.** Löschen ist in einem Mehrschicht-Kern (LSM, WAL, Manifest, Textindex, HNSW, Graph, Checkpoints, Caches) kein Funktionsaufruf, sondern ein **crash-sicheres, nebenläufiges Protokoll**. Es braucht einen durablen Auftrag, einen Lese- und Schreib-Gate, der Wiederauferstehung ausschließt, physisches Entfernen aus *allen* Kopien und einen gemessenen Nachweis. Das Fundament erzwingt die Invarianten. Die obere Schicht entscheidet nur über Zeitpunkt und Tempo.

### Kernentscheidungen

| Nr. | Entscheidung | Begründung |
| --- | --- | --- |
| E1 | **Kein neues Crate.** Ein *Erasure-Kernel* als Modul `contextra-store/src/erasure/` (Journal, Gate, Filter, LSM-/WAL-Purge, Pending-Delete, Ledger). Der *Treiber* (Saga, Scheduling, Scope-Auflösung, Worker) als Modul in `contextra-engine`. Verträge in `contextra-ports`/`-types`. | Layering: Der Store darf nur von `crypto`, `mvcc`, `ports`, `sys`, `types` abhängen ✔ (`capabilities.toml`). Text, Vektor, Graph, Checkpoint kennen nur `ports`/`types` ✔. Nur die Engine sieht alle Schichten ✔. |
| E2 | **Invarianten unten, Policy oben.** Alles, was nicht umgehbar sein darf (Gate, Write-Ahead-Journal, Filter in Flush/Merge/Replay, Löschen der Dateien), liegt im Store. Zeitplanung, Priorität, Scope-Semantik, Layer-Reihenfolge liegen in der Engine. | Kein oberer Code kann den Gate umgehen oder Dateien am Journal vorbei entfernen. |
| E3 | **Zwei Löschklassen.** *Retire* (heute: `delete`, logisch, zeitreisefähig) bleibt. *Erase* ist ein eigener Pfad mit Stufen G1 (gesperrt), G3 (physisch entfernt, gemessen) und optional G2 (kryptographisch gelöscht). | `delete()` ist heute nur ein Tombstone ✔. |
| E4 | **G3 zuerst, G2 danach.** Der Gate (G1) und das physische Entfernen (G3) sind Stufe 1. Kryptographische Löschung (G2) ist Stufe 2 und optional. | Heute versiegelt nichts Daten pro Datensatz: `RecordSealer` hat keine Implementierung, `derive_dek`/`wrap_dek` haben keine Nutzer außerhalb des Crypto-Crates ✔. G2 setzt eine Datenmigration voraus. Ohne sie bleibt G3 die einzige tragfähige Garantie. |
| E5 | **Ein einziger Rewrite-Pfad.** Erase-Compaction ist ein Filter in der bestehenden Merge-Schleife (`compaction/engine.rs`) und im Flush (`lsm/ops/compaction.rs`), kein zweiter SSTable-Rewriter. | Es gibt genau eine Merge-Schleife und genau eine Flush-Schleife ✔. |
| E6 | **Gate-Schlüsselmodell auf Byte-Präfixen, nicht auf `KeyBuf([u8;16])`.** | Store-Schlüssel sind variabel lange Strings (`t:{tenant}:{collection}:…`, `__docid:`, `__txt:`) ✔. Der Port-Typ `KeyBuf`/`[u8;78]` kann sie nicht tragen. |
| E7 | **Hintergrundbetrieb ereignisgesteuert.** G1 sofort und klein, G3 gebündelt, gedrosselt und druckabhängig. Im Leerlauf kein Polling, auf dem Lesepfad ohne aktive Gates ein einziger atomarer Load. | Abschnitt 7. |
| E8 | **Nachweis statt Behauptung.** Die vorhandene Messkette (`LayerScanner` → `ScanSession::finish` → `LayerWitness`) wird verwendet. Nicht gemessene Schichten senken die ausgewiesene Garantie sichtbar. | Scanner für WAL, Text und HNSW existieren bereits ✔. |

---

## 2. Anforderungen und Garantiemodell

### 2.1 Ziele

- **Z1 Vollständigkeit.** Jede Kopie und jedes Derivat wird adressiert: Memtable und immutable Memtables, WAL (aktiv und versiegelt), SSTables über alle Level (inklusive Bloom-/Indexblöcke), Block-Cache, Manifest, Textindex (Postings, `dl:`, `fw:`, Statistik), HNSW (RAM und Datei), Graph, Checkpoints samt Hardlink-Klonen, Snapshots, Embedding- und Query-Caches, Commit-Intents und Altwerte im RAM, Logs.
- **Z2 Korrektheit unter Crash und Nebenläufigkeit.** Nichts wird durch Flush, Compaction, WAL-Replay, Rebuild, Restore oder Konsolidierung wieder lesbar.
- **Z3 Nachweisbarkeit.** Gemessene Restmengen je Schicht, manipulationssicheres Betriebsprotokoll.
- **Z4 Betriebssicherheit.** Begrenzter Einfluss auf Normalbetrieb, Wiederaufnahme nach Absturz, Beobachtbarkeit.
- **Z5 Mandantenisolation.** Ein Auftrag trifft nachweislich nur seinen Scope.

### 2.2 Nicht-Ziele und ehrliche Grenzen

- **Externe Kopien** (Backups, Exporte, Replikate, Client-Caches, LLM-Anbieter) liegen außerhalb. Erase meldet sie als Ausnahme (`OutOfScope`).
- **Flash-Remanenz und Copy-on-Write-Dateisysteme.** Überschreiben ist dort kein Nachweis. Das Konzept stützt G3 deshalb auf *Entfernen und Messen*, nicht auf Überschreiben. Scrubbing ist optional (ADR-B). Die belastbare Antwort auf Remanenz ist G2 (Stufe 2).
- **Heap, Swap, Core-Dumps.** `Bytes` ist refcounted und nicht zeroisierbar. Betriebsvorgabe: kein Swap, keine Core-Dumps. Eigene Puffer werden zeroisiert.
- Keine juristisch signierte Beweiskette. Das Ledger ist ein manipulationssicheres Betriebsprotokoll.

### 2.3 Garantiestufen

| Stufe | Aussage | Mechanismus | Voraussetzung |
| --- | --- | --- | --- |
| **G0 Retire** | Ab Commit in aktuellen Sichten unsichtbar, per Zeitreise sichtbar | Tombstone (heute) | – |
| **G1 Gated** | In *allen* Lesepfaden unlesbar (inklusive Zeitreise, Snapshot, Restore, Export), Schreiben auf den Scope abgewiesen | Gate im Store, Epoche | Stufe 1 |
| **G3 Purged** | Bytes in allen Dateien entfernt, Witness meldet `remaining = 0` je Schicht | Erase-Compaction, Layer-Purge, WAL-Schnitt, Datei-Entsorgung | Stufe 1 |
| **G2 Crypto-Erased** | Primärinhalte in *allen* Kopien unlesbar, weil der Schlüssel vernichtet ist | Salt-/DEK-Vernichtung | Stufe 2, Datensätze versiegelt |

Der Auftrag meldet zu jedem Zeitpunkt die **tatsächlich erreichte** Stufe. „Abgeschlossen“ heißt G3 für alle Schichten, oder G3 mit ausdrücklich dokumentierten Ausnahmen.

### 2.4 Fehlermodell

Prozessabsturz zwischen zwei beliebigen Schritten, `EIO`/`ENOSPC`/`fsync`-Fehler (danach ist der Dateizustand unbestimmt, siehe fsyncgate-Muster im WAL ✔ `PoisonCause`), Torn Writes, nebenläufige Akteure (Writer, Reader, Flush, Compaction, Checkpoint, Konsolidierung, Rebuild), Hash-Kollision der `DocId`, Fehlkonfiguration. Nicht betrachtet: Root-Angreifer im laufenden Prozess.

---

## 3. Befunde am Code (am Commit `3b6a19953`)

Schwere: **K** kritisch, **H** hoch, **M** mittel.

### 3.1 Was heute passiert

| ID | Sev. | Befund | Beleg |
| --- | --- | --- | --- |
| B-01 | K | **`delete` ist nur ein Tombstone.** Es werden `user_key` und `__docid:`-Key tombstoned, Text und Graph werden zum Löschen eingeplant. Embedding und Dokumentkörper bleiben physisch erhalten. | ✔ `engine/src/collection/crud/delete.rs` |
| B-02 | K | **Kein Koordinator.** Kein Aufrufer von `LsmPurger`, `TextPurger`, `HnswPurger`, `GraphPurger` außerhalb ihrer Crates. | ✔ Workspace-Suche |
| B-03 | K | **Keine Implementierung** von `ReadGate`, `ErasureLayer`, `LedgerSink`, `RecordSealer`, `FileKeyring`, `CommitBarrier`/`PurgeBarrier` (nur `Arc`-Weiterleiter). Der Store enthält **keine einzige Gate-Prüfung**. | ✔ `impl … for`-Suche; `grep gate` in `store/src` trifft nur WAL-HMAC-Texte |
| B-04 | K | **Tombstone-GC ist nicht erzwingbar.** Tombstones fallen nur in *voller* Compaction und nur bei `raw_seq < min_snapshot_seq`. Die neueste Version unterhalb des Floors bleibt immer erhalten. Kein Trigger aus dem Löschpfad. Ein einziger Snapshot oder Pin hält alles. | ✔ `store/src/compaction/engine.rs` (`should_gc_tombstone`), `compaction/retention.rs` |
| B-05 | K | **Flush behält Altversionen**: `retain_key_versions(…, floor_seq)` ohne Löschprädikat. | ✔ `store/src/lsm/ops/compaction.rs` (Flush Phase 3) |
| B-06 | H | **Obsolete Dateien werden nur mit `remove_file` entfernt**, Fehler nur geloggt (`warn`/`debug`). Das gilt für kompaktierte SSTables, `.uuid`-Sidecars und versiegelte WAL-Segmente. Ein fehlgeschlagenes Entfernen hinterlässt Klartextreste ohne jede Wiederholung. | ✔ `compaction/engine.rs` (Zeile ~413), `lsm/ops/compaction.rs` (Zeile ~215) |
| B-07 | H | **`delete_prefix` materialisiert alle Schlüssel im RAM** (`scan_prefix(…).collect()`), ist ein Tombstone-Schreiben und nicht gegen parallele Writes abgesichert. `drop_collection` ruft es zweimal auf (Collection, `__txt:`-Namespace); HNSW, Graph und Checkpoints bleiben unberührt. | ✔ `store/src/lsm/ops/write.rs:190`, `engine/src/collection/maintenance.rs` (`drop_collection`) |
| B-08 | H | **`TextPurger` ruft `futures::executor::block_on`** (`block_on_async`) mitten in synchronem Code. Stall- und Deadlock-Risiko im Tokio-Kontext. | ✔ `text/src/inverted/purge.rs:30-34,126,158,172` |
| B-09 | H | **HNSW:** Purge hält `write_mutex` über die gesamte Laufzeit, `update_backlink`-Ergebnis wird mit `let _ =` verworfen, mmap-Knoten werden nur markiert, Datei wird über `std::fs::File::create` geschrieben (nicht über `Vfs`). | ✔ `vector/src/hnsw/purge.rs:42,133`, `hnsw/types.rs:761` |
| B-10 | H | **Graph-Purger** setzt `edge.source_doc_id = Some(doc_id)` und kennt nur Kanten mit Herkunft. Entities, Properties, Hyperedges bleiben. | ✔ `graph/src/purger.rs:102`; Rest ◐ |
| B-11 | H | **`SnapshotRegistry::revoke_covering` hat keinen Aufrufer.** Ein Snapshot oder Pin hält damit den Compaction-Floor beliebig lange fest. | ✔ Aufrufersuche |
| B-12 | H | **Obsolete Intent-Dateien und WAL-Hilfsdateien** werden an mehreren Stellen mit `tokio::fs` direkt statt über `Vfs` geschrieben und entfernt. Das bricht Fehlerinjektion und die Repo-Regel „I/O nur über `Vfs`“. | ✔ `store/src/lsm/recovery.rs` (u. a. Zeilen 449, 462, 656, 725, 860, 878) |
| B-13 | H | **Zielidentität mehrdeutig:** `DocId` ist ein BLAKE3-Hash auf 64 Bit (`from_key`). Kollisionsprüfung existiert nur im Einfügepfad (Reverse-Lookup), nicht in Purgern. | ✔ `types/src/types/domain/ids.rs` (Doku ADR-016) |
| B-14 | M | **Schlüsselhierarchie ohne Nutzer:** `derive_dek`, `wrap_dek`, `AncestorSalts` werden außerhalb des Crypto-Crates nicht verwendet. `legacy-erasure` ist **Default-Feature** von `contextra-crypto`. | ✔ `crypto/Cargo.toml`, `crypto/src/dek.rs` |
| B-15 | M | **`contextra-durable-fs` hat keinen Abhängigen.** Journal, Pending-Delete, Scrub, Fault-Injector sind nur in ihren eigenen Tests verdrahtet. Der Store hat eigene Durability-Bausteine (`Vfs`, `fsync.rs`, `manifest`, `wal`). | ✔ `Cargo.toml`-Suche |
| B-16 | M | **Gate-Port nicht tragfähig:** `ReadGate` hat fail-open-Defaults (`check_raw`→`Open`, `gate_epoch`→`0`), `KeyBuf` hat 16 Byte, `check_raw` erwartet `[u8; 78]`. Store-Schlüssel sind textuell und variabel lang. | ✔ `ports/src/gate.rs`, `store/src/tenant_codec.rs` |
| B-17 | M | **Checkpoint-Restore und Hardlink-Klone** kennen kein Gate. `CheckpointMeta.metadata` ist freies JSON. | ✔ `checkpoint/src/hardlink_cloner.rs`, `manifest.rs`; Restore-Pfad ◐ |
| B-18 | M | **Altwerte im Commit-Pfad:** `record_keys_with_old_values` hält Alt-Bytes im RAM; Lebensdauer nicht belegt. Der Delete-Pfad trägt `skip(self)` ohne `id`, `id` landet daher im Tracing-Span. | ✔ `delete.rs` (`#[tracing::instrument(level="trace", skip(self))]`); Rest ◐ |
| B-19 | M | **Test- und Nachweislage:** drei Purger-Tests sind `#[ignore]`, die Canary-Prüfung `T-DEL-1` ist laut Kommentar rot. | ✔ `store/src/scanner/tests.rs:90,178,204`; ◐ `contextra/tests/deletion_e2e.rs` |

### 3.2 Was tragfähig ist und wiederverwendet wird

| Baustein | Ort | Nutzung |
| --- | --- | --- |
| `Vfs`-Port mit `write_atomic`, `rename`, `sync_dir`, `remove_file` | `ports/src/vfs.rs` ✔ | Gesamtes Erasure-I/O (Repo-Regel „I/O nur über `Vfs`“) |
| Manifest-Framing mit Länge, CRC und Rollover | `store/src/manifest/*` ✔ | Vorlage für das Journal |
| WAL mit Rotation, Versiegelung, Poisoning | `store/src/wal/*`, `lsm/ops/compaction.rs` ✔ | WAL-Schnitt = Rotation + Flush |
| Merge-Schleife mit Token-Bucket (`max_io_bytes_per_second`) und Konkurrenzprüfung unter Write-Lock | `compaction/engine.rs` ✔ | Erase-Filter, Drosselung |
| `SystemPressureMonitor` (`watch`-Kanal, Stufen Normal/Elevated/Critical) | `store/src/system_pressure.rs` ✔ | Drosselung von G3 |
| Hintergrund-Worker-Muster (`interval`, `MissedTickBehavior::Skip`, `CancellationToken`, `MAX_…_PER_TICK`) | `engine/src/background_workers/*` ✔ | Erasure-Worker |
| `KeyMaterialSource`, `OsKeyMaterialSource`, `StaticKeyMaterialSource` | `store/src/key_source.rs` ✔ | Schlüsselquelle für Ledger-MAC und Vault |
| Mess-Kette `LayerScanner`/`ScanSession`/`LayerWitness` (Mint-Schutz per Compile-Fail-Doctest) | `ports/src/scanner/*` ✔ | Nachweis; Scanner für WAL, Text, HNSW vorhanden |
| `SnapshotRegistry::revoke_covering` | `mvcc/src/snapshot.rs` ✔ | Snapshots beim Gate mandantengenau entziehen |
| `Clock`-Port, Lock-Rangordnung (`lsm/lock_order.rs`) | `ports`, `store` ✔ | Zeit, Ränge |

### 3.3 Verbindliche Implementierungsvorgaben aus dem Repo

- `contextra-store`: `#![forbid(unsafe_code)]`, `#![deny(dead_code)]`, `deny(unwrap_used, expect_used, panic, todo, unimplemented, unreachable, indexing_slicing)` ✔. Neuer Code muss verdrahtet sein oder öffentlich exportiert werden.
- Clippy-Verbote (P28): `Instant::now`, `SystemTime::now`, `std::env::var`, `rand::thread_rng` (außer Schlüsselmaterial), `std::collections::HashMap/HashSet` ✔. Zeit nur über `Clock`, Maps über `BTreeMap`/`ahash`.
- Abhängigkeitskanten stehen in `capabilities.toml` (`may_depend_on`) und werden von `tests/layering.rs` geprüft ✔. **Dieses Konzept benötigt keine neue Kante.**
- Schichtenregel „I/O nur über `Vfs`“ (`ports/src/vfs.rs`) ✔.

---

## 4. Zielarchitektur ohne neues Crate

### 4.1 Platzierung

```text
Ring 0   contextra-types     ErasureId, EraseTarget, Guarantee, ErasureState, Status, Fehler (Erased)
         contextra-ports     ReadGate v2, ErasureLayer, ScopeResolver, ErasureKernel (Port), LedgerSink
         contextra-crypto    SaltVault (rein im RAM, Stufe 2); Rückbau legacy-erasure
Ring 0/1 contextra-text      TextErasure     ─┐
         contextra-vector    HnswErasure      │  implementieren ErasureLayer
         contextra-graph     GraphErasure     │  (kennen nur ports/types)
         contextra-checkpoint CheckpointErasure┘
Ring 1   contextra-store     erasure/  ◄── ERASURE-KERNEL (durabler Zustand + Durchsetzung)
                               journal · gate · filter · purge_lsm · pending · ledger · vault_store
Ring 3   contextra-engine    erasure/  ◄── ERASURE-TREIBER (Saga, Scope-Auflösung, Layer-Reihenfolge)
                             background_workers/erasure_worker.rs ◄── Scheduling, Budgets
         contextra            Fassade: erase(), erase_status(), erase_dry_run()
         contextra-mcp        ctx_erase, ctx_erase_status
```

### 4.2 Verantwortlichkeiten und Begründung der Platzierung

| Komponente | Crate und Modul | Begründung |
| --- | --- | --- |
| Journal (Write-Ahead der Aufträge) | `store/src/erasure/journal.rs` | Muss vor dem WAL-Replay lesbar sein und gehört zum Boot des Stores. |
| Gate | `store/src/erasure/gate.rs` | Muss in Flush, Merge, Replay, Commit und Lesepfad ohne Umweg greifen. Eine Auslagerung nach oben ließe sich umgehen. |
| Filter (Flush/Merge/Replay) | `store/src/erasure/filter.rs` | Ein Trait-Objekt, das an den drei Schleifen hängt. |
| LSM-/WAL-Purge, Pending-Delete | `store/src/erasure/purge_lsm.rs`, `pending.rs` | Ersetzt `store/src/scanner/purge.rs` (gesperrter `LsmPurger`). Kennt Manifest, Level, WAL-Rotation. |
| Ledger | `store/src/erasure/ledger.rs` | Implementiert `LedgerSink`; teilt Framing und fsync-Disziplin mit Journal. |
| Vault-Persistenz (Stufe 2) | `store/src/erasure/vault_store.rs` | Kryptographische Primitive bleiben rein in `crypto`, Dateizugriff in `store` über `Vfs`. |
| Treiber | `engine/src/erasure/` | Kennt alle Schichten, `Collection`-Schlüsselbildung (`namespaced_key`), Provenienz. |
| Worker | `engine/src/background_workers/erasure_worker.rs` | Dort liegt bereits das Worker-Muster samt Abschaltung. |
| Layer-Adapter | je Crate (`text`, `vector`, `graph`, `checkpoint`) | Nur `ports`/`types` nötig. |

### 4.3 Die einzige öffentliche Fläche des Kernels

Der Kernel exponiert nach oben ausschließlich den Port `ErasureKernel`. Alles andere ist `pub(crate)`. Die Engine kann damit weder Gate noch Journal direkt manipulieren.

```rust
// contextra-ports/src/erasure.rs  (ersetzt den heutigen Platzhalter-Port)
pub trait ErasureKernel: Send + Sync {
    /// R1+R2: Auftrag annehmen, Plan durabel ablegen. Idempotent über (id, plan_hash).
    fn accept<'a>(&'a self, req: &'a EraseRequest, plan: &'a ResolvedScope)
        -> BoxFuture<'a, Result<AcceptReceipt, ErasureError>>;
    /// R3+R4: Gate veröffentlichen. Kehrt zurück, wenn G1 gilt.
    fn publish_gate<'a>(&'a self, id: ErasureId) -> BoxFuture<'a, Result<GateReceipt, ErasureError>>;
    /// LSM, Memtable, WAL, Manifest, Pending-Delete. Wiederaufnehmbar über Cursor, gedrosselt über Budget.
    fn purge_store<'a>(&'a self, id: ErasureId, cx: &'a PurgeCx<'a>)
        -> BoxFuture<'a, Result<PurgeOutcome, ErasureError>>;
    /// Fortschritt der Layer-Purger durabel festhalten (Write-Ahead).
    fn record_layer<'a>(&'a self, id: ErasureId, layer: LayerId, p: LayerProgress)
        -> BoxFuture<'a, Result<(), ErasureError>>;
    /// Witness je Schicht entgegennehmen, Stufe bestimmen, Ledger schreiben, Gate-Lebenszyklus beenden.
    fn seal<'a>(&'a self, id: ErasureId, w: &'a [LayerWitness])
        -> BoxFuture<'a, Result<SealReceipt, ErasureError>>;
    /// Offene Aufträge für den Treiber (nach Neustart, nach Fehlern).
    fn open_jobs(&self) -> Vec<JobView>;
    fn status(&self, id: ErasureId) -> Option<ErasureStatus>;
    /// Nur Lesezugriff auf gesperrte Daten, nur für Purger, nur solange der Auftrag läuft.
    fn privileged_reader(&self, id: ErasureId) -> Result<PrivilegedReader, ErasureError>;
}
```

`ReadGate` v2 hat **keine** Default-Implementierung. Wer ihn implementiert, muss jede Methode entscheiden (fail-closed, B-16).

---

## 5. Kernkomponenten im Detail

### 5.1 Identität, Scope und Provenienz

**Auftragsidentität.** `ErasureId` (128 Bit) kommt aus dem `IdGen`-Port (deterministisch in Tests) und ist zugleich Idempotenzschlüssel. Gleiche `ErasureId` mit gleichem `plan_hash` liefert den bestehenden Auftrag, mit abweichendem `plan_hash` den Fehler `IdempotencyConflict`.

**Typisiertes Ziel statt String** (ersetzt `ScanTarget { target_id: String }`, B-13):

```rust
pub enum EraseTarget {
    Document   { tenant: TenantId, collection: CollectionName, key: String },
    Collection { tenant: TenantId, collection: CollectionName },
    Tenant     { tenant: TenantId },
    // Subject { .. } nur mit Subjekt-Index, siehe ADR-E
}
```

**Auflösung (Planung)** läuft im Treiber (er kennt `namespaced_key` und die Provenienz) und ist read-only und deterministisch. Ergebnis ist ein `ResolvedScope` mit zwei Sichten:

- **Schlüsselsicht** (für den Store): eine Menge von **Byte-Präfixen oder Exaktschlüsseln** der Rohschlüssel, die dem Scope zugeordnet werden können (Nutzerschlüssel, `__docid:`-Schlüssel, `dl:`, `fw:`, Collection-Präfix, `__txt:{collection}:`, Mandantenpräfix `t:{tenant}:`).
- **Dokumentsicht** (für die Indizes): `(TenantId, CollectionName, DocId)`-Mengen. Posting-Listen (`i:{term}`) sind nicht dokumentgebunden und können deshalb nur über die Dokumentsicht gefiltert werden.

Pflichtprüfungen, bevor der Plan persistiert wird:

1. **Schlüsselverifikation.** Der über den Reverse-Lookup (`__docid:`) gefundene Originalschlüssel muss dem angefragten Schlüssel byte-gleich entsprechen. Abweichung (Hash-Kollision) führt zu `ScopeAmbiguous`, nie zu Über-Löschung.
2. **Mandantenprüfung.** Jeder aufgelöste Rohschlüssel trägt das Mandantenpräfix des Auftrags. Verletzung ist ein Invariantenbruch (Abbruch, Alarm).
3. **Keine freien Strings** in Layer-APIs. Layer erhalten ausschließlich `ResolvedScope`.
4. **`plan_hash`** = BLAKE3 über die kanonische Serialisierung. `erase_dry_run` nutzt denselben Code und liefert nur Mengen.

**Provenienz abgeleiteter Daten.** Kanten, Entities, Hyperedges, Communities und Summaries tragen `derived_from: SmallVec<[DocId; 2]>` (Pflicht, nicht `Option`). Leere Restmenge → Objekt wird gepurged. Mehrere Quellen → Quelle entfernen, inhaltsabhängige Felder auf `NeedsRecompute`. Altbestand ohne Provenienz wird im Plan als **nicht verifizierbar** ausgewiesen und senkt die gemeldete Garantie (Migration in WP-8).

### 5.2 Journal (Write-Ahead der Aufträge)

- **Ablage.** `<data>/erasure/journal.log`, ausschließlich über `Vfs`.
- **Framing.** Wie im Manifest: Längenpräfix, CRC32 über den Eintrag, Binärformat mit Versionsbyte. Kein `serde_json`, keine Inhalte.
- **Einträge** (Auszug): `Requested{id, target_hash}`, `Planned{id, plan_hash, layers, scope_summary}`, `GateIntent{id}`, `Gated{id, gated_seq, epoch}`, `LayerProgress{id, layer, cursor, attempt}`, `StoreProgress{id, cursor}`, `Verified{id, witness_root}`, `Sealed{id, ledger_index, guarantee, exceptions}`, `Failed{id, layer, kind}`, `Quarantined{id, layer}`.
- **Persistente Zielbeschreibung.** Der Plan enthält die Präfixe und Exaktschlüssel (nötig für Gate und Filter nach Neustart). Das sind Schlüsselbytes und damit potenziell personenbezogen. Deshalb gilt: Das Journal liegt im Datenverzeichnis unter denselben Zugriffsrechten wie WAL und SSTables, enthält **keine Dokumentinhalte, keine Terme, keine Embeddings**, und wird nach `Sealed` auf die Kurzform (nur HMAC des Plans) verdichtet. Das Verdichten ist ein Rollover wie beim Manifest (`manifest/rollover.rs` ✔).
- **Durabilität.** Jeder Zustandsübergang wird vor seiner Wirkung geschrieben und mit `sync_data`/`sync_all` gesichert. Ein `fsync`-Fehler vergiftet das Journal (kein erneuter Versuch auf demselben Deskriptor), der Auftrag bleibt in seinem letzten Zustand, der Gate bleibt aktiv (fail-closed). Das übernimmt das Muster des WAL (`PoisonCause`).
- **Recovery.** Beim Start: Journal lesen, Torn Tail abschneiden (nur ganz am Ende, CRC-geprüft), Zustandsautomaten pro Auftrag neu aufbauen, Gates *vor* dem WAL-Replay laden.

### 5.3 Zustandsautomat

Jeder Übergang ist write-ahead. **Ab `Gated` gibt es keinen Rücksprung** (nur Beenden oder Quarantäne).

| Zustand | Aktionen | Marker | Stufe |
| --- | --- | --- | --- |
| `Requested` | Validierung, Berechtigung, Idempotenz | `Requested` | – |
| `Planned` | Scope auflösen, Layer-Menge festlegen | `Planned` | – |
| `Gating` | `GateIntent` schreiben, unter `commit_mutex` (Rang 200) `gated_seq` lesen, Gate-Snapshot veröffentlichen, Epoche erhöhen, `revoke_covering`, Caches räumen, `Gated` schreiben | `GateIntent`, `Gated` | → G1 |
| `Purging` | Layer in fester Reihenfolge, je Layer idempotent mit Cursor (5.7) | `LayerProgress`, `StoreProgress` | G1 |
| `Verifying` | `LayerScanner::scan` je Schicht; Restmenge > 0 → zurück nach `Purging` (`attempt++`, begrenzt) | `Verified` | G1 |
| `Sealed` | Ledger-Eintrag (fsync), dann Journal | `Sealed` | → G3 |
| `Completed` | Gate-Lebenszyklus beenden (5.4) | – | G3 |
| `Failed(retryable)` / `Quarantined(layer)` | Backoff mit Jitter, nach Grenzwert Quarantäne und Alarm; Wiederholung läuft stündlich weiter | `Failed`, `Quarantined` | hält letzte Stufe |

Stufe 2 fügt zwischen `Gated` und `Purging` den Zustand `KeysDestroying` ein (5.8).

**Layer-Reihenfolge in `Purging`:** (1) RAM-Caches, (2) Text, Vektor, Graph, (3) Store (Memtable, Erase-Compaction, WAL-Schnitt, Datei-Entsorgung), (4) Checkpoints. Begründung: Der Textindex muss das Forward-Index-Dokument `fw:{doc}` lesen, um die betroffenen Terme zu kennen ✔ (`text/src/inverted/purge.rs:167`), und der Graph navigiert über die Provenienz im Store. Beide lesen über den `PrivilegedReader` des Kernels (ein Lesezugriff, der den Gate bewusst umgeht, nur für den laufenden Auftrag).

**Invarianten**

| ID | Invariante |
| --- | --- |
| I-1 | Reihenfolge Gate → Purge → Witness → Seal wird nie vertauscht oder übersprungen. |
| I-2 | **Write-Ahead:** Jede sichtbare oder destruktive Wirkung folgt einem durablen Journal-Eintrag. |
| I-3 | **Kein Wiederaufleben:** Flush, Compaction, WAL-Replay, Rebuild, Backfill, Restore und Konsolidierung wenden den Gate an. |
| I-4 | **Idempotenz:** Jeder Schritt und jeder Layer-Purge ist beliebig oft wiederholbar. |
| I-5 | **Epoche** steigt streng monoton und wird vor jeder zerstörenden Wirkung veröffentlicht (Release/Acquire). |
| I-6 | **Witness-Ehrlichkeit:** `LayerWitness` entsteht nur in `ScanSession::finish`. `NotMeasured` senkt die ausgewiesene Stufe. |
| I-7 | **Fail-closed:** Nicht ladbarer oder unbekannter Gate-Zustand wirkt wie `Restricted`. Gates werden nie wegen Fehlern oder Zeitablauf geöffnet. |
| I-8 | **Scope-Treue:** Ein Layer berührt nur Objekte aus `ResolvedScope`. |
| I-9 | **Gate-Lebensdauer:** Ein Gate-Eintrag wird erst entfernt, wenn der Witness über *alle* Schichten `remaining = 0` meldet und kein markierter Checkpoint oder älterer Snapshot mehr existiert. Sonst bliebe er dauerhaft als `Erased(up_to)` stehen. |
| I-10 | **Eine Lock-Ordnung** (6.1). Keine Sperre über fremde `.await`-Punkte. |
| I-11 | **Keine Blockade durch Dritte:** Pins, Snapshots, Checkpoints können G1 nie verhindern, nur G3 verzögern (mit Alarm). |
| I-12 | **Kein Klartext in Betriebsdaten:** Journal und Ledger enthalten nur Schlüsselbytes des Plans (und nach `Sealed` nur HMACs), keine Inhalte, keine Terme. Tracing und Metriken erhalten Schlüssel- und Dokument-IDs nur als HMAC. |

### 5.4 Gate

**Struktur** (`store/src/erasure/gate.rs`). Ein unveränderlicher `GateSnapshot` hinter einem lesefreundlichen Zeiger. Start mit `parking_lot::RwLock<Arc<GateSnapshot>>` (`parking_lot` ist bereits Store-Abhängigkeit ✔). Falls die Messung (WP-3) das Budget verfehlt, Wechsel auf `arc_swap` (bereits im Workspace vorhanden, `contextra-graph` nutzt `arc-swap = "1.7"` ✔, daher kein neuer Lock-Eintrag nötig). Writer bauen den Snapshot copy-on-write und veröffentlichen ihn atomar.

```rust
struct GateSnapshot {
    epoch: u64,
    key_ranges: Vec<(KeyInterval, GateEntry)>, // sortiert, nicht überlappend: Roh-Byte-Präfixe/Exaktschlüssel
    docs: BTreeMap<TenantId, BTreeMap<CollectionId, BTreeSet<DocId>>>, // Dokumentsicht der Indizes
}
enum GateEntry { Restricted { since: Seq }, Erased { up_to: Seq } }
```

**Schnellpfad ohne Kosten.** Ein `AtomicBool gates_active` (Relaxed-Load). Solange kein Auftrag einen unabgeschlossenen Gate-Eintrag hat, kostet jede Prüfung einen einzigen Load. Mit aktiven Einträgen: O(log n) in RAM, keine I/O.

**Semantik.** `Restricted` = darf weder gelesen noch von der Compaction verworfen werden. `Erased` = unlesbar, darf verworfen werden. Der Port wird um `check_version(key, version_seq)` erweitert: Wird derselbe Schlüssel nach `Sealed` neu angelegt (`seq > up_to`), ist die neue Version lesbar, alte Versionen bleiben gesperrt (Zeitreise).

**Linearisierbarer Mehrschritt-Lesepfad** (Kandidaten → Hydration): `e1 := epoch (Acquire)`, Gate prüfen, lesen, `e2 := epoch`; bei `e1 != e2` wiederholen. Linearisierungspunkt eines Erase ist die Veröffentlichung des Snapshots.

**Durchsetzungspunkte.** Jeder bekommt einen Regressionstest.

| Pfad | Einhängung | Maßnahme |
| --- | --- | --- |
| Punkt-Get, MVCC-Read | `LsmStorage` (unterhalb von `TenantScopedStorage`) | Prüfung vor und nach dem Lesen |
| Scan, Prefix, Range | `lsm/scan.rs`, `scan_prefix` | `variant_for_range`; Erased überspringen, Restricted nach Policy |
| WAL-Replay | `lsm/recovery.rs` | Filter **vor** Anwendung auf die Memtable; Gates sind zu diesem Zeitpunkt geladen **[?]** genauer Einhängepunkt in WP-0 |
| Flush | Flush Phase 3, Schleife über `key_map` (`lsm/ops/compaction.rs`) ✔ | Filter vor `builder.add` |
| Compaction | Merge-Schleife (`compaction/engine.rs`) ✔ | Filter vor `builder.add` (5.5) |
| Commit | Group-Commit-Leader (`lsm/commit.rs`, `group_commit.rs`) **[?]** | Writes auf gesperrte Schlüssel abweisen (`gated_seq`) |
| Textsuche | Kandidatenstrom (BM25/WAND) | Postings gegen Dokumentsicht filtern, Hydration erneut prüfen |
| Vektorsuche | HNSW-Traversierung | `deleted_nodes` ∪ Dokumentsicht maskieren, Über-Fetch |
| Graph | Nachbarn, Traversierung | Provenienz gegen Dokumentsicht |
| Zeitreise | `get_at_seq`, `temporal_filter` | Gate gilt für **alle** Sequenzen |
| Checkpoint-Restore, Export, Import | `restore_checkpoint`, `export.rs`, `import.rs` | Ablehnung bei markierten Checkpoints, Filter |
| Caches | Embedding-, KV-, Query-, Block-Cache | Räumen beim Gate, Epoche im Cache-Schlüssel |

**Start-Reihenfolge.** (1) `Vfs`/Verzeichnis prüfen, (2) Journal öffnen, Gates laden (nicht ladbar → Store startet im Zustand „alles `Restricted`“, kein Fail-open), (3) Manifest laden, (4) **WAL-Replay mit Filter**, (5) Layer öffnen, (6) offene Aufträge dem Treiber melden, (7) Verkehr annehmen.

### 5.5 Physisches Entfernen im Store

**Memtable und immutable Memtables.** Schlüssel des Scopes werden unter kurzer `state.write()`-Sperre (Rang 400) entfernt. Wartende immutable Memtables werden beim Flush gefiltert (I-3).

**Erase-Compaction als Filter in der vorhandenen Merge-Schleife.** Kein zweiter SSTable-Rewriter (E5).

1. **Kandidaten (die Hülle).** Alle SSTables, die Scope-Schlüssel enthalten können (Min/Max-Schlüssel, Bloom-Probe), über **alle** Level. Die Hülle ist zwingend: Würde ein Tombstone verworfen, während ältere Werte in nicht ausgewählten Dateien liegen, leben sie nach dem Gate-Ende wieder auf (I-9).
2. **Prädikat.** `DropKeys(scope)` verwirft jede Version unabhängig vom Snapshot-Floor. Das ersetzt die Regel „neueste Version unter dem Floor bleibt“ (B-04) für Scope-Schlüssel. Snapshots sind durch das Gate geschützt. `CollapseVersions(index_keys)` gilt für umgeschriebene Index-Schlüssel (Posting-Listen `i:…`): ältere Versionen fallen, weil sie die Dokument-Term-Zuordnung noch enthalten.
3. **Ausgabe.** Bestehender `SstableBuilder` mit demselben `key_manager` (verschlüsselte Speicher werden nicht übersehen), neue Datei-IDs, **Manifest-Eintrag `Replace` als Commit-Punkt**.
4. **Streaming.** Keine Materialisierung ganzer Dateien oder Schlüsselmengen. `delete_prefix` (B-07) wird zu seitenweisem Schreiben (Seiten à 1 024 Schlüssel, Cursor im Journal).
5. **Fehler.** Jeder I/O-Fehler beendet den Job typisiert. Gezählt wird nur, was durch den Manifest-Eintrag committet ist. Kein `let _ =`, kein `unwrap_or_default` im Pfad (CI-Gate, WP-0).
6. **Konkurrenz.** Die bestehende Prüfung „Eingabedateien nicht mehr zusammenhängend → Abbruch mit `Ok(false)`“ ✔ gilt auch hier. Der Erase-Job wiederholt dann mit neuem Kandidatensatz (kein Fehler).
7. **Crash.** Temporäre `*.erase.tmp` stehen im Journal und werden beim Start entfernt. Der Manifest-Eintrag ist der Commit-Punkt, Wiederholung findet nichts mehr im Scope (No-op).

**WAL-Schnitt.** Das WAL rotiert bereits: Flush versiegelt das aktive Segment, legt ein neues an und löscht nach erfolgreichem Flush alle versiegelten Segmente ✔. Der Schnitt ist deshalb **erzwungene Rotation + Flush mit Filter + Entfernen der Segmente**. Kein Einzel-Record wird entfernt (HMAC-Kette bleibt intakt). Der fehlgeschlagene Löschversuch eines Segments wird nicht mehr nur geloggt (B-06), sondern in das Pending-Delete-Verzeichnis eingetragen.

**Pending-Delete.** Kleine Datei über `Vfs` (`write_atomic`), Liste von Pfaden. Jede obsolete SSTable, jedes WAL-Segment und jede Sidecar-Datei wird erst dort eingetragen und dann entfernt (`remove_file`, danach `sync_dir`). Ein Neustart arbeitet die Liste ab. Fehler werden gezählt und wiederholt, nicht verschluckt.

**Scrubbing (Überschreiben vor dem Entfernen).** Ist optional (ADR-B). Begründung: Auf SSD, CoW und Journaling-Dateisystemen ist es kein Nachweis, und die vorhandene Logik liegt in `contextra-durable-fs`, das der Store laut `capabilities.toml` nicht einbinden darf ✔. Default: `remove_file` + Verzeichnis-`fsync` + Witness. Wer Remanenz ausschließen muss, nutzt Stufe 2 (G2).

**Caches und Hilfsstrukturen.** Block-Cache-Einträge ersetzter Dateien werden beim Manifest-Swap geräumt. Alte Manifest-Generationen und Footer-Min/Max-Schlüssel können Schlüsselbytes enthalten. Ob alte Manifest-Generationen aufbewahrt werden, klärt WP-0 **[?]**. Der Witness (`SstScanner`/`WalScanner` existieren ✔) deckt das ab.

### 5.6 Commit-Pfad und Residuen

- Der Gate-Check im Commit-Leader weist Writes auf gesperrte Schlüssel mit `ContextraError::Erased` ab (kein „not found“, damit Clients nicht wiederholen).
- `record_keys_with_old_values` und das Kompensations-Ledger halten Altwerte nur bis Commit/Rollback und werden danach zeroisiert ◐ (WP-9).
- `CommitIntent`-Dateien werden nach Abschluss entfernt und laufen über `Vfs` (B-12) ◐.
- Telemetrie: `#[tracing::instrument]` auf Funktionen mit `id`/`key`-Parametern ohne `skip` wird per CI-Gate verboten (Beispiel: `Collection::delete`, B-18).

### 5.7 Layer-Vertrag

Der Platzhalter-Port `ErasureLayer` (ohne Implementierung, Fehler als `String`, `Scanner<Armed>` als leerer Typ) wird ersetzt. Die vier vorhandenen `Purger` (synchron, `ScanTarget` mit String-ID) werden auf den neuen Vertrag umgestellt. Die Mess-Kette `LayerScanner`/`ScanSession`/`LayerWitness` bleibt unverändert (INV-WITNESS-MINT-1).

```rust
pub trait ErasureLayer: Send + Sync {
    fn layer(&self) -> WitnessLayer;                              // vorhandener Typ
    /// RAM-Maskierung und Cache-Räumung, idempotent, schnell.
    fn on_gate<'a>(&'a self, s: &'a ResolvedScope) -> BoxFuture<'a, Result<(), LayerError>>;
    /// Wiederaufnehmbar über cx.cursor. Meldet nur committete Arbeit.
    fn purge<'a>(&'a self, s: &'a ResolvedScope, cx: &'a PurgeCx<'a>)
        -> BoxFuture<'a, Result<PurgeOutcome, LayerError>>;
    fn scanner(&self) -> &dyn LayerScanner;                       // Witness nur über ScanSession
}

pub struct PurgeCx<'a> {
    pub erasure_id: ErasureId, pub clock: &'a dyn Clock, pub cancel: &'a CancellationToken,
    pub budget: &'a IoBudget, pub cursor: Option<Cursor>, pub reader: &'a PrivilegedReader,
    pub progress: &'a dyn ProgressSink,
}
pub enum LayerErrorKind { Retryable, Permanent, Poisoned, ScopeViolation, Cancelled }
```

Regeln: Teilfehler sind `Err`, nie `Ok` mit Zähler. Kein `block_on` im Purge-Pfad. CPU-lastige Teile über `spawn_blocking` mit Abbruchprüfung. Fehler sind typisiert. Jede Layer-Implementierung muss die Konformanzsuite bestehen (Abschnitt 9).

### 5.8 Text, Vektor, Graph, Checkpoints

**Textindex.** `TextPurger` wird async (Entfernung von `block_on_async`, B-08). Pro Dokument: Postings aller Terme (Terme aus `fw:` über den `PrivilegedReader`), `dl:`, `fw:`, Term-Dictionary-Einträge mit `df = 0`, BM25-Statistik (N, avgdl) atomar, Morphologie-/Trie-Caches. Posting-Listen sind neue LSM-Versionen, die alten fallen per `CollapseVersions` in der Erase-Compaction. Kandidatenfilter über die Dokumentsicht.

**Vektorindex.**

- **Phase A (G1):** Maskierung über `deleted_nodes` ∪ Dokumentsicht, Über-Fetch gleicht Top-k aus.
- **Phase B (G3):** Segment-Rewrite statt Markieren: neues Segment ohne die Knoten bauen, Nachbarn über einen Reverse-Adjazenzindex statt O(Layer × N)-Scan neu verbinden, Temp-Datei, `fsync`, `rename`, `fsync` Verzeichnis, Alt-Mapping nach Reader-Drain freigeben, Altdatei über Pending-Delete entfernen. **Datei-I/O über `Vfs`** (B-09). mmap-Knoten werden nie nur markiert.
- `update_backlink`-Fehler werden propagiert. `write_mutex` wird nicht über Datei-I/O gehalten, nur für den Zeiger-Swap.
- **Recall-Guard:** Stichprobe vor und nach dem Purge. Fällt der Recall um mehr als einen konfigurierbaren Wert, wird ein Teil-Rebuild aus dem LSM-Quellbestand **mit Gate-Filter** geplant.
- **Kosten:** Der Rewrite ist O(Indexgröße). Deshalb Koaleszenzfenster (7.2). G1 gilt sofort.

**Graph.** Pflicht-Provenienz (5.1). Primäre Entities werden entfernt. Abgeleitete Objekte mit leerer Restmenge werden gepurged, andere `NeedsRecompute`. Hyperedges werden entfernt, wenn weniger als zwei Teilnehmer bleiben. Kein `TxId::MAX`, Mandantenprüfung bei jedem Zugriff (B-10). Der Persistenzpfad des Graphen ist zu inventarisieren **[?]** (`contextra-graph/src` enthält keine Datei-I/O außer Testdoubles ✔, vermutlich Persistenz über den Store).

**Checkpoints.** Ein Checkpoint pinnt eine `seq_no` und klont SSTables per Hardlink.

- `restore_checkpoint` prüft den Gate und lehnt **markierte** Checkpoints ab (`seq_no ≥ scope.min_write_seq`).
- Politik `CheckpointTaintPolicy`: `Drop` (Default, nach Karenz), `Rewrite` (gefilterter Klon durch Kopie, Original entpinnen), `LegalHold` (ausdrückliche Ausnahme, Auftrag endet mit dokumentierter Ausnahme).
- Hardlinks: Obsolete SSTables werden mit **allen** Namen entfernt. Erst wenn alle Namen weg sind, gilt die Datei als entfernt.
- `drop_checkpoint` propagiert Entpin-Fehler (heute nur `warn` ◐).
- `CheckpointMeta.metadata` (freies JSON) wird Allowlist oder versiegelt.

**Snapshots und Pins.** Beim Gate wird `SnapshotRegistry::revoke_covering(tenant, 0..gated_seq, gated_seq)` aufgerufen (heute ohne Aufrufer, B-11). Manuell gesetzte Pins erhalten ein Ablaufdatum (`reap_expired_pins` existiert ✔) und einen Alarm bei Überschreitung.

### 5.9 Kryptographische Löschung (Stufe 2, optional)

**Warum später.** Heute ist nichts pro Datensatz versiegelt (B-14). G2 verlangt, dass Datensätze schon beim Schreiben unter einem ableitbaren Schlüssel stehen. Das ist eine Datenmigration und ein Eingriff in den Schreibpfad. Stufe 1 liefert ohne sie bereits eine vollständige, gemessene Löschung (G3).

**Hierarchie** (vorhandene Primitive in `crypto/src/dek.rs` ✔):

```text
Root (KeyMaterialSource: OS-Keystore / Datei / extern)
  └─ UnitKey je Mandant
       └─ AncestorSalts { t, c, (s), d } ── derive_dek(unit, salts, doc_tag) → Dek
            └─ AES-256-GCM-SIV (AAD: Mandant, Collection, Schlüssel-Hash, Typ, Formatversion)
```

Die Vernichtung des Salts auf einer Ebene macht alle darunter liegenden DEKs unableitbar.

**SaltVault.** Reine RAM-Logik in `crypto` (Struktur, Umhüllung mit `wrap_dek`, Rotation). Persistenz in `store/src/erasure/vault_store.rs` über `Vfs::write_atomic` + `sync_dir`. Vernichtung = neue Vault-Generation ohne den Salt schreiben **und** den Vault-Epoch-Schlüssel rotieren (verbleibende Salts neu umhüllen, alten Schlüssel vernichten). Mehrere Aufträge werden in einem Fenster gebündelt, weil jede Rotation O(Vault-Größe) kostet.

**Was versiegelt wird:** `StoredDocument` (Körper, Metadaten, Embedding), WAL-Payloads, `fw:`-Forward-Index, Graph-Entity-Payload, `CheckpointMeta.metadata`. **Nicht versiegelbar:** Postings, Term-Dictionary, HNSW-Vektoren, CSR-Struktur (Suche braucht Klartext). Für sie ist G3 Pflicht.

**Migration.** Records tragen eine Formatversion. Lesen unterstützt versiegelt und unversiegelt (Dual-Read). Altbestand wird bei Compaction versiegelt (seal-on-rewrite) und per Backfill nachgezogen. Bis dahin wird für Altbestand nur G1+G3 ausgewiesen.

**Reihenfolge-Konsequenz.** Mit G2 vor dem Purge kann `fw:` nicht mehr gelesen werden. Deshalb legt die Planung die vom Layer benötigten Zwischenstände (z. B. die Term-Menge) versiegelt unter einem Job-Schlüssel im Journal-Umfeld ab. Der Job-Schlüssel wird erst bei `Sealed` vernichtet.

### 5.10 Witness und Ledger

**Witness.** Je Schicht über die vorhandene Kette. Techniken: `Structural`, `DecryptedView` (verschlüsselte Dateien), `NeedleScan` (Byte-Scan nach Nadeln aus dem Scope über Dateien, WAL, mmap, Checkpoints). Nadeln (Schlüsselbytes, DocId als LE/BE/Dezimal/Hex, Terme) existieren nur im RAM. G3 gilt genau dann, wenn jede Schicht `remaining = 0` meldet. `NotMeasured`/`FeatureDisabled` reduziert die Stufe und erscheint als `exception`. Die `witness_root` ist ein BLAKE3-Digest über die Schicht-Digests in fester Reihenfolge.

**Ledger** (`store/src/erasure/ledger.rs`, implementiert `LedgerSink`). Append-only, Hash-Kette plus MAC (Schlüssel aus `KeyMaterialSource`), Ablage **außerhalb** des Datenverzeichnisses (Profil *regulated* verweigert den Start, wenn der Pfad darunter liegt). Einträge: `erasure_id_hmac`, `guarantee`, `layers_root`, `exceptions`, `plan_hash`, `policy_version`, `completed_at`. Kette plus MAC erkennen Veränderung, nicht Kürzung am Ende; dagegen veröffentlicht ein `LedgerAnchor`-Port periodisch den Kopf-Hash. Bei `LedgerCorrupt` werden neue Erase-Aufträge abgelehnt, Lesen bleibt möglich.

---

## 6. Nebenläufigkeit

### 6.1 Lock-Ordnung

Die vorhandene Rangordnung (`lsm/lock_order.rs` ✔) bleibt. Der Erasure-Kernel ergänzt:

| Rang | Konstante (neu) | Primitiv | Zweck | Haltedauer |
| --- | --- | --- | --- | --- |
| 150 | `RANK_ERASURE_PLAN` | `tokio::sync::Mutex` | Serialisiert Annahme, Gate-Veröffentlichung und Journal-Übergänge **pro Store** | < 5 ms (ein `fsync`) |
| 960 | `RANK_ERASURE_GATE_SNAPSHOT` | `parking_lot::RwLock` (nur Arc-Clone) | Zeiger auf den Gate-Snapshot | < 1 µs, nie über `.await` |

Die Gate-Veröffentlichung nimmt Rang 150, dann **den vorhandenen `commit_mutex` (Rang 200)**, liest `gated_seq`, veröffentlicht den Snapshot, gibt frei. Dadurch ist ohne neues Barrier-Primitiv garantiert, dass kein Commit zwischen „Sequenz gelesen“ und „Gate sichtbar“ liegt. Die Ports `CommitBarrier`/`PurgeBarrier` werden dann nicht benötigt und mit den übrigen Platzhaltern entfernt (Abschnitt 8, WP-2). Ob `commit_mutex` im Group-Commit-Pfad alle Commits tatsächlich serialisiert, ist in WP-0 zu belegen **[?]**.

### 6.2 Bekannte Rennen und Gegenmaßnahmen

| ID | Rennen | Gegenmaßnahme |
| --- | --- | --- |
| R1 | Commit zwischen Sequenzlesen und Gate-Veröffentlichung | Beides unter `commit_mutex` |
| R2 | Flush schreibt Scope-Schlüssel in neue SSTable, während der Gate noch nicht greift | Filter im Flush + Gate vor Purge (I-1) |
| R3 | Compaction und Erase-Compaction arbeiten auf überlappenden SSTables | Bestehende Konkurrenzprüfung unter Write-Lock (`Ok(false)` → Wiederholung) |
| R4 | Leser holt Kandidaten vor dem Gate, hydriert danach | Epochen-Protokoll `e1`/`e2` (5.4) |
| R5 | HNSW-Rebuild läuft während des Purge | Rebuild liest mit Gate-Filter; Zeiger-Swap unter kurzer Sperre |
| R6 | Checkpoint wird während des Gate erstellt | Checkpoint-Erstellung liest die Epoche und markiert sich bei Änderung |
| R7 | Neustart zwischen Journal und RAM-Veröffentlichung | Gates werden beim Boot **vor** dem Replay aus dem Journal geladen |
| R8 | Wiederanlage desselben Schlüssels während des Purge | Policy `reinsert = Deny` bis `Sealed`; danach nur `seq > up_to` |

Für R1, R4, R7 gibt es Loom-Tests (Abschnitt 9). Der Store bindet `loom` bereits ein ✔ (`cfg(loom)`-Abhängigkeit in `engine` und `mvcc`).

---

## 7. Hintergrundbetrieb und Ressourcenschonung

### 7.1 Prinzip

**G1 ist klein und sofort, G3 ist groß und wird verschoben.** Alles, was die Latenz des Normalbetriebs beeinflusst, ist O(1) und läuft im Vordergrund (ein Journal-Eintrag, ein Pointer-Swap). Alles, was proportional zur Datenmenge ist (Compaction, Index-Rewrite, Scan), läuft im Hintergrund und wird gedrosselt.

### 7.2 Scheduler

Ein einziger `ErasureWorker` in `engine/src/background_workers/erasure_worker.rs`, nach dem vorhandenen Muster (`CancellationToken`, geordnete Abschaltung, `MissedTickBehavior::Skip`, Obergrenze pro Tick).

- **Ereignisgesteuert.** Der Worker schläft auf einem `tokio::sync::Notify` plus einem groben Timer für fällige Wiederholungen (Vorschlag 5 min). Im Leerlauf null CPU, kein Polling der Gate-Tabelle, kein Scan.
- **Prioritätsklassen.** (1) *Interaktiv:* G1 (Annahme, Gate), nie gedrosselt. (2) *Fällig:* G3 nach Ablauf des Koaleszenzfensters. (3) *Wiederholung:* nach Fehlern mit Backoff.
- **Koaleszenz.** G3-Arbeit wird in Fenstern gebündelt (Vorschlag 5 min, konfigurierbar). Alle Aufträge im Fenster teilen **eine** Erase-Compaction über die Vereinigung ihrer Hüllen, **einen** WAL-Schnitt, **einen** HNSW-Rewrite pro Index. Die Kosten sind damit proportional zu Datenmenge und Fensteranzahl, nicht zur Auftragsanzahl.
- **SLA-Eskalation.** Bei 80 % der SLA-Zeit (Default 24 h) verdoppelt sich der Budgetanteil, bei 100 % Alarm. `Critical`-Druck wird nie ignoriert.

### 7.3 Budgets

| Ressource | Mechanismus | Quelle im Code |
| --- | --- | --- |
| **I/O** | Gemeinsamer Token-Bucket mit der regulären Compaction. Erase darf höchstens `erase_io_share` (Vorschlag 50 %) von `max_io_bytes_per_second` belegen, hat aber Vorrang beim Zuteilen. Beide zusammen überschreiten nie die konfigurierte Grenze. | `CompactionConfig.max_io_bytes_per_second` ✔ |
| **Speicher** | Streaming ohne Materialisierung, Seitengröße konstant (1 024 Schlüssel, Block-Puffer), `MAX_UNITS_PER_TICK`. Kein `Vec` über alle Schlüssel (B-07). | `delete_prefix` wird ersetzt |
| **CPU** | `spawn_blocking` für Index-Rewrites, Abbruchprüfung alle N Einheiten | Muster aus `orphan_workers.rs` ✔ |
| **Druck** | `pressure_rx.borrow()`: `Critical` → G3 pausiert (Schlaf mit Jitter), `Elevated` → halbes Budget. **G1 nie.** | `SystemPressureMonitor` ✔ |
| **Platz** | Journal und Vault reservieren `reserve_bytes` vorab, damit G1 auch bei vollem Datenträger gelingt. Vor G3 prüft ein Preflight den Temp-Bedarf (Summe der Kandidaten-SSTables). Fehlt Platz → Auftrag bleibt bei G1, Alarm `InsufficientSpace`. | neu |
| **Lesepfad** | Ohne aktive Gates ein einziger Relaxed-Load. | 5.4 |

### 7.4 Zielwerte (Vorschläge, in WP-3/WP-12 per Benchmark zu validieren)

| Größe | Ziel |
| --- | --- |
| G1-Latenz (Annahme bis Gate sichtbar) | p99 < 100 ms bei intaktem `fsync` |
| Mehrkosten pro Leseoperation ohne aktive Gates | ≤ 1 Relaxed-Load |
| Mehrkosten pro Leseoperation mit ≤ 1 000 Gate-Einträgen | p99 ≤ 200 ns |
| Leerlauf-CPU des Workers | 0 (kein Tick ohne offene Aufträge) |
| G3 | innerhalb der SLA (Default 24 h), bei Last gedrosselt, bei Leerlauf zügig |

Messort: `benchmarks/contextra-bench` ✔.

### 7.5 Konfiguration

```rust
pub struct ErasureConfig {
    pub coalesce_window: Duration,        // 5 min
    pub sla: Duration,                    // 24 h
    pub erase_io_share: f32,              // 0.5
    pub max_units_per_tick: usize,        // 1_024
    pub retry_backoff: (Duration, Duration), // 1 s .. 5 min, mit Jitter über Clock/Rng-Port
    pub quarantine_after_attempts: u32,   // 10
    pub checkpoint_policy: CheckpointTaintPolicy, // Drop
    pub max_pin_age: Duration,            // 7 d
    pub profile: ErasureProfile,          // Standard | Regulated
}
```

---

## 8. Fehler-, Crash- und Konsistenzmodell

### 8.1 Crash-Matrix (zugleich Testspezifikation)

| ID | Crash-Punkt | Zustand nach Neustart | Wiederaufnahme |
| --- | --- | --- | --- |
| CP-1 | nach `Requested`, vor `Planned` | `Requested` | neu planen (idempotent) |
| CP-2 | nach `Planned`, vor `GateIntent` | nichts sichtbar | Gating wiederholen |
| CP-3 | nach `GateIntent`, vor Veröffentlichung | Gate im Journal | Gate beim Boot vor dem Verkehr laden; `gated_seq` konservativ = letzte wiederhergestellte Sequenz |
| CP-4 | nach Veröffentlichung, vor `Gated` | wie CP-3 | `Gated` nachtragen |
| CP-5 | Layer mitten im Purge | `LayerProgress{cursor}` | ab Cursor fortsetzen, `*.erase.tmp` entfernen |
| CP-6 | nach Manifest-`Replace`, vor `StoreProgress` | Daten entfernt | Wiederholung findet nichts, No-op |
| CP-7 | WAL rotiert, Flush läuft nicht durch | zwei WAL-Segmente | Replay mit Filter, Flush wiederholen |
| CP-8 | Datei entfernt, Verzeichnis-`fsync` fehlt | Datei evtl. noch da | Pending-Delete-Liste arbeitet ab |
| CP-9 | in `Verifying` | `Purging` abgeschlossen | Scan wiederholen |
| CP-10 | nach Ledger-Append, vor `Sealed` | Ledger-Eintrag da | Idempotenz über Commitment, kein Doppeleintrag |
| CP-11 | Gate-Beendigung (I-9) | alter oder neuer Snapshot | beide sicher: Beendigung nur nach Witness 0 |
| CP-12 | Journal-Rollover | alte oder neue Datei | wie Manifest-Rollover |

### 8.2 Fehlerklassen

| Klasse | Beispiele | Reaktion |
| --- | --- | --- |
| `Retryable` | transientes `EIO`, Lock-Timeout, `ENOSPC` | Backoff mit Jitter, nach Grenzwert `Quarantined(layer)` mit Alarm, stündliche Wiederholung läuft weiter |
| `Permanent` | ungültiger Scope, `ScopeAmbiguous`, Mandantenverletzung | Auftrag beenden (nur vor G1) oder Quarantäne mit Alarm |
| `Poisoned` | `fsync`-Fehler | Datei fencen, `HealthState` degradiert, kein erneuter Versuch auf demselben Deskriptor; Datei neu öffnen und per Prüfsumme validieren. Auftrag bleibt `Gated` |
| `ScopeViolation` | Layer wollte Nachbardaten berühren | Abbruch, Sicherheitsalarm |

### 8.3 Konsistenz zwischen Schichten

Der Mehrschicht-Commit ist nicht atomar über alle Indizes. Erase hängt davon nicht ab: Sichtbarkeit ist durch den Gate atomar, die Schichten konvergieren idempotent. Die physische Konsistenz ist *eventual*, begrenzt durch die SLA und überwacht durch den Witness.

---

## 9. Teststrategie

| Ebene | Inhalt | Werkzeug im Repo |
| --- | --- | --- |
| Konformanz je Layer (Pflicht) | C1 Idempotenz · C2 Wiederaufnahme an jedem Cursor · C3 Scope-Isolation (Nachbar bit-identisch) · C4 Witness-Ehrlichkeit (absichtlich verbleibender Canary wird gemeldet) · C5 Fehlerinjektion ergibt `Err`, nie `Ok` · C6 Cancel-Safety · C7 Gate-Respekt | `contextra-testkit` (Conformance-Makros) ◐ |
| Crash-Matrix | CP-1 … CP-12 systematisch erzwingen | `fault_vfs`/`FaultInjector` ◐; `Vfs`-Pflicht macht das erst möglich |
| Nebenläufigkeit | Loom: R1, R4, R7. Stress: Writer + Reader + Flush + Compaction + Erase | `loom` ✔ vorhanden |
| Modelltest | Referenzmodell: nach `Completed` darf kein Schlüssel des Scopes in keiner Schicht gefunden werden; Nachbarn unverändert | `testkit` Referenzmodell ◐ |
| Canary-E2E | Eindeutige Bytefolge schreiben, löschen, alle Dateien und den Prozess-Dump byteweise durchsuchen. Reaktiviert `T-DEL-1` | `contextra/tests/deletion_e2e.rs` ◐ |
| Property-Tests | Erase ∘ Erase = Erase, beliebige Operationsfolgen vor und nach Neustart | `proptest` |
| CI-Gates (`rg`) | Kein `block_on` und kein `let _ =`/`unwrap_or_default()` in Erase-/Purge-Pfaden; kein `tracing::instrument` ohne `skip` auf Funktionen mit `id`/`key`-Parametern; kein `tokio::fs`/`std::fs` in `store/src/erasure/` | `tools/*.py` |
| Mutationstest | Garantie-Mutanten für Gate und Filter | `mutants-guarantees.toml` ✔ |
| Benchmarks | Gate-Overhead, Erase-Compaction-Durchsatz, Interferenz mit Schreiblast | `benchmarks/contextra-bench` ✔ |

---

## 10. Arbeitspakete

Jedes Paket endet mit grünem `cargo build`, `test`, `clippy -D warnings`, `deny check` und `tests/layering.rs`. Wegen `#![deny(dead_code)]` im Store wird neuer Code im selben Paket verdrahtet oder öffentlich exportiert.

| WP | Titel | Hauptsächlich betroffen | Abhängig von | Ergebnis |
| --- | --- | --- | --- | --- |
| 0 | Baseline, Stop-Ship, CI-Gates, offene Punkte klären | Workspace, `tools/` | – | Toolchain-Lauf, alle **[?]** geklärt, CI-Gates rot-grün getestet |
| 1 | Rückbau `legacy-erasure` | `crypto`, `contextra`, `types` | 0 | wie `IMPL-SPEC-ERASURE.md` WP-1 (weiterhin gültig) |
| 2 | Verträge v2 | `types`, `ports` | 1 | `ErasureId`, `EraseTarget`, `Guarantee`, `ReadGate` v2 (Byte-API, kein Default), `ErasureLayer`, `ErasureKernel`, `ContextraError::Erased`; Platzhalter (`ErasureLayer` alt, `Scanner<Armed>`, `CommitBarrier`, `PurgeBarrier`, `FileKeyring`) entfernt |
| 3 | **Kernel-Fundament: Journal, Gate, Filter-Haken** | `store` | 2 | G1 durchgängig für Store-Schlüssel: Journal, Boot-Reihenfolge, Gate in Lese-, Scan-, Commit-, Replay-, Flush-, Merge-Pfad, Lock-Ränge 150/960, `revoke_covering` |
| 4 | **Physisches Entfernen im Store** | `store` | 3 | Erase-Compaction als Merge-Filter, WAL-Schnitt, Pending-Delete, `delete_prefix` streamend, Tombstone-Regel, Witness-Verdrahtung; `LsmPurger` ersetzt, drei `#[ignore]`-Tests aktiv |
| 5 | **Treiber, Worker, API** | `engine`, `contextra`, `mcp` | 3, 4 | Saga, Scope-Auflösung, Scheduler mit Budgets und Druck, `erase`, `erase_status`, `erase_dry_run`; `drop_collection` → `erase(Collection)`; **erste Ende-zu-Ende-Auslieferung (LSM, Dokumentsicht-Filter, Canary-Test grün)** |
| 6 | Textindex | `text` | 4, 5 | Async-Purge, `CollapseVersions` für `i:`, Kandidatenfilter |
| 7 | Vektorindex | `vector` | 4, 5 | Maskierung (G1), Segment-Rewrite (G3), `Vfs`-I/O, Recall-Guard |
| 8 | Graph und Provenienz | `graph`, `engine` | 4, 5 | Pflicht-`derived_from`, `NeedsRecompute`, Hyperedges |
| 9 | Checkpoints, Snapshots, Caches, Commit-Residuen | `checkpoint`, `mvcc`, `engine` | 5 | Restore-Gate, Taint-Politik, Pin-Ablauf, Zeroisierung, Telemetrie-Gate |
| 10 | Witness-Aggregation und Ledger | `store`, `engine` | 6–9 | `LedgerSink`, Anker, `verify_ledger`, Stufenermittlung |
| 11 | Kryptographische Löschung (Stufe 2, opt-in) | `crypto`, `store`, `engine` | 5 | SaltVault, Versiegelung, Dual-Read, seal-on-rewrite, Job-Schlüssel |
| 12 | Testmatrix und Betrieb | alle | fortlaufend | Crash, Loom, Konformanz, E2E, Runbook, Metriken, Alarme |

**Kritischer Pfad:** 0 → 1 → 2 → 3 → 4 → 5 → (6, 7, 8 parallel) → 9 → 10 → 12. WP-11 ist parallel zu 6–10 möglich und hängt nur an WP-5.

**Wert früh.** Nach WP-5 ist Löschen für Dokumente, Collections und Mandanten im Store und für Textpostings (die im LSM liegen) vollständig und nachweisbar. Vektor, Graph und Checkpoints folgen und senken bis dahin die ausgewiesene Garantie ehrlich ab.

---

## 11. Abweichungen vom Repo-Entwurf (`docs/SPEC-ERASURE.md`, `IMPL-SPEC-ERASURE.md`)

| Thema | Repo-Entwurf | Dieses Konzept | Grund |
| --- | --- | --- | --- |
| Crate | Neues `contextra-erasure` | Kernel im Store, Treiber in der Engine | Vorgabe; zusätzlich zwingt das Layering zur Aufteilung |
| Journal | `durable-fs::AppendOnlyJournal` | Eigenes Journal auf `Vfs` | `durable-fs` hat keinen Abhängigen ✔; Store darf es laut `capabilities.toml` nicht einbinden ✔; Repo-Regel „I/O nur über `Vfs`“ |
| Scrub | `scrub_if_safe` als Teil von G3 | optional (ADR-B) | Kein Nachweis auf SSD/CoW; Kante store → durable-fs nicht erlaubt |
| Primärgarantie | G2 (Crypto-Erase) zuerst | G1 + G3 zuerst, G2 Stufe 2 | Nichts ist heute versiegelt (B-14) |
| Gate-Schlüssel | `KeyBuf`/`[u8;78]` plus `RoaringTreemap` | Byte-Präfix-Intervalle plus `BTreeSet<DocId>` | Store-Schlüssel sind variabel lange Strings ✔; Gate-Mengen sind klein, keine neue Abhängigkeit nötig |
| Barrier | Neue `ErasureBarriers` | Gate-Veröffentlichung unter dem vorhandenen `commit_mutex` | Weniger neue Primitive, eine Rangordnung |
| `ArcSwap` | „neue Abhängigkeit“ | Bereits im Workspace (`contextra-graph`) ✔ | Start trotzdem mit `RwLock<Arc<_>>` |
| WAL | Segmentrotation als offener Punkt | Rotation existiert ✔, WAL-Schnitt = Rotation + Flush mit Filter | `lsm/ops/compaction.rs` |
| Flush | nicht erwähnt | eigener Durchsetzungspunkt (B-05) | `key_map`-Schleife |
| `I/O`-Konformität | nicht erwähnt | `tokio::fs`/`std::fs` in Recovery und HNSW auf `Vfs` umstellen (B-09, B-12) | Fehlerinjektion setzt `Vfs` voraus |
| Ressourcen | nur Token-Bucket erwähnt | Abschnitt 7: Ereignissteuerung, Koaleszenz, Druckkopplung, Leerlauf null | Vorgabe |

---

## 12. Offene Punkte und Entscheidungen

### 12.1 In WP-0 zu klären **[?]**

1. Serialisiert `commit_mutex` im Group-Commit-Pfad alle Commits (Voraussetzung von 6.1)?
2. Genauer Einhängepunkt des Filters im WAL-Replay (`lsm/recovery.rs`), und Commit-Leader-Hook (`lsm/commit.rs`, `group_commit.rs`).
3. Werden alte Manifest-Generationen nach dem Rollover aufbewahrt?
4. Persistenzpfad des Graphen (Store oder eigene Dateien)?
5. Lebensdauer und Bereinigung der Intent-Dateien (`recovery.rs` ~716–725, 878).
6. Gehen **alle** Engine-Pfade durch `TenantScopedStorage`, sodass der Gate die vollständigen Rohschlüssel sieht?
7. Nutzung von `CheckpointMeta.metadata` im Produktivcode.
8. Aufrufer von `Purger`/`ScanTarget` und Erreichbarkeit von `ScanSession::finish` über `LayerScanner::scan`.
9. `fsync`-/`rename`-Reihenfolge in `HnswIndex::save_internal`.
10. Ergebnis des vollständigen Workspace-Laufs (`build`, `test`, `clippy`, `deny`), inklusive der drei `#[ignore]`-Tests.

### 12.2 Zu entscheiden (ADR)

| ADR | Frage | Empfehlung |
| --- | --- | --- |
| A | Revision von D-06 („Löschung = Tombstone“) | Ja, für Erase. `delete` bleibt Retire |
| B | Scrubbing als Teil von G3 | Nein im Default. Optional später (Kante store → durable-fs nur dafür) |
| C | Gate-Zeiger: `RwLock<Arc<_>>` oder `arc_swap` | Start `RwLock`, Benchmark in WP-3 |
| D | Standard-SLA und Koaleszenzfenster | 24 h und 5 min, nach Last-Messung anpassen |
| E | Subjekt-basiertes Erase (Subjekt-Index `__subj:` im Schreibpfad) | Erst nach WP-5 und nur im Profil *regulated* |
| F | `delete` löst automatisch `erase(Document)` aus | Nur im Profil *regulated*, Default aus |
| G | Granularität der Schlüsselableitung (Stufe 2) | `PerDocument` als Default, `PerSubject` im Profil *regulated* |

### 12.3 Risiken

| Risiko | Wirkung | Gegenmaßnahme |
| --- | --- | --- |
| Hülle der Erase-Compaction ist groß (viele Level) | Hohe I/O-Last | Koaleszenz, Budget, Fortschritt im Journal, Bloom-Vorfilter |
| Gate-Tabelle wächst | Lesepfad wird langsamer | Gate-Beendigung nach Witness 0 (I-9), Ausnahmen kompakt als Intervalle |
| Vektor-Rewrite senkt Recall | Suchqualität | Recall-Guard, Teil-Rebuild |
| Altbestand ohne Provenienz | Garantie nicht belegbar | Ausweis als Ausnahme, Migration in WP-8 |
| Ressourcenkonkurrenz mit Compaction | Lese-/Schreiblatenz | Gemeinsamer Token-Bucket, Druckkopplung |
| `#![deny(dead_code)]` im Store | Teilweise Auslieferung scheitert | Pakete so schneiden, dass jedes verdrahtet ist |

---

## Anhang A – Was in dieser Analyse geprüft wurde

Geklont wurde `https://github.com/tfufuz1/contextra`, `main`, Commit `3b6a19953`. Gelesen wurden: `Cargo.toml` und `capabilities.toml` (Abhängigkeitskanten), alle Crate-`Cargo.toml`, `tests/layering.rs`, `clippy.toml`, `store/src/{lib.rs, lsm/lock_order.rs, lsm/ops/compaction.rs (Flush), lsm/ops/write.rs (delete_prefix), compaction/engine.rs, compaction/retention.rs, manifest/entry.rs, fsync.rs, system_pressure.rs, tenant_codec.rs, util.rs, wal/*, lsm/recovery.rs (Auszüge)}`, `ports/src/{gate.rs, vfs.rs, scanner/*}`, `mvcc/src/snapshot.rs`, `engine/src/collection/{crud/delete.rs, maintenance.rs, mod.rs}`, `engine/src/background_workers/*`, `text/src/inverted/purge.rs`, `vector/src/hnsw/purge.rs`, `graph/src/purger.rs`, `durable-fs/src/*`, `crypto/{Cargo.toml, src/dek.rs}`, `types/src/{tombstone.rs, types/domain/ids.rs}` sowie die beiden Repo-Entwürfe.

Nicht geprüft (Kennzeichnung ◐ oder **[?]**): Inhalt von `contextra-testkit`, Checkpoint-Restore-Pfad, Graph-Persistenz, `record_keys_with_old_values`-Lebensdauer, Commit-Leader-Hook, Laufzeitverhalten jeder Art.

## Anhang B – Dateikarte der Umsetzung

```text
contextra-types/src/erasure.rs                 NEU   Typen (ErasureId, EraseTarget, Guarantee, ErasureState, ...)
contextra-ports/src/erasure.rs                 ERSETZT  ErasureKernel, ErasureLayer, ScopeResolver, PurgeCx
contextra-ports/src/gate.rs                    GEÄNDERT ReadGate v2 (Byte-API, keine fail-open-Defaults)
contextra-ports/src/{barrier.rs,keys.rs}       ENTFALLEN (Platzhalter ohne Implementierung)
contextra-store/src/erasure/mod.rs             NEU   Fassade ErasureKernel (einzige pub-Fläche)
contextra-store/src/erasure/journal.rs         NEU
contextra-store/src/erasure/gate.rs            NEU
contextra-store/src/erasure/filter.rs          NEU   Haken für Flush/Merge/Replay
contextra-store/src/erasure/purge_lsm.rs       NEU   ersetzt store/src/scanner/purge.rs
contextra-store/src/erasure/pending.rs         NEU
contextra-store/src/erasure/ledger.rs          NEU   (WP-10)
contextra-store/src/erasure/vault_store.rs     NEU   (WP-11)
contextra-store/src/lsm/lock_order.rs          GEÄNDERT  Ränge 150, 960
contextra-store/src/lsm/ops/compaction.rs      GEÄNDERT  Flush-Filter
contextra-store/src/compaction/engine.rs       GEÄNDERT  Merge-Filter, Hülle, Pending-Delete
contextra-store/src/lsm/recovery.rs            GEÄNDERT  Gate vor Replay, Vfs statt tokio::fs
contextra-crypto/src/vault.rs                  NEU   (WP-11), legacy-erasure entfällt (WP-1)
contextra-engine/src/erasure/                  NEU   Treiber, Resolver, Layer-Registrierung
contextra-engine/src/background_workers/erasure_worker.rs  NEU
contextra-{text,vector,graph,checkpoint}       GEÄNDERT  Layer-Adapter auf ErasureLayer
contextra/src, contextra-mcp/src               GEÄNDERT  erase, erase_status, ctx_erase
```
