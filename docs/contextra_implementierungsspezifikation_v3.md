# Contextra Implementierungsspezifikation v3.0

**Status:** Entwurf zur Umsetzung. Ersetzt `docs/SPEC.md` v2.1 für Löschkette, Lesesperre, Schlüsselmodell, Barrieren und Produktumfang. Die Löschanforderungen stammen aus der Löschspezifikation v2.2 (im Folgenden „ER“), übernommen und um Präzisierungen aus der Umsetzungsprüfung ergänzt (Kennung `P-n`).
**Geltung:** Gesamter Workspace `contextra`.
**Kennungen:** `REQ-n` Anforderungen, `INV-n` Invarianten, `REM-n` Entfernungen, `T-n` Tests, `P-n` Präzisierungen gegenüber ER v2.2, `D-n` offene Entscheidungen.

---

## 0. Zweck und Rangfolge

1. Diese Spezifikation (v3.0)
2. Löschspezifikation v2.2 (ER-Anforderungen), soweit nicht durch `P-n` präzisiert
3. Crate-Spezifikationen `docs/crates/*.md`, nachgezogen auf v3.0
4. Crate-lokale READMEs (informativ)

Für Entwicklung und Betrieb gilt: Was nicht in dieser Spezifikation steht, wird nicht gebaut.

---

## 1. Produktumfang

### 1.1 Ziel

Contextra ist eine **offline betreibbare Retrieval-Engine** für RAG-Anwendungen. Sie speichert lokale Dokumente, zerlegt sie, berechnet Vektoren lokal, indiziert sie und liefert auf Anfrage Treffer mit Quellenangabe in einem vorgegebenen Token-Budget. Die Textgenerierung erfolgt **außerhalb** von Contextra durch ein Sprachmodell des Anwenders.

Zusätzlich bleiben zwei Anforderungen aus v2.1 verbindlich, weil sie das Produkt von allgemeinen Vektordatenbanken unterscheiden:

- **Löschung** einzelner Dokumente und Personenbezüge mit Lesesperre und Schlüsselvernichtung (ER).
- **Zeitbezug** über Gültigkeitszeitraum (`valid_at`) als Suchfilter.

### 1.2 Nicht-Ziele

- Keine Textgenerierung, kein Chat, keine Agentenlogik im Kern.
- Kein Netzwerkzugriff im Produktcode. Keine Telemetrie, kein Lizenzserver.
- Keine Echtzeit- oder Regelungsfunktionen, keine Hardwareanbindung, keine Zeitgarantien für Suchantworten.
- Kein Training und keine Feinabstimmung.
- Keine automatische Entscheidung über natürliche Personen (Art. 22 DSGVO).
- Keine Aussage „gelöscht“ ohne gemessene Klasse (ER §5, ER-37, ER-41).
- Keine Serialisierbarkeit über Transaktionen. Snapshot-Isolation genügt (REM-12).
- Keine Graph-Forschungsverfahren im Kernpfad (REM-Gruppe G).

### 1.3 Abgrenzung der Funktionsklassen

| Klasse | Behandlung |
|---|---|
| **Kern-RAG** (Ingest, Chunking, Embedding, Indexierung, Suche, Fusion, Kontext, Quellen) | Vollständig umsetzen (Abschnitt 5) |
| **Spec-Ziel, RAG-fremd** (Löschung, Zeitreise, Mandanten) | Umsetzen gemäß ER und Abschnitt 6, Umfang durch D-1 bestätigt |
| **Weder RAG noch Spec** | Entfernen (Abschnitt 3) |

---

## 2. Architektur

### 2.1 Zielbestand der Crates

| Crate | Ebene | Entscheidung | Inhalt |
|---|---|---|---|
| `contextra-ids` | E0 | behalten | Typisierte IDs, Scope, `TenantScoped<T>` |
| `contextra-env` | E0 | behalten | `Clock`, `SeededRng`, `MetricsSink` (ohne Socket) |
| `contextra-sys` | E0 | behalten | mlock, Direct-IO, Permissions (unsafe-Insel) |
| `contextra-durable-fs` | E0 | behalten, ohne Fault-Injection (REM-20) | Atomares Schreiben, Journal, Scrub |
| `contextra-simd` | E0 | behalten (unsafe-Insel) | Distanzkerne mit Laufzeit-Dispatch |
| `contextra-types` | E0 | verschlanken | Kanonische Datentypen, `ContextraError`, `TokenBudget` |
| `contextra-ports` | E0 | verschlanken, ohne Abhängigkeit auf `types` (REM-30) | Traits, Port-Fehler |
| `contextra-privacy` | E0 | verschlanken | Klassifikation (regelbasiert), Egress-Guard, Verarbeitungsregister |
| `contextra-erasure` | E0 | **neu umsetzen** nach Abschnitt 6 | Löschkette, Gate, Gate-Log, Berichte |
| `contextra-store` | E1 | verschlanken | LSM, WAL, Manifest, SSTable, Blob-Segmente |
| `contextra-vector` | E1 | verschlanken | Flache exakte Suche (Pflicht), HNSW (optional, D-3) |
| `contextra-text` | E1 | verschlanken | BM25, Tokenizer, deutsche Komposita, Stoppwörter |
| `contextra-graph` | E1 | stark verschlanken | Kanten, Provenienz, Kaskade, Pfadsuche (optional) |
| `contextra-rank` | E1 | verschlanken | RRF, Top-k, Erklärung, Zeitfilter |
| `contextra-mvcc` | E1 | verschlanken | Snapshot-Isolation, Zeitreise-Snapshots, kein SSI (REM-12) |
| `contextra-crypto` | E1 | verschlanken | AEAD, KeyManager, KeyTable, Custody-Anker |
| `contextra-checkpoint` | E1 | vereinfachen (D-5) | Snapshot-Export und Wiederherstellung |
| `contextra-ingest` | E2 | **neu** (aus Historie, siehe REQ-RAG-1) | Parser (Feature `ingest-docs`) und Chunker |
| `contextra-embed` | E2 | **neu** | Lokales Embedding über Feature, Mock nur in Tests |
| `contextra-db` | E2 | umsetzen | Orchestrierung, Lineage, Lesepfad mit Gate |
| `contextra` | E2 | **reparieren** (REQ-BLD-1) | Öffentliche Fassade |
| `contextra-mcp` | E3 | reparieren, verschlanken | MCP-Server über stdio |
| `contextra-testkit` | E3 | nicht im Standard-Build | Fault-Injection, Canary, Fakes |
| `contextra-bench` | E3 | reparieren | Benchmarks, Budgets |

Entfernt: `contextra-engine` (Deprecated-Stub), `contextra-vault` (ausgeschlossen, REM-1).

**REQ-ARC-1.** Die Crate-Liste ist in §2.1 von SPEC v2.1 auf den Stand dieser Tabelle zu bringen. Zwei Crates kommen hinzu (`contextra-ingest`, `contextra-embed`). Die Begrenzung „23 Crates“ entfällt.

### 2.2 Abhängigkeitsregeln

Die Regeln R1 bis R6 aus SPEC v2.1 §2.3 gelten unverändert, mit folgenden Präzisierungen:

- **R1:** Kanten zeigen abwärts. `contextra-ports` darf nicht von `contextra-types` abhängen (Zyklusvermeidung). Typen, die Ports brauchen, liegen in `contextra-ids` oder werden über Generics übergeben (REM-30).
- **R1a (neu):** E1-Crates dürfen keine anderen E1-Crates als Abhängigkeit haben. Ausnahme: explizit in Crate-Spezifikation genannt. Insbesondere `contextra-store` hängt nicht von `contextra-crypto` oder `contextra-mvcc` ab. Schlüsselzugriff läuft über `ports`-Traits (REM-31).
- **R2:** Keine Dev-Abhängigkeiten auf höhere Ebenen. Verstöße (`contextra-checkpoint`, `contextra-engine`) werden behoben.
- **R3:** `default = []` in allen Crates. Experimentelle Features sind nie Default.
- **R4:** Jedes Konzept genau einmal definiert. Insbesondere `MAX_SEARCH_K` nur in `contextra-ids` (REQ-BLD-4). Kryptoprimitive (AES-GCM-SIV, HKDF, HMAC, SHA-2, Argon2, Ed25519) nur in `contextra-crypto`.
- **R5:** `#![forbid(unsafe_code)]` außer `sys` und `simd`. Workspace-Lint auf `forbid` setzen (REQ-BLD-3).
- **R6:** Kein `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!` in Nicht-Testcode.

### 2.3 Laufzeitgrenzen

**INV-ARC-1.** Der Produktcode enthält keine Netzwerk-Crates (Liste in REQ-BLD-6), keine Zeitfristen auf dem Suchpfad (INV-DET-1), keine Hardware-Sensorik und keine Zustände, die Lesen blockieren ohne Löschgrund.

---

## 3. Entfernungen (REM)

Die Entfernungen sind vor allen Neuimplementierungen durchzuführen (Phase 1). Jede hat ein Abnahmekriterium als Suchbefehl, der außerhalb von Testcode keine Treffer liefert.

### 3.1 Betriebsprofile und Lizenz

