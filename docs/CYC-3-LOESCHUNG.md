# CYC-3 — Löschzyklus (Ticket, ER0–ER7, Fristen, Restriction/Link, Receipt, Retention)

Ersetzt R14 §8 (ohne §8.4 Schlüssel, siehe CYC-4) und §11.14 Nr. 3 (Retention). Abhängigkeiten: SYS-1 (Ränge, Fehler), SYS-2 §5.3/§5.7/§5.11 (Gate-, Ledger-, Receipt-Format), SYS-3 §6, SYS-4 §2.5/§6, SYS-5 §8, CYC-1 (Sequencer), CYC-4 (Schlüssel), CYC-5 §6 (Purge-Ausführung), CYC-6 (Open, Restore). Rang: gehört zu den Spezifikationen höchsten Vorrangs nach SYS-7 (Index §1). Gate: **G-E**.

## 1. Zweck und Grenzen

Ein Löschauftrag macht Daten eines Clusters (oder Bereichs, Branches, Spaces) in geordneten, unabhängigen, absturzfesten Schritten unlesbar und entfernt sie physisch. Der Kern sagt nie mehr, als die erreichte Stufe trägt (Z-6). Der Kern entscheidet nicht, **ob** gelöscht werden darf (Politik liegt in `engine`, SYS-5); er verweigert nur bei technischem Hindernis oder aktiver Restriction.

**Kernsätze (Garantien).**
1. Ab durablem Gate liefert kein `view`/`page`/`poll` Inhalt des Clusters und kein Commit schreibt hinein; voller Snapshot oder `Erased`, nie Mischsicht (`INV-ERA-1`).
2. Nach `KeysDestroyed` ist alles des Clusters im Kern (Redo, Runs, Blobs, gespeicherte Checkpoints) ohne Clusterschlüssel unlesbar (`INV-ERA-7`). Extension- und Backup-Kopien sind nur erfasst, wenn ihr registrierter Schlüsselscope an dieselbe Vernichtung gebunden ist; sonst bleibt das Gesamt-Receipt `Incomplete`.
3. **„Bytes von H“:** Nutzdaten, Ciphertext, Schlüsselmaterial und `ClusterDirectory`-/Block-/Record-Einträge von H in Runs, Blobs, Segmenten und Redo-Payload. **Nicht** dazu zählen Routing-Metadaten (ClusterId in Frame-Köpfen, Gate-/Control-Frames, Ledger-`tomb_id`); sie sind pseudonyme Metadaten (SYS-5 §8 Nr. 6) und verschwinden mit dem Redo-Segment.
4. Nach `Purged` enthält keine lebende Datei Bytes von H (`INV-ERA-2`).
5. Gate/KeyDestroy skaliert mit der Zielmenge (zuzüglich Providerlatenz); Purge bearbeitet nur Dateien mit Zielcluster (`INV-ERA-11`). `ClusterLocal` vermeidet fremde Survivor-Rewrites in Runs/Blobs, nicht im gemeinsamen Redo.
6. **Korpus-Löschung ist kein Erase:** das Löschen eines `NonPersonal`-Dokuments aus dem Korpus ist ein normales `Delete` plus Index-Tombstone und erzeugt kein Receipt. Nachweispflichtige Löschung braucht eine eigene Löschdomäne (Cluster).

## 2. Akteure, Stufen, Zustandsautomat

| Akteur | Aufgabe |
|---|---|
| Aufrufer (`Eraser`) | ER0, G1, Gate-Einreichung (Aufruferthread, Provideraufrufe erlaubt) |
| Sequencer | Gate-Frame, `Retiring`, Hüllenprüfung, Link/Restrict/Lift |
| Ledger-Writer | ein Thread, eigene Failure Domain, gebündelt |
| Anker-Writer | ein Thread, koalesziert (`anchor_max_age`) |
| Lane `urgent` | G3-Nachzug, Speicherfreigabe, ER2, Plain-Purge, Notpool |
| Lane `compact`/`flush` | Sealed-Purge (eskaliert, §6) |
| `SegmentOwner` | Attestierung (CYC-7 §3) |

```text
EraseLevel (Ziel): Logical | Physical | Scrubbed | Cryptographic
Stage:  IntentRecorded → Gated → KeyDestroying{eta} → KeysDestroyed → Purging → Purged → Done
Orthogonal (Flags, keine Stage):  Blocked{owner|anchor|ledger|provider|…, since, retry_at}, Overdue, Incomplete
Terminal: Done | Failed (Integritäts-/Protokollfehler)
```
`KeyDestroying` und `KeysDestroyed` entfallen bei Zielen ohne Schlüsselvernichtung (`Logical`, `Physical`, `Scrubbed`); der Ticketstand springt von `Gated` nach `Purging` bzw. nach `Done` (`Logical`). `Purging` bei `Logical` entfällt.

**Zielmatrix (FIX-04, normativ).**

| Operation | `Sealed` | `Plain` |
|---|---|---|
| Cluster-Ticket | `Logical`, `Cryptographic` | `Logical`, `Physical`, `Scrubbed` |
| `range`, `branch` | `Logical` (Crypto-Anforderung wird als Cluster-Erase ausgeführt, Receipt weist das aus) | `Logical`, `Physical`, `Scrubbed` |
| `space_drop` | `Logical` | `Logical`, `Physical`, `Scrubbed` |
| `Cryptographic` auf Space-Ebene | `Unsupported` | `Unsupported` |

