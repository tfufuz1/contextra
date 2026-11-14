<img width="3168" height="1344" alt="banner" src="https://github.com/user-attachments/assets/443fa9d1-7b8f-4ffe-aa61-46e85f5a414a" />

<p align="center"><b>CONTEXTRA</b></p>
<p align="center"><i>Das lokale Wissens- und Gedächtnissystem für KI-Assistenten und Agenten</i></p>

## 1. Domäne: Hybrid Retrieval, Informationsextraktion & Suche (`retrieval & search`)

<img width="2752" height="1536" alt="hybrid-retrieval" src="https://github.com/user-attachments/assets/0c24b7b2-7037-42f2-98a0-b439c33c83c7" />

Diese Domäne vereint dichte semantische Vektorsuche, klassische lexikalische Volltextsuche, mehrstufige Graph-Traversierung, automatische Wissenstripel-Extraktion sowie mathematisch kalibrierte Rangfusion zu einer deterministischen Multi-Modal-Retrieval-Pipeline.

### Vektorsuche & Hardware-Beschleunigung (`contextra-vector`, `contextra-simd`)

* **HNSW (Hierarchical Navigable Small World)**:
* **Mehrschichtige Small-World-Hierarchie**: Graph-basierter Index zur approximativen Nächste-Nachbarn-Suche (ANN) mit logarithmischer Suchkomplexität $\mathcal{O}(\log N)$. Jeder eingefügte Vektor $v$ erhält eine zufällige Maximal-Ebene $l = \lfloor -\ln(\text{uniform}(0,1)) \cdot m_L \rfloor$ mit dem Normalisierungsfaktor $m_L = 1 / \ln(M)$.
* **Zweiphasige Traversierung & Diversitäts-Heuristik**: Greedy-Abstieg vom globalen Einstiegspunkt (*Entry Point*) ab Ebene $L_{\max}$ bis Ebene $1$ (mit Beam-Width $ef = 1$) und anschließende Beam-Search auf der Basisebene $0$ (mit konfigurierbarer Suchbreite $ef_{\text{search}}$). Beim Aufbau ($ef_{\text{construction}}$) wählt die **Heuristic Neighbor Selection** (Diversitäts-Pruning) Nachbarn nicht rein nach minimaler Distanz, sondern bevorzugt räumlich diversifizierte Kanten, um die Konnektivität zwischen isolierten Clustern zu sichern.
* **Delete- und Backfill-Algorithmus (`hnsw_delete_and_backfill`)**: Zweiphasiger Lösch- und Reparaturmechanismus ohne kompletten Neuaufbau des Index:
1. *Tombstone-Markierung & Nachbarschafts-Identifikation*: Der Zielknoten $v_{\text{del}}$ wird atomar als gelöscht markiert; alle Knoten $u \in \mathcal{N}_{\text{in}}(v_{\text{del}})$, die auf $v_{\text{del}}$ verweisen, werden auf jeder betroffenen Ebene ermittelt.
2. *Lokales Backfilling & Re-Konnektierung*: Für jeden betroffenen Nachbarn $u$ wird eine lokale Kandidatenmenge aus der 2-Hop-Nachbarschaft von $v_{\text{del}}$ (unter Ausschluss aller Tombstones) gebildet und mittels der Diversitäts-Heuristik neu verknüpft, um Graph-Fragmentierung und Sackgassen dauerhaft zu verhindern. Bei Löschung des globalen Entry Points wird deterministisch der verbleibende Knoten auf der höchsten aktiven Ebene promoviert.




* **SQ8 (Skalare 8-Bit-Quantisierung)**:
* **Dimensionsweise Affine Quantisierung**: Abbildung hochdimensionaler `f32`-Embeddings $x \in \mathbb{R}^D$ auf 8-Bit-Ganzzahlen (`uint8` $[0, 255]$ bzw. `int8` $[-128, 127]$) über kalibrierte dimensionsweise Minima $\min_j$ und Maxima $\max_j$ (mit Perzentil-Clipping gegen statistische Ausreißer):

$$q_j = \text{clip}\left(\text{round}\left(\frac{x_j - \min_j}{\max_j - \min_j} \cdot 255\right), 0, 255\right)$$


* **Asymmetrische Distanzberechnung (ADC)**: Reduziert den Hauptspeicherbedarf um Faktor $4$ bei minimalem Recall-Verlust, indem Query-Vektoren in voller `f32`-Präzision verbleiben und gegen die quantisierten Datenbankvektoren über vorberechnete Skalierungs- und Offset-Vektoren ausgewertet werden.


* **RaBitQ & DiskANN (Out-of-Core & Binäre Quantisierung)**:
* **DiskANN (Vamana-Graph & SSD-Out-of-Core)**: Graphbasierter Out-of-Core-Vektorindex für Vektormengen jenseits der RAM-Kapazität. Hält komprimierte Vektor-Codes im RAM, während Vollpräzisions-Vektoren und Adjazenzlisten in 4-KB-aligned SSD-Blöcken ko-loziert sind. Die Graph-Konstruktion erfolgt über einen Two-Pass-`RobustPrune`-Algorithmus mit Expansionsfaktor $\alpha \ge 1.0$; die Suche nutzt asynchrones I/O-Prefetching entlang eines Beam-Buffers der Breite $W$. Beinhaltet einen **automatischen Kollektions-Fallback** auf In-Memory-HNSW für kleinere Kollektionen oder Umgebungen ohne dedizierte SSD-Partition.
* **RaBitQ (Randomized Binary Quantization)**: Randomisiertes binäres Quantisierungsverfahren (32-fache Speicherreduktion), das normalisierte Vektoren mittels einer zufälligen orthogonalen Rotationsmatrix $P \in \mathbb{R}^{D \times D}$ transformiert und auf 1-Bit-Vorzeichencodes $\bar{x} = \text{sign}(P x) \in \{-1, +1\}^D$ abbildet. Ein unverzerrter geometrischer Distanzschätzer korrigiert die Quantisierungsverzerrung analytisch über das Skalarprodukt $\langle \bar{x}, x_{\text{rot}} \rangle$ und nutzt hardwarebeschleunigte Bit-Operationen (`XOR` + `POPCNT`).


* **Prädikat-augmentierte Vektorsuche (Acorn)**:
* **In-Filtering-Graph-Traversierung**: Wertet boolesche Metadaten-Prädikate $\Phi(v)$ direkt während der Nachbarschaftsexpansion aus, statt erst nach der ANN-Suche zu filtern (verhindert den Recall-Kollaps bei hochselektiven Filtern).
* **Dynamische 2-Hop-Brückenbildung**: Wenn ein direkter Nachbar $u$ das Filterprädikat verletzt ($\neg \Phi(u)$), expandiert der Algorithmus adaptiv dessen 2-Hop-Nachbarn $\mathcal{N}(u)$, um valide Pfadbrücken durch gefilterte Subgraphen aufrechtzuerhalten, und wechselt bei extrem geringer Selektivität kostenbasiert auf exakten Pre-Filtered-Scan.


* **SIMD-Distanzberechnungen (`contextra-simd`)**:
* **Laufzeit-CPU-Dispatch & Register-Kerne**: Automatische Erkennung der Befehlssatzerweiterungen zur Laufzeit mit spezialisierten, 4-fach entrollten Akkumulator-Kernen für **AVX2** (256-Bit, 8 `f32`/Zyklus), **AVX-512** (512-Bit, 16 `f32`/Zyklus inklusive Masked Remainder Handling) und **ARM NEON** (128-Bit, 4 `f32`/Zyklus).
* **Kreuzvalidierte Skalare Fallbacks (`simd-cross-validation`)**: Deterministische skalare Referenzimplementierungen mit kontrollierter Reduktionsbaum-Summation, die in CI- und Self-Check-Läufen bit-identisch (bzw. innerhalb strenger ULP-Schranken) gegen die SIMD-Kerne kreuzvalidiert werden für:
* **Kosinus-Distanz ($\text{Cosine Distance}$)**: $d_{\cos}(u, v) = 1 - \frac{\sum_{i=1}^D u_i v_i}{\max\left(\sqrt{\sum_{i=1}^D u_i^2} \cdot \sqrt{\sum_{i=1}^D v_i^2},\, \epsilon\right)}$,
* **Euklidische Distanz ($L_2$ & quadrierte $L_2^2$)**: $d_{L_2}(u, v) = \sqrt{\sum_{i=1}^D (u_i - v_i)^2}$,
* **Skalarprodukt ($\text{Dot Product}$)**: $d_{\text{dot}}(u, v) = -\sum_{i=1}^D u_i v_i$ (negatives Inner Product zur Vereinheitlichung als Minimierungsmetrik).





