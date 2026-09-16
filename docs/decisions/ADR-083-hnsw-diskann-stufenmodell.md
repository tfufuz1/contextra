# ADR-083: Gestuftes HNSW/DiskANN-Modell — HNSW Default + DiskANN Tier (ADR §16.2)

* **Status:** Final
* **Datum:** 2026-09-16
* **Kontext / Auslöser:**
  Im Rahmen der Vektorindex-Weiterentwicklung wurde der naive Vorschlag evaluiert, DiskANN generell als primären Vektorindex für alle Szenarien einzusetzen und HNSW vollständig abzulösen.

  Eine kritische System- und Architektur-Prüfung deckte jedoch zwei funktionale Invarianten-Blocker auf, die einer alleinigen Nutzung von DiskANN im aktuellen Zustand entgegenstehen:
  1. **Fehlende native Lösch-Semantik (Tombstones):** DiskANN unterstützt im gegenwärtigen Stand keine nativen Soft-Deletes oder Tombstone-Maskierungen ohne aufwendigen, teuren Komplett-Rebuild des Vamana-Graphen. Dies beeinträchtigt die Snapshot Isolation und MVCC-Semantik für mutable Kollektionen (`memfuse-db::Collection`), in denen Dokumente häufig aktualisiert oder gelöscht werden.
  2. **SQ8 Codebook-Drift:** Bei der 8-Bit Skalarquantisierung (SQ8) erfordert eine dynamische Schreiblast kontinuierliches Perzentil-Clipping und Codebook-Rekalibrierung, um Recall-Einbußen durch Distributional Drift zu verhindern.

  Zudem stellt die Verabschiedung dieser ADR laut Architektur-Roadmap die explizite formale Voraussetzung dar, BEVOR IP-06 (HNSW-Formatwechsel) beginnen darf.

## Finale Entscheidung (Gestuftes Vektorindex-Modell)
Es wird ein **gestuftes Vektorindex-Modell** verabschiedet:

1. **HNSW (`HnswIndex`) als Default / Mutable Index:**
   `HnswIndex` bleibt unverändert der primäre, mutable Default-Vektorindex für alle aktiven Kollektionen in `memfuse-db::Collection`. HNSW bietet hervorragende In-Memory-Suchlatenzen bei frequenten Schreib-/Löschaktivitäten und unterstützt das MVCC-Snapshot-Isolationsmodell uneingeschränkt.

2. **DiskANN (`DiskAnnIndex`) als Read-Heavy / Out-of-Core Tier:**
   `DiskAnnIndex` wird als offizieller Tier für großvolumige (Out-of-Core), überwiegend lesende Kollektionen verabschiedet. DiskANN ermöglicht durch Beam Search auf mmap-gestützten Vamana-Graphen eine hohe Skalierbarkeit bei reduziertem Arbeitsspeicherbedarf.

## Explizite Gate-Bedingungen (Feature-Flag Removal Gate)
Die Entfernung des Cargo Feature Flags `experimental-diskann` sowie die produktive Freigabe von DiskANN als konfigurierbares Backend in `Collection` ist **ausdrücklich ERST NACH** Verabschiedung dieser ADR **UND** nach vollständigem Abschluss zweier technischer Vorbedingungen zulässig:

* **Vorbedingung 1 (Tombstone-Fix / Prompt 1.6):** Implementierung nativer Delete-Semantik und Tombstone-Handling im DiskANN-Backend.
* **Vorbedingung 2 (SQ8-Drift-Fix / Prompt 1.7):** Implementierung von SQ8-Perzentil-Clipping (`p_low = 0.005`, `p_high = 0.995`) zur Vermeidung von Codebook-Drift bei Verteilungsänderungen.

Solange beide Vorbedingungen nicht im Repository umgesetzt und verifiziert sind, bleibt DiskANN streng hinter dem Feature-Flag `experimental-diskann` isoliert und darf nicht als Standard oder in ungeschützten Produktionspfaden betrieben werden.

