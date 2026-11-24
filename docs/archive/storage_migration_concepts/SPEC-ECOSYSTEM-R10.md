# Contextra-Ökosystem: Integrationsplan zu `contextra-storage` (R10, Entwurf)

| | |
|---|---|
| Status | Entwurf zur Freigabe. Ergänzt `crates/contextra-storage/SPEC.md` (R9.1). Ändert keine Kernnorm des Speicherkerns; widersprüchliche Punkte sind unten als Entscheidungen ausgewiesen. |
| Gegenstand | Einordnung von `contextra-storage` in das Gesamtsystem: Ist-Zustand aller 40 Crates, Widersprüche, Zielarchitektur, Verträge zwischen Kern und Domänen-Crates, Migrationsreihenfolge und Abnahmekriterien. |
| Evidenzlage | **Rein statische Lesung** des Repositories `tfufuz1/contextra` (Stand Klon am 2026-11-24). Nicht gebaut, nicht getestet, nicht ausgeführt. Jede Aussage über Laufzeitverhalten ist eine Hypothese und mit `[H]` markiert. Befunde mit `[S]` sind statisch am Quelltext belegt, aber noch nicht durch einen Test reproduziert. |
| Priorität | `contextra-storage` hat Vorrang. Domänen-Crates werden an den Kernverträgen ausgerichtet, nicht umgekehrt. Wo der Kern eine Annahme macht, die das Ökosystem nicht erfüllt, wird der Kern nur geändert, wenn die Änderung Löschbarkeit, Atomarität oder Durability betrifft. |
| Sprache | Prosa deutsch, Bezeichner englisch. |

---

## 0. Kurzfassung

1. **Der Kern ist für das Ökosystem noch nicht angeschlossen.** `contextra-storage` hat heute keine Abhängigkeit von irgendeinem Domänen-Crate, und umgekehrt hängt kein Altcrate von `contextra-storage` ab (Cargo-Graph geprüft). Die Fassade `Storage` hat nur `open()` und `catalog()`; `apply`, `view`, `eraser` fehlen [S]. Die Module `engine/*`, `erase/*`, `maint/*`, `checkpoint/*` sind leer [S].
2. **Das Ökosystem stützt sich auf `contextra-store` (LSM) und ein paar Nebenlinien.** Direkte Nutzer: `contextra-engine`, `contextra-db`, `contextra-agent`, `contextra-cognition`, `contextra-graph`, `contextra-checkpoint`, `contextra` (Umbrella), `contextra-py` (Cargo-Dep, Quelltext nutzt nur `contextra-db`). Index-Crates (`vector`, `text`) persistieren teils über eigene Dateien, teils über den Port `StorageEngine`.
3. **Die Löschkette existiert nur als Teilimplementierung und hat mindestens einen potenziell gravierenden Defekt** (siehe §2, P0-1). Ein Crypto-Shred-Mechanismus existiert in zwei Crates, ist aber im Produktionspfad nicht verdrahtet.
4. **Es gibt vier Typ-Doppelungen** mit gleichem Namen und abweichender Semantik (`Stamp`, `Key32`, `DataClass`, `Clock`/`KeyProvider`), dazu zwei Fassaden (`contextra-core` als „Strangler Facade“ und die geplante `contextra-storage`-Fassade).
5. **Zielbild:** `contextra-storage` wird zur einzigen Stelle, an der Persistenz, Versionen, Löschung und Löschnachweise entschieden werden. Domänen-Crates werden zu Besitzern von Segmenten (`SegmentOwner`) oder zu Verbrauchern des Change-Feeds und sprechen nur über die Verträge aus §12 der Kern-Spec. Der Löschorchestrator in `contextra-engine` wird ersetzt durch einen dünnen Aufrufer von `Eraser` plus Owner-Attesten.
6. **Migrationsprinzip:** Erst Kern-Verträge einfrieren (G-F), dann Domänen-Adapter, dann Abschalten des LSM-Pfads hinter G-CUTOVER. Kein Dual-Write.

---

## 1. Ist-Zustand des Ökosystems

### 1.1 Crate-Übersicht (Zeilen = Quelltext ohne Tests, grob)

| Crate | ca. LOC | Rolle | Persistenz | Kopplung an Storage |
|---|---:|---|---|---|
| `contextra-store` | 51 700 | LSM-KV-Kern (aktuell produktiv) | eigene SSTables/WAL | Ist-Kern, wird ersetzt |
| `contextra-storage` (+ `ports`, `fs`, `crypto-ref`, `keys-file`, `keys-cohort`, `storage-bridge`) | 10 800 (Kern) + Platzhalter | neuer Kern | nur Gerüst | keine Nutzer |
| `contextra-engine` | 28 500 | Collection-/Contextra-Fassade, Erasure-Orchestrator | über `LsmStorage` | direkt |
| `contextra-graph` | 28 600 | CSR-Graph, Hyperkanten, Cascade | Port `StorageEngine` + RAM | Port |
| `contextra-ingest` | 27 000 | Dokument→Chunk-Pipeline | keine | keine |
| `contextra-vector` | 25 500 | HNSW (RAM + mmap) und DiskANN | eigene Dateien + `*.tombstone.wal` | keine |
| `contextra-text` | 13 800 | Inverted Index | Port `StorageEngine` | Port |
| `contextra-crypto` | 17 600 | KV-Shredding, Deletion-Proofs v1/2/3/5, AuditChain | RAM + Widerrufslog | keine (Kernabhängigkeit nur Konsumenten) |
| `contextra-db` | 14 200 | Re-Export-Fassade | keine | über `contextra-store` |
| `contextra-agent` | 10 400 | Agent-Engine, DLQ, Scratchpad, Budget | Port + RAM | Collection/Port |
| `contextra-cognition` | 9 100 | Konsolidierung, Maintenance, Compaction-Session | Collection + Intents | direkt |
| `contextra-checkpoint` | 8 800 | Checkpoints, Restore, Orphans | Storage + `std::fs` | direkt |
| `contextra-kvcache` | 7 800 | KV-Prefix-Cache (RAM, Spill über Bridge) | RAM + `KvBridgeStorage` | Port (ohne `delete`) |
| `contextra-adapt` | 7 400 | Bandit, PID, Lyapunov (Regelung) | keine | keine |
| `contextra-privacy` | 7 200 | PII-Erkennung, Egress-Gateway | keine | keine |
| `contextra-mcp` | 8 600 | MCP-Server | über `contextra` | indirekt |
| `contextra-router` | 8 800 | Modellrouter | Port (Prod), `contextra-db` nur dev | Port |
| `contextra-mvcc` | 5 800 | Snapshot-Registry, Seq-Log | RAM | parallel zu Kern-MVCC |
| `contextra-ports` | 5 500 | alte Port-Traits (`StorageEngine`, `VectorIndex`, `KvPrefixStore` …) | – | Port-Schicht Altbestand |
| `contextra-types` | 8 400 | Domänen-IDs (`DocId`, `TenantId`, `TxId`, `EntityId`) | – | gemeinsam |
| `contextra-durable-fs` | 3 500 | Fs-Primitive, Journale, FaultInjection | std::fs | **verwaist** |
| `contextra-storage-fs` | Platzhalter | Fs-Adapter Kern | – | leer |
| `contextra-storage-bridge` | Platzhalter | async Fassade | – | leer |
| `contextra-keys-file` / `-cohort` | Platzhalter | KeyProvider | – | leer |
| `contextra-audit-export` | 1 400 | Verarbeitungsverzeichnis, BSI-Mapping | – | konsumiert `DeletionProofSummary` (frei gefüllte Strings) |
| `contextra-avv-generator` | 1 100 | AVV nach Art. 28 | – | Shim, deprecated |
| `contextra-core` | 370 | „Strangler Facade“: Re-Export Types/Ports/MVCC/Wire | – | parallel zur neuen Fassade |
| `contextra-simd`, `-sys`, `-wire`, `-rank`, `-sandbox`, `-testkit`, `-license`, `-py`, `-infer-*`, `-audit-export` … | – | Hilfs-, Hardware-, Bindings- und Laufzeit-Crates | teils keine | teils keine |

