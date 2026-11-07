# AGENTS.md — `contextra-text`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-text` stellt eine modulare Inverted-Index Volltextsuchmaschine mit BM25-Scoring (normativ k1 = 1.2, b = 0.75), Block-Max-WAND und konfigurierbaren Text-Analyzern bereit (Ring 0).
Als eigenständiger Baustein implementiert er den standardisierten Port `TextIndex` aus `contextra-ports`.
Im Sinne der Entkopplung ist der Text-Index frei von rechtlichen Löschketten und Deletion-Proofs: Er arbeitet append-basiert mit Segment-Postings und exportiert neutrale `Purger`- und `LayerScanner`-Schnittstellen für `contextra-erasure`.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Volltextsuche & Inverted Index)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports`, `contextra-crypto` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **REQ-TXT-001:** Deterministische Inverted Index Postings, BM25/BM25F-Scoring mit Robertson-Spärck-Jones IDF.
- **Identitäts-Entkopplung (S1 / D-69):** Postings nutzen segment-lokale `doc_ord` ohne rohe `DocId`; Zuordnung erfolgt über verschlüsselte Envelopes.
- **REQ-TXT-002:** `LayerScanner`-Implementierung (Layer ID 20) für neutrale Speicher-Scans ohne DSGVO-Bindung.
- **Zero-Panic-Doktrin:** Vorab-Prüfung aller Analyzer-Eingaben; keine Panics bei ungültigen Token-Sequenzen oder leeren Dokumenten.

## 4. Verbotene Muster
- Kein `unsafe` (`#![forbid(unsafe_code)]` strikt erzwungen).
- Keine unkontrollierte Heap-Allokation bei großen Postings-Listen.
- Keine Multi-Level Deletion-Logik oder Rechtsbegriffe im Text-Kern.
- Keine direkten `std::fs`-Aufrufe im Index-Kern (vollständig über den `Vfs`-Port abstrahiert).

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** `TextIndex` (`InvertedIndex`), `Analyzer`, `Purger`, `LayerScanner`.
- **Konsumiert:** `Clock`-Port, `Vfs`-Port.

## 6. Tests, Sabotage & Qualitätssicherung
- **Fuzzing (REQ-QA-003):** BM25 Tokenizer, Analyzer und Posting-List Fuzzer.
- **LayerScan Verification:** Residuen- und Purge-Verifikation im Text-Witness Integrations-Test.
- **Mutations-Testing:** Prüfung der Scoring-Invarianten und WAND-Frühabbruchkriterien.

## 7. Offene Punkte & Technische Schulden
- DEBT-01: Vollständige Bereinigung der restlichen Dateien mit historischem Löschketten-Bezug.
