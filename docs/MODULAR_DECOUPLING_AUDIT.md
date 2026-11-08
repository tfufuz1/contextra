# Contextra Modularisierungs- & Entkopplungs-Audit

> **Datum:** 2026-11-07 21:03:56 | **Analysierte Crates:** 36 | **Gesamtumfang:** 188,176 LOC

## 1. Strategische Zielsetzung & Architekturvision
Dieses Dokument dient als zentrale Orientierung für Architekten und Coding-Agenten zur Transformation des Contextra-Ökosystems von einem fest verdrahteten Monolithen zu einem **vollständig modularen Baukastensystem (Building Blocks)**.

### Kernprinzipien des Baukastensystems:
1. **Vollständige Entkopplung:** Jeder Crate muss isoliert verständlich, testbar und in Fremdprojekten nutzbar sein.
2. **Schnittstellen via Ports/Traits:** Direkte Abhängigkeiten zwischen konkreten Crates werden aufgelöst; Abstraktionen liegen in `contextra-ports` oder generischen Parametern.
3. **Isolation der Löschkette:** Alle DSGVO-, Gesetzes- und multi-layer Deletion-Proof-Logiken werden vollständig in `contextra-erasure` isoliert und aus Kernkomponenten (Store, Vector, Text, Crypto, DB) entfernt.
4. **Zero-Dead-Code & Saubere Fassaden:** Öffentliche Schnittstellen werden minimal gehalten und lecken keine internen Fremdtypen.

## 2. Gesamtübersicht des Baukastensystems

- **Durchschnittlicher Modularitäts-Score:** 70.2 / 100
- **Vollkommen autarke Bausteine:** 4 Crates
- **Dringender Entkopplungsbedarf (Score < 60):** 11 Crates
- **Verstreute Löschketten-/DSGVO-Relikte außerhalb `contextra-erasure`:** 487 Fundstellen
- **Identifizierte Dead-Code-Kandidaten (ungenutzte interne Definitionen):** 81 Stellen
- **Fehlende Port-Abstraktionen (direkte Kernel-Importe):** 254 Stellen

