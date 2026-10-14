# Algorithmen und Mechanismen für ein eingebettetes, deterministisches Wissens- und Retrieval-System

**Überarbeitete Fassung.** Ersetzt den Bericht mit 19 Mechanismen und dem Label „Verifiziert“ für alle Quellen. Stand der Prüfung: 14.10.2026.

---

## 0. Was sich gegenüber der Vorlage geändert hat

Die Vorlage hatte 30 Quellen mit dem Vermerk „Verifiziert“. Bei der Prüfung der Stichproben stimmten viele Angaben nicht. Diese Fassung übernimmt nur, was belegt ist oder ausdrücklich als Eigenableitung gekennzeichnet wird.

**Statuslegende (gilt für die gesamte Fassung):**

- **[A]** Abstract selbst über die arXiv-Seite geprüft (Titel, Autoren, Datum, Zahlen).
- **[G]** Gegenprüfung durch den Bericht „Gegenprüfung 20 IDs“ (Datei 9aa7c1a9). Nicht unabhängig reproduziert.
- **[V]** Volltext gelesen. In dieser Fassung ist kein Volltext gelesen worden.
- **[E]** Eigenableitung. Nicht aus der Quelle, sondern aus den Anforderungen abgeleitet. Muss beim Umsetzen erneut geprüft werden.
- **[X]** Nicht geprüft oder keine Quelle gefunden.

**Korrekturen mit Belegen:**

| Punkt | Vorlage | Befund |
|---|---|---|
| DiBud-Zahlen | „95 % nDCG@20 bei 65 % weniger Zugriffen“ | Im Abstract [A]: nach Budget-Kalibrierung 95,05–97,68 % des mittleren nDCG@20, bei 65,92–99,53 % weniger Zugriffen als vollständiges Top-20. Die Vorlage hat die Spanne verkürzt. Die Zahlen selbst stimmen. |
| DiBud-Autor | „Zhang et al.“ | Einzelautor Chunran Zhang, eingereicht 14.09.2026 [A]. |
| Khanna-Arbeit | „Khanna et al. (2025), ICLR 2026“ | Autoren Sanjeev Khanna, Huan Li, Aaron Putterman, Februar 2025 [A]. Die Venue ICLR 2026 ist nicht belegt [X]. |
| Hierarchical BM25 | „Mackenzie et al.“ | Deshpande und Sundararaman, Juli 2026 [A]. Zahlen 4,4 GB, ~300 ms [G]. |
| HyperQuant | „Duda et al.“, Leech-Gitter, 1,5 Bit/Dim | Domb, Sackstein, Solberg, Juni 2026 [A]. Behandelt LLM- und Diffusionsgewichte mit E8/D4/A2/Z-Gittern. Keine ANN-Distanzen. Verworfen (siehe Abschnitt 6). |
| FastFreshDiskANN | „Luo et al.“, Name „FastFreshDiskANN“ | Yu et al., „Topology-Aware Localized Update Strategy for Graph-Based ANN Index“, März 2025 [A]. Der Name in der Vorlage ist nicht belegt. |
| Serial Safety Net | „Y. Hong et al.“, „SSN subsumiert SSI“ | Kitazawa et al., „Extended Serial Safety Net“, November 2025 [A]. Die Subsumtion steht nicht im Abstract. |
| Motif-Clustering | „Italiano et al.“, Komplexität O(k·deg³), Φ ≤ O(√Φopt) | Autoren Italiano, Konstantinidis, Mpanti, Ranjbar, Juli 2025 [G]. Die Komplexität und Konduktanzschranke sind nicht belegt [X]. |
| Mixed-Precision | „Joydeep Chandra“, 4,8 ms auf Pi Zero 2W, Embedding-Vektoren | Die Arbeit betrifft Stimmbiomarker für bipolare Agitation, 23,4 ms, 617 KB [G]. Anwendung auf Embeddings ist eine Fehlanwendung. Verworfen. |
| ML-Compaction | „63x Schreibverstärkung“, „60 % weniger“, R² = 0,985, VLDB | Die Arbeit (Cutura, Prakash, August 2026) betrifft Apache-Iceberg-Lakehouse-Tabellen, nicht LSM-Trees. Die Zahlen sind im Abstract nicht enthalten [A/G]. Verworfen (siehe Abschnitt 5, Mechanismus M8). |
| Duan-SSSP | „Dijkstra obsolet“, „senkt Latenzschranken für PageRank-Updates drastisch“ | Die Schranke O(m log^{2/3} n) stimmt [A]. Der Abstract behauptet keinen praktischen Speedup. Die Vorlage selbst berichtet, dass Dijkstra bei dichten Graphen teils schneller ist. „Obsolet“ ist daher falsch. |
| 584 Jahre bei 1 Mio. TX/s | Überlauf eines 64-Bit-Zählers | 2⁶⁴ / 10⁶ s ≈ 1,8·10¹³ s ≈ 584.000 Jahre. Die Vorlage hat den Faktor 1000 falsch. |
| Konforme Coverage | „garantierte asymptotische Coverage auf Niveau 1 − α“ | Die Quelle (Jun, Ohn) beschreibt einen Leave-One-Out-Mechanismus, nicht ein Quantil über W Residuen. Die Garantie steht nicht im Abstract [G]. |

---

## 1. Zusammenfassung

Das Zielsystem ist ein eingebettetes Single-Node-Speichersystem für autonome Agenten in Rust. Die Anforderungen sind: bit-reproduzierbare Ausgaben, feste Speicher- und I/O-Budgets, Lokalität der Kosten, keine Panics im Produktionspfad, Auditierbarkeit.

Nach der Überarbeitung bleiben **zwölf Mechanismen** im Bericht, dazu ein negatives Ergebnis (G.2) als Warnung. Drei haben für den Kernbefund eine belegte Quelle (B.1, D.1, F.1). Die übrigen brauchen eine Volltextlektüre vor der Umsetzung oder sind zurückgestellt. Die Tabelle zeigt den Stand.

