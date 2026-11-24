# SP-CATALOG — `catalog` (einzige Wahrheit über Dateien, Gruppen-Commit, Reservierungen)

| | |
|---|---|
| Modul | `contextra-storage/src/catalog/` |
| Besitzt | Catalog-Dateien (`CATALOG-<n>`, `CURRENT`), Edit-Record-Kodierung (alle Op-Tags §5.8), `CatalogState` (gefaltener Zustand), `catalog_edit` (Rang 3), Rollover, Reservierungen `reserve_stamps/ids` |
| Konsumiert | SP-CODEC (Primitive, Siegel/MAC), SP-ENV (`Fs`), SP-KEYS (Catalog-Schlüssel `KDF(RootKey,"ctxs catalog v1")`), SP-PORTS (Id-/Stamp-Typen) |
| Spec-Bezug | R8 §5.8, §6.4, §10.1 (R3), SD-01, SD-19, SD-43; INV-CAT-1/2, INV-ID-1/2, INV-ANCH-1 (Reihenfolge Edit → Anker) |
| Phase | P1 (provisorisch, Teilmenge Ops), P2 (vollständig), G-F |

## 1 Auftrag und Abgrenzung
Der Catalog ist die einzige Wahrheit darüber, welche Dateien, Cluster, Spaces, Restrictions, Böden und offenen Löschungen **leben**. Er **interpretiert** keine Ops semantisch über Faltung hinaus: Bedeutung und Reihenfolgeregeln liegen bei den Absendern (Flush, Purge, Erase). **Darf nicht:** Nutzdaten, Branch-Namen (§8.2), Schlüsselmaterial speichern; Dateien löschen (Unlink macht MAINT/Graveyard); Stamps vergeben.

## 2 Schnittstelle
```rust
pub struct Catalog { .. }                                   // Eigentümer der Dateien; ein Exemplar je Instanz
impl Catalog {
    pub fn open(fs:&dyn Fs, keys:&CatalogKeys, mode:OpenMode) -> Result<(Catalog, CatalogState), CatalogError>; // R3: bis letzten gültigen Record
    pub fn edit(&self, e: CatalogEdit) -> Result<EditAck, CatalogError>;  // Rang 3; Gruppen-Commit; kehrt erst nach sync_data zurück
    pub fn state(&self) -> Arc<CatalogState>;                             // unveränderlicher Schnappschuss des gefalteten Zustands
    pub fn epoch(&self) -> u64;                                           // catalog_epoch (monoton je Edit-Gruppe)
    pub fn rollover_if_needed(&self) -> Result<(), CatalogError>;         // Wartung; atomarer CURRENT-Wechsel
}
pub struct CatalogEdit { pub ops: Vec<CatalogOp> }          // EIN Edit = EIN Durabilitätspunkt (P-2)
pub enum CatalogOp { SpaceAdd{..}, ClusterRegister{..}, FilesAdd{..}, FilesRemove{..}, ReplayFloor(RedoPos), .. /* §5.8, 1:1 */ }
pub struct CatalogState { /* spaces, clusters (Zustand,Pack,Links,Restrictions), files je Space/Level, persistent graveyard bis unlink_ack, blobs, floors, pending erases, reserves, segments, consumers, idem_trim, anchor_mark */ }
```
`CatalogState` ist **copy-on-write** (persistente Maps); `Arc`-Austausch unter Rang 4 macht SP-MEMVER, nicht dieses Modul.

P1 implementiert zunächst nur `space_add`, `cluster_register`, `files_add`, `files_remove`, `replay_floor`, `redo_mark`, `reserve_stamps`, `reserve_ids` und `reserve_space_ids`. Die lokalen Tagwerte 1–8 sind **provisorisch**; alle übrigen Ops bleiben bis zu ihrem Besitzer-SP nicht verfügbar und unbekannte Tags werden abgewiesen. Kein Aufrufer darf die Teilmenge als vollständigen Catalog-Codec behandeln.