| Crate | Modularitäts-Score | Status | LOC | Dateien | Interne Deps | Schnittstellen (T/S/E) | Pub/Gesamt Fns | Dead Code | DSGVO/Lösch-Relikte |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| `contextra-engine` | **10/100** | Stark verdrahtet (Refactoring nötig) | 17,948 | 84 | 13 | 7/78/10 | 349/694 | 0 | 51 |
| `contextra-vector` | **25/100** | Stark verdrahtet (Refactoring nötig) | 11,841 | 45 | 5 | 2/49/6 | 254/440 | 0 | 10 |
| `contextra-store` | **35/100** | Stark verdrahtet (Refactoring nötig) | 23,090 | 77 | 5 | 6/68/18 | 356/871 | 0 | 26 |
| `contextra` | **39/100** | Stark verdrahtet (Refactoring nötig) | 1,233 | 7 | 6 | 1/10/9 | 43/86 | 0 | 2 |
| `contextra-graph` | **48/100** | Stark verdrahtet (Refactoring nötig) | 16,812 | 73 | 2 | 10/94/7 | 278/707 | 0 | 16 |
| `contextra-ports` | **50/100** | Stark verdrahtet (Refactoring nötig) | 4,507 | 41 | 1 | 46/74/20 | 78/521 | 18 | 11 |
| `contextra-text` | **52/100** | Stark verdrahtet (Refactoring nötig) | 5,890 | 26 | 2 | 3/36/3 | 109/365 | 2 | 23 |
| `contextra-mcp` | **53/100** | Stark verdrahtet (Refactoring nötig) | 5,284 | 24 | 11 | 0/29/5 | 81/231 | 5 | 0 |
| `contextra-agent` | **54/100** | Stark verdrahtet (Refactoring nötig) | 4,297 | 18 | 6 | 5/32/6 | 100/219 | 0 | 0 |
| `contextra-kvcache` | **57/100** | Stark verdrahtet (Refactoring nötig) | 4,297 | 11 | 3 | 3/40/4 | 128/252 | 1 | 3 |
| `contextra-crypto` | **58/100** | Stark verdrahtet (Refactoring nötig) | 5,926 | 26 | 2 | 3/46/14 | 177/368 | 0 | 88 |
| `contextra-router` | **61/100** | Akzeptabel | 5,781 | 28 | 4 | 0/25/6 | 73/213 | 0 | 0 |
| `contextra-audit-export` | **64/100** | Akzeptabel | 557 | 5 | 1 | 1/8/1 | 14/19 | 0 | 31 |
| `contextra-avv-generator` | **64/100** | Akzeptabel | 508 | 9 | 1 | 0/2/1 | 16/23 | 0 | 226 |
| `contextra-core` | **66/100** | Akzeptabel | 106 | 1 | 4 | 0/0/0 | 0/0 | 0 | 0 |
| `contextra-infer-candle` | **71/100** | Akzeptabel | 5,015 | 17 | 4 | 2/32/2 | 71/202 | 0 | 0 |
| `contextra-durable-fs` | **75/100** | Akzeptabel | 978 | 6 | 3 | 1/10/8 | 38/79 | 5 | 0 |
| `contextra-cognition` | **76/100** | Akzeptabel | 5,332 | 24 | 4 | 3/45/1 | 77/229 | 0 | 0 |
| `contextra-erasure` | **77/100** | Akzeptabel | 8,628 | 61 | 3 | 23/114/41 | 382/553 | 0 | 0 |
| `contextra-py` | **77/100** | Akzeptabel | 2,235 | 16 | 3 | 0/8/0 | 61/161 | 0 | 0 |
| `contextra-db` | **77/100** | Akzeptabel | 1,503 | 5 | 3 | 0/14/2 | 34/80 | 0 | 0 |
| `contextra-infer-onnx` | **82/100** | Exzellent (Baustein) | 2,053 | 6 | 3 | 0/11/1 | 46/110 | 0 | 0 |
| `contextra-infer-ollama` | **83/100** | Exzellent (Baustein) | 4,973 | 16 | 2 | 2/34/1 | 69/209 | 0 | 0 |
| `contextra-mvcc` | **83/100** | Exzellent (Baustein) | 2,909 | 5 | 2 | 2/13/3 | 124/215 | 0 | 0 |
| `contextra-testkit` | **83/100** | Exzellent (Baustein) | 2,611 | 20 | 2 | 0/11/6 | 48/132 | 0 | 0 |
| `contextra-ingest` | **84/100** | Exzellent (Baustein) | 16,460 | 77 | 1 | 2/89/20 | 132/508 | 30 | 0 |
| `contextra-privacy` | **88/100** | Exzellent (Baustein) | 3,695 | 11 | 2 | 6/33/9 | 91/205 | 0 | 0 |
| `contextra-adapt` | **88/100** | Exzellent (Baustein) | 3,617 | 13 | 2 | 7/34/7 | 99/227 | 0 | 0 |
| `contextra-checkpoint` | **88/100** | Exzellent (Baustein) | 3,584 | 11 | 2 | 3/21/0 | 104/279 | 0 | 0 |
| `contextra-rank` | **88/100** | Exzellent (Baustein) | 3,456 | 21 | 2 | 1/25/8 | 103/167 | 0 | 0 |
| `contextra-license` | **88/100** | Exzellent (Baustein) | 505 | 2 | 2 | 0/5/1 | 34/38 | 0 | 0 |
| `contextra-types` | **90/100** | Exzellent (Baustein) | 5,675 | 25 | 0 | 1/56/35 | 140/332 | 20 | 0 |
| `contextra-simd` | **94/100** | Exzellent (Baustein) | 1,745 | 7 | 1 | 0/4/1 | 62/77 | 0 | 0 |
| `contextra-wire` | **100/100** | Exzellent (Baustein) | 2,038 | 6 | 0 | 0/31/10 | 116/180 | 0 | 0 |
| `contextra-sandbox` | **100/100** | Exzellent (Baustein) | 1,904 | 13 | 0 | 0/11/4 | 26/59 | 0 | 0 |
| `contextra-sys` | **100/100** | Exzellent (Baustein) | 1,183 | 7 | 0 | 0/7/1 | 36/73 | 0 | 0 |