Alles andere ⇒ `Unsupported` vor Mutation. `Cryptographic` bei `KeysInBackups::Yes` ⇒ `Unsupported`; für Sealed bleibt dann nur `Logical`, und `Standard`/`Strict` verbieten diese Provider-Konfiguration für `Personal` (SYS-5 §4, P-2). `Logical` ist als Endziel zulässig (z. B. unter Restriction-Fristen); das Receipt lautet dann `CompleteWithResidual{LogicalOnly}`, das Ticket bleibt per `escalate` eskalierbar. Das Receipt spricht nie von „unwiderruflich“ oder „konform“.

**Idempotenz.** Jeder Auftrag trägt eine zufällige `EraseRequestId` (unabhängig vom 24-h-`IdemToken`). Das Ledger persistiert nur `request_id_hmac` und `request_digest` (`HMAC(ledger_key, …)` über die kanonische, versionierte, tag-/längencodierte Anforderung; Fristen zählen nicht semantisch). Identischer Retry ⇒ dasselbe Ticket; abweichender Scope mit derselben ID ⇒ `IdemConflict`. Ein Retry nach Absturz des Aufrufers findet das Ticket über `request_id_hmac` (FIX-25).

## 3. Ablauf: Protokoll `Eraser::cluster`

Das **logische Gate** ist der durable Redo-Gate-Frame (G2). Alle Catalog-, Ledger-, Anker-Schritte folgen in fester Ordnung. Jede Stufe ist crash-idempotent (Roll-forward, `INV-ERA-4`).

```
ER0  Vorprüfung (Aufruferthread, ohne Sperre)
G1   Ledger EraseIntent durabel
G2   Gate-Frame im Redo                            ◄── LOGISCHER PUNKT
G3   Catalog-Edit {cluster_state(H→Retired,S), erase_pending_add}
G4   Ledger GateApplied
G5   Anker-CAS
ER2  (Cryptographic) KeyDestroyIntent → destroy_many → probe == Absent → KeysDestroyed → Shredded   ◄── KRYPTO-PUNKT
ER3  Plan
ER4  Redo-Barriere
ER5  Umbau (PurgeBatchEdit je Datei/Blob-Inkarnation, Unlink, UnlinkAck)
ER6  Abschluss-Edit {Purged, erase_done, anchor_mark}                                             ◄── PHYSISCHER PUNKT
ER7  Ledger Completed, Anker, ErasureReceipt
```

**Reihenfolge ist Vertrag (`INV-ERA-3`):** G1 < G2 < G3 < G4 < G5 < `KeyDestroyIntent` < `destroy_many` < jedes `PurgeBatchEdit` < Unlink+`sync_dir` < `UnlinkAck` < ER6/`Done`. Bei `Cryptographic` beginnt der Purge nie vor `probe == Absent` (bis dahin nur Planung und Scratch-Reservierung).

### 3.1 ER0 Vorprüfung

1. Cluster existiert (`UnknownCluster`); schon `Retired` ⇒ vorhandenes Ticket liefern.
2. Hülle `H = {c} ∪ transitiv DerivedFrom-gebunden`, berechnet auf dem Aufruferthread gegen `policy_epoch P1`; `|H| > [max_gate_clusters]` ⇒ `EraseFanoutExceeded`.
3. `BLOCK_ERASE` in `H` ⇒ `Err(Restricted)` (`INV-ERA-10`).
4. Level passt zur Protection (Zielmatrix), `info()` des Providers erlaubt `Cryptographic`, Frist-Konsistenz (§6), `purge_deadline ≥ [purge_deadline_min]` (`INV-DL-1`).
5. Profil/Ack-Prüfung (`ProviderAssuranceInsufficient`).
6. Ledger beschreibbar? Nein ⇒ `LedgerUnavailable`; gleiches Gerät ohne Ack ⇒ `LedgerSameDevice`; Anker `Required` unerreichbar ⇒ Ticket wird nicht abgelehnt, sondern bleibt nach G2 `Blocked{anchor}` (keine irreversible Stufe vor Anker); Ablehnung nur bei bekannter Dauerstörung (`AnchorUnavailable` vor G1).

### 3.2 G1 `EraseIntent`

Ledger-Record `EraseIntent` (SYS-2 §5.7, Kind 1) mit `tomb_id`s (≤ `[max_gate_clusters]`, sortiert, 32 B), `tomb_epoch`, `origin`, `purge_deadline_ns`. Ab hier verpflichtet der Intent zur Ausführung, auch wenn der Aufrufer kein Ticket erhielt. Ein Gate ist nur zulässig, wenn sein Intent durabel ist (`INV-LED-3`).

### 3.3 G2 Gate-Frame, Hüllenwachstum, `link_fence`

Gate-Frame (`n_ops = 0`, Body: `scope, ticket, level, policy_epoch_hull = P1, n, cluster[16]×n`). Der Sequencer vergleicht nur `policy_epoch == P1` (RAM), setzt `Live → Retiring(S)`, vergibt den Stamp. Nach Sync und Publikation `Retired(S)`, `gen++`. **`Ok(ticket)` wird erst jetzt (oder bei vergiftetem Redo als `IntentRecorded`) geliefert; ab hier nie mehr ein Fehler.**

