# SPEC-07 — Durability, Recovery, Nebenläufigkeit und Betrieb

**Version 2.0 · November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Storage](SPEC-03-STORAGE-ENGINE.md) · [Versionierung](SPEC-04-VERSIONING.md)

---

## 1. WAL (Write-Ahead Log)

### 1.1 WAL-Segment-Dateiformat

```
WAL-Segment (.cxw)
┌──────────────────────────────────────────────────┐
│ Segment Header (64 Bytes)                        │
│   magic[4] = b"CXW\x01"                         │
│   format_version[2] = u16 LE (1)                 │
│   segment_id[8] = u64 LE (monoton steigend)      │
│   repo_id[32] = [u8; 32] (Repository-Hash)       │
│   created_at[8] = u64 LE (Millisekunden)         │
│   flags[2] = u16 LE                              │
│   header_crc[4] = CRC32c                         │
│   padding[4]                                     │
├──────────────────────────────────────────────────┤
│ Frame 0                                          │
├──────────────────────────────────────────────────┤
│ Frame 1                                          │
├──────────────────────────────────────────────────┤
│ ...                                              │
└──────────────────────────────────────────────────┘
```

**Segment-Rotation:** Neues Segment bei Größe > 64 MiB oder alle 60 s (konfigurierbar).

### 1.2 WAL-Frame-Format

```
WAL Frame:
┌──────────────────────────────────────────────────┐
│ Frame Header (48 Bytes)                          │
│   frame_magic[2] = 0xCF01                        │
│   frame_len[4] = u32 LE (Gesamtlänge inkl. Header) │
│   seq[8] = u64 LE (globale Sequenznummer)        │
│   branch_id[4] = u32 LE                          │
│   commit_ts[8] = u64 LE (HLC)                    │
│   intent_len[2] = u16 LE                         │
│   changeset_len[4] = u32 LE                      │
│   idempotency_key_hash[8] = u64 LE (BLAKE3-64)  │
│   header_blake3[8] = BLAKE3-64 über Header       │
├──────────────────────────────────────────────────┤
│ Intent (intent_len Bytes, UTF-8)                 │
├──────────────────────────────────────────────────┤
│ ChangeSet (changeset_len Bytes, rkyv-serialisiert) │
│   Vec<Mutation>:                                  │
│     Put(CellId, Cell)                            │
│     Delete(CellId)                               │
│     PutEdge(Edge)                                │
│     DeleteEdge(EdgeRef)                          │
│     UpdateAttr(CellId, AttrKey, AttrValue)       │
├──────────────────────────────────────────────────┤
│ Frame CRC (4 Bytes)                              │
│   CRC32c über gesamten Frame-Inhalt              │
└──────────────────────────────────────────────────┘
```

### 1.3 Group Commit

Bündelt WAL-Writes **über Branches**:

| Parameter | Wert | Begründung |
|---|---|---|
| Max. Wartezeit | 1 ms | Latenz-Obergrenze |
| Max. Batch-Größe | 64 Commits | Throughput-Optimierung |
| fsync-Modus | `fdatasync` | Metadaten-Sync nicht nötig |
| Trigger | Wartezeit ODER Batch-Größe erreicht | Whichever comes first |

**Ablauf:**
1. Commit-Request kommt beim WAL-Writer an.
2. Writer sammelt Requests in Puffer (max 1 ms oder 64 Requests).
3. Alle Frames werden sequentiell in die WAL-Datei geschrieben.
4. Ein `fdatasync()`.
5. Alle wartenden Commits werden notifiziert.

### 1.4 WAL-Bereinigung

WAL-Segmente werden gelöscht, wenn:
- Alle Frames im Segment durch einen Manifest-Eintrag abgedeckt sind (d.h. der Fold hat sie in Packs überführt).
- Das Segment nicht das aktive Schreib-Segment ist.
- Eine konfigurierbare Grace-Periode abgelaufen ist (Default: 1 h).

---

## 2. Manifest

### 2.1 Manifest-Format

Das Manifest ist ein **append-only Log** mit CRC-geschützten Einträgen:

