# CYC-5 — Wartungszyklus (Flush, Compaction, Blob-GC, Purge-Ausführung, Scrub, Governor, Catalog-Rollover)

Ersetzt R14 §4.8, §5.4, §8.6 (Ausführung). Abhängigkeiten: SYS-1 §4–§5 (Threads, Ränge), SYS-2 §5.4–§5.6 (Run, Blob, Catalog), SYS-4 §2.2–§2.4, §3 (Zahlen, Budget), CYC-1 (Backpressure, Reserve), CYC-3 §5 (Purge-Algorithmus, Barriere), CYC-4 (Schlüssel). Gate: **G-M** (Modelle M3, Simulation, Messung H-5, H-19, H-26…H-28); Purge-Teile zusätzlich G-E.

## 1. Zweck und Grenzen

Wartung erzeugt, verdichtet und entfernt **Dateien** und hält Reservierungen, Floors und Katalog klein und gültig. Sie ändert nie den logischen Inhalt einer Sicht (Ausnahme: Versionen unter `history_floor`, die für keine Sicht mehr erreichbar sind). Wartung läuft auf Worker-Threads (SYS-1 §4), nie unter Rang 1, und nimmt Rang 3 nur für Edits.

**Kernsätze.**
1. **Jede Dateimengenänderung ist ein Catalog-Edit** (`INV-CAT-1/2`); ein Edit entfernt nur Dateien unter eigenem Claim (`INV-CLAIM-1`).
2. **Nach der Publikation des Gates entsteht keine neue Datei, die Daten eines gegateten Clusters trägt** (§3 Nr. 6, §4 Nr. 7). Damit ist der Purge-Plan (CYC-3 §3.6, ER3) ein Fixpunkt.
3. **Löschung hat Vorrang vor allem anderen** (Lane `urgent`); Wartung verdrängt nie Gate, KeyDestroy oder Commit-Fähigkeit.
4. Wartungsfehler vergiften den Kern nicht (F-1/F-2); nur `sync_*`-Fehler vergiften das betroffene Handle (`INV-FSYNC-1`, `INV-DISK-1`).

## 2. Akteure, Lanes, Quoten, Claims

| Lane (Rang) | Aufträge | Auslöser |
|---|---|---|
| `urgent` (1) | Gate-Nachzug (G3), Speicherfreigabe, ER2, Plain-Purge, Redo-Barriere (Plain), Standby-Redo, Reservierungsauffüllung des Notpools, Notpool-Verzehr | Ereignis |
| `flush` (2) | Flush (§3), Sealed-Purge nach Eskalation (`[sealed_purge_escalate_flush]`) | Druck, Alter, Barriere |
| `compact` (3) | Compaction/Spill (§4), Rebucket, Sealed-Purge, Index-Rebuild-Aufträge der Engine (CYC-7 §5) | Level-/Staging-Schwellen |
| `blobgc` (4) | Blob-GC (§5) | `[blob_gc_ratio]` |
| `idem` (5) | `idem_trim`, `time_mark`, Graveyard-Unlinks, Branch-TTL-Ablauf (CYC-8 §4), Waisenschlüssel-Sweep (CYC-4 §5.2) | Takt `[idem_trim_interval]`, `[time_mark_interval]`, Ereignis |
| `idle` (6) | Scrub (§7), regelmäßige Selbstprüfungen | frei |

**Rangordnung (DEC-80).** Strikte Priorität `urgent > flush > compact > blobgc > idem > idle`; keine gewichtete Teilung. R14 nannte „Lane-Quoten“ ohne Zahlen; die Zahl ist der **E/A-Anteil**, nicht ein Worker-Anteil: `urgent` ist von `[maint_io_fraction]` ausgenommen; alle übrigen Lanes teilen sich `[maint_io_fraction]` der Gerätebandbreite (Token-Bucket je `FsCaps.device_id`, Reihenfolge der Entnahme folgt der Priorität); `idle`-Scrub hat zusätzlich `[scrub_io_fraction]` dieses Anteils. Aushungern ist durch Konstruktion begrenzt: Flush-Last ist durch die Memtable-Aufnahme beschränkt; erhöhen `l0_files` oder `staging_runs` ihre Schwellen, greift `Backpressure` an der Quelle (CYC-1 §8), nicht eine Umverteilung.

