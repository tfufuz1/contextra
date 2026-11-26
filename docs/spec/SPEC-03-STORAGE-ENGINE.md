# SPEC-03 — Storage Engine: Verzeichnis, Packs, Extents, Locator, Verschlüsselung

**Version 3.0 · 26. November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Datenmodell](SPEC-02-DATA-MODEL.md) · [Versionierung](SPEC-04-VERSIONING.md) · [Durability](SPEC-07-DURABILITY.md)

Alle Ganzzahlen in Dateiformaten sind little-endian. Offsets sind relativ zum Beginn der jeweiligen Struktur, sofern nicht anders angegeben. Reservierte Felder MÜSSEN 0 geschrieben und beim Lesen auf 0 geprüft werden.

---

## 1. Verzeichnislayout

```
<store>/
  contextra.toml              # Startkonfiguration (nicht Teil der Wahrheit)
  LOCK                        # flock / LockFileEx
  CURRENT                     # Superblock, 2 × 4 KiB Slots
  KEYS                        # gewrappte Schlüssel, 2 Slots (nur verschlüsselt)
  manifest/MANIFEST-<n>.mlog
  wal/WAL-<n>.cxw             # 64 MiB, vorallokiert
  packs/truth/PACK-<id>.cxp   # Wahrheit, ChangeSets, Derivationen
  packs/index/PACK-<id>.cxp   # ausschließlich abgeleitete Segmente
  locator/LOC-{OBJ,CELL,CONT,DRV}-<n>.cxl
  sidecar/work_queue.cxs  idempotency.cxs  access_stats.cxs
  tmp/
```

Die Trennung `packs/truth` und `packs/index` ist normativ: Das Löschen von `packs/index/` MUSS ohne Datenverlust überstehbar sein. zstd-Wörterbücher sind Objekte in `packs/truth`.

---

## 2. Objekte

```rust
#[repr(u16)]
pub enum ChunkKind {
    ProllyLeaf = 0, ProllyInternal = 1, ExtentBlock = 2, LargeBlob = 3,
    VecSegment = 4, FtsSegment = 5, GraphSegment = 6, BitmapSegment = 7,
    IndexManifest = 8, CommitHeader = 9, Dictionary = 10,
    ChangeSet = 11, Derivation = 12, CellContent = 13, CellMeta = 14,
    ProvRecord = 15, AclExpr = 16,
}
```

`CellContent` und `CellMeta` sind **logische** Objekte: Ihre Hashes werden über CBE gebildet, physisch liegen ihre Felder spaltenweise in Extents. `ChunkId`-Bildung: [SPEC-02](SPEC-02-DATA-MODEL.md) §2.2.

---

## 3. Pack-Format (`.cxp`)

```
┌ Pack Header (64 B) ┐┌ Extents (4-KiB-aligned) ┐┌ Standalone-Objekte (8-B-aligned) ┐
┌ Footer: ExtentDir · ObjectIdx · CellIdx · ContentIdx · DerivationIdx · Filter · Trailer ┐
```

Packs werden in `tmp/` geschrieben, mit `sync_data` gesichert, umbenannt, das Zielverzeichnis synchronisiert und **erst dann** im Manifest registriert (D-1, [SPEC-07](SPEC-07-DURABILITY.md) §1). Danach sind sie unveränderlich. Zielgröße 256 MiB (64 MiB – 1 GiB; kleine Folds dürfen kleiner sein und werden kompaktiert).

### 3.1 Pack Header (64 B)

| Off | Len | Feld |
|---|---|---|
| 0 | 4 | `magic = "CXP\x01"` |
| 4 | 2 | `format_version = 1` |
| 6 | 2 | `compat_flags` |
| 8 | 4 | `incompat_flags` (Bit 0 encrypted, Bit 1 compressed) |
| 12 | 8 | `pack_id` |
| 20 | 16 | `store_uuid` |
| 36 | 16 | `pack_salt` (CSPRNG je Datei) |
| 52 | 4 | `dek_id` (0 = unverschlüsselt) |
| 56 | 4 | `pack_class` (1 truth, 2 index) |
| 60 | 4 | `header_crc32c` (Bytes 0–59) |

