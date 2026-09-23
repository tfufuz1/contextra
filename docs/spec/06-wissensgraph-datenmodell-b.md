---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "06b"
---
### 6.5 Traversal-Semantik

PathRAG wird um einen optionalen Hyperkanten-Expansionsschritt ergänzt: Beim Erreichen eines Knotens während der
Forward-Push-Traversierung werden zusätzlich alle `RoleBinding`-Partner als „virtuelle" Nachbarn mit
rollenspezifischem Gewichtsabschlag (Startwert `0.85`) eingespeist.

### 6.6 Integrationshindernisse H1–H6 und ihre verbindliche Lösung

Diese sechs Hindernisse benennen nicht nur, *dass* eine Integration möglich ist, sondern *welches bestehende
Invariant unter Druck gerät* und wie es gewahrt bleibt.

#### H1 — RCU-Snapshot-Inkonsistenz zwischen CSR und Hyperkanten-Sekundärindex

**Problem:** Separater Hyperkanten-Index außerhalb von `GraphInner` → Zeitfenster für inkonsistenten Zustand.

**Lösung (verbindlich):** Der Hyperkanten-Index wird **Teil von `GraphInner`** selbst — automatisch vom
`ArcSwap`-Swap miterfasst. `GraphInner::estimate_memory_bytes()` MUSS Hyperkanten einschließen und capacity-basiert rechnen; die Budgetprüfung
von `compact_async` verwendet `estimate_compaction_peak_bytes()` (§6.3, Snapshot-Regel S1: kein `scc::HashMap` im Snapshot).

#### H2 — Kanonisches Multi-Key-Locking zur Deadlock-Prävention

**Problem:** `relate_n_ary()` mit N Teilnehmern muss N Entitäten gleichzeitig unter `kv_locks` halten.
Naives Lock-Ordering → Deadlock bei überlappenden, unterschiedlich sortierten Mengen.

**Lösung (verbindlich):**

```rust
impl CsrGraph {
    pub fn relate_n_ary(
        &self,
        predicate: EdgeType,
        participants: &[RoleBinding],
        doc_id: DocId,
    ) -> Result<HyperEdgeId, GraphMutationError> {
        if participants.len() < 2 {
            return Err(GraphMutationError::InsufficientParticipants(participants.len()));
        }

        // 1. Kanonische Sortierung (H2) — Grundlage der Deadlockfreiheit.
        let mut entities: Vec<EntityId> = participants.iter().map(|p| p.entity).collect();
        entities.sort_unstable_by_key(|e| e.0);
        entities.dedup();
        // Hash ausschließlich über KvKeyLocks (feste Seeds, eine Instanz; §5.2a). NIE `RandomState::new()` hier.
        let key_hashes: Vec<u64> = entities.iter().map(|e| self.kv_locks.key_hash(e)).collect();

        // 2. Multi-Key-Lock in sortierter Shard-Reihenfolge.
        let _guards = self.kv_locks.acquire_multi_sorted(&key_hashes)
            .map_err(|_| GraphMutationError::LockAcquisitionTimeout)?;

        // 3. Atomare LSM-Schreibung: Primär + Sekundärindex für JEDEN Teilnehmer.
        let id = HyperEdgeId(self.next_hyperedge_id());
        let hyperedge = HyperEdge {
            id, predicate, participants: Arc::from(participants), weight: 1.0,
            tx_valid_from: None, tx_valid_to: None,
            business_valid_from: None, business_valid_to: None,
            source_doc_id: Some(doc_id),
        };
        self.persist_hyperedge_atomic(&hyperedge)?;

        // 4. RCU-Registrierung (H1).
        self.register_in_rcu_snapshot(&hyperedge)?;

        Ok(id)
    }
}
```

**Loom-Testpflicht (AK-3):** `crates/contextra-graph/tests/loom_relate_n_ary.rs` mit überlappenden,
unterschiedlich geordneten Mengen. **Shard-Stabilität (AK-14, Fassung 2.1):** siehe §5.2a — dieselbe Entität muss in
jedem Aufruf denselben Shard treffen, sonst ist der Ausschluss wirkungslos.

