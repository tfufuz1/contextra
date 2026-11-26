# SPEC-03 — Storage Engine: Packs, PAX-Extents, Locator und Verschlüsselung

**Version 2.1 · November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Datenmodell](SPEC-02-DATA-MODEL.md) · [Versionierung](SPEC-04-VERSIONING.md) · [Durability](SPEC-07-DURABILITY.md)

---

## 1. Objektmodell

### 1.1 Chunks und Objekte

Jedes persistente Datenobjekt in Contextra ist unveränderlich und wird in `.cxp`-Pack-Dateien gespeichert.

```rust
#[repr(u8)]
pub enum ChunkKind {
    ProllyLeaf     = 0,  // Blattknoten im Truth-Prolly-Tree
    ProllyInternal = 1,  // Innerer Knoten im Truth-Prolly-Tree (Hints NICHT im Hash!)
    ExtentBlock    = 2,  // Spaltenorientierter PAX-Extent (Hot-Head + Cold-Tail von N Cells)
    LargeBlob      = 3,  // Out-of-line Payload (> 2 KiB)
    VecSegment     = 4,  // Abgeleitetes Vektorsegment (Vamana/Flat)
    FtsSegment     = 5,  // Abgeleitetes FTS-Segment (FST + Postings)
    GraphSegment   = 6,  // Abgeleitetes Graph-CSR-Segment
    BitmapSegment  = 7,  // Vorkompilierte Roaring-Filter-Bitmaps
    IndexManifest  = 8,  // IndexManifest-Objekt
    CommitObj      = 9,  // Commit-Objekt
    Dictionary     = 10, // Trainiertes zstd-Wörterbuch pro Namespace
}
```

**Hash-Regel (`ChunkId`):**
- Unverschlüsselt: `ChunkId = BLAKE3(canonical_plaintext_bytes)`.
- Verschlüsselt: `ChunkId = BLAKE3::keyed_hash(repo_hash_key, canonical_plaintext_bytes)` (siehe §8.3 — Hash über Klartext mit Repo-Key, niemals über Nonce-Ciphertext!).

### 1.2 PAX-Extent (I/O-Einheit)

Ein **Extent** ist die physische I/O-Einheit (256 KiB – 1 MiB, Ziel 512 KiB, 4-KiB-aligned für `O_DIRECT`). Er fasst bis zu **N** (typisch 128–1024) logisch gruppierte Cells **spaltenorientiert (PAX)** mit strikter Trennung in **Hot Head** und **Cold Tail** zusammen.

### 1.3 Pack-Datei (`.cxp`)

Append-only beim Schreiben, danach strikt unveränderlich (64 MiB – 1 GiB, Ziel 256 MiB).

---

## 2. Pack- und Extent-Dateiformat (löst Reibungspunkt 3)

```
Pack-Datei (.cxp)
┌─────────────────────────────────────────────────────────────┐
│ Pack Header (64 Bytes, Offset 0)                            │
├─────────────────────────────────────────────────────────────┤
│ Extent 0 (4-KiB-aligned) | Extent 1 | ... | Extent N        │
├─────────────────────────────────────────────────────────────┤
│ Stand-Alone Blobs / Index-Segment-Blöcke / Prolly-Knoten    │
├─────────────────────────────────────────────────────────────┤
│ Pack Footer (Extent-Directory, Object-Index, Cell-Index,    │
│              Ribbon-Filter, Footer Header 32B)              │
└─────────────────────────────────────────────────────────────┘
```

### 2.1 Pack Header (64 Bytes, little-endian)

```
Offset  Länge  Feld              Beschreibung
  0       4    magic             b"CXP\x01"
  4       2    format_version    u16 LE = 1
  6       2    compat_flags      u16 LE (unbekannte Bits werden ignoriert)
  8       4    incompat_flags    u32 LE (Bit 0: encrypted, Bit 1: compressed; unbekannt = fail-closed)
 12       8    pack_id           u64 LE (monoton pro Repo)
 20      32    repo_seed         [u8; 32] (Seed für Prolly-Boundary & Handle-MAC)
 52       4    extent_count      u32 LE
 56       4    footer_offset_4k  u32 LE (Footer-Start ab Dateiende in 4-KiB-Blöcken)
 60       4    header_crc        CRC32c über Bytes 0–59
```