| ID | Entfernen | Ort | Abnahme |
|---|---|---|---|
| REM-01 | Crate `contextra-vault` und Verzeichnis | `crates/contextra-vault/`, `exclude` in Workspace | Verzeichnis existiert nicht |
| REM-02 | Profil `BareMetal` und Durability-Modus `MemoryOnly` | `contextra/src/performance_profile.rs`, `contextra-store/src/lsm/ops/write/*`, Observer `requires_durability` | `grep -rn "MemoryOnly\|BareMetal"` ohne Treffer |
| REM-03 | Durability-Modus `WalNoHmac` als wählbares Profil | `performance_profile.rs` | Nur `Full` existiert |
| REM-04 | Löschmodus `TombstoneOnly` | `contextra/src/collection_profile.rs`, `contextra-crypto/src/kv_segment/delete_mode.rs` | Nur `CryptoShred` (Abschnitt 6.9) |
| REM-05 | Profil `EdgeMinimal` und `PowerUserLocal` als Tiers | `collection_profile.rs`, MCP-Schema | Ersetzt durch Konfiguration (Abschnitt 4.5) |
| REM-06 | Lizenzreste: `LicenseGate`, `with_license_gate`, `LicenseRequired`, `enforce_license`, `FeatureRing` | `contextra-ports/src/license.rs`, `builder.rs`, `performance_profile.rs` | `grep -rn "License\|FeatureRing"` ohne Treffer |
| REM-07 | `VectorDeleteMode` (Standard `BackgroundRepair`) | `contextra-ports/src/license.rs` | `grep -rn "BackgroundRepair\|SynchronousRepair"` ohne Treffer |
| REM-08 | Hardware-Notfallbezug im Kommentar | `contextra-crypto/src/anti_tamper.rs` | `grep -rn "hardware sensors\|crash/voltage"` ohne Treffer |

### 3.2 Fail-Open und Zeitfristen

| ID | Entfernen | Ort | Abnahme |
|---|---|---|---|
| REM-09 | Observer-Subsystem mit Latenzfrist und Circuit-Breaker | `contextra-store/src/lsm/observer/`, `observer.rs`, `engine.rs` (Registrierung) | Verzeichnis existiert nicht |
| REM-10 | Zeitbudget im Ranking (`dibud/`) | `contextra-rank/src/dibud/` | Verzeichnis existiert nicht |
| REM-11 | Zeitabhängige Drosselung ohne `Clock` (`bytes_per_sec` mit `thread::sleep`) | `contextra-durable-fs/src/scrub.rs` | Drosselung nur über `Clock` (REQ-TIM-2) |

### 3.3 Transaktionen und Store

| ID | Entfernen | Ort | Abnahme |
|---|---|---|---|
| REM-12 | SSI (serialisierbare Snapshot-Isolation) | `contextra-mvcc/src/ssi.rs` und Aufrufe | Datei existiert nicht |
| REM-13 | Kostenbasierte Kompaktionsplanung | `contextra-store/src/compaction/adaptive/` | Einfache Strategie (Größenschwelle) ersetzt sie |
| REM-14 | Fault-Injection im Produktcode | `contextra-durable-fs/src/fault_injector.rs`, `contextra-store/src/io_seam.rs` (Produktpfad) | Nur in `contextra-testkit` |
| REM-15 | Health-Zustand `Poisoned` als unwiderruflicher Zustand | `contextra-durable-fs/src/health.rs` | Ersetzt durch Abschnitt 4.2 |

### 3.4 Graph und Ranking ohne RAG-Bezug

| ID | Entfernen | Ort | Abnahme |
|---|---|---|---|
| REM-16 | Perkolation (`physio-percolation`) | `contextra-graph/src/percolation.rs` | Datei existiert nicht |
| REM-17 | TL-HFD-Diffusion, APPRH-Shadow, Session-DAG, Textgenerator-Port im Graph, Hyperedge-Vorschläge per LLM | `contextra-graph/src/tl_hfd/`, `apprh/`, `session_dag.rs`, `text_gen.rs`, `hyperedge_suggest.rs` | Verzeichnisse existieren nicht |
| REM-18 | Dichte PPR-Matrix und PPR-Stream | `contextra-graph/src/ppr/dense.rs`, `ppr_stream.rs` | Nur Forward-Push bleibt (optional) |
| REM-19 | Kalibrierung (Platt, Isotonic, Conformal), Resonanz-Fusion, Fusionsvarianten ohne Bedarf (`ext`, `global`, `normalized`) | `contextra-rank/src/calibration/`, `fusion/resonance.rs`, `fusion/ext.rs`, `fusion/global.rs`, `fusion/normalized.rs` | Dateien existieren nicht. RRF, Top-k, Erklärung, Zeitfilter, Drift bleiben. |

### 3.5 Schlüssel- und Schutzreste

| ID | Entfernen | Ort | Abnahme |
|---|---|---|---|
| REM-20 | Testhilfen im Produkt: Fault-Injection-Hooks, `fault_vfs` | `contextra-durable-fs`, `contextra-testkit` (in Standard-Members) | Testkit nicht in `default-members` |
| REM-21 | Erasure-Altcode: `vault/`, `types_local.rs`, `crypto_local.rs`, Saga-Zustände 2.0 (`Gated`→bleibt, `KeyRevoked`, `Purging`, `Verifying`) | `contextra-erasure/src/` | `pub mod vault` existiert nicht |
| REM-22 | Kryptoduplikate | `contextra-erasure` (`sha2`, `hmac`), `contextra-engine` (`sha2`), `contextra-types` (`sha2`) | Nur `contextra-crypto` hat diese Abhängigkeiten |
| REM-23 | KV-Segment-Eviction, falls nicht im Produktpfad | `contextra-crypto/src/kv_segment/` | Prüfung per `cargo tree` und Suche nach Bezeichnern (D-6) |
| REM-24 | Angriffsabwehr-Stub | `contextra-mcp/src/prompt_injection.rs` | Datei existiert nicht |
| REM-25 | Sandbox mit flüchtigen Ergebnissen | `contextra-mcp/src/sandbox.rs` | Datei existiert nicht |
| REM-26 | Re-Export-Modul ohne Eigenfunktion | `contextra-mcp/src/bulk_exfiltration_detector.rs` | Nutzung direkt aus `contextra-privacy` |
| REM-27 | LLM-Klassifikation im Egress-Vault | `contextra-privacy/src/egress_vault.rs` | Regelbasierte Klassifikation (REQ-PRV-2) |
| REM-28 | LLM-Umformulierer als Teil des Kerns | `contextra/src/query_rewriter.rs`, `contextra-ports/src/query_rewriter.rs` | Entfällt aus Kern. Optionales Plugin außerhalb (D-7). |
| REM-29 | Gedächtnis-Typen für Agenten im Typkern (Zettelkasten-Links, `WorkflowState`, `MemoryType`, Verfallskonstanten) | `contextra-types/src/types/domain/misc.rs` | Nur noch Typen, die Ingest, Suche und Löschung brauchen |
| REM-30 | Abhängigkeit `ports → types` | `contextra-ports/Cargo.toml` | Siehe R1 |
| REM-31 | Abhängigkeit `store → crypto`, `store → mvcc` | `contextra-store/Cargo.toml` | Siehe R1a |
| REM-32 | Ollama-, ONNX- und Candle-Pfade in der Konfiguration ohne Implementierung | `contextra-mcp/src/config.rs` | Ersetzt durch Abschnitt 5.5 |
| REM-33 | Verweise auf entfernte Dokumente | `CLAUDE.md`, `AGENTS.md`, Crate-READMEs | Aktualisiert auf v3.0 (REQ-DOC-1) |

Die Fundstellen sind bei Umsetzung per `grep` neu zu erheben. Die Zeilenzahlen in früheren Analysen sind Richtwerte.

---

## 4. Betriebsrahmen: Stop, Autarkie, Ressourcen

### 4.1 Betriebszustände

**REQ-OPS-1.** Ein geöffneter Mandant ist in genau einem der folgenden Zustände:

| Zustand | Lesen | Schreiben | Löschen | Ursache |
|---|---|---|---|---|
| `Ready` | ja | ja | ja | Normalbetrieb |
| `ReadOnly` | ja, aus letztem gültigen Snapshot | nein | nein | Schreibfehler mit intaktem Lesepfad (fsync, ENOSPC) |
| `Quarantine` | nein | nein | nein | Verlust der Konsistenz des Löschjournals oder des Gate-Logs, fehlende Senke bei Restore |
| `Refused` | nein | nein | nein | Start verweigert (Konfiguration, Format, Schlüssel) |

**INV-OPS-1.** Ein Schreib- oder Löschfehler führt nie zu `Quarantine`, außer er betrifft das Gate-Log oder das Löschjournal. Ein fehlgeschlagener fsync bei Dokumentschreibvorgängen führt zu `ReadOnly`.

**REQ-OPS-2 (Übergänge).** `ReadOnly` geht nach erfolgreicher Prüfung der Dateistruktur (Abschnitt 4.3) und einem erfolgreichen fsync-Probeschreibvorgang auf `Ready` über. Dieser Übergang ist ohne Neustart möglich. Er wird im Health-Bericht und im Audit-Log vermerkt.

**REQ-OPS-3 (Löschpfad bei ReadOnly).** Im Zustand `ReadOnly` werden laufende Löschungen fortgesetzt, sobald Schreiben wieder möglich ist. Neue Löschaufträge werden angenommen und als `Requested` durabel protokolliert, sofern das Journal schreibbar ist. Sonst sind sie abzulehnen.