## 3 Regeln
1. **Dateiaufbau:** Snapshot-Record + Edit-Records; nur in vorallokierten 1-MiB-Blöcken (AN-10); Record = `len u32 | seq u64 | prev_crc | ops | crc32c | (Authenticated+: chain_mac)`; Kette über `seq`. Torn Tail verwirft den letzten unvollständigen Record **nur**, wenn danach nichts Gültiges folgt (analog INV-LOG-8; Mid-Catalog-Fehler ⇒ `Corrupt{CatalogMid}`).
2. **Gruppen-Commit (INV-CAT-2):** `edit` reiht in Wartekette; Führer fasst alle Wartenden, **ein** `write` (ausgerichtet) + **ein** `sync_data`; Wartende werden erst danach entlassen und `state()` erst danach aktualisiert. Sync-Fehler ⇒ `Poisoned` (F-3), Wartende erhalten `Indeterminate`.
3. **Faltung:** `CatalogState::apply(op)` ist rein, deterministisch, idempotent je Op-Identität. Unbekannter Tag ⇒ `Unsupported` (Bereich `0xF000…` reserviert). Ein Edit mit `files_remove` einer nicht lebenden Datei ⇒ `Corrupt` (INV-CAT-1).
4. **Kopplung (P-2):** Zustandsgekoppelte Änderungen (`files_add+files_remove+blob_replace+cluster_state+erased_ranges_add+erase_stage+anchor_mark`) sind **ein** `CatalogEdit`; der Encoder lehnt Edits ab, deren Op-Menge nicht zu einem benannten Muster `FlushEdit`, `CompactEdit`, `PurgeBatchEdit`, `UnlinkAckEdit`, `PurgeCompleteEdit`, `RebucketEdit` oder `SpaceDropEdit` passt (Typ-Builder je Muster, kein freier Op-Haufen in Produktionscode). `files_remove` legt im selben Edit einen Graveyard-Eintrag an; `UnlinkAckEdit` folgt erst nach `unlink + sync_dir`. `PurgeBatchEdit` installiert genau eine Datei-/Inkarnationsänderung; nur `PurgeCompleteEdit` darf den endgültigen Stage-Wechsel/Done festschreiben und verlangt einen leeren Ticket-Graveyard.
5. **Idempotenz:** Ergebnisse liegen als interne `SYS_IDEM`-Records atomar im selben Redo-Commit wie der Nutzer-Change; der Catalog hält nur `idem_trim`-Wasserstand/Epoch. Dadurch entsteht kein zweiter Fsync pro Commit und `replay_floor` kann Idem-Records nicht vor Ablauf/Trim verwerfen. Request-Fingerprint-Kollision bei abweichendem Request ergibt `IdemConflict`.
6. **Rollover:** bei `max(8 MiB, Größe letzter Snapshots)`: neuer Snapshot in `CATALOG-<n+1>` (`sync_all`), `CURRENT` per atomarem `rename` + `sync_dir`; altes `CATALOG-<n>` wird Graveyard (nicht gelöscht, bis `CURRENT` durabel). Crash zwischen Schritten: `CURRENT` zeigt auf gültigen Stand.
7. **Reservierungen:** `reserve_stamps` (2²⁰), `reserve_ids` (2¹⁶) und `reserve_space_ids` (2¹⁶) heben Hochwassermarken im Catalog an; `stamp_limit`/`id_limit`/`space_id_limit` sind Teil von `CatalogState`. `SpaceId` ist ein monotoner `u64`-Zähler; angeforderte Blöcke werden vor jeder Vergabe durable reserviert, ungenutzte IDs nach Crash übersprungen und nie wiederverwendet. Wartung hebt bei < 50 % Rest an (SP-MAINT `reserve`); Committer-Fallback (einzige Rang-1+3-Doppelhaltung) ruft `edit` mit genau diesem Op.
8. **Anker-Ordnung:** `edit` liefert `EditAck{epoch}`; Anker-Fortschreibung (SP-LEDGER/ERASE) darf frühestens danach erfolgen. Dieses Modul ruft den Anker nie.
9. **Klartext-Exposition:** Cluster-IDs, Zustände, Dateinamen/-IDs, Größen, Stamp-Bereiche stehen im Klartext (kettengeschützt); alles andere ist leer. Verzeichnis `CATALOG-*` ist `Perm::Private`.
10. **Speicher (E-38):** Snapshot-Parse streaming (kein Voll-Puffer); `CatalogState` ≤ 48 B/Cluster + Dateistruktur; `ClusterRegister`-Massenpfad (Ingest) bündelt bis 4096 Ops je Edit.