### 2.2 Extent-Layout: Hot-Head / Cold-Tail (PAX mit RowIndex-Adressierung)

Innerhalb eines Extents werden bis zu `cell_count` (≤ 65 535, adressiert via `RowIndex: u16`) Cells spaltenorientiert abgelegt. Jede variable Spalte trägt ein `u32[cell_count + 1]`-Offset-Array für O(1)-Zugriff per `RowIndex`:

```
Extent (z.B. 512 KiB, 4-KiB-aligned)
┌────────────────────────────────────────────────────────────────────────────┐
│ Extent Header (128 Bytes)                                                  │
│   magic[4]="CXE1" | version[2] | flags[2] | extent_no[4] | extent_len[4]   │
│   cell_count[4] | hot_end[4] | cold_start[4] | lane_dir_offset[4]          │
│   lane_count[2] | compression[1] | encryption[1] | dict_chunk_id[32]       │
│   nonce_gen[8] | padding[56]                                               │
├────────────────────────────────────────────────────────────────────────────┤
│ Lane Directory (lane_count × 16 Bytes)                                     │
│   lane_id[2] | offset[4] | length[4] | codec[1] | flags[1] | crc32c[4]     │
├────────────────────────────────────────────────────────────────────────────┤
│ HOT HEAD (Offset 128 + lane_dir_size ... hot_end) — ein Prefetch/I/O       │
│   - gid[u64; N]          (sortiert für schnelle Range-Translation!)        │
│   - meta.fixed[N]        (kind, trust, taint, belief, valid_from/to, acl)  │
│   - attr.bloom[N]        (8 B Bloom pro Cell für Attribute/Tags)           │
│   - edge.hint[N]         (4 B Grad-/Kantentyp-Bitmaske, NICHT im Hash)     │
│   - tokcount[N × T]      (u32 je registriertem Tokenizer)                  │
│   - zoom.L2 (offsets[N+1] + UTF-8 Block)                                   │
│   - zoom.L1 (offsets[N+1] + zstd Block)                                    │
│   - emb.q (N × quantized_dim Bytes, z.B. SQ8/binär/Matryoshka-Präfix)     │
├────────────────────────────────────────────────────────────────────────────┤
│ COLD TAIL (cold_start ... extent_len - 4) — nur für finale Gewinner        │
│   - raw (offsets[N+1] + zstd-Blöcke mit Namespace-Wörterbuch;              │
│          bei reinen Meta-Updates optional leer, Verweis via content_hash)  │
│   - tok.ids (offsets[N+1] + delta+bitpacked Token-IDs)                     │
│   - emb.full (N × dim × f16, 4-KiB-aligned für SIMD/O_DIRECT)              │
│   - attrs.ext (rkyv-Archive für seltene/große Attribute)                   │
├────────────────────────────────────────────────────────────────────────────┤
│ Extent Footer CRC (4 Bytes)                                                │
│   CRC32c über gesamten Extent + optional AEAD-Auth-Tag (16 Bytes)          │
└────────────────────────────────────────────────────────────────────────────┘
```

**I/O-Muster:**
1. **Ranking-Read:** Liest die ersten 32–64 KiB (`hot_end`) — liefert Metadaten, Token-Counts, Zoom-L2/L1 und quantisierte Embeddings aller *N* Cells im Extent in einem I/O.
2. **Winner-Read:** Gezielter Offset-Read auf `cold_tail` über `RowIndex` nur für die selektierten Gewinner-Cells (typisch ≤ 12 pro Recall).
3. **Reine Meta-Updates:** Wenn nur `CellMeta` (z. B. Tags, `valid_to`) geändert wird, schreibt der Fold nur ein leichtgewichtiges Meta-Extent (ohne `raw`/`emb.full` zu duplizieren); der Cold-Read löst `raw` über `content_hash` im `RawLocator` auf. Bei der nächsten Compaction/Re-Layout werden sie wieder ko-lokalisiert.

