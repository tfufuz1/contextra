# Contextra Modularisierungs- & Entkopplungs-Audit

> **Datum:** 2026-11-11 21:39:51 | **Analysierte Crates:** 39 | **Gesamtumfang:** 210,380 LOC

## 1. Strategische Zielsetzung & Architekturvision
Dieses Dokument dient als zentrale Orientierung für Architekten und Coding-Agenten zur Transformation des Contextra-Ökosystems von einem fest verdrahteten Monolithen zu einem **vollständig modularen Baukastensystem (Building Blocks)**.

### Kernprinzipien des Baukastensystems:
1. **Vollständige Entkopplung:** Jeder Crate muss isoliert verständlich, testbar und in Fremdprojekten nutzbar sein.
2. **Schnittstellen via Ports/Traits:** Direkte Abhängigkeiten zwischen konkreten Crates werden aufgelöst; Abstraktionen liegen in `contextra-ports` oder generischen Parametern.
3. **Isolation der Löschkette:** Alle DSGVO-, Gesetzes- und multi-layer Deletion-Proof-Logiken werden vollständig in `contextra-erasure` isoliert und aus Kernkomponenten (Store, Vector, Text, Crypto, DB) entfernt.
4. **Zero-Dead-Code & Saubere Fassaden:** Öffentliche Schnittstellen werden minimal gehalten und lecken keine internen Fremdtypen.

## 2. Gesamtübersicht des Baukastensystems

- **Vollkommen autarke Bausteine:** 10 Crates
- **Entkopplungs- / Bereinigungsbedarf:** 30 Crates
- **Verstreute Löschketten-/DSGVO-Relikte außerhalb `contextra-erasure`:** 426 Fundstellen
- **Identifizierte Dead-Code-Kandidaten (ungenutzte interne Definitionen):** 109 Stellen
- **Fehlende Port-Abstraktionen (direkte Kernel-Importe):** 234 Stellen

| Crate | Status | LOC | Dateien | Interne Deps | Schnittstellen (T/S/E) | Pub/Gesamt Fns | Dead Code | DSGVO/Lösch-Relikte |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| `contextra-engine` | **10/100** | Stark verdrahtet (Refactoring nötig) | 17,625 | 88 | 12 | 7/74/9 | 358/705 | 0 | 6 |
| `contextra-vector` | **23/100** | Stark verdrahtet (Refactoring nötig) | 14,872 | 60 | 7 | 16/64/10 | 328/634 | 4 | 19 |
| `contextra-store` | **25/100** | Stark verdrahtet (Refactoring nötig) | 25,794 | 103 | 7 | 14/90/21 | 492/1145 | 0 | 62 |
| `contextra-graph` | **30/100** | Stark verdrahtet (Refactoring nötig) | 17,835 | 90 | 4 | 14/111/10 | 305/813 | 12 | 25 |
| `contextra-text` | **40/100** | Stark verdrahtet (Refactoring nötig) | 7,110 | 38 | 4 | 10/48/7 | 136/442 | 2 | 24 |
| `contextra-mcp` | **48/100** | Stark verdrahtet (Refactoring nötig) | 3,909 | 18 | 12 | 0/22/2 | 55/170 | 5 | 0 |
| `contextra-kvcache` | **49/100** | Stark verdrahtet (Refactoring nötig) | 5,754 | 21 | 4 | 9/63/6 | 178/364 | 4 | 3 |
| `contextra-ports` | **53/100** | Stark verdrahtet (Refactoring nötig) | 4,227 | 42 | 2 | 43/66/20 | 80/475 | 1 | 21 |
| `contextra` | **56/100** | Stark verdrahtet (Refactoring nötig) | 1,182 | 7 | 4 | 1/10/9 | 42/85 | 1 | 2 |
| `contextra-router` | **58/100** | Stark verdrahtet (Refactoring nötig) | 6,111 | 30 | 4 | 1/30/6 | 93/251 | 6 | 0 |
| `contextra-agent` | **60/100** | Akzeptabel | 4,426 | 22 | 5 | 8/34/7 | 102/234 | 0 | 0 |
| `contextra-infer-candle` | **63/100** | Akzeptabel | 6,291 | 31 | 5 | 8/47/3 | 88/284 | 4 | 0 |
| `contextra-audit-export` | **64/100** | Akzeptabel | 747 | 8 | 1 | 2/9/1 | 16/31 | 1 | 33 |
| `contextra-db` | **65/100** | Akzeptabel | 1,161 | 7 | 5 | 0/9/0 | 22/56 | 0 | 0 |
| `contextra-cognition` | **70/100** | Akzeptabel | 5,415 | 26 | 4 | 7/46/2 | 77/236 | 3 | 1 |
| `contextra-avv-generator` | **70/100** | Akzeptabel | 550 | 9 | 0 | 0/3/1 | 17/26 | 0 | 229 |
| `contextra-checkpoint` | **71/100** | Akzeptabel | 4,017 | 20 | 4 | 6/26/1 | 112/335 | 10 | 0 |
| `contextra-crypto` | **76/100** | Akzeptabel | 4,479 | 22 | 4 | 4/36/4 | 135/318 | 0 | 0 |
| `contextra-mvcc` | **76/100** | Akzeptabel | 3,305 | 10 | 4 | 3/21/4 | 140/253 | 0 | 0 |
| `contextra-py` | **77/100** | Akzeptabel | 2,235 | 16 | 3 | 0/8/0 | 61/161 | 0 | 0 |
| `contextra-testkit` | **78/100** | Akzeptabel | 3,745 | 22 | 2 | 0/19/6 | 82/185 | 0 | 0 |
| `contextra-types` | **79/100** | Akzeptabel | 5,258 | 25 | 1 | 1/44/34 | 121/296 | 20 | 0 |
| `contextra-infer-ollama` | **81/100** | Exzellent (Baustein) | 5,633 | 25 | 2 | 8/43/3 | 87/272 | 4 | 0 |
| `contextra-license` | **82/100** | Exzellent (Baustein) | 711 | 6 | 2 | 2/11/3 | 37/62 | 2 | 1 |
| `contextra-ingest` | **84/100** | Exzellent (Baustein) | 17,015 | 80 | 1 | 2/95/20 | 149/532 | 29 | 0 |
| `contextra-rank` | **88/100** | Exzellent (Baustein) | 4,440 | 31 | 2 | 5/35/12 | 126/237 | 0 | 0 |
| `contextra-adapt` | **88/100** | Exzellent (Baustein) | 3,864 | 20 | 2 | 12/37/9 | 105/254 | 1 | 0 |
| `contextra-infer-onnx` | **88/100** | Exzellent (Baustein) | 744 | 9 | 2 | 2/6/4 | 15/37 | 0 | 0 |
| `contextra-vault` | **88/100** | Exzellent (Baustein) | 320 | 4 | 2 | 0/9/2 | 13/22 | 0 | 0 |
| `contextra-simd` | **94/100** | Exzellent (Baustein) | 1,942 | 11 | 1 | 1/5/3 | 65/88 | 0 | 0 |
| `contextra-erasure` | **100/100** | Exzellent (Baustein) | 16,291 | 82 | 0 | 39/199/86 | 596/1005 | 0 | 0 |
| `contextra-privacy` | **100/100** | Exzellent (Baustein) | 4,151 | 13 | 0 | 8/41/11 | 106/241 | 0 | 0 |
| `contextra-wire` | **100/100** | Exzellent (Baustein) | 2,131 | 6 | 0 | 1/33/11 | 121/185 | 0 | 0 |
| `contextra-sandbox` | **100/100** | Exzellent (Baustein) | 1,905 | 13 | 0 | 0/11/4 | 26/59 | 0 | 0 |
| `contextra-sys` | **100/100** | Exzellent (Baustein) | 1,564 | 9 | 0 | 1/11/2 | 53/105 | 0 | 0 |
| `contextra-durable-fs` | **100/100** | Exzellent (Baustein) | 1,428 | 8 | 0 | 2/13/12 | 56/112 | 0 | 0 |
| `contextra-prompt-guard` | **100/100** | Exzellent (Baustein) | 1,085 | 10 | 0 | 1/7/2 | 26/55 | 0 | 0 |
| `contextra-env` | **100/100** | Exzellent (Baustein) | 607 | 7 | 0 | 4/10/0 | 12/67 | 0 | 0 |
| `contextra-ids` | **100/100** | Exzellent (Baustein) | 501 | 1 | 0 | 0/12/2 | 18/39 | 0 | 0 |

