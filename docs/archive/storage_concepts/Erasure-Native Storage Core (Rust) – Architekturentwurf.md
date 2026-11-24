# Erasure-Native Storage Core (Rust) – Architekturentwurf

Nov 23, 2026 · @kiki

## 0. Leitidee und Garantiestufen

**Lethe** (Arbeitsname des Kerns) löscht, indem Löschdomäne, Schlüsseldomäne und Platzierungseinheit identisch sind: der `Cluster`. Wer den Cluster-Schlüssel vernichtet, macht jeden Byte des Clusters (Nutzdaten, Header, Gewichte, Block-Index, Filter, Bereichsmarker) sofort unlesbar. Der physische Purge folgt zeitgebunden und berührt nur Bytes dieses Clusters.

Der Kern kennt weder Personen noch Rechtsgrundlagen. Er liefert vier Primitive: `erase`, `restrict`, `link` und ein manipulationsgeschütztes Ledger mit Rollback-Anker. Alles Weitere (Person → ClusterId, Fristen, Rechtsgrundlagen) liegt oben.

### Garantiestufen

| Stufe | Mechanismus | Wirkt ab | Was garantiert wird | Benannte Restexposition |
| --- | --- | --- | --- | --- |
| G0 Logisch | `EraseGate`-Record im WAL (fsync) + atomarer Publish des `ErasedSet` | Gate-Commit (Stamp `S_e`) | Kein `View`, kein Commit, kein Change-Feed liefert noch Inhalte des Clusters. Pro Cluster gilt: voller Snapshot oder `Erased`, nie eine Mischsicht. | Bytes liegen noch (verschlüsselt) auf Platte; Klartext ggf. noch im RAM |
| G1 Kryptografisch | `KeyProvider::destroy` aller Schlüsselepochen, Receipt im Ledger, Anker-Fortschritt | Receipt-Stärke `Destroyed` | Ciphertext ist ohne Schlüssel nicht lesbar, sofern kein weiterer Schlüsselträger existiert (Escrow, KMS-Backup) | KMS-Karenzzeit (`Scheduled` statt `Destroyed`), Schlüsselkopien im Prozessspeicher bis `zeroize` |
| G2 Physisch | In-place-Scrub der Cluster-Sektionen, Memtable-Drop, WAL-Segment-Ablöse, Katalog-Generationen löschen, Extensions bereinigen | Ticket `Completed` | Kein Byte von Cluster-Klartext, Ciphertext oder ClusterId mehr in den Dateien des Stores (Löschorakel) | SSD-Wear-Leveling, COW-/Snapshot-Dateisysteme, Page-Cache, Swap, externe Backups |

Ticket-Stufen sind einzeln abfragbar (`Gated`, `KeysDestroyed`, `Purged`, `Completed`). Der Kern behauptet nie mehr, als die jeweils erreichte Stufe trägt. Insbesondere gilt: G2 ist eine Aussage über die Dateien des Stores, nicht über das Medium. Das Medium schützt allein G1.

### Konstruktionsregeln, aus denen alles Weitere folgt

1. **Cluster-Reinheit:** Kein Frame, keine Sektion, kein Blob-Chunk enthält Daten zweier Cluster.
2. **Positionsunabhängigkeit:** Die AEAD-AAD enthält keine Datei-ID und keinen absoluten Offset. Überlebende Sektionen lassen sich roh kopieren oder an Ort und Stelle liegen lassen.
3. **Gate vor Schlüssel vor Purge:** Die Reihenfolge ist durch Invarianten erzwungen, damit jeder Crash-Zustand fortsetzbar ist.
4. **Löschkosten ∝ Clustergröße:** Kein Vollscan, keine Entschlüsselung fremder Daten. Ein Test mit Byte-Zählern in `FaultFs` erzwingt das.

## 1. System- und Crate-Architektur

Der Kern ist ein Cargo-Workspace aus reinen Logik-Crates, die nur über Port-Traits mit der Welt sprechen. Alle eigenen Crates tragen `#![forbid(unsafe_code)]`; Adapter-Crates dürfen Dependencies mit `unsafe` nutzen (z. B. RustCrypto), enthalten aber selbst keines. Alles ist synchron: keine Async-Runtime, Hintergrundarbeit läuft auf wenigen `std::thread`-Workern (Flush, Compaction, Purge, Erase-Driver).

### Crates und Abhängigkeitsregeln

```text
lethe-types      IDs (ClusterId, SpaceId, Stamp, Lsn, TombId), Fehler, Konstanten   [keine Deps]
   ^
lethe-ports      Traits: Fs, KeyProvider, Anchor, Clock, Entropy, Cipher, Mac, Kdf
   ^
lethe-format     reine Codecs: SealFrame, WAL-Record, Run, Blob, Catalog, Ledger     [kein I/O]
lethe-seal       Schlüsselplan, AAD-Bau, Nonce-Strategie auf Basis der Ports
   ^
lethe-core       Engine: Memtable, WAL, Flush, Compaction, Erase, Recovery, View/Page, Guards
lethe-ext        Vertraege: DerivedSegment, ChangeFeed, LinkTable-API
   ^
lethe            Fassade (oeffentliche API), re-exportiert core + ext

lethe-adapter-std     StdFs, OsEntropy, SystemClock          (nur hier std::fs)
lethe-adapter-crypto  XChaCha20-Poly1305, HKDF-SHA256, HMAC-SHA256 (RustCrypto)
lethe-adapter-keys    LocalKeyring (Datei), KMS-Adapter (verzoegerte Loeschung)
lethe-adapter-anchor  FileAnchor (Test), TPM-/Transparency-Log-/Object-Lock-Anker
lethe-sim             MemFs, FaultFs, SimClock, DetEntropy, Loeschorakel   [nur dev]
lethe-testkit         Modell-Referenz, Workload-Generatoren, Referenz-Extension
```

Regeln, per CI durchgesetzt:

- Pfeile gehen nur nach oben; `lethe-core` hängt von `lethe-ports`, `-format`, `-seal` ab, nie von einem Adapter.
- `std::fs`, `std::time::SystemTime` und `rand` sind in allen Crates außer den Adaptern per `clippy::disallowed_methods` verboten.
- `lethe-format` ist pur (kein I/O, keine Uhr): `miri`-tauglich und Fuzz-Ziel.
- Kein `mmap`: Lesezugriffe laufen über `Fs::read_at`, damit `FaultFs` jeden Zugriff sieht.
- Keine Payload-Bytes in Logs, Metriken oder Fehlertexten; nur ClusterId, Zähler, Stamps.

### Port-Traits (Skizze)

```rust
pub trait Fs: Send + Sync + 'static {
    type File: FsFile;
    fn open(&self, path: &Path, opts: OpenOpts) -> io::Result<Self::File>;
    fn rename_atomic(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn sync_dir(&self, dir: &Path) -> io::Result<()>;
    fn remove(&self, path: &Path) -> io::Result<()>;
    fn list(&self, dir: &Path) -> io::Result<Vec<DirEntry>>;
    fn free_space(&self) -> io::Result<u64>;
    fn lock_exclusive(&self, path: &Path) -> io::Result<LockGuard>;
}
pub trait FsFile: Send + Sync {
    fn read_at(&self, off: u64, buf: &mut [u8]) -> io::Result<()>;
    fn write_at(&self, off: u64, buf: &[u8]) -> io::Result<()>;   // Scrub, Preallocation
    fn append(&self, buf: &[u8]) -> io::Result<u64>;
    fn sync_data(&self) -> io::Result<()>;
    fn len(&self) -> io::Result<u64>;
    fn set_len(&self, len: u64) -> io::Result<()>;
}

pub trait KeyProvider: Send + Sync {
    fn create(&self, c: ClusterId) -> Result<EpochId>;                       // Epoche 0
    fn rotate(&self, c: ClusterId) -> Result<EpochId>;                       // neue Epoche fuer neue Writes
    fn unwrap(&self, c: ClusterId, e: EpochId) -> Result<Zeroizing<[u8; 32]>>;
    fn destroy(&self, c: ClusterId, scope: EpochScope) -> Result<DestroyReceipt>;  // idempotent
    fn status(&self, r: &DestroyReceipt) -> Result<DestroyState>;            // Scheduled{until} | Destroyed
}
pub trait Anchor: Send + Sync {
    fn read(&self) -> Result<AnchorState>;                                   // {counter, ledger_head}
    fn advance(&self, expect: AnchorState, next: AnchorState) -> Result<()>; // Compare-and-Swap, monoton
}
pub trait Clock: Send + Sync { fn now_wall(&self) -> WallTime; fn now_mono(&self) -> Instant; }
pub trait Entropy: Send + Sync { fn fill(&self, buf: &mut [u8]); }
pub trait Cipher: Send + Sync {                                              // AEAD, Nonce 24 B, Tag 16 B
    fn seal(&self, key: &[u8; 32], nonce: &[u8; 24], aad: &[u8], pt: &[u8], out: &mut Vec<u8>);
    fn open(&self, key: &[u8; 32], nonce: &[u8; 24], aad: &[u8], ct: &[u8], out: &mut Vec<u8>) -> Result<(), AuthError>;
}
pub trait Kdf: Send + Sync { fn expand(&self, ikm: &[u8; 32], info: &[u8], out: &mut [u8]); }
pub trait Mac: Send + Sync { fn tag(&self, key: &[u8; 32], msg: &[u8]) -> [u8; 32]; }
```