**Hüllenwachstum zwischen G1 und G2 (FIX-29).** Änderte sich `policy_epoch` (neuer Link/Restrict/Lift), antwortet der Sequencer `PolicyRetry`. Der Aufrufer berechnet `H' ⊇ H`; ist `H' ⊄ H_intent`, hängt er `EraseIntentExtend(H' ∖ H_intent)` an (G1′, durabel) und reicht den Gate-Frame mit dem neuen `P1′` erneut ein. Nach `[policy_retry_max]` erfolglosen Versuchen gilt das **`link_fence`-Protokoll** (DEC-75):

1. Der Aufrufer fordert `begin_fence(ticket)` an (Sequencer, Rang 1, nur RAM): ab jetzt werden neue `Link`/`Restrict`/`Lift`-Frames **zurückgestellt** (sie erhalten `PolicyRetry`), höchstens `[link_fence_ttl]`. Der Sequencer berechnet **einmalig** die Hülle `H_f` (≤ `[max_gate_clusters]` Knoten, nur RAM, ≤ 250 µs, H-24) und die aktuelle `policy_epoch P_f` und gibt beide zurück.
2. Der Aufrufer hängt (außerhalb Rang 1) `EraseIntentExtend(H_f ∖ H_intent)` und `LinkFence{ticket, hull_digest}` an das Ledger (durabel).
3. Er reicht den Gate-Frame mit `P_f` und Flag `MERGE`-frei ein. Der Sequencer prüft `policy_epoch == P_f` (durch die Fence unverändert), `hull_digest` und gatet. Danach endet die Fence.
4. Läuft `link_fence_ttl` ab, bevor das Gate sequenziert wurde, verfällt die Fence; der Aufrufer beginnt von vorn (höchstens einmal je `[link_fence_ttl]`). Es gibt höchstens eine aktive Fence; weitere Anfragen warten (`urgent`-Credit, geordnet nach Ticket-ID, RACE-8).

Damit gatet der Sequencer **nie** eine nicht vom Intent gedeckte Hülle (`INV-LED-3`), und es gibt weder E/A noch Providerzugriff unter Rang 1.

### 3.4 G3–G5

- **G3** Catalog-Edit (`cluster_state(H→Retired,S)` + `erase_pending_add`, ein Edit) ist Voraussetzung dafür, dass `replay_floor` den Gate-Frame überschreitet (`INV-LOG-5`). Asynchron (`urgent`): Memtable-Einträge der Cluster freigeben/überschreiben, Cache-Puffer überschreiben (`Zeroizing`).
- **G4** Ledger `GateApplied{ticket, S, gate_utc}`.
- **G5** Anker-CAS. Ist ein `Required`-Anker nicht erreichbar, bleibt das Ticket `Blocked{anchor}`; keine irreversible Stufe (KeyDestroy, Purge-Barriere) beginnt.

### 3.5 ER2 Schlüsselvernichtung (nur `Cryptographic`)

Ledger `KeyDestroyIntent` durabel → `KeyProvider::destroy_many(H)` (Batches ≤ 256, alle Epochen) → nur bei `probe == Absent` für **jede** Epoche `e ≤ key_epoch`: Ledger `KeysDestroyed`, Schlüsselcache leeren, `gen++`, Edit `cluster_state → Shredded`. `Disabled` (Karenz) ⇒ Stage `KeyDestroying{eta}`, Ticket `Blocked{provider}`; `KeyUnavailable` ⇒ Retry mit Backoff; fehlender Schlüssel ohne Intent ⇒ `KeyLost` (Quarantäne, `Incomplete`). Details und Zustände: CYC-4 §4.

### 3.6 ER3 Plan, ER4 Redo-Barriere, ER5 Umbau, ER6, ER7

- **ER3** Dateien über Packgruppen-Index/ClusterDirectory (Staging eingeschlossen), Claims (Rang 5), Segmente mit Überlappung (`SegmentOwner`), Redo-Segmente mit Payload von H.
- **ER4/ER5** siehe §5.
- **ER6** nach Verify und Extension-Attesten **ein** Edit `{cluster_state(H→Purged), erase_done, anchor_mark}`.
- **ER7** Ledger `Completed`, Anker fortschreiben, `ReceiptIssued`; Graveyard-Dateien ohne Zielnutzdaten werden separat nach ihren Reader-Pins entfernt.

**Done-Bedingungen (`INV-ERA-8`):** (1) keine lebende Datei/Segment/Inkarnation trägt Bytes von H, (2) Memtable-Einträge freigegeben, (3) Ledger durabel, (4) Anker fortgeschritten, (5) alle Segmentbesitzer attestiert, (6) bei `Cryptographic` `probe == Absent`, (7) Redo-Barriere abgeschlossen. `PurgeCompleteEdit` ist unzulässig, solange das Ticket Graveyard-Dateien oder offene `unlink_ack`s besitzt; Recovery leitet nie aus einem Zwischenbatch `Done` ab.

