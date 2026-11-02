# `contextra-erasure` – Implementierungsspezifikation

|Feld|Wert|
|---|---|
|Zweck-Satz (D-97)|Löschkern: Saga, Gate, Journal, Tresor, Messung, signierter Bericht.|
|Ring|3|
|Erlaubte Workspace-Abhängigkeiten|`types`, `ports`, `crypto` (SPEC 5.3, AR-3, D-34); **keine** Ring-2-Crates (`store`, `text`, `vector`, `mvcc`), kein `engine`, kein `lab`/`ctx-verify` (AR-4)|
|Stufe / Gates|S0; G-D (Löschkern), G-F (Keyslots/Tresor), G0 (Messprotokoll mit `store`)|
|Arbeitspakete|WP-3.1 (Skelett, D-96), WP-4.1 (Tresor), WP-4.2 (Gate, Saga, Journal, Recovery), WP-4.3 (Messung, Bericht, Ledger)|
|SPEC-Anker|8.12, 11.1–11.8, 12, 13, D-01, D-13, D-33, D-35, D-36, D-37, D-39, D-48, D-55, D-57, D-58, D-61, D-72, D-73, D-74, D-78, D-79, D-83, D-85, D-96, F.3, F.9, F.10, Anhang D, 5.8, F-33|

## 1. Ist-Zustand (HEAD `0ede89a26`) [V]

|Punkt|Befund|
|---|---|
|Crate|`crates/contextra-erasure/` enthält **nur** `README.md` und `AGENTS.md`; keine `Cargo.toml`, kein `src/`; **kein Workspace-Mitglied** (`Cargo.toml` Wurzel, `members`/`default-members`). Die Kernmission ist im Code nicht vorhanden (F-33).|
|Löschtypen in `types`|`types/src/erasure.rs` (44 Z.) enthält `ScanMethod` (`Structural`, `NotMeasured(String)`, `FeatureDisabled`), `RequestContext` (`tenant_id`, `tx_id`), `SagaRecord` (`deleted_keys_hash`) – **nichts davon ist Soll** (8.1). Soll-Typen fehlen: `ErasureId`, `EraseTarget`, `LayerWitness`, `ResidualRisk`, `KeyCoverage`, `Scanner<…>` (siehe Datei 01).|
|Ports|`ErasureLayer`, `ReadGate`, `RecordSealer`, `FileKeyring`, `LedgerSink`, `CommitBarrier`, `PurgeBarrier` fehlen in `ports` (Datei 02, D-96).|
|Treffer in `crates/*/src`|`ErasedSet`, `ReadGate`, `ErasureLayer`, `LayerWitness`, `FileVault`: 0; `purge` 3, `saga` 6 (SPEC F-33)|
|Skelett-Texte|`README.md` und `AGENTS.md` des Crates sind abgeleitet und stimmen mit SPEC 5.1/5.3 überein (Zweck-Satz, Ring 3, Abhängigkeiten); `AGENTS.md` nennt D-01, D-13, D-33, D-35, D-37, D-48, D-55, D-57, D-58, D-72, D-74, D-79, D-85, D-96. Es fehlen dort D-39, D-61, D-73, D-78 und T-CRASH-*, T-RESTR-1, T-GATEGROUP-1, T-CLASS-1, T-JRN-3 (Aktualisierung nach D-98).|
|Budget|5.8: Erase-Produktcode (`erasure`, `store/src/erase`, `types/src/erasure`, `crypto/src/vault_cipher.rs`) **≤ 9.000 Zeilen**. Die Datei `crypto/src/vault_cipher.rs` existiert nicht und ist in Datei 06 nicht vorgesehen (siehe L-20).|

## 2. Verantwortung und Abgrenzung

`erasure` ist der **Löschkern** und die dritte Säule des Alleinstellungsmerkmals (D-01): (1) Gate und Purge-Koordination, (2) Schlüsselvernichtung über den Tresor, (3) Messung mit signiertem Bericht. Es kennt Speicher, Text und Vektor **nur** über `ErasureLayer` (P12); den Speicherort der Daten, Dateiformate von `store`/`text`/`vector` und Snapshots kennt es nicht. Es verwaltet:

|Bereich|Inhalt|
|---|---|
|Tresor|`FileVault`: Schlüsselslots `T`, `C`, `S`, `D`, `F` (F.3); implementiert `RecordSealer` (DEK, Slots anlegen) und `FileKeyring` (Dateischlüssel)|
|Gate|`ErasedSet` (Bereichsmenge), `gates.log`, Gruppen-Commit, `ReadGate`-Implementierung, `Restricted`|
|Saga|Zustandsmaschine, Journal, Scope-Dateien, Recovery, Purge-Fenster|
|Messung|`Scanner<Armed>`-Fluss, Positivkontrolle, `LayerWitness`, MK1/MK2/MK3-Feld|
|Nachweis|Bericht (kanonisches JSON, Ed25519), Ledger über `LedgerSink`, Restore-Guard|