## 3. Priorisierte Entkopplungs-Roadmap für Coding-Agenten

### 📦 Crate: `contextra` (Score: 39/100)
- **Umfang:** 1,233 LOC (7 Quelldateien)
- **Abhängigkeiten (6):** `contextra-db, contextra-infer-candle, contextra-license, contextra-ports, contextra-rank, contextra-types`
- **Funktionsinventar:** 43 pub-Funktionen von 86 gesamt
- **Schnittstellen:** 1 Traits, 10 Structs, 9 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-db`: 15 Symbole
  - `contextra-ports`: 9 Symbole
  - `contextra-types`: 10 Symbole
  - `contextra-rank`: 2 Symbole
  - `contextra-infer-candle`: 2 Symbole
  - `contextra-license`: 1 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 2 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-db' (15 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (9 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (10 importierte Symbole).
  - 2 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-db' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-agent` (Score: 54/100)
- **Umfang:** 4,297 LOC (18 Quelldateien)
- **Abhängigkeiten (6):** `contextra-checkpoint, contextra-db, contextra-graph, contextra-ports, contextra-router, contextra-types`
- **Funktionsinventar:** 100 pub-Funktionen von 219 gesamt
- **Schnittstellen:** 5 Traits, 32 Structs, 6 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ports`: 4 Symbole
  - `contextra-types`: 15 Symbole
  - `contextra-checkpoint`: 4 Symbole
  - `contextra-db`: 9 Symbole
  - `contextra-graph`: 1 Symbole
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-types' (15 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-db' (9 importierte Symbole).
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-db' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.

### 📦 Crate: `contextra-crypto` (Score: 58/100)
- **Umfang:** 5,926 LOC (26 Quelldateien)
- **Abhängigkeiten (2):** `contextra-ports, contextra-types`
- **Funktionsinventar:** 177 pub-Funktionen von 368 gesamt
- **Schnittstellen:** 3 Traits, 46 Structs, 14 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 8 Symbole
  - `contextra-ports`: 7 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 88 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - 88 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] DSGVO-/Löschlogik aus contextra-crypto extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-engine` (Score: 10/100)
- **Umfang:** 17,948 LOC (84 Quelldateien)
- **Abhängigkeiten (13):** `contextra-adapt, contextra-checkpoint, contextra-crypto, contextra-graph, contextra-infer-onnx, contextra-mvcc, contextra-ports, contextra-rank, contextra-sandbox, contextra-store, contextra-text, contextra-types, contextra-vector`
- **Funktionsinventar:** 349 pub-Funktionen von 694 gesamt
- **Schnittstellen:** 7 Traits, 78 Structs, 10 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-rank`: 19 Symbole
  - `contextra-types`: 27 Symbole
  - `contextra-ports`: 27 Symbole
  - `contextra-store`: 5 Symbole
  - `contextra-vector`: 4 Symbole
  - `contextra-sandbox`: 3 Symbole
  - `contextra-crypto`: 6 Symbole
  - `contextra-text`: 3 Symbole
  - `contextra-graph`: 10 Symbole
  - `contextra-mvcc`: 3 Symbole
- **⚠️ Direkte Kernel-Verdrahtung:** 198 Importe müssen über `contextra-ports` entkoppelt werden.
- **⚠️ DSGVO-/Löschketten-Lecks:** 51 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-rank' (19 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (27 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (27 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-graph' (10 importierte Symbole).
  - 198 Stellen mit direkten Importen aus konkreten Speicherkernen (fehlende Port-/Trait-Abstraktion).
  - 51 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-rank' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-graph' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Direkte Importe aus konkreten Speicher-Crates durch Ports in 'contextra-ports' ersetzen.
  - [ ] DSGVO-/Löschlogik aus contextra-engine extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-graph` (Score: 48/100)
