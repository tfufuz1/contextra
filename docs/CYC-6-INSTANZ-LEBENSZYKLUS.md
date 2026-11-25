# CYC-6 — Instanz-Lebenszyklus (Create, Open, Recovery, Close, Checkpoint, Restore, Backup-Horizont, DiskFull, Poison)

Ersetzt R14 §5.6, §9. Abhängigkeiten: SYS-1 §3/§6 (Ports, Fehler), SYS-2 §4/§5.1/§5.3/§5.6/§5.10 (Dateien, Redo, Catalog, Manifest), SYS-3 §2 (Fassade), SYS-4 §2.4/§2.5/§5 (Zahlen, Validierung), CYC-1, CYC-3 (Löschprotokoll, Poison-Aufnahme), CYC-4 (Schlüssel), CYC-5 (Flush, Rollover). Rang: gehört mit CYC-3 zu den Spezifikationen höchsten Vorrangs unterhalb SYS-7 (Index §1). Gate: **G-E**.

## 1. Zweck und Grenzen

Der Zyklus legt fest, wie eine Instanz entsteht, deterministisch aus dem Zustand auf Platte wieder lebendig wird, sauber endet, kopiert und zurückgespielt wird und wie sie sich bei vollem Datenträger und bei Fehlern verhält. **Open ist die einzige Stelle, an der die Wahrheit über Löschungen aus Catalog, Redo und Ledger zusammengeführt wird.**

**Kernsätze.**
1. Open öffnet nie *schwächer* als konfiguriert: ein `Required`-Anker, der nicht antwortet, ein nicht lesbares Ledger, ein fehlender Schlüssel führen zu Verweigerung oder eingeschränktem Zustand, nie zu stillem Weiterlaufen (SYS-5, `INV-ANCH-1`).
2. **Vor jeder Nutzer-Sicht** sind alle offenen Löschabsichten in Gates überführt (OP8, `INV-ERA-9`, `INV-ERA-13`); kein Body wird vor der Gate-Auflösung entsiegelt.
3. Restore belebt nie Gelöschtes (`INV-ERA-9`); ein fehlender Schlüssel, Anker oder Fencing-Nachweis ist nie eine Löschung und nie eine Freigabe.
4. Löschung bleibt in jedem Zustand aufnehmbar, in dem ein `ledger_key` verfügbar ist (`INV-DISK-1`, CYC-3 §9.4).
5. Eine Instanz hat genau einen schreibenden Besitzer (`LOCK`, NZ-19).

## 2. Zustände

`InstanceState` (SYS-3 §2): `Normal | DiskFull | Poisoned | ReadOnly(Reason) | LockedKeys | RestoreQuarantine`; intern zusätzlich `Opening`, `Closing`, `Closed` (nie nach außen sichtbar außer `ReadOnly{Closing}`).

| Zustand | Eintritt | erlaubt | verweigert (Fehler) | Austritt |
|---|---|---|---|---|
| `Normal` | Open erfolgreich | alles | — | → `DiskFull`, `Poisoned` |
| `DiskFull` | `free_space < [diskfull_enter_bytes]` oder `ENOSPC` beim Vorallokieren (§8) | Lesen, Löschung (Gate, ER2, Purge), Close | `apply`/Branch-Mutationen (`DiskFull`) | `free_space ≥ [diskfull_exit_bytes]` ⇒ `Normal` |
| `Poisoned` | `sync_*`-/`EIO`-Fehler auf Redo, Catalog, Ledger (nach Fallback) oder Anker-Schreiber (§9) | Lesen aus nicht betroffenen Dateien, Löschaufnahme als Intent, `close(Fast)` | alle Mutationen (`Poisoned`) | nur Neustart |
| `ReadOnly(Reason)` | `OpenMode::ReadOnly`, `Reason ∈ {Requested, Corrupt, ClusterMetaBudget, LedgerUnavailable, Rollback, Closing}` | Lesen, `export_cluster`, Diagnose | Mutationen (`ReadOnly{reason}`) | Neustart mit behobener Ursache |
| `LockedKeys` | RootKey verfügbar, Clusterschlüssel/Tomb-Schlüssel beim Replay `KeyUnavailable` (OP6) | Löschaufnahme (Intent), Diagnose, Hintergrund-Wiederholung | Lesen (`KeyUnavailable`), Schreiben | alle OP6-Schlüssel verfügbar ⇒ OP7–OP10 ⇒ `Normal` |
| `RestoreQuarantine` | Restore (§6) bis `RestoreReconciled` | Löschaufnahme, Diagnose | Lesen (`RestoreQuarantine`), Schreiben | Abschluss von §6 ⇒ `Normal` |

