# Schnittstellenspezifikation: Ablösung `contextra-store` durch `contextra-storage`

| | |
|---|---|
| Status | **Entwurf zur Freigabe (Gate M0).** Keine Implementierung in diesem Dokument. Keine Änderung an SPEC, Ports oder Kernformaten. |
| Gegenstand | Verträge zwischen den bestehenden Crates und `contextra-storage`: Abbildung jeder tatsächlich genutzten Speicher-Schnittstelle, Lücken, Entscheidungen, Migrationsfolge. |
| Normative Grundlage | `crates/contextra-storage/SPEC.md` (R9.1+Ö). Die hochgeladene Datei ist byte-identisch (243 483 Byte). Rangfolge bei Widerspruch: SPEC §15 vor §12.8 vor übrigen Abschnitten. |
| Ergänzt | `crates/contextra-storage/docs/24-cutover.md` (SP-CUTOVER). Abweichungen dazu stehen in §10. |
| Verifikation | Statische Analyse des Klons (HEAD `a0039db22`). **Kein Build:** Toolchain `1.89.0` ist im Container nicht installiert, `static.rust-lang.org` steht nicht auf der Netzwerk-Allowlist. Mit *belegt* ist eine gelesene Datei:Zeile gemeint, mit *vermutet* eine nicht gebaute Schlussfolgerung. |
| Sprache | Prosa deutsch, Bezeichner englisch. |

---

## 1. Kurzfassung

Die Ablösung ist **heute nicht umsetzbar**. `contextra-storage` besitzt keine öffentliche Lese- und Schreib-API: `Storage` stellt nur `open()` und `catalog()` bereit, `contextra-storage-ports/src/facade.rs` hat vier Zeilen, und das Bridge-Crate ist ein Platzhalter. Das Crate ist nicht Workspace-Mitglied und nicht in `capabilities.toml` registriert.

Die Kopplung ist dennoch klein genug, um sie sauber zu kapseln. Der Verbrauch läuft über **einen Port**, `contextra_ports::storage::StorageEngine` (`storage.rs:52`). Dieser Port wird von `LsmStorage` implementiert und von `contextra-db` über den Trait `Backends::Storage` konsumiert (`db/src/backends.rs:26-40`). Die Ablösung kann daher als Adapter erfolgen, ohne den Kern von `contextra-db` umzubauen.

Wichtig ist eine Korrektur gegenüber der Cargo-Sicht: **Der eigentliche Verbraucherkreis ist größer als die Store-Abhängigkeiten.** `checkpoint`, `text`, `graph`, `vector` und `cognition` hängen nicht von `contextra-store` ab, rufen aber über den Port `rollback_to_tx`, `pin_checkpoint` und verwandte Methoden auf. Sie sind von der Ablösung ebenso betroffen (§3).

Die Abbildung ist **nicht verlustfrei**. Vier Blocker (G-01 bis G-04) und fünf hohe Lücken (G-05 bis G-09) verändern Verhalten oder sind ohne Entscheidung nicht umsetzbar. Dazu gehören SSI-Read-Tracking, der WASM-Merge-Operator, Snapshot-Pins gegenüber der 10-Minuten-Lease und die Datenklassifikation (§6). Vor jeder Implementierung sind sechs Entscheidungen zu treffen (§8).

---

## 2. Ist-Zustand (belegt)