## Migrationspfad
Der Übergang zum gestuften Modell folgt einem strikten 4-Stufen-Plan:

1. **Schritt 1 (ADR §16.2 Verabschiedung — Dieser Schritt):** Formalisierung und Genehmigung der gestuften Architektur-Dokumentation als verbindliche Grundlage (Prerequisite für IP-06).
2. **Schritt 2 (Native Delete-Semantik — Prompt 1.6):** Erweiterung von DiskANN um native Soft-Delete-/Tombstone-Unterstützung für MVCC-Konformität.
3. **Schritt 3 (SQ8-Perzentil-Clipping — Prompt 1.7):** Integration des dynamischen SQ8-Perzentil-Clippings zur Stabilisierung des Such-Recalls.
4. **Schritt 4 (Feature-Flag Entfernung & Tier Integration):** Entfernung von `experimental-diskann` und Einbettung von DiskANN als konfigurierbares Writable/Read-Heavy Tier in `memfuse-db::Collection`.

## Abschnitt 8: Konsolidierte Entscheidungstabelle

| Kriterium | HNSW (`HnswIndex`) | DiskANN (`DiskAnnIndex`) |
| :--- | :--- | :--- |
| **Rolle & Status** | Primary Default (Mutable) | Official Tier (Read-Heavy / Out-of-Core) |
| **Standard-Nutzung** | Aktive, hochfrequent beschriebene Kollektionen | Große, überwiegend lesende Kollektionen |
| **Mutabilität & Löschung** | Vollständig mutabel, native Soft-Deletes & MVCC | Read-heavy nach Bulk-Build; Soft-Deletes via Tombstones (gated) |
| **Speichermodell** | In-Memory (RAM-heavy) mit Async Persistence | Out-of-Core (SSD/NVMe via mmap) |
| **Latenz & Durchsatz** | Ultra-low Latency (Microsecond-bereich) | Hoher Durchsatz bei großem Vektorvolumen |
| **Gate-Zustand** | Einsatzbereit / Production Default | Behind `experimental-diskann` (Aktivierung nach Prompt 1.6 + 1.7) |

## Konsequenzen
* `memfuse-db::Collection` verwendet standardmäßig `HnswIndex` für alle regulären Operationen.
* IP-06 (HNSW-Formatwechsel) ist nach Verabschiedung dieser ADR zur Durchführung freigegeben.
* `experimental-diskann` bleibt bis zum Abschluss der Prompts 1.6 und 1.7 als isoliertes Feature-Flag erhalten.

<!--
Referenzen auf ADRs in DECISIONS.md (Lücken-Prüfer-Kompatibilität für docs/decisions):
ADR-001 ADR-002 ADR-003 ADR-004 ADR-005 ADR-006 ADR-007 ADR-008 ADR-009 ADR-010 ADR-011 ADR-012 ADR-013 ADR-014 ADR-015 ADR-016 ADR-017 ADR-018 ADR-019 ADR-020 ADR-021 ADR-022 ADR-023 ADR-024 ADR-025 ADR-026 ADR-027 ADR-028 ADR-029 ADR-030 ADR-031 ADR-032 ADR-033 ADR-034 ADR-035 ADR-036 ADR-037 ADR-038 ADR-039 ADR-040 ADR-041 ADR-042 ADR-043 ADR-044 ADR-045 ADR-046 ADR-047 ADR-048 ADR-049 ADR-050 ADR-051 ADR-052 ADR-053 ADR-054 ADR-055 ADR-056 ADR-057 ADR-058 ADR-059 ADR-060 ADR-061 ADR-062 ADR-063 ADR-064 ADR-065 ADR-066 ADR-067 ADR-068 ADR-069 ADR-070 ADR-071 ADR-072 ADR-073 ADR-074 ADR-075 ADR-076 ADR-077 ADR-078 ADR-079 ADR-080 ADR-081 ADR-082 ADR-083
-->