Nicht in `erasure`: Dokumentlocks (`engine`, nur `EraseTarget::Document`, D-57), die Commit-Barriere selbst (Port `CommitBarrier`, Implementierung in `store`, L-7), die Wahl der Löschziele aus Nutzeranfragen (`engine`), Snapshots und MVCC (`mvcc`; Widerruf über einen Port, L-21), Dateisystem (Port `Vfs`), Zeit/Zufall/ID (Ports `Clock`, `Rng`, `IdGen`), Schlüsselmaterial-Zufall aus dem OS (`crypto::OsCsprng`).

## 3. Zielstruktur (Module) [E]

|Modul|Inhalt|SPEC|
|---|---|---|
|`lib.rs`|Crate-Kopf (Q-01), `ErasureCore`, `ErasureDeps`, `ErasureConfig`|D-91|
|`vault`|`FileVault`: Tabelle (F.3), Slot-I/O, Sondieren, Anlegen, Vernichten + Verifikation, Rehash, Open, Waisen|11.5, D-79, D-85|
|`vault::sealer`|`RecordSealer`-Impl: DEK-Ableitung (`crypto::derive_dek`), DEK-Cache nach `(tenant, coll, cluster)`, `evict`, `make_slots_durable`|D-79, I-9|
|`vault::keyring`|`FileKeyring`-Impl: `create`/`open`/`destroy` (maskierte Dateischlüssel)|D-85, I-9b, I-17|
|`gate`|`ErasedSet` (Intervallbaum, O(log n)), `GateLog` (Appendlog, F.9), `add_batch` (Gruppen-Commit), `ReadGate`-Impl, `lift_restriction`|11.3, D-57, D-58, D-78|
|`journal`|Appendlog (F.9), Reduktion nach `Completed`, Epochenrotation, Commitments|11.6, D-48|
|`scope`|`LayerScope`-Sammlung je Löschung, Persistenz `scope/<erasure_id>.scp`, Einbettung kleiner Umfänge|11.1|
|`saga`|Zustandsmaschine (`Requested … Completed`, `Pending`, `Refused`), Treiber, Retention-Prüfung, Idempotenz|11.2, D-15|
|`purge`|Purge-Fenster, Reihenfolge, Bündelung (`batch_window`), Plattenreserve, `PurgeCx`|11.4, D-39, D-65|
|`measure`|Positivkontrolle, `Scanner`-Fluss, `LayerWitness`-Erzeugung, MK1/MK2/MK3-Feld|12, D-72, I-19|
|`report`|Berichtsbau, Restrisiko-Auswertung, Kanonisierung (F.10), Signatur|11.7, D-13, Anhang D|
|`ledger`|`LedgerEntry`, Datei-`LedgerSink` (außerhalb des Datenverzeichnisses), Kette + MAC, Pruning|11.8, F.9|
|`restore`|Restore-Guard: Zeitregression, Wiederauftauchen, Replay, `ledger_floor`|11.8, D-74, I-20|
|`error`|`ErasureError` → `types::Error`|7.5|

`ErasureCore` ist die einzige Eintrittsstelle für `engine`.

## 4. Schnittstellen [E]

```rust
pub struct ErasureDeps {
    pub vfs: Arc<dyn Vfs>, pub clock: Arc<dyn Clock>, pub ids: Arc<dyn IdGen>, pub metrics: Arc<dyn Metrics>,
    pub layers: Vec<Arc<dyn ErasureLayer>>,            // doc_store, text, vector (von engine verdrahtet)
    pub commit_barrier: Arc<dyn CommitBarrier>,        // L-7
    pub purge_barrier: Arc<dyn PurgeBarrier>,          // L-7
    pub snapshots: Arc<dyn SnapshotRevoker>,           // L-21: engine → mvcc::revoke_covering
    pub probe: Arc<dyn RangeProbe>,                    // L-22: Restore-Guard (b)
    pub ledger: Arc<dyn LedgerSink>,                   // Standard: Datei außerhalb data_dir
    pub keys: KeyHierarchy,                            // aus crypto, nach Keyslot-Unwrap
}
pub struct ErasureConfig { erase_sla: Duration, batch_window: Duration, erase_return_timeout: Duration,
                           backup_retention: Duration, commit_epoch: Duration, min_free_bytes: u64, data_dir: PathBuf }

impl ErasureCore {
    /// Startsequenz 14.1 Nr. 6, 7, 9, 10: Gate-Log laden, Journal lesen, Waisen bereinigen, Ledger prüfen
    pub async fn open(cfg: ErasureConfig, deps: ErasureDeps) -> Result<(Self, OpenOutcome), Error>;
    pub fn read_gate(&self) -> Arc<dyn ReadGate>;
    pub fn sealer(&self) -> Arc<dyn RecordSealer>;
    pub fn keyring(&self) -> Arc<dyn FileKeyring>;
    pub async fn erase(&self, req: EraseRequest) -> Result<EraseReceipt, Error>;           // bis Gated (oder Pending nach erase_return_timeout)
    pub async fn status(&self, id: ErasureId) -> Result<ErasureStatus, Error>;
    pub async fn report(&self, id: ErasureId) -> Result<SignedReport, Error>;
    pub async fn lift_restriction(&self, id: ErasureId, att: RetentionAttestation) -> Result<(), Error>;
    pub async fn acknowledge_ledger_reset(&self, att: OperatorAttestation) -> Result<(), Error>;
    pub async fn run_purge_window(&self) -> Result<WindowOutcome, Error>;                  // Hintergrundtask-Aufruf (engine)
    pub fn health(&self) -> ErasureHealth;                                                 // offene Löschungen, Alter, SLA-Verletzungen, Barrierenwartezeit
}
```