| Aspekt | Befund | Beleg |
|---|---|---|
| Store produktiv verdrahtet | LSM + WAL, asynchron, Tokio. Direkte Cargo-Abhängigkeit: `agent`, `bench`, `db`, `engine`. | Root `Cargo.toml:127`, Crate-Manifeste |
| Store-Symbolbezug (Quellcode) | `db`: 37 Dateien, davon 8 Produktionsdateien. `engine`: 8 Dateien, nur Tests. `bench`: 1 Datei. `agent`: **0 Symbole**, nur Dependency. | `grep contextra_store` |
| Transitive Store-Verbraucher | `contextra`, `contextra-cognition`, `contextra-mcp`, `contextra-py` (über `db`/`engine`/`agent`). | Cargo-Graph berechnet |
| Crate `contextra-storage` im Workspace | **Nicht Mitglied.** Root-`members` enthält nur `contextra-store`. Nicht in `capabilities.toml`. | Root `Cargo.toml:6-40`, `capabilities.toml` |
| Öffentliche Fassade | `Storage::open(Config)` und `Storage::catalog()`. Keine Lese-/Schreibmethode. | `crates/contextra-storage/crates/contextra-storage/src/lib.rs` |
| Fassadenvertrag Ports | `contextra-storage-ports/src/facade.rs`: 4 Zeilen, leer. | `facade.rs` |
| Schreibpfad intern | `committer::commit(frames: &[EncodedFrame], durability)` nimmt bereits kodierte Frames entgegen. Interne API. | `engine/committer.rs:114` |
| Werttypen Schreibvertrag | `Change { ops, guards, idem, durability, allow_mixed, caller }`, `Op::{Put, Delete, RangeDelete, …}`, `Guard::{Key, Range}`, `Durability::{Durable, Relaxed}`. | `storage-ports/src/change.rs` |
| Fehlervarianten | Vorhanden: `Indeterminate`, `Unsupported`, `Erased`, `Restricted`, `Conflict`, `Poisoned`, `DiskFull`, `KeyUnavailable`, `KeyLost`, `BelowFloor`, `Exhausted`, `IdemConflict`, `IdemCapacity`, `Lagged`. **Fehlend** gegenüber SPEC §3.5 (F-7): `SnapshotTooOld`, `ErasePending`, `BackupBeyondHorizon`; gegenüber §5.8/§6.1: `Backpressure`. | `storage-ports/src/error.rs` (grep) |
| Bridge | `contextra-storage-bridge/src/lib.rs` (2 Zeilen): „Platzhalter, P6–P8“. | `lib.rs` |
| Fs- und Keys-Crates | `contextra-storage-fs` und `contextra-keys-file` zusammen 11 Zeilen Rust. | `wc -l` |
| Cargo-Hygiene | Storage-Crate: `thiserror = "1"`; SPEC §3.7.1 verlangt Hauptversion 2 (Root hat `thiserror = "2"`). `crossbeam-skiplist = { workspace = true }`, aber **kein** Eintrag in Root-`workspace.dependencies` (grep: 0 Treffer). Das Crate liegt im Workspace-Verzeichnis, ist aber kein Mitglied. Cargo-Workspace-Regeln lassen einen Abbruch erwarten. | `Cargo.toml` Storage, Root |
| Phasenstand | P0 „In Arbeit“, P1 „In Arbeit“, P2 „In Arbeit“, P3–P4 „Teilweise begonnen“. Kein Gate bestanden. | `docs/PHASE-STATUS.md` |
| Referenzgenerator | `tools/reference/generator.py` verweist auf „SPEC §19.4“ und „§10.x“. Diese Nummerierung existiert in R9.1 nicht (§19 ist das Entscheidungsregister). Zielpfad `crates/contextra-storage/tests/format/golden/` existiert nicht. Das Golden-Layout (`chk`-Feld) passt nicht zu SPEC §5.2. | `generator.py` Kopf, Dateisystem |
| Konformanz-Harness | `contextra-testkit/src/conformance/storage.rs` und `in_memory_store.rs` existieren und implementieren Ports. Differentialtest gegen redb im Store (`tests/differential/redb_operation_sequence.rs`). SSI-Test `ssi_write_skew_integration` im Store. | Dateisystem, store `Cargo.toml` |

---

## 3. Schnittstellen-Inventar: Wer nutzt was

### 3.1 Port-Verbraucher (eigentlicher Kreis)

Gezählt werden Dateien mit Bezug auf `StorageEngine`, `KvBridgeStorage`, `StorageBackend` oder `CheckpointCoordinator`.

| Crate | Dateien mit Port-Bezug (Produktion) | Relevante Methoden |
|---|---:|---|
| `contextra-db` | 36 | alle `StorageEngine`-Methoden über `Backends::Storage` |
| `contextra-text` | 9 | `rollback_to_tx` (7 Dateien), Storage-Port |
| `contextra-cognition` | 10 | Storage-Port |
| `contextra-graph` | 5 | `rollback_to_tx` (5 Dateien) |
| `contextra-checkpoint` | 5 | `rollback_to_tx` (6 Dateien), `pin_checkpoint` |
| `contextra-agent` | 4 | `rollback_to_tx` (1 Datei), Checkpoints |
| `contextra-kvcache` | 3 | `KvBridgeStorage`/Storage-Port |
| `contextra-infer-candle` | 1 | Storage-Port |
| `contextra-vector` | – | `rollback_to_tx` (2 Dateien) |
| `contextra-erasure` | – | `StorageBackend` (Marker-Trait ohne Methoden, `ports/erasure.rs:269`) |

Konsequenz: Eine Migration, die nur `contextra-db` umstellt, lässt `text`, `graph`, `checkpoint` und `vector` mit einem Port-Verhalten zurück, das der Kern nicht mehr liefert. Die Adapterschicht muss den Port vollständig erfüllen, nicht nur für `db`.

### 3.2 Direkte Store-Symbole (Migrationsumfang im Code)

| Store-Schnittstelle | Verbraucher | Richtwert (grep, inkl. Tests) |
|---|---|---|
| `LsmStorage::new/open`, `LsmConfig` | `db` (Produktion: `lifecycle.rs`, `backends.rs`, `collections.rs`), Tests | >120 |
| `commit`, `put`, `get`, `delete`, `rollback` | `db` | commit 33, put 27, get 16, delete 11, rollback 11 |
| `scan_prefix`, `scan_prefix_at`, `get_at_seq` | `db` | 19 / 3 / 4 |
| `get_tracked`, `scan_prefix_tracked`, `supports_ssi_tracking` | `db` | 3 / – / 1 (SSI-Abfrage in 7 Dateien) |
| `put_if_absent`, `delete_prefix`, `delete_many` | `db` | 2 / 2 / 1 |
| `pin_checkpoint`, `unpin_checkpoint` | `db` (`collection/search/checkpoint.rs`) | 2 / 1 |
| `last_tx_id`, `last_seq_no`, `flush`, `close` | `db` | 2 / 2 / 1 / 1 |
| `SystemPressure`, `PressureLevel`, `pressure_receiver` | `db` (`collection/mod.rs`, `crud/internal.rs`), Test `backpressure_test.rs` | 2 Produktionsdateien |
| `DurabilityMode` (öffentlicher Re-Export) | `db` `lib.rs:61` | Breaking-Change-Pfad |
| `MergeOperator`, `WasmMergeOperator` (öffentlich) | `db` `merge_operator_adapter.rs`, `engine/tests/wasm_merge_operator.rs` | – |
| `TenantScopedStorage`, `TenantKeyCodec` (Präfix `t:{tenant}:`) | `db` `contextra_impl/collections.rs`, `lib.rs` | – |
| `sstable::{BlockCache, SstableBuilder, SstableReader}`, `compaction::{CompactionConfig, CompactionEngine}` | nur Tests (`engine`, `db/sprint2_acid_tests.rs`) | – |
| `register_observer`, `set_ssi_validator` | **kein externer Verbraucher** | → streichen |
| `set_clock`, `set_metrics_sink` | Tests, vereinzelt Produktion | prüfen |

