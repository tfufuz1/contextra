---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "90a"
---
# Anhang B — Begründungen, Ist-Zustand, Literatur und Restrisiken je Maßnahme (nachrangig)

> **Rang:** nachrangig zu §0–§20. Dieser Block war in Fassung 2 als eigener Abschnitt „Mikrofeingranulare
> Schnittstellenspezifikation & Systemoptimierung für Contextra Cognitive OS" zwischen §19 und §20 eingebettet und
> trug Nummern (5.1, 6.3.1, …), die mit dem Hauptteil kollidierten. Ab Fassung 2.1 tragen sie das Präfix `B.`.
> Der Anhang liefert Ist-Zustand, Literatur, Migrationspfad und Restrisiken. **Rust-Skizzen in diesem Anhang sind
> nicht normativ**, wo sie von §5–§10 abweichen; dort gelten §5–§10. Korrigierte Stellen sind mit „[v2.1]"
> markiert. Verweise „§4(n)" und „Invariante n" bezeichnen die Invarianten aus §4.i.

Die vorliegende Spezifikation definiert die mikrofeingranulare Architektur für das Contextra Cognitive OS. Die Analyse adressiert die Beseitigung struktureller Flaschenhälse in den Bereichen Wissensgraph-Modellierung, Contextual-Bandit-Routing, Cache-Kontention, Vektorindex-Traversierung, LSM-Storage-Engine, Inferenz-Brücken und kryptographischer Sicherheit. Die Lösungsarchitekturen sind so konzipiert, dass sie direkt in deterministischen, threadsicheren Rust-Code überführt werden können, ohne die systemweiten Invarianten zu verletzen.

## B.5.1 N-äre Hyperkanten im Wissensgraphen (`crates/contextra-graph`)

Die Repräsentation n-ärer Relationen in herkömmlichen Graphdatenbanken führt häufig zu einem semantischen Informationsverlust, wenn komplexe Ereignisse in binäre Subjekt-Prädikat-Objekt-Tripel zerschnitten werden. Die nachfolgenden Spezifikationen definieren die Integration von Hyperkanten in die bestehende Compressed Sparse Row (CSR) Struktur.

### B.5.1.1 — H1: RCU-Snapshot-Integration

**Ist-Zustand im Repo:** `crates/contextra-graph/src/csr.rs` berechnet die Speicherschätzung des asynchronen `compact()`-Prozesses ausschließlich auf Basis der binären Adjazenzliste, während Hyperkanten als getrennte Datenstruktur außerhalb der atomaren Swap-Grenze modelliert werden.

**Referenzierte Literatur:**

- Yan et al., 2023, "Hypergraph Database Storage", arXiv:2302.06119 — Spezifiziert die Repräsentation von n-ären Relationen in speichereffizienten Bipartit-Graphen zur Optimierung von Subhypergraph-Matching-Verfahren.

- Guo et al., 2024, "HyperGraphRAG", arXiv:2503.21322 — Belegt, dass die Isolation von Entitäten und Hyperkanten in parallelen Speicherstrukturen die Retrieval-Genauigkeit in RAG-Systemen signifikant erhöht.


**Mathematische/algorithmische Spezifikation [v2.1 korrigiert]:** Die Speicherkosten des RCU-Snapshots müssen
streng deterministisch berechenbar sein, um Allokationsausfälle zu verhindern. Mit $s_R = \text{size\_of}::<\text{RoleBinding}>() = 16$
(nicht 8), $s_I = \text{size\_of}::<\text{HyperEdgeId}>() = 8$, $s_H = \text{size\_of}::<\text{HyperEdge}>()$ (mehr als 100 Byte,
nicht 32) und der Tabellenschranke $T(c) = \text{buckets}(c)\cdot(\text{size\_of}::<(K,V)>() + 1) + 16$ mit
$\text{buckets}(c) = \text{nextpow2}(\lfloor 8c/7 \rfloor + 1)$ gilt

