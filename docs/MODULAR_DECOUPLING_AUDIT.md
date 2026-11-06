# Contextra Modularisierungs- & Entkopplungs-Audit

> **Datum:** 2026-11-06 16:00:58 | **Analysierte Crates:** 35 | **Gesamtumfang:** 181,416 LOC

## 1. Strategische Zielsetzung & Architekturvision
Dieses Dokument dient als zentrale Orientierung für Architekten und Coding-Agenten zur Transformation des Contextra-Ökosystems von einem fest verdrahteten Monolithen zu einem **vollständig modularen Baukastensystem (Building Blocks)**.

### Kernprinzipien des Baukastensystems:
1. **Vollständige Entkopplung:** Jeder Crate muss isoliert verständlich, testbar und in Fremdprojekten nutzbar sein.
2. **Schnittstellen via Ports/Traits:** Direkte Abhängigkeiten zwischen konkreten Crates werden aufgelöst; Abstraktionen liegen in `contextra-ports` oder generischen Parametern.
3. **Isolation der Löschkette:** Alle DSGVO-, Gesetzes- und multi-layer Deletion-Proof-Logiken werden vollständig in `contextra-erasure` isoliert und aus Kernkomponenten (Store, Vector, Text, Crypto, DB) entfernt.
4. **Zero-Dead-Code & Saubere Fassaden:** Öffentliche Schnittstellen werden minimal gehalten und lecken keine internen Fremdtypen.

## 2. Gesamtübersicht des Baukastensystems

- **Durchschnittlicher Modularitäts-Score:** 60.4 / 100
- **Vollkommen autarke Bausteine:** 4 Crates
- **Dringender Entkopplungsbedarf (Score < 60):** 17 Crates
- **Verstreute Löschketten-/DSGVO-Relikte außerhalb `contextra-erasure`:** 664 Fundstellen

| Crate | Modularitäts-Score | Status | LOC | Dateien | Interne Deps | DSGVO/Lösch-Relikte |
|---|---:|---|---:|---:|---:|---:|
| `contextra-engine` | **10/100** | Stark verdrahtet (Refactoring nötig) | 17,573 | 82 | 14 | 57 |
| `contextra-db` | **15/100** | Stark verdrahtet (Refactoring nötig) | 1,713 | 4 | 13 | 14 |
| `contextra` | **15/100** | Stark verdrahtet (Refactoring nötig) | 1,084 | 6 | 11 | 24 |
| `contextra-vector` | **19/100** | Stark verdrahtet (Refactoring nötig) | 11,720 | 43 | 6 | 12 |
| `contextra-mcp` | **20/100** | Stark verdrahtet (Refactoring nötig) | 5,276 | 24 | 12 | 48 |
| `contextra-store` | **35/100** | Stark verdrahtet (Refactoring nötig) | 23,040 | 77 | 5 | 28 |
| `contextra-agent` | **40/100** | Stark verdrahtet (Refactoring nötig) | 4,220 | 18 | 7 | 2 |
| `contextra-cognition` | **44/100** | Stark verdrahtet (Refactoring nötig) | 5,244 | 19 | 6 | 0 |
| `contextra-graph` | **47/100** | Stark verdrahtet (Refactoring nötig) | 16,585 | 71 | 3 | 5 |
| `contextra-kvcache` | **47/100** | Stark verdrahtet (Refactoring nötig) | 4,174 | 11 | 3 | 5 |
| `contextra-privacy` | **52/100** | Stark verdrahtet (Refactoring nötig) | 3,703 | 12 | 3 | 44 |
| `contextra-text` | **53/100** | Stark verdrahtet (Refactoring nötig) | 5,828 | 26 | 2 | 29 |
| `contextra-router` | **55/100** | Stark verdrahtet (Refactoring nötig) | 5,601 | 28 | 5 | 0 |
| `contextra-crypto` | **58/100** | Stark verdrahtet (Refactoring nötig) | 5,725 | 23 | 2 | 106 |
| `contextra-mvcc` | **58/100** | Stark verdrahtet (Refactoring nötig) | 2,767 | 5 | 2 | 7 |
| `contextra-audit-export` | **58/100** | Stark verdrahtet (Refactoring nötig) | 557 | 5 | 2 | 31 |
| `contextra-ports` | **59/100** | Stark verdrahtet (Refactoring nötig) | 3,333 | 27 | 1 | 9 |
| `contextra-py` | **64/100** | Akzeptabel | 2,246 | 16 | 6 | 0 |
| `contextra-durable-fs` | **64/100** | Akzeptabel | 860 | 6 | 1 | 7 |
| `contextra-avv-generator` | **64/100** | Akzeptabel | 508 | 9 | 1 | 226 |
| `contextra-core` | **66/100** | Akzeptabel | 106 | 1 | 4 | 0 |
| `contextra-erasure` | **67/100** | Akzeptabel | 7,307 | 57 | 3 | 0 |
| `contextra-testkit` | **68/100** | Akzeptabel | 2,318 | 19 | 2 | 3 |
| `contextra-infer-candle` | **71/100** | Akzeptabel | 4,818 | 17 | 4 | 0 |
| `contextra-infer-ollama` | **77/100** | Akzeptabel | 4,944 | 16 | 3 | 0 |
| `contextra-checkpoint` | **77/100** | Akzeptabel | 3,475 | 11 | 3 | 1 |
| `contextra-types` | **80/100** | Exzellent (Baustein) | 5,068 | 22 | 0 | 4 |
| `contextra-infer-onnx` | **82/100** | Exzellent (Baustein) | 1,905 | 6 | 3 | 0 |
| `contextra-ingest` | **83/100** | Exzellent (Baustein) | 16,257 | 77 | 2 | 1 |
| `contextra-adapt` | **88/100** | Exzellent (Baustein) | 3,578 | 13 | 2 | 0 |
| `contextra-rank` | **88/100** | Exzellent (Baustein) | 3,233 | 20 | 2 | 0 |
| `contextra-simd` | **94/100** | Exzellent (Baustein) | 1,653 | 7 | 1 | 0 |
| `contextra-wire` | **95/100** | Exzellent (Baustein) | 2,038 | 6 | 0 | 1 |
| `contextra-sandbox` | **100/100** | Exzellent (Baustein) | 1,892 | 13 | 0 | 0 |
| `contextra-sys` | **100/100** | Exzellent (Baustein) | 1,067 | 7 | 0 | 0 |

## 3. Priorisierte Entkopplungs-Roadmap für Coding-Agenten

### 📦 Crate: `contextra` (Score: 15/100)
- **Umfang:** 1,084 LOC (6 Quelldateien)
- **Abhängigkeiten (11):** `contextra-audit-export, contextra-avv-generator, contextra-crypto, contextra-db, contextra-infer-candle, contextra-license, contextra-ports, contextra-privacy, contextra-rank, contextra-store, contextra-types`
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-db`: 14 Symbole
  - `contextra-ports`: 9 Symbole
  - `contextra-types`: 10 Symbole
  - `contextra-store`: 7 Symbole
  - `contextra-rank`: 2 Symbole
  - `contextra-infer-candle`: 2 Symbole
  - `contextra-license`: 1 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 24 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-db' (14 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (9 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (10 importierte Symbole).
  - 24 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-db' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-agent` (Score: 40/100)
