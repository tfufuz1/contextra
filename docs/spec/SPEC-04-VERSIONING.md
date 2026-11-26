# SPEC-04 — Versionierung und Transaktionen

**Version 2.0 · November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Datenmodell](SPEC-02-DATA-MODEL.md) · [Storage](SPEC-03-STORAGE-ENGINE.md) · [Durability](SPEC-07-DURABILITY.md)

---

## 1. Zustandsmodell: Prolly-Tree + Delta-Log

### 1.1 Prolly-Tree

Probabilistischer B-Baum mit content-addressed Knoten. Knotengrenzen werden aus dem Hash der Schlüssel bestimmt.

**Eigenschaften:**
- **History-unabhängig:** Gleicher Inhalt ⇒ gleiche Struktur ⇒ gleicher Wurzel-Hash.
- **Strukturelles Sharing:** Forks und Commits teilen alle unveränderten Teilbäume.
- **O(Δ·log n) Diff und Merge** (Hash-Vergleich von oben nach unten).
- **Augmentierbar:** Innere Knoten speichern Aggregate (Token-Summen, Zähler, Min/Max-Zeit).

### 1.2 Boundary-Funktion

```rust
/// Bestimmt, ob nach einem Schlüssel eine Knotengrenze liegt.
/// repo_seed verhindert adversarielle Schlüsselwahl.
fn is_boundary(key: &[u8], repo_seed: &[u8; 32]) -> bool {
    let h = blake3::keyed_hash(repo_seed, key);
    let prefix = u32::from_le_bytes(h.as_bytes()[0..4].try_into().unwrap());
    // B = 12 → durchschnittliche Knotengröße ~4096 Bytes bei ~1 Eintrag/Byte
    prefix & ((1 << 12) - 1) == 0
}
```

| Parameter | Wert | Begründung |
|---|---|---|
| Hash-Funktion | BLAKE3 (keyed) | Schnell, kryptographisch, deterministisch |
| B (Boundary-Bits) | 12 | ~4 KiB Ziel-Knotengröße |
| Min. Knotengröße | 1 KiB | Verhindert degenerierte Einzeleintrags-Knoten |
| Max. Knotengröße | 16 KiB | Schutzmechanismus bei adversariellen Keys; erzwungene Grenze |
| Repo-Seed | 32 Bytes, bei `init` zufällig erzeugt | Anti-Adversarial, im Pack-Header gespeichert |

### 1.3 Knoten-Struktur

```rust
/// Innerer Knoten des Prolly-Tree.
pub struct InternalNode {
    pub level: u8,                           // 0 = Blatt-Eltern
    pub children: Vec<ChildEntry>,
}

pub struct ChildEntry {
    pub separator_key: Bytes,                // erster Schlüssel im Kind
    pub chunk_id: ChunkId,                   // Hash des Kind-Knotens
    pub hint: Option<LocationHint>,          // physischer Hint (siehe SPEC-03, 3.2)
    pub aggregates: NodeAggregates,          // augmentierte Daten
}

pub struct NodeAggregates {
    pub count: u32,                          // Anzahl Einträge im Teilbaum
    pub token_sums: SmallVec<[(TokenizerId, u64); 2]>,  // Summe der Token im Teilbaum
    pub time_min: Hlc,                       // frühester Zeitstempel
    pub time_max: Hlc,                       // spätester Zeitstempel
    pub size_bytes: u64,                     // Gesamtgröße in Bytes
}

/// Blattknoten des Prolly-Tree.
pub struct LeafNode {
    pub entries: Vec<(Bytes, Bytes)>,        // (key, value) sortiert
}
```

### 1.4 Delta-Log (LSM-artiger Schreibpuffer)

**Problem:** Prolly-Trees haben bei Einzelwrites erhöhte Schreibamplifikation.

**Lösung:**

```
Write ──► WAL (Group Commit) ──► Branch-MemDelta (sortiert, im RAM)
                                       │
                           Hintergrund-"Fold" (batched)
                                       ▼
                    neue Prolly-Knoten + Extents ──► Manifest-Eintrag
```

**MemDelta:**

```rust
/// In-Memory-Delta eines Branches. Sortierte Skip-List.
pub struct MemDelta {
    pub base: CommitId,                      // Basis-Commit
    pub entries: SkipList<Bytes, DeltaEntry>,
    pub size_bytes: usize,                   // aktuelle RAM-Nutzung
    pub entry_count: usize,
}

pub enum DeltaEntry {
    Put(Bytes),
    Delete,
}
```

