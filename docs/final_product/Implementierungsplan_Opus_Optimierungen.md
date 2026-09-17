# Implementierungsplan: Optimierungen aus den Claude-Opus-Analysen

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
