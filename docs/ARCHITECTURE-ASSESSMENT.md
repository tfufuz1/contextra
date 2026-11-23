# Contextra — Principal Architect Assessment
## Storage-Kern R8 · Vollsystem · Strategische Entscheidungen

> *Denkweise: Fertigprodukt rückwärts, wie ein Staff-Engineer bei Cloudflare, Figma oder Neon es begutachtet — auf der Suche nach „brilliant", „clean", „elegant", „fragile", „missing".*

---

## 0. Gesamtbefund (Executive)

**Das R8-Design ist architektonisch eines der durchdachtesten OSS-nahen Storage-Spezifikationen im LLM-Datenbankraum.** Es übersteigt konzeptuell Qdrant und Weaviate in einem kritischen Punkt: *Löschen ist kein Nachgedanke, sondern die primäre Strukturierungsachse.* Die Cluster-als-Löschdomäne-Idee ist eleganter als alles, was in Qdrant, Milvus oder LanceDB existiert.

### Stärken (hält Vergleich mit Best-in-Class)

| Eigenschaft | Bewertung |
|---|---|
| Cluster = Lösch-/Schlüssel-/Versionsdomäne (L1) | **brilliant** — ein Begriff, ein Mechanismus |
| Drei-Stufen-Löschung (Gate → KeyDestroy → Purge) | **exzellent** — DSGVO nachweisbar statt behauptbar |
| Synchroner Kern + Async-Fassade oben (K-12) | **richtig** — `loom`-testbar, deterministisch simulierbar |
| `forbid(unsafe_code)` im Kern | **zwingend** — keine Ausnahme vertretbar |
| `tomb_id` statt Klartext-ClusterId im Ledger (K-03) | **elegant** — Datensparsamkeit by design |
| `IdemToken` + `EraseRequestId` als Orthogonalkonzepte | **feingranular** — Agenten-Retries und Lösch-Idempotenz getrennt |
| Ports als einzige Abstraktion (Fs, Clock, Anchor, KeyProvider) | **clean** — vollständig testbar, keine Runtime-Abhängigkeit |
| `filter_visible` als Pflicht an jeder Extension-Ausgabe | **kritisch richtig** — verhindert die häufigste Klasse von RAG-Datenlecks |
| Konfliktregister (28 Entscheidungen, begründet) | **outstanding** — Architecture Decision Records inline in der Spec |

### Kritische Lücken (muss vor Release)

| Problem | Risiko | Kapitel |
|---|---|---|
| **P7 Branch/Merge nicht begonnen** | Wettbewerbsdifferenzierung fehlt komplett | §4.1 |
| **Kein gemessener Throughput** | Alle Performance-Zahlen sind Hypothesen | §4.2 |
| **`contextra-store` (Alt) + neuer Kern koexistieren** | Schuldenberg wächst täglich | §4.3 |
| **Externe HNSW/BM25 nicht DSGVO-loyal** | Art. 17 Receipt `Incomplete` immer | §4.4 |
| **Keine MCP-Integration im Kern** | Developer Experience-Lücke | §4.5 |

---

## 1. Was ist architektonisch brilliant

### 1.1 Der Cluster als universelle Einheit

```
Cluster = Löschdomäne ∩ Schlüsseldomäne ∩ Versionsdomäne ∩ Platzierungseinheit
```

Das ist die eleganteste Konsequenz in der gesamten Spezifikation. Jedes andere System — Qdrant, Weaviate, Milvus — behandelt diese vier als orthogonale Konzepte, die an Schnittstellen koordiniert werden müssen. Hier ist die Koordination *strukturell unmöglich zu vergessen*, weil es kein anderes Konzept gibt.

**Vergleich:** LanceDB hat Tables mit optionaler Verschlüsselung. Die Löschung eines Datensatzes erfordert einen Rewrite der Vektordatei. Hier ist `Gate` O(1) und unabhängig von der Datenmenge.