Leser sehen immer `Basis-Baum ⊕ MemDelta` (Merge-Iterator).

### 1.5 Fold-Trigger

| Trigger | Schwelle | Beschreibung |
|---|---|---|
| MemDelta-Größe | 32 MiB | RAM-Budget begrenzen |
| MemDelta-Einträge | 10 000 | Merge-Iterator-Performance |
| Zeitbasiert | 30 s seit letztem Fold | Begrenzt Recovery-Zeit |
| Explizit | `branch.fold()` API | Für Tests und Checkpoints |
| Shutdown | immer | Sauberes Beenden |

---

## 2. Commit-Objekt

```rust
pub struct Commit {
    pub parents: SmallVec<[CommitId; 2]>,  // leer = Initial-Commit, 2 = Merge
    pub changes: Digest,                    // BLAKE3 des Changesets
    pub root: Option<RootSet>,             // Zustandshash (nach Fold berechnet)
    pub author: AgentIdentity,             // Prinzipal + Agent + Run
    pub intent: Intent,                    // Begründung und Idempotenz
    pub ts_tx: Hlc,                        // Transaktionszeit
    pub sig: Option<Signature>,            // ed25519, optional
}

pub struct Intent {
    pub run_id: Option<RunId>,
    pub step_id: Option<StepId>,
    pub idempotency_key: Option<IdempotencyKey>,
    pub message: CompactString,            // Freitext-Begründung (max 1 KiB)
}

/// Wurzel des gesamten versionierten Zustands – inklusive aller Indizes.
pub struct RootSet {
    pub primary: ChunkId,                  // Prolly-Wurzel der Cells
    pub sequences: ChunkId,                // geordnete Folgen mit Token-Summen
    pub graph: ChunkId,                    // Graph-Adjazenz
    pub fts_segments: Vec<SegmentRef>,     // FTS-Segmentreferenzen
    pub vec_segments: Vec<SegmentRef>,     // Vektor-Segmentreferenzen
    pub deletion_vectors: ChunkId,         // Roaring-Bitmaps je Segment
    pub aliases: ChunkId,                  // Handle-Tabelle
    pub policies: ChunkId,                 // Namespace-Policies
}
```

### 2.1 Zweistufige Identität

- **CommitId** = `BLAKE3(rkyv::to_bytes(Commit { root: None, sig: None, ..self }))` → **sofort** nach dem Commit bekannt (ohne auf Fold zu warten).
- **RootSet-Hash** wird beim Fold berechnet und dient der **Zustandsgleichheit** (gleicher Zustand = gleicher Hash, unabhängig von der Commit-Historie).

### 2.2 Signierung

- **Algorithmus:** Ed25519 (64 Bytes Signatur, 32 Bytes Public Key)
- **Signierte Daten:** `BLAKE3(rkyv::to_bytes(Commit { sig: None, ..self }))` — die Signatur ist nicht Teil der signierten Daten.
- **Optional:** Konfigurierbar pro Repository. Bei `require_signatures = true` lehnt die Engine unsignierte Commits ab.

---

## 3. Commit-Protokoll

### 3.1 Schritt-für-Schritt (atomar, crash-sicher)

```
1. begin        → Snapshot (Branch-Head H, Epoche e); Read-Set-Tracking starten
2. buffer       → Writes im Transaktions-ChangeSet (Cells, Facetten, Kanten)
3. validate     → Branch-Head bewegt? → Rebase-Versuch (siehe 3.2)
4. WAL append   → Frame {CRC, BLAKE3-Header, Intent, IdempKey, ChangeSet}
                   Group Commit, Durability nach Transaktionsoption
5. apply        → in Branch-MemDelta + SYNCHRONE Index-Deltas:
                   - FTS-Postings (invertierte Liste, Delta)
                   - Graph-Kanten (Adjazenz-Delta)
                   - Temporal-Index-Einträge
                   - Token-Summen-Updates
                   - Vektor-Delta-Index (Flat-Append)
                   - Handle-Registrierung
6. publish      → atomarer Pointer-Swap (ArcSwap) auf neuen BranchState
                   ← Sichtbarkeitspunkt für alle Leser
7. ack          → Client erhält CommitId + ggf. Warnings
8. fold (async) → MemDelta → Prolly-Knoten + Extents → fsync Packs → Manifest
```

