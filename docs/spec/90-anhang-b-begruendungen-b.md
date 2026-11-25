---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "90b"
---
## B.5.3 Block-Cache Lock-Kontention (`crates/contextra-store`)

Der Cache-Layer ist entscheidend für das LSM-Tree-Leseverhalten.

### B.5.3.1 — Ineffizienter Default (LRU Lock-Kontention)

**Ist-Zustand im Repo:** `LruBlockCacheBackend` nutzt `RwLock`. Jeder Read-Hit mutiert die Double-Linked-List zur Aktualisierung der Recency und erzwingt einen exklusiven Write-Lock.

**Referenzierte Literatur:**

- Zhang et al., 2024, "SIEVE is Simpler than LRU", NSDI 2024 / arXiv:2312.13123.


**Mathematische/algorithmische Spezifikation:** Unter Last verhält sich der RWLock nach dem Gesetz von Amdahl als starker Flaschenhals. Die zu erwartende Wartezeit steigt quadratisch mit der Thread-Anzahl $T$, proportional zu $p_{\text{hit}}^2$.

### B.5.3.2 — SIEVE-Alternative als lock-freier Standard

**Ist-Zustand im Repo:** `quick_cache` (S3-FIFO) ist hinter `block-cache-v2` verfügbar.

**Mathematische/algorithmische Spezifikation:** SIEVE eliminiert List-Reordering beim Read-Hit vollständig (Erfüllung von P25). Jeder Knoten trägt ein atomares `visited`-Bit. Bei einem Hit wird das Bit mit `Ordering::Relaxed` gesetzt ($O(1)$ lock-frei). Die Verdrängung (Eviction) nutzt einen umlaufenden Zeiger (`hand`). Ist das Cache-Limit erreicht, wird `hand` bewegt. Ist `visited == 1`, wird es auf $0$ gesetzt und der Knoten bleibt. Ist `visited == 0`, wird der Knoten entfernt. Worst-Case-Eviction-Komplexität: $O(C)$ wobei $C$ die Cache-Größe ist, Average-Case $O(1)$.

**Rust-Schnittstelle:** [v2.1] Die frühere `crossbeam-epoch`-Skizze ist entfallen (braucht `unsafe` im Safe-Crate `contextra-store`; `Shared<'static,_>` ist `!Send`/`!Sync`). Normativ ist §5.4: `scc::HashMap<K, Arc<SieveNode<V>>>` mit `visited` im Node und Mutex nur im Miss-/Evict-Pfad. `get` ist lock-arm, nicht wait-free.

**Invarianten-Nachweis:** §4(2) kein `unsafe`; §4(4) der Mutex wird nie während `get` gehalten.

**Migrationspfad:** Benchmark in CI gegen `quick_cache`. Bei Erfolg Flag `block-cache-v2` zur SIEVE-Implementation umleiten.

**Restrisiken/offene Fragen:** SIEVE bietet keinen dedizierten Schutz gegen sequenzielle Scans (Scan-Resistance), was bei großen Bereichsabfragen den Cache flushen kann.

### B.5.3.3 — Byte-basierte statt eintragsbasierte Kapazität

**Ist-Zustand im Repo:** Kapazität basiert auf der Element-Anzahl, was bei variablen Werten (Texte, Arrays) unberechenbaren Speicherverbrauch erzeugt.

**Mathematische/algorithmische Spezifikation:** Die Cache-Kapazität wird als $C_{\text{bytes}}$ definiert. Beim Einfügen eines Elements mit Größe $s$ wird atomar $S_{\text{current}} \mathrel{+}= s$ gerechnet. Wenn $S_{\text{current}} > C_{\text{bytes}}$, ruft SIEVE so lange Eviction auf, bis die Bedingung wieder erfüllt ist.

**Rust-Schnittstelle (normativ):**

Rust