| ID | Mechanismus | Quelle | Status der Quelle | Empfehlung |
|---|---|---|---|---|
| B.1 | Budgetierte exakte RRF-Präfixe (DiBud) | 2609.15143 | [A] | Übernehmen, mit Eigenableitung der Zertifizierung (E) |
| D.1 | Hierarchischer lexikalischer Index (L1 im RAM, L2 auf Datenträger) | 2608.00229 | [A] Autoren, [G] Zahlen | Prüfen, Rangsicherheit geht verloren |
| F.1 | Extended Serial Safety Net | 2511.22956 | [A] | Prüfen (Volltext), Abgleich mit dem bestehenden SSI-Validator nötig |
| A.1 | Deterministische SSSP mit O(m log^{2/3} n) | 2504.17033 | [A] Schranke | Zurückstellen für den Hot Path. Nur für Offline-Diffusion prüfen |
| C.1 | Lokale Graphreparatur bei Löschung | 2503.00402 | [A] Titel, Zahlen | Prüfen (Volltext). Algorithmus in der Vorlage nicht belegt |
| C.2 | Prädikatsmarker an Graphkanten (EMA) | 2606.00734 | [G] | Prüfen (Volltext) |
| G.1 | Retrospektive Online-Conformal-Inference | 2511.04275 | [G] | Prüfen (Volltext) |
| G.2 | Conformal Risk Control unter Verteilungsverschiebung | 2606.29054 | [G] | Nicht übernehmen. Als negatives Ergebnis dokumentieren |
| H.1 | Flow-korrigiertes Thompson Sampling | 2606.23933 | [G] | Prüfen (Volltext) |
| H.2 | Catoni-basierte Change-Point-Erkennung | 2505.20051 | [X] Details | Prüfen. Algorithmus der Vorlage ist eine Übertragung |
| I.1 | LeanRAG-Aggregation | 2508.10391 | [G] | Nur asynchron, LLM im Write-Pfad |
| I.2 | Hypergraph-Sparsifikation mit linearen Sketches | 2502.03313 | [A] Autoren | Prüfen (Volltext) |
| K.1 | Deterministischer Musterscanner (Automat) | keine Quelle | [X] | Als Ingenieurentscheidung, nicht als Forschungsbefund führen |

Verworfen oder aus dem Geltungsbereich genommen: A.2 (Motive, Quelle unspezifiziert), D.2 (Inference-free Sparse, offline-ML), E.1 (LSM-Lernpolicy, falsche Domäne), F.2 (Read Promotion, nur mit bekannten Transaktionsprogrammen), J.1 (BMHT, keine Quelle), L.1 (Mixed-Precision, falsche Domäne), M.1 (HyperQuant, KV-/LLM-Bezug). Begründungen in Abschnitt 6.

---

## 2. Geltungsbereich

**Enthalten:** Graphdiffusion und Pfadsuche, Hybrid-Retrieval-Fusion, ANN-Indizes mit Löschung, lexikalische Indizes, Transaktionen und Isolation, Kalibrierung und Drift, Online-Adaption, Sketches für Graphen, deterministische Mustererkennung, deterministische Numerik.

**Ausgeschlossen:** KV-Cache-Verwaltung und Token-Kompression in LLM-Inferenz. Verteilte Systeme und Multi-Node-Konsens. Verfahren, die im synchronen Pfad einen stochastischen LLM-Aufruf brauchen.

**Nicht gesichtet:** Die Vorlage nannte weitere Arbeiten (Entity Resolution, DIAL-KG, H-Mem, Agentic Inference Constraints, Vbox, Evolutionary SSI, Mixed Isolation Read Promotion in weiterer Variante). Diese sind in Anhang B aufgelistet und nicht bewertet.

---

## 3. Systemvorgaben und Fehlerhaltung

Diese Vorgaben gelten für alle Mechanismen. Sie sind Entwurfsregeln, keine Forschungsbefunde.

- **V1 Bit-Reproduzierbarkeit:** Gleiche Eingabe, gleicher Zustand, gleicher Seed erzeugen identische Ausgabe auf allen Zielplattformen.
- **V2 Tie-Breaks:** Jeder Vergleich, der gleich sein kann, hat eine totale Ordnung. Standard ist die numerische Dokument-ID (`u64`).
- **V3 Keine Hash-Reihenfolge als Vertrag:** `std::collections::HashMap` randomisiert den Seed pro Prozess und darf nie die Ausgabereihenfolge bestimmen. `rustc_hash::FxHashMap` ist ohne Seed deterministisch, aber seine Iterationsreihenfolge ist trotzdem kein Vertrag. Ausgaben werden über `BTreeMap` oder über sortierte `Vec` erzeugt.
- **V4 Gleitkomma:** `f32` und `f64` implementieren kein `Ord`. Vergleiche nutzen `total_cmp` oder Festkomma-Ganzzahlen. `partial_cmp().unwrap()` panikt bei NaN und ist verboten. Ein Vergleich über `to_bits()` ist nur für nichtnegative, endliche Werte monoton.
- **V5 Keine Panics im Produktionspfad:** Überläufe, Division durch null und NaN werden als `Result`-Fehler zurückgegeben. Zähler nutzen `checked_add`. `saturating_add` existiert nur für Ganzzahlen, nicht für `f32`.
- **V6 Reduktionsreihenfolge:** Parallele Reduktionen sammeln ihre Teilergebnisse in einem `Vec` und reduzieren in fester Reihenfolge. `rayon`-`reduce` ist nur zulässig, wenn die Reihenfolge festgelegt ist.
- **V7 Budgets vor Ausführung:** Jede Operation hat eine Obergrenze für Zugriffe, Speicher und Zeit, die vor dem Start feststeht.

---

## 4. Mechanismen

Jeder Mechanismus folgt demselben 15-Punkte-Raster. Ein Punkt, der nicht belegt ist, steht als „n.b.“ (nicht belegt) mit Grund.

### B.1 DiBud: budgetierte exakte RRF-Präfixe

1. **Name:** Budgetierte Zertifizierung exakter Präfixe der Reciprocal Rank Fusion.
2. **Quelle:** Chunran Zhang, „Top-K Is Not a Budget for Hybrid Retrieval“, arXiv:2609.15143, eingereicht 14.09.2026, Einzelautor. Status [A].
3. **Problem:** Fester Top-L-Abschnitt vor der Fusion erfüllt nicht die Anforderung an das globale Top-k. Er überlädt I/O oder verwirft relevante Dokumente.
4. **Ein- und Ausgabe:** Eingabe: zwei sortierte Listen (dicht D, spärlich S), Budget B in Zugriffen, Konstante c. Ausgabe: zertifiziertes Präfix A, plus eine Markierung, ob das Budget reichte.
5. **Algorithmus [E]:** Siehe Abschnitt 4.1 unten. Die Quelle beschreibt das Verfahren, die Zertifizierungsbedingung im Detail ist hier eine Eigenableitung.
6. **Invarianten [E]:** Für jedes Dokument d ist L(d) eine untere Schranke und U(d) eine obere Schranke seines wahren RRF-Werts. Ein Dokument wird nur ausgegeben, wenn kein anderes Dokument mehr übertreffen kann.
7. **Komplexität:** Zeit O(B log B), Speicher O(B), unabhängig von der Indexgröße (laut Vorlage, n.b. in der Quelle prüfen).
8. **Determinismus:** Tie-Break über Dokument-ID. Exakte Vergleiche über rationale Werte (siehe 4.1), keine Gleitkommasummen.
9. **Numerik:** Summen von Termen 1/(c+r) sind rational. Für zwei Kanäle genügt `u128` für Zähler und Nenner. Für mehr Kanäle ist die Obergrenze für den Nenner zu prüfen.
10. **Fehlerfälle:** Budget erschöpft vor Zertifizierung: Ausgabe leer oder nur ein Präfix, Flag gesetzt. Leere Listen. Doppelte Dokument-IDs in einer Liste sind ein Eingabefehler.
11. **Evaluation [A]:** Nach Budget-Kalibrierung 95,05–97,68 % des mittleren nDCG@20 bei 65,92–99,53 % weniger Zugriffen gegenüber vollständigem Top-20. Bei Budget 2048 steigt die zertifizierte Ausgabe innerhalb der ersten 100 Positionen um 7,86 % gegenüber balancierter Zugriffsverteilung. Datensatz und Baseline siehe Quelle [X im Detail].
12. **Rust-Umsetzung [E]:** Vorab allokierter Puffer, keine Allokation in der Schleife. Rationale Vergleiche mit `u128`.
13. **Testplan [E]:** Golden-Test gegen Brute-Force-RRF für kleine Listen. Property-Test: die ausgegebene Präfix-Folge ist ein Präfix des exakten RRF-Rankings, bei beliebigem Budget. Determinismustest über zwei Läufe.
14. **Integrationspunkt:** Synchroner Abfragepfad der Hybridsuche.
15. **Risiken:** Ein starker Ausreißer in einer Liste hält die Schranke für alle anderen hoch, bis das Budget erschöpft ist (siehe Abschnitt 8, Frage 3). Die Quelle sagt nichts zu diesem Fall.