### 3.3 Nicht Store-gebunden

`contextra_ports::checkpoint::CheckpointCoordinator` und das Crate `contextra-checkpoint` implementieren keine Store-Methoden für `LsmStorage` (grep: keine `impl … for LsmStorage` außerhalb des Store-Crates). Sie hängen am Port und sind über die Port-Adapter zu migrieren, nicht über den Store.

---

## 4. Abbildung `StorageEngine` → `contextra-storage`

Legende: ✅ verlustfrei · ⚠️ Verhalten ändert sich (siehe §6) · ⛔ ohne Entscheidung nicht abbildbar.

Alle Abbildungen gehen über eine **Transaktionspufferung in der Bridge**. Der Kern kennt keine Transaktions-IDs. Schreibvorgänge eines `TxId` werden gepuffert und bei `commit` als ein `Change` gesendet. `rollback` verwirft den Puffer, ohne E/A.

| # | Port-Methode | Ziel in `contextra-storage` | Bewertung |
|---|---|---|---|
| 1 | `put(tx, k, v)` | `Op::Put` im Tx-Puffer. Wert als `header = ∅`, `body = v`. Körper ≥ 1 KiB → Blob (`PutRef`). | ⚠️ Körper-Limit `max_body_bytes` = 4 MiB hart (§4.6). Größere Werte: Chunking oben (Cutover §3). |
| 2 | `put_if_absent(tx, k, v) → bool` | `Guard::Key{pre: Absent}` im Puffer. Rückgabe-`bool` nur über eine Vorprüfung auf dem Aufruferthread. | ⚠️ Der Rückgabewert ist nur als Vorprüfung exakt. Atomar entscheidet erst der Commit. |
| 3 | `put_batch(tx, …)` | Mehrere `Op::Put` im selben Puffer. | ⚠️ `max_change_ops` 4096, `max_change_bytes` 8 MiB. Eine Transaktion darf nicht gesplittet werden (Atomarität). Dadurch wird das Limit zur harten Nutzergrenze. |
| 4 | `delete(tx, k)` | `Op::Delete` (branch-lokaler Tombstone). | ✅ Logisch, entspricht bisherigem Verhalten. Keine Löschung im Sinne SPEC §9. |
| 5 | `delete_many(tx, keys) → u64` | `Op::Delete` je Schlüssel. | ⚠️ Der Zähler entfällt im Kern. Bridge muss vorher lesen. |
| 6 | `delete_prefix(tx, prefix) → u64` | `Op::RangeDelete` über `[prefix, succ(prefix))`. | ⚠️ Zähler entfällt. `succ(prefix)` ist undefiniert, wenn der Präfix nur aus `0xFF` besteht; Sonderfall nötig. |
| 7 | `commit(tx)` | `Change{durability: Durable}`, ein Gruppen-Fsync. Ergebnis `Receipt(stamp, durable)`. | ✅ atomar über Spaces. ⚠️ `Indeterminate` braucht einen `IdemToken`. Store hat keinen Token: die `TxId` muss als Idempotenzschlüssel dienen. |
| 8 | `rollback(tx)` | Puffer verwerfen. | ✅ reine Bridge-Operation. |
| 9 | `rollback_to_tx(tx)` | Kein Äquivalent. Store schreibt Zustand auf eine frühere Tx zurück. | ⛔ Destruktive Historienänderung ist im Kern nicht vorgesehen. Verbraucher: `checkpoint` (6), `text` (7), `graph` (5), `vector` (2), `agent` (1). Option: neuer Branch am Fork-Punkt, Umzeigen der Lesesicht (§8 SPEC). Destruktive Variante entfällt. |
| 10 | `flush()` | Kein öffentlicher Kern-Flush. Maintenance-Scheduler. | ⚠️ Semantik „synchron flushen“ entfällt. Dauerhaftigkeit wird pro Commit erreicht. |
| 11 | `stats() → StorageStats` | Kein Port. Metrikkatalog SPEC §16. | ⛔ Neu zu definieren (Bridge-Diagnosetyp). |
| 12 | `last_seq_no()` | `Stamp` (instanzlokal, < 2⁵⁶). | ⚠️ Zahlenraum und Lebensdauer ändern sich (SPEC §4.1: nur innerhalb `instance_id` eindeutig). |
| 13 | `last_tx_id()` | Kein Zähler im Kern. Bridge persistiert die höchste `TxId` in einem eigenen Systemzustand. | ⚠️ Persistenz neu zu entwerfen (siehe G-14). |
| 14 | `pin_checkpoint(seq)` | `Hold` (ttl ≤ 24 h, SPEC §9.6) je Cluster oder View-Lease (60 s, max. 10 min, §11.4). | ⛔ Store-Pins sind unbegrenzt; View-Leases nicht (G-04). |
| 15 | `unpin_checkpoint(seq)` | Lösen des Holds bzw. der Lease. | ⚠️ Abhängig von 14. |
| 16 | `get(k)` | `view_durable()` + Gate-Prüfung. | ⚠️ Gelöschte Cluster liefern `Erased`, nicht `None`. Store-Aufrufer behandeln `None` als „nicht vorhanden“. Das führt zu stillem Verhalten, wenn `Erased` nicht behandelt wird. SPEC §7 verbietet „leer“ ausdrücklich. |
| 17 | `get_at_seq(k, seq)` | `view_at(stamp)`. | ⚠️ `BelowFloor` ab `history_floor`. Historie wird im Kern nur über Holds und Floors gehalten. |
| 18 | `get_tracked(tx, k)` | `Guard::Key{pre: Version(stamp_gelesen)}` im Puffer. | ⚠️ OCC-Lesevalidierung statt SSI. Serialisierbar, aber anderes Abbruchmuster (G-02). |
| 19 | `get_at_seq_tracked` | wie 18 mit historischem Stamp. | ⚠️ Vorher prüfen, ob produktiv genutzt. |
| 20 | `scan_prefix_tracked(tx, p)` | `Guard::Range{start, end, since}` über den Präfixbereich. | ⚠️ Konservativ: jede Änderung im Bereich ist ein Konflikt, ohne Schlüsselvergleich (SPEC §6.3). Mehr Fehlabbrüche. |
| 21 | `supports_ssi_tracking() → bool` | `true`, sobald Read-Guards implementiert sind. | ⚠️ `db` verzweigt auf diesen Wert. Konsistent halten. |
| 22 | `scan_prefix(p) → Vec` | `view` + Iterator; Bridge materialisiert wie bisher. | ⚠️ Auch der Store materialisiert unbegrenzt. Unverändert problematisch (G-08). |
| 23 | `scan_prefix_bounded(p, limit, cursor)` | `page` mit Budget. Cursor: `ResumeToken`. | ⚠️ `ResumeToken` ist unter einem **Prozessschlüssel** versiegelt und über Neustarts ungültig (SPEC §7 Punkt 4). Bridge-Cursor braucht eigenes Format (G-08). |
| 24 | `scan_prefix_at(p, seq)` | `view_at(stamp)` + Iterator. | ⚠️ wie 17. |
| 25 | `scan(range)` / `scan_bounded` | Bereichslesen **innerhalb eines Clusters**. | ⛔ Clusterübergreifend nicht zulässig (SPEC §7 Punkt 9). Betrifft Export, Import und Hintergrundworker (G-07). |
| 26 | `scan_view()` (öffentlich am LsmStorage) | Kein Äquivalent. Interner Scanner-Vertrag des Stores. | ⛔ Nicht im Port, nur intern; entfällt mit dem Store. |