Der Header enthält keine erst am Ende bekannten Werte und wird nie nachträglich überschrieben. `repo_seed` steht **nicht** in Packs, sondern im Superblock bzw. gewrappt in `KEYS`.

### 3.2 Extent-Layout

Ein Extent ist die I/O-Einheit (Ziel 512 KiB, 256 KiB – 1 MiB, Länge Vielfaches von 4 KiB). Er enthält 1 bis 65 535 Zeilen **eines Namespace** (ein Wörterbuch je Extent).

**Extent Header (128 B)**

| Off | Len | Feld |
|---|---|---|
| 0 | 4 | `magic = "CXE1"` |
| 4 | 2 | `version` |
| 6 | 2 | `flags` (META_ONLY, DERIVATION_ONLY) |
| 8 | 4 | `extent_no` |
| 12 | 4 | `extent_len` |
| 16 | 4 | `row_count` |
| 20 | 4 | `ns_id` |
| 24 | 4 | `hot_end` |
| 28 | 4 | `cold_start` |
| 32 | 4 | `lane_dir_offset` |
| 36 | 2 | `lane_count` |
| 38 | 1 | `codec_default` |
| 39 | 1 | `encryption` |
| 40 | 32 | `dict_chunk_id` (0 = keines) |
| 72 | 52 | reserviert |
| 124 | 4 | `header_crc32c` (Bytes 0–123) |

**Lane Directory** (`lane_count × 24 B`)

| Off | Len | Feld |
|---|---|---|
| 0 | 2 | `lane_kind` |
| 2 | 2 | `space_slot` (Embedding-Space/Tokenizer-Index, sonst 0) |
| 4 | 4 | `offset` |
| 8 | 4 | `stored_len` |
| 12 | 4 | `raw_len` |
| 16 | 1 | `codec` |
| 17 | 1 | `flags` (HOT, BLOCKED, NULLABLE) |
| 18 | 2 | reserviert |
| 20 | 4 | `crc32c` (über gespeicherte Bytes) |

**Space-Tabelle:** Eine eigene Lane `space.map` bildet `space_slot` auf `SpaceId`/`TokenizerId` (je 32 B Digest) ab.

**Spaltenkodierung:** Fixbreite Lanes sind Arrays. Variable Lanes beginnen mit `u32 offsets[N+1]`, dann Daten; NULL-fähige Lanes tragen zusätzlich eine Bitmaske. Komprimierte variable Lanes sind in **unabhängige Blöcke** (Ziel 16 KiB Klartext) mit Blockverzeichnis `(first_row, stored_off, stored_len, raw_len, crc32c[, tag])` geteilt, sodass ein Gewinner-Read nur die betroffenen Blöcke dekodiert.

```
┌ Header 128 ┐┌ Lane Directory ┐┌ HOT HEAD … hot_end ┐┌ pad ┐┌ COLD TAIL cold_start … ┐┌ crc32c ┐
HOT:  gid[N] (aufsteigend) · record · meta.fixed · meta.var · attr.bloom · edge.hint
      · belief.cache · tokcount · zoom.L2 · zoom.L1 · emb.q[*]
COLD: mime · raw · tok.ids · emb.full[*] (4-KiB-aligned) · attrs.ext
```

`gid` ist die erste Lane und aufsteigend sortiert; bei Gleichstand (mehrere Versionen) nach `born_seq`. Ziel `hot_end ≤ 64 KiB`; wird es überschritten, verkleinert der Writer die Zeilenzahl, `zoom.L1` darf als letzte Hot-Lane in den zweiten Read fallen.

**Integrität:** Jede Lane bzw. jeder Block hat eine eigene CRC32c, damit Teil-Reads prüfbar sind. Die Gesamt-CRC am Ende dient `scrub`. CRC erkennt Medienfehler; Authentizität liefert nur AEAD bzw. die BLAKE3-Prüfung der logischen Objekte.

### 3.3 Meta- und Derivations-Extents