#### 4.1 Zertifizierungsbedingung (Eigenableitung [E])

Sei c die RRF-Konstante, r_ch(d) der Rang von d in Kanal ch, und n_ch der nächste ungelesene Rang in ch (Start: 1).

- Gesehene Schranke: L(d) = Σ über Kanäle, in denen d gelesen wurde, von 1/(c + r_ch(d)).
- Kanalschranke: U_ch = 1/(c + n_ch), falls Kanal ch nicht erschöpft ist, sonst 0.
- Obere Schranke für gesehene Dokumente: U(d) = L(d) + Σ über Kanäle, in denen d nicht gelesen wurde, von U_ch.
- Obere Schranke für ungesehene Dokumente: U_ungesehen = Σ über alle Kanäle von U_ch.

Ein Dokument d wird als nächstes Präfix-Element ausgegeben, wenn für alle noch nicht ausgegebenen Dokumente e ≠ d gilt: L(d) > U(e), oder L(d) = U(e) und ID(d) < ID(e). Außerdem muss L(d) ≥ U_ungesehen gelten, wobei bei Gleichheit dieselbe Tie-Break-Regel gilt.

Die Schleife zieht bei jedem Schritt den Kanal mit dem größeren U_ch (Gleichstand: fester Kanal D vor S). Sie bricht ab, wenn das Budget erschöpft ist oder kein Dokument mehr zertifizierbar ist.

Begründung der Präfix-Eigenschaft: Alle Schranken sind gültig, daher kann kein Dokument außerhalb des Präfixes den Wert eines Präfix-Elements erreichen. Die Behauptung „identisch zu B = ∞“ folgt aus dieser Gültigkeit, ist aber nicht aus der Quelle übernommen.

---

### D.1 Hierarchischer lexikalischer Index

1. **Name:** Zweistufiger BM25-Index mit grobem L1 im RAM und L2 auf Datenträger.
2. **Quelle:** Deshpande und Sundararaman, „Hierarchical BM25: Lexical Search at Billion-Document Scale“, Juli 2026. Autoren [A], Zahlen [G].
3. **Problem:** Ein flacher BM25-Index im Milliardenmaßstab braucht viel RAM. Zufälliges Laden von Datenträgerblöcken erzeugt lange Tail-Latenzen.
4. **Ein- und Ausgabe:** Eingabe: tokenisierte Anfrage, Cluster-Budget C. Ausgabe: approximiertes Ranking.
5. **Algorithmus [G]:** Anfrage gegen L1 bewerten, Top-C-Cluster wählen, deren L2-Blöcke lesen, BM25 lokal berechnen. Details zu Scores und Block-Format sind laut Quelle an globale Statistiken gebunden (~100 KB, [G]).
6. **Invarianten:** Die Zahl der Blocklesezugriffe ist durch C beschränkt. Rangsicherheit ist nicht gegeben.
7. **Komplexität:** O(C) Blocklesezugriffe. Residenter Speicher unabhängig von der Dokumentenanzahl, laut Quelle 4,4 GB für 1 Mrd. Dokumente [G].
8. **Determinismus:** Bei festem C und festem Tie-Break reproduzierbar. Summation in fester Clusterreihenfolge.
9. **Numerik:** Score-Summen in fester Reihenfolge. NaN-Handling wie in V4.
10. **Fehlerfälle:** Leere Cluster. Budget C = 0. Dokument in mehreren Clustern.
11. **Evaluation [G]:** ~300 ms, 4,7–5,6× Durchsatz gegenüber Flat bei 1 Mrd. Dokumenten. Recall im Milliardenmaßstab und Vergleich mit BlockMax-WAND sind laut Quelle offen [X].
12. **Rust-Umsetzung [E]:** Memory-Mapping für L2. L1 als zero-copy-Format. Crate-Auswahl (memmap2, rkyv) ist nicht geprüft [X].
13. **Testplan [E]:** Vergleich gegen flachen BM25 auf einem Teilkorpus, Recall@k und Latenz. Determinismustest.
14. **Integrationspunkt:** Asynchroner Lexikal-Kanal im Fusionsmodul.
15. **Risiken:** Verlust der Rangsicherheit. Recall-Verlust hängt von C ab und ist für den eigenen Korpus zu messen.

---

### F.1 Extended Serial Safety Net

1. **Name:** Erweitertes Serial Safety Net (ESSN) zur Serialisierbarkeit über Snapshot Isolation.
2. **Quelle:** Kitazawa et al., „Extended Serial Safety Net“, November 2025. Status [A].
3. **Problem:** Snapshot Isolation erlaubt Write-Skew. Standard-SSI verhindert ihn, erzeugt aber Abbrüche bei langen Transaktionen.
4. **Ein- und Ausgabe:** Read-Set und Write-Set einer Transaktion. Ausgabe: Commit oder Abbruch.
5. **Algorithmus:** Nicht aus dem Abstract ableitbar. Die Vorlage beschrieb eine Schranke über Start- und Endzeitpunkte; diese Beschreibung ist nicht belegt [X]. Die Quelle lockert die Ausschlussbedingung von SSN.
6. **Invarianten [A]:** Erhält multiversion serializability (MVSR). Commit-Aufwand linear in |R| + |W|.
7. **Komplexität [A]:** Linear in Read- und Write-Set.
8. **Determinismus:** Globaler, total geordneter Commit-Zähler. Verhalten muss mit der Quelle abgeglichen werden.
9. **Numerik und Überlauf:** Der 64-Bit-Zähler überläuft nach ~584.000 Jahren bei 10⁶ Commits pro Sekunde. Überlauf wird als Fehler gemeldet (V5). Kein Panic.
10. **Fehlerfälle:** Sehr große Read-Sets (Speicher). Coarsening als Gegenmaßnahme ist eine Eigenableitung [E].
11. **Evaluation [G]:** Abbruchrate für lange Agenten-Transaktionen um ca. 0,25 (≈ 50 % relativ) niedriger als bei SSN, nicht bei SSI. Die Vorlage hat den Bezug falsch angegeben.
12. **Rust-Umsetzung [E]:** `AtomicU64` für den Zähler. Read- und Write-Sets als sortierte Struktur, nicht `HashSet`, um V3 zu erfüllen.
13. **Testplan [E]:** Write-Skew-Szenarien aus der Literatur als Regressionstests. Vergleich mit einer SSI-Referenz.
14. **Integrationspunkt:** Commit-Validator. Ein bestehender SSI-Validator ist im Repository vorhanden (siehe Anhang A). Die Ablösung oder Erweiterung muss gegen ihn geprüft werden.
15. **Risiken:** Ob die Lockerung der Ausschlussbedingung unter den eigenen Transaktionsmustern sicher ist, ist ohne Volltext nicht entschieden.

