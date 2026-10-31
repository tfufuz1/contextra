---
name: contextra-product-suite
description: >-
  Use this skill to guide the packaging, testing, and release validation of Contextra's sub-products:
  EP1 Memory, EP6 Guard (Privacy Gateway), EP2 Proof, and EP7 Ingest.
---

# Contextra Product Suite – Modul- & Produktvalidierung

Dieser Skill unterstützt Entwickler und Agenten bei der Entkopplung, Validierung und Paketierung der disjunkten Contextra-Produktfamilie nach [PLAN.md §11](file:///home/freddy/Projekte/contextra/PLAN.md#L526) und [docs/SPEC.md §4](file:///home/freddy/Projekte/contextra/docs/SPEC.md#L83).

---

## 1. Die vier Zielprodukte

| Produkt | Zweck & Wertversprechen | Beteiligte Crates | Release-Reihenfolge |
|---|---|---|---|
| **EP6 Guard** | Lokales Privacy-Gateway: Egress-Filterung, PII-Erkennung mit Prüfziffernvalidierung (IBAN, Luhn, St-ID), reversible Tokenisierung. | `contextra-privacy`, `contextra-sandbox` | **1. Produkt (schnellstes Go-to-Market)** |
| **EP1 Memory** | Eingebettete Hybrid-Suche (HNSW + BM25 mit deutscher Morphologie + RRF) mit gemessener DSGVO-Löschung (Ebene A & B). | `store`, `vector`, `text`, `engine`, `db`, `durable-fs` | **2. Produkt (Kern-Alleinstellungsmerkmal)** |
| **EP2 Proof** | Asymmetrisch signierte Lösch- und Revisionsberichte (Ed25519 Proof V5) und AVV-Generator (Art. 28 DSGVO). | `contextra-audit-export`, `contextra-avv-generator` | Add-on zu Memory & Guard |
| **EP7 Ingest** | Format-Parser für unvertrauenswürdige Dokumente (PDF, DOCX, XLSX, HTML, EPUB) in isoliertem Subprozess. | `contextra-ingest` | Nach Subprozess-Isolation & Fuzzing |

---

## 2. Release-Gates (G1 bis G4)

Ein Modul oder Produkt darf nur freigegeben werden, wenn alle vier Gates bestanden sind:

- **G1 (Korrektheit)**: Besteht Konformitäts- und Differentialtests gegen ein unabhängiges Orakel ohne Hänger oder Timeouts.
- **G2 (Haltbarkeit)**: Übersteht Kill- und Crash-Durability-Tests (`SIGKILL` an beliebiger Stelle) ohne Datenverlust und ohne Klartextreste.
- **G3 (Leistung)**: Mindestens das 1,5-fache der etablierten Referenz (z. B. HNSW gegen Exakt-Scan, BM25-Top-10 Recall).
- **G4 (Wartbarkeit)**: Schmaler Port, Zero-Panic (`deny(clippy::unwrap_used)`), `unsafe` nur in benannter Insel mit `// SAFETY:`.

---

## 3. Sub-Workspace Build- und Test-Kommandos

```bash
# EP6 Guard validieren
cargo test -p contextra-privacy

# EP1 Memory validieren
cargo test -p contextra-vector -p contextra-text -p contextra-rank -p contextra-store

# EP2 Proof validieren
cargo test -p contextra-crypto --test deletion_proof_tests
cargo test -p contextra-audit-export
cargo test -p contextra-avv-generator

# EP7 Ingest validieren
cargo test -p contextra-ingest --no-default-features
```

Weitere Details: siehe [references/product_specs.md](file:///home/freddy/Projekte/contextra/.agents/skills/contextra-product-suite/references/product_specs.md).