- **`META_ONLY`:** Bei reinen Metadatenänderungen schreibt der Fold alle Hot-Lanes, aber keinen `raw`/`emb.full`. Der Cold-Read löst über `ContentRowLocator` auf.
- **`DERIVATION_ONLY`:** Spät eintreffende Embeddings oder Summaries werden nie in bestehende Extents geschrieben, sondern in eigene Extents; Compaction ko-lokalisiert sie später. Hot-Head-Vollständigkeit ist daher eine **Optimierung, keine Zusicherung**.

### 3.4 Footer

| Abschnitt | Eintrag | Größe |
|---|---|---|
| ExtentDir | `offset u64 · len u32 · rows u32 · min_gid u64 · max_gid u64` | 32 B |
| ObjectIdx (nach ID sortiert) | `chunk_id[32] · offset u64 · len u32 · kind u16 · flags u16` | 48 B |
| CellIdx | `cell_ver_key[32] · extent_no u32 · row u16 · flags u16` | 40 B |
| ContentIdx | `content_hash[32] · extent_no u32 · row u16 · flags u16` | 40 B |
| DerivationIdx | `derivation_key_hash[32] · extent_no u32 · row u16 · lane u16 · space_slot u16 · flags u16 · reserved u32` | 48 B |
| Filter | Ribbon-Filter über alle Schlüssel (Ziel-FPR 0,1 %) | variabel |

**Trailer (112 B, letzte Bytes der Datei)**

| Off | Len | Feld |
|---|---|---|
| 0 | 4 | `magic = "CXPF"` |
| 4 | 2 | `version` |
| 6 | 2 | `flags` |
| 8 | 48 | `section_off[6]` (u64, Reihenfolge wie oben) |
| 56 | 4 | `footer_len` (inkl. Trailer) |
| 60 | 4 | `extent_count` |
| 64 | 4 | `object_count` |
| 68 | 4 | `cell_count` |
| 72 | 32 | `footer_blake3` (alle Abschnitte vor dem Trailer) |
| 104 | 4 | reserviert |
| 108 | 4 | `trailer_crc32c` (Bytes 0–107) |

Anzahlen der Content- und Derivationseinträge folgen aus Abschnittsgrenzen. Layout-Konstanten werden per `const _: () = assert!(size_of::<T>() == N);` abgesichert.

---

## 4. Locators

| Locator | Schlüssel → Wert |
|---|---|
| `ObjectLocator` | `ChunkId → (pack_id, offset, len)` |
| `CellLocator` | `cell_ver_key = H(content_hash ‖ meta_hash) → (pack_id, extent_no, row)` |
| `ContentRowLocator` | `content_hash → ExtentRowAddr \| BlobSpan` |
| `DerivationLocator` | `H(DerivationKey) → (pack_id, extent_no, row, lane, slot)` |

Der `CellLocator` ist **versionsadressiert**, nicht `Gid`-adressiert: Verschiedene Branches und Zeitstände sehen verschiedene Versionen derselben `Gid`.

- Aufbau: größengestufte, sortierte Runs mit Ribbon-Filter; die zwei jüngsten Runs und ein Sparse-Index aller Runs liegen im RAM. Compaction hält höchstens `max_runs = 8`.
- Mehrfacheinträge (nach Copy-Forward): Der jüngste durch das Manifest registrierte Eintrag gewinnt.
- **Hints:** Prolly-Leser dürfen flüchtige `(pack_id, …)`-Hints cachen. Sie gehen nie in Hashes ein; ein Fehlgriff fällt auf den Locator zurück. Jeder Treffer wird gegen die erwartete ID geprüft.
- Rekonstruktion: vollständig aus Pack-Footern, Kosten proportional zur Summe der Footer-Größen. Dauer wird gemessen, nicht zugesichert.

---

## 5. Lesepfad und Buffer Pool

1. **Ranking-Read:** Hot Head eines Extents (ein Read bis `hot_end`).
2. **Winner-Read:** betroffene Cold-Blöcke über `RowIndex`; typisch ≤ 12 Cells je Recall [Annahme].

### 5.1 Buffer Pool