Hinweis: Die Liste in der Workspace-`Cargo.toml` enthält die neuen Storage-Crates in `default-members` und `members`. Die Pfade existieren; ein Build-Check ist noch nicht gelaufen [S].

### 1.2 Die Datenmodelle, wie sie heute in den Speicher geschrieben werden

| Logischer Datentyp | Ist-Schlüssel (Auswahl) | Erzeugt von | Anmerkung |
|---|---|---|---|
| Dokument | `__col:{name}:\0` (Collection), Tenant-Präfix `t:{tenant}:{col}:` | `contextra-engine` | Dokument-Id vom Aufrufer [S] |
| DocId-Mapping | `__docid:` | `contextra-engine` | |
| Text-Postings | `pl:`, `dl:`, `fw:`, `plb:`, `tbs:` | `contextra-text` | Tombstone-Bit 63 der Sequenz [S] |
| Graph-Kanten | `graph:…`, `__graph:cascade_queue:` | `contextra-graph` | Cascade-Queue in Hex, Loader scannt sie nicht [S] |
| Graph-Entität gelöscht | `graph:entity:deleted:{id}` | `contextra-graph` | ohne `__`-Präfix, Kollisionsrisiko [S] |
| Hyperkante | `{:016x}`-Schlüssel (Persist) vs. dezimal im Test | `contextra-graph`, `contextra-engine` | Inkonsistent [S] |
| Session/Task-Schritt | `task:{id}:step:{n}`, `task:{id}:final` | `contextra-agent` | |
| Audit | `audit:{task}:step:{n}` | `contextra-agent` | keine Hashverkettung trotz README [S] |
| DLQ | `dlq:` | `contextra-agent` | Input im Klartext [S] |
| Checkpoint-Meta | `agent:checkpoint:…` | `contextra-checkpoint` | JSON-Orphan-Register per `std::fs` [S] |
| Maintenance-Intent | `__maintenance_intent:tick` | `contextra-cognition` | |
| KV-Prefix-Segmente | RAM, optional Spill via `KvBridgeStorage` | `contextra-kvcache` | Radix-Index wird bei Eviction nie bereinigt [S] |
| Vektoren | eigene Dateien (`*.tombstone.wal`, mmap) bzw. RAM | `contextra-vector` | DiskANN ignoriert `tx` bei `delete` [S] |

### 1.3 Bestehende Schutzmechanismen (Ist)

- **Logisches Löschen:** `Collection::delete`, TTL-Reaper, Konsolidierung löschen logisch (Tombstone/MVCC-Bit). Derivate (Summaries) bleiben beim Löschen der Quelle bestehen [S].
- **Physischer Purge:** nur `Contextra::drop_collection` → `erase_collection` (`contextra-engine/src/contextra_impl/erasure.rs:334`) und `erase_document` (ohne Aufrufer außerhalb von Tests) [S].
- **Crypto-Shred:** `contextra-crypto/src/kv_shredding.rs` (`revoke_group`, `revoke_record`) und `contextra-store/src/kv/segment.rs` (`KvSegmentManager`). Im Erasure-Pfad nicht aufgerufen; im LSM-Schreibpfad nicht verdrahtet; `KvDeleteMode::CryptoShred` ist Default, aber nur Konfiguration [S].
- **Löschbeweis:** `DeletionProof` in `contextra-crypto/src/deletion_proof/` (Signaturversionen 1, 2, 3, 5) und `DeletionProofSummary` in `contextra-audit-export` (frei befüllbare Strings) [S]. Ein Ereignis-Receipt im Sinne der Kern-Spec existiert nicht.
- **Checkpoints als Pin:** `create_checkpoint` pinnt die Snapshot-Registry; der Pin verhindert Purge bis `drop_checkpoint` [S].

---

## 2. Befunde und Widersprüche

Die Befunde stammen aus vier unabhängigen statischen Analysen (Store/Löschkette; Index-Crates; Konsumenten; Crypto/Ports/Bridge). Sie wurden untereinander und mit der Kern-Spec abgeglichen. Mit `[H]` markierte Punkte sind Hypothesen aus der Lesung und brauchen einen reproduzierenden Test, bevor sie als Fakt gelten.

### 2.1 Priorität P0 (Datenverlust, Löschversagen, Fehlbeweis)