**Worker.** `[maint_workers]` Threads ziehen den höchsten lauffähigen Auftrag. Bei `maint_workers ≥ 2` ist ein Worker für `urgent` reserviert. Bei einem Worker (Entwicklungsrechner) unterbricht `urgent` kooperativ: der laufende Auftrag prüft `cancel` an jeder Blockgrenze, verwirft seine `.building`-Ausgabe und wird später neu begonnen.

**Claims (`INV-CLAIM-1`).** `claim(files, job)` ist atomar unter Rang 5. Ein Claim auf eine Datei, die ein Auftrag niedrigerer Priorität hält, setzt dessen `cancel`; der höhere Auftrag wartet auf den Drain (nur dieses Warten in der Wartung, nie unter Rang 1). Ausgabe-Edit abgelehnt, wenn `files_remove ⊄ Claims`. Ein Auftrag gibt Claims nur nach Edit oder Abbruch frei.

**Auftragsbudget.** Jeder Job reserviert RAM, Scratch und Puffer vor der Allokation (`try_reserve`, Klasse `Inflight`/`Extension`, SYS-4 §3); ohne Token startet er nicht. Unter Druck werden Aufträge in der Reihenfolge `idle`, `idem`, `blobgc`, `compact` zurückgestellt, nie `flush`/`urgent`.

## 3. Flush

Auslöser: Memtable ≥ `memtable_bytes_effective` (SYS-4 §3 Nr. 4), Gesamtdruck, `[redo_max_age]`, Barriere (Plain-Erase, CYC-3 §5.1), Checkpoint (CYC-6 §6), `Clean`-Close (CYC-6 §5), Governor (§8).

1. **Einfrieren.** Aktive Memtable wird immutable, eine neue aktive wird in dieselbe `Version` veröffentlicht (Rang 4, O(1)). Höchstens `[max_immutable_memtables]` je Space; darüber `Backpressure` (CYC-1 §8).
2. **Schlüssel pinnen.** Clusterschlüssel aller Cluster des Jobs werden für die Jobdauer gepinnt (Cache `gen`, CYC-4 §5.1). Providerausfall ⇒ Pause mit Backoff, keine Datenverluste (die Daten liegen im Redo).
3. **Abschnitte bauen.** Cluster aufsteigend nach `ClusterId`: Cluster mit Zustand `Retiring`/`Retired`/`Shredded`/`Purged` werden **übersprungen**. Je Cluster ein Abschnitt: `MiniSection`, wenn der Abschnitt höchstens `[mini_section_max_bytes]` groß ist, sonst `FullSection` (Blöcke zu `[block_target]`, `SubIndex`, `BranchTable`, `FilterBlock`, `RangeDelBlock`, `ChangeIndexBlock`). Körper ab `[value_separation_min]` gehen als Blob-Records in einen Blob (§5), im Eintrag steht `PutRef`.
4. **Platzierung (R-PL-5).** Ein Cluster mit `ClusterLocal`, in einem `Unpacked`-Space oder mit Flushbytes ≥ `[pure_threshold]` erhält einen **Pure-Run**. Alle übrigen (Packed, kleiner) liegen in **einem Staging-Run je Space** und werden pure, sobald ihre Gruppengröße `[pure_threshold]` erreicht (R-PL-3, §4).
5. **Schreiben.** `.building.<tmp_id>` (vorallokiert) → `sync_all` → `rename` → `sync_dir`. `ExtensionBuilder`s (CYC-7 §3) erhalten den Eintragsstrom.
6. **Gate-Validierung vor dem Edit.** Für jeden Cluster der Ausgabe wird `gen` mit dem Wert zu Jobbeginn verglichen. Abweichung (Gate/ER2 zwischenzeitlich) ⇒ die Ausgabe wird **verworfen** (`.building`/Datei entfernt) und der Job startet neu, ohne diesen Cluster (RACE-2). So existiert nach Publikation eines Gates nie eine neue Datei mit Daten des Clusters.
7. **Ein Edit** `{files_add…, replay_floor, anchor_mark}` (Rang 3, Gruppen-Commit). `replay_floor` rückt nur über Redo-Segmente, deren Frames in durablen Runs stehen **und** deren Gate-/Control-Frames im Catalog gespiegelt sind (`erase_pending_add`, `restriction_*`, `cluster_link`, `INV-LOG-5`); fehlt ein Spiegel, fügt der Flush ihn demselben Edit bei.
8. **Veröffentlichen.** Neue `Version` (Rang 4), Immutable-Memtable freigeben, Puffer `Zeroize`, Budget zurück. Danach `Anker`-Fortschreibung (koalesziert, `[anchor_max_age]`), nie vor dem Edit (`INV-ANCH-1`).

