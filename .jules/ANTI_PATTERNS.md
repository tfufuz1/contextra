# Contextra — Anti-Pattern-Matrix (ANTI_PATTERNS.md)

> Vollständiges Verzeichnis aller 42 Anti-Pattern-Matrix (APM) Regeln und crate-spezifischen Pflicht-APMs.
> Quelle: `.jules/contextra-Prompter.html`

## 1. APM-Katalog (42 APMs)

### APM-1: Filesystem-Atomarität
**Kurzbezeichnung:** `APM-1: tmp→fsync→rename→fsync_parent`

Jede Datei-Erstellung muss atomar sein: tmp-Datei → sync_all → rename → fsync_parent_dir. Direktes tokio::fs::write() ohne tmp+rename = Datenverlust bei Absturz.

### APM-2: Kumulative Bedingungen
**Kurzbezeichnung:** `APM-2: if/else-if vs. separate if-Blöcke`

if/else-if-Ketten machen Bedingungen gegenseitig exklusiv — auch wenn beide unabhängig voneinander geprüft werden müssen. Korrekt: zwei separate if-Blöcke, wenn beide Pfade unabhängig ausgeführt werden müssen (z.B. Tombstone-Check UND Custom-Filter). Falsch: if/else-if, weil ein aktiver ersten Zweig den zweiten überspringt. Generischer Name dieses Musters bei Ergebnismengen: "Truncation-before-Filter" — ein Limit/Take/Truncate VOR statt NACH einem Filter (Tombstone-, Permission- oder Custom-Filter) liefert eine zu kleine oder falsche Ergebnismenge, weil aussortierte Treffer bereits unwiderruflich abgeschnitten wurden. Prüfe bei JEDER RRF-/Multi-Step-/Reranking-Pipeline explizit die Reihenfolge: Filter MUSS vor jedem Limit/Truncate stehen.

### APM-3: Best-Effort Locks
**Kurzbezeichnung:** `APM-3: try_write() für Mutationen`

try_write() für Bound-Expansion, State-Updates etc. = stille Fehler unter Last. Mutationen die garantiert ausgeführt werden müssen → write() (blockierend).

### APM-4: Pre-Check vor Aktion
**Kurzbezeichnung:** `APM-4: Budget/Permission vor execute()`

Budget- oder Berechtigungsprüfung NACH der Aktion = die Aktion hat bereits Seiteneffekte ausgeführt. Korrekte Reihenfolge: check() → execute() → consume(). Nach Crash-Recovery ist Budget-Restore Pflicht — ein fehlender Restore gibt Budget dauerhaft aus, ohne es zurückzuführen.

### APM-5: Score vor RRF-Fusion
**Kurzbezeichnung:** `APM-5: score *= X hat 0 Effekt vor RRF`

RRF nutzt ausschließlich Rang-Position, nicht den Score-Wert. score *= boost vor der Fusion hat keinen Effekt auf das Ranking. Boosts müssen POST-RRF angewendet werden.

### APM-6: Geschwister-Konsistenz
**Kurzbezeichnung:** `APM-6: Alle Sibling-Funktionen konsistent prüfen`

Inkonsistente Geschwister-Implementierungen: load_or_create_key_a() korrekt implementiert, aber load_or_create_key_b() mit identischem Muster enthält denselben Fehler. Alle strukturell ähnlichen Funktionen eines Moduls (gleicher Präfix, gleiche Signatur, gleiches Muster) müssen vergleichend geprüft werden — nie nur die Fundstelle.

### APM-7: UTF-8-Byte-Slicing
**Kurzbezeichnung:** `APM-7: Byte-Index auf str = Panic bei ä/ö`

&s[a..b] mit Byte-Indizes panict bei Multibyte-Zeichen (deutsches ä = 2 Bytes). Immer .char_indices() oder unicode-segmentation verwenden, niemals rohe Byte-Arithmetik.

### APM-8: Fire-and-Forget Async
**Kurzbezeichnung:** `APM-8: spawn() in Drop = RAII gebrochen`

tokio::spawn() in Drop-Handler = RAII-Garantie nicht durchgesetzt. Task kann vor Completion gecancelled werden. Außerhalb Tokio-Runtime: Rollback stillschweigend übersprungen.

### APM-9: FFI-Panic-Grenze
**Kurzbezeichnung:** `APM-9: Panic über FFI = Prozessabsturz`

Ein unaufgefangener Rust-Panic, der über eine PyO3-/C-ABI-Grenze läuft, crasht den Host-Prozess statt eine saubere Exception zu werfen. JEDE #[pyfunction]/#[pymethods]-Methode, die synchronen oder async Code aufruft, MUSS über std::panic::catch_unwind (mit AssertUnwindSafe) laufen — betrifft primär contextra-py.

### APM-10: Blockierter Async-Executor
**Kurzbezeichnung:** `APM-10: CPU-Arbeit ohne spawn_blocking blockiert Tokio`

Synchrone, CPU-intensive Arbeit (ONNX-Inferenz, Tokenisierung, Parser, Python-GIL-Aufrufe) direkt auf einem Tokio-Worker-Thread friert alle anderen async Tasks ein. Muss über tokio::task::spawn_blocking laufen; bei PyO3 zusätzlich py.allow_threads() um das GIL freizugeben. Zwei getrennt zu prüfende Ausprägungen: (a) Tokio-Executor-Blockade — spawn_blocking fehlt für CPU-Arbeit im async-Kontext; (b) GIL-Deadlock — ein Rust-Thread hält das Python-GIL während er auf einen anderen Python-Thread wartet (z.B. block_on() ohne vorheriges allow_threads()), wodurch ALLE Python-Threads verhungern, nicht nur der Tokio-Executor. Beide Prüfungen sind Pflicht, nicht austauschbar.

### APM-11: Path-Traversal & Payload-Limits
**Kurzbezeichnung:** `APM-11: Nutzerpfad/-datei ohne Kanonisierung+Limit`

Jeder aus Nutzereingabe stammende Dateipfad (Ingestion, IPC-Commands) MUSS gegen einen Basis-Pfad kanonisiert validiert werden (kein "../"-Ausbruch) UND einer Maximalgröße unterliegen, bevor er gelesen/geparst wird — sonst Path-Traversal bzw. Speicher-Erschöpfung durch übergroße/bösartige Dateien.

### APM-12: Lock-Hierarchie-Inversion
**Kurzbezeichnung:** `APM-12: Locks in falscher Reihenfolge = Deadlock`

Werden in einem Modul mehrere Locks gehalten (z.B. collections → embedder → insert_lock), muss JEDE Aufrufstelle exakt dieselbe Anforderungsreihenfolge einhalten. Eine einzige invertierte Aufrufstelle erzeugt unter Last einen Deadlock, der in Tests oft nicht auftritt.

### APM-13: Nonce-/Key-Domain-Separation
**Kurzbezeichnung:** `APM-13: neue Krypto-Operation ohne eigenen HKDF-Separator`

Jeder neue abgeleitete Schlüssel- oder Nonce-Konstruktionspfad braucht einen eigenen, disjunkten Domain-Separator-String bzw. eine kollisionsfreie Nonce-Konstruktion. Wiederverwendung eines bestehenden Separators für einen neuen Zweck ist ein kritischer Krypto-Befund (Key/Domain Confusion), keine Stilfrage. Zwei Pflicht-Unterprüfungen bei JEDER neuen Nonce-Konstruktion: (1) Nonce-Wiederverwendung/IV-Collision — Instanz-Präfix + Per-Call-Zähler/Random-Suffix dürfen niemals kollidieren, auch nicht nach Prozess-Neustart; (2) Hash-Chain-Kontinuität — bei jeder HMAC-/BLAKE3-Verkettung (WAL, Checkpoint-Manifest) muss geprüft werden, dass ein Kettenbruch (fehlendes prev_hash, Truncation, Teil-Wiederherstellung) kontrolliert als Fehler erkannt wird statt eine korrupte Kette stillschweigend als gültig zu akzeptieren.

### APM-14: Tie-Breaker-Determinismus
**Kurzbezeichnung:** `APM-14: Sortierung nach Score ohne sekundären Schlüssel`

Jede Sortierung nach Score/Rang (RRF, Reranking, Routing-Profile) braucht bei exakt gleichem Primärwert eine deterministische sekundäre Sortierung (z.B. Document-ID). Ohne sie flackert die Ergebnisreihenfolge zwischen identischen Läufen — ein Determinismus-Befund, keine Performance-Frage.

### APM-15: Traversal-Explosion-Cap
**Kurzbezeichnung:** `APM-15: Graph-/Index-Traversal ohne Hard-Cap`

Jeder rekursive oder iterative Traversal-Algorithmus (BFS/DFS, PPR, Compound-Splitter-Rekursion, HNSW-Kandidatenexpansion) braucht ein hartes, PER-LEVEL greifendes Limit (MAX_VISITED_NODES, MAX_CANDIDATES o.ä.). Ein rein globales Ergebnis-Limit schützt nicht vor Explosion während der Traversal selbst (Hub-Node-DoS).