Vault-Slot (F.3, 80 B): `slot_type(1) ‖ tag(16) ‖ payload(32) ‖ status(1) ‖ gen(u48) ‖ mac(24)`. Tabelle: Kopf (F.1, `kind = 10`) + Metablock (64 B) + `capacity` Slots; Position `BLAKE3(slot_type ‖ tag)[0..8] mod capacity`, lineares Sondieren, Suche endet bei `status = 0x00`; Grabstein `0x02` hält die Sondierkette.

Zustandsmaschine (11.2):

```
Requested ─▶ Gated ─▶ KeyDestroyed ─▶ Purging ─▶ Verifying ─▶ Completed
    └───────────┴────────────┴── Pending(reason) ◀── jeder Schritt idempotent;  Refused (terminal, vor Gate)
```

## 5. Implementierungsvorgaben

|ID|Vorgabe|Quelle|Test|
|---|---|---|---|
|ER-01|`erasure` hängt nur an `types`, `ports`, `crypto`. Kein `store`, `text`, `vector`, `mvcc`, `engine`; kein Code aus `lab`/`ctx-verify`. Ein Skelett (Kopf, leeres `lib.rs`) ist ab WP-3.1 Workspace-Mitglied; Löschports und -typen entstehen in `ports`/`types` **vor** jeder Implementierung in `store`/`text`/`vector`.|AR-3, AR-4, D-34, D-96|T-ARCH-3|
|ER-02|Tresor nach F.3: Slots à 80 B, Kapazität Zweierpotenz, Lastfaktor ≤ 0,7 (`(live + tombstones) / capacity`), lineares Sondieren, Slot-MAC `HMAC-SHA256(K_vm, Bytes[0..56))[0..24]` (`K_vm` aus `crypto::vault_mac`). Slot mit ungültigem MAC ⇒ `Corrupt`. Kein vollständiger RAM-Index; LRU-Cache für heiße Slots.|D-79, 11.5, F.3|T-VAULT-5|
|ER-03|Ein **einziger** Wert `VAULT_SLOT_BASE` (Offset des ersten Slots) und die Slotgröße stehen als Konstanten an einer Stelle und werden durch eine Golden-Datei festgelegt (L-4: `128 + 64 + i·80` passt nicht zu Kopf 64 B + Metablock 64 B).|F.3, L-4|T-FMT-1|
|ER-04|**Anlegen** (I-9): fehlende Ahnen-Slots `T`, `C`, `S` und `D` werden geschrieben und gruppenweise mit **einem** `sync_data` des Tresors dauerhaft, **bevor** WAL-Frames mit unter diesen DEKs versiegelten Records dauerhaft werden (`make_slots_durable`, vom Store-Leader aufgerufen, L-9). `F`-Slot dauerhaft vor dem ersten dauerhaften Block der Datei (I-9b). Es existiert nie ein Record ohne Schlüssel.|I-9, I-9b, 11.5|T-VAULT-3 (Crash-Matrix), T-FK-1|
|ER-05|**Vernichten** (D-33): alle 80 Bytes auf 0, danach Byte 49 auf `0x02` (Grabstein; `mac` bleibt 0), `sync_data`, Slot **lesen** und Nullen verifizieren (Bytes 0..49 und 50..80). Fehler ⇒ `Pending` mit Neuöffnen der Datei (D-55), **nie** Wiederholung auf demselben Handle.|11.5, F.3, D-55|T-VAULT-1, T-FSYNC-1|
|ER-06|Ziel → vernichteter Slot: `Document → D`, `Subject → S`, `Collection → C`, `Tenant → T`; **genau ein** Slot der höchsten Ebene; die Zeit bis `Gated` ist O(1) bezüglich der Zielgröße (4 `fsync`). Darunter liegende `D`-Slots entfernt der Purge (zielgrößenabhängig, nicht auf dem Pfad bis `Gated`). Dokumente ohne Betroffenen nutzen `salt_S = 0³²`.|D-79, P16|T-VAULT-2, T-VAULT-4, BM-ERASE-O1|
|ER-07|DEK-Cache nach `(tenant, coll, cluster)`; der Gate-Schritt evictet und zeroisiert betroffene Einträge **vor** der Rückgabe `Gated` (`RecordSealer::evict`). Ein Lesepfad nach `Gated` öffnet nie ein Envelope aus dem Cache.|D-79, T-GATE-1|T-GATE-1, T-VAULT-2|
|ER-08|Tresor-Wachstum: Rehash in neue Datei bei Lastfaktor > 0,7, atomarer Austausch (`write` → `sync_data` → `rename` → `sync_dir`); Grabsteine werden nicht übernommen. `max_docs_per_store = 5·10⁷` ⇒ `Limit::Capacity`. Open prüft Kopf und Stichproben, **kein Vollscan** (≤ 2 s bei 5·10⁷ Slots).|11.5, D-87|T-VAULT-5, BM-VAULT-OPEN|
|ER-09|Waisen (Recovery, D-83): `D`/`F`-Slots ohne Record bzw. Datei werden genullt; Datei ohne Manifest-Eintrag ⇒ `F`-Slot nullen, Datei entfernen; `F`-Slot ohne Datei ⇒ nullen. Absturz nach Manifest-Swap, vor `F`-Nullung oder `unlink` ⇒ Recovery nullt und entfernt.|D-83, D-85, FM-13, FM-14|T-CRASH-5, T-CRASH-6|
|ER-10|`FileKeyring::create`: zufälliger 32-Byte-Dateischlüssel (OS-CSPRNG), Slot `F` mit `payload = key ⊕ fkmask(file_id)`; `destroy`: nach dauerhaftem Manifest-Swap und **vor** `unlink` nullen (I-17), Fehlen prüfen. Beim SSTable-Rewrite bekommt die neue Datei einen neuen Schlüssel.|D-85, I-17|T-FK-1..3|
|ER-11|`ErasedSet`: sortierte Bereichsmenge (Intervallbaum), `check` O(log n), synchron, ohne I/O. Gate-Varianten `Erased` und `Restricted`. `variant_for_range` für Kompaktion: `Erased` verwerfbar, `Restricted` **nie** (I-18).|11.3, PO-07, I-18|T-GATE-1, T-COMPACT-RESTRICT-1|
|ER-12|`gates.log` (`kind = 7`, Appendlog F.9): `Add{gate_id, variant, lo(65), hi(65), gated_seq, erasure_id}` / `Remove{gate_id}`; AEAD, Hash-Kette, **ein** `fsync` je Gruppe. Kompaktion beim `Completed` der Löschung (Entfernen des Eintrags bei `Erased` nach Abschluss ist **nicht** erlaubt, solange das Ledger den Bereich nicht trägt – siehe L-24).|D-58, F.9|T-GATE-1, T-CRASH-4|
|ER-13|**Gate-Gruppen-Commit** (`add_batch`): Gleichzeitige Anfragen teilen sich **eine** exklusive Barriere-Phase (`CommitBarrier::exclusive`) und **einen** Gate-Log-`fsync`. `gated_seq` wird unter der Barriere bestimmt; alles `≤ gated_seq` gehört zum Umfang, Schreibzugriffe danach weist der Commit mit `Erased` ab (I-1).|D-57, D-78|T-GATEGROUP-1, T-CONC-1 (loom)|
|ER-14|Beim Start wird das Gate-Log **vor** dem Öffnen von Lesepfaden geladen (I-2). Fehlt der Journal-Eintrag zu einem Gate, gilt das Gate trotzdem (konservativ) und die Löschung wird vollendet; fehlt das Gate-Log, wird es aus dem Journal rekonstruiert (FM-9).|11.2, I-2, FM-9|T-CRASH-4|
|ER-15|`Restricted` verhindert Lesen, Suchen und Schreiben, vernichtet nichts, wird von der Kompaktion nie verworfen; Aufhebung nur durch `lift_restriction` mit Journal-Eintrag; ein `Erase` auf ein `Restricted`-Ziel ist zulässig und hebt die Restriktion durch Löschung auf (Journal-Eintrag).|D-15, 7.3, I-18|T-RESTR-1|
|ER-16|Normalpfad bis `Gated`: **4 `fsync`** (D-58): (1) Journal `Requested+Intent` mit eingebettetem Umfang, (2) Gate-Log, (3) Tresor-Slot nullen, (4) Journal `KeyDestroyed`; bei Last weniger je Anfrage (Gruppen). Ein Test zählt die `fsync`-Aufrufe über `FaultVfs`.|D-30, D-58|T-SLA-1, BM-ERASE-LAT|
|ER-17|Ziele: `resolve` je Schicht liefert den Umfang **vor** dem Gate; er wird verschlüsselt in `scope/<erasure_id>.scp` persistiert (`kind = 9`, zufälliger Dateischlüssel) oder bei kleinem Umfang in den Journal-Eintrag eingebettet (kein zusätzlicher `fsync`). `resolve` ist O(Schichtstruktur), nicht O(Zieldokumente).|11.1, F.1|T-CRASH-1, T-SUBJ-1|
|ER-18|Idempotenz (I-15): Jeder Saga-Schritt ist idempotent; gleiche `EraseRequest` liefert dieselbe `ErasureId`; Recovery setzt jede nicht `Completed`-Löschung am nächsten Schritt fort. Die Ableitung der `ErasureId` ist in L-23 zu klären (Zeitstempel in der Attestierung).|7.4, I-15|T-CRASH-1, T-CANCEL-1|
|ER-19|Retention (E-L7, E-L12): `ObligationActive`/`LegalClaimsPending` ⇒ `Refused(RetentionObligation)`, mit `EraseMode::Restrict` Gate-Variante `Restricted` ohne Vernichtung; `DataSubjectRequestedRestriction` nur mit `Restrict`; `OperatorOverride` läuft und steht im Bericht. Die Erklärung wird nicht inhaltlich geprüft, aber **signiert im Bericht** geführt.|7.3, D-15|T-API-2, T-RESTR-1|
|ER-20|**Purge-Fenster** (`batch_window`, Standard 60 s, ≤ `erase_sla / 2`; `Timing::Now` sofort; spätestens `erase_sla`): Reihenfolge a) Snapshots des Mandanten widerrufen (Port `SnapshotRevoker`, D-61), b) Textsegment-Rewrite (erstes Fenster, D-69), c) Memtable-Drain, Flush, WAL-Segmente bis Flush-Punkt entfernen (`store`-Schicht, D-73), d) SSTable-Rewrite, `F`-Slots der Altdateien nullen, `unlink`, e) Vektor-Kantenreparatur, f) `D`-Slots der Zielmenge entfernen, `F`-Waisen prüfen, g) Cache-Eviction. Die Schichtreihenfolge ist eine feste Tabelle `LayerId → Priorität` [E]; die Schichten kennen einander nicht.|11.2 Nr. 5, D-39, D-61|T-CRASH-1, BM-ERASE-BATCH|
|ER-21|Purge exklusiv gegen Kompaktion (`PurgeBarrier::purge`); die Kompaktion bricht kooperativ ab (I-16: keine Hintergrundoperation hält eine wartende Löschung > 1 s auf). Fehler beim Rewrite lässt die alte Datei unverändert.|D-78, I-16|T-GATEGROUP-1, BM-PURGE-WAIT|
|ER-22|Plattenreserve: Rewrite reserviert vorab; fehlt Platz ⇒ `Pending(DiskReserve)`, Gate bleibt, `erase` bis `Gated` braucht keinen nennenswerten Platz und läuft immer.|D-65, FM-3|T-CRASH-3, T-OPS-2|
|ER-23|`PendingReason ∈ {Io, SlaExceeded, LayerFailed(id), WitnessMissing, DiskReserve}`. `Pending` ist sichtbar, nie still. `erase` kehrt nach persistiertem Gate **und** vernichtetem Schlüssel zurück (`Gated`) oder nach `erase_return_timeout` mit `Pending`; `erase_sla`-Verletzungen erscheinen im Bericht und in `health()`.|7.4, D-30, FM-12|T-SLA-1|
|ER-24|Cancel-Safety (I-14): Jede mutierende Operation läuft in einer vom Aufrufer unabhängigen Task (`spawn`, Ergebnis über `oneshot`); Abbruch des Aufruferfutures ändert den Ausgang nicht. `Drop` blockiert nie.|I-14, 14.3|T-CANCEL-1|
|ER-25|**Messung:** `Scanner<Unarmed>` → Positivkontrolle (bekanntes Muster in einer Kontrolldatei im Scratch-Bereich; der Scanner findet es) → `ControlEvidence::observed` → `arm` → `Scanner<Armed>` → Scan → `finish(matches, …)`. `matches ≠ 0` ⇒ `Err(Residue)` ⇒ `Pending(WitnessMissing)`; **nie** `Completed` ohne gültigen Zeugen je Schicht (I-4, I-19, HR-10). `Scanner::arm` und `ControlEvidence::observed` werden **nur** unter `erasure/src/measure` aufgerufen.|D-72, I-19, T-RAW-2|T-RAW-2, T-ARCH (Datei 18), T-MUT-1|
|ER-26|Feldmessung: MK1 (kein Eintrag des Ziels in Strukturen des Datenverzeichnisses), MK2 (Rohscan inkl. `tmp/`, alte Segmente, WAL, Snapshots, `scope/`, Berichte auf Klartext-Needles des Betreibers und Muster), MK3-Feld (Schlüsselabwesenheit: Slot der höchsten Ebene genullt, DEK-Ableitung scheitert mit `Erased`, kein `F`-Slot der Zieldateien). Die Feldmessung ist Selbstattestierung (gleiche Decoder wie der Purge) und wird im Bericht so benannt; unabhängig ist sie nur mit `ctx-verify scan` oder im Lab.|12.1, D-35, D-72|T-RAW-1..6|
|ER-27|Bericht: kanonisches JSON (F.10: UTF-8 NFC, sortierte Objektschlüssel, keine Leerzeichen außerhalb von Strings, **keine Gleitkommazahlen**, Zeiten als Ganzzahl-Mikrosekunden UTC, Dauern als Dezimalstring); Ed25519 über `"ctx/report/v1" ‖ kanonische Bytes ohne signature`; Felder `signature` (Base64url), `pub_fingerprint`. Pflichtfelder 11.7 (u. a. `field_measurement` und `lab_evidence` **getrennt**, Klassen G/K/P je `reached/not_reached`, `key_coverage` je Schicht, `segments_rewritten`, `bytes_rewritten`, Restrisiken, Prozesshärtungsstatus); Pflichttexte 1–5 (Anhang D). Keine Inhalte, Schlüssel oder Klartext-Kennungen.|D-13, 11.7, F.10, P13|T-REPORT-1, T-PROOF-1, T-CLASS-1|
|ER-28|Restrisiken: `ResidualRisk` wird aus der statischen Bedingungstabelle (`types`, Datei 01) ausgewertet; die Liste ist vollständig gegenüber den zutreffenden Bedingungen (u. a. `FlashRemanence`, `FsJournalOrCow`, `WrappedKeyRemanence` immer, solange der Anker `FileVault` ist; `TextIndexUntilPurge`, `IndexTopologyInference` bis Purge bzw. Rebuild).|Anhang D, E-L15|T-REPORT-1, T-PROOF-1|
|ER-29|Journal (F.9): Zustände 1–8; Einträge nur `ErasureId`, Zustand, Zeit, Commitments (`BLAKE3-keyed(K_commit(epoch), subject ‖ tenant)`), Zähler, Zeugen-Hashes, Retention-Code, eingebetteter Umfang (nur Bereichs-Tags). Nach `Completed` Reduktion (Zwischenzustände entfernt, R13). Epochenrotation: nach `commit_epoch` neuer Epochenschlüssel, alte Commitments in Journal **und Bericht** werden entfernt; das Ledger folgt dem **nicht**.|11.6, D-48, R13|T-JRN-3, T-REPORT-1|
|ER-30|Ledger (`LedgerSink`, Datei **außerhalb** des Datenverzeichnisses): Eintrag `(commitment, range_set, gated_seq, completed_at)`, Hash-Kette + `ledger_mac`; Lebensdauer = `backup_retention`. Erkennung beim Öffnen: (1) Zeitregression `store.current_seq < max(gated_seq)`, (2) Wiederauftauchen (Datei im Zielbereich mit `min_seq ≤ gated_seq` und Record mit `seq ≤ gated_seq`, über `RangeProbe`, L-22); Records mit `seq > gated_seq` sind legitime Neuanlage. Treffer ⇒ Store **nicht freigegeben**, `Recovering`, automatische Wiederholung (`RestoreReplay`), Bericht mit Vermerk. `next_seq = max(current_seq, ledger_floor) + 1`. Fehlendes Ledger ⇒ `Degraded: LedgerMissing`, Bestätigung per `acknowledge_ledger_reset` (Journal-Eintrag); Einträge jenseits `backup_retention` werden entfernt.|11.8, D-74, I-20|T-RESTORE-1..3, BM-RESTORE-OPEN|
|ER-31|`fsync`-Fehler auf einer Datei des Löschkerns (Tresor, Journal, Gate-Log, Ledger, Scope) ⇒ Datei fatal, nie Wiederholung auf demselben Handle; Löschpfad: `Pending`, Datei neu öffnen und Zustand prüfen.|D-55, I-10|T-FSYNC-1|
|ER-32|Zeit nur über `Clock`, IDs über `IdGen`; Schlüssel-, Salt- und Nonce-Material aus dem OS-CSPRNG (`crypto::OsCsprng`); Dateien nur über `Vfs`. Kein `std::fs`, `tokio::fs`, `block_on`.|P7, D-04, D-03|T-LINT-2|
|ER-33|Fehler- und Lognachrichten: feste Texte, `tracing` mit festen Ereigniscodes; keine Inhalte, Kennungen, Schlüssel, Pfade mit Personenbezug (`tenant_tag_hex` im Pfad ist ein Tag, kein Klartext).|HR-4, I-11|T-LOG-1|
|ER-34|Budget: ≤ 9.000 Zeilen Erase-Produktcode über alle in 5.8 genannten Orte; Zeilenabdeckung ≥ 90 %, ≥ 95 % in der Entscheidungslogik (`saga`, `gate`, `measure`); **keine überlebenden Mutanten** in `erasure`, `classify`, Matcher, Gate, Tresor (G-D).|5.8, 15.1|`tokei`, `cargo-llvm-cov`, `cargo-mutants`, T-MUT-1|
|ER-35|`ErasureHealth` liefert: offene Löschungen, Alter der ältesten, SLA-Verletzungen, maximale Wartezeit auf die Purge-Barrier, `LedgerMissing`; `engine` baut daraus `health()` (D-64). Keine Metrik trägt ID-Labels.|D-64, PO-09|T-OPS-1|