## 3. Priorisierte Entkopplungs-Roadmap für Coding-Agenten

### 📦 Crate: `contextra` (Score: 56/100)
- **Umfang:** 1,182 LOC (7 Quelldateien)
- **Abhängigkeiten (4):** `contextra-db, contextra-infer-candle, contextra-ports, contextra-types`
- **Funktionsinventar:** 42 pub-Funktionen von 85 gesamt
- **Schnittstellen:** 1 Traits, 10 Structs, 9 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-db`: 13 Symbole
  - `contextra-ports`: 8 Symbole
  - `contextra-types`: 10 Symbole
  - `contextra-infer-candle`: 2 Symbole
- **⚠️ Dead Code:** 1 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 2 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-db' (13 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (10 importierte Symbole).
  - 2 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören in 'contextra-erasure' (SPEC §3.1 / INV-7).
  - 1 nicht verdrahtete private Symbole/Inventar-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-db' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Abhängigkeit von 'contextra-types' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Löschrelikte aus contextra entfernen und in 'contextra-erasure' modularisieren; Indizes bieten nur neutrale Maintenance-Methoden.
  - [ ] Inventarisieren vor Verschiebung (E-06/E-07): Kein Algorithmus wird gelöscht; hinter Feature 'experimental' sichern oder verdrahten.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] check_ring` (crates/contextra/src/performance_profile.rs:209)

### 📦 Crate: `contextra-adapt` (Score: 88/100)
- **Umfang:** 3,864 LOC (20 Quelldateien)
- **Abhängigkeiten (2):** `contextra-ports, contextra-types`
- **Funktionsinventar:** 105 pub-Funktionen von 254 gesamt
- **Schnittstellen:** 12 Traits, 37 Structs, 9 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 1 Symbole
- **⚠️ Dead Code:** 1 ungenutzte interne Definitionen identifiziert.
- **Befunde:**
  - 1 nicht verdrahtete private Symbole/Inventar-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Inventarisieren vor Verschiebung (E-06/E-07): Kein Algorithmus wird gelöscht; hinter Feature 'experimental' sichern oder verdrahten.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] next_f64` (crates/contextra-adapt/src/rng.rs:15)

### 📦 Crate: `contextra-agent` (Score: 60/100)
- **Umfang:** 4,426 LOC (22 Quelldateien)
- **Abhängigkeiten (5):** `contextra-checkpoint, contextra-db, contextra-ports, contextra-router, contextra-types`
- **Funktionsinventar:** 102 pub-Funktionen von 234 gesamt
- **Schnittstellen:** 8 Traits, 34 Structs, 7 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ports`: 5 Symbole
  - `contextra-types`: 15 Symbole
  - `contextra-checkpoint`: 4 Symbole
  - `contextra-db`: 9 Symbole
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-types' (15 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-db' (9 importierte Symbole).
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-types' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Abhängigkeit von 'contextra-db' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.

### 📦 Crate: `contextra-audit-export` (Score: 64/100)
- **Umfang:** 747 LOC (8 Quelldateien)
- **Abhängigkeiten (1):** `contextra-types`
- **Funktionsinventar:** 16 pub-Funktionen von 31 gesamt
- **Schnittstellen:** 2 Traits, 9 Structs, 1 Enums
- **⚠️ Dead Code:** 1 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 33 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - 33 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören in 'contextra-erasure' (SPEC §3.1 / INV-7).
  - 1 nicht verdrahtete private Symbole/Inventar-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Löschrelikte aus contextra-audit-export entfernen und in 'contextra-erasure' modularisieren; Indizes bieten nur neutrale Maintenance-Methoden.
  - [ ] Inventarisieren vor Verschiebung (E-06/E-07): Kein Algorithmus wird gelöscht; hinter Feature 'experimental' sichern oder verdrahten.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] collect_records` (crates/contextra-audit-export/src/lib.rs:341)

### 📦 Crate: `contextra-avv-generator` (Score: 70/100)
- **Umfang:** 550 LOC (9 Quelldateien)
- **Abhängigkeiten (0):** `keine`
- **Funktionsinventar:** 17 pub-Funktionen von 26 gesamt
- **Schnittstellen:** 0 Traits, 3 Structs, 1 Enums
- **⚠️ DSGVO-/Löschketten-Lecks:** 229 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - 229 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören in 'contextra-erasure' (SPEC §3.1 / INV-7).
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Löschrelikte aus contextra-avv-generator entfernen und in 'contextra-erasure' modularisieren; Indizes bieten nur neutrale Maintenance-Methoden.

### 📦 Crate: `contextra-checkpoint` (Score: 71/100)
- **Umfang:** 4,017 LOC (20 Quelldateien)
- **Abhängigkeiten (4):** `contextra-env, contextra-ids, contextra-ports, contextra-types`
- **Funktionsinventar:** 112 pub-Funktionen von 335 gesamt
- **Schnittstellen:** 6 Traits, 26 Structs, 1 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-env`: 6 Symbole
  - `contextra-ids`: 1 Symbole
  - `contextra-ports`: 2 Symbole
  - `contextra-types`: 4 Symbole
- **⚠️ Dead Code:** 10 ungenutzte interne Definitionen identifiziert.
- **Befunde:**
  - 10 nicht verdrahtete private Symbole/Inventar-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Inventarisieren vor Verschiebung (E-06/E-07): Kein Algorithmus wird gelöscht; hinter Feature 'experimental' sichern oder verdrahten.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] default_id_width_bits` (crates/contextra-checkpoint/src/meta.rs:94)
  - `[fn] get_at_seq` (crates/contextra-checkpoint/src/guard/tests.rs:30)
  - `[fn] stats` (crates/contextra-checkpoint/src/guard/tests.rs:75)
  - `[fn] scan` (crates/contextra-checkpoint/src/guard/tests.rs:109)
  - `[fn] create_named_checkpoint` (crates/contextra-checkpoint/src/compat/ports_impl.rs:15)

### 📦 Crate: `contextra-cognition` (Score: 70/100)
- **Umfang:** 5,415 LOC (26 Quelldateien)
- **Abhängigkeiten (4):** `contextra-engine, contextra-graph, contextra-ports, contextra-types`
- **Funktionsinventar:** 77 pub-Funktionen von 236 gesamt
- **Schnittstellen:** 7 Traits, 46 Structs, 2 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 5 Symbole
  - `contextra-engine`: 2 Symbole
  - `contextra-graph`: 2 Symbole
  - `contextra-ports`: 2 Symbole
- **⚠️ Dead Code:** 3 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 1 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - 1 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören in 'contextra-erasure' (SPEC §3.1 / INV-7).
  - 3 nicht verdrahtete private Symbole/Inventar-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Löschrelikte aus contextra-cognition entfernen und in 'contextra-erasure' modularisieren; Indizes bieten nur neutrale Maintenance-Methoden.
  - [ ] Inventarisieren vor Verschiebung (E-06/E-07): Kein Algorithmus wird gelöscht; hinter Feature 'experimental' sichern oder verdrahten.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] fetch_turns` (crates/contextra-cognition/src/ports.rs:42)
  - `[fn] apply_intent` (crates/contextra-cognition/src/ports.rs:53)
  - `[fn] compute_decay` (crates/contextra-cognition/src/ports.rs:76)

