# Löschkette – Lückenlosigkeit: Audit gegen HEAD, Kernel-Vertrag Rev. 2, Datenort-Matrix

*Stand: 28.11.2026 · Basis: `tfufuz1/contextra` `main`, HEAD `e12873de6` · Ergänzt `00-INDEX.md`, korrigiert 02, 03, 05, 09, 10, 11 (jeweils Abschnitt „Revision 2“) · Neu: Befunde **K-45 … K-60**, Nachträge **A-4 … A-8**, Crash-Punkte **CP-19 … CP-30***

> **Verifikationsgrenze.** Statische Lesung des Quelltextes, keine Rust-Toolchain, nichts kompiliert oder ausgeführt. **✔** = Datei/Zeile gelesen. **[?]** = Annahme mit Prüfauftrag (Test-ID genannt). Jeder Befund nennt den Beleg; Tests, die den Befund bestätigen sollen, **müssen vor der Korrektur rot sein** (Rot-Grün).

## 1. Zweck und Vorgehen

Die Spezifikationen 00–13 wurden gegen den Code am HEAD abgeglichen. Seit dem Doku-Stand (`e23bcdd`) wurden 195 Dateien geändert: Store-Kernel, Gate, Journal, Ledger, Pending-Delete, Text- und HNSW-Layer, Engine-Treiber/-Worker/-API, MCP-Werkzeuge, Testkit, CI-Gates sind **vorhanden**. Die Lücken liegen deshalb weniger in fehlenden Dateien als in **Verträgen zwischen Komponenten**, die einzeln plausibel sind und zusammen nicht tragen. Dieses Dokument

1. listet die Befunde mit Beleg und normativer Korrektur (§2),
2. fixiert den korrigierten Kernel-/Treiber-Vertrag (§3),
3. legt eine **vollständige Datenort-Matrix** fest, gegen die „lückenlos“ prüfbar ist (§4),
4. ergänzt Crash-Matrix und Tests (§5, §6),
5. ordnet Arbeitspakete (§7).

## 2. Befunde (verbindlich)

Schweregrad: **S1** = Löschzusage kann falsch sein oder Job hängt; **S2** = Härtung/Beobachtbarkeit; **S3** = Hygiene.

### 2.1 Zustandsautomat und Treiber