$$S_{\text{total}} = S_{\text{adj}} + \sum_{e \in E_H}\bigl(s_H + \vert e\vert \cdot s_R\bigr) + \sum_{v}\deg_H(v)\cdot s_I + \vert E_H\vert\cdot s_I + \sum_{m} T_m(\text{cap}_m)$$

wobei $\text{cap}_m$ die **Kapazität** (nicht die Länge) jeder Tabelle $m \in \{\text{node},\, H,\, \text{idx}\}$ ist. Die Spitze
beim Rebuild ist $S_{\text{peak}} = S_{\text{total}}^{\text{alt}} + S_{\text{privat}}^{\text{neu}}$ mit vorbelegten Tabellen
($\text{cap} = \text{len}$); geteilte Payloads ($\text{Arc}$) zählen einmal. Normativ ist §6.3.

**Rust-Schnittstelle:** [v2.1] Die frühere Skizze (`scc::HashMap` im Snapshot, `&'a [RoleBinding]`,
`capacity()*32`) ist entfallen. Normativ sind §6.3 (`GraphInner`, `estimate_*`) und §6.4 (`HyperEdge`,
`ArcSlice`, `HyperEdgeView`). Der Snapshot enthält keine Container mit innerer Mutabilität (Snapshot-Regel S1).

**Lock-/Nebenläufigkeitsmodell:** Der Lesezugriff ist durch die Verwendung von `arc_swap::ArcSwap<GraphInner>` vollständig lock-frei ($O(1)$). Modifikationen berechnen einen neuen `GraphInner`-Zustand im Hintergrund und publizieren diesen atomar.

**Invarianten-Nachweis:** Die Definition erfüllt Invariante 5 (Speicherbudget-Transparenz), da die exakte Byte-Berechnung des Hyperkanten-Indizes vor der Kompaktierung evaluiert wird. Invariante 7 (Zero-Copy) wird gewahrt, indem `HyperEdgeView` einen `ArcSlice<RoleBinding>` hält, der sich den `Arc<[RoleBinding]>` des Snapshots teilt (Referenzzähler-Inkrement statt Kopie, §6.4). [v2.1]

**Migrationspfad:** Ein Migrations-Job muss bestehende `GraphInner`-Strukturen deserialisieren und die leeren `hyperedges`-Maps initialisieren, bevor die RCU-Pointer ausgetauscht werden.

**Restrisiken/offene Fragen:** [v2.1] Jede Veröffentlichung klont die Map-Strukturen (O(Hyperkanten + Entitäten) Referenzzähler-Inkremente, keine Payload-Kopie). Wachstum einer nicht vorbelegten Tabelle kann die Schätzung um die Verdopplungsdifferenz überschreiten; deshalb MUSS der Rebuild vorbelegen (§6.3).

### B.5.1.2 — H2: Deadlock-freies Multi-Key-Locking

**Ist-Zustand im Repo:** Naives Locking über `kv_locks` für eine Hyperkante mit $N$ Teilnehmern führt zu zyklischen Wartebedingungen, wenn zwei überlappende Transaktionen die Locks in unterschiedlicher Reihenfolge anfordern.

**Referenzierte Literatur:**

- Sarkar et al., 2020, "LSM-tree compaction", arXiv:2202.04522 — Analysiert Nebenläufigkeitskontrollen in skalierbaren Speicherarchitekturen und totale Ordnungen in Lock-Hierarchien.


**Mathematische/algorithmische Spezifikation:** Die Deadlock-Freiheit bei der Belegung von $N$ unabhängigen Schlüsseln erfordert die Einhaltung einer totalen Ordnung $\leq_{L}$ über die Sperrenmenge $L$. Sei $h: \text{EntityId} \to \mathbb{N}$ eine eindeutige Hash-Abbildung auf den Shard-Index. Für eine Menge von Entitäten $E = \{e_1, \dots, e_N\}$ wird die Sperrsequenz $S = \text{sort}(\{h(e_i) \mid e_i \in E\})$ generiert. Doppelte Shard-Indizes werden entfernt. Die Komplexität für die Akquise beträgt im Worst-Case $O(N \log N)$ für die Sortierung und $O(K)$ für das Sperren, wobei $K \leq N$ die Anzahl der betroffenen Shards ist.