**Spill (Staging → Level).** Je Wartungsgruppe `G` mit gestagten Bytes `s_G` und der Größe `L_G` ihrer Level-1-Datei(en): Spill, sobald `s_G ≥ max([spill_min_bytes], L_G / [spill_ratio])` (das begrenzt die Schreibverstärkung auf `1 + spill_ratio`, H-26), oder das älteste gestagte Run `[spill_max_age]` erreicht, oder `staging_runs = [max_staging_runs]`. Spill ist ein Compaction-Auftrag (§4).

**`MiniSection`-Regel.** Ein Abschnitt, der beim Zusammenführen `[mini_section_max_bytes]` überschreitet, wird `FullSection`; der Rückweg gibt es nicht. Overhead-Ziel H-11.

## 4. Compaction

**Wartungsgruppe (R-PL-4).** Pure-Cluster: der Cluster; Packed: die `PackKey`-Gruppe. Compaction mischt nur innerhalb einer Gruppe (`INV-PL-3`). Level `0 … [max_levels] − 1`; L0 darf überlappen, ab L1 sind Dateien je Gruppe disjunkt im Schlüsselraum `(cluster, branch, key)`.

1. **Auslöser.** (a) L0 einer Gruppe ≥ `[l0_compaction_trigger]` Runs; (b) Level `i` überschreitet `run_target_bytes × [level_multiplier]^i`; (c) Spill (§3); (d) Rebucket/Demotion/Promotion (Nr. 6); (e) Purge als natürliche Compaction (CYC-3 §5.2 Auslöser a). Jobgröße ≤ `[compaction_job_max_bytes]`.
2. **Claims** auf Eingaben (Rang 5), Schlüssel pinnen, Budget reservieren.
3. **Merge.** k-Wege-Merge nach `cmp`. Ein Abschnitt, der in nur **einer** Eingabe vorkommt und keine GC braucht, wird **roh kopiert** (kein Entsiegeln, `block_uid`/`section_uid` bleiben, Frame-CRCs geprüft). Alle anderen werden entsiegelt, zusammengeführt und mit neuen UIDs und frischem Zufallsnonce versiegelt (die Rohkopie-Regel bleibt gültig, weil die UID mit dem Block wandert).
4. **Versions-GC je Cluster und Space** mit `floor = min(gc_floor, kleinster fork_stamp lebender Branches)` (`INV-BR-3`, CYC-8 §7): `KeepLatest`: je Schlüssel die neueste Version `≤ floor` und alle neueren; `KeepUntilFloor`: alle Versionen `≥ history_floor` plus die neueste darunter; `AppendOnly`: keine GC. Tombstones und `RangeDelete`-Fragmente entfallen erst auf der untersten Ebene der Gruppe, wenn keine ältere Quelle sie überdecken kann und ihr Stamp `< floor`. Versionen innerhalb `erased_ranges` mit `stamp ≤ erase_stamp` sind für jede Sicht unerreichbar und entfallen sofort physisch (auch bei `Logical`).
5. **Ausgabe.** Aufteilung bei `[run_target_bytes]` an Clustergrenzen; ein Pure-Cluster größer als das Ziel darf in mehreren Dateien liegen (je Datei ein zusammenhängender Abschnitt). `Retired`/`Shredded`/`Purged`-Cluster werden nicht kopiert. `ExtensionBuilder::merge` für Extension-Blöcke (CYC-7 §3). Verifikation: Footer lesen, `footer_tag` gegen die geschriebene Datei, Directory dekodieren.
6. **Promotion/Demotion/Rebucket.** Gruppe ≥ `[pure_threshold]` ⇒ Pure (R-PL-3); Pure unter `[pure_demote_ratio] × [pure_threshold]` ⇒ `rebucket` zurück in die Gruppe (`pack_assign` im **selben** Edit wie die Dateimengen, `INV-PL-4`, RACE-7). Governor kann `pure_threshold` zur Laufzeit anheben (§8).
7. **Edit** `{files_add(Ausgaben), files_remove(Eingaben), blob_live_delta…}` (Rang 3). Vorher Validierung wie §3 Nr. 6: hat sich der Zustand eines Eingabeclusters seit Jobbeginn geändert (`gen`), wird die Ausgabe verworfen und neu begonnen (RACE-3). `files_remove` legt im selben Edit den Graveyard-Eintrag an.
8. **Purge-Anrechnung.** Entfernt der Edit Eingaben, die Abschnitte von `Retired`-Clustern trugen, und tragen die Ausgaben keine, gilt das als `PurgeProgress` für deren Tickets (CYC-3 §5.2 Auslöser a); die Graveyard-Regeln (Unlink, `unlink_ack`) bleiben unverändert, `Done` verlangt weiterhin alle Atteste.
9. **Schreibverstärkung.** Metadaten ≤ H-5-Ziel, Körper ≤ H-5-Ziel; je Gruppe Staging→L1 ≤ `1 + spill_ratio` (H-26).