### 1.2 Drei-Stufen-Löschung mit Reihenfolge als Vertrag

```
Gate (ms) < KeyDestroy (s) < Purge (min-h)
```

> **L7: Reihenfolge ist Vertrag. Nur Roll-forward.**

Das ist ein inhärenter Sicherheitsbeweis. Es gibt keinen Code-Pfad, der `Purge` vor `Gate` ausführen kann — die Reihenfolge ist strukturell erzwungen, nicht durch Test-Coverage. Das ist der Unterschied zwischen einem System das *verspricht* zu löschen und einem das *beweist* es.

**Receipt-Grammatik** (`Blocked` / `Incomplete` / `CompleteWithResidual`) ist präzise: Kein System behauptet mehr als es getan hat.

### 1.3 Synchroner Kern

Der Entscheid K-12 (`Synchroner Kern, Async-Fassade oben`) ist mutig und richtig. Gründe:

1. **`loom`-Testbarkeit**: Race-Freiheit durch formales Modell statt Hoffnung
2. **TLA+-Gates**: Gate G-0 ist blockiert bis TLA+-Modell besteht — das ist Software-Engineering von Systemen wie FoundationDB
3. **Flat-Combining Committer**: Ein Schreiber mit Gruppen-Fsync ist der WiscKey-Insight — write amplification durch Gruppierung minimieren, ohne komplexe Multi-Writer-Koordination

### 1.4 `forbid(unsafe_code)` im Kern mit `unsafe`-Inseln

```toml
# capabilities.toml
contextra-simd   → unsafe_island = true  # SIMD-Distanzberechnungen
contextra-sys    → unsafe_island = true  # mmap, mlock
contextra-wire   → unsafe_island = true  # FlatBuffers zero-copy
contextra-storage → #![forbid(unsafe_code)]
```

Das ist das exakt richtige Muster. `unsafe` nur wo physisch notwendig (SIMD, syscalls, zero-copy Deserialisierung), Kern ohne es. `cargo geiger` im CI erzwingt das.

### 1.5 `tomb_id` im Ledger

```
tomb_id = HMAC(ledger_key, "ctxs cluster tomb v1" || instance_id[16] || ClusterId[16])
```

Kein Klartext-Identifier im dauerhaften Löschnachweis. Das verhindert Datenminimierungs-Verletzungen im Audit-Log selbst. Gleichzeitig bleibt es ein *linkbares Pseudonym* — korrekt ausgewiesen als solches, nicht als anonym behandelt. Das ist seltene Ehrlichkeit in Spezifikationen.

---

## 2. Was ist elegant gelöst

### 2.1 Zweistufige Guard-Prüfung

```
Aufrufer: fp = SipHash(prozessschlüssel, space||cluster||branch||key)
Committer: Konfliktfenster (Ring 1M fp/stamp-Paare, O(1))
           → Treffer: exakter Langsampfad
```

Kein I/O unter Rang 1 (Committer-Sperre). Hashkollisionen aktivieren nur den Langsampfad, können nie Konflikte verschweigen. Das ist das Guard-Design von Dolt/CRDB, aber für Single-Node optimiert.

### 2.2 Packgruppen + EraseLocality

```
ClusterLocal  → eine Datei je Cluster (höhere FD-Last, O(1)-Unlink)
PackedBounded → geclusterte Sections in gemeinsamen Dateien (IO-effizienter, Purge rewritet)
```

Beide Modi haben klar benannte Tradeoffs. `ClusterLocal` ist das richtige Default für DSGVO-kritische Personendaten, `PackedBounded` für unpersönliche Wissensdaten. Das gibt Betreibern echte Kontrolle.

### 2.3 Scroll-Token als AEAD

```rust
ResumeToken = AEAD(prozessschlüssel, {stamp, cluster, branch, letzter_schlüssel})
```