`LockedKeys` wiederholt OP6/OP7 im Hintergrund (Backoff `[key_probe_interval]`); der Kern wartet nicht auf Aufrufer. **KEK nicht erreichbar** liefert gar keine Instanz: `Err(Locked{cause: Keys})`, weil ohne RootKey weder Ketten noch Ledger prüfbar sind. Die Engine MUSS einen Löschauftrag, den sie nicht als `Ok(Ticket)` bestätigt bekam, dauerhaft halten und wiederholen (Auftragswarteschlange der Engine, SYS-5 §7); der Kern kann ohne KEK keinen Intent aufnehmen.

## 3. Open (deterministisch, `Storage::open`, Blockierklasse B1)

### 3.1 Vor E/A

Validierung der `OpenConfig` (SYS-4 §5 Nr. 1–5), Ports vollständig, Profil geprüft, Umgebung geprüft (`RLIMIT_NOFILE`, `free_space ≥ erase_reserve_bytes + diskfull_enter_bytes`, `FsCaps`). Erst danach OP1.

### 3.2 Ablauf OP1…OP10

| OP | Schritt | Vor-/Nachbedingung | Fehler |
|---|---|---|---|
| OP1 | `LOCK`: `Fs::lock(Exclusive)`, bei `ReadOnly` `Shared` | Besitz gesichert | belegt ⇒ `Locked{cause: Lock}` |
| OP2 | `FORMAT` lesen (Kopf, `format_ver`, `min_reader_ver`), `KEYS` lesen, `kek(kek_id)` ⇒ RootKey entwickeln, `FORMAT-MAC` prüfen, abgeleitete Schlüssel (`redo_key`, `catalog_key`, `file_key`, `ledger_key`, `LedgerScopeKey`, `idem_key`) ableiten | RootKey im RAM (`Zeroizing`) | `Unsupported` (Version/Bits), `Corrupt{Format\|Keys}`, KEK `Unavailable`/`Denied` ⇒ `Locked{cause: Keys}` |
| OP3 | `CURRENT` lesen (CRC), `CATALOG-<n>`: Kette prüfen bis zum letzten gültigen Record; Snapshot laden, Edits mit `seq > q_start` anwenden (CYC-5 §8.2) | Catalog-Zustand im RAM | gültige Records nach einem ungültigen ⇒ `Corrupt{Catalog}`; Torn Tail am Ende ⇒ gültiges Präfix (Heilung nur schreibend, Nullen) |
| OP2a | **Anker und Ledger** (nach OP3, weil `catalog_epoch` bekannt sein muss): `anchor.read() = A`. (a) `catalog_epoch < A.catalog_epoch` ⇒ `Rollback` (F-9); (b) das Ledger MUSS einen Record mit `mac == A.ledger_head` und `seq ≥ A.erase_epoch`-Marke enthalten, sonst `Rollback`; (c) Ledger-Records **hinter** dem Anker sind ein Crash-Präfix (G1 vor G5) und werden vor Schreib-Freigabe idempotent fortgesetzt (OP8); (d) Ketteprüfung des gesamten Ledgers. Kürzeres Ledger als der Anker ⇒ `Rollback` | `AnchorPolicy::Required` und Anker unerreichbar ⇒ `Locked{cause: Anchor}` (nie schwächer öffnen). Ledger nicht lesbar ⇒ `LedgerUnavailable` (schreibend verweigert; `ReadOnly{LedgerUnavailable}` mit Diagnose) |
| OP4 | Verzeichnisse scannen (`Fs::list`): Dateien ohne Catalog-Eintrag = **verwaist** (noch nicht löschen); Catalog-Dateien ohne Datei oder mit abweichender `len` ⇒ Datei vergiftet (kleinste Einheit, `Corrupt{File}`, in `OpenReport.quarantined`) | | fehlendes **Redo-Segment ≥ `replay_floor`** ⇒ `Corrupt{RedoMissing}` (Open scheitert) |
| OP5 | Provider prüfen: offene Tomb-Epoche `tomb_key` vorhanden? `list()`-Abgleich vorbereitet (CYC-4 §5.2) | | Tomb-Schlüssel fehlt ⇒ `KeyLost` der Tomb-Domäne (CYC-4 §7 Nr. 4): `OpenReport.warnings`, neue Intents `KeyUnavailable`, Restore quarantined |
| OP6 | **Redo ab `replay_floor`, zwei Durchläufe.** (1) Kette prüfen (`redo_key`), Gruppenköpfe/Frames, Torn-Tail-Regel (SYS-2 §5.3.1); sammeln: Cluster-Retires, `erased_ranges`, Branch-/Space-Gates, Gate-Frames ohne Catalog-Spiegel, ungültige Hüllenfrage; Ledger-Intents (OP2a) sind zusätzliche Gate-Wahrheit. (2) Retired-Cluster überspringen (**nie entsiegeln**), Selektor-Masken anwenden, Branch-/Merge-Frames in Stampreihenfolge, Memtable aufbauen; `policy_epoch` wird aus dem letzten gültigen Frame wiederhergestellt | `KeyUnavailable` ⇒ `LockedKeys`; bestätigtes `KeyLost` ohne Erase-Intent ⇒ Cluster-Quarantäne (Lesen/Schreiben `KeyLost`); Segmente mit `format_ver ≠` aktuell und Replay nötig ⇒ `MigrationNeedsCleanClose` |
| OP7 | `visible = durable = ` letzter gültiger Stamp; `next_stamp = max(stamp_limit, letzter gültiger Redo-Stamp, max Run-Stamp) + 1` (`INV-STAMP-3`); Redo-Schreiber, Syncer, Ledger-/Anker-Schreiber starten (schreibend) | | `Exhausted` bei `Epoch::MAX` |
| OP8 | **Gate-Auflösung vor jeder Nutzer-Sicht.** Offene Löschungen = Intents ohne `GateApplied` ∪ `erase_pending` ∪ Gate-Frames ohne `erase_done` ∪ `Retired ≠ Purged` ∪ Space `Dropping` (auch `origin = Poisoned`/`Fallback`, Fallback-Ledger wird vorher zusammengeführt, CYC-3 §9.4). Je Intent: G2 nachziehen (Gate-Frame mit `origin` unverändert, Flag `FALLBACK_INTENT` falls aus der Fallbackkette, Hülle = Intent), G3-Edit, G4, G5; dann Stufen fortsetzen (Lane `urgent`) | kein `Body` vor Abschluss entsiegelt; `ReadOnly`-Open: Gates nur als RAM-Masken (`pending_erase`, Zuordnung über HMAC der `tomb_id`s, kostet `Cluster × aktive Epochen`, H-23) | fehlender `LedgerScopeKey` ⇒ `RestoreQuarantine`/`LockedKeys`; Catalog-Handle vergiftet ⇒ `Poisoned` |
| OP9 | Verwaiste Dateien (OP4) und `.building.*` entfernen (nach OP8), Graveyard (`files_remove` ohne `unlink_ack`) unlinken, `sync_dir`, `unlink_ack` | nur schreibend | |
| OP10 | Reserven herstellen (`RESERVE`, Scratch-Slots, Reservesegment, Standby-Redo, `reserve_*`), Admission prüfen (FD-Limit, `max_live_files`, `cluster_meta_budget`), Wartung starten, `OpenReport`, Zustand bestimmen (§3.4) | | mehr Cluster als `resident_clusters_max` ⇒ `ReadOnly{ClusterMetaBudget}` |

