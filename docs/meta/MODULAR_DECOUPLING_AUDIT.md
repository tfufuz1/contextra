# Contextra Modularisierungs- & Entkopplungs-Audit

> **Datum:** 2026-11-06 16:39:11 | **Analysierte Crates:** 36 | **Gesamtumfang:** 181,918 LOC

## 1. Strategische Zielsetzung & Architekturvision
Dieses Dokument dient als zentrale Orientierung für Architekten und Coding-Agenten zur Transformation des Contextra-Ökosystems von einem fest verdrahteten Monolithen zu einem **vollständig modularen Baukastensystem (Building Blocks)**.

### Kernprinzipien des Baukastensystems:
1. **Vollständige Entkopplung:** Jeder Crate muss isoliert verständlich, testbar und in Fremdprojekten nutzbar sein.
2. **Schnittstellen via Ports/Traits:** Direkte Abhängigkeiten zwischen konkreten Crates werden aufgelöst; Abstraktionen liegen in `contextra-ports` oder generischen Parametern.
3. **Isolation der Löschkette:** Alle DSGVO-, Gesetzes- und multi-layer Deletion-Proof-Logiken werden vollständig in `contextra-erasure` isoliert und aus Kernkomponenten (Store, Vector, Text, Crypto, DB) entfernt.
4. **Zero-Dead-Code & Saubere Fassaden:** Öffentliche Schnittstellen werden minimal gehalten und lecken keine internen Fremdtypen.

## 2. Gesamtübersicht des Baukastensystems

- **Durchschnittlicher Modularitäts-Score:** 58.1 / 100
- **Vollkommen autarke Bausteine:** 4 Crates
- **Dringender Entkopplungsbedarf (Score < 60):** 17 Crates
- **Verstreute Löschketten-/DSGVO-Relikte außerhalb `contextra-erasure`:** 664 Fundstellen
- **Identifizierte Dead-Code-Kandidaten (ungenutzte interne Definitionen):** 284 Stellen
- **Fehlende Port-Abstraktionen (direkte Kernel-Importe):** 307 Stellen

