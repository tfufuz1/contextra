# SPEC-07 — Durability, Recovery, GC, Sicherheit und Betrieb

**Version 2.1 · November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Storage](SPEC-03-STORAGE-ENGINE.md) · [Versionierung](SPEC-04-VERSIONING.md)

---

## 1. Write-Ahead Log (`contextra-wal`)

**Ein gemeinsamer WAL** für alle Branches eines Repositories (`branch_id` ist Teil jedes Frames). Damit genügt **ein einziger `fdatasync`** pro Group-Commit-Batch unabhängig von der Branch-Zahl (kein Mehr-Store-2PC!).

### 1.1 WAL-Segment (`.cxw`) & Frame-Format

Vorallokierte 64-MiB-Segmente (Rotation bei 64 MiB oder 60 s):

```
WAL Frame (little-endian, 8-Byte-aligned):
┌─────────────────────────────────────────────────────────────┐
│ Frame Header (56 Bytes)                                     │
│   magic[2]=0xCF01 | frame_len[4] | lsn_seq[8] | branch_id[4]│
│   commit_ts_hlc[8] | durability[1] | frame_kind[1]          │
│   intent_len[2] | payload_len[4] | idem_key_len[2]          │
│   payload_digest[16] (BLAKE3-128 über Payload für Idem!)    │
│   header_crc[4] | reserved[2]                               │
├─────────────────────────────────────────────────────────────┤
│ IdempotencyKey (idem_key_len B) + Intent (intent_len B)     │
├─────────────────────────────────────────────────────────────┤
│ Payload (rkyv-serialisiert, nach frame_kind):               │
│   CommitTx { mutations: PutContent | PutMeta | RetireCell   │
│                        PutEdge | DeleteEdge | CatalogOp }   │
│   BranchOp { Create | Savepoint | RollbackTo | Discard }    │
│   WorkOp   { Enqueue | Complete | PermanentFail }           │
├─────────────────────────────────────────────────────────────┤
│ Frame CRC32c (4 Bytes über gesamten Frame)                  │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Group Commit & Fail-Stop-Regel („fsyncgate")

- **Group Commit (`durable` Default):** Leader/Follower bündelt über alle Branches bis `max_wait = 1 ms` oder `64 Commits` → sequentielles `write()` → **ein `fdatasync()`** → Wakeup aller wartenden Transaktionen.
- **`strict` Mode:** Sofortiges `fdatasync()` ohne Bündelwartezeit + Verzeichnis-`fsync` bei neuen Refs.
- **Fail-Stop („fsyncgate"):** Scheitert `fdatasync()` auf WAL oder Manifest, markiert die Engine die Instanz sofort als **`repo_poisoned` (ReadOnly/Failed)**, wiederholt den Sync auf derselben Datei **niemals** und erfordert geordnetes Schließen + Recovery (kein Silent Data Corruption durch verschobene Kernel-Dirty-Pages).

### 1.3 WAL-Bereinigung

Ein WAL-Segment wird recycelt/gelöscht, wenn alle enthaltenen `lsn_seq` kleiner/gleich `manifest.checkpoint_lsn` sind (d. h. vollständig durch `Fold` in dauerhafte Packs überführt), es nicht das aktive Schreibsegment ist und die konfigurierbare Grace-Periode (Default 1 h für `G3`-Zeitreise innerhalb Fold) abgelaufen ist.

---

## 2. Manifest-Log und `CURRENT`-Superblock

Das Manifest (`manifest/MANIFEST-*.mlog`) ist ein append-only Struktur-Log (CRC32c + `prev_entry_blake3` gegen Bit-Rot/Torn-Tails; **kein** kryptographischer Löschbeweis). Der aktive Pfad und Tail-Offset stehen in der doppelt gepufferten Datei `CURRENT` (atomar per `write + fdatasync + rename + dir_sync` aktualisiert).

### 2.1 Manifest-Entries

- `AddPack { pack_id, path, extent_count, byte_size }` / `RemovePack { pack_id }`
- `UpdateRefs { branches: Vec<(BranchId, Name, HeadCommitId, ForkBase)> }` (CAS + Reflog-Kette)
- `UpdateObjectLocator { runs }` / `UpdateCellLocator { runs }`
- `UpdateIndexManifest { branch_id, index_manifest_ref }` *(separat vom `RootSet`!)*
- `GidCounter { next_gid: u64 }` *(globaler monotoner Zähler für `Gid` & Handle-Sequenz)*
- `CheckpointLsn { lsn_seq: u64 }`
- `SidecarCheckpoint { idempotency_snapshot, work_queue_snapshot, access_stats_snapshot }`

**Rotation:** Bei > 16 MiB oder > 10 000 Entries beginnt ein neues Manifest mit vollständigem `Checkpoint`-Entry; danach atomares `CURRENT`-Update.

---

## 3. Crash-Recovery

```
1. CURRENT-Superblock laden (A/B-Buffer mit höherer gültiger Generation + CRC).
2. Aktives Manifest lesen bis zum letzten vollständigen, CRC-validen Entry.
3. Packs, Locators, Branch-Refs, IndexManifests, GidCounter und Sidecar-Checkpoints laden.
   (Fehlen Locator-Runs → automatischer O(Packs)-Neuaufbau aus Pack-Footern.)