**Beobachtung zu OP2a.** `A.erase_epoch` ist die höchste Erase-Epoche (Zähler der durablen `EraseIntent`/`SpaceDropIntent`, CYC-6 §7); der Catalog trägt sie im `anchor_mark`. Der Anker wird nie vor dem durablen Edit fortgeschrieben (`INV-ANCH-1`); daher ist **Catalog-Epoche ≥ Anker** der Normalfall und die Umkehrung ein Rollback. Ein durables Edit **hinter** dem Anker wird vom Anker-Schreiber nachgezogen (Absturzmatrix §4).

### 3.3 Fehlerzuordnung und `OpenReport`

`OpenReport{state, recovered_stamp, tickets_resumed, quarantined, warnings}`. `quarantined` führt Gründe (`MissingFile`, `KeyLost{cluster-count}`, `CorruptFile`), nie ClusterIds im Klartext (SYS-1 §6). `warnings`: `TombKeyLost`, `HostProbeUnknown`, `WeakAnchor`, `LedgerSameDeviceAck`, `OrphansRemoved{n}`.

### 3.4 Zustandsbestimmung (Reihenfolge)

1. Restore läuft ⇒ `RestoreQuarantine`. 2. OP6 mit `KeyUnavailable` ⇒ `LockedKeys`. 3. Catalog-/Ledger-/Redo-Handle bereits vergiftet ⇒ `Poisoned`. 4. `OpenMode::ReadOnly` oder ein Open-Grund aus der Tabelle ⇒ `ReadOnly(Reason)`. 5. `free_space < [diskfull_enter_bytes]` ⇒ `DiskFull`. 6. sonst `Normal`.