| Crate | Modularitäts-Score | Status | LOC | Dateien | Interne Deps | Schnittstellen (T/S/E) | Pub/Gesamt Fns | Dead Code | DSGVO/Lösch-Relikte |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| `contextra-engine` | **5/100** | Stark verdrahtet (Refactoring nötig) | 17,573 | 82 | 14 | 5/64/10 | 331/665 | 10 | 57 |
| `contextra-db` | **14/100** | Stark verdrahtet (Refactoring nötig) | 1,713 | 4 | 13 | 0/13/2 | 16/74 | 2 | 14 |
| `contextra` | **15/100** | Stark verdrahtet (Refactoring nötig) | 1,084 | 6 | 11 | 1/10/5 | 40/80 | 1 | 24 |
| `contextra-vector` | **17/100** | Stark verdrahtet (Refactoring nötig) | 11,720 | 43 | 6 | 2/47/5 | 246/432 | 4 | 12 |
| `contextra-mcp` | **17/100** | Stark verdrahtet (Refactoring nötig) | 5,276 | 24 | 12 | 0/29/5 | 81/231 | 7 | 48 |
| `contextra-store` | **29/100** | Stark verdrahtet (Refactoring nötig) | 23,040 | 77 | 5 | 6/68/18 | 356/867 | 12 | 28 |
| `contextra-graph` | **37/100** | Stark verdrahtet (Refactoring nötig) | 16,585 | 71 | 3 | 8/91/7 | 268/665 | 33 | 5 |
| `contextra-agent` | **39/100** | Stark verdrahtet (Refactoring nötig) | 4,220 | 18 | 7 | 4/30/6 | 99/213 | 2 | 2 |
| `contextra-cognition` | **42/100** | Stark verdrahtet (Refactoring nötig) | 5,244 | 19 | 6 | 1/42/1 | 72/218 | 4 | 0 |
| `contextra-text` | **43/100** | Stark verdrahtet (Refactoring nötig) | 5,828 | 26 | 2 | 3/36/3 | 107/363 | 30 | 29 |
| `contextra-kvcache` | **46/100** | Stark verdrahtet (Refactoring nötig) | 4,174 | 11 | 3 | 3/38/4 | 126/244 | 3 | 5 |
| `contextra-router` | **47/100** | Stark verdrahtet (Refactoring nötig) | 5,601 | 28 | 5 | 0/21/6 | 60/211 | 16 | 0 |
| `contextra-ports` | **49/100** | Stark verdrahtet (Refactoring nötig) | 3,333 | 27 | 1 | 33/46/8 | 40/384 | 46 | 9 |
| `contextra-privacy` | **52/100** | Stark verdrahtet (Refactoring nötig) | 3,703 | 12 | 3 | 6/33/9 | 91/207 | 1 | 44 |
| `contextra-crypto` | **53/100** | Stark verdrahtet (Refactoring nötig) | 5,725 | 23 | 2 | 3/42/14 | 170/353 | 11 | 106 |
| `contextra-mvcc` | **55/100** | Stark verdrahtet (Refactoring nötig) | 2,767 | 5 | 2 | 2/12/3 | 115/205 | 7 | 7 |
| `contextra-audit-export` | **58/100** | Stark verdrahtet (Refactoring nötig) | 557 | 5 | 2 | 1/8/1 | 14/19 | 0 | 31 |
| `contextra-py` | **64/100** | Akzeptabel | 2,246 | 16 | 6 | 0/8/0 | 61/161 | 0 | 0 |
| `contextra-durable-fs` | **64/100** | Akzeptabel | 860 | 6 | 1 | 3/11/6 | 32/81 | 0 | 7 |
| `contextra-avv-generator` | **64/100** | Akzeptabel | 508 | 9 | 1 | 0/2/1 | 16/23 | 0 | 226 |
| `contextra-erasure` | **65/100** | Akzeptabel | 7,307 | 57 | 3 | 10/76/22 | 335/415 | 4 | 0 |
| `contextra-testkit` | **65/100** | Akzeptabel | 2,318 | 19 | 2 | 0/11/6 | 47/121 | 6 | 3 |
| `contextra-core` | **66/100** | Akzeptabel | 106 | 1 | 4 | 0/0/0 | 0/0 | 0 | 0 |
| `contextra-checkpoint` | **68/100** | Akzeptabel | 3,475 | 11 | 3 | 2/21/0 | 99/267 | 18 | 1 |
| `contextra-infer-candle` | **69/100** | Akzeptabel | 4,818 | 17 | 4 | 2/32/2 | 70/195 | 4 | 0 |
| `contextra-types` | **70/100** | Akzeptabel | 5,068 | 22 | 0 | 1/39/21 | 124/309 | 20 | 4 |
| `contextra-ingest` | **73/100** | Akzeptabel | 16,257 | 77 | 2 | 2/90/20 | 130/509 | 30 | 1 |
| `contextra-infer-ollama` | **74/100** | Akzeptabel | 4,944 | 16 | 3 | 1/33/1 | 68/204 | 6 | 0 |
| `contextra-infer-onnx` | **81/100** | Exzellent (Baustein) | 1,905 | 6 | 3 | 0/10/1 | 42/97 | 3 | 0 |
| `contextra-rank` | **87/100** | Exzellent (Baustein) | 3,233 | 20 | 2 | 1/25/8 | 97/155 | 2 | 0 |
| `contextra-adapt` | **88/100** | Exzellent (Baustein) | 3,578 | 13 | 2 | 7/34/7 | 99/227 | 1 | 0 |
| `contextra-license` | **88/100** | Exzellent (Baustein) | 502 | 2 | 2 | 0/5/1 | 33/37 | 1 | 0 |
| `contextra-simd` | **94/100** | Exzellent (Baustein) | 1,653 | 7 | 1 | 0/3/1 | 53/68 | 0 | 0 |
| `contextra-wire` | **95/100** | Exzellent (Baustein) | 2,038 | 6 | 0 | 0/31/10 | 116/180 | 0 | 1 |
| `contextra-sandbox` | **100/100** | Exzellent (Baustein) | 1,892 | 13 | 0 | 0/11/4 | 26/59 | 0 | 0 |
| `contextra-sys` | **100/100** | Exzellent (Baustein) | 1,067 | 7 | 0 | 0/3/1 | 28/62 | 0 | 0 |