| ID | Sev. | Befund (Beleg) | Korrektur |
| --- | --- | --- | --- |
| **K-45** | S1 | **Reihenfolgenwiderspruch.** Der Treiber ruft `layer.purge` und danach `kernel.record_layer` vor `purge_store` auf (`engine/erasure/driver.rs`, `purge_phase`). `record_layer` verlangt Zustand `Purging` (`store/erasure/mod.rs:1253`), den nur `purge_store` setzt (`mod.rs:1165-1182`). Der erste Layer-Fortschritt endet in `InvalidTransition`. 03 §4.2 und Index §4 widersprechen sich damit selbst. | **A-4** `ErasureKernel::begin_purge` (§3.2). Der Treiber ruft es vor dem ersten Layer. |
| **K-46** | S1 | **Fehlerpfad bricht.** `fail()` lehnt `Failed→Failed` ab (`types/erasure.rs:476-479` erlaubt aus `Failed` nur `Purging`, `Verifying`, `Quarantined`). Der Treiber verwirft das Ergebnis (`let _ = self.kernel.fail(…)` in `on_failure` und `accept_and_gate`) und meldet trotzdem `Retry` bzw. `Quarantined`. Die Schwelle `10` ist im Kernel hart codiert (`mod.rs:1622`), der Treiber nutzt `cfg.quarantine_after_attempts` und mappt vorab auf `Permanent`. `fail(layer = None)` mit ≥ 10 Versuchen quarantäniert nie; der Zustand bleibt `Failed`, `drive_job` prüft nur `Quarantined` und fährt den Job jede Runde erneut. | §3.3: ein Schwellenwert, im Kernel; `fail`-Ergebnis wird ausgewertet; `DriveOutcome::Quarantined` nur, wenn der Kernel `Quarantined` meldet. |
| **K-47** | S1 | **Verifikations-Reste werden nie nachgearbeitet.** `verify_and_seal` meldet bei `remaining > 0` `Retryable{layer}`. `to_job_view` zählt einen Layer als erledigt, sobald sein Cursor `None` ist (`mod.rs:239-247`); `fail()` setzt den Cursor nicht zurück (`mod.rs:1605-1670`). `purge_phase` überspringt den Layer (`layers_done.contains`). Ergebnis: dieselbe Verifikation scheitert bis zur Quarantäne. | §3.4: `Failed{layer: Some(l)}` löscht `layer_cursor[l]` (live und beim Replay), kein neues Record-Tag. |
| **K-48** | S1 | **Layer sind nicht verdrahtet.** `register_erasure_layer` hat keinen Produktivaufrufer (`engine/erasure/api.rs:218`; Treffer nur im Testkit). `ErasureRoot` hat außerhalb von `crypto` keinen Verwender. Graph und Checkpoint haben **keine** `ErasureLayer`-Implementierung (nur Text `text/src/erasure.rs`, HNSW `vector/src/hnsw/erasure.rs`). `Sealed` wird so mit leerem Text/Vektor/Graph-Lauf erreicht; die Garantie wirkt höher, als sie ist (verletzt I-6). | §3.6 `required_layers` + Builder-Verdrahtung (A-6); ohne Pflicht-Layer kein `purged = true`. |
| **K-49** | S1 | **`on_gate`-Fehler werden verschluckt.** `mask_layers` loggt nur (`warn`). Eine nicht maskierte RAM-Sicht bleibt unbemerkt; der Job läuft weiter bis `Sealed`. | §3.5: `on_gate`-Fehler ⇒ Layer „unmasked“, Retry mit Backoff, kein `Verifying` solange ein Layer unmasked ist. Layer-Ergebnisse müssen zusätzlich selbst gegen `ReadGate` filtern (Konformanztest `CONF-LAYER-READ-1`). |
| **K-50** | S2 | **Wildcard-Fallbacks im Journal-Encoder.** `journal.rs:97-157`: `_ => 1` (unbekannte `ErasureReason` wird `UserRequest`), `_ => 4` (unbekanntes `TargetKind` wird `Subject`). Beide Enums sind `non_exhaustive`; ein neuer Wert würde falsch persistiert. | Encoder fallibel (`Err(JournalPoisoned)`/`InvalidTarget`); Gate G-2 verbietet `_ =>` in `journal.rs`-Encodern (§6). |
| **K-51** | S1 | **WAL-Zeuge kann falsch „sauber“ melden.** `WitnessLayer::WalAllSegments` liest nur das **aktive** WAL (`store/erasure/purge_lsm.rs:772-790`), ignoriert Lesefehler (`if let Ok`) und sucht Rohbytes. Bei aktivierter WAL-Verschlüsselung (`KeyManager`) existiert ein Klartext-Treffer nie; versiegelte, noch nicht gelöschte Segmente (`pending_sealed_wals`) werden nicht gelesen. | §3.7: alle Segmente, Lesefehler ⇒ `ScanFailed` (nicht `remaining = 0`), Technik `Structural` (Frame-Decode) bei Verschlüsselung. |
| **K-52** | S2 | **`ErasureId` aus Uhr-gesätem PRNG.** Fallback `SeededRng::new(clock ^ …)` (`engine/erasure/mod.rs:93-98`). ID dient als Idempotenzschlüssel und Status-Handle; vorhersagbar und kollisionsanfällig zwischen Instanzen. | **A-7** OS-CSPRNG über `KeyMaterialSource` für IDs; `SeededRng` nur `cfg(test)`/Testkit. Jitter darf PRNG bleiben. |
| **K-53** | S2 | **Observer-Zustellung nicht durabel.** `notify_observers` startet untracked `tokio::spawn`; nach Neustart wird nur `mask_layers`, nicht der Observer-Aufruf wiederholt (`mod.rs: start`). RAM-Caches externer Observer behalten gelöschte Inhalte, wenn der Prozess zwischen `Gated` und Zustellung stirbt. | §3.8: Observer idempotent, beim Start Replay `Gated` für alle offenen Jobs, Tasks über `TaskTracker`, Shutdown wartet. |
| **K-54** | S1 | **Wiederauferstehung durch `restore_checkpoint`.** `ensure_restore_allowed` (`checkpoint/src/store.rs:799-810`) prüft nur den Meta-Schlüssel `{ns}:checkpoint:{name}` gegen **aktive** Gates. Nach `Completed` ist der Gate-Eintrag entfernt (I-9). Die Barriere `last_tx > meta.tx_id` greift nicht, weil Erase keine `TxId` erzeugt **[?]** `TST-CKPT-RESTORE-3`. Ein vor der Löschung erzeugter Checkpoint stellt gelöschte Daten wieder her. | **A-5** persistenter *Erasure-Floor* (§3.9) plus Pflicht-Layer Checkpoint (09). |
| **K-55** | S3 | Ergebnis von `snaps.reap_expired(...)` wird verworfen (`let _ =`); Entzug bleibt unbeobachtet. | Anzahl in `ErasureStatus.exceptions`/Metrik; G-2-Baseline schrumpft. |
| **K-56** | S2 | `tenant_of` fällt bei leerer Scope-Menge auf `TenantId::SYSTEM` zurück (`driver.rs`). Ein Scan liefe im Systemkontext. | Leerer Scope ⇒ `InvalidTarget` schon in `accept`; kein Fallback. |
| **K-57** | S1 | **Ein Job monopolisiert den Worker.** `run_layer` und die Store-Schleife laufen bis `cursor == None` ohne Rückgabe an den Scheduler. `max_units_per_tick` begrenzt nur einen Aufruf, nicht die Runde. Andere Jobs und SLA-Prüfung warten. | §3.10: ein Tick je Job und Pass, `DriveOutcome::Progress`, Round-Robin, SLA-Jobs zuerst. |
| **K-58** | S1 | **Start-Fenster.** Zwischen Öffnen des Stores (Gate aktiv) und `mask_layers` sind Layer-Lesepfade (Textsuche, HNSW) nicht maskiert, sofern sie nicht selbst `ReadGate` prüfen. | §3.5/§3.11: Reihenfolge Store → Layer registrieren und maskieren → Lesepfade freigeben (`NotReady`). |
| **K-59** | S2 | `tools/erasure_gates_baseline.txt` führt neue G-2-Verstöße in `engine/erasure/driver.rs` (5) und `mod.rs` (1) sowie 22 G-3-Verstöße in CRUD-Pfaden (`instrument` ohne `skip`, Schlüssel/IDs im Span). Beide gehören zu K-46/K-55 bzw. I-12. | Verstöße beheben, Baseline-Zeilen entfernen (Baseline darf nur schrumpfen). |
| **K-60** | S1 | Weiter offen aus 00: **K-33** (`open_early` zieht `prk` aus `fill_key_bytes`, `mod.rs:344`; `recovery.rs` nutzt `OsKeyMaterialSource`), **A-1** (`register_collapse` fehlt im Port), **A-3** (`LAYER_TEXT`/`LAYER_CHECKPOINTS` fehlen). **K-40 ist behoben** (`dek.rs:76` propagiert `?`), **K-24/K-17** sind umgesetzt. | siehe 00 §3, 05 Revision 2. |

