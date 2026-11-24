# INDEX — Implementierungsspezifikationen `contextra-storage` (R9.1)

| | |
|---|---|
| Normativ | `crates/contextra-storage/SPEC.md`; Besitzer-SP `01-ports.md` bis `25-r8-audit-remediation.md` in diesem Crate |
| Lesekopien | `docs/implement/01-ports.md` bis `07-run.md` sind Spiegel ohne unabhängige Normwirkung; bei Abweichung gilt die SPEC und danach der Besitzer-SP hier |
| Status 01–25 | Implementierungsverträge und Audit-Traceability liegen als `NN-*.md` neben dieser Datei |
| Status Code | R8 dokumentiert den historischen Ausgangsstand als nicht kompiliert/gemessen. Aktueller Teilcheck: Storage + Crypto-Referenz kompilieren mit `--ignore-rust-version`; die vollständigen Gates sind offen. |
| Phasenfortschritt | [PHASE-STATUS.md](PHASE-STATUS.md) — isolierter Fortschritt und offene Gates |
| Prinzip | **Disjunktion:** jeder Typ, jede Datei, jede Invariante hat genau einen Besitzer-SP; Fremdzugriff nur über dessen Schnittstelle (§1 dieser Datei) |

## 1 Besitzmatrix (Modul → SP)

| SP | Datei | Modul / Crate | Phase |
|---|---|---|---|
| PORTS | 01 | `contextra-storage-ports` | P0 |
| CODEC | 02 | `codec/` | P0–P1 |
| ENV | 03 | `env/`, `sync/` | P0–P5 |
| KEYS | 04 | `keys/`, `crypto-ref`, `keys-file`, `keys-cohort` | P0–P6 |
| REDO | 05 | `store/redo/` | P1–P2 |
| CATALOG | 06 | `catalog/` | P1–P2 |
| RUN | 07 | `store/run/` | P2 |
| BLOB | 08 | `store/blob/` | P4 |
| MEMVER | 09 | `engine/{memtable,version,registry,graveyard}` | P1–P3 |
| COMMIT | 10 | `engine/{committer,guard_window,volatile,idem}` | P1–P3 |
| READ | 11 | `engine/{views,cursor,page,cache}` | P1–P4 |
| CLUSTER | 12 | `erase/{cluster_table,links,restrictions}`, `engine/packgroups` | P1–P6 |
| ERASE | 13 | `erase/{gate,protocol,tickets,receipt}` | P1–P6 |
| RECOVERY | 14 | `recovery/` | P1–P2 |
| VERIFY | 15 | `testing/`, `tools/reference/`, `formal/` | P0–P8 |
| MAINT | 16 | `maint/{scheduler,flush,compaction,blob_gc,scrub,reserve}` | P2–P5 |
| EXT | 17 | `engine/{segments,extension,feed}` | P5–P6 |
| BRIDGE | 18 | `contextra-storage-bridge` (async, Altstand-Adapter, Signatur) | P6–P8 |
| PURGE | 19 | `maint/purge` | P2–P6 |
| LEDGER | 20 | `erase/ledger` | P1–P6 |
| BRANCH | 21 | `engine/branches`, `SYS_BRANCH` | P7 |
| CKPT | 22 | `checkpoint/` | P6 |
| FACADE | 23 | `lib.rs`, `config`, `spaces`, Metrik-/Observer-Katalog | P1–P8 |
| CUTOVER | 24 | `tools/migrate`, Umstellungsplan | P8+ |
| AUDIT | 25 | SPEC.md §12.8 sowie Anhang B.2/B.3: Audit-Remediation und Revisions-Traceability | vor G-0/G-F, danach je Besitzer-SP |

Regeln: Ein Typ, der ≥ 2 SPs teilen, steht in PORTS. Ein Dateiformat gehört dem SP, dessen Datei es ist; Primitive (varint, Siegel, Tag) immer CODEC. Kein SP ruft einen höheren SP (Layering-Test `layering`, Richtung wie SPEC §3.1).

## 2 Abhängigkeits- und Phasengraph (Arbeitspakete)