**Rust-Schnittstelle (normativ):**

Rust

```
use std::sync::RwLockWriteGuard;

#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("lock poisoned")]
    Poisoned,
    #[error("lock acquisition timed out")]
    Timeout,
}

pub struct MultiKeyGuard<'a> {
    _guards: Vec<RwLockWriteGuard<'a, ()>>,
}

impl KvKeyLocks {
    pub fn acquire_multi_sorted(&self, sorted_key_hashes: &[u64]) -> Result<MultiKeyGuard<'_>, LockError> {
        let mut shard_indices: Vec<usize> = sorted_key_hashes.iter()
            .map(|&h| (h & self.shard_mask) as usize)
            .collect();
        shard_indices.sort_unstable();
        shard_indices.dedup();

        let mut guards = Vec::with_capacity(shard_indices.len());
        for idx in shard_indices {
            guards.push(self.shards[idx].write().map_err(|_| LockError::Poisoned)?);
        }
        Ok(MultiKeyGuard { _guards: guards })
    }
}
```

**Lock-/Nebenläufigkeitsmodell:** Durch die Erzwingung einer monoton steigenden Erwerbsreihenfolge über die physischen Shard-Indizes wird die Entstehung von Zyklen im Betriebsmittel-Zuweisungsgraphen mathematisch ausgeschlossen.

**Invarianten-Nachweis:** Erfüllt strikt Invariante 4 (Deadlockfreiheit bei Multi-Key-Locking) durch den Beweis der totalen Ordnung vor der Lock-Akquise.

**Migrationspfad:** Sämtliche Mutations-APIs (`relate_n_ary`), die mehr als eine Entität berühren, müssen verbindlich auf `acquire_multi_sorted` umgestellt werden.

**Restrisiken/offene Fragen:** Eine hohe Kollisionsrate (viele Entitäten hashen auf denselben Shard) reduziert die Parallelität, was ein Re-Tuning der `shard_mask` bei wachsender Graphgröße erfordert.

### B.5.1.3 — H3: Atomare Multi-Entity-Registrierung (`relate_n_ary`)

**Ist-Zustand im Repo:** Es existiert keine Transaktionsklammer, die einen Write-Ahead-Log (WAL) Eintrag für $N$ Graph-Knoten atomar in das LSM-System flusht und gleichzeitig den RCU-Graphen aktualisiert.

**Referenzierte Literatur:**

- Yan et al., 2023, "Hypergraph Database Storage", arXiv:2302.06119 — Spezifiziert atomare Schreiboperationen in n-ären Relationen über strukturierte Delta-Logs.


**Mathematische/algorithmische Spezifikation:** Die Funktion `relate_n_ary` operiert als logische Transaktion. Die Atomarität wird durch die Vorab-Allokation einer deterministischen `TxId` und das sequentielle Schreiben der Tupel $(e_i, \text{HyperEdgeId})$ in das WAL unter einem einzelnen Group-Commit gewährleistet. Die Komplexität ist $O(\vert{}P\vert{} \cdot \log(\text{MemTable}))$, wobei $\vert{}P\vert{}$ die Anzahl der Teilnehmer ist.

**Rust-Schnittstelle (normativ):**

Rust

```
use contextra_core::{DocId, TxId};

pub trait GraphCollectionMutation {
    fn relate_n_ary(
        &self,
        predicate_tag: u32,
        participants: &[RoleBinding],
        doc_id: DocId,
    ) -> Result<HyperEdgeId, GraphMutationError>;
}
```

**Lock-/Nebenläufigkeitsmodell:** Exklusive Write-Sperren werden über `acquire_multi_sorted` (H2) auf Entitätsebene gehalten, bis der `fsync` in das WAL erfolgreich beendet wurde. Rollback erfolgt durch Löschung der unvollständigen In-Memory-Einträge bei I/O-Fehlern.