## 3. Kernel- und Treiber-Vertrag Rev. 2 (normativ)

### 3.1 Prinzipien (unverändert, hier präzisiert)

(P1) Ab `Gated` kein Rücksprung vor `Gated`. (P2) Jeder Schritt ist wiederholbar; Seiteneffekt **vor** Journal-Eintrag, Eintrag nur nach dauerhaftem Seiteneffekt. (P3) Kein Verwerfen von Fehlern: ein `Result` aus `fail`, `record_layer`, `on_gate`, `reap_expired`, Observer wird ausgewertet. (P4) Der Kernel entscheidet über Zustand, der Treiber nur über Zeitpunkt.

### 3.2 A-4 `begin_purge`

```rust
// ports::ErasureKernel – neue Methode, additiv, am Ende des Traits
fn begin_purge<'a>(&'a self, id: ErasureId) -> BoxFuture<'a, Result<(), ErasureError>>;
```

| Eigenschaft | Vertrag |
| --- | --- |
| Vorbedingung | Zustand `Gated`, `KeysDestroying`, `Purging` (No-op), `Failed{..}`, `Quarantined{..}` **nur** nach `resume` (§3.3); `gated_seq.is_some()`; Journal nicht poisoned |
| Wirkung | Wenn nicht `Purging`: `PurgeStart` ins Journal (`sync_data`), Zustand `Purging`, Gate-Eintrag `Restricted → Erased{up_to}`. Genau die Logik aus `purge_store` (`mod.rs:1146-1190`), **extrahiert**; `purge_store` ruft intern `begin_purge` und bleibt damit idempotent |
| Format | unverändert (vorhandenes Record `PurgeStart`), `JOURNAL_VERSION` bleibt 1 |
| Fehler | `InvalidTransition`, `JournalPoisoned`, `NotFound` |
| Treiber | ruft `begin_purge` **vor** dem ersten Layer jeder Runde, danach `run_layer`; `record_layer` bleibt auf `Purging` beschränkt (kein Aufweichen der Vorbedingung) |

Begründung `Erased` vor Layern: Das Gate sperrt Lesen bereits ab `Gated`; `Erased` erlaubt nur dem Store-Compaction, Scope-Schlüssel zu verwerfen. Die Collapse-Registrierung (A-1) betrifft ausschließlich `CollapseVersions` in `purge_store` und wird vor dessen Start von den Layern abgeschlossen (Layer laufen zuerst).

### 3.3 Fehler, Versuche, Quarantäne

1. **Ein** Schwellenwert: **neues** Feld `StoreErasureConfig.quarantine_after_attempts` (Default 10; heute im Kernel hart codiert, `mod.rs:1622`). Der Treiber konvertiert nicht mehr; das vorhandene `engine::ErasureConfig.quarantine_after_attempts` (`config.rs:14`) entfällt und wird beim Bau an die Store-Konfiguration durchgereicht.
2. `fail(id, layer, kind)` erreicht ≥ Schwelle ⇒ `Quarantined{layer}`; bei `layer == None` ist `layer = "job"` (Konstante `QUARANTINE_JOB_LAYER`). Kein Zustand `Failed` mit Versuchszahl ≥ Schwelle.
3. Der Treiber wertet den Rückgabewert aus: `Ok(Quarantined)` ⇒ `DriveOutcome::Quarantined`; `Ok(Failed)` ⇒ `Retry(backoff)`; `Err(JournalPoisoned)` ⇒ `Poisoned`; jeder andere `Err` ⇒ `Poisoned` + `error!` (Zustand unbekannt ist fail-closed, das Gate bleibt).
4. `Failed → Failed` bleibt **verboten**. Weil der Treiber nach jedem Fehler zuerst `begin_purge` ruft (Zustand wieder `Purging`), tritt es nicht auf. Tritt `InvalidTransition` aus `fail` trotzdem auf, ist das ein Programmfehler (Test `TST-DRV-FAIL-2`).
5. **A-8 `ErasureKernel::resume(id) -> Result<(), ErasureError>`**: einziger Weg aus `Quarantined`; Operator-/API-Aufruf, schreibt `Failed{layer, attempt: 0}`-Übergang nach `Purging`-Vorbereitung (`attempt` wird 0). Nie automatisch. Engine-API `Contextra::erase_resume(id)`; MCP: **kein** Werkzeug (zerstörungsfreier Operatorweg bleibt außerhalb des Agenten).