```
Manifest-Datei:
┌──────────────────────────────────────────────────┐
│ Manifest Header (32 Bytes)                       │
│   magic[4] = b"CXM\x01"                         │
│   format_version[2] = u16 LE (1)                 │
│   repo_id[8] = u64 LE                            │
│   flags[2] = u16 LE                              │
│   padding[16]                                    │
├──────────────────────────────────────────────────┤
│ Entry 0 (variabel)                               │
├──────────────────────────────────────────────────┤
│ Entry 1                                          │
├──────────────────────────────────────────────────┤
│ ...                                              │
└──────────────────────────────────────────────────┘
```

### 2.2 Manifest-Entry

```
Entry:
┌──────────────────────────────────────────────────┐
│ entry_len[4] = u32 LE                            │
│ entry_type[1] = u8                               │
│ wal_seq[8] = u64 LE (höchste verarbeitete WAL-Seq) │
│ payload (rkyv-serialisiert):                     │
│   AddPack { pack_id, path, extent_count, size }  │
│   RemovePack { pack_id }                         │
│   UpdateRefs { branches: Vec<(Name, CommitId)> } │
│   UpdateLocator { locator_runs: Vec<LocatorRef> }│
│   AddSegment { segment_ref }                     │
│   RemoveSegment { segment_ref }                  │
│   HandleCounter { next_handle: u64 }             │
│   Checkpoint { full_state_snapshot }              │
│   AccessStats { sidecar_snapshot }               │
│ entry_crc[4] = CRC32c                            │
└──────────────────────────────────────────────────┘
```

### 2.3 Manifest-Rotation

- **Trigger:** Manifest-Datei > 16 MiB oder > 10 000 Entries.
- **Neues Manifest:** Beginnt mit `Checkpoint`-Entry (vollständiger Zustand).
- **CURRENT-Datei:** Atomar umgeschrieben (`write + rename`) auf neues Manifest.
- **Altes Manifest:** Nach Bestätigung des neuen gelöscht.

---

## 3. Crash-Recovery

### 3.1 Recovery-Ablauf

```
1. CURRENT lesen → aktive Manifest-Datei bestimmen
2. Manifest von Ende lesen, letzten gültigen Entry finden (CRC-Check)
3. Zustand rekonstruieren: Pack-Liste, Refs, Locator, Handle-Counter
4. WAL-Segmente scannen: Frames mit seq > manifest.wal_seq sammeln
5. Frames validieren: CRC + BLAKE3-Header prüfen
6. Torn Tail verwerfen: letzter unvollständiger Frame wird ignoriert
7. Gültige Frames wiedergeben: Branch-MemDeltas rekonstruieren
8. Index-Deltas nachholen: aus MemDelta-Diff ableiten
9. Ready — Engine ist konsistent
```

### 3.2 Recovery-Invarianten

| ID | Invariante | Konsequenz |
|---|---|---|
| INV-1 | Ref/Index-Eintrag nie geschrieben bevor Pack-Objekt durabel | Recovery findet alle referenzierten Objekte |
| INV-2 | Pack-Inhalt ist Quelle der Wahrheit; Index rekonstruierbar | Locator-Verlust = Neuaufbau aus Pack-Footern |
| INV-3 | Fehler bei Pack-fsync → Pack + Instanz vergiftet (Fail-Stop) | Kein Silent Data Corruption |
| INV-4 | Commits sind unveränderlich | Keine nachträgliche Manipulation |
| INV-5 | WAL-Frames sind self-describing (Branch, Intent, ChangeSet) | Recovery braucht keinen Vorzustand |

### 3.3 Recovery-Ziele

| Metrik | Ziel |
|---|---|
| Recovery-Zeit bei 1 GiB WAL | < 5 s |
| Recovery-Zeit bei 100 MiB WAL (typisch) | < 500 ms |
| Datenverlust nach bestätigtem Commit | 0 (bei `durable`) |
| Datenverlust bei `buffered` | max. 10 ms an Commits |
| Datenverlust bei `volatile` | alles seit letztem durable Commit |

---

## 4. Garbage Collection

### 4.1 Generationelles Mark-and-Sweep

**Protokoll: SATB + Remark** (für gleichzeitige Schreiber):