```
impl<K, V> SieveCacheBackend<K, V> {
    pub fn capacity(&self) -> usize; // Return max bytes
    pub fn current_size(&self) -> usize; // Return current bytes via Relaxed AtomicUsize
}
```

**Lock-/Nebenläufigkeitsmodell:** Lock-frei mittels `fetch_add` / `fetch_sub`.

**Invarianten-Nachweis:** §4(5) Transparenz des Speicherbudgets.

**Migrationspfad:** Alle Insertion-Pfade müssen die Funktion zur Byte-Größen-Schätzung des jeweiligen Typs implementieren.

**Restrisiken/offene Fragen:** Ungenauigkeiten bei der Schätzung des Struct-Overheads im RAM.

## B.5.4 GraphRAG & Community Detection — vertieft

### B.5.4.1 — Vollständiger mathematischer Übergang auf binäre Gewichte

**Ist-Zustand im Repo:** Hyperkanten werden vom Leiden-Algorithmus ignoriert.

**Referenzierte Literatur:**

- Traag et al., 2019, "From Louvain to Leiden".


**Mathematische/algorithmische Spezifikation:** Um Hyperkanten $e \in E_H$ in den binären Leiden-Solver zu integrieren, ohne die Modularitätsberechnung $Q$ zu verzerren, nutzen wir eine Stern-Expansion. Sei $v_e$ ein synthetischer Knoten für $e$. Für jedes $u \in e$ entsteht eine Kante $(u, v_e)$ mit dem normalisierten Gewicht:

$$w(u, v_e) = \frac{2\,w(e)}{\vert{}e\vert{} - 1}$$ [v2.1: Konvention K, siehe §6.6 H6; Fassung 2 hatte $w(e)/(\vert{}e\vert{}-1)$]