`DestroyReceipt` trägt eine Stärke (`LocalWipe`, `KmsScheduled{until}`, `KmsDestroyed`). Das Ticket meldet `KeysDestroyed` nur, wenn die Stärke die konfigurierte Mindeststufe erreicht. Sonst bleibt es auf `KeysScheduled` und der Kern fragt `status` periodisch nach.

### Öffentliche API (Skizze)

```rust
let store = Store::open(cfg, ports)?;                         // Recovery laeuft hier
let mut tx = store.txn();                                     // Multi-Space-Commit
tx.put(cluster, SPACE_EPISODES, key, Entry { header, body, weight: tokens, lanes })
  .guard(Guard::ClusterUnchangedSince(stamp))
  .idem(IdemToken(uuid));
let stamp = tx.commit()?;                                     // Group-Commit, ein fsync

let view = store.view();                                      // MVCC-Snapshot, kurzlebig (Lease)
let page = view.page(cluster, SPACE_EPISODES, PageReq {
    dir: Dir::Backward, from: Cursor::End, budget: Budget { tokens: 8_000, max_entries: 64 },
    lane_filter: Some(LaneRange { lane: 0, lo: t0, hi: t1 }),   // Bitemporal: nur Index/Header-Frames
    header_pred: None,
})?;                                                           // Bodies werden lazy per page.body(i) gelesen

let ticket = store.erase(cluster, EraseOpts::default())?;     // kehrt nach durablem Gate zurueck
ticket.wait(Stage::Completed, deadline)?;
```

Die `Volatile`-Region (`store.volatile(cluster)`) ist eine reine In-Memory-Map mit denselben Schlüsseln und Löschsemantik, aber ohne WAL und ohne fsync; sie überlebt keinen Neustart und wird beim Gate sofort genullt.

## 2. Datenmodell und On-Disk-Formate

Alle Zahlen sind little-endian. Jede Datei beginnt mit einem 4-KiB-Header (Magic, Format-Version, `store_uuid`, CRC32C). Der Kern interpretiert nur Schlüssel (`Vec<u8>`, lexikografisch), `weight: u32`, `lanes: [i64; 4]` und Sichtbarkeits-Metadaten; `header` und `body` bleiben opake Bytes.

### Logisches Modell

| Begriff | Bedeutung |
| --- | --- |
| `ClusterId` | 128 Bit, kryptografisch zufällig, opak, nie wiederverwendet. Darf nie aus Personendaten abgeleitet werden. |
| `Space` | Benannter Strom pro Cluster (`u16`), z. B. Agentenzustand, Episoden, Chunks. Alle Spaces eines Clusters teilen dessen Schlüssel. |
| `Entry` | `key`, `stamp`, `flags` (Tombstone, Blob-Ref), `weight`, `lanes`, `header`, `body`. |
| `Stamp` | Monotoner `u64` pro Commit; Sichtbarkeitsgrenze für `View`. |
| `Lsn` | Position im WAL; nur intern. |
| `TombId` | `HMAC(ledger_key, ClusterId)`: Das Ledger erkennt gelöschte Cluster, ohne die ClusterId zu speichern. |

### Verzeichnislayout

```text
store/
  LOCK                       exklusiver Prozess-Lock
  CURRENT                    Zeiger auf aktive Katalog-Generation (atomar per rename + sync_dir)
  catalog/   CAT-<gen>.snap, CAT-<gen>.edits
  wal/       <seq>.lwal      16-MiB-Segmente, zzgl. vorallokiertes RESERVE-Segment
  runs/      <id>.lrun       64-256 MiB, unveraenderlich bis auf In-place-Scrub
  blobs/     <id>.lblob      grosse Koerper, ebenfalls cluster-rein
  ledger/    LEDGER.lled     append-only, hashverkettet
  reserve.bin                Notbetriebs-Reserve (z. B. 64 MiB), wird bei DiskFull freigegeben
```

Die Zahl der Dateien wächst mit dem Datenvolumen, nicht mit der Zahl der Cluster: Millionen kleiner Cluster teilen sich Runs als Sektionen.

### Versiegelungsrahmen (`SealFrame`)

```text
+---------+------+-----------+------------+----------+--------------------+----------+
| ver u8  | kind | epoch u32 | local_ord  | nonce 24 | ciphertext (len u32|  tag 16  |
|         | u8   |           | u32        | (random) |  vorangestellt)    |          |
+---------+------+-----------+------------+----------+--------------------+----------+
AAD = "lethe/v1" || store_uuid || cluster_id || epoch || space || kind || local_ord || local_ord2
```

- **Positionsunabhängig:** `local_ord` zählt nur innerhalb der Sektion (Block-Nummer, Chunk-Nummer). Weder Run-ID noch Dateioffset gehen in die AAD ein. `cluster_id` und `store_uuid` stammen aus dem Sektions-Header bzw. dem Store; ein vertauschter Frame scheitert am Tag.
- **Nonce:** 192 Bit zufällig aus `Entropy` (XChaCha20-Poly1305). Kein Zählerzustand, kein Wiederverwendungsrisiko bei Neu-Versiegelung in der Compaction. `DetEntropy` nur in Tests.
- **Teilschlüssel:** `HKDF-Expand(K_cluster[epoch], "lethe/v1/" + kind)` je Frame-Art, damit WAL-, Index-, Header-, Body- und Blob-Frames getrennte Schlüssel nutzen.
- Der Klartext-Rahmen enthält nur Art, Epoche, Ordnungszahl und Länge. Nichts davon ist nutzerbezogen.

### WAL (`.lwal`)

```text
SegHeader(4096): magic "LWAL", ver, store_uuid, seg_no, base_lsn, crc
Record: [len u32][crc32c u32][kind u8][lsn u64][stamp u64] payload
  kind Commit      : n_groups u16, dann je Gruppe: cluster_id[16], epoch u32, SealFrame(ops-Liste dieses Clusters)
  kind EraseGate   : n u16, tomb_id[32] * n, gate_stamp   (Kaskade = ein Record)
  kind CatalogMark : flushed_lsn (Hinweis, Katalog ist massgeblich)
  kind Restrict    : cluster_id, restriction_id u64, mode, until
```

- Die Hülle (`len`, `crc`, `lsn`, `stamp`) liegt im Klartext, damit Recovery den Schwanz ohne Schlüssel findet. Nutzdaten sind ausschließlich in `SealFrame`s.
- Ein Commit über mehrere Spaces und Cluster ist **ein** Record und damit atomar.
- Versiegelt wird vor dem Commit-Lock; der Stamp steht nur in der Hülle, nicht in der AAD.
- Recovery liest bis zum ersten ungültigen Record. Folgt danach noch ein gültiger Record mit passender LSN-Kette, ist es Korruption in der Mitte (harter Fehler), sonst abgerissener Schwanz (wird abgeschnitten).