### 📦 Crate: `contextra-crypto` (Score: 76/100)
- **Umfang:** 4,479 LOC (22 Quelldateien)
- **Abhängigkeiten (4):** `contextra-env, contextra-ids, contextra-ports, contextra-types`
- **Funktionsinventar:** 135 pub-Funktionen von 318 gesamt
- **Schnittstellen:** 4 Traits, 36 Structs, 4 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ids`: 4 Symbole
  - `contextra-types`: 4 Symbole

### 📦 Crate: `contextra-db` (Score: 65/100)
- **Umfang:** 1,161 LOC (7 Quelldateien)
- **Abhängigkeiten (5):** `contextra-cognition, contextra-engine, contextra-ports, contextra-types, contextra-vault`
- **Funktionsinventar:** 22 pub-Funktionen von 56 gesamt
- **Schnittstellen:** 0 Traits, 9 Structs, 0 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ports`: 8 Symbole
  - `contextra-types`: 7 Symbole
  - `contextra-engine`: 37 Symbole
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-engine' (37 importierte Symbole).
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-engine' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.

### 📦 Crate: `contextra-engine` (Score: 10/100)
- **Umfang:** 17,625 LOC (88 Quelldateien)
- **Abhängigkeiten (12):** `contextra-adapt, contextra-checkpoint, contextra-graph, contextra-infer-onnx, contextra-mvcc, contextra-ports, contextra-rank, contextra-sandbox, contextra-store, contextra-text, contextra-types, contextra-vector`
- **Funktionsinventar:** 358 pub-Funktionen von 705 gesamt
- **Schnittstellen:** 7 Traits, 74 Structs, 9 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-rank`: 17 Symbole
  - `contextra-types`: 27 Symbole
  - `contextra-ports`: 21 Symbole
  - `contextra-graph`: 10 Symbole
  - `contextra-store`: 5 Symbole
  - `contextra-text`: 6 Symbole
  - `contextra-vector`: 4 Symbole
  - `contextra-sandbox`: 3 Symbole
  - `contextra-mvcc`: 3 Symbole
- **⚠️ Direkte Kernel-Verdrahtung:** 195 Importe müssen über `contextra-ports` entkoppelt werden.
- **⚠️ DSGVO-/Löschketten-Lecks:** 6 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-rank' (17 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (27 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (21 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-graph' (10 importierte Symbole).
  - 195 Stellen mit direkten Importen aus konkreten Speicherkernen.
  - 6 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören in 'contextra-erasure' (SPEC §3.1 / INV-7).
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-rank' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Abhängigkeit von 'contextra-types' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Abhängigkeit von 'contextra-ports' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Abhängigkeit von 'contextra-graph' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Direkte Kernel-Importe für Ebene E0/E1 vollständig eliminieren (autark); für Ebene E2 via Backends-Trait bündeln.
  - [ ] Löschrelikte aus contextra-engine entfernen und in 'contextra-erasure' modularisieren; Indizes bieten nur neutrale Maintenance-Methoden.

### 📦 Crate: `contextra-graph` (Score: 30/100)
- **Umfang:** 17,835 LOC (90 Quelldateien)
- **Abhängigkeiten (4):** `contextra-env, contextra-ids, contextra-ports, contextra-types`
- **Funktionsinventar:** 305 pub-Funktionen von 813 gesamt
- **Schnittstellen:** 14 Traits, 111 Structs, 10 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ports`: 20 Symbole
  - `contextra-types`: 13 Symbole
- **⚠️ Dead Code:** 12 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 25 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-ports' (20 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (13 importierte Symbole).
  - 25 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören in 'contextra-erasure' (SPEC §3.1 / INV-7).
  - 12 nicht verdrahtete private Symbole/Inventar-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-ports' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Abhängigkeit von 'contextra-types' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Löschrelikte aus contextra-graph entfernen und in 'contextra-erasure' modularisieren; Indizes bieten nur neutrale Maintenance-Methoden.
  - [ ] Inventarisieren vor Verschiebung (E-06/E-07): Kein Algorithmus wird gelöscht; hinter Feature 'experimental' sichern oder verdrahten.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] default_tl_hfd_sigma` (crates/contextra-graph/src/types.rs:240)
  - `[fn] default_tl_hfd_delta` (crates/contextra-graph/src/types.rs:242)
  - `[fn] default_tl_hfd_gamma` (crates/contextra-graph/src/types.rs:243)
  - `[fn] default_tl_hfd_max_iterations` (crates/contextra-graph/src/types.rs:244)
  - `[fn] default_tl_hfd_max_top_k_expansion` (crates/contextra-graph/src/types.rs:245)

### 📦 Crate: `contextra-infer-candle` (Score: 63/100)
- **Umfang:** 6,291 LOC (31 Quelldateien)
- **Abhängigkeiten (5):** `contextra-crypto, contextra-env, contextra-ports, contextra-rank, contextra-types`
- **Funktionsinventar:** 88 pub-Funktionen von 284 gesamt
- **Schnittstellen:** 8 Traits, 47 Structs, 3 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ports`: 25 Symbole
  - `contextra-types`: 8 Symbole
  - `contextra-crypto`: 6 Symbole
  - `contextra-rank`: 1 Symbole
- **⚠️ Direkte Kernel-Verdrahtung:** 6 Importe müssen über `contextra-ports` entkoppelt werden.
- **⚠️ Dead Code:** 4 ungenutzte interne Definitionen identifiziert.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-ports' (25 importierte Symbole).
  - 6 Stellen mit direkten Importen aus konkreten Speicherkernen.
  - 4 nicht verdrahtete private Symbole/Inventar-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-ports' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Direkte Kernel-Importe für Ebene E0/E1 vollständig eliminieren (autark); für Ebene E2 via Backends-Trait bündeln.
  - [ ] Inventarisieren vor Verschiebung (E-06/E-07): Kein Algorithmus wird gelöscht; hinter Feature 'experimental' sichern oder verdrahten.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] load_segment` (crates/contextra-infer-candle/src/traits.rs:70)
  - `[fn] export_attention_weights` (crates/contextra-infer-candle/src/compat/attention.rs:7)
  - `[fn] provider_name` (crates/contextra-infer-candle/src/compat/embedding.rs:20)
  - `[fn] score_grounding` (crates/contextra-infer-candle/src/compat/grounding.rs:21)

### 📦 Crate: `contextra-infer-ollama` (Score: 81/100)
- **Umfang:** 5,633 LOC (25 Quelldateien)
- **Abhängigkeiten (2):** `contextra-ports, contextra-types`
- **Funktionsinventar:** 87 pub-Funktionen von 272 gesamt
- **Schnittstellen:** 8 Traits, 43 Structs, 3 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ports`: 10 Symbole
  - `contextra-types`: 3 Symbole