```
P0  01 → 02.1 → 03.1 → 03.2 ; 03.4 ; 04.1 → 04.2 ; 15.1/15.2 ; 15.9(CI)            ── G-0
P1  02.2 → 05.1 → 05.2 → 05.3 ; 06.1 ; 09.1 ; 10.1–10.3 ; 11.1 ; 12.1 ; 13.1 ; 20.1 ; 14.1 ; 23.1 ── Crash-Sweep (WS)
P2  06.2–06.4 → 07.1–07.4 → 16.1 (Flush) → 19.1 (Purge) → 14.2 ; 13.2 (ER4/ER6)                    ── G-F (Freeze)
P3  10.4–10.7 ; 06.5                                                                                ── G-W
P4  08.* ; 11.2–11.6 ; 07.5                                                                         ── G-R
P5  16.2–16.5 ; 03.7 ; 17.1 (Feed) ; 09.4                                                           ── G-M
P6  12.2–12.4 ; 13.3–13.6 ; 20.2 ; 22.* ; 04.4/04.5 ; 17.2–17.4 ; 19.2 ; 18.1–18.4                  ── G-E
P7  21.*                                                                                            ── G-V
P8  15.8 ; 18.5–18.7 ; 24.* ; Windows 03.8 ; Soak                                                   ── G-P
```
**Audit-Abhängigkeit:** E-46 bis E-82 werden am jeweils ausgewiesenen Gate geschlossen. Alle API-/Port-Entscheidungen (insbesondere `SpaceId u64`, E-72) sind vor G-0 normativ; G-F friert `SYS_IDEM`, Branch-Merge-Basis/ForkPoint/ChangeIndex, positionsunabhängiges Blob-AAD, Space-ID-Allocator, PurgeBatch-/Complete-Edits, Ledger-Domänen/Byteordnung und Extension-Manifest/Attestationsformat samt Referenzvektoren ein. G-E/G-P verlangt die Gesamtsystem-Erasure-, DiskFull-, Lokalitäts- und RAM-Abnahmen; A-24/Fencing wird bei G-CUTOVER operational nachgewiesen.

Kritischer Pfad bis G-F: 01.1 → 01.2 → 02.1 → 03.1 → 03.2 → 05.1 → 05.2 → 06.x → 07.x → 19.1 → 13.2 → Freeze. Alles dahinter ist Format-stabil.

## 3 Neue Errata (blinde Flecken, Beschluss, Besitzer)