### Run (`.lrun`)

```text
RunHeader(4096)
Section*  (je 4-KiB-ausgerichtet, genau ein Cluster)
   SecHeader(plain 64 B): magic, cluster_id, epoch, frame_count, sec_len, crc
   MetaFrame  (sealed): Block-Index = je Block {first_key, last_key, idx_off, hdr_off, body_off,
                        entry_count, weight_sum, lane_min[4], lane_max[4]}, Key-Bloom, Range-Marker
   Block*     je Block drei Frames, alle section-relativ adressiert:
      IdxFrame  (sealed): key, stamp, flags, weight, lanes, Offsets pro Entry
      HdrFrame  (sealed): konkatenierte Header
      BodyFrame (sealed): Inline-Koerper (< Schwelle) oder Blob-Refs
Directory (plain): je Sektion {cluster_id, epoch, off, len, dead:u8, crc}
Footer(64): dir_off, dir_len, dir_crc, magic
```

Die **Kopf-/Körper-Trennung** liegt in den drei Frames pro Block. Ein Token-Budget-Paging entschlüsselt nur `MetaFrame` und `IdxFrame`: Gewichte, Lanes und Offsets liegen dort und werden als Block-Aggregat (`weight_sum`, `lane_min/max`) schon im Meta-Index geführt. `HdrFrame` wird nur bei Header-Prädikaten geöffnet, `BodyFrame` nur beim Zugriff auf einen Körper. Lane-Filter (z. B. `valid_from/valid_to` als bitemporale Lanes) prunen ganze Blöcke über `lane_min/max`, ohne Header zu lesen.

Blockgröße: Ziel 16-32 KiB Klartext. Bei Kleinstclustern schrumpft die Sektion auf ein Block-Triplett, der Platzverlust durch 4-KiB-Ausrichtung ist durch den Wert `section_align` (512 B oder 4 KiB) einstellbar.

Das einzige Klartext-Artefakt pro Cluster ist das `Directory` (opake ClusterId, Offsets). Der Per-Run-Cluster-Bloom nutzt einen Schlüssel-SipHash mit Store-Geheimnis, damit er keine ClusterIds preisgibt.

### Blob (`.lblob`)

Wie ein Run, aber die Sektion besteht aus 64-KiB-Chunk-Frames mit `local_ord = chunk_no` und `local_ord2 = blob_local_id`. Körper ab Schwellenwert (Default 16 KiB) liegen hier; der Entry hält `(blob_id, section_ord, blob_local_id, len)`. Streaming-Reads entschlüsseln nur die benötigten Chunks.

### Katalog

Snapshot plus Edit-Log, beides CRC-gesichert, `CURRENT` zeigt per atomarem Rename auf die Generation. Edits: `AddRun`, `RemoveRun`, `AddBlob`, `RemoveBlob`, `SetFlushedLsn`, `ClusterState{cluster_id, Live|Erasing|Erased, restriction_flags}`, `LinkAdd/LinkRemove`, `ScrubPending{run, cluster}`, `SegmentRegistry`. Gelöschte Cluster verschwinden bei der nächsten Snapshot-Generation; alte Generationen werden im Purge gelöscht.

### Ledger (`LEDGER.lled`)

```text
Entry: seq u64 | prev_hash[32] | wall_time | kind u8 | tomb_id[32] | stamp | lsn |
       receipt_len u32 | receipt | entry_hash[32] = SHA-256(...) | mac[32] = HMAC(ledger_key, entry_hash)
kinds: GateCommitted, KeysScheduled, KeysDestroyed, PurgeProgress, Completed, RestrictionSet, RestrictionLifted, LinkCascade
```

Nach jedem Eintrag, der einen Löschfortschritt belegt, ruft der Kern `Anchor::advance(counter+1, ledger_head)` auf. Das Ledger enthält keine ClusterIds, nur `tomb_id`s, und keine Payload-Bytes.

## 3. Schlüsselhierarchie, Rotation und Vernichtung

Jeder Cluster besitzt einen eigenen 256-Bit-Schlüssel pro Epoche. Die einzige Kopie in Klartext liegt zeitweise im Speicher des `KeyProvider`-Aufrufers (`Zeroizing`); persistent existiert der Schlüssel nur eingewickelt unter einem Master-KEK oder in einem KMS.

```text
Master-KEK / KMS-Schluessel            (Betreiber, ausserhalb des Stores)
   |  wrap
   v
K_cluster[epoch]   256 Bit, zufaellig, pro Cluster und Epoche
   |  HKDF-Expand(info = "lethe/v1/" + frame_kind)
   v
K_wal | K_meta | K_idx | K_hdr | K_body | K_blob | K_idem     (Teilschluessel, nur im RAM)

Ledger-Schluessel, Bloom-Geheimnis: Store-weit, getrennt vom Cluster-Material
```

### Epochen und Rotation ohne Neuverschlüsselung

- Eine Epoche ändert nur, womit **neue** Frames versiegelt werden. Bestehende Sektionen behalten ihre Epoche (im Sektions-Header und in der AAD). Das Alte wird nicht angefasst.
- KEK-Rotation ist Sache des Providers (Re-Wrap der gespeicherten `K_cluster`), nicht des Stores.
- Compaction darf Sektionen mehrerer Epochen eines Clusters zusammenführen und mit der aktuellen Epoche neu versiegeln; erzwungen ist das nie.
- `destroy(cluster, EpochScope::All)` vernichtet alle Epochen. Ein Teil-Scope ist für Betreiber-Rotationsrichtlinien vorgesehen, aber nie für Löschtickets.

### Vernichtungsstärke und ehrliche Nachweise

| Provider | `destroy` bedeutet | Receipt | Hinweis |
| --- | --- | --- | --- |
| LocalKeyring | Wrapped-Key-Datensatz überschreiben, Datei neu schreiben, entfernen | `LocalWipe` | Kopien in Dateisystem-Snapshots oder Backups des Keyrings bleiben außerhalb der Kontrolle |
| KMS pro Cluster | Schlüssel zur Löschung vormerken | `KmsScheduled{until}` | Viele KMS erzwingen Karenzzeiten (Tage); bis dahin ist der Schlüssel technisch wiederherstellbar |
| KMS pro Cluster | Karenzzeit abgelaufen | `KmsDestroyed` | Erst ab hier ist G1 voll erreicht |
| Hierarchisch (ein KMS-Schlüssel, viele gewrappte DEKs) | Gewrappten DEK-Datensatz löschen | `LocalWipe` | Nur belastbar, wenn **keine** Kopie des gewrappten DEK existiert (Backups des Keyrings ausschließen) |

Das Ticket führt die Stufen `KeysScheduled` und `KeysDestroyed` getrennt. Betreiber stellen per Konfiguration eine Mindeststärke ein; die Nachweisurkunde (Ledger-Eintrag plus Anker-Stand) nennt die tatsächlich erreichte Stärke wörtlich.

### Schlüssel-Cache und Speicherhygiene

- Der Cache hält `KeyHandle`s (`Arc`, `Zeroizing`) mit Epoche und Erase-Generation. Ein Gate invalidiert den Eintrag atomar; `Drop` nullt die Bytes.
- Kryptografie und KMS-Aufrufe laufen vor dem Commit-Lock in `Txn::prepare`. Im Commit-Pfad gibt es nur Cache-Treffer.
- Entschlüsselte Index-, Header- und Body-Caches sind pro Cluster geschlüsselt und werden beim Gate synchron verworfen (und genullt).
- Ein `MemLock`-Port (mlock, `MADV_DONTDUMP`) existiert nur im Adapter, weil dort `unsafe` toleriert wird; der Kern bleibt `forbid(unsafe_code)`.

### Zufall und Nonces

`Entropy` ist der einzige Zufallsweg. 192-Bit-Nonces werden pro Frame frisch gezogen (Kollisionswahrscheinlichkeit vernachlässigbar bis 2^64 Frames je Schlüssel). `DetEntropy` erzeugt in Tests reproduzierbare, aber eindeutige Nonces; die Test-Suite prüft Nonce-Eindeutigkeit explizit (INV-9).