### APM-16: NaN/Inf-Propagation in Scoring
**Kurzbezeichnung:** `APM-16: ungefilterter NaN/Inf-Wert in Score-Vergleich`

Jede Score-Berechnung (BM25-IDF, Routing-Confidence, Reranking) muss NaN/Inf-Werte am Eingang filtern oder klemmen, BEVOR sie in Vergleiche, Sortierung oder Aggregation (RRF) einfließen — f32::max(0.0, NaN) ist NaN, ein ungefilterter Wert macht die gesamte nachgelagerte Fusion nicht-deterministisch statt nur einen einzelnen Score falsch zu machen.

### APM-17: Isolationsstufen-Verwechslung
**Kurzbezeichnung:** `APM-17: Snapshot-Pin ≠ Serializable`

Ein gepinnter Snapshot garantiert NUR einen konsistenten Lesezustand zum Pin-Zeitpunkt (Stale-Read-Schutz) — er garantiert KEINE Serializable-Isolation. Read-Then-Write über zwei separate Transaktionen ohne Re-Validierung des gelesenen Zustands beim Commit = Write Skew (zwei Transaktionen ändern disjunkte Daten basierend auf sich überschneidenden, aber jeweils veralteten Lesezugriffen). Ebenso: Lesen von Daten einer noch nicht committeten fremden Transaktion = Dirty Read. Jede neue Funktion, die "lesen → Entscheidung → schreiben" über einen await-Punkt hinweg tut, braucht eine explizite Dokumentation, welche Isolationsgarantie sie bietet (Snapshot-zum-Zeitpunkt-X vs. Read-Committed vs. Serializable) — "es nutzt ja einen Snapshot" ist KEIN Beweis für Serializable-Sicherheit.

### APM-18: Check-Then-Act ohne Versionsstempel
**Kurzbezeichnung:** `APM-18: Prüfen und Handeln getrennt durch Await = ABA/Lost Update`

Wird ein Zustand geprüft (Check) und danach — getrennt durch einen await-Punkt oder Lock-Freigabe — gehandelt (Act), kann der Zustand dazwischen von einer anderen Task verändert und ggf. zum ursprünglichen Wert zurückgeändert worden sein (ABA-Problem), oder eine gleichzeitige Änderung wird beim Commit stillschweigend überschrieben (Lost Update) bzw. ein bereits verändertes Objekt fälschlich als "unverändert, also löschbar" behandelt (Phantom Erasure — exakt der historische Tombstone-Masking-Bug: PUT nach Tombstone-Check wurde durch einen parallelen DELETE maskiert). Korrekt: Versions-/Sequenznummer beim Check lesen, beim Commit erneut vergleichen (Compare-And-Swap-Semantik) — kein Check-Then-Act ohne Versionsstempel-Revalidierung.

### APM-19: TxId als Wall-Clock-Ersatz
**Kurzbezeichnung:** `APM-19: TxId/Seq niemals aus SystemTime ableiten oder mit ihr vergleichen`

TxId/Sequenznummern sind eine LOGISCHE Zeitachse (nur der zentrale Allocator garantiert Monotonie); SystemTime/Wall-Clock ist eine PHYSIKALISCHE Zeitachse (nicht monoton bei NTP-Korrektur, Systemuhr-Änderung, Zeitzonenwechsel). Jede Vermischung beider Achsen — TxId aus SystemTime ableiten, TxId gegen SystemTime vergleichen, oder physische Timestamps als Ordnungskriterium für Commit-Reihenfolge nutzen — ist ein Bi-Temporale-Achsenverwechslung-Befund. Zusätzlich: der Allocator selbst MUSS bei hoher Nebenläufigkeit geprüft werden, dass er niemals doppelt vergibt oder rückwärts springt (TxId-Regression/Non-Monotonicity) — jeder neue Allocator-Pfad (Recovery, Multi-Shard, Crash-Restart) braucht einen expliziten Monotonie-Test über den Neustart hinweg.

### APM-20: Unbegrenzte Warteschlange ohne Cap
**Kurzbezeichnung:** `APM-20: Jede Queue/Channel braucht Kapazitätslimit + Backpressure`

Jede Queue/VecDeque/mpsc-Channel zwischen einem schnelleren Producer und einem langsameren Consumer braucht ein hartes Kapazitätslimit UND ein definiertes Backpressure-Verhalten (Reject/Block/Evict-Oldest) — analog zum bereits korrekt gelösten Muster MAX_TELEMETRY_EVENTS in AgentContext::events. Ohne Cap wächst der Speicherverbrauch unbegrenzt (OOM-Risiko unter Last). Ergänzend: ein unbegrenzt wartender Consumer/Producer ohne Fortschrittsgarantie unter Kontention (kein Deadlock im klassischen Sinn — keine zyklische Lock-Kette — aber trotzdem kein Fortschritt, z.B. durch permanent bevorzugte konkurrierende Tasks) ist Starvation und muss durch Fairness-Mechanismen (z.B. FIFO-Queueing, Timeout-basierte Priorisierung) ausgeschlossen werden.

### APM-21: Poisoning-Exposition bei std::sync
**Kurzbezeichnung:** `APM-21: std::sync::Mutex nach Panic dauerhaft vergiftet`

std::sync::Mutex/RwLock wird nach einem Panic innerhalb des kritischen Abschnitts dauerhaft "poisoned" — jeder nachfolgende .lock()-Aufruf liefert Err() und kann bei ungeprüfter Nutzung (.unwrap() auf das Lock-Ergebnis) einen kaskadierenden Folge-Panic in völlig unbeteiligtem Code auslösen. Zwei zulässige Strategien: (a) parking_lot::Mutex/RwLock verwenden (nicht-poisonable per Design — Projekt-Standard, siehe crate-weite Nutzung), oder (b) bei bewusster std::sync-Nutzung Poisoning explizit über .into_inner()/.unwrap_or_else(PoisonError::into_inner) mit dokumentierter Recovery-Strategie behandeln. Jede neue Mutex-/RwLock-Instanz ist ein Fund, wenn sie std::sync nutzt UND das Poisoning nicht explizit behandelt.

### APM-22: Score-Konfidenz ohne Kalibrierungsnachweis
**Kurzbezeichnung:** `APM-22: Confidence-Score ohne ECE-Test ist nicht validiert`

Jeder als "Confidence"/"Score" exponierte Wert, der Routing-, Eskalations- oder Rerank-Entscheidungen triggert, braucht einen dokumentierten Kalibrierungstest (Expected Calibration Error oder gleichwertig) — NaN/Inf-Filterung (APM-16) macht einen Score nur numerisch sicher, sagt aber nichts darüber aus, ob er die tatsächliche Erfolgswahrscheinlichkeit korrekt widerspiegelt. Ein unkalibrierter, aber überkonfidenter Score führt zu Oszillation zwischen Eskalationsstufen (ECE-Spikes) oder systematischem Over-/Under-Escalation. Symptom in der Praxis: ein plötzlicher Einbruch von Recall/Precision in der Vektorsuche (Recall Collapse) ist häufig ein Kalibrierungs-Symptom, kein reiner Index-Bug — bei Recall-Regressionen zuerst die Score-Kalibrierung des vorgeschalteten Routings/Rerankings prüfen, nicht nur den Index selbst.

### APM-23: Statische Verteilungsannahme ohne Drift-Signal
**Kurzbezeichnung:** `APM-23: Modelle/Heuristiken ohne Prüfpfad für Eingabe-Drift degradieren lautlos`

Modelle oder Heuristiken, die implizit auf einer Trainings-/Referenzverteilung basieren (Importance-Scoring, Compound-Splitter-Recall, Embedding-Qualität für neue Domänen), brauchen einen Prüfpfad für Covariate Shift — eine Verschiebung der Eingabeverteilung gegenüber der Referenz, die sich NICHT als Crash oder Fehler zeigt, sondern als schleichende Qualitätsdegradation ohne Alarm. Jede neue Heuristik/jedes neue Scoring-Modul MUSS entweder (a) einen expliziten Drift-Metrik-Test gegen ein Referenz-Set haben, oder (b) explizit als "kein Drift-Monitoring, bewusste Entscheidung, Grund: X" dokumentiert werden — stillschweigendes Fehlen ist der Befund.

### APM-24: Provenienzverlust bei Aggregation
**Kurzbezeichnung:** `APM-24: Fusion mehrerer Signalquellen ohne Herkunfts-Nachweis pro Teilergebnis`

Jede Fusion mehrerer Signalquellen (RRF über HNSW+BM25+Graph+Metadaten, Reranking, PPR-Community-Aggregation) muss die Herkunft und Gewichtung jedes Teilergebnisses nachvollziehbar mitführen (Provenance). Ein reiner Endscore ohne Quellzuordnung ("dieses Ergebnis kam aus Signal X mit Gewicht Y") ist ein Befund, kein akzeptables Implementierungsdetail — er macht Debugging von Ranking-Anomalien und spätere Kalibrierung (APM-22) unmöglich, weil der kausale Beitrag jeder Quelle nicht mehr rekonstruierbar ist, sobald die Fusion einmal durchlaufen ist.