**P-1 (Ersetzt Poisoned).** Der Zustand `Poisoned` aus `contextra-durable-fs` entfällt. Die Schreibsperre ist ein Attribut des Zustands `ReadOnly` mit einem typisierten Grund (`FsyncFailed`, `NoSpace`, `IoError`).

### 4.2 Fehlerklassen

| Klasse | Beispiel | Reaktion |
|---|---|---|
| `Transient` | I/O-Fehler auf Journal beim Schreiben | Wiederholung mit Backoff, dann `ReadOnly` |
| `Persistent` | fsync schlägt wiederholt fehl | `ReadOnly`, Alarm |
| `Integrity` | Prüfsummenfehler im Journal oder Gate-Log | `Quarantine` des betroffenen Mandanten |
| `Config` | Ungültige Konfiguration beim Öffnen | `Refused` mit Liste aller Fehler |
| `Crypto` | AEAD-Fehler | einheitlicher Fehler ohne Unterscheidung (Erased-Äquivalent auf dem Lesepfad) |

**INV-OPS-2 (Fail-Closed beim Lesen).** Fehlt ein Schlüssel oder ist ein Gate-Zustand unbekannt, liefert der Lesepfad `Erased`, nie Klartext.

**INV-OPS-3 (Kein stiller Verlust).** Jeder Fehler wird mit Typ, Mandant, Zeitpunkt und Zustandsübergang im Audit-Log festgehalten, bevor die Antwort erfolgt.

### 4.3 Startsequenz

Der Start ist in festen Phasen aufgebaut. Schlägt eine Phase fehl, endet der Start mit `Refused` und einer vollständigen Fehlerliste.

| Phase | Inhalt |
|---|---|
| `C0` | Konfiguration prüfen (Abschnitt 8) |
| `C1` | Datenverzeichnis beschreibbar, Plattenreserve ≥ `disk_reserve_bytes` |
| `C2` | Schlüsselanker öffnen (Passphrase oder Anker), KDF-Parameter prüfen |
| `C3` | Manifest, WAL, SSTable-Prüfsummen |
| `C4` | Löschjournal und Gate-Log wiedergeben (ER-33), Gate-Snapshot laden |
| `C5` | Offene Löschvorgänge fortsetzen oder als `Pending` einreihen |
| `C6` | Index-Konsistenz: Anzahl Vektoren, Textstatistik gegen Segmentköpfe |
| `C7` | Zustand `Ready` oder `ReadOnly` setzen, Lesepfade freigeben |

**REQ-OPS-4.** Lesepfade werden erst nach `C4` freigegeben (ER-33). Vor `C4` liefert jede Anfrage `Erased` oder einen typisierten Fehler.

### 4.4 Autarkie

**REQ-AUT-1 (Netzfreier Produktcode).** Der Produktbau (`cargo build --workspace`, Default-Features) enthält keine Crates aus folgender Liste, direkt oder transitiv: `reqwest`, `hyper`-Client-Features, `ureq`, `tonic`, `h2`, `native-tls`, `openssl`-Client. Prüfung über `cargo tree -e normal` als CI-Gate.

**REQ-AUT-2 (Netzfreier Testlauf).** Die Integrationstests laufen in einem Netzwerk-Namensraum ohne Schnittstellen (`unshare -n` oder gleichwertig). Ein Test, der Netzwerk benötigt, schlägt fehl.

**REQ-AUT-3 (Lokale Modelle).** Embedding-Modelle und Reranker werden als Dateien geladen. Die Konfiguration enthält den erwarteten SHA-256-Wert. Abweichung führt zu `Refused`. Modelle werden nie zur Laufzeit nachgeladen.

**REQ-AUT-4 (Lokale Dokumente).** Parser verarbeiten nur lokale Dateien und Datenströme. Externe Ressourcen (Links, eingebettete Bilder aus URLs, Schemata) werden nicht aufgelöst.

**REQ-AUT-5 (Kein Lizenzmodell).** Keine Lizenzprüfung, kein Schlüsselserver, keine Aktivierung.

**REQ-AUT-6 (Telemetrie).** `MetricsSink` schreibt nur in lokale Dateien oder den Speicher des Prozesses. Es gibt keinen Export.

### 4.5 Ressourcen

**REQ-RES-1 (Speicherbudget).** Die Konfiguration `memory_budget_bytes` begrenzt den Arbeitsspeicher für Memtable, Blockcache und Vektorarena. Der Wert wird beim Start gegen die verfügbaren Ressourcen geprüft. Die Überschreitung löst Flush aus, nie Abbruch.

**REQ-RES-2 (Profile durch Konfiguration).** Es gibt kein Profilsystem. Die Parameter (Speicherbudget, Blockcachegröße, Gruppen-Commit-Fenster, Schwellen für Flush und Verdichtung) sind Konfigurationswerte mit Startwerten (Abschnitt 8). Die Löschparameter sind nicht Teil davon (INV-DEL-2).

**REQ-RES-3 (Begrenzte Warteschlangen).** Warteschlangen sind begrenzt. Ist die Grenze erreicht, wird der Aufrufer mit `Backpressure` abgewiesen. Blockieren ohne Grenze ist verboten.

**REQ-RES-4 (Festgelegte Obergrenzen).** Obergrenzen (Trefferzahl `k`, Dokumentgröße, Chunkzahl je Dokument, Batchgröße) stehen einmal in `contextra-ids` und sind über Konfiguration veränderbar, nicht pro Crate definiert.

### 4.6 Determinismus

**INV-DET-1.** Gleiche Eingabe bei gleichem Datenbestand liefert identische Rangfolge und identische Trefferliste. Zeitfristen, Lastabhängigkeit und Zufall beeinflussen das Ergebnis nicht. Tie-Break ist die aufsteigende `DocId`.

**REQ-TIM-1.** Alle Zeitquellen laufen über `Clock` aus `contextra-env`. Direkte Aufrufe von `Instant::now` oder `SystemTime::now` im Produktcode sind verboten, außer in `Clock`-Implementierungen.

**REQ-TIM-2.** Drosselung (REM-11) nutzt `Clock` und ist in Tests deterministisch (`ManualClock`).

**REQ-TIM-3.** Die einzigen Zeitfristen im Produkt sind die Löschfristen (ER-14, ER-32) und Netzwerk-Timeouts des Egress-Kanals, die beim Egress fail-closed wirken.

---

## 5. RAG-Kernpipeline

### 5.1 Ingest

**REQ-RAG-1 (Chunker wiederherstellen).** Der Chunker (`MarkdownChunker`, `ChunkerConfig`) wird aus der Git-Historie des Repositorys in `contextra-ingest` übernommen. Fundstelle per `git log --all -- '*chunker*'`. Anpassungen: keine Abhängigkeit auf entfernte Crates, Ausgabe mit Zeichenoffsets und Seitenangabe, Chunk-ID deterministisch aus `DocId` und Index.

**REQ-RAG-2 (Chunk-Parameter).** Standard: Zielgröße 512 Tokens, Überlappung 64 Tokens, Trennung an Absatz-, Satz- und Abschnittsgrenzen in dieser Reihenfolge. Die Tokenzählung ist eine lokale Schätzung über Zeichen und Wortgrenzen (kein Modell zur Laufzeit). Parameter sind konfigurierbar.

**REQ-RAG-3 (Chunk-Eigenschaften).** Jeder Chunk trägt: `doc_id`, `version`, `chunk_index`, Zeichenbereich im Quelltext, Seitenangabe (falls vorhanden), Abschnittspfad (falls vorhanden), `source_uri` (lokaler Pfad oder Kennung). Diese Felder sind Teil der Quellenangabe in Treffern.

**REQ-RAG-4 (Formate).** Feature `ingest-docs` (nicht Default) liefert Parser für PDF (Text, keine OCR), DOCX, XLSX (Tabellen als Text je Blatt), HTML (Textextraktion ohne Skripte), Markdown und Klartext. Unbekannte Formate werden abgewiesen, nie raten.

**REQ-RAG-5 (Ingest-Fehler).** Ein fehlerhaftes Dokument bricht den Stapel nicht ab. Der Bericht listet je Dokument: Erfolg, Teilerfolg (mit Hinweisen) oder Fehler mit Typ.

**REQ-RAG-6 (Personenbezug beim Ingest).** Ein Dokument mit Personenbezug braucht deklarierte Betroffene (ER §5 / §9.1 v2.1, ER-I-1). Fehlt die Deklaration, wird das Dokument als Klasse K mit Hinweis `UnclassifiedPersonalData` gespeichert. Der Trockenlauf (ER-14) prüft es nachträglich.

### 5.2 Embedding

**REQ-EMB-1 (Port).** `contextra-ports` definiert `EmbeddingProvider` mit `dim()`, `model_id()`, `embed_batch(&[&str]) -> Vec<Vec<f32>>`. Die Modell-ID und Dimension gehen in den Index-Kopf ein.

**REQ-EMB-2 (Lokale Backends).** `contextra-embed` bietet je ein Feature: `embed-onnx` oder `embed-candle` (D-2). Beide laden lokale Dateien (REQ-AUT-3). Standard: kein Backend. Ohne Backend ist die Fassade nur für Import und Suche mit vorberechneten Vektoren nutzbar.