**Invarianten-Nachweis:** §4(3) Determinismus wird eingehalten, da die Transaktionsgenerierung unabhängig von der Thread-Ausführung sequentiell geordnet ist.

**Migrationspfad:** Das offene Enum `SignalKind` wird nicht modifiziert. Hyperkanten-Treffer fließen additiv als `SignalKind::Graph` in die Ranking-Fusion ein.

**Restrisiken/offene Fragen:** Lange Transaktionen durch I/O-Latenz beim WAL-Flush blockieren konkurrierende Leseoperationen auf den betroffenen Entitäts-Shards.

### B.5.1.4 — H4: Nachweispflicht vor Implementierung

**Ist-Zustand im Repo:** Es fehlt ein analytischer Nachweis, ob die Cliquen-Expansion oder eine native Bipartit-Darstellung für die Nachbarschaftstraversierung optimal ist.

**Referenzierte Literatur:**

- Guo et al., 2024, "HyperGraphRAG", arXiv:2503.21322 — Bipartite Transformation von Hypergraphen für effizientes RAG.


**Mathematische/algorithmische Spezifikation:** Bei der Cliquen-Expansion einer Hyperkante $e$ mit Fan-out $N$ entstehen $\frac{N(N-1)}{2}$ binäre Kanten. Die Traversierung eines Knotens $v \in e$ kostet $O(N)$. In der bipartiten Stern-Expansion (ein künstlicher Knoten $v_e$ pro Hyperkante, verbunden mit allen $v \in e$) entstehen exakt $N$ Kanten. Die Traversierung von $v$ zu allen Nachbarn in $e$ erfolgt über $v_e$ in zwei Hops und kostet ebenfalls $O(N)$. Da die Speicherkomplexität der Stern-Expansion jedoch $O(N)$ gegenüber $O(N^2)$ beträgt, ist die bipartite Repräsentation (Stern-Expansion) für Speicherung und Traversierung zwingend vorzuziehen.

|**Metrik**|**Cliquen-Expansion**|**Bipartite Repräsentation (Stern)**|
|---|---|---|
|Kantenanzahl|$O(N^2)$|$O(N)$|
|Speicherplatz|Hoch|Minimal|
|Pfadlänge|1 Hop|2 Hops|
|Traversierung|$O(N)$|$O(N)$|

**Rust-Schnittstelle (normativ):** Keine direkte API-Schnittstelle; dies ist eine architekturelle Entscheidungsvorgabe.

**Lock-/Nebenläufigkeitsmodell:** Lese-Pfad über RCU (`ArcSwap`) erfordert keine Anpassung der Locks für Zwei-Hop-Traversierungen.

**Invarianten-Nachweis:** Erfüllt Invariante 6 ($O(\text{Seed})$ statt $O(\text{Graph})$), da die Traversierung durch den Nachbarschaftsgrad $N$ limitiert bleibt und nicht quadratisch explodiert.

**Migrationspfad:** Das Schema für Hyperkanten muss die `flatbuffers-drift-gate` in CI erfolgreich passieren, bevor diese Struktur eingeführt wird.

**Restrisiken/offene Fragen:** Die Zwei-Hop-Semantik verlängert die effektive Pfadtiefe in GraphRAG-Algorithmen, was Anpassungen in der Decay-Funktion beim Forward-Push Personalized PageRank erfordert.

### B.5.1.5 — H5: Kaskadierende Invalidierung ohne Kostenexplosion

**Ist-Zustand im Repo:** Die Löschung eines Dokuments löst eine ungebundene Kaskade von Invalidierungen aus. Bei Hyperkanten mit tausenden Teilnehmern führt dies zur Blockade des Main-Threads.

**Referenzierte Literatur:**

- Sarkar et al., 2020, "LSM-tree compaction", arXiv:2202.04522 — Analysiert Tombstone-Propagierung in Speichersystemen.