- **⚠️ Dead Code:** 4 ungenutzte interne Definitionen identifiziert.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-ports' (10 importierte Symbole).
  - 4 nicht verdrahtete private Symbole/Inventar-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-ports' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Inventarisieren vor Verschiebung (E-06/E-07): Kein Algorithmus wird gelöscht; hinter Feature 'experimental' sichern oder verdrahten.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] next_f64` (crates/contextra-infer-ollama/src/error.rs:86)
  - `[fn] model_name` (crates/contextra-infer-ollama/src/traits.rs:27)
  - `[fn] embed_async` (crates/contextra-infer-ollama/src/traits.rs:29)
  - `[fn] embed_batch_async` (crates/contextra-infer-ollama/src/traits.rs:32)

### 📦 Crate: `contextra-infer-onnx` (Score: 88/100)
- **Umfang:** 744 LOC (9 Quelldateien)
- **Abhängigkeiten (2):** `contextra-ports, contextra-types`
- **Funktionsinventar:** 15 pub-Funktionen von 37 gesamt
- **Schnittstellen:** 2 Traits, 6 Structs, 4 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ports`: 3 Symbole

### 📦 Crate: `contextra-ingest` (Score: 84/100)
- **Umfang:** 17,015 LOC (80 Quelldateien)
- **Abhängigkeiten (1):** `contextra-types`
- **Funktionsinventar:** 149 pub-Funktionen von 532 gesamt
- **Schnittstellen:** 2 Traits, 95 Structs, 20 Enums
- **⚠️ Dead Code:** 29 ungenutzte interne Definitionen identifiziert.
- **Befunde:**
  - 29 nicht verdrahtete private Symbole/Inventar-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Inventarisieren vor Verschiebung (E-06/E-07): Kein Algorithmus wird gelöscht; hinter Feature 'experimental' sichern oder verdrahten.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] parse_error` (crates/contextra-ingest/src/formats/web/html.rs:110)
  - `[fn] get_document` (crates/contextra-ingest/src/formats/web/html.rs:112)
  - `[fn] set_quirks_mode` (crates/contextra-ingest/src/formats/web/html.rs:116)
  - `[fn] same_node` (crates/contextra-ingest/src/formats/web/html.rs:120)
  - `[fn] elem_name` (crates/contextra-ingest/src/formats/web/html.rs:124)

### 📦 Crate: `contextra-kvcache` (Score: 49/100)
- **Umfang:** 5,754 LOC (21 Quelldateien)
- **Abhängigkeiten (4):** `contextra-crypto, contextra-env, contextra-ports, contextra-types`
- **Funktionsinventar:** 178 pub-Funktionen von 364 gesamt
- **Schnittstellen:** 9 Traits, 63 Structs, 6 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-env`: 2 Symbole
  - `contextra-crypto`: 6 Symbole
  - `contextra-ports`: 32 Symbole
  - `contextra-types`: 10 Symbole
- **⚠️ Direkte Kernel-Verdrahtung:** 6 Importe müssen über `contextra-ports` entkoppelt werden.
- **⚠️ Dead Code:** 4 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 3 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-ports' (32 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (10 importierte Symbole).
  - 6 Stellen mit direkten Importen aus konkreten Speicherkernen.
  - 3 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören in 'contextra-erasure' (SPEC §3.1 / INV-7).
  - 4 nicht verdrahtete private Symbole/Inventar-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-ports' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Abhängigkeit von 'contextra-types' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Direkte Kernel-Importe für Ebene E0/E1 vollständig eliminieren (autark); für Ebene E2 via Backends-Trait bündeln.
  - [ ] Löschrelikte aus contextra-kvcache entfernen und in 'contextra-erasure' modularisieren; Indizes bieten nur neutrale Maintenance-Methoden.
  - [ ] Inventarisieren vor Verschiebung (E-06/E-07): Kein Algorithmus wird gelöscht; hinter Feature 'experimental' sichern oder verdrahten.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] next_u64` (crates/contextra-kvcache/src/contract.rs:29)
  - `[fn] requires` (crates/contextra-kvcache/src/compat/adapters.rs:180)
  - `[fn] conflicts` (crates/contextra-kvcache/src/compat/adapters.rs:184)
  - `[fn] scan_units` (crates/contextra-kvcache/src/compat/scanner.rs:72)

### 📦 Crate: `contextra-license` (Score: 82/100)
- **Umfang:** 711 LOC (6 Quelldateien)
- **Abhängigkeiten (2):** `contextra-ports, contextra-types`
- **Funktionsinventar:** 37 pub-Funktionen von 62 gesamt
- **Schnittstellen:** 2 Traits, 11 Structs, 3 Enums
- **⚠️ Dead Code:** 2 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 1 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - 1 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören in 'contextra-erasure' (SPEC §3.1 / INV-7).
  - 2 nicht verdrahtete private Symbole/Inventar-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Löschrelikte aus contextra-license entfernen und in 'contextra-erasure' modularisieren; Indizes bieten nur neutrale Maintenance-Methoden.
  - [ ] Inventarisieren vor Verschiebung (E-06/E-07): Kein Algorithmus wird gelöscht; hinter Feature 'experimental' sichern oder verdrahten.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] monotonic_nanos` (crates/contextra-license/src/types.rs:125)
  - `[fn] authorize` (crates/contextra-license/src/types.rs:182)

### 📦 Crate: `contextra-mcp` (Score: 48/100)
- **Umfang:** 3,909 LOC (18 Quelldateien)
- **Abhängigkeiten (12):** `contextra, contextra-crypto, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ports, contextra-privacy, contextra-prompt-guard, contextra-rank, contextra-router, contextra-types, contextra-wire`
- **Funktionsinventar:** 55 pub-Funktionen von 170 gesamt
- **Schnittstellen:** 0 Traits, 22 Structs, 2 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra`: 53 Symbole
  - `contextra-privacy`: 8 Symbole
  - `contextra-wire`: 3 Symbole
  - `contextra-types`: 8 Symbole
  - `contextra-ports`: 3 Symbole
  - `contextra-prompt-guard`: 15 Symbole
  - `contextra-rank`: 2 Symbole
- **⚠️ Dead Code:** 5 ungenutzte interne Definitionen identifiziert.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra' (53 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-prompt-guard' (15 importierte Symbole).
  - 5 nicht verdrahtete private Symbole/Inventar-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Abhängigkeit von 'contextra-prompt-guard' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Inventarisieren vor Verschiebung (E-06/E-07): Kein Algorithmus wird gelöscht; hinter Feature 'experimental' sichern oder verdrahten.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] search_text` (crates/contextra-mcp/src/egress_guard.rs:18)
  - `[fn] search_hybrid` (crates/contextra-mcp/src/routing.rs:13)
  - `[fn] get_community` (crates/contextra-mcp/src/routing.rs:43)
  - `[fn] prepare_context` (crates/contextra-mcp/src/routing.rs:54)
  - `[fn] default_collection` (crates/contextra-mcp/src/explain.rs:21)

### 📦 Crate: `contextra-mvcc` (Score: 76/100)
- **Umfang:** 3,305 LOC (10 Quelldateien)
- **Abhängigkeiten (4):** `contextra-env, contextra-ids, contextra-ports, contextra-types`
- **Funktionsinventar:** 140 pub-Funktionen von 253 gesamt
- **Schnittstellen:** 3 Traits, 21 Structs, 4 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-env`: 4 Symbole
  - `contextra-ids`: 4 Symbole
  - `contextra-types`: 5 Symbole