## 4. Zustandsmaschinen und Protokolle

Jedes Protokoll ist so geschnitten, dass ein Crash nach jedem einzelnen Schritt fortsetzbar ist. Sperren heißen `L1` bis `L4` (Ordnung siehe Abschnitt 6).

### Cluster-Zustände

```text
            create/put                 EraseGate durable
   (neu) ---------------> Live ----------------------------> Erasing
                           |  ^                                  | KeyProvider.destroy + Ledger
                 restrict  |  | lift                             v
                           v  |                              KeysDestroyed
                        Restricted                               | Scrub + WAL/Katalog-Purge + Extensions
                   (erase => Err(Restricted))                    v
                                                              Erased (Tombstone, nie wieder Live)
```

### `apply` (Commit)

1. **Vorbereiten, ohne Sperre:** Cluster-Zustand prüfen, Schlüssel-Handles beschaffen (`unwrap` höchstens hier), Ops pro Cluster zu einem `SealFrame` versiegeln. `IdemToken` gegen die heiße Tabelle und notfalls gegen den `_idem`-Space des Clusters prüfen (kalte I/O hier, nie im Lock).
2. **Guard-Vorprüfung, ohne Sperre:** gegen `cluster.head_stamp` (atomar) und die `HotVersionMap`. Konservativ: Unklar heißt Konflikt (retrybar), nie falsches Akzeptieren. Keine Platten-I/O.
3. **Enqueue** in die Commit-Queue; der erste Wartende wird Leader (Flat-Combining).
4. **Leader unter `L3 commit_lock`:** Batch ziehen, Guards und `ErasedSet` erneut prüfen (rein im Speicher), LSN und Stamp vergeben, WAL-Records serialisieren, `append`, **ein** `sync_data` für den Batch, Memtable einfügen, `committed_stamp` mit Release publizieren, Lock lösen, Wartende wecken.
5. **Nach dem Lock:** Cache-Bereinigung, Metriken, Flush-Trigger.

Fehlerpfade: Schreib- oder Sync-Fehler vergiftet den Store (`Poisoned`), weil ein fehlgeschlagenes `fsync` keinen sicheren Retry erlaubt; Recovery entscheidet beim Neustart. `ENOSPC` vor dem Append führt zu `DiskFull` (siehe unten).

### `view` und `page`

1. `view()`: `read_stamp = committed_stamp` laden (Acquire), dann `Arc<Version>` und `Arc<ErasedSet>` laden. Eintrag in die `ViewRegistry` mit Lease (Default 60 s), damit GC-Wasserzeichen und Purge-Fristen berechenbar bleiben.
2. `page(cluster, space, req)`: Cluster im aktuellen `ErasedSet` prüfen (Erased gibt `Err(Erased)`); Sektionen über Directory finden; `MetaFrame`, dann `IdxFrame` je Block entschlüsseln (Cache pro Cluster).
3. Iteration `Forward` oder `Backward`, Sichtbarkeit per `stamp <= read_stamp`; Blöcke mit `lane_min/max` außerhalb des Filters werden übersprungen. Token-Budget wird aus `weight` im `IdxFrame` summiert; ein Eintrag, der das Budget sprengen würde, beendet die Seite (die Seite enthält mindestens einen Eintrag, wenn einer existiert).
4. Header-Prädikat nur auf Kandidaten und nur über `HdrFrame`. Bodies bleiben `Lazy` (`page.body(i)`).
5. **Check-after-read:** Vor der Rückgabe wird der `ErasedSet` ein zweites Mal geprüft. Wurde der Cluster inzwischen gegatet, bekommt der Aufrufer `Err(Erased)` statt Daten.

### `flush`

1. Unter `L3` kurz: aktive Memtable gegen eine neue tauschen, `flush_lsn` festhalten, Lock lösen.
2. Hintergrund: je Cluster Blöcke bilden und versiegeln (Cluster im `ErasedSet` überspringen), Sektionen schreiben, Directory und Footer, `sync_data`.
3. Unter `L2 catalog_mutex`: `erase_epoch` mit dem Wert vom Start vergleichen. Hat sich ein Gate dazwischengeschoben, werden die Sektionen betroffener Cluster vor dem Publish gescrubbt (oder der Run verworfen).
4. `AddRun` + `SetFlushedLsn` ins Edit-Log, `sync`, neue `Version` publizieren.
5. WAL-Segmente mit `max_lsn <= flushed_lsn` löschen, soweit keine Restriction oder offene Erase-Tickets sie halten.

### `compaction`

1. Auswahl nach Größenstufen; Eingaben pro Cluster zusammenführen.
2. Liegt ein Cluster nur in einem Eingabe-Run und enthält keine abgelaufenen Versionen: **Rohkopie** der Sektion (kein Entschlüsseln, positionsunabhängige AAD).
3. Sonst: nur diese Cluster-Sektionen entschlüsseln, MVCC-GC bis zum `View`-Wasserzeichen, mit neuer Nonce versiegeln.
4. Atomar: `RemoveRun*` + `AddRun*` als eine Edit-Gruppe. Alte Dateien werden gelöscht, sobald keine `Version` sie mehr referenziert.
5. Gleiche Gate-Nachprüfung wie beim Flush; Erased-Cluster werden nie kopiert.

### `erase` (drei Phasen, Ticket-gesteuert)

**Phase A, Gate (synchron):**

1. Kaskadenmenge aus der `LinkTable` bestimmen (transitiver Abschluss über `DerivedFrom`), ohne Sperre.
2. Unter `L3`: Restrictions prüfen (`Err(Restricted)` ohne Override), Link-Version validieren, Stamp `S_e` vergeben, `EraseGate`-Record mit allen `tomb_id`s in den Group-Commit geben, `sync_data`.
3. `ErasedSet` per Pointer-Swap publizieren, `ClusterTable` auf `Erasing`, Katalog-Edit `ClusterState`. Ab hier liefern `view`/`page`/`ChangeFeed` nichts mehr; `erase()` gibt das Ticket mit Stufe `Gated` zurück.

**Phase B, Krypto (Driver-Thread, idempotent):**

4. `KeyProvider::destroy(c, All)` je Cluster; Receipt holen.
5. Schlüssel-Cache, Memtable-Bereiche, Index-/Header-/Body-Caches und `Volatile`-Regionen des Clusters verwerfen und nullen.
6. Ledger: `KeysScheduled` bzw. `KeysDestroyed` mit Receipt anhängen, `sync`, `Anchor::advance`. Erst jetzt Stufe `KeysDestroyed`.

**Phase C, Purge (bounded by `purge_deadline`):**

7. **In-place-Scrub:** Für jede Sektion des Clusters in Runs und Blobs: zuerst `ScrubPending` im Katalog, dann Sektionsbytes mit Nullen überschreiben, `sync_data`; im `Directory` den Eintrag auf `dead` setzen und die ClusterId nullen. Das kostet nur Clustergröße, keinen Platz und keinen Rewrite fremder Daten.
8. **WAL:** Die Memtable ist bereits ohne den Cluster. Sobald ein Flush alle LSNs bis zur Gate-LSN abdeckt, werden die betroffenen WAL-Segmente gelöscht (bei Bedarf Flush erzwingen).
9. Katalog: neue Snapshot-Generation ohne den Cluster, alte Generationen und Edit-Logs löschen.
10. Run-Rewrite zur Platzrückgewinnung ist lazy (Compaction, wenn `dead`-Anteil über Schwelle); für die Compliance nicht nötig.
11. Extensions: `DerivedSegment::purge(cluster)` bis `Done`. Das Ticket wartet auf alle im `SegmentRegistry` eingetragenen Segmente.
12. Ledger `Completed`, `Anchor::advance`. Stufe `Completed`.

Phase B und C starten nach einem Crash aus dem Ledger und dem `ClusterState` neu. Jeder Schritt ist idempotent.

### `DiskFull`-Notbetrieb