### 3.4 Layer-Wiederholung nach Verifikations-Rest (K-47)

Regel für `fail(id, Some(l), _)` und für das Replay von `JournalRecord::Failed{layer: Some(l), ..}`: `layer_cursor.remove(l)`. Folge: `to_job_view().layers_done` enthält `l` nicht mehr, der Treiber führt `l` erneut von Cursor `None` aus. Kein neues Record-Tag, keine Formatänderung. Kosten: ein transienter Fehler mitten im Layer startet diesen Layer neu. Das ist zulässig, weil jede `ErasureLayer::purge` idempotent sein **muss** (02 §4.5, Konformanz C1–C7 in 13; `CONF-LAYER-IDEM-1`). Layer mit teuren Läufen (HNSW-Datei-Rewrite) puffern ihren Fortschritt intern.

### 3.5 `on_gate`, Maskierung, Lesepfad (K-49, K-58)

* `on_gate` Fehler ⇒ der Treiber merkt `unmasked(layer)` im Scheduler-Eintrag **und** bricht die Runde mit `Retry(backoff)` ab; `drive_job` geht nicht nach `Verifying`, solange ein Layer unmasked ist.
* **Fail-closed im Layer:** Jeder Layer, der Treffer liefert (Textsuche, HNSW, Graph), filtert das Ergebnis **vor Rückgabe** über `ReadGate::check_*` (zusätzlich zu `on_gate`). `on_gate` ist Beschleunigung (Bitmaske), nicht Sicherheitsgrenze. Konformanztest `CONF-LAYER-READ-1`: Layer vor `on_gate` gesperrt, Treffer eines gesperrten Dokuments darf nie erscheinen.
* Start: Lesepfade der Engine liefern `ContextraError::NotReady`, bis (1) Kernel offen und Gate rekonstruiert, (2) alle Layer registriert, (3) `mask_layers` für alle offenen Jobs erfolgreich. `NotReady` ist neue Variante am Ende von `ContextraError` (K-6).

### 3.6 Pflicht-Layer (K-48, A-6)

```rust
// engine::erasure::ErasureConfig – neu
pub required_layers: BTreeSet<WitnessLayer>,   // Default je Profil, siehe unten
```

* `accept` prüft `required_layers ⊆ registrierte Layer ∪ Store-Layer`. Fehlt einer: `ErasureError::LayerMissing{layer}` (neue Variante am Ende). Nur im Profil **Regulated** verbindlich verweigernd.
* In allen anderen Profilen: Auftrag läuft, `Sealed` setzt `purged = false` und trägt `exceptions += FeatureDisabled{layer}` (vorhandene Variante, kein Formatwechsel). `Completed` wird erreicht, die ausgewiesene Garantie bleibt `gated` (I-6, „senkt sichtbar“).
* Profil-Defaults: *Regulated* ⇒ `{Text, Vector, Graph, Checkpoints}` soweit die Crate im Build aktiv ist; *Standard* ⇒ `{}` mit Ausweis; *Embedded* ⇒ `{}`.
* **Builder-Verdrahtung (11):** `Contextra::build` registriert `TextErasure`, `HnswErasure`, `GraphErasure`, `CheckpointErasure` in der Reihenfolge Text → Graph → Vektor → Checkpoint (Registry-Order), **nach** Kernel-Öffnung und **vor** Freigabe der Lesepfade. Test `TST-ENG-WIRE-1` schlägt fehl, wenn `register_erasure_layer` nur im Testkit aufgerufen wird.

### 3.7 Zeugen (K-51)

`LayerScanner::scan` liefert je Einheit `{hash, found}`; neu ist der Fehlerfall:

| Situation | Verhalten |
| --- | --- |
| Datei nicht lesbar / Segment gesperrt | `Err(ScanFailed)`; der Treiber wertet das als `Retryable` am Layer, **nie** als `remaining = 0` |
| WAL | iteriert aktives WAL, `pending_sealed_wals` und jedes `wal-*.log` im Datenverzeichnis; versiegelte, noch in `PendingDelete` stehende Segmente zählen als Rest, bis `PendingDelete` sie entfernt hat |
| WAL verschlüsselt | Technik `Structural`: Frames mit dem `KeyManager` entschlüsseln und Schlüssel vergleichen; Rohbyte-Suche allein ist unzulässig |
| Zeuge `NotMeasured` | senkt die Garantie (I-6), blockiert aber `Sealed` nicht, wenn `required_layers` erfüllt ist |