### 3.5 Anlegen (`Storage::create`, B1)

1. Validierung (SYS-4 §5) und Prüfung, dass `root` **leer** ist (`InvalidInput{NotEmpty}`); kein Überschreiben.
2. Sentinel `CREATING` schreiben (`sync_all`, `sync_dir`). Solange er existiert, ist das Verzeichnis keine Instanz (`open` ⇒ `InvalidInput{CreateIncomplete}`; Wiederholung von `create` verlangt ein geleertes Verzeichnis; Waisenschlüssel räumt CYC-4 §5.2).
3. `instance_id`, `shard_key`, RootKey, `kek_current()`; `KEYS` schreiben; `create_tomb_epoch(0)`; `FORMAT` (Art-F-Parameter aus `FormatParams`, SYS-4 §2.2) mit `format_mac`.
4. Erster `CATALOG-0` mit leerem Snapshot (`seq 0`), Systemspaces 1, 2, 3 (`space_add` mit den Optionen aus SYS-2 §2), `reserve_*`-Limits, `kek_id`, `CURRENT`.
5. `ledger_dir` anlegen, `LedgerHead.prev_hash = HMAC(ledger_key, "ctxs ledger genesis v1" ‖ instance_id)`.
6. Anker initialisieren (`advance(None → AnchorValue{0, ledger_head, 0})`); schlägt das fehl und die Policy ist `Required` ⇒ `AnchorUnavailable`, keine Instanz.
7. `RESERVE`, Scratch-Slots, erstes Redo-Segment und Standby vorallokieren (`FsCaps.prealloc_reserves_space` geprüft).
8. `sync_all` aller Dateien, `sync_dir` aller Verzeichnisse, **zuletzt** `CREATING` entfernen (`sync_dir`). Rückgabe `InstanceId`.

Absturz vor Schritt 8: kein gültiger Instanzzustand, nichts wird geöffnet.

## 4. Absturzmatrix (normativ; vollständig als Testsuite `crash_sweep`)

| Unterbrechung | Soll |
|---|---|
| vor Redo-`sync` | Change nicht sichtbar |
| nach Redo-`sync`, vor Rückkehr | Change vorhanden; `IdemToken` verhindert Doppelung |
| Run-Bau / vor Edit / vor Unlink | `.building` entfernt bzw. Run verwaist/entfernt (OP9); Redo spielt ein |
| mitten in einer Catalog-Gruppe | gültiges Präfix gilt |
| Rollover (CYC-5 §8.2) | siehe dort |
| Ledger-Intent torn | nur vollständiger valider Intent gilt; Retry findet das Ticket oder legt neu an |
| G1 durabel, G2 fehlt | OP8 rekonstruiert das Gate aus dem Intent |
| G2 durabel, G3/G4/G5 fehlen | Nachzug in OP8; keine irreversible Folge vor Anker |
| Anker durabel, KMS-Aufruf offen | `KeyDestroyIntent` prüfen, `destroy_many`/`probe` idempotent fortsetzen |
| Schlüssel vernichtet, `KeysDestroyed` fehlt | `probe == Absent` bei passendem Intent ⇒ nachholen |
| Redo-Barriere unterbrochen | ER4 neu, idempotent |
| `PurgeBatchEdit` durabel, Unlink offen | Verwaisten-Suche, `unlink_ack` idempotent |
| Edit durabel, Anker alt | Nachzug in OP2a/OP8 |
| Branch-Fork-Frame durabel, Memtable nicht | Replay stellt die Tabelle wieder her |
| Merge-Frame durabel, Source-Zustand nicht | `Merged` aus dem Frame abgeleitet (CYC-8) |
| `DiskFull` beim Vorallokieren | sauberer Fehler, keine Teilschreibung |
| Checkpoint abgebrochen | `dst` unvollständig ohne Siegel; Quelle unverändert |
| Crash zwischen IG-2 und IG-3 (CYC-7 §4) | Staging unerreichbar; Orphan-Worker räumt auf |
| `create` vor Schritt 8 (§3.5) | `CREATING` bleibt; keine Instanz |
| Restore vor `RestoreReconciled` (§6) | wieder `RestoreQuarantine` beim nächsten Open; Schritte idempotent |

