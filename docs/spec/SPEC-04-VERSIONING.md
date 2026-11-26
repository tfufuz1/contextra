# SPEC-04 — Versionierung, Transaktionen, Merge und Kernel-Sidecars

**Version 3.0 · 26. November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Datenmodell](SPEC-02-DATA-MODEL.md) · [Indizes](SPEC-05-INDEXES.md) · [Durability](SPEC-07-DURABILITY.md)

---

## 1. Prolly-Tree

### 1.1 Eigenschaften

History-unabhängig (INV-1), strukturell geteilt, O(Δ·log n) Diff/Merge, augmentierbar.

### 1.2 Grenzfunktion (größenbewusst)

Parameter: `MIN = 2 KiB`, `TARGET = 8 KiB`, `MAX = 32 KiB`, `SPAN = TARGET − MIN = 6144`. Gezählt wird die unkomprimierte CBE-Länge der Einträge.

```rust
fn closes_node(level: u8, key: &[u8], entry_len: u32, size_incl: u32, seed: &[u8; 32]) -> bool {
    if size_incl >= MAX { return true; }
    if size_incl < MIN  { return false; }
    let mut h = blake3::Hasher::new_keyed(seed);
    h.update(&[level]); h.update(key);
    let x = u32::from_le_bytes(first4(h.finalize().as_bytes()));
    let threshold = ((u64::from(entry_len) << 32) / u64::from(SPAN)).min(u64::from(u32::MAX));
    u64::from(x) < threshold
}
```

Die Abschlusswahrscheinlichkeit ist proportional zur Eintragsgröße; die erwartete Knotengröße liegt damit unabhängig von der Eintragsgröße nahe `TARGET` (der Anteil erzwungener `MAX`-Schnitte liegt rechnerisch bei etwa e⁻⁵ ≈ 0,7 %). Eine Grenzwahrscheinlichkeit pro Schlüssel ohne Größenbezug ist unzulässig. Ein einzelner Eintrag > `MAX` ist verboten (große Werte werden ausgelagert). Die Entscheidung hängt nur von sortiertem Inhalt und `repo_seed` ab, ist ganzzahlig und damit plattformstabil. Nach einer Änderung wird neu geschnitten, bis eine alte Grenze wieder getroffen wird.

### 1.3 Knoten

```rust
pub struct InternalNode { pub level: u8, pub children: Vec<ChildEntry> }
pub struct ChildEntry {
    pub last_key: Bytes,              // größter Schlüssel im Teilbaum
    pub chunk_id: ChunkId,
    pub agg: NodeAggregates,          // count, bytes, token_sums, valid_min, valid_max
}
```

Kodierung in CBE; Aggregate sind reine Funktionen der Blattinhalte und gehen in den Hash ein. Physische Hints gehören nie in Knoten.

---

## 2. Commit, ChangeSet, RootSet

```rust
/// Unveränderlich. CommitId = ChunkId(kind = CommitHeader, CBE(self)).
pub struct CommitHeader {
    pub parents: SmallVec<[CommitId; 2]>,   // parents[0] = Branch-Vorgänger
    pub changes: ChunkId,                   // ChangeSet relativ zu parents[0]
    pub author: AgentIdentity,
    pub intent: Intent,                     // run_id, step_id, Begründung
    pub seq: Seq,
    pub hlc: Hlc,
    pub durability: Durability,
}

/// Deterministische Wahrheit.
pub struct RootSet {
    pub primary: ChunkId,     // CellId → CellRecord
    pub sequences: ChunkId,   // (SeqId, position, gid) → Token-Zählungen
    pub graph: ChunkId,       // G: / R:
    pub aliases: ChunkId,     // A: gid → CellId · C: CellId → gid · X: (ns, ExternalKey) → gid
    pub policies: ChunkId,    // Katalog, AclExpr, Producer
    pub conflicts: ChunkId,   // ConflictCells
    pub provenance: ChunkId,  // P: ProvRecord · E: Evidenz
}
// truth_hash = ChunkId(kind = RootSet, CBE(RootSet))

/// Nachträgliche, unveränderliche Bindung Commit → Zustand (Manifest-Record).
pub struct TruthBinding { pub commit: CommitId, pub roots: RootSet, pub truth_hash: Digest }
```