Canary-Test `T-DEL-1` läuft in **beiden** Modi (Klartext-WAL, verschlüsseltes WAL). Ein Zeuge, der im verschlüsselten Modus nie Treffer liefern kann, gilt als Defekt (`TST-STORE-WIT-2`).

### 3.8 Observer (K-53)

1. Observer-Aufrufe laufen auf dem Engine-`TaskTracker`; `stop()` wartet auf sie (begrenzt durch `shutdown_grace`).
2. Observer müssen **idempotent** sein (Vertrag in 02 §4.8, hier verschärft).
3. Beim Start: für jeden offenen Job ab `Gated` wird `ObservedPhase::Gated` erneut zugestellt. Caches (EpochCache, Antwort-/Embedding-Caches) werden beim Start ohnehin geleert; externe Observer erhalten damit denselben Reset-Auslöser.
4. Zustellfehler zählen (`observer_failures`), blockieren aber weder `Sealed` noch `Completed`. Ausnahme *Regulated*: ein Observer, der `required` ist, blockiert `Completed` bis zur erfolgreichen Zustellung (Config `required_observers`).

### 3.9 A-5 Erasure-Floor (K-54)

Zweck: Wiederherstellen (Restore, Import, Backup-Wiedereinspielung) darf bereits abgeschlossene Löschungen nie rückgängig machen, auch wenn der Gate-Eintrag nach I-9 weg ist.

* **Floor-Eintrag** je `Completed`-Auftrag: `(erasure_id_hmac, plan_hash, gated_seq, scope_digest)`; liegt im verdichteten Journal-Summary (`completed_summaries`) und im Ledger. Er wird **nicht** vom Rollover gelöscht (§5.5 in 03 behält die Summary-Frames).
* **Restore-Regel:** `restore_checkpoint(name)` wird verweigert (`ContextraError::Erased`), wenn ein Floor-Eintrag existiert mit `gated_seq > meta.seq_no` **und** der Checkpoint nicht als `Rewritten` (09 §5.3) markiert ist. Der Vergleich nutzt `gated_seq`, nicht `TxId` (Erase erzeugt keine `TxId`).
* **Bereinigte Checkpoints** (`Drop`/`Rewrite`) sind Teil von `required_layers` (§3.6); `Sealed` setzt `Checkpoints.remaining == 0` voraus.
* Import/Backup außerhalb des Prozesses: nicht erreichbar; wird im Ausweis als `exceptions += OutOfScopeExternalCopies` (vorhanden) geführt (§4 Zeile 20).

### 3.10 Planung (K-57)

* `run_layer` und die Store-Schleife führen **einen** `purge`-Aufruf je Job und Pass aus. Danach `record_layer`/`StoreProgress`, Rückgabe `DriveOutcome::Progress` (neue Variante, am Ende).
* Der Worker sortiert fällige Jobs: (1) SLA-gefährdet (≥ 80 %), (2) älteste `first_seen`; er führt je Pass höchstens `cfg.max_jobs_per_pass` aus und kehrt danach in die Schleife zurück (Cancel-Prüfung, Notify).
* Leerlauf-CPU 0 bleibt: bei `Progress` direkt weiter, sonst `Notify`/Deadline.
* SLA-Eskalation: bei Überschreitung `error!` **und** Laufzeitfeld `sla_breached_at` im `ErasureStatus` (nicht im `Sealed`-Frame, kein Journal-Formatwechsel); keine Zustandsänderung.

### 3.11 Startreihenfolge (normativ, ersetzt Index §5 Satz 1 der Treiberschicht)

```text
1  ErasureRootProvider.root()            (A-2; fehlt Wurzel bei vorhandenem Journal/Ledger ⇒ Start verweigert)
2  StoreErasureKernel::open_early        (Journal, Ledger, Pending-Delete, Gate fail-closed)
3  LsmStorage-Konstruktor                (Replay-Filter und Gate VOR WAL-Replay, K-13)
4  Layer registrieren (Text → Graph → Vektor → Checkpoint)
5  mask_layers + Observer-Replay für alle offenen Jobs
6  Lesepfade freigeben (NotReady endet)
7  Worker starten
```

## 4. Datenort-Matrix (Prüfmaßstab für „lückenlos“)

Spalten: **G1** = Lesesperre wirkt (Gate), **G3** = physisches Entfernen, **Zeuge** = `WitnessLayer`, **Stand** = Code am HEAD. Eine Zeile ist erst „geschlossen“, wenn G1, G3, Zeuge **und** Crash-Test vorhanden sind. Zeilen ohne G3-Fähigkeit (Zeile 19, 20) werden im `ErasureStatus.exceptions` ausgewiesen.