#### H3 — `SignalKind` ist ein geschlossenes Enum

**Lösung (verbindlich):** Hyperkanten-Treffer fließen als zusätzliche Kandidaten **in `SignalKind::Graph`** ein —
PathRAG liefert bereits ein Graph-Signal; Hyperkanten-Expansion ist ein interner Erweiterungsschritt der Pfadsuche.
`SignalKind` bleibt strukturell unverändert. Diff-Test-Pflicht: `signal_kind_no_new_variant.rs`.

#### H4 — FlatBuffers-Schemaerweiterung erfordert aktives CI-Drift-Gate

**Lösung (verbindlich, harte Vorbedingung):** Das FlatBuffers-CI-Drift-Gate MUSS produktiv und grün sein,
**bevor** das `HyperEdge`-FlatBuffers-Schema gemerged wird — per CI-Job-Abhängigkeit erzwungen (`needs: [flatbuffers-drift-gate]`).

#### H5 — Cascade-Invalidierung: hartes Fan-out-Limit, persistente idempotente Queue

**Änderung Fassung 2.1:** Fassung 2 gab im Überlauffall `Err(PartialCascadeQueued(proof))` zurück und ließ die
Hintergrundqueue offen. Ein Teilerfolg ist kein Fehler, und ein „Beweis" vor Abschluss der Löschung ist irreführend.
Ist-Zustand im Repo: Die zurückgestellten Hyperkanten liegen nur in einer In-Memory-`VecDeque`; außerhalb von Tests
habe ich keinen `enqueue`-Aufruf gefunden, sie gehen bei einem Absturz verloren oder werden nie befüllt.

**Lösung (verbindlich, Pflichtbestandteil):**

1. **Ein atomarer WAL-Commit** enthält die synchronen Tombstones (die ersten `θ` Hyperkanten in aufsteigender
   `HyperEdgeId`-Reihenfolge, §4(3)) **und** die Queue-Einträge für den Rest. Es gibt keinen Zustand „Rest weder
   tombstoniert noch in der Queue".
2. **Persistente Queue:** LSM-Präfix `__graph:cascade_queue:{doc_id_be}:{hyperedge_id_be}` (Big-Endian, damit der
   Präfix-Scan je Dokument geordnet ist). Ein `put` auf denselben Schlüssel ist idempotent.
3. **Worker** (Start beim Öffnen und bei Benachrichtigung): Präfix-Scan, Batches von `CASCADE_BATCH = 128`. Je
   Hyperkante: `tombstone_hyperedge_atomic(id)` **und** Löschen des Queue-Schlüssels in **einer** Transaktion; Locks
   nach H2 nur für die Dauer eines Batches, nie über Batches hinweg (RCU-Lesepfad bleibt frei).
4. **Idempotenz:** `tombstone_hyperedge_atomic` setzt `tx_valid_to` nur, wenn es `None` ist, und schreibt sonst
   nichts. Ein wiederholter Lauf auf dasselbe `doc_id` erzeugt keine doppelten Tombstones.
5. **`DeletionProof`** wird erst ausgestellt, wenn für das `doc_id` kein Queue-Eintrag mehr existiert: im Report
   (`queued_for_background == 0`) oder beim Abschluss durch den Worker (persistiert unter
   `__graph:cascade_proof:{doc_id_be}`, abrufbar über `cascade_status`). Nie für eine unvollständige Löschung.
6. **Fehlersemantik:** `Ok(CascadeReport)` auch bei Teilverarbeitung; `Err` nur für echte Fehler (Lock, I/O,
   Budget). Die Variante `GraphMutationError::PartialCascadeQueued` entfällt (§6.7).