### APM-25: Fixe Byte-/Index-Grenzen ohne Bounds-Check
**Kurzbezeichnung:** `APM-25: Direkter Slice-/Index-Zugriff ohne Bounds-Check = Panic/OOB`

Jeder direkte Slice-/Index-Zugriff (arr[i], &buf[a..b]) außerhalb der bereits durch APM-7 abgedeckten UTF-8-Sonderfälle braucht einen expliziten Bounds-Check oder einen .get()/.get(a..b)-Pfad mit kontrolliertem Err/None-Handling — insbesondere bei Längen-/Offset-Feldern, die aus Dateien, Netzwerk oder deserialisierten Strukturen (WAL-Records, SSTable-Header, FlatBuffers) gelesen wurden und daher potenziell korrupt oder adversarial sind. Ein ungeprüfter Index aus externer Quelle ist ein Buffer-Overflow/Underflow-Risiko, kein reiner Panic-Safety-Stilfehler.

### APM-26: Ressourcen-Handle ohne RAII-Nachweis
**Kurzbezeichnung:** `APM-26: Neuer Datei-/Socket-/Mmap-Pfad ohne Open/Close-Zyklustest`

Jede neu geöffnete Datei, jeder Socket, jede mmap-Region braucht einen expliziten Test, der Erstellung und Freigabe über N Zyklen zählt und beweist, dass kein Handle/File-Descriptor je Zyklus zurückbleibt (Referenzmuster: tests/resource_leaks.rs, 1.000 Open/Close-Zyklen in contextra-store). Dieses Muster gilt für JEDES Crate mit Datei-/Handle-I/O, nicht nur für den Storage-Layer — jeder neue I/O-Funktionspfad (SSTable-Open, WAL-Rotation, mmap-Remap, Socket-Accept) braucht diesen Nachweis bei seiner Einführung, nicht erst nachträglich als Audit-Befund.

### APM-27: Kompressions-/Verschachtelungs-Bomben & Event-Flooding ungeprüft
**Kurzbezeichnung:** `APM-27: Parser für Fremdformate ohne Verhältnis-/Tiefen-Limit + gedrosselte Events`

Jeder Parser für komprimierte/verschachtelte Fremdformate (ZIP/DOCX-Container, verschachteltes XML/Entity-Expansion, PDF-Objekt-Streams, E-Mail-MIME) braucht ZUSÄTZLICH zur bereits geprüften absoluten Dateigröße ein hartes Kompressions-Verhältnis-Limit (komprimiert:dekomprimiert) UND eine Verschachtelungstiefen-Grenze — eine kleine Eingabedatei kann bei extremer Entpackungsrate trotzdem den Speicher erschöpfen (Zip Bomb/Billion-Laughs), auch wenn die absolute Größe unauffällig ist. Ergänzend für IPC-Events an der MCP-stdio-Grenze: jeder pro-Dokument oder pro-Datei ausgelöste Fortschritts-/Status-Event (z.B. Ingestion vieler Dateien via contextra_insert-Batch) MUSS gebatcht oder gedrosselt werden — ungepufferte Events pro Einzeldatei überlasten die stdio-JSON-RPC-Schnittstelle (IPC Event Flooding). [v30: contextra-tauri (vormaliger primärer Anwendungsfall dieses Musters für PDF/DOCX/E-Mail-Container) wurde per ADR-077 physisch entfernt — dieses APM bleibt für contextra-mcp und künftige Fremdformat-Parser gültig.]

### APM-28: Dangling Edge nach inkonsistenter GC
**Kurzbezeichnung:** `APM-28: Graph-Kanten-Löschung muss an dieselbe Tombstone-Transaktion gebunden sein`

Graph-Kanten, die auf LSM-persistierte Dokument-IDs verweisen, dürfen niemals auf eine bereits per Tombstone gelöschte ID zeigen (Dangling Edge/Phantom Node). Die kaskadierende Löschung (Graph-GC bei Dokument-Löschung) MUSS an dieselbe Tombstone-Transaktion gebunden sein wie die Dokument-Löschung selbst — "best effort" oder asynchron nachgelagerte Graph-GC ist ein Befund, weil ein Crash zwischen Dokument-Tombstone und Graph-GC eine dauerhaft inkonsistente Kante hinterlässt. Jede neue remove_*-Funktion im Graph-Modul muss explizit dokumentieren, in welcher Transaktion die zugehörige Kanten-Bereinigung erfolgt.

### APM-29: Silent Fallback statt Fehlerpropagation
**Kurzbezeichnung:** `APM-29: .ok()/.unwrap_or_default() auf Result-Pfaden = stille Fehlerunterdrückung`

.ok(), .unwrap_or_default(), .unwrap_or_else(|_| ...) auf einem Pfad, der eigentlich ein Result mit ? weiterreichen sollte, ist Error Swallowing — ein Befund, keine Stilfrage, auch wenn das Projekt bereits eine "Zero-Panic"-Doktrin durchsetzt (die verhindert nur Panics, nicht stille Fehlerunterdrückung). Jeder Fallback-auf-Default nach einem fehlgeschlagenen Deserialisierungs-, I/O- oder Netzwerk-Aufruf muss explizit begründet werden ("Default ist hier fachlich korrekt, weil X") oder durch echte Fehlerpropagation ersetzt werden — ein unkommentierter Silent-Fallback in Produktionscode ist immer ein Fund.

### APM-30: God-Object-Verantwortungsdrift
**Kurzbezeichnung:** `APM-30: Zentraler Struct akkumuliert Felder ohne Layer-Zugehörigkeits-Prüfung`

Jeder zentrale, breit injizierte Typ (AppState, OrchestratorEngine, Facade-Structs) läuft über viele Sessions hinweg Gefahr, immer neue Felder/Methoden anzusammeln, die eigentlich in eine tiefere Schicht gehören (Layer/DAG-Verantwortungsdrift, God Object/Facade Bloat). Bei JEDER Erweiterung eines solchen zentralen Typs: explizit prüfen und im Review-Kommentar beantworten — "Ist dieses neue Feld/diese neue Methode originär Verantwortung dieses Layers, oder Symptom einer fehlenden Abstraktion in einer tieferen Schicht?" Eine unbeantwortete Erweiterung eines God-Object-Kandidaten ist ein Befund.

### APM-31: Kooperative Blockade ohne Fortschrittsgarantie
**Kurzbezeichnung:** `APM-31: Livelock/Starvation — kein Deadlock, aber trotzdem kein Fortschritt`

Komplementär zu APM-12 (Lock-Hierarchie-Inversion/Deadlock, zyklisches Warten): auch OHNE zyklische Lock-Kette kann ein System keinen Fortschritt machen. Livelock: zwei oder mehr Tasks reagieren wiederholt aufeinander (z.B. beide geben ein Lock frei und versuchen es sofort erneut) ohne dass eine von ihnen je fertig wird — Zustand ändert sich ständig, ohne dass sich etwas Sinnvolles bewegt. Starvation: eine Task wird dauerhaft von schnelleren/bevorzugten konkurrierenden Tasks verdrängt, obwohl kein Deadlock vorliegt. Jeder Retry-Loop mit Backoff/Jitter, jede Fairness-kritische Warteschlange (siehe auch APM-20) braucht eine explizite obere Schranke für Wartezeit/Retry-Anzahl UND einen Test, der Fortschritt unter simultaner Konkurrenz mehrerer Tasks nachweist — "es deadlockt nicht" ist kein Beweis für Fortschrittsgarantie.

### APM-32: Vertrauenskette ohne Validierungsbeweis
**Kurzbezeichnung:** `APM-32: Hash-/Signaturkette bei Wiederherstellung ungeprüft akzeptiert`

Contextra ist air-gapped ohne TLS/Zertifikatsketten — "Broken Chain of Trust" bedeutet hier NICHT Zertifikatsvalidierung, sondern die Validierungskette von Hash-/Signatur-Verkettungen bei (Teil-)Wiederherstellung: Checkpoint-Manifest-BLAKE3-Hashes, WAL-HMAC-Chain-Validierung nach einem Crash oder einer partiellen Datei-Wiederherstellung. Jede Wiederherstellungs-/Recovery-Funktion MUSS explizit nachweisen, dass sie eine unterbrochene oder manipulierte Kette (fehlender/falscher prev_hash, übersprungenes Segment) als Fehler erkennt UND ablehnt, statt die verbleibende Kette stillschweigend als vollständig gültig zu behandeln.

### APM-33: Fehlende Redundanz an kritischen Systemknoten
**Kurzbezeichnung:** `APM-33: Kritischer Singleton ohne Recovery-Pfad = SPOF`

Jeder architektonisch zentrale Singleton (z.B. OnceLock<Runtime> in contextra-py, ein einzelner Tokio-Runtime-Instanz-Pool) ist ein Single Point of Failure, wenn sein Ausfall/Panic den gesamten Prozess ohne kontrollierten Recovery-Pfad mitreißt. Kein Befund bedeutet nicht "keine Singletons" (manche sind architektonisch bewusst und korrekt, z.B. exakt 1 Runtime pro Prozess) — sondern dass für JEDEN solchen Knoten explizit dokumentiert ist, was bei seinem Ausfall passiert (kontrollierter Prozess-Exit mit Fehlermeldung vs. unklarer Hang vs. stiller Datenverlust) und dass dieses Verhalten getestet ist.