## 6. Rückbau und Migration

|Position|Maßnahme|WP|Nachweis|
|---|---|---|---|
|`types/src/erasure.rs`: `ScanMethod`, `RequestContext`, `SagaRecord`|entfernen; Soll-Typen siehe Datei 01 (`ErasureId`, `EraseTarget`, `LayerWitness`, `ResidualRisk`, `KeyCoverage`, `Scanner<…>`)|WP-3.1|Grep = 0, T-ARCH-3|
|`Cargo.toml` Wurzel|`crates/contextra-erasure` in `members` und `default-members`; `Cargo.lock` im selben Commit erneuern (D-93)|WP-3.1|T-BUILD-1|
|Neue `crates/contextra-erasure/Cargo.toml`|`description` = Zweck-Satz, `[package.metadata.contextra] ring = "3"`, Abhängigkeiten `types`, `ports`, `crypto` (+ Kleinstabhängigkeiten: `zeroize`, `serde`, `serde_json`, `blake3`, `sha2` nur bei belegter Verwendung); keine Features außer Dev-Features|WP-3.1|T-ARCH-5, T-CFG-1|
|`README.md`/`AGENTS.md` des Crates|Ist-Zustand „leer“ entfernen, Testliste um die Lücken aus Abschnitt 1 ergänzen (D-98)|WP-3.1|T-DOC-1|
|`crypto/src/vault_cipher.rs` im Budget 5.8|Budgetort umbenennen auf `crypto/src/hierarchy` und `crypto/src/dek` (L-20)|WP-3.1 (SPEC-Änderung)|`tokei`|
|`store/src/erase` (`DocStoreLayer`)|bleibt in `store` (Datei 08); `erasure` kennt sie nur als `ErasureLayer`|WP-4.2|T-ARCH-3|