| ID | Befund | Beleg | Konsequenz |
|---|---|---|---|
| P0-1 | `LsmPurger::purge` ignoriert `_target` und filtert nach dem gesamten Tenant-Präfix `t:{tenant}:`. `erase_document` übergibt zwar ein dokumentspezifisches Target, der Purger wertet es aber nicht aus. | `contextra-store/src/scanner/purge.rs:32,41`; `contextra-engine/.../erasure.rs:155,186,236` [S] | **[H]** Löschung eines Dokuments könnte alle Records des Mandanten entfernen (Überlöschung). Muss sofort durch einen Test „fremde Dokumente bleiben erhalten“ belegt oder widerlegt werden. Der bestehende E2E-Test legt nur ein Dokument an und sieht das nicht. |
| P0-2 | Das Target `t:{tenant}:{col}:doc:{id}` wird vom Text-Scanner nicht als Dokument erkannt; HNSW-Purger findet dadurch keinen Treffer, HNSW-Scanner zählt nur Tombstones. Der „unabhängige Verifier“ (Kommentar `erasure.rs:4`) nutzt dieselbe Target-Logik wie der Purger. | `contextra-text/src/inverted/scanner.rs:23-52`; `contextra-vector/src/hnsw/purge.rs:33-60` [S] | Der Erasure-Beweis ist nicht unabhängig, und die Verifikation kann Reste übersehen. |
| P0-3 | Der LSM-Purger schreibt Retained-Daten **unverschlüsselt** (`SstableBuilder::create` ohne KeyManager), während Compaction und Recovery verschlüsselt schreiben. Bei verschlüsseltem Store landen Klartextreste auf der Platte. | `purge.rs:79`; `compaction/engine.rs:509`; `lsm/recovery.rs:779` [S] | Verstoß gegen Verschlüsselungsversprechen; fällt nur bei Encrypted-Stores. |
| P0-4 | Der Purger schluckt Fehler (zehn `let _ =` bzw. `unwrap_or_default()`) und meldet trotzdem `Ok(PurgeStats)`. Ein nicht lesbarer SST gilt als „ohne Treffer“. | `purge.rs:81,128,200-221` [S] | Fehlermeldung „gelöscht“ ohne Löschung möglich. |
| P0-5 | Der Restore-Pfad von `contextra-checkpoint` verletzt die dokumentierte Barriere: Mock-Tests liefern konstant `last_tx_id = 0`, `Checkpoint::restore` umgeht die Barriere über `rollback_to_tx`. | `checkpoint/store.rs:675,685,879-884`; `tests/time_travel_correctness.rs:174` [S] | „Rückkehr zu jedem früheren Stand“ (README) wird im Test nicht geprüft und im Agenten-Pfad nicht durchgesetzt. |
| P0-6 | Erasure hat keinen Gate-Schritt und keinen Schlüssel-Vernichtungsschritt. Crypto-Shred ist nicht verdrahtet. Derivate (Summaries), DLQ, Audit, Checkpoint-Meta, Block-Cache, Hardlink-Klone, Logs liegen außerhalb der Löschkette. | `erasure.rs:126,334`; `kv/segment.rs:147`; Analysebericht §2 [S] | Die Aussage „kryptografisch gelöscht“ ist für Personendaten heute nicht haltbar. |

### 2.2 Priorität P1 (Konsistenz, Restart-Sicherheit, Namenskonflikte)

| ID | Befund | Beleg |
|---|---|---|
| P1-1 | Hyperkanten werden nicht über den Loader wiederhergestellt; Cascade-Queue-Überlauf wird nach Crash nicht gelesen; Löschfehler im Queue-Pop werden ignoriert. | `graph/src/csr/graph_persist.rs`, `cascade.rs:326-334`, `hyperedge.rs:91` [S] |
| P1-2 | Hex- vs. Dezimal-Schlüssel für Hyperkanten. Restart-Test findet einen hex geschriebenen Eintrag nicht (stille Ausnahme über `if let Some`). | `hyperedge.rs ~130`, `hyperedge_cascade_crash_recovery.rs ~143` [S] |
| P1-3 | `add_entity_impl` löscht den Entity-Tombstone nicht. Gleiche ID nach Löschung verschwindet beim nächsten Start. `EntityId::from_doc_id` leitet Entity-IDs aus Dokument-IDs ab. | `csr/graph_index/mutations.rs:81-100`, `ids.rs ~300` [S] |
| P1-4 | `EntityId::from(&str)`: Dezimalstring wird als Zahl geparst, sonst BLAKE3-64. `"42"` und `DocId(42)` kollidieren; Entity- und Dokument-Namensraum sind nicht getrennt. | `types/domain/ids.rs:370-378` [S] |
| P1-5 | `DocId::from_key` ist BLAKE3, auf 64 Bit gekürzt (Geburtstagsschranke ~2^32). Inhaltsabhängige IDs sind nicht global eindeutig. Ein 128-Bit-Feature existiert, ist aber optional. | `ids.rs:157-208` [S] |
| P1-6 | `TxId` 0 und 1 kollidieren über `segment_id.max(1)` im Spill; `TxId` wird als `HyperEdgeId` wiederverwendet. Segment-IDs stammen vom Aufrufer. | `kvcache/service.rs:210,474`; `engine/relate.rs:141` [S] |
| P1-7 | DiskANN-`delete` schreibt sofort ins Tombstone-WAL und ignoriert `tx`; `commit` ist No-op. HNSW staged dagegen in `tx_buffer`. Zwei Semantiken für denselben Port. | `vector/diskann/vector_index_impl.rs:63-100` vs. `hnsw/vector_index_impl.rs:229-244` [S] |
| P1-8 | Text-Purge verschluckt Einzel-Delete-Fehler (Closure liefert immer `Ok`); `block_on_async` nutzt `futures::executor::block_on` in synchronen Methoden und kann bei tokio-gebundenem Storage blockieren. | `text/inverted/purge.rs:19-25,335-336` [S] |
| P1-9 | KV-Cache: Radix-Index wird bei Eviction und Entfernen nicht bereinigt; `delete_tenant` schreibt Spill-Löschung als `put(…, &[])` (Port hat kein `delete`); Fehler mit `.ok()?` verschluckt. Content-Index ist `blake3(tokens)` pro Tenant. | `kvcache/store.rs:38,248,457-500`, `service.rs:449-480,475` [S] |
| P1-10 | Chunk-IDs sind `blake3(canonical_hash | …)` über den gesamten kanonischen Text; doc_id fließt nicht ein. Verstößt gegen `INV-PRIV-1` (kein persistierter Klartext-Hash), weil Gleichheit zweier Dokumente über Mandanten/Löschdomänen hinweg sichtbar wird. | `ingest/chunking/meta.rs:22-32`, `canonical.rs:91-95` [S] |
| P1-11 | Re-Ingest: `insert` prüft keine Existenz; erneutes Einfügen ruft `index.insert` erneut auf. Ob das HNSW-Duplikate erzeugt, ist nicht geprüft [H]. Alte Chunks bleiben stehen. | `collection/crud/insert.rs:267-355` [S] |
| P1-12 | Zwei Konzepte namens „Tombstone“: MVCC-Marker (`TOMBSTONE_BIT`) und Löschbeleg (`tomb_id` im Spec). | Store/Types vs. Spec §4.1 [S] |
| P1-13 | `DataClass` doppelt: `contextra-crypto/audit_chain.rs:27-33` (Public/Internal/Confidential) vs. `storage-ports/ids.rs:57-61` (NonPersonal/Personal). | [S] |
| P1-14 | `Stamp` doppelt definiert (Kern `ids.rs:8`, Altbestand `ports/persistence.rs:410`), `Clock` zweimal in `contextra-ports` (`persistence.rs:162` vs. `clock.rs:8`), `Key32` als Struct (neu) vs. Alias `[u8;32]` (alt). Cipher-Traits mit gleichem Namen, verschiedene Signaturen (`seal` infallibel vs. `Result`). `KeyProvider` alt `kek()` ohne Argument vs. neu `kek(KekId)`. | Analysebericht §2 [S] |
| P1-15 | Zwei AEAD-/MAC-Implementierungen im Altbestand (AES-GCM-SIV in `crypto.rs`, XChaCha20 in `persistence/cipher.rs`); neue `contextra-crypto-ref` nutzt XChaCha20 mit anderem HKDF-Salt. Abgeleitete Schlüssel sind nicht kompatibel. `Blake3Mac` existiert doppelt. | Analysebericht §1 [S] |
| P1-16 | `contextra-durable-fs` ist verwaist; `contextra-storage-fs/realfs.rs` ist Platzhalter; `03-env.md` widerspricht sich (Wiederverwendung von durable-fs vs. `rustix`). Fault-Injection doppelt (`storage/env/faultfs.rs` vs. `durable-fs/fault_injector.rs`). | [S] |
| P1-17 | `contextra-core` (Strangler Facade) und die neue `contextra-storage`-Fassade haben denselben Zweck. | `contextra-core/src/lib.rs:23-94` [S] |
| P1-18 | `contextra-kvcache` kennt `KvBridgeStorage` ohne `delete`. Die Doku nennt `contextra-store::LsmStorage` als Implementierung, kvcache hat aber keine Store-Abhängigkeit. | `ports/kv_bridge_port.rs:6-24` [S] |
| P1-19 | Das Audit im Agent-Pfad ist ohne Hashverkettung, obwohl die README sie verspricht. `contextra_crypto::AuditChain` wird von den Konsumenten nicht genutzt. | `agent/audit.rs:108`; README:163 [S] |
| P1-20 | `is_already_committed` in der DLQ scannt den gesamten Storage (`scan_prefix(b"")`), nutzt Substring-Match (`step:1` trifft `step:12`) und hartcodiert den Collection-Namen `"agent"`. | `agent/dlq.rs:192-226` [S] |
| P1-21 | Fail-open: `OrchestratorEngine::new` (deprecated) ersetzt bei Fehler die Checkpoint-Registry durch einen No-Op. | `agent/engine.rs:82-135` [S] |
| P1-22 | Audit-Export und AVV-Generator konsumieren keine Erasure-Receipts; `DeletionProofSummary` nimmt frei gefüllte Strings auf, Testdaten erzeugen Beispielbeweise („del-proof-201“). `bsi_mapping.rs:41` verweist auf eine nicht existierende Datei und nennt „v4“, der Code kennt v1/2/3/5. | [S] |
| P1-23 | Graph/Text/Vector/Rank kennen keine Mandanten; Mandantenbezug nur in kvcache und Engine. Mandanten-Filter im Ingest/Engine liegt daher im Präfix. | [S] |