**REQ-EMB-3 (Modellwechsel).** Ein Index ist an eine Modell-ID und Dimension gebunden. Eine Suche mit anderem Modell wird abgewiesen. Ein Wechsel erfordert Neuberechnung. Die Drift-Prüfung (`drift.rs`) vergleicht Modell-ID und Stichprobenstatistik und meldet Abweichung.

**REQ-EMB-4 (Mock).** Der Mock-Embedder liegt in `contextra-testkit`. Er ist im Produktcode nicht erreichbar.

### 5.3 Index

**REQ-IDX-1 (Flache exakte Suche).** Pflicht ist die exakte Suche über alle lebenden Vektoren mit SIMD-Distanz (Kosinus oder Skalarprodukt auf normierten Vektoren). Vektoren werden als fp32 gespeichert. Dies ist der Referenzpfad für Korrektheit.

**REQ-IDX-2 (Quantisierung optional).** SQ8 als optionales Format. Die Rangfolge der Top-k-Kandidaten wird mit fp32 nachgerechnet, bevor Treffer ausgegeben werden (Präzisionsschritt).

**REQ-IDX-3 (HNSW optional, D-3).** HNSW ist Feature `index-hnsw`, nicht Default. Ist es aktiv, gelten ER-24 und ER-25 (Neuaufbau bei Tombstone-Anteil) und die Rangfolge muss gegen die flache Suche geprüft werden (Rückrufmessung im Benchmark).

**REQ-IDX-4 (Textindex).** BM25 mit Parametern `k1` und `b` aus Konfiguration. Deutsche Komposita werden zerlegt. Stoppwörter für Deutsch und Englisch. Die Domänenwörterbücher (Medizin, Recht) sind Features, nicht Default.

**REQ-IDX-5 (Inkrementelle Statistik).** Gemäß ER-21 und ER-22. Verringerung bei Löschung im selben Schreibschritt wie der Gate-Eintrag.

**REQ-IDX-6 (Segmentbindung).** Jedes Indexsegment trägt genau eine Quellklasse (ER-I-2) und genau einen Schlüsselbezug (ER-26).

### 5.4 Suche und Fusion

**REQ-SRC-1 (Ablauf).**

1. Gate-Prüfung der Anfrage (Mandant, Collection, Filter).
2. Vektorsuche (flach) und Textsuche jeweils mit Kandidatenzahl `k_cand = max(k, 50)`, begrenzt durch `MAX_SEARCH_K`.
3. Gate-Prüfung je Kandidat (ER-5). Gelöschte Kandidaten werden verworfen und die Liste bis `k_cand` aufgefüllt, soweit möglich.
4. Filter: Gültigkeit (`valid_at`), Versionsstand (`as_of`), Metadaten.
5. Fusion mit Reciprocal Rank Fusion, `k = 60`, Gewichte aus Konfiguration.
6. Auswahl der Top-k, Tie-Break `DocId` aufsteigend (INV-DET-1).
7. Ausgabe mit Provenienz und optionaler Erklärung.

**REQ-SRC-2 (Zeitreise).** `as_of` wählt den Snapshot per Sequenznummer. `valid_at` filtert nach Gültigkeitszeitraum. Das Gate wird immer gegen den aktuellen Stand geprüft (ER-6).

**REQ-SRC-3 (Trefferformat).** Jeder Treffer enthält: `chunk_id`, `doc_id`, `version`, `score`, `provenance` (wie REQ-RAG-3), `signals` (Rang je Kanal), `erklärung` (optional).

### 5.5 Kontext und Token-Budget

**REQ-CTX-1.** `assemble_context(hits, TokenBudget) -> ContextPack` wählt Treffer in Rangfolge, bis das Budget erreicht ist. Gekürzte Chunks werden am Satzende gekürzt, nicht mitten im Wort. Die Quellenangaben bleiben vollständig.

**REQ-CTX-2.** `ContextPack` enthält die Liste der verwendeten Chunks mit Quellenangabe und eine Liste der ausgelassenen Treffer mit Grund (`Budget`, `Erased`, `Duplicate`).

**REQ-CTX-3.** Duplikate (gleicher Textinhalt aus verschiedenen Versionen) werden nur einmal aufgenommen, mit der neuesten Version.

### 5.6 Öffentliche Fassade

Die Fassade stellt genau diese Operationen bereit. Weitere Funktionen erfordern eine Spezifikationsänderung.

| Operation | Zweck |
|---|---|
| `Contextra::open(config) -> Result<Contextra, OpenError>` | Startsequenz (Abschnitt 4.3) |
| `ingest(collection, SourceDoc, IngestOptions) -> IngestReport` | RAG-Ingest |
| `search(SearchQuery) -> SearchResponse` | Hybridsuche mit Treffern |
| `assemble(SearchResponse, TokenBudget) -> ContextPack` | Kontext |
| `erase(ErasureRequest) -> ErasureTicket` | Löschauftrag (Abschnitt 6) |
| `erase_status(TicketId) -> ErasureStatus` | Status und Berichtsverweis |
| `health() -> Health` | Zustand, Gründe, Zähler |

`remember`, `recall` und `forget` aus `agent_memory.rs` entfallen aus dem Kern (D-7). Sie sind bei Bedarf ein Adapter über `search` und `ingest`.

### 5.7 MCP-Schnittstelle

**REQ-MCP-1.** Der MCP-Server exponiert dieselben Operationen wie die Fassade, über stdio. Tool-Schemata werden aus den Fassadentypen erzeugt, nicht von Hand gepflegt.

**REQ-MCP-2.** Standardwerte in Schemata sind verschlüsselte Collections und Standardkonfiguration. Es gibt keinen Tier-Parameter (REM-05).

**REQ-MCP-3.** Die Fehlermeldung für nicht eingebaute Backends nennt das Feature, das zu aktivieren ist, und verweist nicht auf entfernte Provider.

### 5.8 Datenschutzfilter

**REQ-PRV-1.** Entity-Erkennung (PII) ist regelbasiert (Muster, Prüfziffern, Listen). Sie dient der Klassifikation bei Ingest (REQ-RAG-6) und der Ausgabeprüfung.

**REQ-PRV-2.** Die Klassifikation über ein Sprachmodell entfällt (REM-27). Unsicherheit wird als `Unclassified` markiert und führt nicht zu automatischer Freigabe.

**REQ-PRV-3.** Der Egress-Guard prüft jede Ausgabe nach außen (MCP-Antwort, Export) gegen die Klassifikation. Die Prüfung ist fail-closed. Timeouts sind nicht vorgesehen, weil die Prüfung lokal und ohne E/A erfolgt.

---

## 6. Löschkette v3.0

Dieser Abschnitt ist die Umsetzung von ER v2.2. Präzisierungen und Festlegungen stehen in `P-n`. Wo ER und `P-n` widersprechen, gilt `P-n`.

### 6.1 Präzisierungen zu ER v2.2

**P-2 (Zustandsnamen).** Der kanonische Zustandssatz ist: `Requested`, `Gated`, `Cleaning`, `Cleaned`, `MeasuredA`, `KeyDestroyBegun`, `KeyDestroyed`, `MeasuredB`, `Reported`, `Completed`. Dazu `Restricted{grund, bis}`, `Released`, `Refused`, `Pending{grund, fortsetzen}`. ER-47 und ER-33 werden hierauf angeglichen. `Measured` aus ER-47 entfällt.

**P-3 (Frühe Vernichtung, ER-31).** ER-31 bricht INV-DEL-ORDER-1 aus SPEC v2.1. Die frühe Vernichtung wird in v3.0 **nicht** implementiert, bis D-1 entschieden ist. Standard ist die Reihenfolge `S3, S4, S5`. Begründung: Klartextreste in Caches, Schreibpuffern und Indizes bleiben bei früher Vernichtung unbemerkt, weil die Messung nach der Vernichtung nur noch Chiffrat sieht.

**P-4 (Trockenlauf und Standardmodus, ER-13).** `DeleteAll` als Standard löscht Dokumente, die andere Personen betreffen, ohne deren Zustimmung. Das ist nicht in jedem Fall verhältnismäßig. Umsetzung: Standardmodus ist `Restricted(MultiSubjectReview)` bei jedem Mitbetroffenen, nicht `DeleteAll`. `DeleteAll` ist nur mit ausdrücklicher Freigabe durch den Betreiber im Auftrag zulässig (D-1).

**P-5 (Flache Suche, ER-23).** ER-23 setzt fp32 und flache Suche voraus. Dies ist mit REQ-IDX-1 vereinbar, die Größenobergrenze für flache Suche ist aber zu setzen (Startwert 200.000 Vektoren je Collection). Darüber gilt die Warnung `IndexTooLargeForFlat` und REQ-IDX-3 greift (D-3).

**P-6 (Schlüsselbezug von Clustern, ER-27).** Die Cluster-ID wird aus dem Betroffenenschlüssel mit HKDF abgeleitet. Das ist ein Kennzeichen, kein Datenschlüssel, und mit ER-4 vereinbar. Die Ableitung ist nur zulässig, wenn der Betroffenenschlüssel im Anker liegt und nach Vernichtung die Cluster-ID nicht mehr zuordenbar ist. Test T-ER-17.

**P-7 (Inkrementelle Statistik und Postinglisten, ER-21).** Die Verringerung von `N`, `df` und `avgdl` ist nur für Abfragen korrekt, wenn Postinglisten die gelöschte Version nicht mehr liefern. Die Postinglisten bleiben bis zur Segmentersetzung (S3) mit der gelöschten Version erhalten. Abfragen filtern über das Gate (ER-5). Die Statistik wird damit vor S3 korrigiert, Postings erst nach S3 entfernt. Test T-ER-14.