## 7. Arbeitspakete

1. **WP-3.1 (Skelett):** Crate-Kopf, leere Module, Workspace-Mitglied; Ports und Typen in `ports`/`types` (Dateien 01, 02); `testkit`-Conformance für `RecordSealer`, `FileKeyring`, `ReadGate`, `LedgerSink` (rot, mangels Implementierung).
2. **WP-4.1 (Tresor, 8 PW):** (rot) T-VAULT-1..5, T-FK-1..3, Crash-Matrix I-9/I-9b, Golden-Datei F.3 → (grün) `vault` (Tabelle, Slot-I/O, Rehash, Open), `sealer`, `keyring`. Voraussetzung: `crypto` (Datei 06) `hierarchy`, `dek`, `labels`.
3. **WP-4.2 (Gate, Saga, Journal, Recovery, 10 PW):** (rot) T-GATE-1, T-GATEGROUP-1 (mit `loom`), T-CRASH-1..6 (Abbruch nach jedem `write`/`fsync`/`rename`/`unlink`), T-RESTR-1, T-SLA-1 (4-`fsync`-Zählung) → (grün) `gate`, `journal`, `scope`, `saga`, `purge`. Gleichzeitig `store`: `DocStoreLayer`, Kompaktions-Kooperation, Barrieren-Implementierung (Datei 08).
4. **WP-4.3 (Messung, Bericht, Ledger, 6 PW):** (rot) T-RAW-1..6 (Feldteil), T-REPORT-1, T-PROOF-1 (gegen `ctx-verify`, Datei 17), T-CLASS-1, T-JRN-3, T-RESTORE-1..3 → (grün) `measure`, `report`, `ledger`, `restore`.
5. **G-D-Verbund:** Lab (Datei 16) führt MK1–MK3 und T-MUT-1 aus; Überlebende Mutanten blockieren.
6. **Koordination G0:** BM-ERASE-WA mit `store` (WP-3.4); der Löschkern liefert nur den Rahmen (Saga, Gate); die Dateimenge bestimmt `store`.

