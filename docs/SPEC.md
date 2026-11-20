# Contextra Spezifikation v4.1 (RAG-Edition, Soll-Zustand)

**Datum:** 20. November 2026
**Status:** Final zur Umsetzung. Einzige normative Quelle für Produktumfang, Architektur, Löschkette, Betrieb, Crate-Verträge und Refactoring-Reihenfolge.
**Geltung:** Gesamter Workspace `contextra`.
**Ersetzt:** `docs/SPEC.md` v4.0 (Archiv: `docs/archive/SPEC_v4.0.md`). Die Dokumente v2.1, Löschspezifikation v2.2 und Implementierungsspezifikation v3.0 sind Quellarchiv unter `docs/archive/`.
**Kennungen:** `ADR-n` Entscheidung, `REQ-<Bereich>-n` Anforderung, `INV-n` Invariante, `REM-n` Entfernung, `CMP-n` Compat-Auflösung, `F-n` Befund, `T-n` Test, `G-n` Build-Gate. MUSS, DARF NICHT, SOLL nach RFC 2119.
**Offene Punkte:** keine. Jeder Befund aus Kapitel 0.3 hat eine Entscheidung (Kapitel 14). Die Spezifikation beantwortet alle Fragen, die für die Crate-Implementierungsspezifikationen (`crates/<name>/PLAN.md`) nötig sind.

---

## 0. Zusammenfassung

### 0.1 Grundentscheidungen

Contextra ist eine **RAG-Engine für Unternehmensdaten** mit **DSGVO-konformer Löschung personenbezogener Daten**. Sechs Grundentscheidungen tragen diese Spezifikation:

1. **Löschung und Versionierung sind getrennte Systeme** (Kapitel 4). Die Löschung arbeitet auf *Seals* (verschlüsselte Inhaltseinheiten), die Versionierung verweist nur auf Seals.
2. **Keine Eigennachweise im Produkt.** Die Löschkette führt Operationen aus und protokolliert sie. Prüfung geschieht in Tests und CI (ADR-02).
3. **Schlüssel werden sofort vernichtet, Aufräumarbeit läuft gebündelt im Hintergrund** (Kapitel 7). Ausnahme für Segmentschlüssel, die erst nach dem Neuschreiben des Segments vernichtet werden (ADR-31).
4. **Suche ist ANN-first.** HNSW mit SQ8 und exakter Nachberechnung. Exakte Suche für kleine Bestände und als Referenz (ADR-04).
5. **Zugriffsrechte, Filter und Lösch-Sichtbarkeit sind eine Bitmap-Mechanik** (Kapitel 6.5).
6. **Schichtung wird durch Ports erzwungen, nicht durch Konvention.** Kein E1-Crate kennt ein anderes E1-Crate. Verbindungen laufen über Traits in `contextra-ports` und die Verdrahtung in `contextra-db`. Kompatibilitätsschichten (`compat/`) existieren nicht mehr (ADR-51).

Zusätzlich gilt für abgeleitete Daten: **Nichts liegt unverschlüsselt auf dem Datenträger** (INV-9). Das schließt Postings, Graph-Struktur und HNSW-Struktur ein.

### 0.2 Änderungsprotokoll gegenüber v4.0

| Thema | v4.0 | v4.1 | Grund |
|---|---|---|---|
| Graph | nicht in Crate-Tabelle, Anhang mit Vorschlag | `contextra-graph` als optionaler E1-Baustein, reduziert, sealed (ADR-21 bis ADR-27) | F-01 bis F-06 |
| `contextra-types` | eigenes Crate, verschlankt | aufgelöst: IDs nach `ids`, DTOs nach `ports`, Rest zu Besitzern (ADR-33) | F-08 |
| `contextra-mvcc` | wird `contextra-version` | Umbenennung vollzogen, Inhalt auf Katalog und Snapshots reduziert | ADR-06 |
| `compat/` | v4.0: rund 5.800 Zeilen zu entfernen | Gemessen 7.465 Zeilen plus 120 bedingte Kompilierungen, vollständig aufgelöst (CMP-1 bis CMP-10) | Nutzeranforderung, F-09 |
| Schlüsselslot | 80 Byte | 128 Byte, Sektor-ausgerichtet (ADR-29) | F-12 |
| K_ref, K_registry | HKDF aus KEK | Zufallsschlüssel im Anker, nicht rotiert (ADR-40) | F-10 |
| Segmentierte Persistenz | unklar, D-Klasse „wie Quelle“ | jede abgeleitete Datei segmentweise verschlüsselt (INV-9, ADR-30) | F-11 |
| Schlüsselvernichtung | sofort für alles | sofort für Seals, Segmentschlüssel nach Neuschreiben (ADR-31) | F-11 |
| Argon2-Prüfung | Mindestlaufzeit 250 ms messen | Parametergrenzen prüfen, keine Laufzeitmessung | F-16 |
| Erase-Klasse | „einziger Betroffener ist der Antragsteller“ | „keine anderen deklarierten Betroffenen am Seal“ | F-18 |
| Scheduler-Latenz | „Suchlatenz p95“ ohne Quelle | Latenzfenster in `db` (ADR-43) | F-23 |
| Schichten | ports→types, store→crypto/erasure/mvcc, simd→types | alle Verletzungen beseitigt, G-03 verschärft | F-07 |
| Testkit im Standard-Build | `default-members` enthält testkit | nicht mehr Standard-Member | F-27 |
| Mehrfach-`MAX_SEARCH_K` | sechs Definitionen mit Werten 1.000 und 10.000 | eine Definition in `ids`, Wert 1.000 (ADR-52) | F-13 |
| Zeit | `Instant::now` verboten | verboten, 77 Stellen werden migriert (G-12) | F-26 |
| Unsafe in Tests | nicht geregelt | nur Test-Targets mit Allowlist (ADR-53) | F-28 |
| Migration | Annahme | keine Migration, Entwicklungsdaten verworfen (ADR-44) | OPEN-5 |

### 0.3 Befundregister (Architektur und Spezifikation)

Jeder Befund ist mit der Quelle belegt (Dateiname, Zeile oder Messung in Anhang B). Entscheidungen stehen in der letzten Spalte.

| ID | Ort | Befund | Entscheidung |
|---|---|---|---|
| F-01 | Crate-Tabelle 3.1 | `contextra-graph` (34.611 Zeilen, 153 Dateien, im Workspace) fehlt in der Tabelle. Die Gesamtsumme ist deshalb falsch. | ADR-21, 3.1 |
| F-02 | Anhang v4.0 „contextra-graph“ | `ConsistencyEnforcer` als „zwingend für DSGVO-Löschung“ bezeichnet. Modul-Doku sagt: Mustererkennung, keine Kantenbereinigung. | ADR-25 (Entfernung) |
| F-03 | Anhang v4.0 | `SessionBranchTree` als USP geführt, widerspricht Nicht-Ziel 1.3 und REM-24. | ADR-24 (Entfernung) |
| F-04 | `graph/src/types.rs` | `Entity.name` und `Edge.label` sind `Arc<str>` im Klartext. Persistenz über `bincode` in den Store ohne Verschlüsselung. Widerspricht REQ-FMT-2. | ADR-23 |
| F-05 | `graph/PLAN.md` v2.1 | Graph soll `ErasureLayer` aus `contextra-erasure` implementieren und Generationen unter `ArtKey` führen. Widerspricht R2 und ADR-01. | ADR-22, ADR-27 |
| F-06 | `graph/src/csr/graph_write/tombstone.rs`, `cascade.rs` | Löschen ist Soft-Delete mit Transaktionszeitstempeln. Kein Schlüssel- und kein Scrub-Bezug. Ein gelöschter Knoten bleibt physisch bis zur Kompaktion. | ADR-22, 6.10 |
| F-07 | `contextra-ports/Cargo.toml`, `store/Cargo.toml`, `simd/Cargo.toml` | Verletzungen von R1 (ports → types, 5 Fundstellen), R2 (store → crypto, 16 Fundstellen; store → mvcc und erasure), R7 (simd → types). | ADR-32, 3.3 |
| F-08 | `contextra-types` (9.596 Zeilen) | Querschnitts-Crate, aus dem Ports und Module ihre Typen beziehen. Erzeugt Kopplung ohne Besitzer. | ADR-33, REM-30 |
| F-09 | `compat/` in sieben Crates | Bridge-Schicht zwischen alten und neuen Typen, dazu Doppelmodule für denselben Kern (z. B. `vector/compat/hnsw_vector_index.rs` neben `vector/hnsw/`). Schafft zwei Codepfade, 120 bedingte Kompilierungen, `*-compat`- und `workspace`-Features. | ADR-51, Kapitel 12.5 |
| F-10 | REQ-KEY-1 / REQ-KEY-5 | `K_ref` und `K_registry` werden per HKDF aus der MandantenKEK abgeleitet. Eine Epochenrotation ändert sie, dann stimmen alle Betroffenen-Tags nicht mehr. | ADR-40 |
| F-11 | 5.1, INV-2, ADR-03 | D-Klasse „wie Quelle“ ist technisch nicht umsetzbar: Postings und Graph-Struktur mischen Seals. Schlüsselvernichtung und Rekonstruktion widersprechen sich. | ADR-30, ADR-31 |
| F-12 | REQ-KEY-3 | Slot 80 Byte. Slots kreuzen 512-Byte-Sektoren und 4-KiB-Seiten. Torn Writes sind nicht ausgeschlossen. REQ-REC-2 setzt aber Atomarität voraus. | ADR-29 |
| F-13 | Code: sechs `MAX_SEARCH_K` | Gleicher Name mit Werten 1.000 und 10.000 in sechs Crates. 86 Verwendungen. Verletzt R4. | ADR-52 |
| F-14 | REQ-KEY-7 | Argon2-Parameter und Mindestlaufzeit „250 ms“ werden beim Start gemessen. Messung ist laufzeitabhängig und widerspricht ADR-02. | ADR-16 (Neufassung) |
| F-15 | REQ-KEY-4 | Vernichtung „an Ort und Stelle“ ohne Angabe, wie Gleichzeitigkeit mit Lesern des Caches geregelt ist. | REQ-KEY-4 (neu gefasst, 5.5) |
| F-16 | REQ-KEY-7 | Siehe F-14. | ADR-16 |
| F-17 | REQ-ER-2 / 7.4 E1 | Klasse „Erase“ verlangt, dass der einzige Betroffene „der Antragsteller“ ist. Der Antragsteller ist ein Operator, kein Betroffener. Die Bedingung ist unbrauchbar. | 7.3 |
| F-18 | 7.4 E1 vs. REQ-CON-1 | Gate läuft im Schreib-Task „des Mandanten“, Schreib-Tasks sind aber pro Shard (Mandant × Collection). | ADR-28, 8.4 |
| F-19 | REQ-CON-4 | „Ingest nach Gated erhält SubjectSuppressed oder neuen Seal“ ist nicht eindeutig. | 7.4 / REQ-ING-7 |
| F-20 | 5.4 vs. 7.4 E2 | Registry-Zustand „Active“ in 5.4, „Erased“ in 7.4. Zustandsmenge undefiniert. | 5.7 |
| F-21 | 7.4, REQ-CON-6 | Gruppen-Commit und Gate-Publikation greifen nicht geordnet ineinander; Reihenfolge der Veröffentlichung fehlt. | 7.4, 8.4 |
| F-22 | REQ-SCR-4 | Scrub-Pause bei Latenzüberschreitung ohne definierte Messgröße. | ADR-43 |
| F-23 | 10.3 | „Suchlatenz p95 während Scrub ≤ +5 %“ ohne Messquelle im Produkt. Widerspricht ADR-02 (Messung im Produkt). | ADR-43 (Messung nur im Test- und Bench-Crate, Latenzfenster nur als Steuergröße) |
| F-24 | REQ-VEC-4 | Start liest alle `VectorCode`-Datensätze entschlüsselt. Bei L ~7,7 GB, Startziel 60 s. Kein Pfad für Teilladen oder Wiederverwendung. | 6.3 (Persistenz mit Graph-Snapshot) |
| F-25 | REQ-TXT-2 | `df(t)` wird beim Löschen nicht korrigiert. Ranking-Abweichung bis Scrub. Deterministisch, aber nicht exakt. | 6.4 (dokumentierte Abweichung mit Obergrenze) |
| F-26 | Code: 77 Stellen | `Instant::now`/`SystemTime::now` außerhalb `contextra-env`. Verletzt REQ-TIM-1. | G-12 |
| F-27 | `Cargo.toml` `default-members` | `contextra-testkit` und `contextra-graph` im Standard-Build, obwohl Testkit „nicht im Standard-Build“ laut 3.1. | 12.6 |
| F-28 | `tests/alloc_profiler.rs`, `graph/tests/ppr_alloc_test.rs` | `unsafe impl GlobalAlloc` außerhalb `sys`/`simd`. G-02 erlaubt das nicht. Test-Allocatoren sind ein legitimer Sonderfall. | ADR-53 |
| F-29 | REQ-REC-2 | Entscheidungsregel für halb geschriebene Slots ist mehrdeutig (Zustand `Destroyed` oder Integrity). | 7.7 (eindeutige Tabelle) |
| F-30 | REQ-ER-3 | Review-Erinnerung ohne Konsequenz bei Ablauf. Ein offener Auftrag im Zustand `Restricted` ist für Art. 17 nicht abgeschlossen, die Spezifikation sagt das nicht. | 7.5 (Pending-Frist) |
| F-31 | REQ-SRC-1 / 6.5 | Mandanten-Principals kommen laut Spezifikation aus der Anfrage. Im MCP-Kontext ist die Anfrage vom Modell beeinflussbar. | ADR-38 |
| F-32 | Spezifikation 7.2 | Erasure-Scope `Subject` ohne Berechtigungsprüfung des Anforderers spezifiziert. | ADR-38, 7.2 |
| F-33 | REQ-CTX-4 | Dedup per Textgleichheit ohne Angabe des Vergleichsverfahrens. Ein Klartext-Hash über Inhalt ist Personenbezug. | ADR-50 |
| F-34 | Text, Vektor, Graph | Abgeleitete Persistenz ohne Festlegung, wie Segment-Schlüssel verwaltet werden. Dazu fehlt ein Kopfformat für Segmente. | 5.8 |
| F-35 | 7.11 / Checkpoints | Checkpoints kopieren Datenverzeichnisse. Nicht festgelegt, ob `keys/` und `erasure/` mitgehen. Restore könnte Gate-Zustand zurücksetzen. | ADR-42 |
| F-36 | ADR-17 / REQ-FLT-3 | Zwei Schwellen mit Wert 20.000 (`flat_below` und `brute_force_max`) mit unterschiedlichem Zweck, ohne Abgrenzung. | 6.3 / 6.5 |
| F-37 | Anhang B v4.0 | 23 Workspace-Mitglieder (tatsächlich 22), Ist-Umfang 249.000 (tatsächlich 239.968 inkl. Bench), Ingest 21.000 Zeilen und 155 Dateien (tatsächlich 30.737 und 129). | Anhang B |
| F-38 | 12.2 REM-19, REM-01 | `contextra-engine` und `contextra-vault` sind bereits als Verzeichnis entfernt. Die Einträge sind veraltet. Verbleibende Verweise in Kommentaren nicht gelistet. | REM-01, REM-19 |
| F-39 | 12.2 REM-25, REM-15, REM-29 | Pfade falsch: `ports/src/ports/query_rewriter.rs` existiert nicht (richtig: `ports/src/query_rewriter.rs`), `store/src/compat` existiert nicht, `docs/crates` und Root-`AGENTS.md` fehlen. | 12 |
| F-40 | Dokumentation | Crate-Dokumente (`PLAN.md`, `INVENTAR.md`, `SPEC.md`) referenzieren unterschiedliche Spezifikationsstände (v2.1, v2.2, v3, v4). | REM-14, 13.5 |
| F-41 | `contextra-db` | `contextra-graph` ist nicht optional eingebunden, der Code im Orchestrator nutzt es nicht. Tote Abhängigkeit. | 3.3 / REM-31 |
| F-42 | Graph-Features | `default` aktiviert `experimental-graph`, `hyperedge-suggest` (LLM-Validierung im Feature `workspace`). Default-Build enthält LLM-Pfad und Agentengedächtnis. | ADR-24 |
| F-43 | `contextra-erasure` | Enthält `vault/`, `crypto_local.rs`, `types_local.rs`, Saga 2.0 mit `Armed`. Die Spezifikation v4.0 verbietet diese Typen (REM-10), der Code hat sie noch. Eine Crate-Neuaufbau-Anweisung fehlt. | 13.1 M2, 15 |
| F-44 | REQ-ING-2 | Formatliste mit `fmt-ole`, `fmt-odf`, `fmt-structured` nicht eindeutig gegen OPEN-4 abgegrenzt. | ADR-35 |

---

## 1. Produkt

### 1.1 Ziel

Contextra ist eine **eingebettete Rust-Bibliothek für Retrieval in RAG-Anwendungen** mit MCP-Schnittstelle (stdio) und optionalem Server-Binary. Sie nimmt Dokumente auf, zerlegt sie, erzeugt Embeddings über ein austauschbares Backend, indiziert Vektoren und Text, optional einen Entitätsgraph, beantwortet Hybridsuchen mit Zugriffsfilter und liefert Kontextpakete mit Quellenangabe innerhalb eines Token-Budgets. Textgenerierung liegt außerhalb.

Verbindlich:

- **Löschung personenbezogener Daten** nach Kapitel 7.
- **Mandantentrennung** mit eigener Schlüsselhierarchie je Mandant.
- **Zugriffsfilter** (Security Trimming) auf Dokument- und Chunk-Ebene.
- **Versionierung** mit optionaler Zeitreise (Kapitel 4.5).
- **Entitätsgraph** als optionaler Baustein je Collection (Kapitel 6.10).

### 1.2 Referenzlast und Referenzmodelle

| Stufe | Chunks je Mandant | Dimension | Anfragen | Betrieb |
|---|---|---|---|---|
| S | 1 Mio. | 384 | ≥ 200 QPS | 16 vCPU, 64 GB RAM, NVMe |
| L | 10 Mio. | 768 | ≥ 100 QPS | 16 vCPU, 128 GB RAM, NVMe |

Referenz-Embedding (ADR-36): `multilingual-e5-small` (384 Dimensionen) für Stufe S, `multilingual-e5-base` (768) für Stufe L, beide als ONNX mit festem SHA-256. Bis 100 Mandanten je Prozess. Größere Bestände skalieren über mehrere Prozesse, aufgeteilt nach Mandant (8.7).

### 1.3 Nicht-Ziele

- Keine Textgenerierung, kein Chat, keine Agentenlogik, **kein Agentengedächtnis** (ADR-24).
- Keine LLM-Aufrufe im Kern. Keine LLM-basierte Klassifikation, Umformulierung oder Hyperkanten-Validierung (REQ-PRV-2, ADR-24).
- Keine Echtzeit-, Regelungs- oder Hardwarefunktionen. Keine Zeitgarantien auf Suchantworten.
- Keine interaktiven Mehrfachtransaktionen. Ein Schreibstapel ist atomar (REQ-CON-6).
- Keine Replikation und kein Cluster im Kern in v4.x (ADR-46).
- Keine automatische Entitätsextraktion im Kern. Entitäten und Relationen liefert der Aufrufer (ADR-48).
- Keine Aussage über Kopien außerhalb des Systems (Kapitel 7.9).
- Keine Aussage über die rechtliche Einordnung. Die Software führt Operationen aus, bewertet nicht und garantiert kein Compliance-Ergebnis (ADR-45, REQ-DOC-2).

### 1.4 Entscheidungsfilter

Jede Funktion wird gegen drei Fragen geprüft. Fällt eine Antwort negativ aus, wird sie entfernt oder zum optionalen Feature mit Abnahmegate.

1. Hilft sie Qualität, Latenz oder Durchsatz einer Unternehmens-RAG, messbar im Benchmark?
2. Ist sie für Löschung, Mandantentrennung oder Zugriffsschutz nötig?
3. Ist ihr Mehraufwand im Normalbetrieb kleiner als ihr gemessener Nutzen?

---

## 2. Entscheidungsregister

Alle Entscheidungen sind final. Änderungen erfordern eine neue Spezifikationsversion. Die Spalte „Status“ zeigt, ob eine Entscheidung gegenüber v4.0 neu gefasst (**neu gefasst**), ergänzt (**neu**) oder unverändert (**bestätigt**) ist.