### APM-34: Zero-Copy-Lebensdauer über FFI-Grenze
**Kurzbezeichnung:** `APM-34: FlatBuffer/Bytes-Referenz überlebt Rust-Allokation nicht garantiert`

Zero-Copy-Rückgabepfade über die PyO3-Grenze (z.B. search_fb()/hybrid_search_fb(), die FlatBuffer-Bytes zurückgeben) müssen beweisen, dass die zugrunde liegende Rust-Allokation die GESAMTE Lebensdauer des von Python gehaltenen PyBytes-/Buffer-Objekts übersteht — kein Buffer-Reuse, kein Rückgeben eines Puffers an einen Pool, bevor Python das zugehörige Objekt fallen gelassen hat (sonst Use-After-Free, sichtbar als sporadische Datenkorruption erst unter GC-Druck in Python, nicht im Rust-Unit-Test). Jeder neue Zero-Copy-Rückgabepfad braucht einen expliziten Lebensdauer-Test unter Python-GC-Druck (gc.collect() nach Rückgabe, vor Zugriff).

### APM-35: Typ-Präzisionsverlust an FFI-Grenze
**Kurzbezeichnung:** `APM-35: u64→Python-Float statt Int = stille Präzisions-/Kollisionsgefahr`

Jede FFI-Typkonvertierung (Rust↔Python, Rust↔C-ABI) muss geprüft werden auf stillen Präzisionsverlust — z.B. Rust u64 nach Python float statt int konvertiert, was bei großen IDs (TxId, DocId-Hash-Anteile) zu Kollisionen führt, weil f64 nur 53 Bit exakte Mantisse hat. Jede neue #[pyfunction]/#[pymethods]-Signatur mit u64/i64/u128-Feldern braucht einen expliziten Test mit einem Wert > 2^53, der beweist, dass der Python-seitige Wert exakt (nicht gerundet) ankommt.

### APM-36: Vektor-Dimensionalität ungeprüft am Eingang
**Kurzbezeichnung:** `APM-36: Abweichende Embedding-Dimension in HNSW ohne API-Eingangs-Validierung`

Jeder Pfad, der einen Vektor in einen HNSW-/DiskANN-Index einfügt, muss die Dimension DIREKT am API-Eingang gegen die Index-Dimension validieren (z.B. 384 vs. 768 bei einem Embedder-Wechsel) — ein Dimensions-Mismatch, der erst tief in der Distanzberechnung als Speicherzugriffsfehler oder falsches Ergebnis auffällt, statt sofort am Eingang als kontrollierter ContextraError::Index abgewiesen zu werden, ist ein Befund. Jeder neue Embedder-Integrationspfad (Ollama, ONNX, zukünftige Provider) braucht diesen Eingangs-Check als ersten Schritt, nicht als nachgelagerte Fehlerbehandlung.

### APM-37: Bit-Rot ohne Checksummen-Beweis
**Kurzbezeichnung:** `APM-37: Gelesene Datenträger-Bytes ohne starke Checksumme als valide interpretiert`

Silent Data Corruption (Bit Rot) auf Datenträgern wird nur erkannt, wenn JEDE persistierte Struktur (SSTable-Block, WAL-Segment, Checkpoint-Manifest) beim Lesen gegen eine mitgespeicherte starke Checksumme (BLAKE3/CRC32, nicht nur die ohnehin vorhandene HMAC-Authentizitätsprüfung) verifiziert wird, BEVOR die Bytes als valide Daten interpretiert oder deserialisiert werden. Ein neuer Lesepfad ohne diese Verifikation vor der Deserialisierung ist ein Befund — auch wenn ein Bit-Flip in der Praxis selten ist, ist das Fehlerbild (stillschweigend falsche statt erkennbar korrupte Daten) besonders teuer.

### APM-38: Sequenz-/TxId-Bindung gegen Wiedereinspielung
**Kurzbezeichnung:** `APM-38: Alter, gültiger Log-Eintrag an neuer Position ohne kryptografische Bindung erkennbar`

Ein alter, ursprünglich gültiger WAL-/Checkpoint-Eintrag darf nicht an einer NEUEN Position akzeptiert werden (Replay Attack) — dazu müssen Sequenznummer und/oder TxId kryptografisch IN das HMAC/den Hash eingebunden sein (nicht nur der Payload), sodass ein Verschieben eines validen, alten Eintrags an eine andere Position die Verifikation fehlschlagen lässt. Jeder neue Wiederherstellungs-/Replay-Pfad (WAL-Replay, Checkpoint-Restore, MCP-stdio-Request-Handling) muss explizit dokumentieren, ob und wie er gegen Wiedereinspielung geschützt ist — "das Protokoll ist zustandslos-idempotent" ist nur dann eine gültige Begründung, wenn dies explizit geprüft und nicht nur angenommen wurde.

### APM-39: Amplifikations-Budget-Regression
**Kurzbezeichnung:** `APM-39: Write-/Read-/Space-Amplification ohne Regressions-Schranke`

Write-, Read- und Space-Amplification (Verhältnis von tatsächlichem I/O bzw. Speicherverbrauch zur logischen Datenmenge) sind LSM-Tree-inhärente Kennzahlen, kein Zufallsprodukt — jede Änderung an Compaction-Strategie, SSTable-Layout oder WAL-Rotation braucht eine Vorher/Nachher-Messung gegen einen dokumentierten Budget-Schwellenwert, nicht nur eine einmalige Benchmark-Erwähnung im Audit-Report. Eine Änderung, die eine der drei Amplifikationskennzahlen signifikant verschlechtert, ohne dass dies als bewusster Trade-off (mit Begründung) dokumentiert ist, ist ein Befund.

### APM-40: Premature-Abstraction ohne Empirie-Beleg
**Kurzbezeichnung:** `APM-40: Neue Komplexitätsschicht ohne gemessenen Performance-Gewinn`

Jede neu eingeführte Abstraktionsschicht, Cache-Ebene, generische Parametrisierung oder Optimierung, die mit Performance begründet wird, braucht einen VORHER-Benchmark-Beleg (nicht nur eine plausible Vermutung) — Komplexitätsaufbau ohne gemessenen Gewinn ist Premature Optimization und ein Wartungsrisiko ohne belegten Nutzen. Bei Review/Korrektur-Aufgaben: Frage explizit stellen — "Existiert ein Benchmark, der diese Optimierung rechtfertigt, oder ist sie eine unbelegte Annahme?"

### APM-41: Geteilter Zustand ohne Synchronisationsnachweis
**Kurzbezeichnung:** `APM-41: Neuer shared-mutable State ohne expliziten Race-Test`

Generisches Basismuster für Race Conditions/Data Races, komplementär zu den spezifischeren APM-12 (Lock-Reihenfolge), APM-18 (Check-Then-Act) und APM-21 (Poisoning): JEDER neu eingeführte geteilte veränderliche Zustand (Arc<Mutex<_>>, Arc<RwLock<_>>, static mit interior mutability, geteilter Zustand über mehrere MCP-Tool-Aufrufe/Sandbox-Sessions hinweg) braucht bei seiner Einführung einen dedizierten Concurrency-Test mit mindestens 2 parallelen Zugreifern, nicht nur eine grep-basierte Plausibilitätsprüfung der Lock-Nutzung — ein "sieht synchronisiert aus" ersetzt keinen tatsächlichen parallelen Test-Lauf.

### APM-42: Prüf-Tooling-Drift
**Kurzbezeichnung:** `APM-42: Prüf-Tooling/Doku behauptet Vollständigkeit ohne Frische-Nachweis`

Jedes Werkzeug, jede Dokumentation oder jedes Skript, das eine Vollständigkeits- oder Korrektheitsaussage über den Code trifft (Datei-Inventare, DAG-Diagramme, Coverage-Zahlen, "verifiziert"-Badges — eingeschlossen dieser Prompter selbst), muss selbst gegen die Quelle nachprüfbar sein: im Idealfall automatisch generiert, mindestens aber mit einem sichtbaren Stand-/Frische-Datum UND einem expliziten Reality-Check-Schritt versehen (siehe Schritt 0 in diesem Prompt). Eine hartkodierte oder von Hand gepflegte Aussage über den Code-Zustand ohne Mechanismus, sich selbst als veraltet zu erkennen, ist ein Befund — unabhängig davon, ob sie im Moment der Prüfung zufällig noch korrekt ist. Gilt für ARCHITECTURE.md-DAG-Diagramme, SOURCE_OF_TRUTH.md-Zahlen und jede sonstige "Stand: geprüft"-Behauptung im Repo gleichermaßen.

## 2. Crate-spezifische Pflicht-APMs & Fokus-Profile

