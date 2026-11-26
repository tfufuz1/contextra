# SPEC-04 — Versionierung, Transaktionen und Nebenläufigkeit

**Version 2.1 · November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Datenmodell](SPEC-02-DATA-MODEL.md) · [Indizes](SPEC-05-INDEXES.md) · [Durability](SPEC-07-DURABILITY.md)

---

## 1. Zustandsmodell: Prolly-Tree + MemDelta

### 1.1 Prolly-Tree (Truth Layer)

Probabilistischer, inhaltsadressierter B-Baum für alle logischen Relationen des `RootSet`:
- **History-unabhängig (INV-1):** Gleiche Schlüssel-Wert-Menge ⇒ deterministische Knotengrenzen ⇒ identischer Wurzel-Hash, unabhängig von Schreibreihenfolge, Batches oder Compaction.
- **Strukturelles Sharing:** O(1)-Forks; Branches teilen unveränderte Teilbäume.
- **O(Δ·log n) Diff und 3-Wege-Merge:** Hash-Gleichheit überspringt ganze Teilbäume sofort.

### 1.2 Boundary-Funktion

```rust
fn is_boundary(key: &[u8], repo_seed: &[u8; 32]) -> bool {
    let h = blake3::keyed_hash(repo_seed, key);
    let prefix = u32::from_le_bytes(h.as_bytes()[0..4].try_into().unwrap());
    prefix & ((1 << 12) - 1) == 0 // B = 12 → Ziel-Knotengröße ~4–8 KiB
}
```

- **Min./Max.-Knotengröße:** Min `1 KiB`, Max `16 KiB` (harte Schutzgrenze gegen adversarielle Key-Folgen).

### 1.3 Knotenstruktur (Hints strikt außerhalb des Hashs)

```rust
pub struct InternalNode {
    pub level: u8,
    pub children: Vec<ChildEntry>,
}

pub struct ChildEntry {
    pub separator_key: Bytes,
    pub chunk_id: ChunkId,               // Hash des Kind-Knotens
    pub aggregates: NodeAggregates,      // count, token_sums, time_min/max, size_bytes
}

/// Optionales physisches Hint-Array — wird in RAM/Sidecar gehalten und geht NIEMALS
/// in `ChunkId = BLAKE3(InternalNode)` ein! Re-Layout bricht keinen einzigen Hash.
pub struct UnhashedHints {
    pub child_hints: SmallVec<[Option<(PackId, u32)>; 16]>,
}
```

### 1.4 MemDelta & Hintergrund-Fold

Einzelwrites landen über das WAL zuerst sortiert in einem pro-Branch **In-Memory-`MemDelta`** (Skip-List) plus synchronen In-Memory-Index-Deltas (`vec_delta`, `fts_delta`, `graph_delta`, `bitmap_delta`). Leser sehen stets die konsistente Überlagerung `Basis-Prolly-Tree ⊕ MemDelta`.

**Fold-Trigger:** MemDelta > 32 MiB, > 10 000 Einträge, > 30 s seit letztem Fold, `branch.fold()` explizit oder Shutdown.

---

## 2. Commit, Deterministisches `RootSet` und separates `IndexManifest` (löst Reibungspunkt 1)

Um strikte **History-Unabhängigkeit (INV-1)** mit asynchroner **Vamana-/FTS-Segment-Compaction ([SPEC-05](SPEC-05-INDEXES.md) §2.4 & §3)** zu vereinen, wird der Wurzelzustand strikt in **Wahrheit (`RootSet`)** und **physische Indizes (`IndexManifestRef`)** getrennt:

```rust
pub struct Commit {
    pub parents: SmallVec<[CommitId; 2]>,
    pub changes: Digest,                      // BLAKE3 des ChangeSets
    pub truth_root: Option<RootSet>,          // Deterministischer Zustandshash (nach Fold)
    pub index_manifest: Option<IndexManifestRef>, // Separater Seiten-Zeiger! NICHT in CommitId/truth_hash!
    pub author: AgentIdentity,
    pub intent: Intent,
    pub seq: u64,                             // Globale monotone Commit-Sequenz
    pub ts_tx: Hlc,
    pub sig: Option<Signature>,               // Ed25519 (optional/konfigurierbar)
}

/// 100 % deterministischer, history-unabhängiger Wahrheitszustand.
/// Jedes Feld ist ein kanonischer Prolly-Tree ohne physische Segment-Layouts.
pub struct RootSet {
    pub primary: ChunkId,                     // CellId → CellRecord { gid, content_hash, meta_hash, flags }
    pub sequences: ChunkId,                   // (SeqId, position) → (Gid, token_sums augmentiert)
    pub graph: ChunkId,                       // G:(src, kind, dst, id) & R:(dst, kind, src, id)
    pub aliases: ChunkId,                     // Bidirektional CellId ↔ Gid (+ Handle-Zuordnung)
    pub policies: ChunkId,                    // Katalog: Namespaces, Schemas, EmbeddingSpaces, ACL-Gruppen
    pub conflicts: ChunkId,                   // Offene & aufgelöste ConflictCells
    pub provenance: ChunkId,                  // Append-only ProvRecords
}

/// Nicht-deterministischer / kompaktionsabhängiger Index-Zeiger.
/// Compaction und paralleler Vamana-Build ändern NUR dieses Objekt im Manifest —
/// `CommitId` und `truth_hash = BLAKE3(RootSet)` bleiben völlig unberührt!
pub struct IndexManifestRef {
    pub manifest_id: ChunkId,                 // Zeiger auf IndexManifest-Objekt
    pub source_truth_hash: Digest,            // Wahrheits-Hash, bis zu dem Segmente gebaut sind
    pub watermark_seq: u64,                   // Höchste abgedeckte globale Seq
}
```

### 2.1 Zweistufige Identität & Zustandsgleichheit

- **`CommitId`:** `BLAKE3(rkyv(Commit { truth_root: None, index_manifest: None, sig: None, ..self }))` — **sofort** beim Commit bekannt (noch vor dem Fold).
- **`truth_hash`:** `BLAKE3(rkyv(RootSet))` — nach Fold berechnet. Zwei Branches mit identischer logischer Fakten-/Zellmenge haben **garantiert denselben `truth_hash`**, egal in welcher Reihenfolge geschrieben oder wie oft Vamana-/FTS-Segmente kompaktiert wurden!

---

## 3. Transaktions- und Commit-Protokoll (`Txn<Mode>`)

### 3.1 Typestate-API (verhindert unregistrierte SSI-Reads konstruktiv)

```rust
let mut tx: Txn<ReadWrite> = branch.begin(Isolation::Snapshot)?; // Pinnt Branch-Head + Epochen-Lease
let cell = tx.get(cell_id)?;     // Lesen UND Read-Set-Registrierung sind untrennbar!
tx.put_content(content)?;
tx.put_meta(meta)?;
tx.link(edge)?;
let commit_id = tx.commit(CommitOpts::new("Fakt extrahiert").idempotency_key("k1"))?;
```

Reine Leseabfragen nutzen einen lifetime-gebundenen **`Snapshot<'db>`** ohne Read-Set-Overhead. Es gibt keine API-Methode, die eine externe `snapshot_seq` akzeptiert.

### 3.2 Commit-Ablauf (ein Commit-Punkt, kein Mehr-Store-2PC)

1. **`begin`:** Branch-Head + MemDelta-Seq + Snapshot-Lease (TTL 300 s, erneuerbar) pinnen.
2. **`buffer`:** Mutationen im `ChangeSet` sammeln; Pflichtprüfung auf Schema, Vektor-Validität (kein NaN/∞, Dimension passend) und **Pflichtprovenienz** (`prov_id` oder `Hypothesis`).
3. **`idempotency`:** Prüfe Kernel-Sidecar `IdempotencyTable` (`sidecar/idempotency.cxs`). Treffer mit gleichem `changes`-Digest ⇒ sofortiges `idempotent_replay` mit gespeicherter `CommitId`; abweichender Digest ⇒ `idempotency_mismatch`!
4. **`validate` & OCC/Rebase:**
   - **Snapshot Isolation (Default):** Prüfe Write-Write-Disjunktheit auf `CellId`/`Gid` seit `tx.base`.
   - **Serializable (SSI, opt-in):** Prüfe zusätzlich Read-Set + Prädikat-Bereiche (z. B. `(NsId, predicate)`-Scans, Graph-Nachbarschaften) seit `tx.base`. Bei `tx.base_seq < pruned_through` ⇒ Fail-Closed `snapshot_too_old`.
   - **Funktionale Prädikate:** Bei `CellKind::Fact` mit `functional`-Prädikat im Konflikt zu lebendigem Fakt ⇒ Policy `SupersedeOnNewer` schließt `old.meta.valid_to` automatisch (nur `CellMeta`-Update!) oder legt `ConflictCell` an.
   - Falls Head bewegt & disjunkt ⇒ automatischer Rebase; sonst `rebase_failed` (retryable).