### 2.3 Positive Befunde (übernehmen)

- `contextra-text` hat den vollständigsten Löschpfad über Port-Schlüssel und ist gut testbar.
- `contextra-checkpoint` arbeitet ohne Datenkopie (Pin + Metadaten); Hardlink-Klone sind ein brauchbarer Ansatz, brauchen aber Filterung.
- `contextra-ingest` ist storage-frei und damit leicht umzuhängen.
- `contextra-adapt` ist rein numerisch und storage-frei; sie kann unverändert bleiben.
- Die Negativtests (Crash/Fault) im Kern sind der richtige Maßstab; die Altcrates haben weniger davon.
- `contextra-privacy` und Egress-Gateway: Die Entscheidungslogik ist sauber getrennt; es fehlt nur die Verbindung zur Löschung (`pii_vault_forces_crypto_shred` ist nur eine Entscheidungsfunktion).

---

## 3. Zielarchitektur

### 3.1 Schichten

```
Oberfläche:   contextra (Umbrella) · contextra-db · contextra-py · contextra-mcp · contextra-router-API · contextra-wire
                 │  (alle sprechen die neue Domänen-API; kein Zugriff auf Store/Kern direkt)
Domäne:       contextra-engine (Collections, Dokumente, Relationen, Transaktionen auf Domänenebene)
              contextra-ingest · contextra-cognition · contextra-agent · contextra-checkpoint(-Orchestrierung) · contextra-privacy (Entscheidung)
Owner:        contextra-vector(HNSW/DiskANN als Rebuildable-Segment) · contextra-text · contextra-graph · contextra-kvcache · contextra-rank(ohne Storage)
                 │  Verträge: SegmentOwner, ExtensionBlock, Consumer/Change-Feed, Watermark, filter_visible
Löschung/Bew.:  contextra-audit-export · contextra-avv-generator · Receipt→DeletionProof-Bridge (contextra-storage-bridge)
Kern:         contextra-storage (apply/view/eraser/checkpoint/ledger/anchor)  ←  contextra-storage-ports
              contextra-storage-fs · contextra-crypto-ref (dev) / Produktions-KeyProvider (keys-file, keys-cohort, HSM/KMS-Adapter)
```

Grundregeln:

- **R-E1** Nur der Kern persistiert Nutzdaten, die einer Löschung unterliegen. Owner-Segmente werden über `SegmentOwner` registriert; kein Domänen-Crate öffnet eigene Dateien mit personenbezogenen Daten.
- **R-E2** Domänen-Crates kennen `ClusterId`, nie den Klartext-Personenbezug. Die Zuordnung Person→Cluster liegt im `IdentityDirectoryOwner` (Spec §12.7).
- **R-E3** Kein Domänen-Crate wird über `contextra-store` direkt verwendet, sobald der Kern-Adapter steht. Der Store bleibt Übergangsimplementierung hinter einem Trait.
- **R-E4** Abhängigkeitsrichtung strikt nach unten. `contextra-storage` darf nur `storage-ports` und bewilligte Krypto-Adapter kennen (Layering-Test besteht bereits).
- **R-E5** Asynchron ist nur in `contextra-storage-bridge` und oben. Der Kern bleibt synchron (Spec K-12).

### 3.2 Abbildung der Domänen-Konzepte auf den Kern

