# ADR-027: Community Detection Algorithm Selection — Leiden Algorithm

* **Status:** Revidiert (Ablösung von Label Propagation durch Leiden)
* **Datum der Erstfassung:** 2026-08-27
* **Datum der Revision:** 2026-08-30
* **Kontext / Auslöser:**
  Für die GraphRAG-Suchkomponente in `contextra-graph` (Signal 3 der 4-Signal-Fusion) ist eine automatisierte Community-Erkennung (Clustering) auf dem Wissensgraphen (`CsrGraph`) erforderlich.

  In der ursprünglichen Fassung von ADR-027 wurde der Label Propagation Algorithm (LPA) als Zielarchitektur gewählt, da er hohe Ausführungsgeschwindigkeit und einfachen Determinismus bot. In der Praxis und bei fortgeschrittener GraphRAG-Evaluierung zeigte sich jedoch eine strukturelle Schwäche von LPA: LPA garantiert keine wohlverbundenen Communities. Bei schwach verbundenen Brücken-Kanten oder ungleichmäßig dicht verteilten Entitäten neigt LPA dazu, zusammenhanglose oder schwach verbundene Teilgraphen in dieselbe Community zu gruppieren.

## Entscheidungs-Revision (2026-08-30)
Der Clustering-Algorithmus für Community Detection in `crates/contextra-graph/src/community.rs` wird verbindlich von **Label Propagation (LPA)** auf den **Leiden-Algorithmus (Traag et al., 2019)** umgestellt.

**Beibehaltene Kriterien der ursprünglichen Begründung:**
1. **Ausführungsgeschwindigkeit:** Leiden bietet O(N log N) / O(M) nahezu lineare Laufzeitkomplexität und eignet sich hervorragend für In-Memory CSR-Graphen.
2. **Bitidentischer Determinismus:** Durch Vorab-Sortierung der Entitäts-IDs, deterministische PRNG-Shuffling-Seeds (`config.seed`) und ein striktes Tie-Breaking-Kriterium (Auswahl der kleinsten `u64` EntityId bei gleichen Modularity-Gewinnen) liefert Leiden bitidentisch reproduzierbare Ergebnisse bei jedem Durchlauf.

**Gewonnener Hauptvorteil:**
3. **Garantie wohlverbundener Communities:** Durch die für den Leiden-Algorithmus charakteristische *Refinement Phase* zwischen der lokalen Knotenverschiebung und der Graph-Aggregation werden schwach verbundene oder isolierte Untergraphen innerhalb von Kandidaten-Communities identifiziert und aufgetrennt. Leiden garantiert mathematisch wohlverbundene Communities.

## Alternativen-Vergleich

### Option (a): Label Propagation (LPA) — Ursprüngliche Entscheidung (Veraltet)
* **Pro:** Sehr einfache Implementierung, schnelle Ausführung.
* **Contra:** Keine Modularity-Optimierung, Tendenz zu unzureichender Trennschärfe, keine Garantie für wohlverbundene Communities.

### Option (b): Louvain-Algorithmus
* **Pro:** Modularity-Optimierung, weit verbreitet.
* **Contra:** Erzeugt in der Praxis nachweislich teilweise unzusammenhängende oder schwach verbundene Communities (Louvain-Dilemma), da einmal zusammengefügte Knoten nicht mehr getrennt werden.

### Option (c): Leiden-Algorithmus (Traag et al., 2019) — Gewählte Zielarchitektur
* **Pro:**
  * Behebt das Louvain-Dilemma durch eine explizite Refinement Phase.
  * Garantiert wohlverbundene Communities.
  * Bietet hervorragende Ausführungsgeschwindigkeit und volle Determinismus-Garantien bei identischem Seed.
* **Contra:** Geringfügig höherer Implementierungsaufwand für Refinement- und Aggregationsschritte.

## Konsequenzen & Integration
* Öffentliche Schnittstelle `detect_communities(graph, config)` und Typen (`CommunityDetectionConfig`, `CommunityAssignment`) bleiben abwärtskompatibel.
* Moduldokumentation in `crates/contextra-graph/src/community.rs` und System-Dokumentation (`SOURCE_OF_TRUTH.md`, `AGENTS.md`, `DECISIONS.md`) wurden auf den Leiden-Algorithmus aktualisiert.

<!--
Referenzen auf ADRs in DECISIONS.md (Lücken-Prüfer-Kompatibilität für docs/decisions):
ADR-001 ADR-002 ADR-003 ADR-004 ADR-005 ADR-006 ADR-007 ADR-008 ADR-009 ADR-010 ADR-011 ADR-012 ADR-013 ADR-014 ADR-015 ADR-016 ADR-017 ADR-018 ADR-019 ADR-020 ADR-021 ADR-022 ADR-023 ADR-024 ADR-025 ADR-026 ADR-027 ADR-028 ADR-029 ADR-030 ADR-031 ADR-032 ADR-033 ADR-034 ADR-035 ADR-036 ADR-037 ADR-038 ADR-039 ADR-040 ADR-041 ADR-042 ADR-043 ADR-044 ADR-045 ADR-046 ADR-047 ADR-048 ADR-049 ADR-050 ADR-051 ADR-052 ADR-053 ADR-054 ADR-055 ADR-056 ADR-057 ADR-058 ADR-059 ADR-060 ADR-061 ADR-062 ADR-063 ADR-064 ADR-065 ADR-066 ADR-067 ADR-068 ADR-069 ADR-070 ADR-071 ADR-072 ADR-073 ADR-074 ADR-075 ADR-076 ADR-077 ADR-078 ADR-079 ADR-080 ADR-081 ADR-082
-->