```rust
pub const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT: usize = 1_000;
pub const CASCADE_BATCH: usize = 128;

pub struct CascadeTicket(pub u64);

pub struct CascadeReport {
    pub tombstoned_synchronously: usize,
    pub queued_for_background: usize,
    pub ticket: Option<CascadeTicket>,                       // Some, solange queued_for_background > 0
    pub deletion_proof: Option<contextra_crypto::DeletionProof>, // nur bei vollständiger Löschung
}

pub struct CascadeStatus {
    pub pending: usize,
    pub deletion_proof: Option<contextra_crypto::DeletionProof>,
}

pub fn cascade_invalidate_hyperedges_for_superseded_doc(
    graph: &CsrGraph,
    doc_id: DocId,
    fanout_limit: usize,
) -> Result<CascadeReport, GraphMutationError> {
    let mut affected = graph.hyperedges_for_doc(doc_id);
    affected.sort_unstable();                          // deterministisch (§4(3))
    let split = affected.len().min(fanout_limit);      // split ≤ len: `split_at` kann nicht paniken
    let (sync_part, async_part) = affected.split_at(split);

    let mut tx = graph.begin_cascade_tx(doc_id)?;      // EIN WAL-Commit für beide Teile
    for id in sync_part { tx.tombstone(*id)?; }        // idempotent
    for id in async_part { tx.enqueue(doc_id, *id)?; } // idempotent (put auf festen Schlüssel)
    tx.commit()?;

    let queued = async_part.len();
    Ok(CascadeReport {
        tombstoned_synchronously: sync_part.len(),
        queued_for_background: queued,
        ticket: (queued > 0).then(|| graph.cascade_ticket(doc_id)),
        deletion_proof: if queued == 0 { Some(graph.issue_deletion_proof(doc_id)?) } else { None },
    })
}

pub fn cascade_status(graph: &CsrGraph, doc_id: DocId) -> Result<CascadeStatus, GraphMutationError> {
    unimplemented!() // pending = Anzahl Queue-Einträge; Proof aus __graph:cascade_proof:{doc_id_be}, falls vorhanden
}
```

**Testpflicht (AK-6, AK-11):** `hyperedge_cascade_fanout.rs` (Umschalten bei > θ) und
`hyperedge_cascade_crash_recovery.rs`: Absturz nach dem Commit und vor dem ersten Worker-Batch, Neustart, danach
sind alle Hyperkanten des Dokuments genau einmal tombstoniert und die Queue ist leer; zweiter Lauf ist ein No-op.

#### H6 — Community-Detection/Leiden sieht Hyperkanten nicht

**Lösung (verbindlich für Sichtbarkeit):**

```rust
pub struct CommunityDetectionConfig {
    pub resolution_gamma: f32,
    /// H6: sichtbares Unvollständigkeits-Flag, Default false.
    pub hyperedges_included: bool,
}

pub struct CommunityAssignment {
    pub node_to_community: AHashMap<EntityId, u32>,
    pub hyperedges_included: bool, // 1:1 aus Config, im Report sichtbar
}
```

**Stern-Expansion als Zielarchitektur:** Der Hypergraph wird in einen bipartiten Graphen überführt. Jede
Hyperkante wird als künstlicher Knoten `v_e` repräsentiert; es entstehen nur binäre Kanten mit $O(|e|)$
Skalierung (statt $O(|e|^2)$ bei Cliquen-Expansion). `StarExpansionIterator` (§7.5) erzeugt die virtuellen Kanten
on-the-fly, ohne die Hyperkanten zu klonen.

**Konvention K (verbindlich ab Fassung 2.1).** Sei $N = |e| \ge 2$ und $w = w(e)$.

- *Referenz-Clique* $K(e)$: jedes ungeordnete Teilnehmerpaar erhält $p(e) = w / \binom{N}{2}$. Die Gesamtmasse
  der Hyperkante ist damit $w$, unabhängig von $N$ (große Hyperkanten dominieren nicht). Für $N = 2$ ergibt sich
  die binäre Kante mit Gewicht $w$.
- *Sternkante* je Teilnehmer $u \leftrightarrow v_e$: $a(e) = N \cdot p(e) = \dfrac{2\,w(e)}{|e| - 1}$
  (`star_weight`, siehe unten).