Reiner Lese-Cache über unveränderliche Daten, keine Dirty Pages. Rahmenklassen 4 KiB und 16 KiB, größere Spannen als verkettete Rahmen. Eviction W-TinyLFU. Pins sind RAII-Guards; ein Pool ohne freien Rahmen liefert Backpressure, nie unbegrenzte Allokation (INV-10).

| Partition | Anteil am `block_cache` (Default) |
|---|---|
| Prolly intern (bevorzugt resident) | 10 % |
| Prolly Blätter | 15 % |
| Extent Hot Heads | 30 % |
| Vektorsektoren | 20 % |
| FTS-Postings / Graph | 15 % |
| Cold Tails (Scan-resistent) | 10 % |

Validierung (CRC, AEAD-Tag, `bytecheck`) erfolgt genau einmal beim Laden in einen Rahmen.

### 5.2 I/O-Backend

```rust
pub trait IoBackend: Send + Sync {
    fn submit_reads(&self, reqs: &mut [ReadReq<'_>]) -> Result<()>;
    fn alignment(&self) -> usize;   // 1 (buffered) oder 4096 (O_DIRECT)
}
```
Baseline `pread`-Threadpool; Feature `io-uring` mit registrierten, ausgerichteten Puffern. `O_DIRECT` nur für unveränderliche Reads und nie gemischt mit gepufferten Writes derselben Datei im offenen Zustand. Kein `mmap`.

---

## 6. Kompression

| Daten | Verfahren | Level |
|---|---|---|
| `raw`, `zoom.*` | zstd mit Namespace-Wörterbuch (≤ 112 KiB) | 3 online, 19 Compaction |
| `tok.ids` | Delta + Bitpacking (`ceil(log2 vocab)`) + zstd | 3 |
| `meta.*`, Attribute | Dictionary / RLE / Frame-of-Reference | — |
| `emb.q` | SQ8 / binär / RaBitQ, unkomprimiert | — |
| `emb.full` | `f16` (Default) oder `f32`, unkomprimiert | — |

Wörterbuch-Training nach je 10 000 neuen Cells; alte Wörterbücher bleiben erreichbar, solange ein lebender Extent sie referenziert (GC-Kante). Dekompression MUSS `raw_len` als harte Obergrenze durchsetzen.

---

## 7. Compaction und Affinity Re-Layout

**Compaction** wird ausgelöst bei Live-Anteil < 50 % je Pack, zu vielen kleinen Packs oder Wörterbuchwechsel. Protokoll: neue Packs schreiben und synchronisieren → Locator-Runs schreiben → Manifest-Eintrag `{AddPack…, AddLocatorRuns, Condemn(old)}` → nach Epochen-Ablauf `RemovePack` und Unlink.

**Re-Layout** (opt-in, Aktivierung nur nach nachgewiesener I/O-Reduktion ≥ 20 % auf repräsentativem Trace):

| Signal | Gewicht |
|---|---|
| Co-Retrieval, Co-Expand | 1,0 |
| Gleiches Dokument / gleiche Episode | 0,8 |
| Semantische Nähe | 0,5 |
| Zeitliche Nähe | 0,3 |

Gewichteter Co-Access-Graph → deterministische, geseedete Label-Propagation mit Gruppenkapazität → neue Extents → ausschließlich Locator-Umbiegung. `ChunkId`s, `content_hash`, `meta_hash`, `RootSet` und `CommitId` bleiben unverändert. 5 % Exploration. Gruppierung überschreitet nie Namespace-Grenzen.

---

## 8. Verschlüsselung at rest (optional)

### 8.1 Schlüsselhierarchie

```
KEK            Passphrase (Argon2id) oder KMS über CipherProvider — nie auf Disk
 ├ wrap → id_key      (256 bit, beim Anlegen erzeugt, NIE rotiert)   → ChunkId
 ├ wrap → repo_seed   (256 bit)                                      → Prolly-Grenzen, Handle-MAC
 └ wrap → DEK[dek_id] (256 bit, rotierbar, mehrere gleichzeitig)
file_key = HKDF-SHA256(ikm = DEK, salt = file_salt, info = "cx/" ‖ file_kind ‖ store_uuid ‖ file_id)
```