**Mathematische/algorithmische Spezifikation:** Um eine $O(N^2)$-Kostenexplosion bei der Kaskadenlöschung zu verhindern, wird die Löschmenge $D$ evaluiert. Ist $\vert{}D\vert{} \leq \theta$ (mit $\theta = 1000$), erfolgt die Löschung (Tombstone-Schreibung) synchron, $O(\vert{}D\vert{})$. Ist $\vert{}D\vert{} > \theta$, wird die Menge $D$ an der Grenze $\theta$ geteilt. Die ersten $\theta$ Elemente werden synchron verarbeitet. Der Rest $D \setminus D_{\theta}$ wird als asynchroner Task in die Background-Queue delegiert, wodurch die synchrone Latenz konstant $O(\theta)$ wird.

**Rust-Schnittstelle (normativ):**

Rust

```
use contextra_core::DocId;

pub const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT: usize = 1_000;

pub struct CascadeReport { // [v2.1] vollständige Fassung in §6.6 H5 (ticket, deletion_proof)
    pub tombstoned_synchronously: usize,
    pub queued_for_background: usize,
}

pub fn cascade_invalidate_hyperedges_for_superseded_doc(
    graph: &CsrGraph,
    doc_id: DocId,
    fanout_limit: usize,
) -> Result<CascadeReport, GraphMutationError>;
```

**Lock-/Nebenläufigkeitsmodell:** Der asynchrone Worker akquiriert Locks in kleinen Batches, um den RCU-Lese-Pfad nicht zu blockieren.

**Invarianten-Nachweis:** §4(6) Die Ausführungszeit der synchronen Funktion ist strikt durch das Fan-out-Limit $\theta$ nach oben beschränkt, was System-Latenz-Spikes verhindert.

**Migrationspfad:** Default-Aktivierung des Background-Workers beim Hochfahren der Contextra-Engine.

**Restrisiken/offene Fragen:** Abstürze während der asynchronen Verarbeitung können verwaiste Hyperkanten hinterlassen. [v2.1] Die Delete-Queue ist persistent und idempotent, normativ in §6.6 H5. Zur DLQ-Replay-Logik siehe §B.6.3.1.

### B.5.1.6 — H6: Projektion auf binäre Kantengewichte für Community Detection (Leiden)

**Ist-Zustand im Repo:** Der Leiden-Algorithmus iteriert über die binären Kanten. Hyperkanten werden durch `hyperedges_included: false` ignoriert.

**Referenzierte Literatur:**

- Traag et al., 2019, "From Louvain to Leiden: guaranteeing well-connected communities", arXiv:1810.08473 — Referenz zur Maximierung der Graph-Modularität in komplexen Netzwerken.


**Mathematische/algorithmische Spezifikation:** Für den Leiden-Algorithmus, der auf die Maximierung der Modularität $Q$ ausgelegt ist, werden Hyperkanten über einen Iterator als bipartiter Graph projiziert. Um zu verhindern, dass große Hyperkanten das Modularity-Clustering dominieren, wird das Gewicht $w(u, v_e)$ zwischen Teilnehmer $u$ und Hyperkanten-Knoten $v_e$ skaliert:

$$w(u, v_e) = \frac{2\,w(e)}{\vert{}e\vert{} - 1}$$ [v2.1: Konvention K, siehe §6.6 H6; Fassung 2 hatte $w(e)/(\vert{}e\vert{}-1)$]

Die Komplexität der Iteration bleibt $O(\vert{}E_B\vert{}) = O(\sum_{e \in E_H} \vert{}e\vert{})$.

**Rust-Schnittstelle:** [v2.1] Normativ ist §7.5 (`StarExpansionIterator` über `Arc<GraphInner>`, Item `StarEdge`).

**Lock-/Nebenläufigkeitsmodell:** Der Iterator arbeitet lock-frei auf einer unveränderlichen RCU-Snapshot-Referenz (`Arc`).

**Invarianten-Nachweis:** §4(7) Zero-Copy. Der Iterator generiert die virtuellen Kanten "on-the-fly" ohne Allokation einer neuen Adjazenzmatrix im Speicher.