5. **`WAL append`:** Schreibe WAL-Frame (inkl. `IdempotencyKey` + `changes`-Digest!) über Group Commit. **Dies ist der einzige Durability-Commit-Punkt** (bei `volatile` entfällt Schritt 5).
6. **`apply & publish`:** Wende Änderungen auf `MemDelta`, synchrone In-Memory-Index-Deltas und `IdempotencyTable` an; publiziere neuen `BranchState` per atomarem `ArcSwap`.
7. **`ack & async fold`:** Client erhält `CommitId`; Hintergrund-Fold überführt `MemDelta` → `Prolly` + `PAX-Extents` + `Manifest`; abgeleitete Index-Builds und Work-Queue-Jobs laufen asynchron mit Watermark-Tracking.

---

## 4. Branches, Session-Isolation und Savepoints

### 4.1 Branch-Modell & Schutz von `main`

- **O(1)-Fork (`< 50 µs`):** Erzeugt benannten Ref auf `CommitId` + leeres `MemDelta` + leere Index-Deltas; physische Indizes und Prolly-Teilbäume werden Zero-Copy geteilt.
- **Prinzip: Agenten schreiben nie direkt nach `main`.**
  - Agenten erhalten Capabilities auf Session-Branches (`agent/<id>/run-<n>` oder `sess/<agent>/<id>`).
  - Direkte Commits auf geschützte Branches (`main`, `release/*`) werden ohne `ProtectedBranchWrite`-Capability mit `permission_denied` abgelehnt.
  - Übernahme nach `main` erfolgt ausschließlich über **Merge-Requests (`mr/<n>`)** und die deterministischen **Merge-Gates `G1`–`G7`** (§6.3).

### 4.2 Branch-Typen & Lifecycle

| Muster | Zweck | Default-Durability | Default-TTL |
|---|---|---|---|
| `main` | Kuratierte Hauptwahrheit | `durable` | unbegrenzt |
| `dev/<name>` | Langlebige Integration | `durable` | unbegrenzt |
| `sess/<agent>/<id>` / `agent/*` | Agenten-Arbeitszweig mit Quoten | `durable` oder `buffered` | 24 h – 7 d |
| `mr/<n>` | Merge-Request inkl. Gate-Report & `ConflictCell`s | `durable` | bis Abschluss + 7 d |
| `spec/<uuid>` | Spekulativer Branch (Was-wäre-wenn) | `buffered` | 24 h |
| `scratch/<id>` | Ephemeres Arbeitsgedächtnis | `volatile` | Sitzungsende |
| `tag/<name>` / `checkpoint/<name>` | Unveränderlicher Snapshot-Zeiger | — | unbegrenzt (pinnt GC) |

**Savepoint je Tool-Call:** `branch.savepoint(name)` merkt sich `(CommitId, seq)`. `branch.rollback_to(sp)` setzt den Head per CAS zurück (`O(1)`); dazwischenliegende Commits bleiben nur forensisch im verketteten Reflog (bis TTL).

---

## 5. Nebenläufigkeit

- **Leser:** Komplett lock-frei über `ArcSwap<BranchState>` + Epochen-Reklamation (`crossbeam-epoch`). Leser blockieren nie Schreiber.
- **Schreiber:** Ein **`BranchSequencer`-Actor pro Branch** (vollständig parallel zwischen verschiedenen Branches!). Der globale WAL-Writer bündelt Frames per Group Commit.

---

## 6. Drei-Wege-Merge, Konflikte und Merge-Gates