| ID | Entscheidung | Status | Begründung | Verworfen |
|---|---|---|---|---|
| ADR-01 | Löschung wirkt auf **Seals**, nicht auf Versionen. Version und Seal sind getrennte Systeme mit Einbahn-Schnittstelle. | bestätigt | Löschung darf nicht von Versionskette oder Zeitreise abhängen. | Gate-Mengen über Versionsordinale |
| ADR-02 | **Keine Eigennachweise im Produkt**: keine Selbstmessung, keine signierten Berichte, kein Prüfwerkzeug, keine Messklassen. Journal, Ledger und Tests in CI. | bestätigt | Selbstprüfung mit denselben Annahmen ist wertlos und kostet Laufzeit. | Canary im Produkt, JCS/Ed25519 |
| ADR-03 | **Frühe Schlüsselvernichtung** für Seal-Schlüssel. Reihenfolge: Gate, Schlüssel, Arbeitsspeicher, dann gebündelter Scrub. | neu gefasst | Seal-Klartext ist nur über den Seal-Schlüssel erreichbar. Ausnahme für Segmentschlüssel: ADR-31. | Reihenfolge S3–S5 aus v3.0 |
| ADR-04 | **HNSW mit SQ8** ist Standardindex. Exakte flache Suche bis 20.000 Vektoren je Shard und als Referenz im Test. DiskANN und RaBitQ entfallen. | bestätigt | Flache Suche ist ab 1 Mio. Vektoren zu langsam. | DiskANN |
| ADR-05 | **Graph-Crate wird verkleinert und als optionaler Baustein geführt.** Details ADR-21 bis ADR-27. | neu gefasst | v4.0 hatte hier keine Entscheidung (Textfragment). | Vollbehalt, Entfernen ohne Abnahmegate |
| ADR-06 | `contextra-mvcc` wird `contextra-version`: Versionskatalog, Snapshots, Aufbewahrung. Kein SSI, kein TxBuffer. | bestätigt | RAG braucht atomare Schreibstapel und Snapshot-Lesen. | SSI, interaktive Transaktionen |
| ADR-07 | **Ein Schreib-Task je Shard** (Mandant × Collection). Gate ist mandantenweit (ADR-28). Lesen sperrfrei über Snapshots. | neu gefasst | Trennt Schreibserialisierung und Gate-Konsistenz sauber. | Ein Schreiber je Prozess |
| ADR-08 | **KeyTable als Slot-Datei mit Überschreiben an Ort** (Größe 128 Byte, ADR-29). | neu gefasst | Vernichtung ist ein Schreibvorgang statt Shard-Neuschreiben. | 256 Shards mit `tmp → rename` |
| ADR-09 | **Gebündelte Bereinigung (Scrub)** gemeinsamer Strukturen mit Frist, Drosselung und Priorität unter dem Suchpfad. | bestätigt | Ein Stapel schreibt jedes betroffene Segment einmal. | Sofortige Bereinigung je Auftrag |
| ADR-10 | **Mehrbetroffene: `Restricted` mit Review** als Standard. `DeleteAll` nur durch ausdrückliche Entscheidung. | bestätigt | Fremde Dokumente ohne Zustimmung zu vernichten ist nicht in jedem Fall verhältnismäßig. | `DeleteAll` als Standard |
| ADR-11 | **Kein Cluster-Segmentieren personenbezogener Daten.** Jeder Seal hat eigenen Schlüssel. | bestätigt | Schlüssel je Seal macht Dateitrennung überflüssig. | Cluster-Segmente |
| ADR-12 | **Ingest-Crate und ONNX-Embedder aus der Historie zurückholen** (Abschnitt 13.2), beschneiden auf den Formatumfang aus ADR-35. | neu gefasst | Mengen korrigiert, siehe F-37. | Neuschreiben |
| ADR-13 | **Embedding-Backends:** lokal (ONNX, Standard) und HTTP gegen interne Dienste (eigenes Crate, Feature). Kern netzfrei. | bestätigt | Unternehmen betreiben zentrale Inferenzdienste. | Nur lokal |
| ADR-14 | **Zeitreise je Collection schaltbar, Standard aus.** Suchindizes enthalten nur die aktuelle Fassung. Gilt auch für den Graph. | bestätigt | Historie im Index vergrößert Speicher, Latenz und Löschaufwand. | Historie immer im Index |
| ADR-15 | **Schlüsselwurzel:** K1 (Passphrase, Argon2id) und K1b (Schlüsseldatei) in v4.x. K3 (KMS) in M6. | bestätigt | Passphrase passt nicht zu unbeaufsichtigtem Serverbetrieb. | TPM |
| ADR-16 | **Argon2id-Parameter** sind Konfiguration mit Mindestwerten (m ≥ 64 MiB, t ≥ 3, p ≥ 1). **Keine Laufzeitmessung beim Start.** Empfohlene Werte in der Betriebsdokumentation. | neu gefasst | Laufzeitmessung widerspricht ADR-02 und ist nicht reproduzierbar (F-14, F-16). | Mindestlaufzeit 250 ms |
| ADR-17 | **Zugriffsfilter ist Kernfunktion.** ACL-Tags und Metadatenfilter laufen über Bitmaps und die ACORN-Traversierung. | bestätigt | Ohne Security Trimming nicht einsetzbar. | Filterung in der Anwendung |
| ADR-18 | **Bulk-Ingest** darf mit gelockerter Dauerhaftigkeit laufen (Sync am Jobende). Löschvorgänge sind nie gelockert. | bestätigt | Importe von Millionen Dokumenten sind mit `fsync` je Gruppe zu langsam. | Immer volle Dauerhaftigkeit |
| ADR-19 | **Selbstdeklarationen entfallen** (`contract.rs`, `ANCHOR`, `FILE-CONTEXT`, Auditor-Skripte). Verhalten prüft `contextra-testkit`. | bestätigt | Deklarationen veralten, Tests nicht. | Deklarationen pflegen |
| ADR-20 | **Standard ist Klasse P.** Jede Collection gilt als personenbezogen, bis sie als K erklärt wird. Die Entity-Erkennung warnt, entscheidet nie. | bestätigt | Speicherweg darf nicht von Trefferquote abhängen. | Klassifikation entscheidet |
| ADR-21 | **`contextra-graph` ist ein optionaler E1-Baustein** (Feature `graph` in `contextra-db`, Standard aus, je Collection `graph = true`). Er bleibt nur bestehen, wenn Gate G-GRAPH (11.5) erreicht wird. Sonst Entfernung in M5. | neu | PathRAG liefert erklärbare Belege. Der Nutzen ist nicht gemessen, daher Abnahmegate. | Vollbehalt, sofortige Entfernung |
| ADR-22 | **Graph ist abgeleiteter Index** (Klasse D). Jede Kante und jeder Knoten trägt **Seal-Provenienz** (Menge von `SealOrd`). Löschen erfolgt über Gate, Tombstones und Scrub, nicht über eine eigene Saga. | neu | Vermeidet eine zweite Löschsaga und hält den Graph an Seals gebunden. | `ErasureLayer`-Generationen (PLAN v2.1) |
| ADR-23 | **Keine Klartext-Entitäten oder Labels im Graph.** Namen und Kantenbezeichnungen liegen als verschlüsselte Records mit dem DocKey des Quell-Seals. `EntityId = BLAKE3-keyed(K_ref, Typ ‖ normalisierter Name)`, 128 Bit. | neu | Behebt F-04. | Klartext-Labels (`Arc<str>`) |
| ADR-24 | **Entfernt aus dem Graph:** TL-HFD, Lovász, APPRH, Percolation, Dense PPR, PPR-Kostenmodell, Shadow-Hooks, `SessionBranchTree` (Agentengedächtnis), Edge-Reinforcement, Hyperkanten, LLM-Hyperkanten-Validierung, `text_gen`, Community-Erkennung, `cascade.rs`, `purger`, `provider`, `storage`, `compat/`. Liste 12.4. | neu | Widerspricht Nicht-Ziel 1.3 und Kern-Entscheidung 6. Behebt F-03, F-42. | Beibehalten |
| ADR-25 | **`ConsistencyEnforcer` entfällt.** | neu | Erkennt Muster, bereinigt nicht. Keine Löschfunktion (F-02). | Beibehalten als „DSGVO-Baustein“ |
| ADR-26 | **Graph folgt der Zeitreise der Versionen** (ADR-14). Bitemporale Felder (`tx_valid_from/to`, `business_valid_*`) entfallen. `as_of` auf Graph-Operationen liefert `HistoryNotIndexed`. | neu | Zwei Zeitmodelle widersprechen sich. | Bitemporaler Graph |
| ADR-27 | **Graph-Scrub** durch Konsolidierung der Nachbarlisten und Neuaufbau ab 30 % toter Knoten. Kein Generationsmodell mit `ArtKey`. | neu | Gleiche Mechanik wie HNSW (REQ-VEC-5). | Generation `g+1` mit `ArtKey` je Generation |
| ADR-28 | **Gate ist mandantenweit.** Eine Gate-Datei, ein `ArcSwap` je Mandant, geteilt von allen Collections. LiveMask-Änderungen gehen an die Shards. Die Leseprüfung (REQ-FLT-5) gilt sofort. | neu | Behebt F-18. Ein Löschauftrag wirkt über Collections hinweg in einem Schritt. | Gate je Shard |
| ADR-29 | **KeyTable-Slots sind 128 Byte, Sektor-ausgerichtet.** Slot-Index ist `SealOrd` bzw. `SegOrd`. | neu | 128 Byte teilt 512-Byte-Sektoren und 4-KiB-Seiten ohne Kreuzung (F-12). | 80 Byte |
| ADR-30 | **Alle abgeleiteten Dateien sind segmentweise verschlüsselt** (INV-9). Jedes Segment hat einen Segmentschlüssel `SegKey` in einem eigenen Slot-Raum der KeyTable. | neu | Behebt F-11. Keine Klartext-Postings, keine Klartext-Graphstruktur auf Platte. | Unverschlüsselte Segmente mit Gate-only |
| ADR-31 | **ADR-03 gilt für Segmentschlüssel eingeschränkt (ADR-03a):** Ein Segmentschlüssel wird vernichtet, nachdem der Scrub das Segment ohne die toten Seals neu geschrieben und veröffentlicht hat. Restfenster ≤ `scrub_deadline`, im Betriebshandbuch ausgewiesen (7.9). | neu | Ein Segment mischt lebende und tote Seals. Früher Vernichten würde lebende Daten vernichten. | Segmentschlüssel-Vernichtung sofort |
| ADR-32 | **Schichtentrennung über Ports:** `store ↛ crypto/erasure/version`, `erasure ↛ crypto`, `version ↛ erasure`. Schlüsselvernichtung über Port `KeyDestroyer` (Implementierung in `crypto`), Scrub über Port `ScrubTarget` (Implementierung in `vector`, `text`, `graph`, `store`). Orchestrierung in `db`. | neu | Behebt F-07. Macht Crate-Grenzen prüfbar (G-03). | Direkte Abhängigkeiten |
| ADR-33 | **`contextra-types` wird aufgelöst:** IDs und Ordinale nach `contextra-ids`, DTOs und Ports-Typen nach `contextra-ports::dto`, Rest zum Besitzer. | neu | Behebt F-08. Kein Querschnitts-Crate ohne Besitzer. | Verschlanken |
| ADR-34 | **Netzfreier Kern** (R8, REQ-NET-1). Netzwerkcode nur in `embed-http`, `server`, `kms`. | bestätigt | Auditierbarkeit. | — |
| ADR-35 | **Ingest-Formatumfang fest:** Text, Markdown, HTML, DOCX, PPTX, XLSX/CSV, PDF (Text, kein OCR), EML/MBOX, JSON. ODF, OLE (`.doc`, `.ppt`, `.msg`) und YAML/TOML entfallen. | neu | Behebt F-44. Formate mit Nachfrage werden in einer neuen Spezifikationsversion aufgenommen. | Nachziehen bei Bedarf |
| ADR-36 | **Embedding-Referenzmodelle:** `multilingual-e5-small` (384) Standard, `multilingual-e5-base` (768) Stufe L. Reranker `bge-reranker-v2-m3` ONNX, standardmäßig aus. Modell-SHA-256 ist verbindlich. | neu | Konkrete Referenzen für Benchmarks und Modellwechsel. | Modell offen lassen |
| ADR-37 | **Suchtexte und Abfrage-Embeddings werden nie gespeichert, geloggt oder gecacht.** `MetricsSink` erhält nur Zähler und Zeiten. | neu | Suchanfragen enthalten häufig personenbezogene Daten. | Abfrage-Log für Debugging |
| ADR-38 | **Principals und Berechtigungen kommen aus der Konfiguration des Hosts,** nie aus Tool-Argumenten oder Anfragefeldern, die ein Modell beeinflussen kann. Erasure-Anforderer müssen in der Konfiguration als `erasure_admin` geführt sein. | neu | Behebt F-31, F-32. | Principals aus Anfrage |
| ADR-39 | **Plaintext-Caches:** nur der Schlüssel-Cache (REQ-KEY-6) ist standardmäßig aktiv. Der Chunk-Text-Cache ist optional, Standardgröße 0. Ergebniscaches entfallen. | neu | Jeder Plaintext-Cache braucht einen SealScoped-Eintrag und Löschtests. Minimieren. | Ergebniscache |
| ADR-40 | **Strukturschlüssel `K_ref` und `K_registry` sind Zufallsschlüssel**, im Anker unter dem Wurzelschlüssel gewrappt, nicht per HKDF aus der KEK abgeleitet und nicht rotiert. Sie sterben nur mit dem Mandanten. | neu | Behebt F-10. | HKDF aus KEK |
| ADR-41 | **mlock-Pflicht für Schlüssel:** Der Start verweigert, wenn Schlüsselspeicher nicht gesperrt werden kann. Ausnahme `allow_unlocked_keys = true` nur für Entwicklung, im Betriebshandbuch verboten. Core-Dumps werden deaktiviert (`PR_SET_DUMPABLE = 0`). | neu | Schlüssel im Swap sind ein Lesepfad für die Vernichtung. | Warnung statt Verweigerung |
| ADR-42 | **Checkpoints schließen `keys/` und `erasure/` aus.** Schlüssel gehen nur über die Schlüsselsicherung (7.11). Der Gate-Zustand wird nie aus einem Checkpoint zurückgesetzt. | neu | Behebt F-35. | Vollständige Kopie |
| ADR-43 | **Latenzfenster in `db`:** 64 logarithmische Buckets, Schritt 1 s, Fenster 10 s, lock-frei geführt. p95 wird vom Scheduler jede Sekunde berechnet. Scrub-Pause nur über diese Größe. Messung im Produkt ist nur diese Steuergröße, kein Nachweis. | neu | Behebt F-22, F-23. | Messung in Testkit-Lasttests als Produktsignal |
| ADR-44 | **Keine Migration.** Entwicklungsdaten im Format v4.0 werden verworfen. Ein Export-Werkzeug entsteht nur mit einer neuen Spezifikationsversion und einem Pilotkunden. | neu | Schließt OPEN-5. | Migrationswerkzeug auf Vorrat |
| ADR-45 | **Keine rechtliche Bewertung im Produkt oder in der Spezifikation.** Dokumentation verwendet nur technische Aussagen (REQ-DOC-2). Die Einordnung trifft der Verantwortliche. | neu | Schließt OPEN-1. | Rechtliche Aussage in der Software |
| ADR-46 | **Hochverfügbarkeit und Replikation sind in v4.x ausgeschlossen.** | neu | Schließt OPEN-3. | Replikation des Löschlogs in v4.x |
| ADR-47 | **Erstes KMS-Backend (K3): HashiCorp Vault Transit**, selbst betrieben. Zweites Backend (AWS KMS) nur mit neuer Spezifikationsversion. | neu | Schließt OPEN-2. Passt zu on-premises-Unternehmen. | Mehrere Anbieter in M6 |
| ADR-48 | **Entitäten und Relationen liefert der Aufrufer** (`SourceDoc.graph`). Der Kern extrahiert nichts. | neu | Regelbasierte Extraktion ist fehleranfällig und gehört in die Quellsysteme. | Regelbasierte Extraktion im Kern |
| ADR-49 | **Hybridfusion-Gewichte:** Vektor 1,0, Text 1,0, Graph 0,0 (nur aktiv bei `GraphMode::PathRag`). RRF mit `k = 60`. | neu | Festlegung vor Benchmark, Änderung nur über Konfiguration. | Gewichte aus Trainingsdaten |
| ADR-50 | **Dedup und Idempotenz über keyed Digest:** `digest = BLAKE3-keyed(K_ref, Quellbytes)`. Kein unkeyed Hash über Inhalte. | neu | Behebt F-33. Unkeyed Hashes von Personendaten sind Personenbezug. | Textvergleich |
| ADR-51 | **`compat/` wird vollständig aufgelöst.** Keine Bridge-Module, keine Doppelmodule, keine `*-compat`-, `workspace`- oder `types-compat`-Features. Ports werden direkt von Kernmodulen implementiert. Umfang und Zielpositionen: 12.5. | neu | Nutzeranforderung. Behebt F-09. | Bridge-Schicht behalten |
| ADR-52 | **`MAX_SEARCH_K = 1_000`**, einzige Definition in `contextra-ids`. `k_cand` und `rerank_k` sind daraus abgeleitet und ebenfalls begrenzt. | neu | Behebt F-13. Ein Wert, eine Quelle. | Sechs abweichende Werte |
| ADR-53 | **`unsafe`-Inseln:** `sys` und `simd` im Produktcode. Test-Targets dürfen einen globalen Test-Allocator mit `unsafe` enthalten. Liste in `tests/unsafe_islands.rs`, sonst verboten. | neu | Behebt F-28. | Verbot ohne Ausnahme |
| ADR-54 | **Ein Crate, eine Ebene, eine Verantwortung.** Kein E1-Crate hängt von einem anderen E1-Crate ab (R2). E2-Crates hängen nur von E0, E1 und den E2-Crates nach 3.3 ab, ohne Zyklus. Kein Crate führt einen Feature-Schalter, der einen zweiten Codepfad für denselben Kern einschaltet. | neu | Verhindert die Entstehung neuer Bridge-Schichten. | Feature-Varianten |

---

## 3. Architektur

### 3.1 Zielbestand der Crates

Umfang alt = Zeilen laut Messung am HEAD `2fd0994` (inkl. Tests, Anhang B). Umfang Ziel ist eine **Schätzung** (±25 %).

| Crate | Ebene | Entscheidung | Alt | Ziel | Hinweis |
|---|---|---|---|---|---|
| `contextra-ids` | E0 | behalten, Typen aus `types` aufnehmen, einzige Definition von Limits | 1.484 | 2.000 | ADR-33, ADR-52 |
| `contextra-env` | E0 | behalten (`Clock`, `SeededRng`, `MetricsSink`) | 2.658 | 2.000 | R10 |
| `contextra-sys` | E0 | behalten (unsafe-Insel: mlock, Direct-IO, Dumpable) | 2.987 | 2.500 | ADR-41 |
| `contextra-simd` | E0 | behalten (unsafe-Insel), Abhängigkeit auf `types` entfernt, `compat/` aufgelöst | 6.461 | 3.500 | CMP-7 |
| `contextra-durable-fs` | E0 | behalten, Fault-Injection und `Poisoned` raus | 3.491 | 2.500 | REM-13 |
| `contextra-ports` | E0 | Traits und DTOs, nur Abhängigkeit `ids` | 7.208 | 3.000 | ADR-32, ADR-33 |
| `contextra-crypto` | E1 | AEAD, KDF, KeyTable, Anker, `RecordSealer`, `SegmentSealer`, `KeyDestroyer`, Wurzel-Backends | 11.953 | 4.000 | Kapitel 5 |
| `contextra-erasure` | E1 | Löschkette, Gate, Log, Ticket, Scrub-Planung. Neu aufgebaut | 6.708 | 4.500 | Kapitel 7, REM-10 |
| `contextra-version` | E1 | aus `contextra-mvcc`. Versionskatalog, Snapshots, Aufbewahrung | 8.159 | 2.500 | ADR-06 |
| `contextra-store` | E1 | LSM, WAL, SST, Manifest, KV-Familien. Keine Krypto-Abhängigkeit | 35.383 | 14.000 | G-STORE, CMP-4 |
| `contextra-vector` | E1 | HNSW, SQ8, exakt, ACORN, Filter. DiskANN, RaBitQ raus | 34.675 | 11.000 | ADR-04 |
| `contextra-text` | E1 | BM25, WAND, Tokenizer, Stemmer, Stoppwörter | 18.176 | 10.000 | 6.4 |
| `contextra-graph` | E1 | Optional. Reduzierter CSR-Graph mit Seal-Provenienz und PathRAG | 34.611 | 6.000 | ADR-21 bis ADR-27 |
| `contextra-rank` | E1 | RRF, Top-k, Zeitgewichtung, Erklärung | 10.809 | 4.000 | REM-18 |
| `contextra-privacy` | E1 | Entity-Erkennung (Warnung), Egress-Guard, Verarbeitungsregister | 9.278 | 3.500 | REM-20 |
| `contextra-checkpoint` | E1 | Hartlink-Snapshot und Restore, `keys/` und `erasure/` ausgeschlossen | 10.901 | 3.000 | ADR-42 |
| `contextra-ingest` | E2 | aus Historie, beschnitten auf ADR-35 | – | 14.000 | ADR-12, 13.2 |
| `contextra-embed` | E2 | ONNX-Embedder und Reranker-Adapter | – | 2.500 | ADR-36 |
| `contextra-db` | E2 | Orchestrierung: Schreibpfad, Lesepfad, Scheduler, Wiederanlauf, Verdrahtung der Ports | 412 | 6.000 | 13.1 M2 |
| `contextra` | E2 | Fassade, öffentliche API, Konfigurationsladen | 3.807 | 1.500 | 6.8 |
| `contextra-mcp` | E3 | MCP über stdio | 7.815 | 2.500 | ADR-38 |
| `contextra-testkit` | E3 | Prüfwerkzeuge für Tests (FaultVfs, Canary-Scanner, Mocks). Nur Dev-Abhängigkeit | 7.389 | 7.000 | F-27 |
| `contextra-bench` | E3 | Benchmarks, Budgets, Basisvergleich. Verzeichnis `benchmarks/contextra-bench/` | 5.199 | 3.000 | 10.4 |
| `contextra-embed-http` | E3 | HTTP-Embedder, Feature-Crate | – | 1.200 | ADR-13 |
| `contextra-server` | E3 | Optional, HTTP/gRPC mit Auth (M6) | – | 2.500 | REQ-SRV-1 |
| `contextra-kms` | E3 | Optional, Vault-Transit-Adapter (M6) | – | 1.000 | ADR-47 |

Entfernt: `contextra-types` (REM-30), `contextra-engine` und `contextra-vault` (bereits nicht mehr vorhanden, REM-01, REM-19). Umbenannt: `contextra-mvcc` zu `contextra-version`.

**Summe (Zielspalte):** Produktive Crates ohne `server` und `kms` rund 105.700 Zeilen. Mit `server` und `kms` rund 109.200. Testkit und Bench zusätzlich rund 10.000, gesamt rund 119.200. Gegenüber dem Ist-Stand von 239.968 Zeilen (Anhang B) sind das rund 50 % weniger (±25 %).

### 3.2 Ebenen und Regeln

```
E3  mcp  embed-http  server  kms  testkit(dev)  bench(bench)
E2  contextra (Fassade) ── db ──┬── ingest   (keine internen Abhängigkeiten)
                                 └── embed
E1  crypto  erasure  version  store  vector  text  graph(opt)  rank  privacy  checkpoint
E0  ports  ids  env  sys  simd  durable-fs
```

- **R1** Abhängigkeiten zeigen abwärts. `ports` hängt nur von `ids` ab.
- **R2** E1-Crates hängen nie von anderen E1-Crates ab. E2-Crates hängen nur von E0, E1 und anderen E2-Crates ab, ohne Zyklus (siehe 3.3).
- **R3** `default = []` in jedem Crate. Ausnahme: `contextra-durable-fs`, `contextra-simd` und `contextra-rank` (Feature `std`), die in 12.6 begründet sind. Optionale Funktionen sind Features mit einer Zeile Dokumentation.
- **R4** Jedes Konzept genau einmal definiert. Kryptoprimitive nur in `contextra-crypto`. Limits nur in `contextra-ids`.
- **R5** `#![forbid(unsafe_code)]` in jedem Crate außer `sys` und `simd`. Test-Targets nach ADR-53.
- **R6** Kein `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!` und keine Indexierung mit `[]` auf nicht geprüften Indizes außerhalb von Tests. Clippy-Lints auf `deny`.
- **R7** Keine `compat`-Module, keine Features mit `compat`, `workspace` oder `types-compat` im Namen (ADR-51, ADR-54).
- **R8** Kein Netzwerk im Kern (REQ-NET-1).
- **R9** Ein Port-Trait wird genau in `contextra-ports` definiert. Implementiert wird es genau in dem Crate, das den Zustand besitzt (3.4).
- **R10** Zeit nur über `Clock` aus `contextra-env`. Zufall nur über `OsRng` (Schlüssel, Nonces, IDs) oder `SeededRng` (Tests und Benchmarks).

**REQ-NET-1.** Der Abhängigkeitsbaum der Kern-Crates (E0 bis E2 ohne `embed-http`) enthält keine Netzwerk-Crates (Sperrliste G-05). Adapter-Crates dürfen Netzwerk nutzen, nur hinter Features, nie als Default. Jeder ausgehende Endpunkt steht im Verarbeitungsregister (REQ-PRV-4) und läuft durch den Egress-Guard (REQ-PRV-3).

### 3.3 Erlaubte Abhängigkeiten (normativ, geprüft in G-03)

Die vollständige Matrix steht maschinenlesbar in Anhang E. Kurzfassung:

| Crate | Erlaubte interne Abhängigkeiten |
|---|---|
| `ids` | – |
| `env` | `ids` |
| `sys` | – |
| `simd` | – |
| `durable-fs` | `env`, `ids`, `sys` |
| `ports` | `ids` |
| `crypto` | `ports`, `ids`, `env`, `durable-fs` |
| `erasure` | `ports`, `ids`, `env`, `durable-fs` |
| `version` | `ports`, `ids`, `env` |
| `store` | `ports`, `ids`, `env`, `sys`, `durable-fs` |
| `vector` | `ports`, `ids`, `env`, `simd`, `durable-fs` |
| `text` | `ports`, `ids`, `env`, `durable-fs` |
| `graph` | `ports`, `ids`, `env`, `durable-fs` |
| `rank` | `ports`, `ids` |
| `privacy` | `ids`, `env` |
| `checkpoint` | `ports`, `ids`, `env`, `durable-fs` |
| `ingest` | – |
| `embed` | `ports`, `ids` |
| `db` | alle E1-Crates, `ports`, `ids`, `env`, `ingest`, `embed` |
| `contextra` | `db`, `ports`, `ids`, `privacy` |
| `mcp` | `contextra`, `ports`, `privacy` |
| `embed-http` | `ports` |
| `kms` | `ports` |
| `server` | `contextra`, `ports` |
| `testkit` (dev) | alle außer `bench`, `server`, `kms` |
| `bench` (bench) | `contextra`, `db`, `vector`, `text`, `store`, `rank`, `graph` |

`contextra-db` führt `graph` und `text` nur über Feature-Schalter (siehe Kapitel 9). Ohne Feature `graph` kompiliert `db` ohne Graph-Code (T-GR-12).

### 3.4 Besitz der Zustände und Port-Implementierungen

| Zustand | Besitzer (Crate) | Port, den das Crate implementiert |
|---|---|---|
| Gate je Mandant | `erasure` | `ReadGate` |
| Versionskatalog, Sichtbarkeit | `version` | `Visibility`, `ContentLifecycle` (Sender) |
| Schlüsseltabelle, Anker | `crypto` | `RecordSealer`, `SegmentSealer`, `KeyDestroyer`, `RootKeyProvider` |
| Seal-Registry | `store` (KV-Familie), Logik in `erasure` | `KvStore` (store), Registry-Port in `erasure` |
| Löschlog, Ledger | `erasure` | `ErasureSink` (optionale Senke) |
| LSM, WAL, SST | `store` | `KvStore`, `SegmentStore` |
| HNSW, SQ8 | `vector` | `VectorIndex`, `ScrubTarget` |
| Postings | `text` | `TextIndex`, `ScrubTarget` |
| Entitätsgraph | `graph` | `GraphIndex`, `ScrubTarget` |
| Checkpoints | `checkpoint` | `Checkpointer` |
| Embedding | `embed` | `EmbeddingProvider`, `Reranker` |

Die Verdrahtung erfolgt in `contextra-db` (13.1 M2). Kein anderes Crate kennt andere Besitzer.

### 3.5 Verzeichnisstruktur der Daten

```
<data_dir>/
  LOCK                                   Prozesssperre
  tenants/<tenant>/
    keys/keytable.cxkt                   Schlüsseltabelle, Seal- und Segment-Raum (5.5)
    keys/anchor.cxan                     gewrappte KEK-Epochen, K_ref, K_registry (5.6)
    erasure/erasure.cxel                 Löschlog (7.8)
    erasure/gate.cxgs                    Gate-Snapshot, atomar ersetzt (7.8)
    erasure/ledger.jsonl                 Ledger, nur anhängen (7.10)
    registry/                            Seal-Registry, KV-Familie im Store (5.7)
    collections/<collection>/
      store/                             LSM, WAL, SST, Manifest
      vector/                            Graphstruktur, Deltas, Header, Segmente (verschlüsselt)
      text/                              Postings-Segmente (verschlüsselt)
      graph/                             optional, Segmente (verschlüsselt)
      versions/                          Versionskatalog
```

Ein Mandant ist ein Verzeichnis und ein Schlüsselsatz. Mandanten-Offboarding ist O(1) in der Schlüsselarbeit (7.2).

### 3.6 Repository-Struktur je Crate

```
crates/<name>/
  Cargo.toml                 Features nach R3, keine compat-Features
  README.md                  eine Seite: Zweck, öffentliche API, Verweis auf PLAN.md
  PLAN.md                    Implementierungsspezifikation nach Vorlage 13.5
  src/                       Module nach Kapitel 15, keine Bridge-Module
  tests/                     Integrationstests, Fixtures in tests/fixtures/
```

Entfallen: `INVENTAR.md`, `SPEC.md`, `AGENTS.md` pro Crate, `contract.rs`, `clippy.toml` pro Crate (Lints kommen aus dem Workspace), `proptest-regressions`-Ordner außerhalb von `tests/` (REM-14).

---

## 4. Entkopplung von Löschung und Versionierung

### 4.1 Problem im Altbestand

In v2.1 bis v4.0 hängen beide Systeme aneinander: Der Schlüssel gehörte zur Version, das Gate führte Mengen über Versionsordinale, die Zeitreise brauchte eine Sonderregel, Lineage trug `source_version`, Löschen verlangte das Auflösen der Versionskette. Jede Änderung an einem System berührte das andere. Der Graph-Entwurf (PLAN v2.1) hätte ein drittes System mit eigener Generationskette hinzugefügt (F-05).

### 4.2 Das Seal-Modell

Ein **Seal** ist eine unveränderliche Inhaltseinheit einer Dokumentrevision: Chunks, Texte, Vektoren, Original, Graph-Labels. Er hat:

- eine dichte, nie wiederverwendete Ordinalzahl `SealOrd(u32)` je Mandant,
- einen eigenen Zufallsschlüssel `DocKey` (32 Byte) im Schlüsselraum Seal,
- einen Eintrag in der **Seal-Registry** (5.7) mit Zustand, Dokument, Collection, Chunk-Slots, Betroffenen-Tags und Klasse.

Die Versionierung speichert pro Revision nur einen undurchsichtigen `ContentRef = SealOrd`. Sie öffnet ihn nie.

| Frage | Besitzer |
|---|---|
| Welche Fassung gilt zum Zeitpunkt T? Wie lange wird sie aufbewahrt? | `contextra-version` |
| Ist dieser Inhalt lesbar, eingeschränkt oder vernichtet? Wo liegt sein Schlüssel? | `contextra-erasure` (Zustand), `contextra-crypto` (Schlüssel) |
| Welche Seals gehören zu einem Dokument oder Betroffenen? | Seal-Registry, Logik in `erasure` |

### 4.3 Einbahn-Schnittstelle

```
Version ──(ContentRetired)──► Erasure         Aufbewahrung abgelaufen: Seal erledigen
Erasure ──────────────────✗──► Version        keine Rückwirkung, nie
```

- Wenn die Versions-GC eine Revision entfernt und kein anderes Dokument den Seal nutzt, sendet sie `ContentRetired(seal, RetireReason::RetentionExpired)`. Die Löschseite vernichtet den Seal mit Grund `Retention` im Ledger (Ablauf wie 7.4 ab E1).
- Eine Löschung nach Kapitel 7 ändert den **Versionskatalog nicht**. Revisionen mit vernichtetem Seal bleiben als Metadaten (Zeitstempel, Revisionsnummer). Ihr Inhalt liefert `Erased`.
- Die Versionierung kennt keine Schlüssel, kein Gate, keine Betroffenen und keine Saga.

### 4.4 Traits (in `contextra-ports`, Pfad `ports::erasure`, `ports::version`, `ports::crypto`)

```rust
pub struct SealOrd(pub u32);
pub struct ChunkSlot(pub u32);
pub struct SegOrd(pub u32);

pub enum SealState { Open, Restricted, Erased }          // Lesesicht, abgeleitet aus Registry und Gate
pub enum RetireReason { RetentionExpired }

/// Mandantenweite Lesesperre. Besitzer: contextra-erasure. Lesen ist O(1), keine E/A.
pub trait ReadGate: Send + Sync {
    fn state(&self, s: SealOrd) -> SealState;
    fn seq(&self) -> u64;                                 // wächst bei jeder Gate-Änderung
}

/// Sichtbarkeit nach Versionsstand. Besitzer: contextra-version.
pub trait Visibility: Send + Sync {
    fn is_current(&self, c: ChunkSlot) -> bool;
    fn visible_at(&self, c: ChunkSlot, seq: u64) -> bool;  // nur bei index_history = true
}

/// Verschlüsselung je Seal. Besitzer: contextra-crypto.
pub trait RecordSealer: Send + Sync {
    fn seal(&self, s: SealOrd, kind: RecordKind, aad: &[u8], pt: &[u8], out: &mut Vec<u8>) -> Result<(), ContextraError>;
    fn open(&self, s: SealOrd, kind: RecordKind, aad: &[u8], ct: &[u8], out: &mut Vec<u8>) -> Result<(), OpenError>;
}

/// Verschlüsselung je Segment (ADR-30). Besitzer: contextra-crypto.
pub trait SegmentSealer: Send + Sync {
    fn new_segment(&self) -> Result<SegOrd, ContextraError>;
    fn seal_chunk(&self, seg: SegOrd, chunk_index: u32, pt: &[u8], out: &mut Vec<u8>) -> Result<(), ContextraError>;
    fn open_chunk(&self, seg: SegOrd, chunk_index: u32, ct: &[u8], out: &mut Vec<u8>) -> Result<(), OpenError>;
}

/// Schlüsselvernichtung. Besitzer: contextra-crypto. Batch mit einem fdatasync (REQ-KEY-4).
pub trait KeyDestroyer: Send + Sync {
    fn destroy_seals(&self, seals: &[SealOrd]) -> Result<(), ContextraError>;
    fn destroy_segments(&self, segs: &[SegOrd]) -> Result<(), ContextraError>;
    fn destroy_tenant(&self) -> Result<(), ContextraError>; // MandantenKEK aller Epochen und Anker-Eintrag
}

pub enum OpenError { KeyAbsent, Integrity }   // beide werden auf dem Lesepfad zu Erased bzw. Fehler ohne Unterscheidung (INV-6)

/// Einbahn-Ereignis Version → Löschung. Besitzer: contextra-version (Sender), Empfang in db.
pub trait ContentLifecycle: Send + Sync {
    fn content_retired(&self, s: SealOrd, why: RetireReason);
}

/// Zwischenspeicher mit Klartext oder abgeleitetem Inhalt müssen Seals entfernen (ADR-39).
pub trait SealScoped: Send + Sync {
    fn invalidate_seals(&self, seals: &roaring::RoaringBitmap);
}

/// Scrub-Ziel. Besitzer: jeweils das Crate mit dem Zustand (vector, text, graph, store).
pub trait ScrubTarget: Send + Sync {
    fn name(&self) -> &'static str;
    fn plan(&self, dead: &roaring::RoaringBitmap) -> Result<ScrubPlan, ContextraError>;
    fn execute(&self, plan: &ScrubPlan, io: &mut IoBudget) -> Result<ScrubProgress, ContextraError>;
}
```