**P-8 (Speicherformat, ER-28).** Handkodierte Formate sind nur dort verbindlich, wo Plattenbytes signiert oder gemessen werden (Journal, Gate-Log, KeyTable, Segmentkopf). Konfiguration und Berichte nutzen serde. Der Aufwand wird auf vier Formate begrenzt (Abschnitt 7).

**P-9 (Senke, ER-43 und ER-45).** `commit_key_retention` ist definiert als Dauer, die Schlüsselsicherungen mindestens abdecken müssen (Standard 400 Tage). `restore_horizon` ist die maximale Restore-Altersgrenze. Es gilt `commit_key_retention ≥ restore_horizon`. Startwert `restore_horizon = 30 Tage`. Die Prüfung erfolgt in `C0`.

**P-10 (Ein Schreiber, ER-7).** Die Barrierefreiheit ist durch genau einen Schreibtask garantiert. Mehrere Schreibanfragen werden in einer Warteschlange serialisiert (REQ-RES-3). Der Snapshot-Swap ist die einzige Sichtbarkeitsoperation.

### 6.2 Datenpartitionen und Schlüssel

Umsetzung von ER §1 und §2 mit folgenden Festlegungen:

- **REQ-KEY-1.** Jede Version eines P-Dokuments erhält einen `DocDek` (32 Byte, `OsRng::try_fill_bytes`, Fehler führt zu `Err`). Keine Schlüsselwiederverwendung über Versionen (ER-1).
- **REQ-KEY-2.** Der Wrap erfolgt mit AES-256-GCM-SIV, Nonce 96 Bit aus `OsRng`, AD = `key_id ‖ kek_epoch ‖ algo_id` (ER-3). Die Nonce wird für den Nadelscan (T-ER-07) im Inventar festgehalten.
- **REQ-KEY-3.** Die Wurzel liegt im Custody-Anker (K1 Argon2id, K2 Schlüsselmodul, K3 externer KMS). K1 ist Standard. K2 und K3 sind nicht Teil der ersten Auslieferung, ihr Port liefert `CapabilityUnsupported`.
- **REQ-KEY-4.** Schlüsselmaterial: `Zeroizing`, kein `Clone`, kein `Debug`, kein `Serialize` (INV-11).
- **REQ-KEY-5 (KeyTable-Shards).** 256 Shards nach `key_id`-Bereich (ER-12). Schreiben: `tmp → fsync → rename → fsync(Verzeichnis)` für den betroffenen Shard.

### 6.3 Lesesperre

Umsetzung von ER §3 mit den folgenden Typen. Die Mengen sind Roaring-Bitmaps über dichten Ordinalen.

```rust
pub struct GateSnapshot {
    pub erased_ver:    RoaringBitmap,              // VerOrd
    pub erased_subj:   RoaringBitmap,              // SubjOrd
    pub restricted:    BTreeMap<u32, Restriction>, // VerOrd -> Grund und Ende
    pub superseded:    RoaringBitmap,              // VerOrd
    pub redacted_k:    RoaringBitmap,              // KOrd
    pub high_water:    u32,                        // nie verkleinert
    pub seq:           u64,                        // Sequenz des letzten Log-Eintrags
}

pub enum GateDecision { Open, Erased, Restricted { audit_required: bool } }

pub trait ReadGate: Send + Sync {
    fn check_version(&self, v: VerOrd, ctx: ReadCtx) -> GateDecision;
    fn check_subject(&self, s: SubjOrd) -> bool; // true = gesperrt
}
```

- **REQ-GATE-1.** Die Prüfreihenfolge ist `erased_ver`, dann `restricted`, dann `superseded` (ER §3.2).
- **REQ-GATE-2.** Ordinale werden nie wiederverwendet. Die Zuordnung Ordinal zu ID liegt im verschlüsselten Kopf des Mandanten.
- **REQ-GATE-3 (Publikation).** Schreiber: Gate-Eintrag, `fsync`, neuer `GateSnapshot`, Tausch über `arc_swap::ArcSwap`. Leser greifen über `load()` zu (ER-7, P-10). `arc-swap` wird als Abhängigkeit aufgenommen (D-8).
- **REQ-GATE-4 (Fehler).** Schlägt `fsync` fehl, wird der Snapshot nicht getauscht und der Mandant geht in `ReadOnly` mit Grund `GateLogFsync`. Das Gate-Log ist betroffen, daher ist der Zustand `Quarantine` (ER-8). Siehe Abschnitt 4.2.

**Gate-Log-Format.** Rahmen: `[magic "CXGL" u32][version u16][seq u64][op u8][len u16][payload][mac 32]`. MAC: HMAC-SHA256 mit `HKDF(K_gate, "ctx/gate/v3" ‖ tenant)`. Die Kette verknüpft jeden Eintrag mit dem vorherigen MAC.

**REQ-GATE-5 (Verdichtung, ER-9).** Wöchentlich oder bei Schwellwert: neuer Snapshot-Eintrag mit MAC, neue Datei, atomarer Tausch, alte Datei archiviert. `erased_ver`, `erased_subj`, `redacted_k` werden nie verkleinert.

### 6.4 Lesepfade

Jeder Lesepfad prüft das Gate, bevor Schlüsselmaterial angefasst wird (ER-5). Die Prüfung erfolgt auf jedem Kandidaten.

| Pfad | Prüfung | Verhalten bei `Erased` |
|---|---|---|
| Suche (Vektor, Text) | je Kandidat vor Fusion | verwerfen, auffüllen |
| Abruf per ID | vor Entschlüsselung | `NotFound` (gleich wie nie existiert) |
| Zeitreise (`as_of`) | aktueller Stand, nicht historischer | `NotFound` |
| Cache-Treffer | `source_set ∩ erased_ver ≠ ∅` gilt als Miss | Miss |
| Export | je Eintrag | auslassen |
| Ingest-Dubletten | je Kandidat | nicht als Duplikat werten |

**REQ-GATE-6.** Ein Test prüft jeden Eintrag der Tabelle (T-ER-11).

### 6.5 Ableitungen

- **REQ-DER-1.** `Derived<T>` kapselt Nutzdaten mit `NonEmpty<SourceSet>`. `put_derived` ist die einzige Schreibfunktion für Indizes und Caches (ER-18).
- **REQ-DER-2.** Betroffenheit: `source_set ∩ erased_ver ≠ ∅` für Klasse D aus P, `source_set ∩ redacted_k ≠ ∅` für Klasse D aus K (ER-19).
- **REQ-DER-3.** Betroffene Ableitungen werden in S2 entfernt, nicht als veraltet markiert (ER-20).
- **REQ-DER-4 (Vektorarena).** Slots der Version werden nach S3 mit Nullen überschrieben (`zeroize`) und erst nach vollständigem Segmentneuaufbau wiederverwendet (ER-23).

### 6.6 Speicherformate

| Modus | Schlüssel | Einsatz | Löschwirkung |
|---|---|---|---|
| `RecordSealed` | `DocDek` je Version | P-Datensätze, Vektor- und Textpayload | Schlüsselvernichtung |
| `SegmentSealed` | `ArtKey` je Segment | K, D, Journal | Ersatz und Altsegment entfernen |

- **REQ-FMT-1.** Personenbezogene Versionen liegen in eigenen Segmenten je Betroffenencluster, sofern der Cluster größer als `cluster_seg_min` (4 MiB) ist (ER-26).
- **REQ-FMT-2.** Segmentgröße höchstens `segment_max_bytes` (256 MiB, ER-29).
- **REQ-FMT-3.** Jedes persistierte Objekt trägt Magic, Formatversion und Algorithmuskennung. Eine Änderung ohne Versionsanhebung ist ein Fehler (INV-1).

### 6.7 Löschsaga

Jeder Schritt ist idempotent. Der Journalzustand ist die Wahrheit.

| Schritt | Takt | Inhalt | Durabilitätspunkt | Zustand danach |
|---|---|---|---|---|
| **S0** | 1 | Validierung, Berechtigung, Aufbewahrungsprüfung, Trockenlauf (ER-14), Standardmodus nach P-4 | — | `Requested` |
| **S1** | 1 | Gate-Eintrag, `fsync`, Snapshot-Tausch, Ticket | Gate-Log `fsync` | `Gated` |
| **S2** | 2 | Ableitungen mit betroffenem `source_set` entfernen (REQ-DER-3), Statistik verringern (REQ-IDX-5) | Journal je Ableitung | `Cleaning` |
| **S3** | 2 | Caches zeroizen, Schreibpuffer leeren, Segmente ersetzen, Vektorslots nullen, Blobs überschreiben und entfernen, Postinglisten bereinigen (P-7) | Journal je Teilschritt | `Cleaned` |
| **S4** | 2 | Messung A mit Positivkontrolle (Abschnitt 6.8) | Messbericht | `MeasuredA` |
| **S5** | 2 | Intent-Journal, Zeroize, KeyTable-Shard ersetzen, Registereintrag | `fsync` Intent, `fsync` Shard | `KeyDestroyBegun`, dann `KeyDestroyed` |
| **S6** | 2 | Messung B: Nadelscan, Entschlüsselungsprobe, Canary | Messbericht | `MeasuredB` |
| **S7** | 3 | Epochen-Rewrap, falls ausstehend oder Schwellwert erreicht (ER-11) | Journal | unverändert |
| **S8** | Hintergrund | Verdichtung, WAL-Entfernung mit Prüfung, Checkpoint-Sweep, Index-Neuaufbau bei ER-25 | Manifest | unverändert |
| **S9** | 3 | Bericht (Abschnitt 6.10), Abschluss | Berichtsdatei `fsync` | `Reported`, dann `Completed` |