### 2.3 Pack Footer

```
Pack Footer:
┌─────────────────────────────────────────────────────────────┐
│ Extent Directory (extent_count × 32 B):                     │
│   extent_offset[8] | extent_len[4] | cell_count[4] |        │
│   min_gid[8] | max_gid[8]                                   │
├─────────────────────────────────────────────────────────────┤
│ Object Index (sorted ChunkId → PackSpan):                   │
│   chunk_id[32] | kind[1] | offset[8] | len[4]               │
├─────────────────────────────────────────────────────────────┤
│ Cell Index (sorted CellVerKey → ExtentRowAddr):             │
│   cell_ver_key[32] | extent_no[4] | row_idx[2] | flags[2]   │
├─────────────────────────────────────────────────────────────┤
│ Ribbon Filter (über Object + Cell Keys, 0.1 % FPR)          │
├─────────────────────────────────────────────────────────────┤
│ Footer Header (32 B): magic=b"CXPF", Offsets, BLAKE3, CRC32c│
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Zweistufiger Locator-Index (löst Reibungspunkt 3)

Statt für jede einzelne Lane jeder Cell einen eigenen Locator-Eintrag anzulegen (was bei 1M Cells × 6 Lanes zu 6M Einträgen führt), trennt Contextra sauber zwischen **Objekt-Adressierung** und **Zellzeilen-Adressierung**:

### 3.1 `ObjectLocator` (für Prolly-Knoten, Segmente, Blobs, Dictionaries)

`ChunkId ([u8; 32]) → PackSpan { pack_id: u64, offset: u64, len: u32 }`
- Typische Dichte: ~1 Eintrag pro 4–16 KiB Prolly-Knoten / Index-Block.

### 3.2 `CellLocator` (für PAX-Extents)

1. **`MetaRowLocator`:** `cell_ver_key = BLAKE3(content_hash || meta_hash) → ExtentRowAddr { pack_id: u64, extent_no: u32, row_idx: u16 }` (14–16 Bytes Payload — **ein einziger Eintrag pro Cell** für alle Hot-Head- und Cold-Tail-Lanes!).
2. **`ContentRowLocator` (Fallback bei entkoppeltem Meta-Update):** `content_hash → ExtentRowAddr | BlobSpan` — liefert `raw`, `tok.ids`, `emb.full` falls ein reines Meta-Extent keinen eigenen Cold-Tail trägt.

**Struktur & Lookup:**
- Size-tiered sorted Runs (`locator/LOC-*.cxl`) mit **Ribbon-Filter (0.1 % FPR)** pro Run; die obersten 2 Runs bleiben im RAM (`T0`).
- **Parent-Hints (unhashed):** Prolly-Tree-Leseoperationen können einen flüchtigen In-Memory-/Sidecar-Hint `(PackId, ExtentNo)` nutzen; trifft der Hint nicht, greift der Locator-Lookup. Hints gehen **niemals** in Knoten-Hashes ein!
- **Recovery:** Beide Locators sind zu 100 % aus den Pack-Footern rekonstruierbar (`O(Pack-Anzahl)`, typisch < 1 s).

---

## 4. Gelerntes Layout (Affinity Re-Layout)

### 4.1 Prinzip

Wer gemeinsam abgerufen wird (`Access-Trace`), soll physisch im selben Extent liegen, um Extent-Zugriffe pro Recall zu minimieren. Re-Layout ist **als Hintergrund-Optimierung jederzeit abbrechbar** (Standard-Hot-Head/Cold-Tail wirkt ab Phase 1; Affinity-Clustering ab Phase 3 bzw. nach Benchmark-Nachweis ≥ 20 % I/O-Reduktion).

### 4.2 Datenquellen & Verfahren

| Signal | Gewicht | Quelle |
|---|---|---|
| Co-Retrieval & Co-Expand | 1.0 | `AccessStats`/Trace-Sampling (10 % Recalls, 100 % Expands) |
| Struktur-Prior | 0.8 | Gleiches Dokument / gleiche Episode |
| Semantik / Zeit | 0.5 / 0.3 | Vektor-Cluster / Entstehungsnähe |

**Ablauf:**
1. Co-Access-Graph aufbauen, mit inkrementeller Label-Propagation (oder Multilevel-Partitionierung < 1000 Knoten) in Extent-große Gruppen schneiden.
2. **Neue PAX-Extents schreiben** (alle Lanes `meta` bis `emb.full` wieder gemeinsam gepackt, `raw_ref` aufgelöst).
3. Erst nach vollständigem `fdatasync` des neuen Packs: **nur `CellLocator` (`MetaRowLocator` + `ContentRowLocator`) im Manifest atomar umbiegen**.
4. **Invariante:** Alle `ChunkId`s, `content_hash`, `meta_hash`, `RootSet`-Wurzeln und `CommitId`s bleiben **zu 100 % unverändert**!
5. **5 % Exploration:** 5 % der Recalls streuen leichte Diversität ein, um Feedback-Verfestigung zu vermeiden.

---

## 5. Partitionierter Buffer Pool & I/O-Pfad (kein `mmap`)

### 5.1 Buffer Pool (`contextra-store`)

Da alle Packs, Prolly-Knoten und Index-Segmente **unveränderlich** sind, ist der Buffer Pool ein reiner **Read-Cache ohne Dirty-Pages**:
- **Seitengrößen:** 4 KiB (Vektor-Sektoren, Hot-Head-Reads), 16 KiB (Prolly-Knoten, Packs/Cold-Tail-Blöcke).
- **Eviction:** W-TinyLFU oder CLOCK-Pro mit Pin-Zählern je Frame (Zero-Copy `rkyv`-Validierung via `bytecheck` erfolgt exakt einmal beim Laden des Frames in den Pool!).
- **Strikte Teilbudgets:** Getrennte Kontingente für `ProllyInternal` (gepinnt), `ProllyLeaf`, `ExtentHotHead`, `IndexVec`, `IndexFts`, `ColdTail` — ein großer Dokumenten-Scan verdrängt niemals heiße Graph-/Vektor-Strukturen.

### 5.2 I/O-Backend

- **Baseline (alle OS):** `pread`/`pwrite` über dedizierten Thread-Pool.
- **Linux High-Perf (opt-in Feature `io-uring`):** `io_uring` mit `O_DIRECT`-aligned Puffern (4 KiB) für Cold-Tail- und Vamana-Sektor-Reads.
- **`mmap` ist im Produktpfad unzulässig:** Verhindert unkontrolliertes OS-Paging, TLB-Shootdowns und unauffangbare `SIGBUS`-Signale bei Medienfehlern (`mmap` höchstens in Entwicklungs-/Offline-Tools).

---

## 6. Kompression & Wörterbücher

| Lane / Objekt | Verfahren | Level | Details |
|---|---|---|---|
| Text (`raw`, `zoom.L1`) | `zstd` mit Namespace-Wörterbuch | 3 (online), 19 (Compaction) | Wörterbuch max. 112 KiB, `dict_chunk_id` im Extent-Header |
| `tok.ids` | Delta + SIMD-Bitpacking + `zstd` | 3 | Bit-Breite = `ceil(log2(vocab_size))` |
| `emb.q` (Hot Head) | SQ8 / binär / RaBitQ | — | Unkomprimiert für direktes SIMD-Scoring |
| `emb.full` (Cold Tail) | `f16` (Default) oder `f32` (`zstd:1`) | — | 4-KiB-aligned für direkten Rerank-Read |
| `meta` & `attrs` | Spalten-Dictionary + RLE + FoR | — | Zero-Copy-Filterung |

Wörterbuch-Training läuft alle 10 000 neue Cells pro Namespace; alte Extents bleiben über ihre `dict_chunk_id` lesbar.

---

## 7. Verzeichnisstruktur auf Disk

```
<repo-root>/
├── contextra.toml               # Statische Start-Konfiguration
├── lock                         # flock(LOCK_EX | LOCK_NB) / LockFileEx (Single-Process)
├── manifest/
│   ├── MANIFEST-000001.mlog     # Append-only Manifest-Log (CRC32c + Prev-Hash)
│   └── CURRENT                  # Doppelt gepufferter atomarer Superblock
├── wal/
│   └── WAL-000001.cxw           # Gemeinsamer WAL für alle Branches (vorallokiert, 64 MiB)
├── packs/
│   └── PACK-000001.cxp          # Unveränderliche Pack-Dateien (Truth, Extents, Indizes)
├── locator/
│   ├── LOC-OBJ-000001.cxl       # ObjectLocator sorted runs
│   └── LOC-CELL-000001.cxl      # CellLocator sorted runs
├── sidecar/                     # NICHT Teil des versionierten RootSet!
│   ├── idempotency.cxs          # WAL-gestütztes Idempotenz-Journal (mit TTL)
│   ├── work_queue.cxs           # Persistente Job-Tabelle (Leases nicht versioniert)
│   └── access_stats.cxs         # Eventual-consistent Zugriffsstatistiken
├── dictionaries/                # zstd-Wörterbücher (auch als Chunk referenziert)
└── tmp/                         # Ziel vor atomarem write + fdatasync + rename + dirsync
```

---

## 8. Verschlüsselung At-Rest (optional, KEK/DEK + Keyed Klartext-Hash)

### 8.1 Schlüsselhierarchie

```
KEK (Master Key / Passphrase via Argon2id / KMS-Hook über CipherProvider, NIE auf Disk)
 └── wraps DEK (256-bit Data Encryption Key, pro Store, in Header gewrappt)
      ├── repo_hash_key = HKDF-BLAKE3(DEK, repo_seed, "contextra-cas-hash-v1")
      ├── handle_mac_key= HKDF-BLAKE3(repo_seed, "contextra-handle-mac-v1")
      └── pack_enc_key  = HKDF-SHA256(DEK, pack_id, "contextra-pack-aead-v1")