| ID | Befund | Beschluss | SP |
|---|---|---|---|
| E-29 | Altstand `MergeOperator` (WASM, Compaction-Zeit) vs. NZ-4 (Tag 6 reserviert) | Kern bleibt merge-frei. Bridge emuliert Read-Modify-Write über `Guard::Version` + Retry; Compaction-Merge entfällt, Altdaten-Semantik wird beim Cutover geprüft | BRIDGE 18.5 |
| E-30 | Altstand ist `async`, Kern synchron; Future-Cancel nach Übergabe an den Committer verliert das `Receipt` | Bridge: eigener Blocking-Pool, **IdemToken Pflicht** bei jedem `apply` der Bridge; Cancel ⇒ Retry mit gleichem Token ⇒ `AlreadyApplied` | BRIDGE 18.1 |
| E-31 | `rollback_to_tx`, `clear_intent_locks_*` haben kein Kernäquivalent (Historie unveränderlich) | Abbildung auf Branch: Transaktion = temporärer Branch, `branch_drop` = Rollback; sonst `Unsupported` mit Migrationshinweis | BRIDGE 18.3, BRANCH |
| E-32 | Altstand-Schlüsselraum ist flach + `TenantKeyCodec`-Präfix; Kern erwartet `(space, cluster, branch, key)` | Space `LEGACY_KV` (Plain, `clustered`): ein Plain-Cluster je Tenant; Präfix-Codec entfällt. Personenbezogene Daten **nicht** hierhin (kein Cryptographic) | BRIDGE 18.2 |
| E-33 | R8 hat keine Auffindung von Clustern nach Neustart (`Inspector` nur `testing`), `ClusterSpec` undefiniert | `Clusters::list(filter)` + `ClusterSpec{protection, pack, tag}`; `tag` opak ≤ 32 B, nur bei Plain im Klartext im Catalog, bei Sealed im versiegelten Cluster-Header (SYS-Eintrag) | CLUSTER |
| E-34 | `Guard::Range` (E-05) ist im Konfliktfenster (Ring je Schlüssel-Fingerprint) nicht prüfbar | zweiter Ring: grobe Präfix-Buckets `(space,cluster,branch,fnv(prefix≤8B))`; Treffer ⇒ exakter Langsampfad; Fallback bleibt korrekt | COMMIT |
| E-35 | R8 §4.6 widersprüchlich: `max_body_bytes` 16 MiB > `max_change_bytes` 8 MiB > `max_group_bytes` 4 MiB (SP-REDO E-27) | Kette `max_body ≤ ½·max_change`, `max_change ≤ redo_segment_bytes/8`: Defaults **body 4 MiB, change 8 MiB**, Gruppe darf nur bei **Einzelframe** `max_group_bytes` überschreiten, nie `redo_segment_bytes/8`. Größere Nutzlasten: Chunking oben (Schlüsselsuffix). Validierung in `Config::validate` | PORTS-Konstanten, FACADE |
| E-36 | `max_open_files` 1024 vs. Pure-Datei je großem Cluster/Level | Table-Cache (FD-LRU, Refcount, Pin während Lesen); Verzeichnisse bleiben im RAM, FDs nicht | RUN |
| E-37 | MVCC-GC-Regel unter Branches, Holds, Consumer-Fenstern nicht formal | `gc_floor = min(View-Stamps, Hold-Stamps, Consumer-Stamps, fork_stamp lebender Branches)`; ohne Pins `durable_stamp+1`. `history_floor` ist der tatsächlich verfügbare Historienrand, monoton und erst nach GC/Compaction fortgeschrieben; stets `history_floor <= gc_floor` | MAINT |
| E-38 | 16 Mio. Cluster ⇒ ~2 GiB RAM (128 B/Cluster, H-12) und lange Recovery | Kompakter `cluster_table`-Eintrag ≤ 48 B, Branch-Tabellen lazy; Standard `max_clusters` für Embedded **4 Mio.**, Anhebung per Config; Snapshot-Parse streaming | CLUSTER, CATALOG |
| E-39 | Ledger wächst unbegrenzt (append-only, Kette) | Ledger-Segmente mit Siegel-Eintrag (`seg_digest`), Archivierung über `ColdTier`; `verify_receipt` bekommt Segment-Slice | LEDGER |
| E-40 | `Relaxed` ohne Gegenstück zur Erzwingung der Durabilität | `Storage::durable_barrier() -> Receipt` (ein Gruppen-Sync) | COMMIT |
| E-41 | `IntegrityMode` (E-11) ungeklärt | **Instanzweit in `FORMAT`**, unveränderlich: `Plain` (CRC), `Authenticated` (Tag/Kette), `Encrypted` (Siegel); `personal` ⇒ `Encrypted` | RECOVERY |
| E-42 | Memtable-Löschung nach Gate (H-6: ≤ 10 ms unabhängig von Memtable-Größe) ist in einer globalen Skiplist ∝ Einträge | **Memtable partitioniert je Cluster** (`ClusterMap → Skiplist`): Gate = Partition aushängen (O(1)), Reclaim im `urgent`-Lane; Flush liefert Abschnitte direkt | MEMVER |
| E-43 | Bulk-Import (Cutover, Ingest 10⁶ Dok.) ohne Pfad | `Origin::Migration` + `Relaxed` + `durable_barrier()`; Gruppenbündelung; Rückdruck greift | CUTOVER, COMMIT |
| E-44 | Metrik-/Observer-Katalog nur als Liste | Ein Katalog mit Namen, Typ, Labels (keine Inhalte), Besitzer-SP in FACADE §3 | FACADE |
| E-45 | Redo-Frame passt nicht in Restsegment | Gruppe rotiert **vor** Schreiben; Frame > `redo_segment_bytes/8` ⇒ `ChangeTooLarge` (INV-LIM-1) | REDO/COMMIT |
| E-46 | Branch-Metadaten brauchen vorversiegelte Nutzlast, obwohl ID/Stamp sequenziert werden | `reserve_ids` außerhalb Rang 1; Branch-Frame aufruferseitig versiegeln. Commit-Stamp bleibt äußerer Record-Stamp und wird nicht im versiegelten Payload dupliziert. Sequencer validiert Reservierung und Branch-Head erneut; kein Provider unter Rang 1 | BRANCH/COMMIT |
| E-47 | Merge-Key-Guards übersehen konkurrierende `RangeDelete`/`EraseRange`; explizite Fork-Zeit fehlt | Intervallkonflikte werden gegen Punkt- und Bereichsmutationen geprüft; `ForkPoint::At(S)` verlangt `history_floor <= S <= parent_head`; `Head` wird im Commit linearisiert. `gc_floor` ist separat der früheste Retentions-Pin und darf `history_floor` nur begrenzen, nicht ersetzen | BRANCH/COMMIT |
| E-48 | 24-h-Idempotenz lebt nur im Replay-Bereich und kann nach `replay_floor`-Vorrücken verloren gehen | Aktive Token-Ergebnisse werden unabhängig vom Redo-Replay-Floor dauerhaft gehalten; Ablauf nur nach TTL und durabler `idem_trim`-Marke; Scope/Hash und Retired-Verhalten spezifizieren | COMMIT/CATALOG |
| E-49 | `range` hat keine kryptografische Selektivlöschung; Blob-Lochstrategie setzt optionale Hole-Punch-Unterstützung voraus | Keine Crypto-Löschzusage für Teilbereiche eines Cluster-Schlüssels. Hole-Ratio-Schwelle und Umschreiben in neue Blob-Inkarnation; Cluster-Key-Rotation bleibt separates, teures Verfahren und ist nicht Teil v1 | ERASE/BLOB |
| E-50 | `space_drop` umgeht Restrictions und ein aggregierter Tombstone könnte unklaren Restore-Scope lassen | Vor Gate `BLOCK_ERASE` prüfen; monoton reservierte SpaceId u64 nie wiederverwenden; durables `SpaceDropIntent` mit HMAC-Space-Tombstone und Scope-Digest vor Gate, GateApplied+Anchor vor Purge; Restore gate't gesamten Space. Space-Crypto-Erasure in v1 verboten, weil Cluster mehrere Spaces teilen können | ERASE/LEDGER/RECOVERY |
| E-51 | Memtable-Klartextreklamation ist bei Skiplist-Aliasing/`Bytes`-Clones nicht garantiert | Verbindliche Reclamation-Regel: keine unkontrollierten Klartext-Clone-Pfade; Zeroize bei letzter exklusiver Freigabe, Gate invalidiert Generation und wartet/markiert aktive Leser. OS-/Swap-Reste bleiben Restrisiko; Zeroize-Container allein ist kein Beweis | MEMVER/READ |
| E-52 | Blob-Purge mit unveränderten Offsets ist auf FS ohne Hole-Punch unbeschränkt teuer; Plain-Barriere invalidiert Feed-Consumer | Abschnittsweise neue Inkarnation ab konfigurierter Hole-Ratio; `Plain`-Barriere liefert explizites Feed-Epoch/`Lagged`-Signal und dokumentierte Rebuild-Route, nie stilles Überspringen | BLOB/ERASE/EXT |
| E-53 | Extension-Watermark ist nicht explizit an durable Kern-Sicht gebunden | Indexierung darf nur `durable_stamp` bestätigen; `view()`-Daten sind nicht als dauerhaft angewandt quittierbar. `filter_visible` ist obligatorisch und wird an der Query-Fassade erzwungen | EXT/READ |
| E-54 | `Mixed`-Segmente haben keine per-Cluster-Krypto-/RAM-/Löschkosten-Garantie | Jeder Segmentbesitzer erklärt Schutzdomäne, Datenanteil, Rewrite-Kosten, RAM-Budget, Gate-Filter und Attestationsformat. Für `Sealed` nur clusterweise verschlüsselte Sektionen oder gleichwertige Schlüsselvernichtung; sonst keine Crypto-Erasure-Attestation | EXT/ERASE |
| E-55 | Gesamtsystem-RAG-Atomarität und `DeletionProof` v3 sind nicht durch den Storage-Receipt abgedeckt | Ein Change kann KV-/Dokument-/Vektor-Payloads atomar persistieren, nicht globalen HNSW-Rewire. Derived-Index-Watermarks und signierte obere Proof-Brücke als getrennte Verträge; kein R8-Receipt behauptet Gesamtsystemlöschung | EXT/BRIDGE |
| E-56 | Providerfehlerzustände `Unavailable`, `Destroyed`, `Disabled`, `Lost` sind semantisch nicht gleichwertig | Nur bestätigtes `Absent` erfüllt Crypto-Erasure; Nichterreichbarkeit/Verlust blockieren bzw. ergeben `Incomplete`, nie `KeysDestroyed` | KEYS/ERASE/RECOVERY |
| E-57 | Redo-Schlüssel-Domäne und Recovery-Fehlerzustände waren widersprüchlich/mehrdeutig | Sealed-Redo-Op unter Cluster-Epoch-Key; RootKey-Redo-Key nur Ketten-MAC. `KeyUnavailable` hält Recovery in `LockedKeys`; bestätigtes Fehlen ohne Erasure-Nachweis ist `KeyLost/Corrupt` mit Quarantäne, niemals stilles Überspringen oder `Shredded` | REDO/RECOVERY/KEYS |
| E-58 | Blob-AAD an `blob_id/offset` würde Rohkopie über Repack verhindern | AAD bindet stabile `record_uid` plus Instanz/Space/Cluster/Epoche; nie Datei-/Blob-ID oder Offset; Replay über Clustergrenzen bleibt durch AAD ausgeschlossen | CODEC/BLOB/PURGE |
| E-59 | Branch-Diff-Kosten waren nur an Branch-Einträge gebunden, obwohl Parent-Änderungen seit Fork zählen | Cluster-sealed stamp-sortierter `ChangeIndex` erfasst Punkt- und Intervallmutationen beider Seiten; Diff proportional zu Mutationen seit gemeinsamem Floor plus Ausgabe | BRANCH/RUN/MAINT |
| E-60 | Extension-Timeout konnte Purge-Liveness unbegrenzt blockieren und verspätete Atteste waren nicht gefenced | Owner-Deadline, Generation/Fencing, `Blocked` + Ledger/Alarm; keine Done ohne gültige Atteste. Persistente Derivat-Consumer müssen registriert sein | EXT/ERASE/LEDGER |
| E-61 | `weight_sum`/Tokenprofile im Klartext-Directory widersprechen `INV-SEAL-1` | Sealed SubIndex und Filter sind cluster-sealed; Directory enthält keine Gewichte, `page` entsiegelt nur kleine Metadaten und liest keinen Body zur Skip-Entscheidung | RUN/READ |
| E-62 | `ClusterLocal` und INV-ERA-11 versprachen zu enge Purge-Kosten trotz gemeinsamem Redo und Pins | Getrennte Bounds für Run/Blob, Redo-Retention, Checkpoint-Pins und Extensions; keine pauschale O(Clusterbytes)-Gesamtaussage | ERASE/PURGE/VERIFY |
| E-63 | DiskFull-Scratch-Recycling war mit Reflink-/Hardlink-Checkpoint-Pins nicht abgesichert | Harte Garantie nur für unabhängige Kopien oder attestiert entfernte Referenzen vor Scratch-Recycling; sonst Receipt-Restrisiko und kein entsprechendes Done | CKPT/PURGE/LEDGER |
| E-64 | TTL/Deadline allein auf UTC erlaubt Clock-Rollback oder unplausible Sprünge | Monotone Laufzeit plus UTC und durabler `clock_floor`; bei Sprüngen Retention-/Hold-Übergänge stoppen und `ClockUncertain` melden | CLOCK/ERASE/BRANCH |
| E-65 | `DerivedFrom` über mehrere Löschdomänen kann beim Löschen eines Subjects fremde Daten kaskadierend entfernen | Cross-subject Kaskaden verbieten; getrennte Ableitungen oder nicht-kaskadierende Referenzen, validiert vor Link-Commit | CLUSTER/ERASE |
| E-66 | Limits und Branch-Tiefen-Annahmen waren zwischen Spec, Config und Messzielen nicht deckungsgleich | `max_body_bytes=4 MiB`, `max_change_bytes=8 MiB`; H-21 misst die konfigurierte Max-Tiefe, Default bei Verfehlung senken oder Branches flatten | PORTS/FACADE/BRANCH/VERIFY |
| E-67 | Per-Datei Purge-Install und endgültiger Abschluss waren als ein `PurgeEdit` missverständlich | `PurgeBatchEdit` pro betroffener Datei/Inkarnation, Unlink nach Reader-Drain, finales `PurgeCompleteEdit` erst nach allen Segment-Attesten | CATALOG/PURGE/ERASE |
| E-68 | Referenzcode leitet vor XChaCha noch einen HKDF/HChaCha-Subkey ab; dies widerspricht der festgelegten direkten XChaCha-Siegelform | Quellschlüssel ist bereits nach Schutzdomäne/Epoche scoped; XChaCha erhält ihn direkt. `Authenticated` nutzt domain-separated HMAC über Modus/Nonce/AAD/Payload. Vor G-F Code und unabhängige Referenzvektoren angleichen; keine Formatfreigabe bei Abweichung | CODEC/KEYS/VERIFY |
| E-69 | Transitive `DerivedFrom`-Kaskade kann durch viele gültige Links größer als ein atomarer Gate-Change werden | Link-Admission begrenzt jede mögliche Löschhülle auf `max_gate_clusters=4096`; Hüllenberechnung außerhalb Rang 1, Graph-Generation bei Commit erneut prüfen; Überlauf ⇒ `EraseFanoutExceeded`, keine Mutation | CLUSTER/COMMIT/ERASE |
| E-70 | Geteilte HNSW-/BM25-/CSR-Topologie überlebt Cluster-Key-Zerstörung und kann O(Index)-Rewrite erfordern | Clusterreine Payload+Beziehungen dürfen `MixedPerClusterKey` nutzen; alle clusterübergreifenden Strukturen sind bounded `Rebuildable`, am Gate sofort query-gefiltert und erst nach Rewrite/Verify attestiert | EXT/ERASE/BRIDGE |
| E-71 | Catalog-Gate vor Ledger-Tombstone lässt Crash/Restore ein angenommenes Delete aus älterem Backup wiederbeleben; Range-Gates sind ohne Selector ebenfalls nicht rekonstruierbar | Versiegeltes Ledger-Intent (inkl. verschlüsseltem Range-/Branch-Scope) vor Gate; `GateApplied` und Anchor danach, Anchor vor KeyDestroy/Purge; Recovery setzt Intent vor Open fort | ERASE/LEDGER/RECOVERY/ANCHOR |
| E-72 | `SpaceId u16` begrenzt die Lebensdauerzahl von Spaces unnötig und Wiederverwendung nach Catalog-Trim wäre für Space-Drop-Reconcile unsicher | Monoton reservierte `SpaceId u64`, dauerhafter High-Water-Mark (`reserve_space_ids`), Systemzone 0…255, User-IDs ab 256; `Exhausted` statt Wrap; Frame/Catalog/Gate/HMAC-Vektoren vor G-F | PORTS/CATALOG/CODEC/ERASE |
| E-73 | Lokaler `LOCK` und Rollback-Anchor verhindern keine gleichzeitig schreibenden Restore-Kopien auf verschiedenen Hosts | Restore erhält `InstanceId`, benötigt externen Fencing-Nachweis vor Schreiböffnung; ohne Nachweis bleibt `RestoreQuarantine`. G-CUTOVER enthält Zwei-Host-Rehearsal | CKPT/RECOVERY/CUTOVER |