| # | Ort | Kann Scope-Bytes enthalten | G1 | G3 (Entfernung) | Zeuge | Crash | Stand / Lücke |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Commit-Pfad (Group-Commit in-flight) | Ja (Batch vor `append_batch`) | `prepare_commit_ops` **und** vor `wal.append_batch` (K-1) | wird nie persistiert, wenn Gate trifft | – | CP-1/-2 | ✔ spezifiziert; Test `TST-STORE-ROT-1` bleibt Pflicht |
| 2 | WAL aktiv | Ja | Replay-Filter (`recovery.rs:322/331`) | `erase_wal_cut`: rotieren, gefilterter Flush, Segment löschen | `WalAllSegments` | CP-24 | ✔ Kernel; **Zeuge defekt (K-51)** |
| 3 | WAL versiegelt / `pending_sealed_wals` | Ja | Replay-Filter | `PendingDelete` | `WalAllSegments` | CP-17/-18/-24 | ✔; Zeuge sieht es nicht (K-51) |
| 4 | MemTable aktiv + immutable | Ja | Lesemaske, Schreib-`Erased` | `MemTable::retain` (K-4) | `Memtable` | – | ✔ |
| 5 | SSTables (Daten-, Index-, Bloom-Blöcke) | Ja, auch Index-/Bloom-Material | Lesemaske | Erase-Compaction 1:1, leeres Ergebnis ⇒ `Remove` (K-2) | `Sstables` | CP-15…17 | ✔; `[?]` Bloom-/Indexblöcke der **nicht** umgeschriebenen Dateien: Scope-Prüfung `sstable_overlaps_filter` darf Min/Max-Schlüssel nicht allein nutzen `TST-STORE-SST-3` |
| 6 | Block-Cache (RAM) | Ja | – | `evict_file` je ersetzter/entfernter Datei (`purge_lsm.rs:275,344`) | – | – | ✔ |
| 7 | Manifest (Min/Max-Schlüssel je SST) | Ja (Schlüsselbytes) | – | `Replace` schreibt neu berechnete Bereiche **[?]** `TST-STORE-MAN-2`; alte Manifest-Generationen/Rollover-Dateien müssen entfernt werden | Manifest-Scanner **fehlt** | CP-16/-17 | **Lücke:** kein Zeuge `Manifest`; Aufgabe WP-14.4 |
| 8 | MVCC-Versionen, Snapshots, Pins | Ja (alte Versionen) | `check_version` je Seq | Compaction ohne Pin; `revoke_covering` nur Mandanten-Snapshots (K-21), Pin-Alterung (04) | `MvccVersions` | – | ✔; Entzugs-Ergebnis unbeobachtet (K-55) |
| 9 | Journal (Plan-Blob), Pending-Delete-Liste, `*.erase.tmp` | Ja (Zielbytes) | – | Zielbytes nach `Completed` aus Journal entfernen (I-12), Rollover | `Journal` | CP-13…15 | ✔; Tests `TST-STORE-JRN-*`; Temp-Pfade nach Abbruch über `collect_temp_file_paths` |
| 10 | Ledger | Nein (nur HMAC-IDs) | – | – | – | CP-26 | **K-33:** Schlüssel nicht persistent ⇒ nach Neustart `LedgerCorrupt` |
| 11 | Text-Index (`i:`/`plb:`, `resident_index`, `meta:stats`) | Ja | `on_gate` + Ergebnisfilter | `TextErasure.purge` (Postings neu schreiben, Stats neu rechnen) | `Custom("TextIndex")` | CP-28 | ✔ Code; **nicht verdrahtet (K-48)**; A-3-Konstanten offen |
| 12 | HNSW (RAM, Quantisierung, `save`/mmap, `deleted_nodes`) | Ja (Vektoren sind personenbezogen) | `on_gate` + Ergebnisfilter | `HnswErasure.purge` (Graph reparieren, Vektoren ersetzen/nullen), Datei-Rewrite optional (K-37) | `Hnsw` | CP-28 | ✔ Code (98 LOC); **nicht verdrahtet (K-48)**; `[?]` Quantisierungs-Codebook enthält keine Einzelvektoren `TST-VEC-CB-1` |
| 13 | Graph (`__graph:entity/edge/community`, CSR) | Ja (tenantweit, K-38) | über Provenienz (`EdgeProvenance`) | **Layer fehlt** | **fehlt** | – | **Lücke S1:** `GraphErasure` nicht implementiert (08) |
| 14 | Checkpoints, Hardlink-Klone, Orphan-Registry | Ja | `ensure_restore_allowed` (nur Meta-Schlüssel) | **Layer fehlt** (`drop_checkpoint_for_erase`, Rewrite) | **fehlt** | CP-29 | **Lücke S1 (K-54)** |
| 15 | KV-Cache / `KvSegmentCipher` / Salt-Tresor (Stufe 2) | Ja (verschlüsselt) | – | Schlüsselvernichtung (`KeysDestroying`) | `Keys` | CP-27 | Spezifiziert (05 §5); `kv-shredding` Feature fehlt |
| 16 | Engine-Caches (EpochCache, Antwort-/Embedding-Caches, Query-Plan-Cache) | Ja | – | `ErasureObserver` | – | – | ✔ `caches.rs`; Replay (K-53) |
| 17 | Logs, Traces, Metriken | Ja, wenn IDs/Schlüssel in Spans | – | nicht entfernbar ⇒ **verhindern** (G-3) | – | – | **22 G-3-Altlasten (K-59)** |
| 18 | Antworten in Flug (MCP, Facade, offene Streams, `Cursor`) | Ja | `ReadGate` beim Lesen | Abbruch offener Cursor beim `Gated` `[?]` `TST-MCP-CUR-1` | – | – | Lücke dokumentieren in 12 |
| 19 | Prozessspeicher (Heap, Swap, Core-Dumps, `mmap`) | Ja | – | `zeroize` für Schlüssel; Klartextpuffer nicht garantierbar | – | – | **Ausnahme** dauerhafte Garantiegrenze „Prozessspeicher“ (Dokumentation und Ledger-Scope-Text, **kein** neuer persistierter Ausnahme-Wert) |
| 20 | Dateisystem-Snapshots, Backups, SSD-Wear-Leveling, Journaling-Reste | Ja | – | außerhalb der Kontrolle; nur Stufe 2 (Schlüsselvernichtung) wirkt | – | – | **Ausnahme** `exceptions += OutOfScopeExternalCopies` (vorhanden); Doku fordert Stufe 2 für Regulated |