Ein Commit-Objekt wird **nie** nachträglich ergänzt. `RootSet`, `IndexManifest`-Zeiger und Signatur sind separate Records, die auf die `CommitId` verweisen. Optionale Ed25519-Signaturen signieren die `CommitId`; eine zweite, optionale Attestierung signiert die `TruthBinding`. Für `strict`-Commits ist die Commit-Signatur Pflicht, falls der Store einen Signaturschlüssel konfiguriert hat; ein Namespace mit `require_signature` lehnt unsignierte Commits ab.

### 2.1 Materialisierung

Nicht jeder Commit besitzt eine `TruthBinding`. Es gilt:

```
state(c) = state(m) ⊕ changes(c_1) ⊕ … ⊕ changes(c)
m = nächster materialisierter Vorfahr entlang parents[0]
```

- Der Fold MUSS alle `ChangeSet`s und `CommitHeader` durabel in `packs/truth` ablegen, bevor der Checkpoint über sie hinausrückt. Damit ist jeder Commit innerhalb der Retention unabhängig vom WAL adressierbar.
- **Materialisierungspflicht** für: Heads geschützter Branches (nach spätestens einem Fold), Tags/Checkpoints, Merge-Basen und -Ergebnisse sowie mindestens jeden 10 000. Commit je Linie. Fork und Savepoint erzwingen **keinen** synchronen Fold.
- `ChangeSet` ist eine nach `(Baum, Key)` sortierte Liste `Put{value} | Delete`. Sein Digest enthält weder `seq` noch `hlc` noch `born_seq` (diese setzt die Anwendung des Commits).

---

## 3. Branch-Zustand und MemDelta

```rust
pub struct BranchState {
    pub head: CommitId,
    pub head_seq: Seq,
    pub base: Arc<Materialized>,        // CommitId + RootSet
    pub delta: Arc<DeltaStack>,         // unveränderliche Schichten
    pub index: Arc<IndexView>,          // Manifest + RAM-Deltas mit Sichtbarkeitsgrenze
}
```

`DeltaStack` ist ein Stapel versiegelter Schichten plus eine persistente, strukturell geteilte geordnete Map. Jeder publizierte `BranchState` ist ein unveränderlicher Wert. Daraus folgen ohne Sonderfälle:

- **Fork O(1):** neuer Ref, der dieselben `Arc`s teilt — inklusive noch nicht gefalteter Änderungen.
- **Snapshot:** Klonen eines `Arc<BranchState>`.
- **Rollback auf Savepoint:** Zurücksetzen auf einen früheren Zustand. Liegt der Wert nicht mehr im RAM (Neustart), wird er aus `state(c)` rekonstruiert.

RAM-Index-Deltas sind append-only Arenen; jeder Zustand referenziert eine Sichtbarkeitsmaske (Länge plus Ausschlussmenge). Rollback setzt die Maske zurück.

**Fold-Trigger:** Delta > 32 MiB, > 10 000 Einträge, > 30 s, explizit oder beim Schließen. Ein Fold faltet einen Präfix versiegelter Schichten, schreibt Packs und Locators, registriert die `TruthBinding` im Manifest und publiziert einen neuen Zustand mit neuer Basis. Folds verschiedener Branches deduplizieren identische Objekte.

---

## 4. Transaktionen

### 4.1 Typestate-API

```rust
let tx: Txn<'_, ReadWrite> = branch.begin(Isolation::Snapshot)?;  // pinnt BranchState + Lease
let cell = tx.get(id)?;           // Lesen registriert untrennbar im Read-Set
tx.put_content(id, content)?;
tx.put_meta(meta)?;
tx.link(edge)?;
let receipt = tx.commit(CommitOpts::new("Fakt extrahiert")
    .idempotency(IdemKey::new(run, step, "put-fact-1")))?;
```

Lesezugriffe nutzen `Snapshot<'db>` ohne Read-Set. Keine öffentliche Funktion akzeptiert eine fremde Snapshot-Sequenz. `Txn` ist `!Sync`; `Drop` ohne Commit verwirft.