## 4 Querschnittsregeln für alle SPs
1. Kein `unwrap/expect/panic/indexing`; `Result` mit typisierter Fehlerklasse (F-1…F-9); Fehlertexte ohne Inhalte (Canary-Test).
2. Jeder Decoder: Fuzz-Ziel + Rundreise-Proptest + goldene Vektoren (nach G-F).
3. Jede Invariante aus SPEC §15 steht als `// INV-…` im Test des Besitzers (Zitierprüfung, CI).
4. Jeder Schreibweg ist in `FaultFs`-Crash-Sweep aufgenommen, **wenn er geschrieben wird** (nicht nachgezogen).
5. Keine Sperre über Rückrufen (`Observer`, `KeyProvider`, `SegmentOwner`); Rangstack aktiv.
6. Jeder SP liefert: Schnittstelle, Regeln, Fehler, Invarianten/Tests, Errata, WP-Tabelle, Abnahme (Schema aus 01–05).

## 5 Aufwand (PW, Summe je SP; Schätzung, keine Zusage)
01–05 ≈ 39 · 06: 8 · 07: 11 · 08: 7 · 09: 9 · 10: 12 · 11: 12 · 12: 9 · 13: 12 · 14: 8 · 15: 18 · 16: 12 · 17: 8 · 18: 12 · 19: 8 · 20: 6 · 21: 12 · 22: 7 · 23: 8 · 24: 10 ⇒ **≈ 266 PW** (≈ 15 Monate bei 4 Entwicklern + 1 Test; deckt SPEC §14 ± Verifikationsaufwand).