## 5. Blob-GC

**Lokalisierung (DEC-84, Format-Nachtrag in SYS-2 §5.4/§5.5).** `PutRef = (blob_id, record_uid, len)` enthält keinen Offset. Jeder Blob-Cluster-Abschnitt trägt daher einen versiegelten `RecordIndexBlock` `{(record_uid, rel_off u32, len u32)}` (AAD `kind = 4`, Section), der mit dem Directory-Eintrag in RAM-Cache gehalten wird (Klasse `BlockCache`). Jeder Run führt im `PackMeta` die Liste `blob_refs` der von ihm referenzierten `blob_id`.

1. **Zähler.** Der Catalog führt je Blob `live_bytes` (Edit `blob_live_delta`, mit `applied_seq` je Zähler, §8.2 `INV-CAT-3`). Compaction-Edits tragen die Deltas der weggefallenen `PutRef`.
2. **Auslöser.** Lochanteil `1 − live/total ≥ [blob_gc_ratio]` und tote Bytes `≥ [blob_gc_min_bytes]`.
3. **Lebendmenge.** Aus den Runs der `blob_refs`-Rückwärtssuche (Catalog-RAM-Index `blob → Runs`) werden die `PutRef` gelesen (Clusterschlüssel gepinnt). Referenzen können nur **abnehmen** (Compaction kopiert bestehende `PutRef`, erzeugt nie neue): eine veraltete Lebendmenge ist eine Obermenge und damit sicher.
4. **Ausführung.** Mit `punch_hole` darf die `blob_id` bleiben, tote Records werden ausgestanzt. Sonst (oder über `[blob_gc_ratio]`): neuer Blob mit den überlebenden Records (Rohkopie, `record_uid` bleibt), Edit `blob_replace{old, new, live_bytes}` (atomar). `blob_replace` registriert eine **Weiterleitung** `old → new` im Catalog-RAM; Lesen einer `PutRef` mit `old` folgt ihr. Die Weiterleitung entfällt, sobald kein lebender Run `old` in seinem `blob_refs` führt (abgeleitet, kein eigener Op). Die alte Datei geht in den Graveyard.
5. **`INV-BLOB-1`.** Ein Blob/eine Inkarnation wird nur entfernt, wenn kein lebender Run ihn in `blob_refs` führt (bei Purge: sobald der Cluster aus allen referenzierenden Runs entfernt ist; Blob-Purge folgt CYC-3 §5.2 über das Directory des Blobs, ohne Schlüssel).
6. **`punch_hole` nie Voraussetzung** (`INV-ERA-2`, CYC-3 §5.2).

## 6. Purge-Ausführung