4. WAL-Segmente scannen für Frames mit lsn_seq > manifest.checkpoint_lsn:
   - Torn Tail (ungültige Länge / CRC / nicht-monotone Seq) verwerfen → Präfix-Konsistenz (INV-3).
5. Gültige WAL-Frames idempotent wiedergeben:
   - Rekonstruiere Branch-MemDeltas + synchrone In-Memory-Index-Deltas.
   - Rekonstruiere nicht-versioniertes IdempotencyTable aus WAL-IdempotencyKeys.
   - Setze WorkQueue-Tasks mit Status `Leased` zurück auf `Pending` (keine Zombie-Leases!).
6. Fehlen abgeleitete IndexManifest-Segmente oder hinkt watermark_seq hinterher:
   - Sofortige Betriebsbereitschaft via Flat-Fallback; asynchrone Index-Builds in Work Queue einreihen.
```

---

## 4. Garbage Collection (SATB + Remark)

### 4.1 Erreichbarkeits-Wurzeln & Protokoll

- **Roots:** Alle aktiven `Branch`-Heads, benannte `Tags`/`Checkpoints`, das konfigurierbare `Reflog`-Fenster (Standard 1000 Einträge bzw. `branches_days`), aktive Transaktionen/Savepoints, `Snapshot`-Leases (Default **300 s** TTL — verhindert, dass vergessene Leser GC dauerhaft blockieren!) sowie In-Flight-Schreibobjekte.
- **5 Phasen (`SATB + Remark`):**
  1. **ROOTS:** Sammle alle Wurzel-`CommitId`s und aktiven `IndexManifestRef`s zum Snapshot `S0`.
  2. **MARK:** Traversiere `Commit → Parents`, `RootSet → Prolly-Knoten → CellContent/CellMeta/Blobs`, sowie aktive `IndexManifest → Segmente`.
  3. **CANDIDATES:** Unmarkierte Objekte in Packs älter als `S0`, deren `retire`-Alter den `retention_horizon` überschritten hat (bzw. verwaiste Absturz-Packs).
  4. **REMARK:** Synchronisiere alle seit `S0` eingegangenen Commits und Deduplizierungs-Wiederverwendungen (`Write-Barrier` schützt alle jungen/wiederbenutzten `ChunkId`s).
  5. **SWEEP (nach Grace-Periode 1 h):** Vollständig tote Packs sofort löschen; Packs mit `< 50 %` Live-Anteil per **Copy-Forward** in neue Packs/Extents verdichten (optional kombiniert mit Affinity Re-Layout) und nur Locator aktualisieren.

---

## 5. Nebenläufigkeits- & Speichermodell

### 5.1 Speicherbudgets (`MemoryBudget` — INV-11)

Strikte Obergrenze beim Öffnen verteilt sich auf disjunkte Kontingente:
- **`fixed` (25 %):** Manifest, Locator-Hot-Set, FST-Dictionaries, quantisierte Codes, CSR-Offsets, Roaring-ACL-Bitmaps.
- **`memtable` (15 %):** `MemDelta` + `vec_delta`/`fts_delta` aller Branches.
- **`block_cache` (40 %):** Partitionierter Buffer Pool (W-TinyLFU, getrennt für Prolly-Knoten, Extent-Hot-Heads, Cold-Tails, Vamana-Sektoren, FTS-Postings).
- **`query` (10 %):** Arbeitspeicher für aktive Abfragen, Fusion, Packer, Reranker.
- **`jobs` (10 %):** Fold, Compaction, Vektor-/FTS-Segment-Builds, Ingest.

**Backpressure-Stufen:** `Normal` → `Elevated` (> 80 % Auslastung: Ingest-Drosselung, früherer Fold, Low-Prio-Index-Builds pausieren) → `Critical` (> 95 %: neue Schreibtransaktionen erhalten `backpressure { retry_after_ms }`, nie OOM oder stilles Verwerfen!).

### 5.2 I/O & Locking

- **Lesen:** Wait-free Head-Lookup via `ArcSwap<BranchState>`, epochenbasierte Speicherfreigabe (`crossbeam-epoch`), zero-copy `rkyv` mit einmaliger `bytecheck`-Validierung beim Cache-Load.
- **Schreiben:** `BranchSequencer` pro Branch; `loom`-geprüfte Atomics und Lock-Hierarchie (`Manifest → BranchMemDelta → BufferPoolShard`).
- **Kein `mmap` im Produktpfad:** Portables `pread`/`pwrite` als Standard, Linux `io_uring` + `O_DIRECT` als opt-in Feature hinter `IoBackend`-Trait.

---

## 6. Retention & Tiering (`T0 ↔ T1 ↔ T2`)

- **Branch-Retention:** `KeepAll`, `KeepLast(N)`, `KeepDays(D)`, `KeepMinMax { min, max_days }`, kombiniert mit `undo_window` (Default 24 h) und optionalem Admin-`ShallowTruncate` (Erzeugen eines wurzellosen Snapshot-Commits zur Kappung tiefer Historie).
- **Storage-Tiering (`ObjectStoreTier` für S3/MinIO/R2 ab Phase 4):** `T0` RAM (< 1 µs) ↔ `T1` NVMe (< 100 µs) ↔ `T2` S3/Objektspeicher (< 500 ms für alte Cold-Tails & archivierte Versionen).

---

## 7. Sicherheit: Trust, Taint, Capabilities, Egress & Audit

### 7.1 Orthogonale Trust & Taint-Propagierung

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum TrustTier {
    System        = 5, // Engine-Katalog & Systemregeln
    Verified      = 4, // Kryptographisch / redaktionell verifiziert (Mensch)
    Curated       = 3, // Kuratierte interne Wissensbasis / User-Eingabe
    ToolVerified  = 2, // Deterministisches/signiertes Tool
    AgentInferred = 1, // LLM-Extraktion / Schlussfolgerung
    Untrusted     = 0, // Externe Quelle (Web, E-Mail) oder Hypothesis
}

bitflags::bitflags! {
    pub struct TaintFlags: u16 {
        const EXTERNAL          = 0b0001; // Stammt aus externer/ungeprüfter Quelle
        const INJECTION_SUSPECT = 0b0010; // Heuristic/Scanner-Treffer beim Ingest
        const UNVERIFIED_TOOL   = 0b0100; // Nicht-verifizierter Tool-Output
        const PII_SUSPECT       = 0b1000; // Für Ausgabe-/Egress-Policies markiert (NICHT Löschkette!)
    }
}
```