## 3. Priorisierte Entkopplungs-Roadmap für Coding-Agenten

### 📦 Crate: `contextra` (Score: 15/100)
- **Umfang:** 1,084 LOC (6 Quelldateien)
- **Abhängigkeiten (11):** `contextra-audit-export, contextra-avv-generator, contextra-crypto, contextra-db, contextra-infer-candle, contextra-license, contextra-ports, contextra-privacy, contextra-rank, contextra-store, contextra-types`
- **Funktionsinventar:** 40 pub-Funktionen von 80 gesamt
- **Schnittstellen:** 1 Traits, 10 Structs, 5 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-db`: 14 Symbole
  - `contextra-ports`: 9 Symbole
  - `contextra-types`: 10 Symbole
  - `contextra-store`: 7 Symbole
  - `contextra-rank`: 2 Symbole
  - `contextra-infer-candle`: 2 Symbole
  - `contextra-license`: 1 Symbole
- **⚠️ Direkte Kernel-Verdrahtung:** 7 Importe müssen über `contextra-ports` entkoppelt werden.
- **⚠️ Dead Code:** 1 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 24 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-db' (14 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (9 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (10 importierte Symbole).
  - 7 Stellen mit direkten Importen aus konkreten Speicherkernen (fehlende Port-/Trait-Abstraktion).
  - 24 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
  - 1 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-db' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Direkte Importe aus konkreten Speicher-Crates durch Ports in 'contextra-ports' ersetzen.
  - [ ] DSGVO-/Löschlogik aus contextra extrahieren und in 'contextra-erasure' modularisieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] rewrite` (crates/contextra/src/query_rewriter.rs:85)

### 📦 Crate: `contextra-agent` (Score: 39/100)
- **Umfang:** 4,220 LOC (18 Quelldateien)
- **Abhängigkeiten (7):** `contextra-checkpoint, contextra-db, contextra-graph, contextra-ports, contextra-router, contextra-store, contextra-types`
- **Funktionsinventar:** 99 pub-Funktionen von 213 gesamt
- **Schnittstellen:** 4 Traits, 30 Structs, 6 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ports`: 4 Symbole
  - `contextra-types`: 15 Symbole
  - `contextra-checkpoint`: 4 Symbole
  - `contextra-store`: 1 Symbole
  - `contextra-db`: 10 Symbole
  - `contextra-graph`: 1 Symbole
- **⚠️ Direkte Kernel-Verdrahtung:** 3 Importe müssen über `contextra-ports` entkoppelt werden.
- **⚠️ Dead Code:** 2 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 2 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-types' (15 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-db' (10 importierte Symbole).
  - 3 Stellen mit direkten Importen aus konkreten Speicherkernen (fehlende Port-/Trait-Abstraktion).
  - 2 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
  - 2 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-db' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Direkte Importe aus konkreten Speicher-Crates durch Ports in 'contextra-ports' ersetzen.
  - [ ] DSGVO-/Löschlogik aus contextra-agent extrahieren und in 'contextra-erasure' modularisieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] load_checkpoint` (crates/contextra-agent/src/engine.rs:91)
  - `[fn] take_snapshot` (crates/contextra-agent/src/engine.rs:104)

### 📦 Crate: `contextra-audit-export` (Score: 58/100)
- **Umfang:** 557 LOC (5 Quelldateien)
- **Abhängigkeiten (2):** `contextra-ports, contextra-types`
- **Funktionsinventar:** 14 pub-Funktionen von 19 gesamt
- **Schnittstellen:** 1 Traits, 8 Structs, 1 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 1 Symbole
- **⚠️ DSGVO-/Löschketten-Lecks:** 31 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - 31 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] DSGVO-/Löschlogik aus contextra-audit-export extrahieren und in 'contextra-erasure' modularisieren.