Ehrliche Grenze: ein beschädigter, letzter quittierter Gruppenabschnitt hinter dem letzten `redo_mark`, dem nichts Gültiges folgt, ist von einem Torn Write nicht unterscheidbar; die Heilung kürzt dann. `redo_mark` hält das Fenster klein.

## 5. Close (`Storage::close(mode)`, B1)

`CloseMode::Clean | Fast` (DEC-85).

**Clean** (Voraussetzung für Format-Upgrade, SYS-2 §9 Nr. 4; Zustand ≠ `Poisoned`): (1) Zustand `ReadOnly{Closing}`: neue Mutationen verweigert, Credits laufen leer; (2) Syncer schreibt die letzte Gruppe, `sync_data`; (3) alle Memtables werden geflusht (Barriere, CYC-5 §3); (4) Edit `replay_floor = Ende des Redo`, `anchor_mark`; (5) Ledger-/Anker-Schreiber leeren die Kanäle, Anker fortschreiben; (6) laufende Wartungsjobs abbrechen (ihre `.building`-Ausgaben entfernen); (7) alle Views widerrufen; (8) `sync_dir` aller Verzeichnisse, Sperren lösen. **Sauber geschlossen** ⇔ `replay_floor == Redo-Ende` ∧ `anchor == Catalog`. Offene Löschtickets bleiben offen und laufen beim nächsten Open weiter; Clean verlangt **nicht** deren Abschluss.

**Fast:** Schritte (1), (2), (5), (6), (7), (8) ohne (3)/(4); das Redo bleibt zum Replay. Ein `Drop` ohne `close` ist einem Absturz gleichwertig.

Ein Fehler in `sync_*` während des Close ⇒ `CloseReport{clean: false}` und Poison des Handles (F-3); Daten bleiben über das Redo wiederherstellbar.

## 6. Checkpoint und Restore

### 6.1 Kern-Checkpoint (`Checkpoints::create`, B1)

1. Flush-Barriere (CYC-5 §3); 2. kurz unter `catalog_edit` (Rang 3): `Version` pinnen, `erase_epoch` und Ankerwert lesen; 3. unabhängige `Copy`-Dateien aller Dateien der gepinnten `Version` (`Reflink`/`Hardlink` nur mit registriertem Checkpoint-Owner, der vor Scratch-Recycling Referenzen entfernt und attestiert; sonst Restrisiko, Disclosure) plus ein **Catalog-Snapshot** des gepinnten Zustands (eine Datei mit einem Snapshot-Record); 4. Manifest siegeln (SYS-2 §5.10: `instance_id`, `created_utc`, `erase_epoch`, `tomb_epoch_open`, `catalog_epoch`, `open_tickets_digest`, Dateiliste, `retired_not_purged_plain_clusters`). `KEYS` ist enthalten; **KEK und Clusterschlüssel nicht** (`INV-KEY-1`); das Ledger liegt nie im Checkpoint. Der Checkpoint ist ein konsistenter Schnitt bei `Version` V; Frames nach V gehören nicht dazu. Ein `Retired`, noch nicht gepurgter Plain-Cluster steht im Manifest (Disclosure SYS-5 §8 Nr. 2).
Abbruch ⇒ `ckpt/<name>/` ohne Siegel (`MANIFEST` fehlt), wird beim nächsten Open entfernt.

### 6.2 Systemweiter Checkpoint (`contextra-checkpoint`)

Koordination über `SnapshotParticipant` (`prepare`, `capture`, `release`): (1) `prepare` je Teilnehmer: Kern hält einen `Hold`, Index-Teilnehmer pinnen ihre Generationen (`EpochPinRegistry`); (2) Kern-Checkpoint (§6.1) bei V; (3) jeder Index-Teilnehmer liefert `(partition, generation, applied_stamp ≤ V)`; (4) das Manifest des Snapshot-Sets wird per `Mac`-Port beglaubigt und in einem `NonPersonal`-System-Space der Engine abgelegt; (5) `release`. Benannte `SnapshotSet`s sind lease-begrenzt (`[snapshot_set_lease_max]`) und halten einen `Hold` (`[max_hold_ttl]`, verlängerbar). **Erase überstimmt jeden Pin** (FIX-08): wird ein Cluster gegatet oder eine Generation zur Vernichtung freigegeben, wird das betroffene Set widerrufen (`SnapshotRevoked`); ein Pin hält nie einen Generationsschlüssel am Leben (RACE-25).

### 6.3 Restore (`Checkpoints::restore`)

Restore **ersetzt** die bisherige Instanz und erhält die `InstanceId` (NZ-19). `RestoreConfig{checkpoint, root, ledger_dir, fencing: FencingEvidence}`.

