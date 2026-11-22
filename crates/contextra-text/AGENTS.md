# AGENTS.md — `contextra-text`

## 1. Rolle
`contextra-text` stellt eine Inverted-Index Volltextsuchmaschine mit BM25 Scoring, Block-Max-WAND und Text-Analyzer-Schnittstellen bereit.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Volltextsuche)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`)

## 3. Garantien & Anforderungen (§33, §10)
- **REQ-TXT-001:** Deterministische Inverted Index Postings, BM25/BM25F Scoring.
- **REQ-TXT-002:** `LayerScan` Implementierung (Layer ID 20) für Residuen-Spannungs-Scans.
- **Durchsetzungsmechanismus (§27.6):** `#![forbid(unsafe_code)]`, Tokenizer Unit- & Integrationstests.

## 4. Verbotene Muster
- Kein `unsafe` (`#![forbid(unsafe_code)]`).
- Keine unkontrollierte Speicherallokation bei großen Postings-Listen.

## 5. Implementierte & konsumierte Ports (Anhang A)
- Implementiert: `TextIndex` (`InvertedIndex`), `Analyzer` (Standard-Analyzer).
- Konsumiert: `Clock` Port.

## 6. Tests & Gates
- **Fuzzing (REQ-QA-003):** BM25 Tokenizer & Posting Fuzzer.
- **LayerScan Verification:** Residuen-Scans im Text-Witness Integrations-Test.

## 7. Offene Punkte
- keine.

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: Indexstruktur; kein direkter Bezug zu `contextra-store`.
- Vertrag IF-6: später `ExtensionBuilder`/Segment (Spec §15.9); Pflicht `rewrite_without`.
- Verboten: der Kern kennt keine Volltextindizes.
- Arbeitspakete: nach G-6.