---

### Volltextsuche & Linguistische Analyse (`contextra-text`)

* **BM25 Scoring**:
* **Okapi-BM25-Relevanzmodell**: Probabilistische Bewertung von Dokument $D$ bezüglich Query $Q$ auf Basis von Termfrequenz $f(t, D)$, inverser Dokumentfrequenz $\text{IDF}(t)$ und Dokumentlängennormalisierung ($k_1 \approx 1.2$, $b \approx 0.75$):

$$\text{Score}(D, Q) = \sum_{t \in Q} \ln\left(\frac{N - n(t) + 0.5}{n(t) + 0.5} + 1\right) \cdot \frac{f(t, D) \cdot (k_1 + 1)}{f(t, D) + k_1 \cdot \left(1 - b + b \cdot \frac{|D|}{\text{avgdl}}\right)}$$


* **Multi-Field-Gewichtung & IDF-Clamping**: Unterstützt gewichtete Dokumentfelder (Titel, Überschriften, Body, Tags) sowie nicht-negative IDF-Glättung für extrem häufige Korpusterme.


* **Block-Max WAND (Weak AND / BMW)**:
* **Zweistufiges Dynamisches Pruning**: Komprimierte Posting-Listen (Delta-kodierte DocIDs und Termfrequenzen) werden in feste Blöcke (z. B. 128 Postings) partitioniert, für die jeweils der maximale Block-Score $U_{t, B}$ vorberechnet ist.
* **Pivot- & Block-Skip-Auswertung**: Die Term-Cursor werden aufsteigend nach aktueller DocID sortiert. Über die globalen Term-Obergrenzen $U_t$ wird der Pivot-Term $p$ mit $\sum_{i=0}^{p} U_{t_i} > \theta$ (aktueller Top-$k$-Schwellenwert) bestimmt. Bevor Posting-Blöcke dekomprimiert werden, prüft BMW die feingranulare Block-Summe $\sum_{i=0}^{p} U_{t_i, B(d)} > \theta$ und springt mittels Skip-Pointern (`advance_to`) direkt über unkritische Dokumentbereiche hinweg.


* **Deutscher Morphologie- und Normalisierungs-Algorithmus**:
* **Deterministische Umlaut- & Unicode-Normalisierung**: Unicode-NFC-Normalisierung, Case-Folding sowie deterministische Faltung deutscher Umlaute und Ligaturen (`ä` $\to$ `ae`/`a`, `ö` $\to$ `oe`/`o`, `ü` $\to$ `ue`/`u`, `ß` $\to$ `ss`), sodass sowohl Umlaut- als auch ASCII-Schreibweisen identisch gematcht werden.
* **Regelbasierte Kompositazerlegung (Compound Splitting)**: Dynamische Programmierung und regelbasierte Morphem-Segmentierung für zusammengesetzte deutsche Substantive unter Berücksichtigung deutscher Fugenlaute (`-s-`, `-es-`, `-n-`, `-en-`, `-er-`, `-e-`). Sowohl das ungeteilte Kompositum (höhere Präzision) als auch die validierten Teilwörter (höherer Recall) werden mit abgestuften Termgewichten in den Index bzw. die Query eingespeist.
* **Sprachspezifische Stoppwort-Filterung & Stemming**: Entfernung hochfrequenter Funktionswörter anhand eines kuratierten deutschen Stoppwort-Lexikons sowie deterministische Suffix-Reduktion.


* **Inverted Index mit CoW-Snapshots**:
* **Copy-on-Write-(CoW-)Snapshot-Isolation**: Strukturierte Verwaltung unveränderlicher Segment-Posting-Listen und FST-/Hash-Term-Lexika über atomare Referenzzähler (`Arc`). Lese-Queries operieren vollständig lock-frei auf einem konsistenten Point-in-Time-Snapshot, während Schreib- und Merge-Operationen neue Segmente im Hintergrund erzeugen und atomar per Pointer-Swap einhängen.



---

### Graph-Suche, Relationen & GraphRAG (`contextra-graph`, `contextra-db`)

* **Personalized PageRank (Forward-Push PPR)**:
* **Lokaler Approximationsalgorithmus nach Andersen-Chung-Lang**: Zielgerichtete Graph-Traversierung ausgehend von einer Seed-Knotenmenge $S$. Verwaltet für jeden besuchten Knoten $u$ die akkumulierte Wahrscheinlichkeitsmasse $p(u)$ und das Residuum $r(u)$. Solange ein Knoten existiert, für den $\frac{r(u)}{d_{\text{out}}(u)} > \varepsilon$ gilt, wird $(1-\alpha) r(u)$ auf $p(u)$ übertragen und die verbleibende Masse $\alpha \cdot r(u)$ entlang der ausgehenden Kanten auf Nachbarn verteilt ($\alpha \in (0,1)$ Dämpfungsfaktor). Garantiert eine Approximationsschranke in $\mathcal{O}\left(\frac{1}{\varepsilon \alpha}\right)$ Zeit unabhängig von der Gesamtgröße des Graphen.


* **CSR (Compressed Sparse Row)**:
* **Cache-lokale Adjazenzrepräsentation**: Kompakte Speicherung dünnbesetzter Graphstrukturen über drei kontiguierliche Arrays: `row_ptr` ($\in \mathbb{N}^{|V|+1}$ als Start-Offsets pro Knoten), `col_idx` ($\in \mathbb{N}^{|E|}$ für Zielknoten-IDs) und `values` ($\in \mathbb{R}^{|E|}$ für Kantengewichte). Ergänzt durch gespiegelte CSC-Strukturen (Compressed Sparse Column) für rückwärtsgerichtete Traversierungen und einen Delta-Overlay-Buffer für lock-freie Echtzeit-Mutationen.


* **K-Path Diffusion & APPRH**:
* **K-Path Diffusion**: Tiefenbegrenzte, abklingende Pfad-Ausbreitung bis zur maximalen Hop-Tiefe $K$ mit pfadlängenabhängiger Dämpfung $\gamma^k$, Grad-Normalisierung ($d(v)^{-\beta}$ zur Dämpfung uninformativer Super-Hubs) und Zyklenerkennung.
* **APPRH (*Adaptive Personalized PageRank with Hyperedge Expansion*)**: Verallgemeinerung des Forward-Push-PPR auf Hypergraphen. Propagiert Wahrscheinlichkeitsmasse abwechselnd über Knoten und mehrstellige Hyperkanten (bipartite Inzidenz-Expansion) und skaliert den Fluss invers zur Kardinalität $|e|$ sowie proportional zum Hyperkantengewicht.