## 8. Tests und Benchmarks

|ID|Prüft|
|---|---|
|T-VAULT-1..5|Slot-Nullung (Nullen verifiziert, Fehlen geprüft, Grabstein), Hierarchie (S/C/T vernichten ⇒ DEKs darunter unableitbar), Reihenfolge I-9, Zeit bis `Gated` unabhängig von der Zielgröße, Rehash/Lastfaktor/Kapazität/Open ohne Vollscan|
|T-FK-1..3|Dateischlüssel: Anlage vor erstem Block, Vernichtung nach Swap vor `unlink`, freigegebene Blöcke nicht entschlüsselbar|
|T-GATE-1, T-GATEGROUP-1, T-RESTR-1, T-COMPACT-RESTRICT-1|Gate, Gruppen-Commit, Restriktion, Kompaktions-Kooperation|
|T-CRASH-1..6, T-CANCEL-1, T-FSYNC-1, T-WAL-2 (Anteil)|Saga-Abbrüche, Journal ohne Gate-Log, Waisen, `F`-Nullung nach Swap, Cancel, `fsync`-Fehler|
|T-RAW-1..6 (Feld), T-KEYCOV-1, T-CLASS-1|Zeugen je Schicht, `key_coverage`, Klassen G/K/P einzeln|
|T-REPORT-1, T-PROOF-1|Bericht ohne Inhalte, vollständige Restrisiken, `ctx-verify` akzeptiert/lehnt Manipulation ab|
|T-RESTORE-1..3, T-JRN-3|Zeitregression, Wiederauftauchen, Neuanlage mit `seq > gated_seq`, Ledger-Lebensdauer = `backup_retention`, Sequenz monoton, Journal-Rotation|
|T-SLA-1, T-OPS-1/2|`erase` bis `Gated` im Zeitrahmen D-30 (4 `fsync`), SLA-Verletzung, `health()`, Plattenreserve|
|T-MUT-1|Absichtlich gebrochene Gate-/Tresor-/Purge-Varianten werden vom Lab erkannt; `cargo-mutants` ohne Überlebende in `gate`, `vault`, `saga`, `measure`|
|T-TENANT-1, T-SUBJ-1, T-ISO-1 (Verbund)|Offboarding, Subject-Erase über Collections, Isolation|
|BM-ERASE-LAT, BM-ERASE-O1, BM-ERASE-BATCH, BM-PURGE-WAIT, BM-VAULT-OPEN, BM-RESTORE-OPEN|laut 15.2: p50 ≤ 10 ms (NVMe), Zeit(10⁶)/Zeit(10²) ≤ 1,2, Fensterdauer ≤ `erase_sla − batch_window`, Wartezeit p99 ≤ 1 s, Open ≤ 2 s, Ledger-Prüfung ≤ 1 s|
|Fuzz|Tresor-Slot-Parser, Journal-/Gate-Log-/Ledger-Replay (Truncation, Bitflips), Berichts-Kanonisierung|
|`loom`|Barrierenmodell (shared/exklusiv, `purge_waiting`), Gate-Gruppen|