`id_key` und `repo_seed` sind bewusst von der DEK-Rotation entkoppelt: Andernfalls änderten sich bei Rotation alle Objekt-IDs und Baumstrukturen. Eine Kompromittierung von `id_key` oder `repo_seed` ist nur durch vollständigen Export/Neuaufbau behebbar; dies ist dokumentierte Betriebsgrenze.

### 8.2 AEAD und Nonces (INV-14)

Algorithmen: AES-256-GCM (mit Hardwareunterstützung) oder XChaCha20-Poly1305.

| Dateiart | Schlüsselbindung | Nonce | AAD |
|---|---|---|---|
| Pack | `pack_salt` je Datei | `(extent_no u32, lane u16, space u16, block u32)` bzw. `(0xFFFF_FFFF, section, 0, index)` | `store_uuid ‖ pack_id ‖ Position ‖ raw_len` |
| WAL | `segment_salt` je Segmentdatei | Zähler = Byte-Offset des Frames | Frame-Header |
| Manifest, Locator, Sidecar-Snapshot | `file_salt` je Datei | Record-Zähler | Record-Header |

Regeln:
1. Jede Datei erhält ein **frisches Zufalls-Salt**; ein nach Absturz neu erzeugtes Pack mit gleicher `pack_id` hat damit einen anderen Schlüssel.
2. In eine nach Absturz geöffnete WAL-, Manifest- oder Sidecar-Datei wird **nie weitergeschrieben**; Recovery beginnt immer eine neue Datei mit neuem Salt.
3. AES-GCM: je Dateischlüssel höchstens 2³² Nachrichten; das wird durch die Dateigrößen strukturell eingehalten und per Assertion geprüft.
4. WAL-Segmente werden nicht in-place recycelt, wenn verschlüsselt; Wiederverwendung nur mit neu geschriebenem Header, neuem Salt und neuer `segment_epoch`.

Verschlüsselt werden Extent-Lanes, Standalone-Objekte, Footer-Abschnitte, WAL-Frame-Rümpfe, Manifest-Records, Locator-Runs, Sidecars. Klartext bleiben nur Dateiheader und Längenfelder.

### 8.3 Rotation

- **KEK:** nur `KEYS` neu wrappen (zwei Slots, Generation + CRC): O(1).
- **DEK:** neuer `dek_id` für neue Dateien; Compaction schreibt alte um; alter DEK wird erst entfernt, wenn keine Datei ihn referenziert.
- Aktivieren der Verschlüsselung für einen bestehenden Klartext-Store ändert alle IDs und erfolgt nur per `ctx snapshot-export` in einen neuen Store.

---

## 9. Fehlerverhalten

| Befund | Reaktion |
|---|---|
| CRC-/Tag-/Hashfehler in Index-Pack | Segment aus Sicht entfernen, Flat-Fallback, Rebuild |
| Fehler in Truth-Pack, andere Kopie bekannt | Locator auf Kopie, Quarantäne der defekten Datei, Alarm |
| Fehler in Truth-Pack ohne Kopie | `pack_corrupted` für betroffene Lesezugriffe; Restore nötig; GC pausiert |
| Fehler in Derivations-Lane | Eintrag verwerfen, neu berechnen |
| `ENOSPC` beim Pack-Schreiben | Temporärdatei entfernen, Fold aussetzen, Backpressure; Wahrheit bleibt im WAL |

---

## 10. Abnahme

- Golden-Files je Format; ≥ 24 h Fuzzing für Pack, Extent, Footer, Locator.
- Test: Löschen von `packs/index/` ⇒ Rebuild; Löschen von `locator/` ⇒ Rekonstruktion mit identischen Antworten.
- Re-Layout ändert nachweislich keine ID.
- Crash-Injektion beim Pack-Schreiben erzeugt nie eine registrierte, unvollständige Datei und nie wiederverwendete Nonces.

---

*Weiter: [SPEC-04 Versionierung](SPEC-04-VERSIONING.md)*