### 3.2 Automatischer Rebase

Wenn zwischen `begin` und `validate` andere Commits auf denselben Branch eingetroffen sind:

```rust
fn try_rebase(tx: &Transaction, current_head: CommitId) -> Result<(), RebaseError> {
    // 1. Prüfe ob der Base-Commit ein Vorfahre des aktuellen Heads ist
    if !is_ancestor(tx.base, current_head) {
        return Err(RebaseError::Diverged);
    }

    // 2. Berechne die Änderungen seit Base
    let intervening_changes = diff(tx.base, current_head);

    // 3. Prüfe Disjunktheit mit Read-Set und Write-Set
    if tx.read_set.intersects(&intervening_changes)
       || tx.write_set.intersects(&intervening_changes) {
        return Err(RebaseError::Conflict);
    }

    // 4. Rebase: Setze neue Base auf current_head
    tx.rebase_onto(current_head);
    Ok(())
}
```

Bei Rebase-Fehlschlag: Transaktion wird an den Client zurückgegeben mit `RetryableError::Conflict`.

### 3.3 Atomarität

Sichtbarkeit nur über Schritt 6 (Pointer-Swap). Leser sehen ein Commit vollständig (inkl. aller synchronen Index-Deltas) oder gar nicht.

**Wichtig:** Die synchronen Index-Deltas in Schritt 5 sind **In-Memory-Strukturen**, die zusammen mit dem MemDelta im BranchState liegen. Sie werden beim Fold in persistente Segmente überführt.

---

## 4. Branches, Forks, Tags

### 4.1 Fork

```rust
/// Fork erzeugt einen neuen Branch als Ref auf bestehenden Commit.
/// O(1): nur Ref + leeres MemDelta. Ziel: < 50 µs.
pub fn fork(&self, name: &str) -> Result<Branch> {
    let head = self.head();
    let branch = Branch {
        name: BranchName::parse(name)?,
        head: head.clone(),
        mem_delta: MemDelta::empty(head.id),
        // Index-Deltas starten leer
        vec_delta: FlatVecIndex::empty(),
        fts_delta: FtsPostings::empty(),
        graph_delta: GraphDelta::empty(),
        // Sidecar-Daten werden nicht geforkt (eventual-consistent)
    };
    self.refs.create(branch)?;
    Ok(branch)
}
```

### 4.2 Branch-Namenskonvention

| Muster | Verwendung | TTL |
|---|---|---|
| `main` | Hauptbranch | unbegrenzt |
| `agent/<id>/run-<n>/step-<k>` | Agent-Arbeitsbranch | konfigurierbar, Default 7 d |
| `spec/<uuid>` | Spekulativer Branch | konfigurierbar, Default 24 h |
| `checkpoint/<name>` | Manueller Checkpoint | unbegrenzt |
| `scratch/<id>` | Volatiler Scratch-Branch | Sitzungsdauer |

### 4.3 Branch-Lifecycle

```
Created ──► Active ──► Merged ──► GC'd
                  └──► Abandoned ──► GC'd (nach TTL)
```

| Zustand | Beschreibung |
|---|---|
| `Created` | Fork ausgeführt, noch kein Commit |
| `Active` | Commits vorhanden, Arbeit im Gange |
| `Merged` | In Ziel-Branch gemergt, Ref bleibt als Tombstone (konfigurierbare Dauer) |
| `Abandoned` | Explizit oder per TTL als aufgegeben markiert |
| `GC'd` | Ref gelöscht, nicht mehr erreichbar (Objekte werden durch GC aufgeräumt) |

### 4.4 Speculative Branches

- **TTL:** Default 24 h, konfigurierbar pro Branch bei Erstellung.
- **Automatische GC:** Hintergrund-Job prüft alle 5 Minuten.
- **Promotion:** `branch.promote(new_ttl)` verlängert oder entfernt TTL.
- **Squash-Commit:** `branch.squash_into(target)` fasst alle Commits zusammen und mergt.

### 4.5 Scratch-Branches

- **Durability:** `volatile` — nur RAM, kein WAL-fsync.
- **Verwendung:** Arbeitsgedächtnis pro Schritt, Hypothesen-Testing.
- **Promotion:** `scratch.promote_to_durable()` erzeugt einen `squash_commit` mit `durability = durable` und schreibt WAL.
- **Verlust bei Crash:** Akzeptabel und dokumentiert.