`ScrubPlan`, `ScrubProgress`, `IoBudget` sind Werttypen in `ports::erasure`. `ContextraError` ist die einzige Fehlerart der öffentlichen API (6.8).

### 4.5 Versionierung

**REQ-VER-1 (Katalog).** `contextra-version` führt je Dokument eine Kette `(rev, seq, valid_from, valid_to, content_ref, state)`. `state` ist `Current`, `Superseded` oder `Retired`.

**REQ-VER-2 (Snapshots).** Lesen läuft gegen einen Snapshot mit Sequenznummer. Das Anheften verhindert die GC der Fassungen, die der Snapshot sieht. Es gibt keine Schreibtransaktionen, nur den atomaren Schreibstapel (REQ-CON-6).

**REQ-VER-3 (Aufbewahrung je Collection).** `version_retention`: `LastN(n)` mit Standard `n = 3`, `Days(d)` oder `KeepAll`. Ausgelaufene Revisionen werden entfernt. Der Seal bekommt `ContentRetired`, sobald keine andere Revision ihn nutzt.

**REQ-VER-4 (Löschen im Normalbetrieb).** `delete_document(doc)` setzt die aktuelle Revision auf `Retired`. Die Collection-Richtlinie `on_delete` ist `Retain` (Standard, Inhalt bleibt bis Aufbewahrungsende) oder `Shred` (sofort `ContentRetired`). Das ist **keine** Löschung nach Kapitel 7. Eine Löschung nach Kapitel 7 erfasst alle Seals des Dokuments, auch retained (Registry, 4.2).

**REQ-VER-5 (Zeitreise, ADR-14).**
- `index_history = false` (Standard): Suchindizes enthalten nur `Current`. Wechsel auf eine neue Revision setzt die Bits der alten Chunks in `Visibility` auf nicht current. Das ist kein Löschvorgang und erzeugt keinen Scrub. `as_of` bei `get` liest die Revision aus dem Store. `as_of` bei `search` wird mit `HistoryNotIndexed` abgewiesen.
- `index_history = true`: Chunks tragen `seq_from, seq_to`. `Visibility::visible_at` filtert. Speicher und Latenz steigen mit der Revisionszahl, Benchmark weist das aus.
- `valid_at` ist ein Bereichsfilter auf Attribute und von `as_of` unabhängig (6.5).

**REQ-VER-6 (Zeitreise und Löschung).** Die Zeitreise prüft das Gate nach Seal gegen den **aktuellen** Stand. Die Version wählt aus, das Gate gibt frei. Ein gelöschter Seal liefert `Erased`.

### 4.6 Zusammensetzung im Lesepfad

```
live(Anfrage) = visible(Version) ∩ open(Gate) ∩ acl(Principal) ∩ meta(Filter)
```

Alle vier sind Bitmaps über Chunk-Slots oder Seal-Ordinale. Die Gate-Prüfung ist zusätzlich am Materialisierungspunkt (REQ-FLT-5). Die Löschung fügt keinen eigenen Pfad hinzu (REQ-SRC-3).

**INV-1.** Kein Crate außer `db` kennt gleichzeitig `ReadGate` und `Visibility` als Implementierung.

### 4.7 Mandantenweites Gate (ADR-28)

Das Gate gehört dem Mandanten, nicht der Collection. Ein Löschauftrag mit Scope `Subject` oder `Tenant` wirkt in einem Schritt über alle Collections. Die Reihenfolge ist:

1. Gate-Änderung ist durabel (`fdatasync` des Löschlogs und der Gate-Datei), dann `ArcSwap`-Publikation. Ab hier liefert `ReadGate::state` `Erased`.
2. Jede Collection erhält die LiveMask-Änderung über ihren Schreib-Task. Das ist nur Optimierung, Korrektheit hängt an Schritt 1 und der Materialisierungsprüfung (REQ-FLT-5).

---

## 5. Datenmodell und Speicher

### 5.1 Datenklassen

| Klasse | Inhalt | Verschlüsselung | Löschmechanismus |
|---|---|---|---|
| **P** | Dokumente, Chunks, Vektoren, Text, Graph-Labels einer Collection mit `personal_data = Possible` (Standard) | `DocKey` je Seal (Nutzdaten), `SegKey` je Segment (abgeleitete Dateien) | Gate, Schlüsselvernichtung (7.4), Scrub |
| **K** | Collection mit `personal_data = None`, vom Betreiber erklärt | `SegKey` je Segment; Klartext nur mit `k_class_encryption = false` | Gate, Scrub durch Segmentersatz |
| **D** | Abgeleitet: Postings, Graphstruktur, HNSW-Struktur, Caches | `SegKey` je Segment (INV-9), Caches nur im Speicher | Gate, Scrub, `SealScoped` |
| **E** | Löschlog, Ledger, Schlüsseltabelle, Anker, Registry | eigene Schlüssel (5.4), Ledger ohne Schlüssel | Aufbewahrungsfrist des Ledgers |

**INV-9 (Keine Klartext-Ableitung).** Keine Datei unter `tenants/` enthält bei Klasse P oder D Klartext von Nutzdaten, Tokens, Namen oder Graphlabels. Test T-ENC-01 durchsucht das Datenverzeichnis nach Canaries (Testkit).

**ADR-20 (Standard ist P).** Der Standard verlässt sich **nicht** auf die Erkennung personenbezogener Daten. Die Erkennung (REQ-PRV-1) warnt bei Treffern in K-Collections und entscheidet nie über den Speicherweg.

### 5.2 Datensatzarten

| Art | Klasse | Inhalt | Verschlüsselung |
|---|---|---|---|
| `DocBlob` | P | Originaldokument | `DocKey` (Seal) |
| `ChunkText` | P | Text und Provenienz je Chunk | `DocKey` (Seal) |
| `VectorFull` | P | fp32-Vektor je Chunk | `DocKey` (Seal) |
| `SealedMeta` | P | `external_id`, Quellpfad, Metadaten außer Filterfeldern | `DocKey` (Seal) |
| `GraphLabel` | P | Entitätsname, Relationsbezeichnung | `DocKey` des Quell-Seals (ADR-23) |
| `VectorCode` | D | SQ8-Code je Chunk im Segment | `SegKey` (Segment) |
| `Postings` | D | BM25-Postings je Segment | `SegKey` (Segment) |
| `GraphStruct` | D | Knoten-Slots, Kanten, Seal-Provenienz als Ordinale | `SegKey` (Segment) |

`VectorCode` ist eine abgeleitete Datei (D), kein Seal-Datensatz. Der Code-Inhalt ist eine Funktion des fp32-Vektors. Bei einem Scrub wird er neu aus `VectorFull` berechnet, solange der Seal lebt, oder aus dem Segment entfernt (6.3).

### 5.3 Datensatz-Umschlag für Seal-Datensätze

Speicherwert im KV-Store (Schlüssel `(seal: u32 BE, kind: u8, chunk: u32 BE)`):

```
[ver u8 = 1][kind u8][chunk u32 LE][nonce 12][ciphertext n][tag 16]
```

- AEAD: AES-256-GCM, Nonce 96 Bit zufällig aus `OsRng`.
- AD = `tenant_id (16) ‖ collection (u32 LE) ‖ seal (u32 LE) ‖ kind (u8) ‖ chunk (u32 LE) ‖ ver (u8)`.
- Obergrenze je `DocKey`: `max_records_per_seal = 2^24`. Überschreitung ist `Refused` für den Schreibstapel, der Seal wird nicht erweitert, sondern neue Revision wird angelegt.
- Der Schlüssel `seal` steht im Store-Schlüssel, nicht im Umschlag, damit die Registry-Zuordnung ohne Entschlüsselung funktioniert.

**REQ-FMT-2 (kein Klartext in Metadaten).** Quellpfade, Dateinamen, `external_id` und Betroffenen-Kennungen liegen nur in `SealedMeta` bzw. als keyed Hash. Der Zugriff über `external_id` läuft über `BLAKE3-keyed(K_ref, external_id)`. Klartextmetadaten sind auf die deklarierten Filterfelder (6.5) begrenzt. Der Betreiber muss sie als nicht personenbezogen deklarieren, die Erkennung prüft das als Warnung.

### 5.4 Schlüsselhierarchie

```
Wurzel (K1 Passphrase / K1b Schlüsseldatei / K3 Vault Transit)
  └─ Anker (keys/anchor.cxan), gewrappt unter Wurzelschlüssel
       ├─ MandantenKEK[epoche]        zufällig, Rotation nach epoch_len
       │     ├─ DocKey je Seal         zufällig, in KeyTable gewrappt (Seal-Raum)
       │     └─ SegKey je Segment      zufällig, in KeyTable gewrappt (Segment-Raum)
       ├─ K_ref                        zufällig, nicht rotiert (ADR-40)
       └─ K_registry                   zufällig, nicht rotiert (ADR-40)
```

**REQ-KEY-1.** `DocKey`, `SegKey`, `MandantenKEK`, `K_ref`, `K_registry` kommen aus `OsRng` und werden nie aus anderen Schlüsseln abgeleitet. Fehler der Zufallsquelle führen zu `Err`. HKDF ist nur für Schlüssel aus dem Wurzelschlüssel zulässig, zum Beispiel für die Trennung von Anker und Schlüsselsicherung.

**REQ-KEY-2 (Wrap).** AES-256-GCM-SIV unter dem jeweiligen Umschlagschlüssel. Nonce 96 Bit aus `OsRng`. AD = `raum (u8) ‖ index (u32 LE) ‖ epoche (u32 LE) ‖ algo_id (u8)`.

**REQ-KEY-7 (Wurzel).** Das Trait `RootKeyProvider` (Port) liefert den Wurzelschlüssel für die Anker-Entwrappung. Implementierungen: `Passphrase` (Argon2id, Parameter nach ADR-16), `KeyFile` (Rechte 0600 geprüft, Inhalt 32 Byte), `VaultTransit` (M6). Der Start verweigert (`Refused`) bei leerer Passphrase, Parametern unter den Mindestwerten, falschen Dateirechten, nicht gesperrtem Speicher (ADR-41).

**REQ-KEY-8 (Wurzelrotation).** `contextra key rewrap-root` schreibt nur den Anker neu (atomar, 5.6). Keine Schlüssel in der KeyTable werden berührt.

### 5.5 KeyTable (`keys/keytable.cxkt`)

Zwei Räume in einer Datei. Der Seal-Raum ist indiziert mit `SealOrd`, der Segment-Raum mit `SegOrd`. Beide Räume haben feste Basisoffsets im Header.

**Header (4096 Byte, Sektor- und Seitengröße):**

| Offset | Länge | Feld |
|---|---|---|
| 0 | 4 | Magic `CXKT` |
| 4 | 2 | Formatversion (1) |
| 6 | 2 | Slot-Größe (128) |
| 8 | 8 | Basisoffset Seal-Raum |
| 16 | 8 | Basisoffset Segment-Raum |
| 24 | 4 | Aktuelle Epoche |
| 28 | 4 | CRC32C über Bytes 0–27 |
| 32 | 4064 | reserviert (0) |

**Slot (128 Byte, ausgerichtet auf Offsets ganzzahlig teilbar durch 128):**

| Offset | Länge | Feld |
|---|---|---|
| 0 | 1 | `state`: 0 Frei, 1 Live, 2 Destroyed |
| 1 | 1 | `flags` (0) |
| 2 | 2 | reserviert |
| 4 | 4 | `epoche` (LE) |
| 8 | 12 | `wrap_nonce` |
| 20 | 48 | `wrapped` (32 Byte Schlüssel + 16 Byte Tag) |
| 68 | 4 | `crc32c` über Bytes 0–67 |
| 72 | 56 | reserviert (0) |

**REQ-KEY-3.** Slot-Index = `SealOrd` (Seal-Raum) bzw. `SegOrd` (Segment-Raum). Slots werden nie wiederverwendet, nur durch Epochen-Neuaufbau der Datei (REQ-KEY-5).

**REQ-KEY-4 (Vernichtung, neu gefasst zu F-15).** Ablauf eines Vernichtungsstapels:
1. Der Aufrufer hat das Gate bereits publiziert (7.4 E1).
2. `KeyDestroyer` nimmt den **Schreib-Lock des Schlüssel-Caches**. Neue Leser warten, laufende Leser (Read-Lock) schließen ab. Dadurch ist kein Schlüssel im Cache, dessen Slot gleich verschwindet.
3. Cache-Einträge der Seals werden entfernt, `SealScoped` wird aufgerufen.
4. Slots werden per `pwrite` an Ort überschrieben: `wrapped` und `wrap_nonce` mit Nullen, `state = 2`, CRC neu. Die Slots sind nach Offset sortiert, alle Schreibvorgänge eines Stapels teilen einen `fdatasync`.
5. Cache-Lock wird freigegeben. Alle gehaltenen Kopien im Speicher werden mit `zeroize` gelöscht.

**REQ-KEY-5 (Epochenwechsel).** Alle `epoch_len` (Standard 30 Tage) entsteht eine neue MandantenKEK. Ein Hintergrundlauf liest alle `Live`-Slots sequentiell und wrappt sie um. Gemischte Epochen sind zulässig. Wenn kein Slot mehr die alte Epoche trägt, wird die alte KEK aus dem Anker entfernt. Aufwand O(Seals + Segmente), sequentielle E/A. Bei 10 Mio. Seals rund 1,3 GB je Lauf (128-Byte-Slots). Das begrenzt den Zeitraum, in dem eine verbliebene Kopie eines gewrappten Schlüssels nutzbar wäre (Grenze für Copy-on-Write und Flash, 7.9).

**REQ-KEY-6 (Schlüssel-Cache).** Entwrappte `DocKey` und `SegKey` liegen in einem LRU-Cache (`key_cache_entries`, Standard 65.536) in gesperrtem Speicher (mlock). Kein `Clone`, `Debug`, `Serialize`. `Zeroizing` beim Verwerfen.

### 5.6 Anker (`keys/anchor.cxan`)

```
Header:  Magic "CXAN" (4) | Version u16 | Tenant-ID (16) | Anzahl Einträge u32 | CRC32C u32
Eintrag: Art u8 (1 KEK, 2 K_ref, 3 K_registry) | Epoche u32 | Nonce 12 | Länge u16 | gewrappter Schlüssel n | CRC32C u32
```

- Gewrappt unter dem Wurzelschlüssel mit AES-256-GCM-SIV, AD = `art ‖ epoche ‖ tenant_id`.
- **Schreiben atomar:** `tmp → fdatasync → rename → fsync(Verzeichnis)`. Das Entfernen einer alten KEK ist dasselbe Verfahren.
- Mandanten-Offboarding (7.2): Anker wird ohne KEK-Einträge neu geschrieben, dann KeyTable und Verzeichnis entfernt.

### 5.7 Seal-Registry

KV-Familie `reg/` im Store. Schlüssel `seal: u32 BE`. Wert: AEAD mit `K_registry`, AD = `reg ‖ seal`.

Inhalt: `seal_state`, `doc_id (16)`, `collection (u32)`, `first_slot (u32)`, `chunk_count (u32)`, `class (u8)`, `record_count (u32)`, `subject_tags (Liste aus 16-Byte-Tags)`.

**Seal-Zustände (F-20, einzige Menge):**

| Zustand | Bedeutung | Abgeleiteter `SealState` |
|---|---|---|
| `Active` | Lebt, Inhalt gültig | `Open`, oder `Restricted` wenn im Gate eingeschränkt |
| `Retired` | Revision entfernt, Seal wartet auf Aufbewahrungsende | `Open` nur für `get` mit `as_of` bis zum Aufbewahrungsende. Suche und aktuelles `get` sehen den Seal nicht |
| `Gated` | Löschauftrag durabel, Schlüssel noch vorhanden | `Erased` |
| `KeysDestroyed` | Schlüssel vernichtet, Scrub läuft | `Erased` |
| `Erased` | Scrub abgeschlossen, nur Registry-Minimalform | `Erased` |

Übergänge sind monoton. `Gated → Active` gibt es nicht. Ein Zurücksetzen ist nur durch `Restricted → Released` möglich, und `Restricted` ist kein Seal-Zustand, sondern eine Gate-Einschränkung (7.5).

**Betroffenen-Index.** `subject_tag → RoaringBitmap<SealOrd>` in der Familie `sub/`. `subject_tag = BLAKE3-keyed(K_ref, SubjectRef)` (16 Byte). Die Betroffenen-Kennung im Klartext steht nirgends.

**Dokument-Index.** `doc_id → [SealOrd]` in der Familie `doc/`.

Chunks eines Seals liegen in **zusammenhängenden Slots**. Das macht LiveMask-Änderungen Bereichsoperationen.

### 5.8 Segmente und Segmentschlüssel (ADR-30, F-34)

Jede abgeleitete Datei (Postings, Vektor-Struktur, SQ8-Codes, Filter-Bitmaps, Graph-Struktur) besteht aus Segmenten. Ein Segment hat einen Segmentschlüssel `SegKey` im Segment-Raum der KeyTable.

**Segment-Kopf (Klartext, 64 Byte):** Magic `CXSG`, Formatversion u16, `kind` u8 (`postings`, `vector_struct`, `vector_code`, `filter`, `graph_struct`), `seg_ord` u32, Chunkgröße u32 (65.536), Anzahl Chunks u32, CRC32C. Keine Nutzdaten, keine Identifikatoren außer `seg_ord`.

**Segment-Körper:** Folge von Chunks, je Chunk:

```
[nonce 12][länge u32 LE][ciphertext n][tag 16]
```

- AEAD AES-256-GCM, Nonce zufällig, AD = `seg_ord ‖ kind ‖ chunk_index`.
- Obergrenze je `SegKey`: `max_chunks_per_segkey = 2^24`.
- Ein Segment wird nur durch Neuschreiben geändert (Copy-on-Write). Änderungen an Segmenten gehen ausschließlich über Stapel oder Scrub.

**REQ-SEG-1 (Lesen).** Lesen eines Segments öffnet zuerst den Segment-Header, dann Chunks. Fehlt der Segmentschlüssel (`KeyAbsent`), gilt das Segment als nicht lesbar. Der Aufrufer behandelt es als Index-Fehler im Sinn von 8.2 und baut es aus den Quellen neu auf (REQ-IDX-REBUILD, 6.3).

**REQ-SEG-2 (Schlüssel-Lebenszyklus).** Ein neues Segment bekommt einen neuen `SegKey`. Ein Segment ist ein Schlüssel-Lebenszyklus. Der Scrub (7.6) erzeugt ein neues Segment, veröffentlicht es und vernichtet danach den alten `SegKey` (ADR-31).

### 5.9 IDs und Ordinale

- `DocId`: zufällig, 128 Bit, nie aus Inhalt oder Pfad abgeleitet.
- `chunk_id = BLAKE3(doc_id ‖ rev ‖ chunk_index)`, 128 Bit.
- `SealOrd` und `SegOrd`: vergeben vom Schreib-Task, monoton, nie wiederverwendet. Bei Erreichen von `2^32 − 1` weist der Mandant neue Schreibstapel mit `Refused` ab (Zählwert im Anker-Header der KeyTable).
- `tenant_id`: 16 Byte Zufall, nicht aus dem Verzeichnisnamen abgeleitet.

### 5.10 Handkodierte Plattenformate (REQ-FMT-4, neu gefasst)

Genau fünf Formate sind handkodiert mit geprüften Feldzugriffen und Fixture-Tests:

| Format | Datei | Kapitel |
|---|---|---|
| `CXKT` | Schlüsseltabelle | 5.5 |
| `CXAN` | Anker | 5.6 |
| `CXEL` | Löschlog | 7.8 |
| `CXGS` | Gate-Snapshot | 7.8 |
| `CXSG` | Segment-Kopf und -Körper | 5.8 |

Der Seal-Datensatz-Umschlag (5.3) ist ein Wert im KV-Store und nutzt das Store-Format. Alles andere (WAL, SST, Manifest) nutzt die bestehende Kodierung der Store-Crate mit Versionsfeld. Ungültige Längen führen zu `Integrity`. Jedes Format hat eine Version im Kopf.

**REQ-FMT-3 (Version).** Unbekannte Versionen werden mit `Integrity` abgelehnt. Frühere Versionen werden nicht gelesen (ADR-44). Eine Formatänderung erhöht die Version und braucht eine neue Spezifikationsversion.

---

## 6. RAG-Kernpipeline

### 6.1 Ingest

**REQ-ING-1 (Pipeline).** `Parse → Chunk → Digest → Embed (Stapel) → Seal und Store → Index`. Jede Stufe hat eine begrenzte Warteschlange (64 Stapel) und eigene Parallelität. Rückstau pflanzt sich bis zum Aufrufer als `Backpressure` fort. Kein Blockieren.

**REQ-ING-2 (Formate).** Feature je Format, Standard nur `ingest-text`:

| Feature | Formate |
|---|---|
| `ingest-text` (Standard) | TXT, Markdown, JSON |
| `ingest-web` | HTML |
| `ingest-docx` | DOCX |
| `ingest-pptx` | PPTX |
| `ingest-sheets` | XLSX, CSV |
| `ingest-pdf` | PDF (eingebetteter Text, kein OCR) |
| `ingest-email` | EML, MBOX |

Unbekannte Formate werden mit `Refused` abgewiesen, nie geraten. ODF, OLE und YAML/TOML sind nicht enthalten (ADR-35). Die Features `workspace`, `types-compat` und `fuzz` entfallen aus `ingest`. Fuzz-Ziele liegen unter `crates/contextra-ingest/fuzz/`, nicht als Feature.

**REQ-ING-3 (Chunking).** Ziel 512 Token, Minimum 50, Trennung an Abschnitt, Absatz, Satz. Der Abschnittspfad (Breadcrumb) wird **vor** Embedding und BM25 dem Chunk-Text vorangestellt: `embed_text = breadcrumb + "\n" + text`.

**Tokenschätzung (deterministisch, ohne Modell).** `tokens = ceil(W × 1,35)`, `W` = Anzahl Wortsegmente nach Unicode Word Boundaries (`unicode-segmentation`). Gilt für Chunking und Kontextbudget gleichermaßen.

**REQ-ING-4 (Provenienz).** Jeder Chunk trägt `doc_id`, `rev`, `chunk_index`, Zeichenbereich, Seite, Abschnittspfad. Quellpfad und Dateiname liegen in `SealedMeta`.

**REQ-ING-5 (Fehler je Dokument).** Ein defektes Dokument bricht den Stapel nicht ab. `IngestReport` führt je Dokument `Ok`, `Unchanged` (gleicher Digest, ADR-50), `Partial` mit Hinweisen oder `Failed` mit Fehlerart.

**REQ-ING-6 (Betroffene deklarieren).** `SourceDoc.subjects` nimmt die vom Quellsystem bekannten Betroffenen auf (Tags werden im Aufrufer aus dem Quellschlüssel gebildet, nicht aus dem Inhalt). Die Entity-Erkennung läuft ohne Deklaration. Treffer erzeugen `UndeclaredPersonalData` im Report, nie eine automatische Zuordnung.

**REQ-ING-7 (Unterdrückung, neu gefasst zu F-19).** Der Schreib-Task prüft beim Commit jedes Dokuments die deklarierten Betroffenen-Tags gegen die Unterdrückungsliste. Ist **ein** deklarierter Tag unterdrückt, wird das gesamte Dokument mit `SubjectSuppressed` abgewiesen. Der Rest des Stapels läuft weiter. Die Prüfung und der Commit laufen unter dem Commit-Lock aus 8.4, dadurch ist eine Race mit einer gleichzeitigen Löschung ausgeschlossen.

**REQ-ING-8 (Idempotenz).** `key = (collection, external_id)`, `digest = BLAKE3-keyed(K_ref, Quellbytes)`. Gleicher Schlüssel und gleicher Digest: `Unchanged`, keine neue Revision. Gleicher Schlüssel und neuer Digest: neue Revision mit neuem Seal, alte Revision wird `Superseded`.

**REQ-ING-9 (Bulk).** `IngestMode::Bulk { sync_every }` synchronisiert alle 64 MiB oder 5 s und am Jobende. Bestätigt wird erst nach dem Sync. Ein Abbruch wiederholt den Job, REQ-ING-8 macht das sicher. Löschaufträge sind nie `Bulk`.

**REQ-ING-10 (Graph-Eingabe).** `SourceDoc.graph` (optional) enthält `entities` (Schlüssel des Aufrufers, Typ aus geschlossenem Vokabular, Name) und `relations` (Quell-Schlüssel, Ziel-Schlüssel, Relationstyp aus geschlossenem Vokabular, Gewicht in [0, 1]). Grenzen je Dokument: 10.000 Entitäten, 50.000 Relationen. Überschreitung ist `Refused` für das Dokument.

### 6.2 Embedding

**REQ-EMB-1 (Port).** `EmbeddingProvider { dim(), model_id(), embed_batch(&[&str]) }`. `model_id` und `dim` stehen im Index-Kopf. Batchgröße 64, maximale Eingabe 512 Token (Chunking garantiert das, ein größerer Eingang ist `Refused`).

**REQ-EMB-2 (Backends).** `contextra-embed`: lokales ONNX (`ort`), Modelldatei mit erwartetem SHA-256 (ADR-36), Abweichung führt zu `Refused`. `contextra-embed-http`: konfigurierter interner Dienst, TLS, Allowlist, Zeitgrenzen. Ohne Backend läuft die Fassade mit vorberechneten Vektoren (Aufrufer liefert sie).

**REQ-EMB-3 (Datenabfluss).** Das HTTP-Backend sendet Chunk-Text nach außen. Das ist Datenabfluss im Sinne des Egress-Guards (REQ-PRV-3). Der Endpunkt muss im Verarbeitungsregister stehen, und für P-Collections muss `allow_personal = true` je Endpunkt gesetzt sein.

**REQ-EMB-4 (Modellwechsel).** Ein Index gehört zu genau einer `model_id` und `dim`. Anfragen mit anderem Modell sind `ModelMismatch`. Ein Wechsel erfordert Neuberechnung aus `VectorFull` oder `DocBlob`. `drift` vergleicht Stichprobenstatistik und meldet Abweichung, ohne automatisch zu handeln.

**REQ-EMB-5 (Reranker, optional).** `Reranker`-Port mit ONNX-Cross-Encoder (ADR-36), standardmäßig aus. Bewertet die besten `rerank_input` (Standard 50) Kandidaten, danach Auswahl der Top-k.

### 6.3 Vektorindex

**REQ-VEC-1 (HNSW).** `M = 16`, `ef_construction = 200`, `ef_search = clamp(max(128, 4·k), 128, 1024)`. SQ8 ab `index_flat_below` (Standard 20.000 Vektoren je Shard, ADR-04). Im Arbeitsspeicher liegen SQ8-Codes und Graph, **nicht** die fp32-Vektoren.

**REQ-VEC-1a (SQ8-Kalibrierung).** Wertebereich je Dimension aus den ersten 20.000 Vektoren beim Übergang von exakt zu HNSW. Eingefroren bis zum vollständigen Neuaufbau. Kalibrierung steht im Index-Kopf.

**REQ-VEC-2 (Exakt).** Bis `index_flat_below` Vektoren je Shard sucht der Index exakt mit SIMD-Kosinus. Der exakte Pfad ist auch die Referenz für Rückrufmessungen (T-RAG-04).