Nach Cluster-Löschung gibt `ResumeToken` → `Err(Erased)`. Kein Leck durch gecachte Paging-Tokens. Das ist ein Detail das die meisten Systeme vergessen und das in Prod zu Security-Incidents führt.

### 2.4 Memtable partitioniert nach Cluster

```
Gate-Cost: Cluster-Partition aushängen → O(1)
Reclaim:   im urgent-Lane, nicht unter Committer-Sperre
```

Das löst das Scaler-Problem von globalen Skiplists bei Gate. FoundationDB hat Ähnliches, aber komplexer durch Multi-Node-Koordination.

### 2.5 `filter_visible` als Pflicht-API-Vertrag

```rust
// INV-VIS-1: Jedes Extension-Ergebnis MUSS filter_visible passieren
// bevor es den Aufrufer erreicht.
```

Dieser Invariant ist die wichtigste Datenschutz-Garantie im gesamten System. Er stellt sicher dass kein HNSW-Index, kein BM25-Posting, kein Graph-Walk einen gelöschten Datensatz liefern kann — unabhängig davon wie schnell die Extension aufräumt. Das decoupled den Lösch-Zeitpunkt vom Lösch-Beweis.

---

## 3. Was ist feingranular durchdacht

### 3.1 Fehlertaxonomie (9 Klassen, keine Überschneidungen)

```
F-1 transient    → Retry mit Backoff (EINTR)
F-2 Ressource    → kein Poison; DiskFull; Löschung läuft weiter
F-3 E/A-hart     → niemals wiederholen; Poisoned; Recovery per Neustart
F-4 Integrität   → kleinste Einheit vergiften (Block → Abschnitt → Datei)
F-5 Format       → Unsupported; Öffnen verweigert
F-6 Schlüssel    → KeyUnavailable vs KeyLost (Alarm, keine Autokorrektur)
F-7 Aufrufer     → typisiert mit hint()
F-8 Logik        → Test: Abbruch; Prod: Poison + Observer; kein panic
F-9 Rollback     → schreibend verweigert; ReadOnly mit Diagnose
```

`KeyUnavailable` vs. `KeyLost` ist der kritische Unterschied: Ein fehlender Schlüssel ohne Löschbeleg ist *kein Erfolg*, sondern ein Integritätsfehler. Systeme die das nicht unterscheiden (die meisten) erzeugen `Shredded`-Ergebnisse wo Daten eigentlich verloren sind.

### 3.2 Notbetrieb bei ENOSPC

```
Redo/Catalog/Ledger: nur vorallokierte Bereiche (ENOSPC nur beim Vorallokieren)
Gate/KeyDestroy:     Emergency-Reserve-Segmente (unabhängig von User-I/O)
Purge:               eigener Scratch-Pool (min. 2 × max_file_size)
```

Das System kann löschen wenn der Disk voll ist. Das ist für DSGVO-Compliance nicht optional — Art. 17-Fristen gelten auch bei Speicherknappheit.

### 3.3 Clock-Fail-Closed

```
UTC-Sprung > max_clock_skew → Retention/Hold-Übergänge stoppen
                             → ClockUncertain-Alarm
                             → keine vorzeitige Löschung
                             → keine Frist-Compliance vorgespiegelt
```

Die Alternative (most systems) ist stille UTC-Nutzung, die zu falschen Löschzeiten oder ausbleibenden Löschungen führt.

### 3.4 `PurgeBatchEdit` / `PurgeCompleteEdit` Trennung

```
PurgeBatchEdit:    je Datei/Inkarnation → installiert atomar
PurgeCompleteEdit: erst nach ALLEN Segment-Attesten + Extension-Acks
```

Kein `Done` ohne vollständigen Beweis. Kein Stage-Sprung der rückgängig gemacht werden kann.

---

## 4. Kritische Architekturbewertung — Lücken und Tradeoffs

### 4.1 Branch/Merge (P7: Nicht begonnen) — KRITISCH