* **N-äre Relationen & Zettelkasten-Links (`relate`, `links`)**:
* **N-äre Relationen (`relate`)**: Thread-sichere Modellierung und Abfrage mehrstelliger Hyperkanten $R(e_1:\text{Rolle}_1, e_2:\text{Rolle}_2, \dots, e_n:\text{Rolle}_n)$ als erstklassige Graph-Entitäten mit Rollen-Indizes und zeitlicher Gültigkeit.
* **Zettelkasten-Links (`links`)**: Typisierte, bidirektionale Verknüpfungen (`SUPPORTS`, `CONTRADICTS`, `ELABORATES`, `DERIVED_FROM`, `SUPERSEDES`) inklusive automatischer Rückwärtskanten-Synchronisation und **kaskadierender Tombstone-Invalidierung** bei `SUPERSEDES`-Beziehungen: Wird Dokument/Fakt $A$ durch $B$ ersetzt ($B \xrightarrow{\text{SUPERSEDES}} A$), werden $A$ und ausschließlich von $A$ abgeleitete Wissensknoten transitiv als veraltet markiert, ohne die Revisions-Provenance zu zerstören.


* **GraphRAG Community Detection & Edge-Reinforcement**:
* **Hierarchische Community Detection**: Modularitätsbasierte Cluster-Erkennung (Leiden-/Louvain-Verfahren) zur Partitionierung des Wissensgraphen in thematische Communities auf mehreren Abstraktionsebenen.
* **Perkolationsschwellen-Detektion**: Kontinuierliche Analyse der Riesenkomponente (*Giant Connected Component*) und des Molloy-Reed-Kriteriums $\kappa = \frac{\langle k^2 \rangle}{\langle k \rangle} > 2$ zur Früherkennung kritischer Konnektivitätsübergänge und semantischer Über-Vernetzung.
* **Dynamisches Kanten-Reinforcement-Learning**: Aktivitätsabhängige Verstärkung ($w_{uv} \leftarrow w_{uv} + \eta \cdot r$) häufig ko-aktivierter und in Antworten bestätigter Wissenspfade, gekoppelt an zeitliches Abklingen ungenutzter Kanten.



---

### Automatische Informationsextraktion (`contextra-db::extraction`)

* **OpenIE-Tripel-Extraktion (`open_ie`)**:
* **Deterministische Tripel-Zerlegung**: Syntaktische und musterbasierte Extraktion strukturierter `(Subjekt, Prädikat, Objekt)`-Wissenstripel aus unstrukturierten Text-Chunks inklusive Auflösung von Konjunktionen, Appositionen und Nominalphrasen.
* **Lückenlose Provenance-Verfolgung**: Jedes extrahierte Tripel speichert deterministische Rückverweise auf Quell-Dokument-ID, Chunk-ID, exakte Zeichen-Offsets und Extraktions-Konfidenz.


* **Regelbasierte Entitäts- & Relationsextraktion (`rule_based`)**:
* **Ingest-gekoppelte Mustererkennung (`auto_extraction`)**: Deterministische Extraktion benannter Entitäten (Organisationen, Personen, Normen, Zeitangaben, Kennungen) und typisierter Relationen während des Dokumenten-Ingests über kombinierte Regex-, Gazetteer- und Kontext-Regeln, steuerbar über granulare Laufzeitmodi (`Off`, `RulesOnly`, `OpenIeOnly`, `Full`).



---

### Multi-Step Retrieval, Query-Orchestrierung & Filterung (`contextra-db`)

* **Multi-Step Retrieval & Query Rewriting (`multistep`, `multistep_pid`)**:
* **Iterativer Abruf- und Reformulierungs-Zyklus**: Mehrstufige Retrieval-Pipeline, die in Schritt $t$ extrahierte Schlüsselentitäten und Graph-Nachbarn zur automatischen Query-Expansion und -Dekomposition für Schritt $t+1$ nutzt.
* **PID-gesteuerte Tiefenregelung (`multistep_pid`)**: Ein integrierter PID-Regler überwacht den marginalen Informationsgewinn (*Marginal Novelty* neuer Treffer gegenüber bereits abgerufenen Dokumenten) und regelt Suchtiefe, Hop-Radius und Kandidatenanzahl dynamisch nach oder löst bei Informationssättigung ein deterministisches Early Stopping aus.


* **Typsicherer Query-Builder (`query_builder`)**:
* **Modulare Abfragekonstruktion**: Compile-Zeit-typsicheres Builder-Interface mit:
* **Such-Scopes (`scope`)**: Eingrenzung auf Mandanten, Kollektionen, Namespaces oder spezifische Dokument-Subsets,
* **Ausführungsstrategien (`strategy`)**: Steuerung zwischen Single-Modal, paralleler Multi-Modal-Hybrid-Suche und iterativem Multi-Step-Retrieval,
* **Signal-Gewichtungen (`weights`)**: Feingranulare Gewichtung von Vektor-, Volltext- und Graph-Signalen,
* **Adaptivem Overfetching (`hybrid_adaptive_overfetch`)**: Dynamische Skalierung der internen Kandidaten-Limits $k_{\text{fetch}} = \lceil k \cdot \gamma(\text{filter\_selectivity}, \text{channel\_overlap}) \rceil$, um Recall-Verluste durch Post-Filtering und disjunkte Kanal-Treffer auszugleichen,
* **Ergebnis-Hydratisierung (`hydrate`)**: Verzögertes, gebündeltes Nachladen schwergewichtiger Dokument-Payloads, Metadaten und Provenance-Kanten erst nach Abschluss der Rangfusion für die finalen Top-$k$-Treffer,
* **Checkpoint-gebundener Suche (`search::checkpoint`)**: Deterministische Abfragen gegen unveränderliche historische Transaktions-Checkpoints (Point-in-Time-Search),
* **Recall-erhaltenden Truncation-Filtern**: Token-budgetbewusstes Kürzen von Kontext-Chunks auf Basis der höchsten Term-/Span-Dichte ohne Verlust relevanter Evidenzstellen,
* **Definierter Fehlersemantik bei Teilausfällen (`hybrid_signal_failure_semantics`)**: Konfigurierbare Degradations-Policies (`FailFast`, `BestEffortDegrade`, `RequirePrimarySignal`) zur deterministischen Behandlung von Timeouts oder Backend-Fehlern einzelner Suchkanäle.




* **Metadaten- & Temporale Filterung (`filter`, `temporal_filter`)**:
* **Metadaten-Prädikat-AST (`filter`)**: Kompilierbarer Filterbaum (`AND`, `OR`, `NOT`, `EQ`, `NEQ`, `LT`, `LTE`, `GT`, `GTE`, `IN`, `BETWEEN`, `PREFIX`, `CONTAINS`, `EXISTS`) mit Pushdown in Invertierte Metadaten-Indizes und Acorn-In-Filtering.
* **Temporale Filterung (`temporal_filter`)**: Bitemporale Gültigkeitsauswertung über fachliche Gültigkeitsintervalle (`valid_from`, `valid_to`) und Systemzeitstempel (`transaction_time`), inklusive Point-in-Time-Schnitten, Intervall-Überlappungs-Logik und zeitlicher Recency-Gewichtung.



---

### Rangfusion & Score-Kalibrierung (`contextra-rank`, `contextra-db::fusion`)

* **Weighted RRF (Weighted Reciprocal Rank Fusion)**:
* **Skaleninvariante Multi-Signal-Fusion**: Kombination unkalibrierter Ranglisten aus Vektor-, Volltext- und Graph-Kanälen mit Kanalgewichten $w_i \ge 0$ und Glättungskonstante $k = 60$:

$$\text{RRF}(d) = \sum_{i \in \mathcal{C}} \frac{w_i}{k + \text{rank}_i(d)}$$




* **mRRF (Margin-modulierte RRF)**:
* **Score-Abstands-sensitive Fusion**: Erweitert klassisches RRF um einen normalisierten Margin-Term $\Delta s_i(d)$, der den Score-Vorsprung eines Treffers gegenüber dem nachfolgenden Rangplatz bzw. dem Kandidaten-Median misst, um hochkonfidente Spitzentreffer deutlich von flachen Score-Plateaus abzuheben.