### 📦 Crate: `contextra-cognition` (Score: 42/100)
- **Umfang:** 5,244 LOC (19 Quelldateien)
- **Abhängigkeiten (6):** `contextra-engine, contextra-graph, contextra-ports, contextra-store, contextra-types, contextra-vector`
- **Funktionsinventar:** 72 pub-Funktionen von 218 gesamt
- **Schnittstellen:** 1 Traits, 42 Structs, 1 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-engine`: 10 Symbole
  - `contextra-ports`: 16 Symbole
  - `contextra-types`: 9 Symbole
  - `contextra-graph`: 14 Symbole
  - `contextra-store`: 2 Symbole
  - `contextra-vector`: 2 Symbole
- **⚠️ Direkte Kernel-Verdrahtung:** 8 Importe müssen über `contextra-ports` entkoppelt werden.
- **⚠️ Dead Code:** 4 ungenutzte interne Definitionen identifiziert.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-engine' (10 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (16 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (9 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-graph' (14 importierte Symbole).
  - 8 Stellen mit direkten Importen aus konkreten Speicherkernen (fehlende Port-/Trait-Abstraktion).
  - 4 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-engine' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-graph' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Direkte Importe aus konkreten Speicher-Crates durch Ports in 'contextra-ports' ersetzen.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] write_tick_intent` (crates/contextra-cognition/src/maintenance_scheduler.rs:331)
  - `[fn] complete_tick_intent` (crates/contextra-cognition/src/maintenance_scheduler.rs:355)
  - `[fn] monotonic_nanos` (crates/contextra-cognition/src/maintenance_scheduler.rs:548)
  - `[fn] context_manager_respects_token_budget` (crates/contextra-cognition/src/context.rs:301)

### 📦 Crate: `contextra-crypto` (Score: 53/100)
- **Umfang:** 5,725 LOC (23 Quelldateien)
- **Abhängigkeiten (2):** `contextra-ports, contextra-types`
- **Funktionsinventar:** 170 pub-Funktionen von 353 gesamt
- **Schnittstellen:** 3 Traits, 42 Structs, 14 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 8 Symbole
  - `contextra-ports`: 7 Symbole
- **⚠️ Dead Code:** 11 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 106 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - 106 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
  - 11 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] DSGVO-/Löschlogik aus contextra-crypto extrahieren und in 'contextra-erasure' modularisieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] file_keys_are_distinct_per_path` (crates/contextra-crypto/src/crypto.rs:523)
  - `[fn] same_path_same_passphrase_same_key` (crates/contextra-crypto/src/crypto.rs:535)
  - `[fn] same_passphrase_salt_path_gives_same_key` (crates/contextra-crypto/src/crypto.rs:548)
  - `[fn] key_manager_field_zeroizes_on_drop` (crates/contextra-crypto/src/crypto.rs:563)
  - `[fn] key_manager_debug_redacts_key` (crates/contextra-crypto/src/crypto.rs:571)

### 📦 Crate: `contextra-db` (Score: 14/100)
- **Umfang:** 1,713 LOC (4 Quelldateien)
- **Abhängigkeiten (13):** `contextra-adapt, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-engine, contextra-graph, contextra-ports, contextra-rank, contextra-store, contextra-sys, contextra-text, contextra-types, contextra-vector`
- **Funktionsinventar:** 16 pub-Funktionen von 74 gesamt
- **Schnittstellen:** 0 Traits, 13 Structs, 2 Enums
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
- **⚠️ Direkte Kernel-Verdrahtung:** 4 Importe müssen über `contextra-ports` entkoppelt werden.
- **⚠️ Dead Code:** 2 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 14 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-ports' (9 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-engine' (37 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-cognition' (38 importierte Symbole).
  - 4 Stellen mit direkten Importen aus konkreten Speicherkernen (fehlende Port-/Trait-Abstraktion).
  - 14 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
  - 2 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-engine' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-cognition' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Direkte Importe aus konkreten Speicher-Crates durch Ports in 'contextra-ports' ersetzen.
  - [ ] DSGVO-/Löschlogik aus contextra-db extrahieren und in 'contextra-erasure' modularisieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] overall_drift_status` (crates/contextra-db/src/lib.rs:458)
  - `[fn] collections_are_isolated` (crates/contextra-db/src/lib.rs:591)