```rust
pub enum FencingEvidence {
    AnchorAdvanced { observed: AnchorValue, next: AnchorValue },   // CAS gelang: der Anker hat den neuen Besitzer akzeptiert
    OperatorAttested { statement_digest: [u8; 32], signatures: Vec<PrincipalSig> },   // FourEyes über Authority; nur Anchor::Weak
}
```
**Fencing (DEC-86).** Schreibend öffnen darf ein Restore erst nach Nachweis, dass der bisherige Besitzer nicht mehr schreiben kann: mit `Strong`-Anker durch einen gelungenen `advance(expect = observed, next)`, wobei `next.catalog_epoch > observed.catalog_epoch`; jeder spätere Fortschreibeversuch des alten Besitzers scheitert an der CAS-Prüfung. Mit `Weak`-Anker (kein Cross-Host-Schutz) nur `OperatorAttested` mit mindestens zwei verschiedenen Principals, geprüft über `Authority::verify` (das Receipt weist `WeakAnchor`). `Strict` verlangt `Strong`.

Schritte (alle idempotent, Zustand `RestoreQuarantine`, jeder Schritt durabel markiert im Ledger `RestoreStep`-frei über Catalog-Edits, kein Nutzer-Read):
1. Manifest-MAC prüfen (RootKey aus der Kopie-`KEYS` über KEK), `instance_id` gleich dem Ledger (`LedgerUnavailable` sonst).
2. **Vergleich Manifest ↔ Ledger/Anker** (löst den R14-Widerspruch „Manifest hinter Anker“): `manifest.erase_epoch < restore_floor` ⇒ `BackupBeyondHorizon{floor, mark}` (§7); `manifest.erase_epoch > höchste erase_epoch im Ledger` ⇒ `Rollback` (das Ledger ist **älter** als die Daten, falsches oder zurückgesetztes Ledger); `manifest.catalog_epoch < anchor.catalog_epoch` ist **normal** (die Kopie ist älter) und kein Fehler. Die Kette des Ledgers und der Anker (wie OP2a) werden geprüft.
3. Dateien wiederherstellen; Catalog laden; die **Catalog-Epoche wird auf `anchor.catalog_epoch + 1` gehoben** (ein Edit) und der Anker CAS-fortgeschrieben (zugleich `FencingEvidence::AnchorAdvanced`); damit gilt die OP2a-Regel (Catalog ≥ Anker) wieder.
4. **Intents scope-genau einspielen (`INV-ERA-9`):** für jeden Ledger-Intent mit `erase_epoch > manifest.erase_epoch` **und** für jeden Intent einer nicht retired Epoche (Backups können ältere Intents nicht kennen): Cluster-Intents über HMAC-`tomb_id` (`Cluster × aktive Epochen`, H-23) auf Cluster der Kopie abbilden und gaten; Range-/Branch-Intents über `scope_ciphertext` (`LedgerScopeKey`) entschlüsseln und als Masken auf alle wiederhergestellten Versionen anwenden; `SpaceDropIntent` über `space_tomb_id` gaten. Wiederhergestellte, bereits einmal gepurgte Cluster erhalten **neue interne Tickets** (`origin = Normal`, Level des Originalintents, Frist `[purge_deadline_min]` ab Restore) und werden erneut gepurgt. Ledger `RestoreReconciled{backup_mark, applied_intents}`.
5. **Schlüssel:** ein Sealed-Cluster der Kopie mit `probe == Absent` und passendem Intent bleibt `Shredded`; ohne passenden Intent `KeyLost` ⇒ Restore bleibt quarantined. `KeyUnavailable`, `KeyLost`, fehlender `LedgerScopeKey`, fehlendes Fencing oder Ledger-Mismatch ⇒ Quarantäne bleibt; ein fehlender Schlüssel/Anker ist nie eine Löschung.
6. Offene KeyDestroy-/Purge-/Extension-Stufen fortsetzen; Index-Generationen ab `replay_floor`/`durable_stamp` neu aufbauen (CYC-7 §5).
7. Erst nach `RestoreReconciled` und abgeschlossenem OP8: Zustand `Normal`.

**Ehrliche Grenze:** wird zusätzlich der Anker-Dienst zurückgesetzt, ist der Schutz nur noch kryptografisch (SYS-5 AT-5).

## 7. Backup-Horizont und Tomb-Retire

Problem: Restore gleicht Backups über `tomb_id` ab. Ewig auflösbare Pseudonyme machen das Ledger dauerhaft personenbezogen; zu früh anonymisierte lassen alte Backups Gelöschtes beleben.