## 9. Abnahme

Definition of Done (Index Abschnitt 8) und: `cargo tree -p contextra-erasure` zeigt nur `types`, `ports`, `crypto`; kein Treffer `std::fs|tokio::fs|block_on|unwrap|expect|panic!` in `src`; Grep `Scanner::arm`/`ControlEvidence::observed` nur unter `src/measure`; alle in Abschnitt 8 genannten T-/BM-IDs als `#[ctx_test]`/`#[ctx_bench]` vorhanden und grün (Referenzplattform für BM); Mutationsbericht ohne Überlebende; Zeilenbudget 5.8 eingehalten (`tokei` über die vier Orte).

## 10. Offene Punkte

- **L-4 (Index):** `VAULT_SLOT_BASE` (ER-03) – Golden-Datei entscheidet; zusätzlich Atomaritätsprobe (D-56) für 80-Byte-Teilschreibvorgänge erweitern (Datei 05).
- **L-5 (Index, aufgelöst):** Typzustandsmuster liegt in `types`; `erasure` darf `arm` nur unter `src/measure` aufrufen (Test in `tests/arch`).
- **L-7, L-9, L-11, L-12 (Index):** Barrieren-Ports, `make_slots_durable`, Schichtkonstruktion mit `Vfs`/`FileKeyring`, mehrere `WalFloor`: Umsetzung wie dort vorgeschlagen.
- **L-20 (neu): Budgetort `crypto/src/vault_cipher.rs`.** SPEC 5.8 zählt eine Datei, die weder existiert noch in der Ziel-Struktur der Krypto-Datei vorkommt. Vorschlag: Budgetorte auf `crypto/src/hierarchy` und `crypto/src/dek` ändern.
- **L-21 (neu): Snapshot-Widerruf.** 11.2 Nr. 5a verlangt, dass der Löschkern Snapshots des Mandanten widerruft (D-61); `erasure` darf `mvcc` nicht kennen (AR-3). Es fehlt ein Port. Vorschlag **[E]**: `SnapshotRevoker { fn revoke_covering(&self, tenant: TenantTag, range: &Matcher, gated_seq: Seq) -> RevokedCount }` in `ports`, implementiert in `engine` über `mvcc::SnapshotRegistry` (Datei 07, MV-03).
- **L-22 (neu): Port für den Restore-Guard (Wiederauftauchen).** 11.8 Nr. 2 braucht `min_seq` der Dateien im Zielbereich und die Existenz von Records mit `seq ≤ gated_seq`; beides kennt nur `store`. Vorschlag **[E]**: Port `RangeProbe { files_in_range(&KeyRange) -> Vec<FileSeqSpan>; has_record_at_or_below(&KeyRange, Seq) -> bool }`, implementiert in `store`.
- **L-23 (neu): Idempotenz der `ErasureId`.** 7.4 leitet die ID „aus Ziel-Commitment und Attestierung“ ab; die Attestierung trägt `attested_at` und `attested_by`, ein Wiederholungsaufruf liefert dann eine andere ID. Außerdem nutzt das Ziel-Commitment `K_commit(epoch)`, das rotiert (D-48), und eine Neuanlage nach `Completed` (erlaubt bei `seq > gated_seq`, D-74) braucht eine **neue** ID. Vorschlag **[E]**: `erasure_id = tag("ctx/erase-id/v1", target ‖ mode ‖ retention_code ‖ generation)` mit `K_idx` (neues Label in Anhang C), `generation` = Zahl abgeschlossener Löschungen desselben Ziels im Ledger.
- **L-24 (neu): Lebensdauer des Gates.** 11.2 Nr. 6 verlangt „Gate-Kompaktion“ bei `Completed`; nach der Kompaktion entfällt das `Erased`-Gate. Daten mit `seq > gated_seq` sind legitime Neuanlage (D-74), aber Altdaten **wiederauftauchender** Dateien (Restore) erkennt dann allein das Ledger. Offen, ob `Completed` die Bereichsgates entfernt (dann ist das Ledger die einzige Sperre gegen Restore bis zum Replay) oder ob `Erased`-Gates bis `backup_retention` bleiben. Vorschlag: Gate entfällt bei `Completed`; Restore-Guard (ER-30) ist die Absicherung; Entscheidung in Anhang H festhalten.
- **Berichtsinhalt `lab_evidence`:** Der Hash des Release-Evidenzbündels wird beim Build eingebettet (D-72); ein Bericht aus einem Build ohne Bündel führt `lab_evidence = null` und sagt das ausdrücklich (Anhang B: nur Zeilen mit EV4 sind ausgewiesen).
- **Zahl der `fsync` bei großem Umfang:** Für `Subject`/`Collection`/`Tenant` mit großem Umfang entsteht eine Scope-Datei (kein Einbetten); sie braucht einen `F`-Slot (`kind = 9`, `key_mode = 0`) und damit mehr als 4 `fsync` bis `Gated`. Die Obergrenze für „kleinen Umfang“ (Einbettung im Journal) ist festzulegen, sonst verfehlt D-30 für große Ziele (BM-ERASE-O1).
