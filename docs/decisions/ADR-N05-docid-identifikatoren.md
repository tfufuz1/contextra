# ADR-N05: Dokumenten-Identifikatoren — Externes DocId(128-Bit/Key) vs. Internes DocIdx(u32)

* **Status:** Proposed / Pending Product-Owner-Entscheidung (gemäß §A2.4 Nr. 3)
* **Datum:** 2026-09-22
* **Kontext / Auslöser:**
  Im MemFuse Cognitive OS besteht eine architektonische Spannung zwischen der externen, benutzerseitigen Dokumenten-Identifikation (`DocId`, 128-Bit BLAKE3-Truncation oder String-Key; siehe ADR-082) und dem internen Kompakt-Index (`DocIdx`, `u32` / 32-Bit In-Memory Slot-Index):

  1. **Externes `DocId` (128-Bit BLAKE3 / String):**
     * Deterministische Hash-Derivierung aus dem Quellschlüssel (`key`).
     * Kryptographisch kollisionsfrei bis in den Billionen-Dokumenten-Bereich ($p(k) \approx 10^{-15}$ bei $10^{12}$ Dokumenten).
     * Bietet ideale Eigenschaften für verteilte Systeme, Re-Derivierung und Zero-Central-Registry-Design.

  2. **Internes `DocIdx` (`u32` Slot-Index):**
     * Kompakter 32-Bit-Ganzzahl-Index (4 Bytes pro Knoten) in In-Memory Vektor-Graphen (`HnswIndex`, `DiskAnnIndex` CSR) und Inverted-Indices (`Bm25Scorer`).
     * Maximale L1/L2-Cache-Effizienz und minimale Speicherbelegung während Vektor-Traversierungen und BM25-Scoring.
     * Limitiert jedoch das System pro Collection auf theoretisch $2^{32}-1$ (~4,29 Milliarden) zeitgleiche In-Memory Slot-Positionen und erfordert zwingend eine interne Bi-Map-Adressübersetzung (`DocId` $\leftrightarrow$ `DocIdx`).

## Offene Product-Owner-Entscheidungsoptionen

*Hinweis: Gemäß §A2.4 Nr. 3 dokumentiert dieses ADR eine offene Strategieentscheidung. Es wird an dieser Stelle KEINE automatisierte Wahl getroffen und KEINE Code-Änderung angestoßen.*

Zur finalen Entscheidung durch den Product Owner (PO) stehen folgende drei Architekturpfade:

### Option A: Beibehaltung der zweistufigen Identifikator-Architektur (Status Quo)
* **Beschreibung:** Externes `DocId` (128-Bit BLAKE3) bleibt die kaskadierende globale ID. Intern verwenden Vektor- und Text-Indizes weiterhin ein kompaktes `DocIdx` (`u32`) mit transparenter Bi-Map-Übersetzung auf Collection-Ebene.
* **Vorteile:** Optimaler RAM-Footprint für Vektor-Graphen (4 Bytes pro Adjazenzzeiger); bewährte In-Memory-Traversierungs-Performanz.
* **Nachteile:** Zusätzlicher Speicher- und Latenz-Overhead für die Bi-Map-Übersetzung (`DocId` $\leftrightarrow$ `DocIdx`); Beschränkung auf $2^{32}-1$ aktive Slots pro Collection.

### Option B: Durchgängige 128-Bit-ID im gesamten Index- und Graph-Layer
* **Beschreibung:** Abschaffung des internen `DocIdx` (`u32`). Vektor-Graphen (`HnswIndex`, `DiskAnnIndex`) und Text-Indizes speichern und verarbeiten direkt die 128-Bit `DocId` (16 Bytes).
* **Vorteile:** Eliminiert die Bi-Map-Übersetzungsschicht vollständig; vereinfacht MVCC- und Snapshot-Logiken; unbegrenzte Skalierbarkeit ohne Slot-Indizierungsgrenzen.
* **Nachteile:** Vervierfachung des Speicherbedarfs für Kantenlisten in Vektor-Graphen (16 Bytes statt 4 Bytes pro Knoten-ID); verminderte CPU-Cache-Effizienz bei Traversierungen.

### Option C: Dynamisch wählbarer/skalierter Slot-Index (Feature-Gated `docidx-u64`)
* **Beschreibung:** Beibehaltung der zweistufigen Trennung, jedoch mit konfigurierbarer/generischer Bitbreite für `DocIdx` (z. B. `u32` als Default für Embedded-Desktop, `u64` per Cargo-Feature / Collection-Schema V2 für Enterprise-Cluster).
* **Vorteile:** Bietet maximale Flexibilität für unterschiedliche Deployment-Szenarien (Embedded vs. Scale-Out Cluster).
* **Nachteile:** Höhere Code-Komplexität durch Generizität über Index-Slots hinweg.

## Konsequenzen

* **Status-Sperre:** Der Status bleibt explizit auf **Proposed / Pending Product-Owner-Entscheidung** gesetzt.
* **Keine Code-Invasivität:** Es erfolgen keine Änderungen an `memfuse-core`, `memfuse-db` oder Index-Crates bis zur formellen Beschlussfassung durch den Product Owner.
