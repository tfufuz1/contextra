# AGENTS.md — `contextra-ingest`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-ingest` stellt isolierte Dokumentformat-Extraktoren (PDF, Office/DOCX/PPTX, Spreadsheets/XLSX/CSV/TSV, HTML, Markdown) und strukturbewusstes Chunking außerhalb des Datenbank-Kerns bereit (Ring 4, SPEC §C-10.3).
Als autonomer Adapter-Baustein transformiert er unstrukturierte Rohdaten deterministisch in strukturierte Text- und Tabellenblöcke, ohne Seiteneffekte auf den Speicherkern auszuüben.
Er erzwingt strenge Resource-Budgets, Zip-Bomb-Prävention und Fail-Closed Panik-Isolierung.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 4 (Dokumenten-Extraktion & Chunking-Adapter)
- **Klasse:** Adapter `[A]`
- **Reifegrad:** `experimental`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-engine` (Spec §C-10.3, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Defensive Eingabeverarbeitung & DoS-Schutz:** Strenger Schutz gegen Zip-Bomben und sparse Spreadsheets (z.B. A1 + XFD1048576) über `ResourceBudget` und `IngestLimits`.
- **Fail-Closed Panik-Isolierung:** `std::panic::catch_unwind` an der Adaptergrenze verhindert, dass Third-Party-Parser-Panics den Host-Prozess zum Absturz bringen.
- **Reinheit & Determinismus (P28):** Keine Systemzeit, kein Zufall, keine Netzwerkanfragen, keine Makroausführung in Dokumenten.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()`, `expect()` oder unkontrolliertem Indexing im Produktionscode.

## 4. Verbotene Muster
- Direkte I/O- oder Netzwerkaufrufe während der Dokumenten-Extraktion.
- Verwendung von `contextra-core` (statdessen direkte Nutzung von `contextra-types`).
- Extrahieren versteckter Inhalte ohne explizite Freigabe in der Ingest-Policy.
- Ausführen von Makros oder Skripten in Dokumenten-Containern.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** `DocumentExtractor`, `IrChunker`.
- **Konsumiert:** `contextra-types`.
- **Bereitgestellt für:** `contextra-db`, `contextra-engine`.

## 6. Tests, Sabotage & Qualitätssicherung
- **Corpus Fuzzing (fuzz/):** Fuzzing-Matrix gegen beschädigte PDFs, manipulierte ZIP-Header und XML-Entity-Expansionen.
- **Budget Exhaustion Tests:** Sabotagetests mit künstlichen Zip-Bomben zur Prüfung des sofortigen Abbruchs (`LimitExceeded`).
- **Contract Tests:** Schnittstellenvalidierung in `tests/contract_*.rs`.

## 7. Offene Punkte & Technische Schulden
- Trennung des Chunking-Kerns in einen autarken `contextra-chunker` zur vollständigen Entkopplung von `contextra-engine`.