* **Resonanz-Bonus**:
* **Multi-Kanal-Koinzidenz-Verstärkung**: Gewährt einen nicht-linearen Zusatzbonus, wenn dasselbe Dokument gleichzeitig in mehreren orthogonalen Retrieval-Modalitäten (z. B. semantischer Vektorraum, exakter BM25-Lexikalindex und Graph-Nachbarschaft) unter den Top-Kandidaten erscheint.


* **Isotonic Regression (PAVA)**:
* **Nicht-parametrische Wahrscheinlichkeitskalibrierung**: Verwendet den **Pool-Adjacent Violators Algorithm (PAVA)** in $\mathcal{O}(n)$ Zeit, um eine monoton steigende Treppenfunktion zu schätzen, die rohe Relevanz-Scores ohne starre Verteilungsannahmen auf kalibrierte Wahrscheinlichkeiten $\hat{p} \in [0, 1]$ abbildet.


* **Platt Scaling**:
* **Parametrische Logistische Kalibrierung**: Anpassung einer Sigmoid-Funktion $\hat{p}(s) = \frac{1}{1 + \exp(A s + B)}$ via regularisierter Maximum-Likelihood-Optimierung (Newton-Verfahren) zur präzisen Abbildung kontinuierlicher Scores auf Wahrscheinlichkeiten.


* **Conformal Prediction**:
* **Verteilungsfreie Konfidenzgarantien**: Split-Conformal-Prediction-Verfahren zur Berechnung mathematisch garantierter Konfidenzintervalle und Vorhersagemengen auf endlichen Kalibrierungsstichproben mit garantierter Überdeckungswahrscheinlichkeit $\mathbb{P}(Y_{n+1} \in \hat{C}(X_{n+1})) \ge 1 - \alpha$.


* **Drift Detection**:
* **Statistische Laufzeit-Überwachung**: Kontinuierlicher Abgleich der Live-Score-Verteilungen gegen Referenz-Kalibrierungsverteilungen mittels Kolmogorov-Smirnov-Test, Population Stability Index (PSI) und Page-Hinkley-Sequenztest zur frühzeitigen Erkennung von Verteilungsverschiebungen (*Covariate/Concept Drift*).



---

## 2. Domäne: Autonome Agenten, Workflow-Orchestrierung & Kognition (`agentic & cognition`)

<img width="2752" height="1536" alt="agentic" src="https://github.com/user-attachments/assets/87780e2a-54bb-492f-817b-df9f0d67c1c3" />

Diese Domäne stellt die deterministische Execution-Engine für mehrstufige Agenten-Prozesse, deren Fehlerbehandlung sowie die langfristige Konsolidierung, Verdichtung und semantische Aggregation des Gedächtnisses bereit.

### Agenten-Execution-Engine (`contextra-agent`)

* **StateGraph Execution-Schleife (`engine`, `graph`, `step`)**:
* **Transaktionaler Zustandsautomat**: Führt gerichtete Agenten-Graphen strikt entlang des atomaren Vier-Phasen-Zyklus **`checkpoint → execute → commit → audit`** aus.
* **Deterministische Laufzeit-Primitiven (`id_gen`, `clock`)**: Alle Knoten-, Schritt- und Event-Identifikatoren werden über eine deterministische ID-Generierung (`id_gen`, z. B. Hash-/Counter-basiert) erzeugt; Zeitstempel stammen aus einer injizierbaren, abstrahierten Zeitquelle (`clock`), was 100 % reproduzierbare Replay-Läufe und deterministische Tests ermöglicht.


* **Bedingtes Routing, Zielprüfung & Loop-Protection (`condition`, `goal_condition`, `loop_protection`)**:
* **Bedingtes Kanten-Routing (`condition`)**: Auswertung typsicherer `EdgeCondition`-Prädikate auf dem aktuellen Graph-Zustand für dynamische Verzweigungen, Fallbacks und Parallel-Fan-Outs.
* **Explizite Zielprüfung (`goal_condition`)**: Formale Verifikation von `GoalCondition`-Invarianten zur validierten Terminierung eines Workflows.
* **Zyklen- und Endlosschleifen-Schutz (`loop_protection`)**: Kombiniert maximale Schritt- und Rekursions-Limits mit kryptografischem State-Fingerprinting (BLAKE3-Hash über Knoten-ID, Zustands-Payload und Tool-Argumente), um wiederholte Null-Fortschritts-Zyklen und Oszillationen sofort zu erkennen und abzufangen.


* **Event-Driven Execution & Dead-Letter Queue (`event_source`, `dlq`)**:
* **Ereignisgesteuerte Agenten-Schleife (`event_source`)**: Asynchrone Verarbeitung externer und interner Ereignisströme über ein einheitliches `EventSource`-Interface mit Backpressure-Steuerung.
* **Deterministische Dead-Letter Queue (`dlq`)**: Isolierung fehlgeschlagener Schritte oder nicht verarbeitbarer Events in einer persistenten DLQ inklusive Fehlerklassifizierung, Retry-Zählern, Exponential-Backoff-Strategien und kontrollierter Wiederverarbeitung (*Replay / Re-Drive*).


* **CLM Scratchpad & System-Prompt-Pinning (`clm_scratchpad`, `context`)**:
* **CLM Scratchpad (*Cognitive Loop Memory Scratchpad*)**: Isolierter, strukturierter Arbeitsspeicher für Zwischenergebnisse, Hypothesen und Tool-Ausgaben innerhalb eines Reasoning-Zyklus mit deterministischem Reset-Flow nach Abschluss einer Teilaufgabe.
* **System-Prompt-Pinning (`context`)**: Latenz- und KV-Cache-optimierte Fixierung unveränderlicher Systeminstruktionen, Rollenprofile und Sicherheitsregeln im Präfix des Kontextfensters, geschützt vor Verdrängung durch Kontext-Verdichtung.


* **Token- & Schritt-Budget-Steuerung (`budget`)**:
* **Race-freies Ressourcen-Accounting (`TokenBudget`)**: Atomares, thread-sicheres Tracking (`AtomicU64` mit Compare-And-Swap-Reservierung) für Prompt-Tokens, Completion-Tokens, Ausführungsschritte, Tool-Aufrufe und Kosten-Budgets über nebenläufige Sub-Agenten hinweg.


* **Tool-Registry, Manifeste & Retrieval-Tool (`tools::manifest`, `tools::registry`, `tools::retrieve`)**:
* **Deklarative Tool-Manifeste (`tools::manifest`)**: Typisierte Spezifikation von Werkzeug-Schnittstellen inklusive Eingabe-/Ausgabe-Schemas, Berechtigungs-Scopes, Timeout-Vorgaben und Seiteneffekt-Klassifizierung.
* **Zentrale Tool-Registry (`tools::registry`)**: Laufzeit-Verzeichnis mit Schema-Validierung vor Ausführung, Versionierung und Dispatch in isolierte Sandbox-Umgebungen.
* **Natives Retrieval-Tool (`tools::retrieve`)**: Direkt im Agenten-Kern verankertes Werkzeug für autonome Hybrid-Retrieval-, Graph- und Time-Travel-Abfragen ohne Serialisierungs-Overhead.


* **Execution-Audit (`audit`)**:
* **Manipulationssichere Ablaufprotokollierung**: Lückenlose, strukturierte Erfassung aller Zustandsübergänge, Routing-Entscheidungen, Tool-Aufrufe, Budget-Abbuchungen und Fehlerereignisse.



---

### Langzeitgedächtnis, Verdichtung & Konsolidierung (`contextra-cognition`)