Der **Algorithmus** steht in CYC-3 §5.2; hier die Ausführung.
1. **Plan (ER3).** Dateien = alle Runs/Blobs, deren `ClusterDirectory`/`ClusterFilter` einen Cluster aus `H` führt (Packgruppen-Index, Staging-Runs einbezogen, Filter positiv ⇒ Directory bestätigt), Blob-Inkarnationen über das Blob-Directory, Redo-Segmente mit Payload von `H`, registrierte Extension-Segmente. Der Plan ist ein **Fixpunkt** (Kernsatz 2): vor ER6 wird er erneut berechnet; findet er neue Dateien, ist das ein Logikfehler (F-8, Observer, Wiederholung).
2. **Claims** auf alle Plandateien, `cancel` laufender Jobs, Drain.
3. **Scratch-Slots.** Zwei vorallokierte `.building.<tmp_id>`-Slots, je ≥ größte Plandatei; beim Open geprüft (SYS-4 §5, `erase_reserve_bytes`). Ein Rewrite nutzt einen Slot; der zweite deckt den Crash-Slot. Nach `unlink_ack` der Quelle wird der verbrauchte Slot aus dem frei gewordenen Platz neu vorallokiert. Ist `free_space` zu knapp (DiskFull), gilt CYC-6 §8.
4. **Je Datei** (`[purge_rewrite_concurrency]` gleichzeitig): Rohkopie der Überlebensabschnitte nach Directory-Offsets in den Slot, neues Directory/PackMeta/Footer, `sync_all`, `rename`, `sync_dir`, dann **ein** Edit `PurgeBatchEdit{files_add(F'), files_remove(F)}`. Vollständig tote Dateien: nur `files_remove`.
5. **Überlebende mit beschädigten Frames (DEC-83).** Die Rohkopie prüft Frame-CRCs nur, um Lesefehler zu erkennen. Besteht ein **Überlebensabschnitt** die CRC nicht, bricht der Purge **nicht** ab: der Abschnitt wird bytegleich kopiert (Beweislage bleibt erhalten, Löschpflicht wird nicht blockiert), Observer `survivor_section_corrupt`, der Abschnitt bleibt für Scrub und Lesen als `Corrupt{Block}` sichtbar. Beschädigte Bytes **von `H`** werden nie gelesen.
6. **Reader-Drain und Unlink.** Nach Fall der letzten `Arc<Version>` (oder `ViewPolicy`) `unlink(F)`, `sync_dir`, `unlink_ack` (Edit); Graveyard bis dahin (`INV-GRAVE-1/2`). Hardlink-/Reflink-Kopien im Checkpoint (CYC-6 §6) verhindern das Freigeben; sie stehen als Disclosure im Receipt.
7. **Redo-Barriere (ER4)** als Wartungsjob (CYC-3 §5.1): (a) Syncer rotiert sofort (Steuernachricht in die Syncer-Warteschlange, kein Rang 1); (b) Flush aller Spaces (`flush`-Lane, für Plain `urgent`); (c) Edit `replay_floor` und `retain_floor` hinter den Schnitt (Consumer davor erhalten `Lagged`); (d) ältere Segmente: Unlink nach Pins/Views (kein `files_remove`, Redo ist nicht im `files`-Katalog; ein Segment lebt, solange `retain_floor` oder ein Pin es hält); (e) `PurgeProgress`. Eine Barriere bedient alle fälligen Tickets (H-14).
8. **Abschluss.** `SegmentOwner`-Atteste, Verify (CYC-3 §10), ER6-Edit.
9. **Auslösung und Dringlichkeit:** CYC-3 §5.2 (Bündelung) und SYS-4 §6 (Schwellen). Plain ⇒ `urgent`; Sealed ⇒ `compact`, Eskalation `flush`/`urgent`.

## 7. Scrub

1. Ein Vollzyklus je `[scrub_interval]`, Bandbreite `[scrub_io_fraction]`, Lane `idle`, Reihenfolge: ältester letzter Scrub zuerst, Dateien eines laufenden Tickets zuletzt.
2. **Geprüft** (ohne Klartext, nur MAC/CRC mit abgeleiteten Schlüsseln): Run/Blob: Dateikopf, Footer-Tag gegen Catalog (`footer_tag`), **vollständiger `content_mac`** (nur der Scrub liest ihn, SYS-2 §5.4), Directory-Seiten-MACs, Blob-Record-CRCs; Redo (behaltene Segmente): Gruppen und Kette bis `replay_floor` rückwärts nur im Rahmen des Prüfbereichs; Catalog und Ledger: Kettenprüfung (`catalog_key`/`ledger_key`); `KEYS`, `FORMAT`: MAC. Index-Partitionen prüft ihr Owner (`PartitionSource::verify`, CYC-7 §5).
3. **Befund (DEC-83).** Die kleinste betroffene Einheit wird vergiftet (Block → Abschnitt → Datei → Segment, F-4): Lesen aus ihr ⇒ `Corrupt{…}`, gesunde Einheiten laufen weiter; Observer `scrub_corrupt_total{file_role}`. Der Kern **repariert nie selbst** und fabriziert keine Daten. Heilung: Restore eines Checkpoints, Neuaufbau abgeleiteter Strukturen, Neuimport der Wahrheit; Löschaufträge bleiben ausführbar (§6 Nr. 5).
4. Ein Befund im Catalog, in `KEYS` oder im Ledger ist ein Integritätsfehler der Instanz (`ReadOnly{Corrupt}` ab dem nächsten Open, laufend: Handle `Poisoned`, CYC-6 §9).
5. Der Scrubstand ist flüchtig (nicht persistiert); nach Neustart beginnt der Zyklus neu.