**Ledger-Ereignisse je Schritt:** G1 `EraseIntent`(+`Extend`, `LinkFence`) · G4 `GateApplied` · ER2 `KeyDestroyIntent`, `KeysDestroyed`/`KeyLost` · ER5 `PurgeProgress`, `UnlinkProgress`, `ExtAck`/`ExtBlocked` · ER6/ER7 `PurgeVerified`, `Completed`, `ReceiptIssued`; Zusatz: `TicketBlocked`/`Resumed`, `DeadlineBreached`, `Escalated`.

## 4. Fehler- und Absturzverhalten je Schritt

| Unterbrechung | Soll |
|---|---|
| ER0 | Ablehnung, nichts persistiert |
| G1 torn | nur vollständiger valider Intent gilt (Ledger-Kette); Retry findet das Ticket oder legt neu an |
| G1 durabel, G2 fehlt | OP8 (CYC-6 §3) rekonstruiert das Gate aus dem Intent (`origin` unverändert), vor jeder Nutzer-Sicht |
| G2 durabel, G3/G4/G5 fehlen | Nachzug vor Open; keine irreversible Folge vor Anker |
| Anker durabel, KMS-Aufruf offen | `KeyDestroyIntent` prüfen, `destroy_many`/`probe` idempotent fortsetzen |
| Schlüssel vernichtet, `KeysDestroyed` fehlt | `probe == Absent` bei passendem Intent ⇒ nachholen |
| Redo-Barriere unterbrochen | ER4 neu, idempotent |
| `PurgeBatchEdit` durabel, Unlink offen | Verwaisten-Suche, `unlink_ack` idempotent |
| Edit durabel, Anker alt | Nachzug in OP2a (CYC-6 §3) |
| Fehler im Ledger-Writer (Sync) | Ledger-Handle `Poisoned`; Fallback-Ledger (§9 Nr. 5); bis dahin Ticket `Blocked{ledger}`, bereits gegatete Cluster bleiben gegatet |

`Poisoned` Redo nach G2 betrifft nur neue Gate-Frames, nicht den Fortgang eines bereits gegateten Tickets: ER2…ER7 sind unabhängig vom Redo-Handle, solange Ledger, Catalog und Anker funktionieren; ein vergifteter Catalog-Handle blockiert alle Edits ⇒ `Blocked{catalog}` bis Neustart.

## 5. Redo-Barriere und Purge

### 5.1 Redo-Barriere (FIX-02)

Redo-Segmente mit Payload von H werden **nicht** durch Segmentschnitt unter Rang 1 abgebaut, sondern durch einen Wartungsjob: Segment rotieren, Memtables flushen, `replay_floor` und `retain_floor` hinter den Schnitt setzen, ältere Segmente in den Graveyard; Consumer mit Position davor erhalten `Lagged`. Eine Barriere bedient alle fälligen Tickets. **Plain** und Space-Drop: sofort bei ER4. **Sealed:** zeitversetzt und gebündelt, spätestens bei `[purge_trigger_fraction]` der Frist; der Ciphertext im Redo ist ab `KeysDestroyed` unlesbar (`INV-ERA-7`), `Done` verlangt dennoch die Entfernung (`INV-LOG-9`). Obergrenze der Dauer ≈ `[redo_max_age]` plus Flushzeit (H-14).

### 5.2 `purge(plan, D)`

```text
für jede Datei/Blob-Inkarnation F einzeln:
   keep = Cluster(F) − D
   keep = ∅  → PurgeBatchEdit{files_remove(F)}                       // legt Graveyard-Eintrag an
   sonst     → F' im vorallokierten Scratch-Slot roh kopieren (ohne die Abschnitte aus D);
               sync_all, rename, sync_dir; PurgeBatchEdit{files_add(F'), files_remove(F)}
nach Reader-Drain: unlink(F); sync_dir; unlink_ack(F); erst dann Scratch-Slot auffüllen
nach allen Dateibatches: SegmentOwner attestieren; PurgeCompleteEdit setzt Purged/Done
```
Rohkopie prüft Frame-CRCs, **nie** das AEAD-Tag (kein Entsiegeln). Pro Einzelschritt ist Scratch ≤ eine maximale Datei; zwei Slots decken Rewrite plus Crash-Slot. Vollständig tote Dateien brauchen keinen Scratch. **Purge DARF NICHT auf `punch_hole` angewiesen sein.** Der Algorithmus gilt gleich für Pure, Packed, Staging, Packgruppen-Abwurf (`D` = alle nicht-Restricted-Cluster der Gruppe), Mandantenabgang (`D` = alle Cluster des `tenant_tag`) und Blob-Inkarnationen (`INV-BLOB-1`: nur entfernen, wenn kein lebender Run mehr einen `PutRef` darauf enthält). Verfahren bei Scrub (`Scrubbed`): Quellbereich vor dem Unlink überschreiben (best-effort; unter CoW/SSD nicht beweisbar, Disclosure).

**Bündelung (Amortisierung).** Je Fenster `W = [purge_window_fraction] × D` wird eine Datei höchstens einmal umgeschrieben; Purge-Bytes je `W` ≤ Σ Größen der betroffenen Dateien (H-28). Auslöser (das Erste, was eintritt): (a) Compaction/Blob-GC schreibt die Datei ohnehin um und lässt Retired-Cluster weg; (b) `[purge_trigger_fraction]` der Frist verbraucht; (c) ≥ `[purge_batch_min]` Cluster in derselben Datei. Dringlichkeit: Plain ⇒ Lane `urgent`; Sealed ⇒ `compact`, Eskalation auf `flush` bei `[sealed_purge_escalate_flush]`, `urgent` bei `[sealed_purge_escalate_urgent]`.