**Reihenfolge (REQ-SAG-1).** Es gilt `S3 → S4 → S5 → S6` (P-3).

**Fristen (REQ-SAG-2).** `clean_deadline` = 1 Stunde für S3. Bei Überschreitung: `Pending{CleanupTimeout}`, Alarm, und S5 darf trotzdem laufen (ER-32, fail-forward). Der Restbestand wird als `R-CLEAN-TIMEOUT` im Bericht geführt.

**Sperre (REQ-SAG-3).** Der Key-Lock der Einheit wird von S1 bis einschließlich S5 gehalten. Konkurrierende Schreibvorgänge auf derselben Einheit werden abgewiesen (ER-34).

**Koordination (REQ-SAG-4).** Pro Scope höchstens eine Saga. Ein Duplikat liefert das bestehende Ticket (ER-34).

### 6.8 Messung

| Messung | Verfahren | Ergebnis |
|---|---|---|
| **A** (S4) | Rohbytes aller Dateien nach kanonischen Termen, Roh-IDs, Embedding-Bytes, Chunk-Prüfsummen; dekodierte Sicht mit allen aktiven Schlüsseln | `reached` bei 0 Treffern |
| **Inventar** (vor S5) | Je `key_id`: Liste der Chiffratblöcke (Datei, Offset, Länge) als Digest und Zähler | Grundlage für B |
| **B1** (S6) | Nadelscan der 96-Bit-Nonce des vernichteten Schlüssels in allen Dateien einschließlich Temp und `.bak` | 0 Treffer |
| **B2** (S6) | Echte `open`-Operation auf je einem Block aus dem Inventar | muss mit `KeyAbsent` scheitern |
| **B3** (S6) | Canary mit eigenem Schlüssel über denselben Vernichtungspfad | belegt Scanner und Pfad |

- **REQ-MES-1 (Typzustand).** `Scanner<Armed>` ist nur über `Canary::verify(scanner) -> Result<Scanner<Armed>, CanaryError>` erzeugbar. Der Konstruktor ist `pub(crate)`.
- **REQ-MES-2 (Zwei Quellen).** Jede Pflichtmessung läuft über den besitzenden Scanner und über eine Black-Box-Probe des Orchestrators. Abweichung ist ein Fehler.
- **REQ-MES-3 (Ehrliche Zustände).** Jede Prüfung meldet `reached`, `not_reached`, `not_implemented` oder `sampled`. Eine nicht implementierte Prüfung wird nie als `reached` ausgegeben (ER §10.1).
- **REQ-MES-4 (Inventarpflicht, ER-38).** Blöcke ohne Zuordnung zu einem vernichteten oder lebenden Schlüssel sind ein Fehler.

### 6.9 Löschmodus

- **REQ-MOD-1.** Es gibt genau einen Löschmodus für Dokumente: Schlüsselvernichtung (`CryptoShred`). `TombstoneOnly` entfällt (REM-04).
- **REQ-MOD-2.** Klasse K ohne Personenbezug wird ebenfalls verschlüsselt (`SegmentSealed`). Löschung über Segmentersatz und Vernichtung des `ArtKey`.
- **REQ-MOD-3.** Löschung ohne Verschlüsselung ist nicht möglich. Eine Konfiguration `encryption = none` wird in `C0` abgewiesen.

### 6.10 Bericht

**REQ-REP-1 (Pflichtfelder, ER §9.1).** Erreichte Klassen (`L`, `P`, `C`, `M`) je mit `reached` oder `not_reached`. Custody-Stufe als eigenes Feld. Messverfahren, Positivkontrollen, Schichtdigests. Annahmen `A-FLASH`, `A-RAM`, `A-EXTBACKUP`, `A-KEYLEAK`, `A-HOSTHIST`. Residuen `R-CLEAN-TIMEOUT`, `R-GROUP-LINK`, `R-SAMPLE`. Löschreihenfolge. Anzahl Mitbetroffener und gewählter Modus, nie Kennungen. `subject_ref_hash` als keyed BLAKE3 unter `K_ref`. Verifikation und Validierung getrennt.

**REQ-REP-2 (Kodierung).** JCS nach RFC 8785, Signatur Ed25519 über die kanonischen Bytes. Der Signaturschlüssel liegt im Anker und ist kein Datenschlüssel. Signierte Felder ohne kanonische Kodierung sind verboten.

**REQ-REP-3 (Bescheinigung).** Einseitige Bescheinigung mit Datum, Scope-Art, Klassen, Annahmen, Ausschlüssen, Digest, Signatur. Keine IDs, Dateinamen oder Inhalte (ER-36).

**REQ-REP-4 (Außentext).** Texte nur aus der geschlossenen Satzbibliothek mit Messbindung. Verbotene Formulierungen nach ER-41 sind durch das Build-Gate gesperrt (REQ-BLD-7).

**REQ-REP-5 (Gegenzeichnung).** Im Betriebsmodus `regulated` ist eine Gegenzeichnung durch eine zweite Person Pflicht (ER-39). Im Modus `standard` ist sie optional.

**REQ-REP-6 (Unabhängiger Prüfer).** Ein eigenes Werkzeug `contextra-verify` prüft Signatur, Kanonisierung, Digests und Zustandsübergänge im Gate-Log. Es teilt keinen Code mit dem Löschkern (ER-40).

### 6.11 Wiederherstellung

**REQ-RST-1 (Zwei Spuren, ER-42).** Datensicherungen enthalten Chiffrat ohne KeyTable und ohne Anker. Schlüsselsicherungen tragen ein signiertes `valid_until`. Abgelaufene Schlüsselsicherungen werden verweigert.

**REQ-RST-2 (Senke, ER-43).** Gate-Log und Schlüsselregister werden in eine Senke gespiegelt, die nicht Teil der Datensicherung ist. Die Senke ist im Modus `regulated` Pflicht (`erasure.sink` in Konfiguration).

**REQ-RST-3 (Restore-Guard, ER-44).** Vor dem ersten Lesezugriff nach Restore: Gate-Log gegen Senke abgleichen, wiederhergestellte Versionen mit `erased_ver` erneut vernichten und bereinigen, Ereignis `restore_reapplied` schreiben. Fehlt die Senke oder ist sie veraltet: `Quarantine`. Im Modus `standard` ist ein Restore ohne Senke nur mit `--accept-unguarded` zulässig und wird im Bericht ausgewiesen.

**REQ-RST-4.** `C0` prüft `commit_key_retention ≥ restore_horizon` (P-9).

### 6.12 Zustandsautomat

```
Requested → Gated → Cleaning → Cleaned → MeasuredA → KeyDestroyBegun → KeyDestroyed → MeasuredB → Reported → Completed

Requested → Restricted{grund, bis}          (reversibel, Art. 18)
Restricted → Gated                          (Eskalation zur Löschung)
Restricted → Released                       (terminal)
Requested → Refused                         (Aufbewahrungspflicht, terminal)
jeder nicht-terminale Zustand → Pending{grund, fortsetzen} → fortsetzen
```

Terminal: `Completed`, `Released`, `Refused`.

**INV-ER-1 (Sperre bleibt).** Eine durable Sperre wird nie aufgehoben, außer durch `Restricted → Released`. Ein Löschauftrag kehrt nie in `Restricted` oder `Released` zurück (ER-I-3).

**INV-ER-2 (Sichtbarkeit).** Ab `Gated` liefert kein Standardpfad Klartext. Ab `KeyDestroyed` ist der Inhalt kryptografisch nicht zugänglich (ER-48).

**REQ-REC-1 (Wiederanlauf, ER-33).** `C4` gibt Gate und Journal vor dem Öffnen der Lesepfade wieder. Zustand zum Wiederanlauf:

| Journalzustand beim Start | Aktion |
|---|---|
| `Requested` | S0 erneut bewerten |
| `Gated` | ab S2 fortsetzen |
| `Cleaning` | S3 fortsetzen |
| `Cleaned` | S4 durchführen |
| `MeasuredA` | S5 durchführen (Intent prüfen) |
| `KeyDestroyBegun` | Vernichtung wiederholen (idempotent) |
| `KeyDestroyed` | S6 durchführen |
| `MeasuredB` | S9 durchführen |
| `Reported` | Abschluss |
| `Pending{…}` | beim nächsten Takt aufnehmen |

**REQ-REC-2.** Ein Intent ohne Ausführungsnachweis wird beim Start wiederholt. Die Vernichtung ist idempotent: Ist der Schlüssel nicht mehr auffindbar, gilt sie als vollzogen.

---

## 7. Datenformate

Vier Formate werden handkodiert (P-8). Alle anderen Persistenzformate nutzen bestehende Kodierungen mit Versionsfeld.