### 📦 Crate: `contextra-engine` (Score: 5/100)
- **Umfang:** 17,573 LOC (82 Quelldateien)
- **Abhängigkeiten (14):** `contextra-adapt, contextra-checkpoint, contextra-crypto, contextra-graph, contextra-infer-onnx, contextra-mvcc, contextra-ports, contextra-rank, contextra-sandbox, contextra-store, contextra-sys, contextra-text, contextra-types, contextra-vector`
- **Funktionsinventar:** 331 pub-Funktionen von 665 gesamt
- **Schnittstellen:** 5 Traits, 64 Structs, 10 Enums
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
- **⚠️ Direkte Kernel-Verdrahtung:** 196 Importe müssen über `contextra-ports` entkoppelt werden.
- **⚠️ Dead Code:** 10 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 57 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-rank' (19 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (27 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (27 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-graph' (10 importierte Symbole).
  - 196 Stellen mit direkten Importen aus konkreten Speicherkernen (fehlende Port-/Trait-Abstraktion).
  - 57 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
  - 10 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-rank' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-graph' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Direkte Importe aus konkreten Speicher-Crates durch Ports in 'contextra-ports' ersetzen.
  - [ ] DSGVO-/Löschlogik aus contextra-engine extrahieren und in 'contextra-erasure' modularisieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] prop_sorted_unique_shards_properties` (crates/contextra-engine/src/collection/kv_lock.rs:130)
  - `[fn] prop_markdown_chunker_never_panics_on_arbitrary_utf8` (crates/contextra-engine/src/collection/tests/misc_tests.rs:599)
  - `[fn] hybrid_search_caps_k_at_max_search_k` (crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:11)
  - `[fn] trigger_expiry_cleanup_deletes_expired_documents` (crates/contextra-engine/src/background_workers/tests.rs:503)
  - `[fn] chunker_empty_input` (crates/contextra-engine/src/chunker/unit_tests.rs:38)

### 📦 Crate: `contextra-graph` (Score: 37/100)
- **Umfang:** 16,585 LOC (71 Quelldateien)
- **Abhängigkeiten (3):** `contextra-adapt, contextra-ports, contextra-types`
- **Funktionsinventar:** 268 pub-Funktionen von 665 gesamt
- **Schnittstellen:** 8 Traits, 91 Structs, 7 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 11 Symbole
  - `contextra-ports`: 11 Symbole
  - `contextra-adapt`: 2 Symbole
- **⚠️ Dead Code:** 33 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 5 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-types' (11 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (11 importierte Symbole).
  - 5 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
  - 33 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-graph extrahieren und in 'contextra-erasure' modularisieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] session_dag_survives_restart` (crates/contextra-graph/src/session_dag.rs:655)
  - `[fn] append_step_CASE_oversized_prompt_returns_invalid_input` (crates/contextra-graph/src/session_dag.rs:725)
  - `[fn] get_node_CASE_existent_and_nonexistent` (crates/contextra-graph/src/session_dag.rs:737)
  - `[fn] children_of_CASE_leaf_node_returns_empty` (crates/contextra-graph/src/session_dag.rs:755)
  - `[fn] new_CASE_empty_and_unicode_strings` (crates/contextra-graph/src/session_dag.rs:774)

### 📦 Crate: `contextra-kvcache` (Score: 46/100)
- **Umfang:** 4,174 LOC (11 Quelldateien)
- **Abhängigkeiten (3):** `contextra-crypto, contextra-ports, contextra-types`
- **Funktionsinventar:** 126 pub-Funktionen von 244 gesamt
- **Schnittstellen:** 3 Traits, 38 Structs, 4 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 11 Symbole
  - `contextra-ports`: 26 Symbole
  - `contextra-crypto`: 5 Symbole