### 4.1 `KvBridgeStorage` (Tier-2-Entkopplung)

`contextra-ports/src/kv_bridge_port.rs` definiert mit `get`, `put` und `commit` die kleinste Oberfläche und ist laut Kommentar für die Entkopplung vorgesehen. **Empfehlung:** Die erste Stufe der Bridge implementiert `KvBridgeStorage` (3 Methoden). `StorageEngine` (26 Methoden) folgt erst, nachdem Konformanztests für die Kernoperationen grün sind.

---

## 5. Mandanten- und Löschdomäne

Der Store kennt Mandanten nur als Schlüsselpräfix (`t:{tenant_id}:…`, `tenant_codec.rs:37-60`). Der Kern kennt Cluster als **Löschdomäne, Schlüsseldomäne, Versionsdomäne und Platzierungseinheit** (SPEC §0, L1). Diese Begriffe überlappen nicht automatisch.

Vorgaben aus SPEC und Cutover:

- `DataClass::Personal` verlangt einen Sealed-Cluster und Durable zusätzlich `Seal::Cluster` (SPEC §4.7, Z-16).
- Legacy-Daten werden als `Plain` importiert, und Legacy-Spaces dürfen **keine personenbezogenen Daten** enthalten (Cutover §3).
- Ein Cluster folgt der Löschdomäne, nicht der Sitzung (SPEC §12.6, „Session ≠ Cluster“).

Daraus ergibt sich eine Entscheidung (E-1), die vor jeder Migration fallen muss:

| Option | Mapping Tenant → Cluster | Vorteil | Nachteil |
|---|---|---|---|
| **A** | Ein Tenant = ein Sealed-Cluster | einfach, wenige Cluster | Löschung einzelner Personen im Tenant nicht möglich |
| **B** (Empfehlung für personenbezogene Collections) | Wurzelcluster je Person/Fall; Zuordnung über `IdentityDirectoryOwner` (SPEC §12.7) | feingranulare Löschung, Eraser-Pfad vollständig | Directory-Owner, Registrierung, Pflege der Zuordnung |
| **C** | Nicht personenbezogene Daten als `Plain`-Cluster | billig | nur für Systemdaten, nie für personenbezogene Inhalte |

