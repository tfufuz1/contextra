# Implementierungsspezifikation & Arbeitsplan: `contextra-ingest`

> **Ring:** 4 | **Klasse:** `R` (Rand / Extension) | **Unsafe:** Streng verboten (#![forbid(unsafe_code)])
> **Architekturrolle:** Autarkes Dokumenten-Parsing, Text-Extraktion und semantisches Chunking (Paket B24)
> **Spezifikationsbezug:** `docs/SPEC.md` §4, `PLAN.md`

---

## 1. Architektonische Leitprinzipien & Baukasten-Vertrag

Für `contextra-ingest` gelten im Rahmen der Transformation zum modularen Baukastensystem folgende verbindliche Richtlinien:

1. **Modulare Autarkie & Isolation:** Der Crate funktioniert als eigenständiger, isolierter Baustein. Direkte Kopplungen an Fremd-Crates werden über wohldefinierte Schnittstellen oder generische Abstraktionen gelöst. Keine Abhängigkeit zu `contextra-engine`.
2. **Entkopplung der Löschkette:** Sämtliche juristischen Begrifflichkeiten (DSGVO Art. 17), Nachweisketten, Deletion-Proofs und Saga-Orchestrierungen verbleiben exklusiv in `contextra-erasure`. Ingest besitzt keinen Lösch- oder DSGVO-Bezug.
3. **Design by Contract (DbC) & Invarianten:** Alle Vorbedingungen, Nachbedingungen und Systeminvarianten werden strikt geprüft (Fail-Closed).
4. **Zero Shared Mutable State:** Parser und Chunker sind synchron und zustandslos. Parallelität liegt beim Aufrufer.
5. **Code- & Unsafe-Disziplin:** `#![forbid(unsafe_code)]` ist im gesamten Crate verbindlich gesetzt. Keine unwrap(), expect() oder Panics im Produktivcode.

---

## 2. Modul- & Komponentenarchitektur

### 2.1 Interne Struktur & Verantwortung

| Komponente / Modul | Verantwortung & Schnittstellen |
|---|---|
| `chunking` | Autarkes semantisches, tokenbasiertes und hierarchisches Chunking (Prose, Paged, Tabular, Email, Structured) |
| `formats` | Extraktion strukturierter und unstrukturierter Formate (PDF, DOCX, XLSX, ODF, HTML, XML, E-Mail, JSON etc.) |
| `detect` | Sichere MIME- und Format-Erkennung mit Magie-Byte-Validierung |
| `canonical` | Sanitization und Erzeugung der standardisierten kanonischen Zwischenform |
| `safe` & `limits` | Nicht abschaltbare Schutzgrenzen gegen Zip-Bomben, Pufferüberläufe und Rekursion |
| `pipeline` & `registry` | Composition Root für Extraktoren und Ingestion-Pipelines |

### 2.2 Abhängigkeitsmatrix & Schnittstellen

- **Erlaubte interne Abhängigkeiten:** `contextra-types` (Ring 0)
- **Verbotene Abhängigkeiten:** Höhere Ringe sowie Querverbindungen außerhalb der deklarierten Ports.
- **Konsumierte / Implementierte Ports:** Reine zustandslose Ingestion-Engine ohne Persistenz-Ports.

---

## 3. Umgesetzte Arbeitspakete (Work Packages)

### WP-ING-1: Vollständige Entkopplung von der Engine (Paket B24)
- **Fokus / Betroffene Dateien:** `Cargo.toml, src/chunking/common.rs, src/chunking/fallback.rs, tests/chunk_prose_parity.rs, tests/contract_tests.rs`
- **Problemstellung & Baukasten-Ziel:** Vollständige Auflösung der Abhängigkeit zu contextra-engine; Ingest agiert rein als autarke Datenvorbereitungs-Pipeline.
- **Ergebnis:**
  - `contextra-engine` aus `Cargo.toml` entfernt.
  - Autarke `estimate_tokens` und `chunk_text_with_overlap` direkt in `chunking/common.rs`.
  - `FallbackChunker` delegiert sauber an `ProseChunker`.
  - Alle Unit- und Integrationstests sowie M2-Isolationsbeweis sind grün.

---

## 4. Test-, Benchmark- & Qualitätssicherungsplan

### 4.1 Unit-, Integration- & Conformance-Tests
- **Unit-Tests:** 100% Testabdeckung für Kernfunktionen und Randfall-Behandlungen.
- **Traceability:** Jeder Test annotiert die Spezifikationsanforderung (z. B. `/// SPEC: REQ-...`).

### 4.2 Property-Based Testing & Fuzzing
- Testen aller Invarianten unter zufälligen Eingaben (proptest).
- Fuzzing aller Parser-, Deserialisierungs- und Eingabepfade unter `fuzz/`.

---

## 5. Definition of Done (DoD) für diesen Baustein

1. Alle deklarierten Arbeitspakete sind vollständig implementiert.
2. `cargo check -p contextra-ingest`, `cargo test -p contextra-ingest` und `cargo clippy -p contextra-ingest` laufen fehlerfrei durch.
3. Keine unzulässigen Querabhängigkeiten oder Lecks von Implementierungsdetails.
4. Die Crate-README dokumentiert den autarken Einsatz in Drittprojekten mit einem Minimalbeispiel unter `examples/extract_and_chunk.rs`.