---

### A.1 Deterministische SSSP (Duan et al.)

1. **Name:** Single-Source-Shortest-Path ohne Sortierbarriere für gerichtete Graphen mit nichtnegativen Gewichten.
2. **Quelle:** Duan, Mao, Mao, Shu, Yin, arXiv:2504.17033, April 2025. Status [A] für Autoren und Schranke.
3. **Problem:** Dijkstra ist durch die Sortierbarriere O(m + n log n) begrenzt.
4. **Ein- und Ausgabe:** Gerichteter Graph, Quelle s, nichtnegative Gewichte. Ausgabe: Distanzen.
5. **Algorithmus:** Im Abstract nicht ausgeführt. Der Pseudocode der Vorlage ist nicht übernommen, weil er undefinierte Begriffe enthält („dynamischer Pivot“, „Block-Listen“). Umsetzung erst nach Volltextlektüre [V nötig].
6. **Invarianten:** Korrektheit nach Quelle [V nötig].
7. **Komplexität [A]:** O(m log^{2/3} n) im Vergleichs-Additions-Modell.
8. **Determinismus:** Die Quelle wird als deterministisch geführt [X]. Gewichtsvergleich über `total_cmp` (V4).
9. **Numerik:** Unendlich als `Option`, nicht als `f32::MAX`. `f32::MAX + w` ergibt `inf`, es gibt keine Sättigung für `f32`.
10. **Fehlerfälle:** Negative oder NaN-Gewichte werden abgewiesen. Unerreichbare Knoten als `None`.
11. **Evaluation [A]:** Der Abstract behauptet keinen praktischen Vorteil. Die Vorlage berichtete selbst, dass Dijkstra bei dichten Graphen teils schneller ist. Eine Gegenstudie (arXiv:2511.03007, nicht geprüft [X]) soll realistische Größen zeigen, in denen Dijkstra schneller ist.
12. **Rust-Umsetzung [E]:** Arena-Allokation für Blöcke. Ob der Aufwand gerechtfertigt ist, ist offen.
13. **Testplan [E]:** Vergleich gegen Dijkstra mit Binär-Heap auf eigenen Graphen, Laufzeit und Speicher.
14. **Integrationspunkt:** Nur Offline-Diffusion, nicht im synchronen Pfad.
15. **Risiken:** Konstantenfaktoren überwiegen bei kleinen Graphen. Ob die Schranke im Zielbereich hilft, ist nicht gezeigt.

---

### C.1 Lokale Graphreparatur bei Löschung

1. **Name:** Tombstone-freie Reparatur eines Graphindex bei Löschung.
2. **Quelle:** Yu et al., „Topology-Aware Localized Update Strategy for Graph-Based ANN Index“, März 2025. Autoren, Titel und Zahlen [A]: 2,47–6,45× Update-Durchsatz gegenüber FreshDiskANN.
3. **Problem:** Tombstones blockieren Pfade und senken den Recall. Vollständige Neuaufbauten sind teuer.
4. **Ein- und Ausgabe:** Gelöschter Knoten, Graph. Ausgabe: aktualisierter lokaler Teilgraph.
5. **Algorithmus:** Die Vorlage beschrieb „Verbinde Vorgänger mit dem ähnlichsten Nachfolger“. Ob das der Quelle entspricht, ist nicht belegt [V nötig]. Die Vorlage sagte „O(1) pro Kante“. Der beschriebene Algorithmus iteriert über alle eingehenden Nachbarn und ist daher nicht O(1).
6. **Invarianten:** Konnektivität und Degree-Grenzen. Der Konnektivitätsbeweis der Vorlage ist nicht belegt [X].
7. **Komplexität:** Pro gelöschtem Knoten O(in-deg · out-deg · Vergleich). Konkrete Schranke nach Volltext.
8. **Determinismus:** Adjazenzlisten nach (Distanz, ID) sortiert halten. Einfügen per Insertion-Sort oder Sortierung nach jeder Änderung.
9. **Numerik:** Distanzen mit `total_cmp`. Leere Ausgangslisten: Fallback auf den Eintrittsknoten.
10. **Fehlerfälle:** Löschung des Eintrittsknotens. Gleichzeitige Suche während der Reparatur.
11. **Evaluation [A]:** Durchsatz-Faktor wie oben. Recall-Verlauf nach Massenlöschung nicht im Abstract [X].
12. **Rust-Umsetzung [E]:** Knotensperren pro Knoten. Die Vorlage nannte `crossbeam::channel` für Freilisten; die Wahl ist nicht geprüft.
13. **Testplan [E]:** Recall-Kurven nach 0, 20 und 50 % Löschung gegen Neuaufbau. Konnektivitätstest. Determinismustest.
14. **Integrationspunkt:** Asynchroner Löschpfad. Im Repository existiert bereits eine Funktion mit diesem Namen (Anhang A).
15. **Risiken:** Gradüberlauf akkumuliert sich. Ein Hintergrund-Sweep ist nötig, aber nicht beschrieben.

---

### C.2 Prädikatsmarker an Graphkanten (EMA)

1. **Name:** Bitvektor-Marker an HNSW-Kanten für prädikatsbewusste Filterung.
2. **Quelle:** Li, Lu, Cheng, Ma et al., „EMA“, arXiv:2606.00734, Mai 2026. Autoren und Zahlen [G]: 1,68–12,25× Speedup.
3. **Problem:** Vorfilterung trennt den Graphen. Nachfilterung iteriert über ungültige Knoten.
4. **Ein- und Ausgabe:** HNSW-Graph mit Attributmarkern. Ausgabe: gefilterte Nachbarn.
5. **Algorithmus [G]:** Jede Kante trägt einen m-Bit-Marker. Kanten werden übersprungen, wenn Marker AND Anfragemaske 0 ist. Die Quelle nennt „konservative“ Marker und einen Mechanismus zur Kantenwiederherstellung [G]; Details [V nötig].
6. **Invarianten:** Keine falschen Negative bezüglich des Marker-Schnitts. Exakte Attributprüfung auf Kandidaten bleibt nötig, weil Marker Falsch-Positive erlauben.
7. **Komplexität:** O(1) pro Kante für den Maskentest. Speicher O(n · M · m) Bit für die Marker.
8. **Determinismus:** Bitoperationen sind deterministisch.
9. **Numerik:** n.b.
10. **Fehlerfälle:** Hohe Selektivität mit vielen Markerkollisionen. Ungleiche Codebook-Granularität.
11. **Evaluation [G]:** 1,68–12,25× Speedup. „Größenordnungen“ und „100 % Recall“ aus der Vorlage sind nicht belegt und gestrichen.
12. **Rust-Umsetzung [E]:** SoA-Layout (Marker getrennt von Vektoren). Popcount über `count_ones()`, das plattformunabhängig ist. `core::arch`-Intrinsics sind nicht nötig.
13. **Testplan [E]:** Recall gegen exakten gefilterten Brute-Force, bei mehreren Selektivitäten. Speicher-Messung.
14. **Integrationspunkt:** Innerste Suchschleife.
15. **Risiken:** Speicherbedarf wächst mit n · M · m. Verhalten bei dynamischen Attributen offen.

