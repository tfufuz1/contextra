# CYC-1 — Schreibzyklus (`apply`, Sequencer, Syncer, Guards, Idempotenz, Backpressure)

Ersetzt R14 §5. Abhängigkeiten: SYS-1 §4–§6 (Threads, Ränge, Fehler), SYS-2 §5.3 (Redo), SYS-3 §4 (API), SYS-4 (Zahlen). Gate: **G-W**.

## 1. Zweck und Grenzen

Der Zyklus beschreibt, wie ein `Change` (atomare Op-Menge) oder ein Kontrollframe (Gate, Link, Restrict, Lift, Branch-Op) zu einem durablen, geordneten, veröffentlichten Redo-Frame wird. Er endet mit `Receipt`. Nicht Teil: Flush und Compaction (CYC-5), Löschprotokoll (CYC-3), Recovery (CYC-6).

Ziele: Z-1 (Atomarität), Z-2 (Durability vor Sichtbarkeit), Z-8 (Race-Freiheit), Z-11 (Idempotenz), Z-17 (keine E/A unter Rang 1), Z-12 (Löschung bei vollem Datenträger).

## 2. Akteure und Zustandsautomat

| Akteur | Aufgabe | Sperren |
|---|---|---|
| Aufrufer | Stufe-A-Guards, Hülle, Krypto, Vorkodierung, Credit | keine über E/A |
| Sequencer | Flat-Combining-Leader, wer `try_lock(Rang 1)` gewinnt; Zustandsprüfung, Stufe B, Stamps | Rang 1 (Rang 2 nur zum Leeren) |
| Syncer | ein Thread: Gruppenbildung, Kette, `write`, `sync_data`, Publikation | keine während E/A |
| Wartung (`urgent`) | füllt Reservierungen, Notpool, Standby-Redo-Segment | Rang 3 |

**Zustand eines Requests:** `Prepared → Enqueued → Sequenced(stamp) → Written → Durable → Published → Acknowledged`. Abbruchzweige: `Rejected` (vor Stamp, nie sichtbar), `Indeterminate` (nach Stamp, Sync-Fehler), `Poisoned`. Ein Request, der `Sequenced` ist, ist entweder `Published` oder gehört zu einer vergifteten Gruppe (`INV-PIPE-2`).

**Zustand eines Clusters im Schreibpfad:** `Live → Retiring(S) → Retired(S)` (SYS-7 Anhang A). `Retiring` existiert nur im Sequencer-RAM und schließt die Schreibadmission, für Leser ist der Cluster bis zur Publikation `Live`.

**Kanonische Sichtbarkeitsachsen:** `durable_stamp` (höchster Stamp, dessen Gruppe `sync_data` überstanden hat) ≤ `visible_stamp` (höchster veröffentlichter Stamp). `visible_stamp > durable_stamp` ist nur durch `Relaxed`-Changes möglich (§3 Nr. 5, Nr. 8).

## 3. Ablauf

Voraussetzung für jeden Schritt ist die Sperrdisziplin SYS-1 §5. Schrittnummern sind stabil und werden von SYS-1 zitiert (Schritt 4 = Credit).

**Frameformen** (SYS-2 §5.2/§5.3; Klarstellung GAP-20): `Change`: `n_ops` = Zahl der Ops. `Gate`: `n_ops = 0`, die Clusterliste steht im Body (`n u16`, höchstens `[max_gate_clusters]`). `Control Restrict/Lift/Link`: `n_ops = 0`, Body im Klartext. `Control Branch*`: `n_ops = 1` (Routing im `ops_head`, Nutzlast versiegelt, CYC-8 §4). `Reserve`: `n_ops = 0`, `stamp = 0`.