**Abnahmeregel:** Eine Auslieferung mit Garantie `purged = true` setzt voraus, dass Zeilen 1–16 „geschlossen“ sind. Zeilen 17–20 sind „verhindert bzw. ausgewiesen“.

## 5. Crash-Matrix (Ergänzung zu 03 §9, CP-1…CP-18 bleiben)

| ID | Crash-Punkt | Erwarteter Zustand | Wiederaufnahme |
| --- | --- | --- | --- |
| CP-19 | nach `accept`, vor `publish_gate` | `Requested/Planned`, Gate nicht veröffentlicht | Treiber ruft `publish_gate` (bereits in `drive_job`), `Gated` ⇒ Weiter; vor `Gated` kein Schreib-Effekt |
| CP-20 | nach `GateIntent`, vor `Gated`-Frame | Gate aus Intent rekonstruiert, fail-closed | `Gating → Gated` im Recovery, kein Fenster ohne Sperre |
| CP-21 | Layer-Seiteneffekt durable, `LayerProgress` fehlt | Layer wird wiederholt | idempotent (`CONF-LAYER-IDEM-1`) |
| CP-22 | nach Ledger-Append, vor `Sealed`/`Completed`-Frame | Ledger hat Eintrag, Journal nicht | `seal` idempotent über `erasure_id_hmac`; kein Doppeleintrag |
| CP-23 | nach `Completed`-Frame, vor Floor-Fortschreibung im RAM | Floor aus Journal-Summary | Recovery baut Floor neu auf (§3.9) |
| CP-24 | WAL-Schnitt: neues WAL angelegt, altes nicht entfernt | altes in `pending_sealed_wals`/Pending-Delete | Boot entfernt (CP-17-Logik) |
| CP-25 | Verifikation scheitert, Crash vor `Failed`-Frame | Zustand `Verifying` | erneute Verifikation, kein Zähler-Sprung |
| CP-26 | Ledger: Torn Tail / Bitflip | Torn: kürzen; Bitflip: `LedgerCorrupt` | wie CP-13/-14, Gate bleibt bis Auflösung |
| CP-27 | Wurzeldatei: Crash bei **erster** Erzeugung (temp, `rename`, `sync_dir`) | entweder keine oder vollständige Wurzel | Journal ohne Wurzel ⇒ Start verweigert, nie Neuerzeugung (A-2) |
| CP-28 | Layer-Datei-Rewrite (HNSW `save`, Text-Segmente) | alte oder neue Datei, nie gemischt | Temp + `rename` + `sync_dir`, Orphan-Aufräumen |
| CP-29 | Checkpoint `Drop`/`Rewrite` halb fertig (Hardlinks) | Meta zeigt Zustand, Dateien teils da | Wiederholen; Restore verweigert bis `Checkpoints.remaining == 0` |
| CP-30 | Zweiter Crash **während** der Recovery | wie CP-19…29 | Recovery ist idempotent und schreibt vor Abschluss nichts außer Torn-Tail-Kürzung |

Alle laufen über `CrashVfs` (13 §4). Zusätzlich Nebenläufigkeit (Loom-Modell, 03 `loom_erasure_gate.rs`): (a) Schreiber gegen Gate-Veröffentlichung, (b) `complete` gegen Lese-Epoche, (c) Layer-`purge` gegen `restore_checkpoint`.

## 6. Tests (neu, je Rot-Grün)

