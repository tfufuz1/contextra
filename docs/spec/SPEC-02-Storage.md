# SPEC-02 — Storage-Kernel

Crates: `contextra-sys`, `contextra-durable-fs`, `contextra-codec` (L0/L1), `contextra-wal` (L1), `contextra-store` (L2). Memtable-Logik und Branch-Sichtbarkeit: `contextra-mvcc` (L3, SPEC-03). Alle Aussagen sind normativ.

## Inhalt
1. Verzeichnislayout und Dateikonventionen · 2. Dateikopf, Blockrahmen, Kodierungen · 3. WAL · 4. Memtable · 5. Segmentfamilien · 6. Truth-Segmente (Content, Attr, übrige), Lane-Patches, Archiv-Tier · 7. Death-Segmente und Liveness · 8. Manifest-Log und Superblock · 9. Seal- und Checkpoint-Protokoll · 10. Recovery · 11. Compaction, Retention, Purge · 12. Blob-Store, GC · 13. Block-Cache und I/O · 14. Verschlüsselung · 15. Integrität, Scrub, Verify · 16. Backup, Restore, Replikationsfähigkeit · 17. Sidecars · 18. Fehlerbehandlung

---

## 1. Verzeichnislayout und Dateikonventionen

```
store/
  CURRENT                          # doppelt gepufferter Superblock (2 × 4 KiB Slots, Generationszähler, CRC)
  LOCK                             # exklusiver Prozess-Lock (flock / LockFileEx)
  keys/keyring.ckr                 # Schlüsselring (nur bei Verschlüsselung): KEK-gewickelte DEK-Epochen, repo_seed
  wal/000042.cxw …                 # 64-MiB-WAL-Segmente, vorallokiert; ein gemeinsamer Log für alle Branches
  manifest/000007.mlog …           # append-only Manifest-Log (CRC32C + BLAKE3-Kette)
  truth/content-<id>.cseg          # Content-Segmente  (relation = content)
  truth/attr-<id>.cseg             # Attr-Segmente     (relation = attr)
  truth/{edge,seq,idmap,prov,catalog,commit,alog}-<id>.cseg
  truth/arch-<id>.cseg             # Archiv-Tier: L0-Rümpfe zoom-down-archivierter Cells (§6.5)
  truth/death-<id>.cdth            # Death-Segmente (§7)
  derived/lane-<id>.clane          # Lane-Patches: nachgelieferte Embeddings, L1, Token-Zähler (§6.6)
  index/<segment-id>/<kind>-<paramhash>.cidx   # abgeleitete Index-Segmente (SPEC-04)
  blobs/<aa>/<hash-hex>            # große Payloads und Quelldokumente, content-addressed
  sidecar/{idempotency,work_queue,jobs,access_stats}.cxs
  tmp/                             # Schreibziele vor atomarem Rename
```