### `contextra-core`
- **Titel:** Senior Rust Systems Engineer — Typ-System, MVCC, Concurrency
- **Layer:** Layer 0 — Fundament (alle anderen Crates abhängig)
- **Fokus & Invarianten:**
```text
KRIT. INVARIANTEN für contextra-core:
• ADR-028: System-TxId vs. Collection-TxId Range-Trennung — was passiert AN der Grenze?
• SnapshotRegistry: gepinnte Snapshots können nicht GC'd werden → Race zwischen Pin und GC?
• TxBuffer-Sharding: korrekte Lock-Granularität, kein Starvation bei hoher TX-Last
• ContextraError → error_dto.rs: JEDE Variante serialisierbar? Vollständigkeit prüfen
• #[deny/forbid(unsafe_code)] verletzt? → 0 unsafe außerhalb expliziter Ausnahmen
• DAG-Konformität: contextra-core darf KEINE contextra-*-Abhängigkeiten haben
```
- **Audit-Befehle:**
```bash
grep -rn "\\.unwrap()\\|\\.expect(" crates/contextra-core/src/ | grep -v "#\\[cfg(test)\\]"
grep -rn "unsafe" crates/contextra-core/src/ | grep -v "// SAFETY:"
grep -rn "^use\\|extern crate" crates/contextra-core/Cargo.toml
grep -n "TxId\\|MAX_COLLECTION_SEQUENCE\\|SYSTEM_TX" crates/contextra-core/src/ -r
grep -n "SnapshotRegistry\\|pin_snapshot\\|unpin" crates/contextra-core/src/ -r
```
- **Tiefen-Audit Extra:**
**TxId-Boundary-Exhaustion-Simulation:**
\

### `contextra-crypto`
- **Titel:** Senior Rust Applied-Cryptography-Ingenieur — AES-256-GCM-SIV, HKDF, HMAC, Zeroize
- **Layer:** Layer 1 — Encryption-at-Rest Fundament (einzige Crate mit Krypto-Primitiven)
- **Fokus & Invarianten:**
```text
KRIT. INVARIANTEN für contextra-crypto:
• RFC-Konformität: RFC 8452 (AES-256-GCM-SIV), RFC 5869 (HKDF), RFC 4231 (HMAC-SHA256), BLAKE3-Ref — Regression via rfc_vectors prüfen
• Nonce-Sicherheit: 12-Byte-Nonce = 4B Instanz-Präfix (OsRng) + 8B per-Call OsRng-Suffix, Kollisionswahrscheinlichkeit p≈2.71×10⁻⁸ bei 10⁶ Nonces
• Key-Domain-Separation: Encryption-Key ("contextra-aes-256-gcm-key"/"contextra-file-key-v1:<id>") und HMAC-Key ("contextra-hmac-sha256-key") MÜSSEN disjunkte Bytes liefern
• Constant-Time-Vergleich: subtle::ConstantTimeEq für JEDEN Tag-/HMAC-Vergleich — niemals == auf Secrets
• Zeroize: ZeroizeOnDrop für VolatileEncryptionKey, IntegrityVerifier und alle Key-Material-Structs lückenlos
• WAL-HMAC-Chaining v3 KRITISCH: Reihenfolge domain-separator → prev_hmac → seq_no → tx_id → op_type+kv MUSS exakt eingehalten werden
• Krypto-Isolation: KEIN anderes Crate darf eigene Ad-hoc-Crypto-Primitiven verwenden — ausschließlich contextra-crypto
• KV-Cache-Bridge (v30, siehe JULES_PROMPTS_KV_CRYPTO_GAP_2026-09-12.md): derive_kv_key() bindet an (TenantId, ModelFingerprint) — jeder Modell-/Quantisierungswechsel MUSS einen anderen Schlüssel erzeugen (P8-Fingerprint-Invalidierung). EvictionWorker läuft auf eigenem OS-Thread (nicht Tokio-Task) — synchrones Zeroizing darf den Async-Executor niemals blockieren
• cargo audit: RUSTSEC-Einträge bekannt in lopdf (contextra-text) und pyo3 (contextra-py) — contextra-crypto selbst muss isoliert bleiben
• OFFENER DESIGN-PLAN (v31, aus implementation_plan-kv2.md — NOCH NICHT genehmigt/umgesetzt; live gegen
  kv_segment/store.rs verifiziert: aktuell EIN globales \
```
- **Audit-Befehle:**
```bash
cargo test -p contextra-security -- rfc_vectors --nocapture
grep -n "fn encrypt_auto_nonce\\|OsRng\\|NONCE_SIZE" crates/contextra-crypto/src/*.rs
grep -n "contextra-aes-256\\|contextra-hmac\\|contextra-file-key" crates/contextra-crypto/src/*.rs
grep -n "ConstantTimeEq\\|ct_eq" crates/contextra-crypto/src/*.rs
grep -n "ZeroizeOnDrop\\|Zeroize" crates/contextra-crypto/src/*.rs
grep -n "compute_checksum_v3\\|verify_and_update_v3\\|WalHmac" crates/contextra-crypto/src/wal_crypto.rs
grep -n "derive_kv_key\\|derive_file_key" crates/contextra-crypto/src/crypto.rs
grep -rn "evict_lru_fair\\|EvictionWorker" crates/contextra-crypto/src/kv_segment/
cargo audit -p contextra-security 2>/dev/null | grep -E "RUSTSEC|Severity"
```
- **Tiefen-Audit Extra:**
**Nonce-Kollisions-Simulation (statistisch):**
\

### `contextra-store`
- **Titel:** Senior Rust Storage-Engine-Ingenieur — Crash-Safety, LSM-Korrektheit
- **Layer:** Layer 1 — Storage Engine (WAL, MemTable, SSTable, Compaction)
- **Fokus & Invarianten:**
```text
KRIT. INVARIANTEN für contextra-store:
• APM-1 KRITISCH: JEDE Datei-Erstellung prüfen — tmp→sync_all→rename→fsync_parent_dir
• fsync-Disziplin: sync_all() MUSS vor Ok-Rückgabe in WAL-append kommen
• APM-3: commit_mutex — rollback_to_tx darf NICHT innerhalb bereits gehaltenem Lock aufgerufen werden
• APM-6: load_or_create_integrity_key() korrekt → alle anderen load_or_create_X() gleich robust?
• std::fs in async: KEIN direktes std::fs::File außerhalb spawn_blocking (Executor-Starvation)
• Compaction-Tombstone: gepinnte Snapshots schützen Tombstones korrekt vor GC?
• WAL-HMAC-Binding: Sequenznummer + Datei-ID in HMAC eingebettet (nicht nur Payload)?
• STATUS v31 (implementiert, siehe PR #2378 "fix(store): replace dirty read with IntentLockManager in
  put_if_absent"): das früher als kritisch dokumentierte TOCTOU-Race in put_if_absent (siehe
  ANALYSEBERICHT_8TAGE.md Fall A) ist durch ein formalisiertes IntentLockManager-Pattern
  (intent_locks: Mutex<HashMap<Vec<u8>, TxId>> in LsmStorage) strukturell behoben — is_key_staged_globally
  wurde entfernt, Dirty-Reads über TxBuffer-Grenzen sind nicht mehr möglich. Bei jeder weiteren Änderung
  an put_if_absent/commit/rollback_to_tx: clear_intent_locks_for_tx()/clear_intent_locks_above_tx() MÜSSEN
  konsistent bei jedem Commit-/Rollback-/Abbruch-Pfad aufgerufen werden — ein vergessener Cleanup-Pfad
  wäre ein neuer Deadlock-/Starvation-Kandidat für zukünftige Schreiber auf denselben Key.
• OFFENE DESIGN-PLÄNE (v31, aus implementation_plan_wal.md/-wal2.md/-lsm.md/-lsm-2.md — NOCH NICHT
  genehmigt/umgesetzt, NUR als Kontext für künftige Planungs-Sessions, nicht als Ist-Zustand behandeln):
  (1) Actor-basierter WAL-Flusher (Entkopplung von CPU-bound Batch-Prep und I/O-bound fsync via
  mpsc-Channel statt tokio::sync::Mutex<File>), (2) Herauslösen von \
```
- **Audit-Befehle:**
```bash
grep -rn "tokio::fs::write\\|std::fs::write" crates/contextra-store/src/ | grep -v test
grep -rn "sync_all\\|sync_data" crates/contextra-store/src/wal.rs
grep -n "commit_mutex" crates/contextra-store/src/ -r | head -15
grep -rn "std::fs::File" crates/contextra-store/src/ | grep -v spawn_blocking | grep -v test
```
- **Tiefen-Audit Extra:**
**WAL Crash-Mid-Write Fault-Injection:**
\