| ID | Prüft | Muss vor Fix rot sein |
| --- | --- | --- |
| `TST-DRV-ORDER-1` | Treiber mit Mock-Layer: `Gated → begin_purge → record_layer` ohne `InvalidTransition` (K-45) | ja |
| `TST-DRV-FAIL-1` | `fail` wird ausgewertet; Quarantäne an Kernel-Schwelle, auch für `layer = None` (K-46) | ja |
| `TST-DRV-FAIL-2` | `Failed→Failed` tritt nicht auf; Treiber ruft vor Layer `begin_purge` | ja |
| `TST-DRV-RESET-1` | Layer-Rest bei Verifikation ⇒ Layer läuft erneut, Job erreicht `Sealed` (K-47); Replay nach Crash liefert denselben Cursor-Zustand | ja |
| `TST-DRV-GATEERR-1` | `on_gate` schlägt fehl ⇒ kein `Verifying`; nach Erfolg weiter (K-49) | ja |
| `TST-DRV-SCHED-1` | Zwei Jobs, einer lang: der zweite macht innerhalb eines Passes Fortschritt (K-57) | ja |
| `TST-ENG-WIRE-1` | `Contextra::build` registriert Text/Vektor/Graph/Checkpoint; Regulated verweigert ohne Pflicht-Layer (K-48) | ja |
| `TST-ENG-NOTREADY-1` | Lesepfad vor Maskierung liefert `NotReady` (K-58) | ja |
| `TST-ENG-ID-1` | `ErasureId` stammt aus `KeyMaterialSource` (Mock zählt Aufrufe), nicht aus `SeededRng` (K-52) | ja |
| `TST-JRN-ENC-1` | Unbekannte `ErasureReason`/`TargetKind` ⇒ `Err`, kein Fallback (K-50) | ja |
| `TST-STORE-WIT-1` | WAL-Zeuge liest versiegelte Segmente; Lesefehler ⇒ `ScanFailed` (K-51) | ja |
| `TST-STORE-WIT-2` | Zeuge findet Schlüssel im **verschlüsselten** WAL (Structural) | ja |
| `TST-CKPT-RESTORE-3` | Restore eines vor `Completed` erzeugten Checkpoints wird verweigert (K-54) | ja |
| `TST-OBS-REPLAY-1` | Neustart zwischen `Gated` und Observer-Zustellung ⇒ Observer erhält `Gated` erneut (K-53) | ja |
| `TST-GATES-ENC-1` | Gate G-2 meldet `_ =>` in `journal.rs`-Encodern | ja |

CI-Gates (`tools/erasure_gates.py`) **neu:** **G-7** kein `tokio::spawn` ohne `TaskTracker` in `engine/src/erasure/`; **G-8** `Result` von `fail`/`record_layer`/`on_gate` nicht verworfen (Teilmenge von G-2, spezifisch mit Symbolliste); **G-9** `_ =>` in `store/src/erasure/journal.rs` verboten.

## 7. Arbeitspakete und Reihenfolge

```text
WP-14.0  Befunde K-45/46/47 (A-4, §3.2–3.4)     Store+Engine+Ports      ← blockiert jeden Praxislauf
WP-14.1  A-1, A-2 (K-33), A-3                     Ports+Store+Crypto      ← Auslieferungsstopp
WP-14.2  Observer/Start/NotReady (K-49, 53, 58)   Engine
WP-14.3  Pflicht-Layer + Builder (K-48, A-6)      Engine+Facade
WP-14.4  Zeugen (K-51) + Manifest-Zeuge (Zeile 7) Store
WP-14.5  Graph-Layer (08) · Checkpoint-Layer + Floor (09, K-54, A-5)
WP-14.6  Scheduler (K-57), ID-Quelle (K-52, A-7), Encoder (K-50)
WP-14.7  Baseline-Abbau G-2/G-3 (K-55, K-56, K-59), neue Gates G-7…G-9
```

Jedes Paket endet mit `cargo build --workspace --locked`, `cargo test --workspace --locked`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`, `tests/layering.rs`, `tools/erasure_gates.py` (Baseline nur schrumpfend) und dem Crash-/Loom-Satz des Pakets. `capabilities.toml` bleibt unverändert, mit einer Ausnahme, die vor Beginn zu prüfen ist: **[?]** ob `contextra-checkpoint` den Kernel-Port (`ErasureKernel`) über `contextra-ports` erreicht (K-36) und `contextra-graph` `ports`/`types` bereits hat.

## 8. Offene Punkte

1. **[?]** `MemTable`-/SST-Min/Max-Prüfung (Zeile 5, 7): ob `sstable_overlaps_filter` Dateien überspringt, die Scope-Schlüssel nur im Index-Block führen (`TST-STORE-SST-3`).
2. **[?]** Quantisierungs-Codebook in HNSW (Zeile 12).
3. **[?]** Offene Cursor beim Gate (Zeile 18): ob `Cursor`/Streams in MCP nach `Gated` weiter Treffer liefern.
4. Mehrinstanz-Betrieb (zwei Prozesse auf einem Datenverzeichnis) ist nicht Gegenstand; der Kernel setzt ein exklusives Verzeichnis-Lock voraus **[?]** `TST-STORE-LOCK-1`.