## 8. Governor, Reservierungen, Catalog-Rollover

### 8.1 Governor

Ein Regelkreis in Lane `idem`/`idle` (keine eigene Priorität), liest Budget, `free_space`, `max_live_files`, Staging/L0-Zähler und stellt **Laufzeitgrößen** (Art R) und Reaktionen:
1. **Dateiflut:** ab 80 % `max_live_files` Anhebung von `pure_threshold` um je ×2 (nur RAM, nicht persistiert); bei 100 % entstehen keine neuen Pure-Dateien (SYS-4 §4 Nr. 4).
2. **DiskFull:** Eintritt/Austritt und `retain_floor := replay_floor` (CYC-6 §8).
3. **Speicherdruck:** Reihenfolge der Freigabe `BlockCache → Extension → Memtable` (SYS-4 §3), vorgezogener Flush.
4. **E/A-Anteil:** `maint_io_fraction`, `scrub_io_fraction` anpassbar (`Storage::tune`).
5. **Reservierungen:** `reserve_stamps`, `reserve_ids`, `reserve_space_ids` bei `< 50 %` per Catalog-Edit (Rang 3), nie zugleich mit Rang 1; der Notpool nur Lane `urgent` (CYC-1 §8 Nr. 3).
6. **Takt `[idem_trim_interval]`:** `idem_trim` hebt die Wasserstandsmarke (monoton, ein Edit, kein Fsync je Commit); **Takt `[time_mark_interval]`:** `time_mark` (CYC-8 §7).
7. **Standby-Redo-Segment** vor Erschöpfung (CYC-1 §3 Nr. 7).
8. **Graveyard-Sweep:** Unlinks nach Reader-Drain, `unlink_ack`.

### 8.2 Catalog-Rollover (DEC-82)

Der Catalog wächst durch Edits. **Schwelle:** angehängte Edit-Bytes seit dem letzten Snapshot `≥ max([catalog_rollover_min_bytes], Größe des letzten Snapshots)`.

**Invariante INV-CAT-3 (neu):** jede Catalog-Op ist eine **absolute, idempotente und monotone Zustandssetzung**; die einzige Ausnahme `blob_live_delta` trägt je Zähler ein `applied_seq` (Replay überspringt Deltas mit `seq ≤ applied_seq`). Damit ist ein **unscharfer** Snapshot zulässig.

