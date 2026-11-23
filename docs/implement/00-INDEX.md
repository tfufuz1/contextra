# INDEX — Implementierungsspezifikationen `contextra-storage` (R8)

| | |
|---|---|
| Normativ | `docs/SPEC-R8.md`; je Komponente `docs/implement/NN-*.md` |
| Status 01–05 | vorhanden (ports, codec-seal, env, keys, redo) |
| Status 06–24 | dieser Satz |
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
| E-37 | MVCC-GC-Regel unter Branches, Holds, Consumer-Fenstern nicht formal | `gc_floor = min(history_floor, fork_stamp lebender Branches, Hold-Stamps)`; Regel für Versions-Verwurf in MAINT §3.4 | MAINT |
| E-38 | 16 Mio. Cluster ⇒ ~2 GiB RAM (128 B/Cluster, H-12) und lange Recovery | Kompakter `cluster_table`-Eintrag ≤ 48 B, Branch-Tabellen lazy; Standard `max_clusters` für Embedded **4 Mio.**, Anhebung per Config; Snapshot-Parse streaming | CLUSTER, CATALOG |
| E-39 | Ledger wächst unbegrenzt (append-only, Kette) | Ledger-Segmente mit Siegel-Eintrag (`seg_digest`), Archivierung über `ColdTier`; `verify_receipt` bekommt Segment-Slice | LEDGER |
| E-40 | `Relaxed` ohne Gegenstück zur Erzwingung der Durabilität | `Storage::durable_barrier() -> Receipt` (ein Gruppen-Sync) | COMMIT |
| E-41 | `IntegrityMode` (E-11) ungeklärt | **Instanzweit in `FORMAT`**, unveränderlich: `Plain` (CRC), `Authenticated` (Tag/Kette), `Encrypted` (Siegel); `personal` ⇒ `Encrypted` | RECOVERY |
| E-42 | Memtable-Löschung nach Gate (H-6: ≤ 10 ms unabhängig von Memtable-Größe) ist in einer globalen Skiplist ∝ Einträge | **Memtable partitioniert je Cluster** (`ClusterMap → Skiplist`): Gate = Partition aushängen (O(1)), Reclaim im `urgent`-Lane; Flush liefert Abschnitte direkt | MEMVER |
| E-43 | Bulk-Import (Cutover, Ingest 10⁶ Dok.) ohne Pfad | `Origin::Migration` + `Relaxed` + `durable_barrier()`; Gruppenbündelung; Rückdruck greift | CUTOVER, COMMIT |
| E-44 | Metrik-/Observer-Katalog nur als Liste | Ein Katalog mit Namen, Typ, Labels (keine Inhalte), Besitzer-SP in FACADE §3 | FACADE |
| E-45 | Redo-Frame passt nicht in Restsegment | Gruppe rotiert **vor** Schreiben; Frame > `redo_segment_bytes/8` ⇒ `ChangeTooLarge` (INV-LIM-1) | REDO/COMMIT |

## 4 Querschnittsregeln für alle SPs
1. Kein `unwrap/expect/panic/indexing`; `Result` mit typisierter Fehlerklasse (F-1…F-9); Fehlertexte ohne Inhalte (Canary-Test).
2. Jeder Decoder: Fuzz-Ziel + Rundreise-Proptest + goldene Vektoren (nach G-F).
3. Jede Invariante aus SPEC §15 steht als `// INV-…` im Test des Besitzers (Zitierprüfung, CI).
4. Jeder Schreibweg ist in `FaultFs`-Crash-Sweep aufgenommen, **wenn er geschrieben wird** (nicht nachgezogen).
5. Keine Sperre über Rückrufen (`Observer`, `KeyProvider`, `SegmentOwner`); Rangstack aktiv.
6. Jeder SP liefert: Schnittstelle, Regeln, Fehler, Invarianten/Tests, Errata, WP-Tabelle, Abnahme (Schema aus 01–05).

## 5 Aufwand (PW, Summe je SP; Schätzung, keine Zusage)
01–05 ≈ 39 · 06: 8 · 07: 11 · 08: 7 · 09: 9 · 10: 12 · 11: 12 · 12: 9 · 13: 12 · 14: 8 · 15: 18 · 16: 12 · 17: 8 · 18: 12 · 19: 8 · 20: 6 · 21: 12 · 22: 7 · 23: 8 · 24: 10 ⇒ **≈ 266 PW** (≈ 15 Monate bei 4 Entwicklern + 1 Test; deckt SPEC §14 ± Verifikationsaufwand).