* **Memory Consolidation (`memory_consolidation`, `consolidation_executor`, `consolidation_locks`)**:
* **Structural Pass & Near-Duplicate-Erkennung**: Mehrstufiger Konsolidierungsdurchlauf (`consolidation_executor`), der mittels **Detect Near Duplicates** (kombiniert aus exakten Hashes, MinHash-Signaturen und Vektor-Kosinus-Ähnlichkeit) redundante Gedächtniseinträge identifiziert und zusammenführt.
* **Sliding-Window-Clustering & Strukturanalyse**: Gruppierung zeitlich und semantisch benachbarter Episoden über ein gleitendes Fenster (*Sliding-Window-Clustering*) zur Vermeidung quadratischer Vergleichskomplexität, gefolgt von einer graph-topologischen Strukturanalyse.
* **Feingranulare Locks & Doppel-Trigger-Schutz (`consolidation_locks`)**: Partitionierte Sperren auf Namespace-/Cluster-Ebene sowie Idempotenz-Guards verhindern Race Conditions und doppelte Ausführung bei zeitgleichen Konsolidierungs-Triggern.


* **Context Compaction (`context_compaction`: `compactor`, `session`, `cleanup`)**:
* **Automatisierte Session- & Dialogverdichtung**: Verdichtet wachsende Dialog- und Transaktionshistorien bei Annäherung an Kontext- oder Token-Schwellenwerte inklusive transaktionaler Bereinigungs- und Wiederholungslogik (`compactor`, `session`, `cleanup`) über drei wählbare Strategien:
* **Truncate-Strategie**: Deterministisches Entfernen der ältesten bzw. niedrigst priorisierten Kontextsegmente unter striktem Erhalt gepinnter System- und Zielvorgaben.
* **Summary-Strategie**: Komprimierende Zusammenfassung vergangener Dialog- und Handlungshistorien in strukturierte Gedächtnisknoten mit Provenance-Link auf die archivierten Rohdaten.
* **Selective-Strategie**: Relevanz- und entropiebasierte Selektion kritischer Fakten, Entitäten, Constraints und Entscheidungen bei gezielter Eliminierung redundanter Zwischenschritte.




* **LeanRAG Stage 3 (Semantische Aggregation & Synthese — `aggregation_phase`, `synthesis_phase`, `semantic_aggregation_facade`, `pre_condensed_input_intake`)**:
* **Pre-Condensed Input Intake**: Übernahme und Normalisierung bereits vorverdichteter Episoden- und Entitätsgruppen in die Aggregations-Pipeline (`semantic_aggregation_facade`).
* **Diagonal-GMM Clustering (`aggregation_phase`)**: Probabilistisches Soft-Clustering semantischer Embeddings mittels Expectation-Maximization (EM) für Gaussian Mixture Models mit diagonalen Kovarianzmatrizen $\Sigma_k = \text{diag}(\sigma_{k,1}^2, \dots, \sigma_{k,D}^2)$, was lineare Skalierung $\mathcal{O}(K \cdot D)$ in der Embedding-Dimension und numerische Stabilität durch Varianz-Regularisierung garantiert.
* **Typ-Denoising**: Mehrheits- und Konfidenz-basierte Bereinigung inkonsistenter oder verrauschter Entitäts- und Relationstypen innerhalb der gebildeten Cluster.
* **Alpha-Knoten-Synthese (`synthesis_phase`)**: Generierung abstrahierter übergeordneter Konzeptknoten ($\alpha$-Knoten), die verdichtete Cluster-Semantik kapseln und hierarchisch mit den zugrunde liegenden Detail-Entitäten verknüpft bleiben.
* **Super-Hyperkanten-Generierung**: Bündelung mehrerer Entitäten und $\alpha$-Knoten zu hochdimensionalen, gewichteten Super-Hyperkanten für abstrahiertes Multi-Hop-Reasoning.


* **Transitivity Veto (`transitivity_veto`)**:
* **Schutz vor transitiver Verschmelzungs-Drift**: Graph-theoretischer Wächteralgorithmus, der fehlerhafte transitive Cluster-Fusionen ($A \approx B \land B \approx C \Rightarrow A \approx C$) blockiert, sobald die direkte semantische oder strukturelle Ähnlichkeit $\text{sim}(A, C)$ einen strikten Veto-Schwellenwert unterschreitet.


* **Grounding Validator (`grounding`)**:
* **Halluzinations-Schutz im Langzeitgedächtnis**: Validierungskomponente innerhalb der Konsolidierungs-Pipeline, die synthetisierte Zusammenfassungen und $\alpha$-Knoten gegen ihre ursprünglichen Quelldokumente prüft (Fakten-, Entitäts- und Entailment-Abgleich) und unbelegte Aussagen vor der Persistierung zurückweist.


* **Memory Decay & Maintenance Scheduler (`decay`, `maintenance_scheduler`, `maintenance_config`, `channel`, `graph_sink`)**:
* **Exponentielles Memory Decay (`decay`)**: Zeitabhängiges Abklingen der Aktivierungsstärke $A(t) = A_0 \cdot e^{-\lambda \Delta t} + \sum_i w_i e^{-\lambda (t - t_i)}$ ungenutzter Erinnerungen mit Verstärkung bei Wiederabruf sowie geregelter Archivierung bzw. Expiration.
* **Asynchroner Maintenance Scheduler (`maintenance_scheduler`, `maintenance_config`)**: Konfigurierbarer Hintergrund-Scheduler zur zyklischen Steuerung von Konsolidierung, Decay, Expiry und LeanRAG-Aggregation über entkoppelte asynchrone Pipelines (`channel`) und transaktionale Graph-Senken (`graph_sink`).



---

## 3. Domäne: Adaptives Routing, Regelungstechnik & Modellsteuerung (`llm & adapt`)
<img width="2752" height="1536" alt="routing" src="https://github.com/user-attachments/assets/d792ecc5-2744-4010-b258-f9011030ad81" />
Diese Domäne steuert die kontextabhängige Auswahl, Online-Exploration, Lastverteilung und Echtzeit-Parametrierung verschiedener KI-Modelle (lokale SLMs, Remote-LLMs, Candidate Pools und Gateways).

### Kontextuelle Banditen & Online-Lernverfahren (`contextra-adapt`, `contextra-router`)

* **LinUCB Contextual Bandits (`bandit`)**:
* **Upper-Confidence-Bound-Modellauswahl**: Wählt für einen Anfrage-Kontextvektor $x_t \in \mathbb{R}^d$ das Modell $a_t \in \mathcal{A}$ mit maximalem geschätzten Nutzen plus Konfidenz-Explorationsbonus:

$$a_t = \arg\max_{a \in \mathcal{A}} \left( \hat{\theta}_a^\top x_t + \alpha \sqrt{x_t^\top A_a^{-1} x_t} \right)$$


* **Sherman-Morrison-Rank-1-Updates & Sketched Projection**: Aktualisiert die inverse Designmatrix $A_a^{-1}$ inkrementell über die Sherman-Morrison-Formel in $\mathcal{O}(d^2)$ statt $\mathcal{O}(d^3)$ Matrixinversion, nutzt **deterministische Sketched Projection** (CountSketch / Sparse Random Projection) zur Dimensionsreduktion hochdimensionaler Kontextvektoren und sichert numerische Langzeitstabilität über Ridge-Regularisierung und periodische Symmetrisierung.


* **Flow-Corrected Thompson Sampling (FC-TS — `flow_thompson`)**:
* **Nicht-stationäres Bayessches Posterior-Sampling**: Zieht Modellparameter aus der Posterior-Verteilung und kompensiert automatische Fluss- und Performance-Drifts (z. B. tageszeitabhängige API-Latenzschwankungen oder Modell-Updates) mittels eines gleitenden **Ring-Buffers**, der veraltete Beobachtungen kontinuierlich aus dem Posterior verdrängt.


* **RIE-Greedy (`rie_greedy`)**:
* **Regularization-Induced Exploration**: Personalisiertes, kontextsensitives Modell-Routing, das empirische Belohnungsschätzer mit expliziten Regularisierungstermen (Entropie-/L2-Strafen) kombiniert, um eine glatte, deterministisch kontrollierbare Exploration ohne unvorhersehbare Zufallssprünge zu erreichen.