### 4.2 Commit-Protokoll

```
1  begin      BranchState pinnen, Lease (300 s, erneuerbar)
2  buffer     ChangeSet sammeln; lokale Prüfung (Schema, Vektoren, Provenienz, Limits)
3  prepare    außerhalb jedes Locks: CBE, Objekt-Hashes, request_digest, ChangeSet-Digest
4  sequence   Branch-Sequencer-Lock:
              a) Idempotenz prüfen und Schlüssel reservieren
              b) gegen den PENDING-Head validieren (inkl. eingereihter, noch unsynchronisierter Commits)
              c) bei disjunkter Bewegung Rebase, sonst rebase_failed
              d) in die WAL-Queue einreihen, Lock freigeben
5  append     WAL-Writer vergibt lsn, seq, hlc, setzt parents[0], berechnet CommitId, schreibt Frame
6  sync       gemäß Durability-Stufe (Group Commit)
7  publish    strikt in seq-Reihenfolge je Branch: Delta anwenden, Idempotenz-Eintrag,
              Work-Queue-Einträge, ArcSwap  ← Sichtbarkeitspunkt
8  ack        CommitReceipt { commit_id, seq, durability_reached }
9  fold       asynchron
```

Regeln:
- Ein `durable`/`strict`-Commit wird **erst nach** erfolgreichem Sync sichtbar. Ein nachfolgender `buffered`-Commit desselben Branches wartet in der Publikationsreihenfolge.
- Schlägt Schreiben oder Sync fehl, wird nichts Ausstehendes publiziert; der Store ist `repo_poisoned`. Der Ausgang dieser Commits ist für Aufrufer **unbekannt** und wird nach Recovery über den Idempotenz-Schlüssel geklärt.
- Abbruch des Aufrufer-Futures nach Schritt 4 bricht den Commit nicht ab.

### 4.3 Isolation

**Konfliktschlüssel** sind `(Baum, Key)` aller Schreibvorgänge, zusätzlich logische Schlüssel: `(ns, ExternalKey)`, `(subject, functional_predicate)`, `EdgeId`.

| Stufe | Validierung gegen Commits seit `tx.base` |
|---|---|
| Snapshot (Default) | Write-Write-Disjunktheit der Konfliktschlüssel; funktionale Prädikate |
| Serializable | zusätzlich Read-Set und Prädikatbereiche (Key-Ranges, Namespace-Scans, Nachbarschaften `G:(src, kind, *)`) |

Der Sequencer hält ein beschnittenes Änderungsfenster; ist `tx.base_seq < pruned_through` ⇒ `snapshot_too_old`. Suchabfragen über abgeleitete Indizes sind in Serializable-Transaktionen **nicht** als Prädikat erfassbar; die API verlangt dafür explizite Bereichsscans und weist `recall` in `Txn<ReadWrite>` mit Serializable ab.

---

## 5. Branches

| Muster | Zweck | Durability | TTL | Geschützt |
|---|---|---|---|---|
| `main` | kuratierte Wahrheit | `durable`/`strict` | — | ja |
| `release/<n>` | Freigabestand | `durable`/`strict` | — | ja |
| `dev/<n>` | Integration | `durable` | — | nein |
| `sess/<agent>/<run>` | Agentenarbeit mit Quoten | `durable`/`buffered` | 24 h (≤ 7 d) | nein |
| `mr/<n>` | Merge-Request, Gate-Report, Konflikte | `durable` | Abschluss + 7 d | nein |
| `spec/<uuid>` | Spekulation | `buffered` | 24 h | nein |
| `scratch/<id>` | Arbeitsgedächtnis | `volatile` | Sitzungsende | nein |
| `tag/<n>`, `checkpoint/<n>` | unveränderlicher Ref | — | — | pinnt GC |