| Format | Magic | Inhalt | Prüfung |
|---|---|---|---|
| Löschjournal | `CXEJ` | Zustandsübergänge, je Eintrag `[len u32][seq u64][tenant u64][erasure_id 16][zustand u8][payload][crc32c u32][mac 32]` | CRC und MAC, Abbruch bei Lücke |
| Gate-Log | `CXGL` | Siehe REQ-GATE (Gate-Log-Format) | MAC-Kette |
| KeyTable-Shard | `CXKT` | `[shard u16][n u32]` plus Einträge `[key_id u64][wrap_nonce 12][wrapped 48][kek_epoch u32]` | Shard-Digest im Kopf |
| Segmentkopf | `CXSG` | `[kind u8: RecordSealed/SegmentSealed][key_ref][count u32][digest 32]` | Digest über Kopf und Records |

**REQ-FMT-4.** Die Felder werden über Puffergrenzen und Längenprüfung gelesen. Ungültige Längen führen zu `Integrity`. Es gibt keine Typumwandlung über Strukturlayouts.

---

## 8. Konfiguration

Die Konfiguration wird beim Öffnen vollständig geprüft. Ungültige Werte führen zu `Refused` mit allen Fehlern.

```rust
pub struct ContextraConfig {
    pub data_dir: PathBuf,
    pub mode: OperatingMode,            // Standard | Regulated
    pub encryption: EncryptionConfig,   // nur Required; None ist ungültig (REQ-MOD-3)
    pub custody: CustodyProfile,        // K1 | (K2, K3 abgewiesen)
    pub embedding: Option<EmbeddingConfig>,  // Modell-Datei, SHA-256, Dimension
    pub index: IndexConfig,             // flat_max_vectors, hnsw_enabled, sq8_enabled
    pub text: TextConfig,               // k1, b, Domänenwörterbücher
    pub ingest: IngestConfig,           // chunk_target, chunk_overlap, formats
    pub memory_budget_bytes: u64,       // REQ-RES-1
    pub erasure: ErasureConfig,
}

pub struct ErasureConfig {
    pub sink: Option<PathBuf>,          // Pflicht im Modus Regulated, außerhalb data_dir
    pub disk_reserve_bytes: u64,        // mindestens 1 GiB
    pub clean_deadline: Duration,       // 1 h
    pub erase_return_timeout: Duration, // 10 s
    pub epoch_len: Duration,            // 30 Tage
    pub seal_after_deletion_max: Duration, // 7 Tage
    pub key_backup_max_age: Duration,   // 30 Tage
    pub commit_key_retention: Duration, // 400 Tage (P-9)
    pub restore_horizon: Duration,      // 30 Tage (P-9)
    pub cluster_seg_min: u64,           // 4 MiB
    pub segment_max_bytes: u64,         // 256 MiB
    pub keytable_shards: u16,           // 256
    pub tombstone_rebuild_ratio: f32,   // 0,20
    pub review_remind_after: Duration,  // 14 Tage, nur Erinnerung
}
```

**Prüfungen in `C0`:**

- `data_dir` vorhanden und beschreibbar.
- Im Modus `Regulated`: `sink` gesetzt, nicht Unterverzeichnis von `data_dir`.
- `disk_reserve_bytes ≥ 1 GiB`.
- Passphrase nicht leer, KDF-Parameter mindestens m = 64 MiB, t = 3.
- `commit_key_retention ≥ restore_horizon` (P-9).
- `embedding.model_sha256` stimmt mit Datei überein (REQ-AUT-3).
- `memory_budget_bytes` ≤ verfügbarer Speicher minus Reserve.

---

## 9. Tests und Gates

### 9.1 Löschtests

| ID | Prüft | Art |
|---|---|---|
| T-ER-01 | Nach Ingest und Flush: kein Klartext-Canary in Dateien | Integration |
| T-ER-02 | Nach Löschung und Purge liefert Suche und Abruf `NotFound`, Canary-Scan 0 | Integration |
| T-ER-03 | Falscher Schlüssel führt zu Verifikationsfehler | Unit |
| T-ER-04 | Neustart nach Löschung: Version bleibt `Erased` | Integration |
| T-ER-05 | Ohne Verschlüsselung ist Konfiguration `Refused` (REQ-MOD-3) | Unit |
| T-ER-06 | Suche findet gelöschtes Dokument nach S1 nicht, auch vor S3 | Integration |
| T-ER-07 | Nadelscan nach S5 findet Nonce des vernichteten Schlüssels nicht | Integration |
| T-ER-08 | Crash-Matrix: Abbruch an jedem Zustandsübergang mit Neustart (REQ-REC-1) | Integration |
| T-ER-09 | Zwei gleichzeitige Löschaufträge für denselben Scope: ein Ticket | Integration |
| T-ER-10 | Gleichzeitiges `ingest` und `erase` derselben Einheit: eines wird abgewiesen | Loom |
| T-ER-11 | Jeder Lesepfad der Tabelle in 6.4 prüft das Gate | Integration |
| T-ER-12 | Zeitreise `as_of` auf gelöschte Version liefert `NotFound` | Integration |
| T-ER-13 | Mehrbetroffene: Standardmodus `Restricted`, keine automatische Löschung (P-4) | Integration |
| T-ER-14 | BM25-Statistik nach Löschung entspricht Neuberechnung ohne gelöschte Version | Property |
| T-ER-15 | Vektorslot nach S3 ist null, Wiederverwendung erst nach Neuaufbau | Unit |
| T-ER-16 | Gate-Log: Manipulation eines Eintrags wird erkannt (MAC-Kette) | Unit |
| T-ER-17 | Clusterkennung nach Vernichtung des Betroffenenschlüssels nicht zuordenbar (P-6) | Integration |
| T-ER-18 | Restore mit altem Schlüssel-Backup: gelöschte Versionen werden erneut vernichtet | Integration |
| T-ER-19 | Restore ohne Senke: Zustand `Quarantine` | Integration |
| T-ER-20 | Berichts-Negativsuite: Bitflip je Feld, Feldvertauschung, falscher Schlüssel, fehlende Annahmen, gefälschte Signatur | Unit |
| T-ER-21 | Messung mit fehlgeschlagener Positivkontrolle führt zu ungültigem Lauf | Unit |
| T-ER-22 | Nicht implementierte Prüfung erscheint als `not_implemented`, nie `reached` | Unit |
| T-ER-23 | Unabhängiger Prüfer `contextra-verify` bestätigt gültige Berichte | Integration |

### 9.2 Betriebstests

| ID | Prüft |
|---|---|
| T-OPS-1 | fsync-Fehler beim Dokumentschreiben: Zustand `ReadOnly`, Suche liefert weiter Ergebnisse aus letztem Snapshot |
| T-OPS-2 | Rückkehr nach fehlgeschlagenem fsync: `ReadOnly` → `Ready` nach Probeschreibvorgang |
| T-OPS-3 | Fehler im Gate-Log: `Quarantine`, keine Lesezugriffe |
| T-OPS-4 | Ungültige Konfiguration: `Refused` mit vollständiger Fehlerliste |
| T-OPS-5 | Speicherbudget erreicht: Flush ausgelöst, kein Abbruch |
| T-OPS-6 | Warteschlange voll: `Backpressure`, kein Blockieren |
| T-OPS-7 | Gleiche Anfrage zweimal unter unterschiedlicher Last: identische Trefferliste (INV-DET-1) |
| T-OPS-8 | Test ohne Netzwerk (REQ-AUT-2) läuft durch |

### 9.3 RAG-Tests

| ID | Prüft |
|---|---|
| T-RAG-1 | Chunker: Chunk-Offsets zeigen auf den Quelltext, Überlappung korrekt |
| T-RAG-2 | Chunker: Golden-Test über Referenzdokumente (Markdown, Klartext) |
| T-RAG-3 | Parser: PDF, DOCX, HTML liefern Text, fehlerhafte Datei bricht Stapel nicht ab |
| T-RAG-4 | Hybridsuche: Rückrufwert gegen Referenzdatensatz (deutsch, `benchmarks/data/de_mini`) |
| T-RAG-5 | Flache und HNSW-Suche (falls aktiv) liefern übereinstimmende Top-10 bei Rückruf ≥ Zielwert |
| T-RAG-6 | Kontextbudget: Summe der Tokens ≤ Budget, Quellen vollständig |
| T-RAG-7 | Modellwechsel: Suche mit anderer Modell-ID wird abgewiesen |
| T-RAG-8 | Treffer enthalten Provenienz (Dokument, Version, Offsets) |

### 9.4 Build-Gates

| ID | Gate | Umsetzung |
|---|---|---|
| REQ-BLD-1 | Fassade kompiliert | `cargo check -p contextra` in CI |
| REQ-BLD-2 | Gesamter Workspace kompiliert | `cargo check --workspace` in CI |
| REQ-BLD-3 | Workspace-Lint `unsafe_code = "forbid"` | `Cargo.toml` und Prüfskript |
| REQ-BLD-4 | `MAX_SEARCH_K` nur einmal definiert | `grep` im Prüfskript |
| REQ-BLD-5 | Layering-Gate nach SPEC §15 und R1 bis R4 | `crates/contextra-testkit/tests/layering.rs` |
| REQ-BLD-6 | Keine Netzwerk-Crates im Produktbaum | `cargo tree -e normal` gegen Sperrliste |
| REQ-BLD-7 | Verbotene Formulierungen (ER-41) | Prüfskript über Quelltext, Dokumentation, Berichtsvorlagen |
| REQ-BLD-8 | `default = []` in allen Crates | Prüfskript |
| REQ-BLD-9 | Keine Erwähnung entfernter Crates im Produktcode | `grep` im Prüfskript |
| REQ-BLD-10 | Konfigurationsbeispiel lädt ohne Fehler | Test |