**Notbetrieb:** DiskFull-Verhalten (Scratch-Pool, Reserve): CYC-6 §8.

## 6. Fristen

1. `purge_deadline` ist Pflichtparameter, `≥ [purge_deadline_min]`. Das Ticket speichert Deadline, monotone Dauer und `clock_floor`.
2. **Frist-Konsistenz (FIX-17, FIX-30, `INV-DL-1`):** `max_delay(KeyProvider) ≤ [key_destroy_fraction_max] × D`, sonst `Err(DeadlineBeforeKeyDestroy)`. Das Receipt weist `t_crypto` und `t_physical` getrennt aus.
3. **Uhr:** ein UTC-Sprung außerhalb `[max_clock_skew]` friert die Deadline-Berechnung ein (`ClockUncertain`, `skew_state = Uncertain`, Alarm, `time_trust = Uncertain` im Receipt). Der Fristablauf wird dann nach monotoner Zeit gerechnet, nicht nach UTC.
4. **Überschreitung (DEC-78):** eine echte, zeitlich verifizierte Überschreitung setzt das Flag `Overdue`, den Zähler `erase_deadline_breaches_total`, Ledger `DeadlineBreached`, Alarm; das Ticket läuft in Lane `urgent` **weiter** (anders als R14, das `Blocked` verlangte: ein Stopp bei Überschreitung wäre kontraproduktiv). Das Receipt trägt Statuscap `DeadlineBreached`.
5. **Views:** `ViewPolicy::Wait(t)` (Unlink wartet höchstens `min(t, Lease-Ende)`; die logische Garantie gilt bereits) oder `Revoke` (Views widerrufen, `revoke_grace`).

## 7. Restriction, Hold, Link

### 7.1 Restriction

`Restriction{ reason u16 ≠ 0, owner[16], effects (BLOCK_ERASE immer | BLOCK_WRITE | BLOCK_READ), expires }`. Ablauf lässt nur den **Vermerk** fallen und löst nie Löschung aus; Auswertung nur bei `skew_state = Ok` (bei `Uncertain` bleibt der Vermerk, fail-closed). Kein Override im Kern (`DEC-12`). `restrict` darf jeder Handle-Besitzer; `lift` nach `LiftPolicy`. `restrict`/`lift`/`link` laufen als Control-Frame im Sequencer, stampgeordnet mit dem Gate (`INV-ERA-10`, `INV-POL-1`). Durchsetzung dreischichtig: Aufruferthread + Sequencer-Revalidierung (`policy_epoch`), Purge verweigert Umbau, `cluster_state`-Edit verweigert `Retired`. `BLOCK_READ` ⇒ `Restricted`; zweckgebundene Lesung nur mit `RestrictedReadCap`.

### 7.2 `lift`

```rust
pub enum LiftAuth { Owner(OwnerId), Admin(AdminCap), FourEyes { requester: PrincipalId, approver: PrincipalId, signature: Vec<u8> } }
```
1. `OwnerOrAdmin` (Standard): `Owner` muss `restriction.owner` entsprechen, `Admin` ist eine nicht klonbare, von der Engine ausgestellte Capability. Der Kern prüft Gleichheit beziehungsweise Vorhandensein, nicht Identität.
2. `FourEyes` (`Strict` verlangt es für Retention-Verlängerung, SYS-5 P-11): `approver ≠ requester`; die Signatur ist über die kanonische Nachricht `"CTXLIFT1" ‖ instance_id ‖ restriction_id u64 ‖ cluster[16] ‖ policy_epoch u64 ‖ nonce[16]` (Bytes, kein Kern-Hash; `Authority::verify(principal, request_digest, signature)` erhält diese Bytes als `request_digest`, DEC-76). Die Signatur steht im Control-Body (`principal_sig`).
3. Replay-Schutz: die Nachricht bindet `policy_epoch`; der Sequencer verlangt `policy_epoch == P1` (sonst `PolicyRetry`) und die Restriction muss noch bestehen. Eine einmal gelöste Restriction ist nicht erneut lösbar.
4. Ablauf: Prüfung auf dem Aufruferthread, Control-Frame `Lift`, Catalog-Spiegel `restriction_lift` vor `replay_floor`, Ledger `RestrictionLifted{tomb_id, reason, effects}`. Ein zurückgestelltes Erase (`Blocked{restricted}`) wird danach automatisch erneut versucht.

### 7.3 Hold

Technisch, `ttl ≤ [max_hold_ttl]`; ein Erase überstimmt jeden Hold ohne Fehler (`INV`-Linie `FIX-08`): betroffene `SnapshotSet`s werden widerrufen (`SnapshotRevoked`).

### 7.4 `link`