- **⚠️ Direkte Kernel-Verdrahtung:** 8 Importe müssen über `contextra-ports` entkoppelt werden.
- **⚠️ Dead Code:** 3 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 5 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-types' (11 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (26 importierte Symbole).
  - 8 Stellen mit direkten Importen aus konkreten Speicherkernen (fehlende Port-/Trait-Abstraktion).
  - 5 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
  - 3 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Direkte Importe aus konkreten Speicher-Crates durch Ports in 'contextra-ports' ersetzen.
  - [ ] DSGVO-/Löschlogik aus contextra-kvcache extrahieren und in 'contextra-erasure' modularisieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] prop_compression_roundtrip` (crates/contextra-kvcache/src/quantize_kivi.rs:836)
  - `[fn] requires` (crates/contextra-kvcache/src/adapters.rs:181)
  - `[fn] conflicts` (crates/contextra-kvcache/src/adapters.rs:185)

### 📦 Crate: `contextra-mcp` (Score: 17/100)
- **Umfang:** 5,276 LOC (24 Quelldateien)
- **Abhängigkeiten (12):** `contextra, contextra-crypto, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-license, contextra-ports, contextra-privacy, contextra-rank, contextra-router, contextra-types, contextra-wire`
- **Funktionsinventar:** 81 pub-Funktionen von 231 gesamt
- **Schnittstellen:** 0 Traits, 29 Structs, 5 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra`: 43 Symbole
  - `contextra-privacy`: 8 Symbole
  - `contextra-wire`: 3 Symbole
  - `contextra-types`: 9 Symbole
  - `contextra-ports`: 5 Symbole
  - `contextra-rank`: 2 Symbole
- **⚠️ Dead Code:** 7 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 48 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra' (43 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (9 importierte Symbole).
  - 48 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
  - 7 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-mcp extrahieren und in 'contextra-erasure' modularisieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] search_text` (crates/contextra-mcp/src/egress_guard.rs:14)
  - `[fn] search_hybrid` (crates/contextra-mcp/src/routing.rs:9)
  - `[fn] get_community` (crates/contextra-mcp/src/routing.rs:39)
  - `[fn] prepare_context` (crates/contextra-mcp/src/routing.rs:50)
  - `[fn] volatile_tool_result_roundtrip` (crates/contextra-mcp/src/sandbox.rs:542)

### 📦 Crate: `contextra-mvcc` (Score: 55/100)
- **Umfang:** 2,767 LOC (5 Quelldateien)
- **Abhängigkeiten (2):** `contextra-ports, contextra-types`
- **Funktionsinventar:** 115 pub-Funktionen von 205 gesamt
- **Schnittstellen:** 2 Traits, 12 Structs, 3 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ports`: 2 Symbole
- **⚠️ Dead Code:** 7 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 7 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - 7 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
  - 7 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] DSGVO-/Löschlogik aus contextra-mvcc extrahieren und in 'contextra-erasure' modularisieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] prop_tx_buffer_isolation` (crates/contextra-mvcc/src/tx_buffer.rs:1265)
  - `[fn] prop_tx_buffer_reap_is_complete` (crates/contextra-mvcc/src/tx_buffer.rs:1299)
  - `[fn] prop_tx_buffer_stage_drain_stage_lifecycle` (crates/contextra-mvcc/src/tx_buffer.rs:1323)
  - `[fn] prop_tx_buffer_partial_discard_isolation` (crates/contextra-mvcc/src/tx_buffer.rs:1372)
  - `[fn] prop_snapshot_registry_min_active` (crates/contextra-mvcc/src/snapshot.rs:463)

### 📦 Crate: `contextra-ports` (Score: 49/100)
- **Umfang:** 3,333 LOC (27 Quelldateien)
- **Abhängigkeiten (1):** `contextra-types`
- **Funktionsinventar:** 40 pub-Funktionen von 384 gesamt
- **Schnittstellen:** 33 Traits, 46 Structs, 8 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 19 Symbole
- **⚠️ Dead Code:** 46 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 9 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-types' (19 importierte Symbole).
  - 9 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
  - 46 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-ports extrahieren und in 'contextra-erasure' modularisieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] take_snapshot` (crates/contextra-ports/src/checkpoint.rs:10)
  - `[fn] restore` (crates/contextra-ports/src/checkpoint.rs:12)
  - `[fn] create_named_checkpoint` (crates/contextra-ports/src/checkpoint.rs:29)
  - `[fn] restore_named_checkpoint` (crates/contextra-ports/src/checkpoint.rs:39)
  - `[fn] drop_named_checkpoint` (crates/contextra-ports/src/checkpoint.rs:45)