- **Umfang:** 16,812 LOC (73 Quelldateien)
- **Abhängigkeiten (2):** `contextra-ports, contextra-types`
- **Funktionsinventar:** 278 pub-Funktionen von 707 gesamt
- **Schnittstellen:** 10 Traits, 94 Structs, 7 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 12 Symbole
  - `contextra-ports`: 15 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 16 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-types' (12 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (15 importierte Symbole).
  - 16 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-graph extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-kvcache` (Score: 57/100)
- **Umfang:** 4,297 LOC (11 Quelldateien)
- **Abhängigkeiten (3):** `contextra-crypto, contextra-ports, contextra-types`
- **Funktionsinventar:** 128 pub-Funktionen von 252 gesamt
- **Schnittstellen:** 3 Traits, 40 Structs, 4 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 12 Symbole
  - `contextra-ports`: 34 Symbole
  - `contextra-crypto`: 5 Symbole
- **⚠️ Direkte Kernel-Verdrahtung:** 8 Importe müssen über `contextra-ports` entkoppelt werden.
- **⚠️ Dead Code:** 1 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 3 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-types' (12 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (34 importierte Symbole).
  - 8 Stellen mit direkten Importen aus konkreten Speicherkernen (fehlende Port-/Trait-Abstraktion).
  - 3 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
  - 1 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Direkte Importe aus konkreten Speicher-Crates durch Ports in 'contextra-ports' ersetzen.
  - [ ] DSGVO-/Löschlogik aus contextra-kvcache extrahieren und in 'contextra-erasure' modularisieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] scan_units` (crates/contextra-kvcache/src/store.rs:857)

### 📦 Crate: `contextra-mcp` (Score: 53/100)
- **Umfang:** 5,284 LOC (24 Quelldateien)
- **Abhängigkeiten (11):** `contextra, contextra-crypto, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ports, contextra-privacy, contextra-rank, contextra-router, contextra-types, contextra-wire`
- **Funktionsinventar:** 81 pub-Funktionen von 231 gesamt
- **Schnittstellen:** 0 Traits, 29 Structs, 5 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-privacy`: 8 Symbole
  - `contextra`: 39 Symbole
  - `contextra-wire`: 3 Symbole
  - `contextra-types`: 8 Symbole
  - `contextra-ports`: 5 Symbole
  - `contextra-rank`: 2 Symbole
- **⚠️ Dead Code:** 5 ungenutzte interne Definitionen identifiziert.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra' (39 importierte Symbole).
  - 5 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] search_text` (crates/contextra-mcp/src/egress_guard.rs:14)
  - `[fn] search_hybrid` (crates/contextra-mcp/src/routing.rs:9)
  - `[fn] get_community` (crates/contextra-mcp/src/routing.rs:39)
  - `[fn] prepare_context` (crates/contextra-mcp/src/routing.rs:50)
  - `[fn] default_collection` (crates/contextra-mcp/src/explain.rs:17)