* **Catoni M-Estimator**:
* **Ausreißer-resistente Belohnungs- und Latenzschätzung**: Robuster M-Schätzer nach Catoni auf Basis der logarithmischen Einflussfunktion $\psi(x) = \text{sign}(x)\ln(1 + |x| + x^2/2)$, der extreme Network-Timeouts oder Latenz-Spikes zuverlässig dämpft, ohne die Lernrate der Banditen zu verzerren.


* **Off-Policy Evaluation & IPS (`offpolicy`)**:
* **Verzerrungsfreie Offline-Policy-Bewertung**: Evaluiert alternative Routing-Strategien $\pi_e$ auf historischen Log-Daten einer Logging-Policy $\pi_0$ mittels **Inverse Propensity Scoring (IPS)** und Self-Normalized IPS:

$$\hat{V}_{\text{IPS}}(\pi_e) = \frac{1}{n} \sum_{t=1}^{n} \frac{\pi_e(a_t \mid x_t)}{\max(\pi_0(a_t \mid x_t), \epsilon_{\text{clip}})} r_t$$



unter strikter Einhaltung der Positivitäts- und Propensity-Logging-Anforderungen.


* **Shadow Mode (`shadow_mode`)**:
* **Risikofreie Live-Evaluation**: Führt neue Bandit- und Routing-Strategien parallel zum produktiven Pfad aus, protokolliert hypothetische Modellentscheidungen, Propensities und geschätzte Kosten, ohne die tatsächliche Modellantwort an den Nutzer zu beeinflussen.



---

### Kontrolltheoretische Regler & Stabilitätsüberwachung (`pid`, `homeostat`, `decay_controller`, `lyapunov`)

* **PID-Latenzregler (`pid`, `pid_latency_controller`)**:
* **Diskreter Regelkreis mit Anti-Windup**: Passt Candidate-Pool-Größen, Overfetch-Faktoren und Timeout-Budgets dynamisch an die Regelabweichung $e(t) = L_{\text{target}} - L_{\text{measured}}(t)$ an:

$$u(t) = K_p e(t) + K_i \sum_{\tau=0}^{t} e(\tau)\Delta t + K_d \frac{e(t) - e(t-1)}{\Delta t}$$



Integriert Clamping- und Conditional-Integration-**Anti-Windup-Schutz** gegen Integrator-Sättigung bei Lastspitzen sowie Tiefpass-Filterung des Differentialanteils.


* **Homeostat-Regler (`homeostat`)**:
* **Selbstregulierender Gleichgewichtscontroller**: Stabilisiert Systemauslastung, Queue-Längen und Modell-Diversität (verhindert Routing-Monokultur auf ein einzelnes Modell) innerhalb definierter homöostatischer Zielkorridore.


* **Exponential Decay-Controller (`decay_controller`)**:
* **Halbwertszeit-basierte Zeitgewichtung**: Steuert den zeitlichen Vergessensfaktor $\gamma = 2^{-\Delta t / t_{1/2}}$ anhand konfigurierbarer Halbwertszeiten (**Half-Life $t_{1/2}$**) zur kontinuierlichen Abwertung historischer Performance-Metriken.


* **Lyapunov-Stabilitätsanalyse (`lyapunov`)**:
* **Mathematische Konvergenz- und Schwingungsüberwachung**: Überwacht zur Laufzeit eine quadratische Lyapunov-Energiefunktion $V(x) = x^\top P x > 0$ über die Zustandsvektoren gekoppelter PID- und Bandit-Regelkreise. Erkennt drohende Oszillationen oder Divergenzen ($\Delta V(x) > 0$) sofort und schaltet deterministisch auf gedämpfte Sicherheits-Parameter um.



---

## 4. Domäne: Datenschutz, Kryptografie, Sicherheit & Compliance (`safety, crypto & compliance`)

<img width="2752" height="1536" alt="security" src="https://github.com/user-attachments/assets/2c02271c-171c-4256-92fc-d43a4b98623d" />

Diese Domäne gewährleistet den Schutz vor Datenexfiltration und Prompt-Injection-Angriffen, sichert kryptografische Schlüssel im Speicher und auf Datenträgern ab und automatisiert regulatorische Nachweispflichten.

### Egress-Schutz, DLP & Prompt-Sicherheit (`contextra-privacy`, `contextra-prompt-guard`)

* **Rule-Based Entity Recognition & DLP**:
* **Deterministische PII-/SPI-Erkennung**: Mehrstufige Inspektion ausgehender Datenströme über kompilierte reguläre Ausdrücke, Aho-Corasick-Gazetteers und mathematische Prüfziffern-Validierung (u. a. ISO-7064-Mod-97-10 für **IBAN**, Prüfziffern für deutsche **Steuer- und Ausweisnummern**, Luhn-Algorithmus für Kreditkarten, E-Mail-Adressen und Telefonnummern).


* **Surrogate Vaulting & Rehydrierung**:
* **Bidirektionale Pseudonymisierung**: Ersetzt erkannte sensible Entitäten vor der Übermittlung an externe LLMs durch typsichere, deterministische Surrogat-Token (z. B. `[IBAN_01]`) und übersetzt diese in der empfangenen Modellantwort über ein flüchtiges, sitzungsgebundenes Mapping-Vault verlustfrei zurück (**Rehydrierung**).


* **k-NN Egress Guard**:
* **Semantische Paraphrasierungs-Schutzbarriere**: Berechnet vor jedem externen Egress die vektorielle $k$-Nächste-Nachbarn-Ähnlichkeit ausgehender Payloads gegen einen geschützten Sperr-Index vertraulicher Referenzinhalte und blockiert Übertragungen, die geschütztes Wissen in paraphrasierter Form preisgeben würden.


* **Bulk Exfiltration Detector**:
* **Sliding-Window-Volumen- & Ratenüberwachung**: Überwacht ausgehende Datenmengen, Token-Raten und einzigartige PII-Entitätszahlen pro Zeitfenster, Mandant und Ziel-Endpoint, um sowohl plötzliche Datenabflüsse (*Burst*) als auch schleichende Extraktion (*Low-and-Slow*) zu unterbinden.


* **Prompt-Injection Detection & Audit (`contextra-prompt-guard`)**:
* **Mehrstufige Instruktions-Isolierung**: Erkennt direkte Jailbreak-Versuche sowie indirekte Prompt-Injections in abgerufenen Dokument-Chunks oder Tool-Antworten via Heuristik-/Signatur-Scanner und struktureller Trennung von System-Instruktionen und Nutzdaten.


* **Context Edit Audit**:
* **Kryptografisch verkettetes Redaktions-Log**: Protokolliert jede Maskierung, Surrogat-Ersetzung und Blockade in einer über **BLAKE3** verketteten, manipulationssicheren Audit-Sequenz ($H_i = \text{BLAKE3}(H_{i-1} \parallel \text{event}_i)$).



---

### Kryptografie, Schlüsselverwaltung & Löschung (`contextra-crypto`, `contextra-erasure`, `contextra-vault`, `contextra-sys`)

* **Authentifizierte Verschlüsselung & Hierarchisches Key-Wrapping (`aead`, `dek`, `kdf`)**:
* **AES-256-GCM-SIV (`aead`)**: Nonce-Reuse-resistente authentifizierte Verschlüsselung nach RFC 8452 mittels synthetischem Initialisierungsvektor, die selbst bei Nonce-Kollisionen Vertraulichkeit und Integrität garantiert.
* **Hierarchisches DEK-Wrapping & MFKD (`dek`)**: Mehrstufige Schlüsselhierarchie mit Key Encryption Keys (KEK), mandanten- und dokumentspezifischen Data Encryption Keys (DEKs) sowie abgeleiteten Schlüsselstrukturen (**MFKD**).
* **Speicherharte Schlüsselableitung (`kdf`)**: Kryptografische Key Derivation via **Argon2id** (konfigurierbare Speicherhärte $m$, Iterationen $t$, Parallelität $p$) sowie subkey-spezifische Expansion mittels **HKDF**.