Folgen für die Schlüssel: Bei Option A/B entfällt das Präfix `t:{id}:` als Partitionsmerkmal, weil der Cluster die Partition ist. Die Bridge darf das Präfix beibehalten (Kompatibilität) oder entfernen; das ist eine Migrationsentscheidung, keine Kernfrage.

---

## 6. Semantische Lücken

Schweregrade: **B** = Blocker für die Produktionsfreigabe, **H** = hoch, **M** = mittel. Jede Lücke nennt die Auswirkung, die Optionen und eine Empfehlung.

**G-01 (B) — Keine öffentliche Schreib-/Lese-API.** SPEC §3.1 nennt `contextra-storage-api` als Composition Root (Feature `default-crypto`). Das Crate existiert nicht. *Empfehlung:* synchrone Fassade `contextra-storage-api` mit `open`, `apply(Change)`, `view`, `page`, `register_cluster`, `erase`. Ohne sie gibt es keinen Ablösepunkt.

**G-02 (B) — Snapshot-Isolation statt SSI.** Store unterstützt SSI-Read-Tracking (`get_tracked`, `scan_prefix_tracked`) und hat einen Write-Skew-Test (`ssi_write_skew_integration`). Der Kern verbietet SSI (NZ-3). *Option 1:* Bridge erzeugt Read-Guards (`Version`/`Range`) und validiert beim Commit. Das ergibt Serialisierbarkeit per OCC-Validierung; der Write-Skew-Test bleibt gültig. Die Abbruchrate und das Muster ändern sich. *Option 2:* SSI-Garantie aufgeben. *Empfehlung:* Option 1, Messung der Abbruchrate gegen den Store vor Gate M3.

**G-03 (B) — Benutzerdefinierter Merge-Operator.** Store: `MergeOperator` mit WASM-Fail-Safe, angewendet bei Compaction (`compaction/merge_operator.rs:7`). Öffentlich re-exportiert von `db` (`WasmMergeOperator`). SPEC NZ-4 schließt benutzerdefinierte Merge-Operatoren im Kern aus; Tag `6` ist reserviert. *Option 1:* Bridge führt Read-Modify-Write mit Guard und begrenzter Wiederholung aus. Kosten: Leseverstärkung bei Hot-Keys; die Fail-Safe-Semantik „beide Versionen behalten“ entfällt, weil es bei Lesezeit keine zwei Versionen zu behalten gibt. *Option 2:* NZ-4 ändern (SPEC-Änderung vor G-F, da Tag 6 Formatbestandteil ist). *Empfehlung:* Option 1; die WASM-Sandbox (`contextra-sandbox`) bleibt unverändert.

**G-04 (B) — Snapshot-Dauer.** Der Such-Checkpoint in `db` (`collection/search/checkpoint.rs`) pinnt eine Sequenznummer für die Dauer einer Suche. Store-Pins sind unbegrenzt. Der Kern begrenzt View-Leases auf 60 s, verlängerbar bis 10 min (§11.4); danach `SnapshotTooOld`. Holds sind technisch auf 24 h befristet und clusterbezogen. Ein globaler Pin über alle Cluster wäre O(Cluster) und damit ausgeschlossen. *Option 1:* Lease-Renewal in der Bridge, Abbruch nach 10 min mit typisiertem Fehler. *Option 2:* Hold je Cluster beim Öffnen, begrenzt auf 24 h. *Empfehlung:* Zuerst die reale Suchdauer unter Last messen. Bis dahin Option 1 mit dokumentiertem Zeitlimit.

**G-05 (H) — Klassifikation der Bestandsdaten.** Der Cutover sieht Legacy-Spaces als `Plain` vor. `db` kennt keine Datenklasse. Ohne Klassifikation kann niemand sagen, ob Bestandsdaten personenbezogen sind. *Empfehlung:* Vor dem Import eine Klassifikation je Collection erfassen (E-1). Nicht klassifizierbare Daten bleiben gesperrt, bis entschieden ist.

**G-06 (H) — Löschung ist neue Semantik.** `delete` im Store ist ein Tombstone, kein Erase. Der Kern bietet drei Stufen (Gate, Schlüssel, Purge). Die Entscheidung, welche `db`-Löschpfade auf `Eraser` umgestellt werden, liegt nicht bei der Migration, sondern bei Produkt und Recht (SPEC §0, rechtlicher Vorbehalt). *Empfehlung:* Entscheidung E-5 vor Gate M3. Bis dahin bleibt `delete` logisch und die Lücke wird im Receipt-Restrisiko geführt.

**G-07 (H) — Clusterübergreifende Scans.** `db` führt Export, Import und Hintergrundworker aus (Ablauf, Waisen, Decay), die über Mandanten iterieren. Der Kern verbietet clusterübergreifende Scans (SPEC §7 Punkt 9). *Option:* Iteration über `Inspector::clusters()` mit Capability, Ausführung je Cluster in der Bridge. Kosten O(Cluster) je Lauf; für Hintergrundworker ist das ein Lastthema. *Empfehlung:* Liste aller Worker erstellen und je Worker die Iterationsstrategie festlegen.