### `contextra-vector`
- **Titel:** Senior Rust Performance-Engineer — HNSW, SIMD, SQ8, DiskANN
- **Layer:** Layer 1 — HNSW Vektor-Index (unsafe SIMD erlaubt)
- **Fokus & Invarianten:**
```text
KRIT. INVARIANTEN für contextra-vector:
• APM-2 KRITISCH: search_filtered() — Tombstone-Prüfung MUSS unconditional vor Custom-Filter:
  KORREKT:  if deleted.contains(idx) { continue; }   // IMMER
            if let Some(f) = filter { if !f(doc_id) { continue; } }  // DANN
  FALSCH:   if let Some(f) = filter { ... } else if deleted.contains(idx) { ... }
• APM-3 KRITISCH: SQ8-Bound-Expansion in do_insert() → try_write() = lastabhängige Präzisionsverluste
• SIMD: JEDER unsafe-Block muss direkt von is_x86_feature_detected!("avx2") umschlossen sein
• SIMD-Konsistenz: Euclidean im SIMD-Pfad und Scalar-Pfad identische Semantik (beide sqrt oder beide squared)
• mmap-TOCTOU: Datei verkürzt/gelöscht während aktivem mmap? → SIGBUS-Risiko
```
- **Audit-Befehle:**
```bash
grep -n "else if.*deleted\\|deleted.*else if\\|filter.*else if" crates/contextra-vector/src/hnsw.rs
grep -n "try_write\\|expand_bounds" crates/contextra-vector/src/hnsw.rs | head -10
grep -n "is_x86_feature_detected\\|is_aarch64" crates/contextra-vector/src/distance.rs
grep -n "unsafe {" crates/contextra-vector/src/distance.rs | head -20
```
- **Tiefen-Audit Extra:**
**HNSW Filter+Tombstone Concurrent Race:**
\

### `contextra-graph`
- **Titel:** Senior Rust Graph-Algorithmen-Ingenieur — CSR, PPR, BFS, Session-DAG
- **Layer:** Layer 1 — CSR-Wissensgraph + Session-DAG
- **Fokus & Invarianten:**
```text
KRIT. INVARIANTEN für contextra-graph:
• CSR-Invariante: offsets.len() == reverse_map.len() + 1 IMMER (auch bei entity-only-commits ohne Kanten!)
• compact()-Frühabbruch: entity-only-commits dürfen offsets-Padding NICHT überspringen
• Rollback-Vollständigkeit: rollback() MUSS id_map UND reverse_map bereinigen (nicht nur staged_edges)
• PPR Dangling Nodes: Score-Mass-Conservation → Summe aller PPR-Scores == 1.0 (Teleportation korrekt?)
• BFS-Hub: Branching-Limit PER LEVEL oder nur globales Ergebnis-Limit? Hub mit 10k+ Kanten → Queue-Explosion?
• Lock: kein Lock über .await-Punkt gehalten (minimale, methodenlokale Lock-Scopes)
```
- **Audit-Befehle:**
```bash
grep -n "compact\\|offsets\\|pending_edges" crates/contextra-graph/src/csr.rs | head -20
grep -n "rollback\\|id_map\\|reverse_map" crates/contextra-graph/src/csr.rs | head -15
grep -n "dangling\\|teleport\\|damping\\|sink" crates/contextra-graph/src/ppr.rs
grep -rn "\\.read()\\|\\.write()" crates/contextra-graph/src/ | grep -v test | head -20
```
- **Tiefen-Audit Extra:**
**CSR Rollback + compact() Sequenz (bekanntes Vorbefund-Thema):**
\

### `contextra-text`
- **Titel:** Senior Rust NLP-Engineer — BM25, Morphologie, UTF-8-Sicherheit
- **Layer:** Layer 1 — BM25 Volltext-Index + German Compound Splitter
- **Fokus & Invarianten:**
```text
KRIT. INVARIANTEN für contextra-text:
• APM-7 KRITISCH: Byte-Slicing auf Strings → deutsches 'ä' = 2 Bytes → Panic bei falschen Indizes
• Compound-Splitter: rekursiv ohne Memoization? O(2^n) für lange Wörter = DoS-Risiko
• GermanMorphTokenizer::new() → Trie bei JEDEM Aufruf neu aufgebaut? → OnceLock<Arc<Trie>> fehlt?
• BM25 IDF-Floor: ln(idf_arg) negativ bei df>N/2 → wie behandelt? 1e-6 statt 0 = Stoppwörter gefärbt
• InvertedIndex Update/Delete: DF-Zähler korrekt dekrementiert? Keine Ghost-Terms in Postings?
• #[forbid(unsafe_code)]: 0 unsafe-Vorkommen erlaubt
```
- **Audit-Befehle:**
```bash
grep -rn "\\[.*\\.\\..* \\]\\|\\.split_at\\(" crates/contextra-text/src/ | grep -v "test\\|//"
grep -n "OnceLock\\|lazy_static\\|once_cell" crates/contextra-text/src/tokenizer.rs
grep -n "idf\\|1e-6\\|floor\\|clamp" crates/contextra-text/src/bm25.rs
grep -n "recurse\\|backtrack\\|fn split" crates/contextra-text/src/morphology.rs
```
- **Tiefen-Audit Extra:**
**Compound-Splitter Pathological-Input-Stress (bekanntes Root-Cause-Thema):**
\

### `contextra-checkpoint`
- **Titel:** Senior Rust Transaktionssystem-Engineer — RAII, Time-Travel
- **Layer:** Layer 1 — Checkpoint-Registry + CheckpointGuard
- **Fokus & Invarianten:**
```text
KRIT. INVARIANTEN für contextra-checkpoint:
• APM-8 KRITISCH: spawn() in Drop-Handler = Fire-and-Forget = RAII-Garantie nicht durchgesetzt
• Panic-Unwind: Wenn Guard während Panic gedroppt wird → State danach konsistent?
• Pin-before-Save-Invariante (ADR-011): Snapshot gepinnt VOR Checkpoint gespeichert?
• Architektur-Abgrenzung: contextra-checkpoint darf contextra-store::checkpoint (pub(crate)) NICHT importieren
• Time-Travel: Rollback zu Checkpoint X → State exakt wie nach X, bytegenau?
```
- **Audit-Befehle:**
```bash
grep -n "spawn\\|fire\\|Drop\\|fn drop" crates/contextra-checkpoint/src/lib.rs | head -10
grep -n "pin\\|unpin\\|snapshot" crates/contextra-checkpoint/src/lib.rs | head -10
grep -rn "contextra_store::checkpoint" crates/contextra-checkpoint/ | head -5
```
- **Tiefen-Audit Extra:**
**Concurrent-Multi-Session-Isolation (100-Iterationen-Stress, bekanntes Härtungsziel):**
\

### `contextra-db`
- **Titel:** Senior Rust Datenbank-Architekt — 4-Signal-Fusion, 2PC, MVCC
- **Layer:** Layer 2 — Orchestrator & 4-Signal-Fusion (kritischster Layer)
- **Fokus & Invarianten:**
```text
KRIT. INVARIANTEN für contextra-db:
• APM-5 KRITISCH: score-Modifikation VOR RRF hat NULLEFFEKT (RRF nutzt nur Rang-Position)
  filter_or_boost() → score *= boost → dann in RRF → boost hat keinen Effekt → muss POST-RRF sein
• Cross-Signal-Snapshot-Isolation: alle 4 Signale (HNSW, BM25, CSR, Filter) gegen DENSELBEN seq/snapshot?
• Lock-Hierarchie: collections → embedder → insert_lock — IMMER in dieser Reihenfolge
• 2PC-Rollback: bei Fehler nach Signal N committed → alle N..4 Signale zurückgerollt?
• Zettelkasten-Zyklen: Relation A→B→A → Traversierung mit Visited-Set und Timeout?
• APM-2: Community-Filter + Tombstone → beide kumulativ (nicht if/else-if)
• RRF-Skalierung (bekannter Befund): weighted_reciprocal_rank_fusion() materialisiert und sortiert VOLLSTÄNDIG (O(U log U) Latenz, O(U) Speicher) — Sub-Engines deckeln über MAX_SEARCH_K=1000, die Fusionsfunktion selbst NICHT. Jede neue Aufruferstelle muss diese Grenze respektieren, sonst Speicher-/Latenz-Explosion bei großen Treffermengen
• Cross-Signal-Snapshot-Isolation (bekannte architektonische Lücke): LSM/BM25/Metadaten binden an seq_no, HNSW und CSR-Graph lesen unpinnten Live-State — neuer Code darf sich NICHT auf implizite Snapshot-Konsistenz von HNSW/Graph verlassen, nachgelagerte Filterung muss veraltete/gelöschte Treffer aktiv verwerfen
• Reranker-Rank-Hijacking (bei aktivem contextra-infer-onnx): pre_rerank_k = k*3 begrenzt den Kandidatenpool VOR Cross-Encoder-Reranking — jede Änderung an diesem Faktor beeinflusst direkt die Angriffsfläche für Keyword-Stuffing
• STATUS v31 (implementiert, verifiziert): der zuvor als offen/vermutlich-obsolet dokumentierte AI-TAG zu
  fusion.rs:45 (ResonanceConfig-NaN-Behauptung) ist korrekt auf RESOLVED aktualisiert — apply_resonance_bonus()
  validiert Scores lückenlos mit r.score.is_finite() vor dem Bonus, mit begleitendem Test
  test_apply_resonance_bonus_nan_safety. Ebenso RESOLVED: merge_metadata()-Typkonsistenz (audit-M-1),
  weighted_reciprocal_rank_fusion_with_options() NaN/Inf/≤0-Gewichtsfilterung (audit-H-4).
• OFFENE DESIGN-PLÄNE (v31, aus implementation_plan_db.md/-db2.md — NOCH NICHT genehmigt/umgesetzt, nur
  Kontext für künftige Planungs-Sessions): (1) "Late Hydration"/Two-Phase-Retrieval — RRF akkumuliert
  zunächst nur DocId+Score (FxHashMap/AHashMap statt SipHash), Metadaten-Merging via merge_metadata()
  und ProvenanceRecord-Bau NUR für die finalen Top-K statt für alle Kandidaten (adressiert direkt die
  oben dokumentierte RRF-Skalierungs-Lücke); (2) PidLatencyController in multistep.rs zur dynamischen
  Begrenzung von k_pool/max_hops unter P95-Latenzdruck, inkl. Backpressure-Kopplung an die
  contextra-infer-candle-Inferenz-Semaphore; (3) NodesGuard-typisierte Locks in consolidation_executor.rs für
  Deadlock-Freiheit beim kaskadierenden CSR-Kanten-Invalidieren. ACHTUNG bei Umsetzung: (1) ändert
  interne Signaturen (Referenzen statt Vec<SearchResult>-Klone) — vor Umsetzung Planer-Session mit
  Prüfung, dass Tie-Breaker- und NaN/Inf-Guard-Verhalten bit-identisch bleibt (Regressionstest Pflicht).
```
- **Audit-Befehle:**
```bash
grep -n "score\\|rrf\\|rank\\|weight" crates/contextra-db/src/fusion.rs | head -20
grep -n "snapshot\\|seq\\|pinned" crates/contextra-db/src/collection/search.rs | head -15
grep -n "collections.*lock\\|embedder.*lock" crates/contextra-db/src/ -r | head -10
grep -n "rollback\\|compensate" crates/contextra-db/src/transaction.rs | head -10
grep -n "visited\\|cycle\\|traverse" crates/contextra-db/src/collection/relate.rs | head -10
grep -n "MAX_SEARCH_K\\|pre_rerank_k" crates/contextra-db/src/ -r | head -10
grep -n "sort_by\\|Vec::sort" crates/contextra-db/src/fusion.rs
```
- **Tiefen-Audit Extra:**
**2PC Fault-Injection über ALLE 4 Sub-Engines (State-of-the-Art-Referenz aus vorherigem Audit):**
\