* **Cryptographic Shredding & Segmentierte KV-Verschlüsselung (`kv_cipher`, `kv_segment`, `contextra-erasure`)**:
* **Punktuelle Kryptografische Löschung**: Verschlüsselt Dokumente und Mandanten-Segmente mit isolierten DEKs (`kv_cipher`, `kv_segment`). Das Vernichten eines einzelnen Schlüssels (**Cryptographic Shredding**) macht die zugehörigen Chiffretexte in KV-Store, Indizes und Backups sofort und mathematisch unumkehrbar unlesbar – ohne zeitaufwendiges Re-Indexing. Beinhaltet einen **asynchronen Eviction-Worker** und **`Deferred Zeroize`** zur sicheren Bereinigung von Schlüsselmaterial aus Caches nach Abschluss laufender Lese-Snapshots.


* **Ed25519 Deletion Proofs & Audit-Chain (`audit_chain`, `anti_tamper`)**:
* **Extern verifizierbare Löschnachweise**: Erzeugt digital mit **Ed25519** signierte **Deletion Proofs v3** inklusive **Präfix-Kollisionsschutz** (Domain-Separation-Tags), einem unveränderlichen **Revocation-Log** sowie kryptografischer Hash-Verkettung (`audit_chain`, `anti_tamper`).


* **WAL-Kryptografie & Vollständigkeitsprüfung (`wal_crypto`, `wal_completeness`)**:
* **Authentifizierte Write-Ahead-Logs**: Sichert jeden WAL-Eintrag über fortlaufende **HMAC-Ketten (v2)** und verifiziert beim Recovery sowie im laufenden Betrieb über **`wal_completeness`**, dass keine Log-Einträge abgeschnitten (*Truncation Attack*) oder eingefügt wurden.


* **Model Fingerprinting (`model_fingerprint`)**:
* **Kryptografische Modell-Integritätsprüfung**: Berechnet deterministische kryptografische Fingerprints (BLAKE3 / SHA-256) über Modellgewichte, Architektur-Header und Tokenizer-Konfigurationen zur Verifikation der Integrität vor jeder Inferenz.


* **RAM-Locking & Volatile Vaulting (`contextra-sys`, `contextra-vault`, `secret`, `volatile_vault`)**:
* **Swap-Schutz & Sichere Speicherbereinigung**: Sperrt sensible Speicherseiten auf Betriebssystemebene gegen Auslagerung auf Datenträger via **`mlock`** (Unix) und **`VirtualLock`** (Windows) und überschreibt flüchtige Geheimnisse (`Secret`, `VolatileVault`) beim Freigeben deterministisch via **`ZeroizeOnDrop`**.



---

### Regulatorische Compliance-Generatoren (`contextra-audit-export`, `contextra-avv-generator`)

* **Audit-Register & BSI-Grundschutz-Mapping (`contextra-audit-export`)**:
* **Automatisierte Nachweis- & Verzeichnis-Exporte**: Generiert strukturierte Verarbeitungsverzeichnisse und Audit-Exporte in **JSON**, **CSV** und **Markdown** mit integrierter automatischer **Sanitization** vertraulicher Nutzdaten sowie direktem Regel-Mapping auf Bausteine des **BSI IT-Grundschutz-Kompendiums** (`bsi_mapping`).


* **DSGVO-AVV-Generator (`contextra-avv-generator`)**:
* **Capability-gesteuerte Vertragsgenerierung**: Synthetisiert deterministisch rechtskonforme Auftragsverarbeitungsverträge gemäß **Art. 28 DSGVO** direkt aus den konfigurierten System-Capabilities mit modularen Klauseln für:
* Weisungsgebundenheit (`instructions`),
* Vertraulichkeit (`confidentiality`),
* Betroffenenrechte (`data_subject_rights`),
* Unterstützungspflichten (`assistance`),
* Kontroll- und Auditrechte (`audit`),
* Vertragsbeendigung und Löschung (`termination`).





---

## 5. Domäne: Speichersystem, Transaktionen, Checkpointing & Isolation (`storage, foundation & durability`)

<img width="2752" height="1536" alt="storage" src="https://github.com/user-attachments/assets/4ac5cdb6-b731-4653-af1c-4435c670a7fc" />

Diese Domäne bildet das fundamentale Persistenz-, Wiederherstellungs- und Sicherheitsfundament mit strikten ACID-Garantien, Zeitreise-Snapshots und hardwarenaher Ausführungsisolation.

### Crash-konsistentes Dateisystem & Journaling (`contextra-durable-fs`)

* **Write-Ahead-Journal (`journal`, `journal_crash_resume`)**:
* **Atomares Dateisystem-Journaling**: Protokolliert Metadaten- und Block-Operationen vorab in einem prüfsummengesicherten Journal und stellt nach Systemabstürzen über **`journal_crash_resume`** einen deterministischen, idempotenten Konsistenzzustand wieder her.


* **Data Scrubbing & Bit-Rot-Schutz (`scrub`, `health`)**:
* **Proaktive Integritätsüberwachung**: Überprüft persistierte Datenblöcke im Hintergrund gegen gespeicherte Kryptoprüfsummen auf stille Speicherkorruption (**Bit-Rot**), bereinigt verwaiste temporäre Dateien und steuert den Betriebszustand über eine formale **Dateisystem-Health-State-Machine**.


* **VFS- & Plattform-Abstraktion (`vfs`, `platform`, `fault_injector`)**:
* **Deterministische Schreibbarrieren & Fehlerinjektion**: Abstrahiert Dateisystemzugriffe über ein einheitliches Virtual File System (`vfs`) mit plattformspezifischen Synchronisationsbarrieren (`fsync` / Directory-Sync in `platform`) sowie einem deterministischen **`fault_injector`** zur systematischen Prüfung sämtlicher Crash- und Torn-Write-Fenster.



---

### LSM-Tree Speicherkern (`contextra-store`)

* **LSM-Tree (Log-Structured Merge-Tree)**:
* **Schreiboptimierte Speicherhierarchie**: Kombiniert lock-freie speicherresidente **MemTables**, unveränderliche blockkomprimierte **SSTables (V4)**, anpassbare assoziative **Merge-Operatoren (`merge_operator_adapter`)** für In-Place-Delta-Aggregationen sowie eine **hierarchische Hintergrund-Kompaktierung** (Leveled/Tiered Compaction mit Tombstone-Garbage-Collection).


* **Zweistufige Bloom-Filter**:
* **Hierarchische Existenz-Filterung**: Probabilistische Filterstruktur auf **SSTable-Ebene** (Vermeidung ganzer Datei-Reads) und **Datenblock-Ebene** (Vermeidung unnötiger Block-Dekompressionen) zur schnellen Abweisung nicht existierender Schlüssel.


* **Write-Ahead-Log (WAL)**:
* **Crash-konsistentes Append-Only-Log**: Persistiert Schreib-Batches sequenziell mit kryptografischen HMAC-Ketten und einem integrierten **Orphan-Scanner** zur Identifikation und Bereinigung unvollständig geschriebener WAL-Fragmente beim Start.



---

### Transaktions-Isolation & Multi-Index-Atomarität (`contextra-mvcc`, `contextra-db::transaction`)

* **SSI (Serializable Snapshot Isolation)**:
* **Anomaliefreie MVCC-Isolation**: Garantiert vollständige Serialisierbarkeit ohne Lesesperren und verhindert zuverlässig Write-Skew-, Phantom-Read- und Anti-Dependency-Anomalien.


* **SequenceLog & ReadSet-Prüfung**:
* **Deterministische Konflikterkennung**: Verwaltet über das `SequenceLog` streng monotone Transaktionszeitstempel und prüft beim Commit sämtliche gelesenen Schlüssel und Bereichsprädikate (`ReadSet`) gegen konkurrierende Schreibvorgänge.