`Clusters::link(derived, sources[≤ [max_links]])` ist ein Control-Frame `Link`. **Admission außerhalb Rang 1** gegen `policy_epoch P1`: (a) azyklisch, (b) Tiefe ≤ `[max_link_depth]`, (c) für jeden Vorfahren-oder-selbst `a` jeder Quelle `|Hülle(a) ∪ Hülle(derived)| ≤ [max_gate_clusters]`, (d) keine Quelle `Retiring`/`Retired`. Traversierung bricht bei `[max_gate_clusters] + 1` Knoten ab ⇒ `EraseFanoutExceeded` ohne Mutation. Der Sequencer prüft nur `policy_epoch == P1` und „Quelle `Live`“, setzt die Kante und erhöht `policy_epoch` um 1. Löschung irgendeiner Quelle zieht den Abgeleiteten im selben Gate-Frame mit (`INV-ERA-12`); ein Kind mit mehreren Eltern wird gelöscht, sobald eines gelöscht wird. **Cross-Subject-`DerivedFrom` ist unzulässig** (die Identity-/Policy-Schicht prüft vor dem Link-Commit); gemeinsame Ableitungen brauchen getrennte Löschdomänen oder nicht kaskadierende `Reference`.

## 8. Weitere Löschpfade

- **`range(space, cluster, range)`:** `EraseRange` ist ein clusterweiter Gate-Marker (`gate.scope = 3`), maskiert sofort alle Branches und älteren Views. Plain: `Logical|Physical|Scrubbed` (Physical rewritet alle Branch-/Stamp-Versionen des Bereichs); Sealed: nur `Logical` (keine selektive Schlüsselvernichtung in v1; für physische/kryptografische Entfernung den ganzen Cluster löschen oder den Bereich vorab in eine eigene Löschdomäne legen). `erased_ranges` werden erst getrimmt, wenn `history_floor > erase_stamp` und keine Pins darunter liegen. Der Selektor steht versiegelt im Ledger (`scope_ciphertext`).
- **`branch(cluster, branch)`:** nur terminale leaf-Branches (`gate.scope = 4`); Plain `Logical|Physical|Scrubbed`, Sealed `Logical`; eine Crypto-Anforderung wird als Cluster-Erase ausgeführt.
- **`space_drop(space, request_id, level, deadline)`:** Prüfung der `BLOCK_ERASE`-Restrictions außerhalb Rang 1, Revalidierung per `policy_epoch`. `SpaceDropIntent` durabel → `Gate{Space(id)}` (`scope = 2`) → `GateApplied` → Anker → bei Plain Purge. Die Mitgliedschaft (`SpaceScopeFrozen{count, scope_digest}`) wird **nach** dem Gate aus der gepinnten Version ermittelt, nie unter Rang 1. `SpaceDropped` ist der Abschluss nach allen Attesten. Wiederanlage braucht eine neue `SpaceId` (`INV-ID-2`). Kein Einzel-Edit für beliebig viele Dateien (Purge je Datei).
- **`pack_expired(PackKey)`:** `H` = nicht-Restricted-Cluster der Gruppe, deterministisch (Sortierung nach `ClusterId`) in unabhängige Tickets ≤ `[max_gate_clusters]` zerlegt (eigener Gate-Frame, eigene Ledger-/Receipt-Kette; keine atomare Gesamtlöschung). Restricted-Cluster erscheinen mit Grund im Aggregat und werden nach Lift erneut versucht. Idempotent zu bereits ausgelösten Einzelaufträgen.
- **`escalate(ticket, level)`:** Ledger `Escalated{from,to,cause}`; zulässig nur aufwärts entlang `Logical < Physical < Scrubbed` (Plain) beziehungsweise `Logical → Cryptographic` (Sealed).
- **`open_tickets()`** (nach Absturz des Aufrufers), **`clusters_batch`**.
- **Mehrbetroffene Daten:** gemeinsamer Fallcluster; jede Person hat ihren Wurzelcluster; Link nur, wo die Löschung der Person den Fallcluster mitziehen soll; sonst Referenz. Keine mehrclustrigen Einträge.
- **`Logical`-Bestand:** `logical_only_clusters_aged` und Alarm bei `[logical_only_alarm_age]` (FIX-35).

## 9. Ledger, Anker, Poison-Aufnahme

### 9.1 Ledger

Append-only, vorallokiert (`[ledger_segment_bytes]`), über `Mac` kettenverknüpft, ein Schreiber mit Gruppenbündelung (Format SYS-2 §5.7). Der Ledger enthält nie Klartext-`ClusterId`/`SpaceId`, Nutzerschlüssel, Nutzdaten, Personen-IDs (`INV-LED-1`). `ledger_key`/`LedgerScopeKey` sind aus dem RootKey abgeleitet und nicht separat gespeichert (FIX-09): ein Restore benötigt KEK + `KEYS` aus der Kopie; das Ledger selbst liegt nie in Checkpoints.

### 9.2 Failure Domain und Verfügbarkeit

`ledger_dir` liegt auf einem anderen Gerät als das Datenverzeichnis (Vergleich `FsCaps.device_id` beziehungsweise `HostProbe.device_ids`); sonst `Err(LedgerSameDevice)`, außer `ledger_same_device_ack` (Restexposition im Receipt, nur `Standard`). Ledger beim schreibenden Öffnen nicht lesbar ⇒ `Err(LedgerUnavailable)`; `ReadOnly` nur mit Diagnose.