## 4 Fehlerverhalten
F-3 bei Write/Sync; F-4 `Corrupt{CatalogMid|CatalogChain}`; F-5 `Unsupported`; F-2 nur beim Vorallokieren des Folgeblocks (`DiskFull`, kein Poison). Fehler tragen Datei-ID, Record-`seq`, nie Inhalte.

## 5 Invarianten / Tests
INV-CAT-1 (Property: Datei lebendig ⇔ `files_add` ohne `files_remove`, Modell `BTreeSet`), INV-CAT-2 (Crash-Sweep: Edit sichtbar ⇒ durabel), INV-ID-1/2; Crash-Sweep über jeden `FaultFs`-Index bei Edit-Lawine + Rollover; Torn-Tail-Tabelle; Fuzz `decode_catalog_record`; Faltungsäquivalenz: `fold(snapshot(S)+edits) == fold(edits)`; goldene Vektoren (Edit-Kette mit Reservierung, Gruppen-Commit, `anchor_mark`) nach G-F.

## 6 Lücken / Errata
- **E-38** Speicherbudget je Cluster (siehe INDEX).
- **E-46 (Edit-Muster-Typen):** Freie `CatalogOp`-Listen sind Fehlerquelle (vergessenes `blob_live_delta`). Beschluss: Builder je Muster (Regel 4); `CatalogEdit::raw` nur unter `testing`.
- **E-47 (Fold-Kosten bei Start):** 10⁶ Edits ⇒ Snapshot häufiger; Rollover-Schwelle wird **zusätzlich** nach Recordanzahl (≥ 2²⁰) ausgelöst.

## 7 Arbeitspakete
| ID | Titel | PW | Abhängig von | Phase | Abnahme |
|---|---|---|---|---|---|
| WP-06.1 | Op-Codec, Record-Kette, Torn-Tail-Reader (Teilmenge P1) | 2.0 | WP-02.1, WP-03.1 | P1 | Rundreise, Fuzz |
| WP-06.2 | Snapshot, Rollover, `CURRENT`-Rename, Open-Pfad | 2.0 | WP-06.1 | P2 | Crash-Sweep Rollover |
| WP-06.3 | Gruppen-Commit `edit` (Rang 3), Poison-Pfad | 1.5 | WP-06.1, WP-03.4 | P2 | INV-CAT-2, loom |
| WP-06.4 | `CatalogState`-Faltung (persistente Maps), Edit-Muster-Builder | 1.5 | WP-06.1 | P2 | INV-CAT-1 Property |
| WP-06.5 | Reservierungen Stamp/Id, `anchor_mark`, `idem_trim` | 1.0 | WP-06.3 | P3 | Reserve-Test, Restart |

## 8 Abnahme
Crash-Sweep ohne Verstoß; Op-Vollständigkeit gegen §5.8 per Tag-Tabellentest; goldene Vektoren nach G-F; kein `indexing`.

## R9.1-Synchronisierung

Purge bleibt pro Datei/Inkarnation nachvollziehbar: `PurgeBatchEdit` installiert jede Ersetzung, `files_remove` pinnt persistent im Graveyard, `UnlinkAckEdit` folgt erst nach Reader-Drain, Unlink und `sync_dir`; allein `PurgeCompleteEdit` darf den Ticket-Abschluss setzen. Open/Recovery heißt OP1…OP10; keine Catalog-Operation hält Rang 1.