### 📦 Crate: `contextra-privacy` (Score: 52/100)
- **Umfang:** 3,703 LOC (12 Quelldateien)
- **Abhängigkeiten (3):** `contextra-avv-generator, contextra-ports, contextra-types`
- **Funktionsinventar:** 91 pub-Funktionen von 207 gesamt
- **Schnittstellen:** 6 Traits, 33 Structs, 9 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ports`: 2 Symbole
  - `contextra-avv-generator`: 1 Symbole
  - `contextra-types`: 2 Symbole
- **⚠️ Dead Code:** 1 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 44 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - 44 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
  - 1 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] DSGVO-/Löschlogik aus contextra-privacy extrahieren und in 'contextra-erasure' modularisieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] default_version` (crates/contextra-privacy/src/processing_registry.rs:48)

### 📦 Crate: `contextra-router` (Score: 47/100)
- **Umfang:** 5,601 LOC (28 Quelldateien)
- **Abhängigkeiten (5):** `contextra-adapt, contextra-ports, contextra-privacy, contextra-types, contextra-wire`
- **Funktionsinventar:** 60 pub-Funktionen von 211 gesamt
- **Schnittstellen:** 0 Traits, 21 Structs, 6 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-adapt`: 17 Symbole
  - `contextra-types`: 10 Symbole
  - `contextra-wire`: 2 Symbole
  - `contextra-privacy`: 4 Symbole
  - `contextra-ports`: 17 Symbole
- **⚠️ Dead Code:** 16 ungenutzte interne Definitionen identifiziert.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-adapt' (17 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (10 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (17 importierte Symbole).
  - 16 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-adapt' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] now_unix_nanos` (crates/contextra-router/src/tests/fixtures.rs:35)
  - `[fn] get_at_seq` (crates/contextra-router/src/tests/fixtures.rs:50)
  - `[fn] delete` (crates/contextra-router/src/tests/fixtures.rs:60)
  - `[fn] rollback` (crates/contextra-router/src/tests/fixtures.rs:66)
  - `[fn] rollback_to_tx` (crates/contextra-router/src/tests/fixtures.rs:69)

### 📦 Crate: `contextra-store` (Score: 29/100)
- **Umfang:** 23,040 LOC (77 Quelldateien)
- **Abhängigkeiten (5):** `contextra-crypto, contextra-mvcc, contextra-ports, contextra-sys, contextra-types`
- **Funktionsinventar:** 356 pub-Funktionen von 867 gesamt
- **Schnittstellen:** 6 Traits, 68 Structs, 18 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 8 Symbole
  - `contextra-mvcc`: 5 Symbole
  - `contextra-ports`: 17 Symbole
  - `contextra-crypto`: 8 Symbole