### 9.3 Anker

`AnchorValue = (erase_epoch, ledger_head, catalog_epoch)`. Fortschreibung nach G3/G4 und **vor** `KeyDestroyIntent` oder Purge-Barriere (blockierend); zusätzlich bei Catalog-Rollover, `close`, spätestens `[anchor_max_age]` bei ausstehendem Zustand (koalesziert). `AnchorPolicy::Required` ist Standard bei Encrypted/Authenticated; ist der Anker nicht fortschreibbar, bleibt das Ticket `Blocked{anchor}`. `Weak` erfordert `acknowledged_weak` und attestiert keinen starken Rollback-Schutz. Referenz: `MemAnchor`, `FileAnchor` (`Weak`).

### 9.4 Löschauftrag auf vergiftetem Handle (`INV-ERA-13`)

Ein `Poisoned` Redo-Handle nimmt keinen Gate-Frame mehr an; die Löschpflicht endet nicht.
1. **Aufnahme:** Request und Restrictions werden gegen den RAM-Zustand geprüft; die Fassade hängt einen `EraseIntent{origin: Poisoned}` an (Tomb-Schlüssel aus dem Cache, **kein** Providerzugriff) und liefert `Ok(Ticket{IntentRecorded})`. Schlägt der Append fehl ⇒ `Err(Poisoned{intent: NotRecorded})`.
2. **Logische Wirkung sofort:** RAM-Menge `pending_erase`; Lesen/Views/`page`/`poll`/Feed liefern `Err(ErasePending)`; Cache-Puffer werden überschrieben. Das ist kein Gate (kein `Erased{at}`).
3. **Neustart:** OP8 (CYC-6 §3) überführt jeden `EraseIntent` ohne `GateApplied` idempotent in ein Gate, vor jeder Nutzer-Sicht.
4. **Keine Abkürzung:** kein KeyDestroy, Purge oder Anker auf dem vergifteten Handle.
5. **Fallback-Ledger (DEC-71):** ist das **Ledger-Handle** selbst vergiftet (Sync-Fehler), eröffnet der Ledger-Writer **einmalig** eine Fallback-Datei `ledger-fallback-<n>.log` mit frischem Handle (anderes Verzeichniseintrag-, gleiche Failure Domain wie `ledger_dir`), deren erste `prev_hash` der letzte **bekannt durable** Kettenwert ist (nicht der Wert des unklaren Endes der vergifteten Datei). Intents dort tragen `origin = Fallback`; ein daraus folgender Gate-Frame trägt das Redo-Flag `FALLBACK_INTENT`, damit OP6/OP8 den Intent in der Fallback-Datei suchen. Der Anker führt den Kopf der aktiven Kette. Beim nächsten schreibenden Open werden die Ketten zusammengeführt: Ledger `FallbackMerged{from_seq, to_seq}` im Hauptstrang, danach gilt eine Kette. Scheitert auch der Fallback-Append ⇒ `Poisoned{intent: NotRecorded}`. Es gibt höchstens eine aktive Fallback-Datei.
6. **Ehrliche Grenze:** Klartextreste im Prozessspeicher bis Prozessende; Observer meldet `poisoned_erase_pending`; das Receipt weist `t_requested` und die Wartezeit aus.

### 9.5 Fehlercodes

| Fehler | Bedingung |
|---|---|
| `LedgerUnavailable` | Ledger weder lesbar noch (Fallback) beschreibbar; schreibendes Open verweigert, `ReadOnly` mit Diagnose |
| `LedgerSameDevice` | gleiche `device_id`, kein Ack |
| `AnchorUnavailable` | `Required`-Anker dauerhaft nicht erreichbar (Open ⇒ `Locked`) oder bei ER0 bekannte Dauerstörung |
| `ProviderAssuranceInsufficient` | Provider unter Profilanforderung ohne Ack |

## 10. Receipt und Status

**`ErasureReceipt`** (Format: SYS-2 §5.11): `{instance_id, ticket, scope_digest, level, t_requested, t_logical, erase_epoch S, t_crypto?, t_physical?, t_verified?, stages[{stage, ledger_seq, evidence_digest}], key_attestations, provider{destroy, keys_in_backups, assurance}, ext_acks, verifier_report_digest, ledger_head, anchor{strength, value}, restrictions_encountered, status, statuscaps, disclosures, residual, time_trust, canonical_bytes}`.

**Status (FIX-03):** `Complete` | `CompleteWithResidual` | `Incomplete` | `Blocked` | `Failed`.

| Status | Bedingung |
|---|---|
| `Complete` | jede für den Level erforderliche Kern- und Owner-Attestation, alle Pflichtmessungen `Complete`, **kein** Statuscap |
| `CompleteWithResidual` | alles erforderliche erfüllt, aber mindestens ein **Statuscap** gilt |
| `Incomplete` | erforderlicher Owner-/Scope-Nachweis fehlt oder eine Pflichtmessung ist `NotMeasured`/`NotImplemented` (`INV-VER-1`); nie eine erfolgreiche Quittung |
| `Blocked` | sicher pausiert, fortsetzbar |
| `Failed` | terminaler Integritäts-/Protokollfehler |

