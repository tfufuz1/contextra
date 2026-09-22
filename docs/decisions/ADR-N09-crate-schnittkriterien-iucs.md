# ADR-N09: Crate-Schnittkriterien I/U/C/S/D

* **Status:** Beschlossen / Final (Prinzip P30, Gesamtspezifikation §4.2, §20.3)
* **Datum:** 2026-09-17
* **Kontext / Auslöser:**
  Im Zuge der Entwicklung neigte das MemFuse Cognitive OS Repository zu einer unkontrollierten Crate-Zersplitterung ("Crate Sprawl"). Ohne ein klares, objektives Regelwerk führen zu viele feingliedrige Crates zu unnötigem Build-Overhead, komplexen Pass-Through-Abstraktionen und unübersichtlichen Monorepo-Abhängigkeiten.

  Um den Crate-Zuschnitt im Workspace streng zu reglementieren, führt Prinzip **P30** eine verbindliche Entscheidungsheuristik basierend auf fünf Kriterien (**I/U/C/S/D**) ein.

---

## 1. Die I/U/C/S/D Schnittkriterien (Prinzip P30)

Ein Modul oder eine Komponente darf nur dann als **eigenes Crate** im Workspace existieren, wenn es **mindestens eines** der folgenden fünf Kriterien erfüllt:

1. **I — Isolation flüchtiger/schwerer Abhängigkeiten (Isolation):**
   Isolation externer, schwerer oder plattformspezifischer C-Bindings/Bibliotheken (z.B. `candle`, `ort`, `wasmtime`, `pyo3`, `reqwest`), um Build-Zeiten zu kapseln und optionale Feature-Gates sauber abzugrenzen.
   *Beispiele:* `memfuse-infer-onnx`, `memfuse-sandbox`, `memfuse-py`.

2. **U — Unsafe-Insel (Unsafe Island):**
   Kapselung von `unsafe`-Code-Blöcken in eine dedizierte, auditierte System-Insel, damit alle abhängigen Crates `#![forbid(unsafe_code)]` erzwingen können.
   *Beispiele:* `memfuse-sys`, `memfuse-simd`, `memfuse-wire`.

3. **C — Eigenes Änderungsrhythmus / Cohesion (Bounded Context):**
   Starke fachliche Kohäsion mit eigenständiger Fach-Domäne und unabhängiger Weiterentwicklung.
   *Beispiele:* `memfuse-graph` (CSR/PPR), `memfuse-vector` (HNSW/DiskANN), `memfuse-text` (BM25).

4. **S — Stabilität & Skalierung / Größe > 8.000 LOC (Size/Compile Parallelism):**
   Crates, deren Quellcode-Umfang 8.000 Zeilen Code überschreitet, um die parallele Kompilierung der Rust-Compiler-Pipeline optimal auszulasten.

5. **D — Richtungserzwingung / Architektur-Ebenen (Dependencies / Composition Root):**
   Erzwingung von unidirektionalen Modul-Abhängigkeiten zur Vermeidung zyklischer Crate-Graph-Beziehungen oder als explizite Composition Root (z.B. Ports vs. Implementierung).
   *Beispiele:* `memfuse-ports` (Ring 0 Abstraktionen), `memfuse-types` (Ring 0 Fundament).

---

## 2. Konsolidierungsregel für Crates ohne Kriterium

Crates, die **keines** dieser fünf Kriterien (I, U, C, S, D) nachweisbar erfüllen, **dürfen nicht als eigenständiges Crate fortbestehen**. Sie müssen mit dem nächstgelegenen logischen Crate verschmolzen werden.

### Anwendungsbeispiel: Zerlegung & Konsolidierung
* `memfuse-core` wurde im Zuge der Ring-Reorganisation aufgeteilt:
  - `unsafe` System-Teile → `memfuse-sys` / `memfuse-simd` (Kriterium U)
  - Interne IPC-Generate → `memfuse-wire` (Kriterium U)
  - Reine Domain-Typen → `memfuse-types` (Kriterium D)
* Sollten sich zwei kleine Hilfs-Crates ohne schwere Dep oder Unsafe identifizieren lassen, werden diese gemäß §A2.4 Nr. 4 zusammengelegt (z.B. `adapt` + `rank`).

---

## 3. Konsequenzen

* Neue Crates dürfen im Monorepo nur noch unter Nachweis mindestens eines I/U/C/S/D-Kriteriums angelegt werden.
* Das CI-Tooling (`cargo xtask check-dag` / Layering-Test) prüft die Einhaltung der Crate-Grenzen und Ring-Architektur.