- Direkte Commits auf geschützte Branches erfordern `ProtectedBranchWrite`, laufen durch G1, G2 und G3 im synchronen Modus und dürfen **keine** offene `ConflictCell` erzeugen (INV-7).
- Geschützte Branches lehnen `volatile` und `buffered` ab.
- **`volatile`:** nicht im WAL, geht bei Neustart verloren. Durabler Zustand referenziert volatile Commits nie als Eltern: Ein Merge aus `scratch/*` schreibt alle Inhalte als reguläre Mutationen und vermerkt die Quelle nur im `intent`.
- **Quoten** je Session: Bytes, Cells, Schreibrate, Embedding-Aufrufe, Tokens ⇒ `quota_exceeded`.
- **Spekulation:** N Branches vom selben Commit; Gewinner wird gemerged, Rest verworfen.

### 5.1 Savepoints

`savepoint(name)` speichert `(commit_id, seq)` durabel (Frame `BranchOp`). `rollback_to(name)` ist ein CAS auf den Ref und zugleich:

1. Idempotenz-Einträge mit Commits jenseits des Savepoints werden **widerrufen**, damit Wiederholungen erneut ausführen.
2. Ausstehende Work-Queue-Einträge dieser Commits werden storniert.
3. Index-Sichtbarkeitsmasken werden zurückgesetzt.
4. Der Reflog behält den alten Head bis zum Ablauf des Reflog-Fensters.

Auf geschützten Branches ist Rollback nur als Admin-Operation zulässig; der Regelfall ist ein kompensierender Commit.

---

## 6. Merge

### 6.1 Basis und Verfahren

Merge-Basis ist der niedrigste gemeinsame Vorfahr. Bei mehreren Basen werden diese rekursiv zu einer virtuellen Basis zusammengeführt; Konflikte darin gelten als „beidseitig geändert". Der Diff überspringt hash-gleiche Teilbäume. Auswertungsreihenfolge: `policies` → `aliases` → `primary` → `provenance` → `graph` → `sequences` → `conflicts`.

Allgemeine Dreiwegregel: nur eine Seite geändert ⇒ übernehmen (unter Gate-Vorbehalt); beide identisch ⇒ übernehmen (Vergleich ohne `born_seq`); sonst Tabelle.

| Gegenstand | Regel bei beidseitiger Änderung |
|---|---|
| Content verschieden | `ConflictCell`. Namespace-Option `text = "diff3"`: überlappungsfreie Hunks werden auf **ungeschützten** Zielen automatisch vereinigt; auf geschützten Zielen nur als Vorschlag in der `ConflictCell` |
| `attrs`, disjunkte Keys | feldweise |
| `attrs`, gleicher Key | `Tags`: Dreiweg-Mengenmerge · `Counter` mit `sum`: `base + Δours + Δtheirs` · sonst Konflikt |
| `valid` | disjunkte Grenzen kombinieren; gleiche Grenze verschieden ⇒ Konflikt; entstehendes leeres Intervall ⇒ Konflikt |
| `trust` | `min(ours, theirs)`; eine Hochstufung braucht stets ein G7-Attest |
| `taint` | `base ∪ Zugänge`; eine Entfernung gilt nur mit Admin-Attest |
| `acl` | Konjunktion der Klauseln (nie Ausweitung) |
| `retire` gegen Änderung | Konflikt; `purge`/Expunge gewinnt immer |
| `parent`, `kind`, `level`, `claim` | Konflikt |
| Kanten | je `EdgeKind`: `add-wins` (Default), `remove-wins`, `manual` |
| `provenance` / Evidenz | Mengenvereinigung nach [SPEC-02](SPEC-02-DATA-MODEL.md) §6.2 (konfliktfrei) |
| `sequences` | Vereinigung; Position ist durch `gid` eindeutig |
| `aliases` `X:` | gleicher `ExternalKey`, verschiedene `Gid` ⇒ Konflikt `DuplicateExternalKey` |
| Katalog | disjunkte Einträge vereinigen, sonst Konflikt |
| `Summary` mit beidseitig geänderten Quellen | nicht mergen; `stale`, neuer Job |
| Embeddings | liegen nicht in der Wahrheit; **nie** mitteln |
| `custom(wasm)` | `wasmtime`, Fuel 10 Mio., 16 MiB, kein WASI, keine Floats, Modul per Hash gepinnt; Fehler ⇒ Konflikt |