**REQ-BLD-11 (Fehlende Dateien).** Zum Umsetzungsbeginn vorhanden sein müssen: `benchmarks/budgets.toml` mit den Werten aus Abschnitt 10, `docs/erasure.md` als Ersatz für die im Spec-Verweis genannte Löschdatei. Verweise auf fehlende Dateien sind Fehler im Prüfskript.

---

## 10. Zielwerte (Startwerte, zu messen)

Alle Werte werden im Benchmark gemessen und in `benchmarks/budgets.toml` eingetragen. Ohne Messung gelten sie nicht als erreicht (INV-7 aus SPEC v2.1).

| Größe | Startwert | Messbedingung |
|---|---|---|
| Suchlatenz p95, flach | ≤ 100 ms | 100.000 Vektoren, 384 Dimensionen, fp32, Textsignal aktiv |
| Speicher je Vektor | ≤ 1,5 × Rohgröße | fp32 plus Index und Metadaten, ohne Blockcache |
| Kaltstart bis `Ready` | ≤ 2 s | 100.000 Vektoren, Journal leer |
| Löschung bis Lesesperre (S1) | ≤ 5 ms p99 | Gruppen-Commit aktiv |
| Löschung bis Klasse P (S4) | ≤ 1 h | Ziel, keine Außenzusage |
| Rückruf HNSW gegen flach (falls aktiv) | ≥ 0,95 Top-10 | Referenzdatensatz |
| Chunker | ≥ 5 MB Text je Sekunde | Einzelkern |

---

## 11. Umsetzung

### 11.1 Phasen

**Phase 0 — Build reparieren.**

- REQ-BLD-1 bis REQ-BLD-5 umsetzen.
- Fassade so weit reparieren, dass sie kompiliert (Typen aus `contextra-db` oder entfernte Importe).
- `Cargo.lock` mit Lockfile-Version 3 erzeugen, damit Rust 1.89 es lesen kann.
- `MAX_SEARCH_K` vereinheitlichen.

**Abnahme:** `cargo check --workspace` grün.

**Phase 1 — Entfernen.**

- REM-01 bis REM-33 in der Reihenfolge der Abschnitte 3.1 bis 3.5.
- Nach jedem Schritt: `cargo check --workspace` und `cargo test --workspace`.
- Entfernungen nicht mit Verhaltensänderungen mischen. Je Entfernung ein Commit.

**Abnahme:** alle Suchbefehle aus Abschnitt 3 liefern keine Treffer außerhalb von Tests. Gesamter Umfang gegenüber Stand HEAD verringert.

**Phase 2 — Löschkette v3.0.**

- Erasure-Crate neu aufbauen nach Abschnitt 6.
- Reihenfolge: Schlüssel und Anker, Journal und Gate-Log, Gate, Saga S0 bis S3, Messung, Schlüsselvernichtung, Bericht.
- Crash-Matrix (T-ER-08) ab dem ersten Schritt, der Daten verändert.

**Abnahme:** alle T-ER-Tests grün, Crash-Matrix ohne Ausnahme.

**Phase 3 — RAG-Kern.**

- Chunker (REQ-RAG-1), Parser (REQ-RAG-4), Embedding-Port und ein lokales Backend (D-2).
- Flache Suche, BM25, RRF, Kontextbudget, Provenienz.
- Fassade und MCP-Server an den neuen Kern anbinden.

**Abnahme:** T-RAG-1 bis T-RAG-8 grün.

**Phase 4 — Betrieb und Autarkie.**

- Zustände nach Abschnitt 4, Startsequenz C0 bis C7.
- Netzfreier Testlauf (REQ-AUT-2), Gates REQ-BLD-6 und REQ-BLD-7.
- Benchmarks und `budgets.toml`.

**Abnahme:** T-OPS-1 bis T-OPS-8 grün, Zielwerte aus Abschnitt 10 gemessen und dokumentiert.

**Phase 5 — Nachweis.**

- Unabhängiger Prüfer `contextra-verify` (REQ-REP-6).
- Dokumentation auf v3.0 (REQ-DOC-1).
- Ende-zu-Ende-Lauf: Ingest, Suche, Löschung, Bericht, Prüfung.

**Abnahme:** Verifikationsbericht zu Phase 5 mit allen Ergebnissen, keine offene Entscheidung aus Abschnitt 12 ohne Festlegung.

### 11.2 Migration

**Annahme.** Es existieren keine produktiven Daten im Format des HEAD. Falls doch, ist vor Phase 2 ein einmaliger Export über die Fassade in ein neutrales Format (JSON Lines mit Quelldateien) nötig und ein Import in die neue Struktur. Dieser Export wird nach Phase 2 nicht weiter gepflegt.

### 11.3 Dokumentation

**REQ-DOC-1.** Folgende Dokumente werden auf v3.0 gebracht oder entfernt:

- `docs/SPEC.md` (§2, §5, §6, §10, §14 ersetzt durch diese Spezifikation)
- `docs/crates/*.md` für alle Crates aus Abschnitt 2.1
- `CLAUDE.md` und `AGENTS.md` (Argon2id-Parameter, Saga-Schritte, Schlüsselhierarchie, Crate-Liste)
- Crate-READMEs (informativ)

Veraltete Angaben (6 Saga-Schritte, `SubjectKey`, `m = 256 MB`, `Poisoned`, Profile `Balanced`, `BareMetal`, Tiers) werden entfernt.

---

## 12. Offene Entscheidungen

| ID | Frage | Empfehlung |
|---|---|---|
| D-1 | Standard bei Mitbetroffenen (`DeleteAll` oder `Restricted`), und ob frühe Schlüsselvernichtung (ER-31) für eine Klasse zugelassen wird | `Restricted` als Standard, frühe Vernichtung nicht implementieren (P-3, P-4) |
| D-2 | Lokales Embedding-Backend: `candle` (GGUF) oder `ort` (ONNX) | Ein Backend zuerst, das andere als Feature. Auswahl nach Messung der Modellgrößen und Plattformen. |
| D-3 | HNSW im Standard | Nein. Flach und exakt bis 200.000 Vektoren, HNSW nur als Feature mit Rückrufmessung. |
| D-4 | Dokumentformate im Umfang | PDF, DOCX, Markdown, Klartext zuerst. HTML und XLSX danach. |
| D-5 | Checkpoint-Umfang | Snapshot-Export und Wiederherstellung. Guard und Orphan-Erkennung entfallen, sofern Phase 2 sie nicht zwingend braucht. |
| D-6 | KV-Segment-Eviction im Produktpfad | Entfernen, falls `cargo tree` und Suche keinen Produktaufrufer zeigen. |
| D-7 | Agentengedächtnis (`remember`, `recall`, `forget`) | Adapter außerhalb des Kerns. Entscheidung durch Auftraggeber. |
| D-8 | Abhängigkeit `arc-swap` für Snapshot-Tausch | Aufnehmen, sofern eine Alternative mit gleichen Eigenschaften nicht vorhanden ist. |
| D-9 | Produktziel: RAG mit Löschung als Kern, oder RAG ohne Löschung | Mit Löschung (Spec-Ziel). Die Kosten sind in Phase 2 begrenzt durch flache Suche und Segmentformate. |
| D-10 | Rechtliche Einordnung der Schlüsselvernichtung als Löschung (ER §15 Punkt 4) | Juristische Prüfung vor Außenkommunikation. Keine Rechtsberatung durch diese Spezifikation. |

---

## 13. Anhang: Befundbezug

Die Befunde aus den Analysen dieser Sitzung sind im Repository nachprüfbar:

- Fassade: 20 Kompilierfehler gegen Stand HEAD (Rust 1.89), `contextra-db` ohne die importierten Typen.
- Dependency-Verstöße: `contextra-ports → contextra-types`, `contextra-store → contextra-crypto` und `contextra-mvcc`, Dev-Dependencies auf `contextra-testkit` in `contextra-checkpoint` und `contextra-engine`.
- Defaults mit Features: `contextra-durable-fs`, `contextra-graph`, `contextra-rank`, `contextra-simd`, `contextra-text`, `contextra`, `contextra-testkit`.
- `MAX_SEARCH_K`: sechsfach definiert, Werte 1.000 und 10.000.
- Lint: `unsafe_code = "deny"` in `Cargo.toml`, Spec verlangt `forbid`.
- Fehlende Dateien: `benchmarks/budgets.toml`, `docs/Finale Löscharchitektur.md`, `crates/contextra-testkit/tests/layering.rs`.
- Chunker: referenziert im Produktpfad (Fassade, MCP), im Workspace fehlend, in der Git-Historie vorhanden.
- Parser und lokale Embedding-Backends: in der Historie vorhanden, aus dem Workspace entfernt.

**Reproduktion:**

```bash
export PATH=/opt/rust189/bin:$PATH
cargo check -p contextra            # Ergebnis am Stand HEAD: Fehler
git log --all -- '*chunker*'        # Fundstellen der Chunker-Implementierung
cargo tree -e normal | grep -E "reqwest|hyper|ureq|tonic"   # Erwartung: keine Treffer
```