### 4.6 Tags und Checkpoints

```rust
/// Tag = benannter, immutabler Ref auf einen Commit.
pub struct Tag {
    pub name: CompactString,
    pub commit: CommitId,
    pub message: Option<CompactString>,
    pub ts: Hlc,
}
```

Checkpoints sind Tags mit Konvention `checkpoint/<name>`. „Rollback auf Schritt 12" = Ref-Update des Branch-Heads auf den Checkpoint-Commit.

---

## 5. Isolation und Konflikte

### 5.1 Isolationsstufen

| Stufe | Mechanismus | Verwendung |
|---|---|---|
| **Snapshot Isolation (Standard)** | Jeder Agent arbeitet auf eigenem Branch; Konflikte erst beim Merge | Standard für Agent-Workflows |
| **Serializable** | Read-Set-Validierung (Bereiche + Bloom-Filter) + Rebase | Für kritische Daten, explizit angefordert |

### 5.2 Single Sequencer pro Branch

```rust
/// Jeder Branch hat genau einen Sequencer-Actor.
/// Der Sequencer serialisiert nur:
///   - Commit-Validierung (Read-Set-Check)
///   - Pointer-Swap (ArcSwap)
/// Der eigentliche Schreibpfad (Serialisierung, WAL) ist parallel.
pub struct BranchSequencer {
    pub branch: BranchName,
    pub inbox: mpsc::Receiver<CommitRequest>,
    pub state: ArcSwap<BranchState>,
}
```

Branches sind **vollständig parallel**. Der einzige globale Engpass ist der WAL-Writer (Group Commit).

---

## 6. Merge

### 6.1 Dreiwege-Merge

Merge(Base, Ours, Theirs) **pro Cell und pro Attribut** mit Strategie aus der Namespace-Policy.

```rust
pub enum MergeOutcome {
    /// Merge ohne Konflikte abgeschlossen
    Merged(CommitId),
    /// Merge mit Konflikten — ConflictCells erzeugt
    Conflicts(Vec<ConflictCell>),
    /// Fast-Forward (kein Merge nötig)
    FastForward(CommitId),
}
```

### 6.2 Merge-Strategien (exhaustive)

| Strategie | Verhalten | Anwendung |
|---|---|---|
| `lww` | Last-Writer-Wins nach HLC | Default für die meisten Felder |
| `union` | Mengen-Vereinigung | Tags, Kanten |
| `sum` | Zähler-Addition (CRDT-Counter) | Zähler, Scores |
| `max` | Maximum | Salience, Confidence |
| `min` | Minimum | Trust (konservativstes Label) |
| `diff3` | Textmerge auf Zeilen-/Satzebene | raw-Inhalt bei Textänderungen |
| `custom(wasm)` | Sandboxed WASM-Merge-Funktion | Anwendungsspezifisch |
| `manual` | Erzeugt `ConflictCell` | Default-Fallback |

### 6.3 WASM-Merge-Funktionen (Entscheidung C-20)

| Parameter | Wert | Begründung |
|---|---|---|
| Runtime | wasmtime | Ausgereift, deterministic mode |
| Fuel-Limit | 10 000 000 Instruktionen | Timeout-Schutz |
| Speicherlimit | 16 MiB | DoS-Schutz |
| WASI | **deaktiviert** | Kein Filesystem, kein Netzwerk, keine Clock |
| Floating-Point | **verboten** (integer-only API) | Determinismus |
| Randomness | **verboten** | Determinismus |
| I/O | Lineare Speicher-Buffers, rkyv-serialisiert | Performance |
| Bei Fuel-Erschöpfung | Fallback auf `manual` (ConflictCell) | Graceful Degradation |

### 6.4 ConflictCell

```rust
pub struct ConflictCell {
    pub id: CellId,                             // eigene CellId (CellKind::Conflict)
    pub target: CellId,                         // welche Cell betroffen
    pub base: Option<CellVersion>,              // gemeinsamer Vorfahr
    pub ours: CellVersion,                      // unsere Version
    pub theirs: CellVersion,                    // deren Version
    pub field: FieldPath,                       // welches Feld (z.B. "attrs.tags", "raw")
    pub suggestion: Option<MergeSuggestion>,    // automatische Vorschläge
    pub created_at: Hlc,
}

pub enum MergeSuggestion {
    TakeOurs,
    TakeTheirs,
    Merged(Bytes),          // automatisch erzeugter Merge-Vorschlag
}
```