Nach dem Zusammenführen prüft die Engine funktionale Prädikate **auf dem Ergebnis**, auch wenn beide Seiten verschiedene `Gid`s geschrieben haben.

### 6.2 ConflictCell

```rust
pub struct ConflictCell {
    pub id: ConflictId,               // H(base, ours, theirs, conflict_key)
    pub key: ConflictKey,
    pub kind: ConflictKind,
    pub base: Option<ValueRef>, pub ours: Option<ValueRef>, pub theirs: Option<ValueRef>,
    pub status: ConflictStatus,       // Open | Resolved { choice, principal, commit }
}
```

Offene Konflikte werden nur in `mr/<n>` materialisiert. `resolve(id, TakeOurs | TakeTheirs | Merged(value))` ist ein Commit auf `mr/<n>`. Inhaltliche Widersprüche ohne Schreibkollision sind keine `ConflictCell`s, sondern `Contradicts`-Kanten.

### 6.3 Merge-Gates

| Gate | Prüfung | Default |
|---|---|---|
| **G1** Schema & Integrität | Typen, Kardinalitäten, funktionale Prädikate, Vorwärts-/Rückwärtskante, keine neuen hängenden Kanten, Alias-Bijektion | blockierend |
| **G2** Provenienz & Vertrauen | Provenienz vorhanden, Signatur falls Pflicht, `trust ≥ min_main_trust` | blockierend |
| **G3** Widerspruch | deterministischer Konfliktscan; optional `Judge` | blockierend bei direktem Widerspruch gegen `Verified`/`Curated`, sonst Warnung |
| **G4** Blast Radius | geänderte + entfernte Cells / lebende Cells des Ziels je betroffenem Namespace > 1 % **und** absolut > 100 | Review (G7) |
| **G5** Grounding | `Fact`/`Summary`: Spannen und `quote_hash`, referenzierte Quellen sichtbar | blockierend für `GL0`-Behauptungen, sonst Quarantäne |
| **G6** Taint & Injection | `INJECTION_SUSPECT`, `EXTERNAL`, Trust-Eskalation, ACL-Ausweitung gegenüber Quellen | blockierend |
| **G7** Review | externer `Reviewer`; Trust-Promotion, geschützte Namespaces | konfigurierbar |

Optionaler, nicht nummerierter Beratungscheck **A1 Retrieval-Regression** (Golden Queries, Top-k-Überlappung, nDCG-Delta): warnt; kann je Namespace blockierend geschaltet werden.

```rust
pub struct GateReport {
    pub target_head: CommitId, pub source_head: CommitId, pub base: CommitId,
    pub results: Vec<GateResult>,     // Pass | Warn | NeedsReview | Fail { cells, reason }
    pub policy_hash: Digest,
}
```

**Atomarität:** Der Merge-Commit wird nur ausgeführt, wenn `target_head`, `source_head` und `policy_hash` unverändert sind (CAS im Sequencer). Andernfalls werden die Gates inkrementell neu bewertet. Gates laufen außerhalb des Sequencer-Locks auf dem virtuellen Ergebnis. Nichtdeterministische Urteile (`Judge`, Reviewer) werden mit Eingabe-Digest im Report festgehalten und nur bei identischem Digest wiederverwendet. Timeout oder Fehler eines blockierenden Gates ⇒ `NeedsReview`, nie `Pass`. Ein Admin-Bypass erzeugt einen eigenen, benannten Audit-Eintrag.

### 6.4 Ergebnis

```rust
pub enum MergeOutcome {
    Merged(CommitId), FastForward(CommitId),
    GateRejected(GateReport), NeedsReview { mr: BranchName, report: GateReport },
    Conflicts { mr: BranchName, conflicts: Vec<ConflictId> },
}
```

Auch ein Fast-Forward in ein geschütztes Ziel durchläuft alle Gates.

---

## 7. Zeitreise

| Stufe | Syntax | Mechanismus |
|---|---|---|
| **Z1** | aktiver `Snapshot` | gepinnter `BranchState` |
| **Z2** | `AS OF COMMIT c`, `main~3` | `state(c)` nach §2.1 |
| **Z3** | `AS OF TIMESTAMP t` | letzter Commit der First-Parent-Linie mit `hlc ≤ t` |
| **Z4** | `history(cell)`, `blame` | `touched_gids`-Filter je ChangeSet |