### 📦 Crate: `contextra-ports` (Score: 53/100)
- **Umfang:** 4,227 LOC (42 Quelldateien)
- **Abhängigkeiten (2):** `contextra-env, contextra-types`
- **Funktionsinventar:** 80 pub-Funktionen von 475 gesamt
- **Schnittstellen:** 43 Traits, 66 Structs, 20 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 42 Symbole
  - `contextra-env`: 4 Symbole
- **⚠️ Dead Code:** 1 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 21 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-types' (42 importierte Symbole).
  - 21 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören in 'contextra-erasure' (SPEC §3.1 / INV-7).
  - 1 nicht verdrahtete private Symbole/Inventar-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-types' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Löschrelikte aus contextra-ports entfernen und in 'contextra-erasure' modularisieren; Indizes bieten nur neutrale Maintenance-Methoden.
  - [ ] Inventarisieren vor Verschiebung (E-06/E-07): Kein Algorithmus wird gelöscht; hinter Feature 'experimental' sichern oder verdrahten.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] fill_key_bytes` (crates/contextra-ports/src/key_material.rs:32)

### 📦 Crate: `contextra-py` (Score: 77/100)
- **Umfang:** 2,235 LOC (16 Quelldateien)
- **Abhängigkeiten (3):** `contextra, contextra-types, contextra-wire`
- **Funktionsinventar:** 61 pub-Funktionen von 161 gesamt
- **Schnittstellen:** 0 Traits, 8 Structs, 0 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-types`: 3 Symbole
  - `contextra`: 9 Symbole
- **Befunde:**
  - Starke direkte Kopplung an 'contextra' (9 importierte Symbole).
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.

### 📦 Crate: `contextra-rank` (Score: 88/100)
- **Umfang:** 4,440 LOC (31 Quelldateien)
- **Abhängigkeiten (2):** `contextra-ports, contextra-types`
- **Funktionsinventar:** 126 pub-Funktionen von 237 gesamt
- **Schnittstellen:** 5 Traits, 35 Structs, 12 Enums

### 📦 Crate: `contextra-router` (Score: 58/100)
- **Umfang:** 6,111 LOC (30 Quelldateien)
- **Abhängigkeiten (4):** `contextra-adapt, contextra-ports, contextra-types, contextra-wire`
- **Funktionsinventar:** 93 pub-Funktionen von 251 gesamt
- **Schnittstellen:** 1 Traits, 30 Structs, 6 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-adapt`: 12 Symbole
  - `contextra-types`: 10 Symbole
  - `contextra-wire`: 2 Symbole
  - `contextra-ports`: 15 Symbole
- **⚠️ Dead Code:** 6 ungenutzte interne Definitionen identifiziert.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-adapt' (12 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (10 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-ports' (15 importierte Symbole).
  - 6 nicht verdrahtete private Symbole/Inventar-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-adapt' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Abhängigkeit von 'contextra-types' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Abhängigkeit von 'contextra-ports' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Inventarisieren vor Verschiebung (E-06/E-07): Kein Algorithmus wird gelöscht; hinter Feature 'experimental' sichern oder verdrahten.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] write_atomic` (crates/contextra-router/src/seam.rs:118)
  - `[fn] create_dir_all` (crates/contextra-router/src/seam.rs:121)
  - `[fn] remove_file` (crates/contextra-router/src/seam.rs:124)
  - `[fn] exists` (crates/contextra-router/src/seam.rs:127)
  - `[fn] list_dir` (crates/contextra-router/src/seam.rs:133)

### 📦 Crate: `contextra-simd` (Score: 94/100)
- **Umfang:** 1,942 LOC (11 Quelldateien)
- **Abhängigkeiten (1):** `contextra-types`
- **Funktionsinventar:** 65 pub-Funktionen von 88 gesamt
- **Schnittstellen:** 1 Traits, 5 Structs, 3 Enums

### 📦 Crate: `contextra-store` (Score: 25/100)
- **Umfang:** 25,794 LOC (103 Quelldateien)
- **Abhängigkeiten (7):** `contextra-crypto, contextra-env, contextra-ids, contextra-mvcc, contextra-ports, contextra-sys, contextra-types`
- **Funktionsinventar:** 492 pub-Funktionen von 1145 gesamt
- **Schnittstellen:** 14 Traits, 90 Structs, 21 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-mvcc`: 5 Symbole
  - `contextra-ids`: 5 Symbole
  - `contextra-ports`: 16 Symbole
  - `contextra-types`: 5 Symbole
  - `contextra-env`: 3 Symbole
  - `contextra-crypto`: 6 Symbole
- **⚠️ Direkte Kernel-Verdrahtung:** 27 Importe müssen über `contextra-ports` entkoppelt werden.
- **⚠️ DSGVO-/Löschketten-Lecks:** 62 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-ports' (16 importierte Symbole).
  - 27 Stellen mit direkten Importen aus konkreten Speicherkernen.
  - 62 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören in 'contextra-erasure' (SPEC §3.1 / INV-7).
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-ports' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Direkte Kernel-Importe für Ebene E0/E1 vollständig eliminieren (autark); für Ebene E2 via Backends-Trait bündeln.
  - [ ] Löschrelikte aus contextra-store entfernen und in 'contextra-erasure' modularisieren; Indizes bieten nur neutrale Maintenance-Methoden.

### 📦 Crate: `contextra-testkit` (Score: 78/100)
- **Umfang:** 3,745 LOC (22 Quelldateien)
- **Abhängigkeiten (2):** `contextra-ports, contextra-types`
- **Funktionsinventar:** 82 pub-Funktionen von 185 gesamt
- **Schnittstellen:** 0 Traits, 19 Structs, 6 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ports`: 39 Symbole
  - `contextra-types`: 10 Symbole
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-ports' (39 importierte Symbole).
  - Starke direkte Kopplung an 'contextra-types' (10 importierte Symbole).
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-ports' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Abhängigkeit von 'contextra-types' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.

### 📦 Crate: `contextra-text` (Score: 40/100)
- **Umfang:** 7,110 LOC (38 Quelldateien)
- **Abhängigkeiten (4):** `contextra-env, contextra-ids, contextra-ports, contextra-types`
- **Funktionsinventar:** 136 pub-Funktionen von 442 gesamt
- **Schnittstellen:** 10 Traits, 48 Structs, 7 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ids`: 3 Symbole
  - `contextra-ports`: 13 Symbole
  - `contextra-types`: 7 Symbole
- **⚠️ Dead Code:** 2 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 24 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-ports' (13 importierte Symbole).
  - 24 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören in 'contextra-erasure' (SPEC §3.1 / INV-7).
  - 2 nicht verdrahtete private Symbole/Inventar-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-ports' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Löschrelikte aus contextra-text entfernen und in 'contextra-erasure' modularisieren; Indizes bieten nur neutrale Maintenance-Methoden.
  - [ ] Inventarisieren vor Verschiebung (E-06/E-07): Kein Algorithmus wird gelöscht; hinter Feature 'experimental' sichern oder verdrahten.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] layer` (crates/contextra-text/src/compat/scanner.rs:196)
  - `[fn] scan_units` (crates/contextra-text/src/compat/scanner.rs:199)