[v2.1: der frühere „Beweis der Informationserhaltung" war falsch — die Teilnehmergradsumme des Sterns $\vert{}e\vert{}\,w/(\vert{}e\vert{}-1)$ entspricht nur für $\vert{}e\vert{}=2$ der Clique mit Paargewicht $w$.] Korrekte Aussage: Mit Referenz-Clique $p = w/\binom{\vert{}e\vert{}}{2}$ und Sternkante $a = \vert{}e\vert{}\,p = 2w/(\vert{}e\vert{}-1)$ ist das Schur-Komplement des Sterns exakt die Clique-Laplace-Matrix (Beweis und Grenzen in §6.6 H6; die Modularität bleibt nur näherungsweise erhalten). Rechenkosten: $O(\vert{}e\vert{})$ für den Stern gegenüber $O(\vert{}e\vert{}^2)$ für die Clique.

**Rust-Schnittstelle (normativ):**

Rust

```
// Iterator implementiert in 5.1.6.
```

**Lock-/Nebenläufigkeitsmodell:** Lock-freier Iterator über Snapshot.

**Invarianten-Nachweis:** §4(6) Komplexität korreliert mit Kantenanzahl, keine $N^2$ Explosion.

**Migrationspfad:** Konfiguration über `CommunityDetectionConfig`.

**Restrisiken/offene Fragen:** Das Einbringen virtueller Knoten reduziert künstlich die Dichte des Netzwerks, was die intrinsische Resolution $\gamma$ des Leiden-Algorithmus verschiebt.

## B.5.5 Vektorindex-Traversierung: HNSW-Dateiformat v2 (`crates/contextra-vector`)

Der Suchpfad in HNSW leidet unter Speicher-Ineffizienzen.

### B.5.5.1 — Allokations-Overhead pro Knoten (Arena-Modell)

**Ist-Zustand im Repo:** Traversierung allokiert `Vec<u32>` pro Knoten (`Cow::Owned`), was den GC und Allocator massiv belastet.

**Referenzierte Literatur:**

- Malkov & Yashunin, 2020, HNSW Originalkonzepte in flat memory layouts.


**Mathematische/algorithmische Spezifikation:** Um Allokationen zu eliminieren, wird eine Mmap-gestützte Arena-Struktur implementiert. Adjazenzlisten werden pro HNSW-Layer $l$ als Compressed Sparse Row (CSR) `offsets_l` und `targets_l` abgespeichert. Ein Zugriff auf Nachbarn von Knoten $i$ im Layer $l$ benötigt zwei Array-Lookups: `start = offsets_l[i]`, `end = offsets_l[i+1]`. Zeit: $O(1)$, Alloc: 0.

**Rust-Schnittstelle (normativ):**

Rust

```
pub struct HnswArenaView<'m> {
    pub arena_vectors: &'m [f32],
    pub layer_offsets: Box<[&'m [u32]]>,
    pub layer_targets: Box<[&'m [u32]]>,
    pub stride: usize,
}

impl<'m> HnswArenaView<'m> {
    #[inline(always)]
    pub fn get_neighbors(&self, node: u32, layer: u8) -> &'m [u32] {
        let offsets = self.layer_offsets[layer as usize];
        let targets = self.layer_targets[layer as usize];
        &targets[offsets[node as usize] as usize .. offsets[node as usize + 1] as usize]
    }
}
```

**Lock-/Nebenläufigkeitsmodell:** Lesezugriffe sind vollständig parallelisierbar, da die Mmap unveränderlich ist.

**Invarianten-Nachweis:** §4(7) Zero-Copy erfüllt. Die Slice-Referenz referenziert den Speicher der Mmap direkt.

**Migrationspfad:** Binär inkompatibles Format. Ein `HNSW_VERSION=2` Header wird eingeführt; ein Hintergrund-Prozess re-indiziert alte Vektoren.

**Restrisiken/offene Fragen:** Inserts erfordern einen RAM-Overlay (Chunked Allocation), der beim Kompaktieren periodisch in die Datei zurückgeschrieben wird.

### B.5.5.2 — Backlink-Lookup nicht O(1)

**Ist-Zustand im Repo:** Lineare Iteration bei der Backlink-Auflösung in Batch-Inserts $O(P \times B)$.

**Mathematische/algorithmische Spezifikation:** Die Auflösung erfordert einen $O(1)$ Hash-Lookup. Eine temporäre HashMap wird am Start der Batch-Verarbeitung generiert. Der Schlüssel ist ein bit-gepackter `u64` bestehend aus `ram_idx` (32 Bit) und `layer` (8 Bit). Zeitkomplexität fällt auf $O(1)$ je Schritt.

**Rust-Schnittstelle (normativ):**

Rust

```
#[derive(Default)]
pub struct SearchScratch {
    pub overlay_backlinks: ahash::AHashMap<u64, &'static [u32]>,
}
```

**Lock-/Nebenläufigkeitsmodell:** Thread-lokaler Scratch-Puffer, lock-frei.

**Invarianten-Nachweis:** §4(1) Keine impliziten Panics durch Boundary Checks, saubere Hash-Ergebnisse.

**Migrationspfad:** Sofort ersetzbar im Insert-Pipeline-Code.

**Restrisiken/offene Fragen:** Hashmap Allokation für extrem kleine Batches eventuell überproportional teuer.

### B.5.5.3 — Distanzpfad Lock/Allokation

**Ist-Zustand im Repo:** Mmap Vektoren werden elementweise gelesen und dekodiert; Quantisierer sperren den Lesevorgang.

**Mathematische/algorithmische Spezifikation:** Der Vektorzugriff muss als konstanter Slice direkt an die SIMD-Engine gereicht werden. Die Quantisierungs-Skalare (`scale`, `min`) des Codebooks werden am Start der Query einmalig per Read-Lock kopiert und in der Engine lokal gekapselt, statt pro Kandidat gesperrt zu werden.

**Lock-/Nebenläufigkeitsmodell:** Einmaliger RWLock-Acquire pro Query.

**Invarianten-Nachweis:** §4(7) Zero-Copy (Übergabe eines Slices statt eines iterativ allozierenden `Vec`).

### B.5.5.4 — NaN-Sicherheit bei SIMD-Distanzberechnung

**Ist-Zustand im Repo:** Skalarer NaN-Check läuft bei jedem Vektorvergleich vor der SIMD-Schleife, was Performance massiv degradiert.

**Mathematische/algorithmische Spezifikation:** NaN-Werte kontaminieren L2-Normen. Um den Check im O(N) Hot-Path zu umgehen, wird die Validierung erzwungen an:

1. Den Query-Vektor $q$ am Start der Funktion ($O(D)$ Skalar).

2. Beim Insert jedes Vektors. Dies wird durch ein Flag `VALIDATED_NO_NAN` im Dateikopf manifestiert. Innerhalb von `dot4_avx2` wird nicht mehr geprüft.


**Rust-Schnittstelle (normativ):**

Rust

```
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2", enable = "fma")]
#[allow(unsafe_code)]
pub unsafe fn dot4_avx2(q: &[f32], arena: &[f32], bases: [usize; 4], dim: usize, out: &mut [f32; 4]) {
    // Implementierung via _mm256_fmadd_ps ohne NaN Check
    unimplemented!()
}
```

**Lock-/Nebenläufigkeitsmodell:** Thread-lokale Register-Operationen.

**Invarianten-Nachweis:** §4(2) `unsafe_code` Begründung: Hardware-Acceleration für die zentrale mathematische Operation; Isolierung durch Garantien aus der Validierungs-Phase.

**Migrationspfad:** Altdaten (Header ohne Flag) triggern den langsamen Pfad, bis der Index kompaktiert wird.

**Restrisiken/offene Fragen:** Keine, da CPU FMA mathematisch deterministisch ist.

### B.5.5.5 — Top-k-Selektion

**Ist-Zustand im Repo:** `sort_unstable_by` sortiert volle Arrays in $O(M \log M)$.

**Mathematische/algorithmische Spezifikation:** Für materialisierte Listen wird Introselect (`select_nth_unstable_by`) genutzt: $O(M)$ im Average/Worst-Case. Für das Streaming im Traversierungspfad wird ein Bounded Min-Heap der Größe $k$ eingesetzt: $O(M \log k)$.

**Rust-Schnittstelle (normativ):**

Rust

```
pub fn select_top_k_materialized(scores: &[f32], id_table: &[u64], k: usize) -> Vec<u32> {
    let mut idx: Vec<u32> = (0..scores.len() as u32).collect();
    let cmp = |&a: &u32, &b: &u32| {
        scores[b as usize].total_cmp(&scores[a as usize])
            .then_with(|| id_table[a as usize].cmp(&id_table[b as usize]))
    };
    if k < idx.len() {
        idx.select_nth_unstable_by(k, cmp);
        idx.truncate(k);
    }
    idx.sort_unstable_by(cmp);
    idx
}
```

**Lock-/Nebenläufigkeitsmodell:** Keine Synchronisation notwendig.

**Invarianten-Nachweis:** §4(3) Determinismus durch total order via `total_cmp` und Tie-Breaker.

**Migrationspfad:** Austausch im `hybrid_search` Code.

**Restrisiken/offene Fragen:** Partielle Sortierung verliert die absolute Ranking-Ordnung über den Index $k$ hinaus.

### B.5.5.6 — CSR-Sentinel statt Option

**Ist-Zustand im Repo:** `Vec<Option<u32>>` in Graph- und Index-Strukturen verbraucht unnötigen Speicher für Diskriminanten.

**Mathematische/algorithmische Spezifikation:** Ersatz von `Option<u32>` durch `u32` mit dem Sentinel `u32::MAX`. Dies halbiert den RAM-Footprint für spärliche Vektoren von 8 auf 4 Byte pro Eintrag.

**Rust-Schnittstelle (normativ):**

Rust

```
pub const SENTINEL_NULL_ID: u32 = u32::MAX;
// Arrays nutzen direkt u32
```

**Lock-/Nebenläufigkeitsmodell:** N/A.

**Invarianten-Nachweis:** §4(5) Reduzierter Speicherverbrauch erhöht Budget-Transparenz.

**Migrationspfad:** Binär inkompatibles Format; bedarf Adapter.

**Restrisiken/offene Fragen:** Keine, solange Systemlimit bei $< 4.2 \times 10^9$ Knoten bleibt.

### B.5.5.7 — Partielle Rebuilds mit Recall-Erhaltungsgarantie

**Ist-Zustand im Repo:** Codebooks in der Skalar-Quantisierung driften, was zu Recall-Verlust bei Updates führt.

**Mathematische/algorithmische Spezifikation:** Das Codebook $C$ wird periodisch rekalibriert, wenn der Kullback-Leibler-Divergenzschätzer (oder die Min/Max Verschiebung) eine Schwelle überschreitet. Partielle Rebuilds der Layer erfolgen im Hintergrund.

**Lock-/Nebenläufigkeitsmodell:** RCU-Mechanik für das Codebook.

**Invarianten-Nachweis:** §4(3) Determinismus der Suchergebnisse bezogen auf die Epoche des Snapshots.

## B.6. Weitere Systembereiche

### B.6.1 LSM-Storage-Engine (`crates/contextra-store`)

#### B.6.1.1 — SSTable-Lock-Handoff

**Spezifikation:** Zur Vermeidung von Stalls beim Flush der MemTable auf Disk wird das Mutex nicht über die I/O-Operation gehalten. Eine `AtomicU64`-Sequenznummer regelt den Handoff der Zuständigkeit für den Gruppen-Commit.

#### B.6.1.2 — Lock-freie WAL-Pipeline

**Spezifikation:** [v2.1] Der WAL nutzt einen MPSC-Actor mit begrenzter Queue und `oneshot`-Ack nach `fsync` (Group Commit), normativ in §5.3. SPSC war falsch (mehrere Produzenten); der Commit kehrt nie vor dem `fsync` zurück (P2).

#### B.6.1.3 — Inkomplette Tombstone-Propagierung

**Spezifikation:** Tombstones dürfen erst verworfen werden, wenn kein aktiver Snapshot (Lese-Transaktion) mehr existiert, der eine Sequenznummer kleiner der des Tombstones referenziert. Beweis: Behalte Versionen mit `seq > min_snapshot_seq` PLUS die neueste Version $\leq \text{min\_snapshot\_seq}$.

#### B.6.1.4 — Manifest-Fehlerbehandlung bei Crash-Recovery

**Spezifikation:** Der Zustand der Kompaktierung (lösche Inputs, füge Output hinzu) muss zwingend ein atomarer `ManifestEntry::Replace` Record sein. Ein teilgeschriebener Add/Remove hinterlässt bei einem Crash doppelte oder verwaiste Daten (Resurrection von gelöschten Schlüsseln).

#### B.6.1.5 — Zero-Copy-Slice

**Spezifikation:** Das Lesen von SSTable Blöcken nach dem Entschlüsseln und der CRC-Prüfung nutzt `bytes::Bytes::slice(4..)`, anstatt den Nutzlastpuffer neu zu kopieren. Erfüllt §4(7).

#### B.6.1.6 — MemTable Range-Sharding

**Spezifikation:** Hash-Sharding der In-Memory-Daten zerstört Präfix-Scans (erfordert 16 B-Tree Traversals). Range-Sharding teilt die Schlüssel alphanumerisch auf Shards auf, sodass ein Präfix-Scan zumeist in einem Lock-Fenster eines Shards bedient werden kann.

#### B.6.1.7 — Checkpoint-Index-Merge

**Spezifikation:** Statt separater Locks für `name_index` und `seq_index` fasst ein `RwLock` einen Struct zusammen, der beide Maps enthält. Atomarität ist gegeben.

### B.6.2 Inference, KV-Bridge & Routing (`crates/contextra-infer-candle`)

#### B.6.2.1 — Asynchrone Zero-Copy-KV-Cache-Eviction-Bridge

**Spezifikation:** Wenn Token-Mengen RAM übersteigen, werden paged KV-Blöcke (verschlüsselt via AES-GCM-SIV) per `Bytes`-Slice direkt in die LSM-Engine gespült. Async-I/O verhindert, dass die GPU-/CPU-Inferenz ins Stocken gerät.

#### B.6.2.2 — Instabilität des PID-Controllers (Lyapunov)

**Spezifikation:** Zur Latenzbegrenzung des Routings misst ein PID-Regler Abweichungen. Formel für Anti-Windup unter Berücksichtigung von $\Delta t$: $I_{new} = \text{clamp}(I_{old} + e \cdot \Delta t, -I_{max}, I_{max})$. Ohne Sättigungsgrenze läuft der Integrator ins Unendliche. Erfüllt deterministische Stabilität.

#### B.6.2.3 — Zero-Copy-Deserialisierung im IPC-Generator

**Spezifikation:** FlatBuffers wird genutzt, um IPC-Nachrichten vom Contextra-Prozess zum MCP-Client (Python/Node) als direkte Referenz in Memory-Mapped Slices bereitzustellen, ohne Deserialisierungs-Kopien (zero-copy).

### B.6.3 Agenten-State, Crypto & MCP

#### B.6.3.1 — Atomare DLQ-Replay-Logik

**Spezifikation:** Ein Event, das fehlschlägt, wird als `(Session, Node, Step)`-Schlüssel persistiert. Idempotenz: Bei Replay prüft die Engine die WAL-Transaktions-ID, um Doppelbuchungen zu verhindern.

#### B.6.3.2 — Zeroize-on-Panic im Egress-Vault

**Spezifikation:** Um PII-Daten nach einem Panic (z.B. Timeout beim Regex-Matching) zu vernichten, werden sensible Strings in `zeroize::Zeroizing<Vec<u8>>` gewrappt. Der Drop-Guard sorgt deterministisch für die Überschreibung im RAM. Verteidigung in der Tiefe (§4(1)). [v2.1] Wirksam nur unter `panic = "unwind"` (Root-Profil, §0.2); im `release-abort`-Profil laufen keine Destruktoren, dort schützt nur das Prozessende, und Core-Dumps sind betrieblich zu deaktivieren.

#### B.6.3.3 — Race Conditions bei Budget-Berechnungen

**Spezifikation:** Das Agent-Budget wird über eine RAII-Struktur verwaltet: `budget.reserve(n) -> Reservation`. Bei Erfolg `reservation.settle()`, bei Drop erfolgt eine garantierte Rückerstattung. Double-Spend ist ausgeschlossen.

#### B.6.3.4 — Lückenhafte WASM-Sandbox-Egress-Isolierung

**Spezifikation:** Wasmtime erfordert strikte Speicherbegrenzungen (`Store::set_fuel`) und Memory-Limits. Cloud-Aufrufe innerhalb des WASM müssen von Datei-Reads logisch getrennt als `CloudEgress` in den Capability-Flags geführt werden.

#### B.6.3.5 — Kryptographisch verifizierbare Deletion Proofs

**Spezifikation:** Wenn ein Record aus dem LSM entfernt wird, erzeugt die HMAC-Kette der WAL einen Nachweis. Quittung: $H(\text{hmac}_{\text{prev}} \parallel \text{delete\_event})$. Verifikation in $O(1)$ Zeit ohne Klartext-Zugang (DSGVO Art. 17 konform).

#### B.6.3.6 — AES-Schlüsselplan-Wiederverwendung

**Spezifikation:** Die Expansionsrunde `new_from_slice` für AES-256-GCM-SIV kostet massive CPU-Zyklen. Die Struktur wird im `KeyManager` pro Schlüssel gecacht und thread-safe (`OnceCell`) wiederverwendet. [v2.1] Nonce-Strategie ⚖️ offen (§9.3, §A2.4 Nr. 5): Ein reiner In-Memory-Zähler beginnt nach Neustart bei 0 und ist ohne persistierten Hochwasserstand unzulässig; das Repo nutzt bewusst `OsRng`-Nonces.

## B.7. Priorisierung und Abhängigkeitsanalyse

Nach dem Schema: Aufwand (Trivial/Gering/Mittel/Hoch) × Nutzen (Latenz-/Speicher-/Korrektheitsgewinn).

### Stufe 0 — Unmittelbar (Korrektheit/Sicherheit)

|**Problem-ID**|**Maßnahme**|**Aufwand**|**Nutzen**|**Abhängigkeit**|
|---|---|---|---|---|
|**6.1.4**|LSM Manifest-Batch-Fsync (`Replace` Record)|Trivial|Verhindert Auferstehung gelöschter Daten|Keine|
|**5.2.1**|Bandit-Dimensionsprüfung / Ridge Math|Gering|Verhindert NaN/Dimensions-Crash|Keine|
|**6.3.4**|Egress-Klassifizierung & Wasmtime Limits|Gering|Sicherheitsisolation|Keine|
|**6.1.3**|Intent-Recovery & Tombstone-Propagierung|Mittel|Deterministisches Recovery|6.1.4|

### Stufe 1 — Hot-Path-Performance

|**Problem-ID**|**Maßnahme**|**Aufwand**|**Nutzen**|**Abhängigkeit**|
|---|---|---|---|---|
|**5.5.1**|HNSW v2 Arena Allocation|Hoch|Beseitigt 90% der Allokationen (3-5x Speedup)|Storage `Bytes` API|
|**6.1.5**|SSTable Zero-Copy-Slice & `StorageEngine::get`|Mittel|Verhindert Vollkopie auf Ebene 0|Keine|
|**6.3.6**|AES-Schlüsselplan Wiederverwendung|Gering|Reduziert Crypto-Overhead bei KV-Cache|Keine|
|**5.3.2**|Block-Cache Byte-Cap & SIEVE Lock-free|Mittel|Beseitigt LRU Kontention|Keine|
|**5.5.5**|Top-k Selektion (Introselect/Heap)|Trivial|$O(M \log M) \to O(M)$|Keine|

### Stufe 2 — Speicher und Struktur

|**Problem-ID**|**Maßnahme**|**Aufwand**|**Nutzen**|**Abhängigkeit**|
|---|---|---|---|---|
|**5.1.1**|N-äre Hyperkanten (RCU Integration)|Hoch|Modell-Exaktheit für LLM Inferenz|H2 (Locks)|
|**5.1.7**|Inkrementelle Graph-Kompaktierung (GC)|Mittel|$O(E)$ Lese-Spikes verhindern|H1|
|**5.5.6**|CSR Sentinel statt Option|Gering|Halbiert RAM-Footprint für CSR|HNSW v2|
|**6.1.7**|Checkpoint-Index-Merge|Gering|Beseitigt Race-Condition|Keine|

### Stufe 3 — Governance/Prozess

|**Problem-ID**|**Maßnahme**|**Aufwand**|**Nutzen**|**Abhängigkeit**|
|---|---|---|---|---|
|**CI**|Feature-Powerset-CI in GitHub Actions|Gering|Sichert Kompilierbarkeit aller Feature-Pfade|Keine|
|**CI**|Kontinuierliches Panic-Inventar-Gate|Mittel|Sichert §4(1) Zero-Panic Doctrine|Keine|

_(Harte Abhängigkeit verzeichnet: H3 `relate_n_ary` setzt das in Stufe 2 implementierte H2 Multi-Key-Locking voraus; der Bandit-Default-Wechsel in 5.2.2 setzt den Erfolg im Sherman-Morrison-Latenzgate voraus)._


---

---

<a id="anhang-c"></a>