`VALID AT v` ist orthogonal kombinierbar. `branch_at(ref, name)` forkt von jedem Commit innerhalb der Retention; sonst `beyond_retention` mit ältestem verfügbarem Commit. Reproduzierbarkeit eines Recalls erfordert `(commit, catalog, index_manifest, derivation producer_ids, now, Parameter)`; fehlt ein früheres Index-Manifest, kann die Rangfolge abweichen — Explain weist das aus. Expungte Inhalte liefern `expunged`.

---

## 8. Kernel-Sidecars (nicht versioniert)

Sidecars sind abgeleitete Zustände des WAL plus periodische Checkpoints.

> **Checkpoint-Regel:** Ein WAL-Segment darf erst entfallen, wenn Truth-Fold, Idempotenz-, Work-Queue- und Derivations-Checkpoint alle seine Frames abdecken und im Manifest registriert sind.

### 8.1 Idempotenz

```
Key   = (scope, run_id, step_id, client_key)       scope = (principal, target_branch_id)
Value = { request_digest, commit_id, seq, result_summary, expires_at, state }
state = Reserved | Committed | Revoked
```

- `request_digest` wird über die **kanonische Client-Anfrage vor jeder ID-Vergabe** gebildet (ohne `Gid`, Zeitstempel, Lease-Token). Der Schlüssel enthält die Argumente **nicht** — sonst wäre eine Abweichung nicht erkennbar.
- Reservierung erfolgt im Sequencer; ein paralleler gleicher Schlüssel wartet auf den Ausgang.
- `Committed` + gleicher Digest ⇒ Erfolg mit `replayed = true`. Anderer Digest ⇒ `idempotency_mismatch`.
- TTL 7 Tage. `volatile`-Branches halten Einträge nur im RAM.

### 8.2 Work Queue

```rust
pub struct WorkEntry {
    pub task_id: TaskId,              // H(task) → doppeltes Enqueue ist wirkungslos
    pub task: WorkTask,
    pub priority: WorkPriority,       // IndexBuild > Embed > Judge > Summarize > Consolidate > Background
    pub origin: (BranchId, Seq),
    pub status: WorkStatus,           // Pending | Leased | Done | Failed
    pub attempts: u8, pub max_attempts: u8,   // Default 3
}
pub struct Lease { pub token: u64, pub owner: WorkerId, pub expires: MonoInstant }
```

| Übergang | Persistenz |
|---|---|
| Enqueue | atomar **im selben Frame** wie der auslösende Commit |
| Lease | nur RAM; Lease-Token monoton |
| Versuchszähler | `WorkOp::Attempt`, `buffered` geschrieben **vor** Auslieferung, damit Absturzschleifen enden |
| Heartbeat | nur RAM |
| Complete | Ergebnis und Statuswechsel **im selben Frame** |
| Neustart | alle Leases verfallen ⇒ `Pending` |

Zustellung ist at-least-once. `complete(task_id, token, result)`: Mit abgelaufenem Token werden reine Derivationen akzeptiert, sofern noch keine existiert (erster gewinnt); Ergebnisse, die Wahrheit ändern, werden mit `work_task_expired` abgewiesen. Derivationen gehen in den Derivation Cache; Fakten laufen über einen Session-Branch und die Gates. Leasing erzeugt nie Commits.

---

## 9. Abnahme

- Property: History-Unabhängigkeit; Diff/Merge-Roundtrip; Merge ohne Konflikte ist in `ours/theirs` kommutativ (bezogen auf den `RootSet`).
- Verteilungstest: mittlere Knotengröße und Anteil erzwungener Schnitte gegen §1.2.
- Crash-Tests: Idempotenz über Neustart und nach WAL-Recycling; Rollback + Retry; Work-Queue-Poison-Task.
- `loom`: Reservierung, Publikationsreihenfolge, Merge-CAS.

---

*Weiter: [SPEC-05 Indizes](SPEC-05-INDEXES.md)*