```
Phase 1: ROOTS
  Wurzeln sammeln:
    - Alle Branch-Heads (Refs)
    - Reflog-Fenster (konfigurierbar, Default: 1000 Einträge)
    - Aktive Sitzungs-Basen (laufende Transaktionen)
    - Pins (explizite Kurzzeit-Pins)
    - In-Flight-Objekte (gerade geschrieben, noch nicht committet)

Phase 2: MARK (ab Kernel-Snapshot S0)
  Traversiere alle erreichbaren Objekte ab Wurzeln:
    - Commits → Parents, RootSet
    - RootSet → Prolly-Wurzel, Segmente, Aliases, Policies
    - Prolly-Knoten → Kind-Knoten, Lane-Blöcke
    - Segmente → Segment-Blöcke

Phase 3: CANDIDATES
  Unmarkierte Objekte aus Packs älter als S0 sind Lösch-Kandidaten.

Phase 4: REMARK
  Alle Ref-/Sitzungsänderungen seit S0 nachmarkieren.
  Schreiber, die per Dedup ein sweep_candidate-Objekt wiederverwenden:
    → Objekt neu schreiben ODER Lösch-Markierung entfernen.

Phase 5: SWEEP
  Grace-Periode abwarten (Default: 1 h).
  Für Packs mit Live-Anteil < 50 %:
    → Copy-Forward: lebende Objekte in neues Pack kopieren.
    → Altes Pack löschen.
  Für Packs mit Live-Anteil ≥ 50 %:
    → Nichts tun (Platz wird bei nächster Compaction zurückgewonnen).
```

### 4.2 GC-Parameter

| Parameter | Default | Beschreibung |
|---|---|---|
| Grace-Periode | 1 h | Zeit zwischen SWEEP-Entscheidung und tatsächlichem Löschen |
| Reflog-Fenster | 1 000 Einträge | Wie viele Reflog-Einträge als Wurzeln gelten |
| Pack-Live-Schwelle | 50 % | Unter dieser Quote: Copy-Forward + Löschen |
| GC-Intervall | 1 h | Wie oft GC läuft |
| Max. GC-Dauer | 30 min | GC bricht ab und setzt beim nächsten Lauf fort |

### 4.3 Extent-Compaction

Kombiniert mit Re-Layout — ein Durchlauf, zwei Zwecke:
1. **Packs mit Live-Anteil < 50 %:** Lebende Extents in neue Packs kopieren (ggf. mit neuem Layout).
2. **Re-Layout-Trigger erfüllt:** Extents nach Affinity-Clustering neu anordnen.
3. **Ergebnis:** Neue Packs (immutable), Locator-Update, alte Packs nach Grace-Periode löschen.

---

## 5. Nebenläufigkeitsmodell

### 5.1 Lese-Pfad

**Wartefrei/lock-arm:**
- Immutable Strukturen + `ArcSwap` für `BranchState`.
- Epochenbasierte Rückgewinnung (`crossbeam-epoch`) für Caches und Locator-Runs.
- Leser blockieren nie Schreiber und umgekehrt.

```rust
pub struct BranchState {
    pub head: CommitId,
    pub mem_delta: Arc<MemDelta>,
    pub vec_delta: Arc<FlatVecIndex>,
    pub fts_delta: Arc<FtsPostings>,
    pub graph_delta: Arc<GraphDelta>,
    pub token_sums: Arc<TokenSumTree>,
    pub aliases: Arc<AliasTable>,
}

// Leser: snapshot = branch.state.load() → arbeitet auf konsistentem Snapshot
// Schreiber: neuen BranchState bauen → branch.state.store(new_state)
```

### 5.2 Schreib-Pfad

**Ein Sequencer-Actor pro Branch:**

```rust
pub struct BranchSequencer {
    inbox: mpsc::Receiver<CommitRequest>,
    state: Arc<ArcSwap<BranchState>>,
    wal_writer: mpsc::Sender<WalBatch>,
}

impl BranchSequencer {
    async fn run(&mut self) {
        while let Some(req) = self.inbox.recv().await {
            // 1. Validierung (Read-Set, Rebase)
            match self.validate(&req) {
                Ok(()) => {},
                Err(e) => { req.respond(Err(e)); continue; }
            }

            // 2. WAL-Write anfordern (Group Commit)
            let wal_future = self.wal_writer.send(req.to_wal_batch());

            // 3. In-Memory-Deltas aktualisieren
            let new_state = self.apply_to_memory(&req);

            // 4. WAL-Bestätigung abwarten
            wal_future.await?;

            // 5. Atomarer Pointer-Swap
            self.state.store(Arc::new(new_state));

            // 6. Client benachrichtigen
            req.respond(Ok(commit_id));
        }
    }
}
```