- Auslöser: `ENOSPC` bei Append oder `free_space < Schwelle`.
- Reguläre `apply`-Aufrufe liefern `Err(DiskFull)`; `view`, `page`, `erase` bleiben nutzbar.
- Gate und Krypto brauchen nur ein kleines WAL-Append und ein Ledger-Append. Dafür liegt ein vorallokiertes `RESERVE`-WAL-Segment bereit; zusätzlich gibt `reserve.bin` bei Bedarf Platz frei.
- Der Scrub in Phase C schreibt in bestehende Bereiche und braucht keinen neuen Platz. Rewrites werden in DiskFull ausgesetzt.
- Verlassen des Modus: Platz wieder über Schwelle, Reserve neu anlegen, Warteschlange läuft weiter.

### `checkpoint` und `restore`

- `checkpoint()`: Flush, `committed_stamp` festhalten, Katalog-Generation plus referenzierte Runs/Blobs kopieren (oder Reflink). Das Manifest enthält Anker-Zähler und Ledger-Head. **Keine** Cluster-Schlüssel gehören ins Checkpoint. Wer harte Links nutzt, muss wissen, dass In-place-Scrub auch die verlinkte Kopie schwärzt (gewollt: Löschung gewinnt).
- `restore(checkpoint)` läuft im Modus `Restoring`: (1) Ledger der Live-Instanz bzw. aus dem Anker-Archiv laden und mit `Anchor::read` prüfen, sonst `RollbackDetected`; (2) Ledger-Kette des Checkpoints muss Präfix des aktuellen Ledgers sein; (3) alle Tombstones erneut anwenden: passende Cluster im Checkpoint werden gescrubbt, ihre Schlüssel (schon vernichtet) bleiben vernichtet; (4) erst dann `Ready`. Ein Checkpoint, dessen Ledger-Stand neuer ist als der Anker, wird abgelehnt.

### `recovery` (beim `open`)

1. `LOCK` nehmen. `CURRENT` lesen, Katalog-Snapshot und Edit-Log laden; Torn-Tail abschneiden.
2. Dateien ohne Katalog-Eintrag (verwaiste Runs/Blobs, `.tmp`) löschen. Das ist sicher, weil eine unreferenzierte Datei nie die einzige Kopie von Daten war.
3. Ledger-Kette prüfen, gegen `Anchor::read` abgleichen (`RollbackDetected`, wenn der Store älter ist als der Anker; zu jüngerem Anker: fehlende Ledger-Einträge sind Korruption).
4. `ClusterTable` aufbauen, Tombstones aus Ledger und Katalog anwenden.
5. WAL ab `flushed_lsn` replayen: Commit-Gruppen von Erased/Erasing-Clustern verwerfen, `EraseGate`-Records anwenden, Cluster mit vernichtetem Schlüssel überspringen.
6. Offene Tickets aus dem Ledger fortsetzen (Phase B/C), offene `ScrubPending`-Einträge abarbeiten, halbe Kompaktionen anhand der Edit-Gruppen auflösen.
7. `Ready`. Recovery ist idempotent: `recover(recover(S)) = recover(S)` für jeden Crash-Zustand `S`.

## 5. RAG-Kaskaden, Extensions, Sperrvermerke und Rollback-Schutz

Die Löschung eines Quellclusters muss jedes abgeleitete Artefakt erreichen: Chunk-Embeddings, ANN-Graphen, BM25-Postings, Knowledge-Graph-Kanten, Zuordnungstabellen. Der Kern erzwingt das über drei Verträge statt über Wissen über diese Strukturen.

### Cluster-Links (atomare Kaskade)

- `Op::Link { parent, child, kind }` ist Teil eines Commits, damit der Link atomar mit den ersten Daten des Kindes entsteht. `kind = DerivedFrom` kaskadiert; `kind = Reference` kaskadiert nicht.
- Der Kern erzwingt **Azyklizität** beim Anlegen (Erreichbarkeitsprüfung im In-Memory-Graphen, Tiefenlimit) und hält die `LinkTable` im Katalog.
- `erase(P)` bestimmt den transitiven Abschluss über `DerivedFrom` und schreibt **einen** `EraseGate`-Record mit allen `tomb_id`s. Es gibt keinen Zustand, in dem P gelöscht ist und ein abgeleiteter Cluster noch lebt.
- Ein Kind mit mehreren Eltern wird gelöscht, sobald **eines** seiner `DerivedFrom`-Eltern gelöscht wird (konservativ: Daten, die aus einer gelöschten Quelle abgeleitet sind, verschwinden). Ausnahmen bildet die obere Schicht mit `Reference`.

### Extension-Vertrag für abgeleitete Strukturen

```rust
pub trait DerivedSegment: Send + Sync {
    fn id(&self) -> SegmentId;
    /// Sofort, ohne I/O: Treffer dieser Cluster ab jetzt unterdruecken.
    fn on_gate(&self, erased: &[ClusterId], at: Stamp);
    /// Physisch entfernen (Posting-Listen, ANN-Knoten, KG-Kanten, Caches).
    fn purge(&self, erased: &[ClusterId], deadline: Instant) -> PurgeOutcome; // Done | Pending{retry_after}
    /// Fuer das Loeschorakel: Ist wirklich nichts mehr vorhanden?
    fn verify_absent(&self, c: ClusterId) -> bool;
    fn watermark(&self) -> Stamp;                                            // bis wohin der Feed verarbeitet wurde
}
```

- Segmente registrieren sich im `SegmentRegistry` des Katalogs. Das Ticket ist erst `Completed`, wenn **jedes** registrierte Segment `Done` gemeldet hat. Ein Segment lässt sich nur über eine eigene, ledger-protokollierte Operation abmelden.
- `on_gate` ist Pflicht-Sofortpfad und darf nicht blockieren. Segmente speichern ihre Daten bevorzugt **als Cluster-Sektionen im Kern** (eigener Space, eigene Sektionstypen); dann erledigt der Kern das Löschen selbst und `purge` ist trivial. Externe Indizes (HNSW-Dateien, Tantivy) müssen clusterbezogene Löschung selbst anbieten oder pro Cluster/Partition aufgebaut sein.
- Ein Segment, das nur eine globale Neuerstellung kennt, darf sich nur mit einem erklärten `Rebuild`-Intervall registrieren; das Ticket trägt dann diese Frist ehrlich als Wartezeit.

### Change-Feed

```rust
enum Change {
    Put      { cluster: ClusterId, space: SpaceId, key: Key, stamp: Stamp, header: HdrRef },
    Delete   { cluster: ClusterId, space: SpaceId, key: Key, stamp: Stamp },
    Redacted { cluster: ClusterId, stamp: Stamp },   // Inhalte entfernt, Existenz bleibt bis Erased
    Erased   { cluster: ClusterId, stamp: Stamp },   // endgueltig; Indexer muss purgen
}
fn poll(&self, since: Stamp, max: usize) -> Result<Vec<Change>>;
```

Das Feed liest in Stamp-Reihenfolge aus WAL und Runs und **lässt gelöschte Cluster nie mit Inhalt aus**: Ab dem Gate erscheint für sie nur noch `Redacted`/`Erased`. Ein Indexer, der vor dem Gate bereits `Put` verarbeitet hat, erhält `Erased` und muss `DerivedSegment::purge` ausführen. Die Wasserzeichen der Segmente (`watermark`) halten WAL-Segmente und Runs nicht länger als nötig fest.

### Zuordnung Person zu Cluster (Muster für die obere Schicht)

Der Kern stellt dafür nichts Personenbezogenes bereit. Empfohlenes Muster: Pro Person ein **Wurzel-Cluster**, der alle Dokument-Cluster per `DerivedFrom` verknüpft und die Zuordnung (Dokument zu ClusterId) als eigenen Space enthält. Die Suche `person -> Wurzel-ClusterId` läuft über einen `HMAC(person_id)`-Schlüssel in einem Verzeichnis, dessen Einträge beim Erase des Wurzel-Clusters per `Op::Link` mitgelöscht werden. Dadurch gibt es keine Zuordnungstabelle, die den Löschvorgang überlebt.