| Nr. | Schritt | Thread / Rang | Vor- und Nachbedingung |
|---|---|---|---|
| 1 | **Vorprüfung ohne Sperre:** Limits (`INV-LIM-1`), Space/Cluster bekannt, `INV-SEAL-1`, Clusterzustand (`Live`), Instanzzustand (`DiskFull`, `ReadOnly`, `Poisoned`), `ReservedSpace`, Idempotenz-Lookup in `SYS_IDEM` | Aufrufer | Treffer mit gleichem Fingerabdruck ⇒ `Ok(Receipt{AlreadyApplied})`; Fehler hier sind `Rejected`, nichts wurde veröffentlicht |
| 2 | **Vorbereiten:** (a) View `V1` pinnen, Guards Stufe A gegen `V1`, Punktfingerabdrücke `fp` (§7); (b) bei Gate/Link/Restrict/Lift/Space-Drop: Hülle `H`, Restriction-Treffer, `policy_epoch P1` notieren; (c) Ops kodieren, `op_uid` ziehen (`Entropy`), Nutzlast versiegeln (Cipher, Clusterschlüssel aus dem Cache oder Provider), Frame vorkodieren (`stamp = 0`, Kette/CRC leer); (d) implizite `Absent`-Guard und `Put` auf den `SYS_IDEM`-Schlüssel anfügen (§6), falls `idem` gesetzt | Aufrufer | Provider-, Cipher-, KDF-, Graph-, Plattenarbeit nur hier (`INV-PIPE-1`); `ClusterKeyUnavailable` ⇒ `KeyUnavailable` (`Rejected`) |
| 3 | **Stamp-Reserve prüfen:** `reserve_stamps` erschöpft ⇒ `Backpressure(Retry{after})` ohne Rang 3. Gate/Control/Link/Branch-Ops ziehen aus dem Notpool (`reserve_stamps_urgent`, `reserve_ids_urgent`) | Aufrufer | Notpool wird nur von Lane `urgent` gefüllt (§8) |
| 4 | **Pipeline-Credit holen** (`commit_pipeline_depth`; `urgent` hat eigenen Credit). Warten ist **nur hier** erlaubt, höchstens `[commit_wait_max]`, dann `Backpressure(Retry{after})` | Aufrufer | Credit wird in Schritt 9 (oder bei `Rejected`/`Poisoned`) freigegeben |
| 5 | **Einreihen:** Request in `intake` anhängen (Rang 2, kurz), Rang 2 freigeben; `try_lock(Rang 1)`: Erfolg ⇒ Leader, sonst auf das `Completion`-Slot des Requests warten. Leader prüft **nach** Freigabe von Rang 1 die Intake erneut (kein verlorener Wakeup, RACE-19) | Aufrufer / Leader | `Completion` ist ein `Arc`-Slot mit Atomic + `park/unpark`, kein Mutex (kein Rang) |
| 6 | **Sequencer** (Rang 1, Rang 2 nur zum Leeren), je Request in Intake-Reihenfolge: (a) Zustand: `Retiring/Retired/Shredded/Purged` ⇒ `ClusterRetired`; `BLOCK_WRITE` ⇒ `Restricted`; Branch nicht `Live` ⇒ `BranchClosed`; (b) Guards Stufe B gegen das Konfliktfenster (§7); (c) bei Gate/Link/Restrict/Lift: `policy_epoch == P1`, sonst `PolicyRetry`; nach `[policy_retry_max]` berechnet der Sequencer die Hülle einmalig selbst unter `link_fence` (CYC-3 §3, `INV-LED-3`); (d) Stamps vergeben (je Frame einer, aufsteigend, aus der Reserve); Branch-Ops ausführen; bei Gate `Live → Retiring(S)`; (e) Konfliktfenster um `(stamp, fp, ck_hash)` fortschreiben; (f) die Gruppe an den Syncer übergeben (Warteschlange, nie blockierend), Rang 1 frei | Leader | Rang 1 ist reines RAM, O(Ops); **verboten:** E/A, Provider/Cipher/KDF, Warten, Graph-Traversierung (einzige Ausnahme 6c) |
| 7 | **Syncer:** Frames finalisieren (Stamp einsetzen, CRC32C, Kettenmac), `GroupHead` bilden, auf `redo_align` auffüllen, **ein** `write_at` + **ein** `sync_data` | Syncer | **Durabilitätspunkt.** `Relaxed`-Gruppen (alle Frames `RELAXED`) werden ohne `sync_data` geschrieben |
| 8 | **Publikation** strikt in Stamp-Reihenfolge, nur nach Schritt 7 derselben Gruppe: Memtable einfügen, Fenstereinträge als durabel markieren, `Retiring → Retired` + `gen++`, `Version` veröffentlichen (Rang 4), `visible_stamp` (Release) und, sofern `sync_data` lief, `durable_stamp` anheben, `Completion`-Slots wecken | Syncer | Leser laden `visible_stamp` (Acquire), dann `Arc<Version>` |
| 9 | **Quittung** `Receipt{instance_id, stamp, durable, status}`, Credit frei | Aufrufer | `durable = true` ⇔ `sync_data` der Gruppe lief |