### 5.3 I/O-Pfad

| Plattform | Methode | Details |
|---|---|---|
| Linux | `io_uring` | Thread-per-Core-Executor; 1 Ring pro Core |
| macOS | `pread`/`pwrite` Threadpool | 4 Threads pro Core (konfigurierbar) |
| Windows | `pread`/`pwrite` Threadpool | Wie macOS |

**API-Schicht:** `tokio` als async Runtime. Verbindung zu io_uring über Channels.

**O_DIRECT:** Für Large-Sequential-Reads (Cold-Tail-Zugriff, Pack-Compaction). Puffer sind 4-KiB-aligned (`memalign`).

### 5.4 Hintergrund-Jobs

| Job | Priorität | Bandbreite |
|---|---|---|
| Fold (MemDelta → Prolly) | Hoch | Unbegrenzt (CPU-bound) |
| WAL-Bereinigung | Hoch | Unbegrenzt |
| Vektor-Segment-Build | Normal | Max. 1 concurrent |
| FTS-Segment-Merge | Normal | Max. 1 concurrent |
| Re-Layout | Niedrig | Max. 50 MiB/s I/O |
| GC | Niedrig | Max. 50 MiB/s I/O |
| Compaction | Niedrig | Max. 100 MiB/s I/O |
| Work-Queue-Dispatch | Normal | Unbegrenzt (leichtgewichtig) |

**ResourceGovernor:** Cache-, Memtable-, I/O-Budget über viele Repository-Instanzen teilbar.

```rust
pub struct ResourceGovernor {
    pub page_cache_budget: usize,       // Default: 40 % des Gesamtspeichers
    pub memtable_budget: usize,         // Default: 20 %
    pub index_cache_budget: usize,      // Default: 20 %
    pub io_bandwidth: IoLimit,          // Max. I/O-Bandbreite für Background-Jobs
}
```

---

## 6. Tiering

### 6.1 Tier-Definitionen

| Tier | Inhalt | Medium | Latenz |
|---|---|---|---|
| T0 | MemDelta, Locator-Hot-Set, Head-Page-Cache, Alias-Tabellen, Delta-Indizes | RAM | < 1 µs |
| T1 | Packs mit Hot Heads + aktuelle Cold Tails | NVMe | < 100 µs |
| T2 | Kalte Tails, Archiv-Segmente, alte Versionen | Objektspeicher (S3) | < 500 ms |

### 6.2 Tiering-Entscheidungen

**Kosten-Nutzen-Score:**
```
tier_score = access_frequency × latency_benefit ÷ storage_cost
```

| Transition | Trigger | Mechanismus |
|---|---|---|
| T0 → T1 | Memory-Pressure > 80 % | Eviction aus Page Cache (LRU) |
| T1 → T2 | Zugriffshäufigkeit < 1/Tag über 30 Tage | Hintergrund-Upload zu S3 |
| T2 → T1 | Zugriff auf kaltes Objekt | On-Demand-Fetch, Cache in T1 |
| T1 → T0 | Zugriff auf Extent-Head | Page-Cache-Promotion |

### 6.3 S3-Backend

```rust
pub trait ObjectStoreTier: Send + Sync {
    fn get(&self, pack_id: PackId, extent_no: u32, range: ByteRange) -> Result<Bytes>;
    fn put(&self, pack_id: PackId, data: Bytes) -> Result<()>;
    fn delete(&self, pack_id: PackId) -> Result<()>;
    fn list(&self) -> Result<Vec<PackId>>;
}
```

Implementierung für S3-kompatible APIs (AWS S3, MinIO, Cloudflare R2).

---

## 7. Retention

### 7.1 RetentionPolicy

```rust
pub enum RetentionPolicy {
    /// Alle Commits behalten
    KeepAll,
    /// Letzte N Commits behalten
    KeepLast(u32),
    /// Commits der letzten D Tage behalten
    KeepDays(u32),
    /// Kombination: mindestens N, höchstens D Tage
    KeepMinMax { min_count: u32, max_days: u32 },
}

pub struct BranchRetention {
    pub policy: RetentionPolicy,
    pub undo_window: Duration,          // Default: 24 h — gelöschte Commits noch erreichbar
}
```