---

### D.2 Entfernt

Der inference-free Sparse Retriever (arXiv:2411.04403, [G]: +3,3 NDCG@10 auf BEIR, 1,1× BM25-Latenz) wird nicht übernommen. Er braucht ein Modell im Indexaufbau, und die Konsistenz über Neuindizierung ist nicht geklärt. Die Vorlage nannte die Parameter S = 30 und λ_d, die im Abstract nicht stehen.

---

### G.1 Retrospektive Online-Conformal-Inference

1. **Name:** Online-Konfidenzintervalle mit rückblickender Aktualisierung.
2. **Quelle:** Jun und Ohn, erschienen in *Pattern Recognition* 180 (2026) [G]. Die Vorlage nannte „Jun (2026), Preprint“. Der Status ist also begutachtet, nicht Preprint.
3. **Problem:** Standard-Conformal-Prediction setzt Austauschbarkeit voraus. Drift verletzt sie.
4. **Ein- und Ausgabe:** Strom von Residuen, Zielfehlerrate α. Ausgabe: angepasster Schwellenwert.
5. **Algorithmus [G]:** Leave-One-Out-Aktualisierung. Die Vorlage beschrieb ein Quantil über ein Fenster von W Residuen. Das ist nicht die Quelle [V nötig].
6. **Invarianten:** Asymptotische Abdeckung nach Quelle [V nötig]. Die Garantie aus der Vorlage ist nicht im Abstract belegt.
7. **Komplexität:** n.b. Nach Volltext.
8. **Determinismus:** Reduktionsreihenfolge fest (V6).
9. **Numerik:** α wird in (0,01; 0,99) begrenzt. Eingaben außerhalb sind Fehler (V5), kein Clamp.
10. **Fehlerfälle:** Kurze Schocks werden spät erkannt.
11. **Evaluation [G]:** Intervalle bis ca. 30 % kleiner bei vergleichbarer Abdeckung.
12. **Rust-Umsetzung [E]:** Kein Gleitkomma-Parallelismus.
13. **Testplan [E]:** Abdeckung auf synthetischen Strömen mit bekannter Verschiebung.
14. **Integrationspunkt:** Wrapper für lokale Klassifikatoren.
15. **Risiken:** Wenig Erfahrung mit dem Verfahren außerhalb der Quelle.

---

### G.2 Conformal Risk Control unter Verteilungsverschiebung (negatives Ergebnis)

1. **Name:** Prüfung von Conformal Risk Control für strukturierte LLM-Ausgaben.
2. **Quelle:** Kotte, arXiv:2606.29054, Juni 2026 [G].
3. **Problem:** Garantien brechen unter Verschiebung.
4. **Ein- und Ausgabe:** n.b.
5. **Algorithmus:** n.b. (Die Vorlage nannte keinen Algorithmus.)
6. **Invarianten:** Die Quelle berichtet, dass statisches CRC und die getesteten adaptiven Verfahren unter Verschiebung das Ziel meist verfehlen [G].
7. **Komplexität:** n.b.
8. **Determinismus:** n.b.
9. **Numerik:** n.b.
10. **Fehlerfälle:** Die Quelle nennt Verschiebung als Fehlerfall.
11. **Evaluation [G]:** Negatives Ergebnis.
12. **Rust-Umsetzung:** Nicht vorgesehen.
13. **Testplan:** Kein Test im Bericht. Empfehlung: Verschiebungstest für G.1 übernehmen.
14. **Integrationspunkt:** Keiner.
15. **Risiken:** Wer G.1 einsetzt, muss die Verschiebung messen. Dieses Ergebnis ist eine Warnung für alle Konfidenzkomponenten.

---

### H.1 Flow-korrigiertes Thompson Sampling

1. **Name:** Thompson Sampling mit explizitem Drift-Modell für alte Beobachtungen.
2. **Quelle:** Naghdi und Baheri, arXiv:2606.23933, Juni 2026 [G]. Die Vorlage nannte „Huang et al.“, das ist laut Gegenprüfung falsch.
3. **Problem:** Altwissen wird bei Drift falsch gewichtet.
4. **Ein- und Ausgabe:** Feature-Vektor, Belohnung. Ausgabe: Routing-Entscheidung.
5. **Algorithmus [G]:** Bayes-Update mit Transport alter Beobachtungen über ein explizites Drift-Modell, Closed-Form-Posterior. Die Vorlage nannte „Flow-Distance“ und O(d³), beides nicht im Abstract [X].
6. **Invarianten:** n.b. Die Vorlage nannte keine Regret-Schranke. Die Quelle nach Volltext prüfen.
7. **Komplexität:** Die Vorlage nannte O(d³). Bei inkrementellen Updates ist O(d²) über Sherman-Morrison möglich, wie im bestehenden LinUCB-Design. Konkrete Angabe nach Volltext.
8. **Determinismus:** Thompson Sampling ist stochastisch. Der Seed wird pro Entscheidung aus Sitzung und Zähler abgeleitet und deterministisch durchgereicht. Globale Zufallsfunktionen sind verboten.
9. **Numerik:** Kovarianz über Cholesky mit Diagonalregularisierung εI. Schlägt die Zerlegung fehl, Fallback auf festen Kanal mit deterministischem Seed.
10. **Fehlerfälle:** Kollineare Features. Leere Historie.
11. **Evaluation:** n.b. Die Vorlage „Reduziert Regret signifikant“ ist nicht belegt.
12. **Rust-Umsetzung [E]:** `rand_chacha` für einen deterministischen Generator. Versionen prüfen.
13. **Testplan [E]:** Replay einer Routing-Historie zweimal, Entscheidungen müssen identisch sein.
14. **Integrationspunkt:** Routing im Query-Planer.
15. **Risiken:** Ohne Volltext ist die Drift-Annahme nicht geprüft.

---

### H.2 Catoni-basierte Change-Point-Erkennung