### `contextra-infer-onnx`
- **Titel:** Senior Rust ML-Integration-Engineer — ONNX, Feature-Gates
- **Layer:** Layer 3 — ONNX CrossEncoder Reranking (optional)
- **Fokus & Invarianten:**
```text
KRIT. INVARIANTEN für contextra-infer-onnx:
• Feature-Gate-Hermetizität: cargo check --no-default-features MUSS 0 Fehler ergeben
• Im Default-Build: 0 unsafe außerhalb #[cfg(feature="onnx")]
• spawn_blocking: ONNX-Inferenz blockiert → spawn_blocking Pflicht (kein GIL-artiger Stall)
• Reranker-Sortierbarkeit: Score-Ties → stabile Sortierung? Korrekte Richtung (höher = besser)?
• Batch-Determinismus: gleicher Text solo und im Batch → identischer Output?
```
- **Audit-Befehle:**
```bash
cargo check -p contextra-infer-onnx --no-default-features 2>&1 | grep -c "^error"
grep -rn "unsafe" crates/contextra-infer-onnx/src/ | grep -v "#\\[cfg(feature"
grep -n "spawn_blocking" crates/contextra-infer-onnx/src/ -r | head -5
```
- **Tiefen-Audit Extra:**
**Adversariales Reranker-Hijacking (bekannte, quantifizierte Schwachstelle):**
\

### `contextra-infer-ollama`
- **Titel:** Senior Rust HTTP-Client + Prompt-Engineer — Injection-Sicherheit
- **Layer:** Layer 3 — Ollama HTTP-Client + ContextPrefixEngine
- **Fokus & Invarianten:**
```text
KRIT. INVARIANTEN für contextra-infer-ollama:
• APM-7 Injection: xml_escape() deckt ALLE 5 XML-Sonderzeichen: < > & " ' ?
• HTTP-Timeout: kein Timeout → hängt ewig → konfigurierter Timeout Pflicht
• Prompt-Injection: build_rag_prompt() — Nutzerinhalt klar vom Prompt-Gerüst getrennt?
• Doppel-Escaping: &amp; nochmal escaped → &amp;amp; → fehlerhafter Output
• OllamaEmbedder: Vektor-Dimension konsistent über alle Aufrufe?
```
- **Audit-Befehle:**
```bash
grep -n "xml_escape\\|escape\\|sanitize" crates/contextra-infer-ollama/src/client.rs | head -10
grep -n "timeout\\|connect_timeout" crates/contextra-infer-ollama/src/client.rs | head -5
grep -n "build_rag_prompt\\|xml_escape" crates/contextra-infer-ollama/src/ -r | head -10
```
- **Tiefen-Audit Extra:**
**Prompt-Injection-Evasion-Matrix (15 bekannte Umgehungstechniken gegen naive Denylists):**
\

### `contextra-agent`
- **Titel:** Senior Rust Workflow-Engineer — State-Machine, Exactly-Once, Budget
- **Layer:** Layer 3 — Agent Workflow Engine (checkpoint → execute → commit → audit)
- **Fokus & Invarianten:**
```text
KRIT. INVARIANTEN für contextra-agent:
• APM-4 KRITISCH: Budget-Check VOR tool.execute() — nicht danach!
  FALSCH: tool.execute() → budget.consume() → check → Tool hat bereits Seiteneffekte
  KORREKT: check → tool.execute() → budget.consume()
• Checkpoint-Restore: replay_from() muss Budget-State wiederherstellen (nicht nur Node/Step/Memory)
• Performance-Datenstruktur: events: Vec mit remove(0) = O(N) — bei häufigem Front-Dequeue VecDeque verwenden
• Identifier-Ambiguität: Knotenname "1" wird als Schrittnummer geparst
• Exactly-Once: Crash zwischen execute und commit → nach Recovery weder doppelt noch gar nicht
• Audit-Log: keine öffentliche API zum Ändern/Löschen von Einträgen
```
- **Audit-Befehle:**
```bash
grep -n "budget\\|consume\\|available\\|execute" crates/contextra-agent/src/engine.rs | head -20
grep -n "replay_from\\|restore\\|budget" crates/contextra-agent/src/engine.rs | head -10
grep -n "events\\|remove(0)\\|pop_front\\|VecDeque" crates/contextra-agent/src/context.rs | head -10
```
- **Tiefen-Audit Extra:**
**Budget-RMW-Race unter (zukünftiger) Parallelisierung (bekanntes Gefahrenpotenzial):**
\

### `contextra-py`
- **Titel:** Senior Rust FFI-Engineer — PyO3, GIL, Zero-Panic-Boundary
- **Layer:** Layer 3 — Python PyO3 Bindings
- **Fokus & Invarianten:**
```text
KRIT. INVARIANTEN für contextra-py:
• Zero-Panic-FFI: JEDER Panic in PyO3-Methoden → Python-Prozess crasht (Segfault-artig)
  → 0 .unwrap()/.expect() in Nicht-Test-Code
• GIL-Freigabe: block_on() OHNE py.allow_threads() → blockiert Python-Interpreter
• OnceLock-Runtime: 2 Python-Threads initialisieren gleichzeitig → exakt 1 Runtime?
• ContextraError→PyErr: ALLE Varianten gemappt? Fehlende = Python crasht statt Exception
• #[forbid(unsafe_code)]: 0 direktes unsafe im Crate-Code (PyO3 abstrahiert FFI)
```
- **Audit-Befehle:**
```bash
grep -rn "\\.unwrap()\\|\\.expect(" crates/contextra-py/src/ | grep -v "#\\[cfg(test)\\]"
grep -n "allow_threads\\|block_on\\|GIL" crates/contextra-py/src/lib.rs | head -10
grep -n "OnceLock\\|get_or_init\\|RUNTIME" crates/contextra-py/src/lib.rs | head -5
cargo clippy --manifest-path crates/contextra-py/Cargo.toml --all-targets -- -D warnings
```
- **Tiefen-Audit Extra:**
**Sub-Interpreter-Isolation (Python 3.12+/3.13, bekannt verifiziert):**
\

### `contextra-router`
- **Titel:** Senior Rust Routing-Engineer — 100% Branch-Coverage, NaN-Safety
- **Layer:** Layer 3 — SLM-Routing Engine (klein & überschaubar → 100% Branch-Coverage Pflicht)
- **Fokus & Invarianten:**
```text
KRIT. INVARIANTEN für contextra-router:
• NaN-Validierung: SlmProfile::new() akzeptiert NaN → score >= NaN = immer false → kein Routing
  Fix: try_new() mit is_finite() && >= 0.0 Check
• NaN-Propagation: f32::max(0.0, NaN) = NaN → Routing nicht-deterministisch
• DAG: contextra-router (Layer 3) darf contextra-mcp (Layer 4) NICHT importieren
• 100% Branch-Coverage: bei dieser Crate-Größe realistisch und Pflicht (cargo llvm-cov --branch)
• Routing-Determinismus: identische Inputs → identisches Ergebnis (100 Iterationen)
```
- **Audit-Befehle:**
```bash
grep -n "min_relevance\\|f32\\|is_finite\\|is_nan\\|SlmProfile" crates/contextra-router/src/ -r | head -10
grep -n "f32::max\\|partial_cmp\\|NaN" crates/contextra-router/src/router.rs | head -5
grep -rn "contextra_mcp\\|contextra-mcp" crates/contextra-router/Cargo.toml
command -v cargo-llvm-cov &>/dev/null || cargo install cargo-llvm-cov --quiet 2>&1 | tail -5
cargo llvm-cov -p contextra-router --all-features --branch 2>&1 | grep -E "TOTAL|Branch"
```
- **Tiefen-Audit Extra:**
**Hot-Reload-Konsistenz unter aktiven parallelen Routing-Calls (bekanntes Härtungsziel):**
\