**Eigenschaften.**
1. Durchsatz = Gruppengröße / Sync-Latenz. Gruppen wachsen mit der Last bis `[max_group_changes]`/`[max_group_bytes]`. Tiefe > 2 nur, wenn `FsCaps.overlapped_sync_beneficial` belegt ist (Fs-Konformanz, SYS-6 §4).
2. Sync-Fehler ⇒ Poison des Redo-Handles; die fehlgeschlagene und alle sequenzierten späteren Gruppen sind `Indeterminate{first,last}` bzw. `Poisoned`; nichts davon wurde veröffentlicht (`INV-PIPE-2`).
3. Leser sehen `Retiring` als `Live`, bis das Gate durabel und publiziert ist; die Schreibadmission ist sofort geschlossen.
4. Die Memtable-Einfügung durch den Syncer ist Standard. Parallele Einfügung durch Aufrufer ist zulässig, wenn der Syncer nach `[publish_timeout]` selbst einfügt (Tail-Latenz-Schutz).
5. **`Relaxed`** (nie für Löschung, nie mit `IdemToken`, FIX-27): Frame trägt `RELAXED`, wird nach dem `write` ohne Sync veröffentlicht. Ein späterer `sync_data` derselben Redo-Datei macht ihn durabel (die Datei ist reihenfolgetreu); `durable_stamp` springt dann über alle vorherigen Relaxed-Frames. Absturz verliert Relaxed-Frames bis zum letzten Sync. Consumer und `view_durable` sehen nur Stamps `≤ durable_stamp` (`INV-CONS-1`).
6. Eine Gruppe enthält Frames **aller** Arten in Stamp-Reihenfolge; ein Gate-Frame und Change-Frames desselben Clusters in einer Gruppe sind zulässig (der Sequencer hat die Reihenfolge entschieden).
7. **Rotation:** Passt die nächste Gruppe nicht mehr in das Segment, schreibt der Syncer ein `SegmentSeal` als letzte Gruppe, `sync_data`, wechselt in das Standby-Segment (`SegmentHead{first_stamp, prev_chain_h, seg_seq+1}`). Das Standby legt Lane `urgent` vor Erschöpfung an (vorallokiert, `Perm::Private`). Fehlt es, legt der Syncer es selbst an (E/A ist auf dem Syncer erlaubt); ein `ENOSPC` beim Vorallokieren ist F-2 (`DiskFull`), kein Poison.
8. `visible_stamp` und `durable_stamp` sind Atomics, monoton, nur vom Syncer geschrieben.

## 4. Fehler- und Absturzverhalten je Schritt

| Schritt | Fehler / Absturz | Wirkung |
|---|---|---|
| 1 | Limit, Zustand, Ports | `Rejected`, kein Stamp, kein Frame |
| 2 | `KeyUnavailable`, Cipher-/Entropy-Fehler (`EntropyError` ⇒ fail-closed) | `Rejected`; kein Teilzustand |
| 3, 4 | Reserve leer, Credit-Timeout | `Backpressure(Retry{after})`, gleicher Request wiederholbar |
| 5 | Aufruferthread stirbt als Leader | ein anderer Aufrufer übernimmt über die Intake-Prüfung; Requests in `intake` ohne lebenden Aufrufer werden vom nächsten Leader sequenziert (der Request besitzt sein `Arc<Completion>`) |
| 6 | Guard Stufe B / Policy | `GuardRetry`/`PolicyRetry`; Request bleibt `Rejected`; Aufrufer wiederholt (§7) |
| 6 → 7 | Absturz nach Stamp, vor `write` | Stamp verloren (Lücke zulässig, `INV-STAMP-3`: nach Neustart `next_stamp = max(stamp_limit, letzter gültiger Redo-Stamp, max Run-Stamp) + 1`); Change nicht sichtbar |
| 7 | `write` zerreißt | Torn Tail (SYS-2 §5.3.1); Gruppe gilt als nicht geschrieben; alle Changes der Gruppe nicht sichtbar |
| 7 | `sync_data` schlägt fehl | Handle `Poisoned`, Gruppe + spätere `Indeterminate`, `Poisoned`-Zustand der Instanz; `ENOSPC` wird nie hier erwartet (vorallokiert) |
| 7 → 8 | Absturz nach `sync_data` | Change vorhanden (Replay); Token verhindert Doppelung |
| 8 | Publikation unterbrochen | Neustart stellt aus Redo her; `Retiring` ohne durables Gate existiert nach Neustart nicht |
| 9 | Aufrufer weg | Change ist trotzdem wirksam; Wiederholung nur mit Token sicher (§5) |