### 📦 Crate: `contextra-types` (Score: 79/100)
- **Umfang:** 5,258 LOC (25 Quelldateien)
- **Abhängigkeiten (1):** `contextra-ids`
- **Funktionsinventar:** 121 pub-Funktionen von 296 gesamt
- **Schnittstellen:** 1 Traits, 44 Structs, 34 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ids`: 16 Symbole
- **⚠️ Dead Code:** 20 ungenutzte interne Definitionen identifiziert.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-ids' (16 importierte Symbole).
  - 20 nicht verdrahtete private Symbole/Inventar-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-ids' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Inventarisieren vor Verschiebung (E-06/E-07): Kein Algorithmus wird gelöscht; hinter Feature 'experimental' sichern oder verdrahten.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] contextra_error_display_no_panic` (crates/contextra-types/src/error.rs:534)
  - `[fn] prop_fusion_weights_never_panics` (crates/contextra-types/src/types/saos.rs:772)
  - `[fn] default_tl_hfd_sigma` (crates/contextra-types/src/types/domain/misc.rs:405)
  - `[fn] default_tl_hfd_delta` (crates/contextra-types/src/types/domain/misc.rs:409)
  - `[fn] default_tl_hfd_gamma` (crates/contextra-types/src/types/domain/misc.rs:412)