### 📦 Crate: `contextra-ports` (Score: 50/100)
- **Umfang:** 4,507 LOC (41 Quelldateien)
- **Abhängigkeiten (1):** `contextra-types`
- **Funktionsinventar:** 78 pub-Funktionen von 521 gesamt
- **Schnittstellen:** 46 Traits, 74 Structs, 20 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 39 Symbole
- **⚠️ Dead Code:** 18 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 11 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-types' (39 importierte Symbole).
  - 11 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
  - 18 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-ports extrahieren und in 'contextra-erasure' modularisieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] read_all` (crates/contextra-ports/src/ledger.rs:161)
  - `[fn] prune` (crates/contextra-ports/src/ledger.rs:170)
  - `[fn] seal` (crates/contextra-ports/src/keys.rs:129)
  - `[fn] make_slots_durable` (crates/contextra-ports/src/keys.rs:161)
  - `[fn] destroy` (crates/contextra-ports/src/keys.rs:195)

### 📦 Crate: `contextra-store` (Score: 35/100)
- **Umfang:** 23,090 LOC (77 Quelldateien)
- **Abhängigkeiten (5):** `contextra-crypto, contextra-mvcc, contextra-ports, contextra-sys, contextra-types`
- **Funktionsinventar:** 356 pub-Funktionen von 871 gesamt
- **Schnittstellen:** 6 Traits, 68 Structs, 18 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 8 Symbole
  - `contextra-mvcc`: 5 Symbole
  - `contextra-ports`: 17 Symbole
  - `contextra-crypto`: 8 Symbole
- **⚠️ Direkte Kernel-Verdrahtung:** 29 Importe müssen über `contextra-ports` entkoppelt werden.
- **⚠️ DSGVO-/Löschketten-Lecks:** 26 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-ports' (17 importierte Symbole).
  - 29 Stellen mit direkten Importen aus konkreten Speicherkernen (fehlende Port-/Trait-Abstraktion).
  - 26 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Direkte Importe aus konkreten Speicher-Crates durch Ports in 'contextra-ports' ersetzen.
  - [ ] DSGVO-/Löschlogik aus contextra-store extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-text` (Score: 52/100)
- **Umfang:** 5,890 LOC (26 Quelldateien)
- **Abhängigkeiten (2):** `contextra-ports, contextra-types`
- **Funktionsinventar:** 109 pub-Funktionen von 365 gesamt
- **Schnittstellen:** 3 Traits, 36 Structs, 3 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ports`: 12 Symbole
  - `contextra-types`: 8 Symbole
- **⚠️ Dead Code:** 2 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 23 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-ports' (12 importierte Symbole).
  - 23 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
  - 2 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-text extrahieren und in 'contextra-erasure' modularisieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] layer` (crates/contextra-text/src/inverted/scanner.rs:194)
  - `[fn] scan_units` (crates/contextra-text/src/inverted/scanner.rs:197)