1. **Name:** Erkennung von Verteilungswechseln bei heavy-tailed Daten.
2. **Quelle:** Genalti et al., arXiv:2505.20051, Mai 2025 [G]. Die Vorlage nannte nur „2505.20051 (2025)“. Verwandt und zu prüfen: Huang und Veeravalli, arXiv:2501.10974, Januar 2025 [G], Change Detection mit endlichem Horizont für stückweise stationäre Bandits.
3. **Problem:** CUSUM löst bei Ausreißern häufig Fehlalarme aus.
4. **Ein- und Ausgabe:** Latenz-Zeitreihe. Ausgabe: Wechsel ja/nein.
5. **Algorithmus [E]:** Catoni-Schätzer mit Bisektion, Sliding-Window-Vergleich. Die Quelle selbst ist eine Bandit-Arbeit mit Regret-Schranken [G]; die Übertragung auf Latenzzeitreihen ist eine Eigenableitung.
6. **Invarianten:** Konzentrationsschranke für heavy-tailed Daten nach Quelle [V nötig].
7. **Komplexität:** O(W · Iterationen) mit fester Iterationsgrenze.
8. **Determinismus:** Bisektion mit festem Intervall und fester Iterationszahl.
9. **Numerik:** Iterationsgrenze verhindert Endlosschleifen. Intervallgrenzen aus den Daten, nicht als Konstante. Nicht konvergierte Iteration ist ein Fehler.
10. **Fehlerfälle:** Zu kleines Fenster.
11. **Evaluation:** n.b.
12. **Rust-Umsetzung [E]:** Einfache Bisektion, keine Bibliothek nötig.
13. **Testplan [E]:** Synthetische Wechsel mit Ausreißern, Fehlalarmrate messen.
14. **Integrationspunkt:** Asynchroner Gesundheitsmonitor.
15. **Risiken:** Die Vorlage nannte „50 Schritte“ als Grenze, das ist eine Annahme, keine Quelle.

---

### I.1 LeanRAG-Aggregation

1. **Name:** Hierarchische Aggregation von Entitäten mit Zusammenfassungsknoten.
2. **Quelle:** Zhang et al., arXiv:2508.10391, August 2025 [G]. Die Vorlage nannte AAAI 2026, ungeprüft [X].
3. **Problem:** Flache Extraktion erzeugt redundante Knoten.
4. **Ein- und Ausgabe:** Entitätsgraph. Ausgabe: mehrstufiger Aggregationsgraph.
5. **Algorithmus [G]:** Clustering der Basisentitäten, Zusammenfassungsknoten, hierarchische Abfrage. Die Zusammenfassungen werden von einem LLM erzeugt.
6. **Invarianten:** n.b.
7. **Komplexität:** Abfrage O(Tiefe · Verzweigungsgrad) laut Vorlage [X].
8. **Determinismus:** Clustering mit festem Seed. Zusammenfassungen sind nicht deterministisch, wenn das LLM variiert. Deshalb nur asynchron, Ergebnis wird mit Hash gespeichert.
9. **Numerik:** n.b.
10. **Fehlerfälle:** LLM-Ausfall im Hintergrund.
11. **Evaluation [G]:** 46 % weniger Retrieval-Redundanz. Die Vorlage sagte „enorm“, das ist eine Bewertung.
12. **Rust-Umsetzung:** Graphstruktur über `petgraph` (Versionsprüfung offen [X]).
13. **Testplan [E]:** Redundanzmessung auf eigenem Korpus.
14. **Integrationspunkt:** Ausschließlich asynchroner Schreibpfad. Der Lesepfad bleibt LLM-frei.
15. **Risiken:** LLM-Kosten im Schreibpfad. Die Vorlage nannte den Hashing-Schritt mit MD5 oder SHA-256. Für Rückverfolgbarkeit reicht SHA-256, MD5 ist ungeeignet, wenn Manipulationsschutz gefragt ist.

---

### I.2 Hypergraph-Sparsifikation mit linearen Sketches

1. **Name:** Lineare Sketches für voll dynamische Hypergraph-Sparsifikation.
2. **Quelle:** Khanna, Li, Putterman, arXiv:2502.03313, Februar 2025 [A] Autoren, Titel und Ansatz. Venue ICLR 2026 [X].
3. **Problem:** Neuberechnung der Sparsifikation braucht globalen Zugriff.
4. **Ein- und Ausgabe:** Strom von Hyperkantenänderungen. Ausgabe: Sparsifikator.
5. **Algorithmus [A paraphrasiert]:** Der Ansatz reduziert die Aufgabe auf wiederholtes Effektiv-Widerstands-Sampling auf gewöhnlichen Graphen, die durch oblivious Vertex-Sampling entstehen. Die Vorlage nannte „L0-Skizzierung“ und „polylog(m)/ε²“; diese Details sind nicht bestätigt [V nötig].
6. **Invarianten:** Spektrale Garantie nach Quelle [V nötig].
7. **Komplexität [A]:** Nahezu lineare Laufzeit für nahezu lineare Sparsifikatoren. Konkrete Schranke nach Volltext.
8. **Determinismus:** Hash-Funktionen mit globalem Seed. Die Quelle erwähnt oblivious Adversary, das ist eine Annahme über den Eingabestrom.
9. **Numerik:** Effektive Widerstände über Lösung linearer Systeme. Konditionszahl prüfen.
10. **Fehlerfälle:** Adaptive Eingaben verletzen die Annahme.
11. **Evaluation:** Die Quelle ist theoretisch. Kein Benchmark nach Abstract [A].
12. **Rust-Umsetzung [E]:** Lineare Algebra aus einer geprüften Bibliothek. Versionsprüfung offen.
13. **Testplan [E]:** Schnittgewichte vor und nach Sparsifikation auf Zufallsgraphen.
14. **Integrationspunkt:** Periodischer Hintergrundjob.
15. **Risiken:** Hohe mathematische Komplexität. Umsetzung nur nach Volltextlektüre.

---

### K.1 Deterministischer Musterscanner

Dieser Mechanismus hat keine Forschungsquelle. Er ist eine Ingenieurentscheidung.

1. **Name:** Linearzeit-Scanner für feste Muster (z. B. E-Mail, IBAN) auf Automatenbasis.
2. **Quelle:** keine [X]. Die Vorlage nannte „Synthese“.
3. **Problem:** Backtracking-Regex kann bei ungünstigen Eingaben sehr lange laufen.
4. **Ein- und Ausgabe:** Byte-Strom. Ausgabe: Treffer mit Position.
5. **Algorithmus [E]:** Automat (DFA) für die Musterfamilie. Ein Durchlauf über die Eingabe.
6. **Invarianten:** Jede Eingabe wird einmal gelesen.
7. **Komplexität:** O(n) in der Eingabelänge. Speicher abhängig vom Automaten. Die Gefahr der Zustandsexplosion bei großen Mustersätzen ist real.
8. **Determinismus:** Automat ist deterministisch.
9. **Numerik:** n.b.
10. **Fehlerfälle:** Zu großer Automat. Muster mit Unicode-Varianten.
11. **Evaluation:** n.b. Eigene Messung nötig.
12. **Rust-Umsetzung [E]:** Crate `regex-automata` bietet DFA-Konstruktion. Die Dokumentation beschreibt lineare Suchzeit. Dieser Punkt ist nicht erneut geprüft [X].
13. **Testplan [E]:** Adversariale Eingaben auf Laufzeit, Zustandszahl messen.
14. **Integrationspunkt:** Ingestionspfad.
15. **Risiken:** Keine Forschungsevidenz für die Muster-Auswahl.