```

### 8.2 AEAD-Verfahren & Schlüsselrotation

- **Algorithmen:** `AES-256-GCM` (Standard mit AES-NI) oder `XChaCha20-Poly1305` (Software-Fallback). Nonce eindeutig abgeleitet aus `(pack_id, extent_no, block_no, generation)`.
- **O(1)-KEK-Rotation:** Master-Key-Wechsel re-wrappt nur den DEK im Superblock (sofort fertig).
- **DEK-Rotation:** Hintergrund-Compaction schreibt neue Packs mit neuem `dek_id`; alte Packs werden nach Verification gelöscht.

### 8.3 Keyed ObjectIds über Klartext (korrigiert Nonce-Bug)

**Regel:** Bei verschlüsselten Repositories gilt:
`ChunkId = BLAKE3::keyed_hash(repo_hash_key, canonical_plaintext_bytes)`.

- **Warum über Klartext (mit `repo_hash_key`) statt über Ciphertext?**
  1. Verhindert Cross-Tenant-/Cross-Repo-Existenz-Orakel (da andere Repos einen anderen `repo_hash_key` haben).
  2. Gleichzeitig bleiben **Content-Addressing, Deduplizierung im Derivation Cache, History-Unabhängigkeit des Prolly-Trees und KEK/DEK-Rotation vollständig erhalten**, weil zwei identische Klartexte trotz unterschiedlicher AEAD-Nonces denselben `ChunkId` erhalten!

---

*Ende SPEC-03. Weiter: [SPEC-04 Versionierung](SPEC-04-VERSIONING.md)*