### Sperrvermerke (`Restriction`)

- `restrict(cluster, id, mode, until)` mit `mode = NoErase` oder `NoErase+NoRead`. Der Kern kennt nur die opake `restriction_id` und die Frist als `Stamp`/Wallclock; Grund und Rechtsgrundlage kennt er nicht.
- `erase` auf einen Cluster mit aktiver Restriction (oder mit einem Nachfahren in der Kaskade, der eine hat) liefert `Err(Restricted{ids})`. Es gibt **keinen** Override-Pfad im Kern; die obere Schicht muss erst `lift(id)` aufrufen.
- Restriction und Löschpflicht stehen so nie im Konflikt innerhalb des Kerns: Der Zustand ist im Ledger sichtbar (`RestrictionSet/Lifted`), das Gate wird erst nach `lift` möglich.

### Rollback-Anker und Ledger

- Das Ledger ist hashverkettet und MAC-gesichert. Der `Anchor` (TPM-NV-Zähler, Object-Lock-Bucket, Transparency-Log, HSM) hält `{counter, ledger_head}` und wird per Compare-and-Swap fortgeschrieben.
- `open` verweigert (`RollbackDetected`), wenn Katalog-Generation oder Ledger-Länge hinter dem Anker zurückliegen. So lässt sich ein altes Dateisystem-Image nicht stillschweigend zurückspielen.
- Ist der Anker nicht erreichbar, gilt die Policy `strict` (Löschvorgang bleibt auf `AnchorPending`, `Completed` wird nicht gemeldet) oder `best_effort` (lokal fortfahren, `AnchorPending` im Ticket sichtbar). Der Nachweis nennt immer den tatsächlichen Anker-Stand.

## 6. Concurrency- und Invarianten-Katalog

Es gibt genau einen Schreiber (den Group-Commit-Leader) und beliebig viele lock-freie Leser. Alles, was Zeit kostet (Krypto, KMS, kalte Platten-I/O, Guard-Vorprüfung, Cache-Bereinigung), liegt außerhalb des Commit-Locks.

### Sperrordnung (immer von oben nach unten, nie umgekehrt)

| Stufe | Sperre | Schützt | Darf unter der Sperre |
| --- | --- | --- | --- |
| L1 | `store_flock` (Prozess-Lock) | exklusiver Zugriff auf das Verzeichnis | nichts Besonderes, nur `open`/`close` |
| L2 | `catalog_mutex` | Edit-Log, Run-/Blob-Menge, `erase_epoch`, `LinkTable` | Katalog-Append + `sync`, `Version`-Publish; kein Entschlüsseln |
| L3 | `commit_lock` | LSN/Stamp-Vergabe, WAL-Append, Memtable-Rotation, Gate-Record | WAL-Append + ein `sync_data`, Memtable-Insert; **kein** KMS, keine Krypto, keine Lesen von Runs |
| L4 | `cluster_table` (Schreibseite) und Memtable-Shards | Zustandsübergänge `Live/Erasing/Erased`, Shard-Einfügungen | reine Speicheroperationen, Mikrosekunden |
| – | `ViewRegistry` | Lease-Liste, Wasserzeichen | lock-frei (Slab mit Atomics) |

L2 und L3 werden nie gleichzeitig in umgekehrter Reihenfolge gehalten: Der Flush nimmt L3 nur kurz zum Memtable-Tausch, danach L2 getrennt. Der Erase-Driver hält während KMS-Aufrufen **keine** der Sperren.

### Lock-freie Pfade

- `view()`: Atomics + `Arc`-Swap (`ArcSwap` oder `RwLock<Arc<_>>` mit kurzer Haltezeit). Reihenfolge: `Version` publizieren, dann `committed_stamp` (Release); Leser laden Stamp (Acquire), dann `Version`.
- `ErasedSet`: unveränderliche Persistent-Map hinter einem Pointer-Swap; Leser sehen immer eine vollständige Momentaufnahme.
- Key-Cache: `Arc<KeyHandle>`; Invalidierung durch neuen `ErasedSet`/Cache-Generation, nicht durch Lock.
- Metriken und Wasserzeichen: Atomics.

### Kritische Race-Nahtstellen

| ID | Race | Auflösung |
| --- | --- | --- |
| R1 | `page` liest, während das Gate publiziert | Check-after-read: zweite `ErasedSet`-Prüfung vor Rückgabe; gelesene Daten gelten als vor dem Gate geliefert |
| R2 | Flush schreibt Sektion eines Clusters, der gerade gegatet wird | `ErasedSet`-Prüfung je Sektion + `erase_epoch`-Vergleich unter L2 vor dem Publish; sonst Scrub oder Verwurf des Runs |
| R3 | Compaction kopiert Sektion eines gegateten Clusters | gleiche Nachprüfung; Rohkopie-Sektionen werden vor Publish gegen `ErasedSet` gefiltert |
| R4 | Writer versiegelt mit gecachtem Schlüssel, Gate kommt dazwischen | Leader prüft `ErasedSet` unter L3; Gruppe wird mit `Err(Erased)` verworfen |
| R5 | Zwei Writer mit gleichem `IdemToken` | Leader prüft heiße Tabelle unter L3; kalter Treffer wurde in `prepare` geprüft |
| R6 | `rotate` während Commit | `KeyHandle` trägt Epoche; Leader akzeptiert nur Epochen `<=` aktuelle, neue Writes holen neues Handle |
| R7 | Checkpoint vs. In-place-Scrub | Checkpoint-Kopien sind eigenständig (Copy/Reflink); harte Links werden mitgescrubbt (dokumentiert, Löschung gewinnt) |
| R8 | Doppeltes `erase` | idempotent: gleicher Tomb → gleiches Ticket |
| R9 | Erase vs. offene Views | Gate invalidiert nur den Cluster, nicht die View; View-Lease (Default 60 s) begrenzt, wie lange alte `Version`s Dateien festhalten |
| R10 | Gate-Crash zwischen WAL-Record und `ErasedSet`-Publish | Recovery wendet den `EraseGate`-Record an, bevor irgendein `view` geöffnet wird |

### Invarianten

| ID | Aussage |
| --- | --- |
| INV-1 Reinheit | Jeder Frame, jede Sektion und jeder Blob-Chunk gehört zu genau einem Cluster. |
| INV-2 Schlüsselabschluss | Jedes persistente Byte, das aus Nutzdaten stammt (Header, Body, Gewichte, Lanes, Index, Filter, Bereichsmarker, Idem-Tokens), liegt in einem `SealFrame` des Clusters. Klartext bleibt nur: ClusterId (opak), Größen, Zähler, Dateistruktur. |
| INV-3 Gate | Ab dem durablen `EraseGate`-Record liefert kein `view`/`page`/`poll` Inhalt des Clusters, und kein Commit schreibt hinein. |
| INV-4 Gate vor Schlüssel | `destroy` wird nie vor dem durablen Gate aufgerufen. |
| INV-5 Schlüssel vor Ack | `KeysDestroyed` nur nach Receipt mit Mindeststärke und gesyncktem Ledger-Eintrag. |
| INV-6 Ledger | Append-only, Hashkette lückenlos, `anchor.counter <= ledger.seq`. |
| INV-7 Keine Wiederbelebung | Tomb im Ledger ⇒ Cluster nach jedem Recovery/Restore nie `Live`; ClusterId nie wiederverwendet. |
| INV-8 Positionsunabhängigkeit | Keine AAD hängt von Datei-ID oder absolutem Offset ab; Rohkopie bleibt gültig. |
| INV-9 Nonce-Eindeutigkeit | Kein Paar (Schlüssel, Nonce) wird zweimal verwendet. |
| INV-10 Atomarität | Ein Commit ist nach jedem Crash vollständig oder gar nicht sichtbar, über alle Spaces. |
| INV-11 Determinismus | Recovery ist eine Funktion von (Dateien, Ledger, Schlüssel) und idempotent. |
| INV-12 Katalog | Eine vom Katalog referenzierte Datei existiert und ist vollständig gesynct; Löschen erst nach dem referenzierenden Edit. |
| INV-13 Stamp-Ordnung | Stamps sind streng monoton, auch über Neustarts; `committed_stamp` wird nach dem Memtable-Insert publiziert. |
| INV-14 Keine Mischsicht | Pro Cluster sieht jede View entweder den vollständigen Snapshot oder `Erased`. |
| INV-15 Purge-Vollständigkeit | Nach `Completed` findet das Löschorakel in den Store-Dateien keine Bytes von Klartext, Ciphertext oder ClusterId des Clusters. |
| INV-16 DiskFull-Liveness | Gate und Krypto bleiben bei `ENOSPC` ausführbar (Reserve); der Scrub braucht keinen neuen Platz. |
| INV-17 Lock-Disziplin | Sperren werden nur in Ordnung L1\<L2\<L3\<L4 gehalten; L3 enthält nur WAL-Append, ein `sync_data` und Speicheroperationen. |
| INV-18 Kaskadenabschluss | Die Erase-Menge ist der transitive `DerivedFrom`-Abschluss zum Gate-Zeitpunkt, in einem Record. |
| INV-19 Restriction | `erase` scheitert genau dann, wenn eine Restriction aktiv ist; kein Override im Kern. |
| INV-20 Zeroization | Schlüssel und Klartext-Puffer sind `Zeroizing`; beim Gate werden sie sofort verworfen. |
| INV-21 Rollback | `open` scheitert, wenn Katalog oder Ledger hinter dem Anker liegen. |
| INV-22 Löschlokalität | Gelesene und geschriebene Bytes eines `erase` sind ≤ c · (Clustergröße) + konstante Metadaten, unabhängig von der Storegröße. |