**Das ist die größte Wettbewerbsdifferenzierung** (K-07: "Git für RAG") und gleichzeitig der größte offene Posten.

**Was fehlt konkret:**
- `SYS_BRANCH`-Space-Implementation
- Branch-Merge mit ChangeIndex-Konfliktprüfung
- TTL-basierte speculative Branch-Expiry
- Branch-Diff (proportional zu Mutationen, nicht zu Datenmenge)

**Bewertung:** Das Design (K-07) ist correct und superior zu Dolt's Prolly-Tree für diesen Usecase. Clusterlokal bedeutet: Löschung eines Clusters löscht alle Branches ohne Sonderfall. Kein DAG-Merge-Problem.

> **Empfehlung: P7 auf P5 vorrücken. Dies ist der USP gegenüber jedem Konkurrenten.**

**Konvergenz mit Archiv-Docs:** Lethe-Final K3 schlägt `branch_id` im Schlüsseltupel + `SYS_BRANCH`-Tabelle vor. Identisch mit R8 §8. Implementierung ist klar und konvergiert.

### 4.2 Keine gemessenen Performance-Zahlen — WICHTIG

Die gesamte Spezifikation enthält keinen einzigen gemessenen Wert. Das Benchmarks-Verzeichnis hat Rahmen (`baseline_metrics.json`, `criterion-baseline.json`), aber keine validierten Zahlen in der SPEC.

**Mindest-Benchmarks die vor Gate G-F existieren müssen:**

| Metrik | Ziel | Invariante |
|---|---|---|
| Write throughput | Durable-Commits/s bei 4KB Payload | — |
| Gate-Latenz p99 | < 10 ms (Gate → Retired) | E-42 |
| Memtable-Gate | O(1) Verifikation | — |
| Purge-Cost | ∝ betroffene Dateien, nicht Store-Größe | INV-ERA-11 |
| Read-Latenz p99 | Punktabfrage Sealed-Cluster | — |
| Recovery-Zeit | Crash bis write-ready (16M-Cluster-Config) | E-38 |

### 4.3 Doppelter Kern (Alt-Store + Neu) — SCHULDEN

`contextra-store` (Ring 1, beta, LSM+WAL) und `contextra-storage` (R8, nicht production-ready) koexistieren.

**Abhängigkeiten des Alt-Stores:**
```
contextra-db        → contextra-store
contextra-agent     → contextra-store
contextra-engine    → contextra-store
contextra-cognition → contextra-store
```

**Risiko:** Jeder Tag Alt-Store in Production ist ein Tag, an dem die DSGVO-Garantien des neuen Kerns nicht gelten. `contextra-core` ist bereits als `deprecated` markiert — das ist der richtige Weg.

> **Empfehlung:** Kein neues Feature in `contextra-store` nach P6. Bridge-Phase (P8 §18) sofort planen.

### 4.4 Extension-Verträge und DSGVO-Lücke — WICHTIG

Externe Indizes (HNSW, BM25, CSR-Graph) haben eigene Datenkopien:

```
E-70: Geteilte HNSW-/BM25-/CSR-Topologie überlebt Cluster-Key-Zerstörung
      → O(Index)-Rewrite bei Cluster-Löschung möglich
```

Die Lösung (`MixedPerClusterKey` + `Rebuildable`) ist korrekt, bedeutet aber:

> **Kein `ErasureReceipt` mit `CompleteWithResidual=false`** so lange externe Indizes nicht attestiert haben.

Für Multi-Tenant-Setups mit 10k Clustern kann das Stunden dauern.

> **Empfehlung:** Cluster-sealed HNSW-Partitionen als Default für `personal = true` Spaces. Der Schlüssel wird vernichtet, kein Rewrite nötig — echtes O(1) Crypto-Erase auch für Vektoren.

### 4.5 Developer Experience (MCP/Ergonomie) — LÜCKE