- **⚠️ Direkte Kernel-Verdrahtung:** 29 Importe müssen über `contextra-ports` entkoppelt werden.
- **⚠️ Dead Code:** 12 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 28 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-ports' (17 importierte Symbole).
  - 29 Stellen mit direkten Importen aus konkreten Speicherkernen (fehlende Port-/Trait-Abstraktion).
  - 28 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
  - 12 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Direkte Importe aus konkreten Speicher-Crates durch Ports in 'contextra-ports' ersetzen.
  - [ ] DSGVO-/Löschlogik aus contextra-store extrahieren und in 'contextra-erasure' modularisieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] prop_lsm_scan_prefix_at_consistency` (crates/contextra-store/src/lsm/tests/scan_tests.rs:438)
  - `[fn] partial_cmp` (crates/contextra-store/src/compaction/engine.rs:473)
  - `[fn] prop_compaction_tombstone_masking_latest_operation_wins` (crates/contextra-store/src/compaction/tests/basic.rs:26)
  - `[fn] concurrent_flush_and_compact_is_safe` (crates/contextra-store/src/compaction/tests/advanced.rs:575)
  - `[fn] poll_read` (crates/contextra-store/src/wal/mod.rs:220)

### 📦 Crate: `contextra-text` (Score: 43/100)
- **Umfang:** 5,828 LOC (26 Quelldateien)
- **Abhängigkeiten (2):** `contextra-ports, contextra-types`
- **Funktionsinventar:** 107 pub-Funktionen von 363 gesamt
- **Schnittstellen:** 3 Traits, 36 Structs, 3 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ports`: 12 Symbole
  - `contextra-types`: 8 Symbole
- **⚠️ Dead Code:** 30 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 29 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-ports' (12 importierte Symbole).
  - 29 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
  - 30 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] DSGVO-/Löschlogik aus contextra-text extrahieren und in 'contextra-erasure' modularisieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] tokenizer_never_panics` (crates/contextra-text/src/tokenizer.rs:313)
  - `[fn] german_tokenizer_never_panics` (crates/contextra-text/src/tokenizer.rs:318)
  - `[fn] prop_high_density_multibyte_never_panics` (crates/contextra-text/src/tokenizer.rs:323)
  - `[fn] german_morph_tokenizer_case_default_constructor` (crates/contextra-text/src/tokenizer.rs:430)
  - `[fn] bm25_scorer_case_full_lifecycle` (crates/contextra-text/src/lib.rs:310)

### 📦 Crate: `contextra-vector` (Score: 17/100)
- **Umfang:** 11,720 LOC (43 Quelldateien)
- **Abhängigkeiten (6):** `contextra-crypto, contextra-mvcc, contextra-ports, contextra-simd, contextra-sys, contextra-types`
- **Funktionsinventar:** 246 pub-Funktionen von 432 gesamt
- **Schnittstellen:** 2 Traits, 47 Structs, 5 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 9 Symbole
  - `contextra-simd`: 18 Symbole
  - `contextra-ports`: 12 Symbole
  - `contextra-mvcc`: 2 Symbole
- **⚠️ Direkte Kernel-Verdrahtung:** 2 Importe müssen über `contextra-ports` entkoppelt werden.
- **⚠️ Dead Code:** 4 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 12 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-types' (9 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-simd' (18 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (12 importierte Symbole).
  - 2 Stellen mit direkten Importen aus konkreten Speicherkernen (fehlende Port-/Trait-Abstraktion).
  - 12 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören ausschließlich in 'contextra-erasure'.
  - 4 ungenutzte private Symbole/Dead-Code-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-types' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-simd' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Abhängigkeit von 'contextra-ports' über Traits in 'contextra-ports' entkoppeln oder als generische Typ-Parameter injizieren.
  - [ ] Direkte Importe aus konkreten Speicher-Crates durch Ports in 'contextra-ports' ersetzen.
  - [ ] DSGVO-/Löschlogik aus contextra-vector extrahieren und in 'contextra-erasure' modularisieren.
  - [ ] Dead Code bereinigen oder Funktionen sauber verdrahten/exportieren.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] prop_per_dim_mse_le_global_mse` (crates/contextra-vector/src/quantize/tests.rs:314)
  - `[fn] record_gauge` (crates/contextra-vector/src/hnsw/tests.rs:244)
  - `[fn] scan_units` (crates/contextra-vector/src/hnsw/scanner.rs:203)
  - `[fn] now_unix_nanos` (crates/contextra-vector/src/hnsw/core_rebuild.rs:998)

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