- **Ableitungsregeln:** `derived_trust = min(source_trusts)` **und** `derived_taint = ⋃ source_taints`.
- Eine Zusammenfassung einer Web-Seite bleibt zwingend `Untrusted | EXTERNAL` und wird beim Rendering gekapselt ([SPEC-06](SPEC-06-CONTEXT-ENGINE.md) §2.2). Trust-Promotion erfordert Gate `G7` / `Operations::Admin`.

### 7.2 Attenuierbare Capabilities

```rust
pub struct Capability {
    pub id: [u8; 16],
    pub caveats: Vec<Caveat>, // Namespaces, BranchPatterns ("sess/a1/*"), Ops (Read|Write|Fork|Merge|ProtectedBranchWrite|Admin),
                              // MaxTrust, Ttl, TokenBudget, MaxBudgetPerRecall
    pub hmac: [u8; 32],       // HMAC-BLAKE3 über Kette
}
```

Agenten erhalten standardmäßig **keine** `ProtectedBranchWrite`-Berechtigung auf `main`. Ein Hauptagent kann Subagenten jederzeit strenger attenuierte Tokens ausstellen (`Fail-closed`).

### 7.3 Egress-Guard (Cloud-LLMs)

Vor jeder Weitergabe von Pack-Inhalten an externe Cloud-Adapter (`Llm`/`Judge` mit `transport = Cloud`) erzwingt das Typestate `GuardedPayload<Unsanitized → Sanitized>`:
- Policy-Check (`LocalOnly | CloudAllowed { namespaces, min_trust, allow_tainted }`),
- Ausschluss von `PII_SUSPECT` / Secret-Patterns (API-Keys, Tokens, IBANs),
- Raten-/Bulk-Exfiltrationslimits pro Session, Timeout Fail-Closed (`egress_blocked`). Lokale Adapter (Candle, Ollama, ONNX) bleiben lokal.

### 7.4 Schlanke Betriebsprotokolle & `ctx verify`

- Strukturierte JSON-Lines (`tracing`) für Auth/Capabilities, Schreib-Commits, Merge-Gates, Trust-Promotions und Egress-Entscheidungen (keine kryptographische Audit-Chain/AVV-Generatoren).
- **`ctx verify` & `ctx scrub`:** Prüft `RootSet`-Merkle-Integrität, `CellLocator`/`ObjectLocator`-Konsistenz, `Gid ↔ CellId`-Bijektion, Watermarks und Block-CRCs. Defekte Index-Segmente werden automatisch verworfen & neu gebaut; defekte Truth-Segmente melden `pack_corrupted` mit Restore-Hinweis.

---

*Ende SPEC-07. Weiter: [SPEC-08 APIs und Fehlerkatalog](SPEC-08-API.md)*