Ablauf (ohne Sperre über E/A außer Rang 3 für die Edits):
1. `q_start := letzte seq`. Neue Datei `CATALOG-<n+1>` vorallokieren, Dateikopf mit `prev_chain_h = chain_h(q_start)`.
2. **Unscharfer Snapshot** schreiben: Zustandsmengen werden shardweise aus RAM gelesen (Rang 6 je Shard, kurz, nie über E/A), Blockweise (`[catalog_block_bytes]`) gestreamt und gesiegelt. Der Snapshot enthält zusätzlich `time_marks` (CYC-8 §7) und `chain_h` des Ledger-Ankers. Währenddessen laufen Edits weiter in die **alte** Datei; sie werden für die neue Datei in einer RAM-Warteschlange (Klasse `Inflight`) mitgeführt.
3. Nach dem Snapshot: unter Rang 3 alle Edits mit `seq > q_start` (aus der Warteschlange) in die neue Datei anhängen und mit **neuer** Kette versehen (Records werden neu MAC'd; Payload und `seq` bleiben), `sync_data`.
4. **Wechsel:** `CURRENT.tmp` → `sync_all` → `rename` → `sync_dir`. Ab jetzt gehen Edits in die neue Datei (Rang 3 hält den Wechsel kurz).
5. `anchor_mark` in der neuen Datei, Anker fortschreiben (`INV-ANCH-1`: nie vor dem durablen Edit).
6. Alte Datei entlinken (`sync_dir`); sie wird von keiner `Version` gelesen.

**Recovery:** CURRENT zeigt auf die gültige Datei; Snapshot laden, Edits mit `seq > q_start` in Reihenfolge anwenden (idempotent). Absturz vor 4: alte Datei gilt, neue `.building`/verwaist (OP9 entfernt). Absturz nach 4, vor 6: alte Datei verwaist, entfernt. Absturz in 3: neue Datei unvollständig, nicht in CURRENT, entfernt.

## 9. Fehler- und Absturzverhalten (Auszug, vollständig in der Testsuite)

| Phase | Unterbrechung | Soll |
|---|---|---|
| Flush: vor Edit | Absturz | `.building`/Run verwaist, Redo spielt ein (CYC-6 §3, OP6/OP9) |
| Flush/Compaction | `ENOSPC` beim Vorallokieren | F-2 (`DiskFull`-Logik), Job abgebrochen, kein Poison |
| Flush/Compaction | `sync_all`-Fehler der Ausgabe | Ausgabe verworfen; wiederholbar, **neue** Datei (Handle vergiftet, F-3) |
| Compaction: Edit | Claim verloren/`gen` geändert | Ausgabe verworfen, Neustart |
| Compaction | Providerausfall | Pause, Backoff (`KeyUnavailable` nur intern) |
| Purge | Absturz zwischen Rename und Edit | Verwaiste `F'` (nicht im Catalog) entfernt, Quelle gilt; Wiederholung |
| Purge | Edit durabel, Unlink offen | Verwaisten-Suche beim Open, `unlink_ack` idempotent |
| Rollover | siehe §8.2 | |
| Scrub | Lesefehler `EIO` | Handle `Poisoned` (F-3), Befund |

## 10. Parameter

SYS-4 §2.2–§2.4, §2.7: `block_target`, `run_target_bytes`, `blob_max_bytes`, `redo_*`, `mini_section_max_bytes`, `pure_threshold`, `pure_demote_ratio`, `spill_*`, `max_staging_runs`, `value_separation_min`, `catalog_*`, `maint_workers`, `maint_io_fraction`, `scrub_*`, `l0_compaction_trigger`, `level_multiplier`, `max_levels`, `compaction_job_max_bytes`, `blob_gc_*`, `memtable_bytes`, `max_immutable_memtables`, `max_live_files`, `idem_trim_interval`, `time_mark_interval`, `anchor_max_age`.

## 11. Entscheidungen und geschlossene Lücken

DEC-80 (strikte Lane-Priorität, Quote = E/A-Anteil, Lane `idem`), DEC-82 (Catalog-Rollover mit unscharfem Snapshot, `INV-CAT-3`), DEC-83 (Scrub repariert nie; Survivor-Korruption blockiert Purge nicht), DEC-84 (Blob-Lokalisierung: `RecordIndexBlock`, `blob_refs`, Weiterleitung); GAP-46 (Lane-Rangfolge und „Quoten“ ohne Zahl; Lane `idem` fehlte), GAP-47 (Rollover-Protokoll fehlte), GAP-48 (Scrub-Befundbehandlung, Blob-Lokalisierung, Survivor-Korruption nicht definiert) (SYS-7 §3).

## 12. Abnahme

`flush_atomic_crash` (Absturz an jedem E/A-Index), `flush_skips_retired`, `flush_gate_validation` (RACE-2), `spill_trigger_amplification` (H-26), `mini_section_boundary`, `compaction_eq_model` (`proptest`, Referenzmodell mit Versionen/Branches/History), `compaction_raw_copy_uid`, `compaction_gc_floor`, `compaction_fork_floor` (RACE-16), `compaction_erase_claims` (RACE-3), `compaction_discards_on_gate_change`, `rebucket_atomic` (RACE-7), `blob_gc_forward_resolution`, `blob_ref_decrease_only`, `blob_no_punch_hole_dependency`, `purge_fixpoint_plan`, `purge_survivor_corrupt_copy`, `purge_scratch_slots`, `redo_barrier_serves_all_tickets` (H-14), `scrub_detects_flip` (Bit-Flip je Dateiart), `scrub_never_repairs`, `rollover_fuzzy_snapshot_crash` (Absturz an jedem E/A-Index, INV-CAT-3), `catalog_op_idempotence` (jede Op doppelt angewandt ≡ einfach), `reserve_refill_rank_disjoint`, `lane_priority_no_inversion`, `governor_pure_threshold`. Modelle: M3, M5. Simulation: RACE-2, 3, 7, 13, 14, 16. Messung: H-5, H-19, H-26, H-27, H-28. Gate G-M.

**Offene Punkte:** keine.