## 7. Verifikation

Die Beweislast liegt auf Testwerkzeugen, die jede Invariante mechanisch angreifen. Alles läuft deterministisch aus einem Seed und ist reproduzierbar.

### `FaultFs`: deterministisches Fehlermodell

`FaultFs` implementiert `Fs` auf einem simulierten Gerät und modelliert die Dinge, die echte Dateisysteme tun:

- **Pufferung bis zum Sync:** Schreibvorgänge sind erst nach `sync_data`/`sync_dir` durabel. Ein Rename ist erst nach `sync_dir` durabel.
- **Torn Writes:** Beim simulierten Crash überlebt jedes unsynchronisierte `write` als Präfix auf Sektor-Granularität (512 B oder 4 KiB), beliebig umsortiert über Dateien hinweg.
- **Fehlerinjektion:** `ENOSPC` bei der n-ten Schreiboperation, `EIO`, Kurzschreibungen, fehlgeschlagenes `fsync` (der Store muss `Poisoned` melden und beim Neustart sauber recovern).
- **Crash-Sweep:** Ein Harness lässt denselben Workload N-mal laufen und crasht beim Fs-Operation Nr. 0..N, jeweils mit mehreren Torn-Varianten. Nach jedem Crash: `recover`, Invarianten prüfen, Modellvergleich, erneutes `recover` (Idempotenz).
- **Byte-Zähler:** je Operation und Datei gelesene/geschriebene Bytes, Grundlage des Lokalitätstests (INV-22).
- **Geister-Journal:** Das Gerät behält optional alle jemals geschriebenen Blöcke (auch überschriebene/gelöschte), um Restexposition auf der Medienebene sichtbar zu machen.

### Löschorakel (Byte-Vollsuche nach Purge)

Jeder Testcluster bekommt zufällige **Canaries**: eindeutige 32-Byte-Muster in `header`, `body`, `lanes`-Kodierung, Idem-Token und Schlüsselmaterial (der Test-`KeyProvider` gibt seine Schlüssel preis). Zusätzlich werden 16-Byte-Stichproben aus jedem versiegelten Frame mitgeschrieben.

| Orakel | Durchsucht | Erwartung nach `Completed` |
| --- | --- | --- |
| O-Files | alle Dateien des Stores einschließlich WAL, Katalog-Generationen, Ledger, `.tmp` | **keine** Treffer für Canaries, Schlüssel, Teilschlüssel, ClusterId, Ciphertext-Stichproben |
| O-Device | Geister-Journal des simulierten Geräts | keine Klartext-Canaries und keine Schlüssel; Ciphertext-Treffer sind erlaubt und werden als G1-Schutz ausgewiesen |
| O-Gate | alle `view`/`page`/`poll`-Aufrufe ab Gate | kein Inhalt des Clusters |
| O-Ledger | Ledger-Bytes | nur `tomb_id`, keine ClusterId, keine Payload |
| O-Segments | `DerivedSegment::verify_absent` | für jedes registrierte Segment `true` |

Das Orakel läuft auch **zwischen** den Phasen (Crash in Phase B oder C, Recovery, Fortsetzung) und nach `restore` eines alten Checkpoints.

### Property-Tests

- **Modellbasiert** (`proptest`): Ein einfaches Referenzmodell (`BTreeMap` je Cluster und Space plus Stamps) läuft parallel zur Engine über zufällige Sequenzen aus `apply`, `view`, `page` (beide Richtungen, Budgets, Lane-Filter), `flush`, `compaction`, `rotate`, `erase`, `restrict`, `crash+recover`.
- **Paging:** Budget wird nie überschritten, Seiten sind lückenlos und reihenfolgetreu, `Forward` und `Backward` liefern dieselbe Menge, Lane-Pruning liefert dieselbe Ergebnismenge wie ein Vollscan.
- **Äquivalenz:** Rohkopie-Compaction und Entschlüsseln-Mergen-Neuversiegeln liefern logisch identische Läufe.
- **Erase:** Kaskadenabschluss, Restriction-Verhalten, Idempotenz, Gate-Atomarität über mehrere Cluster.
- **Lokalität:** Bei wachsender Storegröße (×1, ×10, ×100) bleibt der Byteverbrauch eines `erase` proportional zur Clustergröße.

### Fuzzing und Korruption

- `cargo-fuzz`-Ziele für jeden Decoder in `lethe-format`: `SealFrame`, WAL-Record, Run-Directory, Katalog, Ledger. Anforderung: nie Panic, nie Out-of-Bounds-Allokation, immer `Result`.
- Bit-Flip-Differenztest: Jedes einzelne Byte eines gültigen Frames/Runs/Ledgers wird gekippt. Erwartung: `Corrupt` oder Authentifizierungsfehler, nie stille falsche Daten.
- `miri` auf `lethe-format` und `lethe-seal` (rein, kein I/O).

### Nebenläufigkeit

- `loom` auf den kleinen lock-freien Kernen (Publish-Reihenfolge Version/Stamp, `ErasedSet`-Swap, Flat-Combining-Queue).
- `shuttle` (randomisierte, reproduzierbare Schedules) auf der ganzen Engine über `FaultFs`: Race-Nahtstellen R1 bis R10 sind jeweils ein benannter Szenario-Test mit erzwungenem Interleaving.
- Mutationstests (`cargo-mutants`) auf Erase-Pfad, Recovery und Guard-Logik: Überlebende Mutanten gelten als Testlücke.

### CI-Gates

`cargo deny`, `clippy::disallowed_methods` (kein `std::fs` im Kern), `forbid(unsafe_code)`-Prüfung, Fuzz-Smoke (5 min je Ziel), Crash-Sweep (festes Budget pro PR, Nacht-Läufe mit großem Seed-Raum), p99-Latenz-Benchmarks für Commit und Erase-Gate (`criterion`), Soak-Test über 72 h vor jedem Release.

## 8. Phasenplan mit vertikalen Durchstichen

Phase 1 liefert bereits einen vollständigen Ende-zu-Ende-Pfad (Schreiben, Lesen, Löschen, Orakel, Crash-Sweep), wenn auch auf einem winzigen Funktionsumfang. Jede weitere Phase verbreitert diesen Pfad und darf die Orakel- und Crash-Tests der Vorphasen nicht brechen. Die Aufwände sind grobe Schätzungen für ein Team aus 2-3 erfahrenen Rust-Entwicklern und gelten als Planungsannahme, nicht als Zusage.