**REQ-VEC-3 (Nachberechnung).** Die besten `rerank_k = min(4 · k_cand, 4000)` Kandidaten werden aus `VectorFull` gelesen, über `RecordSealer` entschlüsselt (Gate vorher geprüft, REQ-FLT-5) und in fp32 neu bewertet.

**REQ-VEC-4 (Persistenz, neu gefasst zu F-24).** Auf Platte liegen nur Segmente (ADR-30):
- `vector_struct`: Graphstruktur (Nachbarlisten, Knoten→Slot), Delta-Log als eigenes Segment,
- `vector_code`: SQ8-Codes je Chunk,
- Kalibrierung und Index-Kopf im Kopf des Struktursegments.

Der Start lädt die `vector_code`-Segmente parallel und entschlüsselt sie segmentweise (kein Einzel-Datensatz-Lauf). `VectorFull` wird nicht beim Start gelesen. Ziel für L: unter 60 s bis `Ready` (10.1).

**REQ-VEC-5 (Löschen im Index).** Kein Neuaufbau je Löschung.
1. Sofort: `LiveMask`-Bits löschen. Knoten bleiben für die Navigation, nie in Ergebnissen.
2. Sofort mit der Schlüsselvernichtung: **Routen-erhaltender Scrub** der SQ8-Codes toter Knoten im Speicher. Der Code wird durch den Mittelwert der Codes lebender Nachbarn ersetzt. Der Personenbezug verschwindet aus dem Arbeitsspeicher, die Navigation bleibt erhalten.
3. Im Scrub-Stapel (7.6): **Konsolidierung.** Tote Nachbarn in Nachbarlisten werden durch deren lebende Nachbarn ersetzt. Segmente `vector_struct` und `vector_code` werden ohne tote Knoten neu geschrieben.
4. Neuaufbau nur, wenn der Anteil toter Knoten nach Konsolidierung 30 % übersteigt (`tombstone_rebuild_ratio`).

**REQ-IDX-REBUILD (neu, F-19-Folgefrage).** Ein Index-Segment, das nicht geöffnet werden kann (Schlüssel fehlt, Prüfsumme falsch), führt zum Zustand `Rebuilding` des Shards. Suchen auf dem Shard liefern `IndexUnavailable`, keine Teilergebnisse (REQ-SRC-4). Der Neuaufbau liest `VectorFull` lebender Seals. Eine Löschung erzeugt diesen Fall nie, weil Segmentschlüssel erst nach dem Neuschreiben vernichtet werden (ADR-31).

**REQ-VEC-6 (Verboten).** DiskANN, RaBitQ, `partial_rebuild`, `purge` als eigener Pfad. Beide Algorithmen kehren nur über Gate G-INDEX (11.5) zurück.

### 6.4 Textindex

**REQ-TXT-1 (BM25).** `k1 = 1,2`, `b = 0,75`, konfigurierbar. Block-Max-WAND. Segmente höchstens `segment_max_bytes` (256 MiB). Speicherpuffer für frische Dokumente, Hintergrund-Merge. Segmente sind verschlüsselt (kind `postings`, ADR-30).

**REQ-TXT-1a (Tokenizer).** NFKC-Normalisierung, Kleinschreibung, Unicode-Wortsegmentierung. Snowball-Stemmer für Deutsch und Englisch (`rust-stemmers`). Eingebaute Stoppwortlisten `de` und `en` in `contextra-text/data/`. **Keine Komposita-Zerlegung im Kern** (Wörterbuchabhängigkeit, entfernt; Rückruf-Effekt misst T-RAG-03). Domänenwörterbücher sind Konfiguration eines Adapters, nicht Teil des Kerns.

**REQ-TXT-2 (Löschen im Textindex).** Treffer toter Seals werden im Lesepfad über die LiveMask gefiltert. Postings bleiben bis zum Scrub-Merge (7.6) physisch stehen. Zähler `N` und Längensumme werden beim Löschen exakt verringert (nicht personenbezogen, liegen im Index-Kopf mit CRC). `df(t)` wird beim Merge neu berechnet.

**Dokumentierte Abweichung (F-25).** Bis zum Merge weicht `df(t)` für Terme ab, die im Wesentlichen nur in gelöschten Dokumenten vorkommen. Diese Dokumente erscheinen im Ergebnis nie. Die Abweichung ist deterministisch (REQ-SRC-2) und auf die Scrub-Frist begrenzt (7.9). Sie wird im Benchmark als Metrik ausgewiesen.

**REQ-TXT-3 (Entdeckung).** Für `plan_erasure` (7.3) sucht der Textindex Bezeichner (Name, E-Mail, Personalnummer) als UND über Tokens. Kandidaten werden aus `ChunkText` entschlüsselt und exakt auf Teilzeichenfolge geprüft (NFKC, Kleinschreibung). Begrenzt durch `discovery_max_candidates` (Standard 10.000). Ergebnis: nur Zähler und Seal-Ordinale, nie Text.

### 6.5 Filter, Zugriffsrechte, LiveMask

**REQ-FLT-1 (Filterfelder).** Je Collection deklariert der Betreiber Filterfelder (`tags`, `dept`, `doc_type`, `valid_from`, `valid_to`, `acl`). Inverse Bitmaps (`Feld=Wert → RoaringBitmap<ChunkSlot>`) liegen im Arbeitsspeicher und als Segmente der Art `filter` auf Platte (verschlüsselt, INV-9). Bereichsfelder liegen spaltenweise.

**REQ-FLT-2 (ACL, neu gefasst zu F-31).** Jeder Chunk trägt eine Menge von ACL-Tags. **Principals kommen aus der Konfiguration des Hosts (ADR-38)**, nie aus Anfrage oder Tool-Argumenten. MCP: `mcp.principals` in der Konfiguration. Server: Zuordnung von OIDC-Claims zu Principals über Konfiguration, Standard **verweigern**. Die Filterbitmap ist die Vereinigung der Bitmaps der Principals.

**REQ-FLT-3 (Planer, neu gefasst zu F-36).** Selektivität `s = |Filter| / N`. Ist `|Filter| ≤ filter_exact_max` (Standard 20.000): exakte SIMD-Suche über die Filtermenge. Sonst ACORN-artige Traversierung mit Prädikat (`contextra-vector::acorn`). Die Wahl erfolgt aus Zählwerten, nicht aus Laufzeit (deterministisch). Die beiden Schwellen `index_flat_below` (Indexmodus) und `filter_exact_max` (Filtermodus) haben getrennte Namen und Zwecke.

**REQ-FLT-4 (LiveMask).** Dichte Bitmap je Shard mit einem Bit je Chunk-Slot, atomar gelesen. Sie ist **abgeleitet** aus Registry, Visibility und Gate, und wird beim Start neu aufgebaut. Gate und Registry bleiben die Wahrheit.

**REQ-FLT-5 (Autoritative Endprüfung).** Jedes materialisierte Ergebnis wird einmal gegen `ReadGate::state(seal)` geprüft, **bevor** Schlüsselmaterial angefasst wird. `Erased` und `Restricted` fallen heraus. Fallen Kandidaten heraus (die LiveMask hinkte hinterher), füllt die Suche in höchstens zwei Runden mit erhöhtem `k_cand` auf. Die Antwort kann dann weniger als k Treffer enthalten. Fehlt der Schlüssel (`KeyAbsent`), gilt das als `Erased`.

### 6.6 Suche und Fusion

**REQ-SRC-1 (Ablauf).**
1. Snapshot pinnen (Version-Snapshot und Gate-Sequenz), Filterbitmap bilden (4.6). Principals aus Konfiguration.
2. Vektorsuche (HNSW oder exakt) und Textsuche, je `k_cand = max(2·k, 50)`, begrenzt durch `MAX_SEARCH_K` (ADR-52).
3. Endprüfung je Kandidat (REQ-FLT-5), Nachberechnung (REQ-VEC-3).
4. Optional Graph-Signal (6.10), nur bei `GraphMode::PathRag`.
5. Fusion: gewichtete Reciprocal Rank Fusion, `k = 60`, Gewichte aus Konfiguration (ADR-49).
6. Optional Reranker (REQ-EMB-5).
7. Auswahl der Top-k. Tie-Break aufsteigend nach `DocId`, dann `chunk_id`.
8. Ausgabe mit Provenienz, Signalrängen, optional Erklärung.

**REQ-SRC-2 (Determinismus).** Gleiche Anfrage auf gleichem Snapshot und gleichem Gate-Stand liefert identische Trefferlisten. HNSW-Traversierung je Anfrage single-threaded. Zeit, Last und Zufall wirken nicht auf das Ergebnis.

**REQ-SRC-3 (Löschung ohne Sonderpfad).** Das Gate wirkt über Filterbitmap und Endprüfung. Es gibt keinen zusätzlichen Suchdurchlauf.

**REQ-SRC-4 (Frist).** `SearchOptions.deadline` ist optional. Bei Überschreitung liefert die Suche `DeadlineExceeded`. Es gibt nie ein verkürztes oder anders sortiertes Teilergebnis. Die Prüfung ist kooperativ an Traversierungsschritten.

**REQ-SRC-5 (Trefferformat).** `chunk_id`, `doc_id`, `rev`, `score`, Provenienz ohne Quellpfad (nur bei Berechtigung), `signals`, optional Erklärung.

**REQ-SRC-6 (Keine Abfragespur, ADR-37).** Der Suchtext und das Abfrage-Embedding werden nicht gespeichert, nicht geloggt und nicht gecacht. `MetricsSink` erhält nur Zähler, Dauern und Trefferzahlen.

### 6.7 Kontext

**REQ-CTX-1.** `assemble(response, TokenBudget) → ContextPack` wählt in Rangfolge bis zum Budget. Kürzung am Satzende. Quellen vollständig. Tokenzählung nach 6.1.

**REQ-CTX-2.** `ContextPack` nennt verwendete Chunks und ausgelassene mit Grund (`Budget`, `Erased`, `Duplicate`, `Restricted`).

**REQ-CTX-3 (Gate beim Zusammenstellen).** `SearchResponse` trägt die Gate-Sequenz. `assemble` prüft jeden Chunk erneut, wenn `gate_seq` inzwischen gewachsen ist.

**REQ-CTX-4 (Duplikate, neu gefasst zu F-33).** Gleicher Inhalt aus mehreren Revisionen: nur die neueste Revision. Vergleich über den keyed Chunk-Digest `BLAKE3-keyed(K_ref, chunk_text)`. Ein unkeyed Hash über Inhalte wird nicht gespeichert.

### 6.8 Fassade, Fehler und Schnittstellen

**ContextraError (Port `ports::error`, einzige Fehlerart der öffentlichen API):** `Config(Vec<String>)`, `Refused(String)`, `Backpressure`, `DeadlineExceeded`, `NotFound`, `Erased`, `HistoryNotIndexed`, `SubjectSuppressed`, `ModelMismatch`, `IndexUnavailable`, `ReadOnly(Reason)`, `Quarantine(TenantId)`, `Integrity`, `Unsupported(&'static str)`, `Crypto` (ohne Unterscheidung nach außen, INV-6), `Io(ErrorKind)`.

| Operation | Zweck | Berechtigung |
|---|---|---|
| `Contextra::open(config)` | Startsequenz (8.3) | Prozess |
| `ingest(collection, SourceDoc, IngestOptions)`, `ingest_batch` | Aufnahme | Schreibrecht je Collection |
| `search(SearchQuery)` | Hybridsuche mit Filter und optional `GraphMode` | Principals aus Konfiguration |
| `assemble(SearchResponse, TokenBudget)` | Kontextpaket | wie Suche |
| `get(doc_id, as_of?)` | Abruf, mit Gate | wie Suche |
| `delete_document(doc_id)` | Normales Löschen mit Aufbewahrung (REQ-VER-4) | Schreibrecht |
| `plan_erasure(ErasureRequest)` | Trockenlauf ohne Nebenwirkung (7.3) | `erasure_admin` |
| `erase(ErasureRequest)` | Löschauftrag (Kapitel 7) | `erasure_admin` |
| `erase_status(TicketId)` | Status nach 7.10 | `erasure_admin` |
| `resolve_review(TicketId, Decision)` | Entscheidung bei Mehrbetroffenen | `erasure_admin`, bei `four_eyes` zweiter Admin |
| `set_hold(scope, reason, until)` | Aufbewahrung, Legal Hold | `erasure_admin` |
| `health()` | Zustand, Gründe, Zähler | Prozess |

`remember`, `recall`, `forget` und der Query-Rewriter entfallen (REM-24, REM-25). Wer sie braucht, baut einen Adapter über `search` und `ingest`.

**REQ-MCP-1.** Der MCP-Server bietet dieselben Operationen über stdio. Tool-Schemas werden aus den Fassadentypen erzeugt. Principals und Rollen kommen aus der Konfiguration (ADR-38). Es gibt keine Tier-Parameter. Fehlermeldungen für nicht eingebaute Backends nennen das Feature, das zu aktivieren ist. Löschoperationen sind nur mit `erasure_admin` aufrufbar, und die Konfiguration muss sie ausdrücklich freigeben (Standard aus).

**REQ-SRV-1 (optional, M6).** `contextra-server` bietet HTTP/gRPC mit Authentifizierung (OIDC-Token, Prüfung gegen konfigurierte Schlüssel), bildet Claims auf Principals ab (Standard verweigern), exportiert Metriken im Prometheus-Format und begrenzt Anfragen über Backpressure. Eigenes Binary-Crate. Der Kern bleibt netzfrei.

### 6.9 Datenschutzfilter am Ausgang

**REQ-PRV-1.** Entity-Erkennung ist regelbasiert (Muster, Prüfziffern, Listen). Sie dient der Warnung bei Ingest und der Ausgabeprüfung. Unsicherheit wird als `Unclassified` markiert und führt nie zur automatischen Freigabe.

**REQ-PRV-2.** Keine LLM-Klassifikation im Kern (ADR-24).

**REQ-PRV-3 (Egress-Guard).** Jede Ausgabe über MCP, Server oder Export und jeder Aufruf des HTTP-Embedders läuft durch den Guard. Er arbeitet lokal, ohne E/A, fail-closed.

**REQ-PRV-4 (Verarbeitungsregister).** Das Register (Art. 30 DSGVO) führt Zweck, Kategorien, Empfänger und externe Endpunkte. Es ist Konfigurationsdaten und kein Laufzeitnachweis.

### 6.10 Entitätsgraph (`contextra-graph`, optional, neu)

Dieses Kapitel ersetzt PLAN v2.1 des Crates und den Anhang der v4.0 (F-01 bis F-06, F-41, F-42).

**REQ-GR-1 (Zweck).** Der Graph liefert Belege über Entitäten und Relationen, die Dokumente verbinden. Ausgabe sind Pfade mit Belegen (PathRAG), nicht nur Chunk-Listen. Er ist ein **abgeleiteter Index** (Klasse D, ADR-22). Die Quelle der Wahrheit bleiben die Seals.

**REQ-GR-2 (Datenmodell).**
- `EntityId`: 128 Bit, `BLAKE3-keyed(K_ref, typ_id ‖ normalisierter_name)`. Normalisierung: NFKC, Kleinschreibung, Leerraum zusammengeführt.
- `typ_id`: Index in einem geschlossenen Vokabular der Collection (Konfiguration). Freie Typen sind `Refused`.
- Entitäten tragen **Seal-Provenienz**: Menge der `SealOrd`, die sie nennen (sortiert, Obergrenze 64, darüber Bitmap).
- Kanten: `(from, to, rel_typ_id, weight f32)`, Provenienz wie bei Entitäten. Relationstyp aus geschlossenem Vokabular.
- Namen und Relationsbezeichnungen stehen **nur** als `GraphLabel`-Datensätze, verschlüsselt mit dem `DocKey` des Quell-Seals (ADR-23). Im Graph selbst liegt nie Klartext.
- Zuordnung `chunk_slot → [EntityId]` für Belege (`mentions`).

**REQ-GR-3 (Sichtbarkeit).** Eine Kante ist sichtbar, wenn mindestens ein Provenienz-Seal `Open` ist (ReadGate). Eine Entität ist sichtbar, wenn mindestens eine ihrer Provenienzen `Open` ist. Die Prüfung gilt beim Materialisieren, nicht beim Durchlaufen. Dadurch wirkt eine Löschung sofort, bevor der Scrub läuft.

**REQ-GR-4 (Schreibpfad).** Graph-Änderungen gehören zum Schreibstapel (REQ-CON-6). Eine Revision, die superseded wird, entfernt die Provenienz ihres Seals im selben Stapel. Kanten mit leerer Provenienz werden tombstonet und im Scrub entfernt. Änderungen laufen als Delta-Segment (Art `graph_struct`). Basis-Segmente sind unveränderlich. Kompaktion des Deltas bei mehr als 10 % der Basis oder 1 Mio. Operationen, in einem Scrub-Stapel oder einem Schreibstapel.

**REQ-GR-5 (Traversierung und PathRAG).**
- Seeds: die Top-20 Chunks der Fusionsliste (Schritt 4 aus 6.6).
- Entitäten der Seeds über `mentions`. Expansion per BFS bis `max_hops ≤ 3`, nur über sichtbare Kanten.
- Deterministische Reihenfolge: Frontier nach `EntityId` aufsteigend.
- Begrenzung: `max_expansions = 10.000` je Anfrage. Erreichte Grenze markiert das Ergebnis als `truncated`, sie ist kein Zeitbudget (REQ-TIM-3).
- Pfadwert: `seed_score · Π(weight_e · 0,5^hop)`. Ausgabe: die Top-20 Pfade nach Wert, Tie-Break nach Pfadliste.
- Belege je Pfad: Chunks aus `mentions` der Pfadentitäten, nur lebende Seals.

**REQ-GR-6 (Scrub, ADR-27).** Die Scrub-Schritte für den Graph (7.6):
1. Entitäten und Kanten ohne sichtbare Provenienz entfernen.
2. Provenienzmengen toter Seals entfernen.
3. `mentions` toter Chunk-Slots entfernen.
4. Konsolidierung der Nachbarlisten. Neuaufbau ab 30 % toter Knoten.
5. Neues Basis-Segment, alter `SegKey` nach Veröffentlichung vernichtet (ADR-31).

**REQ-GR-7 (Zeitreise).** `as_of` auf Graph-Operationen ist `HistoryNotIndexed` (ADR-26).

**REQ-GR-8 (Speicher).** Richtwert 96 Byte je Entität und 48 Byte je Kante ohne Labels. Limits je Collection: 5 Mio. Entitäten, 50 Mio. Kanten, Grad je Entität höchstens 512 (überzählige Relationen werden im Report als `Dropped` ausgewiesen).

**REQ-GR-9 (Aktivierung und Gate).** Feature `graph` in `contextra-db` (Standard aus). Je Collection `graph = true`. Abnahme-Gate G-GRAPH (11.5). Wird es verfehlt, wird das Crate in M5 entfernt (REM-31), die Entscheidung ist mit Messung dokumentiert.

**REQ-GR-10 (Was entfällt).** Hyperkanten, Relation-Suggest mit LLM, Community-Erkennung, Edge-Reinforcement, Agenten-DAG, bitemporale Felder, APPRH, TL-HFD, Percolation, Dense PPR, Forward-Push-PPR, `ConsistencyEnforcer`, Entitätsextraktion im Kern, `text_gen`. Liste in 12.4.

---

## 7. Löschkette v4.1

### 7.1 Ziele

1. **Sofort unlesbar** nach bestätigtem Auftrag: Gate durabel, Lesen blockiert, Schlüssel vernichtet.
2. **Ein Auftrag endet** in `Completed`, `Released`, `Refused` oder in einem benannten, fristgebundenen Rest.
3. **Unauffällig im Betrieb:** Gate kostet im Suchpfad ein Bit und eine Endprüfung. Scrub läuft gebündelt, gedrosselt, unter der Priorität des Suchpfads.
4. **Entkoppelt von der Versionierung** (Kapitel 4).
5. **Keine Selbstbewertung** (ADR-02). Die Kette führt aus und protokolliert.

### 7.2 Umfang eines Auftrags

```rust
pub struct ErasureRequest {
    pub scope: Scope,                       // Subject | Document | Collection | Tenant
    pub reason: Reason,                     // Art17 | Withdrawal | Retention | Offboarding | Other
    pub requester: PrincipalRef,            // muss erasure_admin sein (ADR-38)
    pub urgency: Urgency,                   // Normal | Immediate
    pub multi_subject: Option<MultiSubjectPolicy>,  // überschreibt Mandantenstandard
}
pub enum Scope {
    Subject    { subjects: Vec<SubjectRef> },  // Aufrufer nennt die Betroffenen (keyed Tags werden berechnet)
    Document   { doc: DocId },                 // alle Seals des Dokuments, auch retained
    Collection { id: CollectionId },
    Tenant,
}
```

| Scope | Wirkung |
|---|---|
| `Subject` | Alle Seals mit passendem Betroffenen-Tag (Registry, 5.7). Bezeichner ohne Tag laufen über die Entdeckung (7.3), nicht automatisch. |
| `Document` | Alle Seals des `DocId` aus `doc/`, ohne den Versionskatalog zu lesen. |
| `Collection` | Alle Seals der Collection. Bei K-Collections Segmentersatz, bei P-Collections Schlüsselvernichtung. |
| `Tenant` | **O(1) in der Schlüsselarbeit:** Anker ohne KEK-Einträge neu schreiben, dann `destroy_tenant`. Mandant wird im Gate als `tenant_erased` markiert, dadurch sind alle Lesevorgänge `Erased`. Verzeichnis-Entfernung danach asynchron. |

**Unterdrückungsliste (`ErasedSubj`).** Enthält die Betroffenen-Tags gelöschter Betroffener (16 Byte keyed). Sie verhindert die Wiederaufnahme (REQ-ING-7). Sie wächst nur. Sie steht im Gate-Snapshot (7.8).

**ADR-38 (Berechtigung).** Löschoperationen verlangen einen Principal mit `erasure_admin`, der in der Konfiguration steht. Der Standard lässt niemanden zu. Fehlende Berechtigung ist `Refused` und wird im Ledger als Versuch vermerkt.

### 7.3 Trockenlauf und Entdeckung

**REQ-ER-1 (`plan_erasure`).** Ohne Nebenwirkung. Der Bericht nennt: Anzahl Seals je Klasse, Anzahl Chunks, Anzahl Dokumente mit weiteren Betroffenen (nur Anzahl), Aufbewahrungs- und Hold-Treffer (Anzahl und Grund), undeklarierte Treffer aus der Entdeckung (Anzahl). Nie Kennungen, nie Inhalte.

**REQ-ER-2 (Klassifikation, neu gefasst zu F-17).** Jeder Seal des Auftrags wird genau einer Klasse zugeordnet, in dieser Reihenfolge:

| Klasse | Bedingung | Weiterer Weg |
|---|---|---|
| **Hold** | Aufbewahrungspflicht oder Legal Hold aktiv | `Refused` für diesen Seal, Ledger-Eintrag |
| **Review** | Seal hat deklarierte Betroffene, die **nicht** alle im Auftrag stehen (ADR-10) | `Restricted(MultiSubject)` |
| **Erase** | Alle deklarierten Betroffenen des Seals stehen im Auftrag. Keine Hold-Bedingung. | Gate und Vernichtung |
| **Undeclared** | Treffer der Entdeckung ohne Deklaration | Standard `Report`, Mandantenoption `Restrict` |

Ein Seal ohne deklarierte Betroffene und ohne Auftrag „Collection“ oder „Document“ kann nicht in Klasse Erase fallen, wenn der Scope `Subject` ist.

**REQ-ER-3 (Entscheidung).** `resolve_review(ticket, seals, Decision)` mit `DeleteAll`, `Release` (Einschränkung aufheben) oder `RedactReingest(spans)`. Bei `four_eyes = true` muss der Entscheider vom Antragsteller verschieden sein und `erasure_admin` sein. Erinnerungen und Eskalation: 7.5.

**REQ-ER-4 (RedactReingest).** Aus den angegebenen Spannen wird eine bereinigte Fassung erzeugt und über den normalen Ingest als **neue Revision mit neuem Seal** geschrieben. Erst nach deren Commit wird der alte Seal vernichtet. Dazwischen ist der alte Seal `Restricted`. Die Versionierung sieht nur eine reguläre neue Revision.

### 7.4 Ablauf

Jeder Schritt ist idempotent. Das Löschlog ist die Wahrheit. Die Reihenfolge ist fest: **Gate, Schlüssel, Arbeitsspeicher, Scrub**.

| Schritt | Ausführender | Inhalt | Dauerhaft ab | Zustand danach |
|---|---|---|---|---|
| **E0 Annahme** | Tenant-Koordinator | Berechtigung prüfen (ADR-38), Ticket, Log `Requested` (fdatasync). Gleicher Scope → bestehendes Ticket | Log | `Requested` |
| **E1 Gate** | Tenant-Koordinator, unter dem **Erasure-Lock** des Mandanten | Klassifikation (7.3). Log `Gated`/`Restrict` für alle Seals. fdatasync. Gate-Datei atomar ersetzen (`tmp → fdatasync → rename`). `ArcSwap`-Publikation. Registry-Zustand `Gated`. LiveMask-Änderungen an die Shards senden (Warteschlange mit Grenze). | Log, Gate-Datei | `Gated` |
| **E2 Schlüssel** | Tenant-Koordinator über `KeyDestroyer` | Für Klasse Erase: Schlüsselcache-Lock (REQ-KEY-4), `SealScoped`-Invalidierung, Slots überschreiben, ein fdatasync je Stapel, Registry `KeysDestroyed`. Routen-erhaltender Scrub der SQ8-Codes im Speicher (REQ-VEC-5). Log `KeysDestroyed`. | KeyTable, Log | `KeysDestroyed` |
| **E3 Scrub** | Scrub-Scheduler, Ziele über `ScrubTarget` | Gebündelt nach 7.6. Neue Segmente veröffentlichen, dann alte `SegKey` vernichten (ADR-31). Log `Scrubbed` je Ziel. | Manifest je Teilschritt | `Scrubbing` |
| **E4 Abschluss** | Tenant-Koordinator | Registry-Minimalform, Log `Completed`, Status. Überschreitet E3 `scrub_deadline`: Kennzeichen `Overdue`, Alarm über `MetricsSink`, weiterer Versuch. | Log | `Completed` |

**Publikationsreihenfolge (F-21, verbindlich).** Für einen Schreibstapel: (1) Store-WAL mit Gruppen-Commit, (2) Segmente und Index-Dateien mit fsync, (3) Versionskatalog, (4) Registry, (5) Snapshot-Tausch für Leser. Für eine Löschung: (1) Gate-Log, (2) Gate-Datei, (3) Gate-Publikation, (4) Schlüsselvernichtung, (5) Scrub-Publikation je Segment, (6) Vernichtung des alten Segmentschlüssels.

**INV-2 (Sichtbarkeit).** Ab `Gated` liefert kein Lesepfad Klartext des Seals. Ab `KeysDestroyed` gibt es keinen entschlüsselbaren Seal-Datensatz. Segmentschlüssel bleiben bis zum Scrub-Neuschreiben erhalten (ADR-31, 7.9).

**INV-3 (Sperre bleibt).** Eine durable Sperre wird nie aufgehoben, außer durch `Restricted → Released`. Ein Löschauftrag kehrt nie in `Restricted` oder `Released` zurück.

**REQ-ER-5 (K-Klasse).** Bei K-Collections gibt es keinen Seal-Schlüssel. E2 entfällt. E3 ersetzt die betroffenen Segmente. Der Ledger vermerkt `no_key_destruction`.

**Begründung der Reihenfolge (ADR-03, ADR-31).** Seal-Nutzdaten (Text, Vektor, Original, Labels) liegen je Seal verschlüsselt. Abgeleitete Strukturen (Postings, Graph, HNSW) sind segmentweise verschlüsselt, ihre Segmente mischen aber lebende und tote Seals. Deshalb ist der Segmentschlüssel erst nach dem Neuschreiben vernichtbar. Die Gate-Sperre deckt das Restfenster ab.

### 7.5 Zustandsautomat