### 📦 Crate: `contextra-vault` (Score: 88/100)
- **Umfang:** 320 LOC (4 Quelldateien)
- **Abhängigkeiten (2):** `contextra-sys, contextra-types`
- **Funktionsinventar:** 13 pub-Funktionen von 22 gesamt
- **Schnittstellen:** 0 Traits, 9 Structs, 2 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-sys`: 2 Symbole
  - `contextra-types`: 5 Symbole

### 📦 Crate: `contextra-vector` (Score: 23/100)
- **Umfang:** 14,872 LOC (60 Quelldateien)
- **Abhängigkeiten (7):** `contextra-env, contextra-ids, contextra-mvcc, contextra-ports, contextra-simd, contextra-sys, contextra-types`
- **Funktionsinventar:** 328 pub-Funktionen von 634 gesamt
- **Schnittstellen:** 16 Traits, 64 Structs, 10 Enums
- **Schnittstellen-Kopplung (importierte Fremdsymbole):**
  - `contextra-ids`: 2 Symbole
  - `contextra-env`: 7 Symbole
  - `contextra-ports`: 9 Symbole
  - `contextra-types`: 8 Symbole
- **⚠️ Dead Code:** 4 ungenutzte interne Definitionen identifiziert.
- **⚠️ DSGVO-/Löschketten-Lecks:** 19 Fundstellen müssen in `contextra-erasure` überführt werden.
- **Befunde:**
  - Starke direkte Kopplung an 'contextra-ports' (9 importierte Symbole).
  - 19 Referenzen auf DSGVO/Löschkette/Nachweise gefunden. Diese gehören in 'contextra-erasure' (SPEC §3.1 / INV-7).
  - 4 nicht verdrahtete private Symbole/Inventar-Kandidaten identifiziert.
- **Handlungsschritte für Coding-Agenten:**
  - [ ] Abhängigkeit von 'contextra-ports' auflösen: Für Ebene E0/E1 direkte Abhängigkeiten eliminieren (lokale Typen/Fehler, kein 'contextra-ports'). Für Ebene E2 generische Abstraktionen oder State-Passing nutzen.
  - [ ] Löschrelikte aus contextra-vector entfernen und in 'contextra-erasure' modularisieren; Indizes bieten nur neutrale Maintenance-Methoden.
  - [ ] Inventarisieren vor Verschiebung (E-06/E-07): Kein Algorithmus wird gelöscht; hinter Feature 'experimental' sichern oder verdrahten.
- **Dead-Code-Kandidaten (Auszug):**
  - `[fn] now_unix_ms` (crates/contextra-vector/src/contract.rs:53)
  - `[fn] next_u64` (crates/contextra-vector/src/contract.rs:60)
  - `[fn] transform` (crates/contextra-vector/src/contract.rs:142)
  - `[fn] inverse_transform` (crates/contextra-vector/src/contract.rs:144)

## 4. Löschketten- und DSGVO-Fundstelleninventar

Folgende Stellen müssen im Rahmen der Entkopplung aus den jeweiligen Crates bereinigt werden:

#### `contextra` (2 Stellen)
- `crates/contextra/src/builder.rs:330` — Suchbegriff `deletion_proof`: `|| config_for_open.deletion_proof_active != resolved.integrity_proof_active`
- `crates/contextra/src/builder.rs:340` — Suchbegriff `deletion_proof`: `config_for_open.deletion_proof_active = resolved.integrity_proof_active;`

#### `contextra-audit-export` (33 Stellen)
- `crates/contextra-audit-export/src/testkit.rs:4` — Suchbegriff `deletionproof`: `AuditExportError, ControllerContact, DeletionProofSummary, EgressEventSummary,`
- `crates/contextra-audit-export/src/testkit.rs:48` — Suchbegriff `löschung`: `retention_period: "30 Tage nach Abwicklung, danach automatische Löschung"`
- `crates/contextra-audit-export/src/testkit.rs:56` — Suchbegriff `dsgvo`: `legal_basis: "Art. 6 Abs. 1 lit. b DSGVO".to_string(),`
- `crates/contextra-audit-export/src/testkit.rs:63` — Suchbegriff `deletion_proof`: `deletion_proofs: vec![DeletionProofSummary {`
- `crates/contextra-audit-export/src/testkit.rs:83` — Suchbegriff `löschung`: `retention_period: "Löschung bei Mandantenkündigung".to_string(),`
- `crates/contextra-audit-export/src/testkit.rs:89` — Suchbegriff `dsgvo`: `legal_basis: "Art. 6 Abs. 1 lit. f DSGVO".to_string(),`
- `crates/contextra-audit-export/src/testkit.rs:91` — Suchbegriff `deletion_proof`: `deletion_proofs: vec![DeletionProofSummary {`
- `crates/contextra-audit-export/src/bsi_mapping.rs:5` — Suchbegriff `gdpr`: `//! cryptographic mapping table included in the generated GDPR Art. 30 processing register export.`
- `crates/contextra-audit-export/src/error.rs:13` — Suchbegriff `gdpr`: `/// Missing mandatory field required by GDPR Article 30.`
- `crates/contextra-audit-export/src/lib.rs:1` — Suchbegriff `gdpr`: `//! `contextra-audit-export` — GDPR Article 30 Processing Register export generator.`
- `crates/contextra-audit-export/src/lib.rs:5` — Suchbegriff `dsgvo`: `//! (Verzeichnis von Verarbeitungstätigkeiten nach Art. 30 DSGVO) based on neutral audit records.`
- `crates/contextra-audit-export/src/lib.rs:107` — Suchbegriff `dsgvo`: `/// Contact details for the data controller, representative, and data protection officer (Art. 30 Ab`
- `crates/contextra-audit-export/src/lib.rs:137` — Suchbegriff `dsgvo`: `/// Third country transfer record including legal safeguard guarantees (Art. 30 Abs. 1 lit. e DSGVO)`
- `crates/contextra-audit-export/src/lib.rs:163` — Suchbegriff `dsgvo`: `/// Retention period definition per category of personal data (Art. 30 Abs. 1 lit. f DSGVO).`
- `crates/contextra-audit-export/src/lib.rs:202` — Suchbegriff `gdpr`: `/// Summary of a cryptographic deletion proof (GDPR Art. 17 / Art. 30 compliance).`

#### `contextra-avv-generator` (229 Stellen)
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

#### `contextra-cognition` (1 Stellen)
- `crates/contextra-cognition/src/context_compaction/cleanup.rs:14` — Suchbegriff `löschung`: `// INVARIANTEN: Atomare Transaktionen für Löschungen; sequentieller TxId-Fortschritt via AtomicU64.`

#### `contextra-engine` (6 Stellen)
- `crates/contextra-engine/src/lib.rs:247` — Suchbegriff `deletion_proof`: `pub deletion_proof_active: bool,`
- `crates/contextra-engine/src/lib.rs:311` — Suchbegriff `deletion_proof`: `.field("deletion_proof_active", &self.deletion_proof_active)`
- `crates/contextra-engine/src/lib.rs:345` — Suchbegriff `deletion_proof`: `deletion_proof_active: false,`
- `crates/contextra-engine/src/contextra_impl/lifecycle.rs:26` — Suchbegriff `deletion_proof`: `deletion_proof_active: false,`
- `crates/contextra-engine/src/contextra_impl/lifecycle.rs:42` — Suchbegriff `deletion_proof`: `config.deletion_proof_active = false;`
- `crates/contextra-engine/src/contextra_impl/lifecycle.rs:60` — Suchbegriff `deletion_proof`: `config.deletion_proof_active = false;`

#### `contextra-graph` (25 Stellen)
- `crates/contextra-graph/src/compat.rs:8` — Suchbegriff `purger`: `pub mod purger;`
- `crates/contextra-graph/src/lib.rs:66` — Suchbegriff `purger`: `pub mod purger;`
- `crates/contextra-graph/src/lib.rs:136` — Suchbegriff `purger`: `pub use purger::{graph_purger, GraphPurger};`
- `crates/contextra-graph/src/purger.rs:1` — Suchbegriff `purger`: `//! Compatibility re-export for [`crate::compat::purger`].`
- `crates/contextra-graph/src/purger.rs:11` — Suchbegriff `purger`: `pub use crate::compat::purger::*;`
- `crates/contextra-graph/src/purger.rs:13` — Suchbegriff `purger`: `// SECTION Purger Core Implementation`
- `crates/contextra-graph/src/purger.rs:15` — Suchbegriff `purger`: `// !SECTION Purger Core Implementation`
- `crates/contextra-graph/src/compat/purger.rs:1` — Suchbegriff `purger`: `//! Neutrale Wartungs- und Bereinigungs-Implementierung für den Graph-Baustein (`contextra-ports::sc`
- `crates/contextra-graph/src/compat/purger.rs:12` — Suchbegriff `purger`: `// SECTION Purger Core Implementation`
- `crates/contextra-graph/src/compat/purger.rs:24` — Suchbegriff `purger`: `use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};`
- `crates/contextra-graph/src/compat/purger.rs:29` — Suchbegriff `purger`: `/// Purger-Implementierung für [`CsrGraph`].`
- `crates/contextra-graph/src/compat/purger.rs:33` — Suchbegriff `purger`: `pub struct GraphPurger {`
- `crates/contextra-graph/src/compat/purger.rs:37` — Suchbegriff `purger`: `impl GraphPurger {`
- `crates/contextra-graph/src/compat/purger.rs:38` — Suchbegriff `purger`: `/// Erzeugt einen neuen [`GraphPurger`].`
- `crates/contextra-graph/src/compat/purger.rs:44` — Suchbegriff `purger`: `/// Erzeugt einen `Arc<dyn Purger>` für einen gegebenen [`CsrGraph`].`

#### `contextra-kvcache` (3 Stellen)
- `crates/contextra-kvcache/src/compat/scanner.rs:4` — Suchbegriff `purger`: `use contextra_ports::scanner::Purger as LayerPurger;`
- `crates/contextra-kvcache/src/compat/scanner.rs:20` — Suchbegriff `purger`: `impl LayerPurger for KvCachePruner {`
- `crates/contextra-kvcache/src/compat/scanner.rs:109` — Suchbegriff `purger`: `use contextra_ports::scanner::{LayerScanner, Purger, ScanTarget, ScanTechnique, WitnessLayer};`

#### `contextra-license` (1 Stellen)
- `crates/contextra-license/src/types.rs:20` — Suchbegriff `löschkette`: `/// Compliance-Ring: Strenge Auditierbarkeit, Löschketten und regulatorische Garantien.`

#### `contextra-ports` (21 Stellen)
- `crates/contextra-ports/src/erasure_legacy.rs:51` — Suchbegriff `purger`: `LayerScanner, LayerWitness, PurgeStats, Purger, ScanSession, ScanTarget, ScanTechnique,`
- `crates/contextra-ports/src/contract_tests.rs:16` — Suchbegriff `purger`: `use crate::scanner::{LayerScanner, Purger, ScanTarget, ScanTechnique};`
- `crates/contextra-ports/src/contract_tests.rs:324` — Suchbegriff `purger`: `/// Contract test helper verifying [`Purger`] trait implementation compliance.`
- `crates/contextra-ports/src/contract_tests.rs:327` — Suchbegriff `purger`: `pub fn verify_purger_contract<P: Purger>(purger: &P) {`
- `crates/contextra-ports/src/contract_tests.rs:332` — Suchbegriff `purger`: `let stats1 = purger.purge(&ctx, &target).expect("first purge call");`
- `crates/contextra-ports/src/contract_tests.rs:335` — Suchbegriff `purger`: `let stats2 = purger`
- `crates/contextra-ports/src/contract_tests.rs:539` — Suchbegriff `purger`: `fn test_contract_purger_and_scanner() {`
- `crates/contextra-ports/src/contract_tests.rs:542` — Suchbegriff `purger`: `struct MockPurger {`
- `crates/contextra-ports/src/contract_tests.rs:545` — Suchbegriff `purger`: `impl Purger for MockPurger {`
- `crates/contextra-ports/src/contract_tests.rs:565` — Suchbegriff `purger`: `let purger = MockPurger {`
- `crates/contextra-ports/src/contract_tests.rs:568` — Suchbegriff `purger`: `verify_purger_contract(&purger);`
- `crates/contextra-ports/src/conformance.rs:26` — Suchbegriff `purger`: `use crate::scanner::layer::{Purger, ScanTarget};`
- `crates/contextra-ports/src/conformance.rs:174` — Suchbegriff `purger`: `/// Verifies that [`Purger`] methods can be invoked.`
- `crates/contextra-ports/src/conformance.rs:175` — Suchbegriff `purger`: `pub fn assert_purger_contract<T: Purger>(t: &T, ctx: &RequestContext, target: &ScanTarget) {`
- `crates/contextra-ports/src/lib.rs:74` — Suchbegriff `purger`: `/// Storage layer scanners and physical purgers (Vertrag V1, SPEC §12.7.4).`

#### `contextra-store` (62 Stellen)
- `crates/contextra-store/src/scanner/runtime.rs:7` — Suchbegriff `purger`: `/// Laufzeit-Ausführungsstrategie für synchrone Scanner- und Purger-Ports.`
- `crates/contextra-store/src/scanner/runtime.rs:10` — Suchbegriff `purger`: `/// Synchron aufgerufene Port-Methoden wie `Purger::purge` und `LayerScanner::scan_units``
- `crates/contextra-store/src/scanner/store_scan_view.rs:1` — Suchbegriff `purger`: `//! # StoreScanView — Entkoppelter Scan-Kontext für Scanner und Purger (Ring 1)`
- `crates/contextra-store/src/scanner/store_scan_view.rs:3` — Suchbegriff `purger`: `//! [`StoreScanView`] kapselt den minimalen Lesezugriff, den Scanner und Purger auf den`
- `crates/contextra-store/src/scanner/store_scan_view.rs:10` — Suchbegriff `purger`: `//! `LsmPurger` jeweils `Arc<LsmStorage>` und riefen `pub(crate)`-Methoden direkt auf.`
- `crates/contextra-store/src/scanner/store_scan_view.rs:34` — Suchbegriff `purger`: `/// Schlanke, entkoppelte Lesezugriffssicht auf LSM-Speicherschichten für Scanner und Purger.`
- `crates/contextra-store/src/scanner/store_scan_view.rs:96` — Suchbegriff `purger`: `/// Gibt die SSTable-Liste als `Arc<RwLock<...>>` zurück (für Purger-Schreibzugriff).`
- `crates/contextra-store/src/scanner/tests.rs:6` — Suchbegriff `purger`: `// ZWECK: Tests für LSM Scanner & Purger (Vertrag V1, PURGE-PIPELINE W1-P01).`
- `crates/contextra-store/src/scanner/tests.rs:113` — Suchbegriff `purger`: `let purgers = lsm_purgers(&storage);`
- `crates/contextra-store/src/scanner/tests.rs:114` — Suchbegriff `purger`: `purgers[0]`
- `crates/contextra-store/src/scanner/tests.rs:186` — Suchbegriff `purger`: `let purgers = lsm_purgers(&storage);`
- `crates/contextra-store/src/scanner/tests.rs:187` — Suchbegriff `purger`: `let res = purgers[0].purge(&ctx, &target);`
- `crates/contextra-store/src/scanner/tests.rs:231` — Suchbegriff `purger`: `let purgers = lsm_purgers(&storage);`
- `crates/contextra-store/src/scanner/tests.rs:232` — Suchbegriff `purger`: `purgers[0].purge(&ctx, &target).unwrap();`
- `crates/contextra-store/src/scanner/tests.rs:246` — Suchbegriff `purger`: `fn test_scanner_and_purger_without_runtime_returns_err() {`

#### `contextra-text` (24 Stellen)
- `crates/contextra-text/src/inverted.rs:29` — Suchbegriff `purger`: `pub use purge::{text_purger, PurgeReport, TextPurger};`
- `crates/contextra-text/src/lib.rs:81` — Suchbegriff `purger`: `text_purger, text_scanner, BM25MorphIndex, PurgeReport, TextPurger, TextScanner,`
- `crates/contextra-text/src/compat/purge.rs:2` — Suchbegriff `purger`: `// FILE-CONTEXT: TextIndex Physical Purger & BM25 Corpus Statistics Recalculation (§12.7.4).`
- `crates/contextra-text/src/compat/purge.rs:3` — Suchbegriff `purger`: `// ZWECK: Stellt den Purger für physical deletion per document and per collection bereit.`
- `crates/contextra-text/src/compat/purge.rs:22` — Suchbegriff `purger`: `use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};`
- `crates/contextra-text/src/compat/purge.rs:44` — Suchbegriff `purger`: `pub struct PurgeReport {`
- `crates/contextra-text/src/compat/purge.rs:55` — Suchbegriff `purger`: `impl PurgeReport {`
- `crates/contextra-text/src/compat/purge.rs:90` — Suchbegriff `purger`: `impl Default for PurgeReport {`
- `crates/contextra-text/src/compat/purge.rs:97` — Suchbegriff `purger`: `/// Physischer Purger für den Textindex zur Bereinigung von Storage und Metadaten.`
- `crates/contextra-text/src/compat/purge.rs:102` — Suchbegriff `purger`: `pub struct TextPurger<S: StorageEngine + 'static> {`
- `crates/contextra-text/src/compat/purge.rs:106` — Suchbegriff `purger`: `impl<S: StorageEngine + 'static> TextPurger<S> {`
- `crates/contextra-text/src/compat/purge.rs:107` — Suchbegriff `purger`: `/// Creates a new `TextPurger` for the given index.`
- `crates/contextra-text/src/compat/purge.rs:113` — Suchbegriff `purger`: `/// returning a detailed `PurgeReport`.`
- `crates/contextra-text/src/compat/purge.rs:114` — Suchbegriff `purger`: `pub fn purge_document_report(&self, tx: TxId, doc_id: DocId) -> PurgeReport {`
- `crates/contextra-text/src/compat/purge.rs:126` — Suchbegriff `purger`: `fn purge_document_internal_report(&self, tx: TxId, doc_id: DocId) -> PurgeReport {`

#### `contextra-vector` (19 Stellen)
- `crates/contextra-vector/src/lib.rs:121` — Suchbegriff `purger`: `pub use hnsw::{hnsw_purger, hnsw_scanner, HnswPurger, HnswScanner};`
- `crates/contextra-vector/src/compat/purge.rs:11` — Suchbegriff `purger`: `use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};`
- `crates/contextra-vector/src/compat/purge.rs:23` — Suchbegriff `purger`: `pub struct HnswPurger {`
- `crates/contextra-vector/src/compat/purge.rs:28` — Suchbegriff `purger`: `impl HnswPurger {`
- `crates/contextra-vector/src/compat/purge.rs:29` — Suchbegriff `purger`: `/// Erzeugt einen neuen `HnswPurger`.`
- `crates/contextra-vector/src/compat/purge.rs:35` — Suchbegriff `purger`: `/// Erzeugt einen `Arc<dyn Purger>` für einen HNSW Index.`
- `crates/contextra-vector/src/compat/purge.rs:40` — Suchbegriff `purger`: `pub fn hnsw_purger(index: Arc<HnswIndex>) -> Arc<dyn Purger> {`
- `crates/contextra-vector/src/compat/purge.rs:41` — Suchbegriff `purger`: `Arc::new(HnswPurger::new(index))`
- `crates/contextra-vector/src/compat/purge.rs:45` — Suchbegriff `purger`: `impl Purger for HnswPurger {`
- `crates/contextra-vector/src/hnsw/purge.rs:11` — Suchbegriff `purger`: `use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};`
- `crates/contextra-vector/src/hnsw/purge.rs:23` — Suchbegriff `purger`: `pub struct HnswPurger {`
- `crates/contextra-vector/src/hnsw/purge.rs:28` — Suchbegriff `purger`: `impl HnswPurger {`
- `crates/contextra-vector/src/hnsw/purge.rs:29` — Suchbegriff `purger`: `/// Erzeugt einen neuen `HnswPurger`.`
- `crates/contextra-vector/src/hnsw/purge.rs:35` — Suchbegriff `purger`: `/// Erzeugt einen `Arc<dyn Purger>` für einen HNSW Index.`
- `crates/contextra-vector/src/hnsw/purge.rs:40` — Suchbegriff `purger`: `pub fn hnsw_purger(index: Arc<HnswIndex>) -> Arc<dyn Purger> {`