**Migrationspfad:** Der `CommunityDetectionConfig` Struct wird um `hyperedges_included: bool` ergänzt, was standardmäßig auf `false` verbleibt, bis die Validierung gegen Benchmark-Netzwerke abgeschlossen ist.

**Restrisiken/offene Fragen:** Virtuelle Knoten im Leiden-Algorithmus verändern die Modularity-Resolution. Ein Hyperparameter-Tuning des Resolution-Parameters $\gamma$ ist für Netzwerke mit hoher Hyperkanten-Dichte zwingend.

### B.5.1.7 — GC: Speicherverlust durch verwaiste Knoten und defekte Kaskaden

**Ist-Zustand im Repo:** Unvollständige Löschungen hinterlassen Knoten ohne aktive Kanten, was den Speicherbedarf über Zeit aufbläht.

**Referenzierte Literatur:**

- Epoch-based reclamation Techniken analog zu Keir Fraser's EBR-Konzepten (implizit in `crossbeam-epoch`).


**Mathematische/algorithmische Spezifikation:** Die Garbage Collection identifiziert Knoten $v$, für die gilt: $\text{deg}_{\text{in}}(v) + \text{deg}_{\text{out}}(v) == 0$. Die Reklamation erfolgt Epochen-basiert. In der `compact()`-Phase wird der Graph gescannt ($O(\vert{}V\vert{})$). Verwaiste Knoten werden nicht in den neuen `ArcSwap`-Snapshot übernommen.

**Rust-Schnittstelle (normativ):**

Rust

```
pub trait GraphGarbageCollection {
    fn sweep_orphans(&self) -> Result<usize, GraphMutationError>;
}
```

**Lock-/Nebenläufigkeitsmodell:** `sweep_orphans` akquiriert den globalen Schreib-Lock für den neuen Snapshot, beeinträchtigt aber nicht die Leseprozesse auf dem aktiven Snapshot.

**Invarianten-Nachweis:** §4(5) Transparenz des Speicherbudgets wird durch die Freigabe des Speichers beim Austausch der Epochen sichergestellt.

**Migrationspfad:** Hintergrund-Cronjob implementieren, der `sweep_orphans` bei geringer Systemlast aufruft.

**Restrisiken/offene Fragen:** Bei sehr großen Graphen kann der $O(\vert{}V\vert{})$-Scan zu CPU-Spikes führen.

## B.5.2 Contextual-Bandit-Routing (LinUCB) (`crates/contextra-router`)

Das Contextual-Bandit-Modell entscheidet adaptiv über die Retrieval-Strategien. Die aktuelle Implementierung untergräbt jedoch die mathematischen Garantien des LinUCB-Algorithmus.

### B.5.2.1 — Falsche Mathematik im Produktions-Default (Diagonal-Approximation)

**Ist-Zustand im Repo:** Die `DiagonalApproximation` aktualisiert die Kovarianzmatrix-Diagonale iterativ als $\sigma^2_i \mathrel{+}= x_i^2$ und akkumuliert Parameter als $\theta_i \mathrel{+}= r \cdot x_i / \max(\sigma^2_i, 10^{-8})$. Dies ist eine Form der stochastischen Gradientenabstieg-Optimierung (SGD), aber keine echte Ridge-Regression.

**Referenzierte Literatur:**

- Li et al., 2010, "A Contextual-Bandit Approach to Personalized News Article Recommendation" — Etabliert den LinUCB-Standard.

- Zang et al., 2022, arXiv:2201.09910 — "diagonal approximation lacks theoretical justification" und bricht die Regret-Bounds.


**Mathematische/algorithmische Spezifikation:** Die echte LinUCB-Schranke erfordert $\theta = A^{-1}b$. Die aktuelle Implementierung verfehlt dies, da die Updates von $A$ (oder dessen Diagonale) nicht retroaktiv auf die bisher akkumulierten Werte in $b$ angewendet werden. Die Regret-Garantie von $O(d \sqrt{T \log T})$ zerfällt unter der Diagonal-Approximation für korrelierte Features zu einem linearen Regret $O(T)$ im Worst-Case. Um minimale Korrektheit zu wahren, muss $b$ separat akkumuliert und $\theta$ bei jeder Anfrage als $\theta_i = b_i / \sigma^2_i$ berechnet werden.