```
Requested → Gated → KeysDestroyed → Scrubbing → Completed
Requested → Restricted{grund, bis}              reversibel (Art. 18, Review)
Restricted → Gated                              Eskalation zur Löschung
Restricted → Released                           terminal
Requested → Refused                             Aufbewahrungspflicht, terminal
jeder nicht-terminale Zustand → Pending{grund} → Fortsetzung
```

Terminal: `Completed`, `Released`, `Refused`. Gründe für `Pending`: `DiskReserveLow`, `IoBackoff`, `ScrubRetry`, `AwaitReview`.

**REQ-ER-6 (Fristen für Review, neu gefasst zu F-30).** `Restricted` ist kein stabiler Endzustand. Erinnerung nach `review_remind_after` (14 Tage), Eskalation nach `review_escalate_after` (30 Tage): Ledger-Eintrag, Metrik, Status `Overdue`. Keine automatische Löschung und keine automatische Freigabe. Der Status ist für den Verantwortlichen sichtbar, die Entscheidung liegt bei ihm.

### 7.6 Gebündelter Scrub

**REQ-SCR-1 (Fenster).** Der erste Auftrag startet ein Fenster `scrub_window` (Standard 10 Minuten). Am Ende oder bei `scrub_batch_max_seals` (Standard 10.000) läuft **ein** Stapel über alle bis dahin angefallenen Seals. `Urgency::Immediate` startet den Stapel sofort.

**REQ-SCR-2 (Frist).** `scrub_deadline` (Standard 24 Stunden ab `Gated`) ist die Obergrenze für E3. Mindestwert 15 Minuten.

**REQ-SCR-3 (Inhalt eines Stapels).** Je Ziel genau ein Neuschreiben je betroffenem Segment pro Stapel.
- **Textindex:** Segmente mit toten Seals ohne deren Postings neu schreiben. `df` neu berechnen (F-25).
- **Vektorindex:** Konsolidierung (REQ-VEC-5), Neuschreiben von `vector_struct` und `vector_code` ohne tote Knoten.
- **Graph:** REQ-GR-6.
- **Store:** Records toter Seals (`ChunkText`, `VectorFull`, `DocBlob`, `SealedMeta`, `GraphLabel`) werden per Kompaktion entfernt. Standard `compaction_after_erase = true` (Begründung 7.9).
- **Registry:** Einträge toter Seals auf Minimalform kürzen (Ordinal, Zustand `Erased`, Zeitpunkt). Betroffenen-Index bereinigen. Ordinale werden nie wiederverwendet.
- **Zwischenspeicher:** `SealScoped` ist in E2 bereits gelaufen, im Stapel nochmals.

**REQ-SCR-4 (Ressourcen).** `scrub_threads` (Standard 2), `scrub_io_limit` (Standard 64 MiB/s, über `Clock` gedrosselt, REQ-TIM-2). Der Scrub pausiert, solange der p95-Wert des Latenzfensters (ADR-43) über `scrub_pause_p95_ms` liegt (Standard 40 ms bei S). Die Frist hat Vorrang: Nach Ablauf von `scrub_deadline` läuft der Scrub weiter, unabhängig von der Latenz. Die Pause beeinflusst nur den Zeitpunkt, nie ein Suchergebnis.

**REQ-SCR-5 (Fehler).** Ein Fehler führt zu `Pending{ScrubRetry}` mit exponentiellem Backoff (1 s bis 1 h), Alarm über `MetricsSink` und Ledger-Eintrag. Ein Fehler im Scrub gefährdet nie bereits erreichte Zustände, insbesondere nicht die Gate-Sperre.

### 7.7 Wiederanlauf

**REQ-REC-1.** Der Start spielt das Löschlog vor dem Öffnen der Lesepfade eines Mandanten ab (C3, 8.3).

| Zustand beim Start | Aktion |
|---|---|
| `Requested` | E1 erneut auswerten |
| `Gated` | ab E2 fortsetzen |
| `KeysDestroyed` | ab E3 fortsetzen |
| `Scrubbing` | Stapel fortsetzen |
| `Pending{…}` | beim nächsten Takt aufnehmen |
| `Restricted` | Sperre erhalten, Review-Fristen fortführen |

**REQ-REC-2 (Slot-Regeln, neu gefasst zu F-29).** Ein Slot wird beim Start nach dieser Tabelle bewertet:

| CRC des Slots | Log enthält `KeysDestroyed` für dessen Seal oder Segment | state | Ergebnis |
|---|---|---|---|
| gültig | – | 2 | `Destroyed` |
| gültig | – | 0 oder 1 | wie gespeichert |
| ungültig | ja | beliebig | `Destroyed` (Vernichtung war im Gang, die Teilschreibung ist nach AEAD nicht entschlüsselbar) |
| ungültig | nein | beliebig | `Integrity`, Quarantäne des Mandanten |

Der Slot ist 128 Byte und liegt auf 128-Byte-Grenzen (ADR-29). Eine Teilschreibung innerhalb eines Sektors ist daher nicht möglich, solange Sektoren atomar sind. Die Tabelle gilt trotzdem für Geräte, die das nicht garantieren.

**INV-4.** Eine durable Sperre geht nie verloren. Die Zustände `Gated` und `KeysDestroyed` überleben jeden Absturz.

### 7.8 Löschlog `CXEL` und Gate-Snapshot `CXGS`

**Löschlog `CXEL` (`erasure/erasure.cxel`).**

Dateikopf: Magic `CXEL` (4), Version u16, Tenant-ID (16), CRC32C (4).

Datensatz:

```
[len u32 LE][seq u64 LE][ticket 16][op u8][payload n][crc32c u32 LE]
```

`len` zählt die Bytes von `seq` bis einschließlich `payload`, ohne `len` und `crc32c`. Die CRC deckt dieselben Bytes ab.

Op-Codes: 1 `Requested`, 2 `Gated`, 3 `KeysDestroyed`, 4 `ScrubQueued`, 5 `Scrubbed`, 6 `Restrict`, 7 `Release`, 8 `Refuse`, 9 `Pending`, 10 `Completed`, 11 `Hold`, 12 `Retire`, 13 `TenantErased`, 14 `RestoreReapplied`, 15 `AuthDenied`.

Lesen: Sequenzlücken und CRC-Fehler sind `Integrity`, außer im letzten Datensatz am Dateiende (abgeschnittener Schreibvorgang). Diesen Rest schneidet der Start ab und vermerkt das im Log.

**Gate-Snapshot `CXGS` (`erasure/gate.cxgs`).**

Kopf: Magic `CXGS`, Version u16, Tenant-ID, `seq u64`, `tenant_erased u8`. Körper: `erased` als RoaringBitmap (portable Kodierung), `restricted` als Liste `(seal u32, reason u8, until_ms i64)`, `suppression` als Liste 16-Byte-Tags. Trailer: CRC32C.

Schreiben: `tmp → fdatasync → rename → fsync(Verzeichnis)`. Bei 10.000 Einträgen in Log oder wöchentlich.

**REQ-GATE-1 (Veröffentlichung).** Gate-Log durabel, Gate-Datei atomar, dann `ArcSwap`. Leser greifen über `load()` zu. Schlägt `fdatasync` fehl, bleibt der alte Snapshot gültig, die Antwort ist `ReadOnly`, und der Mandant geht in `Quarantine`, bis das Log wieder schreibbar ist.

**REQ-GATE-2 (Prüfreihenfolge).** `tenant_erased` vor `erased`, `erased` vor `restricted`. `Restricted` liefert nur mit Audit-Eintrag, im Standard gar nicht (`restricted_read = Deny`).

### 7.9 Ehrliche Grenzen und Restfenster

Diese Grenzen gelten unabhängig von der Implementierung und stehen im Betriebshandbuch. Sie werden nicht zur Laufzeit gemessen.

| Restform | Ort | Obergrenze | Wer kann sie lesen |
|---|---|---|---|
| Seal-Klartext | Speicher, Platte | 0 nach E2 | niemand (Schlüssel vernichtet) |
| Segmentinhalte toter Seals | Segment, verschlüsselt mit `SegKey` | bis Neuschreiben im Scrub, ≤ `scrub_deadline` (24 h) | Prozess mit `SegKey`. Gate blockiert Lesen. |
| SQ8-Codes toter Knoten | Arbeitsspeicher | 0 nach E2 (routen-erhaltend) | niemand |
| Graphstruktur toter Knoten | Delta-Segment | bis Scrub, ≤ `scrub_deadline` | wie Segmentinhalte |
| Kopien in Checkpoints | Kopie des Datenverzeichnisses | bis Rotation, `checkpoint_retention` (Standard 7 Tage) | Restore mit Schlüsselspur (ADR-42) |
| Schlüsselsicherung | Sicherungsmedium | `key_backup_retention` (Standard 400 Tage) | Restore mit altem Schlüssel, Pflicht zu REQ-BAK-3 |
| Auslagerung, Core-Dump | Swap, Speicherabbild | verhindert durch mlock und `PR_SET_DUMPABLE = 0` (ADR-41) | nur mit Root-Zugriff auf das Gerät |
| Externe Ausleitung | Systeme außerhalb | nicht durch die Software kontrollierbar | nicht zuzuordnen |

Weitere Grenzen:
- **Überschreiben ist eine Annahme.** Auf Copy-on-Write-Dateisystemen und Flash hat das Überschreiben eines Slots keine garantierte physische Wirkung. Die KEK-Rotation (REQ-KEY-5) begrenzt den Zeitraum. Mit K3 (Vault Transit) ist die Vernichtung der KEK im KMS die tragende Maßnahme.
- **Kompaktion des Store (ADR-31, REQ-SCR-3).** Ciphertext ohne Schlüssel ist kryptografisch entfernt. Die Kompaktion entfernt ihn zusätzlich, damit keine Kopie stehen bleibt, die ein späterer Schlüssel-Fehler freilegen könnte. Das ist die Begründung für `compaction_after_erase = true`.
- **Nicht deklarierte Mitbetroffene** werden nur über die Entdeckung gefunden, nie automatisch zugeordnet.
- **Rechtliche Einordnung** der Schlüsselvernichtung als Löschung trifft der Verantwortliche (ADR-45).

### 7.10 Ledger und Status

**REQ-LED-1.** `ledger.jsonl` (nur anhängen, eine JSON-Zeile je Ereignis): `ts` (RFC 3339, UTC), `ticket`, `op`, `scope_kind`, `counts {erase, review, hold, undeclared, keys_destroyed, segments_rewritten}`, `requester_ref`, `decider_ref`. Betroffene erscheinen nur als `subject_ref_hash = BLAKE3-keyed(K_ref, SubjectRef)`. Keine Namen, keine Inhalte, keine Dateinamen.

**REQ-LED-2 (`erase_status`).** Liefert `state`, Zeitstempel je Schritt, Zahlen je Klasse, offene Reste (`scrub_pending: seals, segments, deadline`), `overdue`, `review_age_days`. Der Status beschreibt **ausgeführte Operationen**, nie gemessene Eigenschaften.

**REQ-DOC-2 (Formulierungen).** In Dokumentation, Oberflächen, Statustexten und Werbung gelten keine Aussagen wie „vollständig gelöscht“, „garantiert“, „unwiederbringlich“, „DSGVO-konform“, „zertifiziert“ (ADR-45). `tools/doc_lint.py` prüft das in CI.

### 7.11 Datensicherung und Wiederherstellung

**REQ-BAK-1 (zwei Spuren, neu gefasst zu F-35).** Datensicherungen des Datenverzeichnisses enthalten Chiffrat ohne `keys/` und ohne `erasure/` (ADR-42). Schlüsselsicherungen sind eine eigene Spur (`contextra key backup`): `keys/keytable.cxkt` und `keys/anchor.cxan` mit Zeitstempel im Kopf. `key_backup_max_age` (Standard 30 Tage) begrenzt die Gültigkeit. Abgelaufene Sicherungen werden beim Restore verweigert.

**REQ-BAK-2 (Senke, optional).** `erasure.sink` ist ein Port `ErasureSink` (Datei außerhalb von `data_dir`, Objektspeicher mit Aufbewahrungssperre, Syslog). Der Tenant-Koordinator hängt `Gated`- und `KeysDestroyed`-Einträge nach `fdatasync` des Löschlogs an die Senke an, asynchron mit begrenztem Rückstand (`sink_max_lag`, Standard 60 s, Alarm bei Überschreitung).

**REQ-BAK-3 (Restore-Schritt).** `contextra restore --reapply` liest die Senke, vernichtet in der wiederhergestellten KeyTable alle dort als zerstört geführten Seals und Segmentschlüssel erneut, schreibt `RestoreReapplied` ins Ledger und stößt den Scrub an. Erst danach öffnen Lesepfade.

**REQ-BAK-4 (Erkennung).** Ist die Senke weiter als das lokale Löschlog (höhere Sequenz), erkennt der Start einen Restore und verlangt REQ-BAK-3. Ohne Senke ist ein Restore nicht erkennbar. Das ist eine dokumentierte Grenze. Der Betriebsablauf führt `--reapply` nach jedem Restore aus. Optional `restore_without_sink = Quarantine`.

**REQ-BAK-5 (Aufbewahrungshorizont).** `key_backup_retention ≥ restore_horizon`, und die Senke deckt mindestens `restore_horizon` ab. Die Prüfung erfolgt in C0.

### 7.12 Aufbewahrung und Einschränkung

**REQ-RET-1.** Einschränkung (Art. 18, `Restricted`) und Löschung (Art. 17) sind getrennte Pfade. `Restricted` wird vor `Gated` entschieden und ist keine Rücknahme eines Löschauftrags.

**REQ-RET-2.** `set_hold(scope, reason, until)` blockiert E1 für betroffene Seals (Klasse Hold, 7.3). Gesetzliche Aufbewahrungsfristen können je Collection als Regel hinterlegt sein. Ein Auftrag gegen gehaltene Seals wird für diese Seals `Refused` und im Ledger vermerkt.

### 7.13 Verdrahtung in `contextra-db`

Der Tenant-Koordinator ist eine Komponente von `db`. Er hält den Erasure-Lock, die Verweise auf `KeyDestroyer`, `ReadGate` (Impl. `erasure`), `ContentLifecycle` (Empfänger), `ScrubTarget` (Ziele aus `vector`, `text`, `graph`, `store`) und `ErasureSink`. Er ist die einzige Stelle, die diese Ports zusammenführt (ADR-32).

---

## 8. Betrieb

### 8.1 Zustände je Shard und Mandant

| Zustand | Lesen | Schreiben | Löschen | Ursache |
|---|---|---|---|---|
| `Ready` | ja | ja | ja | Normalbetrieb |
| `ReadOnly{grund}` | ja, letzter Snapshot | nein | Auftrag wird angenommen und nach Rückkehr ausgeführt, sofern Log und KeyTable schreibbar sind | `FsyncFailed`, `NoSpace`, `IoError` bei Datenschreibvorgängen |
| `Rebuilding` | nein (`IndexUnavailable`) | ja, in den Store | ja | Index-Segment nicht lesbar (REQ-IDX-REBUILD) |
| `Quarantine` | nein | nein | nein | Löschlog, Gate oder KeyTable inkonsistent, Integrität verletzt |
| `Refused` | nein | nein | nein | Start verweigert (Konfiguration, Format, Schlüssel, mlock) |

Der Zustand gilt je **Shard**, mit Ausnahme von `Quarantine`. Ein Fehler in einem Shard stoppt die anderen nicht. Ist das Löschlog eines Mandanten betroffen, gilt `Quarantine` für alle Collections dieses Mandanten.

**INV-5.** Ein Schreib- oder Löschfehler führt nie zu `Quarantine`, außer er betrifft Löschlog, Gate oder KeyTable. Ein fehlgeschlagener `fsync` bei Daten führt zu `ReadOnly`.

**REQ-OPS-1 (Rückkehr).** `ReadOnly → Ready` erfolgt ohne Neustart nach bestandener Strukturprüfung und einem erfolgreichen Probeschreibvorgang. `Quarantine → Ready` erfordert einen Neustart mit bestandener Prüfung von Log, Gate und KeyTable. Übergänge stehen im Ledger und im Health-Bericht.

**INV-6 (Fail-closed).** Fehlt ein Schlüssel oder ist ein Gate-Zustand unbekannt, liefert der Lesepfad `Erased`, nie Klartext. AEAD-Fehler sind nach außen nicht unterscheidbar.

### 8.2 Fehlerklassen

| Klasse | Beispiel | Reaktion |
|---|---|---|
| `Transient` | I/O-Fehler beim Journalschreiben | Wiederholung mit Backoff, dann `ReadOnly` |
| `Persistent` | `fsync` schlägt wiederholt fehl | `ReadOnly`, Alarm |
| `Integrity` | Prüfsummenfehler in Log, Gate, KeyTable | `Quarantine` des Mandanten |
| `Config` | Ungültige Konfiguration | `Refused` mit vollständiger Fehlerliste |
| `Crypto` | AEAD-Fehler, `KeyAbsent` | einheitlicher Fehler, auf dem Lesepfad `Erased` |
| `Backpressure` | Warteschlange voll | sofortige Abweisung, kein Blockieren |

Jeder Fehler wird mit Art, Mandant, Zeitpunkt und Zustandsübergang protokolliert, bevor die Antwort erfolgt. Protokolle enthalten keine Nutzdaten (T-LOG-01).

### 8.3 Startsequenz

Je Prozess `C0`, `C1` einmal. Je Mandant `C2` bis `C6`. Mandanten und Collections werden **lazy** geöffnet: die erste Anfrage öffnet einen Shard, `eager_open` kann das ändern.

| Phase | Inhalt |
|---|---|
| `C0` | Konfiguration vollständig prüfen (Kapitel 9). Gleichzeitig Parametergrenzen für Argon2 (ADR-16). |
| `C1` | Datenverzeichnis, Prozesssperre, Plattenreserve ≥ `disk_reserve_bytes`. Schlüsselspeicher sperren (mlock, ADR-41), `PR_SET_DUMPABLE = 0`. |
| `C2` | Wurzel und Anker öffnen. Wurzelschlüssel entwrappen. |
| `C3` | **Löschlog und Gate-Snapshot wiedergeben**, KeyTable auf REQ-REC-2 prüfen. Erst danach sind Lesepfade dieses Mandanten zulässig (REQ-OPS-2). |
| `C4` | Offene Löschaufträge fortsetzen (7.7). |
| `C5` | Store: Manifest, WAL, Prüfsummen der Store-Dateien. Registry lesen. |
| `C6` | Index laden: `vector_struct` und `vector_code` parallel, `text`-Segmente, `graph`-Segmente. LiveMask aus Registry, Visibility und Gate aufbauen. Zustand `Ready`, `Rebuilding` oder `ReadOnly` setzen. |

**REQ-OPS-2.** Vor `C3` liefert jede Anfrage `Erased` oder einen typisierten Fehler. `C6` läuft nur, wenn `C3` und `C4` bestanden sind.

### 8.4 Nebenläufigkeit

**REQ-CON-1 (Schreib-Task je Shard).** Jeder Shard (Mandant × Collection) hat genau einen Schreib-Task. Er serialisiert Ingest-Stapel, Versionsänderungen und Graph-Deltas. Ein Ingest-Stapel ist auf 4 MiB oder 5 ms Arbeitszeit begrenzt, damit ein Löschbefehl höchstens einen Stapel wartet (T-OPS-08).

**REQ-CON-1a (Mandanten-Koordinator, neu zu F-18).** Der Tenant-Koordinator besitzt den **Erasure-Lock** des Mandanten. Er führt E1 bis E4 aus (7.4) und sendet Befehle an die Shard-Tasks. Gate-Publikation braucht den Erasure-Lock.

**REQ-CON-1b (Commit-Lock, neu zu F-19).** Ingest-Commits halten den **Commit-Lock** des Mandanten als Leser. Der Erasure-Lock nimmt den Commit-Lock als Schreiber, bevor er Gate-Änderung und Registry-Scan (E1) ausführt. Dadurch sind zwei Fälle ausgeschlossen: Ein Ingest prüft die Unterdrückungsliste vor der Löschung und committet danach einen Seal mit dem unterdrückten Tag, oder die Löschung scannt die Registry vor dem Commit eines Seals, dessen Tag sie erfassen müsste. Die Dauer des Schreiber-Locks ist auf eine Gate-Publikation begrenzt (T-SEC-03).

**REQ-CON-2 (Snapshots).** Leser arbeiten auf unveränderlichen Snapshots (`ArcSwap`). Schreiber veröffentlichen erst nach der Dauerhaftigkeit (Gruppen-Commit).

**REQ-CON-3 (Gruppen-Commit).** Ingest-Schreibvorgänge teilen sich einen `fdatasync` innerhalb von `group_commit_window` (Standard 2 ms oder 4 MiB). Löschaufträge nutzen ein eigenes Fenster von 0 bis 1 ms.

**REQ-CON-4 (Einzelausführung).** Pro Scope höchstens eine Löschung. Ein Duplikat liefert das bestehende Ticket. Ein Scope `Document` und ein Scope `Subject` mit überlappenden Seals laufen seriell im Erasure-Lock.

**REQ-CON-6 (Schreibstapel).** Ein Schreibstapel (ein oder mehrere Dokumente, Graph-Deltas, Versionsänderungen) wird atomar veröffentlicht: Store, Registry, Index und Versionskatalog erscheinen für Leser gemeinsam mit einem Snapshot-Tausch nach dem Gruppen-Commit (Reihenfolge 7.4). Ein Absturz vor dem Commit lässt keinen sichtbaren Teilzustand zurück.

**REQ-CON-7 (Schlüssel-Cache).** Der Schlüssel-Cache hat einen Lese-Schreib-Lock. Leser (Entschlüsselung) halten den Read-Lock während eines `open`. `KeyDestroyer` nimmt den Write-Lock (REQ-KEY-4).

**REQ-CON-5 (Rechenpool).** Ein gemeinsamer Rechenpool (`compute_threads`, Standard Kerne − 2) bedient Embedding-Nachbearbeitung, Indexaufbau, Merge und Scrub. Suchen laufen auf eigenem Suchpool (`search_threads`).

### 8.5 Ressourcen

**REQ-RES-1 (Speicherbudget).** `memory_budget_bytes` begrenzt Memtable, Blockcache, Textspeicherindex und Zwischenspeicher. SQ8-Codes und Graph zählen zum Index und werden beim Start gegen das Budget geprüft (`Refused` mit Rechnung, wenn sie nicht passen). Überschreitung löst Flush oder Evict aus, nie Abbruch.

**REQ-RES-2 (Shard-Verdrängung).** Ungenutzte Shards werden nach `shard_idle_evict` (Standard 30 Minuten) entladen, sofern das Budget es erfordert, nie ein Shard mit laufendem Auftrag.

**REQ-RES-3 (Warteschlangen).** Alle Warteschlangen sind begrenzt. Bei Erreichen: `Backpressure`.

**REQ-RES-4 (Limits).** Trefferzahl, Dokumentgröße, Chunks je Dokument, Stapelgröße, Entitäts- und Relationsgrenzen stehen einmal in `contextra-ids` und sind über Konfiguration veränderbar, soweit die Konfiguration das vorsieht (9).

**REQ-RES-5 (Löschlast).** Der Löschpfad darf die Suchlatenz im Normalfall nicht messbar verändern (Budget 10.3).

### 8.6 Zeit und Determinismus

**REQ-TIM-1.** Alle Zeitquellen laufen über `Clock`. Direkte Aufrufe von `Instant::now` und `SystemTime::now` sind außerhalb der `Clock`-Implementierungen in `contextra-env` verboten (G-12).

**REQ-TIM-2.** Drosselung nutzt `Clock` und ist mit `ManualClock` testbar.

**REQ-TIM-3.** Zeitgrenzen im Produkt: `deadline` der Suche (kooperativ, Fehler statt Teilergebnis), Scrub-Fenster und -Frist, Fristen für Review, HTTP-Zeitgrenzen im Adapter. Es gibt keine Zeitbudgets, die Ranking oder Kandidatenmenge verändern.

**ADR-43 (Latenzfenster).** `contextra-db` führt ein Fenster mit 64 logarithmischen Buckets, Schritt 1 s, Gesamtlänge 10 s, lock-frei. Der Scheduler berechnet p95 jede Sekunde. Dieses Fenster ist die einzige Quelle für die Scrub-Pause. Es ist keine Messung für einen Nachweis.

**INV-7 (Determinismus).** Siehe REQ-SRC-2.

### 8.7 Betrieb über Prozessgrenzen

- Ein Prozess hält die Sperre auf `data_dir`. Mehrere Prozesse teilen sich ein Verzeichnis nie.
- Skalierung geschieht über Mandanten-Aufteilung auf mehrere Prozesse durch die Anwendung. Hochverfügbarkeit ist ausgeschlossen (ADR-46).
- Metriken: `MetricsSink` schreibt lokal. Export (Prometheus, OpenTelemetry) liegt in `contextra-server`.

---

## 9. Konfiguration

Die Konfiguration wird vollständig in `C0` geprüft. Alle Fehler werden gesammelt gemeldet (`Refused`). Konfiguration ist die einzige Quelle für Principals, Berechtigungen und Vokabulare (ADR-38, REQ-GR-2).

```rust
pub struct ContextraConfig {
    pub data_dir: PathBuf,
    pub root: RootConfig,                    // Passphrase | KeyFile(path) | VaultTransit(...)
    pub embedding: Option<EmbeddingConfig>,  // Backend, Modelldatei, SHA-256, dim
    pub index: IndexConfig,                  // m, ef_construction, ef_search, index_flat_below, tombstone_rebuild_ratio
    pub text: TextConfig,                    // k1, b, stemmer, stoppwörter
    pub ingest: IngestConfig,                // chunk_target, min_tokens, Formate
    pub filter: FilterConfig,                // Felder je Collection, filter_exact_max
    pub versions: VersionConfig,             // je Collection: retention, index_history, on_delete
    pub graph: GraphConfig,                  // je Collection: enabled, Vokabulare, Limits
    pub access: AccessConfig,                // Principals, Rollen, erasure_admin, Zuordnung von Claims
    pub resources: ResourceConfig,           // memory_budget_bytes, compute_threads, search_threads, ...
    pub erasure: ErasureConfig,
}

pub struct ErasureConfig {
    pub scrub_window: Duration,              // 10 min
    pub scrub_batch_max_seals: u32,          // 10_000
    pub scrub_deadline: Duration,            // 24 h, Minimum 15 min
    pub scrub_threads: u16,                  // 2
    pub scrub_io_limit: u64,                 // 64 MiB/s
    pub scrub_pause_p95_ms: u32,             // 40
    pub compaction_after_erase: bool,        // true
    pub epoch_len: Duration,                 // 30 Tage
    pub multi_subject: MultiSubjectPolicy,   // Restrict | DeleteAll | RedactReingest, Standard Restrict
    pub undeclared_hits: UndeclaredPolicy,   // Report | Restrict, Standard Report
    pub four_eyes: bool,                     // false
    pub restricted_read: RestrictedRead,     // Deny | Audit, Standard Deny
    pub review_remind_after: Duration,       // 14 Tage
    pub review_escalate_after: Duration,     // 30 Tage
    pub discovery_max_candidates: u32,       // 10_000
    pub sink: Option<SinkConfig>,
    pub sink_max_lag: Duration,              // 60 s
    pub key_backup_max_age: Duration,        // 30 Tage
    pub key_backup_retention: Duration,      // 400 Tage
    pub restore_horizon: Duration,           // 30 Tage
    pub checkpoint_retention: Duration,      // 7 Tage
    pub disk_reserve_bytes: u64,             // ≥ 1 GiB
}
```

**Standardwerte der wichtigsten Schalter (vollständig, keine stillen Defaults):**

| Schalter | Standard | Grenze |
|---|---|---|
| `index_flat_below` | 20.000 | ≥ 1.000 |
| `filter_exact_max` | 20.000 | ≥ 1.000 |
| `rerank_input` | 50 | ≤ 200 |
| `rerank_k` | `min(4·k_cand, 4000)` | nicht konfigurierbar |
| `key_cache_entries` | 65.536 | ≥ 1.024 |
| `chunk_text_cache_bytes` | 0 | ADR-39 |
| `max_records_per_seal` | 2^24 | nicht konfigurierbar, nur Test |
| `max_chunks_per_segkey` | 2^24 | nicht konfigurierbar, nur Test |
| `allow_unlocked_keys` | false | nur Entwicklung (ADR-41) |
| `graph.enabled` (je Collection) | false | ADR-21 |
| `graph.max_hops` | 3 | ≤ 3 |
| `graph.max_expansions` | 10.000 | ≤ 100.000 |
| `graph.max_degree` | 512 | ≤ 4.096 |
| `access.default` | deny | nicht änderbar |