## 3 Revisionsarbeitspakete (Ergänzung zum Graphen)

Normativ: SPEC §12.8, Anhang B.2 und B.3. Jeder SP führt die ihm zugewiesenen Ergänzungen im Revisionsnachtrag; Zuordnung, Reihenfolge und Abnahmen stehen im Audit-Register. Bei Widerspruch gilt ausdrücklich SPEC §15 vor §12.8 vor dem übrigen Text der SPEC.

| WP | SP | Phase | Voraussetzung | Inhalt |
|---|---|---|---|---|
| 01.5 | PORTS | P0 (vor G-0) | 01.2 | Typen: `KeyProvider`-Angleichung, `HostProbe`, `BlockPruner`, `MeasureKind`, `BackupMark`, Fehlerklassen |
| 20.4 | LEDGER | Slot vor G-F, Ablauf P6 | 20.1, 04.7 | Tomb-Epochen, Retire, `restore_floor`, Gerätetrennung |
| 03.9 | ENV | P5 | 03.3, 01.5 | `HostProbe`-Adapter |
| 07.6 | RUN | P4 | 07.5 | RAM-Präfixsummen (K-30) |
| 11.6 | READ | P4 | 11.4, 01.5 | View-Lease, Pruner-Hook |
| 04.7 | KEYS | P6 (Deklaration P0) | 04.2, 01.5 | Assurance/KeysInBackups/Tomb in Providern |
| 13.7 | ERASE | P6 | 13.1, 20.4, 14.4 | Löschauftrag auf `Poisoned` |
| 13.8 | ERASE | P6 | 13.5, 04.7 | Statuscaps, `MeasureKind` |
| 14.4 | RECOVERY | P6 | 14.3, 20.4 | R2a/R8-Ergänzungen, Restore-Floor |
| 22.4 | CKPT | P6 | 22.3, 20.4 | `BackupMark`, Restore-Prüfung |
| 17.6 / 17.7 | EXT | P6 | 17.2, 11.6 / 17.3 | `block_boundary`, Cache-Konvention |
| 23.5 | FACADE | P6 | 23.2, 13.7, 20.4 | Config/`OpenProfile`, API, Fehler, Metriken |
| 15.10 | VERIFY | P6 | 15.3, 13.7, 20.4, 11.6 | Messarten, Modelle, neue INV-Tests, H-23 |
| 24.8 | CUTOVER | P8 | 24.1, 23.5 | Caller-Matrix für R8.2-Schnittstellen |

Kritischer Pfad bis G-F ändert sich nur um den Format-Slot `tomb_epoch` in 20.4 (Ledger-Format ist Teil des Freeze, SPEC §5.9); alles andere liegt hinter G-F oder ist rein typbezogen (01.5, vor G-0).