## 5. `Indeterminate` und Wiederholung

`Indeterminate{first,last}` bedeutet: der Request war sequenziert; Durabilität unbekannt. `first..=last` sind die Stamps der betroffenen Gruppen. Nach dem Poison ist ein Neustart nötig; die Auflösung geschieht danach.

1. **Mit `IdemToken`:** nach Neustart `apply` mit demselben Token und demselben Request wiederholen. Ist der Change durabel, liefert Schritt 1 `AlreadyApplied` mit dem ursprünglichen Stamp; sonst wird er neu angewandt. Das ist die einzige sichere Wiederholung.
2. **Ohne Token:** nicht wiederholen. Der Aufrufer prüft nach dem Neustart per Lesen (`view()`, Guards `Version`/`Absent` des eigenen Schlüssels), ob der Change vorhanden ist. Ein Change mit mehreren Ops ist atomar, **eine** Op genügt als Beleg.
3. Die Fassade bietet `Storage::resolve(first, last, &Change) -> Resolution{Applied(stamp) | NotApplied | Unknown}` als Hilfe: es vergleicht die `op_uid`-freien Fingerabdrücke der Ops mit den Records im Stampbereich. `Unknown` (z. B. Schlüssel inzwischen überschrieben) verlangt Aufruferlogik.
4. Mutationen, die `Indeterminate` melden können, sind nur `apply` und die Control-Frames. Löschaufträge melden nie `Indeterminate`: sie enden entweder mit `Ok(Ticket)` oder mit `Poisoned{intent}` (CYC-3 §9).

## 6. Idempotenz (`IdemToken`, `SYS_IDEM`)

`IdemToken` = 16 B, vom Aufrufer, zufällig (Aufrufer-Pflicht). Gültigkeit `[idem_ttl]`.

1. **Schlüssel:** `HMAC(idem_key, "ctxs idem key v1" ‖ scope ‖ token)` (32 B), `scope ∈ {0x01 apply, 0x02 merge, 0x03 branch}` (verschiedene Operationen mit demselben Token kollidieren nie), `idem_key = KDF(RootKey, "ctxs idem-mac v1")`.
2. **Wert:** SYS-2 §5.8. Der Kern schreibt in v1 nur `status = Applied`; `InFlight`/`Failed` sind reserviert und werden vom Decoder angenommen, vom Kern nicht erzeugt (GAP-35; die Engine nutzt für mehrteilige Ingests eigene Tokens je Teil-Change, CYC-7 §2).
3. **Request-Fingerabdruck:** `HMAC(idem_key, "ctxs idem request v1" ‖ canonical(change))`, `canonical` = Ops in Einreichungsreihenfolge, Guards in Einreichungsreihenfolge, längenpräfixiert, ohne `since`-Stamps der Guards, ohne `op_uid`, ohne `durability`.
4. **Ablauf:** Schritt 1 sucht den Schlüssel. Treffer, gleicher Fingerabdruck ⇒ `AlreadyApplied(result_stamp)`; Treffer, anderer Fingerabdruck ⇒ `IdemConflict`; kein Treffer ⇒ Schritt 2d hängt `Absent`-Guard und `Put` an **denselben Frame**. Zwei gleichzeitige Requests mit demselben Token: der zweite trifft Stufe B (`fp` des Idem-Schlüssels im Fenster) und erhält `GuardRetry{blocking_stamp}`; er wartet auf `visible_stamp ≥ blocking_stamp` (höchstens `[publish_timeout]`), liest erneut und findet `AlreadyApplied`. Ist die Gruppe des ersten Requests vergiftet, erhält der zweite `Poisoned`.
5. **Ergebnis:** `result_kind` + `result` (≤ 256 B) tragen den Stamp und kleine Ergebnisse (z. B. neue `BranchId`). Größere Ergebnisse sind nicht zulässig; die Engine speichert sie unter eigenem Schlüssel.
6. **Ablauf der Gültigkeit:** `expires_bucket` = Stunden seit `created_utc` (FORMAT). Eine Compaction des Systemspaces entfernt Records mit `expires_bucket < idem_trim`-Marke. `idem_trim` ist eine durable Wasserstandsmarke im Catalog und wird von Lane `idem` der Wartung höchstens einmal je Stunde angehoben, ohne Fsync je Commit. **Nach Ablauf ist ein Wiederholen mit demselben Token ein neuer Request** (dokumentierte Grenze, Aufrufer müssen innerhalb `[idem_ttl]` wiederholen).
7. **Kapazität:** `[idem_max_active]` Tokens, `[idem_max_bytes]` Bytes; Überschreitung ⇒ `IdemCapacity` vor Mutation (§8 von SYS-4); aktive Tokens werden nie verdrängt. Zählung: RAM-Zähler, bei Open aus dem Systemspace rekonstruiert (lazy, Scan über den Systemspace, höchstens `[idem_max_bytes]`).
8. **Fail-closed:** ist der System-DEK/`idem_key` nicht verfügbar, schlagen tokenierte Operationen mit `KeyUnavailable` fehl; untokenierte laufen weiter.
9. Gleicher Token nach Cluster-Retire ⇒ `ClusterRetired`/`Erased`, nie neues Apply. Ein tokenierter Change MUSS `Durable` sein.
10. `EraseRequestId` ist **kein** `IdemToken` (SYS-3 §6, CYC-3 §3): andere Gültigkeit (Ledger), anderer Raum.
11. Die Payload (Fingerabdruck, Ergebnis) wird vor Rang 1 versiegelt (`Seal::ByMode`, System-DEK); der äußere MVCC-Stamp des Records ist der Ergebnis-Stamp (`result_stamp` wird beim Entsiegeln gegen den Record-Stamp geprüft, Abweichung ⇒ `Corrupt`).