| Domänen-Konzept | Kern-Konzept | Begründung |
|---|---|---|
| Mandant (`TenantId`) | Instanz-Partitionierung (eine `contextra-storage`-Instanz je Sharding-Einheit) **plus** `tenant_tag` (16 Byte, opak) in `ClusterSpec` für Packgruppen | Spec K-18: Ein Committer je Instanz; Mandanten-Sharding oben. |
| Person | Wurzelcluster, `Sealed`, `DataClass::Personal` (Spec §12.7) | Löschdomäne |
| Fall / Akte / Arbeitskontext | Fallcluster; Personen über Links (nur wenn die Löschung der Person den Fall mitziehen soll) | K-25 |
| Dokument | Record `doc/<id>/…` im Fallcluster (oder Personenwurzel) | Dokument ist Teil genau einer Löschdomäne |
| Chunk | Record im selben Cluster; ID = Sequenznummer im Cluster oder `MAC(cluster_secret, …)` | verhindert Querverweis über Löschdomänen (P1-10) |
| Vektor / Text-Posting / Graph-Kante | abgeleitete Segmente (Owner) mit Lineage auf Cluster; clusterlokale Mikroindizes möglich | Spec §12.4.1 |
| Session (Agent) | Schlüsselpräfix `sess/<id>/…` im Personencluster **oder** `Volatile`-Space; kein Cluster je Session | K-35 |
| Checkpoint | Manifest + Pin; Restore nur über Kern-Restore, nie über Domänen-Rollback | Spec §10.3 |
| Audit-Eintrag | Record im Cluster der betroffenen Person, `ChangeIndex`-Kette; Hashverkettung über Kern-Stamps | P1-19 |
| DLQ-Eintrag | Record im Cluster der Person oder `Volatile` (wenn Inhalt nicht persistent sein darf) | P1-20 |
| Löschauftrag (Erase-Request) | `Eraser::cluster` mit `EraseRequestId`; Owner-Attest pro Segmentbesitzer | Spec §9.2 |
| Löschnachweis | `ErasureReceipt` (Kern) + Owner-Attest → `DeletionProof v15` über Bridge | P1-22 |
| Transaktion (Domäne) | Kern-`Change` mit `IdemToken`; Domäne bildet `tx_id` auf Kern-Stamp ab, nicht umgekehrt | P1-6 |

**Entscheidung E-01 (Mandanten vs. Cluster):** Ein Mandant enthält viele Cluster. Der Mandant ist **keine** Löschdomäne. Das vermeidet Massenlöschung (P0-1) und hält Scans auf Clusterebene. Mandanten-Löschung ist eine Folge (`pack_expired` / Mandanten-Abgang), nie ein Präfix-Purge.

**Entscheidung E-02 (ID-Namensräume):** `DocId`, `EntityId`, `TxId`, `SpaceId`, `ClusterId` bekommen getrennte Typen in `storage-ports` (keine Ableitung aus Strings, keine Dezimal-Parse-Heuristik). Dokument-ID ist ein Pair `(ClusterId, u64 sequence)`; globale Eindeutigkeit über Instanz-Stamp `(instance_id, stamp)` oder 128-Bit-Zufall. Die BLAKE3-64-Ableitung entfällt (P1-4, P1-5).

**Entscheidung E-03 (Chunk-ID):** Chunk-ID = `HMAC(cluster_secret, "chunk" ∥ doc_seq ∥ chunk_index ∥ cfg_fingerprint)` mit dem im Cluster versiegelten Zufallssecret (Konvention K-32). Der Inhalts-Hash bleibt als Verifikationsfeld nur im Cluster-Seal erhalten, nie als global vergleichbarer Klartext-Hash.

### 3.3 Index-Segmente als Owner

Für jeden Index wird ein `SegmentOwner` mit einer der Klassen aus Spec §12.1 deklariert:

| Index | Klasse (Vorschlag) | Löschweg | Watermark |
|---|---|---|---|
| HNSW / DiskANN, Vektoren pro Cluster | `ClusterSections` (wenn klein) oder `Rebuildable` (tenantweit, partitioniert) | Gate-Filter sofort; physisch per Rebuild-Ersatz + `verify_absent` | `(instance_id, applied_stamp)` |
| Text-Postings | `ClusterSections` (Postings als Kern-Records desselben Clusters) | Kern-Purge über Records | derselbe |
| Graph (CSR) | `Rebuildable`, Kanten als Kern-Records mit Cluster-Lineage; CSR-Layout als RAM-Projektion | Gate-Filter; RAM-Projektion nach Gate neu aufbauen | derselbe |
| KV-Prefix-Cache | `Rebuildable`, nur `Plain` oder clusterlokal; kein Spill über Cluster | Gate-Invalidierung + `clear` | keiner (Cache) |
| Rank | keine Persistenz | – | – |

**Entscheidung E-04 (Vektor-ANN):** Der globale HNSW/DiskANN-Index bleibt `Rebuildable` (Spec §12.4.1, E-70). Clusterlokale Mikroindizes sind für personenbezogene Cluster mit kleinem Umfang zulässig, aber nicht vorgeschrieben. Die Wahl trifft der Owner je Mandant, nicht der Kern.

**Entscheidung E-05 (Tx-Semantik vereinheitlichen):** Alle Owner-Indizes implementieren dasselbe Transaktionsmodell: Staging bis `commit`, Sichtbarkeit nach durablem Watermark. Das DiskANN-Verhalten (P1-7) wird an HNSW angeglichen. Der Port `VectorIndex` wird im Zuge dessen aus `contextra-ports` (alt) auf `storage-ports` (Owner-Vertrag) umgestellt.

---

## 4. Löschkette im Ökosystem (Soll)

### 4.1 Ablauf eines Löschauftrags

```
Domäne (contextra-engine / Oberfläche)
  1. Auflösen: Person/Fall/Dokument → ClusterId-Menge        (IdentityDirectoryOwner, Spec §12.7)
  2. Policy: Restrictions, Retention, Legal Hold              (oberhalb des Kerns, Mechanismus im Kern)
  3. Eraser::cluster(H, request_id, level, deadline)          (ER0–ER7 des Kerns)
       ├─ Gate (logisch)                                      G0
       ├─ Owner-Gate-Hook: on_gate(H, at, gen) an alle SegmentOwner
       ├─ KeyDestroy (Cryptographic)                          G1
       ├─ Purge (Plain/Packed/Staging/Segmente)                G2 — nur über Kern-purge-Algorithmus
       └─ Owner-Attest: verify_absent je Owner (Vektor, Text, Graph, Cache, Checkpoint-Meta, Audit-Index)
  4. ErasureReceipt (Kern, kanonische Bytes)
  5. Bridge: Receipt + Owner-Attest → DeletionProof v15      (contextra-storage-bridge; signiert oben)
  6. Audit-Export: Verarbeitungsverzeichnis referenziert Receipt-Digest (nicht frei gefüllte Strings)
```

### 4.2 Was daraus für bestehende Crates folgt

- `contextra-engine/src/contextra_impl/erasure.rs`: `erase_document` und `erase_collection` werden **ersetzt** durch einen Aufrufer des Kern-`Eraser`. Der bisherige Scan- und Purge-Pfad entfällt; die Verifikation liegt im Kern (`Eraser::verify`) und in den Owner-Attesten. Die Metrik `UNMEASURED_ERASURE_LAYERS` wird durch `MeasureKind` (Spec §13.2.1) ersetzt.
- `contextra-crypto/src/kv_shredding.rs` und `kv/segment.rs` (Store) werden **entfernt**. Ihr Zweck übernimmt der Kern-KeyProvider (`destroy_many`) mit Cluster-Epochen. Die Registry-Persistenz (heute RAM) entfällt, damit keine zweite Schlüsselquelle entsteht.
- `contextra-crypto/src/deletion_proof/` wird durch die Bridge ersetzt. Signaturversionen 1, 2, 3, 5 werden nicht übernommen; `v15` ist der einzige Beweisformat-Vertrag, und er basiert auf `ErasureReceipt`.
- `contextra-audit-export` konsumiert `ErasureReceipt`-Digests, keine Freitext-Strings. `DeletionProofSummary` wird entfernt.
- `contextra-checkpoint` verliert die Eigenregie (Orphan-JSON, Hardlink-Filter). Snapshots laufen über den Kern-Checkpoint (`create`/`restore` in `storage/checkpoint/*`); der Restore prüft die Barriere (P0-5 ist dort erledigt, der Mock wird durch den realen Kern ersetzt).
- `contextra-kvcache` bekommt `delete` im Port (ergänzt in `storage-ports`), damit `delete_tenant` nicht mehr `put(…, &[])` verwendet.