**Prüfungen in `C0`:**

- `data_dir` vorhanden, beschreibbar, nicht auf Netzwerkdateisystem ohne ausdrückliche Option.
- Argon2-Parameter: m ≥ 64 MiB, t ≥ 3, p ≥ 1. Keine Laufzeitmessung (ADR-16).
- `KeyFile` mit Rechten 0600.
- `scrub_deadline ≥ 15 min`. `key_backup_retention ≥ restore_horizon`. `checkpoint_retention ≥ 1 Tag`.
- `embedding.model_sha256` stimmt mit der Datei überein.
- Verschlüsselung für P- und D-Collections ist nicht abschaltbar. `encryption = none` wird abgewiesen.
- `memory_budget_bytes` ≤ verfügbarer Speicher minus Reserve.
- `index_history = true` nur mit ausdrücklicher Bestätigung der Kosten.
- `graph.enabled = true` nur bei aktivem Feature `graph` und definiertem Typvokabular.
- Principals: jeder Principal in Rollen existiert. `erasure_admin` ist nicht leer, wenn Löschoperationen freigegeben sind.
- mlock gelingt, sonst `Refused` (außer `allow_unlocked_keys`).

---

## 10. Leistungsziele

Startwerte. **INV-8:** Ein Wert gilt erst als erreicht, wenn der Benchmark ihn misst und `benchmarks/budgets.toml` ihn enthält (Anhang D). Messhardware laut 1.2.

### 10.1 Suche

| Größe | S (1 Mio.) | L (10 Mio.) | Bedingung |
|---|---|---|---|
| Hybridsuche p50 | ≤ 15 ms | ≤ 30 ms | k = 10, ohne Filter, ohne Reranker, ohne Graph |
| Hybridsuche p95 | ≤ 40 ms | ≤ 80 ms | gleiche Bedingung, Last bei 50 % der Spitze |
| Mit Filter (Selektivität 10 %) p95 | ≤ 1,5 × ungefiltert | ≤ 1,5 × ungefiltert | ACORN-Pfad |
| Mit Graph (`PathRag`) p95 | ≤ 25 ms zusätzlich | ≤ 50 ms zusätzlich | 3 Hops, `max_expansions` |
| Rückruf@10 gegen exakt | ≥ 0,95 | ≥ 0,95 | Referenzdatensatz |
| Durchsatz | ≥ 200 QPS | ≥ 100 QPS | 16 Kerne |
| RAM je Chunk (ohne Graph) | ≤ 1,5 KB | ≤ 1,5 KB | 768 Dimensionen, ohne Blockcache |
| Kaltstart bis `Ready` | ≤ 10 s | ≤ 60 s | lazy Shards, Segmente parallel |

### 10.2 Ingest

| Größe | Ziel | Bedingung |
|---|---|---|
| Store und Index | ≥ 2.000 Chunks/s je Shard | ohne Embedding |
| Chunker | ≥ 5 MB Text/s je Kern | |
| Bulk-Modus | ≥ 1,5 × normaler Ingest | ADR-18 |
| Graph-Ingest | ≥ 20.000 Relationen/s je Shard | nur bei `graph.enabled` |

### 10.3 Löschung und Kosten der Löschung

| Größe | Ziel |
|---|---|
| Auftrag bis Gate-Publikation (E1), bis 1.000 Seals | ≤ 5 ms p99 |
| Auftrag bis Schlüsselvernichtung (E2), bis 1.000 Seals | ≤ 1 s p99 |
| Mandanten-Offboarding (Schlüsselarbeit) | ≤ 100 ms |
| Scrub-Abschluss | ≤ `scrub_deadline` |
| Suchlatenz p95 mit Seal- und Gate-Schicht gegen Basislinie | ≤ +3 % |
| Ingest-Durchsatz mit Seal-Schicht gegen Basislinie | ≥ 95 % |
| Suchlatenz p95 während Scrub gegen ruhig | ≤ +5 % (Pausenregel REQ-SCR-4 erfüllt das nur bis zur Frist, danach gilt die Frist) |
| RAM-Aufschlag der Schicht (4 B `seal_of` je Chunk, 1 Bit LiveMask) | ≤ 1 % |

**Basislinie.** Derselbe Build mit `bench_baseline = true` (nur im Bench-Crate, Seal-Schicht als Durchleitung). Die Basislinie existiert nur zur Messung der Mehrkosten und hat keinen Produktionspfad.

### 10.4 Messmethodik

Jeder Wert hat Datensatz (`benchmarks/data/de_mini` plus synthetisch für S und L, `benchmarks/data/graph_gold` für den Graph), Hardware, Warmlauf, Wiederholungen und Konfidenzangabe. Die Messung ist ein CI-Lauf (nächtlich), kein Laufzeitmerkmal des Produkts (ADR-02).

---

## 11. Tests und Gates

Die Prüfung der Löschung liegt in Testcode. Ein Test darf das Datenverzeichnis nach bekannten Klartextmustern durchsuchen. Diese Fähigkeit existiert nur in `contextra-testkit` und nie im Produkt. Die Testkit-Dienste (FaultVfs, Canary-Scanner, Mock-Uhr, Mock-Store) liegen in `contextra-testkit`. Die bisherigen Fehlerinjektionen in `durable-fs` und `store` entfallen (REM-13).

### 11.1 Löschtests

| ID | Prüft |
|---|---|
| T-ER-01 | Nach Ingest und Flush: kein Klartext-Canary der P-Collection in den Datenverzeichnisdateien (Testwerkzeug) |
| T-ER-02 | Nach E1: Suche, `get`, `assemble` liefern den Inhalt nicht. Auch vor E2 |
| T-ER-03 | Nach E2: `open` liefert `KeyAbsent`. SQ8-Code des toten Knotens enthält den ursprünglichen Code nicht |
| T-ER-04 | Neustart nach jedem Zustand: Gate bleibt, Aufträge setzen fort (Absturzmatrix an jedem `fdatasync`-Punkt, mit `FaultVfs`) |
| T-ER-05 | Halb geschriebener Slot: korrekte Behandlung nach Tabelle REQ-REC-2 |
| T-ER-06 | Zwei gleichzeitige Aufträge, gleicher Scope: ein Ticket |
| T-ER-07 | Gleichzeitiger Ingest und Erase desselben Betroffenen: serialisiert nach REQ-CON-1b, kein Seal bleibt unerfasst (Schedule-Exploration mit `loom`) |
| T-ER-08 | Jeder Lesepfad prüft das Gate: Suche (Vektor, Text), `get`, Zeitreise, Graph, Export, `assemble`, Ingest-Duplikat |
| T-ER-09 | Jeder Zwischenspeicher implementiert `SealScoped` und wird bei E2 geleert (Registrierungstest über alle Cache-Typen) |
| T-ER-10 | Zeitreise auf gelöschten Seal: `NotFound` ohne Sonderregel (REQ-VER-6) |
| T-ER-11 | **Entkopplung:** Löschung ändert den Versionskatalog nicht. Versions-GC ohne `erasure`-Crate kompiliert und läuft. `erasure` ohne `version`-Crate kompiliert und läuft |
| T-ER-12 | `ContentRetired` vernichtet den Seal mit Grund `Retention` |
| T-ER-13 | Mehrbetroffene: Standard `Restrict`, keine automatische Löschung. `RedactReingest` legt neuen Seal an, alter wird erst danach vernichtet |
| T-ER-14 | Unterdrückungsliste verhindert Wiederaufnahme (`SubjectSuppressed`), auch bei Mischung mit nicht unterdrückten Tags |
| T-ER-15 | `Hold` blockiert E1 für betroffene Seals |
| T-ER-16 | Mandanten-Scope: nach `destroy_tenant` ist keine `DocKey` mehr entwrappbar, alle Lesevorgänge `Erased` |
| T-ER-17 | Epochenwechsel: gemischte Epochen lesbar, alte KEK erst nach vollständigem Rewrap entfernt, Absturz mitten im Rewrap |
| T-ER-18 | Scrub-Stapel: jedes betroffene Segment einmal neu geschrieben. Frist eingehalten (mit `ManualClock`) |
| T-ER-19 | Scrub pausiert bei p95-Überschreitung, läuft nach Fristablauf trotzdem |
| T-ER-20 | Konsolidierung: Rückruf des Graphen nach Löschung von 10 % der Knoten ≥ Ziel |
| T-ER-21 | Restore mit alter KeyTable und Senke: Seals werden erneut vernichtet. Ohne Senke und mit `restore_without_sink = Quarantine`: `Quarantine` |
| T-ER-22 | K-Klasse: kein Schlüsselschritt, Segmentersatz, Ledger vermerkt `no_key_destruction` |
| T-ER-23 | Ledger enthält keine Kennungen, Namen oder Inhalte (Property-Test mit Canary-Namen) |
| T-ER-24 | `erase_status` und Ledger enthalten keine Wörter der verbotenen Liste (7.10, `doc_lint`) |
| T-ER-25 | Review-Frist: Erinnerung und Eskalation nach Konfiguration, keine automatische Aktion (REQ-ER-6) |
| T-ER-26 | Segmentschlüssel: Vernichtung erst nach Veröffentlichung des neuen Segments (T-SEG-02) |

### 11.2 Betriebstests

| ID | Prüft |
|---|---|
| T-OPS-01 | `fsync`-Fehler bei Daten: `ReadOnly`, Suche liefert weiter. Rückkehr nach Probeschreibvorgang |
| T-OPS-02 | Fehler im Löschlog oder KeyTable: `Quarantine` nur für den Mandanten |
| T-OPS-03 | Ungültige Konfiguration: `Refused` mit vollständiger Fehlerliste |
| T-OPS-04 | Speicherbudget erreicht: Flush, kein Abbruch |
| T-OPS-05 | Warteschlange voll: `Backpressure`, kein Blockieren |
| T-OPS-06 | Gleiche Anfrage unter verschiedener Last: identische Trefferliste |
| T-OPS-07 | Netzfreier Lauf des Kernbaums (`unshare -n`) |
| T-OPS-08 | Löschauftrag wartet höchstens einen Ingest-Stapel (REQ-CON-1) |
| T-OPS-09 | mlock nicht möglich: `Refused` (außer `allow_unlocked_keys`) |
| T-OPS-10 | `Rebuilding`-Zustand: `IndexUnavailable`, Schreiben in den Store läuft, Neuaufbau erzeugt identischen Rückruf (T-IDX-01) |

### 11.3 RAG-Tests

| ID | Prüft |
|---|---|
| T-RAG-01 | Chunker: Offsets zeigen auf den Quelltext, Golden-Dateien |
| T-RAG-02 | Parser: Beispiele je Format, defekte Dateien brechen den Stapel nicht ab, Fuzz-Ziele |
| T-RAG-03 | Hybridsuche: Rückruf gegen Referenzdatensatz `de_mini`, mit und ohne Stemming |
| T-RAG-04 | HNSW gegen exakt: Rückruf@10 ≥ Ziel, mit und ohne Filter |
| T-RAG-05 | Filterplaner wählt Pfad nach Zählwerten, Ergebnis gleich bei beiden Pfaden |
| T-RAG-06 | ACL: Ergebnis enthält nie Chunks ohne passenden Tag (Property-Test) |
| T-RAG-07 | Kontextbudget: Tokensumme ≤ Budget, Quellen vollständig, Tokenschätzung nach 6.1 |
| T-RAG-08 | Modellwechsel wird abgewiesen (`ModelMismatch`) |
| T-RAG-09 | Breadcrumb-Vorsatz verbessert Rückruf auf dem Kontextpräfix-Szenario |
| T-RAG-10 | Idempotenter Ingest (REQ-ING-8), Wiederholung eines abgebrochenen Bulk-Jobs |
| T-RAG-11 | Unterdrückungs-Race: REQ-CON-1b unter Schedule-Exploration |

### 11.4 Sicherheits-, Verschlüsselungs- und Graphtests

| ID | Prüft |
|---|---|
| T-ENC-01 | Canary-Scan des Datenverzeichnisses nach Ingest, Flush, Scrub: null Treffer für P- und D-Klasse (INV-9) |
| T-ENC-02 | Format-Prüfung: keine unverschlüsselten Dateien unter `tenants/` außer Kopfdaten nach 5.5, 5.6, 5.8 |
| T-ENC-03 | Obergrenzen `max_records_per_seal` und `max_chunks_per_segkey` werden erzwungen (Testparameter klein) |
| T-LOG-01 | Canary in Ingest, Suche und Löschung: kein Klartext in Tracing- und Metrikausgabe |
| T-SEC-01 | Principals nur aus Konfiguration: Anfrage mit fremdem Principal-Feld wird ignoriert |
| T-SEC-02 | `erasure_admin` erforderlich. Versuch ohne Berechtigung ist `Refused` und im Ledger als `AuthDenied` |
| T-SEC-03 | Ingest und Erase gleichzeitig: Commit-Lock serialisiert, kein Seal mit unterdrücktem Tag entsteht (Schedule-Exploration) |
| T-SEC-04 | MCP-Tool-Argument mit Principal-Feld wird ignoriert |
| T-SEG-01 | Segment-Umschlag Roundtrip, falscher `seg_ord` führt zu `Integrity` |
| T-SEG-02 | Scrub-Reihenfolge: neues Segment veröffentlicht, bevor der alte `SegKey` vernichtet wird (Trace) |
| T-SEG-03 | Segment ohne `SegKey`: `IndexUnavailable` und Zustand `Rebuilding` |
| T-KEY-01 | Slot-Layout 128 Byte, Offsets und CRC (Fixture, Golden-Datei) |
| T-KEY-02 | Vernichtungsstapel: ein `fdatasync`, Offsets sortiert (Zähler im FaultVfs) |
| T-KEY-03 | Cache-Lock: parallele Leser während Vernichtung, danach kein Schlüssel des Stapels im Cache |
| T-KEY-04 | Anker-Atomarität: Absturz vor und nach `rename` |
| T-KEY-05 | Wurzelrotation ändert nur den Anker |
| T-KEY-06 | Argon2-Parametergrenzen: zu schwach ist `Refused`, im Startpfad kein Aufruf von `Clock` für Schlüsselarbeit |
| T-GR-01 | Keine Klartextlabels in Dateien (Canary über Graph-Segmente) |
| T-GR-02 | PathRAG deterministisch: gleiche Pfade bei Permutation der Eingabereihenfolge |
| T-GR-03 | Löschung eines Seals: Kanten mit ausschließlicher Provenienz sofort unsichtbar, nach Scrub entfernt. Entitäten ohne Provenienz entfernt |
| T-GR-04 | Kante mit zwei Provenienzen überlebt die Löschung eines der beiden Seals |
| T-GR-05 | Konsolidierung und Neuaufbau: Rückruf der Traversierung nach 10 % Löschung ≥ Ziel |
| T-GR-06 | ReadGate wird bei jeder Kantenmaterialisierung geprüft (instrumentierter Zähler) |
| T-GR-07 | Snapshot-Isolation: Traversierung auf einem gepinnten Snapshot sieht keine späteren Änderungen |
| T-GR-08 | Graph-Änderung im Schreibstapel: atomar bei Absturz (Crash-Matrix) |
| T-GR-09 | Persistenzformat Roundtrip mit CRC. Beschädigte Datei führt zu `Integrity` |
| T-GR-10 | Gate G-GRAPH (11.5) |
| T-GR-11 | `as_of` auf Graph-Operationen liefert `HistoryNotIndexed` |
| T-GR-12 | Build ohne Feature `graph`: `db` enthält keinen Graph-Code (Symbolprüfung) |
| T-IDX-01 | Neuaufbau aus `VectorFull` lebender Seals liefert Rückruf ≥ Ziel gegen Vorzustand |

### 11.5 Entscheidungs-Gates (Verbleib von Modulen)

| Gate | Bedingung | Konsequenz bei Verfehlung |
|---|---|---|
| **G-GRAPH** | Auf `benchmarks/data/graph_gold` (≥ 500 Mehrschritt-Fragen, in M4b erstellt): nDCG@10 hybrid+graph ≥ hybrid + 0,03, p95 ≤ 1,5 × hybrid, Ingest-Durchsatz nach 10.2 | Feature `graph` und Crate werden in M5 entfernt (REM-31) |
| **G-INDEX** | Rückkehr von DiskANN oder RaBitQ nur bei einem Bestand, den HNSW im RAM-Budget nicht hält (> 50 Mio. Chunks je Prozess), mit Rückruf- und Latenzbenchmark | Verboten bis dahin |
| **G-STORE** | Die eigene LSM bleibt, wenn sie in M1 auf Messhardware erreicht: Punktzugriff p99 ≤ 1 ms (warm), ≥ 50.000 Schreibvorgänge/s im Gruppen-Commit, Wiederanlauf nach Absturz in ≤ 10 s bei 10 GB | Sonst Ersatz hinter dem Port `KvStore` durch `redb` oder `fjall`. Entscheidung anhand der Messung |

### 11.6 Build-Gates

| ID | Gate |
|---|---|
| G-01 | `cargo check --workspace`, `cargo test --workspace`, `cargo test --no-default-features` pro Crate, `cargo clippy --workspace -- -D warnings` grün. Toolchain 1.89.0 laut `rust-toolchain.toml` |
| G-02 | `unsafe_code = "forbid"` in allen Crates außer `contextra-sys` und `contextra-simd` (gemessen: nur dort Produktcode mit `unsafe`). Test-Targets mit eigenem Allocator liegen in `tests/unsafe_islands.rs` (ADR-53) |
| G-03 | Schichtprüfung R1, R2, R7, R9 über `cargo metadata` gegen Anhang E (`crates/contextra-testkit/tests/layering.rs`). Datei wird in M0 angelegt |
| G-04 | Genau eine Definition von `MAX_SEARCH_K` (`grep -rn "const MAX_SEARCH_K" crates \| wc -l` = 1) |
| G-05 | `cargo tree -e normal` der Kern-Crates enthält keine Netzwerk-Crates (Sperrliste: `reqwest`, `hyper`, `ureq`, `tonic`, `h2`, `native-tls`, `openssl`, `rustls`) |
| G-06 | `default = []` in jedem Crate außer den in R3 begründeten Ausnahmen |
| G-07 | Keine Erwähnung entfernter Crates oder Symbole im Produktcode (Liste 12) |
| G-08 | Dokumentationslint (REQ-DOC-2 und REQ-DOC-3) |
| G-09 | Keine Selbstdeklarationsdateien (REM-14): `grep -rln "ANCHOR\[\|FILE-CONTEXT"` leer, keine `contract.rs`, keine `INVENTAR.md`/`SPEC.md`/`AGENTS.md` in `crates/` |
| G-10 | `benchmarks/budgets.toml` vorhanden, Verweise auf fehlende Dateien sind Fehler |
| G-11 | Compat-Auflösung: `find crates benchmarks -type d -name compat` leer. `grep -rn "compat\|workspace\|types-compat\|ports-compat" crates/*/Cargo.toml` leer, außer dem Feature-Namen `workspace` in keinem Crate |
| G-12 | `grep -rn "Instant::now\|SystemTime::now" crates --include=*.rs` außerhalb `crates/contextra-env/` leer |
| G-13 | `grep` auf `unwrap()`, `expect(`, `panic!(`, `todo!(`, `unimplemented!(`, `unreachable!(` in Produktcode außerhalb `#[cfg(test)]` leer. Clippy-Lints auf `deny` |
| G-14 | CI-Lauf von T-ENC-01 und T-LOG-01 grün |

---

## 12. Entfernungen und Compat-Auflösung

Je Entfernung ein Commit. Nach jedem Commit `cargo check --workspace`. Keine Verhaltensänderung im selben Commit. Pfade sind am HEAD `2fd0994` geprüft (Anhang B). Abnahme ist der angegebene Befehl.

### 12.1 Robotik, Profile, Lizenz

| ID | Entfernen | Ort | Abnahme |
|---|---|---|---|
| REM-01 | Eintrag `crates/contextra-vault` in `[workspace] exclude` (Verzeichnis existiert nicht mehr) | `Cargo.toml` | `grep -n "contextra-vault" Cargo.toml` leer |
| REM-02 | Profile `BareMetal`, `EdgeMinimal`, `PowerUserLocal`, Modi `MemoryOnly`, `WalNoHmac`, Löschmodus `TombstoneOnly` | `contextra/src/performance_profile.rs`, `contextra/src/collection_profile.rs`, `contextra-store/src/lsm/ops/write/`, `contextra-crypto/src/kv_segment/delete_mode.rs`, MCP-Schema | `grep -rn "BareMetal\|MemoryOnly\|WalNoHmac\|TombstoneOnly\|EdgeMinimal"` leer |
| REM-03 | Lizenzreste: `LicenseGate`, `FeatureRing`, `VectorDeleteMode` | `contextra-ports/src/license.rs`, `contextra-ports/src/builder.rs` | `grep -rn "License\|FeatureRing\|BackgroundRepair"` leer |
| REM-04 | Hardware-Sensorik und Anti-Tamper | `contextra-crypto/src/anti_tamper.rs` | Datei fehlt |
| REM-05 | Observer-Subsystem mit Latenzfrist und Circuit-Breaker | `contextra-store/src/lsm/observer/`, `contextra-store/src/lsm/observer.rs` | Verzeichnis und Datei fehlen |
| REM-06 | Zeitbudget im Ranking | `contextra-rank/src/dibud/` | Verzeichnis fehlt |
| REM-07 | `thread::sleep`-Drosselung ohne `Clock` | `contextra-durable-fs/src/scrub.rs` | nur `Clock`-Drosselung (G-12) |
| REM-08 | Kostenbasierte Kompaktionsplanung | `contextra-store/src/compaction/adaptive/` | einfache Größenschwelle |

### 12.2 Eigennachweise und Altlöschung

| ID | Entfernen | Ort | Abnahme |
|---|---|---|---|
| REM-09 | DiskANN, RaBitQ, `partial_rebuild.rs`, `purge.rs` als eigener Pfad, Feature `experimental` | `contextra-vector/src/diskann/`, `quantize_rabitq.rs`, `partial_rebuild.rs`, `purge.rs` | Dateien fehlen. `grep -rn "diskann\|rabitq"` leer außer in Entfernungsnachweis |
| REM-10 | Altes Erasure: `vault/`, `types_local.rs`, `crypto_local.rs`, `chain/saga.rs` (Saga 2.0), `gate/saga_gate.rs`, `Armed`, `ControlEvidence`, `MeasureClass`, `KeyCoverage`, `LayerWitness`, `CommitBarrier`, `BarrierExclusive`. Crate wird nach Kapitel 7 neu aufgebaut | `contextra-erasure/src/` | `pub mod vault`, die Typen und Dateien fehlen |
| REM-11 | Messen, Signieren, Berichten: `audit_chain.rs`, `wal_completeness.rs`, `wal_crypto.rs`, `kv_segment/`, `kv_cipher.rs`, `dek.rs` (ersetzt durch `DocKey`-Modul) | `contextra-crypto/src/` | Dateien fehlen. Kein `ed25519`, kein JCS im Workspace |
| REM-12 | SSI, TxBuffer, ReadSet | `contextra-mvcc/src/ssi.rs`, `contextra-mvcc/src/tx_buffer.rs` | Dateien fehlen |
| REM-13 | `Poisoned`, Fault-Injection im Produktcode | `contextra-durable-fs/src/health.rs`, `contextra-durable-fs/src/fault_injector.rs`, `contextra-store/src/io_seam.rs`, `contextra-store/src/io_seam/` | Fault-Injection nur in `contextra-testkit` |
| REM-14 | Selbstdeklarationen: `contract.rs` (9 Dateien), `ANCHOR`-Kommentare (460 Dateien), `FILE-CONTEXT` (417 Dateien), `INVENTAR.md`, `SPEC.md`, `AGENTS.md` pro Crate, Auditor-Skripte (`port_contract_auditor.py`, `contract_diff_detector.py`, `type_registry_sync.py`, `isolation_proof.py`, `erasure_leak_prober.py`, `fsync_durability_tracer.py`, `error_topology_mapper.py`, `concurrency_lock_auditor.py`, `unsafe_boundary_tracer.py`, `crate_inventory.py`, `generate_function_inventory.py`, `generate_interface_spec.py`, `dead_code_microprobe.py`, `feature_audit.py`, `abhaengigkeits_budget.py`, `gate_dependencies.py`, `test_matrix_advisor.py`, `orchestrate_diagnostics.py`, `pattern_diagnostic.py`, `agentic_context_pack.py`, `rust_ast_inspector.py`, `async_boundary_scanner.py`, `zero_panic_analyzer.py`, `determinism_guard.py`, `contract_diff_detector.py`) | `crates/*/src/`, `crates/*/`, `tools/` | G-09 grün. `tools/` enthält nur `doc_lint.py`, `loc.py`, `test_coverage_matrix.py`, `baseline/` und die Skripte aus 11.6 |
| REM-15 | Alle `compat/`-Module und `compat.rs`. Umfang 7.465 Zeilen, Auflösung nach 12.5 | siehe 12.5 | G-11 grün |
| REM-16 | Altmodell-Texte: Berichte, Bescheinigung, Satzbibliothek, Annahmen- und Residuenkataloge | `contextra-erasure`, Dokumente | `grep -rn "Bescheinigung\|Satzbibliothek\|A-FLASH\|R-GROUP-LINK"` leer außer in 7.9 |
| REM-17 | Feature `experimental` und alle Feature-Namen mit Präfix `exp-` in allen Crates (`exp-dibud`, `exp-resonance`, `experimental-rank`, `experimental-graph`) | `Cargo.toml` aller Crates | `grep -rn "experimental\|exp-" crates --include=Cargo.toml` leer |

### 12.3 Funktionsumfang