### 6.1 Normative Merge-Regeln

Merge(Base, Ours, Theirs) arbeitet in **O(Δ·log n)** über den Prolly-Diff und die **entkoppelten Cell-Teile**:

| Bereich | Situation | Normative Regel |
|---|---|---|
| **`CellContent`** | Beidseitig auf verschiedenen Inhalt geändert | **Konflikt** (`ConflictCell` für `raw`). **Niemals Embeddings mitteln oder per Cosine-Schwelle still wählen!** |
| **`CellMeta.attrs`** | Disjunkte Attribut-Keys geändert | Automatischer **feldweiser Merge** (`content_hash` bleibt unberührt!) |
| **`CellMeta.attrs.tags`** | Beidseitig Tags geändert | `union` (oder Policy aus Namespace) |
| **`CellMeta.time.valid_to`** | Eine Seite schließt Gültigkeit, andere ändert Tags | Konfliktfrei kombinierbar (Entkopplung von Content/Attrs!) |
| **`CellMeta.trust`** | Verschiedene Stufen | **`min(ours, theirs)`** (konservativer Wert); Hochstufung nur per Gate G7 |
| **`CellMeta.taint`** | Verschiedene Taint-Flags | **`ours ∪ theirs`** (Vereinigung) |
| **`CellMeta.belief`** | Beidseitig neue Evidenzen | **Neuberechnung über deduplizierte `source_roots`** (keine Doppelzählung!) |
| **`Edge`** | Mengenänderung | `AddWins` (Default) / `RemoveWins` / `Manual` je `EdgeKind`-Policy |
| **`aliases` / `provenance`** | Einträge beider Seiten | `union` (konfliktfrei durch globale `Gid`s und `ev_seq`) |
| **`Summary` (`level > 0`)** | Quell-Chunks auf beiden Seiten geändert | **Nicht mergen** → als `stale` markieren und in Work Queue neu einplanen |
| **Custom Merge** | Policy `custom(wasm)` | `wasmtime` (Fuel 10M, 16 MiB RAM, kein WASI/Float); Fallback auf `manual` |

### 6.2 `ConflictCell` (Main bleibt konfliktfrei — INV-7)

Ungelöste Konflikte werden **nicht** still in `main` gemergt: Der Merge-Versuch materialisiert `ConflictCell`s im Merge-Request-Branch **`mr/<n>`** (`MergeOutcome::Conflicts(mr_branch, list)`), während `main` sauber bleibt. Ein Agent oder Kurator löst per `resolve(conflict_id, TakeOurs | TakeTheirs | Merged(value))` auf `mr/<n>` auf und wiederholt den Merge.

### 6.3 Merge-Gates (`G1`–`G7`) vor Übernahme nach `main`

Jeder Merge nach `main` (außer expliziter Admin-Bypass mit Audit-Eintrag) durchläuft diff-inkrementell die konfigurierbare Gate-Pipeline:

| Gate | Prüfung | Default-Verhalten |
|---|---|---|
| **G1 Schema & Integrität** | Typen, Kardinalitäten, `functional`-Prädikate, keine dangling edges | Blockierend |
| **G2 Provenienz & Vertrauen** | Pflichtprovenienz (`G0`–`G2`), gültige Signaturen (falls aktiv), `trust ≥ min_main_trust` | Blockierend (sonst Quarantäne/Review) |
| **G3 Konfliktscan** | Überlappende Gültigkeiten, Widersprüche gegen `main`-Fakten (deterministisch + asynchroner Judge mit Timeout) | Blockierend bei `Direct` gegen `Verified`/`Curated`, sonst Warnung |
| **G4 Blast Radius** | Anteil geänderter/gelöschter `main`-Cells > Schwellwert (Default 1 % pro Session) | Stoppt Auto-Merge → Review |
| **G5 Grounding-Check** | Bei `Fact`/`Summary`: Citation-Binding & MAC-Handle-Prüfung | Blockierend / Quarantäne (`G3`) |
| **G6 Taint & Injection** | `INJECTION_SUSPECT`/`EXTERNAL`-Scan; verhindert Trust-Eskalation über `AgentInferred` | Cap / Blockierend |
| **G7 Human/Curator Review** | Externer `Reviewer`-Port für geschützte Namespaces oder Trust-Promotion | Konfigurierbar |