Das System ist mächtig aber komplex. Für "echten Mehrwert für europäische Unternehmen" braucht es:

| Fehlt | Impact |
|---|---|
| **MCP-Server** als first-class Integration | LLM-Agent-Anbindung in Stunden, nicht Wochen |
| **LLM-lesbare Fehler** (wie Mnemos: `"Branch run-42 existiert nicht. Vorhanden: ..."`) | Agenten-DX drastisch besser |
| **Short Handles** für LLM-Kontext (`d7k2` statt UUID) | LLMs kopieren lange IDs schlecht |
| **Python SDK** (`contextra-py` existiert, Status unklar) | Wichtigste Zielgruppe |
| **Produktdoku aus Nutzerperspektive** | Kein Unternehmen liest die SPEC |

---

## 5. Strategische Kompromiss-Entscheidungen (finale Urteile)

### 5.1 K-07: Clusterlokale Branches OHNE Merkle/Content-Addressing → ✅ RICHTIG

**Argument für Merkle (Engram/Mnemos):** Structural Sharing, O(1) Diff durch Content-Hashing, Dedup.

**Warum R8 recht hat:**
- Dedup = Querverweis zwischen Löschdomänen (`INV-PRIV-1`) → verletzt DSGVO
- Content-Hash über Klartext = Seitenkanal
- Merkle-DAG über Cluster-Grenzen macht `Gate` + `KeyDestroy` unmöglich ohne Rewrite fremder Cluster

**Die Eleganz:** Branch-Diff kostet O(Mutationen seit Fork), nicht O(Daten). `ChangeIndexBlock` speichert alle Mutationen stamp-sortiert. Das ist günstiger als Prolly-Tree für kleine-bis-mittelgroße Branches und explodiert nicht bei vielen Clustern.

### 5.2 K-08: Kein Content-Addressing/Dedup → ✅ RICHTIG (mit Caveat)

**Caveat:** Ohne Dedup wächst der Store bei ähnlichen Dokumenten. Bei Unternehmens-RAG ist das akzeptabel — die Alternative (Dedup mit Cluster-übergreifenden Hashes) bricht die Löschdomänen-Garantie.

**Alternative die funktionieren würde (v2):** Dedup *innerhalb* eines Clusters (Lethe K25: `SYS_DCACHE` per EU). Bewahrt Löschsicherheit, gibt intra-cluster Dedup.

### 5.3 K-12: Synchroner Kern → ✅ RICHTIG

**Argument für Async (Cairn, Mnemos):** Höhere Concurrency, weniger Thread-Overhead.

**Warum synchron recht hat:**
- `loom`-Testbarkeit: Race-Freiheit formal beweisbar
- TLA+-Modelle funktionieren mit synchronem Zustandsraum
- Ein Schreiber + Gruppen-Fsync = einfachstes System das Durability liefert
- Async-Fassade oben gibt trotzdem Concurrency auf API-Ebene

### 5.4 K-04: KeyProvider als Port (nicht Cohort als Kern) → ✅ RICHTIG

Cohort-Strategie (≤1024 Cluster je KEK) ist eine *Provider-Implementation*, kein Kern-Konzept. Der Kern sieht einen einfachen Port:

```rust
fn destroy_many(&self, cs: &[ClusterId]) -> Result<DestroyReceipt, KeyError>;
```

Das hält den Kern einfach und skaliert trotzdem auf Millionen Cluster (Cohort-Provider) oder HSM-Integration (dedizierter Provider je Cluster).

### 5.5 K-18: Ein Committer je Instanz → ✅ RICHTIG (mit expliziter Grenze)

Die Spec benennt das als "ehrlich benannte Grenze": Skalierung durch Tenant-Sharding auf Instanzebene. Correct für v1 on-premise. Für Multi-Tenant Cloud wäre es ein Bottleneck — aber das ist NZ-20 (kein Multi-Node in v1).