**Satz (Schur-Komplement).** Eliminiert man $v_e$ aus der Laplace-Matrix des Sterns, entsteht exakt die
Laplace-Matrix von $K(e)$.
*Beweis.* Teilnehmerblock $L_{pp} = aI$, Kopplung $L_{pv} = -a\mathbf 1$, $L_{vv} = Na$. Schur-Komplement:
$L_{pp} - L_{pv}L_{vv}^{-1}L_{vp} = aI - \tfrac{a}{N}\mathbf 1\mathbf 1^{\top}$. Die Clique mit Paargewicht $p$ hat
$p(NI - \mathbf 1\mathbf 1^{\top})$. Beide sind gleich genau dann, wenn $a = Np$. $\square$
Numerisch geprüft für $N \in \{2,3,5,8,20\}$ (maximale Abweichung $\approx 10^{-16}$).

**Was der Satz nicht besagt:** Er gilt für Laplace-basierte Größen (effektive Leitfähigkeit, Diffusion, PPR-artige
Prozesse). Die **Modularität** $Q$ auf dem Sterngraphen ist nicht identisch mit $Q$ auf der Clique, weil `v_e`
Grad trägt (Grad $= N\cdot a$). Der Resolution-Parameter $\gamma$ bleibt tuning-pflichtig, `hyperedges_included`
bleibt `false`, bis die Validierung gegen Benchmark-Netzwerke abgeschlossen ist.

**Korrektur gegenüber Fassung 2:** Dort stand $a = w/(|e|-1)$ mit dem „Beweis", die Teilnehmergradsumme des Sterns
$|e|\,w/(|e|-1)$ entspreche der Clique. Bei Paargewicht $w$ hat die Clique aber die Gradsumme $|e|(|e|-1)\,w$
(N=3: $1{,}5\,w$ gegen $6\,w$); gleich sind beide nur bei $|e|=2$. Die alte Formel entspricht Konvention K mit
halbem Hyperkantengewicht. Der Ist-Zustand im Repo verwendet $a = w$ ohne Normierung. **Umstellung:** Konvention K
ist eine Modellierungsentscheidung (§A2.4 Nr. 6). Wer die Größenabhängigkeit der Masse will, wählt die
Zhou-Konvention $p = w/(|e|-1)$, dann ist $a = N\,w/(|e|-1)$; in beiden Fällen gilt $a = N p$, und der Test
prüft gegen die konfigurierte Konvention.

```rust
/// Konvention K: a(e) = 2·w(e)/(|e|−1). `None` bei |e| < 2 oder nicht endlichem Gewicht.
pub fn star_weight(w: f32, n: usize) -> Option<f32> {
    if n < 2 || !w.is_finite() { None } else { Some(2.0 * w / (n as f32 - 1.0)) }
}
```

**Testpflicht (AK-10):** `crates/contextra-graph/tests/star_expansion_equals_clique.rs` — (a) für zufällige
Hyperkanten ($N \in [2, 64]$) stimmt das Schur-Komplement des Sterns mit der Clique-Laplace-Matrix überein
(relative Toleranz $10^{-9}$, f64-Referenz), (b) $N = 2$ ergibt die binäre Kante, (c) die Iterationsreihenfolge ist
deterministisch, (d) der Iterator klont keine Hyperkante (Allokationszähler).

### 6.7 Hyperkanten-Fehler-Enum

```rust
#[derive(Debug, thiserror::Error)]
pub enum GraphMutationError {
    #[error("lock acquisition timed out")]
    LockAcquisitionTimeout,
    // Fassung 2.1: `PartialCascadeQueued` entfällt. Teilverarbeitung ist `Ok(CascadeReport)` (§6.6 H5).
    #[error("role binding invalid: {0}")]
    RoleBindingInvalid(String),
    #[error("rcu snapshot reclamation pending, retry")]
    EpochReclamationPending,
    #[error("hyperedge requires >= 2 participants, got {0}")]
    InsufficientParticipants(usize),
}
```

---

<a id="7-retrieval"></a>