| Phase | Vertikaler Durchstich | Exit-Kriterium (Gate) | Grobe Dauer |
| --- | --- | --- | --- |
| P0 Fundament | Workspace, `forbid(unsafe_code)`, Ports, `MemFs`/`FaultFs` (Grundmodell), `SealFrame` mit XChaCha20-Poly1305-Adapter, Orakel-Skelett, CI-Gates | Bit-Flip-Test und Fuzz-Smoke für `SealFrame` grün; Orakel findet absichtlich eingestreute Canaries | 2 Wochen |
| P1 Erase-Walking-Skeleton | Ein Space, `apply` (put), WAL + Memtable, `view`/`page` forward, `erase` mit Gate + LocalKeyring + Ledger (noch ohne Anker); Recovery aus WAL | Crash-Sweep über alle Fs-Operationen grün; O-Files findet nach `Completed` nichts, obwohl Daten nur in WAL/Memtable lagen | 4 Wochen |
| P2 Runs und Katalog | Flush, cluster-reine Sektionen mit positionsunabhängiger AAD, Katalog + `CURRENT`, In-place-Scrub, WAL-Segment-Ablöse, vollständige Recovery | O-Files nach Erase für Daten in Runs grün; Crash-Sweep inkl. Flush und Scrub; Modelltest grün | 5 Wochen |
| P3 Lastprofil und Agenten-Primitive | Kopf-/Körper-Trennung, Blobs, Gewichte, Lanes und Backward-Paging, `Guard`/`IdemToken`, `Volatile`, `ViewRegistry` mit Lease, Group-Commit | Paging-Propertytests (Budget, Vollständigkeit, Lane-Pruning); Commit-Benchmark mit Zielwerten (p99); Erase unter Last | 5 Wochen |
| P4 Compaction und Notbetrieb | Leveling, Rohkopie, Entschlüsseln-Mergen, MVCC-GC, `DiskFull`-Modus mit Reserve | Lokalitätstest INV-22 bei ×1/×10/×100; ENOSPC-Sweep: Erase bleibt ausführbar | 4 Wochen |
| P5 Anker, Checkpoint, Restore | Anker-Port + Referenzadapter, Hashketten-Ledger, `checkpoint`/`restore`, Rollback-Erkennung, KMS-Adapter mit `KmsScheduled`/`KmsDestroyed` | Restore alter Images erzeugt keine lebenden gelöschten Cluster (O-Files nach Restore); `RollbackDetected`-Tests | 4 Wochen |
| P6 Kaskaden | `Cluster-Links`, `DerivedSegment`-Vertrag, Change-Feed, Referenz-Extension (Mini-ANN + Mini-BM25 im Testkit), Restriction | Erase eines Wurzel-Clusters räumt alle abgeleiteten Strukturen im selben Gate; O-Segments grün; Feed liefert nie Inhalte nach dem Gate | 5 Wochen |
| P7 Härtung | `loom`/`shuttle`-Szenarien R1-R10, Fuzz-Kampagnen, Mutationstests, 72-h-Soak, Bedrohungsmodell, Nachweisformat (maschinenlesbarer Erasure-Report) | Alle `INV-*` mit mindestens einem automatisierten Test belegt; keine offenen Korruptionsbefunde | 4 Wochen |

### Prioritäten innerhalb der Phasen

1. Zuerst jede Invariante testbar machen, dann optimieren. Leistungsarbeit (Cache-Strategien, Block-Größen) beginnt frühestens in P3 mit Messwerten aus dem Benchmark-Harness.
2. In jeder Phase wird der Crash-Sweep um die neuen Fs-Operationen erweitert, nie nachgezogen.
3. Formatänderungen sind bis P5 erlaubt; ab P5 gilt `format_version` mit Migrationspflicht, weil Checkpoints Altformate enthalten können.

## 9. Restrisiken, offene Entscheidungen und Nicht-Ziele

Der Kern kann nur Bytes löschen, die er kontrolliert. Diese Tabelle benennt, was bleibt und wer dafür zuständig ist. Sie ist die Grundlage für ehrliche Aussagen gegenüber Aufsicht und Betroffenen; eine Rechtsberatung ersetzt sie nicht.

### Restexpositionen

| Bereich | Restrisiko | Gegenmaßnahme / Zuständigkeit |
| --- | --- | --- |
| Prozessspeicher | Klartext in Heap, Core-Dumps, Swap, Hibernation | `Zeroizing`, Cache-Drop beim Gate; `MemLock`/`DONTDUMP` im Adapter; Betreiber: Swap verschlüsseln, Core-Dumps aus |
| OS-Page-Cache | Gelöschte Seiten können kurz im Cache liegen | Cluster-Bytes liegen ohnehin nur als Ciphertext auf dem Datenträger; optional `O_DIRECT`-Adapter |
| SSD / Wear-Leveling | Überschriebene Blöcke können in der FTL überleben | Deshalb ist G1 (Schlüsselvernichtung) die tragende Garantie; G2 gilt für Dateien, nicht für das Medium |
| COW-/Snapshot-Dateisysteme, Volume-Snapshots | Frühere Blockversionen bleiben bestehen | Nur Ciphertext; Betreiber muss Snapshots ohne Schlüssel führen und Aufbewahrungsdauer begrenzen |
| Externe Backups | Alte Ciphertext-Kopien | Lesbar nur mit Schlüsseln. Schlüssel gehören nicht ins Backup; Ledger-Tombstones werden beim Restore erneut angewendet |
| KMS-Karenzzeit | Schlüssel bis zum Fristende wiederherstellbar | Ticket führt `KeysScheduled` getrennt von `KeysDestroyed`; Nachweis nennt die Stärke |
| Schlüsselkopien | Escrow, HSM-Backups | Betreiberentscheidung; muss in die Destroy-Policy eingerechnet werden |
| Längen-Seitenkanal | Sektions- und Frame-Längen sind sichtbar | optionales Padding auf Bucket-Größen (Kosten: Platz) |
| ClusterId | Pseudonym, im Directory sichtbar | Zufällig, nie ableitbar; verschwindet beim Scrub; Ledger hält nur `tomb_id` |
| Obere Schichten | Prompt-Caches, Embedding-Kopien, Logs, Fine-Tuning-Daten, Modellgewichte | **Nicht** im Kern lösbar. Nur Daten, die über `Cluster-Links` und `DerivedSegment` angebunden sind, werden mit gelöscht |

### Offene Entscheidungen

| Entscheidung | Optionen | Vorschlag |
| --- | --- | --- |
| AEAD | XChaCha20-Poly1305 vs. AES-256-GCM-SIV | XChaCha20-Poly1305 (zufällige 192-Bit-Nonce ohne Zählerzustand) |
| Anker-Backend zuerst | Datei (Test), TPM-NV, Object-Lock/Transparency-Log | Datei-Referenz in P5, produktiv eine externe Option je Betreiber |
| Lease-Dauer von `View` | 10 s bis 5 min | 60 s Default, konfigurierbar; begrenzt Purge-Fristen |
| `purge_deadline` Default | Stunden bis Tage | Betreiberentscheidung, Metriken für Überschreitung |
| Block-/Run-Größe | 16-32 KiB, 64-256 MiB | per Benchmark in P3/P4 festlegen |
| Commit-Fenster | sofort vs. wenige Millisekunden | adaptiv: sofort bei geringer Last, Gruppierung bei Last |
| `ArcSwap` vs. `RwLock<Arc>` | Dependency vs. Eigenbau | `ArcSwap`, solange Review der Dependency akzeptiert wird |

### Explizite Nicht-Ziele

- Keine verteilte Replikation, kein Konsens (2PC, Raft) im Kern. Replikation und Failover liegen außerhalb und müssen Löschnachweise über das Ledger und den Anker einbinden.
- Keine eigene Kryptografie: ausschließlich etablierte Primitive hinter `Cipher`, `Mac` und `Kdf`.
- Kein Tokenizer, kein Embedding-Modell, kein ANN-Algorithmus im Kern. `weight` und `lanes` sind opake Zahlen, die die obere Schicht liefert.