## 7. Guards und Konfliktfenster

**Stufe A (Aufrufer, ohne Sperre).** Pin `V1`. Jede Guard wird **exakt** gegen `V1` ausgewertet (Plattenzugriff, ChangeIndex-Scan erlaubt):
- `Absent`: kein sichtbares Put (ein Tombstone oder ein überdeckendes `RangeDelete` zählt als abwesend).
- `Version(s)`: der neueste sichtbare Eintrag hat Stamp `s`.
- `Unchanged{since}`: keine Mutation des Schlüssels (Put, Delete, überdeckendes RangeDelete, `EraseRange`) mit Stamp `> since` und `≤ V1`.
- Range-Guard: keine Mutation im Bereich seit `since`.
Verstoß ⇒ `Err(Conflict{contended:false})`. Beweise für Stufe B: Punkt `(V1, fp)`, `fp = SipHash-1-3(prozessschlüssel, space ‖ cluster ‖ branch ‖ key)` (64 Bit); Range `(V1, ck_hash)`, `ck_hash = SipHash-1-3(prozessschlüssel, space ‖ cluster ‖ branch)`.

**Stufe B (Sequencer, nur RAM, O(Guards)).** Das **Konfliktfenster** ist ein stamp-sortierter Ring der letzten `guard_window_entries` Mutationen `(stamp, fp, ck_hash)`, auch noch nicht durabler Gruppen, nur im RAM.
- Punkt-Guard: ein Eintrag mit gleichem `fp` und `stamp > V1` ⇒ `GuardRetry{blocking_stamp = stamp}`.
- Range-Guard: ein Eintrag mit gleichem `ck_hash` und `stamp > V1` ⇒ `GuardRetry`. Das ist konservativ (kein Schlüsselvergleich unter Rang 1): Falsch-Positive führen zu Wiederholung, nie zu falsch-Negativen.
- **Fensterfuß** = Stamp des zuletzt verdrängten Eintrags (0, solange nichts verdrängt wurde). `V1 < Fensterfuß` ⇒ `GuardRetry`.
- Mutationen jeder Op werden eingetragen (Put, Delete, jedes `RangeDelete` mit `ck_hash` und `fp = 0`-Sondereintrag für Range, `EraseRange`, Branch-Ops mit `ck_hash`).
- Der Sequencer urteilt nie über Anwendungskonflikte; es gibt keinen Langsampfad (Anhang D).