---

## 7. Durability-Stufen

| Stufe | WAL | Sync-Verhalten | Latenz-Ziel | Einsatz |
|---|---|---|---|---|
| `volatile` | nein | nur RAM | < 10 µs | `scratch/*` Wegwerf-Branches |
| `buffered` | ja | periodisch (≤ 10 ms) | < 1 ms | Hochfrequente Roh-Observations |
| `durable` *(Default)* | ja | Group Commit (`fdatasync`, ≤ 1 ms / 64 Commits) | < 2 ms p99 | Standard für `main` & Sessions |
| `strict` | ja | sofortiges `fdatasync` je Commit + Dir-Sync bei neuem Ref | ~2–5 ms | Kritische Katalog-/Audit-Commits |
| `replicated` | ja | Quorum-Ack (ab Phase 4) | < 50 ms | Multi-Node-Replikation |

*Regel:* `main` erlaubt im Produktivbetrieb weder `volatile` noch unbestätigtes Verwerfen.

---

## 8. Nicht-versionierte Kernel-Tabellen: Idempotenz & Work Queue (löst Reibungspunkt 2)

Operativer Laufzeitzustand darf **keine** Commit-Lawinen erzeugen und **nicht** über Branch-Merges verschleppt werden. Daher liegen Idempotenz-Journal und Work Queue **außerhalb des `RootSet`** in WAL-gesicherten Kernel-Sidecars (`sidecar/`):

### 8.1 Idempotenz-Journal (`sidecar/idempotency.cxs`)

- **Schlüssel:** `(branch_id: u32, run_id: RunId, step_id: StepId, key: CompactString)`.
- **Wert:** `(commit_id: CommitId, payload_digest: Digest, result_summary: Bytes, expires_at: Timestamp)`.
- **Crash-Sicherheit ohne `RootSet`-Pollution:** Jeder `Commit`-WAL-Frame enthält `idempotency_key` und `changes`-Digest. Beim WAL-Replay rekonstruiert der Kernel das Idempotenz-Journal deterministisch mit demselben WAL-Commit — null Zusatz-Commits, 100 % Crash-Garantie (INV-6)!
- **Abweichender Retry:** Wird derselbe Key mit anderem `payload_digest` wiederholt ⇒ Fehler `idempotency_mismatch` (verhindert stille Überschreibung fehlerhafter Retries).
- **TTL:** Default 7 Tage; Background-Sweep entfernt abgelaufene Einträge ohne `RootSet`-Änderung.

### 8.2 Work-Queue-Leases (`sidecar/work_queue.cxs`)

Siehe [SPEC-06](SPEC-06-CONTEXT-ENGINE.md) §7.2: Enqueue, Leasing (`lease_owner`, `lease_expires_at`), `attempts` und Heartbeats aktualisieren **ausschließlich** die nicht-versionierte Sidecar-Queue. Erst das **Arbeitsergebnis** eines Workers (Embedding, Summary, Fakt) wird als regulärer, idempotenter Commit im `RootSet` verbucht!

---

## 9. Bitemporale Zeitreise

| Granularität | Syntax | Mechanismus |
|---|---|---|
| **G1** | `Snapshot` | `Pin` auf aktiver `Seq` (Kurzzeit für laufende Transaktionen) |
| **G2** | `Commit` / `main~3` / `main@{ts}` | Direkter Zugriff auf `Commit.truth_root` (und Fallback falls Index fehlt) |
| **G3** | `<commit>:tx[i]` | Replay aus WAL-Journal innerhalb eines noch nicht gekappten Fold-Batches |
| **G4** | `history(cell_id)` / `blame` | Commit-Metadaten führen `touched_gids`-Bloom/Liste für schnelles Skipping |

**`branch_at(ref_spec, new_branch)`** forkt direkt ab jedem historischen Commit innerhalb des Retention-Horizonts; jenseits der Retention liefert die Engine `beyond_retention` mit dem ältesten verfügbaren Commit.

---

*Ende SPEC-04. Weiter: [SPEC-05 Indizes](SPEC-05-INDEXES.md)*