### 4.3 Restexposition, die im Receipt ausgewiesen wird

Jeder Löschbeleg führt die Liste aus Spec §9.9 mit. Für das Ökosystem kommen hinzu:

- Dateisystem-/Journal-Reste, Swap, Core-Dumps (Host-Befund über `HostProbe`).
- Externe Kopien: Exporte (`contextra-avv`-Exporte, Logs), Provider-Logs von Embedding-/Modell-APIs, Prompt-Caches. Diese liegen außerhalb des Kerns und werden als `Declared` ausgewiesen.
- Modellgewichte und Fine-Tuning-Daten: nicht im System löschbar; im Receipt als `OutOfScope` markiert, wenn sie personenbezogene Daten enthalten könnten.

---

## 5. Verträge zwischen Kern und Ökosystem

Diese Tabelle definiert, welches Crate was über welchen Vertrag nutzt. Sie ist die verbindliche Schnittstellenliste für die Refaktorierung.

| Vertrag | Anbieter | Verbraucher | Form |
|---|---|---|---|
| `apply(Change) -> Receipt` | Kern (`storage`) | Engine, Ingest, Cognition, Agent, Checkpoint | synchron; Bridge async |
| `view_durable() -> VersionView` | Kern | Engine (Recall), Owner-Builder, Agent | Snapshot-Lease (Spec §11.4) |
| `Cursor` / `page(budget) -> Page` | Kern | Engine (Token-Budget), Agent-Kontext | `ResumeToken` AEAD-versiegelt |
| `Eraser::cluster / range / branch / space_drop` | Kern | Engine (Löschorchestrierung), Privacy-Entscheidung | Ticket + Receipt |
| `SegmentOwner` (Registrierung, `on_gate`, `rewrite_without`, `verify_absent`, `watermark`) | Owner-Crates (vector, text, graph, kvcache, cognition-Derivate) | Kern | Trait, Generation-Fencing |
| `ExtensionBlock` / `BlockPruner` | Kern (Container), Owner (Inhalt) | Owner | schattensicher nur für deklarierte Owner |
| Change-Feed (`Consumer`) | Kern | Owner-Builder, Cognition (Konsolidierung), Graph-Projektion | mindestens einmal; Stamp-geordnet |
| `IdentityDirectoryOwner` | Identity-Crate (neu, oberhalb Kern) | Engine, Privacy, Audit | Owner mit Subjekt-Key |
| `KeyProvider` | `keys-file` / `keys-cohort` / HSM-KMS | Kern | Port, Konformanzsuite |
| `Authority` | Policy/IAM (oben) | Kern (`LiftPolicy::FourEyes`) | Port |
| `Anchor` | `storage-fs` (weak) / Strong-Adapter | Kern | Port |
| `ErasureReceipt` → `DeletionProof v15` | Bridge | Audit-Export, Oberfläche | kanonische Bytes + Signatur oben |
| `MemoryBudget` / `ResourcePermit` | Kern (Governor) | alle Owner | Admission vor Allokation |
| `Metrics` / `Observer` | Kern (Katalog §16) | alle | low-cardinality, nie Inhalte |

### 5.1 Was die Domänen-Crates aus dem Kern nicht mehr bekommen dürfen

- Kein Zugriff auf `Fs`, `Clock`, `Entropy` direkt; sie erhalten Ports über die Fassade.
- Keine Schlüssel. Clusterschlüssel verlassen den KeyProvider nie; Owner arbeiten nur mit Seal-Diensten des Kerns (`SegmentIo`).
- Kein `unsafe`/`mmap` im Owner-Pfad, der personenbezogene Daten liest (Kern-Regel §3.1 gilt für alle Owner, die Nutzdaten halten).

---

## 6. Migrationsplan (Refaktorierung des Ökosystems)

Die Reihenfolge folgt den Phasen der Kern-Spec (§14). Jede Phase hat ein Abnahmekriterium, das vor der nächsten Phase steht.

### Phase M0 — Sofortmaßnahmen (vor jedem Kernausbau)

| # | Maßnahme | Betroffen | Abnahme |
|---|---|---|---|
| M0-1 | **P0-1 reproduzieren:** Test „Löschung eines Dokuments lässt fremde Dokumente im selben Tenant unberührt“. Bei Fehlschlag: Purger-Target-Auswertung korrigieren oder `erase_document` deaktivieren (Feature-Gate, Default aus). | `contextra-store/scanner/purge.rs`, `contextra-engine/erasure.rs` | Test grün; bis dahin kein produktiver Aufruf von `erase_document` |
| M0-2 | P0-3 und P0-4: Purger verschlüsselt schreiben; Fehler propagieren, kein `let _ =` in Purge | `contextra-store/scanner/purge.rs` | Negativtests mit `FaultFs`-Äquivalent (vorhandene Fehlerinjektion im Store) |
| M0-3 | P0-5: Checkpoint-Mock durch realen LSM-Pfad ersetzen; Barriere-Test | `contextra-checkpoint/tests/time_travel_correctness.rs` | Test schlägt fehl, wenn `last_tx > tx_id` zurückgibt Restore-Fehler |
| M0-4 | Dokumentation korrigieren: README-Aussagen zu Crypto-Shred, Audit-Verkettung und Rückkehr zu jedem Stand als „geplant“ kennzeichnen, bis Tests existieren | `README.md:159,163,190` | Review |
| M0-5 | `contextra-core` als Strangler-Facade einfrieren; keine neuen Abhängigkeiten darauf | `contextra-core` | Lint/Review |
| M0-6 | Graph: Präfixe vereinheitlichen (`__`), Hyperkanten-Hex/Dezimal angleichen, Cascade-Queue beim Laden lesen | `contextra-graph` | Restart-Test, der Produktionscode ausführt |

### Phase M1 — Typ- und Vertragsbereinigung (parallel zu Kern P1/P2)