| ID | Entfernen | Ort | Abnahme |
|---|---|---|---|
| REM-18 | Kalibrierung, Resonanz-, Ext-, Global-, Normalized-Fusion | `contextra-rank/src/calibration/`, `contextra-rank/src/fusion/{resonance,ext,global,normalized}.rs` | Dateien fehlen |
| REM-19 | Verweise auf `contextra-engine` in Kommentaren und Dokumenten. Das Crate selbst fehlt bereits (F-38) | `contextra-mvcc/src/lib.rs`, `contextra-testkit/src/lib.rs`, `contextra-checkpoint/src/lib.rs`, `contextra-text/src/lib.rs`, `contextra-rank/src/lib.rs`, `contextra-rank/src/temporal.rs`, `contextra-rank/tests/legacy_reference/`, `contextra-crypto/src/lib.rs`, `contextra-graph/src/entity_extraction.rs`, `contextra-types/src/types/saos.rs`, `crates/contextra/PLAN.md`, `crates/contextra-db/PLAN.md` | `grep -rn "contextra-engine\|contextra_engine"` leer |
| REM-20 | LLM-Klassifikation, Egress-Vault, Egress-Gateway, Bulk-Exfiltration, Kontext-Edit-Audit | `contextra-privacy/src/{egress_vault,egress_gateway,bulk_exfiltration_detector,context_edit_audit}.rs` | Dateien fehlen. Verbleibend: `entity_recognizer`, `data_class`, `egress_guard`, `processing_registry`, `guarded_payload` |
| REM-21 | MCP-Sandbox, Prompt-Injection-Stub, Reexport-Modul | `contextra-mcp/src/{sandbox,prompt_injection,bulk_exfiltration_detector}.rs` | Dateien fehlen |
| REM-22 | Ollama-, Candle- und ONNX-Konfiguration ohne Implementierung | `contextra-mcp/src/config.rs` (Datei bleibt, Schlüssel entfallen) | ersetzt durch 9 und REQ-EMB-2 |
| REM-23 | Duplizierte Distanzkerne in `contextra-vector/src/distance/` (nur noch Dispatch auf `contextra-simd`), Abhängigkeit `simd → types` | `contextra-vector/src/distance/`, `contextra-simd/Cargo.toml` | ein Kernel-Satz |
| REM-24 | Agentengedächtnis: `contextra/src/agent_memory.rs`, Zettelkasten-Typen, `WorkflowState`, `MemoryType`, Zerfallskonstanten, `contextra-types/src/types/domain/misc.rs` | `contextra/src/agent_memory.rs`, `contextra-types` (REM-30) | Dateien und Typen fehlen |
| REM-25 | Query-Rewriter (LLM): `contextra/src/query_rewriter.rs`, `contextra-ports/src/query_rewriter.rs`, `contextra-ports/src/inference.rs`, `contextra/tests/llm_query_rewriter.rs` | genannte Pfade | Dateien fehlen. `grep -rn "LlmTextGenerator\|QueryRewriter"` leer |
| REM-26 | Tenant-Präfix-Codec im Store (ersetzt durch Verzeichnis je Mandant) | `contextra-store/src/tenant_codec.rs`, `contextra-store/src/tenant_codec/` | Dateien fehlen |
| REM-27 | Verletzungen der Regeln R1, R2, R7: `ports → types`, `store → crypto`, `store → mvcc`, `store → erasure`, `simd → types`, `checkpoint → testkit` außerhalb von dev-dependencies | `Cargo.toml` der betroffenen Crates, Quellen mit `use contextra_types`/`use contextra_crypto` in `store` | G-03 grün |
| REM-28 | Duplizierte Kryptoabhängigkeiten (`sha2`, `hmac` in `erasure`, `engine`, `types`). Nur `contextra-crypto` führt Krypto-Crates | `Cargo.toml` | `grep -rln "sha2\|hmac" crates/*/Cargo.toml` nur `crypto` |
| REM-29 | Verweise auf entfernte Dokumente: `README.md` (Root), `CLAUDE.md`, `benchmarks/README.md`, `benchmarks/contextra-bench/AGENTS.md`, Crate-READMEs. `docs/crates/` und Root-`AGENTS.md` existieren nicht (F-39) | genannte Dateien | G-07 und G-08 grün |
| REM-30 | **`contextra-types` als Crate** (9.596 Zeilen). Inhalt: IDs und Ordinale nach `contextra-ids`, DTOs nach `contextra-ports::dto`, Rest zum Besitzer (ADR-33) | `crates/contextra-types/` | Verzeichnis fehlt, `grep -rn "contextra_types\|contextra-types"` leer |
| REM-31 | **Graph-Entfernung (nur bei G-GRAPH verfehlt).** Sonst Reduktion nach 12.4 | `crates/contextra-graph/` | Verzeichnis fehlt, oder G-GRAPH grün |
| REM-32 | `contextra-ingest`: `fmt-odf`, `fmt-ole`, `fmt-structured`, `compat/`, `canonical.rs` (Pipeline nutzt `ir.rs`), Features `workspace`, `types-compat` | `crates/contextra-ingest/` (zurückgeholt, 12.13.2) | Abnahme: Features entfallen, `grep -rn "canonical"` leer |
| REM-33 | Nicht zurückholen: `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-wire`, `contextra-py`, `contextra-prompt-guard` (in Commit `e5ec76c28` eingefroren) | Historie | nicht im Workspace |
| REM-34 | Verbleib im Workspace-`Cargo.toml`: `[workspace.dependencies]` mit Ollama, Candle, Frozen-Zone-Kommentaren. `[workspace.lints]` bleibt | `Cargo.toml` | `grep -n "Frozen\|ollama\|candle"` leer |
| REM-35 | `contextra-crypto/src/dek.rs` durch `DocKey`-Modul ersetzt | `contextra-crypto/src/dek.rs` | Datei fehlt |
| REM-36 | Duplikat `StorageEngine` (Port) und `KvStore` (Port). Nur `KvStore` bleibt | `contextra-ports/src/` | `grep -rn "StorageEngine"` leer |
| REM-37 | Zwei bedingte Kompilierungen für `graph`-Persistenz hinter Feature `workspace`. Ersetzt durch Segmentpersistenz (6.10) | `contextra-graph/src/csr/graph_persist.rs` | Datei fehlt |

### 12.4 Graph-Reduktion (nur wenn G-GRAPH grün, REM-31)

**Behalten** (werden neu geschrieben, nicht verschoben):
- `src/csr/` (Basis-Segmente, Delta, Snapshot, Konsolidierung, Tombstones als Provenienz),
- `src/path_rag/` (PathRAG nach REQ-GR-5),
- `src/provenance.rs` (Seal-Provenienz),
- `src/types.rs` (reduziert auf IDs, Entitäts- und Kantenwerte ohne Klartext),
- `src/error.rs`, `src/graph_view.rs` nur als interne Lesesicht.

**Neu:** `src/label.rs` (GraphLabel-Records), `src/seal.rs` (Sealing über `RecordSealer`, `SegmentSealer`), `src/index.rs` (Implementierung `GraphIndex`), `src/scrub.rs` (`ScrubTarget`).

**Entfernen:**

| Pfad | Grund (ADR) |
|---|---|
| `src/tl_hfd/` | TL-HFD mit Lovász-Erweiterung, ohne Kundennutzen (ADR-24) |
| `src/apprh/` | APPRH, Gate-Monitor, Shadow-Vergleich im Hot-Path (ADR-24) |
| `src/percolation.rs` | Konstruiert neue Zusammenhänge nach Löschungen (ADR-24) |
| `src/ppr/` (vollständig, inkl. Forward-Push) | Zweiter Algorithmus neben PathRAG; Wiedereinführung nur über G-GRAPH (ADR-24) |
| `src/session_dag.rs` | Agentengedächtnis, widerspricht 1.3 (ADR-24) |
| `src/edge_reinforcement.rs`, `src/edge_reinforcement_buffer.rs`, `src/csr/inner/reinforcement.rs` | Adaptives Gedächtnis (ADR-24) |
| `src/hyperedge.rs`, `src/hyperedge_suggest.rs`, `src/csr/graph_write/hyperedge.rs`, `src/cascade.rs` | Hyperkanten und LLM-Validierung (ADR-24), Kaskade durch Gate ersetzt (ADR-22) |
| `src/consistency_enforcement.rs` | Musterlerner, keine Löschfunktion (ADR-25) |
| `src/community.rs`, `src/community/` | Community-Erkennung ohne Anforderung (ADR-24) |
| `src/entity_extraction.rs` | Extraktion liegt beim Aufrufer (ADR-48) |
| `src/text_gen.rs` | LLM-Port (ADR-24, REM-25) |
| `src/csr/visibility.rs`, `src/csr/tests/bitemporal_tests.rs`, `tx_id`-Felder in Kanten | bitemporale Felder (ADR-26) |
| `src/csr/path_graph.rs`, `src/path_rag/k_path.rs` | Zweiter Pfadalgorithmus (`k_path`) |
| `src/provider.rs`, `src/storage.rs`, `src/purger.rs`, `src/ppr_stream.rs`, `src/arc_slice.rs` | Ports und Bridges (CMP-5), Ersatz durch `KvStore` und `ScrubTarget` |
| `src/compat/`, `src/compat.rs` | CMP-5 |
| `src/test_doubles.rs` | nach `contextra-testkit` |
| `benches/`, `fuzz/`, `examples/`, `clippy.toml`, `proptest-regressions/`, `INVENTAR.md`, `PLAN.md`, `Cargo.toml`-Features `workspace`, `types-compat`, `ports-compat`, `session-dag`, `experimental-graph`, `hyperedge-suggest`, `entity-extraction`, `apprh-diffusion`, `k-path-diffusion`, `graph-connectivity-health`, `physio-percolation`, `ppr-forward-push`, `edge-reinforcement-learning`, `test-support` | ADR-24, R3, ADR-54 |
| Tests unter `tests/` außer `persistence`, `path_rag_at_snapshot`, `reader_writer_concurrency`, `ppr`-frei Golden-Dateien | Die übrigen Tests testen entfernte Funktionen |

**Cargo-Abhängigkeiten nach der Reduktion:** `contextra-ids`, `contextra-env`, `contextra-ports`, `contextra-durable-fs`, `blake3`, `roaring`, `thiserror`. `arc-swap` bleibt (Snapshots). `scc`, `bincode`, `serde`, `serde_json`, `ahash`, `tracing`, `parking_lot` entfallen, soweit nicht von `ids` oder `ports` benötigt. `contextra-types` entfällt (REM-30).

### 12.5 Compat-Auflösung (ADR-51)

Jedes `compat/`-Modul hat eine Zielposition. Die Bridge-Logik wird gelöscht, Port-Implementierungen werden direkt im Kernmodul geschrieben. Umfang gemessen am HEAD (Anhang B).

| ID | Verzeichnis | Zeilen | Zielposition für jeden Inhalt |
|---|---|---|---|
| CMP-1 | `contextra-mvcc/src/compat/` | 82 | `error.rs` → `contextra-version/src/error.rs`. `mod.rs` entfällt. |
| CMP-2 | `contextra-checkpoint/src/compat/` | 247 | `ports_impl.rs` → `checkpoint/src/port.rs` (Impl. `Checkpointer`). `snapshot_bridge.rs` entfällt (Duplikat von `snapshot.rs`). `error.rs` → `checkpoint/src/error.rs`. |
| CMP-3 | `contextra-text/src/compat/` | 3.060 | `morph_index.rs` → `text/src/index.rs`. `scorer.rs` → `text/src/bm25.rs`. `types.rs` → `text/src/types.rs`. `inverted_ports.rs` → `text/src/inverted.rs`. `scanner.rs` und `purge.rs` → `text/src/scrub.rs` (`ScrubTarget`). `storage.rs` (`StorageBridge`) entfällt, Text nutzt `KvStore` direkt. `mock.rs` → `contextra-testkit`. `tests.rs`, `search_tests.rs` → `text/tests/`. `tx_tests.rs` entfällt (SSI). `error.rs` → `text/src/error.rs`. |
| CMP-4 | `contextra-store` | – | Kein `compat/`-Verzeichnis (F-39). Abhängigkeiten zu `crypto`, `mvcc`, `erasure` werden gelöst (REM-27). |
| CMP-5 | `contextra-graph/src/compat/` und `compat.rs` | 1.688 | Komplett entfernt. Inhalte werden im Neuaufbau nach 12.4 geschrieben: `purger.rs` → `scrub.rs`, `graph_index_ext.rs` → `index.rs`, `collection_mutation.rs` und `mutations.rs` → `write.rs`, `traversal.rs` → `traverse.rs`, `error.rs` → `error.rs`. `provider.rs`, `storage_bridge.rs`, `inherent_shims.rs`, `algo_error_bridge.rs` entfallen. |
| CMP-6 | `contextra-rank/src/compat/` | 327 | `clock.rs` entfällt (`Clock` aus `env`). `types.rs` → `rank/src/types.rs`. `error.rs` → `rank/src/error.rs`. `mod.rs` entfällt. |
| CMP-7 | `contextra-simd/src/compat/` | 144 | `mod.rs` entfällt. Dispatch liegt in `simd/src/kernel.rs` (zusammen mit REM-23). |
| CMP-8 | `contextra-vector/src/compat/` | 1.917 | `hnsw_vector_index.rs` → `vector/src/hnsw/index.rs` (Impl. `VectorIndex`), Duplikate zu `vector/src/hnsw/` werden zusammengeführt. `diskann_vector_index.rs` entfällt (REM-09). `scanner.rs`, `purge.rs` → `vector/src/scrub.rs` (`ScrubTarget`). `candidate_stream.rs` → `vector/src/stream.rs`, nur wenn der Filterplaner ihn braucht, sonst entfällt. `types.rs` → `vector/src/types.rs`. `error.rs` → `vector/src/error.rs`. |
| CMP-9 | `contextra-ingest/compat/` (Historie) | 96 | Nicht zurückgeholt. Die Logik wird mit dem Pipeline-Code neu geschrieben (ADR-12). |
| CMP-10 | Bedingte Kompilierungen mit `types-compat`, `ports-compat`, `workspace` (120 Stellen laut Messung) | – | Alle Schalter entfallen. Verbleibende `cfg(feature)` nur für Features aus der Liste in Kapitel 15. |

**Summe gemessen:** 7.443 Zeilen in `compat/`-Verzeichnissen und 22 Zeilen in `compat.rs`, zusammen 7.465. Der Wert in v4.0 (rund 5.800) war zu niedrig.

**Regel für die Auflösung (verbindlich, G-11):** Ein Port wird im Besitzer-Crate implementiert. Doppelmodule für denselben Kern werden zusammengeführt, bevor sie gelöscht werden. Test-Mocks wandern nach `contextra-testkit`. Kein Feature darf einen zweiten Codepfad für denselben Kern einschalten (ADR-54).

### 12.6 Workspace und Build

- `default-members` ohne `contextra-testkit` und `contextra-bench` (F-27). Standard-Build: alle produktiven Crates.
- `contextra-graph` ist Mitglied des Workspace, aber nur über `db`-Feature `graph` gebaut.
- `default = []` in allen Crates, außer den drei aus R3:
  - `contextra-durable-fs`: `default = ["std"]` (Standardbibliothek, `crc` entfällt).
  - `contextra-simd`: `default = ["std"]`.
  - `contextra-rank`: `default = ["std"]`. Die Experimental-Features `exp-dibud`, `exp-resonance`, `experimental-rank` entfallen (REM-18).
- Keine Feature-Namen mit `compat`, `workspace`, `types-compat`, `ports-compat`, `experimental`.
- `rust-toolchain.toml` bleibt auf 1.89.0. Das Sandbox-Netz erreicht `static.rust-lang.org` nicht. Entwicklung ist auf jedem Rust ab 1.89 erlaubt, CI prüft mit 1.89.0.

---

## 13. Umsetzung

### 13.1 Meilensteine

Aufwand grob als S (≤ 1 Woche), M (2–3 Wochen), L (4–6 Wochen), XL (über 6 Wochen), jeweils für ein Zweierteam, Schätzung ±50 %.

| M | Inhalt | Aufwand | Abnahme |
|---|---|---|---|
| **M0 Basis** | Alte Dokumente archivieren (`docs/archive/`). Build repariert (Fassade kompiliert). `MAX_SEARCH_K` vereinheitlicht (ADR-52). Workspace-Lint `forbid`. `budgets.toml` angelegt. `layering.rs` angelegt (G-03). `default-members` bereinigt. Alle `Instant::now`/`SystemTime::now` durch `Clock` ersetzt (G-12). | M | G-01, G-02, G-03, G-04, G-10, G-12 |
| **M1 Bereinigung** | REM-01 bis REM-37 in Reihenfolge 12.1, 12.2, 12.3. `compat/` nach 12.5. `contextra-types` aufgelöst (ADR-33). `store` von `crypto`, `erasure`, `mvcc` gelöst (REM-27). `mvcc` zu `version` umbenannt. Benchmark für G-STORE. | L | G-03, G-07, G-08, G-09, G-11, G-13. G-STORE entschieden |
| **M2 Seal und Löschkette** | `contextra-crypto` (KeyTable 128 Byte, Anker, `RecordSealer`, `SegmentSealer`, `KeyDestroyer`, Wurzel-Backends). `contextra-erasure` neu (Log, Gate mandantenweit, Koordinator, Scrub-Planung). `contextra-version` (Katalog, Visibility, `ContentLifecycle`). Seal-Registry. Verdrahtung in `contextra-db` (Tenant-Koordinator, Commit-Lock, Erasure-Lock). Absturzmatrix ab dem ersten schreibenden Schritt | XL | T-ER-01 bis T-ER-26, T-KEY, T-SEG, T-SEC-01 bis T-SEC-04, T-ENC-01 bis T-ENC-03 grün |
| **M3 Ingest und Embedding** | `contextra-ingest` aus Historie (13.2), beschnitten. `contextra-embed` (ONNX, Referenzmodelle nach ADR-36). Pipeline mit Backpressure, Bulk-Modus | L | T-RAG-01, 02, 10 grün. G-14 vorbereitet |
| **M4 Suche** | HNSW mit SQ8 und Nachberechnung, Segmentpersistenz (verschlüsselt), Filter und ACL, Planer, Fusion, Reranker-Port, Kontext, Fassade, MCP. Scrub-Ziele vector, text, store | L | T-RAG-03 bis 09, T-RAG-11, T-ER-08, T-IDX-01, T-OPS-10, T-LOG-01 grün |
| **M4b Graph (optional)** | `contextra-graph` nach 12.4. Feature `graph` in `db`. Graph-Gold-Korpus erstellen. | L | T-GR-01 bis T-GR-12 grün. G-GRAPH gemessen |
| **M5 Betrieb und Leistung** | Zustände, Startsequenz mit Lazy-Laden, Ressourcenregeln, Latenzfenster (ADR-43), Scrub-Drosselung, `embed-http`, Benchmarks gegen Basislinie, `budgets.toml` befüllt. G-GRAPH-Entscheidung umgesetzt (REM-31 bei Verfehlung) | M | T-OPS-01 bis 09, T-PERF grün. G-GRAPH und G-INDEX entschieden |
| **M6 Enterprise** | `contextra-kms` (Vault Transit, ADR-47), `contextra-server` (Auth, Metriken), Betriebshandbuch mit Grenzen (7.9), Restore-Ablauf (7.11), Schlüsselsicherung (`contextra key backup`) | L | Ende-zu-Ende-Lauf: Ingest, Suche, Löschung, Schlüsselsicherung, Restore mit `--reapply`. G-01 bis G-14 grün |

**Reihenfolge.** M0 vor M1. M2 vor M3 und M4, weil der Seal-Schreibpfad Voraussetzung ist. M3 und M4 sind danach parallel möglich. M4b ist optional und hängt nur an M4. Kein Meilenstein startet, bevor der Vorgänger seine Abnahme hat.

**Refactoring-Reihenfolge innerhalb von M1 (verbindlich):**
1. `contextra-ids` übernimmt IDs aus `types` (kein Verhalten).
2. `ports` von `types` lösen: DTOs nach `ports::dto`, dann `types` ohne Verwendung (REM-27, REM-30).
3. `simd` von `types` lösen (REM-23).
4. `store` von `crypto`, `erasure`, `mvcc` lösen. Verschlüsselung über Port `RecordSealer` (ist in M2 implementiert, bis dahin Test-Impl.).
5. `compat/` je Crate nach CMP-1 bis CMP-8. Reihenfolge: `simd`, `mvcc`, `rank`, `checkpoint`, `graph`, `vector`, `text`.
6. Features `*-compat`, `workspace` entfernen (CMP-10).
7. Verbleibende Entfernungen aus 12.1 bis 12.3.
8. Jeder Schritt ist ein eigener Commit mit `cargo check --workspace` grün.

### 13.2 Wiederverwendung aus der Git-Historie

Die Historie ist am HEAD `2fd0994` mit einem Fetch der Tiefe 400 verfügbar. Ältere Commits fehlen, die genannten Hashes sind vorhanden. Die Commit-Hashes wurden geprüft (Anhang B). Zurückgeholt wird per `git checkout <commit>^ -- <pfad>` in einen Arbeitszweig, dann Compilerfehler beheben, auf neue Ports umstellen, Tests laufen lassen, erst dann in den Hauptzweig.

| Bestandteil | Fundstelle | Umfang (gemessen) | Maßnahme |
|---|---|---|---|
| Ingest-Crate | `e5ec76c28^:crates/contextra-ingest/` | 129 Quelldateien, 30.737 Zeilen (mit Tests und Fuzz) | Zurückholen. Entfernen nach REM-32. Ziel 14.000 Zeilen. Feature-Matrix nach ADR-35 |
| Markdown-Chunker | `ae524bedf^:crates/contextra-db/src/chunker/` | 8 Dateien | Nur falls `ingest` ihn nicht abdeckt. Test `embedding_dimension_switch.rs` übernehmen |
| ONNX-Embedder und Reranker | `e5ec76c28^:crates/contextra-infer-onnx/` | 14 Dateien, 1.176 Zeilen | Als `contextra-embed`. `reqwest`-Pfad entfällt (kein Netz im Kern, ADR-13) |
| Candle-Embedder | `e5ec76c28^:crates/contextra-infer-candle/` | 47 Dateien, 8.910 Zeilen | Nicht zurückholen (REM-33, ADR-36) |
| Audit-Export | `dd895a253^:crates/contextra-audit-export/` | 34 Einträge | Nicht zurückholen (ADR-02) |

### 13.3 Migration

**Entscheidung (ADR-44):** Keine Migration. Entwicklungsdaten im Format v4.0 werden verworfen. Es gibt keine produktiven Daten. Ein Exportwerkzeug entsteht nur mit einer neuen Spezifikationsversion und einem Pilotkunden.

### 13.4 Dokumentation

**REQ-DOC-3 (Dokumentationslint).** `tools/doc_lint.py` prüft vor jedem Merge: keine Verweise auf entfernte Dokumente, Crates, Symbole und Features (Liste 12), keine verbotenen Begriffe in Ledger- und Statusausgaben (7.10), jede Referenz `REQ-`, `ADR-`, `INV-`, `REM-`, `CMP-`, `T-`, `G-` ist definiert. Das Skript ist der einzige Prüfer dieser Regeln (G-08).

**REQ-DOC-1.** Auf v4.1 gebracht oder entfernt: `docs/SPEC.md` (dieses Dokument), `CLAUDE.md`, `README.md` (Root), `benchmarks/README.md`, Crate-READMEs nach 3.6. Veraltete Angaben (6- und 9-Schritt-Saga, `SubjectKey`, `m = 256 MB`, `Poisoned`, Profile, Erfüllungsklassen, Siegel, PLAN v2.1 des Graphen) fallen weg.

### 13.5 Vorlage für `crates/<name>/PLAN.md`

Jedes Crate erhält genau eine Implementierungsspezifikation. Sie wird aus diesem Dokument abgeleitet und verweist nur auf Kapitel und IDs dieser Spezifikation. Abschnitte:

1. **Zweck und Ebene** (aus Kapitel 3.1 und 3.2).
2. **Vertrag:** Ports, die implementiert werden (Kapitel 3.4), öffentliche Typen (Kapitel 15).
3. **Abhängigkeiten:** erlaubte Liste aus 3.3, Cargo-Auszug.
4. **Module:** Dateiliste mit Verantwortung je Datei. Keine Datei namens `compat*`.
5. **Entfernungen:** IDs aus Kapitel 12, mit Abnahme-Befehl.
6. **Umbau:** Schritte in Reihenfolge, je Schritt ein Commit.
7. **Tests:** IDs aus Kapitel 11, zusätzliche crate-lokale Tests mit Begründung.
8. **Abnahme:** Gates aus 11.6, die das Crate betreffen.

Der Auftrag an die Entwickler ist damit vollständig, eine Rückfrage an diese Spezifikation ist nicht vorgesehen. Fragen zur Umsetzung klärt der Verantwortliche per Änderung dieses Dokuments.

---

## 14. Nachweis der Befundauflösung

Jeder Befund aus 0.3 ist hier einem Abschnitt und einem Test zugeordnet. Eine Änderung ohne Eintrag hier gilt als unvollständig.

| Befund | Auflösung in | Nachweis |
|---|---|---|
| F-01 | 3.1, ADR-21, 6.10 | G-GRAPH, T-GR-01 bis T-GR-12 |
| F-02 | ADR-25, 12.4 | G-07 |
| F-03 | ADR-24, 1.3 | G-07 |
| F-04 | ADR-23, REQ-FMT-2, 5.2 | T-GR-01, T-LOG-01 |
| F-05 | ADR-22, ADR-27, 6.10 | T-GR-03 |
| F-06 | ADR-22, REQ-GR-4, REQ-GR-6 | T-GR-03, T-GR-04 |
| F-07 | ADR-32, 3.3, REM-27 | G-03 |
| F-08 | ADR-33, REM-30 | G-07 |
| F-09 | ADR-51, 12.5 | G-11 |
| F-10 | ADR-40, REQ-KEY-1 | T-KEY-05 |
| F-11 | ADR-30, ADR-31, INV-9, 7.4 | T-ENC-01, T-SEG-02 |
| F-12 | ADR-29, 5.5, REQ-REC-2 | T-KEY-01, T-ER-05 |
| F-13 | ADR-52, REQ-RES-4 | G-04 |
| F-14 | ADR-16, C0 | T-KEY-06 |
| F-15 | REQ-KEY-4, REQ-CON-7 | T-KEY-03 |
| F-16 | ADR-16 | T-KEY-06 |
| F-17 | 7.3 (REQ-ER-2) | T-SEC-02, T-ER-12 |
| F-18 | ADR-28, REQ-CON-1a, 4.7 | T-ER-07 |
| F-19 | REQ-ING-7, REQ-CON-1b | T-ER-14, T-SEC-03, T-RAG-11 |
| F-20 | 5.7 | T-ER-11 |
| F-21 | 7.4 (Publikationsreihenfolge) | T-ER-04 |
| F-22 | ADR-43, REQ-SCR-4 | T-ER-19 |
| F-23 | ADR-43, 10.3 | ADR-02, G-14 |
| F-24 | REQ-VEC-4 | T-IDX-01 |
| F-25 | REQ-TXT-2 | Metrik in T-RAG-03 |
| F-26 | REQ-TIM-1 | G-12 |
| F-27 | 12.6 | G-01 |
| F-28 | ADR-53 | G-02 |
| F-29 | REQ-REC-2 | T-ER-05 |
| F-30 | REQ-ER-6 | T-ER-25 |
| F-31 | ADR-38, REQ-FLT-2 | T-SEC-01, T-SEC-04 |
| F-32 | ADR-38, 7.2 | T-SEC-02 |
| F-33 | ADR-50, REQ-CTX-4 | T-RAG-10, T-ENC-02 |
| F-34 | 5.8 | T-SEG-01 |
| F-35 | ADR-42, REQ-BAK-1 | Abnahme Ende-zu-Ende (M6) |
| F-36 | REQ-FLT-3, 9 | T-RAG-05 |
| F-37 | Anhang B | Messung in Anhang B |
| F-38 | REM-01, REM-19 | `grep` in 12.1, 12.3 |
| F-39 | 12 (Pfade korrigiert) | Abnahmebefehle in 12 |
| F-40 | REQ-DOC-1, 13.5 | G-09 |
| F-41 | 3.3, REM-31 | G-03 |
| F-42 | ADR-24, 12.6 | G-06 |
| F-43 | REM-10, 13.1 M2 | T-ER-11, G-09 |
| F-44 | ADR-35, REQ-ING-2 | T-RAG-02 |

---

### 14.1 Offene Punkte aus v4.0

| Punkt | Frage | Entscheidung | Ort |
|---|---|---|---|
| OPEN-1 | Rechtliche Einordnung des Produkts | Keine Bewertung im Produkt und in der Spezifikation. Die Einordnung trifft der Verantwortliche | ADR-45, REQ-DOC-2 |
| OPEN-2 | Erstes KMS-Backend | HashiCorp Vault Transit, selbst betrieben. AWS KMS nur mit neuer Spezifikationsversion | ADR-47 |
| OPEN-3 | Hochverfügbarkeit und Replikation | Ausgeschlossen in v4.x | ADR-46 |
| OPEN-4 | Formatliste des Ingest (`fmt-ole`, `fmt-odf`, `fmt-structured`) | Entfallen nach REM-32. Unterstützt nach REQ-ING-2: TXT, Markdown, JSON, HTML, DOCX, PPTX, XLSX, CSV, PDF (ohne OCR), EML, MBOX. ODF, OLE und YAML/TOML entfallen | ADR-35, REQ-ING-2 |
| OPEN-5 | Migration der Entwicklungsdaten | Keine Migration. Daten im Format v4.0 werden verworfen | ADR-44 |

## 15. Crate-Verträge

Jede Zeile ist eine Vorgabe für das Crate-`PLAN.md`. Ports sind in 3.4 und 4.4 definiert.