- **Umfang:** 4,220 LOC (18 Quelldateien)
- **Abhängigkeiten (7):** `contextra-checkpoint, contextra-db, contextra-graph, contextra-ports, contextra-router, contextra-store, contextra-types`
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ports`: 4 Symbole
  - `contextra-types`: 15 Symbole
  - `contextra-checkpoint`: 4 Symbole
  - `contextra-store`: 1 Symbole
  - `contextra-db`: 10 Symbole
  - `contextra-graph`: 1 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 2 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-types' (15 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-db' (10 importierte Symbole).
  - 2 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-db' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-agent extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-audit-export` (Score: 58/100)
- **Umfang:** 557 LOC (5 Quelldateien)
- **Abhängigkeiten (2):** `contextra-ports, contextra-types`
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 1 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 31 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - 31 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] DSGVO-/Löschlogik aus contextra-audit-export extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-cognition` (Score: 44/100)
- **Umfang:** 5,244 LOC (19 Quelldateien)
- **Abhängigkeiten (6):** `contextra-engine, contextra-graph, contextra-ports, contextra-store, contextra-types, contextra-vector`
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-engine`: 10 Symbole
  - `contextra-ports`: 16 Symbole
  - `contextra-types`: 9 Symbole
  - `contextra-graph`: 14 Symbole
  - `contextra-store`: 2 Symbole
  - `contextra-vector`: 2 Symbole
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-engine' (10 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (16 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (9 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-graph' (14 importierte Symbole).
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-engine' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-graph' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.

### 📦 Crate: `contextra-crypto` (Score: 58/100)
- **Umfang:** 5,725 LOC (23 Quelldateien)
- **Abhängigkeiten (2):** `contextra-ports, contextra-types`
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 8 Symbole
  - `contextra-ports`: 7 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 106 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - 106 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] DSGVO-/Löschlogik aus contextra-crypto extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-db` (Score: 15/100)
- **Umfang:** 1,713 LOC (4 Quelldateien)
- **Abhängigkeiten (13):** `contextra-adapt, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-engine, contextra-graph, contextra-ports, contextra-rank, contextra-store, contextra-sys, contextra-text, contextra-types, contextra-vector`
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-adapt`: 6 Symbole
  - `contextra-sys`: 1 Symbole
  - `contextra-types`: 8 Symbole
  - `contextra-ports`: 9 Symbole
  - `contextra-graph`: 3 Symbole
  - `contextra-store`: 2 Symbole
  - `contextra-vector`: 2 Symbole
  - `contextra-engine`: 37 Symbole
  - `contextra-cognition`: 38 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 14 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-ports' (9 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-engine' (37 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-cognition' (38 importierte Symbole).
  - 14 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-engine' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-cognition' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-db extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-engine` (Score: 10/100)
- **Umfang:** 17,573 LOC (82 Quelldateien)
- **Abhängigkeiten (14):** `contextra-adapt, contextra-checkpoint, contextra-crypto, contextra-graph, contextra-infer-onnx, contextra-mvcc, contextra-ports, contextra-rank, contextra-sandbox, contextra-store, contextra-sys, contextra-text, contextra-types, contextra-vector`
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-rank`: 19 Symbole
  - `contextra-types`: 27 Symbole
  - `contextra-ports`: 27 Symbole
  - `contextra-sandbox`: 3 Symbole
  - `contextra-store`: 5 Symbole
  - `contextra-crypto`: 6 Symbole
  - `contextra-vector`: 4 Symbole
  - `contextra-text`: 3 Symbole
  - `contextra-graph`: 10 Symbole
  - `contextra-mvcc`: 3 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 57 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-rank' (19 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (27 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (27 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-graph' (10 importierte Symbole).
  - 57 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-rank' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-graph' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-engine extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-graph` (Score: 47/100)
- **Umfang:** 16,585 LOC (71 Quelldateien)
- **Abhängigkeiten (3):** `contextra-adapt, contextra-ports, contextra-types`
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 11 Symbole
  - `contextra-ports`: 11 Symbole
  - `contextra-adapt`: 2 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 5 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-types' (11 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (11 importierte Symbole).
  - 5 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-graph extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-kvcache` (Score: 47/100)
- **Umfang:** 4,174 LOC (11 Quelldateien)
- **Abhängigkeiten (3):** `contextra-crypto, contextra-ports, contextra-types`
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 11 Symbole
  - `contextra-ports`: 26 Symbole
  - `contextra-crypto`: 5 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 5 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-types' (11 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (26 importierte Symbole).
  - 5 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-kvcache extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-mcp` (Score: 20/100)
- **Umfang:** 5,276 LOC (24 Quelldateien)
- **Abhängigkeiten (12):** `contextra, contextra-crypto, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-license, contextra-ports, contextra-privacy, contextra-rank, contextra-router, contextra-types, contextra-wire`
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-privacy`: 8 Symbole
  - `contextra`: 43 Symbole
  - `contextra-wire`: 3 Symbole
  - `contextra-types`: 9 Symbole
  - `contextra-ports`: 5 Symbole
  - `contextra-rank`: 2 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 48 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra' (43 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (9 importierte Symbole).
  - 48 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-mcp extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-mvcc` (Score: 58/100)
- **Umfang:** 2,767 LOC (5 Quelldateien)
- **Abhängigkeiten (2):** `contextra-ports, contextra-types`
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ports`: 2 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 7 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - 7 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] DSGVO-/Löschlogik aus contextra-mvcc extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-ports` (Score: 59/100)
- **Umfang:** 3,333 LOC (27 Quelldateien)
- **Abhängigkeiten (1):** `contextra-types`
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 19 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 9 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-types' (19 importierte Symbole).
  - 9 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-ports extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-privacy` (Score: 52/100)
- **Umfang:** 3,703 LOC (12 Quelldateien)
- **Abhängigkeiten (3):** `contextra-avv-generator, contextra-ports, contextra-types`
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ports`: 2 Symbole
  - `contextra-avv-generator`: 1 Symbole
  - `contextra-types`: 2 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 44 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - 44 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] DSGVO-/Löschlogik aus contextra-privacy extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-router` (Score: 55/100)
- **Umfang:** 5,601 LOC (28 Quelldateien)
- **Abhängigkeiten (5):** `contextra-adapt, contextra-ports, contextra-privacy, contextra-types, contextra-wire`
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-adapt`: 17 Symbole
  - `contextra-types`: 10 Symbole
  - `contextra-wire`: 2 Symbole
  - `contextra-privacy`: 4 Symbole
  - `contextra-ports`: 17 Symbole
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-adapt' (17 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (10 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (17 importierte Symbole).
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-adapt' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.

### 📦 Crate: `contextra-store` (Score: 35/100)
- **Umfang:** 23,040 LOC (77 Quelldateien)
- **Abhängigkeiten (5):** `contextra-crypto, contextra-mvcc, contextra-ports, contextra-sys, contextra-types`
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 8 Symbole
  - `contextra-mvcc`: 5 Symbole
  - `contextra-ports`: 17 Symbole
  - `contextra-crypto`: 8 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 28 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-ports' (17 importierte Symbole).
  - 28 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-store extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-text` (Score: 53/100)