### 5.6 K-02: Kern erzeugt ClusterId (128-Bit-Zufall) → ✅ RICHTIG

Keine Wiederverwendung, nicht ableitbar, kein zusätzliches Schlüsselmaterial im Klartext-Pfad. Aufrufer-IDs (wie `eu_tag` aus Lethe-Spec R6) erzeugen unnötige Oberfläche für Timing-Angriffe und Wiederverwendungsfehler.

---

## 6. Empfehlungen: Prioritäten

### Priorität 1 — Kritisch (vor G-F)

```
1. P7 Branch/Merge vorziehen → dies ist der USP
2. Mindest-Benchmark-Suite (Gate-Latenz, Throughput, Recovery-Zeit)
3. Extension-Verträge: cluster-sealed HNSW für personal=true Spaces
4. Format-Freeze (G-F): Referenzvektoren aus Python-Encoder
```

### Priorität 2 — Wichtig (vor G-E)

```
5. Bridge-Plan für contextra-store-Ablösung (Cutover P8)
6. MCP-Server-Integration (Developer Experience, Marktfähigkeit)
7. KeyProvider-Conformance-Suite fertigstellen
8. Ledger-Segmentierung gegen unbegrenztes Wachstum (E-39)
9. Python-SDK-Status klären und stabilisieren
```

### Priorität 3 — Strategisch (v2)

```
10. Intra-Cluster Dedup (SYS_DCACHE Pattern aus Lethe)
11. Cohort-KEK-Provider (Millionen Cluster, 2 KMS-Calls pro Löschung)
12. Multi-Node-Protokoll (separates Gate, eigenes Fencing, NZ-20)
13. Kompression (v1 reserviert: NZ-10)
```

---

## 7. Systemarchitektur

```
┌─────────────────────────────────────────────────────────────────┐
│  API / Fassade (Async-Fassade, tokio)                           │
│  MCP-Server · REST/gRPC · Python SDK · contextra-py             │
├─────────────────────────────────────────────────────────────────┤
│  contextra-storage R8 — Synchroner Kern (#![forbid(unsafe_code)])│
│                                                                 │
│  ┌──────────────┐  ┌─────────────────────────────────────────┐  │
│  │ Engine       │  │ Erase-Kern                              │  │
│  │ Committer    │  │ Gate (ER1, O(1))                        │  │
│  │ Memtable     │  │ KeyDestroy (ER2, KeyProvider)           │  │
│  │ MVCC/Version │  │ Purge (ER5, einziger Löschalgorithmus)  │  │
│  │ Branches     │  │ Hashverkettetes Ledger                  │  │
│  │ IdemToken    │  │ Rollback-Anker                          │  │
│  └──────────────┘  └─────────────────────────────────────────┘  │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Store                                                    │   │
│  │ Redo-Log (vorallokiert, 64MiB)                           │   │
│  │ Catalog (einzige Wahrheit über Dateien)                  │   │
│  │ Run-Files (LSM: Pure | Packed | Staging)                 │   │
│  │ Blob-Files (WiscKey-Separation ≥1KiB)                    │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
│  Extensions via Verträge (SegmentOwner · filter_visible Pflicht)│
│  HNSW-Index · BM25-Volltext · Graph-Index                       │
├─────────────────────────────────────────────────────────────────┤
│  Ports (injizierbar, testbar)                                   │
│  Fs (MemFs/RealFs/FaultFs) · KeyProvider (Cohort/HSM/File)      │
│  Clock (monoton+UTC, fail-closed) · Anchor · Entropy            │
├─────────────────────────────────────────────────────────────────┤
│  Unsafe-Inseln (kontrolliert, cargo geiger)                     │
│  contextra-simd (SIMD) · contextra-sys (mlock) · wire (FB)      │
└─────────────────────────────────────────────────────────────────┘
```

---

## 8. Vergleichsbewertung gegenüber Alternativen