---

## 5. Gestrichene Mechanismen

| ID | Vorlage | Grund |
|---|---|---|
| A.2 | Motif-Clustering mit O(k·deg³) und Konduktanzschranke | Quelle (Italiano et al.) beschreibt ein kern- und BFS-basiertes Verfahren. Komplexität und Schranke nicht belegt [G/X]. Zurückgestellt, bis Volltext vorliegt. |
| D.2 | Inference-free Sparse Retriever | Offline-Modell im Indexaufbau, Parameter nicht im Abstract. Entfernt. |
| E.1 | ML-Adaptive-LSM-Compaction | Quelle betrifft Iceberg-Lakehouse-Tabellen, nicht LSM-Trees. Die Zahlen aus der Vorlage stehen nicht im Abstract. Die Quelle selbst spricht für eine einfache Schwellenregel. Eine solche Regel wird als Eigenentwurf [E] empfohlen: Compaction nur, wenn der erwartete Gewinn über einer festen Schwelle liegt. |
| F.2 | Read Promotion | Setzt bekannte, statische Transaktionsprogramme voraus. Für freie Agenten-Transaktionen nicht anwendbar. Zurückgestellt. |
| J.1 | Bounded Merkle Hash Tree für Löschnachweise | Keine Quelle, nur „Industriekonzept“. Die Vorlage nannte Merkle-Strukturen über SSTables ohne Beleg. Zurückgestellt. Eine Literaturrecherche zu verifizierbarer Löschung ist offen [X]. |
| L.1 | Mixed-Precision Information Bottleneck | Quelle betrifft Stimmdaten [G]. Die 4,8-ms-Angabe steht nicht dort. Entfernt. |
| M.1 | HyperQuant | Quelle betrifft LLM- und Diffusionsgewichte sowie KV-Cache, ausgeschlossen (Abschnitt 2). Entfernt. |

**Weitere verworfene Aussagen der Vorlage:**

- „Dijkstra obsolet“: falsch (siehe Abschnitt 0).
- „Vamana/HNSW Soft-Deletes ruinieren Tail-Latenz“: plausibel, aber nicht belegt. Entfernt als Begründung.
- „Klassisches WAND ist als globales Retrieval verworfen, Latenzspitzen von Sekunden“: nicht belegt. Entfernt.
- „PostgreSQL SSI führt zu massiven False-Aborts“: nicht belegt. Entfernt.
- „CUSUM ist extrem anfällig“: plausibel, nicht belegt. Entfernt.
- „SimHash ohne Seed ist nicht deterministisch“: Stimmt nur in der Form „ohne Seed ist es nicht reproduzierbar“. Mit festem Seed ist es deterministisch. Der Satz ist korrigiert.
- „Dense Linear Predictors für KV-Caches“: KV-Bezug, ausgeschlossen.
- „PQ unter 2 Bit verliert exponentiell an Genauigkeit“: nicht belegt.

---

## 6. Querschnitt

### 6.1 Determinismus

- Ausgabereihenfolge hängt nie von Hash-Iteration ab (V3).
- Alle Vergleiche sind total geordnet (V2, V4).
- Zufall läuft über einen Seed, der aus Sitzung und Zähler abgeleitet wird. Globale Zufallsfunktionen sind verboten.
- Parallele Reduktionen in fester Reihenfolge (V6).

### 6.2 Fehlerbehandlung

- Keine Panics im Produktionspfad (V5). `unwrap` und `expect` sind nur in Tests erlaubt.
- Überläufe sind Ergebnisfehler.
- Budgetüberschreitung liefert einen markierten Teilergebnis-Typ, keine stille Kürzung.

### 6.3 Budgets

Jede Operation hat eine Obergrenze für Zugriffe und Speicher. Anytime-Verhalten: Bei erschöpftem Budget wird das zertifizierte Präfix geliefert, kein Abbruch ohne Ausgabe.

---

## 7. Priorisierung

| ID | Nutzen für Determinismus und Budget | Reife der Quelle | Aufwand | Reihenfolge |
|---|---|---|---|---|
| B.1 | Hoch: exakte Präfixe unter Budget | Preprint, Abstract belegt | Mittel | 1 |
| D.1 | Hoch: RAM-Grenze | Preprint, Zahlen [G] | Mittel | 2 |
| F.1 | Hoch: Serialisierbarkeit | Preprint, Abstract belegt | Hoch | 3 (nach Abgleich mit bestehender Validierung) |
| C.1 | Hoch: Löschung ohne Tombstones | Preprint, Abstract belegt | Hoch | 4 (nach Volltextlektüre) |
| C.2 | Mittel: Filterung | Preprint | Mittel | 5 |
| G.1 | Mittel: Konfidenz | Begutachtet [G] | Niedrig | 6 |
| H.1 | Mittel: Routing | Preprint [G] | Mittel | 7 |
| H.2 | Mittel: Drift | Preprint [G] | Niedrig | 8 |
| I.2 | Niedrig bis mittel: Graphkompression | Preprint, Autoren belegt | Hoch | 9 |
| I.1 | Niedrig: nur asynchron | Preprint [G] | Mittel | 10 |
| A.1 | Niedrig: nur Offline | Preprint, Schranke belegt | Hoch | 11 |
| K.1 | Mittel: Ingestion | Keine Quelle, Technik | Niedrig | parallel |

---

## 8. Implementierungsreihenfolge

**Phase 1: Fundament.** V1 bis V7 als Prüfregeln in Tests verankern. Fehlertyp und Budget-Typ definieren. Determinismustest über zwei Läufe.

**Phase 2: Fusion.** B.1 mit Golden-Tests gegen Brute-Force-RRF. Property-Test für Präfix-Eigenschaft.

**Phase 3: Lexikalisch und Vektor.** D.1 gegen flachen BM25 auf Teilkorpus. C.1 erst nach Volltextlektüre von 2503.00402. C.2 nach Volltextlektüre von 2606.00734.

**Phase 4: Transaktionen.** F.1 erst nach Volltextlektüre und Abgleich mit dem bestehenden SSI-Validator.

**Phase 5: Adaption und Konsolidierung.** G.1, H.1, H.2 mit Verschiebungstests. I.1 nur asynchron. I.2 nach Volltext.

---

## 9. Offene Fragen und nötige Messungen