- **Umfang:** 5,828 LOC (26 Quelldateien)
- **Abhängigkeiten (2):** `contextra-ports, contextra-types`
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ports`: 12 Symbole
  - `contextra-types`: 8 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 29 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-ports' (12 importierte Symbole).
  - 29 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-text extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-vector` (Score: 19/100)
- **Umfang:** 11,720 LOC (43 Quelldateien)
- **Abhängigkeiten (6):** `contextra-crypto, contextra-mvcc, contextra-ports, contextra-simd, contextra-sys, contextra-types`
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 9 Symbole
  - `contextra-simd`: 18 Symbole
  - `contextra-ports`: 12 Symbole
  - `contextra-mvcc`: 2 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 12 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-types' (9 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-simd' (18 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (12 importierte Symbole).
  - 12 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-simd' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-vector extrahieren und in 'contextra-erasure' modularisieren.

## 4. Löschketten- und DSGVO-Fundstelleninventar

Folgende Stellen müssen im Rahmen der Entkopplung aus den jeweiligen Crates bereinigt werden:

#### `contextra` (24 Stellen)
- `crates/contextra/src/collection_profile.rs:13` — Suchbegriff `dsgvo`: `//!   regulierten Unternehmensumgebungen (DSGVO, Compliance).`
- `crates/contextra/src/collection_profile.rs:24` — Suchbegriff `deletionproof`: `//! - **INV-COLLECTION-PROFILE-1**: Wenn kryptografische Löschnachweise (DeletionProof) aktiv sind,`
- `crates/contextra/src/collection_profile.rs:45` — Suchbegriff `dsgvo`: `/// gegeben, solange der Betreiber es nicht ausdrücklich aktiviert (Datenminimierung gemäß Art. 5 DS`
- `crates/contextra/src/collection_profile.rs:80` — Suchbegriff `deletionproof`: `#[error("deletion-proof active but KvDeleteMode::TombstoneOnly cannot satisfy a KV-side DeletionProo`
- `crates/contextra/src/collection_profile.rs:90` — Suchbegriff `deletion_proof`: `/// 2. Prüfen, ob `deletion_proof_active` mit `KvDeleteMode::TombstoneOnly` kombiniert wurde (Invari`
- `crates/contextra/src/collection_profile.rs:97` — Suchbegriff `deletion_proof`: `if resolved.deletion_proof_active && self.kv_delete_mode == KvDeleteMode::TombstoneOnly {`
- `crates/contextra/src/builder.rs:372` — Suchbegriff `deletion_proof`: `|| config_for_open.deletion_proof_active != resolved.deletion_proof_active`
- `crates/contextra/src/builder.rs:382` — Suchbegriff `deletion_proof`: `config_for_open.deletion_proof_active = resolved.deletion_proof_active;`
- `crates/contextra/src/performance_profile.rs:24` — Suchbegriff `löschung`: `//!   mit aktiven Löschnachweisen verbietet asynchrone Vektor-Löschung (`VectorDeleteMode::Backgroun`
- `crates/contextra/src/performance_profile.rs:44` — Suchbegriff `dsgvo`: `/// Standardmäßig wird das `Compliance`-Profil gewählt (Security-by-Default / DSGVO-Konformität).`
- `crates/contextra/src/performance_profile.rs:56` — Suchbegriff `deletion_proof`: `pub deletion_proof_active: bool,`
- `crates/contextra/src/performance_profile.rs:77` — Suchbegriff `deletionproof`: `/// - `Compliance`: Full Durability, SynchronousRepair, DeletionProof aktiv, Sovereign-Ring.`
- `crates/contextra/src/performance_profile.rs:78` — Suchbegriff `deletionproof`: `/// - `Balanced`: WalNoHmac, BackgroundRepair, DeletionProof inaktiv, Fast-Ring.`
- `crates/contextra/src/performance_profile.rs:79` — Suchbegriff `deletionproof`: `/// - `BareMetal`: MemoryOnly, BackgroundRepair, DeletionProof inaktiv, Fast-Ring.`
- `crates/contextra/src/performance_profile.rs:85` — Suchbegriff `deletion_proof`: `deletion_proof_active: true,`

#### `contextra-agent` (2 Stellen)
- `crates/contextra-agent/src/clm_scratchpad.rs:21` — Suchbegriff `purger`: `PurgeReceipt, SignalModality, VaultChunk, VaultConfig, VolatileContextVault,`
- `crates/contextra-agent/src/clm_scratchpad.rs:187` — Suchbegriff `purger`: `pub purge_receipt: PurgeReceipt,`

#### `contextra-audit-export` (31 Stellen)
- `crates/contextra-audit-export/src/testkit.rs:4` — Suchbegriff `deletionproof`: `AuditExportError, ControllerContact, DeletionProofSummary, EgressEventSummary,`
- `crates/contextra-audit-export/src/testkit.rs:48` — Suchbegriff `löschung`: `retention_period: "30 Tage nach Abwicklung, danach automatische Löschung"`
- `crates/contextra-audit-export/src/testkit.rs:56` — Suchbegriff `dsgvo`: `legal_basis: "Art. 6 Abs. 1 lit. b DSGVO".to_string(),`
- `crates/contextra-audit-export/src/testkit.rs:63` — Suchbegriff `deletion_proof`: `deletion_proofs: vec![DeletionProofSummary {`
- `crates/contextra-audit-export/src/testkit.rs:83` — Suchbegriff `löschung`: `retention_period: "Löschung bei Mandantenkündigung".to_string(),`
- `crates/contextra-audit-export/src/testkit.rs:89` — Suchbegriff `dsgvo`: `legal_basis: "Art. 6 Abs. 1 lit. f DSGVO".to_string(),`
- `crates/contextra-audit-export/src/testkit.rs:91` — Suchbegriff `deletion_proof`: `deletion_proofs: vec![DeletionProofSummary {`
- `crates/contextra-audit-export/src/bsi_mapping.rs:5` — Suchbegriff `gdpr`: `//! cryptographic mapping table included in the generated GDPR Art. 30 processing register export.`
- `crates/contextra-audit-export/src/bsi_mapping.rs:25` — Suchbegriff `deletion_proof`: `/// Code location path within the workspace (e.g. "crates/contextra-crypto/src/deletion_proof.rs").`
- `crates/contextra-audit-export/src/bsi_mapping.rs:41` — Suchbegriff `deletion_proof`: `code_location: "crates/contextra-crypto/src/deletion_proof.rs".to_string(),`
- `crates/contextra-audit-export/src/bsi_mapping.rs:43` — Suchbegriff `deletionproof`: `note: "Digitale Signatur für Löschnachweise (DeletionProof v4). BSI TR-02102-1 §1.5/§3.2 bewertet Ed`
- `crates/contextra-audit-export/src/error.rs:12` — Suchbegriff `gdpr`: `/// Missing mandatory field required by GDPR Article 30.`
- `crates/contextra-audit-export/src/lib.rs:1` — Suchbegriff `gdpr`: `//! `contextra-audit-export` — GDPR Article 30 Processing Register export generator.`
- `crates/contextra-audit-export/src/lib.rs:5` — Suchbegriff `dsgvo`: `//! (Verzeichnis von Verarbeitungstätigkeiten nach Art. 30 DSGVO) based on egress gateway logs`
- `crates/contextra-audit-export/src/lib.rs:23` — Suchbegriff `dsgvo`: `/// Contact details for the data controller, representative, and data protection officer (Art. 30 Ab`

#### `contextra-avv-generator` (226 Stellen)
- `crates/contextra-avv-generator/src/assistance.rs:1` — Suchbegriff `dsgvo`: `//! Unterstützung bei Art. 32–36 DSGVO gemäß Art. 28 Abs. 3 lit. f DSGVO.`
- `crates/contextra-avv-generator/src/assistance.rs:5` — Suchbegriff `dsgvo`: `//! sicherheits- und compliancebezogenen Anforderungen der DSGVO.`
- `crates/contextra-avv-generator/src/assistance.rs:7` — Suchbegriff `dsgvo`: `//! Gesetzlicher Kernpunkt nach Art. 28 Abs. 3 lit. f DSGVO:`
- `crates/contextra-avv-generator/src/assistance.rs:9` — Suchbegriff `dsgvo`: `//! - **Art. 32 DSGVO:** Technische und organisatorische Maßnahmen zur Sicherheit der Verarbeitung.`
- `crates/contextra-avv-generator/src/assistance.rs:10` — Suchbegriff `dsgvo`: `//! - **Art. 33 & 34 DSGVO:** Meldung von Verletzungen des Schutzes personenbezogener Daten (Data Br`
- `crates/contextra-avv-generator/src/assistance.rs:12` — Suchbegriff `dsgvo`: `//! - **Art. 35 & 36 DSGVO:** Durchführung von Datenschutz-Folgenabschätzungen (DSFA) sowie`
- `crates/contextra-avv-generator/src/assistance.rs:18` — Suchbegriff `avv`: `use crate::error::AvvError;`
- `crates/contextra-avv-generator/src/assistance.rs:21` — Suchbegriff `avv`: `// ANCHOR[contextra-avv-generator:ASST-001]`
- `crates/contextra-avv-generator/src/assistance.rs:22` — Suchbegriff `dsgvo`: `/// Rendert Abschnitt 10: „Unterstützung bei Einhaltung der Pflichten gemäß Art. 32–36 DSGVO“.`
- `crates/contextra-avv-generator/src/assistance.rs:30` — Suchbegriff `dsgvo`: `/// 3. Unterpunkt Art. 32 DSGVO: Gewährleistung der Sicherheit durch TOMs.`
- `crates/contextra-avv-generator/src/assistance.rs:31` — Suchbegriff `dsgvo`: `/// 4. Unterpunkt Art. 33 & 34 DSGVO: Unverzügliche Meldung von Datenschutzvorfällen.`
- `crates/contextra-avv-generator/src/assistance.rs:32` — Suchbegriff `dsgvo`: `/// 5. Unterpunkt Art. 35 & 36 DSGVO: Mitwirkung bei Datenschutz-Folgenabschätzungen.`
- `crates/contextra-avv-generator/src/assistance.rs:35` — Suchbegriff `avv`: `/// Gibt [`AvvError::RenderError`] zurück, falls Schreiboperationen in den Puffer fehlschlagen.`
- `crates/contextra-avv-generator/src/assistance.rs:36` — Suchbegriff `avv`: `pub fn render_section(out: &mut String) -> Result<(), AvvError> {`
- `crates/contextra-avv-generator/src/assistance.rs:39` — Suchbegriff `dsgvo`: `"\n## 10. Unterstützung bei Einhaltung der Pflichten gemäß Art. 32–36 DSGVO"`

#### `contextra-checkpoint` (1 Stellen)
- `crates/contextra-checkpoint/src/hardlink_cloner.rs:162` — Suchbegriff `löschung`: `// 1. Snapshot in der `SnapshotRegistry` pinnen (_guard verhindert Compactor-Löschung).`

#### `contextra-crypto` (106 Stellen)
- `crates/contextra-crypto/src/anti_tamper.rs:1` — Suchbegriff `löschung`: `//! # Manipulationsschutz und Notfall-Löschung (`anti_tamper.rs`)`
- `crates/contextra-crypto/src/anti_tamper.rs:8` — Suchbegriff `löschung`: `//! - Manuelle Notfalllöschung (`emergency_wipe`) überschreibt Speicher deterministisch mit Nullen.`
- `crates/contextra-crypto/src/crypto.rs:338` — Suchbegriff `deletion_proof`: `pub fn derive_deletion_proof_key(&self) -> Result<[u8; 32]> {`
- `crates/contextra-crypto/src/crypto.rs:513` — Suchbegriff `deletion_proof`: `fn test_derive_deletion_proof_key() {`
- `crates/contextra-crypto/src/crypto.rs:515` — Suchbegriff `deletion_proof`: `let deletion_key = km.derive_deletion_proof_key().unwrap();`
- `crates/contextra-crypto/src/ed25519_proof.rs:12` — Suchbegriff `deletion_proof`: `// HOTSPOTS: [sign_deletion_proof_v3, verify_deletion_proof_v3]`
- `crates/contextra-crypto/src/ed25519_proof.rs:25` — Suchbegriff `deletion_proof`: `pub use crate::deletion_proof::hash_deleted_keys_length_prefixed;`
- `crates/contextra-crypto/src/ed25519_proof.rs:29` — Suchbegriff `deletionproof`: `pub enum DeletionProofError {`
- `crates/contextra-crypto/src/ed25519_proof.rs:39` — Suchbegriff `deletionproof`: `#[error("Unsupported DeletionProof signature_version: {0}")]`
- `crates/contextra-crypto/src/ed25519_proof.rs:47` — Suchbegriff `deletionproof`: `pub struct DeletionProofKeyPair {`
- `crates/contextra-crypto/src/ed25519_proof.rs:55` — Suchbegriff `deletionproof`: `impl DeletionProofKeyPair {`
- `crates/contextra-crypto/src/ed25519_proof.rs:74` — Suchbegriff `deletionproof`: `/// Erzeugt ein `DeletionProofKeyPair` aus einem bestehenden Signierschlüssel.`
- `crates/contextra-crypto/src/ed25519_proof.rs:102` — Suchbegriff `deletion_proof`: `pub fn sign_deletion_proof_v3(keypair: &DeletionProofKeyPair, proof_payload: &[u8]) -> [u8; 64] {`
- `crates/contextra-crypto/src/ed25519_proof.rs:115` — Suchbegriff `deletionproof`: `/// - [`DeletionProofError::InvalidVerifyingKey`]: Wenn der Verifikationsschlüssel ungültig ist.`
- `crates/contextra-crypto/src/ed25519_proof.rs:116` — Suchbegriff `deletionproof`: `/// - [`DeletionProofError::InvalidSignature`]: Wenn die Signatur nicht zum Payload oder Schlüssel p`

#### `contextra-db` (14 Stellen)
- `crates/contextra-db/src/volatile_vault.rs:6` — Suchbegriff `dsgvo`: `//! einen flüchtigen Zwischenspeicher für hochgradig sensitive Kontextdaten (DSGVO/Privacy).`
- `crates/contextra-db/src/volatile_vault.rs:124` — Suchbegriff `purger`: `pub struct PurgeReceipt {`
- `crates/contextra-db/src/volatile_vault.rs:129` — Suchbegriff `löschung`: `/// Zeitstempel, an dem die Löschung vollzogen wurde.`
- `crates/contextra-db/src/volatile_vault.rs:145` — Suchbegriff `deletionproof`: `/// Optionaler Verweis auf den zugehörigen Löschnachweis (z. B. DeletionProof URN).`
- `crates/contextra-db/src/volatile_vault.rs:146` — Suchbegriff `deletion_proof`: `pub deletion_proof_reference: Option<String>,`
- `crates/contextra-db/src/volatile_vault.rs:263` — Suchbegriff `purger`: `/// 3. Chunk-Liste leeren, Vault als konsumiert markieren und [`PurgeReceipt`] ausstellen.`
- `crates/contextra-db/src/volatile_vault.rs:264` — Suchbegriff `purger`: `pub fn purge(mut self) -> PurgeReceipt {`
- `crates/contextra-db/src/volatile_vault.rs:284` — Suchbegriff `purger`: `PurgeReceipt {`
- `crates/contextra-db/src/volatile_vault.rs:330` — Suchbegriff `deletion_proof`: `deletion_proof_reference: Option<impl Into<String>>,`
- `crates/contextra-db/src/volatile_vault.rs:348` — Suchbegriff `deletion_proof`: `deletion_proof_reference: deletion_proof_reference.map(|r| r.into()),`
- `crates/contextra-db/src/volatile_vault.rs:582` — Suchbegriff `deletion_proof`: `assert_eq!(receipt.deletion_proof_reference.as_deref(), Some(proof_ref));`
- `crates/contextra-db/src/lib.rs:21` — Suchbegriff `dsgvo`: `//!   Graph-CSR-Strukturen (Compressed Sparse Row) und kryptographische DSGVO-Art.-17-Löschbeweise.`
- `crates/contextra-db/src/lib.rs:57` — Suchbegriff `purger`: `CommitReceipt, PurgeReceipt, SignalModality, VaultChunk, VaultChunkMetadata, VaultConfig,`
- `crates/contextra-db/src/lib.rs:188` — Suchbegriff `purger`: `CommitReceipt, PurgeReceipt, SignalModality, VaultChunk, VaultChunkMetadata, VaultConfig,`

#### `contextra-durable-fs` (7 Stellen)
- `crates/contextra-durable-fs/src/scrub.rs:7` — Suchbegriff `dsgvo`: `//! Wird genutzt von: Bereinigungsroutinen, Lösch-Orchestrierern und DSGVO-Löschpfaden.`
- `crates/contextra-durable-fs/src/scrub.rs:12` — Suchbegriff `löschung`: `//! - [`scrub_if_safe`]: Hauptfunktion zur sicheren Löschung mit vorgeschalteter Risikoanalyse.`
- `crates/contextra-durable-fs/src/scrub.rs:32` — Suchbegriff `löschung`: `//! Belastbare und zuverlässige Datenlöschung auf Flash-Medien ist **ausschließlich via Crypto-Erase`
- `crates/contextra-durable-fs/src/journal.rs:3` — Suchbegriff `löschung`: `//! Zweck: Dauerhafte Protokollierung und Nachverfolgung von Dateien, die zur physischen Löschung`
- `crates/contextra-durable-fs/src/journal.rs:7` — Suchbegriff `löschung`: `//! Wird genutzt von: Bereinigungs- und Shredding-Routinen (z. B. WAL- und SSTable-Löschungen).`
- `crates/contextra-durable-fs/src/journal.rs:20` — Suchbegriff `löschung`: `// ZWECK: Durable Pending-Delete Journal zur Aufzeichnung und Nachverfolgung ausstehender Löschungen`
- `crates/contextra-durable-fs/src/journal.rs:33` — Suchbegriff `löschung`: `/// das Journal öffnen und alle noch anhängigen Löschungen zuverlässig nachholen (Idempotenz).`

#### `contextra-engine` (57 Stellen)
- `crates/contextra-engine/src/lib.rs:15` — Suchbegriff `deletion_proof`: `pub use contextra_crypto::deletion_proof::{`
- `crates/contextra-engine/src/lib.rs:16` — Suchbegriff `deletionproof`: `DeletionLayer, DeletionProof, DeletionScope, ExcludedScope,`
- `crates/contextra-engine/src/lib.rs:238` — Suchbegriff `deletion_proof`: `pub deletion_proof_active: bool,`
- `crates/contextra-engine/src/lib.rs:302` — Suchbegriff `deletion_proof`: `.field("deletion_proof_active", &self.deletion_proof_active)`
- `crates/contextra-engine/src/lib.rs:336` — Suchbegriff `deletion_proof`: `deletion_proof_active: false,`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:3` — Suchbegriff `löschung`: `//! **Zweck:** Fail-Closed ErasureOrchestrator für gemessene Löschungen (DeletionProof-Saga).`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:9` — Suchbegriff `löschung`: `// ZWECK: Orchestrierung der fail-closed Datenlöschung (ErasureOrchestrator, SPEC §12.7.4).`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:10` — Suchbegriff `deletionproof`: `// INVARIANTEN: DeletionProof entsteht ausschließlich nach unabhängiger LayerWitness-Messung aller S`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:11` — Suchbegriff `purger`: `// NICHT-OFFENSICHTLICH: Physische Purger laufen in fixer Reihenfolge (Text -> Vector -> LSM). Scans`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:16` — Suchbegriff `deletion_proof`: `use contextra_crypto::deletion_proof::LayerWitnessExt;`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:29` — Suchbegriff `löschung`: `/// Zur Löschung Vormerkung/Fehlgeschlagen.`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:95` — Suchbegriff `deletionproof`: `/// Cryptographic DeletionProof issued iff verified == true.`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:96` — Suchbegriff `deletionproof`: `pub proof: Option<DeletionProof>,`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:183` — Suchbegriff `purger`: `// --- STUFE C: Physische Purger in fester Reihenfolge Text -> Vector -> LSM ---`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:190` — Suchbegriff `purger`: `let (purgers_ok, purge_err) = tokio::task::spawn_blocking(move || {`

#### `contextra-graph` (5 Stellen)
- `crates/contextra-graph/src/consistency_enforcement.rs:11` — Suchbegriff `löschung`: `//! Implementiert reines Pattern-Signal — keine automatische Löschung (siehe`
- `crates/contextra-graph/src/consistency_enforcement.rs:15` — Suchbegriff `löschung`: `//! Dieses Modul implementiert KEINE automatische Löschung von Kanten. `ConsistencyEnforcer` liefert`
- `crates/contextra-graph/src/lib.rs:69` — Suchbegriff `deletionproof`: `DeletionProof, GraphGarbageCollection, HyperedgeCascadeReport, MAX_HYPEREDGE_CASCADE_FANOUT,`
- `crates/contextra-graph/src/cascade.rs:1` — Suchbegriff `löschung`: `//! Kaskadierende Invalidierung und Bereinigung von Hyperkanten bei Dokumentenlöschung.`
- `crates/contextra-graph/src/cascade.rs:105` — Suchbegriff `deletionproof`: `pub type DeletionProof = CascadeDeletionRecord;`

#### `contextra-ingest` (1 Stellen)
- `crates/contextra-ingest/src/limits.rs:73` — Suchbegriff `löschung`: `/// Nachverfolgte Löschungen (Tracked Deletions) einbeziehen? (Standard: false)`

#### `contextra-kvcache` (5 Stellen)
- `crates/contextra-kvcache/src/eviction_worker.rs:12` — Suchbegriff `löschung`: `//! 2. **[`emergency_wipe()`]**: Für seltene Notfall-Löschungen (z. B. Prozess-Shutdown,`
- `crates/contextra-kvcache/src/eviction_worker.rs:212` — Suchbegriff `löschung`: `/// Synchroner Notfall-Wipe für sofortige Speicherlöschung.`
- `crates/contextra-kvcache/src/service.rs:9` — Suchbegriff `löschung`: `//! - Scratchpad-Invalidierung & Mandantenlöschung ([`TenantId`])`
- `crates/contextra-kvcache/src/service.rs:472` — Suchbegriff `dsgvo`: `/// Entfernt alle RAM-Segmente und Radix-Knoten für einen Mandanten (DSGVO Art. 17 Mandantenlöschung`
- `crates/contextra-kvcache/src/service.rs:544` — Suchbegriff `löschung`: `// Prüft Erstellung über KvCacheServiceBuilder und Durchsetzung der Mandantentrennung bei Lookups un`

#### `contextra-mcp` (48 Stellen)
- `crates/contextra-mcp/src/server_dispatch.rs:171` — Suchbegriff `deletionproof`: `"description": "Delete a document (unverified) or drop an entire collection (with verified DeletionP`
- `crates/contextra-mcp/src/server_dispatch.rs:244` — Suchbegriff `deletionproof`: `"description": "Delete an entire collection and issue a verified collection-wide cryptographic Delet`
- `crates/contextra-mcp/src/proof_key_env.rs:3` — Suchbegriff `deletion_proof`: `// ZWECK:       Einziger Resolver für DeletionProof-Schlüssel (CONTEXTRA_DELETION_PROOF_KEY / CONTEX`
- `crates/contextra-mcp/src/proof_key_env.rs:4` — Suchbegriff `deletion_proof`: `// INVARIANTEN: Bevorzugt CONTEXTRA_DELETION_PROOF_KEY; Loggt Schlüsselmaterial NIEMALS.`
- `crates/contextra-mcp/src/proof_key_env.rs:42` — Suchbegriff `deletionproof`: `/// Reine Hilfsfunktion zur Auflösung des DeletionProof-Schlüssels ohne Zugriff auf die Prozess-Umge`
- `crates/contextra-mcp/src/proof_key_env.rs:91` — Suchbegriff `deletion_proof`: `/// Liest beide Umgebungsvariablen (`CONTEXTRA_DELETION_PROOF_KEY` und `CONTEXTRA_PROOF_KEY`),`
- `crates/contextra-mcp/src/proof_key_env.rs:94` — Suchbegriff `deletion_proof`: `let primary = std::env::var("CONTEXTRA_DELETION_PROOF_KEY").ok();`
- `crates/contextra-mcp/src/proof_key_env.rs:102` — Suchbegriff `deletion_proof`: `"CONTEXTRA_PROOF_KEY ist veraltet (deprecated). Bitte CONTEXTRA_DELETION_PROOF_KEY verwenden."`
- `crates/contextra-mcp/src/proof_key_env.rs:110` — Suchbegriff `deletion_proof`: `"Konflikt: Sowohl CONTEXTRA_DELETION_PROOF_KEY als auch CONTEXTRA_PROOF_KEY sind mit unterschiedlich`
- `crates/contextra-mcp/src/tools_crud.rs:230` — Suchbegriff `löschung`: `"reason": "Schichteigene Messung für Einzel-Dokument-Löschung nicht angebunden",`
- `crates/contextra-mcp/src/server_tools.rs:428` — Suchbegriff `löschung`: `"reason": "Schichteigene Messung für Einzel-Dokument-Löschung nicht angebunden",`
- `crates/contextra-mcp/src/sandbox.rs:45` — Suchbegriff `deletionproof`: `/// | `contextra_forget` | `DatabaseWrite` | Delete document or drop collection with verified Deleti`
- `crates/contextra-mcp/src/sandbox.rs:47` — Suchbegriff `deletionproof`: `/// | `contextra_drop_collection` | `DatabaseWrite` | Delete entire collection and issue a verified `
- `crates/contextra-mcp/src/sandbox.rs:94` — Suchbegriff `deletionproof`: `description: "Delete a document (unverified) or drop an entire collection (with verified DeletionPro`
- `crates/contextra-mcp/src/sandbox.rs:172` — Suchbegriff `deletionproof`: `description: "Delete an entire collection and issue a verified collection-wide cryptographic Deletio`

#### `contextra-mvcc` (7 Stellen)
- `crates/contextra-mvcc/src/tx_buffer.rs:118` — Suchbegriff `löschung`: `/// Optionale Zusatzdaten zur Löschung.`
- `crates/contextra-mvcc/src/tx_buffer.rs:833` — Suchbegriff `löschung`: `/// Liefert `Some(true)` bei Einfügung, `Some(false)` bei Löschung oder `None`, falls der Schlüssel `
- `crates/contextra-mvcc/src/tx_buffer.rs:1188` — Suchbegriff `löschung`: `// Prüft die Semantik von is_key_staged_for_tx für Einfügungen und Löschungen.`
- `crates/contextra-mvcc/src/snapshot.rs:8` — Suchbegriff `löschung`: `//! In einem MVCC-System erzeugen Löschungen sogenannte Tombstones (Grabsteine).`
- `crates/contextra-mvcc/src/seq_log.rs:47` — Suchbegriff `löschung`: `/// - 8 Bytes für `delete_seq: Option<u64>` (Sequenznummer der Löschung)`
- `crates/contextra-mvcc/src/seq_log.rs:413` — Suchbegriff `löschung`: `/// deren Löschung strikt kleiner als `min_active_seqno` ist.`
- `crates/contextra-mvcc/src/seq_log.rs:453` — Suchbegriff `löschung`: `/// Liefert alle Änderungen (Einfügungen und Löschungen), die strikt nach `snapshot_seq``

#### `contextra-ports` (9 Stellen)
- `crates/contextra-ports/src/lib.rs:64` — Suchbegriff `purger`: `/// Storage layer scanners and purgers (Vertrag V1, SPEC §12.7.4).`
- `crates/contextra-ports/src/lib.rs:93` — Suchbegriff `purger`: `LayerScanner, LayerWitness, PurgeStats, Purger, ScanSession, ScanTarget, ScanTechnique,`
- `crates/contextra-ports/src/lib.rs:129` — Suchbegriff `purger`: `fn _assert_dyn_purger(_: Option<&dyn Purger>) {}`
- `crates/contextra-ports/src/lib.rs:149` — Suchbegriff `purger`: `_assert_dyn_purger(None);`
- `crates/contextra-ports/src/scanner/witness.rs:2` — Suchbegriff `löschung`: `// ZWECK: Unveränderlicher LayerWitness für gemessene Restbestände nach einer Löschung (SPEC §12.7.4`
- `crates/contextra-ports/src/scanner/layer.rs:2` — Suchbegriff `purger`: `// ZWECK: Definition der Schichten, Scantechniken, Target-Deskriptoren und Purger-Traits (Vertrag V1`
- `crates/contextra-ports/src/scanner/layer.rs:114` — Suchbegriff `purger`: `/// Trait für physische Bereinigungsaktionen (Purger) einer Schicht.`
- `crates/contextra-ports/src/scanner/layer.rs:115` — Suchbegriff `purger`: `pub trait Purger: Send + Sync {`
- `crates/contextra-ports/src/scanner/layer.rs:116` — Suchbegriff `löschung`: `/// Führt die physische Löschung für das gegebene Ziel aus.`

#### `contextra-privacy` (44 Stellen)
- `crates/contextra-privacy/src/avv_generator.rs:1` — Suchbegriff `avv`: `//! `avv_generator` — Re-Export-Schicht auf `contextra-avv-generator`.`
- `crates/contextra-privacy/src/avv_generator.rs:5` — Suchbegriff `dsgvo`: `//! (AVV) gemäß Art. 28 DSGVO (Datenschutz-Grundverordnung). Die Logik wurde in das dedizierte Crate`
- `crates/contextra-privacy/src/avv_generator.rs:6` — Suchbegriff `avv`: `//! `contextra-avv-generator` ausgelagert.`
- `crates/contextra-privacy/src/avv_generator.rs:12` — Suchbegriff `avv`: `//! Veraltet (deprecated). Neue Implementierungen sollten direkt das Crate `contextra-avv-generator``
- `crates/contextra-privacy/src/avv_generator.rs:14` — Suchbegriff `avv`: `#![deprecated(note = "use contextra-avv-generator crate directly")]`
- `crates/contextra-privacy/src/avv_generator.rs:16` — Suchbegriff `avv`: `// ANCHOR[contextra-privacy:AVV-001]`
- `crates/contextra-privacy/src/avv_generator.rs:17` — Suchbegriff `avv`: `/// Re-Export des Kontexts für die AVV-Generierung aus `contextra-avv-generator`.`
- `crates/contextra-privacy/src/avv_generator.rs:18` — Suchbegriff `avv`: `#[deprecated(note = "use contextra_avv_generator::AvvContext")]`
- `crates/contextra-privacy/src/avv_generator.rs:19` — Suchbegriff `avv`: `pub use contextra_avv_generator::AvvContext;`
- `crates/contextra-privacy/src/avv_generator.rs:21` — Suchbegriff `avv`: `// ANCHOR[contextra-privacy:AVV-002]`
- `crates/contextra-privacy/src/avv_generator.rs:22` — Suchbegriff `avv`: `/// Generiert einen AVV-Entwurf im Markdown-Format anhand des übergebenen Kontexts.`
- `crates/contextra-privacy/src/avv_generator.rs:25` — Suchbegriff `avv`: `/// 1. Übergibt die Kontextstruktur an `contextra_avv_generator::render_avv_markdown`.`
- `crates/contextra-privacy/src/avv_generator.rs:30` — Suchbegriff `avv`: `/// * `ctx`: Referenz auf den AVV-Kontext mit Angaben zu Verantwortlichem und Auftragsverarbeiter.`
- `crates/contextra-privacy/src/avv_generator.rs:34` — Suchbegriff `avv`: `#[deprecated(note = "use contextra_avv_generator::render_avv_markdown")]`
- `crates/contextra-privacy/src/avv_generator.rs:35` — Suchbegriff `avv`: `pub fn generate_avv_draft(ctx: &contextra_avv_generator::AvvContext) -> String {`

#### `contextra-store` (28 Stellen)
- `crates/contextra-store/src/scanner/purge.rs:6` — Suchbegriff `purger`: `// ZWECK: Physical Purger implementation für LSM-Storage-Schichten (§12.7.4, INV-WITNESS-MINT-1).`
- `crates/contextra-store/src/scanner/purge.rs:17` — Suchbegriff `purger`: `use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};`
- `crates/contextra-store/src/scanner/purge.rs:23` — Suchbegriff `purger`: `/// Purger für LSM Storage Engine.`
- `crates/contextra-store/src/scanner/purge.rs:24` — Suchbegriff `purger`: `pub struct LsmPurger {`
- `crates/contextra-store/src/scanner/purge.rs:28` — Suchbegriff `purger`: `impl LsmPurger {`
- `crates/contextra-store/src/scanner/purge.rs:29` — Suchbegriff `purger`: `/// Erzeugt einen neuen `LsmPurger`.`
- `crates/contextra-store/src/scanner/purge.rs:35` — Suchbegriff `purger`: `impl Purger for LsmPurger {`
- `crates/contextra-store/src/scanner/mod.rs:6` — Suchbegriff `purger`: `// ZWECK: Export und Orchestrierung der LSM Storage-Scanner und Purger (SPEC §12.7.4, Verträge W1-P0`
- `crates/contextra-store/src/scanner/mod.rs:7` — Suchbegriff `purger`: `// INVARIANTEN: lsm_scanners und lsm_purgers erzeugen nur valide, isolierte Instanzen für Memtable, `
- `crates/contextra-store/src/scanner/mod.rs:21` — Suchbegriff `purger`: `pub use purge::LsmPurger;`
- `crates/contextra-store/src/scanner/mod.rs:26` — Suchbegriff `purger`: `use contextra_ports::scanner::{LayerScanner, Purger};`
- `crates/contextra-store/src/scanner/mod.rs:38` — Suchbegriff `purger`: `/// Erzeugt den LSM-Storage-Purger für eine gegebene LsmStorage-Instanz.`
- `crates/contextra-store/src/scanner/mod.rs:39` — Suchbegriff `purger`: `pub fn lsm_purgers(storage: &Arc<LsmStorage>) -> Vec<Arc<dyn Purger>> {`
- `crates/contextra-store/src/scanner/mod.rs:40` — Suchbegriff `purger`: `vec![Arc::new(LsmPurger::new(Arc::clone(storage)))]`
- `crates/contextra-store/src/scanner/tests.rs:6` — Suchbegriff `löschkette`: `// ZWECK: Tests für LSM Scanner & Purger (Vertrag V1, LÖSCHKETTE W1-P01).`

#### `contextra-testkit` (3 Stellen)
- `crates/contextra-testkit/src/reference_model.rs:152` — Suchbegriff `löschung`: `/// 3. Gibt `Some(bytes)` zurück, falls es ein aktiver Wert war, oder `None` bei Löschung/Nichtexist`
- `crates/contextra-testkit/src/conformance/text.rs:8` — Suchbegriff `löschung`: `//! 2. **Löschung (Never retrievable):** Gelöschte und bestätigte Dokumente tauchen bei Folgesuchen `
- `crates/contextra-testkit/src/conformance/vector.rs:8` — Suchbegriff `löschung`: `//! 2. **Löschung & Tombstone-Konsistenz:** Gelöschte Dokumente dürfen weder in `search` noch in `al`

#### `contextra-text` (29 Stellen)
- `crates/contextra-text/src/inverted.rs:24` — Suchbegriff `löschung`: `//! physische Löschung (Purge) und Scans für Verifiable Deletion.`
- `crates/contextra-text/src/inverted.rs:37` — Suchbegriff `purger`: `pub use purge::{text_purger, PurgeReport, TextPurger};`
- `crates/contextra-text/src/lib.rs:51` — Suchbegriff `purger`: `text_purger, text_scanner, BM25MorphIndex, InvertedIndex, Language, PurgeReport, TextPurger,`
- `crates/contextra-text/src/inverted/purge.rs:1` — Suchbegriff `purger`: `// FILE-CONTEXT: TextIndex Physical Purger & BM25 Corpus Statistics Recalculation (§12.7.4).`
- `crates/contextra-text/src/inverted/purge.rs:2` — Suchbegriff `purger`: `// ZWECK: Stellt den Purger für physical deletion per document and per collection bereit.`
- `crates/contextra-text/src/inverted/purge.rs:7` — Suchbegriff `löschung`: `//! Physische Löschung (Purge) für Dokumente und Kollektionen im Textindex.`
- `crates/contextra-text/src/inverted/purge.rs:12` — Suchbegriff `dsgvo`: `//! # Zweck & DSGVO-Konformität`
- `crates/contextra-text/src/inverted/purge.rs:13` — Suchbegriff `dsgvo`: `//! Stellt sicher, dass bei Verifiable Deletion (DSGVO Art. 17) sämtliche Spuren`
- `crates/contextra-text/src/inverted/purge.rs:21` — Suchbegriff `purger`: `use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};`
- `crates/contextra-text/src/inverted/purge.rs:39` — Suchbegriff `purger`: `pub struct PurgeReport {`
- `crates/contextra-text/src/inverted/purge.rs:50` — Suchbegriff `purger`: `impl PurgeReport {`
- `crates/contextra-text/src/inverted/purge.rs:63` — Suchbegriff `löschung`: `/// Liefert `true`, wenn die Löschung fehlerfrei und vollständig war.`
- `crates/contextra-text/src/inverted/purge.rs:85` — Suchbegriff `purger`: `impl Default for PurgeReport {`
- `crates/contextra-text/src/inverted/purge.rs:92` — Suchbegriff `purger`: `/// Physischer Purger für den Textindex zur Bereinigung von Storage und Metadaten.`
- `crates/contextra-text/src/inverted/purge.rs:93` — Suchbegriff `purger`: `pub struct TextPurger<S: StorageEngine + 'static> {`

#### `contextra-types` (4 Stellen)
- `crates/contextra-types/src/witness.rs:5` — Suchbegriff `dsgvo`: `//! Für DSGVO-/Compliance-konforme Löschnachweise (Deletion Proofs) muss nachgewiesen werden,`
- `crates/contextra-types/src/tombstone.rs:6` — Suchbegriff `löschung`: `//! bei einer Löschung nicht physisch überschrieben, sondern mit einem Löschmarker ("Tombstone")`
- `crates/contextra-types/src/error.rs:333` — Suchbegriff `deletion_proof`: `/// KV-seitige `KvDeleteMode`-Konfiguration inkonsistent mit `deletion_proof_active`.`
- `crates/contextra-types/src/lib.rs:22` — Suchbegriff `avv`: `//! - `contextra`, `contextra-adapt`, `contextra-agent`, `contextra-audit-export`, `contextra-avv-ge`

#### `contextra-vector` (12 Stellen)
- `crates/contextra-vector/src/lib.rs:50` — Suchbegriff `purger`: `hnsw_purger, hnsw_scanner, HnswConfig, HnswIndex, HnswPurger, HnswScanner, RebuildStatus,`
- `crates/contextra-vector/src/hnsw/purge.rs:2` — Suchbegriff `purger`: `// ZWECK: Fail-closed Hard-Purge für HNSW Vektor-Indizes (Purger Trait, Welle 2/3).`
- `crates/contextra-vector/src/hnsw/purge.rs:9` — Suchbegriff `purger`: `use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};`
- `crates/contextra-vector/src/hnsw/purge.rs:15` — Suchbegriff `purger`: `/// Purger-Implementierung für `WitnessLayer::HnswIndex`.`
- `crates/contextra-vector/src/hnsw/purge.rs:16` — Suchbegriff `purger`: `pub struct HnswPurger {`
- `crates/contextra-vector/src/hnsw/purge.rs:20` — Suchbegriff `purger`: `impl HnswPurger {`
- `crates/contextra-vector/src/hnsw/purge.rs:21` — Suchbegriff `purger`: `/// Erzeugt einen neuen `HnswPurger`.`
- `crates/contextra-vector/src/hnsw/purge.rs:27` — Suchbegriff `purger`: `/// Erzeugt einen `Arc<dyn Purger>` für einen HNSW Index.`
- `crates/contextra-vector/src/hnsw/purge.rs:28` — Suchbegriff `purger`: `pub fn hnsw_purger(index: Arc<HnswIndex>) -> Arc<dyn Purger> {`
- `crates/contextra-vector/src/hnsw/purge.rs:29` — Suchbegriff `purger`: `Arc::new(HnswPurger::new(index))`
- `crates/contextra-vector/src/hnsw/purge.rs:32` — Suchbegriff `purger`: `impl Purger for HnswPurger {`
- `crates/contextra-vector/src/hnsw/mod.rs:16` — Suchbegriff `purger`: `pub use purge::{hnsw_purger, HnswPurger};`

#### `contextra-wire` (1 Stellen)
- `crates/contextra-wire/src/adapter.rs:25` — Suchbegriff `löschung`: `/// - Keine Zeiger-Transmutes (`transmute`) oder Lebenszeit-Löschungen (`lifetime erasures`).`