### 📦 Crate: `contextra-vector` (Score: 25/100)
- **Umfang:** 11,841 LOC (45 Quelldateien)
- **Abhängigkeiten (5):** `contextra-mvcc, contextra-ports, contextra-simd, contextra-sys, contextra-types`
- **Funktionsinventar:** 254 pub-Funktionen von 440 gesamt
- **Schnittstellen:** 2 Traits, 49 Structs, 6 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 9 Symbole
  - `contextra-simd`: 18 Symbole
  - `contextra-ports`: 13 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 10 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-types' (9 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-simd' (18 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (13 importierte Symbole).
  - 10 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-simd' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-vector extrahieren und in 'contextra-erasure' modularisieren.

## 4. Löschketten- und DSGVO-Fundstelleninventar

Folgende Stellen müssen im Rahmen der Entkopplung aus den jeweiligen Crates bereinigt werden:

#### `contextra` (2 Stellen)
- `crates/contextra/src/builder.rs:372` — Suchbegriff `deletion_proof`: `|| config_for_open.deletion_proof_active != resolved.integrity_proof_active`
- `crates/contextra/src/builder.rs:382` — Suchbegriff `deletion_proof`: `config_for_open.deletion_proof_active = resolved.integrity_proof_active;`

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
- `crates/contextra-audit-export/src/bsi_mapping.rs:41` — Suchbegriff `deletion_proof`: `code_location: "crates/contextra-crypto/src/deletion_proof/mod.rs".to_string(),`
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

#### `contextra-crypto` (88 Stellen)
- `crates/contextra-crypto/src/crypto.rs:339` — Suchbegriff `deletion_proof`: `pub fn derive_deletion_proof_key(&self) -> Result<[u8; 32]> {`
- `crates/contextra-crypto/src/crypto.rs:514` — Suchbegriff `deletion_proof`: `fn test_derive_deletion_proof_key() {`
- `crates/contextra-crypto/src/crypto.rs:516` — Suchbegriff `deletion_proof`: `let deletion_key = km.derive_deletion_proof_key().unwrap();`
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
- `crates/contextra-crypto/src/ed25519_proof.rs:117` — Suchbegriff `deletion_proof`: `pub fn verify_deletion_proof_v3(`
- `crates/contextra-crypto/src/ed25519_proof.rs:121` — Suchbegriff `deletionproof`: `) -> Result<(), DeletionProofError> {`

#### `contextra-engine` (51 Stellen)
- `crates/contextra-engine/src/lib.rs:20` — Suchbegriff `deletion_proof`: `pub use contextra_crypto::deletion_proof::{`
- `crates/contextra-engine/src/lib.rs:21` — Suchbegriff `deletionproof`: `DeletionLayer, DeletionProof, DeletionScope, ExcludedScope,`
- `crates/contextra-engine/src/lib.rs:247` — Suchbegriff `deletion_proof`: `pub deletion_proof_active: bool,`
- `crates/contextra-engine/src/lib.rs:311` — Suchbegriff `deletion_proof`: `.field("deletion_proof_active", &self.deletion_proof_active)`
- `crates/contextra-engine/src/lib.rs:345` — Suchbegriff `deletion_proof`: `deletion_proof_active: false,`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:11` — Suchbegriff `deletionproof`: `// INVARIANTEN: DeletionProof entsteht ausschließlich nach unabhängiger LayerWitness-Messung aller S`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:12` — Suchbegriff `purger`: `// NICHT-OFFENSICHTLICH: Physische Purger laufen in fixer Reihenfolge (Text -> Vector -> LSM). Scans`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:17` — Suchbegriff `deletion_proof`: `use contextra_crypto::deletion_proof::LayerWitnessExt;`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:98` — Suchbegriff `deletionproof`: `/// Cryptographic DeletionProof issued iff verified == true.`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:99` — Suchbegriff `deletionproof`: `pub proof: Option<DeletionProof>,`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:186` — Suchbegriff `purger`: `// --- STUFE C: Physische Purger in fester Reihenfolge Text -> Vector -> LSM ---`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:193` — Suchbegriff `purger`: `let (purgers_ok, purge_err) = tokio::task::spawn_blocking(move || {`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:194` — Suchbegriff `purger`: `let text_purger = contextra_text::text_purger(text_index_arc);`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:195` — Suchbegriff `purger`: `let hnsw_purger = contextra_vector::hnsw_purger(vector_index_arc);`
- `crates/contextra-engine/src/contextra_impl/erasure.rs:196` — Suchbegriff `purger`: `let lsm_purgers = contextra_store::scanner::lsm_purgers(&storage_arc);`

#### `contextra-graph` (16 Stellen)
- `crates/contextra-graph/src/purger.rs:1` — Suchbegriff `purger`: `//! Neutrale Wartungs- und Bereinigungs-Implementierung für den Graph-Baustein (`contextra-ports::sc`
- `crates/contextra-graph/src/purger.rs:15` — Suchbegriff `purger`: `use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};`
- `crates/contextra-graph/src/purger.rs:20` — Suchbegriff `purger`: `/// Purger-Implementierung für [`CsrGraph`].`
- `crates/contextra-graph/src/purger.rs:24` — Suchbegriff `purger`: `pub struct GraphPurger {`
- `crates/contextra-graph/src/purger.rs:28` — Suchbegriff `purger`: `impl GraphPurger {`
- `crates/contextra-graph/src/purger.rs:29` — Suchbegriff `purger`: `/// Erzeugt einen neuen [`GraphPurger`].`
- `crates/contextra-graph/src/purger.rs:35` — Suchbegriff `purger`: `/// Erzeugt einen `Arc<dyn Purger>` für einen gegebenen [`CsrGraph`].`
- `crates/contextra-graph/src/purger.rs:39` — Suchbegriff `purger`: `pub fn graph_purger(graph: Arc<CsrGraph>) -> Arc<dyn Purger> {`
- `crates/contextra-graph/src/purger.rs:40` — Suchbegriff `purger`: `Arc::new(GraphPurger::new(graph))`
- `crates/contextra-graph/src/purger.rs:43` — Suchbegriff `purger`: `impl Purger for GraphPurger {`
- `crates/contextra-graph/src/purger.rs:80` — Suchbegriff `purger`: `async fn test_graph_purger_empty_target() {`
- `crates/contextra-graph/src/purger.rs:82` — Suchbegriff `purger`: `let purger = graph_purger(graph);`
- `crates/contextra-graph/src/purger.rs:86` — Suchbegriff `purger`: `let stats = purger.purge(&ctx, &target).expect("purge should succeed");`
- `crates/contextra-graph/src/purger.rs:92` — Suchbegriff `purger`: `async fn test_graph_purger_existing_edges() {`
- `crates/contextra-graph/src/purger.rs:117` — Suchbegriff `purger`: `let purger = graph_purger(graph.clone());`