**`contextra-ids`** (E0). Typen: `DocId`, `TenantId`, `CollectionId`, `SealOrd`, `SegOrd`, `ChunkSlot`, `TicketId`, `SubjectRef`, `PrincipalRef`. Limits: `MAX_SEARCH_K = 1_000` (einzige Definition, ADR-52), `MAX_CHUNKS_PER_DOC`, `MAX_BATCH_BYTES`. Abhängigkeiten: keine internen. Features: keine. Tests: Serialisierung, Ordinal-Monotonie, Limit-Konsistenz (G-04).

**`contextra-env`** (E0). `Clock` (real, `ManualClock`), `SeededRng`, `MetricsSink`. Einzige Stelle mit `Instant::now`/`SystemTime::now` (G-12). Tests: Clock-Semantik, Zähler ohne Nutzdaten (T-LOG-01).

**`contextra-sys`** (E0). mlock, Dumpable, Direct-IO. Einzige `unsafe`-Insel neben `simd`. Tests: mlock-Verweigerung führt zu `Refused` (T-OPS-09).

**`contextra-simd`** (E0). Distanzkerne SIMD und Skalar, Dispatch (CMP-7, REM-23). Abhängigkeiten: keine. Features: `std`. Tests: Kernel gegen Skalar (Property-Test).

**`contextra-durable-fs`** (E0). Atomares Schreiben (`tmp → fsync → rename → fsync(Verzeichnis)`), Gruppen-Sync, Dateisperre. Entfernen: REM-07, REM-13. Tests: Absturzmatrix mit `FaultVfs` aus testkit.

**`contextra-ports`** (E0). Alle Traits aus 4.4, 3.4 und 6.2 (`EmbeddingProvider`, `Reranker`), `KvStore`, `SegmentStore`, `VectorIndex`, `TextIndex`, `GraphIndex`, `Checkpointer`, `RootKeyProvider`, `ErasureSink`, `ContextraError`, DTO-Modul `dto` (`SourceDoc`, `SearchQuery`, `SearchResponse`, `ContextPack`, `ErasureRequest`, `ScrubPlan`, `ScrubProgress`, `IoBudget`). Abhängigkeit: `ids`, `roaring`. Entfernen: REM-03, REM-25, REM-30 (Import), REM-36. Tests: Objektsicherheit der Traits (Compile-Test), DTO-Roundtrip.

**`contextra-crypto`** (E1). `aead` (AES-256-GCM, AES-256-GCM-SIV), `kdf` (Argon2id, HKDF nur aus Wurzel), `keytable` (5.5), `anchor` (5.6), `sealer` (`RecordSealer`, `SegmentSealer`), `destroyer` (`KeyDestroyer`, REQ-KEY-4), `root` (`Passphrase`, `KeyFile`, `VaultTransit` in M6), `cache` (REQ-KEY-6, REQ-CON-7), `secret` (zeroize-Typen ohne `Debug`). Entfernen: REM-04, REM-11, REM-35. Abhängigkeiten: `ports`, `ids`, `env`, `durable-fs`. Tests: T-KEY-01 bis T-KEY-06, T-ENC-03.

**`contextra-erasure`** (E1). `log` (CXEL, 7.8), `gate` (CXGS, `TenantGate: ReadGate`, 7.8), `ticket`, `classify` (7.3), `chain` (E0 bis E4 mit Ports, 7.4), `state` (7.5), `scrub_plan` (7.6, reine Planung), `review` (REQ-ER-3, REQ-ER-6), `hold` (REQ-RET-2), `ledger` (7.10). Entfernen: REM-10, REM-16. Abhängigkeiten: `ports`, `ids`, `env`, `durable-fs`. Tests: T-ER-01 bis T-ER-26 (Anteile), T-LOG-01.

**`contextra-version`** (E1, aus `mvcc`). `catalog` (REQ-VER-1), `snapshot` (REQ-VER-2), `retention` (REQ-VER-3), `visibility` (`Visibility`, `ContentLifecycle`-Sender), `error`. Entfernen: REM-12. Tests: T-ER-10, T-ER-11, REQ-VER-5 (Zeitreise mit `index_history`).

**`contextra-store`** (E1). `lsm/` (Memtable, SST, Manifest, WAL), `kv` (`KvStore`), `segments` (`SegmentStore` für Dateien mit Kopf nach 5.8, ohne Verschlüsselung selbst), `compaction` (Kompaktion mit Löschfilter für Records toter Seals, REQ-SCR-3), `fsync`, `budget`, `stats`. Entfernen: REM-02, REM-05, REM-08, REM-26, REM-27, CMP-4. Abhängigkeiten: `ports`, `ids`, `env`, `sys`, `durable-fs`. Tests: G-STORE-Benchmark, Wiederanlauf, Kompaktionsfilter.

**`contextra-vector`** (E1). `hnsw/` (Graph, Delta, Persistenz nach REQ-VEC-4), `exact` (SIMD), `sq8` (Kalibrierung, REQ-VEC-1a), `acorn` (Filtertraversierung), `planner` (REQ-FLT-3), `scrub` (`ScrubTarget`, REQ-VEC-5), `index` (`VectorIndex`), `types`, `error`. Entfernen: REM-09, REM-23, CMP-8. Tests: T-RAG-04, T-RAG-05, T-IDX-01, T-ER-03.

**`contextra-text`** (E1). `tokenize` (REQ-TXT-1a), `stopwords/` (Daten), `bm25`, `wand`, `inverted` (Segmente), `index` (`TextIndex`), `merge`, `scrub` (`ScrubTarget`), `discovery` (REQ-TXT-3), `types`, `error`. Entfernen: CMP-3, Komposita-Code (REQ-TXT-1a). Tests: T-RAG-03, T-ER-02, Metrik F-25.

**`contextra-graph`** (E1, optional). Siehe 12.4 und 6.10. Tests: T-GR-01 bis T-GR-12.

**`contextra-rank`** (E1). RRF, Top-k, Zeitgewichtung (`valid_at`), Erklärung. Entfernen: REM-06, REM-18, CMP-6. Features: `std`. Tests: Determinismus bei gleichen Eingaben, Tie-Break (REQ-SRC-1 Schritt 7).

**`contextra-privacy`** (E1). `entity_recognizer` (Muster, Prüfziffern, Listen), `data_class`, `egress_guard` (fail-closed), `processing_registry`, `guarded_payload`. Entfernen: REM-20. Abhängigkeiten: `ids`, `env`. Tests: Erkennungsgrenzen, Guard fail-closed.

**`contextra-checkpoint`** (E1). Hartlink-Snapshot und Restore, Ausschluss `keys/` und `erasure/` (ADR-42), Retention nach `checkpoint_retention`. Entfernen: CMP-2. Abhängigkeiten: `ports`, `ids`, `env`, `durable-fs`. Dev-Abhängigkeit `testkit`. Tests: Ausschluss, Restore-Test mit Gate-Erhalt.

**`contextra-ingest`** (E2). `formats/` (ADR-35), `chunking/` (REQ-ING-3), `detect/` (MIME-Erkennung, unbekannte Formate abweisen), `pipeline/` (Parse-Stufe), `ir.rs` (einziger Zwischenformat-Typ), `limits`, `report`. Keine internen Abhängigkeiten. Features: `ingest-*` nach 6.1. Tests: T-RAG-01, T-RAG-02, Fuzz-Ziele unter `fuzz/`.

**`contextra-embed`** (E2). `onnx/` (ort, Modellprüfung per SHA-256), `reranker/`, `provider` (`EmbeddingProvider`-Impl.). Abhängigkeiten: `ports`, `ids`. Features: keine. Tests: Modellhash, Batchgleichheit (`batch_equiv`), Modellwechsel.

**`contextra-db`** (E2). Orchestrierung: Schreib-Task je Shard, Tenant-Koordinator, Commit-Lock, Erasure-Lock, Startsequenz (8.3), Scheduler, Latenzfenster (ADR-43), Verdrahtung der Ports (3.4, 7.13). Features: `graph`, `text`, `vector`, `embed`, `ingest` (Standard alle außer `graph`). Abhängigkeiten: alle E1-Crates, `ports`, `ids`, `env`, `ingest`, `embed`. Tests: T-OPS-01 bis T-OPS-10, T-SEC-03, T-RAG-11, T-ER-07.

**`contextra`** (E2, Fassade). Öffentliche API nach 6.8, Konfigurationsladen, Fehlerabbildung. Entfernen: REM-02 (Profile), REM-24, REM-25. Abhängigkeiten: `db`, `ports`, `ids`, `privacy`. Tests: API-Oberflächentest, Konfigurationsfehler (T-OPS-03).

**`contextra-mcp`** (E3). stdio-Server, Tool-Schemas aus den Fassadentypen, Principals aus Konfiguration (ADR-38). Entfernen: REM-21, REM-22. Abhängigkeiten: `contextra`, `ports`, `privacy`. Tests: T-SEC-04, Schema-Roundtrip.

**`contextra-testkit`** (E3, nur dev). FaultVfs, Canary-Scanner (T-ENC-01), Mock-Store, Mock-Uhr, Schedule-Exploration (`loom`), Mock-Embedder, Mock-Gate. Enthält `mock.rs` aus CMP-3. Abhängigkeiten: alle außer `bench`, `server`, `kms`. Tests: G-03 (`layering.rs`).

**`contextra-bench`** (E3, `benchmarks/contextra-bench/`). Benchmarks gegen `budgets.toml`, Basisvergleich (`bench_baseline`), Graph-Gold-Auswertung (G-GRAPH). Abhängigkeiten: `contextra`, `db`, `vector`, `text`, `store`, `rank`, `graph`.

**`contextra-embed-http`** (E3). HTTP-Embedder mit TLS, Allowlist, Zeitgrenzen. Abhängigkeiten: `ports`. Features: `http` (Standard aus). Tests: Endpunkt-Allowlist, Egress-Guard-Anbindung.

**`contextra-server`** (E3, M6). HTTP/gRPC, OIDC, Metriken (Prometheus), Backpressure. Abhängigkeiten: `contextra`, `ports`. Tests: Standard-Deny, Claim-Mapping.

**`contextra-kms`** (E3, M6). Vault-Transit-Adapter (`RootKeyProvider`, ADR-47). Abhängigkeiten: `ports`. Tests: Verweigerung bei Verbindungsfehler, kein Klartext im Log.

---

## Anhang A: Verbleib der Konzepte aus v4.0

| Altes Konzept | Neu |
|---|---|
| ER-1/2 `DocDek` je Version | Schlüssel je **Seal** (ADR-01), Version ohne Schlüsselbezug |
| ER-3 Wrap mit AES-GCM-SIV | beibehalten (REQ-KEY-2) |
| ER-4 Keine HKDF für Datenschlüssel | beibehalten (REQ-KEY-1), präzisiert für Strukturschlüssel (ADR-40) |
| ER-5 Gate vor Entschlüsselung | beibehalten (REQ-FLT-5) |
| ER-6 Zeitreise gegen aktuellen Stand | folgt aus der Trennung (REQ-VER-6) |
| ER-7/8 `fsync → Snapshot-Swap`, Quarantäne | beibehalten (REQ-GATE-1) |
| ER-9 Gate-Log-Verdichtung | Gate-Snapshot `CXGS` (7.8) |
| Gate-Log HMAC-Kette | entfällt, CRC und Sequenz (ADR-02) |
| ER-10/11 Generationen, Epochen-Rewrap | Epochen, Rewrap in KeyTable (REQ-KEY-5) |
| ER-12 KeyTable-Sharding | Slot-Datei mit Überschreiben an Ort (ADR-08), 128 Byte (ADR-29) |
| ER-13 `DeleteAll` als Standard | `Restrict` als Standard (ADR-10) |
| ER-14 Trockenlauf | beibehalten, erweitert um Entdeckung (7.3) |
| ER-15/16 Konfliktdokumente, Review | beibehalten (REQ-ER-2, REQ-ER-3) |
| ER-17 `RedactSplit` | `RedactReingest` (REQ-ER-4) |
| ER-18 `Derived<T>` mit Quellmenge | ersetzt durch Seal-Bezug jedes Chunks, Provenienz im Graph (ADR-22) und `SealScoped` |
| ER-20 Keine veralteten Ableitungen | beibehalten (Ableitungen verweisen auf Seals, werden gefiltert und im Scrub entfernt) |
| ER-21/22 Inkrementelle BM25-Statistik | `N` und Länge exakt, `df` beim Merge (REQ-TXT-2) |
| ER-23 Flache Suche, `zeroize` der Slots | HNSW mit routen-erhaltendem Scrub (REQ-VEC-5) |
| ER-24 Kein Umverdrahten | aufgegeben: Konsolidierung (REQ-VEC-5) |
| ER-25 Neuaufbau bei 20 % | Neuaufbau erst bei 30 % nach Konsolidierung |
| ER-26/27 Cluster-Segmente | entfallen (ADR-11) |
| ER-28 Handkodierte Formate | auf fünf begrenzt (REQ-FMT-4, 5.10) |
| ER-29 `segment_max_bytes` | beibehalten (REQ-TXT-1) |
| ER-30/31 Reihenfolge, frühe Vernichtung | frühe Vernichtung ist Standard (ADR-03), für Segmentschlüssel eingeschränkt (ADR-31) |
| ER-32 Bereinigungsfrist | `scrub_deadline` (7.4) |
| ER-33 Wiederanlauf | beibehalten, Slot-Tabelle (REQ-REC-2) |
| ER-34 Einzelausführung | beibehalten (REQ-CON-4) |
| ER-35 bis 37 Bericht, JCS, Ed25519, Bescheinigung | entfallen (ADR-02), ersetzt durch Ledger und Status (7.10) |
| ER-38 Inventarpflicht | entfällt mit der Messung |
| ER-39/40 Verifikation, Prüfwerkzeug | entfallen. Vier-Augen-Prinzip als Option für Entscheidungen |
| ER-41 Sprach-Gate | als Dokumentationsregel (REQ-DOC-2) |
| ER-42 Zwei Spuren | beibehalten, Schlüssel und Erasure getrennt (REQ-BAK-1, ADR-42) |
| ER-43/44/45 Senke, Restore-Guard, Horizont | beibehalten (7.11) |
| ER-46/47/48 Einschränkung, Automat | beibehalten, Automat in 7.5 mit Review-Fristen (REQ-ER-6) |
| v3 Zustände `MeasuredA`, `Reported` u. a. | entfallen |
| v3 Custody K1 bis K3 | K1, K1b, K3 (ADR-15, ADR-47) |
| v3 REQ-IDX-1 Flach als Pflicht | exakt nur klein und als Referenz (ADR-04) |
| Graph-Entwurf (PLAN v2.1) | Generationen mit ArtKey verworfen (ADR-27), Provenienz statt Klassenbindung (ADR-22) |
| Graph-Anhang v4.0 (USP-Liste) | überarbeitet in 6.10 und ADR-21 bis ADR-27. `ConsistencyEnforcer` und `SessionBranchTree` entfallen (ADR-24, ADR-25) |

## Anhang B: Feststellungen am Repository (HEAD `2fd0994`, 20.11.2026)

Messung mit `find`, `grep`, `wc` und `git` am geklonten Stand. Alle Zahlen der Spalte „Messung“ wurden gezählt. Nur Angaben mit „Schätzung“ oder „nicht geprüft“ sind keine Messung.

| Fakt | v4.0 | Messung | Quelle |
|---|---|---|---|
| Workspace-Mitglieder | 23 | **22** (21 Crates + `benchmarks/contextra-bench`) | `Cargo.toml` `members` |
| Rust-Zeilen in `crates/` | 244.000 | **233.961** (inkl. Tests) | Summe über `crates/*/src` und `tests` |
| Rust-Zeilen in `benchmarks/contextra-bench` | 5.200 | **5.199** | `wc -l` |
| Rust-Zeilen gesamt inkl. `benchmarks/*.rs` | 249.000 | **239.968** | `find crates benchmarks -name '*.rs'` |
| `contextra-graph` Umfang | nicht erfasst | **34.611** Zeilen, **153** Dateien | `wc` |
| `contextra-ingest` (Historie `e5ec76c28^`) | 155 Dateien, ~21.000 | **129** `.rs`-Dateien, **30.737** Zeilen | `git archive` + `wc` |
| `contextra-infer-onnx` (Historie) | ~1.000 | **14** Dateien, **1.176** Zeilen | `git archive` + `wc` |
| `contextra-infer-candle` (Historie) | nicht erfasst | **47** Dateien, **8.910** Zeilen | `git archive` + `wc` |
| `compat/`-Zeilen gesamt | rund 5.800 | **7.465** (7.443 in `compat/`-Verzeichnissen, 22 in `compat.rs`) | `find -type d -name compat` + `wc` |
| `cfg(feature)`-Stellen mit `compat`/`workspace` | nicht erfasst | **120** | `grep -rn` |
| `MAX_SEARCH_K`-Definitionen | sechs | **sechs**, Werte 1.000 (ids, types, graph) und 10.000 (text, rank, vector) | `grep -rn "const MAX_SEARCH_K"` |
| `MAX_SEARCH_K`-Verwendungen | nicht erfasst | **86** | `grep -rn` |
| Abhängigkeit `ports → types` | nicht erfasst | **5** Verwendungen im Code, Cargo-Abhängigkeit vorhanden | `grep`, `Cargo.toml` |
| Abhängigkeit `store → crypto` | nicht erfasst | **16** Verwendungen, Cargo-Abhängigkeit | `grep`, `Cargo.toml` |
| Abhängigkeit `store → mvcc`, `store → erasure` | nicht erfasst | Cargo-Abhängigkeit vorhanden | `Cargo.toml` |
| Abhängigkeit `simd → types` | „entfernen“ | vorhanden | `Cargo.toml` |
| Abhängigkeit `db → graph` | nicht erfasst | vorhanden, im Orchestrator ungenutzt | `Cargo.toml`, `grep` |
| `unsafe` im Produktcode | nicht geprüft | nur in `contextra-sys` und `contextra-simd`. In Test-Targets zusätzlich `contextra-text/tests/alloc_profiler.rs` und `contextra-graph/tests/ppr_alloc_test.rs` (entfällt mit 12.4). Vektor-Distanzen enthalten kein `unsafe` | `grep -rn "unsafe "` ohne Kommentarzeilen |
| `Instant::now`/`SystemTime::now` außerhalb `env` | nicht erfasst | **77** | `grep -rn` |
| `unwrap`, `expect`, `panic!` | nicht erfasst | **2.942** Treffer in `crates/*/src` inkl. `#[cfg(test)]`-Module, **6.939** in `crates/` gesamt. Produktcode ohne Tests zählt M0 genau | `grep -rn` |
| Netzwerk-Crates in Kern-Manifesten | „keine“ | keine. Die Grep-Treffer sind Teilwörter (`hyperedge`, `Isotonic`). G-05 prüft exakt | `grep` mit Wortgrenzen |
| Feature `workspace` in `graph` (nicht Default) | nicht erfasst | vorhanden, lädt Typen und Ports | `Cargo.toml` |
| Graph-Persistenz | nicht erfasst | `bincode` über Port `StorageEngine`, nur mit Feature `workspace` | `csr/graph_persist.rs` |
| Graph-Klartext | nicht erfasst | `Entity.name: Arc<str>`, `Edge.label: Arc<str>` | `graph/src/types.rs` |
| `ConsistencyEnforcer` | „zwingend für DSGVO-Löschung“ | Mustererkennung, keine Kantenbereinigung (Modul-Doku) | `graph/src/consistency_enforcement.rs` |
| Graph-Default-Features | nicht erfasst | `session-dag`, `entity-extraction`, `hyperedge-suggest`, `experimental-graph` | `Cargo.toml` |
| LLM-Pfad im Graph | nicht erfasst | `validate_candidates_with_llm` hinter `workspace` | `graph/src/hyperedge_suggest.rs` |
| `contextra-engine` | „entfernt“ (REM-19) | Verzeichnis fehlt, Verweise in Kommentaren | `ls`, `grep` |
| `contextra-vault` | „entfernt“ (REM-01) | Verzeichnis fehlt, Eintrag in `exclude` | `ls`, `Cargo.toml` |
| `ports/src/ports/query_rewriter.rs` | REM-25 | fehlt. Richtig: `ports/src/query_rewriter.rs` | `ls` |
| `store/src/compat` | REM-15 | fehlt | `ls` |
| `docs/crates/`, Root-`AGENTS.md` | REM-29 | fehlen | `ls` |
| `benchmarks/budgets.toml`, `docs/erasure.md`, `docs/Finale Löscharchitektur.md` | fehlend | fehlen | `ls` |
| `tests/layering.rs` (G-03) | angelegt bis M0 | fehlt | `ls` |
| `tests/unsafe_islands.rs` | G-02 | vorhanden | `ls` |
| `ANCHOR[`-Kommentare | nicht erfasst | **460** Dateien | `grep -rl` |
| `FILE-CONTEXT`-Kommentare | nicht erfasst | **417** Dateien | `grep -rl` |
| `contract.rs` | nicht erfasst | **9** Dateien | `find` |
| Skripte in `tools/` | nicht erfasst | **29** | `ls tools` |
| `Cargo.lock` | Version 4 | Version 4 | `head` |
| Toolchain | 1.89.0 | 1.89.0 in `rust-toolchain.toml`, Download im Sandbox blockiert, `stable` vorhanden | `rustup` |
| Workspace-Lint `unsafe_code` | `deny` | `deny`. Ziel `forbid` (G-02) | `Cargo.toml` |
| Historie | „5.952 Commits“ | Gesamtzahl nicht geprüft (flacher Fetch mit Tiefe 400). Commit-Hashes `e5ec76c28` (18.11.), `ae524bedf` (19.11.), `dd895a253` (17.11.) existieren | `git cat-file` |
| `e5ec76c28` Inhalt | Ingest, Embedder, Candle eingefroren | zusätzlich `contextra-wire`, `contextra-py`, `contextra-prompt-guard`, `contextra-infer-ollama` | Commit-Message |
| Vektor-Modul ACORN | „ACORN-artige Traversierung“ | `contextra-vector/src/acorn/` vorhanden | `ls` |
| Vektor-Modul HNSW | vorhanden | `contextra-vector/src/hnsw/` vorhanden | `ls` |

Die Zeilenzahlen der Zielspalte (Abschnitt 3.1) sind Schätzungen mit ±25 % Spanne.

## Anhang C: Glossar

| Begriff | Bedeutung |
|---|---|
| **Seal** | Unveränderliche Inhaltseinheit einer Dokumentrevision mit eigenem Schlüssel und dichter Ordinalzahl |
| **SegKey** | Schlüssel eines abgeleiteten Segments (Postings, Vektor- und Graphstruktur). Vernichtung erst nach Neuschreiben (ADR-31) |
| **Gate** | Mandantenweite Lesesperre über Seal-Ordinale (`Open`, `Restricted`, `Erased`) und `tenant_erased` |
| **LiveMask** | Abgeleitete Bitmap lebender Chunk-Slots im Suchpfad |
| **Scrub** | Gebündelte Bereinigung abgeleiteter Strukturen nach Neuschreiben der Segmente |
| **Konsolidierung** | Ersetzen toter Nachbarn in HNSW- oder Graph-Nachbarlisten durch deren lebende Nachbarn |
| **Routen-erhaltender Scrub** | Ersetzen des SQ8-Codes eines toten Knotens durch den Mittelwert lebender Nachbarn |
| **Provenienz** | Menge der Seals, aus denen eine Kante oder Entität stammt |
| **PathRAG** | Traversierung eines Entitätsgraphen mit Pfadbewertung und Belegen |
| **Ledger** | Nur-anhängen-Protokoll der Löschaufträge ohne Kennungen und Inhalte |
| **Senke** | Unabhängiger Speicher für Gate- und Vernichtungsereignisse, nicht Teil der Datensicherung |
| **Commit-Lock** | Mandantenweiter Lock: Ingest-Commits lesen, Löschung schreibt (REQ-CON-1b) |
| **ContentRef** | Undurchsichtiger Verweis der Versionierung auf einen Seal |
| **Compat-Auflösung** | Entfernen der Bridge-Schichten und Verschieben ihrer Inhalte in die Besitzer-Module (12.5) |

## Anhang D: `benchmarks/budgets.toml` (Startwerte)

```toml
# Startwerte. Jeder Wert wird erst verbindlich, wenn der Benchmark ihn misst (INV-8).
[search.s]
hybrid_p50_ms = 15
hybrid_p95_ms = 40
filtered_factor_max = 1.5
recall_at10_min = 0.95
qps_min = 200
ram_per_chunk_kb_max = 1.5
cold_start_ready_s = 10

[search.l]
hybrid_p50_ms = 30
hybrid_p95_ms = 80
filtered_factor_max = 1.5
recall_at10_min = 0.95
qps_min = 100
ram_per_chunk_kb_max = 1.5
cold_start_ready_s = 60

[graph]
pathrag_extra_p95_ms_s = 25
pathrag_extra_p95_ms_l = 50
ingest_relations_per_s = 20000
ram_per_entity_b = 96
ram_per_edge_b = 48

[ingest]
chunks_per_s_per_shard = 2000
chunker_mb_per_s_per_core = 5
bulk_factor_min = 1.5

[erasure]
e1_p99_ms_1000_seals = 5
e2_p99_s_1000_seals = 1
tenant_offboarding_ms = 100
search_overhead_max_pct = 3
ingest_ratio_min_pct = 95
search_overhead_during_scrub_max_pct = 5
ram_overhead_max_pct = 1

[keys]
argon2_m_min_mib = 64
argon2_t_min = 3
argon2_p_min = 1
slot_size_bytes = 128
```

## Anhang E: Schichtregeln für G-03 (maschinenlesbar)

```toml
# Erlaubte interne Abhängigkeiten je Crate. Prüfung über cargo metadata.
# Dev-Abhängigkeiten werden nur auf contextra-testkit geprüft.
[allowed]
contextra-ids          = []
contextra-env          = ["contextra-ids"]
contextra-sys          = []
contextra-simd         = []
contextra-durable-fs   = ["contextra-env", "contextra-ids", "contextra-sys"]
contextra-ports        = ["contextra-ids"]
contextra-crypto       = ["contextra-ports", "contextra-ids", "contextra-env", "contextra-durable-fs"]
contextra-erasure      = ["contextra-ports", "contextra-ids", "contextra-env", "contextra-durable-fs"]
contextra-version      = ["contextra-ports", "contextra-ids", "contextra-env"]
contextra-store        = ["contextra-ports", "contextra-ids", "contextra-env", "contextra-sys", "contextra-durable-fs"]
contextra-vector       = ["contextra-ports", "contextra-ids", "contextra-env", "contextra-simd", "contextra-durable-fs"]
contextra-text         = ["contextra-ports", "contextra-ids", "contextra-env", "contextra-durable-fs"]
contextra-graph        = ["contextra-ports", "contextra-ids", "contextra-env", "contextra-durable-fs"]
contextra-rank         = ["contextra-ports", "contextra-ids"]
contextra-privacy      = ["contextra-ids", "contextra-env"]
contextra-checkpoint   = ["contextra-ports", "contextra-ids", "contextra-env", "contextra-durable-fs"]
contextra-ingest       = []
contextra-embed        = ["contextra-ports", "contextra-ids"]
contextra-db           = ["contextra-crypto", "contextra-erasure", "contextra-version", "contextra-store", "contextra-vector", "contextra-text", "contextra-graph", "contextra-rank", "contextra-privacy", "contextra-checkpoint", "contextra-ports", "contextra-ids", "contextra-env", "contextra-ingest", "contextra-embed"]
contextra              = ["contextra-db", "contextra-ports", "contextra-ids", "contextra-privacy"]
contextra-mcp          = ["contextra", "contextra-ports", "contextra-privacy"]
contextra-embed-http   = ["contextra-ports"]
contextra-kms          = ["contextra-ports"]
contextra-server       = ["contextra", "contextra-ports"]

# Regeln, die die Tabelle nicht ausdrückt:
[rules]
e1_crates = ["crypto", "erasure", "version", "store", "vector", "text", "graph", "rank", "privacy", "checkpoint"]
e1_no_e1_dependency = true                       # R2
ports_only_ids = true                            # R1
compat_forbidden = true                          # R7, CMP-10
forbidden_crate_names = ["contextra-types", "contextra-mvcc", "contextra-engine", "contextra-vault"]
```

---

*Ende der Spezifikation v4.1.*