| # | Maßnahme | Ergebnis |
|---|---|---|
| M1-1 | `storage-ports` ist die einzige Quelle für `Stamp`, `Key32`, `DataClass`, `Clock`, `Entropy`, `ClusterId`, `SpaceId`, `BranchId`, `InstanceId`, `TombId`. `contextra-types`, `contextra-ports`, `contextra-mvcc` re-exportieren oder werden entfernt. | keine Doppeldefinition mehr |
| M1-2 | `DocId`, `EntityId`, `TxId` aus `contextra-types` neu fassen (E-02): Pair `(ClusterId, u64)`, getrennte Namensräume | Migrationstool für bestehende IDs |
| M1-3 | Altes Cipher-/MAC-Set aus `contextra-crypto` entfernen; `contextra-crypto-ref` wird Referenz, Produktion nutzt `keys-file`/Adapter | ein AEAD, ein MAC, ein HKDF-Salt |
| M1-4 | `contextra-durable-fs` entweder in `storage-fs` integrieren oder entfernen; Fault-Injection auf eine Quelle | ein Fs-Stack |
| M1-5 | `DeletionProofSummary`, AVV-Shim und `bsi_mapping`-Verweis bereinigen | keine Freitext-Beweise |
| M1-6 | Port `KvBridgeStorage` um `delete` erweitern | P1-18 erledigt |

### Phase M2 — Owner-Anpassung (nach G-F des Kerns)

| # | Owner | Maßnahme |
|---|---|---|
| M2-1 | `contextra-text` | Postings als Kern-Records (`ClusterSections`); `TextPurger` durch Kern-Purge ersetzen; `resolve_tombstones` nur noch als Owner-Interna |
| M2-2 | `contextra-vector` | HNSW/DiskANN-Owner; einheitliches Staging; `*.tombstone.wal` entfällt; mmap nur für `Rebuildable`-Segmente nach Kern-Atom-Vertrag |
| M2-3 | `contextra-graph` | Kanten/Entitäten als Kern-Records; CSR als RAM-Projektion, Neuaufbau über Change-Feed; Hyperkanten als Records; `EntityId`-Namensraum getrennt |
| M2-4 | `contextra-kvcache` | `Rebuildable`-Owner; `delete` im Port; Radix-Bereinigung bei Eviction; Content-Index mit Cluster-Secret statt `blake3(tokens)` |
| M2-5 | `contextra-ingest` | Chunk-IDs nach E-03; Re-Ingest mit Versionszähler je Dokument, altes Dokument per Change ersetzt; kein `max_chunks`-Stillfehler ohne Meldung |
| M2-6 | `contextra-agent` | Session-Records im Personencluster; DLQ und Audit als Kern-Records, Audit mit Stamp-Hashkette; `is_already_committed` über Idem-Token; Fail-open entfernen |
| M2-7 | `contextra-cognition` | Intents/Konsolidierung als Change; Derivate mit `link(derived, sources)` (Lineage), Löschung der Quelle kaskadiert auf Derivate |
| M2-8 | `contextra-checkpoint` | Kern-Checkpoint; Orphan-Register in den Kern; Hardlink-Klone durch Kern-Klon mit `rewrite_without`-Filter ersetzen |
| M2-9 | `contextra-engine` | Löschorchestrierung ersetzen (§4.2); `Collection` als Dokument-/Relationen-Schicht über `view`/`apply`; Mandantenbezug über `ClusterSpec` |
| M2-10 | `contextra-privacy` | Egress-Entscheidung mit `Eraser`-Aufruf verbinden (`pii_vault` → Cryptographic-Ticket) |

### Phase M3 — Oberfläche und Anbindung

| # | Crate | Maßnahme |
|---|---|---|
| M3-1 | `contextra-storage-bridge` | async-Fassade, Mapping `ErasureReceipt → DeletionProof v15`, Transaktionsmapping |
| M3-2 | `contextra-db`, `contextra`, `contextra-py`, `contextra-router`, `contextra-mcp` | nur noch über Domänen-API; Store-Import entfernen |
| M3-3 | `contextra-router` | Prod-Abhängigkeit auf `storage-ports`-Port umstellen (ist bereits Port-basiert) |
| M3-4 | `contextra-audit-export`, `contextra-avv-generator` | Konsumenten von Receipts; AVV-Feld für Löschfristen aus `ClusterSpec.due` |
| M3-5 | `contextra-wire`, `contextra-sandbox`, `contextra-license`, `contextra-simd`, `contextra-sys`, `contextra-adapt`, `contextra-rank` | keine Storage-Berührung; nur Abhängigkeitsbereinigung |

### Phase M4 — Abschaltung des LSM (G-CUTOVER)

- `contextra-store` bleibt bis G-CUTOVER im Workspace, produktiv nicht mehr referenziert. Danach Archiv-Tag `legacy-store-final`, Entfernung aus dem Workspace.
- Migrationswerkzeug: Export aus LSM über Domänen-API nach Kern-`apply`; Verifikation per Löschorakel und Rekonzilierung der Zähler. Kein Dual-Write.

### Abhängigkeitsreihenfolge (Kurz)

```
M0  →  M1 (Typen, Verträge)  →  Kern G-F  →  M2 (Owner)  →  M3 (Oberfläche)  →  G-CUTOVER  →  M4
```

---

## 7. Entscheidungen, die vom Team zu treffen sind

| ID | Frage | Vorschlag | Auswirkung bei Ablehnung |
|---|---|---|---|
| D-1 | Ist ein Mandant Löschdomäne? | Nein (E-01) | Massenlöschung bleibt technisch möglich, aber Kostenproblem und Überlöschungsrisiko |
| D-2 | Vektor-Index tenantweit `Rebuildable` oder clusterlokal? | Tenantweit, partitioniert (E-04) | Hohe RAM-Kosten bei vielen kleinen Clustern (K-35) |
| D-3 | Chunk-ID mit Cluster-Secret HMAC? | Ja (E-03) | Dedup über Mandanten/Cluster hinweg bleibt sichtbar (P1-10) |
| D-4 | Restore-Barriere hart erzwingen oder Zeitreise nur lesend? | Hart (Kern-Spec §10.3); Zeitreise lesend über `view_at` | Restore auf alte Stände ohne Löschprüfung möglich |
| D-5 | `Volatile` für DLQ-Inhalte mit personenbezogenen Daten? | Ja, wenn Inhalt personenbezogen ist; Metadaten persistent | Klartext-DLQ bleibt |
| D-6 | Sessions im Personencluster oder eigener Cluster? | Personencluster mit Schlüsselpräfix (K-35) | 10⁷ Mikro-Cluster belasten RAM (H-25) |
| D-7 | Strong-Anker verpflichtend für Produktion? | Ja, `Required` bei Encrypted (Spec §9.7) | Rollback-Schutz nur schwach |
| D-8 | `contextra-core` (Strangler) erhalten oder entfernen? | Entfernen nach M3 | doppelte Fassaden |
| D-9 | Eigene Crate-Topologie für `storage-*` beibehalten? | Ja, Layering-Test ist sinnvoll; aber `storage-fs` und `durable-fs` zusammenführen (M1-4) | |
| D-10 | Ist der Audit-Hash-Chain im Agent-Pfad Pflicht? | Ja (P1-19); Verankerung über Kern-Stamp | fehlende Manipulationssicherheit trotz Doku |