#### `contextra-kvcache` (3 Stellen)
- `crates/contextra-kvcache/src/store.rs:790` — Suchbegriff `purger`: `use contextra_ports::scanner::Purger as LayerPurger;`
- `crates/contextra-kvcache/src/store.rs:805` — Suchbegriff `purger`: `impl LayerPurger for KvCachePruner {`
- `crates/contextra-kvcache/src/store.rs:1287` — Suchbegriff `purger`: `LayerScanner, Purger, ScanTarget, ScanTechnique, WitnessLayer,`

#### `contextra-ports` (11 Stellen)
- `crates/contextra-ports/src/erasure_legacy.rs:35` — Suchbegriff `purger`: `LayerScanner, LayerWitness, PurgeStats, Purger, ScanSession, ScanTarget, ScanTechnique,`
- `crates/contextra-ports/src/conformance.rs:26` — Suchbegriff `purger`: `use crate::scanner::layer::{Purger, ScanTarget};`
- `crates/contextra-ports/src/conformance.rs:174` — Suchbegriff `purger`: `/// Verifies that [`Purger`] methods can be invoked.`
- `crates/contextra-ports/src/conformance.rs:175` — Suchbegriff `purger`: `pub fn assert_purger_contract<T: Purger>(t: &T, ctx: &RequestContext, target: &ScanTarget) {`
- `crates/contextra-ports/src/lib.rs:73` — Suchbegriff `purger`: `/// Storage layer scanners and physical purgers (Vertrag V1, SPEC §12.7.4).`
- `crates/contextra-ports/src/lib.rs:116` — Suchbegriff `purger`: `LayerScanner, LayerWitness, PurgeStats, Purger, ScanSession, ScanTarget, ScanTechnique,`
- `crates/contextra-ports/src/lib.rs:152` — Suchbegriff `purger`: `fn _assert_dyn_purger(_: Option<&dyn Purger>) {}`
- `crates/contextra-ports/src/lib.rs:173` — Suchbegriff `purger`: `_assert_dyn_purger(None);`
- `crates/contextra-ports/src/scanner/layer.rs:2` — Suchbegriff `purger`: `// ZWECK: Definition der Schichten, Scantechniken, Target-Deskriptoren und Purger-Traits (Vertrag V1`
- `crates/contextra-ports/src/scanner/layer.rs:114` — Suchbegriff `purger`: `/// Trait für physische Bereinigungsaktionen (Purger) einer Schicht.`
- `crates/contextra-ports/src/scanner/layer.rs:115` — Suchbegriff `purger`: `pub trait Purger: Send + Sync {`

#### `contextra-store` (26 Stellen)
- `crates/contextra-store/src/scanner/purge.rs:6` — Suchbegriff `purger`: `// ZWECK: Physical Purger implementation für LSM-Storage-Schichten (§12.7.4, INV-WITNESS-MINT-1).`
- `crates/contextra-store/src/scanner/purge.rs:18` — Suchbegriff `purger`: `use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};`
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
- `crates/contextra-store/src/scanner/tests.rs:6` — Suchbegriff `purger`: `// ZWECK: Tests für LSM Scanner & Purger (Vertrag V1, PURGE-PIPELINE W1-P01).`