**Statuscaps** (begrenzen auf `CompleteWithResidual`): `LogicalOnly`, `WeakKeyAssurance`, `BoundedBackupKeys{until}` (entfällt bei neu berechnetem Receipt nach Ablauf, solange der Provider `Absent` bestätigt), `WeakAnchor` (bestätigt), `LedgerSameDeviceAck`, `HostKnownExposed`, `DeadlineBreached`, unregistrierte externe Persistenz.
**Disclosures** (immer aufgeführt, **kein** Cap): Katalog SYS-5 §8. Ein Disclosure beschreibt, was der Level laut §2 nicht abdeckt; es macht ein Receipt nicht unerreichbar.
Die Signatur liegt oben (`engine::compliance`, `crypto::deletion_proof`, Ed25519, optional, nie Voraussetzung für `Done`). Der Kern liefert kanonische Bytes und die offline lauffähige Funktion `verify_receipt` (Kette, Anker, Stufenreihenfolge, Zeitmonotonie; braucht keinen Tomb-Schlüssel, nur den `Mac`-Port).

**`Eraser::verify`** (strukturell, ohne Schlüssel, Pflichtmessungen): (a) keine lebende Datei/Segment/behaltenes Redo-Segment trägt Nutzdaten-/Directory-Einträge von H (Routing-Header und Gate-Frames ausgenommen), (b) Memtables ohne H, (c) bei `Cryptographic` `probe == Absent`, (d) Ledgerkette bis zum Anker gültig. Messarten (`Complete`, `Sample{n,N}`, `Declared`, `NotMeasured`, `NotImplemented`): SYS-6 §7.

## 11. Retention

Retention ist die einzige zeitgesteuerte Löschung (`INV-DECAY-1`).
- `due` ist der Zeitpunkt, zu dem der Löschauftrag **entsteht**. `engine::compliance` löst zu `due` (oder spätestens im nächsten Worker-Lauf, Takt ≤ 15 min) `Eraser::clusters_batch` je ≤ `[max_gate_clusters]` Cluster aus; das **Gate liegt am `due`**, die Frist `purge_deadline` läuft ab `due` (DEC-08).
- `due_bucket` (Monat/Quartal/Jahr) steuert nur die **physische Ko-Lokation**; kein Cluster wird später gegatet, weil sein Bucket noch offen ist. `pack_expired` ist die Sammelform nach Bucket-Ende.
- **Verlängerung** nur über `Restriction{BLOCK_ERASE, reason = RetentionExtension}` (FIX-22); `due` ist unveränderlich. Verkürzung ist ein normaler `Eraser`-Auftrag.
- Bei `ClockUncertain` stoppt Retention fail-closed (kein Löschen, Alarm); sie holt nach, sobald `skew_state = Ok`, in Reihenfolge `due` aufsteigend.
- Standardskew `[max_clock_skew]`.

## 12. Parameter

SYS-4 §2.5: `purge_deadline`, `purge_deadline_min`, `purge_window_fraction`, `deadline_reserve_fraction`, `key_destroy_fraction_max`, `purge_trigger_fraction`, `sealed_purge_escalate_*`, `purge_batch_min`, `purge_rewrite_concurrency`, `link_fence_ttl`, `max_gate_clusters`, `max_links`, `max_link_depth`, `logical_only_alarm_age`, `anchor_max_age`, `max_clock_skew`, `erase_reserve_bytes`, `purge_scratch_reserve_bytes`.

## 13. Entscheidungen und geschlossene Lücken

DEC-03, DEC-06, DEC-08, DEC-09, DEC-10, DEC-11, DEC-12, DEC-13, DEC-32, DEC-71 (Fallback-Ledger), DEC-75 (`link_fence`-Protokoll), DEC-76 (Lift-Nachricht = kanonische Bytes), DEC-78 (Fristüberschreitung ⇒ `Overdue`, kein Stopp); GAP-39, GAP-40, GAP-41, GAP-42 (SYS-7 §3).

## 14. Abnahme

`erase_order_contract` (Ledger/Catalog/Redo-Reihenfolge je Schritt, `INV-ERA-3`), `erase_crash_sweep` (Absturz an jedem E/A-Index über ER0…ER7, mit `ErasureOracle` zwischen den Phasen), `intent_covers_hull` (`INV-LED-3`), `link_fence_progress`, `poisoned_erase_intent` (`INV-ERA-13`), `fallback_ledger_merge`, `restriction_blocks_erase`, `lift_fourEyes_replay`, `deadline_arithmetic`, `deadline_breach_continues`, `purge_amortization` (H-28), `purge_no_punch_hole`, `receipt_status_matrix`, `verify_receipt_offline`, `retention_due_gate`, `retention_clock_uncertain`, `pack_expired_decompose`. Modelle: M2, M3, M4, M5, M6. Simulation: RACE-2, 3, 5, 6, 8, 9, 10, 11, 13, 14, 22, 25. Orakel: O-Files, O-Device, O-Gate, O-Ledger, O-Segments, O-Restore, O-Survivors. Messung: H-6…H-10, H-14, H-17, H-28. Gate G-E.

**Offene Punkte (extern):** rechtliche Prüfung der Fristen und der Aussage „kryptografisch gelöscht“ im Einsatzfall; `Strong`-Anker und HSM/KMS-Provider für den Produktivbetrieb.