**G-08 (H) — Scan-Cursor und Materialisierung.** `scan_prefix` liefert `Vec` (unbegrenzt, Store wie Kern). `ResumeToken` ist prozessgebunden und nach Neustart ungültig. *Empfehlung:* Bridge-Cursor = letzter Schlüssel + gepinnter Stamp, versiegelt mit einem persistenten Schlüssel aus dem `KeyProvider`. Alternativ: Cursor nur innerhalb einer Sitzung zulassen und das dokumentieren.

**G-09 (H) — `DurabilityMode::WalNoHmac`.** Die Kette im Kern ist immer MAC-gesichert (SPEC §5.5). Es gibt kein Äquivalent. `db` re-exportiert `DurabilityMode` öffentlich (`lib.rs:61`). *Empfehlung:* Modus entfernen. `Full` → `Durable`, `MemoryOnly` → `Volatile`-Space (SPEC §4.5). Der öffentliche Re-Export wird als Breaking Change in `db` behandelt.

**G-10 (M) — Fehlerkatalog unvollständig.** Fehlend: `SnapshotTooOld`, `ErasePending`, `BackupBeyondHorizon`, `Backpressure(Retry)` (§3.5 F-7, §5.8, §6.1). `Erased` ersetzt `None` (siehe Zeile 16 in §4). *Empfehlung:* Varianten in `storage-ports/src/error.rs` nachziehen, bevor die Bridge sie benötigt. Das ist eine Port-Änderung ohne Formatwirkung.

**G-11 (M) — Druck-Signal fehlt.** `db` drosselt Schreibvorgänge über `pressure_receiver` (`PressureLevel::{Normal, Elevated, Critical}`). Im Kern gibt es kein Äquivalent; SPEC kennt `MemoryBudget` und `free_space`, aber kein Signal-Port. *Empfehlung:* Bridge leitet das Signal aus `MemoryBudget`-Belegung, `DiskFull`-Zustand und Reserve ab. Schwellwerte sind zu messen.

**G-12 (M) — Größenlimits gegen reale Daten.** Körper hart 4 MiB, Change 8 MiB, 4096 Ops, Schlüssel 1 KiB (Standard). Embeddings, Chunks und BM25-Postings der Engine können diese Grenzen erreichen. *Empfehlung:* Messung der Größenverteilung je Collection vor Gate M2. Überschreitungen werden auf Ebene oben gechunked (Cutover §3), nicht im Kern gelockert.

**G-13 (M) — Synchron gegen asynchron.** Der Kern ist synchron (SPEC §3.4), `StorageEngine` ist asynchron (BoxFuture). *Empfehlung:* Bridge führt synchrone Aufrufe über `spawn_blocking` aus. Dimensionierung des Blocking-Pools und Latenzmessung gehören in Gate M2.

**G-14 (M) — Bestandsmigration fehlt.** Cutover nennt `tools/migrate/`; das Verzeichnis existiert nicht. Der Store-Dateiformat-Leser lebt nur im alten Crate. Cutover erlaubt einen isolierten Migrationsadapter, der den Store liest. Die im Cutover genannte Funktion `durable_barrier` existiert nicht (grep: 0 Treffer in `contextra-storage`). `Origin::Migration` existiert (`change.rs`). *Empfehlung:* Migrationstool als eigenes Crate außerhalb des Kerns; Importpfad über `Origin::Migration`, `Relaxed` nur mit abschließendem Sync-Schritt, den das Cutover-Dokument korrekt beschreiben muss (G-16).

**G-15 (M) — Metriken und Beobachtung.** `with_metrics_sink`/`set_metrics_sink` werden in Tests und vereinzelt in Produktion genutzt. Der Kern definiert einen geschlossenen Metrikkatalog mit fester Label-Allowlist (SPEC §16). *Empfehlung:* Mapping auf den Kernkatalog; Store-Metriknamen ohne Entsprechung entfallen dokumentiert.

**G-16 (M) — Cutover-Dokument stimmt nicht mit dem Cargo-Graph überein.** Cutover §2 nennt `contextra-graph`, `contextra-cognition`, `contextra-checkpoint`, `contextra`, `contextra-py` als Verbraucher. Cargo-Graph: `graph`, `checkpoint` und `router` hängen nicht von `contextra-store` ab; `mcp` fehlt in der Liste; `text`, `vector` und `kvcache` fehlen, obwohl sie den Port nutzen (§3.1). *Empfehlung:* Cutover §2 auf die Inventarliste aus §3 dieses Dokuments aktualisieren.

**G-17 (M) — Testhooks.** Store-Tests nutzen `simulate_wal_append_failure_for_test` und `restore_wal_file_handle_for_test`. Der Kern hat `FaultFs`, `Sim` und `testing` (Feature `testing`). *Empfehlung:* Fehlerszenarien aus den Store-Tests als Fault-Injection-Szenarien im Kern neu spezifizieren, nicht portieren.