- **`erase_epoch`** ist ein Zähler, der mit jedem durablen `EraseIntent`/`SpaceDropIntent` (G1) um 1 steigt, im Ledger und im `anchor_mark`. `last_erase_epoch(e)` ist der Wert des letzten Intents mit `tomb_epoch = e` und wird final, sobald `e` `Closed` ist.
- **`backup_horizon`** = maximale Aufbewahrungsdauer aller wiederherstellbaren Kopien (**P**, sobald Retire aktiv). `Storage::backup_mark() -> BackupMark{instance_id, erase_epoch, created_utc, tomb_epoch}` (B0). Checkpoint-Manifeste tragen sie automatisch; externe Backups MÜSSEN sie mitführen.
- **Standard:** kein Retire (Epoche `0`, `tomb_id` linkbar, Restore gleicht gegen alle Intents ab, kein `restore_floor`).
- **Retire-Bedingungen (alle):** (1) Epoche `Closed`; (2) `now − closed_at ≥ backup_horizon` bei `skew_state = Ok`; (3) kein offenes Ticket, kein `EraseIntent` ohne `Completed`, kein offener Restore mit `tomb_id`s dieser Epoche.
- **Ablauf (Roll-forward):** Ledger `TombEpochRetireIntent{epoch, restore_floor' = max(restore_floor, last_erase_epoch(e))}` durabel → Catalog `restore_floor(restore_floor')` → Anker → `retire_tomb_epoch(e)` ⇒ `Absent` (bei `Disabled`: Ticket-ähnlicher Zustand `Blocked{provider}`, Wiederholung) → Ledger `TombEpochRetired`. Ein Intent ohne Providerbestätigung bleibt `Blocked`.
- **Restore-Regel:** Kopie mit `mark.erase_epoch < restore_floor` ⇒ `Err(BackupBeyondHorizon{floor, mark})`, kein Override. Eine Kopie **ohne** Marke wird abgelehnt, sobald irgendeine Epoche retired ist. Gegen nicht retired Epochen gleicht Restore weiter ab (`[max_active_tomb_epochs]`).
- **Wirkung:** Nach Retire sind alle `tomb_id`s der Epoche anonym (`INV-LED-2`); Kettenprüfung und `verify_receipt` bleiben gültig. Kopien von Tomb-Schlüsseln außerhalb des Providers heben die Anonymität auf; ein `Bounded`-Provider macht Retire erst nach `until` wirksam (CYC-4 §9.1).

## 8. DiskFull (Notbetrieb)

Redo, Catalog und Ledger schreiben nur in **vorallokierte** Bereiche (AN-10); `ENOSPC` tritt nur beim Vorallokieren auf, ohne Teilschreibung, und ist F-2 (kein Poison, `INV-DISK-1`).

1. **Eintritt:** Governor-Prüfung `free_space < [diskfull_enter_bytes]` oder `ENOSPC` beim Vorallokieren. **Austritt:** `free_space ≥ [diskfull_exit_bytes]` (Hysterese).
2. **Wirkung:** `apply` ⇒ `DiskFull`; Flush/Compaction pausieren, sofern sie keinen Platz freigeben; Blob-GC/Scrub pausieren; Consumer: `retain_floor := replay_floor`, Consumer dahinter erhalten `Lagged` (Retention-Platz wird nie gegen Gate/KeyDestroy gehalten, CYC-7 §4). Lesen, Gate, KeyDestroy, Purge laufen weiter.
3. **Reserve:** `RESERVE` hat `[erase_reserve_bytes]`; Inhalt (SYS-4 §5 Nr. 2): Scratch-Pool (`[purge_scratch_reserve_bytes]`, zwei Slots), Notsegmente für Redo, Catalog, Ledger (Gate-/Control-/Ledger-Frames passen hinein, `INV-LOG-7`) und Dateisystem-Headroom. **Nutzung:** Lane `urgent` gibt Platz durch `set_len`-Verkleinerung von `RESERVE` frei und legt **sofort** das Segment/den Slot an (sequenziell, nie unter einer Sperre); ein Fehlschlag ist ein Betriebsalarm (`erase_reserve_free_bytes`).
4. **Purge unter DiskFull:** Rewrite nur im Scratch-Slot; nach `unlink_ack` der Quelle wird zuerst der Slot wieder vorallokiert, erst danach zählt der freie Platz für den Austritt.
5. **Voraussetzung:** `FsCaps.prealloc_reserves_space` (auch unter CoW/Quota). Fehlt sie, verweigert `open` den harten DiskFull-Erase-Modus oder weist das Restrisiko im Receipt aus (SYS-4 §5 Nr. 5).
6. `DiskFull` wird nicht persistiert; jedes Open wertet neu aus.