**Rust-Schnittstelle (normativ):**

Rust

```
pub struct CorrectedDiagonalBandit {
    pub precision_diag: Vec<f32>, // A_diag
    pub b: Vec<f32>,
    pub theta: Vec<f32>,
    pub alpha: f32,
}

impl CorrectedDiagonalBandit {
    pub fn update(&mut self, context: &[f32], reward: f32) {
        // ... update precision_diag and b, then compute theta = b / precision_diag
        unimplemented!()
    }
}
```

**Lock-/Nebenläufigkeitsmodell:** Mutationen sind sequenziell.

**Invarianten-Nachweis:** §4(3) Determinismus bleibt gewahrt.

**Migrationspfad:** Unmittelbares Update der bestehenden Struktur.

**Restrisiken/offene Fragen:** Die Diagonale ignoriert Feature-Korrelationen bei dichten LLM-Embeddings.

### B.5.2.2 — Sherman-Morrison-Update als Performance-Blocker (SIMD)

**Ist-Zustand im Repo:** Die korrekte Matrixinversion via Sherman-Morrison ist hinter dem Feature-Flag `egress-sherman-morrison` versteckt, da die $O(d^2)$-Operation ohne SIMD zu langsam ist.

**Referenzierte Literatur:**

- arXiv:2501.13139, "Efficient LinearUCB for Embedded Learning Systems" — Optimierung durch Sherman-Morrison und SIMD-Vektorisierung.


**Mathematische/algorithmische Spezifikation:** Die Sherman-Morrison-Formel für ein Rang-1-Update lautet:

$$(A + xx^T)^{-1} = A^{-1} - \frac{A^{-1}xx^T A^{-1}}{1 + x^T A^{-1} x}$$

Um die Latenz zu drücken, erfordert dies Vektorisierung. Die $d \times d$ Matrix muss cache-aligned ($64$ Byte) im Row-Major-Format im Speicher liegen, um False Sharing zu vermeiden. Die Berechnung von $v = A^{-1}x$ und das Update werden durch `fma` (Fused Multiply-Add) SIMD-Instruktionen beschleunigt. Komplexität: $O(d^2 / W)$, wobei $W=8$ für 256-Bit AVX.

**Rust-Schnittstelle:** [v2.1] Die frühere Skizze (`[f32; D * D]`, `x.data.len() != D`, `unsafe` im Router) ist entfallen: Sie kompiliert auf stable nicht und die Dimensionsprüfung war wirkungslos. Normativ ist §8.2 (Laufzeit-Dimension, Safe Rust).

**Lock-/Nebenläufigkeitsmodell:** Thread-lokale Ausführung ohne I/O.

**Invarianten-Nachweis:** §4(2) [v2.1] `contextra-router` bleibt `forbid(unsafe_code)`; SIMD-Kerne kämen als sichere API aus `contextra-simd` (Unsafe-Insel). §4(1) Harte `Result`-Dimensionsprüfung, kein `.unwrap()`.

**Migrationspfad:** CI-Gate für Latenz validieren, dann Feature-Flag `egress-sherman-morrison` zum Standard erheben.

**Restrisiken/offene Fragen:** Floating-Point-Präzisionsverlust über Millionen von Updates. Ein periodischer Cholesky-Rebuild von Grund auf ist empfehlenswert.

### B.5.2.3 — Fehlende Drift-Bandit-Kopplung

**Ist-Zustand im Repo:** Ein `LyapunovDriftWatcher` erkennt Konzeptdrift in der Feature-Verteilung, löst aber keine Parameteranpassung im Router aus.

**Referenzierte Literatur:**

- Wu et al., 2020, "Non-stationary contextual bandit" — Methoden zur Anpassung von Exploration unter Drift.


