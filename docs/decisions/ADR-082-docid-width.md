# ADR-082: DocId-Breite — 64-Bit-Kollisionsrisiko bei Skalierung

* **Status:** Vorgeschlagen
* **Datum:** 2026-09-16
* **Kontext / Auslöser:**
  Gemäß dem unabhängigen Review-Befund in `MEMFUSE_UNABHAENGIGE_REVIEW_2026-09-16.md`, Abschnitt 3.7, erzeugt der Algorithmus in [`crates/memfuse-core/src/types/domain.rs:258–277`](../../crates/memfuse-core/src/types/domain.rs#L258-L277) (`DocId::from_key`) eine 64-Bit-Dokumenten-ID, indem der BLAKE3-Hashwert eines Zeichenketten-Schlüssels (`key`) auf seine ersten 8 Bytes (Little-Endian `u64`) trunkiert wird.

  Das System behandelt Kollisionen in der Orchestrierungsschicht (`memfuse-db::Collection`) bereits vollkommen **fail-safe**: Bei Schreiboperationen (`insert_op` / `update_op`) wird über einen Reverse-Lookup geprüft, ob unter der abgeleiteten `DocId` bereits ein abweichender Originalschlüssel existiert. Falls ja, wird die Operation kontrolliert mit `MemFuseError::Internal` abgelehnt. Dies verhindert stille Datenkorruption oder ungewolltes Überschreiben und steht im vollen Einklang mit der **Zero-Panic-Doktrin**.

  **Mathematische Kollisionswahrscheinlichkeit (Geburtstagsparadoxon):**
  Bei einem Suchraum von $N = 2^{64} \approx 1{,}84 \times 10^{19}$ möglichen Hashwerten berechnet sich die Wahrscheinlichkeit $p(k)$ für mindestens eine Kollision bei $k$ eindeutigen Dokumentenschlüsseln näherungsweise über:
  $$p(k) \approx 1 - e^{-\frac{k^2}{2N}}$$

  Daraus ergeben sich folgende konkrete Schwellenwerte für Kollektionsgrößen:
  * **$k \approx 4{,}29 \times 10^9$ Dokumente ($\approx 2^{32}$ / 4,3 Milliarden):** $p(k) \approx 50\%$ Kollisionswahrscheinlichkeit (klassisches Schwellenwert-Limit).
  * **$k = 600 \times 10^6$ Dokumente (600 Millionen):** $p(k) \approx 1{,}0\%$ Fehlerrate bei Neueinfügungen.
  * **$k = 100 \times 10^6$ Dokumente (100 Millionen):** $p(k) \approx 0{,}27\%$ Kumulative Kollisionswahrscheinlichkeit.
  * **$k = 10 \times 10^6$ Dokumente (10 Millionen):** $p(k) \approx 0{,}0027\%$.

  Mit wachsender Dokumentenanzahl im dreistelligen Millionenbereich beginnen Schreiboperationen somit per Design mit messbarer Wahrscheinlichkeit fehlzuschlagen. Das Problem ist kein Korrektheitsbug, sondern eine mathematisch bedingte Kapazitäts- und Verfügbarkeitsgrenze.

## Entscheidung (Vorschlag zur Entscheidung durch das Team)
Die endgültige Wahl zwischen der Beibehaltung der 64-Bit `DocId` und der Umstellung auf eine 128-Bit ID hat strategische Produkt- und Kompatibilitätsauswirkungen auf bestehende Datenbank-Schemata und alle Workspace-Crates. Die Entscheidung wird ausdrücklich als **Vorschlag zur Abstimmung im Entwicklerteam** eingebracht.

**Vorgeschlagener Beschluss:**
Für die aktuelle Version v0.x wird Option (a) gewählt und die Kapazitätsgrenze dokumentiert. Für das nächste Major-Release (v1.0.0 / v2.0.0) wird Option (b) angestrebt, um MemFuse zukunftssicher für Milliarden-Skalierung im Enterprise-Kognitionsbereich auszulegen.

## Alternativen-Vergleich

### Option (a): Kapazitätsgrenze dokumentieren und 64-Bit `DocId` beibehalten
* **Beschreibung:** `DocId` bleibt ein transparenter Wrapper um `u64` (8 Byte). Die maximale empfohlene Kollektionsgröße wird normativ auf **100 Millionen Dokumente pro Collection** begrenzt und in der Spezifikation dokumentiert.
* **Pro:**
  * Implementierungsaufwand ist nahe Null.
  * Minimale Speicher- und Serialisierungs-Footprints (8 Byte pro DocId in HNSW-Indices, SSTable Data Blocks, Graph-Kanten und IPC-DTOs).
  * Maximale Cache-Effizienz und Befehlssatz-Performance bei Vergleichen (`u64` register-native).
  * Vollständig abwärtskompatibel mit allen bestehenden Datenbeständen, WALs und SSTables.
* **Contra:**
  * Harte mathematische Skalierungsgrenze. Bei Workloads oberhalb hunderter Millionen Dokumente pro Collection erfordert das System zwingend Sharding oder eine spätere Breaking-Change-Migration.

### Option (b): Mittelfristige Umstellung auf 128-Bit-IDs (`u128` / `[u8; 16]`)
* **Beschreibung:** `DocId` übernimmt 16 Bytes (128 Bit) aus dem BLAKE3-Hash-Digest.
* **Pro:**
  * Bei $N = 2^{128} \approx 3{,}4 \times 10^{38}$ sinkt die Kollisionswahrscheinlichkeit selbst bei $k = 10^{12}$ (1 Billion Dokumente) auf ein vernachlässigbares Niveau ($p(k) \approx 10^{-15}$).
  * Beseitigt das Verfügbarkeitsrisiko durch Ablehnungen bei extremer Skalierung dauerhaft.
* **Contra:**
  * **Breaking Change:** Das Binärformat aller Persistenzmedien (SSTables, WAL Layout, Checkpoint Manifests, Graph CSR Repräsentationen) ändert sich inkompatibel.
  * **Multi-Layer DAG-Auswirkung:** Betrifft Typdefinitionen und Serialisierer über nahezu den gesamten Crate-DAG:
    * `memfuse-core`: `DocId`, `EntityId`, DTOs.
    * `memfuse-store`: SSTable Sparse Index Keys, WAL Frame Header, Block Caches.
    * `memfuse-index`: HNSW/DiskANN Node IDs, Quantizer State Mappings.
    * `memfuse-graph`: CSR Row Pointers & Target DocId Array Storage.
    * `memfuse-db`: Reverse Mappings & Query Fusion.
    * `memfuse-mcp`, `memfuse-py`: IPC DTOs, C-FFI / PyO3 Types.
  * Verdoppelt den Speicherbedarf aller DocId-Felder im RAM und auf Disk.

## Grober Migrationspfad für Option (b)
Sollte sich das Team für Option (b) entscheiden, wird folgender sanfter Migrationspfad empfohlen:
1. **Schema-Versionierung:** Anheben der SSTable- und WAL-Manifest-Version auf `Version 2`.
2. **Feature-Flag / Typ-Abstraktion:** Isolation von `DocId` hinter einem konfigurierbaren Repräsentations-Typ oder festen 16-Byte Bytearray-Slices.
3. **Kompatibilitätsfenster & Migrations-Tool:** Bereitstellen eines Offline-Konvertierungstools (`cargo xtask migrate-docid-128`), das V1-Datenbanken liest, die 64-Bit DocIds in 128-Bit DocIds erweitert und die Persistenzstrukturen neu aufbaut.

## Konsequenzen
* Die Dokumentation (`GESAMTSPEZIFIKATION.md` und `ARCHITECTURE.md`) wird um den formalen Kapazitätshinweis (100 Mio. Dokumente/Collection) ergänzt.
* Es wird kein Quellcode im aktuellen Audit-Task verändert; der vorliegende ADR dient als Entscheidungsgrundlage für die Produktleitung.
