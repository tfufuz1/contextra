# Contextra – Gesamtspezifikation 2.3 (eigenständige Fassung, Zielzustand v1.0)

> **Fassung 2.3 (Entwurf, Löschpfad überarbeitet):** Auf Basis von 2.2.1. **§12 ist vollständig durch die überarbeitete Löschspezifikation ersetzt** (zehn Löschschichten, Wirkungsklassen L/P/C*/C1–C3/M, Schlüsselvernichtung als Ebene B, Destroy-Protokoll, Epoch Seal, Zwei-Spuren-Backup, Berichtsformat V5). Alle davon abhängigen Stellen in §0 bis §11 und §13 bis §26 sind angepasst; die Änderungen stehen in §26.5. Rechtsfragen bleiben in §25.1 offen.

```
Dokument:  Normative, eigenständige Gesamtspezifikation, Implementierungsspezifikation und Meta-Plan
Fassung:   2.3 (Entwurf, baut auf 2.2.1 auf)
Stand:     26.10.2026
Status:    Entwurf zur Freigabe
Ablage:    docs/SPEC.md (nach Freigabe)
Ersetzt:   die Fassungen 1.3, 2.0 und 2.1 vollständig. Alle früheren Fassungen sind obsolet und DÜRFEN NICHT mehr herangezogen werden.
Zielgruppe: Menschen und LLM-Agenten, die Contextra sofort und ohne weitere Unterlagen verstehen, bauen und prüfen müssen.
```

> **Selbständigkeit.** Dieses Dokument ist die **einzige Quelle der Wahrheit** für Architektur, Verhalten, Konstanten, Tests, Gates und Plan von Contextra v1.0. Es verweist auf **kein** anderes Dokument. Jedes `§x` in diesem Text meint einen Abschnitt **dieses** Dokuments. Dateien des Repositorys (z. B. `capabilities.toml`) werden nur als **Artefakte genannt, deren Inhalt oder Prüfregel hier festgelegt ist**; sie sind nie Voraussetzung, um diesen Text zu verstehen. Fehlt ein Detail, gilt: **nicht raten**, Fail-Closed-Verhalten wählen (INV-FAILCLOSED-1) und den Punkt als offenen Punkt (§25) melden.

---

## 0. LLM-Schnellstart

### 0.1 Was Contextra ist

Contextra ist ein **lokaler, eingebetteter Wissens- und Gedächtnisspeicher in Rust** (Single-Node, ein Prozess hält ein Datenverzeichnis). Er **speichert** Dokumente (Schlüssel-Wert-Speicher mit einem Write-Ahead-Log als einziger Wahrheit), **findet** sie hybrid (Vektorsuche HNSW + Volltext BM25, Fusion per RRF, mandantengetrennt, Rechteprüfung vor der Distanzberechnung) und **löscht gemessen** (physische Bereinigung über sechs Datenschichten, Messung am Speicher, Löschbericht mit ausgewiesenen Grenzen). Zugänge: Rust-API (`contextra`, `contextra-db`) und MCP-Server (stdio) mit genau drei Agentenwerkzeugen. Zielgruppe des Produkts (Hypothese, §25): Berufsgeheimnisträger (Kanzleien, Steuerbüros, Praxen); KI-Agenten sind ein zweiter Zugang zum selben Kern.

### 0.2 Die zwölf Regeln für jede Änderung

1. **Ring und Klasse zuerst.** Jede Kante zeigt auf gleichen oder niedrigeren Ring (§4.1); Kern `[K]` hängt nie von Rand `[R]`/Experimentell `[E]`/Archiv `[A]` ab (P30).
2. **Kein Panic in Produktion.** Kein `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, `assert!`, `process::exit`, ungeprüftes Indexieren (P7). Fehler sind typisiert (`ContextraError`, §13.6).
3. **Zeit, Zufall, IDs nur über Ports** `Clock`, `Rng`, `IdGen` (P28). Verboten in Kernlogik: `Instant::now`, `SystemTime::now`, `thread_rng`, `std::env::var`, `HashMap/HashSet` mit RandomState. Ausnahme: Schlüssel-/Salt-Material aus dem System-RNG.
4. **Alles Persistente geht durch das WAL** (P24). Sicherheitszustand liegt ausschließlich in den fünf Sidecar-Logs (§7.9; ergänzend der Keystore `MFKS`).
5. **I/O nur über `Vfs`**; Commit = durabler Flush mit plattformgerechter Primitive (§7.7); `fsync`-Fehler werden **propagiert**, nie mit `let _ =` verworfen.
6. **Unsafe nur in drei Inseln** (`simd`, `sys`, `wire`), jeder Block mit `// SAFETY:` (P2).
7. **Mandant kommt aus dem `RequestContext`**, nie aus Literalen, Defaults oder Nutzerparametern (INV-TENANT-1, VETO-F10).
8. **Sicherheit ist kein Feature-Schalter.** Verschlüsselung, Messung und Löschbericht sind in jedem Build aktiv; kein `cfg(not(feature))`-Ersatz für Sicherheitscode (A1).
9. **Messen vor Behaupten.** Ein Restbestand-Zähler entsteht nur aus einem Scanner der besitzenden Schicht; Konstanten oder Aufrufer-Werte sind nie Messung (§12.7, §12.10).
10. **Fail-Closed.** Unbekannter Zustand, unbekannte Formatversion, fehlender Sicherheitsport, gebrochene Kette: Ablehnung mit benanntem Grund, nie Raten (INV-FAILCLOSED-1).
11. **Eine Typfamilie je Konzept.** Löschtypen existieren nur in `crypto`/`ports`; Ports nur in `ports` (A2, §5).
12. **Ein PR = ein Arbeitspaket (WP) = höchstens ein Crate-Schnitt**; nach jedem Schnitt Build, Löschtest, Zeilenzählung (REQ-CHG-2).

### 0.3 Karte: Konzept → Crate → Modul

|Konzept|Crate (Ring)|Modul / Ort|
|---|---|---|
|IDs, Fehlerklassen, `RequestContext`, `LayerWitness`-Grundtypen|`types` (0)|–|
|Alle Traits (Ports)|`ports` (0)|§5|
|Snapshots, SSI, Sequenzen|`mvcc` (0)|§8|
|AEAD, KDF, Signatur, `KmsProvider`-Impl, Löschbericht, AuditChain, RevocationLog|`crypto` (0)|§12|
|HNSW, SQ8, ACL-Bitset|`vector` (0)|§9|
|BM25, Analyzer, Block-Max-WAND|`text` (0)|§10|
|RRF-Fusion, Normalisierung, Provenienz/Explain|`rank` (0, nur `fusion/`)|§11.5|
|SIMD-Kernel / OS-Primitive / Wire-Layout|`simd` / `sys` / `wire` (0, Unsafe-Inseln)|§4.6|
|WAL, Memtable, SSTable, Manifest, Kompaktion, `Vfs`, Journal|`store` (1)|§7|
|Interne Checkpoints, Restore-Guard|`checkpoint` (1)|§12.9, §12.17, §13.8|
|Planner, Chunking, Löschsaga, Epoch Seal, Lesesperre, Startsequenz|`engine` (3)|§11, §12.8, §12.2.4|
|Collections, Transaktionen, Embedded-API|`db` (3)|§13.9|
|MCP-Server, IPC-Proxy, CLI|`mcp` (4)|§13|
|Composition Root, öffentliche Rust-API|`contextra` (4)|§13.4|

### 0.4 Die zehn wichtigsten Begriffe

**Scope**: (Tenant, Collection) – Einheit von Schlüssel und Index. **Tenant/Mandant**: Isolationsgrenze (VETO-F10). **Akte**: in der Produkt-API eine Collection (Annahme O-14). **Saga**: durable Zustandsfolge einer Löschung (§12.8). **Schicht (`DeletionLayer`)**: Ort, an dem Nutzdaten liegen können; acht Typvarianten, sechs gemessen (§12.4). **Witness (`LayerWitness`)**: Messbeleg einer Schicht, nur von deren Scanner erzeugbar. **Löschbericht**: signierter Report aus Messungen (§12.7). **Lesesperre / `ErasedSet`**: ab `Frozen` liefert kein Lesepfad Daten des Scopes (§12.9). **Sidecar-Log**: append-only Sicherheitslog neben dem WAL (§7.9). **Ratchet**: Messwert darf nur besser werden (REQ-CHG-4).

### 0.5 Was es in v1.0 nicht gibt

Graph-Reasoning, Kognition/Konsolidierung, Router/Agentenlaufzeit, Online-Adaption, KV-Cache, DiskANN, Echtzeit-Audio, verteilter Betrieb, Löschbeweis als Rechtsnachweis, ein Referenz-Prüfer als Produktbestandteil (nur Testorakel), Schlüsselverwahrung K2/K3 als ausgelieferte Stufe (Port vorhanden, aktiv erst nach bestandener Konformitätssuite, §12.2.3), Python-Wheels, Admin-Werkzeuge außer `doctor`/Status, Windows für Server und MCP. Diese Fähigkeiten sind entweder archiviert (Anhang B, nicht normativ) oder zurückgestellt (§2.3). Wer sie benötigt, braucht einen ADR (§18.6).

### 0.6 Vorrang- und Lückenregeln

1. Normativ sind Sätze mit **MUSS, DARF NICHT, SOLL, KANN**; Sätze ohne Schlüsselwort beschreiben nur.
2. Bei Widerspruch gilt: **speziellerer Abschnitt vor allgemeinerem**; Anforderungstexte in den Fachabschnitten vor dem Katalog (§19); Invarianten (§3.4) und Vetos (§3.1) vor allem anderen.
3. Zahlen mit „Startwert“ sind kalibrierbar (§15); Zahlen mit „fest“ nicht.
4. Was als **Annahme**, **Hypothese**, **nicht gemessen** oder **offen** gekennzeichnet ist, ist keine Zusage; es steht in §25.
5. Fehlt eine Angabe: Fail-Closed, melden, nicht erfinden.

---

## 1. Geltung, Belegbasis, Konventionen

### 1.1 Geltung

Dieses Dokument beschreibt den **Zielzustand v1.0** und den **Weg dorthin** (§22). Es ersetzt die Vorgängerfassungen, deren verwertbarer Inhalt vollständig übernommen oder ausdrücklich gestrichen ist (§26.2). Nach Freigabe liegt es unter `docs/spec/SPEC.md`; die Vorgängerfassungen wandern in das Archiv-Tag `archiv/v0` (§18.5).

### 1.2 Belegbasis des Ist-Standes

|Quelle|Stand|Stufe|
|---|---|---|
|Repo-Klon, HEAD `922f4098` (Merge PR #5287)|25.10.2026|gelesen|
|Maschinelle Zählung und Messungen, Commit `db68bdf74`, 125.348 Code-Zeilen gesamt (71.898 Kernprodukt); Spikes E1/E2|Einzelläufe|gemessen (einmalig)|
|Heuristik-Scan (automatischer Prüf-Scanner), Lauf auf Commit `d11ea8cbd`, 42 Crates, 184 Kanten|älter als HEAD|Heuristik: liefert Prüfaufträge, keine Beweise|
|Erneuter Klon, HEAD `8c6b470` (Merge PR #5298), 33 Crates|26.10.2026|gelesen (Stichproben: Stub, Mandanten-Hooks, Löschtests, Features, `unsafe`, Mock-Typen)|
|Funktionsinventar, HEAD `8c6b470`, 5.034 Funktionen|maschinell|ausgewertet; Zeilen = Funktionsrümpfe inkl. Tests, **nicht** mit Code-Zeilen vergleichbar|
|Repo-Konfiguration und Regeln: `capabilities.toml`, `governance/`, `rules/`, Vetos-, Verfassungs- und Agentendatei, ADR-077|HEAD|gelesen|

**Nicht geprüft:** Kompilierung, Testsuite, Clippy, Miri, CI-Stand, Windows, fjall/redb, Ausführung der vorhandenen Löschtests auf HEAD `8c6b470`. Befunde aus Heuristik-Scans (Stufe „Heuristik“) sind in §21 als solche markiert und werden in WP 0.2 neu gemessen.

### 1.3 Zählweise

Zeilenbudgets sind **Code-Zeilen** (ohne Leer-/Kommentarzeilen, ohne Tests) aus dem Werkzeug `loc.py`. Rohzeilen (`wc -l`) und Funktionsinventar-Werte dürfen nicht damit gemischt werden. Das Gate `loc-budget` pinnt genau ein Zählwerkzeug.

### 1.4 Konventionen

- **Schlüsselwörter:** MUSS, DARF NICHT, SOLL, KANN sind normativ (§0.6).
- **Normativitätsklassen** (je Crate/Abschnitt): **`[K]` Kern** gilt uneingeschränkt für v1.0. **`[R]` Rand** gilt nur bei aktiviertem Feature (Release-Default aus) und DARF Kern-Garantien nie schwächen. **`[E]` Experimentell** ist nicht normativ. **`[A]` Archiv** ist aus dem Workspace entfernt (Tag `archiv/v0`), nicht normativ (Anhang B).
- **ID-Schemata:** `INV-*` Invariante (§3.4), `P*` Prinzip, `A*` Architekturregel, `EP*` Engineering-Prinzip, `VETO-*`, `REQ-*` Anforderung (§19), `E*` Entscheidung (§20), `U-*` Ist-Befund (§21), `O-*` offener Punkt (§25), `P-*` rechtlicher Prüfpunkt (§25.2), `T-*` Pflichttest (§17.5), `AC-*` Abnahmekriterium (§12.12), `WP x.y` Arbeitspaket (§22), `G*` Produktgarantie (§2.4), `RB*` Robustheitsgarantie (§2.5). **IDs sind stabil; entfallene IDs werden nicht neu vergeben.**
- **Mechanismen-Kürzel:** **T** Typsystem, **G** Gate, **P** Property-/Modelltest, **I** Integrationstest über die Fassade, **C** Chaos, **F** Fuzz, **L** loom, **B** Benchmark.
- **Evidenzstufen:** **E0** Behauptung · **E1** am Code gelesen · **E2** Gate-/Skriptlauf auf bekanntem Commit, Protokoll archiviert · **E3** Test-/Chaoslauf reproduzierbar auf unabhängigem Runner · **E4** E3 plus unabhängiges Orakel, Mutation-Wirksamkeit und externer Review. `maturity = stable` setzt E4 voraus (§17.6).
- **Beweisprinzip (INV-EVIDENCE-1):** Jede Zusage dieses Dokuments ist ein **Ziel**, kein Befund, bis der genannte Mechanismus sie maschinell belegt. Berichte, Audits, Skriptausgaben und Agentenaussagen sind Hypothesen und werden am Code geprüft.

---

## 2. Produkt

### 2.1 Definition

**Contextra ist ein lokaler, eingebetteter Wissens- und Gedächtnisspeicher in Rust, der speichert, hybrid findet (Vektor + BM25 + Fusion, mandantengetrennt) und gemessen löscht.** Single-Node: ein Primärprozess hält ein Datenverzeichnis, weitere lokale Prozesse sprechen über IPC mit ihm. Kein Sharding, kein P2P, keine Replikation, kein Netz-Listener, keine Telemetrie nach außen.

**Designpunkt:** ≤ 10⁷ Vektoren und ≤ 10² Mandanten je Knoten. Aussagen darüber hinaus sind nicht belegt. **Arbeitsannahme (per Benchmark zu belegen):** Agentengedächtnis umfasst je Betroffenem 10³ bis 10⁶ Einträge; SQ8 mit exaktem Nachranking, sichere Standard-Kernel und Partitionierung je Betroffenem stützen sich darauf.

**Eigenschaften:** _Isoliert_ (Mandanten physisch getrennt: Schlüssel, Indexe, Dateien), _Deterministisch_ (gleicher Anfangszustand + gleiche WAL-Folge ⇒ gleicher Zustand; nichtdeterministische Quellen gehen nur als aufgezeichnete Werte ins WAL, INV-NONDET-1), _Zero-Overhead_ (Hot-Path generisch monomorphisiert), _Ehrlich_ (Aussagen nur aus Messung, INV-CLAIMS-1).

### 2.2 Produktmissionen und Entscheidung

|Mission|Entscheidung v1.0|Begründung|
|---|---|---|
|M1 Persistenter Prefix-Cache (KV-Cache)|**Archiv**, ausgeschlossen|braucht GPU- und Modellkontrolle, trägt das Löschversprechen nicht|
|M2 Minimale Kontext-Latenz (**Time-to-Context, TTC**: Anfrage → fusionierter, berechtigter Kontext; p50/p99/p99.9 mit Konfidenzintervall)|**Kern**|Mittel: SIMD-Dispatch (nur soweit belegt), Arena-Allokation, Block-Max-WAND, SQ8 mit exaktem Rescoring, Bounded-Compute-Pool, Zero-Copy ab Block-Cache|
|M3 Grounding und Mandantensicherheit|**Kern**|Index je Tenant, ACL-Bitset vor Distanz, BM25-Statistik je Tenant: Burggraben-Hypothese|
|M4 Belegbare Antworten (Provenienz)|**Kern**|jedes Ergebnis ist auf einen WAL-Eintrag rückführbar (Quelle → Chunk → Embedding → Ergebnis); `rank/fusion` liefert Explain|
|M5 Deterministische Filterung (WASM)|**Rand** (`sandbox` optional)|nicht Teil des Versprechens|

### 2.3 Nicht-Ziele v1.0

Graph-Reasoning, Kognition/Konsolidierung, Router/Agentenlaufzeit, Online-Adaption, KV-Cache-Verwaltung, DiskANN/RaBitQ, Windows für Server und MCP (§13.7), verteilter Betrieb, Echtzeit-Audio/STT (VETO-OP03), Löschbeweis als Rechtsnachweis, ausgelieferte Verwahrung K2/K3 (Port vorgesehen, §12.2.3), Python-Wheels und Admin-Werkzeuge in v1.0 (E36), Dokumentformat-Parser im Kern (§10.3).

### 2.4 Produktgarantien G1–G6 (Löschung, Lesen, Isolation, Recovery)

|ID|Garantie|Test (§17.5)|
|---|---|---|
|G1|Nach Klasse P (Siegel 2, §12.8.5) finden Byte-Scans in allen Ebene-A-Strukturen (Memtable, Vektor- und Textindex, Caches) keine Klartextreste des Scopes; nach Klasse M zusätzlich auch nicht in SSTables, WAL, Blob-Segmenten und internen Checkpoints. Persistente Kopien liegen nach C1 bis C3 nur noch als Chiffrat unter vernichteten Schlüsseln vor (unter den Annahmen aus §12.5.2).|T-DEL-1|
|G2|Ein Löschbericht (V5) weist nur die Klasse aus, deren Voraussetzungen gemessen sind (INV-CLASS-MEASURED-1); er entsteht nur aus Scanner-Witnesses; die Signatur verifiziert mit falschem Schlüssel nicht.|T-DEL-2/3|
|G3|Absturz an jeder Stelle der Saga, einschließlich D1 bis D6 und Epoch Seal, führt zu Fortsetzung oder `Partial{outstanding}`, nie zu einem Bericht ohne Löschung und nie zu einem lebenden Schlüssel einer gelöschten Einheit (INV-NO-RESURRECTION-1).|T-DEL-4/5|
|G4|Ab `Frozen` liefert kein Lesepfad Daten des Scopes; ein Restore stellt sie nicht wieder her.|T-DEL-7/8/9|
|G5|Kein Mandant liest, findet oder löscht Daten eines anderen (VETO-F10).|T-TEN-1/2|
|G6|Nach hartem Abbruch ist der Zustand aus dem WAL rekonstruierbar („WAL First“, deterministische Recovery).|T-REC-2|

### 2.5 Robustheitsvertrag `[K]`

„100 % robust“ wird **nie** zugesagt. Zugesagt wird ein **begrenztes Fehlermodell**, jede Zusage durch einen wiederholbaren Test gedeckt (§17), und **Fail-Closed** für alles außerhalb des Modells (§7.8).

**Fehlermodell (innerhalb der Garantien):** (1) Prozess-Kill (`SIGKILL`) an beliebiger Stelle; (2) Stromausfall mit Verlust nicht geflushter Daten; (3) Teilschreibungen (Torn Writes) auf Sektor-/Blockebene; (4) Umordnung nicht geflushter Schreibvorgänge; (5) `fsync`/`fdatasync`-Fehler; (6) volles Medium (`ENOSPC`); (7) Bit-Rot einzelner Blöcke – **erkannt** (Prüfsumme, AEAD-Tag), nicht korrigiert; (8) beliebige, auch bösartige Eingaben (Bytes, Größen, ungültiges UTF-8).

|ID|Garantie|Hauptmechanismen|
|---|---|---|
|RB1 Crash/WAL|Jede bestätigte Transaktion ist nach Recovery vollständig oder gar nicht sichtbar; „bestätigt“ heißt durabel auf dem Medium (§7.3, §7.7); nach `fsync`-Fehler wird nie weitergeschrieben (INV-FSYNC-1).|WAL-Kette, Recovery, `Vfs`-Port, deterministische Simulation|
|RB2 Löschung|Nach Abschluss einer Saga sind die Inhalte des Scopes in den **gemessenen Schichten** nicht mehr vorhanden; der Bericht nennt belegte und nicht belegte Schichten (§12).|Scanner je Schicht, Canary-Orakel|
|RB3 Audit/Provenienz|Löschungen, Schlüsselwiderrufe, Zugriffe und Freigaben stehen in einer manipulationserkennenden Kette (AuditChain, RevocationLog).|HMAC-Kette, Sidecar-Logs|
|RB4 Isolation|SSI auf dem Keyspace, Snapshot-Isolation in Indexen (§8); keine mandantenübergreifenden Datenflüsse; keine internen Schlüssel über die öffentliche API (INV-SCAN-1).|SSI-Validator, Index je Tenant, Property-/Fuzz-Tests|

**Ausdrücklich nicht zugesichert:** SSD-Wear-Leveling-Reste; Backups und Dateisystem-Snapshots außerhalb von Contextra; an andere Ziele gesendete Daten; Prozessspeicherabbilder, Swap, Core-Dumps, Hibernation; Fremdprozesse (auch Root) mit Schreibzugriff auf das Datenverzeichnis; Hardware, die einen Flush bestätigt, ohne ihn auszuführen; Dateisysteme außerhalb der Garantieplattform (§13.7).

**Betriebszustände** `Healthy → Degraded → ReadOnly → Poisoned` sind in §7.8 normiert. Jede Garantie verweist über §17.7 auf **einen** wiederholbaren Befehl (`cargo xtask verify`).

### 2.6 Ehrliche Grenzen des Löschversprechens

Contextra garantiert Löschung **innerhalb seiner Kontrolle** und trennt dabei zwei Ebenen, die **nur gemeinsam** wirken: **Ebene A** bereinigt physisch alle Klartext-Strukturen (Arena-Slots, Caches, Postings, Antwort-Caches; Hygiene: Purge-Kompaktion, WAL-Scrub, Checkpoint-Rewrite, Graph-Neuaufbau), **Ebene B** vernichtet die Schlüssel (DocDek, ArtKey, über Epoch Seal auch die KEK-Epoche), die persistente Reste schützen, die Contextra nicht überschreiben kann (Flash, Copy-on-Write). Die Wirkung wird in Klassen **L, P, C\*, C1–C3, M** ausgewiesen (§12.5); jede Klasse nennt ihre Annahmen (A-FLASH, A-ALGO, A-KEYLEAK, A-RAM, A-EXTBACKUP, A-HOSTHIST, A-CLOCK). **Schlüsselvernichtung allein gilt nie als Löschung**; ob sie rechtlich als Löschung gilt, ist offen (P-01, P-02) und wird im Produkt nicht vorweggenommen (E44). **Nicht garantiert:** RAM-Kopien, Swap/Hibernation, Core-Dumps (A-RAM), in Modellparameter eingeflossenes Wissen, extern ausgeleitete Daten, historische Medienzustände (SSD, Journal; bei K1 Annahme A-FLASH), Backups außerhalb der Spuren D und K, Dateisystem-Snapshots, aktive ältere Snapshots. Diese Grenzen stehen im Bericht (`excluded`, nie leer, §12.20). **Verboten** in jeder Außenaussage: „100 % gelöscht“, „garantiert“, „DSGVO-konform“, „unwiederbringlich“, „Löschung nach Art. 17“, „kryptografisch gelöscht“ ohne Custody und Annahmen. Die verbindliche Formulierung entsteht aus der Satzbibliothek (§12.11).

---

## 3. Verfassung

### 3.1 Vetos (unverhandelbar)

- **VETO-F10 – Keine mandantenübergreifenden Datenflüsse.** Kein Cache, Index (auch keine Kante innerhalb eines Index), Embedding, Statistik (z. B. IDF) oder Audit-Eintrag überschreitet Tenant-Grenzen. Der Tenant wird aus der Transaktion abgeleitet, nie aus Nutzerparametern. Status: `permanent_rejected`.
- **VETO-F02 – HNSW-Löschung nur per Tombstone + Slot-Nullung + Rebuild**, **kein** Rewiring der Nachbarn. Status: bestätigt (Review-Frist 2026-10-07 war abgelaufen; Bestätigung in WP 0.3).
- **VETO-OP03 – Kein Echtzeit-Audio/STT** im Kern. Status: geschlossen (kein betroffener Code, WP 0.3).
- **Single-Node-Doktrin** – kein Sharding, kein P2P, keine verteilte Replikation.

Vetos und Invarianten sind maschinenlesbar in der Verfassungsdatei des Repos versioniert (je ID); Änderungen brauchen einen ADR mit Begründung, Alternativen und signierter Freigabe (§18.6).

### 3.2 Engineering-Prinzipien EP1–EP15

1. **Fail-closed**, kein Prozessabbruch durch erwartbare Fehler; I/O-Fehler sind typisierte Fehler, nie SIGBUS/Abort (INV-IO-1).
2. **Messbare Testtiefe:** jede `INV-*` hat einen Test, der ihre Verletzung erkennt (Mutation belegt Wirksamkeit); Schwellen stehen in `governance/ratchet.toml` und dürfen nur besser werden. Tests werden aus Spezifikation und Invarianten abgeleitet, nicht aus der Implementierung desselben Erzeugers.
3. **Null technische Schuld; Wired-or-it-doesn't-exist (P29):** Eine Fähigkeit existiert nur, wenn die Composition Root sie verdrahtet **und** ein Integrationstest sie über den echten Pfad beweist.
4. **Benchmarking mit fremder Baseline:** jede veröffentlichte Zahl nennt Commit, Modell-Hash, Hardware, Workload, Vergleichsbasis (§16).
5. **Atomare Zyklen:** ein Zyklus = ein Rot→Grün-Schritt, ein Scope, ein verifizierter Commit; Merges serialisiert über eine Merge-Queue auf dem **Merge-Ergebnis** (§18.2).
6. **Jepsen-artiges Chaos-Testing** mit `FaultVfs` (§17.4).
7. **Deterministische Verdrahtung:** nur die Fassade instanziiert und verbindet; keine globalen Singletons.
8. **IoC:** Merge, Metrik, Analyzer, Observer, Embedder, Ranker sind Ports mit Registrierung.
9. **Automatisierte Audits sind selbst Testobjekte** (Meta-Gate, §17.3).
10. **Absolute Verifizierbarkeit:** jede Behauptung (gelöscht, berechtigt, unverändert) ist belegt oder ausdrücklich als nicht belegt deklariert.
11. **Compiler vor Heuristik:** was `rustc`, `clippy`, `cargo metadata`, `cargo public-api` prüfen können, prüfen sie; Text-/Regex-Audits ergänzen nur.
12. **Bauen oder kaufen – belegt:** Eigenbau eines Standardbausteins (LSM, HNSW, BM25, Block-Cache, SIMD-Kernel, MCP-Protokollschicht) bleibt nur bei belegtem Vorteil in Kerneigenschaften (Löschmessung, Determinismus, Mandantentrennung) oder Leistung gegenüber der besten fremden Implementierung hinter demselben Port. Kryptografische Primitive werden nie selbst gebaut. Fremdimplementierungen dienen auch dort als Orakel im Test, wo sie als Abhängigkeit nicht taugen.
13. **Evidenz vor Behauptung:** maßgeblich ist maschinell erzeugte, wiederholbare Evidenz (Befehl, Toolchain, Commit, Exit-Code, Dauer, Ausführungszahlen, Hash); ein nicht ausgeführtes Prüfwerkzeug ist **rot**, nie „bedingt grün“.
14. **Tiefe vor Breite:** der Kern ist geschlossen (§4.8); eine neue Fähigkeit betritt ihn erst, wenn die Evidenz des bestehenden Kerns die Zielstufe erreicht hat.
15. **Ein unabhängiges Orakel je Kerngarantie** (RB1–RB4): Referenzmodell, Brute-Force, Canary-Scanner; nicht aus denselben Datenstrukturen und nicht vom selben Erzeuger (INV-ORACLE-1, §17.8).

### 3.3 Architekturprinzipien

**Prinzipien P** (Nummern stabil, Lücken gewollt):

|ID|Prinzip|
|---|---|
|P1|Ring-Disziplin: Kanten (normal/build) zeigen auf gleichen oder niedrigeren Ring.|
|P2|`unsafe` nur in drei Inseln (`simd`, `sys`, `wire`) im eigenen Code; jeder handgeschriebene Block trägt `// SAFETY:` und ist durch Test/Miri gedeckt. Generierter Code nur in `@generated`-Dateien, Generator und Version gepinnt, Ausgabe in der CI reproduzierbar geprüft, nur über sichere Adapter genutzt, von Fuzz/Miri erfasst. Edition-2024-Attribute `#[unsafe(no_mangle)]`/`#[unsafe(export_name)]`/`#[unsafe(link_section)]` zählen als `unsafe`. Native Drittabhängigkeiten stehen im Supply-Chain-Inventar (§18.4).|
|P5|DAG-Integrität: keine Aufwärtskanten, keine Zyklen (normal+build, **mit aufgelösten Features**); Dev-Aufwärtskanten nur als in `capabilities.toml` deklarierte Ausnahme.|
|P7|Zero-Panic in Produktion (Liste in §0.2 Regel 2, zusätzlich unchecked Slicing); Lint-Satz in §17.3 auf `deny`. Ergebnisse von `Mutex::lock()`/`RwLock::*` (Poisoning) sind typisierte Fehler mit definiertem Gesundheitsübergang (§7.8). Ratchet-Baselines dürfen nur sinken; Ziel 0, ein Ratchet ist kein Beleg für „null“. Release-Profil: `overflow-checks = true` für Kern-Crates. FFI-Builds (`py`): `panic = "unwind"` mit Abfangen an der Grenze; Binaries: `panic = "abort"`.|
|P24|Aller **Anwendungszustand** (Dokumente, Metadaten, Indexabsichten, Lineage, Retention-Entscheidungen) geht durch das WAL. Die einzigen weiteren persistenten Zustände sind die Sidecar-Logs (§7.9, INV-SIDECAR-1).|
|P26|Sync-Kern, Async-Schale: Ring-0-Crates führen **keine Async-Runtime** (`tokio`, `async-std`) als Produktions-/Build-Abhängigkeit und blockieren nie auf einer Runtime; Algorithmen sind synchron. Port-Traits dürfen Futures zurückgeben. Umsetzung in v1.0 als Ratchet (§6).|
|P28|Determinismus: Zeit, Zufall, IDs aus `Clock`, `Rng`, `IdGen`; `SystemTime::now`, `Instant::now`, `thread_rng`, `HashMap/HashSet` (RandomState) in Kernlogik per `clippy::disallowed_*` verboten; Ausnahme: Schlüssel-/Salt-Material.|
|P29|Wired-or-it-doesn't-exist (EP3).|
|P30|Kern-Isolation: `[K]`-Crates hängen weder normal noch build (im Release-Default-Feature-Set) von `[R]`/`[E]`/`[A]` ab; Rand-Fähigkeiten erreichen den Kern nur über Ports, die die Fassade injiziert.|

**Architekturregeln A1–A10 (dieser Fassung):**

|ID|Regel|Durchsetzung|
|---|---|---|
|A1|**Sicherheit ist kein Feature.** Messung, Verschlüsselung und Löschbericht sind in jedem Build aktiv; kein `cfg(not(feature))`-Modul ersetzt einen Typ aus `crypto`.|Gate `no-security-stub`|
|A2|**Eine Typfamilie je Konzept.** Berichts-/Beweistypen existieren nur in `contextra-crypto`.|Gate (REQ-PRF-4)|
|A3|**Messen vor Behaupten.** `LayerWitness` hat keinen öffentlichen Konstruktor außerhalb der Scanner der besitzenden Crates.|Typsystem + Gate|
|A4|**Kern kennt keinen Rand (P30), aber auch keinen Plugin-Typ.** Verbindung nur über Ports und den Composition Root.|`ring-layering` strikt mit `may_depend_on` (§4.4)|
|A5|**Kein Default ohne Absicht.** Schlüssel, Mandant, Zeit, Zufall und IDs kommen aus `RequestContext` bzw. Ports; kein Env-/Literal-Fallback.|Gate `determinism` (Ratchet), `disallowed-methods`|
|A6|**Ein Fehlerbaum.** Alle Crates münden in `ContextraError`; FFI und MCP mappen auf stabile Codes (§13.6).|Gate `error-domain`|
|A7|**Kleine öffentliche Oberfläche.** `pub` nur für dokumentierte API, sonst `pub(crate)`.|`cargo public-api`-Snapshot|
|A8|**Jede Invariante hat einen Test oder wird gestrichen** (Tabelle INV → Testname).|Gate `inv-coverage` (v1.0: Invarianten zu G1–G6)|
|A9|**Ein Dokument je Thema.** Normatives steht nur in `docs/spec/SPEC.md`.|INV-DOC-SINGLE-SOURCE-1, Gate `doc-stamp`|
|A10|**Keine neue Fähigkeit vor dem zugehörigen Test.**|Review, Phasenabnahme|

### 3.4 Invariantenregister (einzige Tabelle; jede `INV-*` einmal definiert)

Spalten: ID · Klasse · Invariante · Mechanismus (Kürzel §1.4) · Anforderung (§19). `INV-SHRED-1` der Fassung 1.3 bleibt in dieser Form entfallen; seine Funktion übernehmen `INV-DEL-ORDER-1`, `INV-NO-RESURRECTION-1`, `INV-BACKUP-SPLIT-1` und `INV-CLASS-MEASURED-1` (E44, §26.2). Archivierte Invarianten `[A]` stehen in Anhang B.

|ID|Kl.|Invariante|Mech.|REQ|
|---|---|---|---|---|
|INV-CORE-ISOLATION-1|K|Umsetzung von P30: Gate über `cargo metadata` (aufgelöst, **inkl. Default-Features der Kern-Crates**) lehnt jede Kante `[K] → [R]/[E]/[A]` ab; Features dieser Klassen sind im Release-Default aus.|G|REQ-ARCH-003|
|INV-KMS-1|K|Jeder Schlüssel stammt von einem `KmsProvider`; kein Modul hält Master-Material außerhalb des Providers; mindestens die lokale Implementierung ist verdrahtet. Ableitung direkt aus Passphrase mit festem/globalem Salt (Muster `CryptoKey::try_new(passphrase, konstanter_salt)`) ist außerhalb des Keystore-Pfads verboten.|G+I|REQ-KEY-001|
|INV-DELETION-WIRE-1|K|`forget` löst **alle zehn** Schichten über die Composition Root aus: acht werden gemessen, `Graph` und `KvCache` sind `FeatureDisabled` (Start belegt „keine Daten dieses Typs“); externe Sicherungen stehen als Ausschluss im Bericht (§12.4, §12.20). Ein Bericht mit Klasse P oder höher entsteht nur, wenn jede Schicht belegt oder ausgewiesen ist (Typestate; Auslassen = Kompilierfehler).|T+I|REQ-DEL-001|
|INV-SSI-ACTIVE-1|K|SSI ist im Produktionspfad aktiv; Read-Sets inkl. Prefix-/Range-Phantomschutz werden bei jedem Commit validiert.|P+C+L|REQ-MVCC-001|
|INV-INDEX-SNAPSHOT-1|K|Jeder Indexeintrag trägt `born_seq`/`died_seq`; Suchen am Snapshot `S` sehen nur `born_seq ≤ S < died_seq`; `as_of` gilt für Indexe nur bis zum Retention-Horizont der Tombstones.|P+C+L|REQ-MVCC-002|
|INV-DOC-SINGLE-SOURCE-1|K|Jede Tatsache (Konstante, Ring, Reifegrad, Format, ID) hat genau eine kanonische Quelle (`capabilities.toml`, Verfassungsdatei, Code); Abgeleitetes wird generiert oder per Doc-Truth-Gate geprüft; doppelt vergebene IDs sind Gate-Fehler.|G|REQ-GOV-004|
|INV-RING0-DETERMINISM-1|K|Ring 0 ist deterministisch: kein I/O außerhalb der Ports (insb. `Vfs`), keine Uhr/kein Zufall ohne Port, kein Ergebnis hängt von Hash-Iterationsreihenfolge, Thread-Scheduling oder unfixierter paralleler Float-Reduktion ab.|G+P|REQ-ARCH-015|
|INV-NONDET-1|K|Ergebnisse nichtdeterministischer Quellen (Embedding-Inferenz, LLM-Ausgabe, Zufall) werden als **Werte** ins WAL geschrieben; Recovery und Index-Rebuild lesen sie dort und rufen die Quelle nie erneut auf.|P+G|REQ-DUR-012|
|INV-WAL-SSOT-1|K|Jeder persistente **Anwendungszustand** ist aus WAL + Manifest rekonstruierbar; Indexe sind abgeleitet. Sicherheitszustand, der nicht aus dem WAL ableitbar ist (weil WAL-Scrub ihn entfernt), liegt ausschließlich in den Sidecar-Logs.|C+P|REQ-DUR-016|
|INV-ATOMIC-IDX-1|K|Metadaten- und Indexänderung gehören zu **einem** WAL-Batch; Recovery wiederholt Index-Intents idempotent.|C+P|REQ-DUR-011|
|INV-INDEX-APPLY-1|K|Indexe werden von **einem** Applier strikt in WAL-Reihenfolge angewendet; HNSW-Level folgen aus `hash(doc_id, seed)`, nie aus einem Laufzeit-RNG. Parallelität nur bei Rebuild mit deterministischer Partitionierung.|C+P|REQ-DUR-011|
|INV-TENANT-1|K|Tenant-Ableitung aus der Transaktion; jede Schlüsselkonstruktion enthält das Tenant-Präfix.|T+P+F+G|REQ-TEN-001|
|INV-TENANT-IDX-1|K|Vektor- und Text-Indexe sind **physisch je (Tenant, Collection[, Modell-Hash])** getrennt; keine tenantübergreifende Kante, Posting-Liste oder Statistik.|T+P+F+G|REQ-IDX-001|
|INV-KEYSPACE-1|K|LSM-Schlüssel, Index-IDs und Dateinamen enthalten nur opake IDs/Hashes, nie personenbezogene Inhalte.|T+P+F+G|REQ-IDX-010|
|INV-DERIVED-1|K|`forget` kaskadiert über die Lineage auf Ableitungen (Chunk-Präfixe, Zusammenfassungen, Konsolidierungsfakten, Embeddings-Caches); Ableitungen mit weiteren Quellen werden neu abgeleitet oder als `stale` markiert, nie stehen gelassen.|I+P|REQ-DEL-007|
|INV-ENCRYPT-REST-1|K|Nutzdaten sind at-rest AEAD-verschlüsselt in zwei Ebenen (§12.2): Engine-Schlüssel `K_eng` (Struktur/Integrität) und DocDek/ArtKey (Nutzwerte, §12.2). Verschlüsselung ist Standard; ohne sie weist `doctor` rot aus; nur `EdgeMinimal`/Entwicklung.|I+G+P|REQ-KEY-003|
|INV-ENCRYPT-REST-2|K|Per `mmap` zero-copy gelesene Dateien sind nur zulässig, wenn `doctor` ein verschlüsseltes Volume nachweist; sonst `pread`/Page-Decrypt-Cache.|I+G+P|REQ-KEY-004|
|INV-FAILCLOSED-1|K|Unklarer Zustand ⇒ Ablehnung, nie Raten.|C+I|REQ-ARCH-016|
|INV-KDF-1|K|Passphrasen-Schlüssel über Argon2id mit zufälligem, persistiertem Salt und persistierten Parametern (`KdfHeader`; m ≥ 64 MiB, t ≥ 3, Kalibrierung ≥ 250 ms). Reines HKDF auf eine Passphrase und feste/globale Salts sind verboten. Passphrase nie ausschließlich per Umgebungsvariable; Keyfile, OS-Keyring oder interaktive Eingabe stehen bereit.|I+G+P|REQ-KEY-002|
|INV-FSYNC-1|K|Schlägt `fsync`/`fdatasync` fehl, ist der Handle unbestimmt: kein Ack, kein Retry auf demselben Handle, WAL-Handle wird vergiftet (`Poisoned`), Weiterbetrieb erst nach Neustart mit Recovery-Replay.|C|REQ-DUR-002|
|INV-DURABILITY-PRIM-1|K|Der Commit-Punkt nutzt die plattformgerechte Durability-Primitive (§7.7); „fsync“ ohne diese ist kein Commit.|C|REQ-DUR-001|
|INV-ENOSPC-1|K|Volles Medium hinterlässt keinen Phantom-Eintrag und keinen halb angewendeten Batch; WAL-Segmente werden vorab reserviert, sodass `ENOSPC` bei Rotation statt im Record auftritt; Wechsel in einen definierten lesenden Zustand, retrybarer Fehler.|C|REQ-DUR-003|
|INV-IO-1|K|Lesepfade geben I/O-Fehler als typisierte Fehler zurück. `mmap` nur für vollständig prüfsummenverifizierte, unveränderliche, volumenlokale Dateien; sonst `pread`.|G+C|REQ-DUR-004|
|INV-FORMAT-1|K|Jede persistente Datei trägt Magic und Formatversion; neuere Version ⇒ Öffnen verweigert; Migration nur explizit, getestet, mit Vorab-Backup. Formate sind explizit spezifiziert (Feldreihenfolge, Endianness, Längen), nicht aus `serde`-Derives abgeleitet.|G+F|REQ-DUR-005|
|INV-LOCK-ORDER-1|K|Vollständige, geordnete Sperrenhierarchie; keine Sperr-Guards über `.await` (§8.2).|L+G|REQ-MVCC-004|
|INV-HNSW-SLOT-1|K|Arena-Slots gelöschter HNSW-Knoten werden nicht wiederverwendet, solange veraltete Kantenreferenzen möglich sind (§9.3).|L+G|REQ-IDX-002|
|INV-EMBED-1|K|Kein stiller Platzhalter-Embedder; Mock nur hinter `cfg(test)`/`test-utils`; `test-utils` in keinem Release-/Default-Set.|G+I|REQ-IDX-011|
|INV-EMBED-2|K|Collection persistiert `embedding_model_hash` und Dimension; abweichender Hash wird abgewiesen (§11.1).|G+I|REQ-IDX-011|
|INV-WIRING-2|K|Sicherheits-Ports (Liste in `governance/security_ports.toml`, §5.2) haben in Produktion eine **reale** Implementierung; Null-, Test- und Accept-All-Implementierungen (konstantes `true`/`Ok(())`) als Standard sind unzulässig.|G+I|REQ-FAC-003|
|INV-DELETION-PORTS-1|K|Die Löschpfad-Ports `LayerScanner`, `Purger`, `ErasureJournal`, `Vfs`, `DurableLog`, `KmsProvider` sind Sicherheits-Ports (INV-WIRING-2) und in `security_ports.toml` geführt.|G+I|REQ-DEL-023|
|INV-PROOF-COVERAGE-1|K|Jede der zehn Schichten erscheint in einem Bericht genau einmal als belegt (`Structural`), `NotMeasured` oder `FeatureDisabled`.|T+I|REQ-DEL-003|
|INV-WITNESS-MINT-1|K|`LayerWitness` entsteht ausschließlich in `ScanSession::finish` (Konstruktoren `pub(crate)`, kein `Default`/`Deserialize`); `ScanSession` ist nur über `LayerScanner::begin_scan` mit bestandener Positivkontrolle (`ControlToken`) erhältlich; Konstruktion außerhalb davon, auch in Produktionstests, ist verboten (Ausnahme `testkit`).|T+G|REQ-DEL-004|
|INV-PROOF-KEY-1|K|Der Signierschlüssel des Berichts stammt aus dem `KeyManager`/`KmsProvider`, nie aus einem frei übergebenen Byte-String; es gibt keinen Fallback-/Standard-/Testschlüssel.|T+P|REQ-DEL-005|
|INV-CLAIMS-1|K|Externe Zusicherungen werden aus dem Bericht (`outcome`, `claims`, `excluded`) **erzeugt**, nie aus dem Zielzustand; Außentexte entstehen nur aus der Satzbibliothek (§12.11).|G|REQ-DEL-006|
|INV-PROVENANCE-1|K|Merges sind scope-treu, beschreibend benannt, serialisiert über eine Merge-Queue und auf dem Merge-Ergebnis kompilierend (§18.2).|G|REQ-GOV-001|
|INV-PLATFORM-1|K|Durabilitäts- und Löschgarantien gelten **nur auf der Garantieplattform** (§13.7); andere Plattformen sind Best Effort: `doctor` weist `BestEffort` aus, Berichte tragen `platform_guarantee = false`. Löschlogik stützt sich nie auf Unix-Unlink-Semantik offener Dateien. Funktionen ohne Plattformäquivalent (`mlock`) sind in `doctor` deklariert.|CI+C|REQ-PLT-001|
|INV-IPC-1|K|Genau ein Primärprozess hält das Datenverzeichnis; weitere Clients nutzen ausschließlich lokales IPC (Unix-Socket/Named Pipe, Peer-Credentials, kein TCP/UDP).|I+C+G|REQ-MCP-002|
|INV-STDOUT-1|K|`stdout` gehört ausschließlich dem MCP-Protokoll; Logging nach `stderr`/Datei; `clippy::print_stdout` ist in allen Crates des MCP-Binärs `deny`.|I+C+G|REQ-MCP-001|
|INV-CANCEL-1|K|Anfrageabbruch (`notifications/cancelled`, Timeout, Disconnect) wirkt kooperativ bis in die Engine; vor dem WAL-Append bricht er ab, danach wird er ignoriert (Commit bleibt atomar).|I+C+G|REQ-MCP-007|
|INV-FORGET-GUARD-1|K|Breite Löschungen (Selektor, Collection, Tenant, > `forget_confirm_threshold` Dokumente) brauchen eine Bestätigung über einen Kanal **außerhalb des Tool-Aufrufs** (§13.3).|I+P|REQ-MCP-005|
|INV-TRUST-1|K|Jeder Eintrag trägt `trust ∈ {user, agent, external}` und `source`; Recall liefert Inhalt strukturiert als Daten mit Trust-Label, nie als Anweisung.|I+P|REQ-MCP-006|
|INV-LOG-1|K|Logs, Metriken, Fehlertexte und Panic-Meldungen enthalten keine Nutzinhalte, nur IDs und Zähler.|G+P|REQ-AUD-002|
|INV-RETENTION-1|K|Ranking-Abwertung löscht nie. Löschung entsteht ausschließlich durch ausdrückliche Aktion oder eine opt-in `RetentionPolicy` mit Dry-run, Karenzzeit und Audit (§11.6).|P+I|REQ-COG-003|
|INV-EVIDENCE-1|K|Jede Zusage ist ein Ziel, bis der Mechanismus sie maschinell belegt; ein nicht ausgeführtes oder nicht verfügbares Prüfwerkzeug ist rot.|G|REQ-QA-001|
|INV-ORACLE-1|K|Jede Kerngarantie RB1–RB4 hat ein unabhängig entstandenes Prüfmaß.|P|REQ-QA-002|
|INV-SCAN-1|K|Die öffentliche API enthält keine Funktion, keinen Typ und kein Feld, das Schlüsselmaterial (WAL-Integritätsschlüssel, DEK, KEK, Signierschlüssel) im Klartext zurückgibt; Test-Zugänge liegen hinter `cfg(test)` oder einem Test-Feature außerhalb jedes Release-/Default-Sets.|G|REQ-ARCH-014|
|INV-SIDECAR-1|K|Außer WAL, Manifest, SSTables, Blob-Segmenten und Indexdateien existiert persistenter Zustand nur in den Sidecar-Logs (§7.9). Jedes Sidecar trägt Magic und Version, eine HMAC-Kette mit eigenem Zweckschlüssel und eine Manifest-Marke. Wirkungen eines WAL-Commits stehen als `SidecarIntent` im selben Batch und werden idempotent angewendet; sicherheitskritische Wirkungen folgen „Sidecar vor Wirkung“.|G+C+I|REQ-DUR-016|
|INV-EMERGENCY-DEL-1|K|Ein Betroffener ist in jedem Betriebszustand mindestens **sperrbar**: in `ReadOnly` (ENOSPC) sind `forget`-Start und Lesesperre über eine vorab reservierte Notreserve ausführbar; in `Poisoned` ist der Sperr-only-Betrieb über unabhängige Sidecar-Handles zulässig, WAL-Schreiben nicht. Fehlt auch das, gilt eine prozesslokale Lesesperre (fail-closed) bis zum Neustart (§7.10).|C+I|REQ-DUR-017|
|INV-ASOF-ERASE-1|K|Historische Lesezugriffe (`as_of`, offene Snapshots, Checkpoint-Lesen) unterliegen der Lesesperre; ein gelöschter oder gesperrter Betroffener ist in keinem Snapshot lesbar, auch wenn die Altversion physisch existiert. Snapshots haben Höchstalter `snapshot_max_age`.|P+L|REQ-DEL-030|
|INV-BOOT-1|K|Der Start folgt der festen Phasenfolge B0–B9 (§13.5): Die Lesesperre (`ErasedSet`) ist armiert, bevor irgendein Lesepfad Verkehr annimmt; jede Phase kann den Start mit benanntem Grund verweigern; kein Teilstart.|I+C|REQ-FAC-005|
|INV-KEYPURPOSE-1|K|Jeder Schlüssel dient genau einem Zweck und entsteht aus dem Zweckregister (§12.3) mit eigenem HKDF-Label; ein Label ohne Registereintrag ist Gate-Fehler.|G+P|REQ-KEY-011|
|INV-RESTORE-HORIZON-1|K|`restore_checkpoint` und jeder Import verweigern fail-closed jedes **Schlüssel-Backup** (Spur K), dessen `valid_until` überschritten oder dessen Alter nicht bestimmbar ist; Daten-Backups (Spur D) sind ohne Schlüssel inert. Die Konfiguration erzwingt `key_backup_max_age ≤ erased_set_retention`. Ein `ErasedSet`-Eintrag bleibt mindestens `erased_set_retention` zuordenbar (Referenz-Epochen, §12.10).|I+C+G|REQ-DEL-032|
|INV-DEL-ORDER-1|K|Die Schlüsselvernichtung (Schritt 5) beginnt erst, wenn Klasse P (Siegel 2) für jede Ebene-A-Schicht der Einheit ausgestellt und das Pre-Destruction-Inventar durabel ist; einzige Ausnahme ist der Notpfad (§7.10).|I+C|REQ-DEL-N20|
|INV-NO-RESURRECTION-1|K|Beim Start (B6/B7) und nach jedem Restore steht für jede `ErasedSet`-Einheit ab `Frozen` kein lebender DocDek und kein lebender ArtKey in der KeyTable; Abweichung ⇒ Vernichtung wird wiederholt, Alarm (§12.8.4).|C+P|REQ-DEL-N21|
|INV-BACKUP-SPLIT-1|K|Kein Daten-Backup (Spur D) enthält Schlüsselmaterial; jedes Schlüssel-Backup (Spur K) trägt ein signiertes `valid_until` und wird danach verweigert (§12.17).|I+G|REQ-DEL-N22|
|INV-CLASS-MEASURED-1|K|Ein Bericht weist höchstens die Klasse aus (L, P, C*, C1–C3, M), deren Voraussetzungen gemessen sind; die Klasse ist nie höher als die Messung trägt (§12.5.1).|T+P|REQ-PRF-2|
|INV-CLAIM-BIND-1|K|Außentexte entstehen nur aus einer festen Satzbibliothek; jeder Satz ist an einen gemessenen Wert gebunden; ein Satz ohne Messwert bricht den Build (Gate `claim-generator`, §12.11).|G|REQ-DEL-035|
|INV-SURFACE-1|K|Jede persistente Datei- und Strukturklasse des Format-Registers (§7.11) steht in genau einer Zeile der Schichttabelle (§12.4) oder in den Ausschlüssen (§12.20); ein neues Format ohne Schichtzuordnung bricht den Build (Gate `surface-coverage`).|G|REQ-DEL-N23|
|INV-NO-DUMMY-PROOF-1|K|Kein Codepfad erzeugt einen `DeletionProof` ohne Scanner-Witness; `contextra_delete` und `contextra_drop_collection` erzeugen keinen Bericht außerhalb der Saga.|G+I|REQ-PRF-4|
|INV-EPOCH-SEAL-1|K|Die Klassen C1 bis C3 gelten erst nach dem ersten Epoch Seal, der die Löschung umfasst; bis dahin lautet die Klasse C\* mit `residuals: [R-KEKEPOCH until]` (§12.2.4).|C+P|REQ-DEL-N24|
|INV-ARTKEY-1|K|Jedes Index-Artefakt hat genau einen ArtKey; beim Neuschreiben wird der alte ArtKey vernichtet (§12.2.8).|P+C|REQ-DEL-N16|
|INV-ADMISSION-1|R|(`sandbox`) Ein `AdmittedModule` ist nur über `ModuleVerifier::verify` konstruierbar; SHA-256-Digest aus den Bytes, nie vom Aufrufer. Unsignierte Module nur mit `ApprovalRisk::Low`, im Audit als `unsigned`.|T+P+F|REQ-SBX-001|
|INV-APPROVAL-1|R|(`sandbox`) `High`-/`Elevated`-Module laufen nicht vor `ApprovalStatus::Approved`.|T+P+F|REQ-SBX-002|
|INV-WASM-DET-1|R|(`sandbox`) Gleiches Modul, Eingabe und Fuel liefern auf x86-64 und aarch64 bit-identische Ergebnisse; Wall-Clock-/Epoch-Unterbrechung ist nur Sicherheitsnetz und fail-closed.|T+P+F|REQ-SBX-003|
|INV-EGRESS-BUDGET-1|R|(`privacy`) Jede externe Wirkung ist budgetiert, auditiert und widerrufbar.|I+P|REQ-PRV-002|
|INV-LICENSE-1/-2/-3|E|(`license`) Verstoß/Ablauf/fehlende Aktivierung degradiert auf Basisfunktion, nie Datenverlust oder -sperre; niedrigste Klasse (`Fast`) immer `Ok`; falsche Installationsbindung ⇒ `NotActivated` ohne Informationsleck.|P|REQ-EXP-003|

**Zählung:** `[K]` 60 (53 der Fassung 1.3 minus `INV-SHRED-1` plus acht neue: `INV-DEL-ORDER-1`, `INV-NO-RESURRECTION-1`, `INV-BACKUP-SPLIT-1`, `INV-CLASS-MEASURED-1`, `INV-CLAIM-BIND-1`, `INV-SURFACE-1`, `INV-NO-DUMMY-PROOF-1`, `INV-EPOCH-SEAL-1`, `INV-ARTKEY-1`; die Summe ist vom Doc-Truth-Gate nachzuzählen), `[R]` 4, `[E]` 3 (als eine Zeile geführt). Das Doc-Truth-Gate prüft: jede `INV-*` einmal definiert, einmal im Mechanismus geführt, von mindestens einer Anforderung referenziert, IDs eindeutig (REQ-GOV-004).

---

## 4. Workspace-Architektur

### 4.1 Ringe und Klassen

`capabilities.toml` ist die **einzige** Quelle für Ring, Klasse (`class`: `kern`/`rand`/`experimentell`) und `maturity` je Crate sowie für deklarierte Dev-Ausnahmen; Skripte und Gates lesen nur dort (INV-DOC-SINGLE-SOURCE-1). Die Datei MUSS auf die Tabellen dieses Abschnitts gesetzt werden (WP 3.6). Richtung: Ring 4 → 3 → 2 → 1 → 0.

```
Ring 4  Fassade/Schnittstellen   contextra (Composition Root)   contextra-mcp   [R] contextra-py   [E] contextra-license, audit-export, avv-generator
Ring 3  Orchestrierung           contextra-engine   contextra-db   [R] contextra-privacy
Ring 2  Isolation/Inferenz       [R] contextra-infer-candle   [R] contextra-sandbox
Ring 1  Persistenz               contextra-store   contextra-checkpoint
Ring 0  Fundament                types  ports  mvcc  crypto  vector  text  rank(nur fusion/)  | Inseln: sys  simd  wire
Werkzeug                         xtask  testkit  bench  *-fuzz
Außerhalb des Kerns (§4.9)       [R] contextra-ingest   [R] Tauri-App
```

**Wirkung der Ringe:** Das Ring-Gate erlaubt jede Abwärtskante. Deshalb war z. B. `engine (R3) → infer-candle (R2)` nie ein Ringverstoß, wohl aber ein Klassenverstoß `[K] → [R]` (P30). Die Absicherung läuft über `may_depend_on` (§4.4) und das Klassen-Gate, nicht über Ringe allein.

**Reifegrad:** `maturity = stable` setzt voraus: (a) alle Sicherheits-Ports des Crates sind produktiv verdrahtet (INV-WIRING-2), (b) alle Gates (§17.3) grün, (c) Fuzz-, loom- und Miri-Abdeckung nach §17.1. Ein Crate mit nur Null-/Test-Implementierungen seiner Sicherheits-Ports ist `experimentell`. Zielreife: `[K]` ⇒ `stable` (erst bei E4), `[R]`/`[E]` ⇒ `experimental`.

### 4.2 Crate-Katalog mit Budget

Budgets sind Obergrenzen in Code-Zeilen (§1.3), gemessen auf Commit `c4f7299b6`; sie dürfen nur sinken, außer eine Phasenabnahme nennt ausdrücklich Zuwachs.

|Crate|Ring|Kl.|Unsafe|Ist|Budget v1.0|Aufgabe der Umbauphase|
|---|---|---|---|---|---|---|
|`contextra-types`|0|K|nein|2.611|2.700|`RequestContext`, `LayerWitness`-Grundtypen, `ModelIdentity`, IDs, Fehlerklassen, `MemoryType` (Default `Semantic`)|
|`contextra-ports`|0|K|nein|1.526|1.500|Ports archivierter Crates entfernen, Mocks raus (§5)|
|`contextra-sys`|0|K|**ja**|428|450|SAFETY-Lücken schließen; mmap, madvise, Durability-Primitive, Dateisperren, Speicherdruck, `mem_lock`, Win32-ACL|
|`contextra-simd`|0|K|**ja**|1.113|1.200|Kernel `pub(crate)`, SAFETY, Miri; Dispatch AVX-512 > AVX2 > NEON > skalar; **Skalarpfad ist Standard**, bis die Wiederholung von E2 einen Vorteil über 20 % zeigt (E33); skalare Referenz ist Orakel|
|`contextra-wire`|0|K|**ja**|190 (+1.201 generiert)|200|Zero-Copy-Dekodierung/Layout-Casts; Lint-`allow` entfernen|
|`contextra-mvcc`|0|K|nein|1.585|1.600|SequenceLog, SnapshotRegistry, TxBuffer, ReadSet, SSI-Validator; SSI-Nachweis (U-06)|
|`contextra-crypto`|0|K|nein|2.992|3.200|AEAD, KDF, Signaturen, `KmsProvider`-Impl (K0/K1), Löschbericht (V5, §12.7.5), `EffaceableKeySlot`-Impl (K1), KeyTable-/Wrap-Logik, Padmé, AuditChain, RevocationLog; `KmsProvider` verdrahten; **I/O-frei und lock-frei** (Persistenz über Port `DurableLog`); kein Wissen über Cache-Segmente|
|`contextra-vector`|0|K|nein|6.701|6.800|HNSW, SQ8, ACL-Bitset, `Metric`-Trait, Planner; DiskANN ausgliedern (Archiv), `async` entfernen (Ratchet, §6), `VectorScanner`|
|`contextra-text`|0|K|nein|2.329|2.400|BM25/BM25F, Block-Max-WAND, `Analyzer`-Port; `async`/`unsafe`/Panic entfernen, `TextScanner`|
|`contextra-rank`|0|K (reklass.)|nein|1.709 (`fusion/`)|1.750|nur `fusion/`: RRF, Normalisierung, Provenienz/Explain; Rest ins Archiv|
|`contextra-store`|1|K|nein|13.066|13.400|LSM, WAL, Blob, Manifest, Kompaktion, Block-Cache, CDC; implementiert `Vfs`, `DurableLog`, `ErasureJournal`, Rohdatei-Scanner; Fehlerpropagation, `unsafe` raus, `LsmScanner`, Purge-Kompaktion|
|`contextra-checkpoint`|1|K|nein|1.630|1.700|Snapshots, Backup-Export (ohne Keystore), verifizierte Wiederherstellung, Restore-Guard, `CheckpointScanner`|
|`contextra-engine`|3|K|nein|10.114|8.500 Ziel / 10.200 Grenze|Planner, Fusion-Aufruf, Bounded-Compute-Pool, `as_of`, Chunking, Reaper, Saga, Lesesperre, Startsequenz; Entkopplung von Archiv-Crates, Stub-Löschung|
|`contextra-db`|3|K|nein|585|600|Collections, Transaktionen, Tenant-Scoping; kennt **keine** Inferenz-Backends|
|`contextra-mcp`|4|K|nein|3.314|3.400|MCP-Server (stdio, JSON-RPC 2.0), IPC-Proxy, CLI; 3-Werkzeug-Fassade; kein `unsafe`|
|`contextra`|4|K|nein|729|800|**Fassade und einzige Composition Root**; öffentliche Rust-API; schlanker Default|
|**Kern gesamt**||||**50.622**|**52.000 Obergrenze**||

**Rechnung Ist:** 2.611 + 1.526 + 428 + 1.113 + 190 + 1.585 + 2.992 + 6.701 + 2.329 + 1.709 + 13.066 + 1.630 + 10.114 + 585 + 3.314 + 729 = **50.622**. Die Reserve von 1.378 bis 52.000 ist eine Annahme für Saga, Scanner und Restore-Guard. `checkpoint` (1.630) ist Kern, weil Restore Gelöschtes zurückbringen kann (E18).

**Optional (Features, nicht im Standardbau), 7.670 Zeilen:** `infer-candle` 2.660 (Embedding-Backend, `[R]`), `privacy` 1.331 (`[R]`), `license` 307 (`[E]`), `audit-export` 214 (`[E]`), `avv-generator` 196 (`[E]`), `sandbox` 1.105 + `mcp/sandbox.rs` 431 (`[R]`), `py` 1.426 (`[R]`). Details: Anhang A.

**Archiv** (Tag `archiv/v0`, aus dem Workspace), 25.981 Zeilen + `contextra-core` 106: `graph`, `cognition`, `adapt`, `kvcache`, `router`, `agent`, `infer-onnx`, `infer-ollama`, `vector/diskann`, `rank` ohne `fusion/`, `contextra-core` (Kompatibilitätsfassade, entfällt; keine neuen Abhängigkeiten darauf). Beschreibung: Anhang B.

**Werkzeug-Crates:** `testkit` (Fixtures, Chaos-Harness, `FaultVfs`, `ManualClock`, Fake-Verwahrer; nur `dev-dependency`, nie in Release-/Default-Features), `xtask` (Gates; **Schutzpfad**; Zeilenziel setzt die Inventar-Auswertung, Evidenz `evidence/xtask-inventory/`, §17.3/§22), `xtask-heavy` (Fuzz/Miri/Mutation/Chaos), `bench` (Suite, Baselines inkl. fremder Referenzen; 4.303 Zeilen; enthält `unsafe`; von Kernaussagen getrennt, §17.10).

### 4.3 Zielstruktur der Kern-Crates (Module)

Vorhandene Verzeichnisse von `store` (`compaction`, `engine`, `kv`, `lsm`, `manifest`, `memtable`, `sstable`, `system_pressure`, `tenant_codec`, `util`, `wal`) bleiben.

|Crate|Zielmodule und Schnittstellen|
|---|---|
|`store`|`wal/` (Append, Replay, HMAC), `lsm/` (Memtable, Flush, Leseweg), `sstable/`, `manifest/`, `compaction/` (inkl. **bereichsweise Purge-Kompaktion** `compact_range(scope)`), `scan.rs` (**neu**: `LsmScanner` für Memtable, SST, WAL), `fsync.rs` (einzige Stelle für Datei-/Verzeichnis-Sync, gibt Fehler zurück, plattformabhängig `fdatasync`/`F_FULLFSYNC`)|
|`vector`|`hnsw/` (Aufbau, Suche, Tombstone, Rebuild mit `Clock`/`Rng`), `quant/`, `scan.rs` (**neu**: `VectorScanner` über Slots, Snapshot, Quantisierungscodes)|
|`text`|`tokenizer/` (keine Panics; Fehler statt `panic!` in `tokenizer.rs:30`), `bm25/`, `scan.rs` (**neu**: `TextScanner` über Postings und Termliste)|
|`crypto`|`kms/` (`KmsProvider`-Implementierungen K0/K1), `proof/` (Löschbericht: `DeletionProof` V5 schreiben, V1–V3 lesen, `issue`/`verify`, Satzbibliothek), `keytable/` (Wrap, Re-Wrap, Epoch-Seal-Logik), `revocation_log`, `audit_chain`|
|`rank`|`fusion/` (RRF, Normalisierung, Explain/Provenienz)|
|`checkpoint`|`create`, `restore` (mit `ErasedSet`, Horizont), `manifest` (signiert), `scan.rs` (`CheckpointScanner`)|
|`engine`|`engine.rs` (Lebenszyklus), `remember/`, `recall/`, `forget/` (`saga.rs`, `plan.rs`, `recovery.rs`, `guard.rs`), `collection/`, `boot/` (B0–B9, §13.5), `observe.rs`|
|`db`|Embedded-API, Collection-Handle, keine Logik|
|`mcp`|`server/` (Transport, Prinzipal, Pipeline §13.2), `tools/` (3 Agentenwerkzeuge, `doctor`, Status), `confirm.rs` (Außenkanal), `config.rs` (ohne Mocks)|
|`contextra`|`builder.rs` (Composition Root), `features.rs`, `doctor`|

### 4.4 Kantenregeln (`may_depend_on`, strikt)

`capabilities.toml` führt je Crate `may_depend_on`. Heute meldet das Ring-Gate 66 „nicht deklarierte“ Kanten (Warnung) und 2 Fehler. **Ziel: alle Kanten deklariert, Warnungen = Fehler.**

|Crate|darf (normal) abhängen von|verboten (Beispiele aus dem Ist)|
|---|---|---|
|`types`|–|–|
|`ports`|`types`|`graph`, `kvcache`-Typen in Port-Signaturen|
|`sys`, `simd`|`types` (nur falls nötig)|alles andere|
|`wire`|`types`|–|
|`mvcc`|`types`, `ports`|–|
|`crypto`|`types`, `ports`, `sys`|`store`|
|`vector`|`types`, `ports`, `simd`|`store`, `graph`|
|`text`|`types`, `ports`|`store`, `graph`|
|`rank`|`types`, `ports`|`graph`, `adapt`|
|`store`|`types`, `ports`, `mvcc`, `crypto`, `sys`, `wire`|`graph` (Dev-Kante `graph → store`), `sandbox` (Dev-Kante `store → sandbox`)|
|`checkpoint`|`types`, `ports`, `crypto`|`store` (Zugriff nur über Port `StorageEngine`)|
|`engine`|`types`, `ports`, `mvcc`, `crypto`, `store`, `vector`, `text`, `rank`, `checkpoint`|`graph`, `adapt`, `kvcache`, `infer-*`, `privacy`, `license`, `sandbox` (jeweils nur über Port)|
|`db`|`types`, `ports`, `engine`|`cognition`, `graph`, `adapt`|
|`mcp`|`types`, `ports`, `wire`, `db`, `contextra` (nur Fassaden-API; Offline-Prüfwerkzeuge über Fassaden-Re-Export)|`rank`, `adapt`, `privacy`, `license`, `infer-ollama`, direkter Zugriff auf `crypto`/`store`|
|`contextra`|alle; Rand/Experimentell nur als `optional = true`|`license` und `infer-candle` als Pflicht|

**Kantenregeln:**

1. **Fundament.** `types`, `ports` und die Inseln `wire`, `sys`, `simd` bilden das Fundament; jedes Crate darf sie nutzen, soweit seine Zeile sie nennt. Typen, die mehrere Ring-0-Kerne teilen, liegen in `ports`/`types`, nicht in einem der Kerne.
2. **Kerne untereinander** (`vector`, `text`, `rank`, `mvcc`, `crypto`) hängen nur voneinander ab, wenn die Kante in der Tabelle steht und begründet ist.
3. **Ring 2.** Adapter (`infer-*`, `sandbox`) nutzen nur das Fundament.
4. **Ring 4.** `mcp` und `py` erreichen Ring 2 und 3 ausschließlich über die Fassade `contextra`; das Lizenz-Gate wird nur über den Port `LicenseGate` genutzt; die Fassade re-exportiert die Offline-Prüfwerkzeuge.
5. **Löschtypen** (Bericht, Schicht-Beleg, Messung, Receipt) existieren genau einmal in `crypto`/`ports`; andere Crates re-exportieren sie. Eine zweite Typfamilie für Löschung, Schlüssel oder KV-Segmente ist unzulässig.
6. **Test-Doubles.** Mocks, Fake-Uhr, Fehlerinjektion und Platzhalter-Embedder liegen in `testkit` oder hinter dem Feature `testing`; Test-Hooks (Panic-Auslöser, Schlüsselzugänge) nur hinter `cfg(test)` oder `test-hooks`. Beide Features liegen in keinem Release- oder Default-Set.
7. **Aufwärts-Dev-Kanten** nur als deklarierte Ausnahme in `capabilities.toml`; Ziel: nach `testkit` oder `tests/` des höheren Rings verlagern.
8. **`db` kennt keine Inferenz-Backends**; Embedder kommen als Port von der Fassade.
9. **Port-Besitz.** Jeder Port aus §5.2 ist in `ports` definiert; ein Trait mit Austauschrolle außerhalb von `ports` ist unzulässig. Crate-interne Hilfs-Traits sind „intern“ klassifiziert.
10. **Einzel-Besitz der Standardfusion.** RRF-Fusion und die Basis-Erklärung (Modalität, Score, WAL-Seq) liegen ausschließlich in `rank/fusion`; `engine` ruft sie auf und enthält keine zweite Standardfusion. _(Änderung gegenüber früheren Fassungen, in denen RRF in `engine` lag; E19.)_

**Stretch (nicht Pflicht für v1.0):** `engine` bindet `store`, `vector`, `text`, `checkpoint` nicht direkt, sondern über Ports und den Composition Root. Das ist eine Kern→Kern-Kante und verletzt weder P30 noch die Ringe; es lohnt sich erst nach Phase 3.

### 4.5 Features und Standardbau

- `default = []` in allen Crates; die Fassade `contextra` hat `default = ["embedded"]` (Kern ohne Inferenz-Backend; heute `default = ["fast","candle"]`, U-11).
- Rand/Experimentell nur über benannte Features der Fassade: `candle`, `privacy`, `sandbox`, `py`, `compliance` (= privacy + audit-export + avv-generator), `license`.
- `tokio` **ohne** `features = ["full"]`; nur `rt-multi-thread`, `sync`, `time`, `macros` im Edge-Crate `mcp` und im Composition Root.
- Tote Features (24 laut Scan) werden gelöscht; `unexpected_cfgs = "deny"`.
- Die App aktiviert `candle` per Feature; der Standardbau bleibt `embedded` ohne Inferenz-Backend und ohne Dokumentformate (E34).
- Je Produkt gibt es eine Crate-Hülle (`products/`); das Gate `product-hull` prüft sie gegen `cargo metadata` (Ziel: Kern = 16 Crates ohne Fuzz/Tooling; heute 25 Crates im Standard-MCP-Produkt), das Gate `module-removal` baut das Produkt ohne jedes Rand-Modul.

### 4.6 Unsafe-Inseln (genau drei)

`simd`, `sys`, `wire`. (1) Jede Insel exponiert eine **sichere** API. (2) Jeder `unsafe`-Block trägt `// SAFETY:` (Plattformcode wie Win32 eingeschlossen). (3) Miri-/Property-Tests gegen eine sichere Referenz. (4) Kein anderes Crate enthält `unsafe` im Produktionscode: `#![forbid(unsafe_code)]` in jedem Nicht-Insel-Crate, `unsafe_code = "deny"` workspaceweit, Testtargets nur für Test-Allokatoren/Fehlerinjektion mit lokalem `allow`. (5) Generierter Code folgt P2. (6) Das Gate weist generierten und handgeschriebenen `unsafe`-Code getrennt aus und prüft beide; **Dateien dürfen nicht anhand ihres Namens (`test_*.rs`) von der Prüfung ausgenommen werden.** `mcp` führt `unsafe` nicht einmal als Ausnahme (unvertraute Eingaben über stdio).

### 4.7 Rand-Brücken: wie Kern-Crates Rand-Fähigkeiten nutzen

Kern-Crates erreichen Rand-Fähigkeiten ausschließlich über Ports, die die Fassade bei aktiviertem Feature injiziert (P30). Fehlt die Fähigkeit, verhält sich der Kern wie in der Spalte „Ohne Rand“; er startet und besteht seine Tests auch dann (REQ-ARCH-024).

|Fähigkeit|Port|Anbieter|Ohne Rand|
|---|---|---|---|
|Graph-Modalität|`GraphIndex`|`graph` `[A]`|Planner lässt die Modalität aus; Ergebnis weist `modalities_unavailable` aus; Löschschicht `Graph` = `FeatureDisabled`|
|Reranking|`Reranker`|`infer-*` `[R]`|entfällt; RRF-Reihenfolge gilt|
|Embeddings|`EmbeddingProvider`|`infer-*` `[R]`|`EmbeddingBackend = None`: nur Textsuche (§11.1)|
|KV-Prefix-Cache|`KvPrefixStore`|`kvcache` `[A]`|inaktiv (kein stilles Fallback); Schicht `KvCache` = `FeatureDisabled`|
|Adaptive Regelung|`AdaptiveController`|`adapt` `[A]`|feste Startwerte aus §15|
|Kalibrierung, DiBuD|`Calibrator`|`rank` (Rest) `[A]`|rohe RRF-Scores|
|Egress, Erkennung, Injektion|`EgressChannel`, `EntityRecognizer`, `InjectionDetector`|`privacy` `[R]`|Egress gesperrt (INV-WIRING-2)|
|WASM-Hooks|`ModuleVerifier`, `FilterModule`|`sandbox` `[R]`|nicht registrierbar|
|Lizenzprüfung|`LicenseGate`|`license` `[E]`|keine Prüfung; der Kern kennt kein Lizenzkonzept|
|Multi-Step-Retrieval|`QueryRewriter`|`cognition` `[A]`|Single-Step-Planner|

Regeln: (1) **Modul-Entfernung:** Der Kern baut und testet ohne jedes `[R]`/`[E]`/`[A]`-Crate. (2) Die Fassade hängt von Rand-Crates nur über optionale Features ab, im Release-Default aus. (3) Typen, die Kern und Rand teilen (Ergebnis-, Signal-ID-Typen), liegen in `types`/`ports`, nie doppelt. (4) Ein Kern-Crate, das ein Rand-Crate als Abhängigkeit führt, verletzt INV-CORE-ISOLATION-1 auch dann, wenn die Kante nur für eine Teilfunktion gebraucht wird.

### 4.8 Der Kern ist geschlossen

Ein Crate oder Feature betritt den Kern nur per ADR (§18.6) und erst, wenn die Evidenz des bestehenden Kerns die Zielstufe (§1.4, E4) erreicht hat. `[R]`/`[E]`-Code DARF Beweislast, CI-Zeit oder Angriffsfläche des Kerns nicht vergrößern (EP14). Prüfbar über REQ-ARCH-003 und REQ-ARCH-012.

### 4.9 Produktschicht außerhalb des Kerns (E34)

`contextra-ingest` (PDF, DOCX, E-Mail → Text mit Quellenangabe; Zip-Bomb-Schutz und Größenlimits; jeder Extraktor ist Fuzz-Ziel) und die **Tauri-App** sind Klasse `[R]`, importieren nur `contextra` und `contextra-db` (ingest zusätzlich `types`) und zählen nicht zum Kernbudget. **Kein `[K]`-Crate importiert `ingest` oder App-Code** (P30, Kipptest E34). Formatparser in Ring 0–3 sind verboten; bis zur Aktivierung von `ingest` bleibt Ingestion im Kern auf Text/Markdown beschränkt (§10.3).

---

## 5. Ports (Vertragsschicht)

### 5.1 Grundsatz

Ports sind die **einzigen** Schnittstellen zwischen Kern und Außenwelt. Konkrete Implementierungen wählt nur die Fassade (§13.4). Ports werden **genutzt und verdrahtet, nicht ersetzt**: Es gibt bereits `EmbeddingProvider`, `KmsProvider`, `Clock`, `Rng`, `IdGen`, `MetricsSink`, `observability` (K-5). Die Methodenauswahl benennt den Vertrag, nicht die vollständige Signatur. Die Namen des Ports sind die Namen im Code: `IdGen` (nicht `IdGenerator`).

### 5.2 Port-Katalog (v1.0)

Spalte „Sicherheitsport“ = geführt in `governance/security_ports.toml` (INV-WIRING-2).

|Port|Aufgabe / zentrale Methoden|Zustand heute|Entscheidung v1.0|Sicherheitsport|
|---|---|---|---|---|
|`StorageEngine` (+ `StorageRead` sync / `StorageWrite` async)|Schlüssel-Wert-Zugriff mit Snapshot- und SSI-Verfolgung: `get`, `get_at_seq`, `get_tracked`, `scan_prefix_tracked`, `supports_ssi_tracking`, `put`, `put_if_absent`, `put_batch`|vorhanden (`BoxFuture`)|behalten; Teilung durchsetzen (§6)|nein|
|`Snapshot` / `Checkpoint`, `CheckpointCoordinator`|Sequenzstand; Snapshots, benannte Checkpoints, Restore|`Snapshot` UNWIRED|verdrahten (Restore-Guard) oder löschen|nein|
|`VectorIndex`, `TextIndex`|`insert`, `insert_batch`, `search`, `search_at`, `search_filtered`, `delete`, `commit`, `rollback`|vorhanden|behalten|nein|
|`HybridSearchProvider`|`search_hybrid`|vorhanden|behalten|nein|
|`EmbeddingProvider`|`provider_name`, `embed`, `embed_batch`, `embedding_dim`, Modellidentität (§11.1)|vorhanden; `MockEmbedder` im Produktionscode|behalten; Mock nach `testkit`|nein|
|`Clock`|`now_unix_nanos`, `monotonic_nanos`|vorhanden|**verbindlich** statt `Instant::now` (5 Stellen in `core_rebuild.rs`; 49 `determinism`-Verstöße)|nein|
|`Rng`|`next_u64`, `fill_bytes`, `next_unit_f64`|vorhanden|behalten; HNSW-Aufbau seeded|nein|
|`IdGen`|`next_id`|vorhanden|Namen vereinheitlichen|nein|
|`MetricsSink`|`record_counter/gauge/histogram`, keine Nutzinhalte|**NULL_ONLY**; `TestMetricsSink` im Produktionscode|echte Senke (strukturiertes Log) im Composition Root; Test-Senke raus|nein (Null-Objekt zulässig)|
|`observability`, `lifecycle`|Beobachtung/Lebenszyklus|vorhanden|behalten|nein|
|`KmsProvider`|erzeugen, ableiten, wrappen, entwrappen, vernichten (liegt heute in `crypto/src/wal_crypto.rs:18`)|**UNWIRED**|in `ports` verschieben, K0/K1 verdrahten|**ja**|
|`Vfs`|gesamter Dateizugriff mit Durability-Primitiven: Öffnen, Lesen, Schreiben, `sync_file`, `sync_dir`, atomares Umbenennen, Entfernen, Sperren|Implementierung in `store`|verdrahten; `FaultVfs` in `testkit`|**ja**|
|`DurableLog`|Append-only-Log mit `fsync` je Eintrag: `append`, `replay`, `truncate_checked`|Implementierung in `store`|verdrahten|**ja**|
|`ErasureJournal`|durables Saga-Journal: `append` (inkl. `fsync`), `replay`|Implementierung in `store`|verdrahten|**ja**|
|`LayerScanner`, `Purger`|Messung und Bereinigung je Schicht (§5.3)|**neu**|neu|**ja**|
|`GroupRevocationListener`|Widerruf von Gruppenberechtigungen: `on_revoked`|vorhanden|behalten|**ja**|
|`BlockCache`|`get`, `insert`, `evict`; Standard SIEVE|in `store`|behalten|nein|
|`MergeOperator`|`full_merge`, `partial_merge`; Rust (statisch) oder WASM `[R]`|in `store`|behalten (§7.5)|nein (WASM über `ModuleVerifier`)|
|`CompactionPlanner`, `WalObserver`, `ReplayProgressSink`|Hintergrund-Planer; CDC (inhaltsfrei, Cursor); Replay-Fortschritt|vorhanden|behalten|nein (Null-Objekt zulässig bei `ReplayProgressSink`)|
|`kv`|KV-Port|vorhanden|prüfen, ob Rand|nein|
|`Reranker`, `QueryRewriter`|Cross-Encoder-Reranking; Multi-Step-Query-Umschreibung|vorhanden|Rand|nein|
|`ModuleVerifier`, `FilterModule`|WASM-Admission/Filter|vorhanden|Rand (`sandbox`)|**ja** `[R]`|
|`EntityRecognizer`, `InjectionDetector`, `EgressChannel`|Egress-Erkennung und -Kanal|vorhanden|Rand (`privacy`)|**ja** `[R]`|
|`LicenseGate`|`check_ring`, `authorize`|`license` `[E]`|nur Edge|nein|
|`plugin`|Plugin-Port|TEST_ONLY|archivieren|–|
|`graph`, `graph_index`, `attention`, `kv_bridge_port`, `TextGenerator`, `GraphIndex`, `KvPrefixStore`, `AdaptiveController`, `Calibrator`, `GroundingValidator`|Ports zu archivierten Crates|teils UNWIRED|**mit den Crates ins Archiv**|–|

**Nicht Teil von v1.0:** `KeyCustodian`, `ErasureParticipant` (ersetzt durch `LayerScanner` + `Purger`), `NotaryPort`, Sidecar `ErasureKeyStore`, `EpochKeyStore`. **Neu in 2.3 (Sicherheitsports, in `security_ports.toml` zu führen):** `EffaceableKeySlot` (§12.2.3), `ErasedSetMirror` (§12.17.3); **Nicht-Sicherheitsports:** `RecipientLedger` (§12.19.2); **Rand `[R]`:** `TimestampAuthority` (§12.7.9).

### 5.3 Die zwei neuen Ports

```rust
/// Jede Schicht, die Nutzerdaten hält, misst selbst.
pub trait LayerScanner: Send + Sync {
    fn layer(&self) -> DeletionLayer;
    fn scan(&self, ctx: &RequestContext, scope: &ErasureScope, method: ScanMethod)
        -> Result<LayerWitness, ScanError>;      // Witness trägt gescannt/verbleibend/Methode/Digest
}
/// Physische Bereinigung je Schicht.
pub trait Purger: Send + Sync {
    fn layer(&self) -> DeletionLayer;
    fn purge(&self, ctx: &RequestContext, scope: &ErasureScope) -> Result<PurgeStats, PurgeError>;
}
```

`LayerWitness` hat keinen öffentlichen Konstruktor außerhalb der Scanner-Implementierungen der besitzenden Crates (A3, INV-WITNESS-MINT-1); damit ist `LayerCleanupProof::new_after_verified_empty(…, 0)` nicht mehr schreibbar; der Witness entsteht nur über `ScanSession::finish` (§12.7.4). Details und Typen: §12.7.

### 5.4 Regeln

|ID|Regel|
|---|---|
|REQ-PORT-1|Kernlogik (Ring 0–3) ruft weder `Instant::now`, `SystemTime::now`, `rand::thread_rng` noch `std::env::var` auf. Ausnahme: kryptografisches Schlüssel-/Salt-Material aus dem System-RNG, dokumentiert in `determinism-allow.toml`.|
|REQ-PORT-2|`KmsProvider` hat keinen Default und keinen Fallback. Fehlender Verwahrer verweigert den Start in B2.|
|REQ-PORT-3|Kein Mock/Fake/Null-Typ eines Sicherheitsports im Produktionscode; das Gate `security-ports` (heute „FEHLT“) prüft jeden Sicherheitsport auf eine produktive Implementierung **und** einen Verdrahtungstest (INV-WIRING-2).|
|REQ-PORT-4|Port-Signaturen sind semver-geprüft (`cargo public-api`); eine inkompatible Änderung verlangt eine neue Port-Version und eine Migrationsregel. Jede Erweiterung ist deterministisch (P28), budgetiert, auditiert, widerrufbar und besteht vor Registrierung eine Konformitätstestsuite.|

### 5.5 Namensauflösung

|Im Text/Code verwendet|Auflösung|
|---|---|
|`IdGenerator`|Port `IdGen`|
|`DurableAppendLog`|Port `DurableLog`|
|`IsotonicCalibrator`, `PidLatencyController`, `CrossEncoderReranker`, `SignedLicenseGate`|Implementierungen von `Calibrator`, `AdaptiveController`, `Reranker`, `LicenseGate` (nur Rand/Archiv)|
|`EmbeddingCache`|Cache-Schicht unter `CachesAndVault` (§12.4), kein Port|
|`AdmittedModule`, `LayerWitness`|Typen, keine Ports|
|`AdaptiveDecayController`, `ContextPrefixEngine`, `MultiStepEngine`, `ContextEditAuditSink`|Archiv-Typen (Anhang B); keine Zuordnung nötig|

Ein Name ohne Zuordnung ist Gate-Fehler (INV-DOC-SINGLE-SOURCE-1).

---

## 6. Nebenläufigkeitsmodell (E15, E29)

**Befund:** `store` hat 117–144 `async fn`, `engine` 162, `vector` 49, `text` 43 (Ring 0 gesamt 92 laut Scan); `StorageEngine` liefert `BoxFuture`. Das Gate `async-io` (Blockieren im async-Kontext) hat 18 Verstöße; das Gate `ring0-async` ist grün, prüft aber nur die `tokio`-Abhängigkeit. Ob die `async fn` in Ring 0 nur Port-Signaturen bedienen, ist nicht gemessen (Spike S-ASYNC).

**Entscheidung (gestufte Grenze):**

1. **Ring 0 bleibt Ziel laufzeitfrei und synchron** (`vector`, `text`, `rank`, `crypto`, `mvcc`, `types`: kein `async fn`, kein `tokio`, kein `BoxFuture`). Der **Umbau des Bestands ist zurückgestellt** (E29), in v0.1 und v1.0 nicht Pflicht. Es gilt ein **Ratchet:** keine neue `async fn`, keine neue `tokio`-Abhängigkeit in Ring 0; der Bestand darf nur sinken. Spike S-ASYNC ist reine Messung: Sind ≥ 70 % der `async fn` ohne Verhaltensänderung entfernbar, erfolgt der Umbau in Phase 4; sonst wird E15 in Ring 0 auf „Async-Fassade über synchronem Kern“ reduziert und neu entschieden.
2. **`store` (Ring 1)** darf async bleiben, aber ausschließlich an der Port-Grenze `StorageWrite`; lesende Pfade sind synchron (`StorageRead`). Blockierende Datei-I/O in `async fn` läuft über `spawn_blocking` oder einen dedizierten I/O-Thread (die 18 Verstöße werden behoben).
3. **`engine` (Ring 3)** hält den Compute-Pool: CPU-lastige Operationen (Embedding, Fusion, HNSW-Suche) laufen auf dem Blocking-Pool, nie auf dem Executor-Thread.
4. **Tokio nur an den Rändern:** Die Laufzeit wird im Composition Root bzw. `mcp` erzeugt; Bibliotheksnutzer ohne Async erhalten eine synchrone Fassade (`block_on` intern, dokumentiert).
5. **Abbruch (INV-CANCEL-1):** `RequestContext` trägt `CancelToken` und `Deadline`; synchrone Schleifen prüfen sie an definierten Punkten (HNSW-Suche je Ebene, Kompaktion je Block, Saga-Schritte). `forget` ist abbrechbar nur **zwischen** Saga-Schritten.
6. **Sperrenordnung (INV-LOCK-ORDER-1):** keine Locks über `await`; `clippy::await_holding_lock` auf `deny`; loom-Tests für `mvcc` und `engine/collection/kv_lock.rs` (§8.2).

|ID|Anforderung|
|---|---|
|REQ-ASYNC-1|Ratchet (Stufe B): `rg "async fn\|BoxFuture"` in den Ring-0-Crates steigt nie; `cargo tree -p <ring0-crate>` zeigt keine neue Kante zu `tokio` oder `futures-executor`. Endziel 0 (Ausnahmeliste ADR-pflichtig) nach Phase 4.|
|REQ-ASYNC-2|Gate `async-io` hat 0 Verstöße und prüft Blockieren im async-Kontext zeilenweise (heute heuristisch).|
|REQ-ASYNC-3|Jede `spawn`-Stelle hat eine Obergrenze (Backpressure); Gate `unbounded-spawn`.|

---

## 7. Persistenz (`contextra-store`, `contextra-checkpoint`) `[K]`

### 7.1 Datenpfad und Blob-Segmente

`put/delete/merge` → TxBuffer → **WAL-Batch (Group Commit, ein Commit-Punkt)** → MemTable → Flush → SSTable (L0) → Kompaktion (L1…Ln). Werte ≥ `blob_threshold` (Start 64 KiB) gehen als **Blob-Segment** statt durch den WAL-/LSM-Wertpfad. Lesen: MemTable → immutable MemTables → SSTables (Bloom-gefiltert) mit Block-Cache. Alle Werte sind Dokument-Records (eine Verschlüsselungseinheit, §12.2).

**Blob-Segmente:** append-only Werte; LSM/WAL halten nur Zeiger + BLAKE3-Digest. **Reihenfolge:** Segment durabel flushen → WAL-Commit mit Zeiger → Ack. Orphan-Segmente (Absturz dazwischen) räumt Recovery per Manifest ab. Jeder Record im Segment liegt unter dem DocDek seiner Betroffenen-Einheit (§12.2.5); die Segmentdatei wird bei der Hygiene neu geschrieben (§12.8.2 Schritt 8).

### 7.2 Dateiformate

| Format | Magic | Inhalt |
| --- | --- | --- |
| WAL | `MFW3` (Ops `IndexIntent`/`IndexApplied`) | Records: Sequenznummer, Länge, `scope_id`, `key_id`, Nonce, AEAD-Ciphertext, **HMAC-SHA256-Kette** (jedes Record bindet den Vorgänger-Tag). Segmente vorab reserviert. |
| SSTable | `MFSX` (Legacy `MFST`, Entscheidung in WP 2.4) | Datenblöcke, Index-Block, Bloom-Filter, Footer mit Prüfsumme/Tag, AEAD je Block (Engine-Schlüssel); Werte darin je Record unter dem DocDek der Einheit. |
| Blob-Segment | `MFBL` | Append-only Werte, Kopf mit Digest; je Record ein Schlüsselbezug auf den DocDek der Einheit. |
| Manifest | `MFMN` | Versionierte Liste der Live-Dateien je Level, **High-Water-Mark** (höchste bestätigte WAL-Sequenz) gegen Truncation. |
| Keystore | `MFKS` | `KdfHeader` (Kennung `MFKD`), gewrappte Wurzelschlüssel; bei Betriebsart K0 der Integritätswurzelschlüssel im Klartext. |
| Sidecar-Logs | `MFKR`, `MFRV`, `MFAU`, `MFEJ`, `MFES` | `KeyRegistry`, `RevocationLog`, `AuditChain`, `ErasureJournal`, `ErasedSet` (§7.9). |

**Formatversionierung (INV-FORMAT-1):** Magic **und** Formatversion in jeder Datei; neuere als unterstützte Version ⇒ Öffnen verweigert (kein stilles Herabstufen); Downgrade-Angriffe (Ersetzen durch ältere Version) werden abgewiesen. Ältere Versionen öffnet nur ein expliziter, getesteter Migrationspfad: `contextra migrate` erzeugt zuvor ein verifiziertes Backup und bietet `--dry-run`. Pro Release liegt ein **Kompatibilitätskorpus** (Golden-Files je Format und Version) im Repository; die CI öffnet ihn mit dem aktuellen Code.

### 7.3 WAL-Garantien

- **Commit-Punkt = durabler Flush des Batches** (Primitive je Plattform, §7.7). Ack erst danach.
- **Group Commit:** Mehrere Transaktionen teilen sich einen Flush; Latenz/Durchsatz über `PerformanceProfile` (§13.7).
- **Kettenprüfung beim Recovery:** Bruch der HMAC-Kette ⇒ Abbruch am letzten gültigen Record (torn tail wird abgeschnitten; Korruption in der Mitte ⇒ fail-closed).
- **Anti-Truncation:** Manifest-High-Water-Mark > WAL-Ende ⇒ Manipulation/Datenverlust wird erkannt, nicht verschwiegen.
- **Index-Atomarität (INV-ATOMIC-IDX-1):** Ein Batch enthält Daten-Ops **und** `IndexIntent{index_id, op}`. `IndexApplied` ist ein **nicht-durabler Hinweis** (kein eigener Flush): Recovery wiederholt Intents idempotent; ein Commit kostet genau einen Flush.
- **CDC:** `WalObserver` erhält geordnete, deterministische Ereignisse mit dauerhaftem Cursor (at-least-once, idempotent konsumierbar). **Ereignisse tragen keine Nutzinhalte** (nur `op`, `scope_id`, opake IDs, `seq`), sodass Observer keine Löschkopien halten.
- **`fsync`-Fehler (INV-FSYNC-1)** und **volles Medium (INV-ENOSPC-1)** wie in §3.4. **Backpressure:** erreichen L0-Dateien oder MemTable-Bytes die Schwellen aus §7.4, werden Schreiber gedrosselt statt Speicher unbegrenzt zu belegen.

**7.3.1 Recovery-Algorithmus (Speicher-Teilfolge von B4–B7)**

1. DirLock erwerben. Kann das Dateisystem keine exklusive Sperre garantieren (Netzwerk-/Cloud-Sync-/unbekannte FUSE-Dateisysteme), wird das Öffnen im persistenten Betrieb verweigert; nur `MemoryOnly` läuft ohne Sperre.
2. Keystore und Manifest laden und prüfen.
3. WAL ab letztem Checkpoint kettenvalidieren.
4. Records in MemTable replayen; offene `IndexIntent`s vom Applier idempotent anwenden (INV-INDEX-APPLY-1); Embedding-/LLM-Werte stammen aus dem WAL (INV-NONDET-1).
5. High-Water-Mark abgleichen.
6. Sidecar-Logs validieren und gegen ihre Manifest-Marken abgleichen; ausstehende `SidecarIntent`s aus dem WAL idempotent anwenden.
7. `ErasedSet` laden und Lesesperre armieren (INV-BOOT-1); `ErasureJournal` replayen und offene Löschsagas zur Fortsetzung vormerken (§12.8).
8. Retention-Entscheidungen (WAL-Records, §11.6) in ihren `PendingDeletion`-Zustand überführen.
9. Erst dann Lese-/Schreibverkehr zulassen; Lesen eines durch den `ErasedSet` gesperrten Betroffenen bleibt auch danach verweigert.

Poison-Locks werden erkannt und über definierte Wiederherstellung behandelt. Die Checkpoint-Frequenz begrenzt die Recovery-Zeit auf ein je Tier festgelegtes Budget (§16).

### 7.4 Kompaktion

- **Tiered** und **adaptiv** (`CostBasedAdaptivePlanner`): minimiert Write-/Read-/Space-Amplification anhand gemessener Workload-Statistiken.
- Tombstones werden erst auf dem untersten Level bzw. nach Snapshot-Horizont verworfen (MVCC-sicher); heute nur bei Vollkompaktion mit `raw_seq < min_snapshot_seq` (`compaction/engine.rs:712–720`) – Fenster mit Restdaten bis zur Kompaktion ist belegt (Spike E1).
- **Hygiene (Saga-Schritt 8)** erzwingt bereichsweise **Purge-Kompaktion** betroffener SSTables (`compact_range(scope)`, §12.8.2). Sie ist nicht mehr Voraussetzung der Schlüsselvernichtung, aber Voraussetzung der Klasse M.
- Kompaktion ist budgetiert (I/O-Rate, CPU) und vom `SystemPressureMonitor` drosselbar.
- **Write-Stall-Policy:** ab `l0_slowdown` (Start 8) werden Schreiber gedrosselt; ab `l0_stop` (Start 20) warten sie höchstens `write_stall_timeout` (Start 5 s) und erhalten danach den retrybaren Fehler `Busy`. Rückstau ist **kein Betriebszustand**: er ändert weder Gesundheitszustand noch Garantien.

### 7.5 MergeOperator (IoC auf Schreibsemantik)

Port `MergeOperator { full_merge, partial_merge }`. Implementierungsarten: **Rust** (statisch, generisch) und **WASM** `[R]`. Verbindliche **CRDT-Gesetze** (Assoziativität, Kommutativität, Idempotenz soweit zutreffend), property-getestet für **LWW-Register**, **G-Counter**, **OR-Set**. Ein WASM-Operator wird nur registriert, wenn er die Gesetz-Suite besteht und die Determinismus-Konfiguration (Anhang A.1) erfüllt. Operatoren sind deterministisch (P28).

### 7.6 Caches, Druck, I/O-Regeln

- **Block-Cache:** ein Standard (**SIEVE**) hinter dem Port `BlockCache`; Alternativen sind Port-Implementierungen. Bloom-Filter je SSTable (konfigurierbare FPR); Prefix-Bloom für Range-Scans.
- **`SystemPressureMonitor`** (über `sys`): bei Speicherdruck werden Caches verkleinert und Kompaktion gedrosselt – kontrolliert, nie OOM-Kill.
- **I/O (INV-IO-1):** Standard ist `pread` mit Prüfsumme; entschlüsselte Blöcke leben als `Bytes` im Block-Cache (Zero-Copy ab dort). **`mmap`** nur für unveränderliche, vollständig prüfsummenverifizierte Dateien auf lokalem Volume (`doctor` prüft Dateisystemtyp); Verkürzen/Überschreiben durch Fremdprozesse wird durch DirLock und Rechte ausgeschlossen. Verschlüsselte Dateien werden nicht zero-copy gemappt (INV-ENCRYPT-REST-2).

### 7.7 Durability-Primitive je Plattform (INV-DURABILITY-PRIM-1)

| Plattform | Datei-Flush | Verzeichniseintrag (Create/Rename) |
| --- | --- | --- |
| Linux | `fdatasync` (Metadaten ändernd: `fsync`) | `fsync` auf das Elternverzeichnis |
| macOS | `fcntl(F_FULLFSYNC)` (reines `fsync` garantiert keine Durabilität) | `F_FULLFSYNC` auf das Verzeichnis |
| Windows | `FlushFileBuffers` | `MoveFileEx(MOVEFILE_WRITE_THROUGH)`/`ReplaceFile` |

Jede neue oder umbenannte Datei (WAL-Segment, SSTable, Manifest-Wechsel) gilt erst nach Flush des Elternverzeichnisses als existent. Sämtlicher Dateizugriff läuft über den Port `Vfs`; die Implementierung liegt in `store` (`fsync.rs`), die Fehlerinjektion `FaultVfs` in `testkit`; beide bilden diese Primitive ab.

### 7.8 Betriebszustände und Gesundheitsübergänge

| Zustand | Bedeutung | Eintritt | Verlassen |
| --- | --- | --- | --- |
| `Healthy` | alle Garantien aktiv | nach erfolgreicher Recovery | – |
| `Degraded` | nicht-kritischer Teil ausgefallen (Block-Cache, `[R]`-Feature, Hintergrundkompaktion); Kern-Garantien bleiben | Ausfall einer abgeleiteten Komponente | Komponente gesund oder Neustart |
| `ReadOnly` | Lesen erlaubt, Schreiben abgelehnt (retrybar); Löschen über die Notreserve zulässig (§7.10) | `ENOSPC` bei Rotation, `fsync`-Fehler auf einem Sidecar-Handle, Lock-Poisoning in Schreibstrukturen mit sicherer Wiederherstellung | Platz frei / Neustart |
| `Poisoned` | Durabilität unbestimmt: kein Ack, kein Schreiben | `fsync`-/`fdatasync`-Fehler am WAL (INV-FSYNC-1), unwiederherstellbarer Poison-Lock | ausschließlich Neustart mit Recovery-Replay |

Regeln: (1) Übergänge sind monoton in Richtung `Poisoned`, außer durch Neustart bzw. dokumentierte Selbstheilung `Degraded → Healthy`. (2) Jeder Übergang wird auditiert (ohne Nutzinhalte) und über `doctor` sichtbar. (3) Ein Poisoned-Lock ist ein typisierter Fehler mit definiertem Übergang, nie `unwrap`. (4) Unbekannte Zustände ⇒ fail-closed. (5) Prüfung: REQ-DUR-014.

### 7.9 Sidecar-Logs: die einzigen Persistenzdomänen neben dem WAL

Der WAL ist die einzige Wahrheit für Anwendungszustand (P24). Sicherheitszustand kann nicht aus dem WAL abgeleitet werden, weil S3 WAL-Segmente scrubbt, und muss Löschungen **überleben**. Er liegt deshalb in genau diesen Dateien; die Liste ist **abschließend** (INV-SIDECAR-1):

| Sidecar | Magic | Inhalt | Primärer Schreiber | Kettenschlüssel (§12.3) | Im Backup |
| --- | --- | --- | --- | --- | --- |
| `KeyRegistry` | `MFKR` | Schlüsselereignisse: Erzeugen, Wrap, Rotation | `KmsProvider` über `KeyManager` | `K_sc/keyreg` | ja (nur Ereignisse, keine Schlüssel) |
| `RevocationLog` | `MFRV` | Widerruf von Schlüsseln, Modulen, Berechtigungen | Autorisierung, Löschsaga | `K_sc/revoke` | ja |
| `AuditChain` | `MFAU` | Zugriffe, Löschungen, Berichte, Freigaben; tenant-getrennt | Applier (aus `SidecarIntent`) und Sicherheitspfade | `K_sc/audit` | ja |
| `ErasureJournal` | `MFEJ` | Saga-Übergänge (§12.8) | Orchestrator | `K_sc/journal` | nein (Betriebszustand) |
| `ErasedSet` | `MFES` | Referenz-Hashes gelöschter/gesperrter Betroffener und Einheiten samt Referenz-Epoche (§12.9, §12.10); wird an den `ErasedSetMirror` gespiegelt (§12.17.3) | Orchestrator | `K_sc/erased` | **nein** (§7.9 Regel 5) |
| Keystore | `MFKS` | `KdfHeader`, gewrappte Wurzelschlüssel | `KmsProvider` (lokal) | (Integritätswurzel) | **nein** (Schlüsselmaterial) |

Regeln:

1. **Form.** Jedes Sidecar ist append-only, trägt Magic und Formatversion, eine HMAC-Kette mit eigenem Zweckschlüssel und einen `fsync` je Eintrag über den Port `DurableLog`. Die **Manifest-Marke** (Name, letzte Sequenz, Kettenkopf) wird bei jedem Checkpoint und jeder Rotation aktualisiert; sie erkennt Tail-Truncation und Ersetzen durch eine ältere Version. Das Löschen eines Sidecars bei vorhandenen Daten ist Manipulation (fail-closed).
2. **Intent-Muster (WAL zuerst).** Wirkungen, die ein WAL-Commit auslöst (Audit-Einträge zu Datenoperationen, Lineage-Hinweise), stehen als `SidecarIntent{log, payload_hash}` im selben WAL-Batch (wie `IndexIntent`). Der Applier schreibt sie nach dem Commit idempotent über `(log, seq)`; Recovery wiederholt offene Intents (§7.3.1 Schritt 6). Ein Commit kostet weiterhin genau einen Flush.
3. **Sidecar vor Wirkung.** Sicherheitskritische Wirkungen, die nicht aus einem WAL-Commit folgen, werden zuerst im Sidecar durabel und erst danach wirksam: Autorisierungswiderruf (`RevocationLog`), Saga-Übergang (`ErasureJournal` **vor** Ausführung des Schritts), `ErasedSet`-Eintrag (**vor** der ersten Löschoperation).
4. **Autorität.** Sidecars sind nicht aus dem WAL rekonstruierbar und deshalb autoritativ; ein beschädigtes Sidecar in der Mitte der Kette ist fail-closed (Start verweigert, `doctor` benennt Datei und Sequenz).
5. **Backup.** Checkpoints enthalten `KeyRegistry`, `RevocationLog` und `AuditChain`, aber weder `MFKS` noch `ErasedSet`. Der `ErasedSet` gehört dem laufenden System: Der Restore-Guard liest den `ErasedSet` des **Zielsystems**.
6. **Ausfall.** Ein `fsync`-Fehler auf einem Sidecar-Handle vergiftet genau diesen Handle (INV-FSYNC-1) und führt gemäß §7.10 zu `ReadOnly` bzw. `Poisoned`.

### 7.10 Notlöschpfad und Poison-Umfang

Löschen ist eine Schutzfunktion und darf nicht an Betriebszuständen scheitern, die Löschen selbst auslösen (volles Medium, `fsync`-Fehler).

| Ereignis | Zustand | Löschen erlaubt? |
| --- | --- | --- |
| `ENOSPC` bei WAL-Rotation | `ReadOnly` | **Ja.** Tombstone-Batch, Journal-Einträge und `ErasedSet`-Eintrag nutzen die Notreserve (`emergency_reserve_bytes`, Start 64 MiB, mindestens zwei WAL-Segmente plus Journal, bei Start vorab reserviert). S2 (Purge-Kompaktion) braucht Platz: Schlägt sie mit `StorageFull` fehl, bleibt die Saga `Partial{outstanding: Lsm}` und setzt nach Freigabe fort; der Bericht weist das aus. |
| `fsync`-Fehler am WAL-Handle | `Poisoned` | **Nur Sperr-only:** `RevocationLog`, `ErasedSet` und `ErasureJournal` über ihre **eigenen, gesunden** Handles; keine WAL-Schreibzugriffe. Der Tombstone-Batch wird nach dem Neustart von der Saga aus dem Journal nachgeholt. |
| `fsync`-Fehler an einem Sidecar-Handle | `ReadOnly` (Funktion fail-closed) | Betroffen ist genau die abhängige Funktion: `RevocationLog` ⇒ keine Autorisierung neuer Schreibvorgänge; `ErasureJournal` ⇒ keine neue Saga, aber Lesesperre im RAM; `KeyRegistry` ⇒ keine Schlüsselerzeugung. |
| Ausfall aller Löschpfad-Handles | `Poisoned` | Prozesslokale Lesesperre für den angefragten Betroffenen (INV-EMERGENCY-DEL-1); Retry nach Neustart. |

Leseverkehr im Zustand `Poisoned` ist nur zulässig, wenn der `ErasedSet` armiert ist (INV-BOOT-1); andernfalls verweigert der Zustand auch das Lesen. Der Poison-Umfang ist pro Persistenzdomäne definiert und monoton wie in §7.8.

### 7.11 Format-Register und Regeln

Ein Register `docs/spec/FORMATS.md` führt jedes persistente Artefakt (Gate `format-registry`: jede Datei im Datenverzeichnis hat einen Eintrag). Soll-Stand (Spalte „Ist-Befund“ ist in WP 2.4 aus dem Code auszufüllen; „prüfen“ = nicht gelesen):

| Artefakt | Magic | Version | Prüfsumme | Ist-Befund |
| --- | --- | --- | --- | --- |
| SSTable | `MFSX` (Legacy `MFST`) | prüfen | prüfen | beide Magics im Code (`sstable/block_cache.rs:340–341`); Legacy-Pfad migrieren oder entfernen |
| Manifest | `MFMN` | prüfen | prüfen | `manifest/core.rs:17` |
| WAL | `MFW3` | prüfen | HMAC (`wal/hmac.rs`) | Panic `hmac.rs:194`, `wal/mod.rs:92` |
| Löschbericht (`DeletionProof`) | `MFDR` | V5 (explizites Layout, kein `serde`-Derive); V1–V3 nur lesbar; V4 bleibt unvergeben (E46) | Ed25519 | im Repo höchstens V3 (U-39); Golden-File Pflicht (REQ-PRF-5) |
| KeyTable | `MFKT` | prüfen | Blake3 | neu in 2.3; Ersetzen-Semantik, atomar (REQ-FMT-5), §12.2.9 |
| `ErasedSetMirror`-Eintrag | `MFEM` | prüfen | HMAC | neu in 2.3, §12.17.3 |
| `AuditChain`, `RevocationLog`, KV-Segment | prüfen | prüfen | prüfen | bincode (v1.3.3); Versionsfeld offen |
| HNSW-Snapshot, Text-Segmente | prüfen | prüfen | prüfen | nicht gelesen |
| Checkpoint-Manifest | – | – | Blake3, signiert | – |
| Wire (FlatBuffers) | – | Schema `schemas/contextra.fbs`, `flatbuffers 24.3` | – | generierter Code eingecheckt; `flatc` fehlt in der Analyseumgebung; Gate `check-flatbuffers-drift` existiert |

**Umfang in v1.0 (E30):** Pflicht für WAL, SSTable, Manifest, `AuditChain`, `RevocationLog`, Checkpoint-Manifest. HNSW-Snapshot, Text-Segmente, KV-Segment und Wire folgen, sobald ihre Leser gelesen sind (WP 2.4).

| ID | Regel |
| --- | --- |
| REQ-FMT-1 | Jedes persistente Artefakt beginnt mit Magic (4 Byte), Formatversion (u16), Flags, Längenfeld und trägt eine Prüfsumme (CRC32C für Blöcke, Blake3 für Manifeste/Segmente). |
| REQ-FMT-2 | Ein Leser verweigert **fail-closed** jede Version, die er nicht kennt; der Schreiber schreibt nur die aktuelle Version. |
| REQ-FMT-3 | Für jede unterstützte Version liegt ein **Golden-File** im Repo; Round-Trip-Tests laufen in CI. |
| REQ-FMT-4 | Serde-abgeleitete Layouts sind für signierte oder persistente Daten nur zulässig, wenn ein Golden-File sie festschreibt und die Serialisierungsbibliothek exakt gepinnt ist. |
| REQ-FMT-5 | Atomares Ersetzen: temporäre Datei, `fsync`, `rename`, `fsync` des Verzeichnisses (Linux `fdatasync`, macOS `F_FULLFSYNC`); Fehler werden propagiert. Das gilt auch für den Keystore. |
| REQ-FMT-6 | Migration läuft offline über `contextra migrate` mit Trockenlauf und vorheriger Sicherung; keine stille Migration beim Öffnen. |
| REQ-FMT-7 | Vor 1.0 sind inkompatible Formatwechsel erlaubt, wenn `migrate` sie abdeckt; ab 1.0 sind On-Disk-Formate stabil und nur über neue Versionsnummern erweiterbar. |

### 7.12 Datenverzeichnis

Beim Start (B1) wird das Dateisystem geprüft (INV-PLATFORM-1: unterstützt lokale ext4/xfs/btrfs/zfs/APFS/NTFS; **abgelehnt:** NFS/SMB, unbekannte FUSE, Cloud-Sync-Ordner wie OneDrive, Dropbox, iCloud Drive) und die Verzeichnissperre gesetzt (INV-IPC-1). `doctor` meldet Abweichungen mit benannter Ursache. Weder Netzwerk- noch Cloud-Sync-Dateisysteme werden im persistenten Betrieb zugelassen.

---

## 8. MVCC und Serializable Snapshot Isolation (`contextra-mvcc`) `[K]`

### 8.1 Komponenten

- **SequenceLog:** monoton wachsende, WAL-gebundene Commit-Sequenz.
- **SnapshotRegistry:** aktive Snapshots; bestimmt GC-/Tombstone-Horizont. Ein Snapshot ist **immer ein konkreter Sequenzstand, nie `u64::MAX`**.
- **TxBuffer:** privater Schreibpuffer, Read-your-writes.
- **ReadSet:** erfasst Punkt-, **Prefix-/Range-Lesezugriffe (Phantomschutz)**; SSI-Validierung bei Commit erkennt gefährliche Strukturen (rw-Antidependenzen) ⇒ retrybarer `SerializationFailure`. Ob SSI tatsächlich aktiv ist, ist unbelegt (`supports_ssi_tracking` liefert konstant `true`, U-06); Nachweis durch T-SSI-1 oder Anspruch streichen.
- **Tenant:** aus der Transaktion abgeleitet; Schlüsselkonstruktion ohne Tenant-Kontext ist typbedingt unmöglich.
- **Kein Saga-/Kompensationsmodell** für Datenoperationen: der Normalfall ist atomar über den WAL-Batch; Fehler nach dem Commit-Punkt werden **vorwärts** behoben (Recovery wendet Intents erneut an), nie kompensiert. (Die Löschsaga, §12.8, ist ein eigener durabler Ablauf.)
- **Zeitreisen:** `as_of(seq | zeit)` liest historische Snapshots, solange nicht kompaktiert/gelöscht. **Löschung wirkt rückwirkend (INV-ASOF-ERASE-1):** Ein Betroffener im `ErasedSet` oder in `Frozen` ist auch in jedem historischen Snapshot unlesbar, selbst wenn S2/S3 noch `Pending` sind. Ein Snapshot hat das Höchstalter `snapshot_max_age` (Start 15 min); danach wird er als `SnapshotExpired` (final) ungültig, nicht stillschweigend verlängert (REQ-DEL-031).

| Lesetyp | Isolation |
| --- | --- |
| Punkt-/Prefix-/Range-Reads auf dem Keyspace | SSI (Phantomschutz) |
| ANN-, BM25-Suchen | Snapshot-Isolation via `born_seq`/`died_seq` (INV-INDEX-SNAPSHOT-1); approximative Suchen haben keinen definierbaren Phantomschutz |
| Optional `serializable_index_reads = true` | Index-Lesen registriert einen Collection-Marker im ReadSet; jeder konkurrierende Schreiber derselben Collection löst Konflikt aus (konservativ, kann Abbrüche erhöhen) |

### 8.2 Sperrenordnung (INV-LOCK-ORDER-1)

Alle Sperren der Engine (`collections`, Key-Sperren, Embedder, Hooks, Konfiguration, Uhr, Metriken) stehen in **einer** vollständigen, geordneten Hierarchie; nur diese Ordnung gilt beim Erwerb mehrerer Sperren. Mehrschlüssel-Operationen sperren Shards in aufsteigender Reihenfolge und deduplizieren sie; Shard-Hasher nutzen feste Seeds. Der Sperrtyp je Ebene (Mutex oder RwLock) ist dokumentiert und durch einen Test festgeschrieben. Guards von `std`/`parking_lot` DÜRFEN NICHT über `.await` gehalten werden (`await_holding_lock = deny`). Für jede nebenläufige Kernstruktur (SnapshotRegistry, TxBuffer, RevocationLog, HNSW-Arena, Group-Commit, Key-Sperren) existiert ein loom-Modell (§17.1).

---

## 9. Vektor-Subsystem (`contextra-vector`) `[K]`

### 9.1 HNSW

Arena-Allokation, Backlinks, **adaptives ef**, Startwerte **m = 16, efConstruction = 200, efSearch = 64**, `MAX_SEARCH_K = 1000` (zu kalibrieren). Persistenz: WAL (`IndexIntent`) + Snapshot der Graphstruktur; Wiederaufbau deterministisch (INV-INDEX-APPLY-1: ein Applier in WAL-Reihenfolge, Level aus `hash(doc_id, seed)`; `Clock`/`Rng` über Ports, keine `Instant::now`-Aufrufe wie heute in `core_rebuild.rs`).

### 9.2 Mandantentrennung (INV-TENANT-IDX-1)

Je **(Tenant, Collection, Embedding-Modell-Hash)** existiert ein eigener Index (eigene Datei, eigener Schlüssel). Ein gemeinsamer Graph über Mandanten ist unzulässig (Kanten wären mandantenübergreifende Datenflüsse, VETO-F10). Der ACL-Bitset (§9.4) wirkt **innerhalb** eines Tenants (Rolle/Zeile). Suchlatenz und Recall eines Mandanten hängen nicht von anderen ab.

### 9.3 Löschung (VETO-F02) und Slot-Wiederverwendung

Nur **Tombstone + Pruning**; Knoten werden nicht „weggeflickt“ (**kein Rewiring der Nachbarn**). Stufe S1: der Vektor-Payload (Vollpräzision **und** SQ8-Code) des Knotens wird **sofort im Slot genullt** und der Index-Snapshot neu geschrieben; der Routing-Knoten bleibt (trägt keine Nutzdaten); Pruning entfernt tote Kanten beim Rebuild. Übersteigt der Tombstone-Anteil `tombstone_rebuild_ratio` (Start 0,2), läuft ein deterministischer Rebuild im Hintergrund.

**INV-HNSW-SLOT-1:** Ein Arena-Slot wird erst wiederverwendet, wenn keine eingehende Kante mehr auf ihn zeigt, oder jede Kante trägt eine Generationsmarkierung. Wird der Einstiegsknoten gelöscht, erfolgt die Neuwahl deterministisch. `partial_rebuild.rs` im Vektor-Crate ist daraufhin zu prüfen, ob es VETO-F02 widerspricht.

### 9.4 Gefilterte Suche, ACL und Planner

- **Planner:** Selektivität `s = |erlaubt| / |Index|`. Ist `|erlaubt| ≤ exact_scan_max` (Start 20 000), läuft ein **exakter SIMD-Scan** über die erlaubten IDs (kein Graph, kein Recall-Verlust). Sonst **ACORN-artige gefilterte Traversierung** (Prädikat während der Traversierung, erweiterte Nachbarschaft, ef skaliert mit 1/s bis zu einer Obergrenze).
- **ACL-Bitset (RLS):** pro Rolle ein Bitset berechtigter Vektor-IDs; **Prüfung vor Distanzberechnung**. Modus `strict`: nur berechtigte Knoten werden bewertet *und* expandiert; fällt dadurch die erreichbare Menge unter `k`, greift der exakte Scan statt einer stillen Unterlieferung. Ob das heute so umgesetzt ist, ist offen (O-5-Bezug, Review D2).
- **Seitenkanäle:** Ergebnisse enthalten keine Scores oder Counts nicht berechtigter Knoten; Latenz wird in Klassen gemessen, nicht als konstant zugesichert.

### 9.5 Metriken (IoC)

Generischer `Metric`-Trait mit Zero-Sized-Types (`Cosine`, `L2`, `InnerProduct`, `Hamming` …) ⇒ **monomorphisierter Hot-Path ohne dyn-Dispatch**. WASM-Metriken `[R]` dürfen ausschließlich im Top-k-Rerank laufen, nie in der Traversierung.

### 9.6 Quantisierung

**SQ8** (Skalar) ist die RAM-Vorstufe; die besten `ef` Kandidaten werden mit den Originalvektoren **exakt nachgerankt**. SIMD-Dispatch AVX-512 > AVX2 > NEON > skalar; skalar ist Orakel (Bit-/ULP-Toleranz getestet); in v1.0 ist der Skalarpfad Standard (E33). Das Ergebnis hängt nie vom Cache-Zustand ab. **RAM-Formel:** `N·D` Byte (SQ8) + Graph `N·R·4` Byte; Beispiel 1 Mio. × 1536 ≈ 1,5 GB SQ8 + Graph statt ≈ 6,1 GB fp32.

### 9.7 Skalierung über RAM hinaus `[A]`

RaBitQ (1-Bit-Codes) und ein DiskANN-artiger Modus (SSD-residente Vollpräzisionsvektoren per `mmap`) sind **archiviert** (`vector/diskann`) und erst relevant, wenn der Workload den Designpunkt (§2.1) überschreitet. Wiederaufnahme nur per ADR; Vorbedingungen: INV-ENCRYPT-REST-2, deterministisches Ergebnis unabhängig vom Hot-Set, `MAX_SEARCH_K` auch im Filterpfad, Filterfaktor `gamma = 0` ⇒ Ergebnis identisch zum ungefilterten Pfad, Tie-Breaker Score absteigend/DocId aufsteigend; kein Rewiring (VETO-F02).

### 9.8 Adaptiver Overfetch

Liefert ein selektiver Filter weniger als `k` Treffer, MUSS das System in deterministischen Stufen nachladen: `3·k → 6·k → 12·k`, bis `k` Treffer erreicht sind oder eine harte Scan-Obergrenze greift. Abbruch ausschließlich über deterministische Zähler, nie über Zeit-/Wall-Clock-Heuristiken (P28). **Abnahme:** Ein Filter mit 1 % Selektivität liefert exakt `k` Treffer oder nachweislich alle im Snapshot vorhandenen passenden Treffer.

---

## 10. Text-Subsystem (`contextra-text`) `[K]`

### 10.1 BM25

**BM25** (Startwerte k1 = 1,5, b = 0,75, per Recall-Benchmark zu kalibrieren), **BM25F** mit Feldgewichten, **Block-Max-WAND**. **Statistiken (IDF, Dokumentlängen) werden je Tenant geführt** (INV-TENANT-IDX-1). Der invertierte Index ist segmentiert, über `IndexIntent` konsistent, je Tenant/Collection getrennt; Löschung per Segment-Neuschreibung mit neuem ArtKey, alter ArtKey sofort vernichtet (§12.2.8, §12.8.2 Schritt 3); Segmentdateien liegen unter dem ArtKey. Terme, die nur in gelöschten Dokumenten vorkamen, MÜSSEN aus dem Termwörterbuch verschwinden (sonst bleibt ein Inhaltsrest).

### 10.2 Analyzer

`Analyzer`-Port: Unicode-Normalisierung (NFKC), Kleinschreibung, Standard-Tokenisierung; Tokenizer ohne Panics. `GermanAnalyzer` `[R]` (Komposita-Zerlegung per Wörterbuch + Fugen-Heuristik, Lemmatisierung, Umlaute/ß) und Fachvokabulare `[R]` (Synonyme, Abkürzungen, Normbezüge, geschützte Begriffe, nie Stoppwörter) sind optional; der Vorteil für den DACH-Markt ist eine Marktannahme, nicht belegt (O-6). SPLADE `[E]` ist nicht Teil von v1.0.

### 10.3 Dokumentformat-Extraktoren `[R]`

PDF-, E-Mail- und DOCX-Ingestion MUSS, sobald aktiviert, mit Zip-Bomb-Schutz und Größenlimits abgesichert sein und in einem eigenen Adapter-Crate außerhalb des Kerns (`contextra-ingest`, §4.9) laufen; Formatparser in Ring 0–3 sind verboten; jeder Extraktor ist Fuzz-Ziel. Bis zur Aktivierung bleibt Ingestion im Kern auf Text/Markdown beschränkt.

---

## 11. Embedding, Engine und Abfragepfad (`contextra-engine`, `contextra-rank`) `[K]`

### 11.1 Embedding-Identität (INV-EMBED-1/-2)

Es gibt keinen stillen Platzhalter-Embedder: Mock-/Fake-Embedder existieren nur hinter `cfg(test)` oder `test-utils` (Gate über `cargo metadata`); fehlende oder unbekannte Provider-Konfiguration ist ein Startfehler. `EmbeddingBackend ∈ {Candle, None}` in v1.0 (`None` = nur Textsuche; ONNX/Ollama archiviert). Jede Collection persistiert `embedding_model_hash` und Dimension im Collection-Manifest; Einfügen und Suche mit abweichendem Hash werden abgewiesen (`EmbedderMismatch`), auch bei gleicher Dimension; **Modellwechsel = Migration** mit Re-Embedding und vollständigem Neuaufbau des Vektorindex (neuer Index je INV-TENANT-IDX-1, atomarer Umschalt-Commit), nie stiller Mischbetrieb (T-EMB-1). Ein Semantik-Selbsttest („Hund“/„Welpe“ näher als „Hund“/„Steuererklärung“) ist festes CI-Gate je Backend. Die Modellkonfiguration wird aus dem Modellverzeichnis geladen, nie durch Defaults ersetzt.

**Modellidentität (REQ-IDX-015):** `embedding_model_hash` ist `BLAKE3` über die kanonische Kodierung von `(Gewichts-Digest, Tokenizer-Digest, Quantisierungskennung, Pooling-/Normalisierungskonfiguration, Dimension, Präfix-Konvention)`; der Typ `ModelIdentity` liegt in `types`, jedes Backend liefert ihn, die Fassade vergleicht ihn bei jedem `insert`/`search`. Zwei Modelle gleicher Dimension haben verschiedene Hashes; ein Backend ohne berechenbare Identität (z. B. Remote-Endpunkt ohne Gewichts-Digest) ist für persistente Collections unzulässig. **Embeddings sind personenbezogene Daten** (Text ist teilweise rekonstruierbar) und werden von den Löschschichten `VectorIndex` und `CachesAndVault` erfasst (§12.4).

### 11.2 Quelle der Wahrheit, Query-Builder, Planner

**Prinzip D1:** Store und WAL sind die einzige Wahrheit. Vektor- und Textindex sind **abgeleitet** und aus dem Store rekonstruierbar. Ein Index darf nie Daten enthalten, die der Store nicht hat; ein Löschvorgang ist erst abgeschlossen, wenn Store **und** Indizes gemessen sind (§12.4); nach Absturz werden Indizes aus dem Store abgeglichen.

- **HybridQueryBuilder:** deklarativ: Vektor + Text + `ScopeConstraint` (Tenant, Domäne, Projekt, Klassifikation) + `ValidityWindow` + `as_of` + Budget. `ScopeConstraint` wird in jede Modalität **vor** der Berechnung gepusht.
- **Planner** (deterministisch, regelbasiert): wählt Modalitäten und Parameter (Text-only, Hybrid) aus Anfrage, Budget und Index-Statistik; gleicher Zustand ⇒ gleicher Plan.
- **Ausführung:** Modalitätszweige im **Bounded-Compute-Pool**; Fusion über `rank/fusion`; kooperative Abbruchpunkte (INV-CANCEL-1).
- **Bi-temporal:** Gültigkeitszeit × Transaktionszeit („Was galt am Datum X, wie wir es am Datum Y wussten?“).
- **Provenienz:** Jedes Ergebnis trägt Lineage (WAL-Seq, Quelle, Chunk, Embedding-Version), `trust` und `source` (INV-TRUST-1).

### 11.3 `remember`

1. Validierung (Schema, Größe, Limits §11.8, Mandant aus `RequestContext`); `trust`/`source` setzen; `MemoryType` (Default `Semantic`).
2. Chunking nach §11.7.
3. Embedding über `EmbeddingProvider` **außerhalb von Sperren** (Blocking-Pool, §6; `max_embed_batch`, bricht bei Abbruch ab); das Ergebnis ist ein aufgezeichneter Wert (INV-NONDET-1). Große Werte: Blob-Segment vor dem Commit.
4. **Ein WAL-Batch:** Primärdaten + `IndexIntent`s (Vektor, Text) + Lineage + `SidecarIntent` (Audit) → durabler Flush (Commit-Punkt, `fsync`-Fehler propagieren; „WAL First“).
5. Der Applier wendet die Indexe an; bei Fehler gilt der Index als „nachzuziehen“ und wird beim nächsten Start abgeglichen.
6. Quittung `RememberReceipt` mit Dokument-ID, Sequenznummer (Provenienz-Anker), `embedder_id`.

### 11.4 `recall`

1. `RequestContext` und Snapshot-Sequenz festlegen; Planner; `ScopeConstraint`/ACL-Bitset.
2. Query-Embedding (Blocking-Pool).
3. Vektorsuche (mandantengetrennter Index, filterbewusst, ACL vor Score) **und** BM25 parallel am Snapshot, beide mit `max_results`-Grenze (Gate `check-max-results-unbound`).
4. Fusion über `rank/fusion` (RRF, Normalisierung, Provenienz), deterministisch bei gleichem Seed.
5. **Lesesperre:** Treffer, deren Dokument im `ErasedSet` steht, werden entfernt, **bevor** irgendetwas zurückgegeben wird (REQ-DEL-011).
6. Ausgabegrenze `max_response_tokens` (Start 4.000); Kürzung wird ausdrücklich gekennzeichnet; Ergebnis `{content, trust, source, lineage}`; Audit-Eintrag.

### 11.5 Fusion und Provenienz (`rank/fusion`)

**RRF** mit `k = 60`; deterministische Tie-Breaker: **Score absteigend, DocId aufsteigend** (REQ-IDX-009). Die Basis-Erklärung je Treffer (Modalität, Score, WAL-Seq, Quelle) ist die Grundlage von M4 und entsteht ausschließlich hier (Kantenregel 10, §4.4). Ohne `Calibrator` (Archiv) gelten rohe RRF-Scores und keine „Unsicher“-Ausgabe.

### 11.6 Retention-Reaper (INV-RETENTION-1)

Ein periodischer Hintergrund-Worker entfernt Einträge, die eine **opt-in `RetentionPolicy`** der Collection nach Karenzzeit (Standard 7 Tage im Zustand `PendingDeletion`, per `restore` rückholbar) erfasst. Batches sind begrenzt (Start 100 Dokumente/Tick), Herunterfahren per Cancellation. Jede Entscheidung wird **als Record ins WAL geschrieben** (Zeit aus `Clock`, Ursache, Policy-Version); die Entfernung folgt der Löschkette (§12.8). Eine Policy ohne Dry-run-Lauf und Audit-Eintrag ist nicht aktivierbar. Ranking-Abwertung (Decay) löscht nie. Ein storage-interner TTL-Mechanismus auf LSM-Ebene, falls vorhanden, ist davon abzugrenzen und zu dokumentieren. *Offene Zuordnung:* ob Reaper und `MemoryType` in `engine`/`types` leben (WP 3.2).

### 11.7 Chunking (MUSS)

Jedes Dokument wird vor Indexierung über einen `MarkdownChunker` zerlegt: (1) Zielgröße 512 und Obergrenze 1024 Token (Startwerte, per Recall-Benchmark zu kalibrieren); (2) Overlap bei überlangen Absätzen im Sliding-Window-Fallback 10–15 % der Zielgröße; (3) **atomare Blöcke:** Codeblöcke und Tabellen werden nicht mitten im Block getrennt; überschreitet ein Block die Obergrenze, wird er an Zeilen-/Zeilenblockgrenzen geteilt; (4) **Determinismus:** gleicher Text und gleiche Konfiguration ⇒ identische Chunk-Grenzen und Chunk-IDs (REQ-IDX-012); (5) jeder Chunk trägt `source_span` (Bytebereich im Quelldokument) für die Lineage. Ingestion ist ohne diese Konfiguration nicht aktivierbar. Das kontextuelle Chunk-Präfix (LLM-generiert) ist `[A]`/aus.

### 11.8 Limits und Quoten (DoS-Schutz, Startwerte)

`max_document_bytes` 16 MiB, `max_chunks_per_document` 4.096, `max_batch_ops` 10.000, `max_collections_per_tenant` 256, `max_in_flight_per_principal` 8. Überschreitung ⇒ strukturierter, nicht retrybarer Fehler `LimitExceeded`; einzige Ausnahme `max_in_flight_per_principal` ⇒ retrybares `Busy`. **Quoten (REQ-TEN-005):** `tenant_quota_bytes`, `tenant_quota_docs`; in `EnterpriseShared`/`EnterpriseRegulated` Pflichtwerte, sonst optional; Überschreitung ⇒ `QuotaExceeded` (final, bis Platz geschaffen ist); die Quote zählt freigegebenen Platz erst nach S2.

### 11.9 Determinismus-Versprechen (Reduktion)

Zugesichert wird: **gleiche Plattform, gleiche Eingabe, gleicher Seed → gleiche Ergebnisse**. Plattformübergreifende Bit-Identität wird **nicht** versprochen (SIMD-Rundung). Tests: T-DET-1 (HNSW-Aufbau), `check-recall-stability`, `check-nan-hot-loop`.

### 11.10 Offene Inhaltsfragen

Retention-Reaper/`MemoryType`-Ort (§11.6), BM25-Morphologie für deutsche Komposita (Marktannahme), Planner und Overfetch-Feinheiten (§9.4, §9.8). Sie werden in WP 3.2 beim Schneiden der Engine erfasst und entschieden.

---

## 12. Kryptografie, Löschung, Löschbericht, Audit (`contextra-crypto` und Schicht-Crates) `[K]`

### 12.1 Rechtlicher und fachlicher Rahmen (keine Rechtsberatung)

> **Rechtshinweis.** Technische Spezifikation, keine Rechtsberatung. **Software kann eine Rechtswirkung weder herbeiführen noch garantieren.** Sie kann den technischen Zustand herstellen, den die vertretenen Rechtsauffassungen verlangen, diesen Zustand messen und belegen und ehrlich benennen, was sie nicht erreicht. Die rechtliche Bewertung trägt der Verantwortliche. Punkte „P-xx“ (§25.1) sind vor jeder Außenkommunikation fachanwaltlich zu prüfen. Das Ziel „rechtlich nicht anfechtbar“ wird deshalb **technisch** als „unter jeder plausiblen Auffassung bedient, gemessen, mit benannten Annahmen“ operationalisiert (§12.1.3).

**Belegstärke:** **[R]** verbindliches Recht · **[B]** Behördenauslegung (EDPB, DSK, Landesbehörden, Standard-Datenschutzmodell, nicht bindend) · **[P]** Fachliteratur/Praxis/Norm · **✔** an der Primärquelle geprüft · **◐** nur Sekundärquelle · **○** allgemeiner Kenntnisstand · **⚠** umstritten/offen.

#### 12.1.1 Grundsatz

1. Art. 17 DSGVO verpflichtet zur Löschung **ohne unangemessene Verzögerung** (unverzüglich), auch ohne Antrag, definiert „Löschen“ aber nicht und schreibt keine Methode vor [R]. Löschen ist selbst Verarbeitung (Art. 5, 32). Das Standard-Datenschutzmodell (Baustein 60) versteht Löschen als **Unkenntlichmachen**: Der angeordnete Informationsverlust muss tatsächlich wirken [B ✔]. Technische Unmöglichkeit rechtfertigt keinen Verstoß (EDPB 02/2025 v2.0, Rn. 50) [B ✔].
2. Die Aufsichten fordern **Wirksamkeit und Nachweisbarkeit** (EDPB 02/2025 Rn. 76; CEF-Bericht 2025: Löschung verifizieren und nachweisen können) [B ✔]. Art. 5 Abs. 2, 24, 32 verlangen Rechenschaft [R].
3. Es ist **keine höchstrichterliche Entscheidung bekannt**, die Schlüsselvernichtung als Löschung anerkennt (P-01, P-02).
4. **Konstruktionsgrundsatz Verteidigung in der Tiefe.** Contextra stellt für jede erreichbare Datenschicht den Zustand her, den die **strengste** Auffassung verlangt (physische Bereinigung, Ebene A und Hygiene). Für nicht erreichbare Reste (Flash, Backups) kommt ein **zweiter, unabhängiger Mechanismus** hinzu (Schlüsselvernichtung mit Epoch Seal und Backup-Trennung, Ebene B). Jede Aussage im Bericht nennt ihre Voraussetzungen und Grenzen.
5. **Beide Mechanismen sind notwendig, keiner genügt allein.** Ohne Ebene A bleiben Klartext und invertierbare Suchstrukturen (Vektoren, Ghost Vectors). Ohne Ebene B bleiben Flash-Reste und Kopien lesbar, weil Überschreiben auf SSD nicht zuverlässig wirkt (Wear-Leveling). Ohne die Backup-Spur wären Backups eine ungelöschte Kopie.

#### 12.1.2 Quellenlage

Die Spalte „Prüfung 2.3“ trennt, was in der Recherche (v3.1) geprüft wurde, von dem, was bei der Erstellung dieser Fassung am 26.10.2026 **zusätzlich** gegengeprüft wurde. Eine Quelle mit ◐ oder ○ DARF NICHT in Außentexte übernommen werden, bevor der Volltext geprüft ist (P-14).

|Quelle|Aussage (paraphrasiert)|Status|Prüfung v3.1|Prüfung 2.3|
|---|---|---|---|---|
|EDPB Guidelines 02/2025 Blockchain, v2.0 final (07.07.2026)|Rn. 51: Nach Schlüsselvernichtung sind verschlüsselte Daten unverständlich, solange der Algorithmus nicht gebrochen ist, die Entschlüsselung nicht fortschreitet und der Schlüssel nicht kompromittiert war; verschlüsselte Daten bleiben personenbezogen. Rn. 52: Hash mit Salt oder Schlüssel bleibt personenbezogen, nach Löschung des Schlüssels nicht mehr verknüpfbar, wenn dieser nicht kompromittiert war. Rn. 103–104: „effektive Anonymisierung“ durch Löschung der Off-chain-Daten. Rn. 85, 96, Empf. 13: Notfallplan für Algorithmenwechsel, periodische Neubewertung.|final; Analogie (Blockchain), keine unmittelbare Aussage zu RAG|✔|nicht erneut geöffnet|
|EDPB Guidelines 02/2026 Anonymisierung, Entwurf v1.0 (07.07.2026), Konsultation bis 30.10.2026|Kriterien No Record Isolation, No Linkage, No Inference; kontextueller Ansatz; Verschlüsselung ist reversibel und kein Ersatz für Anonymisierung; Anonymisierung ist selbst Verarbeitung.|Entwurf|◐|offen (P-14)|
|EDPB Guidelines 01/2025 Pseudonymisierung|keine Endfassung gefunden|Entwurf|◐|offen|
|EDPB CEF 2025 Recht auf Löschung|764 Verantwortliche, 32 Aufsichten, 9 Formalverfahren, sieben wiederkehrende Probleme (u. a. Backups, Anonymisierung). Issue 6: Löschung in Backups nicht immer ratsam, dann aber Verfahren, um Löschungen nachzuverfolgen und auf wiederhergestellten Systemen umzusetzen. Best Practice: Daten durch Zufallszeichen ersetzen. Empfehlung: anerkannte Standards befolgen, Löschung verifizieren und demonstrieren. Issue 7: einfache Pseudonymisierung oder Maskierung genügt nicht. Issue 4: Einschränkung nach Art. 18, wenn die Entscheidung aussteht.|Bericht|✔|**bestätigt**: angenommen 10.02.2026, veröffentlicht 18.02.2026 (Pressemitteilung und Berichts-PDF auf edpb.europa.eu gefunden)|
|NIST SP 800-88 Rev. 2 (26.09.2025), FAQ vom 16.07.2026|Cryptographic Erase (CE) nur bei: kein früherer Klartext, ausreichende Schlüssellänge und Entropie, Zeroization **aller** Zielschlüssel (DEK, Wrapping, Master); Backup- und Escrow-Kopien brauchen eigene Regeln; Teil-CE nur bei absoluter Sicherheit; Verification und Validation getrennt; Zertifikat mit Methode, Technik, Validierungsentscheidung, Concurrence. Rev. 1 ist zurückgezogen; technikspezifische Verfahren verweist Rev. 2 an IEEE 2883.|Norm/FAQ|✔ (FAQ), ◐ (Norm)|**bestätigt**: Veröffentlichung 26.09.2025, Rev. 1 zurückgezogen, CE-Vorbedingung „Verschlüsselung seit Bereitstellung aktiv, kein Klartext je geschrieben“ (Sekundärquellen); Volltext der Norm nicht geöffnet|
|ISO/IEC 27040:2024|alle Kopien der Zielschlüssel sind zu bereinigen, sonst gilt es nicht als CE|Norm|◐|offen|
|IEEE 2883:2022|technikspezifische Sanitisierungsverfahren je Medium|Norm|○|nur als Verweis von NIST Rev. 2 bestätigt|
|BayLfD 30. TB (2020) Nr. 12.5|Zeitversetzte Backup-Löschung zulässig bei technischer Unmöglichkeit, begründeter Löschfrequenz, Zugriff nur über Restore, Nachlöschung bei jedem Restore, Dokumentation|Aufsichtsauffassung|✔|–|
|SDM Baustein 60 V1.0a|Löschen als Unkenntlichmachen|Aufsichtsauffassung|✔ (Auszug)|–|
|EuGH C-413/23 P (EDPS/SRB), 04.09.2025|Relativer Personenbezug; für den Verantwortlichen bleiben pseudonyme Daten personenbezogen, für Empfänger ohne Mittel kann dies anders sein|Urteil|◐|offen|
|AG Lörrach, 03.03.2025, 3 C 1099/24|Kein Anspruch der betroffenen Person auf Löschnachweis ohne Streitfall|Instanzgericht|◐|offen|
|OLG Dresden, 15.10.2024, 4 U 940/24|Verantwortlicher muss vom Auftragsverarbeiter aussagekräftige Löschbestätigung einholen|Berufungsgericht|◐|offen|
|OLG Dresden, 03.02.2026, 4 U 292/25|Anonymisierung als „Minus“ zur Löschung auf Verlangen|Berufungsgericht|◐ nur Sekundärquelle|offen|
|DSB Österreich, Bescheid 05.12.2018|Unkenntlichmachung zulässig, völlige Irreversibilität nicht nötig|Behörde|◐|offen|
|eIDAS Art. 41, 42|Vermutung für Datum und Datenintegrität bei qualifiziertem Zeitstempel; § 371a ZPO betrifft qualifizierte Signaturen, nicht Zeitstempel unmittelbar|Recht|◐|offen|
|Ghost Vectors, arXiv 2606.18497|Soft-gelöschte HNSW-Vektoren bleiben rekonstruierbar; Ghost-Knoten verändern Treffermengen und erzeugen Timing-Kanal|Preprint|✔|–|

#### 12.1.3 Rechtsstabilitätsmatrix

Contextra MUSS gleichzeitig unter allen plausiblen Auffassungen bestehen. Die Matrix ordnet jedem Maßstab den erfüllenden technischen Zustand zu (Klassen: §12.5).

|Auffassung|Maßstab|Erfüllender Zustand|Rest|
|---|---|---|---|
|T1 Strenge Vernichtung|alle Kopien im Einflussbereich physisch entfernt (SDM, EDPB Rn. 50, 63)|Klasse **M**: P plus Hygiene (physische Kompaktion, WAL-Scrub, Checkpoint-Rewrite, Index-Neuaufbau)|Flash-Reste, externe Backups (ausgewiesen)|
|T2 Unkenntlichmachung|niemand ist mit vertretbarem Aufwand zur Identifizierung in der Lage (SDM, DSB, OLG Dresden 4 U 292/25, Entwurf 02/2026)|Klasse **C** plus Restmetadaten-Selbsttest (§12.7.6) plus Perspektivenmatrix|Aufwandsbewertung ist Rechtsfrage (P-01, P-02)|
|T3 Cryptographic Erase|NIST-Bedingungen, ISO/IEC 27040 (EDPB Rn. 51 analog)|C2/C3 mit CE-Traceability; C1 mit Annahme A-FLASH|Hardware-Restrisiko bei K1|
|T4 Einschränkung/Aufbewahrung|Art. 18, Art. 17 Abs. 3 (CEF Issue 4)|Klasse **L**, `Restricted`, Legal Hold|Dauer ist Rechtsfrage (O-12)|
|T5 Backups|Löschungen nachverfolgen und beim Restore umsetzen (CEF Issue 6, BayLfD 12.5)|Zwei-Spuren-Backup, Restore-Guard, `ErasedSetMirror` (§12.17)|Externe Backups, die Contextra nicht sieht|

#### 12.1.4 Perspektivenmatrix

Der Entwurf 02/2026 prüft Anonymität aus Sicht jeder Stelle mit realistischem Zugang. Der Bericht trägt deshalb ein Urteil **je Perspektive**, erzeugt aus den gemessenen Werten (nie statisch).

|Perspektive|Zugang|nach L|nach P|nach C|
|---|---|---|---|---|
|Q1 Leser (MCP-Client, API, `as_of`, Export)|Lesepfade|gesperrt|gesperrt|gesperrt|
|Q2 Betreiber mit Prozess- und Dateizugriff **und** Passphrase|Engine, Dateien, Keystore|Klartext nur über gesperrte Lesepfade|keine Klartextstruktur mit Scope-Inhalt|persistente Kopien unter vernichtetem DocDek nicht lesbar|
|Q3 Dateiangreifer ohne Passphrase|Datenverzeichnis|verschlüsselt|verschlüsselt|verschlüsselt|
|Q4 Medienangreifer mit Flash-Resten, ohne Passphrase|Chip-off, Controller-Reste|verschlüsselt|verschlüsselt|verschlüsselt|
|Q5 Medienangreifer mit Flash-Resten **und** Passphrase|wie Q4 plus Schlüssel|lesbar|lesbar (Chiffrat)|**C1: möglich** (A-FLASH). **C2/C3: nicht möglich**|
|Q6 Besitzer eines Daten-Backups ohne Schlüssel-Backup|Backup-Dateien|verschlüsselt|verschlüsselt|nicht lesbar|
|Q7 Besitzer eines Schlüssel-Backups älter als die Löschung|Schlüssel-Backup|lesbar bis Fensterende|lesbar bis Fensterende|unlesbar nach `key_backup_max_age`; Restore-Guard greift beim Einspielen|
|Q8 Empfänger/Dritte (Exporte, LLM-Anbieter, Auftragsverarbeiter)|außerhalb von Contextra|nicht erfasst|nicht erfasst|nicht erfasst (§12.20, `RecipientLedger` §12.19)|

#### 12.1.5 Technische Anforderungen aus dem Rahmen

(Die Kennungen heißen in 2.3 `LA-n`, um die Kollision mit den Architekturregeln A1–A10 (§3.3) zu beseitigen; Fassung 2.2.1 führte beide als „A1…“.)

|ID|Anforderung|Rechtslage|Umsetzung|
|---|---|---|---|
|LA-1|Nach der Löschung keine Person identifizierbar, Informationsgehalt nicht mit vertretbarem Aufwand reproduzierbar; auch Kopien beim Verantwortlichen erfasst.|[B ✔]|§12.4, §12.5, §12.7, §12.16|
|LA-2|Alle Repräsentationen im Wirkungsbereich: Quelle, Chunk, Vektor, Volltextindex, Graph, Cache, Log, interne Checkpoints, Schlüsselmaterial.|[P]; Embeddings ⚠|§12.4 (zehn Schichten)|
|LA-3|Alle Kopien der Zielschlüssel sind erfasst (Wrap-Blobs, Epochen, Schlüssel-Backups).|[P ◐] NIST, ISO 27040|§12.2.4 (Epoch Seal), §12.17 (Spur K), Nadelscan §12.7.3|
|LA-4|Löschgranularität ohne Kollateralverlust.|[B ✔]|DocDek je Betroffenen-Einheit (§12.2.5, §12.15)|
|LA-5|Verbleibende Metadaten (Längen, Header, IDs, Zeitstempel) identifizieren nicht wieder; Pseudonyme laufen ab.|[P]; Entwurf 02/2026|Padmé, verschlüsselte Header (§12.2.6), §12.7.6, §12.10|
|LA-6|Löschung ist verifizierbar und nachweisbar; Verification und Validation getrennt.|[R/B ✔]|§12.7|
|LA-7|Bericht ist datensparsam.|⚠ umstritten|§12.10, §12.7.8|
|LA-8|Nach Rücksicherung unmittelbar erneut löschen; Überwachung; Rücksicherung nur innerhalb des Horizonts.|[B ✔]|§12.9, §12.17|
|LA-9|Aufbewahrungspflichten: Einschränkung statt Löschung.|[R] + ⚠|`Restricted` (§12.8.6)|
|LA-10|Krypto-Agilität: Austauschplan, periodische Neubewertung.|[B ✔ Analogie]|§12.2.9, REQ-DEL-028|
|LA-11|Unverzüglich; Monatsfrist; Alarm- und Eskalationsschwellen.|[R ○]|§12.8, §12.19, §15.5|
|LA-12|Ehrliche Produktaussagen; keine Rechtsbehauptung.|⚠|§2.6, §12.11|
|LA-13|Löschkonzept dokumentierbar (Löschinventar, Vorlage `doctor export-concept`).|[B ✔]|§12.14|
|LA-14|Empfänger werden informiert; Auftragsverarbeiter-Löschung nachweisbar.|[R ○/◐]|`RecipientLedger` (§12.19), Bescheinigung (§12.7.8)|
|LA-15|Prozessspeicher, Dumps, Swap sind adressiert; NIST-Vorbedingung „kein früherer Klartext auf dem Medium“.|[P ◐]|§12.18|
|LA-16|Wirkung ist unter Fehlern und Abbrüchen garantiert (Crash, ENOSPC, `fsync`-Fehler, Restore).|[B] (Rechenschaft)|§12.8.4, §7.10, §12.12|

**Pflichten des Betreibers (nicht ersetzbar durch Software).** `doctor export-concept` erzeugt eine **nicht rechtsverbindliche** Vorlage: Verzeichnis der Löschschichten und Löschfristen je Collection; Backup-Richtlinie (Daten-Backup, Schlüssel-Backup, `key_backup_max_age`, Ablageorte, `ErasedSetMirror`-Ziel); Textbaustein für Verzeichnis der Verarbeitungstätigkeiten und Auftragsverarbeitung (Löschbestätigung, OLG Dresden 4 U 940/24); Ablauf bei Restore. Ob und an wen Bescheinigungen ausgeliefert werden, entscheidet der Verantwortliche.

### 12.2 Primitive, Schlüsselhierarchie, Verwahrung

#### 12.2.1 Primitive

**AES-256-GCM-SIV** (nonce-misuse-resistant AEAD), **Argon2id** (Passphrase-KDF, INV-KDF-1), **HKDF** (nur aus hochentropischem Material, nie aus Passphrasen), **HMAC-SHA256** (WAL-/Sidecar-Ketten, Referenz-Hashes), **Ed25519** (Berichtssignatur), **SHA-256/BLAKE3** (Hash). Zufall für Schlüssel, Nonces, Salts stammt aus dem System-RNG (P28-Ausnahme). Geheimnisse sind `zeroize`-geschützt (`Zeroizing<[u8; 32]>`, kein `Clone` ohne Zeroisierung, konstantzeitiger Vergleich). Kryptografische Primitive werden nie selbst gebaut. Schlüsselmaterial erscheint nie in `Debug`, Logs oder Fehlertexten.

#### 12.2.2 Hierarchie

```
Wurzel (Custody-Slot K1 | K2 | K3)                    EffaceableKeySlot (§12.2.3)
 └─ KEK[coll, epoche]       je Collection und Epoche, gewrappt unter der Wurzel
     ├─ DocDek[einheit]     je Betroffenen-Einheit, gewrappt unter KEK
     ├─ ArtKey[artefakt]    je Index-Artefakt (Text-Segment, Graph-Snapshot), gewrappt unter KEK
     └─ K_ref[tenant, ref_epoche]   Referenzschlüssel (§12.10), zufällig, gewrappt unter KEK
K_eng                       Struktur und Integrität (keine Nutzdaten), HKDF aus der Wurzel
K_sc/*                      Zweckschlüssel der Sidecar-Ketten (HMAC), HKDF aus K_eng
```

Jeder Schlüssel hat genau einen Zweck und ein Label im Zweckregister (INV-KEYPURPOSE-1, §12.3). **Die Sidecar-Ketten und `K_eng` überleben jede Löschung von Nutzdaten und enthalten keine Nutzinhalte** (INV-LOG-1). Das ist Voraussetzung dafür, dass Berichte und Ketten nach der Löschung prüfbar bleiben.

#### 12.2.3 Custody-Stufen und Port `EffaceableKeySlot`

|Stufe|Wurzel|Klasse|Status in v1.0|
|---|---|---|---|
|K0|keine Verschlüsselung (nur Entwicklung, `doctor` rot)|nur L, P|ausgeliefert|
|K1|Argon2id aus Passphrase (m ≥ 64 MiB, t ≥ 3, ≥ 250 ms), Software-Keystore|C1|ausgeliefert|
|K2|Siegelschlüssel in manipulationsresistentem, nicht exportierbarem Modul (z. B. TPM-2.0-Objekt, HSM); Eignung je Plattform ist Spike S-K2|C2|**Port vorgesehen**, nicht ausgeliefert bis Konformitätssuite besteht|
|K3|externer KMS/HSM mit Zeitfenster-Löschung und Vernichtungsattest|C3|wie K2|

Bis zur bestandenen Konformitätssuite liefern K2 und K3 `CapabilityUnsupported`, **nie** einen Stub, der Erfolg meldet (REQ-DEL-027). Hardware-Effaceable-Konzepte (z. B. Apple Effaceable Storage) sind Schnell-Löschen eines Volumenschlüssels und kein Mechanismus für beliebige Einzelschlüssel; eine Plattform gilt nur als K2, wenn die Suite nachweist, dass Schlüsselmaterial nach Vernichtung nicht aus Resten rekonstruierbar ist. Damit ist E27 (K2/K3 gestrichen) **teilweise zurückgenommen** (E44, §20).

```rust
pub trait EffaceableKeySlot: Send + Sync {
    fn custody(&self) -> Custody;                         // K1 | K2 | K3 (K0 hat keinen Slot)
    fn wrap(&self, material: &Zeroizing<[u8; 32]>, ad: &[u8]) -> Result<WrappedBlob, KeyError>;
    fn unwrap(&self, blob: &WrappedBlob, ad: &[u8]) -> Result<Zeroizing<[u8; 32]>, KeyError>;
    fn rotate(&self) -> Result<SlotGeneration, KeyError>; // neue Wurzel-Generation; die alte bleibt bis destroy
    fn destroy(&self, generation: SlotGeneration) -> Result<DestroyAttestation, KeyError>;
}
```

`DestroyAttestation` enthält Custody, Verfahren (Zeroization), Zeitpunkt (aus `Clock`) und einen Beleg der Implementierung. **Nur K1 liefert einen lokalen Beleg ohne Dritt-Attest** (Selbstauskunft, kein Beweis der Abwesenheit auf dem Medium). `KmsProvider` bleibt der Port für Erzeugen und Ableiten; er hat keinen Default und keinen Fallback (REQ-PORT-2).

#### 12.2.4 KEK-Epochen und Epoch Seal

**Problem.** Wird nur der DocDek vernichtet, liegt sein gewrappter Blob als Rest auf dem Flash. Wer KEK und Passphrase besitzt, kann ihn entschlüsseln. NIST Rev. 2 zählt Wrapping-Schlüssel zu den CE-Zielschlüsseln, ISO/IEC 27040 verlangt die Bereinigung aller Schlüsselkopien.

**Epoch Seal.**

1. erzeugt `KEK[coll, e+1]`, gewrappt unter der Wurzel;
2. wrappt **alle lebenden** DocDeks, ArtKeys und `K_ref` unter `KEK[e+1]` neu (Re-Wrap, kein Datenrewrite);
3. schreibt die KeyTable atomar neu (REQ-FMT-5);
4. vernichtet `KEK[e]` und dessen Wrap-Kopie nach dem Vernichtungsprotokoll (§12.2.7);
5. bei K1 mit `rotate_passphrase_on_seal`: wechselt die Passphrase, sodass die alte Wurzel nicht mehr ableitbar ist, **wenn die alte Passphrase unbekannt ist**;
6. bei K2/K3: ruft `rotate()` und anschließend `destroy()` am Slot.

Nach dem Seal sind Flash-Reste gewrappter DocDeks früherer Epochen nur mit `KEK[e]` entschlüsselbar, und `KEK[e]` existiert nur noch als Rest unter der Wurzel. Bei K2 und K3 endet die Kette im Modul. Bei K1 bleibt die **Annahme A-FLASH** (§12.5.2); die Passphrasen-Rotation begrenzt sie zeitlich.

**Auslöser.** Zeitgesteuert (`kek_epoch_len`, Startwert 30 Tage), auf Anforderung (`seal-now`) und automatisch, wenn seit dem letzten Seal mindestens eine Löschung stattgefunden hat und `seal_after_deletion_max` erreicht ist (Startwert 7 Tage; **in `EnterpriseRegulated` Pflichtwert ≤ 72 h**, H-07). Löschungen innerhalb einer Epoche teilen sich einen Seal (Batch).

**Klassenwirkung (INV-EPOCH-SEAL-1).** C1 bis C3 gelten erst nach dem ersten Seal, der die Löschung umfasst. Zwischen Vernichtung und Seal weist der Bericht die Klasse `C*` (vorläufig) mit `residuals: [R-KEKEPOCH until]` aus.

**Crash-Sicherheit.** Der Seal ist ein eigener durabler Ablauf mit Journal. Ein Absturz mitten im Re-Wrap hinterlässt `KEK[e]` und `KEK[e+1]` nebeneinander und wird beim Start (B7) fortgesetzt. Der Seal ist idempotent.

**Kosten.** Re-Wrap ist O(Anzahl Schlüssel), nicht O(Datenmenge). Die Messung gehört in `benchmarks/budgets.toml` (O-24, O-40).

#### 12.2.5 DocDek-Regeln

1. Jede Betroffenen-Einheit (§12.15) erhält **genau einen** DocDek, erzeugt bei `remember`.
2. Jeder Record in WAL, SSTable und Blob-Segment wird mit dem DocDek **seiner Einheit** verschlüsselt. Ein Block mit Records mehrerer Einheiten enthält mehrere Records unter verschiedenen Schlüsseln; die Blockverschlüsselung unter `K_eng` bleibt als äußere Schicht. **Die Vernichtung eines DocDek macht deshalb keinen anderen Record unlesbar** (kein Kollateralverlust, LA-4).
3. Vektor-Payload (fp32 und SQ8) im persistenten Snapshot liegt je Slot unter dem DocDek.
4. Der gewrappte DocDek liegt in der **KeyTable** (Ersetzen-Semantik), nicht in einem Append-Log. Das `KeyRegistry`-Sidecar führt nur Ereignisse ohne Schlüsselmaterial und ohne Wrap-Blob (REQ-DEL-N5).
5. Der Key-Lock der Einheit wird über die **gesamte** Saga gehalten; ein konkurrierendes `remember` auf die Einheit wird abgewiesen (REQ-DEL-012).
6. **Entschlüsselte Schlüssel im Speicher** liegen ausschließlich in einem registrierten `KeyCache`, der bei Vernichtung **vor** dem Schreiben der neuen KeyTable invalidiert und zeroized wird (H-04). Kein Modul hält eine Kopie außerhalb des `KeyCache`.
7. Bestand ohne DocDek (Altformat, Scope-DEK) gilt als `LegacyUnkeyed`: höchstens Klasse P, bis `contextra migrate --rekey-units` die Records je Einheit neu verschlüsselt hat (H-12, REQ-DEL-N37). `doctor` weist den Anteil aus.

#### 12.2.6 Record-Format und Restmetadaten

- AEAD: AES-256-GCM-SIV, Nonce 96 Bit zufällig, Tag 128 Bit; AD = `scope_id ‖ doc_key_id ‖ record_version`.
- **Padding:** Klartextlänge nach dem Padmé-Schema aufgefüllt (Leckage O(log log M), Overhead höchstens 12 %); Mindestlänge `min_record_bytes` (Startwert 256).
- **Header:** `doc_key_id` und Sequenzbezug stehen **innerhalb** des unter `K_eng` verschlüsselten Blockbereichs, nicht im Klartext des Records (REQ-DEL-N15).
- **Wrap der Schlüssel:** AEAD-Wrap (AES-256-GCM-SIV), zufällige 96-Bit-Nonce, AD = `key_id ‖ kek_epoch`. Die Nonce dient als **Fahndungsnadel** (§12.7.3). AES-KWP (SP 800-38F) ist per ADR wählbar, hat keine Nonce und braucht ein anderes Suchmuster.

#### 12.2.7 Vernichtungsprotokoll (Destroy-Protokoll, H-03)

Die Schlüsselvernichtung ist ein durabler, idempotenter Ablauf mit festen Schritten. Jeder Schritt wird **vor** der Ausführung im `ErasureJournal` vermerkt (Sidecar vor Wirkung, §7.9 Regel 3).

|Schritt|Aktion|Durabilität / Ausfallverhalten|
|---|---|---|
|D1|Journal-Intent `destroy{key_ref, wrap_nonce_digest}`; Pre-Destruction-Inventar der Chiffratblöcke je `key_id` liegt vor (§12.7.3)|`fsync`; ohne Inventar kein D2|
|D2|Key-Lock halten; `KeyCache`-Eintrag invalidieren und zeroizen|im RAM; danach ist kein Lesen mit diesem Schlüssel mehr möglich|
|D3|Neue KeyTable-Generation **ohne** den Eintrag schreiben: temporäre Datei, `fsync`, `rename`, `fsync` des Verzeichnisses (REQ-FMT-5)|Ausfall hier: Schlüssel lebt, Einheit bleibt `Frozen`, Retry beim Start (kein „vernichtet“-Bericht)|
|D4|Alte KeyTable-Generation und alle Temp-/`.bak`-Kopien entfernen (`durable_remove`), Verzeichnis-`fsync`|Reste der alten Datei auf dem Medium sind Gegenstand des Epoch Seal (§12.2.4)|
|D5|`RevocationLog`-Eintrag `key_destroyed{key_ref_hash}`; `KeyRegistry`-Ereignis `destroyed`|`fsync`|
|D6|Journal-Done; Nadelscan (§12.7.3)|Bericht erst nach D6|

**Regeln.** (1) Ein Schlüssel gilt erst nach **D3** als vernichtet; der Bericht trägt `key_destroyed` erst nach D6. (2) Recovery wiederholt D2 bis D6 idempotent. (3) Die frühere Formulierung „Eintrag mit Nullen überschreiben, fsync, Eintrag entfernen“ ist **kein** Wirkmechanismus auf SSD und Copy-on-Write-Dateisystemen und wird nur als zusätzliche Hygiene beibehalten; die Wirkung trägt der Epoch Seal. (4) Bei `ENOSPC` nutzt D3 die Notreserve (§7.10); bei `fsync`-Fehler an der KeyTable gilt §7.10 (`Poisoned`, Sperr-only). (5) Nach D3 ist ein Wiederauftauchen des Schlüssels ausgeschlossen, weil ihn nur die KeyTable und Schlüssel-Backups der Spur K halten (§12.17); der Restore-Guard vernichtet ihn dort erneut.

#### 12.2.8 Index-Artefakte (ArtKey)

Postings und Graph enthalten viele Dokumente und können nicht unter einem DocDek liegen.

|Artefakt|Schlüssel|Bereinigung|
|---|---|---|
|Vektor-Payload im Snapshot|DocDek|Vernichtung des DocDek|
|Graph-Snapshot (Topologie, Slot-Nummern)|ArtKey der Snapshot-Generation|Neuaufbau, danach Vernichtung des ArtKey (§12.16)|
|Text-Segment (Postings, Termwörterbuch, Längen)|ArtKey je Segment|Segment neu schreiben mit neuem ArtKey, alten ArtKey sofort vernichten|

INV-ARTKEY-1: Jedes Index-Artefakt hat genau einen ArtKey; beim Neuschreiben wird der alte vernichtet (REQ-DEL-N16).

#### 12.2.9 KeyTable, KeyRegistry, Krypto-Agilität, Keystore

- **KeyTable** (Magic `MFKT`, §7.2) ist ein eigener Speicherbereich mit Ersetzen-Semantik. Nach jeder Vernichtung und jedem Seal wird die Segmentdatei ohne vernichtete Einträge neu geschrieben (REQ-DEL-N4). Atomares Ersetzen nach REQ-FMT-5, auch für den Keystore.
- **`KeyRegistry`** (`MFKR`) führt Ereignisse (`created`, `destroyed`, `sealed`) mit Referenz-Hash, ohne Schlüsselmaterial und ohne Wrap-Blob; append-only, HMAC-verkettet, über `DurableLog` mit `fsync` je Eintrag. Ein gelöschtes Log samt Marker bei vorhandenen Daten gilt als Manipulation (fail-closed).
- **Krypto-Agilität (REQ-DEL-028, P-13).** Jedes persistierte kryptografische Objekt trägt Algorithmus- und Formatkennung. Ein Wechsel von AEAD, KDF, Hash oder Signatur ist ohne Datenverlust möglich (Re-Wrap bzw. Neuverschlüsselung per Kompaktion). Das Notfallverfahren bei Algorithmenversagen (EDPB Rn. 85, 96, Empf. 13) ist dokumentiert und wird halbjährlich per `doctor crypto-drill` geübt; die Eignung der Primitive wird periodisch neu bewertet.
- `mlock` über `sys`; scheitert die Sperrung, meldet `mem_lock` dies wahrheitsgemäß, und `doctor` weist es aus (§12.18).

### 12.3 Zweckregister und Domänentrennung (INV-KEYPURPOSE-1)

Jeder Schlüssel dient genau einem Zweck; HKDF-Label `contextra/v1/<zweck>`. Das Register `governance/key_purposes.toml` ist die einzige Quelle; das Gate vergleicht es mit dem Code (REQ-KEY-011). Die Datei **fehlt im Repo** (U-29, O-28) und ist Voraussetzung von WP L-2.

|Zweck|Label bzw. Herkunft|Ableitung|Wirkung der Vernichtung|
|---|---|---|---|
|Struktur-Schlüssel `K_eng`|`contextra/v1/eng`|HKDF aus der Wurzel|keine Nutzdaten; nie für Nutzdaten|
|WAL-Integrität|`contextra/v1/wal`|HKDF aus `K_eng`|keine|
|Sidecar-Ketten|`contextra/v1/sc/keyreg`, `…/revoke`, `…/audit`, `…/journal`, `…/erased`|HKDF aus `K_eng`|keine; Ketten sind keine Datenschlüssel|
|Keystore-Integritätswurzel|`contextra/v1/keystore-root`|bei K0 im Klartext, sonst gewrappt|keine|
|`KEK[coll, epoche]`|zufällig, 256 Bit, gewrappt|Wurzel (`EffaceableKeySlot`)|Epoch Seal; bei Collection-Löschung alle `KEK[coll,*]`|
|`DocDek[einheit]`|zufällig, 256 Bit, gewrappt|`KEK`|**Löschmaßnahme Ebene B** der Einheit|
|`ArtKey[artefakt]`|zufällig, 256 Bit, gewrappt|`KEK`|Löschmaßnahme für Index-Artefakte|
|`K_ref[tenant, ref_epoche]`|zufällig, 256 Bit, gewrappt; **bewusst nicht per HKDF ableitbar** (sonst nicht vernichtbar)|`KEK`|Pseudonym-Ablauf (§12.10)|
|Berichtssignatur|Signierschlüssel des `KeyManager`|`KmsProvider`|keine; Berichte bleiben prüfbar (INV-PROOF-KEY-1)|
|Surrogat-Vault `[R]`|`contextra/v1/vault`|HKDF aus `K_eng`|Löschschicht `CachesAndVault`|

Regeln: (1) Labels sind paarweise verschieden; Wiederverwendung eines Schlüssels für zwei Zwecke ist unzulässig. (2) Ein Label ohne Registereintrag ist Gate-Fehler. (3) Die Zwecke `DocDek`, `ArtKey`, `KEK-Epoche` und `K_ref` (statt `K_commit`) ersetzen die Streichung aus E27. (4) **Der Berichtssignaturschlüssel wird nie durch einen Seal oder eine Löschung vernichtet**; ein Wechsel erzeugt eine neue `signer_key_id`, die alte bleibt als Verifikationsschlüssel im `RevocationLog` nachvollziehbar (H-16).

### 12.4 Die zehn Löschschichten (`DeletionLayer`)

Der Typ `DeletionLayer` hat in 2.3 **zehn** Varianten (2.2.1: acht). Jede Schicht gehört einer Ebene an (**A** Klartext-Strukturen, die für die Suche Klartext brauchen und physisch bereinigt werden; **B** persistente Reste, die unter einem Schlüssel stehen) und hat **genau einen** Besitzer-Crate, der Scanner **und** Purger stellt (Kantenregel 5, §4.4).

|Schicht|Typname|Ebene|Besitzer (Scanner / Purger)|Maßnahme|Messung|Besonderheit|
|---|---|---|---|---|---|---|
|S1 Primärdaten (LSM, WAL)|`Lsm`|A (Memtable), B|`store` (`LsmScanner`)|Records unter DocDek; Memtable zeroizen; Hygiene: `compact_range(scope)`, WAL-Scrub (Segmente ohne Scope neu schreiben, alte entfernen, Verzeichnis-`fsync`)|Klartext, Chiffrat, Nadel|Memtable-Beleg erst nach `force_flush`; Tombstone-GC nur bei Vollkompaktion (§7.4)|
|S1b Blob-Segmente|`BlobSegments`|B|`store` (`LsmScanner`)|Records unter DocDek; Hygiene: Segment-Rewrite|wie `Lsm`|**neu in 2.3**; früher unter `Lsm` versteckt, daher Lücke im Oberflächen-Inventar|
|S2 Vektorindex|`VectorIndex`|A, B|`vector` (`VectorScanner`)|Arena-Slots (fp32, SQ8) nullen; Snapshot-Payload unter DocDek; Graph unter ArtKey, Neuaufbau (§12.16)|Slots mit Payload ≠ 0, Snapshot-Datei|kein Rewiring (VETO-F02); Slot-Nummern bis zum Neuaufbau nicht wiederverwendet (INV-HNSW-SLOT-1)|
|S3 Textindex|`TextIndex`|A, B|`text` (`TextScanner`)|Segmente neu schreiben (neuer ArtKey, alter sofort vernichtet); Terme nur gelöschter Dokumente aus dem Wörterbuch; Statistik (IDF, Längen, Zähler) neu berechnen|Postings, Wörterbuch, Positionen|Terme müssen verschwinden, sonst Inhaltsrest|
|S4 Caches und Vault|`CachesAndVault`|A|`store`, `engine`, `privacy`|Block-, Embedding-, RRF-, Antwort-Cache und Surrogat-Vault invalidieren und zeroizen|`CacheScanner` (Enumeration nach Scope)|Antwort-Caches sind Pflicht (REQ-DEL-N7)|
|S5 Audit, Lineage, Sidecars|`AuditLineage`|–|`engine`, `crypto` (`SidecarScanner`, `LineageScanner`)|Ableitungen kaskadieren (INV-DERIVED-1); Sidecars enthalten nur Referenz-Hashes, keine Nutzinhalte, keine Schlüssel|Referenzen ohne Klartext|Ableitungen mit weiteren Quellen neu ableiten oder `stale`|
|S6 Interne Checkpoints|`Checkpoints`|B|`checkpoint` (`CheckpointScanner`)|Policy `Rewrite` (Standard), `ExpireNow`, `ExpireAfterHold` nur unter Hold (§12.5.4)|Rohsicht und entschlüsselte Sicht|Hardlink-Klone derselben SSTables liegen im Datenverzeichnis, also in der Kontrolle von Contextra|
|S7 Schlüsselmaterial|`KeyMaterial`|B|`crypto`, `store` (`KeyScanner`)|KeyTable, Keystore, Wrap-Blobs, Temp-/`.bak`-Kopien|Nadelscan (§12.7.3)|**neu in 2.3**; ohne diese Schicht fehlt der CE-Beleg (LA-3)|
|– Graph|`Graph`|–|– (`graph` `[A]`)|–|Start (B3) belegt „keine Daten dieses Typs“ ⇒ `FeatureDisabled`|sonst `NotMeasured(NoPhysicalScanner)`|
|– KV-Cache|`KvCache`|–|– (`kvcache` `[A]`)|–|wie `Graph`|wie `Graph`|
|S8 Externe Sicherungen|– (Ausschluss)|–|–|ausweisen (`ExternalBackups`); Daten-Backups der Spur D sind keine externen Backups, sofern Contextra sie erzeugt (§12.17)|keine|§12.20|

**Regeln.**

1. Jede Schicht stellt genau einen Witness-Typ aus. `FeatureDisabled` ist nur zulässig, wenn der Rand-Feature-Schalter aus ist **und** der Start den fehlenden Datenbestand belegt; sonst `NotMeasured(NoPhysicalScanner)`.
2. Eine Schicht, in der Daten liegen können und für die kein Scanner existiert, ist `NotMeasured`; der Bericht ist dann höchstens `Partial` (REQ-DEL-010, -020).
3. **Ableitungen (INV-DERIVED-1)**: `forget` kaskadiert über die Lineage; der Bericht führt sie als eigene Einträge.
4. **Embeddings und Vektorbytes sind personenbezogen** (vorsorglich, P-05): „Text gelöscht, Vektor im Index“ ist keine Löschung.
5. CDC-Ereignisse sind inhaltsfrei (§7.3).
6. Contextra führt keine eigenen Prompt- oder Gesprächshistorien außerhalb der Primärdaten und der AuditChain; Verläufe des Hosts stehen als `HostHistories` in `excluded` (A-HOSTHIST).
7. **Oberflächen-Inventar (INV-SURFACE-1, Gate `surface-coverage`).** Jede persistente Datei- und Strukturklasse des Format-Registers (§7.11) steht in genau einer Zeile dieser Tabelle oder in §12.20. Ein neues Format ohne Schichtzuordnung bricht den Build.
8. **Migration der Typen.** Der Code kennt sieben abweichende Varianten (`LsmMemtable`, `SsTableAllLevels`, `HnswIndex`, `WalAllSegments`, `CsrGraph`, `KvCacheSegments`, `EmbeddingCache`; U-41). Die Abbildung auf die zehn Schichten steht in §21 (U-41) und ist Teil von WP L-1; Berichte V1 bis V3 bleiben lesbar und tragen `coverage_known = false`.

### 12.5 Löschwirkungsklassen, Annahmen, Checkpoint-Policy, aktive Snapshots

#### 12.5.1 Klassen

Die Klassen bauen aufeinander auf. **Ein Bericht darf nur die höchste Klasse ausweisen, deren Voraussetzungen gemessen sind** (INV-CLASS-MEASURED-1). Die Stufen S0 bis S3 und SK der Fassung 2.2.1 gehen in den Klassen und der Saga (§12.8) auf.

|Klasse|Name|Aussage|Voraussetzungen (gemessen)|
|---|---|---|---|
|**L**|Gesperrt|Scope für alle Lesepfade gesperrt; rechtlich einer Einschränkung nach Art. 18 entsprechend|`ErasedSet`-Eintrag durabel (`fsync`), Lesepfade geprüft (inkl. `as_of`, Export, Checkpoint-Erzeugung)|
|**P**|Klartext bereinigt|In keiner Klartextstruktur, die Contextra beschreibt, ist ein Inhalt des Scopes auffindbar|Alle Ebene-A-Schichten gemessen, Positivkontrolle bestanden|
|**C***|Vorläufig kryptografisch|Schlüssel vernichtet (D3 vollzogen), Epoch Seal steht aus|Nadelscan, Inventar, Kanarien-Probe bestanden; `residuals: [R-KEKEPOCH until]`|
|**C1**|Reste unlesbar, Software-Schlüssel|Alle persistenten Kopien liegen nur als Chiffrat unter vernichteten Schlüsseln; **Annahme A-FLASH**|wie C*, plus Epoch Seal vollzogen, Custody K1|
|**C2**|wie C1, Wurzel im Modul|A-FLASH entfällt für die Schlüsselwurzel|wie C1, Custody K2 mit bestandener Konformitätssuite|
|**C3**|wie C1, externer KMS|A-FLASH ersetzt durch Anbieterattest (Prüfpflicht des Verantwortlichen, P-18)|wie C1, Custody K3|
|**M**|Messvollständig|P und C erreicht; alle Hygienemaßnahmen vollzogen (physische Kompaktion, Checkpoint-Policy, Graph-Neuaufbau); offen bleiben nur die Pflicht-Ausschlüsse|P, C1/C2/C3, Hygiene gemessen|

**Orthogonale Angaben im Bericht (immer vorhanden):** `legal_state ∈ {Eingeschränkt, TechnischGelöscht, BeschränktAufbewahrt}`; `backup_window ∈ {Closed, OpenUntil(date), ExternalUnmanaged}`; `assumptions[]`; `residuals[]` (zeitlich begrenzte Reste mit Frist, z. B. `R-GRAPH until`, `R-KEKEPOCH until`); `platform_guarantee`.

#### 12.5.2 Annahmecodes

|Code|Annahme|
|---|---|
|A-FLASH|Flash-Reste gewrappter Schlüssel und die Passphrase kommen nicht zusammen in die Hand derselben Stelle (nur bei K1)|
|A-ALGO|Der Algorithmus (AES-256-GCM-SIV, Argon2id, HKDF) ist nicht gebrochen (EDPB Rn. 51)|
|A-KEYLEAK|Der Schlüssel war vor der Vernichtung nicht kompromittiert (EDPB Rn. 51)|
|A-RAM|Kein Klartext oder Schlüssel liegt in Swap, Dumps, Hibernation (P-03; §12.18)|
|A-EXTBACKUP|Backups außerhalb von Contextra sind nicht erfasst|
|A-HOSTHIST|Verläufe des Host-Systems (Prompt-Historien) sind nicht erfasst|
|A-CLOCK|Die Systemuhr ist nicht manipuliert (Horizont- und Fristentscheidungen, H-09)|

#### 12.5.3 Wirkung je Plattform und Medium

Contextra behauptet **keine** Löschwirkung auf Medienebene. Auf SSD, Copy-on-Write-Dateisystemen (btrfs, zfs, APFS) und Journalen wirkt Überschreiben nicht zuverlässig; deshalb trägt auf Medienebene die Schlüsselvernichtung (Ebene B) und nicht das Überschreiben. Dateisystem-Snapshots, Time Machine und Journale gelten als externe Kopien und stehen im Bericht (§12.20).

#### 12.5.4 Checkpoint-Policy

`checkpoint_policy` je Collection: `Rewrite` (**Standard**, ohne den Datensatz neu schreiben, Manifest und Prüfsumme neu ausstellen), `ExpireNow` (betroffene Checkpoints sofort verwerfen), `ExpireAfterHold` (nur unter `Restricted`/Legal Hold, höchstens `checkpoint_hold`, danach verwerfen). Der Standard wechselt gegenüber 2.2.1 von `ExpireAfterHold` auf `Rewrite` (O-15), weil Records nach der Vernichtung des DocDek in jedem Checkpoint unlesbar sind und ein Rewrite die Hygiene vollendet, ohne Wiederherstellungspunkte zu verlieren, die andere Einheiten betreffen. Bis zum Rewrite lautet das Ergebnis `Partial{outstanding: Checkpoints}` bzw. Klasse C ohne M, mit Anzahl, ältestem Alter und Ablaufdatum (`RetainedUnderRestoreGuard`). Ein gepinnter Checkpoint (`pin_checkpoint`) zählt zur Schicht `Checkpoints`. Interne Checkpoints sind **Spur D** (§12.17): Sie enthalten keine KeyTable.

#### 12.5.5 Aktive Snapshots

`forget` ermittelt den ältesten aktiven Snapshot, der den Scope sieht, wartet bis `deadline` auf dessen Ende (Obergrenze `snapshot_max_age`) oder schließt mit `excluded = ActiveSnapshot{ids}` ab; der Bericht trägt dann einen eingeschränkten Geltungsbereich (`proof_scope`). Offene Snapshots verzögern die physische Hygiene (Status `Pending`), **nicht** die Schlüsselvernichtung: Ein offener Snapshot liest über den `KeyCache`, der bei D2 invalidiert wird; er erhält danach `SnapshotExpired` und liest den Scope nicht mehr (INV-ASOF-ERASE-1).

### 12.6 AuditChain und RevocationLog

**AuditChain:** append-only, hash-verkettet, signierte Checkpoints, tenant-getrennt; enthält Zugriffe, Löschungen, Berichte (alle vier Siegel), Freigaben, Validierungsentscheidungen, `Unfreeze`. **RevocationLog:** widerrufene Schlüssel/Module/Berechtigungen, `key_destroyed`-Einträge, Haltezustände (Legal Hold); wird bei **jeder** Autorisierung konsultiert (fail-closed). Beide sind durabel (Flush vor Wirksamkeit, Persistenz über `DurableLog`) und crash-sicher; Tail-Truncation ist über einen signierten Kettenkopf erkennbar. Beide Dekoder sind Fuzz-Ziele. Audit-Inhalte enthalten nur Referenz-Hashes (INV-LOG-1). Schreiben DARF den bestehenden Log nie zerstören (REQ-DEL-022). Löschereignisse stehen mit `subject_ref_hash`, nie mit einer ID. Ein externer Zeitanker ist optional (§12.7.9).

### 12.7 Messprinzip, Witness und Löschbericht

#### 12.7.1 Messprinzip

1. **Messen vor Behaupten.** Ein Beleg entsteht nur aus einem Scanner der besitzenden Schicht (INV-WITNESS-MINT-1) und aus einer Messung am **Speicher**, nicht am Lesepfad: Sichtbarkeit (`scan_prefix`, Suche) belegt keine physische Abwesenheit.
2. Ein Messwert MUSS vom Scanner der Schicht erzeugt werden und trägt Methode, `units_scanned`, `residual_units`, einen Digest über Scan-Parameter und Dateiliste (kein Klartext) und einen Zeitstempel aus dem `Clock`-Port. `units_scanned = 0` ist nicht konstruierbar (Ausnahme: Schlüsselabwesenheit, §12.7.3).
3. Eine **Konstante oder ein vom Aufrufer gelieferter Zählerwert** DARF NIE als Messung dienen (REQ-DEL-008). **Es gibt keinen Fallback-, Standard- oder Testschlüssel für Berichte** (REQ-DEL-009).
4. Fehlt einer Schicht die Fähigkeit zu messen, ist sie `NotMeasured`; fehlende Fähigkeit ist `CapabilityUnsupported`, nie ein Erfolgsstub (REQ-DEL-010).
5. **Zwei unabhängige Messungen** je Pflichtschicht: Scanner der Schicht **und** Black-Box-Probe durch den Orchestrator über einen internen Prüfzugang.
6. **Positivkontrolle.** Jeder Scan-Lauf weist an einem absichtlich nicht gelöschten Kontroll-Datensatz nach, dass er anschlägt (AC-1); ein Lauf ohne bestandene Kontrolle ist **ungültig**.
7. **Verification und Validation sind getrennte Zustände** (NIST Rev. 2, §12.7.7).

#### 12.7.2 Ebene A: Klartext-Scanner

1. Rohbyte-Scan **aller** Dateien des Datenverzeichnisses (WAL, SSTables, Blob-Segmente, Index-Artefakte, Checkpoints, Temp, `.bak`, Keystore, KeyTable, Journal, Sidecars) nach kanonischen Termen, Roh-IDs, Embedding-Bytes und Chunk-Prüfsummen des Scopes.
2. Dekodierte Sicht über die normalen Lesepfade mit **allen aktiven Schlüsseln**.
3. Positivkontrolle (AC-1) in beiden Sichten.

Der produktive Scanner arbeitet **strukturell und per Rohbyte-Suche über bekannte Muster**; der Canary-Byte-Scan (Spike S-E1: 2.000 → 0 Treffer) ist zusätzlich Test-Werkzeug (§17.9).

#### 12.7.3 Ebene B: drei Nachweise (Schlüsselabwesenheit)

1. **Pre-Destruction-Inventar.** Vor der Vernichtung wird je `key_id` die Liste aller Chiffratblöcke (Datei, Offset, Länge) als Digest und Zähler festgehalten. Danach gilt: Jeder Block ist einem vernichteten oder lebenden Schlüssel zugeordnet; **Blöcke ohne Zuordnung sind ein Fehler (fail-closed)**. Das Inventar ist nur vor der Vernichtung möglich (deshalb D1).
2. **Nadelscan.** Gesucht wird die **Wrap-Nonce** (96 Bit, zufällig) des vernichteten Schlüssels, die vor der Vernichtung im Journal liegt, in allen Dateien: KeyTable, Keystore, Sidecars, Checkpoints, Temp, `.bak`. Ergebnis: **null Treffer**. Ein Hash des Blobs eignet sich nicht, weil die Rohbyte-Suche alle Offsets hashen müsste. Die Nadel wird nach Abschluss aus dem Journal gelöscht und nur als Digest geführt.
3. **Kanarien-Probe.** Ein Kanarien-Datensatz mit eigenem DocDek wird verschlüsselt und vom Scanner entschlüsselt, dann **über denselben Vernichtungspfad** (D1 bis D6) vernichtet; die Entschlüsselung muss scheitern. Die Probe belegt Scanner und Vernichtungsweg. Sie belegt **nicht** die Vernichtung des konkreten DocDek; dafür stehen Nadelscan und Inventar.

**Grenzen (stehen immer im Bericht):** Der Scanner sieht keine Flash-Remanenz unterhalb des Dateisystems, keinen Schlüssel in RAM, Swap, Dumps (A-RAM) und keine externen Backups.

#### 12.7.4 Typen

```rust
/// Nur über `ResidualReport::from_scan` konstruierbar. Kein `Default`, kein `From<usize>`.
pub struct ResidualReport {
    layer: DeletionLayer,
    plane: Plane,                 // A | B
    method: ScanMethod,           // RawFileScan | DecryptedViewScan | IndexIntrospection | BlackBoxProbe | NeedleScan | InventoryDiff
    units_scanned: u64,           // > 0, außer NeedleScan (Nadelabwesenheit zählt gescannte Dateien)
    residual_units: u64,
    control_matches: u64,         // positive Kontrolle (AC-1); > 0
    evidence_digest: [u8; 32],    // Scan-Parameter, Dateiliste, Prüfsummen; kein Klartext
    measured_at: UnixNanos,       // aus `Clock`, nicht vom Aufrufer
}

/// Konstruktion nur im sealed `ScanSession`-Pfad (siehe unten). Kein Default, kein Deserialize.
pub struct LayerWitness { /* Felder privat */ }
impl LayerWitness {
    pub fn layer(&self) -> DeletionLayer;     // Enum, kein String (U-41)
    pub fn scanned(&self) -> u64;
    pub fn remaining(&self) -> u64;
    pub fn kind(&self) -> WitnessKind;        // Structural | NotMeasured(NotMeasuredReason) | FeatureDisabled
    pub fn digest(&self) -> [u8; 32];
}
```

**Mint-Härtung (H-02, INV-WITNESS-MINT-1).** Der Zustand im Code ist, dass `LayerWitness::issue_from_scanner` `pub` ist und nur über `#[doc(hidden)]` „geschützt“ wird (U-40); jeder Crate kann damit einen Witness mit `remaining = 0` erzeugen. 2.3 verlangt: (a) Konstruktoren sind `pub(crate)`; (b) ein Witness entsteht ausschließlich in `ScanSession::finish(self, clock) -> LayerWitness`, und `ScanSession` ist nur über `LayerScanner::begin_scan` erhältlich, das eine bestandene Positivkontrolle (`ControlToken`) verlangt; (c) die Trait-Methode `LayerScanner::scan` hat eine **Default-Implementierung**, die `begin_scan`, `scan_units`, `finish` in dieser Reihenfolge ruft; Scanner-Crates implementieren nur `scan_units`; (d) das Gate `witness-mint` lehnt jede Konstruktion außerhalb von `ScanSession` ab, auch in Tests des Produktionscodes (Ausnahme: `testkit`).

#### 12.7.5 Bericht `DeletionProof` V5

**Format.** V5 ersetzt den in 2.2.1 eingefrorenen V4 (E26, ersetzt durch E46). **Der Repo-Stand kennt höchstens `SignatureVersion::V3`** (U-39); V4 existiert im Code nicht. 2.3 überspringt V4 nicht: V1 bis V3 bleiben lesbar (`coverage_known = false`), V5 ist das einzige schreibbare Format; die Nummer V4 bleibt unvergeben. Magic `MFDR` (§7.2), Golden-File, Migrationspfad, Fuzz-Ziel (REQ-FMT-1 bis -3). Serialisierung explizit spezifiziert (Feldreihenfolge, Endianness, Längen), nicht aus `serde`-Derives; `bincode` wird nicht für V5 verwendet (U-10 entfällt für V5).

**Pflichtfelder.** `class` (L, P, C*, C1, C2, C3, M), `seal_no` (1 bis 4), `legal_state`, `backup_window`, `assumptions[]`, `residuals[]`, je Schicht (`layer`, `plane`, `method`, `units_scanned`, `residual_units`, `control_matches`, `evidence_digest`, Ergebnis), `custody` und `DestroyAttestation`, Epoch-Seal-Referenz, `subject_ref_hash` (und `unit_ref_hash[]`), `scope`, `uncovered_layers[]` (`UncoveredLayer{layer, reason}`), `excluded[]` (**Pflicht, nie leer**), `perspectives[]` (je Q1 bis Q8 ein erzeugtes Urteil, §12.1.4), `verification`, `validation` (§12.7.7), `requested_at`/`completed_at` (aus `Clock`), `clock_source`, `platform_guarantee`, Verweis auf die AuditChain-Position, `signer_key_id`, Ed25519-Signatur (Schlüssel aus dem `KmsProvider`, INV-PROOF-KEY-1), Prüfsumme über alle Felder.

**NIST-Zertifikatsfelder.** Produkt und Version; Medium (Dateisystemtyp, Laufwerksklasse falls bekannt); Methode (Purge); Technik (Cryptographic Erase, Blockrewrite); Validierungsentscheidung; Concurrence; **CE-Traceability** (Algorithmen, Schlüsselstärken, Schlüsseltypen, Escrow-/Backup-Historie aus Spur K).

```rust
impl DeletionProof {
    /// Nur `ErasureOrchestrator` ruft `issue`; Aufrufer außerhalb von `engine` können keinen Bericht bilden.
    pub fn issue(ctx: &RequestContext, saga: &SagaRecord, witnesses: &[LayerWitness],
                 class: ReachedClass, assumptions: &[Assumption], residuals: &[Residual],
                 excluded: &[ExcludedScope], custody: &DestroyAttestation,
                 key: &SigningKeyHandle, clock: &dyn Clock) -> Result<DeletionProof, ProofError>;
    pub fn verify(&self, vk: &VerifyingKey) -> Result<VerifiedProof, ProofError>;
}
```

**Ist-Stand.** `DeletionProof::issue` ignoriert den Parameter `_custody` (U-42), kennt keine Klasse, keine Annahmen und keine Siegel. Das ist mit V5 zu beheben (WP L-9).

|ID|Regel|
|---|---|
|REQ-PRF-1|Der Stub `no_crypto_stubs` (`engine/src/lib.rs:17`) und die leeren Mandanten-Hooks werden **gelöscht**; `crypto` ist nicht-optionale Abhängigkeit der Engine. Der Stub akzeptiert heute **jede** Restzahl (`new_after_verified_empty` liefert immer `Ok`), das ist ein Verstoß gegen A1 (U-21).|
|REQ-PRF-2|`issue` liefert `Err`, wenn eine Pflichtschicht `remaining > 0` hat oder weder `Structural` noch `FeatureDisabled` ist, wenn die Klasse höher ist als die Messung trägt, wenn `excluded` leer ist oder wenn Annahmen fehlen, die die Klasse erfordert. `verify` mit falschem Schlüssel liefert `Err`.|
|REQ-PRF-3|Der Bericht nennt je Schicht Methode, Zahlen, Digest sowie `excluded`; die Außenformulierung wird aus dem Bericht erzeugt (INV-CLAIMS-1). Die `custody` wird für die Klasse **genutzt** (C1 bis C3).|
|REQ-PRF-4|`rg "default_test_deletion_proof_key"` im Produktionscode ergibt **0 Treffer** (heute `contextra-mcp/src/tools_crud.rs:206`, U-22). Der Schlüssel kommt aus dem `KmsProvider`. `contextra_delete` und `contextra_drop_collection` erzeugen keinen Bericht außerhalb der Saga (INV-NO-DUMMY-PROOF-1).|
|REQ-PRF-5|V5 mit Golden-File, Migrationspfad und Fuzz-Ziel (ersetzt „entfällt, E26“).|
|REQ-PRF-6|`doctor verify-report` prüft offline Signatur, Prüfsumme, Klasse, Siegelkette und Zeitanker (ersetzt „entfällt, E26“; REQ-DEL-N9). Ein unabhängiger Referenz-Prüfer in anderer Sprache dient **nur als Testorakel** (INV-ORACLE-1) und gehört nicht zum Produkt.|
|REQ-PRF-7|Jeder Bericht (alle vier Siegel) wird als Eintrag in die AuditChain geschrieben (Magic, Version, Prüfsumme, `fsync`) und ist für den Betreiber lesbar (`report(id)`).|

**Regression.** `review_stub_proof` bleibt dauerhaft im Repo: Bericht mit Restbestand → `Err`; falscher Schlüssel → `Err`; Bericht ohne Scanner-Witness → nicht kompilierbar.

#### 12.7.6 Restmetadaten-Selbsttest

Nach der Vernichtung prüft `doctor` stichprobenartig, ob verbleibende Chiffrat-Records einer gelöschten Einheit (a) sich über Klartext-Header zuordnen lassen (**No Linkage**), (b) eine einzigartige Länge haben (**No Record Isolation**, Verteilung nach Padding), (c) aus Zählern, Statistik oder Zeitstempeln Inhalte ableiten lassen (**No Inference**). Zusätzlich laufen Entropietests auf Restdaten, die Chiffrat sein müssen; ein unerwartet niedriger Wert zeigt einen Chiffrierfehler an. Der Test liefert eine **Warnung, kein Rechtsurteil**.

#### 12.7.7 Verification und Validation

|Zustand|Bedeutung|
|---|---|
|`Verified`|Alle Schichten gemessen, Positivkontrollen bestanden|
|`Validated`|Eine zuständige Person nimmt das Ergebnis ab (Validierungsentscheidung). **Im Regulated-Tier Pflicht, mit zweiter Signatur (Concurrence) einer anderen Person**|

Die Klasse im Bericht gilt für beide Zustände; `Validated` ist ein zusätzliches Feld. Ein Bericht ohne `Validated` im Regulated-Tier trägt den Vermerk `ValidationPending` und gilt nicht als Abschluss (REQ-DEL-N20).

#### 12.7.8 Löschbescheinigung (abstrakt, datensparsam)

Aus dem Bericht wird eine einseitige Bescheinigung erzeugt (OLG Dresden 4 U 940/24, AG Lörrach 3 C 1099/24). Sie **enthält**: Datum, Art des Scopes (z. B. „Akte“), erreichte Klasse je Schichtgruppe, Annahmen, Ausschlüsse (zusammengefasst, nie gekürzt weitergegeben), Bericht-Digest, Signatur, Zeitanker-Status. Sie **enthält nicht**: IDs, Dateinamen, Inhalte, Embedding-Daten. Sie dient der Beantwortung betroffener Personen, der Auftragsverarbeitung und Aufsichtsanfragen; ob und an wen sie auszuliefern ist, entscheidet der Verantwortliche. Ein Anspruch der betroffenen Person auf diesen Nachweis ist nicht gesichert (AG Lörrach).

#### 12.7.9 Zeitanker

Der Digest des Berichts wird optional über einen qualifizierten Zeitstempel (RFC 3161, eIDAS Art. 41, 42) verankert. Der Zeitstempel belegt, dass der Bericht zu diesem Zeitpunkt existierte (Datum und Datenintegrität), **nicht**, dass die Löschung stattgefunden hat. Offline-Betrieb: Anforderung, sobald eine Verbindung besteht (`anchor: pending`); Verifikation offline möglich, wenn Zertifikatskette und Vertrauensliste als Schnappschuss im Bericht liegen. Ein ausgestelltes Token ist nicht widerrufbar; der Zeitstempel überträgt nur den Digest. Anbieterwahl offen (O-34). Der Zeitanker ist ein Port `[R]` (`TimestampAuthority`), nicht Kern.

#### 12.7.10 Aufbewahrung des Berichts

Der Bericht enthält nur Referenz-Hashes und gilt selbst als personenbezogen (EDPB Rn. 52). Aufbewahrung nach `report_retention` (Rechtsfrage P-07), danach pseudonym-ablaufend (§12.10). **Beweiskraft:** Der Bericht ist eine signierte Behauptung des löschenden Systems; die Signatur belegt nicht, dass Bytes fehlen. Unabhängige Aussagekraft entsteht aus gemessener Bereinigung, Nadelscan, Inventar und reproduzierbaren Canary-Läufen (§17.9).

### 12.8 Löschsaga

#### 12.8.1 Zustände

```
Requested → Authorized → Frozen → LogicallyDeleted → PlaintextPurged → PlaintextMeasured
          → KeyDestroyed → CryptoMeasured → (Sealed) → Hygiene → HygieneMeasured → Reported → Completed
              │             └─(Aufbewahrungspflicht/Legal Hold, ab Frozen)→ Restricted{basis, until}
              └→ Rejected                                   (jeder Zustand) → Partial{outstanding, reason} → Retry/Escalate
```

Weitere Zustände: `ReviewRequired` (Mixed-Segmente, §12.15), `Unfrozen` (Rücknahme vor `KeyDestroyed`, §12.8.3).

#### 12.8.2 Reihenfolge (Fast-Path vor Hygiene, E42)

**Fast-Path** (Ziel: Minuten bis Stunden):

|Schritt|Zustand / Aktion|Durabilität|Klasse danach|
|---|---|---|---|
|1|`Frozen`: `ErasedSet`-Eintrag (mit `subject_ref_hash` und `unit_ref_hash[]`), Lesesperre, `legal_state = Eingeschränkt`; Spiegelung an den `ErasedSetMirror` (§12.17.3)|`fsync` **vor** jeder Änderung|**L** (Siegel 1)|
|2|`LogicallyDeleted`: Tombstone im WAL-Batch, Lineage-Kaskade|WAL-Commit|L|
|3|`PlaintextPurged`: Ebene A bereinigen (Arena-Slots nullen, Caches zeroizen, Postings im Speicher, Antwort-Cache); Text-Segmente neu schreiben (neuer ArtKey, alter ArtKey sofort vernichtet)|Journal-Eintrag vor jedem Schritt|–|
|4|`PlaintextMeasured`: Scanner Ebene A; **Pre-Destruction-Inventar** je `key_id`|Bericht mit Positivkontrolle|**P** (Siegel 2)|
|5|`KeyDestroyed`: DocDek vernichten nach Destroy-Protokoll D1 bis D5 (§12.2.7), nach optionaler `erasure_grace`|Journal-Intent vor Vernichtung|–|
|6|`CryptoMeasured`: Nadelscan, Kanarien-Probe, Chiffrat-Identität (D6)|Bericht|**C***|
|7|Epoch Seal (§12.2.4), soweit fällig oder `seal-now`|Journal|**C1/C2/C3** (Siegel 3)|

**Hygiene** (terminiert, ≤ `purge_rebuild_max_days`, Startwert 14 Tage):

|Schritt|Aktion|Begründung|
|---|---|---|
|8|Physische Kompaktion der betroffenen SSTables (`compact_range(scope)`), WAL-Scrub, Blob-Rewrite, Checkpoint-Policy|T1; Platz|
|9|Graph-Neuaufbau (Generationswechsel), danach ArtKey der alten Generation vernichten (§12.16)|schließt `R-GRAPH`|
|10|`HygieneMeasured`, `Reported`, Siegel 4|**M**|

**Begründung der Reihenfolge.** Records sind je Einheit verschlüsselt (§12.2.5); eine gemeinsame Blockstruktur macht die Vernichtung deshalb nicht kollateral. Die Fassung v3.0 (Bereinigung vor Vernichtung) verzögerte die wirksamste Maßnahme um bis zu 30 Tage. Ein Messargument für „Bereinigung zuerst“ bleibt (Inventar und entschlüsselte Sicht sind nur vorher möglich); es ist in Schritt 4 abgedeckt. **Verworfen: „Key-first“** (Vernichtung unmittelbar nach `Frozen`): sie verhindert die Messung der entschlüsselten Sicht und schließt die Rücknahme bei Irrtum aus; die Karenz (§12.8.3) ersetzt sie.

**Ordnungsinvariante (INV-DEL-ORDER-1, H-05).** Schritt 5 DARF NICHT beginnen, bevor Siegel 2 (Klasse P) für **jede Ebene-A-Schicht** der Einheit ausgestellt und das Pre-Destruction-Inventar durabel ist. Ausnahme: Notpfad (§7.10), in dem nur L erreichbar ist.

#### 12.8.3 Karenz und Rücknahme

Bis Schritt 5 ist die Löschung rücknehmbar (`Unfreeze`, Kategorie `Admin`, Begründung im Audit; **im Regulated-Tier zwei Personen**, H-10). `erasure_grace` (Startwert 0, Obergrenze 72 h, im Regulated-Tier Pflichtwert ohne Standard, REQ-DEL-N26) legt fest, wie lange nach Schritt 4 gewartet wird; währenddessen gilt `legal_state = Eingeschränkt`. Der Retention-Reaper (§11.6) nutzt die Karenz von 7 Tagen im Zustand `PendingDeletion`. Nach D3 gibt es **keine Rücknahme** (kein `Unfreeze`, nur Neuanlage aus Quellmaterial des Betreibers). Das Modell folgt dem Wartefenster üblicher KMS-Dienste (AWS 7 bis 30 Tage), ist aber auf Stunden verkürzt, um die Pflicht zur unverzüglichen Löschung nicht zu verzögern.

#### 12.8.4 Crash-Verhalten

- Absturz vor Schritt 5: Recovery setzt bei `PlaintextPurged` fort; der Scope bleibt gesperrt.
- Absturz nach D3: Die Vernichtung ist vollzogen; Recovery wiederholt D4 bis D6 und misst nur noch. **Ein Bericht trägt nie Klasse C vor abgeschlossener Messung.**
- Intent ohne Ausführungsnachweis wird beim Start wiederholt; Vernichtung ist idempotent.
- Der Epoch Seal ist ein eigener durabler Ablauf (§12.2.4).
- **Keine Wiederauferstehung (INV-NO-RESURRECTION-1, H-06).** Beim Start (B6/B7) und nach jedem Restore prüft die Engine für **jeden** `ErasedSet`-Eintrag im Zustand ≥ `Frozen`, dass kein lebender DocDek und kein lebender ArtKey der betroffenen Einheiten in der KeyTable steht; Abweichung ⇒ Einheit bleibt gesperrt, Vernichtung wird wiederholt (`resurrection_repaired` im Journal), Alarm.
- **Uhr (H-09, A-CLOCK).** Entscheidungen, die von Zeit abhängen (Horizont, Fristen, Karenz, `valid_until`), schlagen fail-closed fehl, wenn die Uhr hinter den zuletzt in AuditChain oder Journal geschriebenen Zeitpunkt zurückspringt; `doctor` weist `ClockRegression` aus. Nur die Verkürzung einer Frist (strengere Wirkung) darf bei unsicherer Uhr weiterlaufen.

#### 12.8.5 Siegel (gestufte Teilberichte)

|Siegel|Zeitpunkt|Inhalt|
|---|---|---|
|1|nach Schritt 1|Klasse L, `legal_state`|
|2|nach Schritt 4|Klasse P|
|3|nach Schritt 7|Klasse C1/C2/C3 mit Annahmen (bzw. C* mit `R-KEKEPOCH until`)|
|4|nach Schritt 10|Klasse M|

Jedes Siegel ist ein signierter Bericht, in die AuditChain eingetragen und mit dem vorherigen verkettet (REQ-DEL-N27). Der Betreiber kann damit nachweisen, **wann welcher Zustand erreicht war**.

#### 12.8.6 `Restricted` und Legal Hold

Bei Aufbewahrungspflicht (Art. 17 Abs. 3) oder Legal Hold geht die Saga nach Schritt 1 in `Restricted{basis, until}`: **keine Löschoperation, keine Schlüsselvernichtung**, Lesesperre bleibt (AC-11). Nach Ablauf setzt die Saga bei Schritt 2 fort. `ExpireAfterHold` gilt nur für Checkpoints. Ob und wie lange: Rechtsfrage (O-12). Das Archiv ist vom aktiven Retrieval-Index zu trennen (P-09). Ein Hold wird im `RevocationLog` und Audit geführt und betrifft die Saga, nicht die Lesesperre.

#### 12.8.7 Zustandsmodell: Regeln

|ID|Regel|
|---|---|
|REQ-FGT-1|Jeder Übergang wird **vor** der Ausführung ins durable Journal geschrieben (`fsync` mit Fehlerpropagation); jeder Schritt ist idempotent.|
|REQ-FGT-2|Beim Start (B7) setzt die Engine jede Saga ohne `Completed` fort oder meldet `Partial{outstanding}`.|
|REQ-FGT-3|`Frozen` schreibt den `ErasedSet`-Eintrag durabel **vor** der ersten Löschoperation; ab dann gilt die Lesesperre.|
|REQ-FGT-4|Fehler von `fsync`, `rename`, `write` in Purgern führen zu `Partial`, nie zu stillem Weiterlaufen.|
|REQ-FGT-5|Ein Scanner-Ergebnis mit `remaining > 0` hält die Saga in `PlaintextMeasured`/`HygieneMeasured` mit `ResidueRemaining{layer, count}`; es entsteht ein Bericht, kein höherer Klassenanspruch.|
|REQ-FGT-6|Die Saga ist abbrechbar nur **zwischen** Schritten; Abbruch hinterlässt `Partial`.|
|REQ-FGT-7|Der `RequestContext` (Mandant, Prinzipal) kommt vom Aufrufer; kein Fallback auf `TenantId::SYSTEM` und kein `unwrap_or(TxId::new(1))` (heute `tools_crud.rs:127,193`, `server_tools.rs:449`, U-23).|
|REQ-DEL-036|**Fristverfolgung:** Die Saga trägt `requested_at` und Schwellen (§12.19.1); jeder `Partial`-Zustand erzeugt je Schwelle genau einen Alarm, eine Eskalation bzw. eine Meldung (ohne Nutzinhalte). Kein `Partial` ohne ausstehende Schicht, Grund und Ablaufdatum. Contextra sendet nichts nach außen.|

#### 12.8.8 `forget` aus Aufrufersicht

Autorisierung (Kategorie `Destructive`) und Forget-Guard (§13.3) → Schritt 1 (`Frozen`, durabel) → **sofortige Rückgabe** eines `DeletionReceipt` mit ehrlichem Teilbericht (Siegel 1, Klasse L) und `PendingDeletion`-Handle → Schritte 2 bis 7 (Minuten bis Stunden) → Hygiene → Bericht M. `report(id)` liefert den jeweils höchsten Stand. Idempotent und wiederaufnehmbar nach Absturz.

### 12.9 Lesesperre, Key-Lock und Restore-Guard

- **Lesesperre.** Mit `Frozen` trägt der persistente `ErasedSet` nur Referenz-Hashes. `get`, `search`, `hydrate`, `export` und Checkpoint-Erzeugung prüfen ihn **vor** Rückgabe (REQ-DEL-011); sie gilt auch für `as_of` (INV-ASOF-ERASE-1). Die Prüfung verwendet beide Hashes (`subject_ref_hash`, `unit_ref_hash`).
- **Key-Lock.** Das Löschen hält den Key-Lock der Einheit über die **gesamte** Saga (REQ-DEL-012); konkurrierendes Schreiben darf nichts wieder auftauchen lassen (loom).
- **`ErasedSet`-Eintrag (H-08).** `{entry_seq, scope_ref, subject_ref_hash?, unit_ref_hash[], ref_epoch, requested_at, state ∈ {Frozen, Restricted, KeyDestroyed, Hygiene, Done}, prev_mac}`. `unit_ref_hash = HMAC(K_ref, doc_key_id)` trägt Löschungen **ohne** Betroffenenbezug (einzelnes Dokument, Collection). Ohne `unit_ref_hash` könnte der Restore-Guard Einheiten ohne `subject_ref` nicht wiederfinden.
- **Restore-Guard.** Vor dem ersten Lesezugriff nach einem Restore: (1) `ErasedSet` des **Zielsystems** lesen (nicht aus dem Backup); (2) für jeden Eintrag Schritt 3 ff. erneut anwenden, wiederhergestellte DocDeks vernichten (Destroy-Protokoll); (3) Messung wie §12.7; (4) Journal-Ereignis `restore_reapplied`; (5) Dokumentation (Grund, Zeitpunkt) im Audit (BayLfD 12.5). Der Restore-Guard greift **vor** B9 (`INV-BOOT-1`).
- **Restore-Horizont (INV-RESTORE-HORIZON-1, neu gefasst).** Der Horizont gilt für **Schlüssel-Backups** (Spur K): Ein Schlüssel-Backup mit `valid_until` in der Vergangenheit oder unbestimmbarem Alter wird verweigert (`RestoreBeyondHorizon`, final). Daten-Backups (Spur D) unterliegen keinem Horizont, weil sie ohne Schlüssel inert sind. Die Konfiguration erzwingt `key_backup_max_age ≤ erased_set_retention` (Startfehler in B0). Ein `ErasedSet`-Eintrag bleibt mindestens `erased_set_retention` nach der Löschung zuordenbar (§12.10).
- **Unbekanntes Alter.** Ist das Erzeugungsdatum eines Schlüssel-Backups nicht aus dem signierten Manifest bestimmbar oder greift `ClockRegression`, wird verweigert.

|ID|Regel|
|---|---|
|REQ-RST-1|`ErasedSet` ist durabel, enthält nur Referenz-Hashes (keine Klartext-IDs) und wird in B6 vor Verkehr geladen (INV-BOOT-1).|
|REQ-RST-2|`restore_checkpoint` sperrt vor dem ersten Lesezugriff, wendet jeden Eintrag erneut an und schreibt `restore_reapplied` ins Journal (REQ-DEL-013).|
|REQ-RST-3|Schlüssel-Backups, deren signiertes `valid_until` überschritten oder deren Alter nicht bestimmbar ist, werden fail-closed verweigert; Start prüft `key_backup_max_age ≤ erased_set_retention`.|
|REQ-RST-4|Checkpoint-Erzeugung und Export prüfen den `ErasedSet` vor dem Schreiben.|
|REQ-RST-5|Backups außerhalb von Contextra stehen als `ExternalBackups`/`ExternalUnmanaged` im Bericht; die Rücksicherungsregel trägt der Verantwortliche.|

### 12.10 Referenzschlüssel, Datensparsamer Bericht, Pseudonym-Ablauf

**Zweck.** Berichte, `ErasedSet`, `KeyRegistry` und Journal müssen auf Betroffene verweisen, ohne sie zu identifizieren. **Terminologie (H-13).** Der Begriff „Commitment“ der Fassung 2.2.1 entfällt: `HMAC-SHA256` ist ein **Hash mit Schlüssel** im Sinne EDPB Rn. 52 und kein kryptografisches Commitment. Er heißt **Referenz-Hash**; der Schlüssel **Referenzschlüssel** `K_ref`. Ein echtes Commitment-Schema ist optional (O-36).

**Regeln.**

1. `subject_ref_hash = HMAC-SHA256(K_ref[tenant, ref_epoche], kanonische Betroffenen-Referenz)`. Der Hash bleibt **personenbezogen** (Rn. 52), solange `K_ref` existiert.
2. Die kanonische Referenz hat **Mindestentropie** (zufällige Betroffenen-ID, nicht Name oder E-Mail) und steht nur im Dokumentdatensatz. `unit_ref_hash` bildet über die zufällige 128-Bit-`doc_key_id`.
3. `K_ref` ist zufällig, nicht per HKDF ableitbar, unter `KEK` gewrappt, je (Tenant, Epoche); es wird vom Epoch Seal mitgenommen.
4. Nach `ref_epoch_len` (90 Tage) **plus** `erased_set_retention` wird `K_ref[tenant, epoche]` vernichtet (Destroy-Protokoll) und im `RevocationLog` eingetragen. Danach sind die Hashes dieser Epoche keiner Person mehr zuordenbar, soweit `K_ref` nicht in Schlüssel-Backups steht (Spur K ist befristet: `key_backup_max_age ≤ erased_set_retention`, daher unkritisch). Der Ablauf entfernt keine Kettenglieder, sondern die Zuordenbarkeit; die Ketten bleiben über ihre Köpfe prüfbar.
5. Sidecars enthalten keine Nutzinhalte und keine Schlüssel.
6. `erased_set_retention ≥ key_backup_max_age + Sicherheitszuschlag` (Startwert insgesamt 90 Tage; Obergrenze nach Rechtsprüfung P-06, P-07, P-16; die 365 Tage der Fassung 2.2.1 sind nicht mehr nötig, weil nur Schlüssel-Backups eine Rücksicherung gelöschter Inhalte ermöglichen). In `EnterpriseRegulated` Pflichtwert ohne Standard; nie unbegrenzt.

Kipptest: Will der Eigentümer auch die Referenz-Epochen streichen, werden `ErasedSet`-Einträge nach `erased_set_retention` per atomarem Rewrite entfernt (O-17); der Restore-Guard verliert dann die Wiedererkennung und `restore_unguarded` wird ausgewiesen.

### 12.11 Claim-Taxonomie und Außenformulierung

|Claim|Voraussetzung (gemessen)|
|---|---|
|`LogicalSuppression`|Klasse L: Lesesperre aktiv, Black-Box-Probe findet nichts|
|`PhysicalPurge{layer}`|Klasse P bzw. Hygiene: Segment neu geschrieben, Altdatei entfernt, Rohdateiscan ohne Canary, `units_scanned > 0`|
|`KeyDestroyed{custody}`|D1 bis D6 vollzogen, Nadelscan 0, Inventar vollständig|
|`CryptoErasure{C1\|C2\|C3}`|wie `KeyDestroyed`, Epoch Seal vollzogen, Annahmen benannt|
|`Restricted`|Aufbewahrungspflicht/Legal Hold; Lesesperre aktiv; keine Löschoperation|

**Verboten** (Gate `deletion-honesty`): „100 % gelöscht“, „unwiederbringlich“, „DSGVO-konform“, „Art.-17-konform“, „rechtskonform gelöscht“, „garantiert gelöscht“, „kryptografisch gelöscht“ ohne Custody und Annahmen, „Löschzertifikat“ ohne Scope und Klasse, „Löschung nach Art. 17“, jede Rechtsbehauptung.

**Satzbibliothek (INV-CLAIM-BIND-1).** Außentexte entstehen **nur** aus einer festen Satzbibliothek; es gibt keinen Freitext. Jeder Satz ist an eine gemessene Tatsache gebunden; ein Satz ohne gemessene Tatsache ist ein **Build-Fehler** (Gate `claim-generator`). Beispiele:

- **Klasse C1:** „Der Scope ist gesperrt. Klartextstrukturen sind bereinigt. Persistente Kopien sind nur als Chiffrat vorhanden. Die zugehörigen Schlüssel sind vernichtet. Dies gilt unter den Annahmen A-FLASH, A-ALGO, A-KEYLEAK, A-RAM. Folgende Ausschlüsse bestehen: …“
- **Klasse M:** wie C, ergänzt um die vollzogene physische Bereinigung und die Liste der Pflicht-Ausschlüsse.
- **Klasse L:** „Der Scope ist für alle Lesezugriffe gesperrt. Eine Bereinigung der Daten ist **noch nicht** gemessen.“

Das Gate `deletion-honesty` vergleicht jeden **statischen** Außentext (README, MCP-Werkzeugbeschreibungen, `doctor`-Texte) mit den Claims, die der Bericht der Standardkonfiguration (K1: höchstens C1) ausstellt; ein Text, der mehr behauptet, ist Gate-Fehler (REQ-DEL-035, AC-19, AC-24). Die Bezeichnung „Außenformulierung der Fassung 2.2.1“ („Der lokale Schlüssel wurde nicht als Löschmaßnahme vernichtet“) entfällt, weil sie der Wirklichkeit in 2.3 widerspricht.

### 12.12 Abnahmekriterien Löschung

Eine Löschung gilt erst als bewiesen, wenn alle gültigen Kriterien grün sind. AC-7, AC-12, AC-14, AC-15 bleiben **unbelegt** (Inhalt der Fassung 1.3 nicht rekonstruierbar, §25.3); die Nummern werden nicht neu vergeben. Die Epochen-Anforderung deckt AC-21 ab.

|ID|Kriterium|Methode|
|---|---|---|
|AC-1|Nach Löschung und Bericht enthält **keine** Datei des Datenverzeichnisses (WAL, SSTables, Blob-Segmente, Index-Artefakte, Checkpoints, Temp, `.bak`, Keystore, KeyTable, Journal) den Canary (≥ 32 Zufallsbyte als Text), dessen Embedding-Bytes, eindeutige Terms oder die Roh-ID. Geprüft in **zwei Sichten**: Rohbytes und entschlüsselte Sicht mit allen überlebenden Schlüsseln. **Positive Kontrolle** in beiden Sichten; sonst ist der Lauf ungültig.|Scanner im Test + unabhängiger `grep -r` + Positivkontrolle|
|AC-2|Wird eine Messung durch eine Konstante ersetzt, schlägt mindestens ein Test fehl.|Mutation|
|AC-3|Absturz an jedem Journalzustand und an jedem Schritt D1 bis D6: danach `Completed` mit vollständigen Messungen oder `Partial`, nie `Completed` ohne Messung.|Crash-Matrix (`FaultVfs`)|
|AC-4|Restore eines Checkpoints vor der Löschung: Betroffener gesperrt, danach erneut gelöscht, `restore_reapplied` im Journal.|Integration|
|AC-5|Zwischen `Frozen` und `Completed` liefert kein Lesepfad Daten des Betroffenen, auch parallel.|loom/Property|
|AC-6|Berichts-Negativsuite: Bit-Flip je Feld, Feldvertauschung, Versions-Downgrade, anderer Mandant, falscher Schlüssel, fehlendes `excluded`, Klasse ohne Annahmen, gefälschtes Siegel ⇒ Fehler mit benanntem Grund.|Unit + Fuzz|
|AC-8|Neustart nach Löschung: gesperrt/gelöscht, Journal konsistent.|Integration|
|AC-9|Mit `ManualClock` und festen Eingaben ist der Bericht bytegleich reproduzierbar.|Golden-Test|
|AC-10|Das Gate `deletion-honesty` erkennt konstante Messwerte, verworfene Prüfergebnisse, unbedingte `Ok(true)`-Verifikation und öffentliche Konstruktoren mit Zählerwerten in einem absichtlich fehlerhaften Beispiel.|Gate-Selbsttest|
|AC-11|Legal Hold ⇒ `Restricted`, keine Löschoperation, **keine Schlüsselvernichtung**, Lesesperre aktiv, `basis` und `until` im Bericht.|Integration|
|AC-13|Gleichzeitiges `put`/`delete` derselben ID ergibt nach Abschluss nie einen sichtbaren Zustand.|loom|
|AC-16|Checkpoint vor der Löschung: Bericht höchstens Klasse C ohne M mit `RetainedUnderRestoreGuard{count, oldest_age, expires_at}`, solange der Checkpoint die Einheit enthält; nach `Rewrite`/`ExpireNow`/Ablauf Klasse M.|Integration + Canary-Scan|
|AC-17|`restore_checkpoint` weist ein Schlüssel-Backup, das `valid_until` überschritten hat oder dessen Alter unbestimmbar ist, mit `RestoreBeyondHorizon` ab; eine Konfiguration mit `key_backup_max_age > erased_set_retention` startet nicht.|Integration + Chaos|
|AC-18|Nach Vernichtung von `K_ref[tenant, epoche]` ist kein Eintrag dieser Epoche in `ErasedSet`, `AuditChain` und Journal mehr einem Subjekt zuordenbar; die Ketten bleiben prüfbar.|Property|
|AC-19|Kein statischer Außentext enthält einen Claim, den der Bericht der Standardkonfiguration (K1) nicht ausstellt.|Gate-Selbsttest|
|AC-20|Mit `ManualClock`: ein `Partial`-Zustand erzeugt nach den drei Schwellen je genau einen Alarm, eine Eskalation bzw. eine Meldung, reproduzierbar, auch nach Absturz.|Integration + Chaos|
|AC-21|**Epoch Seal.** Nach dem Seal: Re-Wrap aller lebenden Schlüssel, `KEK[e]` vernichtet, keine Datei des Datenverzeichnisses enthält Wrap-Nonce oder Wrap-Blob eines vernichteten Schlüssels; Abbruch an jedem Teilschritt endet in Fortsetzung, idempotent.|Crash-Matrix + Property|
|AC-22|**Backup-Trennung.** Kein Daten-Backup enthält Schlüsselmaterial (Format- und Literalscan). Backup vor Löschung, Löschung, Restore mit altem und mit frischem Schlüssel-Backup: Scope bleibt gesperrt, kein lesbarer Rest; abgelaufenes Schlüssel-Backup wird verweigert.|Backup-/Restore-Test|
|AC-23|**Mixed-Segmente.** `BlockAll`, `RedactAndRekey`, `DeleteAll` verhalten sich nach §12.15; kein Segment wird automatisch freigegeben oder gelöscht.|Integration|
|AC-24|**Claim-Generator.** Jeder Satz der Satzbibliothek ist an einen Messwert gebunden; ein absichtlich überzogener Text lässt das Gate fehlschlagen; die Perspektivenurteile folgen den Messwerten.|Gate-Selbsttest|
|AC-25|**Nadelscan.** Eine absichtlich zurückgelassene Wrap-Nonce (Test-Mutation) wird gefunden; ohne Nonce 0 Treffer.|Mutation + Property|
|AC-26|**Kein Dummy-Bericht.** Kein Codepfad erzeugt einen `DeletionProof` ohne Scanner-Witness; `rg` nach `default_test_deletion_proof_key` und `new_after_verified_empty(…, 0)` im Produktionscode ergibt 0 Treffer; Mutation (Witness-Konstante) lässt Tests scheitern.|Gate + Mutation|
|AC-27|**Keine Wiederauferstehung.** Nach jedem Crash und jedem Restore steht kein lebender Schlüssel einer `ErasedSet`-Einheit in der KeyTable.|Crash-Matrix + Property|
|AC-28|**Destroy-Durabilität.** Stromausfall an jedem Punkt von D1 bis D6 (`FaultVfs`): Schlüssel lebt und Einheit bleibt `Frozen`, oder Schlüssel ist vernichtet; nie „vernichtet“ gemeldet bei vorhandenem Schlüssel.|Chaos|
|AC-29|**Mirror.** Restore nach Totalverlust: mit aktuellem `ErasedSetMirror` bleibt der Scope gesperrt; ohne aktuellen Mirror nur mit `--accept-unguarded`, `restore_unguarded` im Bericht.|Integration|
|AC-30|**Padding und Header.** Kein Record trägt `doc_key_id` im Klartext; die Längenverteilung bleibt nach Padmé unter dem Overhead von 12 %; der Restmetadaten-Selbsttest (§12.7.6) schlägt an einem absichtlich unpaddeten Beispiel an.|Format-Test + Property|
|AC-31|**Rücknahme.** `Unfreeze` nach D3 wird abgewiesen; im Regulated-Tier braucht `Unfreeze` zwei Personen.|Integration|
|AC-32|**Uhr.** Bei Uhrrücksprung verweigern Horizont-, Karenz- und `valid_until`-Entscheidungen fail-closed (`ClockRegression`).|Chaos (Uhrsprung)|
|AC-33|**Verification/Validation.** Im Regulated-Tier ist ein Bericht ohne Concurrence-Signatur `ValidationPending` und gilt nicht als Abschluss.|Typestate + Integration|

### 12.13 Löschpfad: Komponenten und Platzierung

**Entwurfsprinzipien:** (1) Messen vor Behaupten: Ein Claim existiert nur, wenn ein Typ, der nur aus einer Messung konstruierbar ist, ihn trägt. (2) Fail-Closed: keine Bescheinigung bei Teilausfall, keine Fallbacks. (3) Verteidigung in der Tiefe: Sperre → logische Löschung → Klartextbereinigung → Schlüsselvernichtung → Seal → Hygiene → Messung → Bericht. (4) Ehrliche Grenzen (§2.6, §12.20). (5) Kein Architekturverstoß: Ports in `ports`, reine Logik in `crypto`, Persistenz in `store`, Orchestrierung in `engine`, Handler dünn. (6) Datensparsame Evidenz. (7) Determinismus: Zeit nur über `Clock`, IDs aus Zählern und Hashes, Schlüsselmaterial nur aus dem System-RNG.

|Komponente|Crate|Ring|Inhalt|
|---|---|---|---|
|IDs, Fehler, Grundtypen: `ErasureId`, `SubjectRefHash`, `UnitRefHash`, `ErasureError`, `DeletionLayer` (zehn Varianten), `LayerWitness`-Grundtypen, `ReachedClass`, `Assumption`|`types`|0|reine Datentypen|
|Ports: `LayerScanner`, `Purger`, `ErasureJournal`, `Vfs`, `DurableLog`, `KmsProvider`, `EffaceableKeySlot`, `ErasedSet`-Zugriff, `ErasedSetMirror`, `RecipientLedger`, `TimestampAuthority` `[R]`|`ports`|0|nur Traits|
|Reine Krypto- und Kodierungslogik: Bericht V5 (Signatur, Verifikation, Satzbibliothek), `ResidualReport`, KeyTable- und Wrap-Logik, Padmé, Kettenlogik|`crypto`|0|synchron, I/O-frei, lock-frei|
|Persistenz: atomare Dateien, KeyTable-Datei, `DurableLog`-Impl, Journal-Dateien, Rohdatei-Scanner|`store`|1|`fsync`, Temp-Datei und Rename, Verzeichnis-`fsync`|
|Scanner/Purger je Schicht: LSM/WAL/Blob/KeyMaterial (`store`), HNSW (`vector`), Volltext (`text`), Checkpoint (`checkpoint`), Caches/Audit/Lineage (`store`/`engine`)|je Crate|0/1/3|implementieren `LayerScanner`/`Purger` (Default-`scan` über `ScanSession`)|
|Orchestrator: Saga, Lesesperre, Key-Lock, Epoch Seal, Restore-Guard, Legal-Hold-Gate, Fristen|`engine`|3|verdrahtet Scanner/Purger über Ports; `crypto` ist nicht optional|
|Werkzeug `memory_forget`, CLI (`doctor`, `confirm`, `verify-report`, `seal-now`)|`mcp` über `contextra`|4|nur Aufruf und JSON-Abbildung; **keine Berichtskonstruktion**|
|Test-Doubles (Fake-Uhr, Fehlerinjektion, Fake-Verwahrer)|`testkit`|–|nur dort|

Löschtypen existieren genau einmal (Kantenregel 5, Gate `deletion-types-once`). Ohne Löschpfad-Orchestrator liefert dessen Konstruktor `Err(CapabilityUnsupported)`, nie einen Stub, der Erfolg meldet (REQ-DEL-027). Die MCP-Antwort enthält `class`, `claims`, `excluded` und den Bericht (oder `null` bei `Partial`/`Restricted`), nie einen Nachweis ohne Messung.

### 12.14 Konformität, Risiken, Löschinventar

`crypto` und `engine` bleiben `#![forbid(unsafe_code)]`; Zeit über `Clock`; Schlüsselmaterial ausschließlich über den System-RNG; Produktionscode ohne `unwrap`/`expect`/`panic!`; Verschlüsselungstypen existieren unabhängig von Features (A1).

|Risiko|Gegenmaßnahme|
|---|---|
|Strukturelle Messung übersieht Reste|Nadelscan, Inventar, Canary-Byte-Scan, `excluded` im Bericht; Fachprüfung vor jeder Vermarktung|
|Flash-Reste gewrappter Schlüssel (K1)|Epoch Seal, Passphrasen-Rotation, Annahme A-FLASH ausgewiesen, K2/K3 per Port|
|Schlüssel-Backup enthält vernichteten Schlüssel|befristet (`key_backup_max_age`), Restore-Guard vernichtet beim Einspielen|
|Abbruch während Vernichtung|Destroy-Protokoll D1 bis D6, idempotent, Crash-Matrix|
|Wiederauferstehung durch Restore oder Fehler|INV-NO-RESURRECTION-1, Prüfung in B6/B7|
|Purge-Kompaktion/Segment-Rewrite kostet Latenz und Write-Amplification|Segmentgrößen begrenzen; Messung mit der Write-Amplification-Suite (§16)|
|Padding und Re-Wrap kosten Platz und Zeit|Padmé ≤ 12 %, Re-Wrap O(Schlüssel); Budgets in `budgets.toml` (O-24, O-40)|
|Ablauf der Referenzschlüssel schwächt den Restore-Guard|Horizont bezieht sich nur auf Schlüssel-Backups (§12.9)|
|Fehlbedienung (irrtümliche Löschung)|Karenz `erasure_grace`, Forget-Guard (§13.3), `Unfreeze` bis D3|
|Rechtswertung offen|kein Produkttext ohne fachkundige Freigabe (§25.1)|

**Löschinventar (`doctor`, REQ-DEL-037).** Die Software erzwingt keine Löschfristen, macht ihr Fehlen aber sichtbar. `doctor` weist je Collection ohne Nutzinhalte aus: (1) ob eine Löschfrist/`RetentionPolicy` gesetzt ist (nein = Warnung); (2) Verwahrungsstufe K0/K1/K2/K3; (3) Anteil `LegacyUnkeyed`; (4) interne Checkpoints (Anzahl, ältester, Policy); (5) `erased_set_retention`, `key_backup_max_age`, `kek_epoch_len`, Alter des letzten Seal, Alter der ältesten Schlüssel-Backups, Konsistenz; (6) offene `Partial`-Sagas mit Alter und ausstehender Schicht; (7) `ErasedSetMirror`-Zustand; (8) Härtungslücken (§12.18); (9) offene Residuale (`R-GRAPH`, `R-KEKEPOCH`) mit Frist. Das Inventar ist Beobachtung, keine Rechtsaussage.

### 12.15 Betroffenen-Einheiten, Mehrfachbezug, Mixed-Policy

**Einheit.** Eine Betroffenen-Einheit ist die kleinste Menge von Records, die ohne Kollateralverlust für einen Betroffenen gelöscht werden kann. Standard: ein Dokument mit genau einem Betroffenen.

**Referenzen.** Jedes Dokument trägt `subjects: [subject_ref]`; `subject_ref` ist eine zufällige ID mit Mindestentropie (§12.10), vergeben vom Aufrufer (oder durch eine optionale Erkennung `[R]`). Ohne Angabe gilt das Dokument als Einheit des Aufrufers.

**Mehrfachbezug.** Ein Dokument mit mehreren Betroffenen wird in Segmente zerlegt; jedes Segment erhält die Menge seiner Betroffenen; Segmente mit gleicher Menge teilen einen DocDek. Bei Löschung für Betroffenen A:

|Segmentart|Behandlung|
|---|---|
|Menge = {A}|DocDek vernichten|
|Menge enthält A und andere (Mixed)|Standard `BlockAll`: Segment bleibt gesperrt, `Partial{ReviewRequired}` mit Frist; der Betreiber entscheidet|
|Menge enthält A nicht|unberührt|

**Mixed-Policy** (je Collection): `BlockAll` (fail-closed, **Standard**), `RedactAndRekey` (Betreiber liefert redigierten Text; neuer DocDek, Re-Embedding, alter DocDek vernichtet), `DeleteAll` (Segment vollständig löschen; Betreiber trägt die Verantwortung für den Kollateralverlust). **Contextra entscheidet nie selbst**, ob ein Mixed-Segment nach Entfernung von A noch personenbezogen ist (Art. 17 Abs. 3, Rechtsfrage). `BlockAll` ist rechtlich eine Einschränkung (Klasse L für das Segment) und läuft in die Fristenverfolgung (§12.19).

**Ableitungen** (Zusammenfassungen, Chunk-Präfixe, Embedding-Caches) kaskadieren über die Lineage (INV-DERIVED-1); Ableitungen mit weiteren Quellen werden neu abgeleitet oder als `stale` markiert.

**Collection- und Tenant-Löschung (Akte).** Alle DocDeks, ArtKeys und `KEK[coll,*]` werden in einer Saga vernichtet; die Index-Dateien der Collection entfallen; bei Tenant-Löschung zusätzlich `K_ref` des Tenants nach Retention. Die Fassung 2.2.1 behandelte dies als „Hygiene ohne Claim“; in 2.3 ist es eine **Löschmaßnahme mit Bericht** (E27 ersetzt).

### 12.16 Index-Generationen und Restinformation

**`R-GRAPH`.** Im HNSW-Graph bleiben nach der Löschung Routing-Knoten ohne Nutzdaten (VETO-F02 verbietet Rewiring). Die Kanten enthalten nur interne Slot-Nummern; die Topologie verrät Nachbarschaftsbeziehungen zu gelöschten Vektoren (No Inference). Sie bleibt als `R-GRAPH` mit Frist im Bericht, bis der Neuaufbau erfolgt ist.

**Neuaufbau.** (1) Hintergrund-Neuaufbau erzeugt Generation g+1 ohne gelöschte Dokumente (deterministisch, INV-INDEX-APPLY-1); (2) atomarer Umschalt-Commit auf g+1; (3) Vernichtung des ArtKey der Generation g. Mehrere Löschungen teilen sich einen Neuaufbau. Auslöser: Tombstone-Anteil ≥ `tombstone_rebuild_ratio` (0,2), Fristablauf (`purge_rebuild_max_days`, Startwert 14), `seal-now`. **VETO-F02 bleibt unberührt:** Der Neuaufbau ist ein vollständiger Generationswechsel, kein partielles Rewiring.

**Restkanäle.** Treffermengen: Ghost-Knoten verändern in der Studie 95 % der Top-K-Mengen und erzeugen einen Timing-Unterschied von rund 4 %; der Neuaufbau beseitigt sie. Statistik (IDF, Dokumentlängen, Zähler) liegt im Text-Segment und wird mit der Segment-Neuschreibung neu berechnet.

**Einordnung.** Qdrant, Milvus, Weaviate und Chroma/FAISS behandeln Löschung als Wartung ohne Frist und ohne Nachweis (Sekundärquellen). Contextra erzwingt die Frist und misst.

### 12.17 Backup und Restore (Zwei-Spuren-Backup)

#### 12.17.1 Spuren

|Spur|Inhalt|Lebensdauer|Schutz|
|---|---|---|---|
|**D** (Daten)|SSTables, WAL, Blob-Segmente, Index-Artefakte (Chiffrat), **ohne** KeyTable, **ohne** Keystore, ohne `ErasedSet`|beliebig (Betreiberentscheid)|ohne Schlüssel nicht lesbar|
|**K** (Schlüssel)|KeyTable-Snapshot (gewrappte DocDeks, ArtKeys, KEK-Epochen, `K_ref`)|**befristet** auf `key_backup_max_age` (Startwert 30 Tage), signiertes `valid_until`|verschlüsselt unter der Wurzel|

**Wirkung.** Nach Löschung und Ablauf aller Schlüssel-Backups, die den DocDek enthalten, existiert der Schlüssel nirgends mehr; Daten-Backups sind für diese Einheit unlesbar. Die Löschung wirkt in Backups spätestens nach `key_backup_max_age`, **ohne Eingriff in das Backup**. Bericht: `backup_window = OpenUntil(deletion_at + key_backup_max_age)` für von Contextra verwaltete Backups, `ExternalUnmanaged` für alles andere.

**Regeln.** INV-BACKUP-SPLIT-1: Kein Daten-Backup enthält Schlüsselmaterial; jedes Schlüssel-Backup trägt ein Ablaufdatum und wird danach verweigert. Der interne `checkpoint` ist Spur D (§12.5.4); `export` schreibt Spur D und optional Spur K nur über einen eigenen, protokollierten Befehl (`export-keys`), der `valid_until` setzt.

#### 12.17.2 Restore-Guard und Horizont

Siehe §12.9. Zusätzlich: Ein Restore **ohne** Schlüssel-Backup lässt Daten inert (Q6). Ein Restore mit Schlüssel-Backup innerhalb `valid_until` stellt Schlüssel zurück, die der Guard für jede `ErasedSet`-Einheit sofort wieder vernichtet, **bevor** B9 Verkehr zulässt.

#### 12.17.3 `ErasedSetMirror`

Nach Totalverlust fehlt der `ErasedSet` des Zielsystems. Der `ErasedSetMirror` (Port, Magic der Einträge `MFEM`) schreibt jeden `ErasedSet`-Eintrag (Referenz-Hashes, Zeitpunkt, Epoche) zusätzlich an ein vom Betreiber bestimmtes Ziel **außerhalb** des Datenverzeichnisses (zweiter Datenträger, WORM, Fremdsystem). Im Regulated-Tier ist er **Pflicht**. Ohne Mirror oder bei einem Mirror, der älter ist als das Schlüssel-Backup, ist ein Restore nur mit `--accept-unguarded` möglich; Bericht und `doctor` weisen `restore_unguarded` aus. **Schreibreihenfolge (H-11):** Ist der Mirror beim Einfrieren nicht erreichbar, blockiert das `Frozen` **nicht** (Löschen darf nicht an Fremdsystemen scheitern); der Eintrag wird `MirrorPending`, `doctor` alarmiert nach 24 h, und der Bericht trägt `residuals: [R-MIRROR until]`. Ein zentrales Löschregister (O-35) ist ein zulässiges Mirror-Ziel.

#### 12.17.4 Verhältnis zu Aufsichtsauffassungen

CEF Issue 6 und BayLfD 12.5 verlangen Nachverfolgung und Nachlöschung beim Restore sowie Begründung, wenn Backups nicht sofort bereinigt werden; der Zwei-Spuren-Ansatz erfüllt dies mit einem festen, dokumentierten Zeitfenster. Ob ein Chiffrat ohne Schlüssel für den Besitzer des Daten-Backups keine personenbezogene Information mehr ist, bleibt P-02.

### 12.18 Härtung (Prozessspeicher, Dumps, Swap)

NIST verlangt für CE, dass vor der Verschlüsselung kein Klartext auf das Medium gelangt ist (LA-15). Contextra setzt:

1. **Core-Dumps aus:** `RLIMIT_CORE = 0`, Prozessattribut gegen Dumps; Schlüssel- und Klartext-Arenen sind von Dumps ausgenommen (`MADV_DONTDUMP` bzw. Plattformäquivalent).
2. **Schlüsselspeicher gesperrt:** `mlock` für Schlüsselpuffer; meldet die Sperrung Fehler, weist `doctor` dies aus.
3. **Swap:** `doctor` prüft, ob Swap aktiv und verschlüsselt ist; große Arenen lassen sich nicht sperren, daher Warnung bei unverschlüsseltem Swap.
4. **Temp-Dateien:** keine Klartext-Zwischenspeicher; Zwischenergebnisse liegen im Speicher oder verschlüsselt unter ArtKey/`K_eng`.
5. **Dateisystem:** unterstützte lokale Typen (INV-PLATFORM-1); Journal, Snapshots (APFS, btrfs, zfs) und Time Machine gelten als externe Kopien und stehen im Bericht.
6. **Logs:** nur IDs und Zähler (INV-LOG-1).
7. **Hibernation:** `doctor` weist auf aktive Hibernation hin (A-RAM).

Die plattformspezifischen Aufrufe sind in Spike S-HRD zu verifizieren (O-38). Bis dahin gilt: Fehlt ein Mittel, meldet `doctor` die Lücke, und A-RAM bleibt im Bericht. Im Regulated-Tier ist eine bekannte Härtungslücke eine **Startwarnung mit Bestätigungspflicht**, kein stiller Betrieb.

### 12.19 Fristen, Empfänger, Legal Hold

#### 12.19.1 Fristen

|Frist|Quelle|Wert|
|---|---|---|
|Beantwortung|Art. 12 Abs. 3|ein Monat; Verlängerung um zwei Monate mit Mitteilung binnen eines Monats|
|`Frozen` (Klasse L)|intern|sofort nach Autorisierung|
|Klasse P|intern, Zielgröße|≤ 1 Stunde|
|Klasse C*|intern|unmittelbar nach P (zuzüglich `erasure_grace`)|
|Epoch Seal|intern|≤ `seal_after_deletion_max` (7 Tage; Regulated ≤ 72 h)|
|Hygiene (Klasse M)|intern, P-11|≤ `purge_rebuild_max_days` (14 Tage)|
|Alarm bei `Partial`|intern|24 h|
|Eskalation bei `Partial`|intern|14 Tage|
|Meldung Fristgefährdung|intern|25 Tage|

Die Werte sind Zielgrößen. **Contextra sendet nichts nach außen**; Alarme erscheinen in `doctor`, Metrik, CLI, MCP-Antwort und AuditChain. Ob eine Verzögerung zulässig ist, entscheidet die Technik nicht (P-11, P-19).

#### 12.19.2 `RecipientLedger` (Art. 17 Abs. 2, Art. 19, P-12)

Je Dokument führt Contextra ein Register der Ausleitungen (Export, Egress-Kanal `[R]`, Checkpoint-Export) **ohne Inhalte**: Empfängerkennung, Zeitpunkt, Datenklasse. Bei einer Löschung liefert `doctor` die Empfängerliste. Die Mitteilung an Empfänger erfolgt durch den Verantwortlichen. Ausleitungen außerhalb von Contextra sind Ausschluss (§12.20).

### 12.20 Ausschlüsse (verpflichtend im Bericht) und Nicht-Gegenstand

**Ausschlüsse** (`excluded`, nie leer; der Text darf bei Weitergabe nicht gekürzt werden):

- `ExternalBackups`: Sicherungen außerhalb der Spur D/K (A-EXTBACKUP);
- `ProcessMemoryResidue`: Prozessspeicher, Swap, Dumps, Hibernation vor der Vernichtung (A-RAM);
- `HistoricalMediaStates`: hardwareinterne Reste (Wear-Leveling, Controller-Cache); Gegenmaßnahme Verschlüsselung plus Epoch Seal, Restannahme A-FLASH bei K1;
- `ModelParameters`, `ExfiltratedData`: Trainings- und Fine-Tuning-Daten externer Modelle, Kopien beim Auftragsverarbeiter, Modellanbieter-Logs, Prompt-Caches;
- `HostHistories`: Host-Verläufe (A-HOSTHIST);
- `DerivedOutsideLineage`: Ableitungen außerhalb der Lineage (vom Betreiber angelegte Exporte);
- `FilesystemSnapshots`: Dateisystem-Snapshots und Journale;
- `ActiveSnapshot{ids}` (§12.5.5).

(Der Ausschluss `KeyMaterialBackups` der Fassung 2.2.1 entfällt: Schlüssel-Backups sind Spur K und erfasst; Schlüsselkopien **außerhalb** von Spur K stehen unter `ExternalBackups`.)

**Nicht Gegenstand:** maschinelles Verlernen in Modellgewichten; Löschung beim LLM-Anbieter; Löschung in Backup-Medien außerhalb der Spuren D/K; rechtliche Wertung der Löschwirkung.

---

## 13. Schnittstellen, Laufzeit und Plattform

Dieser Abschnitt legt die Außenflächen fest, über die Contextra betrieben wird: die Rust-API, den MCP-Server, die CLI und den lokalen IPC-Proxy. Er ergänzt die Verdrahtungs- und Persistenzregeln aus §5, §7 und §12.

### 13.1 Zugänge und Betriebsmodell

- **Rust-API:** `contextra` (Fassade, Composition Root) und `contextra-db` (Collections, Transaktionen). Öffentliche Funktionen: `Contextra::open`, `db()`, `remember`, `recall`, `forget`, `report(id)`, `restore_checkpoint`, `pin_checkpoint`, `as_of`, `shutdown`.
- **MCP-Server (`contextra-mcp`):** Transport ist ausschließlich **stdio, JSON-RPC 2.0**. `stdout` gehört dem Protokoll (INV-STDOUT-1); Logs gehen nach `stderr` oder in eine Datei. Es gibt keinen TCP- oder UDP-Listener (INV-IPC-1).
- **Primärprozess und Proxy (INV-IPC-1):** Genau ein Primärprozess hält das Datenverzeichnis und die Engine. Jeder weitere MCP-Host-Start erkennt den Primär über Lock- und Endpoint-Datei und arbeitet als **stdio↔IPC-Proxy**. IPC ist ein Unix-Domain-Socket (Verzeichnis `0700`, Socket `0600`) bzw. eine Named Pipe mit ACL. Peer-Credentials (`SO_PEERCRED`, `getpeereid`, `GetNamedPipeClientProcessId`) werden geprüft. Der Prinzipal besteht aus OS-Nutzer und Client-Name und wird über die Konfiguration einem Tenant zugeordnet. Der Primärprozess endet nach `idle_timeout` (Startwert 15 min) ohne Clients; fällt er aus, übernimmt der nächste Client nach Recovery. Alle Clients teilen dieselben Garantien.
- **CLI:** `init`, `doctor`, `status`, `confirm <token>`, `migrate`. Die Befehle `verify-proof` (offline-Prüfung) und ein MCP-Statuswerkzeug sind in v1.0 nicht enthalten (O-Punkt, §25).

### 13.2 Werkzeugoberfläche (MCP)

In v1.0 stehen genau **drei Agentenwerkzeuge** bereit: `memory_remember`, `memory_recall`, `memory_forget`. Defaults für Tenant, Scope und Budget kommen aus der Konfiguration, nie aus dem Aufruf (INV-TENANT-1). Jedes Werkzeug trägt MCP-Annotationen (`readOnlyHint`, `destructiveHint`), die aus seiner Kategorie erzeugt werden: `memory_recall` ist `Read`, `memory_remember` ist `Write`, `memory_forget` ist `Destructive`.

Der Vollwerkzeugsatz `contextra_*` der Vorgängerfassung (Kategorien `Read`, `Write`, `Destructive`, `Admin`, `Egress`, `Plugin`) entfällt für v1.0 (E36, §0.5). Administrative Funktionen laufen über CLI und Fassade. Änderungen an dieser Festlegung sind ADR-pflichtig (§18.6).

**Sicherheitspipeline (fünf Stufen je Aufruf, REQ-MCP-004):**
1. Schema- und Größenvalidierung (RPC-Größe, Argumentgrenzen; fuzz-geprüft, REQ-MCP-008).
2. Authentifizierung und Prinzipal (§13.1).
3. Autorisierung gegen Kategorie, Scope und `RevocationLog` (fail-closed).
4. Budget-, Rate- und Ausleitungsprüfung (`max_in_flight_per_principal`, `max_response_tokens`).
5. Audit-Eintrag vor und nach der Ausführung (ohne Nutzinhalte, INV-LOG-1).

### 13.3 Forget-Guard (INV-FORGET-GUARD-1)

`memory_forget` mit **einer** ID und `dry_run` als Standard ist ohne weitere Bestätigung zulässig. Folgende Fälle sind bestätigungspflichtig:

- Selektor-Löschungen,
- Löschungen auf Collection- oder Tenant-Ebene,
- Löschungen von mehr als `forget_confirm_threshold` Dokumenten (Startwert 10).

Das Werkzeug antwortet dann mit `ConfirmationRequired { token, preview }`. Der Token ist **nur über einen Kanal außerhalb des Tool-Aufrufs** einlösbar, z. B. `contextra confirm <token>` oder ein Dialog des Betriebssystems. Das LLM selbst kann den Token nicht einlösen. Damit ist eine durch Prompt-Injection ausgelöste Massenlöschung ausgeschlossen (REQ-MCP-005).

### 13.4 Fassade und Composition Root

Die Fassade `contextra` ist der **einzige** Ort, an dem konkrete Implementierungen gewählt und verdrahtet werden (P29, REQ-FAC-001). `mcp` und `py` rufen nur ihre API auf. Die Fassade:

1. liest Konfiguration, Tier und Profil (§14, §13.7);
2. instanziiert die Ports in Ringreihenfolge: `Vfs`-Implementierung aus `store`, dann `KmsProvider` aus `crypto`, dann `store`, dann die Indexe `vector` und `text`, dann `engine`, dann `checkpoint` und `db`. Rand-Module werden nur bei aktiviertem Feature eingebunden (§4.7, Anhang A);
3. registriert Observer (WAL → Indexe, Audit, CDC) und injiziert Embedder und Reranker als Ports (§5.2);
4. führt beim Start die Phasen B0–B9 aus (§13.5), einschließlich eines **Verdrahtungs-Selbsttests**: jeder Sicherheitsport hat eine reale Implementierung (INV-WIRING-2, REQ-FAC-002);
5. stellt die öffentliche Rust-API bereit (§13.1).

Ohne Löschpfad-Orchestrator liefert dessen Konstruktor `Err(CapabilityUnsupported)` und nie einen Stub, der Erfolg meldet (REQ-DEL-027, §12.13).

### 13.5 Startsequenz B0–B9 (INV-BOOT-1, REQ-FAC-005)

Der Start folgt der festen Phasenfolge. Jede Phase kann den Start mit benanntem Grund verweigern. Teilstarts sind unzulässig.

| Phase | Schritt | Verweigert den Start bei |
|---|---|---|
| B0 | Konfiguration, Tier und Profil prüfen (§14). Insbesondere gilt `key_backup_max_age ≤ erased_set_retention` (§12.9); im Regulated-Tier sind `erasure_grace`, `seal_after_deletion_max`, `erased_set_retention` und ein `ErasedSetMirror`-Ziel Pflichtwerte. | ungültiger Konfiguration, Tier-Verletzung |
| B1 | Datenverzeichnis: Dateisystemtyp (INV-PLATFORM-1), Verzeichnissperre (INV-IPC-1) | nicht unterstütztem Dateisystem (NFS/SMB, unbekanntes FUSE, Cloud-Sync-Ordner), belegter Sperre |
| B2 | Schlüssel: `KmsProvider` und `K_eng` | fehlendem Keystore oder nicht entschlüsselbarem Keystore |
| B3 | Sidecar-Logs validieren (Magic, Kette, Manifest-Marke, §7.9) **und** Schichtbelegung prüfen: `Graph` und `KvCache` sind nur dann `FeatureDisabled`, wenn keine Daten dieses Typs vorhanden sind (§12.4) | gebrochener Kette, fehlender Sidecar-Datei bei vorhandenen Daten, Daten eines deaktivierten Schichttyps |
| B4 | WAL- und Manifest-Recovery (§7.3.1, Schritte 1–5) | Lücke in der WAL-Kette (INV-FAILCLOSED-1) |
| B5 | Ausstehende `SidecarIntent`s idempotent anwenden (§7.3.1, Schritt 6) | dauerhaft scheiterndem Intent |
| B6 | `ErasedSet` laden und Lesesperre armieren (INV-BOOT-1, REQ-RST-1) | unlesbarem oder manipuliertem `ErasedSet` |
| B7 | Löschsagas aus dem `ErasureJournal` fortsetzen oder als `Partial` melden (REQ-FGT-2); unterbrochene Destroy-Protokolle (D1–D6) und Epoch Seals fortsetzen; **Wiederauferstehungsprüfung** (INV-NO-RESURRECTION-1); Retention-Entscheidungen in `PendingDeletion` überführen (§11.6) | gebrochener Journal-Kette |
| B8 | Verdrahtungs-Selbsttest der Fassade (§13.4, Punkt 4) | fehlender Sicherheits-Port-Implementierung (INV-WIRING-2) |
| B9 | Verkehr zulassen | — |

**Regeln:** (1) Lesepfade sind erst nach B6 zugelassen; B9 setzt B6 voraus. (2) Die Reihenfolge wird durch ein Gate geprüft (`boot-order`, §17.3). (3) Die Zuordnung der Schichtbelegung zu B3 ist eine Festlegung dieser Fassung; die Vorgängerfassung hatte sie in B3 nicht vorgesehen.

### 13.6 Fehlermodell (REQ-MCP-010, A6)

Alle Crates münden in `ContextraError`. FFI und MCP geben stabile Fehlercodes aus. Jeder Fehler ist **retrybar** oder **final**.

| Klasse | Codes |
|---|---|
| retrybar | `SerializationFailure` (SSI-Konflikt), `Busy` (Schreibstau, `max_in_flight_per_principal`), `StorageFull` (ENOSPC mit Notreserve) |
| final | `LimitExceeded`, `QuotaExceeded`, `Denied`, `Corrupt`, `ConfirmationRequired`, `CapabilityUnsupported`, `EmbedderMismatch`, `RestoreBeyondHorizon`, `SnapshotExpired`, `TenantErased`, `ResidueRemaining` (nur als Sagazustand), `Poisoned` (nur Neustart hilft) |

Ein Fehler darf keine Nutzinhalte enthalten (INV-LOG-1).

### 13.7 Plattform, Garantiebereich und Performance-Profile (INV-PLATFORM-1)

**Garantieplattformen.** Durabilitäts- und Löschgarantien gelten nur auf den Plattformen der folgenden Matrix mit Tier 1 und Tier 2.

| Plattform | Tier | Besonderheiten |
|---|---|---|
| Linux | 1 | Referenz; `fdatasync`; `mlock` wirksam |
| macOS | 2 | `F_FULLFSYNC` Pflicht; `mlock` begrenzt; APFS-Snapshots und Time Machine sind externe Kopien (deklariert) |
| Windows | offen (O-18) | offene oder gemappte Dateien sind nicht löschbar (Handles freigeben, `FILE_SHARE_DELETE`); ACL über `sys`; Virenscanner-Ausschluss für das Datenverzeichnis wird empfohlen (`doctor`-Hinweis) |

**Dateisysteme.** Unterstützt sind lokale ext4, xfs, btrfs, zfs, APFS und NTFS. Abgelehnt im persistenten Betrieb sind Netzwerk-Dateisysteme (NFS, SMB), unbekannte FUSE-Mounts und Cloud-Sync-Ordner (z. B. OneDrive, Dropbox, iCloud Drive). `doctor` erkennt sie, und der Start verweigert sie (B1, §7.12). Jede Garantieplattform führt CI-Läufe für die Durability-Primitive, die Stufen S2 und S3 sowie `mmap` aus. Auf anderen Plattformen ist der Betrieb Best Effort: `doctor` weist `BestEffort` aus, und Berichte tragen `platform_guarantee = false` (REQ-PLT-001). Die Aufnahme von Windows als Garantieplattform für den Server und den MCP-Betrieb ist in v1.0 ausgeschlossen (§0.5). Ob Windows als Bibliothek Best Effort unterstützt wird, ist offen (O-Punkt, §25).

**Durability-Primitive.** Der Commit-Punkt nutzt die plattformgerechte Primitive (§7.7). Löschlogik stützt sich nie auf die Unix-Unlink-Semantik offener Dateien (REQ-PLT-003).

**PerformanceProfile.** Die Profile legen Gruppierung und Prefetch fest, nie die Korrektheitsinvarianten:

- `Compliance`: maximale Durabilität (Flush je Commit oder kleine Gruppen), Audit synchron.
- `Balanced`: Group Commit, Standard-Audit.
- `BareMetal`: größere Gruppen, aggressives Prefetch.

**DurabilityMode.** Es gibt genau zwei Modi: `Full` (WAL, HMAC-Kette und AEAD; Standard und einziger Modus für `Compliance`) und `MemoryOnly` (keine Persistenz, keine Verzeichnissperre). Einen Modus ohne Integritätskette gibt es nicht (`WalNoHmac` entfällt, REQ-DUR-009). In `Full` sind DirLock und Recovery-Prüfungen nicht abschaltbar.

### 13.8 Checkpoints, Backup und Restore

- **Erzeugung:** Ein Checkpoint ist ein konsistenter Snapshot an einem Sequenzstand. Seine Erzeugungszeit steht im signierten Manifest (`Clock`-Port). Checkpoint-Erzeugung und Export prüfen den `ErasedSet` vor dem Schreiben (REQ-RST-4).
- **Export:** verschlüsselt und **ohne Keystore und ohne KeyTable** (Spur D). Schlüssel werden nur über den eigenen Befehl `export-keys` als Spur K mit signiertem `valid_until` exportiert (§12.17).
- **Restore:** `restore_checkpoint` verifiziert Kette, Manifest und Format, sperrt vor dem ersten Lesezugriff, liest den `ErasedSet` des **Zielsystems** und wendet jede bekannte Löschung erneut an. Jede Wiederanwendung erzeugt im Journal ein Ereignis `restore_reapplied` (REQ-DEL-013, REQ-RST-2).
- **Restore-Horizont (INV-RESTORE-HORIZON-1):** Schlüssel-Backups (Spur K), deren `valid_until` überschritten oder deren Alter nicht bestimmbar ist, werden mit `RestoreBeyondHorizon` abgewiesen (REQ-RST-3, §12.9, §12.17). Daten-Backups (Spur D) enthalten keine Schlüssel und unterliegen keinem Horizont.
- **Checkpoint-Policy:** je Collection `Rewrite` (Standard), `ExpireNow` oder `ExpireAfterHold` (nur unter `Restricted`/Legal Hold, §12.5.4). Interne Checkpoints sind Spur D und enthalten keine KeyTable. Ein gepinnter Checkpoint (`pin_checkpoint`) zählt zur Schicht `Checkpoints`.

### 13.9 Embedded-API und Collections

`contextra-db` stellt Collection-Handles und Transaktionen bereit. Die Logik liegt in `engine`; `db` enthält keine eigene Geschäftslogik (§4.3). Jede Collection persistiert `embedding_model_hash` und Dimension (INV-EMBED-2). Die Collection ist die Produkt-Einheit „Akte“ (Annahme O-14, §25). `drop_collection` ist eine Löschung im Sinne von §12.8 und läuft durch den Forget-Guard (§13.3).

---

## 14. Konfiguration und Deployment-Tiers

### 14.1 Deployment-Tiers (REQ-PLT-002)

| Tier | Zweck | Kennzeichen |
|---|---|---|
| `EdgeMinimal` | Embedded, Edge | kleine Caches, SQ8, minimale Hintergrundarbeit; Verschlüsselung nur in Entwicklung (K0) |
| `PowerUserLocal` | Einzelnutzer-Workstation | volle Funktionen, lokale Inferenz (Rand) |
| `EnterpriseShared` | mehrere Mandanten, ein Knoten | strikte Tenant-Isolation, Quoten (REQ-TEN-005) |
| `EnterpriseRegulated` | reguliert (Recht, Medizin, Finanz) | Verschlüsselung Pflicht, vollständiger Audit, Human-Approval für Rand-Module |

`PowerUserLocal` stammt aus der Vorgängerfassung; 2.2 nennt ihn nicht. Die Aufnahme ist als O-Punkt geführt (§25). Jedes Tier setzt die Pflichtwerte aus §15 durch; ein Verstoß ist ein Startfehler in B0.

### 14.2 Konfigurationsregeln

1. Es gibt keinen Konfigurationswert ohne explizite Quelle. Fehlende Werte sind Startfehler oder Fail-closed, nie ein stiller Default (A5).
2. `key_backup_max_age ≤ erased_set_retention` (INV-RESTORE-HORIZON-1).
3. `erased_set_retention` ist in `EnterpriseRegulated` Pflichtwert ohne Standard; der Standard ist nie unbegrenzt.
4. Quoten `tenant_quota_bytes` und `tenant_quota_docs` sind in `EnterpriseShared` und `EnterpriseRegulated` Pflicht.
5. Alle Konstanten mit „Startwert“ stehen in §15 und sind per Benchmark kalibrierbar (§16). Konstanten mit „fest“ sind nicht konfigurierbar.

---

## 15. Startwerte und Kalibrierung

Werte mit **„fest“** sind unveränderlich. Werte mit **„Startwert“** sind zu kalibrieren; die Kalibrierung läuft über die Benchmarks aus §16 und verändert den Code nur über ein Gate. Jede Konstante hat genau eine Quelle im Code oder in der Konfiguration (INV-DOC-SINGLE-SOURCE-1).

### 15.1 Formate und Kryptografie (fest)

| Parameter | Wert |
|---|---|
| Magics | WAL `MFW3`, SSTable `MFSX` (Legacy `MFST` entscheidet WP 2.4), Blob `MFBL`, Manifest `MFMN`, Keystore `MFKS`, Sidecars `MFKR`, `MFRV`, `MFAU`, `MFEJ`, `MFES`; jede Datei trägt zusätzlich eine Formatversion (INV-FORMAT-1) |
| WAL-Integrität | HMAC-SHA256-Kette (Vorgänger-Tag im Record) und AEAD |
| AEAD | AES-256-GCM-SIV; Nonce 96 Bit zufällig; Tag 128 Bit (28 Byte Overhead je Record); AD = `scope_id ‖ key_id ‖ doc_id_hash` |
| Schlüssel | Wurzel (`EffaceableKeySlot`, K1: Argon2id); `KEK[coll,epoche]`, `DocDek`, `ArtKey`, `K_ref` je 256 Bit zufällig, gewrappt; `K_eng` per HKDF(Label) (§12.2.2, §12.3) |
| KDF-Untergrenze | Argon2id m ≥ 64 MiB, t ≥ 3, Kalibrierung auf ≥ 250 ms (INV-KDF-1) |
| Signatur, Hash | Ed25519; SHA-256 (Merkle, Digest); BLAKE3 (Blob-Digest, Manifest) |
| Löschbericht | `DeletionProof` Formatversion 5 (Magic `MFDR`); V4 bleibt unvergeben (E46) |
| Unsafe-Inseln | genau drei: `simd`, `sys`, `wire` (§4.6) |

### 15.2 Storage

| Parameter | Startwert |
|---|---|
| `blob_threshold` | 64 KiB |
| `l0_slowdown` / `l0_stop` | 8 / 20 L0-Dateien |
| `write_stall_timeout` | 5 s, danach retrybar `Busy` |
| Block-Cache | SIEVE hinter Port `BlockCache` |
| `emergency_reserve_bytes` | 64 MiB (mindestens zwei WAL-Segmente plus Journal; bei Start vorab reserviert, §7.10) |
| Recovery-Zeitbudget | je Tier in `benchmarks/budgets.toml` (§16) |

### 15.3 Index und Retrieval

| Parameter | Startwert |
|---|---|
| HNSW `m` / `efConstruction` / `efSearch` | 16 / 200 / 64 |
| `MAX_SEARCH_K` | 1000 (auch im Filterpfad) |
| `exact_scan_max` | 20 000 erlaubte IDs |
| `tombstone_rebuild_ratio` | 0,2 |
| Overfetch | `3·k → 6·k → 12·k`, harte Scan-Obergrenze |
| BM25 `k1` / `b` | 1,5 / 0,75 |
| RRF `k` | 60 (Tie-Breaker: Score absteigend, DocId aufsteigend) |
| Designpunkt | ≤ 10⁷ Vektoren und ≤ 10² Mandanten je Knoten (fest, §2.1) |

### 15.4 Chunking, Limits, Betrieb

| Parameter | Startwert |
|---|---|
| Chunk-Ziel / Obergrenze | 512 / 1024 Token; Overlap 10–15 % |
| `max_document_bytes` | 16 MiB |
| `max_chunks_per_document` | 4 096 |
| `max_batch_ops` | 10 000 |
| `max_collections_per_tenant` | 256 |
| `max_in_flight_per_principal` | 8 |
| `max_response_tokens` | 4 000 |
| `forget_confirm_threshold` | 10 Dokumente |
| `idle_timeout` (IPC) | 15 min |
| Retention-Karenzzeit / Reaper-Batch | 7 Tage / 100 Dokumente je Tick |
| `snapshot_max_age` | 15 min (REQ-DEL-031) |

### 15.5 Löschpfad

| Parameter | Startwert | Herleitung |
|---|---|---|
| `erased_set_retention` | 90 Tage (≥ `key_backup_max_age` + Zuschlag; Pflichtwert ohne Standard in `EnterpriseRegulated`) | Rechtsgrundlage und Frist offen (P-07, P-16); Untergrenze `key_backup_max_age` (§12.10) |
| `key_backup_max_age` | 30 Tage (≤ `erased_set_retention`) | Lebensdauer eines Schlüssel-Backups (Spur K, §12.17); Frist offen (P-06) |
| `kek_epoch_len` | 30 Tage | Epoch Seal zeitgesteuert (§12.2.4) |
| `seal_after_deletion_max` | 7 Tage (Regulated: ≤ 72 h) | Epoch Seal nach Löschung (H-07) |
| `erasure_grace` | 0 (Obergrenze 72 h; Regulated: Pflichtwert ohne Standard) | Karenz zwischen Klasse P und Schlüsselvernichtung (§12.8.3) |
| `purge_rebuild_max_days` | 14 Tage | Hygiene und Graph-Neuaufbau (§12.8.2, §12.16) |
| `min_record_bytes` | 256 | Padmé-Padding (§12.2.6) |
| `ref_epoch_len` | 90 Tage | Länge einer Referenz-Epoche (§12.10) |
| `checkpoint_policy` | `Rewrite` | Standard (§12.5.4; O-15) |
| `checkpoint_hold` | 25 Tage | nur für `ExpireAfterHold` unter `Restricted`; interne Zielgröße unter der Monatsfrist (P-11, P-15) |
| `partial_alarm_after` | 24 Stunden | Interne Zielgröße (P-11, P-19) |
| `partial_escalate_after` | 14 Tage | dito |
| `partial_deadline_report` | 25 Tage | dito; Monatsfrist Art. 12 Abs. 3 DSGVO bleibt die Obergrenze |
| Retry-Backoff der Saga | exponentiell, Start 1 s, Deckel 5 min, Zeit über `Clock` | deterministisch testbar mit `ManualClock` |
| Canary | mindestens 32 Zufallsbyte als Text | Eindeutigkeit gegen Zufallstreffer (AC-1) |
| `K_ref`-Lebensdauer | Epochenende + `erased_set_retention` | §12.10 |

**KEK-Epochen** (§12.2.4) und **Referenz-Epochen** (§12.10) sind in 2.3 Bestandteil dieser Tabelle; die Lösch-Epochen der Fassung 1.3 (Rotation je Dokument mit Batch-Fenster, `N_epoch`, `L_wrap`) bleiben gestrichen, weil der Epoch Seal sie ersetzt.

### 15.6 Kognition und Rand

Die Parameter der Vorgängerfassung für Decay, Working-TTL, `VolatileContextVault`, `ContextPrefixEngine`, `GaspValidator` und `MultiStepEngine` gehören zu archivierten oder randständigen Crates (Anhang B). Sie sind nicht normativ, solange das Feature nicht aktiviert und ein ADR nicht erteilt ist.

---

## 16. Messbare Verträge und Benchmarks

Jeder Vertrag ist ein CI-prüfbares Kriterium. Absolute Zahlen je Tier stehen in **einer** Datei, `benchmarks/budgets.toml` (INV-DOC-SINGLE-SOURCE-1). Veröffentlichungen nennen Commit, Modell-Hash, Hardware, Workload, Vergleichsbasis und 95-%-Konfidenzintervall (EP4).

| Vertrag | Kriterium |
|---|---|
| **Durabilität** | 0 verlorene bestätigte Commits über alle injizierten Crash-Punkte (§17.4), je Plattform der Matrix §13.7 |
| **Konsistenz** | 0 Divergenz zwischen Primärdaten und Indexen nach Recovery (INV-ATOMIC-IDX-1); Index-Rebuild aus dem WAL ist bit-identisch (INV-INDEX-APPLY-1) |
| **Isolation** | 0 Cross-Tenant-Leaks in Property- und Fuzz-Tests (Daten, Scores, Statistiken, Timing-Klassen); Latenz eines Mandanten hängt nicht von der Last anderer ab (§9.2) |
| **Time-to-Context (TTC)** | p50, p99, p99.9 mit 95-%-Konfidenzintervall je Tier und Last (kalt und warm; gefiltert und ungefiltert); Regression über Schwelle bricht die CI |
| **ANN-Recall** | Recall@10 ≥ 0,95 bei Standard-`efSearch` (Startwert); gefilterte Suche liefert exakt `k` oder alle passenden Treffer (§9.8) |
| **Retrieval-Qualität** | Recall@5 auf einem öffentlichen Gedächtnis-Benchmark mit reproduzierbarem Harness; Abweichungen werden per Ursachenanalyse erklärt, nicht umgangen |
| **Recovery-Zeit** | Zeit bis „Verkehr zugelassen“ (B9) ≤ Budget je Tier bei definierter WAL-Länge; Messgröße ist die Checkpoint-Frequenz (§7.3.1) |
| **Löschung** | Jeder `forget` liefert sofort einen ehrlichen Teilbericht und nach Abschluss der Purge-Schritte den vollständigen Bericht oder `Partial` mit ausstehender Schicht, Grund und Ablaufdatum. Nach einem Crash an jedem Punkt der Saga bleibt der Scope gesperrt; die Saga setzt idempotent fort (REQ-FGT-2). Der Bericht behauptet nur gemessene Schichten und die gemessene Klasse (§12.5, §12.11); Fast-Path bis Klasse C\* in Minuten bis Stunden, Hygiene (M) ≤ 14 Tage (§12.19.1) |
| **Löschfrist** | Zeit bis `Completed` oder bis zum dokumentierten `Partial` wird gemessen; jede Überschreitung der drei Schwellen erzeugt genau einen Alarm, eine Eskalation bzw. eine Meldung (AC-20); kein `Partial` ohne Grund und Frist (REQ-DEL-036) |
| **Write-Pfad** | Durchsatz und Latenz je `PerformanceProfile` inkl. Group-Commit-Kurve; Write- und Space-Amplification der Kompaktion innerhalb des Budgets; kein unbegrenzter Speicherzuwachs unter Dauerlast (Backpressure, §7.4) |
| **Speicher** | RAM-Bedarf folgt der Formel aus §9.6 ± Toleranz; unter Speicherdruck wird verkleinert oder gedrosselt, nie OOM-Kill (§7.6) |
| **Vergleich** | Jede Leistungsaussage hat mindestens eine fremde Baseline unter identischem Workload; ohne Baseline gilt die Zahl als nicht belegt (§16.3) |

### 16.1 Messmethodik

Festgelegte Hardware-Klasse, fixierte CPU-Frequenz und Affinität, Warm-up, mindestens 30 Wiederholungen, 95-%-Konfidenzintervall; kalter und warmer Cache werden getrennt gemessen. Schwellen für Regressionen: p50 > 5 %, p99 > 10 % gegenüber der versionierten Baseline (Startwerte, kalibrierbar).

### 16.2 Pflicht-Benchmarks

TTC end-to-end je Tier; ANN-Recall und Latenz (ungefiltert, gefiltert, Mandantenmix); BM25 und BM25F; RRF-Fusion; Schreibpfad inkl. Group-Commit-Kurve; Recovery-Zeit; Kompaktion (Write- und Space-Amplification); Löschkette (Dauer der Stufen S1 bis S3); Block-Cache; SIMD-Kernel gegen die skalare Referenz (E33); Speicherbedarf.

### 16.3 Fremde Baselines (Pflicht für Vergleichsaussagen)

KV/LSM gegen RocksDB und redb; ANN gegen hnswlib oder faiss und usearch; Volltext gegen eine etablierte Implementierung (z. B. SQLite-FTS5 oder Tantivy); kombiniert gegen eingebettete Vektor-Datenbanken der Zielgruppe (z. B. sqlite-vec, LanceDB). Ein Benchmark, der nur Contextra misst, ist Regressions-, kein Vergleichsbenchmark (REQ-QA-007).

### 16.4 Vorbehalt

Die Zahlenwerte für Recovery-Zeitbudgets und Vertragsschwellen liegen in `benchmarks/budgets.toml` und sind in dieser Fassung **nicht** festgelegt. Sie sind nicht gemessen und nicht belegt, bis ein Messlauf nach §16.1 sie bestätigt (E0 bis E2, §1.4).

---

## 17. Qualitätssicherung, Tests und Gates

### 17.1 Prüfmittel

Unit-Tests, Property-Tests (proptest), modellbasierte Tests (Referenzmodell gegen Engine), Integrationstests über die Fassade (P29), Fuzz, loom, miri, Mutationstests, Chaos (§17.4) und Benchmarks (§16).

- **Fuzz-Matrix:** Jede Eintrittsstelle für nicht vertrauenswürdige oder persistierte Bytes hat mindestens ein Fuzz-Target. Ein Gate vergleicht die Matrix mit den Eintrittsstellen im Code (REQ-QA-003). Eintrittsstellen: WAL, SSTable und Blob, Manifest und Keystore, `AuditChain`-, `RevocationLog`- und Beweisdekoder, Wire, MCP und IPC, Text (Analyzer, Chunker), Extraktoren (`contextra-ingest`, §4.9). „SKIPPED“ gilt als rot (REQ-QA-004).
- **loom:** Modelle für jede nebenläufige Kernstruktur (Inventar): SnapshotRegistry, TxBuffer, ReadSet und SSI-Validator, Group-Commit, Applier-Übergabe, `RevocationLog`, `KeyRegistry`, Key-Sperren, HNSW-Arena, Block-Cache. Eine neue nebenläufige Struktur ohne loom-Modell bricht das Gate (REQ-MVCC-005).
- **miri:** läuft für alle Unsafe-Inseln gegen die sichere Referenz und für jeden Test, der Zeiger- oder Layout-Logik berührt.
- **Mutation:** überlebende Mutanten in `[K]`-Crates sind Defekte oder begründete Ausnahmen; der Score je Crate steht in `governance/ratchet.toml` und darf nur steigen (REQ-QA-005).
- **Testdichte** ist risikoproportional: Unsafe-Inseln, `store`, `crypto` und der Commit-Pfad von `engine` liegen über dem Workspace-Median.

### 17.2 Red→Green-Disziplin

Jede Änderung beginnt mit einem fehlschlagenden Test, der die Anforderung oder Invariante ausdrückt. Danach folgt die minimale Implementierung und das Refactoring unter grünem Baum (EP5, §18.1). Ein Test, der erst nach der Implementierung entstanden ist, belegt eine Invariante nur, wenn Mutation seine Wirksamkeit zeigt (EP2).

### 17.3 Gate-Katalog

Jedes Gate ist eine Zeile in `governance/gates.toml` mit Name, Phase, Pflichtstatus (lokal oder CI), Risikoklasse, Besitzer und Fixture. Ein Gate, das nicht ausgeführt wurde, ist rot (INV-EVIDENCE-1). Ring-, Kanten- und Feature-Prüfungen laufen über `cargo metadata` mit aufgelösten Features (Regel „Compiler vor Heuristik“, EP11). Öffentliche API prüft `cargo public-api`.

Die folgende Tabelle führt die Gates, die diese Spezifikation nennt. Der jeweilige Status wird dynamisch in den generierten Berichten erfasst (INV-DOC-SINGLE-SOURCE-1, siehe `cargo xtask verify`).

| Gate (Spezifikation) | Prüft | Repo-Stand |
|---|---|---|
| `ring-layering` / `ring-dag` | Kantenregeln, DAG, `may_depend_on` (INV-CORE-ISOLATION-1, REQ-ARCH-001/002/018) | vorhanden als `ring-dag` |
| `capabilities-consistency` | Ring, Klasse, Reifegrad aus `capabilities.toml` (REQ-ARCH-011/023) | vorhanden |
| `unsafe-inventory` | `unsafe` nur in drei Inseln, getrennt nach generiert und handgeschrieben (REQ-ARCH-004/005) | vorhanden |
| `feature-matrix` | Feature wirkt und ist getestet (REQ-ARCH-012/020) | vorhanden |
| `public-api` | Oberfläche, Schlüssel nicht erreichbar (REQ-ARCH-014, INV-SCAN-1) | vorhanden |
| `wired` | Verdrahtung und Selbsttest (P29, REQ-ARCH-010) | vorhanden |
| `security-ports` | reale Implementierung je Sicherheitsport (INV-WIRING-2) | Gate im Katalog vorhanden (`governance/security_ports.toml` vorhanden) |
| `key-purposes` | Zweckregister gegen Code (INV-KEYPURPOSE-1) | Gate im Katalog vorhanden (`governance/key_purposes.toml` vorhanden) |
| `witness-mint` | `LayerWitness` nur in Scannern (INV-WITNESS-MINT-1) | vorhanden |
| `deletion-types-once` | Löschtypen genau einmal (REQ-DEL-027) | vorhanden |
| `deletion-honesty` | Außentexte gegen K1-Claims (INV-CLAIMS-1, REQ-DEL-035) | vorhanden |
| `surface-coverage` | jede persistente Dateiklasse einer Löschschicht zugeordnet (INV-SURFACE-1, §12.4) | im Gate-Katalog vorhanden |
| `claim-generator` | Außentexte nur aus Satzbibliothek, jeder Satz an Messwert gebunden (INV-CLAIM-BIND-1, AC-24) | im Gate-Katalog vorhanden |
| `doc-truth` | Konstanten, Ringe, IDs gegen Code (INV-DOC-SINGLE-SOURCE-1) | vorhanden |
| `symbol-uniqueness` | doppelt vergebene IDs und Symbole | vorhanden |
| `determinism` / `ratchet` | P28, Ratchet-Baselines, Verbotsliste | vorhanden |
| `format-registry` | jede Datei im Datenverzeichnis hat einen Registereintrag (§7.11) | Entsprechung `format-corpus`; Registerprüfung nicht geprüft |
| `no-security-stub` | keine Sicherheitsstubs (A1) | im Gate-Katalog vorhanden |
| `inv-coverage` | jede INV mit Test (A8) | im Gate-Katalog vorhanden |
| `error-domain` | alle Fehler münden in `ContextraError` (A6) | im Gate-Katalog vorhanden |
| `doc-stamp` | Dokumentstempel und Single Source (A9) | im Gate-Katalog vorhanden |
| `loc-budget` | Code-Zeilenbudget nach `loc.py` (§1.3, §4.2) | im Gate-Katalog vorhanden |
| `boot-order` | Phasen B0–B9 in Reihenfolge (INV-BOOT-1) | im Gate-Katalog vorhanden |
| `async-io` | kein Blockieren im async-Kontext (REQ-ASYNC-2) | im Gate-Katalog vorhanden |
| `unbounded-spawn` | jede `spawn`-Stelle begrenzt (REQ-ASYNC-3) | im Gate-Katalog vorhanden |
| `check-max-results-unbound` | `max_results` nie unbegrenzt (§11.4) | xtask-Modul vorhanden |
| `check-recall-stability` | Recall-Stabilität (§11.9) | xtask-Modul vorhanden |
| `check-nan-hot-loop` | NaN-Prüfung im Hot-Loop (§11.9) | xtask-Modul vorhanden |
| `check-flatbuffers-drift` | Wire-Schema gegen generierten Code (§7.11) | xtask-Modul vorhanden |
| `ring0-async` | keine Async-Runtime in Ring 0 (P26, REQ-ASYNC-1) | im Gate-Katalog vorhanden |
| `product-hull`, `module-removal` | Produkt baut ohne Rand-Module (§4.5) | `product-hull` im Gate-Katalog vorhanden |

Weitere Gates des Repos ohne Entsprechung in dieser Tabelle (u. a. `merge-result-build`, `protected-paths`, `gate-weakening`, `test-integrity`, `scope-guard`, `provenance`, `claims-in-pr`, `veto-guard`, `adr-index`, `deny`, `audit`, `sbom`, `fuzz-matrix`, `fuzz-regression`, `miri`, `loom-inventory`, `mutation`, `chaos`, `bench-regression`, `platform-matrix`, `evidence-complete`, `gate-selftests`, `catalog-sync`, `ci-hygiene`) sind mit den Abschnitten §17.1, §18.2 und §18.4 abgedeckt.

**Meta-Gate:** Jedes Audit-Skript hat einen Selbsttest mit synthetischem Repository (REQ-QA-008). Grenzwerte dürfen kein Gate faktisch abschalten. Gate-Läufe sind auf einem unabhängigen Runner reproduzierbar; rohe Protokolle werden archiviert (REQ-QA-009).

**Lint-Satz (Workspace, `deny`):** `unsafe_code`, `unsafe_op_in_unsafe_fn`, `unused_must_use`; Clippy: `undocumented_unsafe_blocks`, `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `indexing_slicing`, `exit`, `dbg_macro`, `await_holding_lock`, `await_holding_refcell_ref`, `cast_possible_truncation`, `cast_sign_loss`; `disallowed_methods`, `disallowed_types`, `disallowed_macros` für die P28-Verbotsliste und die `assert!`-Familie in Produktionscode; `print_stdout` im MCP-Binär (INV-STDOUT-1). Ausnahmen nur als begründetes, lokal begrenztes `#[allow]`; das Gate zählt sie (Ziel 0). Im Release-Profil ist `overflow-checks = true` für Kern-Crates gesetzt (P7).

### 17.4 Chaos und Fehlerinjektion (Jepsen-Stil)

`FaultVfs` (in `testkit`) injiziert seed-reproduzierbar an **jedem** Schreib- und Durability-Punkt: Crash, Torn Write, `fsync`-Fehler (INV-FSYNC-1), ENOSPC (INV-ENOSPC-1), `EMFILE`, Bit-Flips, Umordnung von Schreibvorgängen vor dem Flush, Rechtefehler (REQ-QA-006). Weitere Szenarien: Uhrsprünge über den `Clock`-Port, Absturz des IPC-Primärprozesses mit Übernahme, konkurrierende Clients, Handle-Semantik bei Windows, Abbruch während Restore und Löschkette.

Nach jedem Szenario laufen Recovery und der Invarianten-Checker: keine verlorenen Bestätigungen, Kette gültig, Indexe konsistent, SSI-Historie serialisierbar, nach `forget` ist nichts Gelöschtes lesbar (Scanner je Schicht), und der Sidecar-Zustand ist konsistent. Fehlschläge sind mit Seed exakt replizierbar.

### 17.5 Pflichttests (T-*)

Die Pflichttests der Garantien G1–G6 (§2.4) sind in dieser Fassung benannt. Die Nummern sind neu festgelegt; jeder Test hat einen Mechanismus nach §1.4.

| Test | Garantie | Prüfung | Mechanismus |
|---|---|---|---|
| T-DEL-1 | G1 | nach `Completed` keine Klartextreste in LSM, Vektor-, Textindex und Checkpoints (Canary-Scan) | C + I |
| T-DEL-2 | G2 | Bericht entsteht nur aus gemessenen Restzahlen 0 mit Ergebnis `Erased` | T + P |
| T-DEL-3 | G2 | Signatur verifiziert mit falschem Schlüssel nicht | P |
| T-DEL-4 | G3 | Absturz an jeder Stelle der Saga führt zu Fortsetzung oder `Partial` | C (Crash-Matrix) |
| T-DEL-5 | G3 | kein Bericht ohne Löschung bei Absturz | C |
| T-DEL-7 | G4 | ab `Frozen` liefert kein Lesepfad Daten des Scopes | L + P |
| T-DEL-8 | G4 | Restore stellt gelöschte Daten nicht wieder her | I |
| T-DEL-9 | G4 | Lesesperre gilt für `as_of` und offene Snapshots (INV-ASOF-ERASE-1) | P + L |
| T-TEN-1 | G5 | kein Mandant liest, findet oder löscht Daten eines anderen | P + F |
| T-TEN-2 | G5 | keine Cross-Tenant-Statistik, -Kante oder -Posting (INV-TENANT-IDX-1) | P + G |
| T-REC-2 | G6 | Zustand nach hartem Abbruch aus dem WAL rekonstruierbar | C + P |
| T-SSI-1 | RB4 | SSI aktiv; Read-Sets mit Phantomschutz validiert (INV-SSI-ACTIVE-1, U-06) | P + C + L |
| T-EMB-1 | — | Modellwechsel wird abgewiesen; Migration nur mit Neuaufbau (§11.1) | I + G |
| T-DET-1 | RB1 | HNSW-Aufbau mit festem Seed ist bit-identisch (INV-INDEX-APPLY-1) | P |

Ergänzend gilt: AC-1 bis AC-33 aus §12.12 sind Abnahmekriterien für die Löschung. AC-7, AC-12, AC-14 und AC-15 bleiben unbelegt (Inhalt der Fassung 1.3 nicht rekonstruierbar, §25.3); die Nummern werden nicht neu vergeben. Neue Pflichttests zu G3/G4: T-DEL-10 (Destroy-Protokoll, AC-28), T-DEL-11 (Epoch Seal, AC-21), T-DEL-12 (Backup-Trennung, AC-22), T-DEL-13 (Nadelscan, AC-25), T-DEL-14 (keine Wiederauferstehung, AC-27).

### 17.6 Reifegrad (`maturity`)

`maturity` wird aus der niedrigsten Evidenzstufe der Mechanismen einer Garantie berechnet und nicht behauptet (§1.4). `maturity = stable` setzt E4 voraus (EP10). Ein Crate, dessen Sicherheitsports nur Null- oder Testimplementierungen haben, ist `experimentell` (§4.1). Für Zielbilder gilt: `[K]` ⇒ `stable` erst bei E4, `[R]` und `[E]` ⇒ `experimental`.

### 17.7 Verifikationsbefehl

`cargo xtask verify` führt alle Pflicht-Gates in definierter Reihenfolge aus und erzeugt je Gate eine maschinenlesbare Evidenzdatei mit Befehl, Toolchain-Version, Commit, Exit-Code, Dauer, Anzahl ausgeführter und übersprungener Tests und Hash der Protokolle. Jede Garantie in §2.4 und §2.5 verweist auf diesen Befehl (REQ-QA-001, INV-EVIDENCE-1). Die Zuordnung „Garantie → Test → Gate“ ist Teil des Befehls.

### 17.8 Orakel und deterministische Simulation (INV-ORACLE-1)

Jede Kerngarantie RB1–RB4 (§2.5) hat ein Prüfmaß, das nicht aus denselben Datenstrukturen stammt und nicht vom selben Erzeuger wie der geprüfte Pfad (EP15):

| Garantie | Orakel |
|---|---|
| RB1 Crash und WAL | Referenzmodell der WAL-Semantik; Crash-Simulation mit `FaultVfs` (seed-reproduzierbar) |
| RB2 Löschung | Canary-Scanner (§17.9) und Referenz-Verifier in anderer Sprache (der Referenz-Verifier entfällt mit REQ-PRF-6; in v1.0 wirkt nur der Canary-Scanner, Lücke, §25) |
| RB3 Audit und Provenienz | unabhängige Referenzimplementierung der HMAC-Kette |
| RB4 Isolation | Brute-Force-Referenz für ANN und ACL; History-Checker für SSI |

Eine Simulation ist deterministisch: gleicher Seed ergibt gleiche Fehlerfolge.

### 17.9 Canary-Läufe

Ein Canary-Lauf legt Daten mit eindeutigen, zufällig erzeugten Markern an (Klartext ≥ 32 Byte, daraus abgeleitete Embedding-Bytes, seltene Terms, Roh-IDs). Er führt die Löschkette aus und durchsucht danach **jede Datei** des Datenverzeichnisses, einschließlich Checkpoints, Temp-Dateien, `.bak`, Keystore und Journal, in Rohsicht und in entschlüsselter Sicht. Jeder Lauf hat eine positive Kontrolle an einem absichtlich nicht gelöschten Datensatz (AC-1). Ein Lauf ohne bestandene Kontrolle ist ungültig. Treffer im Klartext sind Defekte; Chiffrat darf verbleiben. Der Lauf ist reproduzierbar (Befehl, Seed, Commit) und wird zusätzlich mit einem unabhängigen Werkzeug (z. B. `grep -r`) gegengeprüft.

### 17.10 Benchmarks und Trennung

Benchmark-Code liegt im Crate `bench` (§4.2). Er enthält `unsafe`-Code, ist aber von Kernaussagen getrennt. Kernaussagen stützen sich nie auf Benchmarks allein.

---

## 18. Entwicklungsprozess und Governance

### 18.1 Arbeitszyklen

Ein Arbeitszyklus umfasst einen Rot→Grün-Schritt, einen Scope und einen verifizierten Commit (EP5). Autonome Agenten arbeiten nach Aufgaben mit Akzeptanzkriterien. Ein Ergebnis wird nur akzeptiert, wenn alle Gates auf dem **Merge-Ergebnis** grün sind. Teilverdrahtung wird nicht akzeptiert (P29, REQ-ARCH-010). Ein Agentenbericht ist eine Hypothese und kein Beleg (INV-EVIDENCE-1).

### 18.2 Scope-Treue, Merge-Queue und Provenienz (INV-PROVENANCE-1, REQ-GOV-001/002)

Jeder Merge auf `main` trägt eine Aufgaben-ID und einen deklarierten Pfad-Scope. Merges laufen serialisiert über eine Merge-Queue, die das Merge-Ergebnis baut und testet. Das Gate blockiert vor dem Merge:

- Änderungen außerhalb des deklarierten Scopes;
- Commits, deren Diffstat nicht zum Titel passt;
- Änderungen an geschützten Pfaden (`governance/protected-paths.toml`: CI-Konfiguration, Gates, Audits, Kompatibilitätskorpus, Verfassung) ohne ADR-Verweis und Freigabe;
- das Entfernen von Tests, Audits oder Gates (REQ-QA-010);
- eine Merge-Basis, die älter ist als der letzte Merge in denselben Dateien (stille Rücknahme).

Commit-Nachrichten sind beschreibend. `main` kompiliert zu jedem Zeitpunkt.

### 18.3 Release-Kriterien (Version 1.0, REQ-GOV-005)

Ein Release trägt nur Aussagen, die durch die folgenden Kriterien belegt sind. Die Kriterien sind vor der Messung festgelegt:

1. Alle Pflicht-Gates sind auf einem unabhängigen Runner grün (§17.3).
2. Mindestens ein schriftlicher Review von `store` (WAL, Recovery, Kompaktion) und `crypto` (Löschbericht, `KeyRegistry`, `RevocationLog`) durch eine Person außerhalb der Agenten-Pipeline.
3. Messbare Verträge aus §16 sind erfüllt, einschließlich reproduzierbarem Recall@5 und Vergleich mit fremden Baselines.
4. Keine kritischen oder hohen Advisories in erreichbarem Code.
5. Gepflegtes `CHANGELOG.md` (Keep a Changelog); Kompatibilitätskorpus und Migrationstest (§7.11).
6. Juristisch geprüfte Außenformulierung zur Löschung (INV-CLAIMS-1, §12.11) und zu Compliance-Vorlagen.
7. Soak-Test ohne Panik und ohne Korruption über einen vorab festgelegten Zeitraum (Startwert 30 Tage) auf allen Plattformen der Tier-1- und Tier-2-Matrix.
8. Kontrollierter Soft-Launch mit Nutzern außerhalb des Entwicklungsteams.

`[R]`- und `[E]`-Crates sind von Release-Aussagen ausgeschlossen, solange sie nicht ausdrücklich in die Kriterien aufgenommen sind.

### 18.4 Supply-Chain

- Advisory-Ignorierung (`deny.toml`) nur mit Einzelbegründung, Erreichbarkeitsanalyse und Ablaufdatum. Sammel-Ignores sind unzulässig; kritische und hohe Advisories werden nicht ignoriert (REQ-GOV-006).
- Workspace-weite `tokio`-Features `full` sind unzulässig. Feature-Listen gelten je Crate (§4.5).
- Doppelte Versionen derselben Abhängigkeit unterliegen einem Budget (`cargo tree -d`).
- Native Drittabhängigkeiten und ihre `unsafe`-Anteile werden inventarisiert. „Pure Rust“ gilt nur für Pfade ohne solche Abhängigkeiten.
- Persistente Formate hängen nie von nicht mehr gewarteten Abhängigkeiten ab (INV-FORMAT-1).
- Gepinnte Abhängigkeiten, reproduzierbare Builds, SBOM, Lizenzprüfung (`deny`) und `cargo-vet`-Audit-Trail.

### 18.5 Archivierung

Archivierte Crates und Features (Anhang B) werden in den Git-Tag `archiv/v0` verschoben und aus dem Workspace entfernt. Die Vorgängerfassungen werden dort ebenfalls abgelegt. Der Tag ist nicht normativ.

### 18.6 ADR-pflichtige Entscheidungen

Folgende Punkte ändern Garantien oder Formate und dürfen nur per ADR in `docs/decisions/` geändert werden. Ein ADR nennt Begründung, Alternativen und eine ausdrückliche, signierte Freigabe.

- Aufnahme eines Crates oder Features in den Kern (§4.8);
- Auslagerung von `router`, `agent`, `license` oder `audit-export` in eigene Repositories;
- Aktivierung von RaBitQ oder DiskANN (§9.7);
- Änderung der Plattform-Tiers und der Garantieplattform (§13.7, §13.7);
- Änderung eines persistenten Formats (INV-FORMAT-1, §7.11);
- Standardwert von `erasure_required` (P-15) und Standard-`checkpoint_policy` (§12.5.4);
- Dauer von `erased_set_retention`, `key_backup_max_age`, `kek_epoch_len` und `ref_epoch_len` (§15.5);
- Aktivierung von K2/K3 als ausgelieferte Verwahrungsstufe (§12.2.3);
- Änderung eines Vetos (§3.1) oder einer Invariante (§3.4);
- Änderung der MCP-Werkzeugoberfläche (§13.2);
- Änderung einer Festlegung aus §20.

### 18.7 Dokumentationsmodell (INV-DOC-SINGLE-SOURCE-1)

Jede Tatsache hat genau eine kanonische Quelle: Ringe, Klasse, Reifegrad und Dev-Ausnahmen in `capabilities.toml`; Prinzipien und Vetos in der Verfassung (`governance/constitution.toml` und `CONSTITUTION.md`); Konstanten im Code und in §15; Architekturentscheidungen in `docs/decisions/`; die Soll-Architektur in dieser Spezifikation. Status- und Tagesberichte sind generierte Projektionen und nie Architekturquelle. Abgeleitete Texte werden generiert oder per Doc-Truth-Gate geprüft (REQ-ARCH-023).

Der Implementierungsplan ist eine begleitende Arbeitsunterlage. Er darf Status gegen einen benannten Commit führen, aber keine Soll-Aussagen ändern.

---

## 19. Anforderungskatalog

### 19.1 Herkunft und Regeln

Der Katalog enthält jede normative Anforderung, die in dieser Fassung vorkommt, mit stabiler ID, Klasse (`[K]`, `[R]`, `[E]`), Durchsetzungsmechanismus (Kürzel nach §1.4) und Status. Die Anforderungen aus der Vorgängerfassung 1.3 (1.3 §33) sind übernommen, soweit sie zu dieser Fassung passen. Anforderungen, die diese Fassung in den Fachabschnitten neu definiert oder nur verweist, sind ergänzt.

**Status-Spalte:** *übernommen* = Wortlaut aus 1.3 §33 unverändert. *angepasst* = Wortlaut sinngemäß an 2.2 angepasst, Grund angegeben. *gestrichen* = entfällt in v1.0, Grund angegeben. *archiviert* = betrifft Anhang-B-Crates, nicht normativ. *in 2.2 definiert* = Wortlaut steht in einem Fachabschnitt dieser Fassung. *offen* = nur verwiesen, Wortlaut fehlt; Eintrag in §25.

**Querverweise:** Verweise im übernommenen Wortlaut auf „§“ folgen noch der Nummerierung der Vorgängerfassung. Die Zuordnung zu den Abschnitten dieser Fassung steht in §3.4 (Invariantenregister) und in §26.2. Ein Umbau der Querverweise ist offen (O-Punkt, §25.2).

### 19.2 Katalog

| ID | Kl. | Anforderung (Wortlaut) | Mechanismus | Status in 2.3 (Basis 2.2.1) |
|---|---|---|---|---|
| REQ-ARCH-001 | [K] | Kanten (normal/build) MÜSSEN auf niedrigeren Ring zeigen oder, innerhalb eines Rings, als `may_use` deklariert und azyklisch sein (P1, 1.3 §6.10). | G: cargo metadata gegen capabilities.toml | übernommen |
| REQ-ARCH-002 | [K] | Der Abhängigkeitsgraph MUSS mit aufgelösten Features azyklisch sein; Dev-Aufwärtskanten nur als deklarierte Ausnahme (P5). | G: Ring-/DAG-Gate | übernommen |
| REQ-ARCH-003 | [K] | `[K]`-Crates DÜRFEN im Release-Default weder normal noch build von `[R]`/`[E]`-Crates abhängen (P30, INV-CORE-ISOLATION-1). | G: Kern-Isolation über cargo metadata | übernommen |
| REQ-ARCH-004 | [K] | `unsafe` DARF NUR in `simd`, `sys`, `wire` stehen; alle anderen Crates MÜSSEN `#![forbid(unsafe_code)]` tragen (P2). | G: Compiler-Lint + Unsafe-Inventar | übernommen |
| REQ-ARCH-005 | [K] | Jeder handgeschriebene `unsafe`-Block MUSS einen `// SAFETY:`-Kommentar tragen; crate-weite `allow` für `undocumented_unsafe_blocks` DÜRFEN NICHT existieren. | G: clippy deny + Allow-Zähler; P: Miri gegen sichere Referenz | übernommen |
| REQ-ARCH-006 | [K] | Produktionscode MUSS panic-frei sein (`unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, `assert!`, `process::exit`, unchecked Indexierung); Poisoning ist typisierter Fehler (P7). | G: Lint-Satz 1.3 §27.5 + Ratchet (Ziel 0) | übernommen |
| REQ-ARCH-007 | [K] | Jede Persistenz MUSS durch das WAL gehen; kein Seitenpfad schreibt Zustand (P24). | C + I | übernommen |
| REQ-ARCH-008 | [K] | Ring-0-Crates MÜSSEN synchrone Kern-Algorithmen bieten und DÜRFEN keine Async-Runtime als Produktions-/Build-Abhängigkeit führen; Port-Traits dürfen Futures liefern, `async fn` außerhalb von Port-Definitionen ist ratchet-begrenzt (P26). | G: Abhängigkeitsbaum-Prüfung (Runtime-frei) + Zählung `async fn` (Ratchet, Ziel 0) | übernommen |
| REQ-ARCH-009 | [K] | Zeit, Zufall und IDs MÜSSEN über `Clock`, `Rng`, `IdGenerator` injiziert werden; `SystemTime::now`, `Instant::now`, `thread_rng`, `HashMap/HashSet` mit RandomState in Kernlogik sind verboten (Ausnahme: Schlüssel-/Salt-Material, P28). | G: clippy `disallowed_*` + P: Wiederholbarkeit | übernommen |
| REQ-ARCH-010 | [K] | Eine Fähigkeit existiert nur, wenn die Composition Root sie verdrahtet und ein Integrationstest sie über den echten Pfad beweist (P29). | G: Wired-Check + I | übernommen |
| REQ-ARCH-011 | [K] | Ring, Klasse (`kern`/`rand`/`experimentell`) und `maturity` je Crate MÜSSEN ausschließlich in `capabilities.toml` stehen. | G: Doc-Truth | übernommen |
| REQ-ARCH-012 | [K] | Features der Klassen `[R]`/`[E]` MÜSSEN im Release-Default aus sein. | G: Feature-Matrix über cargo metadata | übernommen |
| REQ-ARCH-013 | [K] | `contextra-core` DARF keine Logik und keine neuen Abhängigkeiten tragen (nur Migrationsfassade). | G | übernommen |
| REQ-ARCH-014 | [K] | Interne Schlüssel DÜRFEN NICHT über die öffentliche API erreichbar sein (INV-SCAN-1, Definition in 1.3 §30.1). | G: `public-api`-Gate (verbotene Signaturen: Klartext-Schlüsselrückgabe, `*_for_test`-Zugänge in Release-Features) | übernommen |
| REQ-ARCH-015 | [K] | Ring 0 MUSS deterministisch sein: kein I/O außerhalb der Ports, keine Uhr und kein Zufall ohne Port, kein Ergebnis in Abhängigkeit von Hash-Iterationsreihenfolge, Thread-Scheduling oder unfixierter paralleler Float-Reduktion (INV-RING0-DETERMINISM-1). | G (`disallowed_*`, Ring-0-I/O-Scan) + P (Wiederholbarkeit, Reihenfolge-Permutation) | übernommen |
| REQ-ARCH-016 | [K] | Ein unklarer oder unbekannter Zustand MUSS zur Ablehnung führen, nie zu Raten oder Weiterlaufen (INV-FAILCLOSED-1). | C (Fehlerinjektion an jeder Schnittstelle) + I | übernommen |
| REQ-ARCH-017 | [K] | Der Kern DARF keine Echtzeit-Audio- oder STT-Funktion enthalten (VETO-OP03). | G: Deny-Liste über `cargo metadata` (Audio-/STT-Abhängigkeiten) + Review | übernommen |
| REQ-ARCH-018 | [K] | Die Kantenregeln aus 1.3 §6.10 MÜSSEN gelten: Fundament implizit, Kern-Kanten und Adapter-Kanten nur per `may_use`, Ring 4 erreicht Ring 2/3 nur über die Fassade, `LicenseGate` nur über den Port. | G: Ring-Gate über `cargo metadata` und `capabilities.toml` | übernommen |
| REQ-ARCH-019 | [K] | Mocks, Platzhalter-Embedder, Test-Hooks und Test-Schlüsselzugänge DÜRFEN NICHT Teil der öffentlichen Produktions-API sein; sie liegen hinter `testing`/`test-hooks`/`cfg(test)`, die in keinem Release- oder Default-Feature-Set liegen. | G: `public-api`-Gate + `compile_fail`-Doctests | übernommen |
| REQ-ARCH-020 | [K] | Jedes Cargo-Feature MUSS im Code wirken und in der Feature-Matrix getestet sein; ungenutzte (tote) Features DÜRFEN NICHT existieren; Entfernen öffentlicher Features erfolgt nach Deprecation. | G: Feature-Matrix + Feature-Konsistenz | übernommen |
| REQ-ARCH-021 | [K] | Jedes Crate MUSS die Workspace-Lints erben (`lints.workspace = true`); Werkzeug-Workspaces mit eigener Lint-Konfiguration sind in `capabilities.toml` deklariert; crate-weite `allow` für `undocumented_unsafe_blocks` oder `missing_safety_doc` DÜRFEN NICHT existieren. | G: Lint-Vererbungs-Gate + Allow-Zähler | übernommen |
| REQ-ARCH-022 | [K] | Die P28-Verbotsliste (`SystemTime::now`, `Instant::now`, `thread_rng`, RandomState-Hashes in Kernlogik) und die `assert!`-Familie MÜSSEN in einer versionierten `clippy.toml` (`disallowed-*`) stehen; zulässige Ausnahmen (z. B. die Uhr-Implementierung, Schlüssel- und Salt-Material, Tests, Benches, Werkzeuge) stehen in einer begründeten, einzeln geführten Allowlist. | G: clippy `disallowed_*` + Allowlist-Prüfung | übernommen |
| REQ-ARCH-023 | [K] | Reifegrad- und Statusangaben in Crate-Dokumentation und README MÜSSEN aus `capabilities.toml` stammen oder dagegen geprüft werden; widersprüchliche Angaben sind Gate-Fehler. | G: Doc-Truth | übernommen |
| REQ-ARCH-024 | [K] | Der Kern MUSS ohne jedes `[R]`- oder `[E]`-Crate bauen und testen; Rand-Fähigkeiten werden nur über Ports mit dem Verhalten „Ohne Rand“ aus 1.3 §6.11 verdrahtet. | G + I | übernommen |
| REQ-ASYNC-1 | – | Ratchet (Stufe B): `rg "async fn\ | in 2.2 §6 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-ASYNC-2 | – | Gate `async-io` hat 0 Verstöße und prüft Blockieren im async-Kontext zeilenweise (heute heuristisch). | in 2.2 §6 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-ASYNC-3 | – | Jede `spawn`-Stelle hat eine Obergrenze (Backpressure); Gate `unbounded-spawn`. | in 2.2 §6 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-AUD-001 | [K] | Die AuditChain MUSS append-only, hash-verkettet, tenant-getrennt und durabel sein. | P + F | übernommen |
| REQ-AUD-002 | [K] | Logs, Metriken, Fehlertexte und Panic-Meldungen DÜRFEN keine Nutzinhalte enthalten (INV-LOG-1). | G: Log-Scanner + P | übernommen |
| REQ-CHG-2 | – | nur als Verweis in §0.2; Wortlaut offen | offen | Lücke: referenziert, nicht definiert (§25) |
| REQ-CHG-4 | – | nur als Verweis in §0.4; Wortlaut offen | offen | Lücke: referenziert, nicht definiert (§25) |
| REQ-COG-001 | [R] | `grounding_check` MUSS unterhalb der Schwelle `PolicyViolation(LowConfidenceGrounding)` liefern (INV-GASP-1). | I + P | archiviert (Anhang B) |
| REQ-COG-002 | [R] | `MemoryType`-Regeln MÜSSEN gelten (INV-MEMTYPE-1..3). | P | archiviert (Anhang B) |
| REQ-COG-003 | [R] | Decay DARF nie löschen; Löschung entsteht nur durch ausdrückliche Aktion, Working-TTL oder opt-in Policy (INV-DECAY-1, INV-RETENTION-1). | P + I | archiviert (Anhang B) |
| REQ-DEL-001 | [K] | `forget` MUSS alle zehn Löschschichten über die Composition Root auslösen (INV-DELETION-WIRE-1). | T: Typestate + I | angepasst: zehn statt acht Schichten (§12.4) |
| REQ-DEL-002 | [K] | Die Schlüsselvernichtung MUSS nach dem Destroy-Protokoll D1 bis D6 ablaufen, durabel, idempotent und crash-sicher; sie gilt erst nach D3 als vollzogen (§12.2.7). | C: Crash an jedem Punkt + I | angepasst (wieder aufgenommen): Ebene B, nur zusammen mit Ebene A (E44) |
| REQ-DEL-003 | [K] | Jede der zehn Schichten MUSS in einem Bericht genau einmal als belegt oder nicht abgedeckt erscheinen (INV-PROOF-COVERAGE-1). | T: Typestate-Builder + I | angepasst: zehn Schichten |
| REQ-DEL-004 | [K] | Ein Schichtbeleg DARF NUR aus der Speicherschicht ausstellbar sein; Bypass-Konstruktoren sind außerhalb von test/fuzz/benches verboten (INV-WITNESS-MINT-1). | T + G: Aufrufer-Regel | übernommen |
| REQ-DEL-005 | [K] | Der Signierschlüssel des Beweises MUSS aus dem `KeyManager` stammen, nie aus einem frei übergebenen Byte-String (INV-PROOF-KEY-1). | T + P | übernommen |
| REQ-DEL-006 | [K] | Externe Zusicherungen MÜSSEN aus der belegten Garantiestufe erzeugt werden; „100 % gelöscht“ ist unzulässig (INV-CLAIMS-1). | G | übernommen |
| REQ-DEL-007 | [K] | `forget` MUSS über die Lineage auf Ableitungen kaskadieren (INV-DERIVED-1). | I + P | übernommen |
| REQ-DEL-008 | [K] | Ein Schicht-Messwert DARF NUR vom Scanner der Schicht erzeugt werden, MUSS `units_scanned > 0` haben (außer Schlüsselabwesenheit) und DARF NICHT aus einer Konstante oder einem Aufruferwert stammen. | T + Mutation (Konstante ersetzen ⇒ Test rot) + G (`deletion-honesty`) | übernommen |
| REQ-DEL-009 | [K] | Es DARF keinen Fallback-, Standard- oder Testschlüssel zum Signieren eines Löschbeweises geben; ohne konfigurierten Schlüssel entsteht kein Beweis (Fehler). | G: Literal-Scan + I | übernommen |
| REQ-DEL-010 | [K] | Keine Verifikationsfunktion DARF unbedingt „wahr“ liefern; fehlende Fähigkeit ist `CapabilityUnsupported`, nie ein Stub-Erfolg. | G + P | übernommen |
| REQ-DEL-011 | [K] | Ab `Frozen` DARF kein Lesepfad (`get`, `search`, `hydrate`, `export`, Checkpoint-Erzeugung) Daten des Betroffenen liefern, auch unter Parallelität. | P + L | übernommen |
| REQ-DEL-012 | [K] | Löschen MUSS unter dem Key-Lock über die gesamte Transaktion laufen; konkurrierendes Schreiben DARF kein Wiederauftauchen bewirken. | L | übernommen |
| REQ-DEL-013 | [K] | Nach Wiederherstellung eines Checkpoints MUSS jede bekannte Löschung vor dem ersten Lesezugriff erneut angewendet werden (Restore-Guard). | I | übernommen |
| REQ-DEL-014 | [K] | Die Löschkette MUSS als durable, idempotente, wiederaufnehmbare Saga laufen; Absturz in jedem Zustand führt nie zu „abgeschlossen“ ohne Messung. | C: Crash-Matrix | übernommen |
| REQ-DEL-015 | [K] | Nachweis und Ledger DÜRFEN NUR Metadaten und Referenz-Hashes enthalten, keine Klartext-IDs oder Inhalte. | G + P | angepasst: Referenz-Hash statt Commitment (E49) |
| REQ-DEL-016 | [K] | Jeder Claim im Bericht MUSS seine Voraussetzungen aus §12.11 erfüllen; `CryptoErasure{C1\|C2\|C3}` nur nach D1–D6, Epoch Seal und mit benannten Annahmen; K1 liefert höchstens C1. | T + P | angepasst: gestufte Claims (E44, E45) |
| REQ-DEL-017 | [K] | — | F + P + Referenz-Verifier | gestrichen: Referenz-Verifier entfällt (E26, REQ-PRF-6) |
| REQ-DEL-018 | [K] | Bei Aufbewahrungspflicht/Legal Hold MUSS die Saga nach Schritt 1 in `Restricted` übergehen: keine Löschoperation, keine Schlüsselvernichtung, Lesesperre aktiv. | I | übernommen |
| REQ-DEL-019 | [K] | Stufe S3 MUSS WAL-Segmente scrubben, ohne die Kette zu brechen. | C | übernommen |
| REQ-DEL-020 | [K] | Jeder Saga-Schritt und jede Klasse (L, P, C*, C1–C3, M) MUSS pro Schicht durch eine Messung belegt oder als nicht abgedeckt deklariert sein. | I + Canary-Scan (§17.9) | angepasst: Klassen statt Stufen S0–S3/SK |
| REQ-DEL-021 | [K] | `forget` MUSS sofort einen ehrlichen Teilbeweis und ein `PendingDeletion`-Handle zurückgeben; Offene Snapshots verzögern S2 (`Pending`), brechen nicht ab. | I | übernommen |
| REQ-DEL-022 | [K] | Der `RevocationLog` MUSS durabel (fsync vor Wirksamkeit), crash-sicher und bei jeder Autorisierung konsultiert sein; Schreiben DARF NICHT den bestehenden Log zerstören. | C + F | übernommen |
| REQ-DEL-023 | [K] | Die Löschpfad-Ports (INV-DELETION-PORTS-1: `LayerScanner`, `Purger`, `ErasureJournal`, `Vfs`, `DurableLog`, `KmsProvider`, `EffaceableKeySlot`, `ErasedSetMirror`) MÜSSEN in `governance/security_ports.toml` geführt werden und produktiv verdrahtet sein. | G: Port-Klassifikation + Wired-Check | angepasst: Liste um `EffaceableKeySlot`, `ErasedSetMirror` ergänzt |
| REQ-DEL-024 | [K] | Eine Collection mit `erasure_required` MUSS Verschlüsselung (K1 oder höher) voraussetzen; K2/K3 liefern bis zur bestandenen Konformitätssuite `CapabilityUnsupported`, nie einen Stub. | I | angepasst: K1 Standard, K2/K3 nur per Port (§12.2.3) |
| REQ-DEL-025 | [K] | Der Epoch Seal (§12.2.4) MUSS Re-Wrap aller lebenden Schlüssel, atomaren KeyTable-Wechsel und Vernichtung von `KEK[e]` leisten, journalisiert, idempotent und crash-sicher (AC-21). | Crash-Matrix + P | angepasst (wieder aufgenommen): KEK-Epochen statt Lösch-Epochen |
| REQ-DEL-026 | [K] | Gewrappte Schlüssel DÜRFEN NUR in der KeyTable liegen; im Payload, in Sidecars und im `KeyRegistry` stehen keine Wrap-Blobs (REQ-DEL-N5). | G + P: Rohscan, Nadelscan | angepasst: KeyTable statt `ErasureKeyStore` |
| REQ-DEL-027 | [K] | Löschtypen MÜSSEN genau einmal existieren; ohne Löschpfad liefert der Orchestrator `CapabilityUnsupported`; ein Stub, der Erfolg meldet, ist unzulässig. | G: Typfamilien-Zählung + `deletion-honesty` | übernommen |
| REQ-DEL-028 | [K] | Jedes persistierte kryptografische Objekt MUSS Algorithmus- und Formatkennung tragen; ein Algorithmenwechsel MUSS ohne Datenverlust möglich und der Austauschplan dokumentiert sein (Krypto-Agilität). | P: Migrationstest + Review | übernommen |
| REQ-DEL-029 | [K] | Der Rohdatei-Scanner MUSS alle Dateien des Datenverzeichnisses (WAL, SSTables, Checkpoints, Temp, `.bak`, Keystore, Journal) in Rohsicht und entschlüsselter Sicht lesen und durch eine positive Kontrolle belegen, dass er anschlägt (AC-1); ein Scanner, der nur Teilmengen liest oder die Kontrolle nicht besteht, erzeugt keinen sauberen Beleg. | I: Canary-Läufe (1.3 §27.9) | übernommen |
| REQ-DEL-030 | [K] | Die Lesesperre MUSS auch für historische Lesezugriffe (`as_of`, offene Snapshots, Checkpoint-Lesen) gelten, auch wenn die Altversion physisch existiert (INV-ASOF-ERASE-1). | P + L | übernommen |
| REQ-DEL-031 | [K] | Snapshots MÜSSEN das Höchstalter `snapshot_max_age` haben; abgelaufene Snapshots werden als `SnapshotExpired` final ungültig und nicht verlängert (INV-ASOF-ERASE-1). | L + I | übernommen |
| REQ-DEL-032 | [K] | `restore_checkpoint` und jeder Import MÜSSEN Schlüssel-Backups verweigern, deren `valid_until` überschritten oder deren Alter unbestimmbar ist; die Konfiguration MUSS `key_backup_max_age ≤ erased_set_retention` erzwingen (INV-RESTORE-HORIZON-1). | I + C + G | angepasst: Horizont nur für Spur K |
| REQ-DEL-033 | [K] | Referenzschlüssel MÜSSEN je (Tenant, Epoche) zufällig und gewrappt geführt werden; `K_ref[tenant, epoche]` MUSS nach Epochenende + `erased_set_retention` vernichtet werden, sodass die Referenz-Hashes der Epoche unzuordenbar werden, ohne die Prüfbarkeit der Ketten zu verlieren. | P + I | angepasst: Referenz-Epochen (§12.10, E49) |
| REQ-DEL-034 | [K] | Interne Checkpoints MÜSSEN eine eigene Löschschicht `Checkpoints` mit Scanner und `checkpoint_policy` (`Rewrite` Standard, `ExpireNow`, `ExpireAfterHold`) bilden; solange ein Checkpoint die Einheit enthält, ist der Bericht höchstens Klasse C ohne M. | T + I: Canary (AC-16) | angepasst: Standard `Rewrite`, Klassenbezug |
| REQ-DEL-035 | [K] | Statische Außentexte (README, MCP-Werkzeugbeschreibungen, `doctor`-Texte) MÜSSEN aus der Satzbibliothek erzeugt oder gegen die Claims der Standardkonfiguration (K1: höchstens C1) geprüft werden; ein Text mit einem vom Bericht nicht ausgestellten Claim ist ein Gate-Fehler. | G: `deletion-honesty`, `claim-generator` (AC-19, AC-24) | angepasst |
| REQ-DEL-036 | [K] | Jeder `Partial`-Zustand MUSS die Schwellen `partial_alarm_after`, `partial_escalate_after` und `partial_deadline_report` tragen und je Schwelle genau einen Alarm, eine Eskalation bzw. eine Meldung auslösen; kein `Partial` ohne ausstehende Schicht, Grund und Ablaufdatum. | I + C: `ManualClock` (AC-20) | übernommen |
| REQ-DEL-037 | [K] | `doctor` MUSS je Collection ein Löschinventar ohne Nutzinhalte ausweisen (§12.14: Löschfrist, Custody, `LegacyUnkeyed`-Anteil, Checkpoints, Retention-Werte, Alter des letzten Seal, Mirror, Härtungslücken, Residuale, offene `Partial`-Sagas); eine fehlende Löschfrist ist eine Warnung. | I | angepasst |
| REQ-DEL-N4 | [K] | Nach jeder Vernichtung und jedem Seal MUSS die KeyTable-Datei ohne vernichtete Einträge neu geschrieben werden (REQ-FMT-5). | C | in 2.3 definiert (§12.2.9) |
| REQ-DEL-N5 | [K] | `KeyRegistry` führt nur Ereignisse (`created`, `destroyed`, `sealed`) ohne Schlüsselmaterial und ohne Wrap-Blob. | G + P | in 2.3 definiert (§12.2.5) |
| REQ-DEL-N7 | [K] | Antwort-, RRF-, Embedding- und Block-Caches sind Pflichtbestandteil der Schicht `CachesAndVault` und MÜSSEN bei der Löschung invalidiert und zeroizt werden. | I + P | in 2.3 definiert (§12.4) |
| REQ-DEL-N9 | [K] | `doctor verify-report` MUSS Signatur, Prüfsumme, Klasse, Siegelkette und Zeitanker offline prüfen. | I | in 2.3 definiert (§12.7.5) |
| REQ-DEL-N15 | [K] | `doc_key_id` und Sequenzbezug stehen nur innerhalb des unter `K_eng` verschlüsselten Blockbereichs, nie im Klartext des Records; Padding nach Padmé. | F + Format-Test (AC-30) | in 2.3 definiert (§12.2.6) |
| REQ-DEL-N16 | [K] | Jedes Index-Artefakt hat genau einen ArtKey; beim Neuschreiben wird der alte vernichtet (INV-ARTKEY-1). | P + C | in 2.3 definiert (§12.2.8) |
| REQ-DEL-N20 | [K] | Schlüsselvernichtung darf nicht vor Siegel 2 beginnen (INV-DEL-ORDER-1); Validierung im Regulated-Tier mit zweiter Signatur (AC-33). | I + Typestate | in 2.3 definiert (§12.8.2, §12.7.7) |
| REQ-DEL-N21 | [K] | Wiederauferstehungsprüfung in B6/B7 und nach jedem Restore (INV-NO-RESURRECTION-1, AC-27). | C + P | in 2.3 definiert (§12.8.4) |
| REQ-DEL-N22 | [K] | Zwei-Spuren-Backup (INV-BACKUP-SPLIT-1, AC-22); `ErasedSetMirror` im Regulated-Tier Pflicht (AC-29). | I + G | in 2.3 definiert (§12.17) |
| REQ-DEL-N23 | [K] | Oberflächen-Inventar: jede persistente Dateiklasse ist einer Löschschicht zugeordnet (INV-SURFACE-1). | G | in 2.3 definiert (§12.4) |
| REQ-DEL-N24 | [K] | Klassen C1–C3 erst nach dem Epoch Seal; davor C* mit Residuum `R-KEKEPOCH` (INV-EPOCH-SEAL-1). | C + P | in 2.3 definiert (§12.2.4) |
| REQ-DEL-N26 | [K] | `erasure_grace` ist im Regulated-Tier Pflichtwert ohne Standard (Obergrenze 72 h); `Unfreeze` bis D3, im Regulated-Tier mit zwei Personen (AC-31). | I | in 2.3 definiert (§12.8.3) |
| REQ-DEL-N27 | [K] | Jedes Siegel (1 bis 4) ist ein signierter, in die AuditChain verketteter Bericht. | I | in 2.3 definiert (§12.8.5) |
| REQ-DEL-N37 | [K] | Bestand ohne DocDek (`LegacyUnkeyed`) erreicht höchstens Klasse P, bis `contextra migrate --rekey-units` ihn je Einheit neu verschlüsselt hat; `doctor` weist den Anteil aus. | I + Migrationstest | in 2.3 definiert (§12.2.5) |
| REQ-DUR-001 | [K] | Der Commit-Punkt MUSS die plattformgerechte Durability-Primitive nutzen (1.3 §7.8); Ack erst danach (INV-DURABILITY-PRIM-1). | C: FaultVfs je Plattform | übernommen |
| REQ-DUR-002 | [K] | Nach `fsync`/`fdatasync`-Fehler DARF kein Ack, kein Retry auf demselben Handle erfolgen; der WAL-Handle wird vergiftet (INV-FSYNC-1). | C | übernommen |
| REQ-DUR-003 | [K] | Volles Medium DARF keinen Phantom-Eintrag und keinen halb angewendeten Batch hinterlassen; WAL-Segmente werden vorab reserviert (INV-ENOSPC-1). | C | übernommen |
| REQ-DUR-004 | [K] | Lesepfade MÜSSEN I/O-Fehler als typisierte Fehler liefern; `mmap` nur für verifizierte, unveränderliche, lokale Dateien (INV-IO-1). | G (mmap-Nutzung) + C | übernommen |
| REQ-DUR-005 | [K] | Jede persistente Datei MUSS Magic und Formatversion tragen; neuere Version ⇒ Öffnen verweigert; Formate explizit spezifiziert (INV-FORMAT-1). | G: Kompatibilitätskorpus + F | übernommen |
| REQ-DUR-006 | [K] | Die WAL-Kette (HMAC-SHA256 + AEAD) und die Manifest-High-Water-Mark MÜSSEN Bruch, Truncation und Rollback erkennen. | C + F | übernommen |
| REQ-DUR-007 | [K] | Recovery MUSS die Reihenfolge aus 1.3 §7.3.1 einhalten und Verkehr erst danach zulassen. | C + I | übernommen |
| REQ-DUR-008 | [K] | Im persistenten Betrieb MUSS das Öffnen auf Netzwerk-, unbekannten FUSE- und Cloud-Sync-Dateisystemen verweigert werden. | I + C | übernommen |
| REQ-DUR-009 | [K] | Es DARF keinen persistenten Modus ohne Integritätskette geben; DirLock und Recovery-Prüfungen sind in `Full` nicht abschaltbar. | T: Enum-Varianten + G | übernommen |
| REQ-DUR-010 | [K] | Blob-Segmente MÜSSEN vor dem WAL-Commit durabel sein; Orphans räumt Recovery ab. | C | übernommen |
| REQ-DUR-011 | [K] | Metadaten- und Indexänderung MÜSSEN in einem WAL-Batch liegen; ein Applier wendet Indexe in WAL-Reihenfolge an; Rebuild ist bit-identisch (INV-ATOMIC-IDX-1, INV-INDEX-APPLY-1, INV-WAL-SSOT-1). | C + P | übernommen |
| REQ-DUR-012 | [K] | Ergebnisse nichtdeterministischer Quellen MÜSSEN als Werte ins WAL geschrieben werden; Recovery ruft die Quelle nie erneut auf (INV-NONDET-1). | P: Replay ohne Quellenaufruf + G | übernommen |
| REQ-DUR-013 | [K] | Schreiber MÜSSEN bei Rückstau gedrosselt werden (`l0_slowdown`, `l0_stop`); kein unbegrenzter Speicherzuwachs. | B + I | übernommen |
| REQ-DUR-014 | [K] | Betriebszustände `Healthy→Degraded→ReadOnly→Poisoned` MÜSSEN gemäß 1.3 §7.10 übergehen. | C + I | übernommen |
| REQ-DUR-015 | [K] | Sämtlicher Dateizugriff MUSS über den Port `Vfs` laufen; Ring-0-Crates führen keinen direkten Dateizugriff aus; `Vfs` und `DurableLog` sind produktiv verdrahtet (INV-RING0-DETERMINISM-1, INV-WIRING-2). | G: Ring-0-I/O-Scan + C: `FaultVfs` | übernommen |
| REQ-DUR-016 | [K] | Der Anwendungszustand MUSS aus WAL und Manifest rekonstruierbar sein; Sicherheitszustand liegt ausschließlich in den Sidecar-Logs (1.3 §7.11), mit Kette, Manifest-Marke und „Sidecar vor Wirkung“ (INV-SIDECAR-1, INV-WAL-SSOT-1). | G + C + I | übernommen |
| REQ-DUR-017 | [K] | Löschen MUSS in den Zuständen `ReadOnly` und `Poisoned` über die Notreserve und eigene Sidecar-Handles möglich bleiben; ohne jede Möglichkeit gilt die prozesslokale Lesesperre bis zum Neustart (INV-EMERGENCY-DEL-1). | C + I | übernommen |
| REQ-EXP-001 | [E] | Bandit-Routing bleibt aus, bis Regret-Vorteil nachgewiesen (INV-ROUTING-1..3). | OFFEN (nicht normativ) | nicht normativ (offen) |
| REQ-EXP-002 | [E] | Agent-Laufzeit und CLM-Scratchpad (Auslagerung empfohlen). | OFFEN (nicht normativ) | nicht normativ (offen) |
| REQ-EXP-003 | [E] | Lizenz-Prüfreihenfolge und Degradation ohne Datenverlust (INV-LICENSE-1..3). | P | [E] unverändert, Lizenzgate nur am Rand (1.3 §23 Anhang A) |
| REQ-EXP-004 | [E] | Audit-Export und AVV-Vorlagen mit Rechtsprüfungsvorbehalt. | OFFEN (nicht normativ) | nicht normativ (offen) |
| REQ-EXP-005 | [E] | SPLADE ergänzt BM25 hinter Feature-Flag. | OFFEN (nicht normativ) | nicht normativ (offen) |
| REQ-FAC-001 | [K] | Nur die Fassade `contextra` DARF konkrete Implementierungen wählen und verdrahten. | G: Wired-Check | übernommen |
| REQ-FAC-002 | [K] | Beim Start MUSS ein Verdrahtungs-Selbsttest laufen; fehlende Sicherheits-Port-Implementierung ⇒ Start verweigert. | I | übernommen |
| REQ-FAC-003 | [K] | Sicherheits-Ports MÜSSEN reale Implementierungen haben; Null-, Test- und Accept-All-Implementierungen als Produktionsstandard sind unzulässig (INV-WIRING-2). | G: Port-Klassifikation + Accept-All-Erkennung | übernommen |
| REQ-FAC-004 | [K] | Mock-Provider DÜRFEN NUR nach ausdrücklicher Konfiguration aktiv sein; unbekannte oder fehlende Provider-Konfiguration ist ein Startfehler. | G + I | übernommen |
| REQ-FAC-005 | [K] | Der Start MUSS die Phasen B0 bis B9 in fester Reihenfolge durchlaufen; die Lesesperre MUSS vor dem ersten Lesepfad armiert sein; jede Phase kann mit benanntem Grund verweigern, Teilstarts sind unzulässig (INV-BOOT-1). | I + C | übernommen |
| REQ-FGT-1 | – | Jeder Übergang wird **vor** der Ausführung ins durable Journal geschrieben (`fsync` mit Fehlerpropagation); jeder Schritt ist idempotent. | in 2.3 §12.8 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-FGT-2 | – | Beim Start (B7) setzt die Engine jede Saga ohne `Completed` fort oder meldet `Partial{outstanding}`. | in 2.3 §12.8 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-FGT-3 | – | `Frozen` schreibt den `ErasedSet`-Eintrag durabel **vor** der ersten Löschoperation; ab dann gilt die Lesesperre. | in 2.3 §12.8 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-FGT-4 | – | Fehler von `fsync`, `rename`, `write` in Purgern führen zu `Partial`, nie zu stillem Weiterlaufen. `compaction.rs:211` (`let _ = fsync_parent_dir`) wird propagiert. | in 2.3 §12.8 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-FGT-5 | – | Ein Scanner-Ergebnis mit `remaining > 0` hält die Saga in `Measured` mit `ResidueRemaining{layer, count}`; es entsteht ein Bericht, kein `Erased`. | in 2.3 §12.8 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-FGT-6 | – | Die Saga ist abbrechbar nur **zwischen** Schritten (§6 Punkt 5); Abbruch hinterlässt `Partial`. | in 2.3 §12.8 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-FGT-7 | – | Der `RequestContext` (Mandant, Prinzipal) kommt vom Aufrufer; kein Fallback auf `TenantId::SYSTEM` und kein `unwrap_or(TxId::new(1))` (heute `TenantId::try_new(1).unwrap_or(TenantId::SYSTEM)` in `tools_crud.rs:127,193`, `server_tools.rs:448`). | in 2.3 §12.8 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-FMT-1 | – | Jedes persistente Artefakt beginnt mit Magic (4 Byte), Formatversion (u16), Flags, Längenfeld und trägt eine Prüfsumme (CRC32C für Blöcke, Blake3 für Manifeste/Segmente). | in 2.2 §7.11 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-FMT-2 | – | Ein Leser verweigert **fail-closed** jede Version, die er nicht kennt; der Schreiber schreibt nur die aktuelle Version. | in 2.2 §7.11 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-FMT-3 | – | Für jede unterstützte Version liegt ein **Golden-File** im Repo; Round-Trip-Tests laufen in CI. | in 2.2 §7.11 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-FMT-4 | – | Serde-abgeleitete Layouts sind für signierte oder persistente Daten nur zulässig, wenn ein Golden-File sie festschreibt und die Serialisierungsbibliothek exakt gepinnt ist. | in 2.2 §7.11 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-FMT-5 | – | Atomares Ersetzen: temporäre Datei, `fsync`, `rename`, `fsync` des Verzeichnisses (Linux `fdatasync`, macOS `F_FULLFSYNC`); Fehler werden propagiert. Das gilt auch für den Keystore. | in 2.2 §7.11 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-FMT-6 | – | Migration läuft offline über `contextra migrate` mit Trockenlauf und vorheriger Sicherung; keine stille Migration beim Öffnen. | in 2.2 §7.11 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-FMT-7 | – | Vor 1.0 sind inkompatible Formatwechsel erlaubt, wenn `migrate` sie abdeckt; ab 1.0 sind On-Disk-Formate stabil und nur über neue Versionsnummern erweiterbar. | in 2.2 §7.11 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-GOV-001 | [K] | Merges MÜSSEN scope-treu, beschreibend benannt und über eine Merge-Queue auf dem Merge-Ergebnis geprüft sein (INV-PROVENANCE-1). | G | übernommen |
| REQ-GOV-002 | [K] | Änderungen an geschützten Pfaden MÜSSEN einen ADR-Verweis und eine Freigabe tragen. | G: protected-paths | übernommen |
| REQ-GOV-003 | [K] | Änderungen an Garantien oder Formaten aus 1.3 §28.6 MÜSSEN per ADR erfolgen. | Review + G | übernommen |
| REQ-GOV-004 | [K] | Jede Tatsache MUSS genau eine kanonische Quelle haben; doppelt vergebene IDs sind Gate-Fehler (INV-DOC-SINGLE-SOURCE-1). | G: Doc-Truth (inkl. Konsistenz Register 1.3 §5.2 ↔ 1.3 §27.6 ↔ 1.3 §33: jede `INV-*` einmal definiert, einmal in der Matrix, mindestens einmal referenziert) | übernommen |
| REQ-GOV-005 | [K] | Ein 1.0-Release MUSS die Kriterien aus 1.3 §28.1 erfüllen. | Review | übernommen |
| REQ-GOV-006 | [K] | Advisory-Ignorierungen MÜSSEN einzeln begründet, erreichbarkeitsanalysiert und befristet sein; kritische/hohe Advisories werden nicht ignoriert. | G: cargo-deny/audit/vet | übernommen |
| REQ-IDX-001 | [K] | Vektor-, Text- und Graph-Indexe MÜSSEN je (Tenant, Collection[, Modell-Hash]) physisch getrennt sein; keine tenantübergreifende Kante, Posting-Liste oder Statistik (INV-TENANT-IDX-1). | P/F: Cross-Tenant + T | übernommen |
| REQ-IDX-002 | [K] | Ein HNSW-Arena-Slot DARF NICHT wiederverwendet werden, solange eingehende Kanten möglich sind (INV-HNSW-SLOT-1). | L | übernommen |
| REQ-IDX-003 | [K] | HNSW-Löschung MUSS Tombstone-Pruning sein; kein partielles Rewiring (VETO-F02). | G: check_vetoes + P | übernommen |
| REQ-IDX-004 | [K] | Der ACL-Bitset MUSS vor der Distanzberechnung greifen; Ergebnisse enthalten keine Scores/Counts nicht berechtigter Knoten. | P | übernommen |
| REQ-IDX-005 | [K] | Gefilterte Suche MUSS exakt `k` oder alle passenden Treffer liefern (Overfetch 3k→6k→12k, nur deterministische Zähler). | P | übernommen |
| REQ-IDX-006 | [K] | SQ8-Kandidaten MÜSSEN exakt nachgerankt werden; das Ergebnis DARF NICHT vom Cache-Zustand abhängen. | P | übernommen |
| REQ-IDX-007 | [K] | SIMD-Kernel MÜSSEN gegen die skalare Referenz innerhalb dokumentierter Toleranz äquivalent sein. | P + Miri | übernommen |
| REQ-IDX-008 | [K] | BM25-Statistiken (IDF, Dokumentlängen) MÜSSEN je Tenant geführt werden. | P | übernommen |
| REQ-IDX-009 | [K] | RRF-Fusion MUSS k=60 mit Tie-Breaker Score absteigend, DocId aufsteigend verwenden. | P | übernommen |
| REQ-IDX-010 | [K] | LSM-Schlüssel, Index-IDs und Dateinamen DÜRFEN NUR opake IDs/Hashes enthalten (INV-KEYSPACE-1). | G + P | übernommen |
| REQ-IDX-011 | [K] | Kein stiller Platzhalter-Embedder; `test-utils` in keinem Release-/Default-Feature-Set; Modellhash und Dimension je Collection persistiert (INV-EMBED-1/-2). | G + I | übernommen |
| REQ-IDX-012 | [K] | Chunking MUSS deterministisch sein (gleicher Text + Konfiguration ⇒ gleiche Grenzen und IDs) und `source_span` tragen. | P | übernommen |
| REQ-IDX-013 | [R] | Graph-Indexe MÜSSEN je (Tenant, Collection) getrennt sein; PathRAG-Suche MUSS knotenbudgetiert abbrechen. | P | archiviert (Anhang B): Graph |
| REQ-IDX-014 | [R] | `rank`-Kalibrierung und Explain MÜSSEN deterministisch und reproduzierbar sein. | P | archiviert (Anhang B): rank ohne fusion |
| REQ-IDX-015 | [K] | `embedding_model_hash` MUSS aus der kanonischen Kodierung der Modellidentität berechnet und bei jedem Insert und jeder Suche gegen die Collection geprüft werden; Backends ohne berechenbare Identität sind für persistente Collections unzulässig (1.3 §15.3). | G + I | übernommen |
| REQ-KEY-001 | [K] | Jeder Schlüssel MUSS von einem `KmsProvider` stammen; mindestens die lokale Implementierung ist verdrahtet; Passphrasen-Ableitung mit festem/globalem Salt ist verboten (INV-KMS-1). | G + I: Selbsttest beim Start | übernommen |
| REQ-KEY-002 | [K] | Passphrasen-Schlüssel MÜSSEN über Argon2id mit zufälligem, persistiertem Salt und Parametern (m≥64 MiB, t≥3, ≥250 ms) entstehen (INV-KDF-1). | P + I | übernommen |
| REQ-KEY-003 | [K] | Nutzdaten MÜSSEN at-rest AEAD-verschlüsselt sein; ohne Verschlüsselung weist `doctor` rot aus (INV-ENCRYPT-REST-1). | I + G: Klartext-Scanner | übernommen |
| REQ-KEY-004 | [K] | Per `mmap` gelesene Dateien MÜSSEN auf verschlüsseltem Volume liegen, sonst Page-Decrypt-Cache (INV-ENCRYPT-REST-2). | G + I | übernommen |
| REQ-KEY-005 | [K] | AEAD MUSS AES-256-GCM-SIV mit 96-Bit-Nonce, 128-Bit-Tag und AD `scope_id‖key_id‖doc_id_hash` sein. | P: Vektoren + Negativtests | übernommen |
| REQ-KEY-006 | [K] | Das `KeyRegistry`-Log MUSS append-only, HMAC-verkettet und durabel sein; Löschung des Logs bei vorhandenen Daten ist Manipulation (fail-closed). | C + F | übernommen |
| REQ-KEY-007 | [K] | KEK-Rotation MUSS ohne Datenrewrite (Re-Wrap) funktionieren. | P + I | übernommen |
| REQ-KEY-008 | [K] | Geheimnisse MÜSSEN `zeroize`-geschützt sein und konstantzeitig verglichen werden; Schlüsseltypen DÜRFEN NICHT `Copy`/`Clone` ohne Zeroisierung sein. | G + P | übernommen |
| REQ-KEY-009 | [K] | Kryptografische Primitive DÜRFEN NICHT selbst gebaut werden. | Review | übernommen |
| REQ-KEY-010 | [K] | Im Produktionscode DÜRFEN keine Schlüssel-, Passphrasen- oder Signier-Literale stehen (auch nicht als Test-, Default- oder Fallback-Wert). | G: Literal-Scan `no-literal-secrets` | übernommen |
| REQ-KEY-011 | [K] | Jeder Schlüssel MUSS genau einen Zweck aus dem Zweckregister (1.3 §20.2c) haben; Labels sind paarweise verschieden; ein Label ohne Registereintrag ist Gate-Fehler (INV-KEYPURPOSE-1). | G + P | übernommen |
| REQ-KVC-001 | [R] | M1 DARF nur behauptet werden, wenn die Bedingungen B1–B3 aus 1.3 §13.1 erfüllt sind. | B + G | archiviert (Anhang B) |
| REQ-KVC-002 | [R] | `forget` MUSS KV-Prefixe invalidieren, die das Dokument enthalten. | I | archiviert (Anhang B) |
| REQ-MCP-001 | [K] | `stdout` MUSS ausschließlich dem MCP-Protokoll gehören; `print_stdout` ist `deny` im MCP-Binär (INV-STDOUT-1). | G | übernommen |
| REQ-MCP-002 | [K] | Genau ein Primärprozess MUSS das Datenverzeichnis halten; weitere Clients nutzen lokales IPC mit Peer-Credentials, kein TCP/UDP (INV-IPC-1). | I + C | übernommen |
| REQ-MCP-003 | [K] | Die 3-Tool-Fassade (`memory_remember`, `memory_recall`, `memory_forget`) und `ToolCategory`-Annotationen MÜSSEN verfügbar sein. | I | angepasst: nur die drei Agentenwerkzeuge in v1.0 (1.3 §0.5, 1.3 §13.2) |
| REQ-MCP-004 | [K] | Jeder Aufruf MUSS die 5-stufige Sicherheitspipeline durchlaufen. | I | übernommen |
| REQ-MCP-005 | [K] | Breite oder selektorbasierte Löschungen MÜSSEN eine Bestätigung über einen Kanal außerhalb des Tool-Aufrufs verlangen; ein vom Aufrufer setzbares Argument genügt nicht (INV-FORGET-GUARD-1). | I: Prompt-Injection-Szenarien | übernommen |
| REQ-MCP-006 | [K] | Recall MUSS Inhalt strukturiert mit `trust` und `source` liefern, nie als Anweisung (INV-TRUST-1). | I + P | übernommen |
| REQ-MCP-007 | [K] | Abbruch MUSS kooperativ bis in die Engine wirken; nach dem WAL-Append wird er ignoriert (INV-CANCEL-1). | I + C | übernommen |
| REQ-MCP-008 | [K] | Eingangs- und Ausgabegrenzen (RPC-Größe, `max_response_tokens`) MÜSSEN erzwungen und fuzz-geprüft sein. | F + I | übernommen |
| REQ-MCP-009 | [K] | Die CLI MUSS `init`, `doctor`, `verify-proof`, `confirm`, `migrate` bereitstellen. | I | angepasst: `verify-proof` entfällt mit REQ-PRF-6 (1.3 §13.1) |
| REQ-MCP-010 | [K] | Fehler MÜSSEN strukturiert und als retrybar/final klassifiziert sein. | I | übernommen |
| REQ-MVCC-001 | [K] | SSI MUSS im Produktionspfad aktiv sein; Read-Sets inkl. Prefix-/Range-Phantomschutz werden bei jedem Commit validiert (INV-SSI-ACTIVE-1). | P: Referenzmodell + C: History-Checker + L | übernommen |
| REQ-MVCC-002 | [K] | Indexeinträge MÜSSEN `born_seq`/`died_seq` tragen; Suchen sehen nur Einträge mit `born_seq ≤ S < died_seq` (INV-INDEX-SNAPSHOT-1). | P + L | übernommen |
| REQ-MVCC-003 | [K] | Ein Snapshot MUSS immer ein konkreter Sequenzstand sein, nie `u64::MAX`. | P | übernommen |
| REQ-MVCC-004 | [K] | Die Sperrenordnung MUSS vollständig und geordnet sein; Guards DÜRFEN NICHT über `.await` gehalten werden (INV-LOCK-ORDER-1). | L + G (`await_holding_lock`) | übernommen |
| REQ-MVCC-005 | [K] | Jede nebenläufige Kernstruktur MUSS ein loom-Modell besitzen (Inventar 1.3 §27.1). | G: Inventar-Abgleich | übernommen |
| REQ-PLT-001 | [K] | Durabilitäts- und Löschgarantien gelten nur auf der Garantieplattform; `doctor` weist `BestEffort` aus, Beweise tragen `platform_guarantee=false` (INV-PLATFORM-1). | CI-Matrix + C | übernommen |
| REQ-PLT-002 | [K] | Die vier Deployment-Tiers und `PerformanceProfile` MÜSSEN ohne Verletzung von Korrektheitsinvarianten wirken. | I | übernommen |
| REQ-PLT-003 | [K] | Löschlogik DARF sich nicht auf Unix-Unlink-Semantik offener Dateien stützen. | C: Windows-Handle-Nemesis | übernommen |
| REQ-PLT-004 | [K] | `doctor` MUSS Dateisystemtyp und Cloud-Sync-Pfad nach 1.3 §13.7 ausweisen und im persistenten Betrieb nicht unterstützte Typen verweigern. | I | übernommen |
| REQ-PORT-1 | – | Kernlogik (Ring 0–3) ruft weder `Instant::now`, `SystemTime::now`, `rand::thread_rng` noch `std::env::var` auf. Ausnahme: kryptografisches Schlüssel-/Salt-Material aus dem System-RNG, dokumentiert in `determinism-allow.toml`. | in 2.2 §5.4 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-PORT-2 | – | `KmsProvider` hat keinen Default und keinen Fallback. Fehlender Verwahrer verweigert den Start in B2. | in 2.2 §5.4 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-PORT-3 | – | Kein Mock/Fake/Null-Typ eines Sicherheitsports im Produktionscode; das Gate `security-ports` (heute „FEHLT“) prüft jeden Sicherheitsport auf eine produktive Implementierung **und** einen Verdrahtungstest (INV-WIRING-2). | in 2.2 §5.4 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-PORT-4 | – | Port-Signaturen sind semver-geprüft (`cargo public-api`); eine inkompatible Änderung verlangt eine neue Port-Version und eine Migrationsregel. Jede Erweiterung ist deterministisch (P28), budgetiert, auditiert, widerrufbar und besteht vor Registrierung eine Konformitätstestsuite. | in 2.2 §5.4 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-PRF-1 | – | Der Stub `no_crypto_stubs` (`engine/src/lib.rs:17`) und die leeren Mandanten-Hooks (`TenantIsolatedKvStore`: `purge_tenant_segments`, `on_rollback`, `remove_tenant_segment`; `Collection::kv_store` liefert konstant `None`) werden **gelöscht**; `crypto` ist nicht-optionale Abhängigkeit der Engine. Der Stub akzeptiert heute jede Restzahl (`new_after_verified_empty` liefert immer `Ok`, U-21). | in 2.3 §12.7.5 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-PRF-2 | – | `issue` liefert `Err`, wenn eine Pflichtschicht `remaining > 0` hat oder weder `Structural` noch `FeatureDisabled` ist, wenn die Klasse höher ist als die Messung trägt, wenn `excluded` leer ist oder wenn Annahmen fehlen, die die Klasse erfordert. `verify` mit falschem Schlüssel liefert `Err`. | in 2.3 §12.7.5 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-PRF-3 | – | Der Bericht nennt je Schicht Methode, Zahlen, Digest sowie `excluded`; die Außenformulierung wird aus dem Bericht erzeugt (INV-CLAIMS-1). Die `custody` bestimmt die Klasse (C1 bis C3). | in 2.3 §12.7.5 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-PRF-4 | – | `rg "default_test_deletion_proof_key"` im Produktionscode ergibt 0 Treffer (heute `crates/contextra-mcp/src/tools_crud.rs:206` mit Literal-Schlüssel); der Schlüssel kommt aus `KmsProvider`; `rg "contextra_delete\ | in 2.2 §3.3 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-PRF-5 | – | V5 mit Golden-File, Migrationspfad (V1–V3 lesbar) und Fuzz-Ziel; explizites Layout, kein `serde`-Derive (ersetzt „entfällt, E26“; E46). | in 2.3 §12.7.5 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-PRF-6 | – | `doctor verify-report` prüft offline Signatur, Prüfsumme, Klasse, Siegelkette und Zeitanker (ersetzt „entfällt, E26“). Ein Referenz-Prüfer in anderer Sprache dient nur als Testorakel (INV-ORACLE-1). | in 2.3 §12.7.5 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-PRF-7 | – | Jeder Bericht wird als Eintrag in die AuditChain geschrieben (Magic, Version, Prüfsumme, `fsync`) und ist für den Betreiber lesbar (`report(id)`, §13.9). | in 2.3 §12.7 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-PRV-001 | [R] | Egress MUSS alle fünf Schichten typisiert durchlaufen (`GuardedPayload`); ohne reale Erkennung ist Egress gesperrt. | T + I | übernommen |
| REQ-PRV-002 | [R] | Jede externe Wirkung MUSS budgetiert, auditiert und widerrufbar sein (INV-EGRESS-BUDGET-1). | I | übernommen |
| REQ-PRV-003 | [R] | `mlock`-Misserfolg MUSS wahrheitsgemäß gemeldet und in `doctor` ausgewiesen werden. | I | übernommen |
| REQ-QA-001 | [K] | Jede Zusage MUSS durch maschinell erzeugte Evidenz (Befehl, Toolchain, Commit, Exit-Code, Dauer, Hash) belegt sein; nicht ausgeführtes Prüfwerkzeug ist rot (INV-EVIDENCE-1). | G: `cargo xtask verify` (1.3 §27.10) | übernommen |
| REQ-QA-002 | [K] | Jede Kerngarantie G1–G4 MUSS ein unabhängiges Orakel besitzen (INV-ORACLE-1). | P: Referenzmodell / Brute-Force / Verifier anderer Sprache | angepasst: Kerngarantien RB1–RB4 (1.3 §2.5, 1.3 §17.8) statt G1–G4 |
| REQ-QA-003 | [K] | Jede Eintrittsstelle für nicht vertrauenswürdige Bytes MUSS ein Fuzz-Target haben; ein Gate gleicht die Matrix mit dem Code ab. | G | übernommen |
| REQ-QA-004 | [K] | Fuzz- und Miri-Läufe sind Pflicht-Gates; „SKIPPED“ gilt als rot. | G | übernommen |
| REQ-QA-005 | [K] | Mutation-Score je `[K]`-Crate MUSS in `governance/ratchet.toml` stehen und darf nur steigen. | G | übernommen |
| REQ-QA-006 | [K] | Chaos-Suite mit `FaultVfs` MUSS an jedem Schreib-/Durability-Punkt seed-reproduzierbar injizieren. | C | übernommen |
| REQ-QA-007 | [K] | Benchmarks MÜSSEN gegen versionierte Baselines und Budgets laufen; Vergleichsaussagen brauchen eine fremde Baseline. | B | übernommen |
| REQ-QA-008 | [K] | Jedes Audit-Skript MUSS einen Selbsttest mit synthetischem Repository haben; Grenzwerte DÜRFEN kein Gate faktisch abschalten. | G: Meta-Gate | übernommen |
| REQ-QA-009 | [K] | Gate-Läufe MÜSSEN auf einem von der Agenten-Infrastruktur unabhängigen Runner reproduzierbar sein; rohe Protokolle werden archiviert. | Review + CI | übernommen |
| REQ-QA-010 | [K] | Tests DÜRFEN NICHT gelöscht oder abgeschwächt werden, um Gates grün zu machen. | G: gate-weakening / test-integrity | übernommen |
| REQ-RST-1 | – | `ErasedSet` ist durabel, enthält nur Referenz-Hashes (keine Klartext-IDs) und wird in B6 vor Verkehr geladen (INV-BOOT-1). | in 2.3 §12.9 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-RST-2 | – | `restore_checkpoint` sperrt vor dem ersten Lesezugriff, wendet jeden Eintrag erneut an und schreibt `restore_reapplied` ins Journal (REQ-DEL-013). | in 2.3 §12.9 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-RST-3 | – | Schlüssel-Backups, deren signiertes `valid_until` überschritten oder deren Alter nicht bestimmbar ist, werden fail-closed verweigert; Start prüft `key_backup_max_age ≤ erased_set_retention`. | in 2.3 §12.9 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-RST-4 | – | Checkpoint-Erzeugung und Export prüfen den `ErasedSet` vor dem Schreiben. | in 2.3 §12.9 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-RST-5 | – | Backups außerhalb von Contextra stehen als `ExternalBackups`/`ExternalUnmanaged` im Bericht; die Rücksicherungsregel trägt der Verantwortliche. | in 2.3 §12.9 tabelliert | in 2.3 definiert (nicht in 1.3 §33) |
| REQ-SBX-001 | [R] | Ein `AdmittedModule` MUSS nur über `ModuleVerifier::verify` konstruierbar sein; der Digest wird aus den Bytes berechnet (INV-ADMISSION-1). | T + F | übernommen |
| REQ-SBX-002 | [R] | `High`/`Elevated`-Module DÜRFEN NICHT vor `Approved` laufen (INV-APPROVAL-1). | T + I | übernommen |
| REQ-SBX-003 | [R] | Wasmtime-Konfiguration MUSS bit-identische Ergebnisse auf x86-64 und aarch64 liefern (INV-WASM-DET-1). | P | übernommen |
| REQ-TEN-001 | [K] | Der Tenant MUSS aus der Transaktion abgeleitet werden; jede Schlüsselkonstruktion enthält das Tenant-Präfix (INV-TENANT-1). | T + P | übernommen |
| REQ-TEN-002 | [K] | Es DARF keinen mandantenübergreifenden Datenfluss geben (VETO-F10). | P/F | übernommen |
| REQ-TEN-003 | [K] | Ein vom Aufrufer gelieferter Tenant-Wert (fest verdrahtete Tenant-Konstante im Handler) DARF NICHT als Mandantenzuordnung dienen. | T + G: Literal-Scan | übernommen |
| REQ-TEN-004 | [K] | Tenant-Zustände `Aktiv`, `Gesperrt`, `Gelöscht` MÜSSEN gemäß 1.3 §24.5 übergehen; Tenant-IDs gelöschter Tenants werden nie wiederverwendet. | T + I | übernommen |
| REQ-TEN-005 | [K] | Quoten `tenant_quota_bytes` und `tenant_quota_docs` MÜSSEN in den Enterprise-Profilen gesetzt sein; Überschreitung liefert `QuotaExceeded`. | I + G | übernommen |

### 19.3 Zählung

**Zählung (Stand 2.2.1, vor 2.3):** 202 Anforderungen (142 übernommen, 9 angepasst, 4 gestrichen, 11 archiviert/nicht normativ, 33 in den Fachabschnitten definiert, 2 nur verwiesen). **Änderung in 2.3:** Die vier gestrichenen Anforderungen REQ-DEL-002, -025, -026 und REQ-PRF-5/-6 sind in geänderter Form wieder aufgenommen; 13 Anforderungen REQ-DEL-N4 bis N37 sind neu; die Zählung ist vom Doc-Truth-Gate neu zu erzeugen (REQ-GOV-004). Die Lücken REQ-CHG-2 und REQ-CHG-4 sind in §25.3 geführt. Eine Anforderung ohne Status ist ein Gate-Fehler (INV-DOC-SINGLE-SOURCE-1).

---

## 20. Entscheidungsregister

Dieser Abschnitt hält die Gestaltungsentscheidungen fest, die für die Umsetzung bindend sind. Jede Änderung ist ADR-pflichtig (§18.6).

### 20.1 Entscheidungen dieser Fassung

| ID | Thema | Festlegung | Fundstelle |
|---|---|---|---|
| E15 | Nebenläufigkeit (Ring 0) | Ring 0 bleibt synchron und laufzeitfrei; Ratchet; `store` und `engine` nach der gestuften Grenze in §6 | §6 |
| E18 | `checkpoint` im Kern | `checkpoint` ist Kern, weil Restore Gelöschtes zurückbringen kann | §4.2 |
| E19 | Fusion | RRF liegt ausschließlich in `rank/fusion`; `engine` enthält keine zweite Standardfusion (Änderung gegenüber früheren Fassungen) | §4.4, §11.5 |
| E26 | Löschbericht | **ersetzt durch E46** (2.3): `DeletionProof` Formatversion 4 war „eingefroren“, existiert im Repo aber nicht (U-39); V5 ist das einzige schreibbare Format | §12.7.5, REQ-PRF-5/6 |
| E27 | Schlüsselvernichtung | **teilweise aufgehoben durch E44** (2.3). Weiterhin entfallen: Lösch-Epochen (Rotation je Dokument), `subject_scope`, Zwecke `SegKey`, `TenantKek`, `EpochKek`, `INV-SHRED-1` in alter Form. Wieder aufgenommen: DocDek (je Einheit), ArtKey, KEK-Epoche mit Epoch Seal, Destroy-Protokoll, Verwahrungsstufen K2/K3 als Port | §12.2, §12.3 |
| E29 | Async-Umbau | Umbau des Ring-0-Bestands zurückgestellt; gilt nur als Ratchet, bis Spike S-ASYNC entscheidet | §6 |
| E30 | Formatumfang | Pflicht in v1.0: WAL, SSTable, Manifest, `AuditChain`, `RevocationLog`, Checkpoint-Manifest | §7.11 |
| E33 | SIMD | Skalarpfad ist Standard, bis eine Wiederholung von S-E2 einen Vorteil über 20 % zeigt | §4.2, §9.6 |
| E34 | Produktschicht | `ingest` und Tauri-App außerhalb des Kerns (Klasse `[R]`); Standardbau `embedded` ohne Inferenz-Backend | §4.5, §4.9 |
| E36 | Umfang v1.0 | Keine Python-Wheels und keine Admin-Werkzeuge außer `doctor` und Status | §2.3, §0.5 |
| E44 | Schlüsselvernichtung | Löschmaßnahme **Ebene B**; wirkt nur zusammen mit der physischen Bereinigung (Ebene A); erzeugt nie allein eine Löschbehauptung; Begründung: Flash-Reste und Kopien sind durch Überschreiben nicht zuverlässig erreichbar, Klartext-Strukturen nicht durch Schlüsselvernichtung | §12.1.1, §12.2 |
| E45 | Wirkungsklassen | L, P, C\*, C1, C2, C3, M; ein Bericht weist nur die gemessene Klasse aus (INV-CLASS-MEASURED-1) | §12.5 |
| E46 | Berichtsformat | V5 (`MFDR`), explizites Layout; V1–V3 lesbar; V4 bleibt unvergeben | §12.7.5 |
| E47 | Reihenfolge der Saga | Fast-Path (Sperre, Ebene A, Vernichtung, Messung) vor Hygiene; verworfen: „Key-first“ ohne vorherige Messung | §12.8.2 |
| E48 | Backup | Zwei-Spuren-Backup: Daten ohne Schlüssel, Schlüssel befristet (`key_backup_max_age`) | §12.17 |
| E49 | Pseudonyme | Referenz-Hash und Referenzschlüssel statt „Commitment“ (HMAC ist kein Commitment; EDPB Rn. 52) | §12.10 |
| E50 | Mixed-Segmente | nie automatische Freigabe; Standard `BlockAll` | §12.15 |
| E51 | Fristen | interne Zielgrößen; Contextra sendet nichts nach außen | §12.19 |

**Kollision der Kürzel.** Die Spikes S-E1 und S-E2 (§1.2) und die Evidenzstufen E1 und E2 (§1.4) tragen dieselben Kürzel. Die Spikes werden in dieser Ergänzung als **S-E1** und **S-E2** geführt. Eine Umbenennung der Spike-Kürzel in der Quelle ist offen (O-Punkt, §25.2).

**Lücken.** Die Entscheidungen E5 bis E14, E16, E17, E20 bis E25, E28, E31, E32 und E35 sind im Text dieser Fassung nicht ausgeführt. Sie sind nicht rekonstruierbar, ohne die Quelldokumente zu benennen. Sie sind als O-Punkt geführt (§25.3). Die Repo-Datei `ENTSCHEIDUNGEN.csv` nutzt eine andere Nummerierung (E1 bis E8) mit teils gegenläufigem Inhalt (§21, U-36).

### 20.2 Grundsatzentscheidungen der Vorgängerfassung 1.3 (F-01 bis F-11)

Die Grundsatzentscheidungen F-01 bis F-11 der Vorgängerfassung sind in dieser Fassung wie folgt zu lesen:

| ID | Festlegung 1.3 | Status in 2.3 (Basis 2.2.1) |
|---|---|---|
| F-01 | Gestufte Löschmodelle: Scope-DEK und Epochen-Hierarchie | **ersetzt** durch E44: Hierarchie Wurzel → KEK[coll,epoche] → DocDek/ArtKey/`K_ref` mit Epoch Seal (§12.2.2); keine Lösch-Epochen je Dokument |
| F-02 | `crypto` I/O-frei; durable Ablage über Port `DurableLog` in `store` | übernommen (§5.2, §12.2) |
| F-03 | Kein eigenes Crate `durable-fs`; Port `Vfs` in `ports`, Impl in `store` | übernommen (§5.2, §7.7) |
| F-04 | Async in Ring-0-Ports (P26): Runtime verboten, Futures erlaubt | übernommen (P26), konkretisiert durch E15/E29 (§6) |
| F-05 | Diese Spezifikation als einzige Quelle der Soll-Architektur | übernommen (§0.6, §18.7) |
| F-06 | 3-Tool-Fassade plus Vollwerkzeugsatz `contextra_*` | **angepasst**: nur die drei Werkzeuge in v1.0 (§13.2, E36) |
| F-07 | `WalNoHmac` entfällt; nur `Full` und `MemoryOnly` | übernommen (§13.7, REQ-DUR-009) |
| F-08 | Verwahrungsstufen K0 bis K3 | **angepasst**: K0 und K1 ausgeliefert; K2 und K3 als Port `EffaceableKeySlot`, aktiv erst nach bestandener Konformitätssuite (E44, §12.2.3) |
| F-09 | Interne Checkpoints als eigene Löschschicht | übernommen (§12.4, §12.5.4); Standard-Policy `Rewrite` |
| F-10 | Epochenweise Commitment-Schlüssel mit Restore-Horizont | **angepasst**: Referenz-Hash und Referenzschlüssel (E49); Horizont gilt für Schlüssel-Backups (§12.9, §12.10) |
| F-11 | Außenformulierung aus dem Beweis, stufenabhängig | **verschärft**: Satzbibliothek, jeder Satz an Messwert gebunden (§12.11) |

---

## 21. Ist-Befunde

Dieser Abschnitt hält den Befund fest, den Code und Repository zum Zeitpunkt der Ergänzung zeigen. Befunde sind **keine** Soll-Aussagen (§1.4, E0 bis E2). Die Prüfung erfolgte am Klon `tfufuz1/contextra` auf HEAD `a547a42a3` (Merge PR #5302). Die Spezifikation 2.2 nennt den älteren HEAD `8c6b470`; zwischen beiden liegen weitere Änderungen.

| ID | Befund | Ort in der Spezifikation | Prüfung am Klon | Stand |
|---|---|---|---|---|
| U-06 | `supports_ssi_tracking` liefert konstant `true` | §8.1 | Methode in `crates/contextra-ports/src/storage.rs:87` vorhanden; Rumpf nicht gelesen | offen |
| U-10 | `bincode` ist nicht exakt gepinnt | §12.7 (REQ-PRF-5) | `Cargo.toml:88`: `bincode = "1.3.3"` (Caret, kein `=`) | **offen, bestätigt** |
| U-11 | Standard-Features der Fassade sind nicht `embedded` | §4.5 | `crates/contextra/Cargo.toml:38`: `default = ["fast", "candle"]` | **offen, bestätigt** |
| U-21 | Sicherheitsstub `no_crypto_stubs` in der Engine | §12.7 (REQ-PRF-1) | `crates/contextra-engine/src/lib.rs:17` vorhanden | **offen, bestätigt** |
| U-22 | Literal-Schlüssel für Löschberichte | §12.7 (REQ-PRF-4) | `crates/contextra-mcp/src/tools_crud.rs:206` vorhanden | **offen, bestätigt** |
| U-23 | Fallback auf `TenantId::SYSTEM` | §12.8 (REQ-FGT-7) | `tools_crud.rs:127`, `:193`; `server_tools.rs:448` vorhanden | **offen, bestätigt** |
| U-24 | Verworfener `fsync` im Compaction-Pfad | §12.8 (REQ-FGT-4) | `crates/contextra-store/src/lsm/ops/compaction.rs:211` propagiert jetzt mit `?` | **behoben** (Historie: Commit `04d02f8b7`) |
| U-25 | Zeitabhängigkeit `Instant::now` in HNSW-Rebuild | §9.1 | keine Treffer in `crates/contextra-vector/src` | **behoben** (Historie: Commit `a07803c56`) |
| U-26 | `panic!` in der Tokenizer-Initialisierung | §10.2, §0.2 | `crates/contextra-text/src/tokenizer.rs:30` vorhanden | **offen, bestätigt** |
| U-27 | Panic-Stelle im WAL-HMAC | §7.2 (`hmac.rs:194`) | Zeile 194 enthält kein `panic!` mehr (`Ok(key) => key`); Drift | Befund verschoben, Prüfung offen |
| U-28 | `KmsProvider` nur als Trait in `crypto` | §12.2, §5.2 | Trait `pub trait KmsProvider` in `crates/contextra-crypto/src/wal_crypto.rs:18` vorhanden; Verdrahtung nicht geprüft | teilweise bestätigt |
| U-29 | Gate-Dateien fehlen | §17.3 | `governance/security_ports.toml` und `governance/key_purposes.toml` **nicht vorhanden** | **offen, bestätigt** |
| U-30 | Spezifikationspfad fehlt | §1.1 | `docs/spec/SPEC.md` nicht vorhanden; stattdessen `docs/CONTEXTRA_SPEZIFIKATION.md` (261 KB) | **offen** (Ablage nach Freigabe) |
| U-31 | Vetos mit abgelaufener Prüffrist | §3.1 (Status „bestätigt“) | `VETOES.md`: VETO-F02 und VETO-OP03 `conditionally_accepted`, `conditional_review_due: 2026-10-07` | **Widerspruch, offen** |
| U-32 | MCP-Werkzeugnamen weichen ab | §13.2 | `crates/contextra-mcp/src`: Werkzeuge `contextra_insert`, `contextra_search`, `contextra_forget`, `contextra_cloud_query`, `contextra_plugin_status` u. a. | **offen, bestätigt** |
| U-33 | Lizenzmodell uneinheitlich | 1.3 §23.5 nicht geregelt | `LICENSE-MIT` und `LICENSE-APACHE` im Repo; `FRAGEN_FACHBERATUNG.md` nennt BSL 1.1 als Ziel | **offen** (O-Punkt) |
| U-34 | Crate-Anzahl | §1.2 (33 Crates) | Workspace führt 35 Mitglieder (`Cargo.toml`) | Drift |
| U-35 | Mocks im Produktionscode | INV-EMBED-1, REQ-PORT-3 | Treffer für `MockEmbedder` und `TestMetricsSink` u. a. in `crates/contextra-mcp/src/explain.rs`, `config.rs` und `crates/contextra-agent/src/dlq.rs`; Test- oder Produktionscode nicht je Datei geprüft | offen |
| U-36 | Entscheidungsdatei widerspricht der Spezifikation | §20 | `ENTSCHEIDUNGEN.csv` (E1–E8): E1 hält Crypto-Shredding (in 2.3 mit E44 als Ebene B wieder aufgenommen, Widerspruch damit teilweise aufgelöst); E5 nennt `contextra_forget` mit Alias `memory_forget` | **Widerspruch, offen** |
| U-37 | Dummy-Beweise im Löschpfad (laut Repo-Entscheidung E5) | §12.7 | `contextra_delete` erzeugte laut Repo-Entscheidung E5 bisher ungeprüfte Dummy-Beweise (Restzahl 0) | Befund aus dem Repo, nicht geprüft |
| U-39 | Berichtsformat | §12.7.5 | Im Repo existiert höchstens `SignatureVersion::V3`; ein V4 gibt es nicht, 2.2.1 ging irrtümlich davon aus (E26) | **offen, aus Anhang übernommen, am Code nicht geprüft** |
| U-40 | `LayerWitness::issue_from_scanner` ist `pub` und nur per `#[doc(hidden)]` geschützt | §12.7.4 | jeder Crate kann einen Witness mit `remaining = 0` erzeugen | **offen, aus Anhang übernommen, am Code nicht geprüft** |
| U-41 | `DeletionLayer`-Varianten im Code weichen ab | §12.4 | sieben Varianten (`LsmMemtable`, `SsTableAllLevels`, `HnswIndex`, `WalAllSegments`, `CsrGraph`, `KvCacheSegments`, `EmbeddingCache`); Abbildung auf zehn Schichten in WP L-1 | **offen, aus Anhang übernommen, am Code nicht geprüft** |
| U-42 | `DeletionProof::issue` ignoriert den Parameter `_custody` | §12.7.5 | kein Bezug zu Klasse, Annahmen, Siegeln | **offen, aus Anhang übernommen, am Code nicht geprüft** |

**Befundlage.** Vier Befunde sind behoben, zwei wurden verschoben, der Rest ist offen oder widersprüchlich. Die behobenen Punkte stammen aus der Commit-Historie des Klons und sind nicht durch einen Gate-Lauf bestätigt (E1, §1.4).

---

## 22. Phasen und Arbeitspakete

Die Arbeitspakete (WP) sind in dieser Fassung nur benannt. Ihr Umfang, ihre Akzeptanzkriterien und ihre Reihenfolge stehen im Implementierungsplan (Dokument B), der nicht Teil dieser Datei ist (§18.7). Ein Arbeitspaket ist ein Pull-Request mit höchstens einem Crate-Schnitt (REQ-CHG-2).

| Phase | Inhalt | Arbeitspakete | Bezug |
|---|---|---|---|
| 0 | Belegbasis und Vetos | WP 0.2: Heuristik-Befunde neu messen (§21); WP 0.3: VETO-F02 und VETO-OP03 bestätigen (U-31) | §1.2, §3.1 |
| 1 | Verdrahtung Kryptografie und Löschbericht | WP 1.8: `KmsProvider` mit K0 und K1 verdrahten (U-28); WP 1.10: Löschbericht V5 und Pflichtfelder gegen §12.7.5 (ersetzt durch WP L-9); **Löschpfad-Arbeitspakete L-1 bis L-13 (§22.1)** | §12.2, §12.7 |
| 2 | Formate und Persistenz | WP 2.4: Formatregister (§7.11); Entscheidung SSTable-Magic `MFSX` oder Legacy `MFST` | §7.2, §7.11 |
| 3 | Engine, Schnitte und Gates | WP 3.2: Schnitt der Engine, Ort von `Reaper` und `MemoryType` (§11.10); WP 3.6: `capabilities.toml` auf die Tabellen in §4 setzen | §4, §11 |
| 4 | Löschpfad vollenden und Außentexte | WP 4.13 (Inhalt in Dokument B); WP 4.15: Referenz-Epochen (F-10); WP 4.16: Außenformulierung, Satzbibliothek und `deletion-honesty` (F-11, WP L-10) | §12.10, §12.11 |

### 22.1 Arbeitspakete Löschpfad (aus §12)

| WP | Inhalt | Abhängigkeit |
|---|---|---|
| L-12 | **zuerst:** `no_crypto_stubs`, Literal-Schlüssel, Fallback auf `TenantId::SYSTEM` entfernen (U-21 bis U-23) | – |
| L-1 | `DeletionLayer` auf zehn Varianten, Abbildung der sieben Code-Varianten (U-41), Gate `surface-coverage` | – |
| L-2 | `governance/key_purposes.toml` und `security_ports.toml` anlegen (U-29) | – |
| L-3 | `ScanSession`, `ControlToken`, Mint-Härtung, Gate `witness-mint` (U-40) | L-1 |
| L-4 | KeyTable (`MFKT`), DocDek je Einheit, Destroy-Protokoll D1 bis D6 | L-2 |
| L-5 | Epoch Seal mit Journal und Crash-Matrix | L-4 |
| L-6 | Zwei-Spuren-Backup (Spur D/K), Horizont | L-4 |
| L-7 | `ErasedSetMirror`, Restore-Guard, Wiederauferstehungsprüfung (B6/B7) | L-4, L-6 |
| L-8 | Saga mit Siegeln, Fast-Path und Hygiene, Typestate | L-3, L-5 |
| L-9 | Bericht V5 (`MFDR`), Golden-File, Migration V1–V3, Fuzz, `_custody` (U-39, U-42) | L-8 |
| L-10 | Claim-Generator, Satzbibliothek, `deletion-honesty` | L-9 |
| L-11 | Restmetadaten-Test, Padmé, Verification/Validation | L-4 |
| L-13 | Mixed-Policy, `RecipientLedger`, Fristen und Alarme, `LegacyUnkeyed`-Migration | L-8 |

**Phasenabnahme.** Eine Phase ist abgenommen, wenn alle Gates auf dem Merge-Ergebnis grün sind und die zugehörigen Befunde aus §21 mit Commit-Verweis geschlossen oder ausdrücklich als offen geführt sind (EP5, §18.1).

---

## 23. Betrieb, Diagnose und Beobachtbarkeit

### 23.1 `doctor`

`doctor` ist die Selbstdiagnose. Sie prüft und weist aus: Ringe und Kantenregeln, Formate und Versionen, Verschlüsselung an oder aus (INV-ENCRYPT-REST-1, rot bei aus), KDF-Parameter, Dateisystemtyp und Cloud-Sync-Pfad (REQ-PLT-004), Durability-Primitive, Speicherbindung und `mlock` (REQ-PRV-003), Plattformgrenzen (`BestEffort`, INV-PLATFORM-1), Integrität der Sidecar-Ketten, Kapazitäten und Notreserve sowie das **Löschinventar** (§12.14, REQ-DEL-037).

Zusätzlich in 2.3: `doctor verify-report` (Offline-Prüfung eines Berichts), `doctor export-concept` (nicht rechtsverbindliche Vorlage für das Löschkonzept), `doctor crypto-drill` (halbjährliche Übung des Algorithmenwechsels), Restmetadaten-Selbsttest (§12.7.6), Härtungslücken (§12.18), `ClockRegression`, `restore_unguarded` und offene Residuale mit Frist (§12.14).

`doctor` ist Beobachtung und keine Rechtsaussage. Eine fehlende Löschfrist ist eine **Warnung**, kein Fehler.

### 23.2 Gesundheitszustände

Die Zustände `Healthy`, `Degraded`, `ReadOnly` und `Poisoned` sind in §7.8 normiert. Jeder Übergang wird auditiert, ohne Nutzinhalte, und über `doctor` sichtbar (REQ-DUR-014). Ein Rückstau in der Kompaktion ist **kein** Betriebszustand (§7.4).

### 23.3 Logs, Metriken, Fehlertexte

Logs, Metriken, Fehlertexte und Panic-Meldungen enthalten nur IDs und Zähler, nie Nutzinhalte (INV-LOG-1, REQ-AUD-002). Die `MetricsSink`-Senke ist im Composition Root verdrahtet; das Null-Objekt ist für Beobachtungs-Ports zulässig (§5.2). Ein Log-Scanner prüft Logs auf Nutzinhalte (§17.1).

### 23.4 Audit

Die `AuditChain` ist append-only, hash-verkettet, tenant-getrennt und durabel (REQ-AUD-001). Sie enthält Zugriffe, Löschungen, Löschberichte und Freigaben. Löschereignisse stehen mit `subject_commitment`, nie mit einer ID (§12.6).

---

## 24. Datenlebenszyklus

### 24.1 `remember`

Eingabe, Prinzipal und Tenant aus der Transaktion, Limits und Ausleitungsprüfung, `trust` und `source`, `MemoryType` (Standard `Semantic`), Chunking (§11.7), Embedding über den Port als aufgezeichneter Wert (INV-NONDET-1). Große Werte werden vor dem Commit als Blob-Segment geschrieben (§7.1). Dann folgt **ein** WAL-Batch mit Primärdaten, `IndexIntent`s für Vektor und Text, Lineage und `SidecarIntent` für Audit. Nach dem durablen Flush (Commit-Punkt) wendet der Applier die Indexe an. Die Quittung `RememberReceipt` enthält Dokument-ID, WAL-Sequenz als Provenienz-Anker und `embedder_id`.

### 24.2 `recall`

Anfrage, Planner, Snapshot-Sequenz, `ScopeConstraint` und ACL-Bitset. Dann Query-Embedding im Blocking-Pool, Vektorsuche und BM25 parallel am Snapshot mit `max_results`. Die Fusion erfolgt über RRF in `rank/fusion`. Die Lesesperre entfernt Treffer gesperrter Betroffener **vor** der Rückgabe (REQ-DEL-011). Das Ergebnis wird auf `max_response_tokens` begrenzt; eine Kürzung wird ausdrücklich gekennzeichnet. Jeder Treffer trägt `{content, trust, source, lineage}`. Inhalte sind Daten und nie Anweisungen (INV-TRUST-1).

### 24.3 `forget`

Autorisierung (Kategorie `Destructive`) und Forget-Guard (§13.3). Danach läuft die Saga nach §12.8.2:

1. **Fast-Path.** `Frozen` (`ErasedSet`-Eintrag durabel, Lesesperre, Spiegelung an den Mirror; Siegel 1, Klasse L), sofortige Rückgabe eines `DeletionReceipt` mit Siegel 1 und `PendingDeletion`-Handle (REQ-DEL-021); Tombstone-Batch und Lineage-Kaskade (INV-DERIVED-1); Ebene-A-Bereinigung und Messung (Siegel 2, Klasse P); nach optionaler `erasure_grace` Schlüsselvernichtung D1 bis D6 (INV-DEL-ORDER-1); Epoch Seal, soweit fällig (Siegel 3, Klasse C\* bzw. C1 bis C3).
2. **Hygiene** (terminiert, ≤ `purge_rebuild_max_days`): Purge-Kompaktion, WAL-Scrub, Blob-Rewrite, Checkpoint-Policy, Graph-Neuaufbau; Messung, Siegel 4 (Klasse M), verankert in der `AuditChain`.

`report(id)` liefert den jeweils höchsten erreichten Stand. Solange offene Snapshots die Hygiene verzögern, lautet der Status `Pending` (§12.5.5). Die Saga ist idempotent und nach einem Absturz wiederaufnehmbar; ihr Zustand steht im `ErasureJournal`. Die Außenformulierung entsteht aus der Satzbibliothek (INV-CLAIM-BIND-1, §12.11).

### 24.4 Checkpoint und Backup

Siehe §13.8. Export und Restore unterliegen dem Restore-Horizont (INV-RESTORE-HORIZON-1). Checkpoints im Datenverzeichnis sind interne Kopien (Spur D, ohne KeyTable) und eine eigene Löschschicht (§12.4); Schlüssel-Backups (Spur K) sind befristet (§12.17). Backups außerhalb von Contextra sind im Bericht als `ExternalBackups` ausgewiesen (§12.9, REQ-RST-5).

### 24.5 Tenant-Lebenszyklus (REQ-TEN-004)

Ein Tenant durchläuft die Zustände `Aktiv`, `Gesperrt` und `Gelöscht`.

- **Aktiv → Gesperrt** (Kategorie `Admin`): keine Schreibzugriffe; Lesen nur für Betreiber mit Protokollierung; Schlüssel bleiben erhalten. Die Rückkehr nach `Aktiv` ist nur bei unveränderten Schlüsseln zulässig.
- **Gesperrt → Gelöscht** (`erase_tenant`, Kategorie `Destructive`): Der Tenant wird im `ErasedSet` eingetragen, bevor die Saga beginnt (INV-BOOT-1). Jede Collection durchläuft die Stufen S1 bis S3. Je Scope entsteht ein Bericht.
- **Gelöscht** ist endgültig. Die Tenant-ID wird nie wiederverwendet. Schreibversuche liefern `TenantErased` (final).

---

## 25. Offene Punkte

Ein offener Punkt ist **keine** Zusage. Bis zur Klärung enthalten Produkt- und Beweistexte keine Aussage, die davon abhängt (INV-CLAIMS-1, §0.6 Punkt 4).

### 25.1 Rechtliche Prüfpunkte (P-01 bis P-20)

Die Punkte sind offene Prüfaufträge an die Projektverantwortlichen, eine Fachanwältin oder einen Fachanwalt für Datenschutzrecht oder die Datenschutzbeauftragte bzw. den Datenschutzbeauftragten. Keiner ist durch die Technik entschieden. Die Belegstärken folgen der Legende in §12.1.

| ID | Prüfpunkt | Status in 2.3 (Basis 2.2.1) |
|---|---|---|
| P-01 | Ist Schlüsselvernichtung eine Löschung nach Art. 17, eine Anonymisierung oder nur eine Einschränkung? | ⚠ offen |
| P-02 | Entfällt der Personenbezug nach Schlüsselvernichtung für jede Stelle (relativer Personenbezug)? | ⚠ offen |
| P-03 | Welcher Nachweis belegt, dass kein Schlüsselmaterial verbleibt (RAM, Swap, Dumps, Backups des Schlüsselspeichers)? | ⚠ offen; für K0 und K1 nicht ausschließbar (Annahmen A-FLASH, A-RAM, §12.5.2) |
| P-04 | Liegen Quelle, Chunks, Vektoren, Volltextindex, Caches, Logs und Prompt-Historie im Wirkungsbereich? | [P]; durch A2 (§12.1) abgedeckt |
| P-05 | Behandlung von Embeddings (vorsorglich personenbezogen) | ⚠ offen |
| P-06 | Zulässige Backup-Frist, Wiedereinspiel-Sperre, separater Backup-Schlüssel | [B] Sperre; Frist offen |
| P-07 | Nachweis-Metadaten: Dauer, Personenbezug der Lösch-ID; „Lösch-Paradoxon“ | ⚠ umstritten |
| P-08 | Rolle des Anbieters (kein Datenzugriff, keine Telemetrie, kein Key-Recovery) | ◐ |
| P-09 | Aufbewahrungspflichten; Archiv getrennt vom RAG-Index; Einschränkung statt Löschung | [R] + ⚠ |
| P-10 | Werbe- und Produktaussagen (Wettbewerbs- und Vertragsrecht) | ○ |
| P-11 | Löschfrist „unverzüglich“, Monatsfrist und Obergrenze für Batch-Fenster und Epochengröße | [R] ○; interne Zielgrößen (§15.5) bestätigen |
| P-12 | Empfängerinformation (Art. 17 Abs. 2, Art. 19) und Auftragsverarbeiter-Nachweis | [R] ○/◐ |
| P-13 | Krypto-Agilität: Notfallplan bei Algorithmenwechsel | [B] (Analogie) |
| P-14 | Verifikation der mit ◐ und ○ markierten Fundstellen im Volltext | offen |
| P-15 | Behandlung interner Checkpoints als Schicht; Haltefrist; Standardwert von `erasure_required` | ⚠ offen |
| P-16 | Zulässigkeit und Frist dauerhafter Pseudonyme; Länge von `erased_set_retention` und `ref_epoch_len` | ⚠ umstritten |
| P-17 | Cyber Resilience Act (VO (EU) 2024/2847), Anbieterrolle nach dem AI Act, NIS2-Lieferkette. Die Rechtsanalyse nennt Meldepflichten ab 11.09.2026 und Hauptpflichten ab 11.12.2027; Anwendbarkeit prüfen | ○ offen |
| P-18 | AVV, Drittlandtransfer und Beweiswert der Attestierung bei **K3-Verwahrern** | relevant, sobald K3 aktiviert wird (§12.2.3); Klasse C3 verlagert A-FLASH auf den Anbieterattest |
| P-19 | Frist- und Eskalationsregel für `Partial`-Zustände (Schwellen aus §15.5) | [R] ○ |
| P-20 | Weitere Betroffenenrechte (Auskunft, Berichtigung, Einschränkung der Indexe) | offen; im Kern nur `Restricted` (§12.8) |
| P-21 | Zulässigkeit eines Mixed-Segments nach Entfernung eines Betroffenen (Art. 17 Abs. 3) | ⚠ offen (§12.15) |
| P-22 | Berufsrechtliche Aufbewahrung bei Berufsgeheimnisträgern (§ 203 StGB, Kammerrecht) und Verhältnis zu Art. 17 Abs. 3 | ⚠ offen |

### 25.2 Technische und Produkt-Offene Punkte (O-Punkte)

| ID | Frage | Fundstelle |
|---|---|---|
| O-5 | Ist der strikte ACL-Modus umgesetzt, der bei Unterlieferung in den exakten Scan wechselt? (Review D2 der Vorgängerfassung; nicht geprüft) | §9.4 |
| O-6 | Vorteil eines deutschen Analyzers für den DACH-Markt; Marktannahme, nicht belegt | §10.2 |
| O-12 | Dauer und Zulässigkeit der Einschränkung `Restricted`; Rechtsfrage | §12.8 |
| O-14 | „Akte“ gleich Collection (Annahme) | §0.4 |
| O-15 | Standard-`checkpoint_policy`: in 2.3 auf `Rewrite` gesetzt (§12.5.4); Bestätigung durch den Eigentümer | §12.5.4 |
| O-17 | Rewrite des `ErasedSet` nach `erased_set_retention` (Kipptest; dann `restore_unguarded`) | §12.10 |
| O-18 | Windows: Garantieplattform oder nur Best Effort als Bibliothek? | §13.7, §13.7 |
| O-19 | `PowerUserLocal` als Tier übernehmen oder streichen? | §14.1 |
| O-20 | Mechanismus der Bestätigung: Kanal der `confirm`-Befehl oder OS-Dialog; Gültigkeitsdauer des Tokens | §13.3 |
| O-21 | Statuswerkzeug für MCP: ja oder nein in v1.0? | §13.1 |
| O-22 | `verify-proof` (offline) in v1.0 oder entfallen (REQ-PRF-6 entfällt)? | §13.1 |
| O-23 | Referenz-Verifier für RB2 (Orakel): entfällt mit REQ-PRF-6, aber §17.8 nennt ihn | §17.8 |
| O-24 | Budgets und Recovery-Zeitbudgets je Tier in `benchmarks/budgets.toml` (Zahlen) | §16.4 |
| O-25 | Garantieplattform der Vorgängerfassung und Matrix §13.7 (Windows Tier 2) gegen §0.5 abgleichen | §0.5, §13.7 |
| O-26 | Lizenz: MIT OR Apache-2.0 (Repo) oder BSL 1.1 (Fachfragen); Lizenzgate (`license`) `[E]` | 1.3 §23.5, U-33 |
| O-27 | Klasse des Crates `contextra-py`: `[E]` in §4.1, `[R]` in §4.2 und §4.5 | §4.1, §4.2 |
| O-28 | Gate-Dateien `governance/security_ports.toml` und `governance/key_purposes.toml` anlegen | §17.3, U-29 |
| O-29 | Gate-Namen des Repos gegen den Katalog in §17.3 abgleichen | §17.3 |
| O-30 | Umbenennung der Spike-Kürzel S-E1 und S-E2 (Kollision mit Evidenzstufen) | §1.2, §1.4, §20.1 |
| O-31 | Umbau der Querverweise in §19 auf die Nummerierung dieser Fassung | §19.1 |
| O-32 | Inhalt der Arbeitspakete (Dokument B) | §22 |
| O-33 | Auflösung des Widerspruchs bei den Vetos (U-31) | §3.1, §21 |
| O-34 | Anbieter des qualifizierten Zeitstempels (`TimestampAuthority` `[R]`) | §12.7.9 |
| O-35 | zentrales Löschregister als Mirror-Ziel | §12.17.3 |
| O-36 | echtes Commitment-Schema optional | §12.10 |
| O-38 | plattformspezifische Härtung (Spike S-HRD) | §12.18 |
| O-40 | Kosten von Re-Wrap und Padding messen (`budgets.toml`) | §12.2.4, §12.2.6 |
| O-41 | Typ-Abbildung der sieben Code-Varianten auf zehn Schichten (U-41) | §12.4 |

### 25.3 Lücken in der Nummerierung

- **Entscheidungen:** E5 bis E14, E16, E17, E20 bis E25, E28, E31, E32, E35 (nicht ausgeführt, §20.1).
- **Offene Punkte:** O-1 bis O-4, O-7 bis O-11, O-13, O-16 (nicht ausgeführt).
- **Anforderungen:** REQ-CHG-2 und REQ-CHG-4 werden verwiesen (§0.2, §0.4), aber nicht definiert.
- **Arbeitspaket 4.13:** Inhalt im Dokument B.

---

## 26. Änderungsverlauf und Übernahme

### 26.1 Fassungen

| Fassung | Status |
|---|---|
| 1.3 (Stand 25.10.2026) | Vorgänger; als Belegquelle für fehlende Abschnitte dieser Ergänzung herangezogen, mit Vorrang der Fassung 2.2 (§0.6) |
| 2.0 und 2.1 | obsolet laut 2.2; **nicht** herangezogen |
| 2.2 (Stand 26.10.2026) | Basisfassung |
| 2.2.1 (vervollständigt) | Vorgänger dieser Fassung: §13 bis §26 und Anhänge A und B ergänzt |
| 2.3 (Entwurf) | diese Fassung: §12 vollständig ersetzt, abhängige Stellen angepasst (§26.5) |

### 26.2 Übernahme aus 1.3

| 1.3 | Übernahme in 2.3 | Änderung |
|---|---|---|
| 1.3 §21 MCP | §13.1 bis §13.3, §13.6 | Vollwerkzeugsatz entfällt (E36) |
| 1.3 §22 Fassade und B0–B9 | §13.4, §13.5 | B3 zusätzlich mit Schichtbelegung (§12.4) |
| 1.3 §23 Tiers, Profile, Plattform, Lizenz | §13.7, §14.1, §13.7 | Windows-Status offen (O-18); Lizenz nur verwiesen (O-26) |
| 1.3 §24 Datenlebenszyklus | §13.8, §24 | SK entfällt; `report(id)` statt `poll_deletion_proof` |
| 1.3 §25 Konstanten | §15 | Lösch-Epochen (Batch-Fenster, `N_epoch`, `L_wrap`) entfallen; KEK-/Referenz-Epochen und Seal-Konstanten neu |
| 1.3 §26 Messbare Verträge | §16 | Löschvertrag an E44/E45 angepasst; Zahlen offen (O-24) |
| 1.3 §27 Qualität und Gates | §17 | Gate-Katalog gegen den Repo-Stand geprüft (§17.3) |
| 1.3 §28 Governance | §18 | ADR-Liste auf die Entscheidungen dieser Fassung angepasst |
| 1.3 §33 Anforderungskatalog | §19 | 167 Anforderungen übernommen, mit Status |
| 1.3 §34 Grundsatzentscheidungen | §20.2 | F-01 ersetzt, F-06, F-08 angepasst |
| 1.3 §20.13 Rechtliche Prüfpunkte | §25.1 | P-18 entfällt; Rest unverändert |
| 1.3 §20.4a Checkpoints, §20.2b Epochen | §12.4, §12.5.4, §12.10 | Checkpoint-Schicht, KEK-Epochen (Epoch Seal) und Referenz-Epochen übernommen; Lösch-Epochen je Dokument entfallen |
| 1.3 §1 bis §19 (Vision, Subsysteme, Archiv) | Anhang B | nicht normativ |

**Streichungen gegenüber 1.3 (Stand 2.3):** Lösch-Epochen je Dokument (`TenantKek`, `EpochKek`, `SegKey`), `subject_scope`, `ErasureKeyStore` und `KeyRef`, `WalNoHmac`, Vollwerkzeugsatz, `INV-SHRED-1` in alter Form. **Wieder aufgenommen oder ersetzt in 2.3:** Schlüsselvernichtung als Ebene B (SK, `REQ-DEL-002`), Epochen als KEK-Epochen mit Epoch Seal (`REQ-DEL-025`), Verwahrungsstufen K2/K3 als Port, kryptografische Claims als Klassen C1–C3, Referenz-Prüfung als `doctor verify-report` (`REQ-DEL-017` ersetzt).

### 26.3 Korrekturen an 2.2 in dieser Ergänzung

1. §0.2, Regel 4: „sieben Sidecar-Logs“ auf **fünf** Sidecar-Logs korrigiert. Der Keystore `MFKS` ist kein Log; §7.9 führt fünf Logs und den Keystore (erledigt).
2. §13.5: Die Phase B3 wird mit der Schichtbelegung ergänzt, weil §12.4 und REQ-FAC-005 auf dieselbe Phase verweisen.
3. §14.1: `PowerUserLocal` aus 1.3 ergänzt; 2.2 nennt ihn nicht (O-19).
4. §20.1: Kollision der Kürzel E1 bis E4 (Spikes und Evidenzstufen) festgehalten (O-30).
5. Widersprüche, die **nicht** in 2.2 entschieden werden, sind als O-Punkte geführt: Windows (O-18), Vetos (O-33), Klasse `py` (O-27).

### 26.4 Ergebnis der Ergänzung

Ergänzt: §13 bis §26, Anhang A, Anhang B. Die Ergänzung löst alle Verweise auf §13 bis §26 auf. Offen bleiben die in §25 geführten Punkte, die Zahlen in `budgets.toml` und der Inhalt der Arbeitspakete in Dokument B.

### 26.5 Änderungen in Fassung 2.3 (Löschpfad)

1. **§12 vollständig ersetzt** durch die überarbeitete Löschspezifikation (zehn Schichten, Klassen L/P/C\*/C1–C3/M, Schlüsselvernichtung als Ebene B mit Destroy-Protokoll und Epoch Seal, Fast-Path vor Hygiene, gestufte Siegel, Zwei-Spuren-Backup mit `ErasedSetMirror`, Mixed-Policy, Referenz-Hash statt Commitment, Berichtsformat V5, Satzbibliothek, Abnahmekriterien AC-1 bis AC-33).
2. **Angepasst:** §0.2 Regel 9, §0.3, §0.5, §2.3, §2.4 (G1 bis G3), §2.6, §3.4 (sieben Invarianten verschärft oder geändert, neun neu), §4.2/§4.3, §5.2, §7.2, §7.4, §13.5, §13.8, §14.2, §15.1/§15.2/§15.5, §16, §17.3/§17.5, §18.6, §19 (Löschanforderungen), §20 (E26/E27 geändert, E44 bis E51 neu), §21 (U-39 bis U-42), §22 (WP L-1 bis L-13), §25 (P-21, P-22, O-34 bis O-41), Anhang A.
3. **Nicht rückführbar (bewusst offen):** Inhalt von AC-7, AC-12, AC-14, AC-15 und der Entscheidungen E5 bis E14 usw. (§25.3).
4. **Nicht am Code geprüft:** U-39 bis U-42 stammen aus dem Anhang und sind am Repository nicht verifiziert. Rechtsfundstellen mit ◐/○ sind vor Außennutzung im Volltext zu prüfen (P-14).
5. **Zu klären vor Freigabe:** O-15 (Standard-Policy `Rewrite`), E44 (teilweise Rücknahme von E27), Aktivierung von K2/K3, Zählung der Invarianten und Anforderungen durch das Doc-Truth-Gate.

---

## Anhang A. Feature- und Crate-Katalog (`[R]`, `[E]`)

Die Features sind im Standardbau aus. Die Fassade hat `default = ["embedded"]` als Zielbild (E34); der Repo-Stand weicht ab (U-11). Die Zeilenzahlen stammen aus §4.2 und sind Code-Zeilen nach `loc.py`.

| Feature | Crate | Klasse | Zeilen (Ist) | Zweck | Standardbau |
|---|---|---|---|---|---|
| `candle` | `contextra-infer-candle` | `[R]` | 2.660 | Embedding-Backend | aus (Ziel) |
| `privacy` | `contextra-privacy` | `[R]` | 1.331 | Ausleitungsschichten, Surrogat-Vault (§12.4) | aus |
| `sandbox` | `contextra-sandbox` und `mcp/sandbox.rs` | `[R]` | 1.105 + 431 | WASM-Filter und Admission (INV-ADMISSION-1) | aus |
| `py` | `contextra-py` | `[R]`, siehe O-27 | 1.426 | Python-Bindung (E36: nicht in v1.0) | aus |
| `license` | `contextra-license` | `[E]` | 307 | Signierte Aktivierung, Degradation ohne Datenverlust (INV-LICENSE-1 bis -3) | aus |
| `audit-export` | `contextra-audit-export` | `[E]` | 214 | Export für Verarbeitungsverzeichnis | aus |
| `avv-generator` | `contextra-avv-generator` | `[E]` | 196 | Vorlagen für Auftragsverarbeitung (Rechtsprüfungsvorbehalt) | aus |
| `compliance` | Bündel | `[R]`/`[E]` | Summe | `privacy` + `audit-export` + `avv-generator` | aus |
| `test-utils`, `testing`, `test-hooks` | `contextra-testkit` | nur Test | — | Mocks, Fake-Uhr, Fehlerinjektion | nie im Release- oder Default-Set |

**Regeln:** Das Kern-Gate prüft, dass kein `[K]`-Crate von einem `[R]`- oder `[E]`-Crate abhängt (INV-CORE-ISOLATION-1, §4.7). Der Kern baut und testet ohne jedes dieser Features (REQ-ARCH-024).

---

## Anhang B. Archiv (nicht normativ)

Die folgenden Crates und Module sind aus dem Workspace entfernt und stehen im Git-Tag `archiv/v0` (§18.5). Sie sind nicht normativ. Eine Wiederaufnahme braucht einen ADR (§18.6) und die Evidenz des Kerns (§4.8).

| Crate oder Modul | Zeilen | Inhalt in der Vorgängerfassung | Verwandte Anforderungen |
|---|---|---|---|
| `graph` | Teil von 25.981 | PathRAG, budgetierte k-Pfad-Diffusion, Lovász-Extension (1.3 §11) | REQ-IDX-013 |
| `cognition` | — | Multi-Step-Retrieval, Decay, `MemoryType`-Taxonomie (1.3 §17) | REQ-COG-001 bis -003 |
| `adapt` | — | PID-Latenzregler, Anpassung der Datenmodelle (1.3 §5.1) | — |
| `kvcache` | — | Persistenter KV-Prefix-Cache, Mission M1, nur mit Break-even-Gate (1.3 §13) | REQ-KVC-001, -002 |
| `router` | — | Bandit-Routing SLM und LLM (1.3 §18a) | REQ-EXP-001 (offen) |
| `agent` | — | Agent-Laufzeit und Scratchpad (1.3 §18) | REQ-EXP-002 (offen) |
| `infer-onnx`, `infer-ollama` | — | alternative Embedding-Backends | INV-EMBED-1/-2 gelten für jedes Backend |
| `vector/diskann` | — | SSD-residenter Modus (1.3 §9.7) | §9.7, INV-ENCRYPT-REST-2 |
| `rank` ohne `fusion/` | Teil von 1.709 | Kalibrierung und DiBuD | REQ-IDX-014 |
| `contextra-core` | 106 | Kompatibilitätsfassade, entfällt | REQ-ARCH-013 |

Die Gesamtsumme der Archiv-Zeilen ist 25.981 (ohne `contextra-core`). Die Zeilenangaben je Modul sind in der Vorgängerfassung nicht getrennt ausgewiesen; sie bleiben offen, bis das Archiv geprüft ist.