| Aspekt | Contextra R8 | Qdrant | LanceDB | Weaviate | Milvus | FoundationDB |
|---|---|---|---|---|---|---|
| DSGVO Art. 17 (Gate) | **O(1), nachweisbar** | Soft-Delete+Compaction | Rewrite | Soft-Delete | Soft-Delete | Nein |
| Versionierung | **Git-artig, clusterlokal** | Keine | Keine | Keine | Keine | Ja (global) |
| Multi-Modal Storage | **KV+Vector+Text+Graph, atomar** | Vector only | Vector+Table | Vector+Text | Vector | KV only |
| Crypto-Erasure | **3-Stufen, Ledger, Receipt** | Nein | Nein | Nein | Nein | Nein |
| Synchroner Kern | **Ja (loom-testbar)** | Async (Tokio) | Async | Async (Go) | Async (Go) | Async |
| unsafe-free Kern | **Ja** | Nein | Nein | Nein | Nein | Nein |
| RAG-Token-Budget | **weights[], page()** | Nein | Nein | Nein | Nein | Nein |
| Produktionsreife | **P2/8 (In Arbeit)** | GA | Beta | GA | GA | GA |

> **Lücke:** Contextra R8 ist architektonisch überlegen, aber noch nicht production-ready.  
> Der Vorteil muss vor dem ersten GA-Release in gemessene Garantien übersetzt werden.

---

## 9. Das eine Risiko das alles gefährdet

> **Parallelentwicklung ohne Produktionsbeweis.**

Der neue Kern (R8) ist architecturally superior. Der alte Kern (`contextra-store`) ist in Production. Solange der Bridge-Cut (P8) nicht durchgeführt ist, gibt es:

1. Keine DSGVO-Garantien in Production (der Alt-Kern hat keine nachweisbare Löschung)
2. Doppelten Wartungsaufwand
3. Keine einheitliche Test-Matrix

**Die richtige Reihenfolge:**
```
G-F (Format-Freeze)
  → G-W (Write-Pfad vollständig)
  → G-R (Read-Pfad vollständig)
  → G-E (Erase End-to-End, inkl. Extension-Atteste)
  → Bridge (Alt-Ablösung, contextra-store deprecated)
  → G-P (Production-Gate: Soak, Windows-Spike)
```

> **Kein neues Feature in `contextra-db` / `contextra-agent` / `contextra-engine`
> das auf `contextra-store` basiert. Jede neue Funktion baut auf R8 oder wartet.**

---

## 10. Quellen dieser Analyse

| Dokument | Fundstellen |
|---|---|
| `crates/contextra-storage/SPEC.md` (R8) | §1–§15, Konfliktregister K-01..K-28 |
| `crates/contextra-storage/00-INDEX.md` | Phasengraph P0..P8, Errata E-29..E-73 |
| `crates/contextra-storage/PHASE-STATUS.md` | Aktueller Implementierungsstand |
| `docs/archive/OCTA-SPEC-FINAL.md` | Vier-Ebenen-Modell, 19 normative Zusagen |
| `docs/archive/lethe-final-spec.md` | Konfliktauflösung K1..K30, Schichtenmodell |
| `docs/archive/MNEMOS_CONCEPT.md` | LLM-native Ideen: Token-Budget, Zoom, Handles |
| `docs/archive/Optimal-LLM-Datenbank.md` | Engram-Design: LOD-Cluster, Affinity-Layout |
| `capabilities.toml` | Crate-Ringe, Reifegradstufen, unsafe-Inseln |
| `crates/contextra-storage/crates/contextra-storage/src/engine/committer.rs` | Implementierungsstand Committer |

---

*Erstellt: 2026-11-23 · Autor-Rolle: Principal Rust Architect*
*Methode: Reverse-Engineering vom fertigen Produkt rückwärts (brilliant/sauber/elegant/feingranular)*