### `contextra-mcp`
- **Titel:** Senior Rust Protocol-Engineer — stdio JSON-RPC, Sandbox, DoS-Schutz
- **Layer:** Layer 4 — MCP Server (Hauptangriffsfläche für externe Inputs)
- **Fokus & Invarianten:**
```text
KRIT. INVARIANTEN für contextra-mcp:
• ADR-010: AUSSCHLIESSLICH stdio — kein axum/TcpListener/HTTP → jeder REST-Rest = KRITISCH
• Grenzwerte hart: MAX_RPC_BYTES=16MB, MAX_SEARCH_QUERY=64KB → Test an n-1/n/n+1 Bytes
• Slowloris: Inaktivitäts-Timeout für sehr langsame Clients → sonst Ressourcen-DoS
• Binär-Input: nicht-UTF8 auf stdin → kontrollierter Err, kein Panic
• Informations-Leck: interne Pfade/Stacktraces nicht in Fehlermeldungen an Client
• JSON-RPC 2.0: id-Handling, error-Codes, Notification-Semantik korrekt?
• STATUS v31 (implementiert): der zuvor als MAJOR/offen dokumentierte Homoglyphen-Bypass
  (kyrillisch/griechisch, z.B. 'о'→'o', 'і'→'i') in PromptInjectionGuard::normalize_text() ist behoben
  — skeletonize_char() wird jetzt VOR der lowercase-Stufe angewendet (AI-TAG als RESOLVED geführt).
  Bei jeder Änderung an normalize_text()/skeletonize_char(): PFLICHT-Regressionstest mit mindestens
  den historisch bekannten Bypass-Zeichen ('о','і','е' kyrillisch, 'Α' griechisch) gegen ein bekanntes
  Injection-Payload — ein Wiederauftreten dieser exakten Fehlerklasse ist ein kritischer Rückschritt.
```
- **Audit-Befehle:**
```bash
grep -rn "axum\\|TcpListener\\|HttpServer\\|bind(" crates/contextra-mcp/src/ | grep -v test
grep -n "MAX_RPC_BYTES\\|MAX_SEARCH_QUERY\\|16.*1024\\|64.*1024" crates/contextra-mcp/src/ -r | head -5
grep -n "timeout\\|deadline\\|inactivity" crates/contextra-mcp/src/ -r | head -5
```
- **Tiefen-Audit Extra:**
**Slowloris-artiger Stdio-Angriff (bekannt: kein Inaktivitäts-Timeout vorhanden):**
\

### `contextra-calibration`
- **Titel:** Senior Rust Performance-Engineer — Score-Kalibrierung & ECE-Metriken
- **Layer:** Layer 1 — Calibration & Uncertainty-Quantification
- **Fokus & Invarianten:**
```text
KRIT. INVARIANTEN für contextra-calibration:
• ECE-Metrik (Expected Calibration Error): Binning & Confidence-Fitting
• Confidence-Drift: Erkennung von Over-Confidence / Under-Confidence
• Pure-Rust: keine externen C-Bindings
```
- **Audit-Befehle:**
```bash
grep -rn "\.unwrap()\|\.expect(" crates/contextra-calibration/src/ | grep -v "#\[cfg(test)\]"
grep -rn "ece\|confidence\|bin" crates/contextra-calibration/src/
```
- **Tiefen-Audit Extra:**
cargo test -p contextra-calibration --all-features -- --nocapture

### `contextra-infer-candle`
- **Titel:** Senior Rust ML-Engineer — Candle Inferenz, GGUF Quantisierung & KV-Cache-Bridge
- **Layer:** Layer 3 — Pure-Rust ML-Inferenz-Backend (technisches Fundament des Air-Gap-Modus und der KV-Cache-Bridge)
- **Fokus & Invarianten:**
```text
KRIT. INVARIANTEN für contextra-infer-candle:
• Pure-Rust ML: Inferenz über Candle, kein Libtorch/C++ Dependency-Leck (Air-Gap-Versprechen)
• Tensor-Quantisierung: GGUF Q4_K_M/Q8_0/F16-Handling in gguf_loader.rs/model_registry.rs
• Blocking-Inferenz: Inferenz-Pfade MÜSSEN über spawn_blocking laufen, mit Arc<Semaphore> (max_concurrent_inferences) als Backpressure-Vertrag (identisches Muster zu contextra-infer-onnx)
• ModelFingerprint (kanonisch aus contextra-core, NICHT hier neu definieren): compute_fingerprint() in model_registry.rs MUSS deterministisch und über Prozessneustarts stabil sein (SHA-256 über Gewichtsdatei + Quantisierung)
• KV-Cache-Bridge (Feature kv-bridge, v30 NEU): KvBridgeAdapter in kv_bridge.rs konsultiert contextra-crypto::TenantIsolatedKvStore je Retrieval-Chunk (chunk_id, ModelFingerprint, rope_offset). FAIL-OPEN PFLICHT: Cache-Miss oder JEDER Krypto-/Store-Fehler → transparenter Fallback auf vollständigen Prefill, NIEMALS Request-Fehler
• GASP-Grounding (gasp.rs): GaspConfig/DEFAULT_GROUNDING_THRESHOLD — Score unter Threshold MUSS explizites GroundingAssessment mit Warnstatus liefern, nie stillschweigend als "grounded" durchreichen
• swap_model() Hot-Reload darf laufende Inferenz nicht in inkonsistenten Tensor-Zustand versetzen
• generate_with_context() OHNE kv-bridge-Feature: reine Text-Konkatenation, funktional identisch zu generate() — Degradation muss transparent sein
```
- **Audit-Befehle:**
```bash
grep -rn "\\.unwrap()\\|\\.expect(" crates/contextra-infer-candle/src/ | grep -v "#\\[cfg(test)\\]"
grep -rn "spawn_blocking\\|max_concurrent_inferences\\|Semaphore" crates/contextra-infer-candle/src/
grep -rn "compute_fingerprint\\|ModelFingerprint" crates/contextra-infer-candle/src/model_registry.rs
grep -rn "KvBridgeAdapter\\|kv_bridge\\|rope_offset" crates/contextra-infer-candle/src/kv_bridge.rs
grep -rn "GaspConfig\\|DEFAULT_GROUNDING_THRESHOLD\\|GroundingAssessment" crates/contextra-infer-candle/src/gasp.rs
grep -rn "generate_with_context\\|LlmTextGeneratorStreaming" crates/contextra-infer-candle/src/inference.rs
cargo tree -p contextra-infer-candle | grep -i "libtorch\\|torch-sys\\|cxx" # MUSS leer sein — Pure-Rust-Versprechen
```
- **Tiefen-Audit Extra:**
cargo test -p contextra-infer-candle --all-features -- --nocapture
\

### `contextra-bench`
- **Titel:** Senior Rust Benchmark-Engineer — Retrieval-Accuracy-Regression
- **Layer:** Layer 4 — Benchmark-Harness (benchmarks/contextra-bench, KEIN Produktionscode im DAG-Sinn)
- **Fokus & Invarianten:**
```text
KRIT. PUNKTE für contextra-bench:
• Reproduzierbarkeit: fixer Seed/Corpus pro Lauf — Ergebnis MUSS zwischen Läufen ohne Code-Änderung stabil sein
• Baseline-Vergleich (compare.rs / bin/compare_baseline.rs): Regressions-Schwellwert dokumentiert, nicht willkürlich
• long_mem_eval.rs (1350 LOC, größte Datei) und main.rs (1332 LOC) sind NICHT tiefenauditiert — vor Änderung vollständige Lektüre Pflicht (Größenklasse L)
• Kein Produktionscode: Befunde hier fließen NICHT automatisch in Tier-1-Sorgfaltspflicht anderer Crates ein, außer sie decken einen echten Recall-Regressions-Fehler in contextra-db/contextra-vector auf
```
- **Audit-Befehle:**
```bash
cargo check -p contextra-bench --all-targets
find benchmarks/contextra-bench/src -name "*.rs" | xargs wc -l | sort -rn
```
- **Tiefen-Audit Extra:**
cargo run -p contextra-bench --release -- --help
# Regressionslauf ggü. letztem committeten Baseline-Ergebnis (siehe benchmarks/results/):
cargo run -p contextra-bench --release --bin compare_baseline