Konflikte sind **abfragbare, versionierte Daten**: Ein Agent löst sie per `resolve(conflict_id, choice | merged_value)`. Ungelöste Konflikte blockieren **nicht** den Branch — sie sind sichtbar und markiert, aber der Branch ist weiter nutzbar.

**Konflikt-Timeout:** Ungelöste ConflictCells werden nach 7 Tagen (konfigurierbar) mit `lww` automatisch aufgelöst und als solche markiert.

---

## 7. Durability-Stufen

| Stufe | WAL | fsync | Latenz | Einsatz |
|---|---|---|---|---|
| `volatile` | nein | nein | < 10 µs | Scratchpad pro Schritt |
| `buffered` | ja | gebündelt (≤ 10 ms) | < 1 ms | Hochfrequente Beobachtungen |
| `durable` | ja | vor Ack (Group Commit) | < 2 ms (p99) | Standard |
| `replicated` | ja | Quorum-Ack (ab Phase 4) | < 50 ms | Kritische Fakten |

**Default:** `durable`. Namespace-Policy kann Default überschreiben. Transaktion kann pro Commit eine Stufe wählen.

---

## 8. Idempotenz

### 8.1 Idempotenz-Journal

```rust
/// Idempotenz-Schlüssel: (run_id, step_id, idempotency_key)
pub struct IdempotencyKey {
    pub run_id: RunId,
    pub step_id: StepId,
    pub key: CompactString,     // max 256 Bytes
}

/// Journal-Eintrag
pub struct IdempotencyEntry {
    pub key: IdempotencyKey,
    pub commit_id: CommitId,
    pub result: IdempotencyResult,
    pub expires_at: Timestamp,
}
```

### 8.2 Verhalten bei Wiederholung

1. Client sendet Commit mit Idempotenz-Key.
2. Engine prüft Journal:
   - **Key gefunden, nicht abgelaufen:** Liefert gespeichertes Ergebnis zurück, kein erneuter Commit.
   - **Key nicht gefunden:** Normaler Commit-Ablauf, Ergebnis wird im Journal gespeichert.
   - **Key gefunden, abgelaufen:** Wie „nicht gefunden" (erneuter Commit).
3. **TTL:** Default 7 Tage, konfigurierbar. Hintergrund-GC entfernt abgelaufene Einträge.

### 8.3 Journal-Speicherung

Das Idempotenz-Journal ist ein **System-Namespace** (`_system/idempotency`), versioniert im RootSet. Bei Merge: Union (Einträge beider Seiten behalten).

---

## 9. Zeitreise

### 9.1 Granularitäten

| Stufe | Auflösung | Mechanismus |
|---|---|---|
| G1 | Kernel-Seq | `get_at_seq` + Pins (Kurzzeit, für laufende Transaktionen) |
| G2 | **Commit** | `CommitId → RootSet` → vollständiger Zustand |
| G3 | **Transaktion innerhalb Commit** | `StateAt(commit, i)` via WAL-Journal (nur innerhalb eines Fold-Batches) |
| G4 | **Zelle** | Zeilen-/Spaltendiff, `blame` (Cell-Historie über Commit-Kette) |

### 9.2 Adressierung

| Syntax | Bedeutung |
|---|---|
| `main` | Aktueller Head von Branch `main` |
| `main~3` | 3 Commits vor Head |
| `<commit-id>` | Exakter Commit |
| `main@{2026-11-21T10:00Z}` | Branch-Head zum Zeitpunkt |
| `<commit>:tx[7]` | 7. Transaktion innerhalb eines Fold-Batches |

### 9.3 Verzweigen aus der Vergangenheit

```rust
/// Erzeugt einen neuen Branch ab einem historischen Zustand.
/// branch_at(<commit>:tx[i]) erzeugt einen synthetischen Commit.
pub fn branch_at(&self, ref_spec: &RefSpec, name: &str) -> Result<Branch>;
```

Verwendung: „Was-wäre-wenn"-Simulationen, Rollback-Exploration.

---

*Ende SPEC-04. Weiter: [SPEC-05 Indizes](SPEC-05-INDEXES.md)*