**Wiederholung.** Der Aufrufer wartet bei `GuardRetry{blocking_stamp}` auf `visible_stamp ≥ blocking_stamp` (höchstens `[publish_timeout]`), pinnt eine neue View und wiederholt Stufe A höchstens `[guard_retry_max]` Mal, danach `Conflict{contended:true}`. `INV-GUARD-1`: Ein Change wird nur angenommen, wenn jede Guard zur Stamp-Vergabe gilt.

**Guard-Katalog (Konvention für obere Schichten):** Event anhängen ⇒ keine Guard; Zustand/Fakt ändern ⇒ `Version`; Tool-Zustand ⇒ `IdemToken` + `Version`; Branch-Merge ⇒ `Unchanged{since: merge_base}` je Schlüssel.

## 8. Backpressure, Credits, Reservierungen

1. **Credits:** `commit_pipeline_depth` allgemeine Credits plus ein `urgent`-Credit (Gate, Control, Link, Branch-Ops, Löschintent-Frames). Ein Credit entspricht einer Gruppe in Flug, nicht einem Request.
2. **Stufen** (alle außerhalb jeder Sperre): (a) *weich* ab `[backpressure_soft]` des Memtable-Gesamtbudgets: Verzögerung linear bis 1 ms je Commit am Credit-Eingang; (b) *hart* ab `[backpressure_hard]`, bei `l0_files > 2 × l0_compaction_trigger` je Gruppe oder `staging_runs = max_staging_runs`: `Backpressure(Retry{after})`; (c) `DiskFull` (CYC-6 §8) für Nutzer-Commits.
3. **Reservierungen** (SYS-4 §2.4): `reserve_stamps`, `reserve_ids`, `reserve_space_ids` werden von der Wartung bei `< 50 %` per Catalog-Edit (Rang 3) aufgefüllt; Rang 1 und Rang 3 werden **nie** gleichzeitig gehalten. Ist die allgemeine Reserve erschöpft, liefern normale Changes `Backpressure(Retry)`; Gate-/Control-/Link-Frames ziehen aus dem Notpool, den nur Lane `urgent` füllt. Erschöpft der Notpool, ist das ein Betriebsalarm (`erase_reserve_free_bytes`/`stamp_reserve_urgent_low`), und die Reserve wird vor jedem weiteren `urgent`-Frame synchron aufgefüllt (Aufrufer blockiert in Schritt 3, nicht unter Rang 1).
4. **Consumer-Retention:** `retain_floor` und `Lagged` regeln nicht den Commit (CYC-7 §4); nur unter `DiskFull` setzt der Governor `retain_floor := replay_floor`.

## 9. Parameter

Siehe SYS-4 §2.4: `commit_pipeline_depth`, `max_group_changes`, `max_group_bytes`, `guard_window_entries`, `guard_retry_max`, `policy_retry_max`, `publish_timeout`, `commit_wait_max`, `backpressure_soft`, `backpressure_hard`, `reserve_*`, `redo_align`, `redo_segment_bytes`, `idem_ttl`, `idem_max_active`, `idem_max_bytes`.

## 10. Entscheidungen und geschlossene Lücken

DEC-02, DEC-14, DEC-15, DEC-66 (Gate/Control/Link haben `n_ops = 0`, Branch-Ops `n_ops = 1`), DEC-67 (`GuardRetry` trägt `blocking_stamp`, Aufrufer wartet auf Sichtbarkeit statt zu spinnen), DEC-68 (`Completion`-Slot ohne Mutex), DEC-69 (Kern schreibt nur `Applied` in `SYS_IDEM`); GAP-20, GAP-31, GAP-32, GAP-33, GAP-34, GAP-35, GAP-36 (SYS-7 §3).

## 11. Abnahme

Tests: `apply_atomic_crash` (Absturz an jedem E/A-Index, `FaultFs`), `group_commit_order`, `sync_failure_poison` (`INV-PIPE-2`), `relaxed_visibility`, `guard_stage_a_b`, `guard_window_floor`, `guard_retry_waits_for_visible`, `idem_duplicate_concurrent`, `idem_conflict`, `idem_expiry_boundary`, `indeterminate_resolve`, `credit_wait_only`, `no_io_under_rank1` (Syscall-Zähler, H-18/H-24), `leader_handover` (`loom`, RACE-19), `reserve_exhaustion`, `redo_rotation_crash`. Modelle: M1. Simulation: RACE-18, RACE-19, RACE-20. Messung: H-1, H-18, H-24. Gate G-W.

**Offene Punkte:** keine.