**G-18 (M) — Verbleibende Kern-Konformitätsfragen.** Der Referenzgenerator (siehe §2) verwendet eine nicht mehr gültige SPEC-Nummerierung und ein anderes Frame-Layout. Solange diese Goldens nicht gegen R9.1 abgeglichen sind, ist G-F nicht belegbar. *Empfehlung:* Generator und Goldens auf R9.1 neu erzeugen, bevor ein Formatvergleich mit dem Store gilt.

---

## 7. Zielarchitektur der Schnittstellen

```
contextra-db ──────────────┐
contextra-text/graph/      │  unverändert: Port StorageEngine / KvBridgeStorage
contextra-checkpoint/      │
contextra-vector/cognition ┘
          │
          ▼
contextra-storage-bridge        (async, tokio ausschließlich hier)
  · implementiert StorageEngine und KvBridgeStorage
  · Tx-Puffer, Read-Guards, Scan-Cursor, Pins/Leases, Druck-Signal
  · spawn_blocking für synchrone Aufrufe
          │
          ▼
contextra-storage-api           (NEU, synchron, Composition Root, G-01)
          │
          ▼
contextra-storage  ──  contextra-storage-ports (Traits, Werttypen)
          │
          ▼
contextra-storage-fs · contextra-keys-file · contextra-crypto-ref

tools/migrate/ (NEU, außerhalb des Kerns)  ──►  contextra-store (nur Lesezugriff, isoliert)
```

**Abhängigkeitsregeln**

1. Nur `contextra-storage-bridge` darf sowohl `contextra-storage-api` als auch `contextra-ports` einbinden. Das entspricht Cutover §2 und SPEC §3.7.6.
2. `contextra-db` und die übrigen Port-Verbraucher hängen nach der Migration nur an `contextra-ports` und der Bridge (Feature `storage-core`), nicht an `contextra-store`.
3. `contextra-store` wird nach Gate G-CUTOVER nicht mehr von Produktionscrates eingebunden. Der Migrationsadapter ist die einzige verbleibende Abhängigkeit, und sie ist auf `tools/migrate` begrenzt.
4. `contextra-store` bleibt bis zur Stilllegung unverändert (Cutover §1, Sofortmaßnahme 6: Tag `legacy-store-final`).
5. Der Kern bleibt frei von `tokio`, `parking_lot`, `serde*`, `blake3` (SPEC §3.7.1). Die Bridge ist die einzige Stelle, an der diese Abhängigkeiten im Ablöseweg erlaubt sind.

**Umsetzungsreihenfolge im Adapter:** Zuerst `KvBridgeStorage` (3 Methoden), dann `StorageEngine` für Kernoperationen (put, get, delete, commit, rollback, scan_prefix_bounded), dann Read-Guards (SSI-Ersatz), dann Pins und Leases, zuletzt Merge (RMW) und Export/Import.

---

## 8. Offene Entscheidungen

Jede Entscheidung braucht eine Freigabe, bevor das zugehörige Gate beginnt.

| ID | Frage | Optionen | Empfehlung | Entscheidet |
|---|---|---|---|---|
| **E-1** | Mandanten-/Clustermodell und Datenklassifikation | A (Tenant = Cluster), B (Person/Fall = Cluster mit Directory), C (Plain nur für Systemdaten) | B für personenbezogene Collections, A für Systemdaten | Produkt + Recht |
| **E-2** | Merge-Operator | Bridge-RMW mit Guard; NZ-4 ändern | Bridge-RMW | Architektur (Owner SPEC) |
| **E-3** | SSI-Ersatz | OCC-Read-Guards in Bridge; Garantie aufgeben | OCC-Read-Guards | Architektur |
| **E-4** | Snapshot-Dauer | Lease-Renewal mit 10-min-Limit; Hold je Cluster | Messung zuerst, dann Lease-Renewal | Architektur + Betrieb |
| **E-5** | Erase-Pfad in `db` | welche Löschoperationen auf `Eraser` umgestellt werden | Personal-Collections: Eraser; System: logisch | Produkt + Recht |
| **E-6** | `DurabilityMode::WalNoHmac` und öffentlicher Re-Export | entfernen (Breaking) oder Kompatibilitätstyp mit Deprecation | entfernen, Deprecation in einem Release | `db`-Owner |

---

## 9. Migrationsfolge und Gates

Die Gates folgen SPEC §14 und ergänzen Cutover §4. Keine Phase beginnt vor dem Gate der vorherigen.