### 7.2 Shallow-Kappung

Alte Commits werden durch **Snapshot-Commits** ersetzt:
1. Commit-Kette wird ab dem ältesten zu behaltenden Commit gekappt.
2. Ein neuer „Snapshot-Commit" (ohne Parent) wird erzeugt, der den vollständigen Zustand des Kappungspunkts enthält.
3. Alle älteren Commits werden aus dem Reflog entfernt und können GC'd werden.

---

## 8. Replikation und Sync

### 8.1 Objekt-basierter Sync (Git-artig)

Da alles content-addressed ist:

```
1. Frontier-Austausch: "Ich habe Commits X, Y, Z."
2. Fehlende Commits ermitteln: Commit-DAG-Traversal
3. Fehlende Objekte ermitteln: RootSet-Diff
4. Pack-Übertragung: Fehlende Chunks als temporäres Pack
5. Ref-Updates: Branch-Heads aktualisieren
```

### 8.2 Sync-Modi

| Modus | Beschreibung | Verfügbar ab |
|---|---|---|
| Push | Lokale Commits an Remote senden | Phase 4 |
| Pull | Remote-Commits lokal übernehmen | Phase 4 |
| Multi-Device | Bi-direktionaler Sync (Laptop ↔ Cloud) | Phase 4 |
| Read-Replica | Pull-basiert, read-only | Phase 4 |
| Quorum | Schreib-Bestätigung von N/M Replicas | Phase 4+ |

### 8.3 Konflikte bei Sync

Divergierte Branches werden beim Pull als **Merge** behandelt (gleiche Merge-Strategien wie bei lokalem Branch-Merge). ConflictCells werden erzeugt wo nötig.

---

## 9. Sicherheit

### 9.1 Trust-/Taint-Labels

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum TrustLabel {
    System            = 4,   // Engine-interne Daten
    PrincipalUser     = 3,   // Vom Benutzer direkt eingegeben
    ToolVerified      = 2,   // Von einem Tool erzeugt und verifiziert
    ToolUnverified    = 1,   // Von einem Tool erzeugt, nicht verifiziert
    ExternalUntrusted = 0,   // Externe Quelle, nicht verifiziert
}
```

**Vererbungsregel:** `derived_trust = min(source_trusts)` über alle Quellen.

**Hochstufung:** Nur durch privilegierte Prinzipale (Capability `Operations::Admin`), mit Audit-Eintrag im Commit-Intent.

### 9.2 Capability-Tokens

```rust
pub struct Capability {
    pub id: CapabilityId,              // 16 Bytes, zufällig
    pub caveats: Vec<Caveat>,
    pub hmac: [u8; 32],                // HMAC-BLAKE3 über (id, caveats)
}

pub enum Caveat {
    Namespaces(Vec<NsId>),
    Branches(Vec<BranchPattern>),      // Glob-Muster: "agent/*/run-*"
    Operations(OpSet),                 // read | write | fork | merge | admin
    MaxTrust(TrustLabel),              // Darf keine Daten mit höherem Trust lesen
    Ttl(Duration),
    TokenBudget { per_period: u32, period: Duration },
    ReadOnly,
    MaxBudgetPerRecall(u32),           // Max. Token pro Recall
}
```

**Attenuation:** Ein Capability-Halter kann neue Capabilities ableiten, die **nur einschränkender** sind (zusätzliche Caveats). Die Engine verifiziert HMAC-Kette.

**Wire-Format:**
```
CapabilityToken:
  id[16] | caveat_count[2] | caveats[variabel, rkyv] | hmac[32]
  Total: 50 + caveats Bytes, Base64url-codiert für Transport
```

### 9.3 Audit

- Commit-Kette = **Merkle-DAG** → manipulationsevident.
- Lesezugriffe im Access-Trace protokolliert (gesampelt oder vollständig, konfigurierbar).
- Reflog ist verkettet (`prev_hash`), Manipulationserkennung möglich.
- `ctx verify` prüft: Hash-Baum-Integrität, Locator-Konsistenz, Index↔Primärdaten, Lineage, GC-Erreichbarkeit.

---

*Ende SPEC-07. Weiter: [SPEC-08 API](SPEC-08-API.md)*