* **Zwei-Phasen-Commit (2PC) & Kompensierende Aktionen (`db_transaction`, `intent`, `compensating_actions`)**:
* **Atomare Multi-Index-Transaktionen**: Koordiniert Schreibvorgänge über heterogene Indizes (Vektor-, Volltext-, Graph- und KV-Index) hinweg über ein persistentes **Intent-Log (`intent`)**, einen deterministischen Rollback-Manager und inverse **kompensierende Aktionen (`compensating_actions`)**, abgesichert durch adaptive **Backpressure-Steuerung** und strikte **Mandanten-Isolation (Tenant Isolation)**.



---

### Transaktionales Checkpointing & Time-Travel (`contextra-checkpoint`)

* **RAII Auto-Checkpointing & Rollback (`guard`)**:
* **Scope-gebundene Transaktionssicherheit**: Ein RAII-basierter `CheckpointGuard` erzeugt beim Eintritt in kritische Code-Scopes automatisch einen konsistenten Zustandspunkt und führt bei vorzeitigem Abbruch oder Fehler einen deterministischen Rollback aus, sofern kein explizites Commit erfolgt.


* **Hardlink-Cloner (`hardlink_cloner`)**:
* **Zero-Copy-Snapshots**: Erzeugt nahezu verzögerungsfreie Snapshots auf Dateisystemebene durch Hardlinking unveränderlicher SSTable-Dateien ohne physisches Kopieren von Nutzdaten.


* **Time-Travel-Restore, Manifest-Integrität & Orphan-Bereinigung (`store`, `manifest`, `orphan`)**:
* **Punktgenaue Zeitreise-Wiederherstellung**: Ermöglicht das Zurücksetzen der gesamten Datenbank auf historische Transaktionspunkte (**Time-Travel**) unter strenger monotoner `TxID`-Validierung, kryptografisch geprüften Checkpoint-Manifesten (`manifest`) und referenzgezählter Bereinigung verwaister Snapshot-Artefakte (`orphan`).



---

### Ausführungs-Isolation (`contextra-sandbox`)

* **WASM Fuel-Budgeting**:
* **Deterministische Ressourcen- und Laufzeitbegrenzung**: Führt externe Tools und Erweiterungen in einer isolierten WebAssembly-Sandbox ohne unautorisierten Host-Zugriff aus und begrenzt Rechenzeit deterministisch über Instruktions-**Fuel-Budgets** sowie strikte Speicher-Obergrenzen.



---

## 6. Domäne: Ingest, Inferenz, Schnittstellen & Evaluation (`data, llm, apps & benchmarks`)
<img width="2752" height="1536" alt="ingest" src="https://github.com/user-attachments/assets/18e529ac-5458-4d62-8dcb-48c962df873c" />
Diese Domäne erschließt externe Datenquellen, bindet lokale sowie verteilte KI-Inferenz-Backends an, stellt standardisierte Protokoll- und Sprachschnittstellen bereit und sichert Qualität sowie Latenz über automatisierte Benchmarks ab.

### Dokumenten-Ingest & Strukturiertes Chunking (`contextra-ingest`, `contextra-db::chunker`)

* **Prose & Structure Chunking (`blocks`, `sections`, `split`, `windows`, `estimate`)**:
* **Formatbewusste Extraktion & Hierarchie-Erhaltung**: Extrahiert Inhalte aus **PDF**, **DOCX**, **Markdown** und **Klartext**, segmentiert sie in typisierte Strukturblöcke (`blocks`: Absätze, Tabellen, Codeblöcke), rekonstruiert Überschriften-Hierarchien (`sections`), schätzt Token-Budgets deterministisch (`estimate`), teilt entlang natürlicher Satz- und Abschnittsgrenzen (`split`) und generiert überlappende **Sliding-Windows (`windows`)**.


* **Deterministischer Datenbank-Export & -Import (`export`, `import`)**:
* **Verlustfreie Kollektions-Portabilität**: Kanonische Serialisierung und Wiederherstellung vollständiger Datenbankbestände inklusive Vektoren, Volltext-Postings, Graph-Kanten, Hyperkanten und Metadaten mit Integritäts-Checksummen.



---

### Lokale & Verteilte Inferenz-Anbindung (`contextra-infer-candle`, `contextra-infer-onnx`, `contextra-infer-ollama`)

* **GGUF-Metadaten-Parsing & In-Process Inferenz (`contextra-infer-candle`)**:
* **Native On-Device-Ausführung**: Parst **GGUF-Modell-Header** und Quantisierungs-Metadaten via Zero-Copy-Memory-Mapping und führt quantisierte Sprach- sowie Embedding-Modelle latenzarm direkt im Rust-Prozess über das **Candle-Framework** (CPU SIMD / GPU) aus.


* **ONNX Runtime Integration (`contextra-infer-onnx`)**:
* **Hardware-beschleunigte Graph-Inferenz**: Bindet standardisierte ONNX-Modelle mit Session-Pooling und dynamischem Batching für dichte Embeddings, **Cross-Encoder-Reranking** und Klassifikationsaufgaben ein.


* **Ollama REST-Adapter & Circuit-Breaker (`contextra-infer-ollama`)**:
* **Fehlertolerante Server-Anbindung**: Verbindet lokale und entfernte Ollama-Instanzen über Connection-Pooling, Timeout-Steuerung, einen selbstheilenden **Circuit-Breaker** (`Closed → Open → Half-Open`) und automatisches Fallback-Routing.



---

### Protokoll-Schnittstellen & Sprach-Bindings (`contextra-mcp`, `contextra-py`, `contextra-engine`)

* **Model Context Protocol Server (`contextra-mcp`)**:
* **Zustandsloser JSON-RPC 2.0 / STDIO Server**: Auslieferbar als eigenständiges, abhängigkeitsfreies Binary zur nahtlosen Integration von Retrieval-, Graph-, Memory- und Agenten-Funktionen in IDEs und MCP-fähige Agenten-Hosts.


* **Python-Bindings (`contextra-py`)**:
* **Native Python-Integration**: Kompiliert als Python-Wheels (PyPI-Distribution) mit GIL-Freigabe während rechenintensiver Rust-Operationen und direkter Speicherübergabe für Vektor-Arrays.


* **Zentrale Integrations-Engine (`contextra-engine`)**:
* **Einheitliche System-Orchestrierung**: Kapselt Lifecycle-Management, Telemetrie- und Metriken-Instrumentierung, Health-Diagnostik sowie die strikte Durchsetzung domänenübergreifender Architektur-Invarianten.



---

### Benchmarking & Evaluations-Suite (`contextra-bench`)

* **Standardisierte Evaluations-Pipelines (`beir_eval`, `ann_benchmarks`, `path_rag_sweep`)**:
* **BEIR- & Komposita-Evaluation (`beir_eval`)**: Automatisierte Berechnung von $\text{NDCG}@k$, $\text{MRR}@k$ und $\text{Recall}@k$ über Standard-BEIR-Datensätze sowie den spezialisierten deutschen Komposita-Korpus **`de_mini`**.
* **ANN-Recall/Latenz-Kurven (`ann_benchmarks`)**: Systematische Vermessung von Recall@k, Durchsatz (QPS) und Tail-Latenzen ($p_{50}, p_{95}, p_{99}$) über HNSW, SQ8, RaBitQ und DiskANN.
* **Langzeitgedächtnis-Benchmarks (`LoCoMo`, `LongMemEval`)**: Evaluierung von Multi-Session-Erinnerungsabruf, zeitlicher Konsistenz und Konsolidierungsqualität.
* **Multi-Hop-PathRAG-Sweeps & CI-Regression Gates (`path_rag_sweep`)**: Automatisierte Parameter-Sweeps über Graph-Traversierungs- und Fusionsparameter, **100k-Skalierungstests** sowie CI-gestützte **Regression Gates**, die Qualitäts- oder Latenz-Regressionen vor dem Merge deterministisch blockieren.