**Mathematische/algorithmische Spezifikation:** Wird Drift detektiert, muss die Exploration kurzzeitig eskalieren und das Vertrauen in alte Daten verringert werden. Eskalationsformel: $\alpha_t = \min(\alpha_{t-1} \cdot k_{\text{drift}}, \alpha_{\text{max}})$, mit Decay in Folgerunden. Discounting [v2.1 korrigiert]: $A \leftarrow \gamma A$, $b \leftarrow \gamma b$ mit $\gamma \in (0, 1)$, äquivalent $A^{-1} \leftarrow \gamma^{-1} A^{-1}$ (die Unsicherheit wächst, $\theta = A^{-1}b$ bleibt unverändert). Fassung 2 schrieb $A^{-1} \leftarrow \gamma A^{-1}$; das verkleinert die Unsicherheit.

**Rust-Schnittstelle (normativ):**

Rust

```
pub trait BanditPolicy: Send + Sync {
    fn apply_drift_penalty(&mut self, k_drift: f32, alpha_max: f32, gamma: f32);
}
```

**Lock-/Nebenläufigkeitsmodell:** Der Drift-Monitor benachrichtigt den Banditen asynchron über einen MPSC-Channel, um Latenz-Spikes im Inferenz-Pfad zu vermeiden.

**Invarianten-Nachweis:** §4(3) Determinismus der Updates bleibt durch Kanal-Synchronisation erhalten.

**Migrationspfad:** Schnittstelle in `BanditPolicy` implementieren und Channel-Listener im Main-Event-Loop aktivieren.

**Restrisiken/offene Fragen:** Aggressives $\gamma$ kann zu kurzzeitig extrem instabilen Routing-Entscheidungen führen.

### B.5.2.4 — Über-Fitting/Regret-Fehler generell (Off-Policy-Schätzung)

**Ist-Zustand im Repo:** Fehlendes Online-Monitoring der Banditen-Performance.

**Referenzierte Literatur:**

- Joachims et al., 2015, "Counterfactual Risk Minimization", arXiv:1502.02362 — IPS-Methodik.


**Mathematische/algorithmische Spezifikation:** Die kontrafaktische Evaluation einer neuen Policy $\pi_{\text{new}}$ aus geloggten Daten der Policy $\pi_{\text{old}}$ erfolgt über Inverse Propensity Scoring (IPS):

$$V_{\text{IPS}}(\pi_{\text{new}}) = \frac{1}{t} \sum_{i=1}^t r_i \frac{\mathbb{I}(\pi_{\text{new}}(x_i) == a_i)}{P_{\pi_{\text{old}}}(a_i \mid x_i)}$$

Der Nenner wird auf $\max(p, 0.01)$ geklemmt, um Varianz-Explosionen zu dämpfen. Komplexität: $O(t)$. **[v2.1] Voraussetzung:** IPS braucht eine randomisierte Logging-Policy mit Propensity-Untergrenze; ein deterministisches Argmax-LinUCB hat Propensity 1 und macht IPS wertlos (§8.5).

**Rust-Schnittstelle (normativ):**

Rust

```
pub struct OffPolicyEvaluator {
    cumulative_ips: f64,
    samples: u64,
}

impl OffPolicyEvaluator {
    pub fn observe(&mut self, target_action: u32, logged_action: u32, propensity: f32, reward: f32) {
        if target_action == logged_action {
            let p = propensity.max(0.01);
            self.cumulative_ips += (reward / p) as f64;
        }
        self.samples += 1;
    }
}
```

**Lock-/Nebenläufigkeitsmodell:** Lock-freier Akkumulator (Atomic oder thread-lokal).

**Invarianten-Nachweis:** §4(5) Fester Speicherverbrauch (zwei Skalare), keine unsichtbaren Allokationen.

**Migrationspfad:** Die WAL-Struktur muss Propensity-Werte bei jedem Logging mitschreiben.

**Restrisiken/offene Fragen:** IPS ist bias-anfällig, wenn Propensities stark von der Gleichverteilung abweichen.