## 9. Poison, ReadOnly, Handle-Klassen

| Handle | Poison durch | Folge |
|---|---|---|
| Redo | `sync_*`/`EIO` | Instanz `Poisoned`: Gruppe und spätere `Indeterminate`/`Poisoned` (`INV-PIPE-2`); Löschaufnahme als Intent (CYC-3 §9.4) |
| Catalog | `sync_*`/`EIO` | alle Edits blockiert: Instanz `Poisoned`, Tickets `Blocked{catalog}`; bereits gegatete Cluster bleiben gegatet |
| Ledger | `sync_*`/`EIO` | Fallback-Ledger einmalig (CYC-3 §9.4 Nr. 5); scheitert er ⇒ `Poisoned{intent: NotRecorded}` |
| Anker-Schreiber | Fehler/Unerreichbar | Tickets `Blocked{anchor}`, keine irreversible Stufe; Instanz bleibt `Normal` |
| Run/Blob/Segment | `EIO` beim Lesen/Schreiben | nur diese Datei vergiftet (F-4/F-3); kein Instanzzustand |

Ein vergiftetes Handle wird **nie** auf demselben Handle wiederholt (`INV-FSYNC-1`). Der einzige Heilungsweg ist `close(Fast)` und Open (Redo-Replay, OP6). Ein `Poisoned`-Instanzzustand hebt sich nie selbst auf.

## 10. Parameter

SYS-4: `diskfull_enter_bytes`, `diskfull_exit_bytes`, `erase_reserve_bytes`, `purge_scratch_reserve_bytes`, `anchor_max_age`, `backup_horizon`, `max_active_tomb_epochs`, `max_clock_skew`, `key_probe_interval`, `max_open_files`, `fd_reserve`, `max_live_files`, `redo_max_age`, `snapshot_set_lease_max`, `max_hold_ttl`, `purge_deadline_min`.

## 11. Entscheidungen und geschlossene Lücken

DEC-85 (Close-Modi, Definition „sauber geschlossen“), DEC-86 (Restore: Fencing-Nachweis, Manifest↔Ledger-Vergleich, Catalog-Epoche über dem Anker, Re-Purge wiederhergestellter Gelöschter), DEC-89 (Open-Zustandsreihenfolge, `LockedKeys` mit Hintergrundwiederholung, KEK-Ausfall ⇒ keine Instanz, `create` mit `CREATING`-Sentinel); GAP-49 (Anlegen und Abbruch von `create` sowie `LockedKeys` bei KEK-Ausfall nicht definiert), GAP-50 (Close-Modi und „sauberes Close“ fehlten; Migration verlangte es), GAP-51 (Restore-Reconcile: Fencing-Nachweis und Vergleich „Manifest hinter Anker“ widersprüchlich) (SYS-7 §3).

## 12. Abnahme

`open_op_order` (jeder Schritt OP1…OP10 einzeln, Fehlerpfade), `open_never_weaker` (Anker/Ledger/KEK), `open_rollback_detect`, `open_ledger_ahead_of_anchor`, `open_gate_before_view` (O-Gate), `open_readonly_masks_pending_erase`, `open_locked_keys_retry`, `create_crash_sweep`, `close_clean_empty_replay`, `close_fast_replay`, `upgrade_requires_clean_close` (`MigrationNeedsCleanClose`), `crash_sweep` (Matrix §4, Absturz an jedem E/A-Index, `FaultFs`), `checkpoint_cut_consistent`, `checkpoint_excludes_keys_and_ledger`, `restore_orders_intents` (O-Restore), `restore_fencing_required`, `restore_manifest_ledger_compare`, `restore_repurges_deleted`, `restore_quarantine_on_keylost`, `backup_beyond_horizon`, `tomb_retire_flow`, `diskfull_enter_exit_hysteresis`, `diskfull_erase_runs` (INV-DISK-1), `reserve_release_and_allocate`, `poison_no_retry_same_handle`, `poison_close_fast_replay`. Modelle: M4, M6. Simulation: RACE-5, 12, 25. Orakel: O-Restore, O-Gate, O-Device. Messung: H-13, H-23. Gate G-E.

**Offene Punkte (extern):** `Strong`-Anker-Dienst und Betriebsprozess für externes Fencing des alten Besitzers; Backup-Prozess muss `BackupMark` mitführen.
