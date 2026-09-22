# ADR-N07: KV-Cache-Stufen A, B und C (Prefix-Reuse, KvState-Modellierung, Verschlüsselte Segmentdateien)

* **Status:** Proposed / In Evaluation (Stufe A beschlossen; Stufen B & C offen, Gesamtspezifikation §A2.4 Nr. 2)
* **Datum:** 2026-09-17
* **Kontext / Auslöser:**
  Für LLM-Inferenz- und RAG-Pipelines stellt die Wiederverwendung von Key-Value-Caches (KV-Cache) einen zentralen Performance-Hebel dar. Die Implementierung eines Caching-Mechanismus birgt jedoch Risiken bezüglich Speicherverbrauch, Upstream-Forking-Wartungsaufwand und kryptographischer Mandantentrennung.

  Gemäß Gesamtspezifikation §9.2, §20.3 und §A2.4 Nr. 2 muss die KV-Cache-Architektur in einem gestuften Modell strukturiert werden, wobei die Ambition (nur Stufe A vs. A+B+C) vom Product Owner nach empirischen Messungen entschieden wird.

---

## 1. Übersicht der KV-Cache Stufenmodellierung

### Stufe A: In-RAM Prefix-Reuse (Fork-Free Default) — *Beschlossen*
* **Konzept:**
  Wiederverwendung von In-RAM KV-Cache-Blöcken auf Basis von Radix-Tree-Prefix-Matching (`memfuse-kvcache`) unter Nutzung der unveränderten Upstream-Abstraktionen (`ModelWeights::clone()`).
* **Eigenschaften:**
  - Kein Upstream-Fork von Candle/Ort-Modell-Backends erforderlich.
  - Zero-Copy In-Memory Prefix-Lookup.
  - Eliminierung redundanter Prompt-Prefill-Phasen bei identischen Prompt-Anfängen.
* **Risiko / Overhead:** Minimal; geringe Komplexität.

### Stufe B: Eigenes Llama-Modell mit direkter `KvState`-Kopplung — *In Evaluation / Offen*
* **Konzept:**
  Tiefe Integration in die Transformer-Inferenzschleife durch Modikation/Forking der Tensor-Generierung, sodass `KvState` direkt aus dem MemFuse-KV-Cache in die Attention-Matrizen injiziert wird.
* **Eigenschaften:**
  - Fein-granulares Token-Level Paging und Swapping.
  - Höhere Speicher-Effizienz bei stark fragmentierten Caches.
* **Risiko / Overhead:** Erhöhte Wartungs- und Sync-Last gegenüber Upstream-LLM-Bibliotheken (Fork-Risiko Nr. 2).

### Stufe C: Verschlüsselte Segmentdateien / Platten-Spill — *In Evaluation / Offen*
* **Konzept:**
  Auslagerung nicht aktiver KV-Cache-Segmente auf sekundäre Speichermedien (NVMe/SSD) in Form verschlüsselter Segmentdateien (`memfuse-security/kv-encryption`), um RAM-Engpässe bei extrem großen Kontextfenstern zu vermeiden.
* **Eigenschaften:**
  - Nahezu unbegrenzte Kontext-Größe bei moderater I/O-Latenz.
  - Strenge Mandatentrennung via AEAD-AES-256-GCM Verschlüsselung pro Tenant und Segment.
* **Risiko / Overhead:** Komplexität bei I/O-Paging, Key-Management und Garbage Collection.

---

## 2. Neutrale PO-Entscheidungsmatrix zur Ambitionsstufe

Gemäß **Gesamtspezifikation §A2.4 Nr. 2** wird die finale Ambitionsstufe (nur Stufe A betreiben vs. Ausbau auf Stufen A+B+C) neutral vorgelegt und erst nach Messung an Stufe A entschieden.

| Kriterium | Nur Stufe A | Stufen A + B + C |
|---|---|---|
| **Upstream-Fork-Risiko** | Keines (nutzt Standard-Interfaces) | Mittel bis Hoch (Fork-Wartungslast) |
| **RAM-Budget-Grenze** | Auf physischen RAM beschränkt | Durch NVMe-Spill dynamisch erweitert |
| **Entwicklungs- & Testaufwand** | Gering | Hoch (AEAD, Swapping, I/O-Teilausfälle) |
| **Nutzen-Verhältnis** | Hoher Initialnutzen für Prompt-Prefixes | Maximaler Nutzen bei extrem langen Kontexten |

---

## 3. Status & Exit-Kriterium

* **Aktueller Status:** Stufe A beschlossen; Stufen B & C `Proposed / In Evaluation`.
* **Exit-Kriterium für Stufen B & C:**
  1. Produktiver Betrieb und Performance-Benchmark von Stufe A unter Reallast.
  2. Nachweis, dass RAM-Limits bei Ziel-Workloads ohne Platten-Spill nicht eingehalten werden können.
  3. Formeller Beschluss des Product Owners zur Freigabe der Stufen B und/oder C.

---

## 4. Konsequenzen

* Derzeitige Arbeiten beschränken sich auf die isolierte Implementierung von Stufe A in `memfuse-kvcache`.
* Code für Stufe B/C bleibt hinter entsprechenden Feature-Flags (`kv-bridge`) isoliert.