---

## 8. Abnahmekriterien (Ökosystem)

Diese Kriterien sind zusätzlich zu den Gates der Kern-Spec (§14) zu erfüllen.

| ID | Kriterium | Nachweis |
|---|---|---|
| EK-1 | Löschung eines Dokuments lässt alle fremden Records desselben Mandanten unverändert | Integrationstest mit ≥ 2 Personen, Byte-Vergleich |
| EK-2 | Nach `Done` liefert kein Owner einen Treffer für den gelöschten Cluster; Owner-Attest vorhanden | Owner-Konformanzsuite, Löschorakel über alle Owner |
| EK-3 | Bei Cryptographic-Erase sind Derivate (Summaries, Chunks, Vektoren, Postings, Graph-Kanten, Cache-Einträge, Checkpoints, DLQ, Audit-Einträge) als Cluster-Records oder als `Rebuildable` mit Attest erfasst | Erfassungsmatrix, jeder Eintrag mit Owner |
| EK-4 | Restore auf Checkpoint mit neueren Commits schlägt fehl, solange nicht bewusst zurückgerollt wird | realer Kern-Restore-Test (nicht Mock) |
| EK-5 | Kein Domänen-Crate persistiert personenbezogene Daten außerhalb des Kerns oder registrierter Owner | statische Prüfung: Liste aller Dateizugriffe; CI-Lint |
| EK-6 | Kein Löschnachweis enthält frei eingegebene Strings als Beleg | Bridge-Test: Receipt-Digest-Abgleich |
| EK-7 | Doppelte Typen (Stamp, Key32, DataClass, Clock, Cipher, KeyProvider) existieren nicht mehr | Workspace-Lint `duplicate_definitions` (Skript) |
| EK-8 | Hyperkanten, Cascade-Queue und Entity-Tombstones überleben Neustart (Produktionscode im Test) | Restart-Tests |
| EK-9 | Ingest: Re-Ingest eines unveränderten Dokuments erzeugt keine Duplikate im Index | Differenztest |
| EK-10 | Alle Owner-Indizes haben identisches Transaktionsverhalten (Staging, Commit, Sichtbarkeit) | Property-Test gegen Referenzmodell |

---

## 9. Verifikation (Pflichttests vor Refaktorierung)

Die folgenden Tests sollen **vor** dem Umbau geschrieben und gegen den Ist-Stand laufen, damit die Befunde P0-1 bis P0-5 belegt oder widerlegt sind:

1. `test_erase_document_preserves_siblings`: zwei Dokumente, eines löschen, das andere byte-identisch prüfen (P0-1).
2. `test_erase_verifier_uses_independent_targeting`: Verifier darf nicht dieselbe Target-Logik wie Purger nutzen (P0-2).
3. `test_purge_encrypted_store_no_plaintext_residue`: Store mit `Encrypted`, Purge, Dateiscan auf Klartext-Canaries (P0-3).
4. `test_purge_reports_io_error`: Fehler injizieren, `Ok` ist verboten (P0-4).
5. `test_restore_barrier_real_engine`: Commit nach Checkpoint, Restore muss scheitern (P0-5).
6. `test_hyperedge_restart_production_path`: Produktionsloader, kein Nachbau im Test (P1-1, P1-2).
7. `test_entity_tombstone_survives_readd` (P1-3).
8. `test_chunk_id_no_cross_document_equality` (P1-10).
9. `test_reingest_idempotent` (P1-11, EK-9).
10. `test_dlq_idempotence_exact_match` (P1-20).

Jeder Test, der die Befunde bestätigt, erzeugt einen Eintrag in `docs/25-r8-audit-remediation.md` (bzw. dessen Nachfolger) mit Befundnummer, Commit-Hash und Ergebnis.

---

## 10. Offene Punkte und Restrisiken

- **Analysegrenze:** Die Lesung ist statisch. Keine Build-, Test- oder Laufzeitmessung. Die `[H]`-Punkte (insbesondere P0-1) sind die ersten Kandidaten für eine Verifikation und können sich als unzutreffend herausstellen.
- **Ungeprüft:** Die Abhängigkeitsketten in den Workspace-Features (`docid-128`, `legacy-wal-key`, `memory-only-storage`) wurden nicht gegen alle Kombinationen gelesen.
- **Nicht gelesen:** Die Fuzz-Ziele, Benchmark-Kerne und `contextra-simd`-Implementierungen wurden nur oberflächlich betrachtet.
- **Ökonomisch:** Der Umbau betrifft nahezu alle Crates. Realistischer Aufwand: deutlich über dem Phasenplan der Kern-Spec. Die Schätzung in Spec §18 (Risiko 9) gilt umso mehr.
- **Rechtlich:** Die Aussage „kryptografisch gelöscht“ hängt von der Schlüssel-Zusicherung (`KeyAssurance`, Spec §9.4) und vom Backup-Horizont (§10.4) ab. Eine Freigabe für Personendaten erfordert die externe Prüfung aus Spec §18.

---

## Anhang A — Abgleich der Analysequellen

| Analysebereich | Schwerpunkt | Wichtigster Befund |
|---|---|---|
| `contextra-store` und Löschkette | Purge-Target, Verschlüsselung, Fehlerbehandlung | P0-1, P0-3, P0-4 |
| Index-Crates (vector/graph/text/rank/kvcache) | Persistenz, Tombstones, IDs | P1-1 bis P1-9 |
| Konsumenten (engine/db/agent/cognition/ingest/checkpoint) | Löschpfade, Checkpoint, Audit | P0-5, P0-6, P1-19 bis P1-21 |
| Crypto/Ports/Bridge/Rest | Typdoppelungen, Bridge-Stand, Verwaiste Crates | P1-13 bis P1-18, P1-22 |

## Anhang B — Änderungen gegenüber der Kern-Spec R9.1

Diese Spec ändert die Kern-Spec nicht. Folgende Punkte sind **Ergänzungen**, die in R9.2 des Kerns übernommen werden sollen:

- **§12.9 (neu) Ökosystem-Vertrag:** Verweis auf E-01 bis E-05 und die Tabelle in §5.
- **§9.9 Restexposition:** Ergänzung um Kategorien aus §4.3.
- **§15 Invarianten:** neue Kandidaten `INV-ECO-1` (keine Domänen-Persistenz personenbezogener Daten außerhalb des Kerns/Owner), `INV-ECO-2` (kein Präfix-Purge; Löschziel ist immer eine ClusterId-Menge).
- **§2.5 Produktverträge:** Zeile „Löschnachweis“ um „Owner-Attest und Digest-Bindung an DeletionProof v15“ erweitern.