1. **SSSP-Konstanten (A.1):** Ab welchem Graphgrad und welcher Knotenzahl amortisiert sich der Block-Overhead gegenüber einem binären Heap? Nur durch eigene Messung auf Zielhardware.
2. **Read-Set-Granularität (F.1):** Ab welcher Granularität (Seite, Knoten, Zeile) kippt das Verhältnis zwischen Speicher und Fehlabbruchrate?
3. **Ausreißer in DiBud (B.1):** Ein starker Treffer in einer Liste hält die Schranke für alle anderen hoch. Wie verhält sich das zertifizierte Präfix, wenn das Budget vorher erschöpft ist? Test mit synthetischem Ausreißer.
4. **EMA-Codebook (C.2):** Wie steigt die Falsch-Positiv-Rate bei Attributdrift? Eigene Messung über Zeit.
5. **Verschiebung (G.1, G.2, H.1):** Welche Verschiebungsarten sind im Zielsystem realistisch? Ohne diese Angabe ist keine Abdeckung zu garantieren.
6. **Löschnachweis (J.1):** Welche Form von Nachweis verlangt der Anwendungsfall? Literatur dazu ist offen.

---

## Anhang A: Abgleich mit dem bestehenden Code

Geprüft am Repository-Stand 16764583 (main), in dieser Fassung mit Dateizugriff.

| Mechanismus | Bestand im Repository | Hinweis |
|---|---|---|
| C.1 | `remove_with_graph_repair` in `crates/contextra-vector/src/hnsw/deletion.rs` (Zeilen 46 und 50) | Bereits vorhanden. Die Quelle 2503.00402 sollte gegen diese Implementierung verglichen werden. |
| F.1 | `validate_ssi_gate` in `crates/contextra-engine/src/transaction/db_transaction_commit.rs` (Zeile 15) | Bereits vorhanden. Ablösung oder Erweiterung durch ESSN muss gegen diesen Validator geprüft werden. |
| Pfadsuche (nicht im Bericht) | `find_k_paths` in `crates/contextra-graph/src/path_rag/k_path.rs` (Zeilen 47 und 90) | Bereits vorhanden, nach früherer Prüfung ohne Aufrufer außerhalb der Tests. |
| B.1, G.1, H.1, H.2, I.1, I.2 | nicht in dieser Fassung geprüft | Vorher prüfen, ob Teile bereits existieren. |

---

## Anhang B: Nicht bewertete Arbeiten aus der Vorlage

Diese Arbeiten wurden nicht gelesen oder nicht gegengeprüft. Sie sind nicht Teil der Empfehlungen.

- 2610.12514 (Coral-KV): KV-Cache, ausgeschlossen.
- 2306.14048 (H2O): KV-Cache, ausgeschlossen.
- 2606.01065 (Leyline): KV-Cache, ausgeschlossen.
- 2603.20059 (DIAL-KG): Entitätsextraktion, nicht geprüft.
- 2607.26298, 2607.27435 (Entity Resolution): nicht geprüft.
- 2605.15701 (H-Mem): nicht geprüft.
- 2608.15939 (Agentic Inference Constraints): nicht geprüft.
- 2503.05163 (Vbox, Black-Box-DBMS-SSI): nicht geprüft.
- 2610.00889 (Evolutionäre Optimierung SSI-Zyklen): nicht geprüft.
- 2511.00855 (Allan-Poe, GPU-Graphindex, laut Gegenprüfung [G]): GPU-Ausrichtung passt nicht zum CPU- und Edge-Profil.
- 2507.10570 (Italiano et al., Motif-Clustering, [G]): siehe A.2.

---

## Anhang C: Quellenverzeichnis (geführte Quellen)

| arXiv-ID | Titel (laut Seite bzw. Gegenprüfung) | Autoren | Datum | Status |
|---|---|---|---|---|
| 2609.15143 | Top-K Is Not a Budget for Hybrid Retrieval | Chunran Zhang | 14.09.2026 | [A] |
| 2504.17033 | Deterministisches SSSP, O(m log^{2/3} n) | Duan, Mao, Mao, Shu, Yin | 04.2025 | [A] Schranke |
| 2502.03313 | Near-optimal Linear Sketches and Fully-Dynamic Algorithms for Hypergraph Spectral Sparsification | Khanna, Li, Putterman | 05.02.2025 | [A] |
| 2511.22956 | Extended Serial Safety Net | Kitazawa et al. | 11.2025 | [A] |
| 2503.00402 | Topology-Aware Localized Update Strategy for Graph-Based ANN Index | Yu et al. | 03.2025 | [A] Titel, Zahlen |
| 2608.00229 | Hierarchical BM25: Lexical Search at Billion-Document Scale | Deshpande, Sundararaman | 07.2026 | [A] Autoren, [G] Zahlen |
| 2606.23406 | HyperQuant (LLM-/Diffusionsgewichte) | Domb, Sackstein, Solberg | 06.2026 | [A], verworfen |
| 2605.03039 | Mixed-Precision Information Bottlenecks (Stimmdaten) | Chandra | 05.2026 | [G], verworfen |
| 2608.08639 | Smart Compaction (Iceberg) | Cutura, Prakash | 08.2026 | [A]/[G], verworfen |
| 2606.00734 | EMA | Li, Lu, Cheng, Ma et al. | 05.2026 | [G] |
| 2511.04275 | Retrospective Online Conformal Inference | Jun, Ohn | 2026 (Pattern Recognition 180) | [G] |
| 2606.29054 | Conformal Risk Control unter Verteilungsverschiebung | Kotte | 06.2026 | [G] |
| 2606.23933 | Flow-Corrected Thompson Sampling | Naghdi, Baheri | 06.2026 | [G] |
| 2505.20051 | Catoni-basierte Change-Point-Erkennung | Genalti et al. | 05.2025 | [G] |
| 2501.10974 | Change Detection, endlicher Horizont | Huang, Veeravalli | 01.2025 | [G] |
| 2508.10391 | LeanRAG | Zhang et al. | 08.2025 | [G] |
| 2507.10570 | Motif-Clustering auf Hypergraphen | Italiano, Konstantinidis, Mpanti, Ranjbar | 07.2025 | [G] |
| 2501.18377 | Read Promotion, gemischte Isolation | Vandevoort et al. | 01.2025 | [G] |
| 2411.04403 | Inference-free Sparse Retriever | Geng et al. | 11.2024 | [G] |
| 2511.00855 | GPU-Graphindex, Hybridsuche | Allan-Poe | 11.2025 | [G] |

---

## Anhang D: Stand der Prüfung

- Eigene Abstract-Prüfung: 2609.15143, 2504.17033, 2502.03313, 2503.00402 (Titel/Zahlen über Gegenprüfung), 2608.00229 (Autoren). Außerdem die Repository-Stichproben in Anhang A.
- Gegenprüfung übernommen (nicht selbst reproduziert): 2511.22956, 2606.23406, 2608.08639, 2605.03039, 2606.00734, 2511.04275, 2606.29054, 2606.23933, 2505.20051, 2501.10974, 2508.10391, 2507.10570, 2501.18377, 2411.04403, 2511.00855.
- Nicht geprüft: alle Volltexte. Alle Algorithmen mit Status [V nötig] sind vor der Umsetzung zu lesen.
- Keine Build- oder Testläufe in dieser Fassung.
