# Contextra — Löschnachweis: Ist-Analyse und Restplan

**Geprüfter Stand:** `github.com/tfufuz1/contextra` @ `7e1d65b` (main, 10.10.2026, nach #4494). Die Dokumente vom 08.10. (`ef39f9f`) und 09.10. (`ace2a42`) liegen davor.
**Rolle:** Principal Senior Rust Architect, Contextra

---

## 0. Verifikationsstatus (zuerst lesen)

| Aussage | Status |
|---|---|
| Alle Datei-/Zeilenangaben und Befunde in diesem Dokument | **Nur Code-Lektüre.** Nichts wurde kompiliert oder ausgeführt: `static.rust-lang.org` ist in dieser Sitzung per Egress-Policy gesperrt, die Toolchain (Rust 1.89.0) ließ sich nicht installieren. |
| Befund G1 (Tenant-Diskrepanz, §2) | **Wichtigster Einzelbefund, aber unbestätigt.** Er folgt zwingend aus dem gelesenen Code, wurde aber nicht per Test reproduziert. Deshalb ist WP-L0.1 ein Reproduktionstest, bevor irgendetwas repariert wird. |
| Befund G6 (`encryption-at-rest` wird nie aktiviert) | Aus `Cargo.toml`-Lektüre. Prüfbefehl in §9. |
| Übernahmen aus dem Gegenprüfungs-Dokument (WAL, RevocationLog-Hotpath, LSM-Merge) | **Nicht am HEAD neu geprüft**, ausdrücklich markiert. |

---

## 1. Wie der Löschnachweis heute tatsächlich funktioniert

**Datenmodell** (`contextra-crypto/src/deletion_proof.rs`)
- `DeletionProof`: `scope` (Document/Collection/Tenant), `deleted_keys_hash` (Blake3 über sortierte, längenpräfixierte Schlüssel), `deleted_after_tx`, `covered_layers`, `excluded_scopes`, optional `wal_chain_receipt`, `audit_chain_position`, `graph_repair`, `signature`.
- Signaturversionen: v1 (Legacy, signiert Layer nicht mit), v2 (HMAC-SHA256, symmetrisch), v3 (Ed25519).
- `LayerCleanupProof` ist ein Wrapper um einen `DeletionLayer`. Er entsteht über `new_after_verified_empty(layer, usize)` oder `verify_and_create(layer, closure)`; beide sind `pub` und nehmen eine **vom Aufrufer gelieferte** Zahl bzw. Closure (Z. 163, 190).
- `verify()` und `verify_external()` prüfen **ausschließlich die Signatur**. Das steht so im Code (Z. 621: „NICHT-GARANTIE").

**Einziger Produktionspfad, der überhaupt einen Nachweis erzeugt:** `Contextra::drop_collection` (`contextra-engine/src/contextra_impl/collections.rs:300`).
1. Scannt `__col:{name}:` und `__txt:{name}:` über `TenantScopedStorage` → `deleted_keys`.
2. `delete_prefix` + `delete` + `commit` (Tombstones).
3. Scannt dieselben Präfixe erneut; die Anzahl der Treffer geht in `new_after_verified_empty(LsmMemtable, n)`.
4. `DeletionProof::create(...)` → **v2, HMAC**, Schlüssel = Umgebungsvariable `CONTEXTRA_DELETION_PROOF_KEY`.
5. Gibt nur `LsmMemtable` an (korrekt zurückhaltend, Z. 378–383).

**MCP:** `contextra_forget` mit `id` löscht ein Dokument per Tombstone und liefert `proof: null` (`server_tools.rs:428–440`). Nur `forget` ohne `id` ruft `drop_collection` auf (Z. 442–468).

**Nie im Produktionscode erzeugt oder aufgerufen** (per Suche außerhalb von Tests, Fuzz, Examples, Benches):
`DeletionScope::Document`, v3-Proofs, `DeletionProofKeyPair`, `compute_wal_delete_receipt`/`wal_chain_receipt`, `audit_chain_position`, `GraphRepairAttestation`, `verify_external`, `DeletionProofBuilder` (Typestate), alle Layer außer `LsmMemtable`.

---

## 2. Was fehlt — Lückenliste

Priorität: **P0** = Nachweis kann falsch sein oder Löschung findet nicht statt · **P1** = Nachweis ist nicht prüfbar/fälschbar · **P2** = Abdeckung unvollständig · **P3** = Härtung.

### P0 — Der Nachweis kann etwas Falsches bescheinigen

**G1 · Tenant-Diskrepanz im MCP-Löschpfad (unbestätigt, siehe §0)**
- MCP schreibt und liest über `db.collection(name)` (`server_tools.rs:74,223,361,429`) → **un-tenantisierter** Speicher, Schlüssel `__col:{name}:\x00…` ohne Präfix (`collections.rs:182–190`, `collection/mod.rs:604–634`).
- `forget` für eine Collection ruft `drop_collection(name, TenantId::try_new(1)…)` (`server_tools.rs:448`, `tools_crud.rs:124`). Das baut `TenantScopedStorage`, dessen `make_key` jeden Schlüssel mit `t:1:` voranstellt (`tenant_codec.rs:500–506`, Präfix aus Z. 33).
- Folge nach Code-Lage: Scan, `delete_prefix` und Re-Scan arbeiten auf dem Raum `t:1:__col:…`, der leer ist. Die echten Daten bleiben stehen, `remaining == 0`, ein **gültig signierter Nachweis wird trotzdem ausgestellt**. Auch `__col_idx:` wird nicht entfernt (wird untenantisiert geschrieben, `collections.rs:205`), die Collection bliebe in `list_collections` sichtbar.
- Warum es nicht aufgefallen ist: `test_contextra_forget_collection_with_proof` (`contextra-mcp/src/tests.rs`, ~Z. 306–370) prüft nur die Signatur, nie ob die Daten weg sind. `drop_collection_proof_vs_files.rs` besteht per Konstruktion auch bei Restdaten (bei `residue == true` wird nur geprüft, dass `LsmMemtable` der einzige Layer ist). Die Engine-Tests, die Löschen belegen, nutzen `collection_for_tenant`.

**G2 · Der `LsmMemtable`-Nachweis ist inhaltsleer** (bestätigt aus Gegenprüfung 2.1, am HEAD wiederholt gelesen)
- Der Re-Scan nach `commit` liefert per MVCC-Konstruktion immer 0; er kann nie fehlschlagen. Zusätzlich: `deleted_keys` werden **vor** `delete_prefix` gescannt (Z. 323–337 vs. 341) → konkurrierende Schreibzugriffe fehlen im Nachweis; `collection_id` ist ein auf 8 Byte gekürzter Blake3 des Namens mit Fallback `CollectionId::new(1)` (Z. 366–371) → Kollision möglich.

**G3 · `excluded_scopes` bleibt leer** (`collections.rs:402`: `vec![]`)
Die Typdoku nennt das Feld „Pflicht für DSGVO Art. 17" (`deletion_proof.rs:305`). Der Nachweis erklärt nirgends, dass destillierte/konsolidierte Inhalte und LLM-Parameter **nicht** abgedeckt sind.

### P1 — Nicht prüfbar oder fälschbar

**G4 · Kein extern prüfbarer Nachweis im Produktpfad**
`drop_collection` nimmt `proof_key: &[u8]` und erzeugt v2-HMAC. Prüfen kann nur, wer das Geheimnis kennt. v3-Ed25519 existiert, hat aber im Produktionscode keinen Aufrufer: keine Schlüsselverwaltung, keine Public-Key-Verteilung, kein `verify-proof`-Kommando. `DeletionProofBuilder::finish()` ruft ebenfalls `create_with_wal_receipt` (v2) auf (`deletion_proof_typestate.rs:299`).

**G5 · Es gibt keinen Richtlinien-Verifier**
`verify`/`verify_external` akzeptieren auch einen Nachweis ohne Layer und ohne `deleted_keys` (`DeletionProof::create(scope, vec![], …, vec![], vec![], key)` ist in den eigenen Tests gültig). Nichts prüft, ob die deklarierten Layer zum Scope passen.

**G6 · Der ausgelieferte Build nutzt vermutlich eine Stub-Implementierung**
- `contextra-engine` hat `default = []`; Feature `encryption-at-rest` wird in **keiner** `Cargo.toml` der Workspace-Crates aktiviert (nur definiert, `contextra-engine/Cargo.toml:69`). `contextra-mcp` (`default = []`) und die Fassade (`default = ["fast","candle"]`) schalten es nicht ein.
- Ohne das Feature exportiert die Engine ein **eigenes** `LayerCleanupProof` mit `pub`-Feldern, eine eigene HMAC-v2-Signierung und `integrity_warning: "generated without feature encryption-at-rest … no timestamp"` (`contextra-engine/src/lib.rs:13–14, 20–245`). Dort kann jeder Aufrufer `LayerCleanupProof { layer, remaining_count: 0 }` direkt bauen.
- Konsequenz, falls zutreffend: Selbst die Härtungen im Crypto-Crate greifen im ausgelieferten MCP-Binary nicht, und es existieren zwei divergierende Proof-Implementierungen.

**G7 · Konstruktoren sind offen** (bestätigt)
`new_after_verified_empty` und `verify_and_create` sowie sechs `DeletionProof::create*` sind `pub` (`deletion_proof.rs:163,190,332–546`). `LayerCleanupProof::_private: ()` verhindert nur das Struct-Literal.

**G8 · v3-Signaturpayload ohne Domänentrenner**
`construct_v3_payload` (`deletion_proof.rs:738–785`) hängt Felder ohne Präfix und ohne Domänen-Tag aneinander (v2 hat beides). Praktisch gedämpft, weil `bincode`-Vektoren selbst längenpräfixiert sind; trotzdem Härtungsbedarf. `timestamp` ist ein vom Aufrufer übergebener Wert (`attested_at`), nicht authentifiziert. `create_full_v3` verlangt bei `HnswIndex` ein nicht leeres `graph_repair`, das nirgends erzeugt wird.

**G9 · Nachweise werden nicht gespeichert und nicht verkettet**
Der Nachweis existiert nur im Rückgabewert. Kein Eintrag in einer Audit-Chain (keine Nutzung von `AuditChain` in Engine/MCP/DB), keine WAL-Quittung, keine Ablage.

### P2 — Abdeckung

**G10 · Einzeldokument-Löschung hat keinen Nachweis**
`Collection::delete` (`collection/crud/delete.rs`) setzt Tombstones für `user_key` und `doc_key`, stellt Text-/Graph-Löschung in die Transaktion und ruft `index.delete`. Kein Proof, kein Scope `Document`. Das ist das, was ein Agent-Client am häufigsten braucht (`memory_forget(id)` der Spezifikation). Zusätzlich offen: Welche weiteren Schlüsselfamilien (z. B. `__rel:`, Term-/Postings-Schlüssel) pro Dokument verbleiben, ist nicht inventarisiert.

**G11 · Physische Layer haben weder Scanner noch Löschoperation**
- LSM: Es gibt nur `force_flush` (`lsm/flush.rs`), keine gezielte Voll-Compaction oder Purge-Operation. Tombstones werden nur bei **voller** Compaction und nur unterhalb von `min_snapshot_seq` entfernt (`compaction/engine.rs:662–664, 762–765`). Aktive Snapshots halten Versionen.
- Dateien werden per rohem `tokio::fs::remove_file` entfernt (`compaction/engine.rs:269,319,324,489`; `lsm/ops/compaction.rs:193,198,236`), Fehler nur geloggt; `durable_remove` (`contextra-durable-fs`) wird nur in `recovery.rs` genutzt. Keine Überschreibfunktion vorhanden.
- Checkpoints legen **Hardlink-Klone** der SSTables an (`contextra-checkpoint/src/hardlink_cloner.rs`). Ein Entfernen des Originals löscht dann keine Bytes.
- WAL/LSM-Nutzdaten: Nach Code-Lage nutzt das WAL einen einzelnen `KeyManager` (Parameter `key_manager: Option<…>` in `wal/io.rs`, `wal/flusher.rs`); eine Ableitung pro Tenant oder Dokument habe ich nicht gefunden, **aber nicht vollständig durchsucht**. Crypto-Shredding fand ich nur für KV-Cache-Segmente (`contextra-store/src/kv/segment.rs`, `KvDeleteMode::CryptoShred`; Profil in `contextra/src/collection_profile.rs`). Falls das stimmt, trägt die im 08.10.-Plan genannte Alternative „Crypto-Shredding statt Scrub für WAL" für Dokumentdaten nicht. WP-L4.3 beginnt deshalb mit einer Prüfung dieser Annahme.
- HNSW: `do_delete_node_idx` (`core_rebuild.rs:362`) nullt RAM-Vektoren nur, wenn kein Snapshot-Pin entgegensteht, und setzt sonst nur den Tombstone. Der Kommentar dort stellt selbst klar, dass das **kein** `HnswIndex`-Nachweis ist. `remove_with_graph_repair` (Kantenreparatur) hat **keinen Produktionsaufrufer**; im Code verwirft `let _ = update_backlink(...)` Fehler und mmap-residente Nachbarn werden übersprungen (`deletion.rs:281–289`).

**G12 · `DeletionLayer` ist unvollständig**
Es fehlen: BM25-/Text-Index im RAM, Checkpoint-/Snapshot-Klone, Relationen. Ohne diese kann ein Nachweis „alles gelöscht" nicht ehrlich ausgedrückt werden.

**G13 · Tenant- und Collection-weite Löschung**
`purge_tenant` (`collections.rs:429`) gibt `()` zurück: ein `delete_prefix(b"")`, kein Nachweis, keine Verifikation, kein Index-/Graph-Abgleich.

### P3 — Härtung (aus Gegenprüfung übernommen, nicht neu geprüft, außer wo vermerkt)

- **G14 · RevocationLog/Crypto-Shredding (KV-Cache-Spur):** `routing.rs:188` baut `KvSegmentCipher::new(master_km)` **ohne** Log. `RevocationLog` wird im gesamten Nicht-Test-Code außerhalb von `contextra-crypto` nicht verwendet. Der 08.10.-Plan beschreibt einen Aufruf `routing.rs:206 open_or_create`; **diesen Aufruf gibt es am HEAD nicht**. Das Risiko ist daher anders: Widerrufe werden im MCP-Pfad weder persistiert noch durchgesetzt. `RevocationLog::open_or_create` ist weiterhin `pub` und hartkodiert `has_existing_keys: false` (`revocation_log.rs:261–276`, am HEAD bestätigt). Hotpath-Kosten (`is_group_revoked`/`append` O(N)/O(N²)), unsignierter Marker und Fail-open bei Lock-Poisoning stammen aus der Gegenprüfung und sind nicht neu verifiziert.
- **G15 · WAL-Truncation-Anker:** `recovery.rs:349–363` setzt `check_hmac = expected_hwm`, sobald irgendein gültiger SSTable existiert; dann vergleicht `verify_wal_chain_completeness` einen Wert mit sich selbst. Nach Code-Lage wäre die Prüfung in diesem Fall wirkungslos (abgeschnittener WAL-Tail mit Delete → Wiederauferstehung). **Muss per Test belegt werden**, bevor es als Befund gilt.
- **G16 · LSM-N-Wege-Merge:** Der Test `campaign_deepresearch_merge_overlap` schlug laut Gegenprüfung bei `c724d577` fehl. Am HEAD nicht geprüft. Ein Purge über Compaction setzt korrekte Compaction voraus (Datenverlust-Risiko, kein Leck).
- **G17 · Dokumentations-Drift:** `contextra-mcp/src/lib.rs:16–17` und die Tool-Tabelle in `sandbox.rs:45` behaupten, `contextra_delete` stelle einen v3-Nachweis aus; der Code stellt keinen aus.

---

## 3. Korrekturen am Plan vom 08.10.

1. **Abhängigkeitszyklus.** Der Plan lässt `contextra-crypto` Token-Typen aus `contextra-store` importieren (`LayerCleanupProof::from_sstable_scan(token: contextra_store::…)`). Aber `contextra-store` und `contextra-vector` hängen bereits von `contextra-crypto` ab (`contextra-store/Cargo.toml:43`, `contextra-vector/Cargo.toml:20`). Das wäre ein Zyklus.
2. **„Vom Typsystem erzwungen" gilt nicht über Crate-Grenzen.** Rust kann einen `pub`-Konstruktor nicht auf ein bestimmtes anderes Crate beschränken. Wer ein Crate importiert, das den Token-Typ exportiert, kann ihn bauen, falls ein `pub`-Konstruktor existiert. Ehrliche Lösung: Witness-Typen in einem unteren Crate (`contextra-types` oder `contextra-core`) mit **einem** dokumentierten Ausstellungs-Konstruktor, dessen Aufrufer ein `xtask`-Gate auf `store`/`vector`/`engine`-Scanner-Module beschränkt. Das ist Konvention plus CI, kein Typbeweis; das Dokument sollte das so nennen.
3. **Phase 0 „RevocationLog in routing.rs":** trifft am HEAD nicht zu (siehe G14).
4. **Phase 4 „HNSW-mmap-Overlay":** im Engine-Pfad nicht nötig. `Collection::load_index` baut den HNSW-Index beim Start aus den LSM-Werten neu (`collection/mod.rs:672–701`); es gibt im Engine-Pfad keine persistente mmap-Indexdatei, die Vektorbytes hält. Der Overlay lohnt nur, wenn die eigenständige Snapshot-Persistenz von `contextra-vector` produktiv genutzt wird. Das ist hier nicht der Fall. (Die HNSW-Vektoren selbst stehen im LSM als `StoredDocument` mit `embedding`.)
5. **Reihenfolge:** Der Plan beginnt bei Token-Architektur. Vorher muss geklärt sein, ob der bestehende Nachweis überhaupt löscht (G1) und welche Implementierung ausgeliefert wird (G6).

---

## 4. Zielgarantie

> **„Ein `DeletionProof` deklariert einen Layer L nur dann als abgedeckt, wenn der Code des Crates, das L physisch besitzt, nach der Löschung für genau den signierten Schlüsselraum einen Scan ausgeführt hat, der keine lesbaren Daten fand, und der Nachweis diesen Scan (Anzahl geprüfter Dateien/Knoten, TxId, Scope-Hash) enthält. Nicht prüfbare Layer erscheinen nicht in `covered_layers`, nicht abgedeckte Bereiche stehen in `excluded_scopes`."**

Immer außerhalb der Garantie (muss im Nachweistext stehen): SSD-Wear-Leveling und Firmware-Remapping, Dateisystem-Snapshots, Backups, Swap, Core-Dumps, RAM-Kopien außerhalb des Prozesses, bereits exportierte Daten, LLM-Parameter.

**Stufenmodell** (jede Stufe ist einzeln auslieferbar und ehrlich benennbar):

| Stufe | Behauptbar | Voraussetzung |
|---|---|---|
| S0 | „Logisch gelöscht": Tombstones committed, Daten über die API nicht mehr les-/findbar | Phase 0–1 |
| S1 | + HNSW (RAM nullt, keine Eingangskante), Text-Index, Graph | Phase 3 |
| S2 | + SSTables aller Level physisch bereinigt (Dateiscan) | Phase 4.1–4.2, 4.4 |
| S3 | + WAL-Segmente bereinigt | Phase 4.3 |
| nie | SSD-physisch, Backups, Swap | Betriebsbedingung |

---

## 5. Entscheidungen, die du treffen musst

| Nr. | Frage | Empfehlung |
|---|---|---|
| D1 | Stub-Implementierung der Engine: entfernen oder `encryption-at-rest` standardmäßig an? | **Stub entfernen**, `contextra-crypto` zur festen Abhängigkeit machen. Ein Nachweis, der je nach Build-Flag eine andere Semantik hat, untergräbt sich selbst. Test `no_crypto_proof_stub.rs` entfällt bzw. wird auf die echte Implementierung umgestellt. |
| D2 | Signaturverfahren im Produktpfad | **Ed25519 v3 (bzw. v4)** verpflichtend; HMAC v2 nur noch verifizierbar, nie mehr ausgestellt. |
| D3 | Schlüsselverwahrung für das Proof-Schlüsselpaar | Datei unter `~/.contextra/` mit Modus 0600, Public Key separat exportierbar. Umgebungsvariable `CONTEXTRA_DELETION_PROOF_KEY` entfällt für neue Nachweise. |
| D4 | Physisches Löschen: Standardverhalten | Standard **S1**; physischer Purge als expliziter Modus (`purge: true`), weil er I/O-intensiv ist und bei vorhandenen Checkpoints scheitern kann. |
| D5 | Überschreiben (Scrub) alter Dateien | Optional, standardmäßig aus. Wenn an: Nullen schreiben + `fsync` + `durable_remove`, Nachweistext nennt die SSD-Grenze. |
| D6 | Verhalten bei aktiven Checkpoints/Snapshots | **Layer nicht beanspruchen** (und Purge liefert `PurgeBlocked`), nicht stillschweigend weitermachen. |

---

## 6. Implementierungsplan

Format wie in der Spezifikation: ein Arbeitspaket = ein PR = eine Abnahme; Regressionstest zuerst (rot → grün); rohe Logs im PR; keine neue `pub`-Funktion ohne Aufrufer. Aufwand in Agenten-Sitzungen. Abnahmebefehle sind `cargo`-Befehle, die **in dieser Sitzung nicht laufen konnten**.

### Phase 0 — Wahrheit herstellen (keine neuen Features)

**WP-L0.1 Reproduktionstest für G1** (1 Sitzung)
- Datei: `crates/contextra-mcp/tests/forget_collection_removes_data.rs` (neu).
- Ablauf: `contextra_insert` (Collection `c`, Dokument mit Markerstring) → `flush` → `contextra_forget{collection: c, confirm: true}` mit Proof-Schlüssel → `contextra_get`/`contextra_search`/`contextra_collections` und Roh-Scan des DB-Verzeichnisses auf den Marker (Hilfsfunktion aus `drop_collection_proof_vs_files.rs` übernehmen).
- Erwartung heute (nach Code-Lage): **rot**: `get` liefert das Dokument weiterhin, `collections` listet `c`. Wenn der Test grün ist, ist G1 widerlegt und der Plan wird entsprechend angepasst (Prompt-Rahmen: „stoppe und berichte").
- Abnahme: `cargo test -p contextra-mcp --test forget_collection_removes_data` — das Ergebnis dokumentiert den Befund.

**WP-L0.2 Fix G1: Löschraum aus demselben Handle ableiten** (2 Sitzungen)
- Dateien: `collections.rs`, `collection/mod.rs` (neue `Collection::purge_all(&self, tx)`), `server_tools.rs:442–468`, `tools_crud.rs:91–140`.
- Änderung: `drop_collection` erhält einen expliziten Speicherbereich (`StorageScope::{Untenanted, Tenant(TenantId)}`). Die Schlüsselmenge kommt aus dem `Collection`-Handle, das auch geschrieben hat (`user_key_prefix()`, `storage()`), nicht aus neu berechneten Präfixen. Fail-closed: Hat das Handle Daten (Scan > 0) und löscht die Operation 0 Schlüssel, bricht sie ab, statt einen Nachweis auszustellen.
- Grundlage: Es existiert bereits `Collection::drop_collection(&self)` (`collection/maintenance.rs:813`). Es löscht über `self.prefix` und `__txt:{name}:` im Schlüsselraum des Handles (also korrekt), hat aber **keinen Aufrufer**, entfernt `__col_idx:` nicht, setzt HNSW-/Graph-/Text-RAM-Zustand nicht zurück und erzeugt keinen Nachweis. `Contextra::drop_collection` soll dieses Handle-Verfahren nutzen und um diese drei Punkte ergänzen, statt Präfixe selbst zu berechnen.
- Test zuerst: WP-L0.1 wird grün. Zusätzlich: Tenant-Pfad (`tenant_index_persistence_reload.rs`, `invariant_tenant_isolation.rs`) bleibt grün.
- Abnahme: `cargo test -p contextra-mcp -p contextra-engine -p contextra-db`.

**WP-L0.3 Tautologische Tests ersetzen (G2)** (1 Sitzung)
- `drop_collection_proof_vs_files.rs`: `residue == true` ohne Purge muss `LsmMemtable` **und** eine ausdrückliche `physical_not_claimed`-Aussage erzwingen; separate Assertion „Daten nicht mehr per API lesbar". `test_contextra_forget_collection_with_proof` bekommt eine Nach-Löschprüfung.

**WP-L0.4 `excluded_scopes` füllen (G3) und Konkurrenz-Scan (G2)** (1 Sitzung)
- `drop_collection`: `excluded_scopes = [ConsolidatedAndDistilled, LlmParameterMemory]`; `deleted_keys` aus dem `delete_prefix`-Ergebnis bzw. Scan **innerhalb** derselben Transaktion; `collection_id` ohne stillen Fallback (Fehler statt `CollectionId::new(1)`).
- Test: Proof enthält beide Ausschlüsse; zwei konkurrierende Inserts während `drop` tauchen entweder im Nachweis auf oder werden abgelehnt.

**WP-L0.5 Dokumentation wahrheitsgemäß (G17)** (0,5 Sitzung)
- `contextra-mcp/src/lib.rs:16–17`, `sandbox.rs:45`: Tabelle korrigieren. Test: Doc-Truth-Scan aus WP-E der Spezifikation (verbotene Formulierungen) erweitern.

### Phase 1 — Eine Implementierung, geschlossene Konstruktoren

**WP-L1.1 Stub entfernen (G6, Entscheidung D1)** (2 Sitzungen)
- Dateien: `contextra-engine/Cargo.toml`, `contextra-engine/src/lib.rs`, `contextra-engine/tests/no_crypto_proof_stub.rs`, abhängige `cfg(feature = "encryption-at-rest")`-Stellen.
- Test zuerst: ein Test, der im Standard-Build `DeletionProof`-Typ-Identität prüft (`TypeId` gleich `contextra_crypto::deletion_proof::DeletionProof`) — heute rot.
- Abnahme: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p contextra-engine -p contextra-mcp`.

**WP-L1.2 Witness-Typen im unteren Crate (Korrektur Nr. 1/2)** (2 Sitzungen)
- Ort: `contextra-types` (oder `contextra-core`, je nach Ring-Prüfung `cargo xtask check-ring-layering`).
- Typ: `LayerWitness { layer: DeletionLayer, scope_hash: [u8;32], scanned_at_tx: TxId, evidence: LayerEvidence }`. `LayerEvidence` je Layer: `Memtable { entries_seen: u64 }`, `SsTables { files_scanned: u32, manifest_gen: u64 }`, `Hnsw { nodes_checked: u32, zeroed: bool, incoming_edges: u32 }`, `Wal { segments_scanned: u32, last_removed_seq: u64 }` usw.
- Genau **ein** Ausstellungs-Konstruktor `LayerWitness::issue(..)`, `#[doc(hidden)]`.
- Gate: `xtask`-Regel, die Aufrufe von `LayerWitness::issue` auf definierte Scanner-Module beschränkt (Muster wie bestehender Doktrin-Scan). Siehe Phase 6.

**WP-L1.3 Konstruktoren schließen (G7)** (1,5 Sitzungen)
- `LayerCleanupProof::new_after_verified_empty` und `verify_and_create` → `#[cfg(any(test, feature = "test-fixtures"))]`; neuer produktiver Weg `LayerCleanupProof::from_witness(LayerWitness) -> Result<Self>` (prüft `scope_hash` gegen `hash_deleted_keys_length_prefixed`, `scanned_at_tx >= deleted_after_tx`).
- Die sechs `DeletionProof::create*` werden `pub(crate)`; produktiver Weg nur `DeletionProofBuilder::finish()`, der **Ed25519** nutzt (ändert `deletion_proof_typestate.rs:299`).
- Fuzz-Targets (`deletion_proof_tamper.rs`, `deletion_proof_verify_external_fuzz.rs`), Benches und Examples auf `test-fixtures` umstellen.
- Abnahme: `cargo test -p contextra-crypto`, `cargo xtask doctrine-scan --crate contextra-crypto`.

### Phase 2 — Von außen prüfbar

**WP-L2.1 Schlüsselpaar und Signierer (G4, D2/D3)** (2 Sitzungen)
- `Contextra` hält einen `ProofSigner` (Ed25519). Laden/Erzeugen unter `~/.contextra/proof_ed25519.key` (0600), Public Key als `proof_ed25519.pub`. `drop_collection(name, scope)` ohne `proof_key`-Parameter.
- Tests: Schlüssel persistent über Neustart; falsche Dateirechte → Fehler (fail-closed).

**WP-L2.2 Richtlinien-Verifier (G5)** (2 Sitzungen)
- Neu: `DeletionProofPolicy { min_version, required_layers_for(scope), require_nonempty_keys, max_age, required_exclusions }` und `DeletionProof::verify_with_policy(&VerifyingKey, &DeletionProofPolicy) -> Result<PolicyReport>`.
- Test: Nachweis ohne Layer und ohne Schlüssel besteht `verify_external`, fällt aber bei Policy durch.

**WP-L2.3 Signaturversion v4 (G8)** (1,5 Sitzungen)
- Domänen-Tag (`b"contextra-deletion-proof-v4:"`), Längenpräfix je Feld, `scope_hash` und Witness-Evidence im signierten Payload. v3 bleibt verifizierbar.
- Property-Test: zwei verschiedene Proof-Strukturen erzeugen nie dasselbe Payload.

**WP-L2.4 `verify-proof` und Ablage (G9)** (2 Sitzungen)
- Unterbefehl `contextra-mcp verify-proof <datei> --pubkey <datei> [--policy …]` ohne Datenbankzugriff (Spezifikation WP-E).
- Ablage jedes Nachweises in einem append-only Verzeichnis `proofs/` (JSON) und Verkettung über `audit_chain_position`.
- Abnahme: E2E `forget → Proof-Datei → verify-proof` Exit 0; manipulierte Datei Exit ≠ 0.

### Phase 3 — Dokument-Löschung mit Nachweis (S1)

**WP-L3.0 Datenfluss-Inventur (G10)** (1 Sitzung)
- Tabelle in `docs/deletion/DATAFLOW.md`: jeder Schlüssel und jeder RAM-Zustand, den `insert` erzeugt (`user_key`, `doc_key`, `__txt:`-Terme/Postings, `__rel:`, Graph, Metadaten-Indizes, KV-Segmente, Embedding-Cache), und welcher Pfad ihn bei `delete` entfernt. Test: Marker-Roh-Scan über alle Schlüsselräume nach `delete`; jede Lücke wird als eigener Bug erfasst.

**WP-L3.1 `Collection::delete_with_proof(id, signer)`** (3 Sitzungen)
- Scope `DeletionScope::Document`; Zeugen: Memtable (aktiv **und** unveränderliche Memtables, siehe 4.1), HNSW (RAM-Vektor null, kein `doc_to_node`-Eintrag, keine Eingangskante), Text-Index, Graph. Nicht belegbare Layer erscheinen nicht.
- HNSW-Bausteine: Slot-Wiederverwendung mit Ghost-Kanten-Test (aus Gegenprüfung 3.2.3; am HEAD ungetestet), `commit()`-Operationsreihenfolge (`insert(A); delete(A)` in einer Transaktion), Behandlung des Falls „Snapshot-Pin verhindert Nullen" → Layer nicht beanspruchen.
- Abnahme: `cargo test -p contextra-engine --test delete_with_proof`.

**WP-L3.2 MCP-Tool** (1 Sitzung)
- `contextra_forget(id)` liefert `proof` mit `proof_scope: "document"`; Tool-Beschreibungen und `TOOL_SEMANTICS` aktualisieren. (Die Drei-Tool-Fassade `memory_forget` der Spezifikation baut darauf auf.)

**WP-L3.3 `purge_tenant` mit Nachweis (G13)** (2 Sitzungen)
- Rückgabe `DeletionProof` mit Scope `Tenant`; dieselbe Zeugenlogik; HNSW-/Graph-Handles des Tenants verwerfen.

### Phase 4 — Physische Layer (S2, S3)

**WP-L4.0 Voraussetzung klären (G16)** (0,5 Sitzung)
- `cargo test -p contextra-store campaign_deepresearch_merge_overlap` am HEAD ausführen. Rot → Merge-Fix aus dem Originalbericht zuerst. Compaction-Korrektheit ist Voraussetzung für jeden Purge.

**WP-L4.1 Purge-Operation im Store** (4 Sitzungen)
- `LsmStorage::purge_keys(keys|prefix) -> Result<PurgeReport>`: `force_flush`, gezielte volle Compaction aller SSTables, die die Schlüssel enthalten, Tombstone-GC (`is_full_compaction`), alte Dateien per `durable_remove`. Liefert `PurgeBlocked { reason }`, wenn aktive Snapshots/Checkpoints die Versionen halten (D6).
- Zeuge `SsTables`: nach dem Purge öffnet ein Scan **jede** laut Manifest lebende SSTable und prüft die Schlüssel (nicht den Cache).
- Memtable-Zeuge schließt unveränderliche Memtables ein.
- Tests: Property-Test mit absichtlich nicht gelöschten Daten (Scanner muss `Err`/`PurgeBlocked` liefern); Roh-Scan des Verzeichnisses findet den Marker nicht mehr.

**WP-L4.2 `durable_remove` überall + optionaler Scrub (G11, D5)** (1,5 Sitzungen)
- Alle `tokio::fs::remove_file`-Stellen in `compaction/engine.rs` und `lsm/ops/compaction.rs` über `contextra_durable_fs::durable_remove` (in `spawn_blocking`); Fehler propagieren statt nur loggen (`check-result-dropped-io` ist bereits ein Gate).
- Optional `contextra_durable_fs::scrub_and_remove` (Nullen, `fsync`, Entfernen), Konfiguration `secure_delete`.

**WP-L4.3 WAL-Zeuge (G11)** (3 Sitzungen)
- Nach Purge: WAL rotieren, versiegelte Segmente mit TxIds ≤ `deleted_after_tx` entfernen, restliche Segmente durch Entschlüsseln scannen. Manifest-`WalCheckpoint` fortschreiben (`lsm/ops/compaction.rs:144`).
- Vor Phase-4.3-Abschluss **G15 per Test klären**: abgeschnittener WAL-Tail mit Delete, danach Recovery mit vorhandenem SSTable. Rot → Truncation-Anker reparieren.

**WP-L4.4 Checkpoint-Klone (G11, D6)** (2 Sitzungen)
- Zeuge `SsTables` enthält `checkpoint_clones_present`; ist er wahr, wird `SsTableAllLevels` nicht beansprucht. Option: Checkpoints neu schreiben.

**WP-L4.5 HNSW-Kantenreparatur anbinden oder streichen (G11)** (2 Sitzungen)
- Entweder `remove_with_graph_repair` in den Löschpfad verdrahten (inkl. Behebung von `let _ = update_backlink` und des mmap-Nachbarn-`else`) und `GraphRepairAttestation` erzeugen, oder bewusst nur Tombstone + Rebuild nutzen und den Layer `HnswIndex` entsprechend eng definieren. Entscheidung per ADR.

### Phase 5 — Layer-Modell und Crypto-Shredding-Spur

**WP-L5.1 `DeletionLayer` erweitern (G12)** (1 Sitzung): `TextIndex`, `CheckpointClones`, `Relations` (serde-kompatibel anhängen, v4-Signatur).
**WP-L5.2 RevocationLog verdrahten oder streichen (G14)** (3 Sitzungen): Entscheidung, ob KV-Cache-Shredding im MCP produktiv ist. Wenn ja: `open_or_create_if_fresh` mit echtem `has_existing_keys`, `KvSegmentCipher::with_revocation_log`, `open_or_create` auf `test-fixtures` beschränken; Hotpath-/Marker-/Lock-Befunde der Gegenprüfung vorher gegen HEAD prüfen.

### Phase 6 — Durchsetzung

- **xtask-Regeln:** `LayerWitness::issue` nur in Scanner-Modulen; kein `new_after_verified_empty` außerhalb `test`/`fuzz`; keine `DeletionProof::create*` außerhalb des Builders.
- **Property-Tests/Fuzz:** je Scanner ein Test mit absichtlich verbleibenden Daten (muss immer fehlschlagen); neue Fuzz-Targets `verify_deletion_proof_policy`, `deletion_proof_v4_payload`.
- **Doc-Truth-Gate:** Test, der README/Docs/MCP-Beschreibungen auf Formulierungen wie „unwiderruflich gelöscht" oder „100 %" prüft.
- **CI:** `drop_collection`-/`forget`-E2E (Marker-Roh-Scan) als Merge-Pflicht.

---

## 7. Reihenfolge und Parallelisierung

| Stufe | Pakete | Voraussetzung | Ergebnis |
|---|---|---|---|
| 1 | L0.1 → L0.2 → L0.3, L0.4, L0.5 | – | Nachweis löscht und bescheinigt nur Wahres (S0) |
| 2 | L1.1 → L1.2 → L1.3 | Stufe 1 | Eine Implementierung, geschlossene Konstruktoren |
| 3 | L2.1 → L2.2, L2.3 → L2.4 | Stufe 2 | Extern prüfbarer Nachweis |
| 4 | L3.0 → L3.1 → L3.2, L3.3 | Stufe 3 | Dokument-/Tenant-Löschung mit Nachweis (S1) |
| 5 | L4.0 → L4.1 → L4.2 → L4.3, L4.4, L4.5 | Stufe 4 | Physische Layer (S2/S3) |
| 6 | L5.x, Phase 6 | fortlaufend | Vollständigkeit, Durchsetzung |

Parallel möglich: L4.2 (unabhängig), L5.2 (unabhängig von Phase 1–4), L4.3 und L4.5 nach L4.1. **Nicht parallel:** Phase 2 vor L1.1 (sonst zwei Implementierungen signieren) und L3.x vor L0.2 (sonst misst man auf falschem Schlüsselraum).

Grober Gesamtaufwand: Stufe 1–3 ca. 20 Sitzungen, Stufe 4 ca. 8, Stufe 5 ca. 15, Rest ca. 6.

---

## 8. Was nach Abschluss behauptbar ist

| Aussage | Belegbar nach |
|---|---|
| „`forget` entfernt die Daten aus allen über die API erreichbaren Schichten und stellt einen extern prüfbaren Nachweis aus" | Stufe 1–3 (S0) |
| „HNSW-Vektor genullt, keine Eingangskante, Text-/Graph-Einträge entfernt" | Stufe 4 (S1) |
| „Alle SSTables physisch bereinigt, keine Checkpoint-Klone halten die Bytes" | Stufe 5 (S2) |
| „Alle WAL-Segmente bereinigt" | L4.3 (S3) |
| „Daten sind physisch von der SSD verschwunden" | **nie** (Betriebsbedingung) |
| „Kopien in Backups, Swap, Core-Dumps sind entfernt" | **nie** |

---

## 9. Prüfbefehle für dich (ohne Rust-Kenntnisse ausführbar)

```bash
# G6: wird encryption-at-rest im ausgelieferten MCP-Binary aktiviert? (Zeile mit "encryption-at-rest" erwartet, falls ja)
cargo tree -p contextra-mcp -e features -i contextra-engine | grep -i encryption

# G1: Reproduktion (nach WP-L0.1)
cargo test -p contextra-mcp --test forget_collection_removes_data

# G16: LSM-Merge
cargo test -p contextra-store campaign_deepresearch_merge_overlap

# Konstruktoren nach WP-L1.3: darf außerhalb von Tests/Fuzz keinen Treffer liefern
grep -rn "new_after_verified_empty\|verify_and_create" crates --include=*.rs | grep -v "/tests/\|/fuzz/\|/examples/\|/benches/"
```

*Vorbehalt:* Jede Aussage zu Dateien, Zeilen und Verhalten stammt aus statischer Lektüre am HEAD `7e1d65b`. Jedes Arbeitspaket beginnt mit einer Prüfung seiner Annahmen; stimmt eine nicht, wird gestoppt und berichtet statt improvisiert.