**Dateiregeln (verbindlich)**
- Alle Dateien sind **write-once**, außer WAL-Tail, Manifest-Tail, Superblock und Sidecars. Ersetzung = neue Datei + Manifest-Eintrag.
- Persistenzsequenz einer neuen Datei: Schreiben nach `tmp/` → `fdatasync(datei)` → `rename` → `fdatasync(verzeichnis)`. Nur `contextra-durable-fs` darf diese Operationen ausführen (P4, CI-Lint „keine `sync_*`/`fsync` außerhalb"). Windows: `FlushFileBuffers` + `MoveFileEx(MOVEFILE_WRITE_THROUGH)`; die Testsuite `DurableFs` belegt die Semantik pro Plattform.
- Dateien, die in keinem Manifest-Zustand referenziert sind, heißen **verwaist**; GC entfernt sie nach der Karenzzeit (§12).

---

## 2. Dateikopf, Blockrahmen, Kodierungen

**Allgemeiner Dateikopf (64 Byte, Little-Endian):**
`magic[4] | format_version u16 | compat_flags u16 | incompat_flags u32 | file_kind u8 | codec_default u8 | reserved u16 | file_id u128 | created_hlc u64 | key_epoch u32 | reserved u32 | header_crc32c u32`.
Magics: `CWAL` (WAL), `CMLG` (Manifest-Log), `CSEG` (Truth-Segment), `CDTH` (Death), `CLAN` (Lane-Patch), `CIDX` (Index), `CBLB` (Blob), `CSID` (Sidecar), `CKRG` (Schlüsselring).
**INV-11:** unbekannte `incompat_flags` ⇒ Öffnen schlägt fehl (`E_FORMAT_INCOMPATIBLE`); unbekannte `compat_flags` werden ignoriert.

**Blockrahmen:** `len u32 | codec u8 | flags u8 | reserved u16 | crc32c u32 | payload[len]`. CRC32C über die gespeicherten Bytes (bei Verschlüsselung: Chiffretext + Tag). Logische Blockgröße ≤ 16 KiB (Lanes), Index-/Vektorsektoren 4 KiB.

**Codecs:** `0` roh · `1` zstd (Level 3) · `2` zstd + Wörterbuch (je MemoryClass, ab 10 000 Trainingsproben; Wörterbuch als Blob im Katalog) · `3` Bitpacking (FOR) · `4` Delta-Varint · `5` RLE · `6` Dictionary-Kodierung · `7` Roaring (portable Serialisierung).
**Ausrichtung:** 8-Byte-Offsets, Vektorregionen und Index-Sektoren 4-KiB-aligned. Endianness: Little-Endian.
**Serialisierung:** On-Disk-Records als **`rkyv`-Archive** mit versionierten Schemata; Prüfung per `bytecheck` **einmal beim Laden** eines Blocks, danach Vertrauen am Cache-Rahmen (Flag `validated`). Zero-Copy-Zugriff erfolgt ausschließlich über validierte Rahmen (`unsafe` nur in `contextra-codec`, SPEC-09 §4).

---

## 3. Write-Ahead-Log (`contextra-wal`)

### 3.1 Aufbau
- Dateien `wal/NNNNNN.cxw`, **64 MiB**, per `fallocate`/Nullfüllung vorallokiert (konstante Dateigröße ⇒ `fdatasync` ohne Metadaten-Update). Das Ende des gültigen Logs erkennt die Recovery am ersten ungültigen Record.
- **Record:** `len u32 | crc32c u32 | lsn u64 | type u8 | flags u8 | reserved u16 | branch u32 | txn u64 | payload`. CRC über alle Bytes nach dem CRC-Feld. Payload ist **logisch und idempotent** (rkyv). Bei Verschlüsselung ist der Payload AEAD-verpackt, Nonce = `file_id ‖ lsn` (§14).
- `lsn` ist ein monotoner Record-Zähler über alle WAL-Segmente. `Seq` (Commit-Zähler) wird erst im `TxnCommit`-Record vergeben (K-30).

### 3.2 Record-Typen
| Typ | Inhalt |
|---|---|
| `TxnBegin` | `txn`, Branch, Snapshot-Seq, Durability-Klasse |
| `TxnChunk` | Folge von Operationen (≤ 1 MiB je Record): `PutCell{cell_id, content?, attr}`, `PatchAttr`, `Retire`, `PutEdge`, `RetireEdge`, `SeqInsert`, `SeqRemove`, `ProvAppend`, `CatalogOp`, `CommutativeDelta` |
| `TxnCommit` | `seq`, `hlc`, `idem_key`, `idem_payload_hash`, `durability`, `ops_count`, `ops_hash`, `truth_hash_after`, `changes_root`, `gid_base` (erste vergebene Gid), `author`, `intent_hash` |
| `BranchOp` | `Create{name, parent, fork_seq, policy}`, `Delete`, `Rollback{to_seq}`, `PolicyUpdate`, `Tag` |
| `Derivation` | `PutDerivation{content_hash, producer, value_ref}` (idempotent, branch-unabhängig) |
| `SidecarOp` | `IdemPut/IdemExpire`, `WqEnqueue/WqLease/WqComplete/WqFail`, `JobUpsert` (§17) |

- Eine Transaktion ist **sichtbar und durable erst mit ihrem `TxnCommit`**. `TxnBegin`/`TxnChunk` ohne `TxnCommit` werden bei der Recovery verworfen.
- `idem_key` steht **im selben `TxnCommit`-Record** wie der Commit: Idempotenz-Eintrag und Commit sind atomar (INV-6).
- Der Idempotenz-Sidecar wird bei Replay aus diesen Feldern rekonstruiert.

### 3.3 Group Commit
- **Leader/Follower:** Der erste wartende Commit wird Leader, sammelt weitere Records (über alle Branches) bis **200 µs** oder **1 MiB**, schreibt sie sequentiell, führt **einen** `fdatasync` aus und weckt alle Follower. Ack erst danach (Klassen `Group`, `Strict`).
- `Strict`: jeder Commit ein eigener `fdatasync` (keine Gruppierung); zusätzlich `fdatasync(verzeichnis)` bei Segmentwechsel.
- `Relaxed`: Ack nach `write()`; ein Timer-Thread synct spätestens alle 100 ms. `Volatile`: kein WAL-Record; Branch und Rows sind nach Neustart verloren und der Branch wird entfernt.
- Gemischte Gruppen: Der `fdatasync` deckt alle Klassen ab; `Relaxed`-Commits profitieren, ohne Verzögerung zu erzeugen.

### 3.4 Torn Writes, Recycling
- Erkennung durch Länge + CRC + monotone LSN; die Recovery stoppt am **ersten** ungültigen Record (Präfixkonsistenz, INV-3). Ein ungültiger Record **mitten** in nachweislich später gültigen Daten (gültiger Record mit größerer LSN danach) ist Korruption ⇒ `E_CORRUPT_TRUTH` (kein Weitermachen).
- Segmente unterhalb `checkpoint_lsn` (aus dem Manifest) werden recycelt: **neue `file_id`** (und damit neuer Dateischlüssel/Nonce-Raum), Kopf neu geschrieben, Rumpf nicht gelöscht aber durch neuen Kopf entwertet; alternativ gelöscht und neu allokiert (Profil `lite`: löschen).

---

## 4. Memtable

Je Branch eine Memtable-Familie, geführt von `contextra-mvcc`, Datenstrukturen aus `contextra-store`:

| Teil | Struktur |
|---|---|
| Content-/Attr-Zeilen | `crossbeam-skiplist`, Schlüssel `(gid, row_kind, born_seq)` |
| Kanten | Skiplist `(src, kind, dst, edge_id, born_seq)` |
| `seq`, `prov`, `idmap`, `catalog`, `alog`, `commit` | je eine Skiplist |
| Vektor-Arena | append-only Arena je Space (F16/F32); Länge per Release/Acquire veröffentlicht |
| Delta-Indizes | **nicht** im Store, sondern als `DeltaSink`-Implementierungen aus L4 (Flat-Puffer bis 20 000 Vektoren, danach Mini-HNSW; Mini-Invertindex; Kanten-Overlay), SPEC-04 §8 |

- Lesen sieht `Basis (Segmente) ⊕ Memtable` über einen Merge-Iterator, gefiltert durch `born_seq ≤ snapshot.seq`.
- Frisch geschriebene Daten sind damit **sofort in allen Modalitäten** auffindbar (100 % `lane_coverage` für den Memtable-Anteil).
- **Seal-Auslöser:** Schwelle des Profils (SPEC.md §4.2), explizites `flush`/`COMMIT 'msg'`, Druckstufe `Elevated`, Branch-Quote, Merge-Vorbereitung. Session-Branches versiegeln erst bei ≥ 64 000 Zeilen bzw. 8 MiB zu **einem** kleinen Segment je Familie (Segment-Explosion vermeiden).
- Speicher wird per Epochen-Reklamation (`crossbeam-epoch`) freigegeben, nachdem `checkpoint_lsn` den Memtable-Anteil abdeckt und kein Snapshot ihn mehr referenziert.

---

## 5. Segmentfamilien

| Familie | Zeilen | Sortierung | Heiß/Kalt |
|---|---|---|---|
| `content` | Content-Zeilen | `(gid, born_seq)` | Hot-Head/Cold-Tail (§6.1) |
| `attr` | Attr-Zeilen | `(gid, born_seq)` | durchgehend hot (schmal) |
| `edge` | Kanten | `(src, kind, dst, edge_id, born_seq)` | gemischt |
| `seq` | Geschwisterfolge | `(parent, order_key, born_seq)` | hot |
| `idmap` | `CellId ↔ Gid`, `ExternalKey` | `(gid)` + Sortierung nach `CellId` (zwei Spaltenordnungen) | hot |
| `prov` | ProvRecords | `(subject_gid, ev_seq)` | cold |
| `catalog` | Katalogeinträge | `(kind, name, born_seq)` | hot |
| `commit` | Commit-Log (Seq, Branch, Hlc, truth_hash, changes_root, chain_hash, Autor, Intent) | `(seq)` + Hash-Spalte sortiert | warm |
| `alog` | Action-Log der Sessions | `(session, step)` | cold |

Die Trennung `content`/`attr` sorgt dafür, dass **reine Attributänderungen (Tags, ACL, Trust, `valid_to`, `flags`) nur kleine Attr-Segmente schreiben** und den schweren Content-Rumpf nie anfassen.

Jedes Segment trägt im Footer `relation`, `branch` (Eigentümer), `born_seq_min/max`, `gid_min/max`, Zeilenzahl, Roaring64 der enthaltenen Gids (je Zeilenart), Gid-Bloom (für Punktlesen), Spaltenstatistiken, Extent-/Block-Index, BLAKE3 über das Objekt.

---

## 6. Truth-Segmente

### 6.1 Content-Segment: Extent-Layout (Hot-Head / Cold-Tail)

```
Segment := Header(4 KiB) | Extent* | Footer
Extent  := ExtentHeader | HOT HEAD (≤ 64 KiB) | COLD TAIL
            Dir+CRC       keys · cellmeta · hashes · tok · l2 · emb.q   raw · emb.full(4 KiB-aligned) · props.ext
```
- **Zeilen je Extent:** adaptiv 16–256 (Ziel 128); Extent ≤ 1 MiB; der Hot-Head-Budget von 64 KiB begrenzt die Zeilenzahl. `ord` ist die fortlaufende Zeilennummer im Segment; der Footer enthält je Extent `(first_gid, first_ord, offset, len)` als Fence-Index.
- **Spaltenorientiert (PAX):** Innerhalb des Extents liegt jede Lane zusammenhängend (SIMD-freundlich für Distanzen auf `emb.q` und Prädikate auf `cellmeta`).
- **Lanes:** `keys` (Gid delta-kodiert, `born_seq`, `branch`, `flags`) · `cellmeta` (`cell_key[16]`, `kind`, `level`, `class`) · `hashes` (`content_hash[32]`, `entry_hash[16]`) · `tok` (u32 je Tokenizer) · `l2` (Offsets + Dictionary-Text) · `emb.q` (SQ8-Präfix, Default 256 Dim = 256 B/Zeile) · `raw` (zstd-Blöcke à ≈ 64 KiB, Wörterbuch je Klasse) · `emb.full` (F16/F32, ab 4-KiB-Grenze, zeilenweise adressierbar für `O_DIRECT`) · `props.ext` (rkyv).
- **I/O-Muster:** Ranking, Filter und Packen lesen **nur den Hot-Head** (ein Read von ≤ 64 KiB inkl. ExtentHeader). Gewinner lesen zusätzlich `raw` bzw. `emb.full` desselben Extents (zweiter Read). `emb.q` fehlt (null-bitmap), solange das Embedding nicht vorliegt (§6.6).
- **Kompression:** `raw` zstd-3 mit Wörterbuch je MemoryClass; Token-IDs optional (`tok.ids`, bitgepackt); Vektoren **unkomprimiert**.

### 6.2 Attr-Segment
Schmale, durchgehend heiße Spalten: `keys`, `parent`, `trust`, `taint`, `grounding`, `lcb`, `acl`, `flags`, `valid_from/to`, `expires_at`, `support`, `prov_head`, Low-Cardinality-Props (Dictionary-kodiert, je Prop eine Spalte), AttrBloom; `belief` und restliche Props in Cold-Spalten. Blockgröße 16 KiB.

### 6.3 Übrige Familien
Gleicher Segmentcontainer (Header, Blöcke, Footer), familienspezifische Spalten: `edge` (src/kind/dst delta-kodiert, weight, valid, trust, conf, flags, props); `seq` (parent, order_key, child); `idmap` (Gid-sortierte Hauptordnung + zweite sortierte Spalte nach `(ns, cell_key)` mit Gid-Verweis; `ExternalKey`-Wörterbuch als FST); `prov`; `catalog`; `commit`; `alog`.

### 6.4 Zurückgestelltes Affinity-Re-Layout
Gid-Monotonie liefert Zeitlokalität (zusammen eingelesene Dokumente/Episoden liegen benachbart). **Kein Locator, kein gelerntes Re-Layout in v1** (K-04). Reaktivierung nur, wenn (a) ein Benchmark-Workload mit wiederkehrenden Co-Retrieval-Mustern ≥ 20 % Cold-Tail-I/O-Reduktion im Prototyp zeigt **und** (b) der Hot-Head/Cold-Tail-Schnitt allein die Zielwerte nicht erreicht. Dann als Compaction-Ordnungsstrategie (neue Extents, Reihenfolge nach Co-Access-Cluster), ohne `Gid`-Änderung.

### 6.5 Archiv-Tier
Compaction darf den `raw`- und `emb.full`-Rumpf zoom-down-markierter Cells (`ARCHIVED` ∧ Alter ≥ `zoom_down_after_days`) in `truth/arch-*.cseg` verlagern (gleiche Extent-Struktur, nur Cold-Lanes); der Content-Segment-Eintrag behält Hot-Head (`l2`, `tok`, `emb.q`) und einen `ArchRef`. `expand` auf L0 und `include_archived` lesen aus dem Archiv-Tier; Default-Recall liest es nie. Das Tier ist die vorbereitete Schnittstelle zu einem späteren Objektspeicher.

### 6.6 Lane-Patches (`.clane`)
- **Inline-Regel (K-37):** Liegt zum Seal-Zeitpunkt ein Derivat (Embedding, Token-Zähler, L1) im Derivat-Cache (WAL-Record `Derivation` im Memtable), schreibt der Seal es **inline** in die Content-Zeile (`emb.q`/`emb.full`/`tok`/`l1`-Block).
- Sonst: Lane fehlt (null-bitmap). Trifft das Derivat später ein, schreibt der Dispatcher ein **Lane-Patch-Segment** `(lane_kind, space/tokenizer/producer, [(content_hash, gid_hint, value)])`, sortiert nach `content_hash`, plus Bloom. Registrierung im Manifest (`LaneAdd`). Lesen löst eine Lane auf als `inline ∨ Patch(content_hash)`.
- **Alle Zeilen mit gleichem `content_hash`** erhalten die Lane (Deduplizierung der Berechnung, nicht des Speichers).
- Lanes sind **nicht im Truth-Digest** und verändern keinen Commit. Compaction faltet Patches in neue Content-Segmente ein; danach `LaneRetire`.
- `lane_gen` (Zähler im Manifest) wird bei jeder Lane-Registrierung erhöht und erscheint im Explain (INV-15).
- Vektor-Index-Builds nehmen als Quellen **Content-Segmente und Lane-Patches**; Zeilen ohne Vektor stehen nicht im Index (`has_vec`-Bitmap).

---

## 7. Death-Segmente und Liveness

- **Death-Eintrag** `(family u8, segment u64, ord u32, died_seq u64)` entsteht beim Seal, wenn eine neue Zeile desselben Branches eine ältere sichtbare Zeile **im selben Branch** ersetzt oder retired. Zeilen von **Vorfahren-Branches** erhalten **nie** einen Death-Eintrag durch einen Kind-Branch (INV-12); ihre Maskierung geschieht über `Shadow` (SPEC-03 §6.2). Beim Merge nach `main` schreibt der Merge-Commit Deaths für die dort ersetzten Zeilen.
- **Death-Segmente** `.cdth`: sortierte Arrays; je Seal ein kleines immutables Segment; Compaction vereinigt sie.
- **Liveness:** Eine Zeile ist zum Snapshot `S` lebendig, wenn `born_seq ≤ S`, `born_seq ∉ aborted_ranges(branch)` und kein Death mit `died_seq ≤ S` existiert.
- **LivenessMap je Segment:** Roaring32 `dead_head` (Tod ≤ aktueller Branch-Head); wird beim Publizieren eines Manifests **copy-on-write** fortgeschrieben (persistente Struktur, wait-free Lesen). Zeitreise: `dead_at(S)` aus Death-Segmenten gefiltert und je `(segment, generation, S)` gecacht.
- Tote Zeilen bleiben im ANN-Graph als **Routing-Knoten** (Ergebnisse maskiert) bis zur Compaction; es gibt keine Graph-Reparatur.
- **`aborted_ranges`:** je Branch Liste von `(seq_lo, seq_hi)`-Bereichen nach `Rollback`; Zeilen mit `born_seq` darin sind unsichtbar (SPEC-03 §6.7). Compaction entfernt sie physisch, sobald kein Snapshot sie referenziert.

---

## 8. Manifest-Log und Superblock

**Manifest-Log** (`.mlog`): append-only; Eintrag = `len | crc32c | prev_blake3(16) | type | payload`. Die Verkettung dient der Torn-Tail-/Bit-Rot-Erkennung, **nicht** der Sicherheit.

| Eintragstyp | Wirkung |
|---|---|
| `SegmentAdd/Retire` | Truth-/Archiv-/Death-Segment hinzufügen/entfernen (inkl. Footer-Statistik-Kopie) |
| `LaneAdd/Retire` | Lane-Patch |
| `IndexAdd/Retire` | Index-Segment mit `source_segments`, `built_from_seq`, `paramhash`, `builder_version`; erhöht `index_gen` |
| `BranchCreate/Update/Delete`, `Tag` | Branch-Tabelle: Name, `BranchId`, Parent, `fork_seq`, `head_seq`, Policy, `aborted_ranges`, `RootSet` (voll für `main`/`dev`, sonst Delta zum Fork) |
| `Checkpoint` | `checkpoint_lsn`, `checkpoint_seq`, `lane_gen`, `index_gen`, `sidecar_checkpoint_lsn` |
| `Statistics` | aggregierte Planner-Statistiken |
| `ManifestSnapshot` | vollständiger Zustand; erlaubt das Löschen älterer `.mlog`-Dateien |

- **Manifest-Zustand** im RAM als unveränderliches `Arc<Manifest>`; Publikation je Branch per `ArcSwap` (wait-free Leser). Ein `Manifest`-Wert enthält Segmentlisten je Familie, `LivenessMap`s, Branch-Tabelle, IndexManifest (SPEC-04 §1), Lane-Registry.
- **Superblock `CURRENT`:** zwei 4-KiB-Slots (`generation u64`, `mlog_file`, `mlog_offset`, `repo_id`, `format_version`, CRC32C). Update schreibt den inaktiven Slot, `fdatasync`, ab dann ist er aktiv. Beim Öffnen gewinnt der gültige Slot mit der höheren Generation.
- `repo_seed` (32 B, zufällig bei `init`) liegt im Schlüsselring (verschlüsselt) bzw. im Superblock-Schlüsselabschnitt (unverschlüsselt). Daraus per HKDF: `hash_key`, `handle_key`, Bloom-/Hash-Seeds.

---

## 9. Seal- und Checkpoint-Protokoll (Reihenfolge verbindlich)

1. **Truth-Segmente schreiben** (je Familie mit Rows im Memtable-Abschnitt `(checkpoint_seq, seal_seq]`) nach `tmp/`; `fdatasync`; `rename`; Verzeichnis-`fdatasync`.
2. **Death-Segment** analog.
3. **Index-Segmente**, soweit synchron gebaut (Standard: nur `flat`/Mini-Strukturen; schwere Builds asynchron als Jobs).
4. **Manifest-Eintrag** `SegmentAdd + DeathAdd (+ IndexAdd) + Checkpoint` anhängen; `fdatasync` ⇒ **Strukturpunkt**.
5. Superblock `CURRENT` doppelt gepuffert aktualisieren (nur bei Manifest-Dateiwechsel nötig).
6. Neues `Arc<Manifest>` per `ArcSwap` publizieren; Memtable-Anteil bis `checkpoint_seq` per Epochen-Reklamation freigeben; WAL-Segmente unterhalb `checkpoint_lsn` recyceln.

Ein Absturz vor Schritt 4 hinterlässt verwaiste Dateien (GC entfernt sie), der Zustand ergibt sich aus WAL-Replay. Nach Schritt 4 liegt der Zustand vollständig im Manifest. **Der Commit-Punkt ist der WAL-fsync; der Strukturpunkt dient Replay-Kürze und WAL-Recycling** (K-08).

Der Seal erfasst nur Commits bis zu einer `seal_seq`, zu der **alle** vorangehenden Commits aller betroffenen Branches angewandt sind (ein einheitlicher Schnitt je Branch; branchübergreifend unabhängig).

---

## 10. Recovery

1. Superblock lesen (höhere gültige Generation), `.mlog` ab Slot-Offset verifizieren; Torn-Tail abschneiden.
2. Manifest-Zustand aus letztem `ManifestSnapshot` + Folgeeinträgen aufbauen; Branch-Tabelle, Lane-/Index-Registry, `checkpoint_lsn`.
3. Segment-Footer der referenzierten Dateien öffnen (Fence-Index, Bitmaps) und CRC der Footer prüfen. Verwaiste Dateien ignorieren.
4. **WAL-Replay** ab `checkpoint_lsn`: Records bis zum ersten ungültigen validieren; nur vollständige Transaktionen (`TxnCommit` vorhanden) anwenden: Memtable füllen, `RootSet` je Branch inkrementell fortschreiben und mit `truth_hash_after` jedes Commits vergleichen (Abweichung ⇒ `E_CORRUPT_TRUTH`), Sidecar-Ops anwenden, `BranchOp` anwenden.
5. Fehlende/defekte Index-Segmente neu einplanen. Abfragen laufen sofort (Flat-Fallback, `lane_coverage`).
6. `CURRENT` und Manifest sind danach konsistent; ein Seal wird bei Bedarf angestoßen.

**Ziel [Ziel]:** < 10 s je 500 MB WAL (Replay-Durchsatz ≥ 1 Mio. Zeilen/s je Kern-Gruppe; ohne Index-Neubau).

**Durability-Invarianten**
- **INV-D1:** Ein Manifest-Eintrag wird nie geschrieben, bevor die referenzierte Datei durable ist.
- **INV-D2:** Der Truth ist Quelle der Wahrheit; Indizes und Lanes sind rekonstruierbar bzw. ersetzbar.
- **INV-D3:** Fehler bei `fsync`/`fdatasync` ⇒ Store wechselt in `Failed` (Fail-Stop, „fsyncgate"): kein Wiederholungs-Sync derselben Datei, Neustart + Recovery.
- **INV-D4:** Commits sind unveränderlich.

---

## 11. Compaction, Retention, Purge

### 11.1 Strategie
- **Größenabgestuft (Size-Tiered)** je Familie und Branch-Lineage, Faktor 4–8, zeitlich zusammenhängende Bereiche. Tier-0 = Seal-Ausgabe. Trigger: ≥ 4 Segmente in einem Tier (Faktor 4 bei `scale` Bulk-Ingest: 8), oder Fan-out-Budget (> 24 lebendige Content-/Attr-Segmente je Branch-Sicht ⇒ Priorität ↑ via Druckstufe).
- **Overlay-Segmente von Branches** werden beim Merge in die Zielfamilie überführt, nicht früher kompaktiert (Ausnahme: Branch mit > 24 Segmenten).
- **Auftrag:** Eingabe-Segmente + zugehörige Death-Einträge ⇒ (a) Zeilen mit Death jenseits der Retention und ohne Pin entfallen; (b) Zeilen in `aborted_ranges` entfallen; (c) Lane-Patches werden eingefaltet; (d) neues Segment (bei Content: ≤ 8 Mio. Zeilen oder 4 GiB, danach Teilung); (e) Index-Neuaufbau (HNSW: Neuaufbau; Vamana: Streaming-Merge nach FreshDiskANN oder Neuaufbau nach Kostenmodell); (f) Manifest-Eintrag `SegmentAdd/Retire (+IndexAdd/Retire)` atomar in einem Eintrag.
- **Drosselung:** Token-Bucket auf I/O-Bandbreite (Default `lite` 50 MB/s, `standard` 400 MB/s, `scale` 1 500 MB/s, konfigurierbar) und Speicher (`jobs`-Budget). Bei `Critical` pausieren Index-Neuaufbauten; Seal hat Vorrang.
- **Write-Amplification [Ziel]:** < 5× (Fold/Seal ohne Compaction ≈ 1×; Size-Tier-Compaction trägt den Rest); Messung in P1/P2, Alarm bei > 15× im Bulk (R-01).

### 11.2 Physische Entfernbarkeit (INV-9, normativ)
Eine Zeile ist **physisch entfernbar**, wenn **alle** gelten: (a) Death-Eintrag existiert (oder `aborted`), (b) `died_seq < retention_horizon(ns)`, (c) kein gepinnter Snapshot, **kein Branch-Fork-Punkt `fork_seq`** und kein Tag/Backup-Lease sie noch sieht (`min_pinned_seq ≤ died_seq` verhindert Entfernung). `retention_horizon(ns)` = die `Seq` des letzten Commits vor `now − history_days`. Es gibt **keinen Nachweis** der Entfernung (Nicht-Ziel).

### 11.3 Retire, Retention, Purge (Mechanik)
- **Retire:** Attr-Zeile `TOMBSTONE` + Death für die Content-Zeile (SPEC-06 §7). Sichtbarkeit endet am Commit-Snapshot.
- **Retention** je Namespace (`history_days`, `branches_days`, optional `history_versions`): Compaction entfernt Zeilen nach 11.2.
- **Purge** (Admin, best effort): Tombstone aller passenden Cells + **Retention-Override** (`horizon = jetzt`, soweit keine Pins/Tags/Branches blockieren; blockierende werden gemeldet) + Compaction-Aufträge für betroffene Segmente. Derivate und Blobs werden per Referenzzählung (`content_hash`-Verwendung in lebenden oder retention-pflichtigen Zeilen) beim Compaction-/GC-Lauf entfernt. **Keine Garantie, kein Beleg.**

---

## 12. Blob-Store und Garbage Collection

- **Blob:** `blobs/<aa>/<hash-hex>`; Datei = Dateikopf + Blöcke (zstd, optional AEAD). Schreiben per `tmp/`-Rename. Verwendung: `RawPayload::BlobRef`, Quelldokumente (`keep_source`), Artefakte > `max_cell_size`.
- **GC-Wurzeln:** Branch-Refs, Tags, aktive Snapshot-Pins (Epochen-Register), Fork-Punkte, Retention-Fenster, Backup-Leases, aktive Sessions.
- **Mark/Sweep über das Manifest** (Erreichbarkeit = Manifest-Referenz; Dateien haben keinen Zeigergraphen): (1) Kernel-Snapshot des Manifests; (2) Mark aller referenzierten Segmente/Lanes/Indizes; (3) **Write-Barrier:** während des Marks entstandene Dateien gelten als jung und sind ausgenommen; (4) Remark: Ref-/Sessionänderungen seit Mark-Start; (5) Sweep: nicht markierte Dateien älter als **Karenzzeit** (Default 1 h) löschen.
- **Blob-Erreichbarkeit:** Referenzzählung über `content_hash`/`BlobRef`-Verwendung in lebenden und retention-pflichtigen Zeilen; Sweep nur im Compaction-Lauf.
- **Pin-Lease:** Snapshots tragen ein Lease (Default 300 s, verlängerbar; ≤ 4 096 aktive). Abgelaufene Leases werden freigegeben; Abfragen erhalten `E_SNAPSHOT_EXPIRED` (retryable).

---

## 13. Block-Cache und I/O

- **Block-Cache:** reiner Lese-Cache über **unveränderliche** Daten, seitenausgerichtete Rahmen (4/16 KiB), Pin-Zähler, Eviction **W-TinyLFU je Klasse**, Rahmen-Flag `validated` (rkyv/bytecheck erledigt).
- **Klassen und Default-Anteile** des `block_cache`-Budgets (40 % des `MemoryBudget`): `hot_head` (Content-Hot-Heads, Attr-Blöcke) 20 % · `postings` 20 % · `graph_sectors` (ANN-Nachbarsektoren, CSR-Blöcke) 25 % · `vec_blocks` (F16-Rerank) 15 % · `text` (Cold-`raw`) 15 % · `meta` (Footer, Bitmaps, Kataloge) 5 %. Ein großer Scan darf eine andere Klasse nicht verdrängen.
- **I/O-Backend (`IoBackend`-Trait):** Standard `pread`/`pwrite` auf einem Blocking-Pool mit dedizierten Storage-Threads; Linux optional `io_uring` (registrierte Puffer, `O_DIRECT` für Vektor-/Sektor-Reads, SQPOLL optional) hinter Feature `io-uring` und Profil; Aktivierung nur nach Benchmark-Nachweis (P2). **Kein `mmap` im Produktionspfad** (K-26).
- **Prioritäten:** Vordergrund-Query > Seal/Flush > Compaction/Index-Builds > Scrub. Prefetch für Beam-Search-Sektoren und Postings-Blöcke.
- **Blockierende Arbeit** läuft nie auf dem Async-Executor (SPEC-09 §2).

---

## 14. Verschlüsselung at Rest (optional, store-weit) (K-25)

- **Schlüsselhierarchie:** `KEK` (Passphrase via Argon2id, Schlüsseldatei oder KMS über Port `KeyProvider`) wickelt **DEK-Epochen** (`keyring.ckr`). Genau eine Epoche ist aktiv; jede Datei trägt `key_epoch`.
- **Dateischlüssel:** `file_key = HKDF(DEK_epoch, file_id)`. **AEAD:** XChaCha20-Poly1305 (Default) oder AES-256-GCM-SIV. Nonce (24 B) = `file_id ‖ block_no` (Segmente, Indizes, Blobs) bzw. `file_id ‖ lsn` (WAL); WAL-Recycling erzeugt neue `file_id`.
- **KEK-Rotation:** Rewrap der Epochen (O(#Epochen)). **DEK-Rotation:** neue Epoche; neue Dateien nutzen sie, Compaction migriert; alte Epochen bleiben (gewickelt), bis keine Datei sie referenziert.
- **Hashes:** `content_hash` und alle Digest-Hashes sind **keyed mit `hash_key` aus `repo_seed`** und unabhängig von KEK/DEK (K-13). Schlüsselrotation ändert nie Hashes.
- **Nicht enthalten:** Schlüssel je Person/Datensatz, Schlüsselvernichtung als Löschmechanismus, Widerrufsprotokolle.

---

## 15. Integrität, Scrub, Verify

- **CRC32C** bei jedem Block-Read (Hot Path). **BLAKE3** über das Gesamtobjekt im Footer.
- **`scrub`** (Hintergrund/Befehl, niedrigste I/O-Priorität): liest alle Segmente, prüft CRC, Footer-BLAKE3 und Manifest-Verkettung. Defekte **Index**-Segmente werden automatisch neu gebaut; defekte **Lane-Patches** neu berechnet oder ersetzt; defekte **Truth**-Segmente ⇒ `E_CORRUPT_TRUTH` mit Hinweis auf Restore (kein stilles Reparieren).
- **`verify`:** Konsistenz Manifest ↔ Dateien ↔ Katalog ↔ `idmap` ↔ `parent`/`seq`; `verify --deep` **berechnet den Truth-Digest** (LtHash über alle sichtbaren Zeilen des Branch-Heads) und vergleicht ihn mit dem gespeicherten `truth_hash` (INV-1). Damit sind Restore, Compaction und Replikation prüfbar.

---

## 16. Backup, Restore, Replikationsfähigkeit

- **Backup = Dateimenge zu einem konsistenten Manifest-Stand** (Segmente immutable): inkrementell per Dateiliste (neue Segmente seit letztem Backup) + WAL-Tail. **Point-in-Time** = beliebiger `Seq` innerhalb der Retention (Manifest-Stand + WAL bis `Seq`).
- **Online-Snapshot** `snapshot_export(seq)`: pinnt das Manifest (Backup-Lease), kopiert/verlinkt referenzierte Dateien (Hardlink, wenn gleiches Volume; sonst Kopie; bei `link_only` mit Fremdvolume Fehler `E_CROSS_DEVICE`).
- **Restore:** Dateien an neuen Ort, `verify --deep`, Öffnen; Indizes fehlen ⇒ Flat-Fallback + Rebuild. Abnahmetest SPEC-09 §6.
- **Replikation (nach v1):** Log-Shipping des WAL + Dateisync der immutablen Segmente; Follower bedienen Zeitreise-Lesen. Das Format erzwingt dafür nichts Zusätzliches (Single-Writer-Log, immutable Dateien, deterministische Gid-Vergabe, `truth_hash_after` je Commit als Prüfwert).

---

## 17. Sidecars

| Datei | Inhalt | Persistenz |
|---|---|---|
| `idempotency.cxs` | `idem_key → (seq, payload_hash, result_summary, expires_at)`; Default-TTL 24 h | WAL (`TxnCommit`/`SidecarOp`), periodischer Snapshot |
| `work_queue.cxs` | extern leasbare Aufgaben (SPEC-06 §4) | WAL (`SidecarOp`), Snapshot |
| `jobs.cxs` | interne Wartungsjobs (`BuildIndex`, `Compact`, `Scrub`, `Gc`, `Seal`) mit Zustand/Backoff | WAL, Snapshot |
| `access_stats.cxs` | Zugriffsstatistik (SPEC-01 §14) | nur periodischer Snapshot (eventual consistent) |

- Snapshot-Dateien werden per `tmp/`-Rename atomar ersetzt; `sidecar_checkpoint_lsn` im Manifest begrenzt den Replay. Sidecars sind **nicht im Truth-Digest** und erzeugen **keine Commits** (kein Commit-Spam durch Leases). Ergebnisse von Aufgaben werden als reguläre Commits eingespielt.

---

## 18. Fehlerbehandlung im Storage

| Fehler | Reaktion |
|---|---|
| `fsync`/`fdatasync` schlägt fehl (WAL, Manifest, Segment) | Store → `Failed` (INV-D3); alle weiteren Schreib-API liefern `E_STORE_FAILED`; Lesen weiter möglich, soweit sicher; Neustart + Recovery |
| `ENOSPC` beim WAL-Append | Commit scheitert mit `E_STORE_FULL` (retryable nach Freigabe), Store bleibt `ReadOnly`, kein Teilzustand sichtbar |
| CRC-Fehler Truth-Block | Abfrage liefert `E_CORRUPT_TRUTH` für betroffene Zeilen, Alarm, `scrub`-Auftrag |
| CRC-Fehler Index-/Lane-Block | Segment verwerfen (Index ⇒ Flat-Fallback), Rebuild |
| Manifest-Torn-Tail | abschneiden (Recovery) |
| Unbekannte `incompat_flags` | `E_FORMAT_INCOMPATIBLE`, nichts wird verändert |
| Nicht-fatale I/O-Fehler in Hintergrundjobs | Job-Retry mit Backoff; nach N Fehlern Alarm, nie Datenänderung |