#### `contextra-text` (23 Stellen)
- `crates/contextra-text/src/inverted.rs:38` — Suchbegriff `purger`: `pub use purge::{text_purger, PurgeReport, TextPurger};`
- `crates/contextra-text/src/lib.rs:52` — Suchbegriff `purger`: `text_purger, text_scanner, BM25MorphIndex, InvertedIndex, Language, PurgeReport, TextPurger,`
- `crates/contextra-text/src/inverted/purge.rs:2` — Suchbegriff `purger`: `// FILE-CONTEXT: TextIndex Physical Purger & BM25 Corpus Statistics Recalculation (§12.7.4).`
- `crates/contextra-text/src/inverted/purge.rs:3` — Suchbegriff `purger`: `// ZWECK: Stellt den Purger für physical deletion per document and per collection bereit.`
- `crates/contextra-text/src/inverted/purge.rs:22` — Suchbegriff `purger`: `use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};`
- `crates/contextra-text/src/inverted/purge.rs:44` — Suchbegriff `purger`: `pub struct PurgeReport {`
- `crates/contextra-text/src/inverted/purge.rs:55` — Suchbegriff `purger`: `impl PurgeReport {`
- `crates/contextra-text/src/inverted/purge.rs:90` — Suchbegriff `purger`: `impl Default for PurgeReport {`
- `crates/contextra-text/src/inverted/purge.rs:97` — Suchbegriff `purger`: `/// Physischer Purger für den Textindex zur Bereinigung von Storage und Metadaten.`
- `crates/contextra-text/src/inverted/purge.rs:102` — Suchbegriff `purger`: `pub struct TextPurger<S: StorageEngine + 'static> {`
- `crates/contextra-text/src/inverted/purge.rs:106` — Suchbegriff `purger`: `impl<S: StorageEngine + 'static> TextPurger<S> {`
- `crates/contextra-text/src/inverted/purge.rs:107` — Suchbegriff `purger`: `/// Creates a new `TextPurger` for the given index.`
- `crates/contextra-text/src/inverted/purge.rs:113` — Suchbegriff `purger`: `/// returning a detailed `PurgeReport`.`
- `crates/contextra-text/src/inverted/purge.rs:114` — Suchbegriff `purger`: `pub fn purge_document_report(&self, tx: TxId, doc_id: DocId) -> PurgeReport {`
- `crates/contextra-text/src/inverted/purge.rs:126` — Suchbegriff `purger`: `fn purge_document_internal_report(&self, tx: TxId, doc_id: DocId) -> PurgeReport {`

#### `contextra-vector` (10 Stellen)
- `crates/contextra-vector/src/lib.rs:51` — Suchbegriff `purger`: `hnsw_purger, hnsw_scanner, HnswConfig, HnswIndex, HnswPurger, HnswScanner, RebuildStatus,`
- `crates/contextra-vector/src/hnsw/purge.rs:9` — Suchbegriff `purger`: `use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};`
- `crates/contextra-vector/src/hnsw/purge.rs:21` — Suchbegriff `purger`: `pub struct HnswPurger {`
- `crates/contextra-vector/src/hnsw/purge.rs:26` — Suchbegriff `purger`: `impl HnswPurger {`
- `crates/contextra-vector/src/hnsw/purge.rs:27` — Suchbegriff `purger`: `/// Erzeugt einen neuen `HnswPurger`.`
- `crates/contextra-vector/src/hnsw/purge.rs:33` — Suchbegriff `purger`: `/// Erzeugt einen `Arc<dyn Purger>` für einen HNSW Index.`
- `crates/contextra-vector/src/hnsw/purge.rs:38` — Suchbegriff `purger`: `pub fn hnsw_purger(index: Arc<HnswIndex>) -> Arc<dyn Purger> {`
- `crates/contextra-vector/src/hnsw/purge.rs:39` — Suchbegriff `purger`: `Arc::new(HnswPurger::new(index))`
- `crates/contextra-vector/src/hnsw/purge.rs:43` — Suchbegriff `purger`: `impl Purger for HnswPurger {`
- `crates/contextra-vector/src/hnsw/mod.rs:17` — Suchbegriff `purger`: `pub use purge::{hnsw_purger, HnswPurger};`