| Gate | Inhalt | Abnahmekriterium | Voraussetzung |
|---|---|---|---|
| **M0** | Freigabe dieses Dokuments; Entscheidungen E-1…E-6 getroffen; Workspace-Aufnahme von `contextra-storage*`; `capabilities.toml`-Eintrag; Cargo-Korrekturen (§10) | `cargo check --workspace` grün; `deny`-Check grün; Reviewer-Freigabe | – |
| **M1** | Fehlerkatalog (G-10), Fassade `contextra-storage-api` (G-01), Bridge-Skelett mit `KvBridgeStorage` | Konformanzsuite `contextra-testkit/src/conformance/storage.rs` läuft gegen Bridge; Differentialtest gegen Store bei `put/get/delete/commit/rollback` | M0, P1-Stand des Kerns |
| **M2** | `StorageEngine` vollständig für Kernoperationen; Read-Guards (G-02); Messungen G-04, G-12 | Konformanz 100 % für unterstützte Operationen; Abbruchrate und Größenverteilung dokumentiert; **Basislinie des Stores zuerst messen** (`contextra-bench`, `competitive_kv`), weil SPEC keine gemessenen Zahlen enthält | M1 |
| **M3** | `db` mit Feature `storage-core` hinter Backends-Typ; `text`, `graph`, `checkpoint`, `vector` auf Bridge | `ssi_write_skew`-Äquivalent grün; `rollback_to_tx`-Ersatz (Branch) getestet; Such-Checkpoint unter Last | M2, E-3, E-4 |
| **M4** | Merge (G-03) und Export/Import (G-07) | WASM-Merge-Tests (`engine/tests/wasm_merge_operator.rs`) gegen RMW-Variante; Worker-Iteration je Cluster | M3, E-2 |
| **M5** | Migrationstool `tools/migrate` (G-14) mit Klassifikation (G-05) | Importierte Daten identisch (Hash je Collection); Restore-Test; keine Personal-Daten in Plain-Clustern | M4, E-1 |
| **M6** | Cutover je Datenbestand, ein schreibender Eigentümer je Bestand; kein Dual-Write (Cutover §2) | Rollback-Rehearsal; Forward-Export geprüft | M5 |
| **M7** | Stilllegung `contextra-store`: Tag `legacy-store-final`, Entfernen von `deny.toml`-Ausnahme `TRANSITION E29` | `cargo tree` ohne `contextra-store` in Produktionsgraph | M6 |

**Frist-Hinweis:** `deny.toml` führt `contextra-store` als Transitions-Ausnahme bis **2027-04-30** (Zeile 68). Läuft der Cutover später, muss die Ausnahme verlängert oder begründet werden. SPEC §3.7.6 verlangt Entfernen, nicht Verlängern.

---

## 10. Repo-Befunde und Sofortmaßnahmen

| Nr. | Befund | Maßnahme |
|---|---|---|
| R-1 | Root-`Cargo.toml`: `contextra-storage*` nicht in `members` | in M0 aufnehmen (oder bewusst exkludieren) |
| R-2 | `crossbeam-skiplist` fehlt in `workspace.dependencies` | Eintrag ergänzen (SPEC §3.7.1, Memtable-Index) |
| R-3 | Storage-Crate `thiserror = "1"`, Workspace `2` | auf `2` vereinheitlichen (SPEC SD-73) |
| R-4 | `capabilities.toml` kennt kein `contextra-storage*` | Einträge anlegen (Gruppe `storage`) |
| R-5 | `tools/reference/generator.py`: falsche SPEC-Verweise, nicht existierender Zielpfad | Generator auf R9.1 und realen Pfad anpassen (G-18) |
| R-6 | `contextra-agent`: Cargo-Dependency ohne Symbolbezug; Modul-Doku beschreibt `inner_storage()`, `PersistentCheckpointStore`, `CheckpointGuard` (`agent/src/lib.rs:47-52`), diese Typen existieren im Code nicht | Dependency entfernen oder Doku korrigieren |
| R-7 | `tools/generate_migration_edges.py`: Archivliste enthält `contextra-core` (nicht vorhanden), nicht `contextra-store` | Liste aktualisieren, Store als Archivkandidat aufnehmen |
| R-8 | `contextra-storage-ports/src/facade.rs`: leer | mit G-01 füllen oder entfernen |
| R-9 | Cutover §2 Verbraucherliste nicht konsistent (G-16) | aktualisieren |
| R-10 | Cutover §3 nennt `durable_barrier`, die Funktion existiert nicht | Abschnitt korrigieren oder API ergänzen |
| R-11 | `contextra-engine/docs/decoupling/REPORT.md` meldet einen bestehenden Feldfehler in `contextra-store` (`MemtableScanner`), der workspace-weite Builds stört | vor M1 beheben, da der Store bis M7 kompiliert sein muss |

---

## 11. Verifikationsstand und Grenzen

**Belegt durch Lesen des Quellcodes:** Alle Befunde in §2 bis §6 sind an Datei:Zeile gebunden. Verbraucherzahlen sind Textsuchen über `contextra_store` und die Port-Namen; sie können Kommentare und Tests enthalten, und die Angaben sind als Richtwerte zu lesen.

**Nicht belegt:**

- Kompilierbarkeit des Storage-Crates (kein Toolchain-Zugriff).
- Laufzeitverhalten, Latenzen, Abbruchraten der Read-Guards, Größenverteilung realer Daten.
- Vollständigkeit der Referenzvektoren (Anhang A der SPEC: laut Text „nicht vorhanden“).
- Inhalt der Konformanzsuite `contextra-testkit/src/conformance/storage.rs` (nur Existenz geprüft).

**Nicht Gegenstand dieses Dokuments:** Änderungen an Code, Ports oder der SPEC; die rechtliche Bewertung von Löschfristen und Aussagen wie „kryptografisch gelöscht“ (SPEC §0, Vorbehalt).

**Empfohlene erste Prüfung nach Freigabe:** `cargo check` auf dem Workspace mit Toolchain 1.89.0, um R-1 bis R-3 und R-11 zu bestätigen. Das ist die günstigste Messung, weil sie drei Annahmen dieses Dokuments direkt prüft.
