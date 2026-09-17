# MemFuse — Gesamtspezifikation des Endprodukts (v6)

> **Status:** Normativ · Einzige maßgebliche, eigenständige Quelle für Produkt, Architektur, Datenmodell, Algorithmen,
> Sicherheitsmodell und Implementierungsreihenfolge von MemFuse.
> **Geltung:** Dieses Dokument ersetzt vollständig alle vorherigen Spezifikationsfassungen und -deltas
> (u. a. die `FINAL_9`–`FINAL_12`-Reihe, die `v11.1`/`v11.2`-Hyperkanten-Deltas sowie die
> „Mikrofeingranulare Schnittstellenspezifikation"). Es enthält keine Verweise auf diese Vorgängerdokumente mehr,
> sondern führt deren normativen Inhalt zu einem einzigen, in sich geschlossenen Zielbild zusammen.
> **Charakter:** Dies ist eine **Zielarchitektur- und Produktspezifikation**, keine Commit-für-Commit-Verifikation
> eines bestimmten Repository-Standes. Wo zwischen spezifiziertem Zielzustand und (zum Zeitpunkt der letzten
> bekannten Code-Prüfung) tatsächlich implementiertem Zustand ein Unterschied besteht, ist dies explizit als
> **Reifegrad** gekennzeichnet, nicht stillschweigend vermischt.
>
> **Reifegrad-Kennzeichnung, durchgängig verwendet:**
> - 🟢 **Produktiv** — im Code vorhanden, korrekt und als Produktions-Default aktiv.
> - 🟡 **Hinter Feature-Flag** — im Code vollständig und korrekt vorhanden, aber nicht der Produktions-Default;
>   Aktivierung erfordert ein explizites Cargo-Feature oder eine Konfigurationsoption.
> - 🔴 **Spezifiziert, zu bauen** — normativer Zielzustand dieses Dokuments, im Code (Stand der letzten Prüfung)
>   nicht vorhanden.
> - ⚖️ **Produktentscheidung ausstehend** — die technische Umsetzung ist möglich oder bereits vorhanden, der
>   Wechsel des Produktions-Defaults ist jedoch eine bewusste, an ein Kriterium (Benchmark, Major-Release)
>   gebundene Entscheidung, kein Implementierungsrückstand.

---

## Inhaltsverzeichnis

1. [Kernthese & Leitprinzip](#kernthese)
2. [Produktvision, Alleinstellungsmerkmale & Nicht-Ziele](#vision)
3. [Architekturprinzipien P1–P25](#prinzipien)
4. [Systemarchitektur: der Crate-DAG](#architektur)
5. [Speicherschicht: LSM-Tree, WAL und lock-freies Cache-Management](#speicher)
6. [Wissensgraph-Datenmodell: binäre Kanten und n-äre Hyperkanten](#graph)
7. [Retrieval-Pipeline: 4-Signal-Fusion und ihre Algorithmen](#retrieval)
8. [Contextual-Bandit-Routing](#bandit)
9. [Inferenz, KV-Cache-Bridge und Zero-Copy-IPC](#inferenz)
10. [Sicherheits- und Datenschutzmodell](#sicherheit)
11. [Betriebsmodi](#betrieb)
12. [Feature-Flag-Politik: Produktions-Default vs. Opt-in](#features)
13. [Governance & Entwicklungsprozess](#governance)
14. [Abnahmekriterien des Endprodukts](#abnahme)
15. [Rückverfolgbarkeitsmatrix](#matrix)
16. [Roadmap](#roadmap)

---

<a id="kernthese"></a>
## 1. Kernthese & Leitprinzip

**MemFuse ist eine souveräne, vollständig lokal betriebene Gedächtnisschicht für KI-Agenten** — eine eingebettete,
kryptographisch isolierte AI-Memory-Bibliothek in Rust mit Python- und MCP-Bindings, die ohne Cloud-Abhängigkeit,
ohne Telemetrie und ohne API-Key betrieben werden kann.

Ihr Alleinstellungsmerkmal ist die Kombination aus:

- einer **4-Signal-Retrieval-Fusion** (Vektor, Volltext, Graph, Metadaten) statt reiner Vektorsuche,
- einer **kryptographisch integritätsgesicherten Storage-Engine** (LSM-Tree, WAL mit HMAC-Kette, AES-256-GCM-SIV at rest),
- **WASM-/Sandbox-Ausführungsisolation** für Agent-Tool-Aufrufe,
- echter **Air-Gap-Inferenz** (lokales GGUF-Backend, kein Netzwerkzwang) mit verschlüsseltem, LSM-rückfallfähigem KV-Cache,
- einem **Contextual-Bandit-Router**, der Anfragen adaptiv auf Retrieval-Strategien verteilt,
- und — als jüngste, noch zu bauende Erweiterung des Datenmodells — **n-ären Hyperkanten** für Fakten, die sich
  nicht auf ein Subjekt-Prädikat-Objekt-Tripel reduzieren lassen (§6).

**Leitprinzip: Korrektheit schlägt Performance schlägt Feature.** Jede Optimierung, die eine Korrektheitsgarantie
(Datenintegrität, Nebenläufigkeitssicherheit, Wiederherstellbarkeit, Deadlockfreiheit) aufweicht, ist unzulässig —
unabhängig vom Performancegewinn. Jede Performance-Optimierung, die eine noch nicht spezifizierte Fähigkeit
vorwegnimmt, ist nachrangig gegenüber der Fertigstellung bereits spezifizierter Fähigkeiten. Jede Erweiterung eines
bestehenden Subsystems muss geprüft werden gegen die Invarianten, die dieses Subsystem bereits trägt — nicht nur
dagegen, *dass* eine Erweiterung grundsätzlich möglich ist, sondern *welches bestehende Invariant dadurch unter
Druck gerät* und wie es gewahrt bleibt. Dieses Prinzip prägt insbesondere die Hyperkanten-Spezifikation (§6.5).

---

<a id="vision"></a>
## 2. Produktvision, Alleinstellungsmerkmale & Nicht-Ziele

### 2.1 Was MemFuse ist

Eine eingebettete (embedded) Gedächtnisschicht, kein Cloud-Service. MemFuse läuft im Prozess des aufrufenden
Agenten oder als lokaler MCP-Server — es gibt keine serverseitige Multi-Tenant-Instanz und keine Datenübertragung
an Dritte, sofern nicht explizit über das Cloud-Egress-Gateway (§10.4) angefordert.

### 2.2 Distributionswege

| Kanal | Paket | Zielgruppe |
|---|---|---|
| MCP-Server (primär) | `uvx memfuse-mcp --db-path ... --allow-write` | Claude Desktop, Cursor, beliebige MCP-Clients |
| Python-Bibliothek | `pip install memfuse` | In-Process-Einbettung in Python-Agenten |
| Rust-Crate | `cargo add memfuse-db` | Native Rust-Anwendungen |

Eine Desktop-Shell (`memfuse-tauri`) existierte als Prototyp, ist aber zugunsten der PyPI-Bibliothek und des
MCP-Servers als primäre Vertriebswege eingestellt (deprecated).

### 2.3 Alleinstellungsmerkmale und ihr Reifegrad

1. **4-Signal-Hybridsuche** (🟢) — Vektorsuche (HNSW), Volltextsuche (BM25/BM25F), Wissensgraph-Traversierung
   (CSR + Forward-Push-Personalized-PageRank), Metadaten-Filter; fusioniert über Reciprocal Rank Fusion
   (RRF, Default 🟢) oder score-normalisierte Fusion (Opt-in 🟡, mit hartem RRF-Fallback bei Signaldegradation).
2. **Kalibriertes Retrieval mit Lyapunov-Drift-Erkennung** (🟢) — Score-Schwellenwerte werden nicht statisch,
   sondern über ein laufend kalibriertes Modell mit gedeckelter Drift-Eskalation bestimmt.
3. **MCP-native Zero-Trust-Sandbox** (🟢) — Tool-Ausführung mit getrennt konfigurierbarem Fuel- (Rechenschritt-)
   und Wall-Clock-Budget (Default 5 s), orthogonal zueinander konfigurierbar.
4. **Kryptographische DSGVO-Art.-17-Löschung** (🟢) — Löschvorgänge erzeugen einen verifizierbaren `DeletionProof`
   über eine race-freie HMAC-Kette.
5. **Session-DAG** (🟢) — Konversationsverzweigung als persistenter, azyklischer Graph.
6. **Air-Gap-KV-Cache-Bridge mit LSM-Fallback-Spill** (🟢) — der KV-Cache liegt primär verschlüsselt im RAM; bei
   Speicherdruck greift kontrolliertes Auslagern auf die SSD statt verlustbehafteten Verwerfens.
7. **Cloud-Egress Privacy Gateway** (🟢, weitgehend auditiert) — mehrschichtiger DLP-Pfad mit Surrogat-Tokenisierung,
   Bulk-Exfiltration-Erkennung und Rehydration der Cloud-Antwort (§10.4).
8. **Contextual-Bandit-Routing** (🟢 Grundfunktion / 🟡 mathematisch korrekte Variante) — LinUCB-basiertes Routing;
   siehe §8 für die Unterscheidung zwischen Produktions-Default und Ridge-korrekter Opt-in-Variante.
9. **Gestufte Vektorindex-Architektur** (🟢 HNSW / 🟢 DiskANN als Tier) — HNSW als Standard, DiskANN für
   RAM-sprengende Korpora.
10. **Deutsche Morphologie inkl. BM25F** (🟢) — Kompositazerlegung im Volltextindex plus feldgewichtete Bewertung.
11. **Zero-Copy-Storage-Pfad** (🟢) — seit der Grundarchitektur produktiv.
12. **Key-granulare Schreibnebenläufigkeit** (🟢) — `kv_locks` statt collection-weitem Mutex.
13. **N-äre Hyperkanten** (🔴, vollständig spezifiziert, siehe §6) — Fakten mit mehr als zwei Beteiligten als
    erstklassige, atomar invalidierbare Struktur statt Zerlegung in mehrere, im Zusammenhang verlorene Binärkanten.
    Dies ist die einzige der hier geführten Fähigkeiten, die noch **nicht** implementiert ist; sie wird deshalb in
    §6 mit besonderer Tiefe spezifiziert.

### 2.4 Nicht-Ziele

MemFuse ist explizit **kein** Cloud-SaaS-Produkt, **kein** Multi-Tenant-Enterprise-System, **kein** Framework für
LLM-Training, **keine** primär GUI-getriebene Desktop-Anwendung und **kein** Cluster-/Replikations-System. Ein
`memfuse-cluster`-Veto besteht bewusst: verteilter Konsensbetrieb ist kein Ziel der aktuellen Produktphase.
Passives WAL-Shipping für Backup-Zwecke ist als Fernziel vorgesehen (Roadmap-Stufe 4, §16), aber nicht Bestandteil
des Kernprodukts.

---

<a id="prinzipien"></a>
## 3. Architekturprinzipien P1–P25

Diese Prinzipien sind normativ für jede gegenwärtige und künftige Erweiterung des Systems.

**P1 — Korrektheit schlägt Performance schlägt Feature.** Siehe §1.

**P2–P22 — Sovereign-Core-Grundsätze** (u. a. WAL-First-Persistenz, deterministische Recovery, keine
stillschweigende I/O-Fehlerunterdrückung, strikte DAG-Modularität des Crate-Graphen, feingranulare
`Result<T, E>`-Fehlerkategorisierung statt generischer `panic!`-Pfade, Verbot von `unwrap()`/`expect()` auf
potenziell toxischen Ein-/Ausgaben). Diese Grundsätze sind seit der Kernarchitektur unverändert gültig und werden
in §5–§10 an den jeweils betroffenen Subsystemen konkretisiert.

**P23 — Zeitbudgets sind orthogonal konfigurierbar.** Rechenschritt-Budget (Fuel) und Wall-Clock-Budget für
Sandbox-Ausführungen sind zwei unabhängige Achsen. Ein Tool kann rechnerisch günstig, aber durch blockierendes I/O
langsam sein, oder umgekehrt — beide Fälle müssen unabhängig begrenzbar sein. **Produktiv erfüllt** (§10.2).

**P24 — Lokalität vor globaler Neuberechnung.** Jeder Algorithmus, dessen Eingabe eine anfragebestimmte Teilmenge
des Gesamtzustands ist (PPR mit wenigen Seed-Knoten, Cascade-Invalidierung ausgehend von einem Dokument), MUSS
eine zur Anfragegröße proportionale Laufzeit haben — niemals zur Größe des Gesamtzustands ($O(V+E)$ ist für
solche Anfragen unzulässig). Dieses Prinzip ist der normative Grund für den Forward-Push-PPR-Algorithmus (§7.2)
und für das harte Fan-out-Limit der Hyperkanten-Cascade-Invalidierung (§6.5, H5). Es gilt uneingeschränkt für jeden
künftigen Graph- oder Retrieval-Algorithmus.

**P25 — Cache-Treffer sind lock-frei bzw. lock-günstig zu gestalten.** Ein Lesetreffer im Block-Cache soll nach
Möglichkeit keinen exklusiv sperrenden, mutierenden Zugriff erfordern, da Cache-Treffer der mit Abstand häufigste
Zugriffspfad sind und jede darin verborgene Schreibsperre unter Last zur Kontention wird (§5.2).

**Ergänzende, aus der jüngsten Architekturüberarbeitung übernommene Grundsätze:**

- **Nebenläufigkeitssicherheit vor Nebenläufigkeitsperformance:** Sperrenhierarchien werden explizit dokumentiert
  und dürfen nicht durch bloßen Analogieschluss auf neue Mutationspfade übertragen werden, ohne die
  Deadlockfreiheit für den neuen Fall erneut zu beweisen (konkretes Beispiel: §6.5, H2).
- **Geschlossene Enums bleiben geschlossen:** Wo ein Enum bewusst **nicht** `#[non_exhaustive]` deklariert ist
  (z. B. das Signal-Typ-Enum der Fusionsschicht), ist das eine architektonische Entscheidung. Eine neue Kategorie
  von Information wird in ein bestehendes offenes Signal integriert, statt das Enum breaking zu erweitern
  (konkretes Beispiel: §6.5, H3).
- **Kein Sicherungsnetz, keine Schema-Änderung:** Persistenzformat-Änderungen (insbesondere am
  FlatBuffers-IPC-Schema) werden nur vorgenommen, wenn ein automatisiertes CI-Drift-Gate zwischen Schema und
  generiertem Code aktiv läuft (konkretes Beispiel: §6.5, H4).
- **Explizite Unvollständigkeit statt stiller Lücken:** Wo ein Subsystem eine neue Datenklasse strukturell nicht
  berücksichtigt, wird dies über ein sichtbares Konfigurations-/Report-Flag markiert, statt die Lücke
  stillschweigend zu tolerieren (konkretes Beispiel: §6.5, H6).

---

<a id="architektur"></a>
## 4. Systemarchitektur: der Crate-DAG

MemFuse gliedert sich in einen mehrschichtigen Rust-Workspace. Abhängigkeiten verlaufen strikt abwärts; eine
Abhängigkeit, die gegen die Schichtrichtung verstößt, gilt als Architekturdefekt, nicht als Stilfrage.

| Layer | Crates | Verantwortung |
|---|---|---|
| **0** | `memfuse-core-ipc-gen`, `memfuse-core` | FlatBuffers-generierter IPC-Code; Kerntypen, Traits, Fehlerbehandlung, `DocId`/`EntityId` (Default 64-Bit, 128-Bit-Variante feature-gated, §6.1). |
| **1** | `memfuse-store` (LSM-Tree-Storage, WAL, Block-Cache), `memfuse-index` (HNSW/DiskANN-Vektorindex, SIMD-Distanz), `memfuse-text` (BM25/BM25F-Volltextindex, deutsche Morphologie), `memfuse-crypto` (AES-256-GCM-SIV, KV-Segment-Security, Deletion-Proof-Kette), `memfuse-graph` (CSR-Graph, PathRAG, Forward-Push-PPR, Leiden-Community-Detection, Hyperkanten), `memfuse-checkpoint` (Snapshotting), `memfuse-calibration` (Score-Kalibrierung, Drift-Erkennung) | Persistenz- und Indexierungs-Primitive. Keine Kenntnis voneinander außerhalb dieser Schicht. |
| **2** | `memfuse-db` | Öffentliche `Collection`-API, 4-Signal-Fusion, Multi-Step-Query-Engine, Kontext-Kompaktierung, Provenance-Tracking. Konsumiert alle Layer-1-Crates. |
| **3** | `memfuse-ollama` (Ollama-Client, Contextual-Chunk-Prefixing), `memfuse-candle` (natives GGUF-Inferenz-Backend, KV-Cache-Bridge), `memfuse-embed` (ONNX-Embeddings, Cross-Encoder-Reranking, feature-gated), `memfuse-agent` (persistente Agent-Workflow-Engine), `memfuse-router` (Contextual-Bandit-Routing), `memfuse-py` (Python-FFI via PyO3, eigener Cargo-Workspace zur Panic-Strategie-Isolation) | Inferenz-Backends und Anwendungslogik oberhalb der Datenschicht. |
| **4** | `memfuse-mcp` (MCP-Server, Sandbox, Cloud-Egress-Gateway) | Externe Schnittstelle für Agenten (stdio-JSON-RPC). |
| **5** | `memfuse-bench` | Reproduzierbarer Benchmark-Harness für Retrieval-Genauigkeit und Latenz. |

### 4.1 Safety-First-Doktrin

Safe Rust ist der Standard; `#![forbid(unsafe_code)]` gilt per Default und wird nur in einer geschlossenen,
dokumentierten Ausnahmeliste durchbrochen — jeweils mit einem `// SAFETY:`-Beweiskommentar direkt am Code, der die
Invarianten (insbesondere Pointer-Alignment) beweist:

- `memfuse-index`: SIMD-Distanzberechnung (AVX2/AVX-512/NEON) und read-only Memory-Mapping des Indexformats.
- `memfuse-store`: plattformspezifische ACL-Durchsetzung für WAL-Dateien unter Windows.
- `memfuse-db`: `mlock`/`munlock` gegen OS-Swapping sensibler RAM-Puffer (feature-gated).
- `memfuse-embed`: C-FFI zum ONNX-Runtime-Backend (feature-gated).
- `memfuse-core-ipc-gen`: automatisch generierter FlatBuffers-IPC-Code.
- `memfuse-router`: SIMD-Intrinsics für die Sherman-Morrison-Matrixarithmetik (§8.2) — die einzige Ausnahme des
  `#![forbid(unsafe_code)]`-Paradigmas außerhalb der oben genannten Crates.

Jeder Crate außerhalb dieser Liste erzwingt `#![forbid(unsafe_code)]` kompilierzeitlich. Bibliothekscode darf
seinen Host-Prozess niemals durch einen Panic zum Absturz bringen — Fehlerbehandlung erfolgt konsequent über
domänenspezifische `Result<T, E>`-Enums, nicht über `unwrap()`/`expect()` auf potenziell toxischen Daten.

---

<a id="speicher"></a>
## 5. Speicherschicht: LSM-Tree, WAL und lock-freies Cache-Management

### 5.1 Grundprinzip

Keine Zustandsänderung wird im Speicher sichtbar gemacht, bevor sie physisch in das Write-Ahead-Log geschrieben
und mit dem Datenträger synchronisiert wurde (WAL-First). Der Systemzustand muss sich allein aus dem Log
rekonstruieren lassen (deterministische Recovery). Schreibzugriffe sperren nicht die gesamte Collection, sondern
nur die betroffenen Schlüssel über eine key-granulare Lock-Hierarchie:

```
collections (RwLock) → kv_locks (schlüssel-granular, KvKeyLocks) → embedder (RwLock)
```

Diese Hierarchie ist für **Einzelschlüssel**-Mutationen ausgelegt und deadlockfrei bewiesen. Jede künftige
Mutation, die mehrere Schlüssel gleichzeitig unter `kv_locks` hält, muss diesen Beweis für den Mehrschlüsselfall
gesondert führen — sie darf sich nicht per Analogieschluss auf den Einzelschlüsselfall berufen (P24-Ergänzung,
§3; konkret angewendet in §6.5, H2).

### 5.2 Block-Cache: von sperrendem LRU zu lock-freiem SIEVE/S3-FIFO

**Befund:** Ein klassisches `RwLock<LruCache>`-Backend zwingt bei **jedem** Cache-Lesetreffer zur Akquise eines
exklusiven Schreib-Locks, um das Element in der doppelt verketteten LRU-Liste an den Kopf zu bewegen. Unter der
hochgradig parallelen Last eines Multi-Agenten-Systems degeneriert dies zu einem massiven Flaschenhals (Verstoß
gegen P25).

**Zielarchitektur:** Der Block-Cache wird als austauschbares `BlockCacheBackend`-Trait geführt. Neben dem
klassischen LRU-Backend (Produktions-Default) spezifiziert diese Architektur ein lock-freies Backend nach dem
**SIEVE**-Prinzip (ergänzbar um **S3-FIFO** für differenzierte Eviction-Strategien):

- **S3-FIFO** evaluiert die Lebensdauer von Objekten über drei FIFO-Warteschlangen (Small ≈ 10 % der Kapazität,
  Main, Ghost). Neue Objekte betreten die Small-Queue; werden sie dort nicht erneut referenziert, scheiden sie
  schnell aus („Quick Demotion"), was verhindert, dass „One-Hit-Wonders" die Main-Queue blockieren.
- **SIEVE** verzichtet vollständig auf Listen-Neuordnung bei einem Lesetreffer: Ein Cache-Hit reduziert sich auf
  das Setzen eines einzigen atomaren `visited`-Bits (`Ordering::Relaxed`), ohne jede Mutation der Listenstruktur.
  Eviction erfolgt über einen umlaufenden Zeiger („Hand"): Ein Objekt mit gesetztem `visited`-Bit wird begnadigt
  (Bit gelöscht, verbleibt im Cache), ein Objekt mit gelöschtem Bit wird verdrängt. Diese „Lazy Promotion"
  eliminiert den CPU-Overhead für Cache-Hits nahezu vollständig und liefert auf verzerrten (skewed) Workloads eine
  höhere Trefferquote als LRU.

**Schnittstellenspezifikation (Rust, Zielzustand):**

```rust
use crossbeam_epoch::{Atomic, Guard, Shared};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub struct SieveNode<K, V> {
    pub key: K,
    pub value: V,
    pub visited: AtomicBool,
    pub next: Atomic<SieveNode<K, V>>,
}

pub struct SieveCacheBackend<K, V> {
    head: Atomic<SieveNode<K, V>>,
    tail: Atomic<SieveNode<K, V>>,
    hand: Atomic<SieveNode<K, V>>,
    capacity: usize,
    size: AtomicUsize,
    index: scc::HashMap<K, Shared<'static, SieveNode<K, V>>>,
}
```

Ein Lesetreffer über `get` erstellt zunächst einen Epochen-Guard (`crossbeam_epoch`), der garantiert, dass der
referenzierte Speicher während der Guard-Lebensdauer nicht freigegeben wird. Der Lookup erfolgt über eine
concurrent Hash-Map; bei Fund reduziert sich der Hit auf `node.visited.store(true, Ordering::Relaxed)` — keine
Mutexe, keine Spin-Locks, keine Cache-Line-Invalidierung durch Pointer-Updates. Epochenbasierte Speicherfreigabe
(Epoch-Based Reclamation, EBR) verhindert dabei ABA-Probleme und Use-After-Free, wenn Objekte entfernt werden,
während andere Threads sie noch lesen.

### 5.3 Write-Ahead-Log: SPSC-Ring-Puffer statt File-Handle-Mutex

Die klassische Implementierung leidet unter geteilter Eigentümerschaft am File-Handle, was zu HMAC-Ketten-Forks
und stillen Datenverlusten führen kann. Die Zielarchitektur sieht eine lock-freie WAL-Pipe auf Basis eines
Single-Producer-Single-Consumer-(SPSC-)Ring-Puffers vor. Die Puffergröße ist zwingend eine Zweierpotenz, um teure
Modulo-Operationen durch bitweise UND-Maskierung (`& (capacity - 1)`) zu ersetzen. Synchronisation erfolgt über
atomare Lese-/Schreibzeiger mit `Ordering::Acquire`/`Ordering::Release`. Ein dedizierter Flusher-Task übernimmt
exklusiv den `fsync`, wodurch die Transaktionslatenz vom tatsächlichen I/O-Durchsatz entkoppelt wird.

| Komponente | Bisheriges Modell | Zielarchitektur | Laufzeit (Hit/Append) |
|---|---|---|---|
| Block-Cache-Lesepfad | `RwLock<LruCache>` (Produktions-Default) | `SieveCacheBackend` mit `crossbeam_epoch` (Opt-in) | $O(1)$ lock-frei |
| WAL-Synchronisation | Mutex pro File-Handle | SPSC-Atomic-Ring-Puffer + Flusher-Actor | lock-freies Append |
| Schreib-Lock-Handoff | Collection-weiter Mutex | Key-granulares Lock-Striping (`kv_locks`) | lock-freie Hash-Auflösung |

---

<a id="graph"></a>
## 6. Wissensgraph-Datenmodell: binäre Kanten und n-äre Hyperkanten

### 6.1 Binäre Kanten als Grundmodell

Der Wissensgraph wird primär als gerichteter, gewichteter Graph in einer CSR-Struktur (Compressed Sparse Row)
gehalten: `Edge { target: EntityId, weight: f32, edge_type: EdgeType, tx_valid_from/to, business_valid_from/to,
source_doc_id }`. `EdgeType` ist als `#[non_exhaustive] enum { Default }` deklariert — strukturell auf einen
impliziten Prädikat-Typ reduziert; es gibt keine typsystemische Unterscheidung nach Relationsart. Kanten tragen
sowohl **transaktionale** Gültigkeit (MVCC-Systemzeit, `tx_valid_from/to`) als auch **fachliche** Gültigkeit
(Business-Zeit, `business_valid_from/to`) — der Graph ist damit bi-temporal.

`DocId` (Default `u64`, feature-gated `u128` via BLAKE3-Truncation, `#[repr(C, align(16))]`) und `EntityId`
(`u64`) bilden die gemeinsame Identitätsgrundlage für alle Kantentypen, binär wie n-är — keine neue ID-Klasse ist
für Hyperkanten nötig. Der `docid-128`-Rollout ist technisch über praktisch den gesamten Crate-DAG vollzogen,
bleibt aber bewusst 🟡 hinter Feature-Flag: Der Wechsel des Produktions-Defaults ist an einen Major-Version-Cutover
gebunden, gebündelt mit der DiskANN-Tier-Vollfreigabe (⚖️, kein Zieltermin normativ festgelegt).

Cascade-Invalidierung (`cascade_invalidate_edges_for_superseded_doc`) markiert Kanten, deren Quelldokument durch
eine neue Version ersetzt wurde, als tombstoniert; sie arbeitet auf `edges_for_doc(doc_id) -> Vec<(EntityId,
EntityId)>` — strikt auf Kantenpaare festgelegt.

Dieses Modell ist performant und ausreichend für die überwiegende Mehrheit von Fakten, die sich als
Subjekt-Prädikat-Objekt-Aussage darstellen lassen. Es stößt jedoch strukturell an eine Grenze, sobald ein Faktum
per Definition mehr als zwei Beteiligte hat: Ein Ereignis wie *„Anthropic (Subjekt) hat Claude Sonnet 5 (Objekt)
am Datum X (Zeit) für den Enterprise-Tier (Qualifier) veröffentlicht (Prädikat)"* lässt sich im binären Modell nur
als mehrere unabhängige Kanten zerlegen (Subjekt→Objekt, Subjekt→Zeit, Subjekt→Qualifier …), wodurch der
Zusammenhang zwischen den Kanten — dass sie *ein und dasselbe* Ereignis beschreiben — verloren geht.

### 6.2 Hyperkanten: Kernidee und Designentscheidung

**Reifegrad: 🔴 spezifiziert, zu bauen.** MemFuse führt eine zweite, orthogonale Kantenklasse ein: **`HyperEdge`**
— eine n-stellige Relation, die mehrere `EntityId`s in klar benannten Rollen (nicht nur „Quelle"/„Ziel") zu einem
einzigen, gemeinsam versionierten und atomar invalidierbaren Faktum verbindet. Binäre `Edge`s bleiben der
Default-Pfad für einfache Subjekt-Prädikat-Objekt-Fakten (Kompatibilität, unangetasteter Hotpath);
`HyperEdge` ist die Erweiterung für Fakten, die per Definition mehr als zwei Beteiligte haben (Ereignisse,
Transaktionen, n-äre Beziehungen, Zitate mit Quelle+Kontext+Zeitpunkt).

**Designentscheidung (verbindlich):** Es wird **kein** generisches RDF-Reifikations-Pattern verwendet (Hyperkante
als eigener Blank-Node mit N binären Kanten zu den Beteiligten). Begründung: Reifikation würde exakt das zu
lösende Problem reproduzieren — der Zusammenhang der Teil-Kanten ginge im CSR-Traversal erneut verloren, und
Cascade-Invalidierung müsste N synthetische Kanten einzeln statt eine Hyperkante atomar treffen. Stattdessen wird
eine kohärente, erstklassige, im Speicher flach liegende Rust-Struktur mit Zero-Copy-Deserialisierung via
FlatBuffers/Mmap spezifiziert.

### 6.3 Datenstruktur

```rust
/// Eindeutige ID einer Hyperkante — eigener Namensraum, kollidiert nicht mit EntityId/DocId.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HyperEdgeId(pub u64);

/// Rollenbezeichner innerhalb einer Hyperkante (z. B. "subject", "object", "time", "location").
/// Interniert über denselben String-Interner wie EdgeType/Prädikate, um Allokationen im Hotpath
/// zu vermeiden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleId(pub u32);

/// Ein Teilnehmer-Tupel: welche Entität füllt welche Rolle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleBinding {
    pub role: RoleId,
    pub entity: EntityId,
}

/// N-äre Hyperkante — verbindet 2..N Entitäten zu einem gemeinsam gültigen Faktum.
/// Zero-Copy-Variante für den Lesepfad: `ArcSlice` hält lediglich Pointer + Referenzzähler auf
/// den Mmap-Bereich, statt den Puffer beim Einlesen aus dem LSM-Storage zu kopieren.
pub struct HyperEdge<'a> {
    pub id: HyperEdgeId,
    pub predicate: EdgeType,                    // Wiederverwendung des bestehenden Prädikat-Typs
    pub participants: ArcSlice<'a, RoleBinding>, // min. 2 Bindings (sonst degeneriert zur binären Edge)
    pub weight: f32,
    pub tx_valid_from: Option<TxId>,
    pub tx_valid_to: Option<TxId>,
    pub business_valid_from: Option<i64>,
    pub business_valid_to: Option<i64>,
    pub source_doc_id: Option<DocId>,
}
```

**Persistenz:** Neues LSM-Präfix `__graph:hyperedge:` (analog zu `__graph:edge:`), Value =
FlatBuffers-serialisiertes `HyperEdge` (Wiederverwendung der bestehenden FlatBuffers-Toolchain,
`memfuse-core-ipc-gen`, keine neue Serialisierungsschicht). Zusätzlicher Sekundärindex
`__graph:hyperedge_by_entity:{EntityId} -> Vec<HyperEdgeId>` für den Zugriffspfad „gib mir alle Hyperkanten, an
denen Entität X in irgendeiner Rolle beteiligt ist". Einführung als **additives** Schema-Feld — kein Breaking
Change für bestehende `Edge`/`PersistedEdgePayload`-Daten.

**API-Oberfläche:** Neue öffentliche Methode
`Collection::relate_n_ary(predicate, participants: &[(RoleId, EntityId)], doc_id) -> Result<HyperEdgeId>`, analog
zur bestehenden `relate()`-Invariante: **muss** sowohl LSM-Write (Primär- und Sekundärindex) als auch
`graph_index`-Registrierung atomar durchführen. `relate()` (binär) bleibt unverändert bestehen und wird **nicht**
intern auf `relate_n_ary` mit 2 Teilnehmern umgestellt — der binäre Pfad bleibt der unangetastete Hotpath.

**Nicht im Scope:** Automatische NLP-seitige Extraktion n-ärer Fakten aus Freitext (separates, vorgelagertes
Thema der Extraction-Pipeline).

### 6.4 Traversal-Semantik

`CsrGraph` erhält eine neue Methode `hyperedges_for_entity(id: EntityId) -> Vec<HyperEdgeId>`, die intern den
Sekundärindex nutzt — **kein** Eingriff in die bestehende CSR-Adjazenzstruktur für binäre Kanten (additive
Erweiterung, keine Migration des Hotpfads). PathRAG wird um einen optionalen Hyperkanten-Expansionsschritt
ergänzt: Beim Erreichen eines Knotens während der Forward-Push-Traversierung (§7.2) werden zusätzlich alle
`RoleBinding`-Partner der an diesem Knoten anliegenden `HyperEdge`s als „virtuelle" Nachbarn mit
rollenspezifischem Gewichtsabschlag eingespeist (Startwert `0.85`, analog zum bestehenden Hop-Abschlagsfaktor).
Diese Einspeisung erfolgt gegen die Forward-Push-Queue-Logik (nicht gegen eine dichte Power-Iteration), da
Forward-Push die produktive PPR-Zielarchitektur ist (§7.2).

### 6.5 Integrationshindernisse H1–H6 und ihre verbindliche Lösung

Diese sechs Hindernisse sind der eigentliche Kern der Spezifikation: Sie benennen nicht nur, *dass* eine
Integration möglich ist, sondern *welches bestehende Invariant unter Druck gerät* und wie es gewahrt bleibt.

#### H1 — RCU-Snapshot-Inkonsistenz zwischen CSR und Hyperkanten-Sekundärindex

**Problem:** `CsrGraph::compact()`/`compact_async()` tauscht den gesamten Graphzustand atomar über einen
`ArcSwap<GraphInner>`-Pointer aus — Leser sehen nie einen gemischten Alt-/Neu-Zustand. Würde der
Hyperkanten-Sekundärindex als **separate** Struktur außerhalb von `GraphInner` geführt, entstünde ein
Zeitfenster, in dem CSR- und Hyperkanten-Zustand auseinanderlaufen: Ein Leser könnte einen `HyperEdgeId` erhalten,
dessen Teilnehmer-Entität im gerade getauschten Snapshot bereits tombstoniert ist.

**Lösung (verbindlich):** Der Hyperkanten-Sekundärindex wird **Teil von `GraphInner` selbst**
(`hyperedge_index: AHashMap<EntityId, Vec<HyperEdgeId>>` plus `hyperedges: AHashMap<HyperEdgeId, HyperEdge>`) —
automatisch vom bestehenden `ArcSwap`-Swap miterfasst, keine neue Synchronisationsprimitive nötig. Die Freigabe
obsoleter Snapshots wird über `crossbeam_epoch` orchestriert: Eine Epoche wird erst inkrementiert und der
Speicher der alten Graphenstruktur erst dann freigegeben, wenn kein aktiver `Guard` mehr Lesezugriff anfordert.
**Konsequenz:** `GraphInner::estimate_memory_bytes()` muss um die Hyperkanten-Anteile erweitert werden, damit der
`compact_async`-Speicherbudget-Check (`max_compaction_peak_memory_mb`) das tatsächliche Peak-Memory bei großen
Hyperkantenmengen nicht unterschätzt. Dies ist **Voraussetzung**, nicht Nachbarthema der Implementierung.

#### H2 — Kanonisches Multi-Key-Locking zur Deadlock-Prävention

**Problem:** Die Lock-Hierarchie `collections → kv_locks (key-granular) → embedder` (§5.1) ist für Mutationspfade
auf **ein** Schlüsselpaar ausgelegt. `relate()` mutiert eine Kante, typischerweise ein Schlüsselpaar.
`relate_n_ary()` mit N Teilnehmern muss dagegen **N Entitäten gleichzeitig** unter `kv_locks` konsistent halten
(Sekundärindex-Einträge für alle N Teilnehmer müssen atomar mit der Hyperkante selbst geschrieben werden). Werden
diese N Locks naiv nacheinander erworben, entsteht ein klassisches Lock-Ordering-Problem: Zwei gleichzeitige
`relate_n_ary`-Aufrufe mit überlappenden, aber unterschiedlich sortierten Teilnehmermengen (`{A,B,C}` vs.
`{C,B,A}`) können sich gegenseitig blockieren.

**Lösung (verbindlich):** `relate_n_ary()` sortiert die Teilnehmer-`EntityId`s vor dem Locking **kanonisch**
(aufsteigend nach `u64`-Wert) und erwirbt die zugehörigen `kv_locks`-Shards strikt in dieser Reihenfolge:

```rust
pub fn relate_n_ary(
    &self,
    predicate: EdgeType,
    participants: &[RoleBinding],
    doc_id: DocId,
) -> Result<HyperEdgeId, GraphMutationError> {
    let mut entities: Vec<EntityId> = participants.iter().map(|p| p.entity).collect();
    // Kanonische Sortierung erzwingt deterministisches Lock-Ordering
    entities.sort_unstable_by_key(|e| e.0);
    entities.dedup();

    let _guards = self.kv_locks.acquire_multi_sorted(&entities)?;
    // Atomare Insertion in den LSM-Tree und Registrierung im RCU-Snapshot (H1)
}
```

Durch aufsteigendes Sortieren greifen alle Threads in exakt derselben Reihenfolge auf die Ressourcen zu — das
eliminiert zyklische Abhängigkeiten im Wait-for-Graph des Schedulers und garantiert Deadlock-Freiheit. Dies ist
ein **neuer Testfall** (mehr als zwei Schlüssel gleichzeitig unter `kv_locks`), kein Wiederverwendungsfall des
Einzelschlüssel-Beweises — eigenes Abnahmekriterium (§14, AK-3).

#### H3 — `SignalKind` ist ein geschlossenes Enum

**Problem:** Im Unterschied zu `EdgeType` (`#[non_exhaustive]`) ist das Signal-Typ-Enum der Fusionsschicht
(`SignalKind`) **nicht** `non_exhaustive` und hat vier feste Varianten (`Vector`, `Text`, `Graph`,
`EdgeReinforcement`) mit `eq_ignore_ascii_case`-basiertem, allokationsfreiem Matching in `from_name()`/`as_str()`.
Ein fünftes `SignalKind::Hyperedge` würde jeden `match`-Arm im Fusionspfad (Gewichtung, Metadata-Merge-Priorität)
zum Anpassen zwingen — ein Breaking Change am geschlossenen Enum mit hohem Regressionsrisiko für den produktiven
binären Fusionspfad (RRF ist Default).

**Lösung (verbindlich):** Hyperkanten-Treffer werden **nicht** als eigenes Signal geführt, sondern fließen als
zusätzliche Kandidaten **in das bestehende `SignalKind::Graph`-Signal** ein — PathRAG liefert ohnehin bereits ein
Graph-RRF-Signal; Hyperkanten-Expansion ist dort ein interner Erweiterungsschritt der Pfadsuche (§6.4:
„virtuelle Nachbarn mit Gewichtsabschlag"), kein separater Signalkanal. `SignalKind` bleibt strukturell
unverändert — dies ist der **einzig zulässige** Integrationspunkt, nicht eine von mehreren Optionen
(Diff-Test-Pflicht: keine neue Variante, siehe §14, AK-4).

#### H4 — FlatBuffers-Schemaerweiterung erfordert ein aktives CI-Drift-Gate

**Problem:** Ein FlatBuffers-Schema und der davon abgeleitete generierte Code (`memfuse-core-ipc-gen`) können
stillschweigend auseinanderlaufen, wenn kein automatisiertes Gate dies verhindert. Hyperkanten fügen zwangsläufig
neue FlatBuffers-Typen (`HyperEdge`, `RoleBinding`) hinzu — der riskanteste denkbare Zeitpunkt für eine
Schemaänderung ist ausgerechnet einer, an dem ein solches Gate fehlt.

**Lösung (verbindlich, harte Vorbedingung):** Ein FlatBuffers-CI-Drift-Gate (Schema-Datei gegen generierten Code
per CI-Job abgesichert) MUSS produktiv und grün sein, **bevor** das `HyperEdge`-FlatBuffers-Schema gemerged wird.
Diese Reihenfolge ist per CI-Job-Abhängigkeit zu erzwingen, nicht nur organisatorisch zu vereinbaren (§14, AK-5).
Ist dieses Gate bereits vorhanden und produktiv, reduziert sich das Kriterium auf den reinen
Ausführungsnachweis zum Zeitpunkt des Hyperkanten-Merges — andernfalls ist es Blocker Nummer eins vor Beginn der
Hyperkanten-Implementierung.

#### H5 — Cascade-Invalidierung: hartes Fan-out-Limit gegen Kostenexplosion

**Problem:** Die bestehende Cascade-Invalidierung für binäre Kanten arbeitet auf klar begrenzter Kardinalität
(ein Dokument erzeugt typischerweise wenige Kanten). Eine Hyperkante mit z. B. sechs Teilnehmern, die selbst
wieder Teil weiterer Hyperkanten sind (Ereignisketten), kann bei naiver Übertragung des Cascade-Musters zu einem
Fan-out führen, der mit der Teilnehmerzahl **nicht linear**, sondern potenziell **quadratisch** wächst (jede
Invalidierung muss den Sekundärindex für alle ihre Teilnehmer aktualisieren) — ein Verstoß gegen P24.

**Lösung (verbindlich, Pflichtbestandteil, keine optionale Härtung):**
`cascade_invalidate_hyperedges_for_superseded_doc()` erhält von Beginn an ein **hartes Fan-out-Limit**
(Default-Vorschlag: 1.000 betroffene Hyperkanten pro Cascade-Lauf), analog zum bestehenden
`MAX_TRAVERSAL_HOPS`/Visited-Node-Limit-Prinzip im CSR-Graphen — Verhinderung von Hub-Node-bedingter
Ressourcenexplosion. Wird das Limit erreicht, wird der Rest asynchron über die bestehende
Hintergrund-Worker-Infrastruktur nachgezogen statt synchron im Schreibpfad abgearbeitet. Dies garantiert
deterministische Antwortzeiten im synchronen Hotpath (§14, AK-6).

| Kriterium | Fehlerbehandlung (`Result<T, E>`) |
|---|---|
| Lock-Timeout | `GraphMutationError::LockAcquisitionTimeout` |
| Fan-out überschritten | `GraphMutationError::PartialCascadeQueued(DeletionProof)` |
| Ungültige Rolle | `GraphMutationError::RoleBindingInvalid` |
| RCU-Snapshot veraltet | `GraphMutationError::EpochReclamationPending` |

#### H6 — Community-Detection/Leiden sieht Hyperkanten nicht

**Problem:** Der Leiden-Algorithmus für GraphRAG-Community-Erkennung operiert ausschließlich auf binären
Kantengewichten und ist strukturell „blind" gegenüber n-ären Fakten. Ohne Erzwingungsmechanismus würde
`detect_communities()` Hyperkanten-Fakten stillschweigend ignorieren, ohne dass dies für Aufrufer sichtbar wird —
ein Nutzer, der annimmt, Community-Detection berücksichtige „den ganzen Graphen", erhielte ein leises,
undokumentiertes Unvollständigkeitsrisiko.

**Lösung (verbindlich für Sichtbarkeit; Projektion selbst ist Folgethema):** `CommunityDetectionConfig` erhält ein
Feld `hyperedges_included: bool` (Default `false`), das im Ergebnistyp `CommunityAssignment`/Report sichtbar
mitgeführt wird — Unvollständigkeit wird explizit statt implizit. Eine echte Hyperkanten-Projektion bleibt
separates Folgethema (§6.6), aber das Fehlen ist ab der Hyperkanten-Einführung messbar/sichtbar, nicht nur bekannt
(§14, AK-7).

### 6.6 Hyperkanten-Projektion für Leiden: Stern-Expansion als Zielarchitektur

Die Standard-Modularitätsfunktion, auf der Leiden basiert,

$$Q = \frac{1}{2m} \sum_{i,j} \left( A_{ij} - \gamma \frac{k_i k_j}{2m} \right) \delta(c_i, c_j)$$

(mit $m$ = Gesamtgewichtsmasse, $k_i$ = Grad von Knoten $i$, $\gamma$ = Resolution-Parameter, $c_i$ = Community
von Knoten $i$) ist ausschließlich für binäre Adjazenzmatrizen $A$ definiert. Um Hyperkanten nicht stillschweigend
zu ignorieren (H6), aber auch ohne den asynchronen C-FFI-Code des Leiden-Solvers zu modifizieren, muss der
Hypergraph $G_H = (V, E_H)$ auf einen binären Graphen projiziert werden. Zwei Projektionen stehen mathematisch zur
Wahl:

1. **Cliquen-Expansion:** Jede Hyperkante $e$ wird in eine Clique transformiert, bei der alle Knoten in $e$
   paarweise verbunden werden, mit Kantengewicht $\frac{w(e)}{|e| - 1}$ (Erhaltung der Gesamtgewichtsmasse). Führt
   bei großen Hyperkanten zu einer $O(|e|^2)$-Kantenexplosion — für hochvernetzte Hyperkanten inakzeptabel.
2. **Stern-Expansion (verbindlich gewählt):** Der Hypergraph wird in einen bipartiten Graphen überführt. Jede
   Hyperkante $e \in E_H$ wird als eigenständiger, künstlicher „Knoten" repräsentiert; es entstehen nur binäre
   Kanten zwischen Entitätsknoten und dem neuen Hyperkanten-Knoten. Die Kantenanzahl skaliert linear mit
   $O(|e|)$.

Um Kopiervorgänge zu vermeiden, wird die bipartite Inzidenzmatrix $H$ **nicht** physisch materialisiert, sondern
über einen typsicheren Iterator dem Leiden-Algorithmus on-the-fly vorgegaukelt (Zero-Allocation
Arena-CSR-Speicherstruktur, analog zu §5.2/§7.2).

### 6.7 Abnahmekriterien für die Hyperkanten-Erweiterung

Siehe §14 für die vollständige, konsolidierte Liste (AK-1 bis AK-8) inklusive Regressionsfreiheit gegenüber dem
bestehenden binären Pfad.

---

<a id="retrieval"></a>
## 7. Retrieval-Pipeline: 4-Signal-Fusion und ihre Algorithmen

### 7.1 4-Signal-Fusion

Jede Hybridsuche kombiniert bis zu vier unabhängige Signale — Vektor (HNSW-k-NN), Text (BM25/BM25F), Graph
(PPR-Traversierung inkl. Hyperkanten-Erweiterung, §6.4/§6.5-H3) und optional Kanten-Reinforcement
(feature-gated) — über das geschlossene `SignalKind`-Enum (§6.5, H3). Die Fusion selbst erfolgt standardmäßig
über **Reciprocal Rank Fusion (RRF)**, score-blind und robust bei Signalausfall. Eine score-normalisierte Fusion
steht als Opt-in mit hartem RRF-Fallback bei Signaldegradation zur Verfügung — RRF bleibt in jedem Fall Default,
kein Ersatz.

### 7.2 Graph-Signal: Gestreamtes Personalized PageRank via Forward-Push

**Problem der dichten Power-Iteration:** Eine PPR-Berechnung über dichte Power-Iteration berechnet die
stationäre Wahrscheinlichkeitsverteilung über den **gesamten** Graphen — $O(V+E)$ pro Anfrage, unabhängig von der
tatsächlichen Seed-Menge. Das verstößt gegen P24 und führt zu enormen Heap-Allokationen bei großen Graphen.

**Zielarchitektur (🟢 produktiv):** Der lokale **Andersen-Chung-Lang-Forward-Push-Algorithmus** exploriert nur
Knoten, die signifikant zur PageRank-Masse des Seed-Knotens beitragen. Die mathematische Spezifikation stützt
sich auf einen Wahrscheinlichkeitsvektor $p$ und einen Restvektor $r$. Initialisierung für Seed-Knoten $s$:
$r(s) = 1$, $p(s) = 0$. Für jeden Knoten $u$, bei dem $\frac{r(u)}{d(u)}$ einen Fehlertoleranz-Schwellenwert
$\epsilon$ überschreitet, wird eine Push-Operation ausgeführt:

1. $p(u) \leftarrow p(u) + \alpha \cdot r(u)$
2. $r(u) \leftarrow (1 - \alpha) \frac{r(u)}{2}$ (Rückhaltung eines Teils der Restmasse)
3. $r(v) \leftarrow r(v) + (1 - \alpha) \frac{r(u)}{2\, d(u)}$ für alle Nachbarn $v$ (gleichmäßige Verteilung der
   verbleibenden Masse)

Die Laufzeit ist strikt durch $O\!\left(\frac{1}{\alpha \epsilon}\right)$ begrenzt — unabhängig von der
Gesamtgröße des Graphen. Der Graph-Lesepfad ist zusätzlich nebenläufigkeitssicher über RCU-Snapshot-Verfahren:
`CsrGraph::compact()` tauscht den gesamten Graphzustand atomar über `ArcSwap<GraphInner>` aus, sodass Leser
niemals einen gemischten Alt-/Neu-Zustand sehen (Grundlage von H1, §6.5).

### 7.3 Volltextsuche: BM25 mit Block-Max WAND und Feldgewichtung (BM25F)

**🟢 produktiv.** Der Volltextindex hält einen residenten In-Memory-Postinglisten-Index mit
Block-Max-WAND-Traversierung für effiziente Top-k-Suche (kein Live-LSM-Range-Scan pro Query-Term) und
unterstützt feldgewichtete Bewertung (BM25F), sodass z. B. Titel- und Fließtext-Treffer unterschiedlich gewichtet
werden. Deutsche Komposita werden morphologisch zerlegt, sodass z. B. „Urlaubsantragsprozess" auch über „Urlaub",
„Antrag", „Prozess" auffindbar ist.

### 7.4 Vektorindex: gestuftes HNSW + DiskANN

**🟢 produktiv (Stufe 0), 🔴 Stufe 1 spezifiziert.** HNSW ist der Standard-Vektorindex (SIMD-beschleunigte
Distanzberechnung, SQ8-Quantisierung mit konfigurierbarem Perzentil-Clipping gegen Codebook-Drift, native
Tombstone-Löschung). Für Korpora, die den verfügbaren RAM übersteigen, steht DiskANN als zweite Indexstufe zur
Verfügung (mmap-basiertes, read-only Zugriffsmuster, ebenfalls native Tombstones).

**Zielarchitektur „HNSW-Dateiformat v2" (🔴, Arena-Allocator):** Der aktuelle Traversierungs-Hotpath erzeugt pro
abgerufenem Nachbarknoten eine Heap-Allokation (`Vec<u32>`) und verwendet redundante `RwLock`-Sperren pro Knoten.
Ein Arena-Allocator reserviert beim Start einen zusammenhängenden Speicherblock; Knoten-Offsets ersetzen rohe
Pointer, was den Hardware-Prefetcher der CPU maximal ausnutzt:

```rust
pub struct HnswArena<const D: usize> {
    /// Lock-freie Arena, von Mmap oder großem Vektor gestützt
    storage: Arc<MmapArena>,
    /// Epochen-basierte Referenzen zur lock-freien Traversierung
    head: crossbeam_epoch::Atomic<NodeRecord<D>>,
    capacity: usize,
}
```

Bei Updates (Neuverlinkung von Knoten) wird die Adjazenzliste nicht via Mutex gesperrt: Eine neue, verlängerte
Liste wird erstellt, und der Zeiger im Knoten wird über ein atomares `compare_exchange` (CAS) ausgetauscht.
Leser-Threads nutzen `crossbeam_epoch`, damit die alte Liste im Speicher verbleibt, bis der letzte Lesevorgang
abgeschlossen ist (Zero-Allocation Traversal).

**NaN-sichere Distanzberechnung:** Innerhalb der HNSW-Distanzschleife ist das Einfügen von Branches
(`if val.is_nan()`) toxisch für die CPU-Pipeline. Stattdessen werden bitweise SIMD-Maskierungen genutzt: Ein
Vektor-Register wird parallel auf `NaN` evaluiert, eine Bit-Maske erzeugt und ungültige Werte über bitweises
UND/ODER auf `0.0` (bzw. auf Distanz $\infty$) gesetzt, ohne dass der Instruction Pointer verzweigen muss —
garantiert Determinismus und Laufzeitstabilität.

### 7.5 Community-Detection: Leiden

**🟢 produktiv (binärer Pfad).** Graph-Clustering für GraphRAG erfolgt über den Leiden-Algorithmus
(deterministisch, garantiert wohlverbundene Communities — im Unterschied zu Label-Propagation oder Louvain).
Die Hyperkanten-Projektion (Stern-Expansion, §6.6) und das Sichtbarkeits-Flag `hyperedges_included` (§6.5, H6)
sind Teil der Hyperkanten-Erweiterung und entsprechend 🔴 spezifiziert, solange Hyperkanten selbst nicht gebaut
sind.

---

<a id="bandit"></a>
## 8. Contextual-Bandit-Routing

MemFuse integriert einen Multi-Armed-Bandit-Router (LinUCB, Li et al. 2010) zur adaptiven Aussteuerung der
Retrieval-Strategien zwischen Volltext, Vektor und Graph.

### 8.1 Zwei Implementierungsvarianten unterschiedlicher mathematischer Korrektheit

- **`DiagonalApproximation` (🟢 Produktions-Default):** `theta[i] += r_adj * xi / sigma_sq[i].max(1e-8);
  sigma_sq[i] += xi * xi;` — eine laufende Summe ohne Renormierung gegen die Ridge-Regression-Matrix. Dies ist
  strukturell ein SGD-artiges Verfahren mit **keiner** exakten Ridge-Regression im Sinne von $\theta = A^{-1}b$.
- **`ShermanMorrison` (🟡 Opt-in, Feature-Flag):** vollständige inkrementelle Matrixinversion über die
  **Sherman-Morrison-Formel**, die eine Rang-1-Aktualisierung der Inversen in $O(d^2)$ statt $O(d^3)$ ermöglicht:

$$(A + xx^\top)^{-1} = A^{-1} - \frac{A^{-1}xx^\top A^{-1}}{1 + x^\top A^{-1} x}$$

wobei $A = \sum x_t x_t^\top + \lambda I$ und $b = \sum r_t x_t$, $\theta = A^{-1}b$. Nur die exakte Inversion von
$A$ generiert die korrekten Konfidenzintervalle, die das Exploration-Exploitation-Dilemma deterministisch lösen —
diese Variante **ist** eine mathematisch korrekte, inkrementelle Ridge-Regression und die einzige, die die
LinUCB-Regret-Garantie tatsächlich erfüllt.

### 8.2 SIMD-optimiertes Update ohne Heap-Allokation

Um strikte Latenzbudgets im Hotpath einzuhalten, wird die Sherman-Morrison-Aktualisierung über SIMD-Intrinsics
(AVX-512/NEON) parallelisiert. Datenstrukturen sind an 64-Byte-Cache-Lines ausgerichtet, um „False Sharing"
zwischen Threads zu verhindern:

```rust
#[repr(C, align(64))]
pub struct AlignedVector<const D: usize> {
    pub data: [f32; D],
}

pub struct ShermanMorrisonBandit<const D: usize> {
    pub inv_a: AlignedVector<{ D * D }>,   // Inverse Kovarianzmatrix, flach im Speicher
    pub b: AlignedVector<D>,
    pub theta: AlignedVector<D>,
}

impl<const D: usize> ShermanMorrisonBandit<D> {
    /// O(d^2) Lock-free Update der Parameter ohne Heap-Allokationen
    pub fn update_rank_1(&mut self, x: &AlignedVector<D>, reward: f32) -> Result<(), BanditError> {
        // 1. v = A^{-1} x via AVX-512-Intrinsics
        // 2. s = 1.0 + x^T * v
        // 3. inv_a -= (v * v^T) / s via FMA-Instruktionen
        // 4. Update b und theta
        Ok(())
    }
}
```

Jeder `unsafe`-Block für `core::arch`-Intrinsics ist zwingend mit einem `// SAFETY:`-Kommentar zu annotieren, der
die Pointer-Alignment-Invarianten beweist (§4.1).

### 8.3 Gedeckelter Lyapunov-Drift-Regelkreis

Das Routing nutzt dynamisches Budgeting über einen PID-Controller. Unter Traffic-Spikes kann der
$\alpha$-Explorationsparameter unbegrenzt nach oben driften (Exploration-Exploitation-Kollaps). **🟢 produktiv:**
Ein zeitfensterbasiertes `drift_decay_window` (Default 50 Zeitschritte, `drift_gamma = 0.95`) deckelt die
Eskalation. Fällt der PID-Regler in die Ausgangssättigung (CPU-Limit-Saturierung), stoppt der Integrator sofort
(Anti-Windup), statt den Fehler unbegrenzt aufzuaddieren. Die Berechnung berücksichtigt das Delta $dt$ zwischen
Abfragen für abtastratenunabhängige Regelung. Dieser Deckel gilt für **beide** Bandit-Implementierungsvarianten
gleichermaßen.

### 8.4 Normative Bewertung

Der Wechsel des Produktions-Defaults von `DiagonalApproximation` zu `ShermanMorrison` ist an ein
CI-Latenzbudget-Gate gebunden (Kriterium: Sherman-Morrison-Latenz < 5 % der medianen LLM/SLM-Inferenzlatenz) —
⚖️ Produktentscheidung, kein Implementierungsrückstand. Diagonal-Approximation bleibt in jedem Fall als
Low-Memory-Opt-out erhalten.

---

<a id="inferenz"></a>
## 9. Inferenz, KV-Cache-Bridge und Zero-Copy-IPC

### 9.1 Zero-Copy-Eviction-Bridge

**🟢 produktiv.** Die Interprozesskommunikation und die Verwaltung des LLM-Kontexts erfordern durchgehende
Zero-Copy-Datenpipelines auf Basis von `Bytes` und Mmap. FlatBuffers erlaubt das direkte Auslesen von Strukturen
aus einem Byte-Slice (`&[u8]`), ohne Puffer im Heap neu anzulegen; der generierte IPC-Code
(`memfuse-core-ipc-gen`) gibt alle Strings und Vektoren als Slice-Referenzen zurück.

### 9.2 LSM-Fallback-Spill bei Speicherdruck

**🟢 produktiv.** Der KV-Cache wird primär verschlüsselt im RAM gehalten (Paged-Encrypted). Bei Speicherdruck
implementiert das System einen kontrollierten LSM-Fallback-Spill auf die SSD, statt den Cache verlustbehaftet zu
verwerfen oder den Prozess außer Speicher laufen zu lassen. Die Sicherheitsschicht (AES-256-GCM-SIV) instanziiert
die Verschlüsselungsinstanz (`Aes256GcmSiv`) einmalig in einem `OnceLock` und übergibt sie an einen dedizierten
Worker-Thread, der Nachrichten über asynchrone Channels (`mpsc`) entgegennimmt — dies vermeidet den vormaligen
Engpass, den erneuten Aufbau des Key-Schedules bei jeder kryptografischen Operation.

### 9.3 Kryptographisch verifizierbare Löschung

Der Zero-Trust-WASM-Sandbox-Ansatz erfordert kryptografisch verifizierbare Deletion Proofs. Löschungen
(Art. 17 DSGVO) sind deterministisch über HMAC-Ketten abgesichert: Ein gelöschter Schlüssel hinterlässt einen
Tombstone, der integraler Bestandteil des Hash-Trees des LSM-Stores bleibt, wodurch die Löschung gegenüber der
Cloud-Egress-Schicht kryptografisch beweisbar ist.

---

<a id="sicherheit"></a>
## 10. Sicherheits- und Datenschutzmodell

### 10.1 Kryptographische Grundlagen

AES-256-GCM-SIV für Daten at rest, WAL mit race-freier HMAC-Kette, `DeletionProof` für DSGVO-Art.-17-Nachweise
(§9.3). Key-granulare Schreibisolation (`kv_locks`) reduziert zusätzlich die Angriffsfläche für lock-basierte
Denial-of-Service-Muster gegenüber einem collection-weiten Mutex.

### 10.2 WASM-Sandbox

**🟢 produktiv.** Zero-Trust-Ausführungsisolation für Agent-Tool-Aufrufe über `wasmtime`. Fuel-Budget (Rechenschritte)
und Wall-Clock-Limit (`max_wall_clock_ms`, Default 5 s) sind orthogonal konfigurierbar (P23) — ein Tool kann
rechnerisch günstig, aber durch blockierendes I/O langsam sein, oder umgekehrt; beide Fälle müssen unabhängig
begrenzbar sein. `#![forbid(unsafe_code)]` gilt für den Sandbox-Crate uneingeschränkt.

### 10.3 Prompt-Injection-Schutz und Zero-Copy-Lesepfad

Bestandteil des Sicherheitsmodells seit der Kernarchitektur, unverändert gültig: Eingaben aus dem Kontext eines
Agenten werden nicht ungeprüft als Steuerbefehle interpretiert; der Lesepfad ist durchgehend Zero-Copy, um
unnötige Pufferkopien sensibler Daten zu vermeiden.

### 10.4 Cloud-Egress Privacy Gateway (Fünf-Schichten-Architektur)

**🟢 produktiv, weitgehend auditiert.** Für den Fall, dass eine Anfrage dennoch an ein Cloud-LLM weitergereicht
werden soll, besteht ein mehrschichtiger DLP-Pfad:

1. **Token-Vaulting/Pattern-Matching (`EgressVault`):** `RegexSet`-Klassifikation sensibler Entitäten, Payload-Deckel.
2. **Vorabstraktion.**
3. **Graph-Generalisierung.**
4. **Bulk-Exfiltration-Detektor:** erkennt großvolumige, potenziell exfiltrierende Anfragemuster.
5. **Re-Hydration:** `CloudResponseRehydrator::rehydrate` übersetzt Surrogate in der Cloud-Antwort zurück in die
   Originalentität — Round-Trip, unbekannte Surrogat-Token (No-Op) und Multibyte-UTF-8-Grenzfälle
   (Panic-Sicherheit) sind Testpflicht.

**Surrogat-Tokenisierung:** `generate_surrogate`/`get_entity` erzeugen eine session-gebundene, bidirektionale
Zuordnung zwischen Originalentität und Platzhalter (`[USER_ENTITY_xxxx]`-Format, Hash-basiert). Ein formaler
Fünf-Schichten-Vollständigkeitsaudit dokumentiert den Reifegrad dieser Kette.

---

<a id="betrieb"></a>
## 11. Betriebsmodi

MemFuse wird ausschließlich eingebettet betrieben: im Prozess des aufrufenden Agenten (Rust- oder Python-Bindung)
oder als lokaler MCP-Server über stdio-JSON-RPC. Es gibt keinen Server-Modus mit Netzwerk-Listener für
Multi-Tenant-Zugriff. Der Cloud-Egress-Pfad (§10.4) ist der einzige Punkt, an dem Daten das lokale System
verlassen — ausschließlich auf explizite Anforderung, nie als Hintergrundtelemetrie.

---

<a id="features"></a>
## 12. Feature-Flag-Politik: Produktions-Default vs. Opt-in

Ein Breaking-Change- oder Performance-Trade-off-Feature wird hinter einem Cargo-Feature isoliert, bis eine
explizite Produktentscheidung (Major-Version-Cutover bzw. CI-Benchmark-Gate-Erfolg) den Wechsel des Defaults
auslöst. Dies ist **kein Mangel**, sondern verbindliche Politik — die Unterscheidung zwischen „im Code korrekt
gelöst" und „im Produktionsbetrieb tatsächlich wirksam" ist für Abnahme- und Sicherheitszwecke wesentlich.

| Feature-Flag | Reifegrad | Beschreibung |
|---|---|---|
| `cloud-egress-guard` | 🟢 | DLP/Egress-Kontrolle, Surrogat-Tokenisierung, Bulk-Exfiltration-Detektor, Rehydration (§10.4) |
| `bandit-routing` | 🟢 | LinUCB-Grundfunktion, Lyapunov-Kopplung, gedeckelte Drift-Alpha-Eskalation (§8) |
| `egress-sherman-morrison` | 🟡 | Mathematisch korrekte Ridge-Regression-Form (§8.1); einziger Pfad mit LinUCB-Regret-Garantie |
| `kv-bridge` | 🟢 | KV-Cache-Bridge inkl. LSM-Fallback-Spill, Bincode + AES-256-GCM-SIV (§9) |
| `wasm-sandbox` | 🟢 | Fuel- und Wall-Clock-Budget orthogonal (§10.2) |
| `experimental-diskann` | 🟢 (offizieller Tier) | Native Tombstones, SQ8-Perzentil-Clipping (§7.4) |
| `docid-128` | 🟡 | 128-Bit-BLAKE3-Truncation-DocId, Rollout über praktisch alle Crates vollzogen, Default bleibt `u64` (§6.1) |
| `block-cache-v2` | 🟡 | SIEVE-/S3-FIFO-artiges Backend, Default bleibt klassisches `RwLock`-LRU (§5.2) |
| `bm25f` | 🟢 | Feldgewichtete BM25-Bewertung (§7.3) |
| `flatbuffers-drift-gate` (xtask) | 🟢 | CI-Gate gegen Schema-Drift — Vorbedingung für jede künftige Schemaerweiterung, insbesondere Hyperkanten (§6.5, H4) |
| `fault-injection` | 🟢 | Test-only |
| `loom` | Dev-Dependency | Nebenläufigkeits-Modelltests für Group-Commit und (künftig) Multi-Key-Locking (§6.5, H2) |
| `adaptive-decay` / `-control` | 🟢 | Kalibrierungs-Feintuning |
| `partial-index-rebuild` | 🟢 | Inkrementeller Indexaufbau |
| `edge-reinforcement-learning` | 🟢 (Feature-Gate) | Kantenverstärkung als optionales fünftes Fusionsverhalten |
| **Hyperkanten (`relate_n_ary`, `HyperEdge`)** | **🔴 kein Flag — nicht implementiert** | Siehe §6 |

---

<a id="governance"></a>
## 13. Governance & Entwicklungsprozess

- **Keine Statusaussage ohne Code-Gegenprobe:** Jede Aussage über den Reifegrad einer Fähigkeit beruht auf einer
  Prüfung des tatsächlichen Codes (Dateilektüre, Tests, CI-Konfiguration) zum Zeitpunkt der Aussage — nicht auf
  der Übernahme einer früheren, möglicherweise überholten Statuszeile.
- **Persistenzformat-Änderungen sind an ein aktives CI-Drift-Gate gebunden** (§3, §6.5 H4) — keine Ausnahme,
  auch nicht unter Zeitdruck.
- **ADR-Disziplin:** Architekturentscheidungen mit Breaking-Change-Charakter (Bandit-Default, BlockCache-Default,
  DocId-128-Cutover, HNSW/DiskANN-Stufenmodell, Fusionsstrategie) werden als eigenständige ADRs geführt, an ein
  messbares Kriterium (Benchmark-Gate, Major-Release) gebunden und bei Code-Umsetzung formal revidiert — Code, der
  seiner ADR-Dokumentation vorauseilt, ist ein zu schließender Governance-Befund, kein Dauerzustand.
- **Explizite Nicht-Verifikationen werden benannt, nicht verschwiegen:** Wo eine Aussage (z. B. Approximationsgüte
  eines Algorithmus, Inhalt eines Audit-Reports, tatsächliches CI-Laufergebnis) nicht bis auf Ausführungsebene
  geprüft wurde, wird dies als solches gekennzeichnet, statt stillschweigend als „erledigt" geführt.
- **Kein Merge risikoreicher Schemaänderungen ohne Sicherungsnetz** (§6.5, H4) — dies ist die zentrale, aus der
  Hyperkanten-Analyse gewonnene und auf jede künftige Schemaerweiterung übertragbare Lehre.

---

<a id="abnahme"></a>
## 14. Abnahmekriterien des Endprodukts

### 14.1 Kernsystem (Auszug, normativ)

1. `insert_lock` vollständig durch key-granulare `kv_locks` ersetzt — kein collection-weiter Schreib-Mutex im Pfad.
2. HNSW-Hotpath nutzt unaligned-SIMD-Distanzkernel statt Heap-Allokation pro Distanzberechnung.
3. BM25-Suche nutzt residenten Postinglisten-Index mit Block-Max WAND, kein Live-LSM-Scan pro Query-Term.
4. Block-Cache erzwingt bei reinem Lesetreffer im Opt-in-Backend (`block-cache-v2`) keinen exklusiven Write-Lock.
5. DiskANN kann ohne HNSW-Fallback löschen (native Tombstones).
6. Bandit-Router-Latenzbudget ist CI-gated, nicht nur literaturbasiert behauptet.
7. Graph-`compact()` blockiert keine nebenläufigen Leser während des Rebuilds (RCU-Snapshot-Swap).
8. PPR-Retrieval-Anfrage hat eine zur Seed-Menge, nicht zur Graphgröße proportionale Laufzeit (P24).
9. `build_provenance` nutzt eine `ProvenanceBuilder`-Struktur statt vieler Positionsargumente.
10. FlatBuffers-generierter Code hat ein CI-Drift-Gate gegen die Schema-Definition.
11. `SignalKind::from_name`/`SignalKey::from_name` sind allokationsfrei (`eq_ignore_ascii_case`).
12. BM25F ist produktiv nutzbar.
13. KV-Cache-Bridge verfügt über LSM-Fallback-Spill mit Testabdeckung für RAM/LSM-Hit/Miss-Kombinationen.
14. Bandit-Drift-Alpha-Eskalation ist zeitlich und wertmäßig gedeckelt.
15. Cloud-Egress-Gateway: Layer-4-Bulk-Exfiltration-Detektor produktiv, Rehydration inkl. Multibyte-UTF-8-Sicherheit verifiziert.
16. Der Produktions-Default des Bandit-Routers implementiert (⚖️, sobald Gate besteht) eine mathematisch korrekte
    Ridge-Regression ($\theta = A^{-1}b$).
17. Der Produktions-Default des Block-Caches zeigt (⚖️, sobald entschieden) S3-FIFO-/SIEVE-Verhalten.
18. DocId-128 ist (⚖️, gebündelt mit Major-Release) Produktions-Default.
19. Group-Commit-Loom-Test läuft sichtbar grün in CI.

### 14.2 Abnahmekriterien für Hyperkanten (AK-1 bis AK-8, normativ und abschließend)

1. `HyperEdge` mit ≥3 `RoleBinding`s persistiert, überlebt Prozess-Neustart, korrekt über `hyperedges_for_entity`
   für jede beteiligte Entität auffindbar — **und** bleibt nach einem `compact()`-Lauf, der gleichzeitig mit einer
   laufenden Leseoperation ausgeführt wird, konsistent (Regressionstest gegen H1: kein Leser darf je einen
   `HyperEdgeId` sehen, dessen Teilnehmer im selben Snapshot bereits tombstoniert ist).
2. `GraphInner::estimate_memory_bytes()` schließt Hyperkanten-Strukturen ein; der `compact_async`-Budget-Check
   (`max_compaction_peak_memory_mb`) greift nachweislich auch bei Hyperkanten-dominiertem Speicherwachstum.
3. Zwei gleichzeitige `relate_n_ary`-Aufrufe mit überlappenden, unterschiedlich geordneten Teilnehmermengen
   terminieren beide ohne Deadlock (Loom- oder Stress-Test).
4. PathRAG-Hyperkanten-Expansion fließt ausschließlich über das bestehende `SignalKind::Graph`-Signal;
   `SignalKind`-Enum bleibt strukturell unverändert (Diff-Test: keine neue Variante).
5. Das FlatBuffers-CI-Drift-Gate ist grün in CI, **bevor** das `HyperEdge`-FlatBuffers-Schema gemerged wird
   (Reihenfolge-Constraint, per CI-Job-Abhängigkeit erzwungen, nicht nur organisatorisch vereinbart).
6. `cascade_invalidate_hyperedges_for_superseded_doc` bricht bei >1.000 betroffenen Hyperkanten kontrolliert auf
   Hintergrundverarbeitung um; ein Test mit synthetischem High-Fan-out-Graphen belegt, dass der synchrone
   Schreibpfad dabei keine unbeschränkte Latenzspitze erzeugt.
7. `CommunityDetectionConfig::hyperedges_included` ist im Report sichtbar `false`, solange keine
   Hyperkanten-Projektion existiert — ein Test prüft das Flag selbst, nicht nur die zugrundeliegende Funktionalität.
8. Kein Regressions-Impact auf bestehende binäre `relate()`/`Edge`-Benchmarks (Hotpath bleibt unangetastet).

---

<a id="matrix"></a>
## 15. Rückverfolgbarkeitsmatrix

| Bereich | Reifegrad | Verweis |
|---|---|---|
| Key-granulare `kv_locks` statt collection-weitem Mutex | 🟢 | §5.1 |
| HNSW-SIMD-Hotpath (unaligned Distanzkernel, `AHashSet`-Vorallokation, `try_write()`-Pruning) | 🟢 | §7.4 |
| SQ8-Perzentil-Clipping | 🟢 | §7.4 |
| BM25 residenter Index + Block-Max WAND | 🟢 | §7.3 |
| BM25F feldgewichtete Bewertung | 🟢 | §7.3 |
| Block-Cache: klassisches LRU | 🟢 (Default) | §5.2 |
| Block-Cache: SIEVE/S3-FIFO | 🟡 | §5.2 |
| Sherman-Morrison-Bandit + CI-Latency-Gate | 🟡 (Opt-in) / Gate 🟢 | §8 |
| Leiden statt Label-Propagation (binärer Pfad) | 🟢 | §7.5 |
| RCU-Snapshot-Swap für `CsrGraph::compact()` | 🟢 | §7.2 |
| Native DiskANN-Tombstones | 🟢 | §7.4 |
| DocId-128-Bit-Migration (Rollout) | 🟡 | §6.1 |
| Score-normalisierte Fusion mit RRF-Fallback | 🟡 (Opt-in) | §7.1 |
| Forward-Push-PPR | 🟢 | §7.2 |
| `ProvenanceBuilder`-Struktur | 🟢 | §14 |
| FlatBuffers-CI-Drift-Gate | 🟢 | §6.5 (H4), §13 |
| Cloud-Egress Fünf-Schichten (Surrogat-Tokenisierung, Bulk-Exfiltration, Rehydration) | 🟢 | §10.4 |
| KV-Cache-Bridge LSM-Fallback-Spill | 🟢 | §9.2 |
| Bandit-Drift-Alpha-Eskalation gedeckelt | 🟢 | §8.3 |
| Loom-Test sichtbar grün in CI | 🔴 | §14 |
| HNSW-Dateiformat v2 (Arena) | 🔴 | §7.4 |
| RaBitQ/PQ-Quantisierung jenseits SQ8 | 🔴 | §7.4 |
| ADR-Formalrevision (Leiden statt LPA) | 🔴 (Dokumentationsnacharbeit) | §13 |
| **N-äre Hyperkanten (gesamt: Datenmodell, H1–H6, `relate_n_ary`)** | **🔴 vollständig spezifiziert, nicht implementiert** | §6 |

---

<a id="roadmap"></a>
## 16. Roadmap

### Stufe 0 — Unmittelbar

1. Loom-Test für Group-Commit sichtbar grün in CI (reine Verifikationslücke, Test existiert bereits).
2. Benchmark-Ausführung des Bandit-Latency-Gates mit produktivem $d$, um die Sherman-Morrison-Umstellung von
   „Gate existiert" zu „Gate bestanden, Default umgestellt" zu überführen.

### Stufe 1 — Strukturell

3. **N-äre Hyperkanten** (§6), vollständig spezifiziert, unmittelbar startbereit — kein externer Blocker mehr,
   sofern das FlatBuffers-CI-Drift-Gate (H4) produktiv ist. Reihenfolge innerhalb der Implementierung: H4-Nachweis
   → Datenmodell (§6.3) → kanonisches Locking (§6.5 H2) → RCU-Integration (§6.5 H1) → PathRAG-Anschluss (§6.4,
   §6.5 H3) → Cascade-Fan-out-Limit (§6.5 H5) → Leiden-Sichtbarkeitsflag (§6.5 H6) → Stern-Expansion-Projektion
   (§6.6, separates Folgethema).
4. HNSW-Dateiformat v2 (Arena + CSR + allokationsfreie Traversierung).

### Stufe 2 — Produktions-Default-Entscheidungen

5. Bandit-Default `DiagonalApproximation` → `ShermanMorrison`, sobald CI-Gate besteht.
6. Block-Cache-Default `RwLock`-LRU → SIEVE-/S3-FIFO-Backend.
7. RaBitQ- oder PQ-Evaluierung (nach HNSW v2).

### Stufe 3 — Governance & Produktentscheidungen

8. Formale ADR-Revision der Leiden-Umstellung (Code ist der Dokumentation voraus).
9. Major-Release-Planung: 128-Bit-DocId-Cutover mit DiskANN-Tier-Vollfreigabe bündeln — technische Vorbedingung
   ist bereits vollständig erfüllt.

### Stufe 4 — Fernziele

Memory Consolidation (`consolidate_via_llm()`), CausalEdge, passives WAL-Shipping, vollständige
`ProvenanceRecord`-API-Exposition, `edge-reinforcement-learning`-Vollspezifikation, automatische NLP-Extraktion
n-ärer Fakten aus Freitext (explizit außerhalb des Scopes der Hyperkanten-Spezifikation, §6.3).

---

*Diese Spezifikation ist die einzige normative Produktquelle für MemFuse. Sie ist in sich geschlossen: Jede
frühere Fassung, jedes Delta-Dokument und jede Zwischen-Review-Notiz zu den hier behandelten Themen — insbesondere
zur Hyperkanten-Erweiterung (§6) — ist durch dieses Dokument vollständig ersetzt und wird nicht mehr referenziert.
Künftige Änderungen erfolgen als direkte Überarbeitung dieses Dokuments, nicht als weiteres Delta-Dokument.*


## 17. Opus Optimierungen (Roadmap Ergänzung)


Dieser Plan enthält ausschließlich Maßnahmen, die aus den Opus-Architektur-Reviews stammen, im aktuellen Code verifiziert noch **nicht behoben** sind und sich als konkrete, abgrenzbare Implementierungsschritte umsetzen lassen. Bereits erledigte Punkte (WAL-Actor-Exklusivität, PID-Regler mit `dt`, Sherman-Morrison-Bandit, Hyperkanten-Grundgerüst, Block-Cache-v2, `to_lowercase`-Fix, HMAC-Lock-Handoff u. a.) sind nicht enthalten.

Die Reihenfolge der Stufen ist bindend zu verstehen: Stufe 0 blockiert bzw. gefährdet den Betrieb, Stufe 1 ist der größte Hebel für Latenz/Durchsatz, Stufe 2 betrifft Speicherverbrauch und Struktur, Stufe 3 ist Prozess/Governance.

---

## Stufe 0 — Korrektheit & Betriebssicherheit (zuerst)

### 0.1 WAL-Replay: Panic-Pfad bei korrupter oder sich ändernder Datei entschärfen
**Problem:** Die Replay-Routine liest die Dateigröße einmalig vor dem `mmap`, prüft alle Zugriffsgrenzen aber gegen diesen separat gehaltenen Wert statt gegen die tatsächliche Länge der gemappten Region. Verkürzt sich die Datei zwischen beiden Schritten (abgebrochener Schreibvorgang, konkurrierender Prozess, Netzwerk-Dateisystem), führt die anschließende direkte Byte-Indizierung zu einem Index-Panic statt zu einem behandelten Fehler. Mit `panic = "abort"` im Release-Profil beendet das den Prozess beim Öffnen der Datenbank — im schlimmsten Fall bei jedem Neustart erneut (Crash-Loop).
**Maßnahme:**
- Dateigröße ausschließlich aus `mmap.len()` ableiten, keinen separat gelesenen `file_size`-Parameter mehr durchreichen.
- Alle Slice-Zugriffe auf `mmap.get(a..b)` mit `.ok_or(WalCorruption)` statt direkter Indizierung umstellen.
- Die zweite, parallele Scan-Implementierung auf denselben Hilfsfunktions-Pfad reduzieren, damit nicht zwei Implementierungen mit potenziell unterschiedlichen Schranken existieren.
**Nutzen:** Eliminiert den mit Abstand risikoreichsten verbleibenden Panic-Pfad im Storage-Layer; sehr geringer Aufwand bei hoher Risikoreduktion.

### 0.2 Bandit: Dimensionsprüfung von `debug_assert` auf harte Fehlerbehandlung umstellen
**Problem:** `score()` und `update()` des Contextual-Bandit-Routings prüfen die Übereinstimmung von Kontext- und Gewichtsvektor-Dimension nur über `debug_assert_eq!`. Im Release-Build greift diese Prüfung nicht; ein interner `zip` kürzt beide Vektoren stillschweigend auf die kürzere Länge. Ein Dimensionswechsel des Embedding-Modells (z. B. nach einem Modell-Upgrade) würde damit zu einem stillen Teil-Skalarprodukt in der Routing-Entscheidung führen, statt einen Fehler auszulösen. Zusätzlich wird der persistierte Bandit-Zustand ohne Dimensions-Versionierung serialisiert.
**Maßnahme:**
- Dimensionsprüfung in `score()`/`update()` als `Result`-Rückgabe mit eigenem Fehlervariant (`DimensionMismatch { expected, actual }`) statt `debug_assert`.
- Ein Versionsfeld (oder die erwartete Dimension) in `BanditProfileState` mit persistieren und beim Laden gegen die aktuell konfigurierte Embedding-Dimension prüfen.
**Nutzen:** Verhindert eine stillschweigend falsche Routing-Entscheidung nach Modellwechsel — geringer Aufwand, hohe Korrektheitswirkung.

### 0.3 Lyapunov-Drift-Erkennung mit dem Bandit verdrahten
**Problem:** Der Drift-Wächter erkennt Verteilungsverschiebungen der Non-Conformity-Scores zuverlässig und protokolliert sie, ruft aber an keiner Stelle die dafür vorgesehene Reaktionsmethode des Bandits auf. Die Eskalation der Exploration bei erkanntem Drift existiert im Code, ist aber vollständig unverdrahtet und wird derzeit nur aus dem eigenen Unit-Test heraus aufgerufen.
**Maßnahme:** An der Stelle, an der `DriftDetected` erkannt und aktuell nur geloggt wird, zusätzlich die Drift-Reaktionsmethode auf dem zum jeweiligen Profil gehörenden Bandit-Zustand aufrufen (mit einem konfigurierbaren `k_drift`-Faktor).
**Nutzen:** Schließt die einzige fehlende Verbindung zwischen zwei bereits fertigen Komponenten; erschließt eine bereits gebaute, aber bislang wirkungslose Funktion. Sehr geringer Aufwand.

### 0.4 Cloud-Egress-Klassifizierung korrigieren
**Problem:** Die einzige Werkzeugmethode, die eine Anfrage tatsächlich das lokale System verlassen lässt, ist derselben, permissiven Policy-Kategorie zugeordnet wie reine lokale Lesezugriffe. Sie wird dadurch von einem Flag gesteuert, das für harmlose Leseoperationen gedacht ist, nicht für Daten, die den Rechner verlassen.
**Maßnahme:** Eine eigene Kategorie für Cloud-Egress-Methoden einführen, mit eigenem, restriktiverem Default-Policy-Feld, das unabhängig vom allgemeinen Lesezugriffs-Flag konfiguriert wird.
**Nutzen:** Schließt eine sicherheitsrelevante Fehlkategorisierung; geringer Aufwand.

### 0.5 Recovery-Pfad für offene Transaktions-Intents differenzieren
**Problem:** Der Wiederherstellungspfad beim Öffnen der Datenbank behandelt jeden gefundenen offenen Transaktions-Intent unbedingt als „vorwärts committen und Indizes nachziehen“ — unabhängig davon, ob die Transaktion ursprünglich erfolgreich war oder mit einem Fehler abgebrochen wurde. Erfolg und Fehlschlag können dadurch denselben On-Disk-Endzustand erzeugen.
**Maßnahme:** Den Intent-Datensatz um einen expliziten Ergebnisstatus (committed/aborted) erweitern, der beim Schreiben des Intents festgelegt und bei Repair-on-Open ausgewertet wird, statt pauschal vorwärts zu committen.
**Nutzen:** Verhindert, dass abgebrochene Transaktionen nach einem Neustart fälschlich als abgeschlossen behandelt werden. Mittlerer Aufwand, da der Intent-Schreibpfad und alle Aufrufer angepasst werden müssen.

---

## Stufe 1 — Hot-Path-Performance (größter Hebel)

### 1.1 HNSW: Nachbarlisten-Auflösung ohne Allokation
**Problem:** Die zentrale Funktion, die für jeden besuchten Knoten während einer Suche die Nachbarliste auflöst, gibt in jedem ihrer Rückgabepfade eine eigens allokierte Kopie zurück, obwohl die Signatur eine Referenz-oder-Kopie-Abstraktion vorsieht. Eine Suche, die mehrere hundert bis tausend Knoten besucht, erzeugt dadurch ebenso viele Heap-Allokationen allein für Adjazenzlisten — der mit Abstand größte Einzelfaktor im Suchpfad.
**Maßnahme:**
- Kurzfristig (ohne Formatwechsel): Wo die Nachbarn bereits als Referenz vorliegen (In-RAM-Knoten ohne konkurrierenden Schreibzugriff), tatsächlich den Referenz-Zweig statt der Kopie zurückgeben.
- Mittelfristig: Umstellung des On-Disk-/Mmap-Knotenformats auf einen festen, ausgerichteten Datensatz-Stride mit pro Layer zusammenhängend abgelegten Nachbar-Indizes, sodass Nachbarlisten direkt als Byte-Slice referenziert statt dekodiert und kopiert werden können.
**Nutzen:** Größter einzelner Effizienzgewinn im gesamten Suchpfad; die kurzfristige Teilmaßnahme ist mit geringem Aufwand umsetzbar, die vollständige Formatumstellung ist aufwendiger, aber Voraussetzung für alle weiteren HNSW-Optimierungen.

### 1.2 HNSW: Backlink-Auflösung von O(P×B) auf O(1) pro Schritt
**Problem:** Während eines Batch-Inserts wird für jeden im aktuellen Suchschritt expandierten Nachbarn eine lineare Suche über die gesamte Liste bereits vorbereiteter Einfüge-Operationen und deren Backlinks durchgeführt. Bei größeren Batches multipliziert sich das zu einer quadratischen Gesamtkomplexität über die Suche.
**Maßnahme:** Einmal pro Suche eine Hash-Map von `(Nachbar-Index, Layer)` auf die zugehörige aktualisierte Nachbarliste aus den vorbereiteten Einfüge-Operationen aufbauen und im Suchkontext mitführen; Nachschlagen wird dadurch O(1) statt einer linearen Suche pro Schritt.
**Nutzen:** Direkter algorithmischer Gewinn bei Batch-Inserts, insbesondere bei hoher Schreiblast; geringer Umsetzungsaufwand.

### 1.3 HNSW: Lock- und Allokationsvermeidung im Distanzpfad
**Problem:** Innerhalb der Distanzberechnung für quantisierte Vektoren wird bei jedem einzelnen Kandidaten ein Lese-Lock auf den gemeinsam genutzten Quantisierer erneut angefordert. Zusätzlich wird beim mmap-gestützten Distanzpfad der Vektor bei jeder Berechnung Element für Element dekodiert und in einen neu allokierten Puffer geschrieben, statt direkt aus dem gemappten Speicher zu lesen — der eigentliche Zweck der Mmap-Nutzung wird dadurch im heißesten Pfad wieder aufgehoben.
**Maßnahme:**
- Lock auf den Quantisierer einmalig pro Suchaufruf (nicht pro Kandidat) erwerben und als Referenz durch die Distanzschleife reichen.
- Distanzfunktion so umbauen, dass sie direkt auf dem gemappten Byte-Slice operiert (z. B. über SIMD-Kernel, die auf `&[u8]`/`&[f32]`-Slices arbeiten), statt vorab in einen Heap-Vektor zu dekodieren.
**Nutzen:** Reduziert Lock-Kontention und Allokationsrate im am häufigsten durchlaufenen Codepfad des Systems messbar; mittlerer Aufwand, hoher Ertrag.

### 1.4 SSTable: Unnötige Kopie beim Block-Lesen entfernen
**Problem:** Beim Lesen eines Datenblocks wird nach erfolgreicher CRC-Prüfung eine vollständige Kopie des Nutzdaten-Anteils angelegt, nur um die vorangestellten Prüfsummen-Bytes abzuschneiden — obwohl der zugrunde liegende Puffertyp Zero-Copy-Slicing mit geteiltem Referenzzähler unterstützt.
**Maßnahme:** Die Kopieroperation durch ein referenzzählendes Slicing des bestehenden Puffers ersetzen, sodass keine zusätzliche Allokation und kein Memcopy der vollen Blockgröße mehr anfällt.
**Nutzen:** Spart bei jedem Cache-Miss eine vollständige Blockkopie; trivialer Aufwand, sofort messbar.

### 1.5 Kryptografie: AES-Schlüsselplan wiederverwenden statt pro Operation neu aufzubauen
**Problem:** Bei jeder Verschlüsselungs-/Entschlüsselungsoperation wird die AES-256-GCM-SIV-Instanz aus dem Schlüsselmaterial neu aufgebaut. Der Aufbau des Schlüsselplans ist der teuerste Teil der Operation und wird dadurch bei jedem einzelnen Aufruf unnötig wiederholt — betrifft sowohl den WAL-Verschlüsselungspfad als auch Sandbox-/Vault-Operationen.
**Maßnahme:** Die Cipher-Instanz einmalig pro Schlüssel (bzw. Schlüsselgeneration) aufbauen und danach wiederverwenden — etwa über eine einmalig initialisierte, threadsicher geteilte Referenz, die bei Schlüsselrotation gezielt ausgetauscht wird, statt bei jeder Operation neu konstruiert zu werden.
**Nutzen:** Reduziert CPU-Kosten auf allen kryptografisch gesicherten Pfaden spürbar; mittlerer Aufwand wegen der Schlüsselrotations-Semantik, die erhalten bleiben muss.

### 1.6 MemTable: Von Hash-Sharding auf Range-/Präfix-Sharding umstellen
**Problem:** Die MemTable verteilt Schlüssel über eine feste Anzahl unabhängiger, hash-basiert ausgewählter Partitionen. Das beschleunigt einzelne Punktschreibungen, zerstört aber die beiden dominanten Zugriffsmuster des Systems: Beim Flush müssen alle Partitionen zusammengeführt und global neu sortiert werden (statt einer reinen Konkatenation), und ein Präfix-Scan über einen Namensraum muss grundsätzlich alle Partitionen durchsuchen, statt nur die eine, die den Präfix tatsächlich enthält.
**Maßnahme:** Sharding-Grenzen aus dem Namensraum-Präfix ableiten (Range-Sharding), sodass jede Partition intern zusammenhängend bleibt. Alternativ, falls die Präfixverteilung zu ungleichmäßig ist: eine einzelne nebenläufige, geordnete Datenstruktur ohne Sharding-Kompromiss einsetzen.
**Nutzen:** Macht den Flush-Pfad sortierfrei und reduziert Präfix-Scans von „alle Partitionen“ auf „eine Partition“; hoher Aufwand, aber hoher struktureller Ertrag für den am häufigsten durchlaufenen Schreibpfad.

### 1.7 Block-Cache: Byte-basierte Kapazität statt Eintragsanzahl
**Problem:** Die konfigurierte Cache-Kapazität wird in Anzahl Einträgen angegeben und implizit auf eine feste Blockgröße hochgerechnet. Blöcke können jedoch bis zu einer deutlich größeren Maximalgröße anwachsen, sodass ein nominell klein konfigurierter Cache tatsächlich ein Vielfaches an Speicher belegen kann, ohne dass dies vom zentralen Ressourcen-Budget erfasst wird.
**Maßnahme:** Kapazität byte-basiert statt eintragsbasiert führen (Eviction anhand der tatsächlichen Bytegröße der zwischengespeicherten Werte); Anbindung an das zentrale Ressourcen-Tracking, damit der Cache im Gesamtspeicherbudget sichtbar ist.
**Nutzen:** Verhindert unvorhersehbaren Speicherverbrauch unter variabler Blockgröße; mittlerer Aufwand.

### 1.8 RRF-Fusion: `build_provenance`-Signatur entschärfen
**Problem:** Die Funktion zum Aufbau des Provenance-Datensatzes nimmt vierzehn positionelle Parameter entgegen, davon acht vom identischen Typ (`Option<f32>`/`Option<u32>`). Eine Vertauschung zweier Argumente an der Aufrufstelle (z. B. Text- und Graph-Score) kompiliert fehlerfrei und erzeugt eine stillschweigend falsche Provenienz-Zuordnung.
**Maßnahme:** Umstellung auf eine benannte Struct mit einem Feld pro Signal (oder ein `[SignalInput; N]`-Array), sodass Vertauschungen durch das Typsystem ausgeschlossen werden; zusätzlich `&str` statt `String` für Textfelder, um eine Allokation pro Ergebnis einzusparen.
**Nutzen:** Beseitigt eine stille Fehlerquelle in der Ergebnis-Provenienz und eine unnötige Allokation pro Treffer; geringer bis mittlerer Aufwand (Signaturänderung mit mehreren Aufrufstellen).

### 1.9 Text-Suche: Posting-Format von Einzelschlüssel- auf Listenspeicherung umstellen
**Problem:** Jedes einzelne Posting (Term–Dokument-Paar) wird als eigenständiges Schlüssel-Wert-Paar abgelegt, mit dem Term und der Dokument-ID als Teil des Schlüssels. Das führt zu erheblicher Schreibverstärkung (ein Dokument mit vielen Termen erzeugt ebenso viele einzelne Schreiboperationen), zu erheblicher Leseverstärkung (ein häufiger Suchbegriff löst einen unbegrenzten Präfix-Scan über potenziell hunderttausende Einzelschlüssel aus) und zu einem Speicher-Overhead, bei dem der Schlüssel ein Vielfaches des eigentlichen Werts ausmacht.
**Maßnahme:** Umstellung auf ein Format, bei dem die vollständige Postingliste eines Terms als ein einziger Wert abgelegt wird (z. B. Delta-kodierte Dokument-IDs mit Term-Frequenz und Dokumentlänge je Eintrag), ergänzt um einen separat gepflegten Dokumentfrequenz-Zähler pro Term. Das ermöglicht einen einzelnen Lesezugriff pro Suchbegriff statt eines unbegrenzten Scans und entfernt gleichzeitig das separate Nachladen der Dokumentlänge pro Treffer.
**Nutzen:** Größter struktureller Einzelbefund im Textsuche-Pfad; hoher Aufwand (Formatwechsel inkl. Migration bestehender Indizes), aber Voraussetzung für jede weitere Optimierung der Volltextsuche (u. a. echte Top-k-Abbruchbedingungen).

### 1.10 RRF/Textsuche: Volle Sortierung durch begrenzte Selektion ersetzen
**Problem:** Sowohl in der Signal-Fusion als auch in der Textsuche werden sämtliche Kandidaten vollständig sortiert und danach auf die gewünschte Trefferzahl gekürzt, obwohl nur die besten k Einträge benötigt werden.
**Maßnahme:** Auf eine Selektionsmethode umstellen, die die besten k Elemente in linearer Zeit bestimmt und erst diese Teilmenge sortiert, statt die Gesamtmenge vollständig zu sortieren.
**Nutzen:** Reduziert die Komplexität von „M·log M“ auf „M“ bei großer Kandidatenzahl; geringer Umsetzungsaufwand, sobald 1.9 (Textsuche) bzw. die bestehende Fusion-Struktur dies zulässt.

---

## Stufe 2 — Speicher & Struktur

### 2.1 Graph: Inkrementelle Kompaktierung statt vollständigem Rebuild pro Anfrage
**Problem:** Der Personalized-PageRank-Pfad löst bei jeder Anfrage mit Graph-Signal einen vollständigen Rebuild sämtlicher CSR-Spalten-Arrays aus, unabhängig davon, wie viele Änderungen seit dem letzten Rebuild tatsächlich aufgelaufen sind. Der Aufwand pro Anfrage skaliert damit mit der Gesamtgröße des Graphen statt mit der Menge der tatsächlichen Änderungen.
**Maßnahme:** Kompaktierung auf ein inkrementelles Schema umstellen (append-only Delta-Segmente plus periodischer Merge im Hintergrund), sodass Traversierungen den stabilen CSR-Bestand plus ein kleines Delta lesen, statt bei jeder Anfrage neu aufzubauen.
**Nutzen:** Entfernt einen Faktor „Gesamtgraphgröße“ aus dem Anfragepfad; hoher Aufwand, aber der wirkungsvollste strukturelle Eingriff im Graph-Layer.

### 2.2 CSR-Kantenspalten: Speicherverbrauch durch Sentinel-Werte statt `Option`-Tags halbieren
**Problem:** Mehrere Kantenspalten des Graphen sind als `Vec<Option<T>>` geführt, obwohl der zugrunde liegende Typ keine ungenutzten Bitmuster besitzt, die eine Nischenoptimierung erlauben würden. Jedes dieser Felder benötigt dadurch deutlich mehr Speicher, als für den Wertebereich nötig wäre — in Summe rund doppelt so viel Speicher pro Kante wie nötig.
**Maßnahme:** Interne Indextypen auf einen kompakteren Ganzzahltyp umstellen, `Option`-Felder durch dedizierte Sentinel-Werte (z. B. Minimalwert des Zahlbereichs) oder Nischentypen ersetzen, bei identischer fachlicher Semantik.
**Nutzen:** Etwa Halbierung des Speicherbedarfs pro Kante bei großen Graphen; mittlerer Aufwand, da alle Lese-/Schreibstellen der betroffenen Spalten angepasst werden müssen.

### 2.3 Checkpoint-Store: Zwei Indizes unter einem gemeinsamen Lock zusammenführen
**Problem:** Checkpoint-Metadaten werden in zwei separaten Strukturen (Sequenznummer-Index und Namens-Index) mit jeweils eigenem Lock geführt. Aktualisierungen erfolgen über mehrere unabhängige Lock-Erwerbe hintereinander; ein Leser, der über den Namen nachschlägt, kann zwischen den beiden Lookups einen inkonsistenten Zwischenzustand sehen (veraltete oder fehlende Zuordnung).
**Maßnahme:** Beide Indizes in eine gemeinsame, unter einem einzigen Lock geführte Struktur zusammenfassen (z. B. eine Map von Namen auf Metadaten mit abgeleitetem Sequenz-Index), sodass Aktualisierung und Lookup atomar bezüglich beider Zugriffspfade sind.
**Nutzen:** Beseitigt ein Zeitfenster für inkonsistente Checkpoint-Lookups; geringer bis mittlerer Aufwand.

### 2.4 Manifest: Fsync pro Zustandsübergang statt pro Einzeleintrag
**Problem:** Das Manifest führt aktuell für jeden einzelnen Eintrag einen eigenen `fsync` durch, auch wenn mehrere Einträge fachlich zu einem einzigen atomaren Zustandsübergang gehören (z. B. Kompaktierung mit mehreren neuen und mehreren entfernten Dateien). Das erzeugt unnötig viele synchrone Festplattenzugriffe für einen fachlich einzigen Vorgang.
**Maßnahme:** Zusammengehörige Manifest-Änderungen in einem Batch sammeln und mit einem einzigen `fsync` pro Zustandsübergang statt pro Einzeleintrag persistieren.
**Nutzen:** Reduziert die Anzahl synchroner I/O-Operationen bei Kompaktierung und Flush spürbar; mittlerer Aufwand, da die Aufrufstellen entsprechend gebündelt werden müssen.

### 2.5 `memfuse-py` in den Cargo-Workspace aufnehmen
**Problem:** Das Python-FFI-Crate deklariert einen eigenen, separaten Workspace und ist nicht Teil des Root-Workspace. Dadurch wird die PyO3-FFI-Grenze von `cargo build/clippy/test --workspace` nie mitgeprüft, und das Crate nutzt Pfad-Abhängigkeiten statt der geteilten Workspace-Versionen, was einen Versions-Drift bei gemeinsam genutzten Abhängigkeiten strukturell möglich macht.
**Maßnahme:** Crate in die `members`-Liste des Root-`Cargo.toml` aufnehmen; abweichende Profileinstellungen (z. B. Panic-Verhalten für die `cdylib`) gezielt per paketspezifischem Profil erhalten.
**Nutzen:** Stellt sicher, dass die FFI-Grenze denselben CI-Prüfungen unterliegt wie der restliche Workspace; geringer Aufwand.

---

## Stufe 3 — Governance / Prozess

### 3.1 Feature-Kombinationen in CI absichern
**Problem:** Optionale Features (z. B. Edge-Reinforcement-Learning-Pfade und weitere Opt-in-Ausbaustufen) werden im Standard-CI-Lauf nie in Kombination gebaut, sodass nicht sichergestellt ist, dass sie überhaupt kompilieren, geschweige denn korrekt funktionieren.
**Maßnahme:** Einen CI-Schritt ergänzen, der das Powerset der relevanten Feature-Flags baut und mindestens kompiliert (idealerweise inklusive Tests je Kombination).
**Nutzen:** Verhindert stille Bit-Rot in selten aktivierten Codepfaden; geringer Einrichtungsaufwand, laufende CI-Zeitkosten.

### 3.2 Panic-Inventar kontinuierlich pflegen
**Problem:** Auch nach der bereits erfolgten deutlichen Reduzierung der bekannten Panic-Stellen im Produktivcode besteht das Risiko, dass neue `unwrap()`/`expect()`/`panic!()`-Aufrufe unbemerkt in produktiven Code-Pfaden landen, sofern das Gate nicht zwischen Test- und Produktivcode unterscheidet.
**Maßnahme:** Sicherstellen, dass das bestehende Gate ausschließlich Produktivcode unter `src/` (ohne Benchmarks und Tests) zählt und eine harte Obergrenze durchsetzt, die nur sinken, nie steigen darf.
**Nutzen:** Verhindert ein erneutes Anwachsen der bereits reduzierten Panic-Schuld; geringer Aufwand, sofern das Gate bereits die richtige Grundlage hat.

---

## Kurzübersicht nach Aufwand/Nutzen

| Sofort umsetzbar (geringer Aufwand, hoher Nutzen) | Mittelfristig (mittlerer Aufwand) | Struktureller Umbau (hoher Aufwand) |
|---|---|---|
| 0.1 WAL-Replay-Bounds | 1.3 Distanzpfad-Lock/Allokation | 1.1 HNSW-Nachbarformat |
| 0.2 Bandit-Dimensionsprüfung | 1.5 AES-Schlüsselplan wiederverwenden | 1.6 MemTable Range-Sharding |
| 0.3 Drift-Bandit-Kopplung | 1.7 Byte-basierte Cache-Kapazität | 1.9 Text-Posting-Format |
| 0.4 Egress-Kategorisierung | 1.8 build_provenance-Struct | 2.1 Inkrementelle Graph-Kompaktierung |
| 1.2 Backlink-Lookup | 2.2 CSR-Sentinel statt Option | |
| 1.4 SSTable-Zero-Copy-Slice | 2.3 Checkpoint-Index-Merge | |
| 2.5 memfuse-py in Workspace | 2.4 Manifest-Batch-Fsync | |
| 3.1 / 3.2 Governance-Gates | 0.5 Transaktions-Intent-Status | |


## 18. Feature-Flag-Katalog (aus Implementierungsspezifikation)

### 0.3 Cargo-Feature-Katalog (crateübergreifend normativ)

| Feature | Definierender Crate | Default | Wirkung |
|---|---|---|---|
| `docid-128` | `memfuse-core` | aus | `DocId` wird `u128` statt `u64` (§1.3) |
| `block-cache-v2` | `memfuse-store` | aus | `SieveCacheBackend` statt `LruBlockCacheBackend` als aktives Backend (§2.4) |
| `egress-sherman-morrison` | `memfuse-router` | aus | `ShermanMorrisonBandit` statt `DiagonalApproximation` (§10.2) |
| `experimental-diskann` | `memfuse-index` | aus | `DiskAnnIndex` kompiliert und ist über `VectorIndexTier::DiskAnn` wählbar (§5.4) |
| `bandit-routing` | `memfuse-router` | an | Aktiviert den Bandit-Router überhaupt |
| `cloud-egress-guard` | `memfuse-mcp` | an | Aktiviert `egress_gateway`-Modul |
| `wasm-sandbox` | `memfuse-mcp` | an | Aktiviert `sandbox`-Modul |
| `kv-bridge` | `memfuse-candle` | an | Aktiviert `kv_cache_bridge`-Modul |
| `edge-reinforcement-learning` | `memfuse-graph` | aus | Aktiviert `SignalKind::EdgeReinforcement`-Pfad |
| `fault-injection` | `memfuse-store` | nur `dev-dependencies` | Deterministische I/O-Fehlerinjektion für Tests |
| `loom` | `memfuse-store`, `memfuse-graph` | nur `dev-dependencies` | Aktiviert `loom::sync::*` statt `std::sync::*` hinter `#[cfg(loom)]` |



## 19. Fehlertaxonomie

### 19.1 Fehlertaxonomie (crateübergreifend)

| Crate | Fehler-Enum | Einbettet |
|---|---|---|
| `memfuse-core` | `CoreError` | — |
| `memfuse-store` | `StoreError` | `WalError`, `LockError`, `CoreError` |
| `memfuse-crypto` | `CryptoError` | — |
| `memfuse-index` | `IndexError` | `CoreError` |
| `memfuse-graph` | `GraphMutationError`, `GraphError` | `LockError` |
| `memfuse-router` | `BanditError` | — |
| `memfuse-candle` | `KvBridgeError` | `CryptoError` |
| `memfuse-mcp` | `SandboxError`, `EgressError` | `wasmtime::Error` |
| `memfuse-db` | `DbError` | alle Layer-1-Fehler per `#[from]` |

**Regel (verbindlich):** Kein öffentlicher Funktionsrückgabetyp ist `Box<dyn std::error::Error>`. Jeder Crate
exportiert genau einen (oder wenige, klar abgegrenzte) `thiserror`-Fehlertyp(en); `memfuse-db` als oberste
Konsumentenschicht bündelt alle Unterfehler verlustfrei per `#[from]`/`#[error(transparent)]`.

---

<a id="tests"></a>
