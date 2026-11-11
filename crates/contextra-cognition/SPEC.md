# contextra-cognition — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ ist `docs/SPEC.md` (v3, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** Statische Analyse und Quellcode-Inspektion am Stand `5408826f8` / `278a3673`.
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-cognition/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Konsolidierung und Aggregation von Erinnerungen, Kontext-Kompaktierung, semantische Aggregation (LeanRAG / Hyperedges) und periodische Wartungsplanung. *[S]*

## 2. Zielzustand (docs/SPEC.md v3 §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Ebene (Kern-Closure) | E2 (Integration) |
| Kern-Closure (Ziel) | `types`, `ports` |
| Feature-Closures | `exp-hyperedge` → `graph` |
| Default-Features | leer (`default = []`) |
| Fehlertyp (R3) | `CognitionError` |
| Brücke / Invertierung | Bindung an `engine` wird im Schritt S-63 nach `db` (Feature `cognition`) verlagert |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1a/E1b = Kern-Closure ⊆ E0 bzw. E1a · E2 = Integration · E3 = Anwendungen und Testwerkzeuge.

## 3. Ist-Zustand (Stand 2026-11-11)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 5437 Zeilen in 27 Dateien |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | `engine`, `types`, `ports`, `graph` |
| Interne Dev-Abhängigkeiten (Ist) | `testkit` |
| Features (Ist) | `graph-connectivity-health`, `edge-reinforcement-learning` |
| Default-Features (Ist) | keine (`default = []`) |
| zero-panic (Produktion) | 0 Befunde (Tool: `zero_panic_analyzer.py`) |
| unsafe: Vorkommen / Verbot | 0 (`#![forbid(unsafe_code)]` erzwungen) |
| Determinismus (Produktion) | 0 Befunde (Tool: `determinism_guard.py`) |
| Error-Enum (Ist) | `CognitionError` (in `src/error.rs`) |

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** `engine` (wird in S-63 invertiert).
- **Default-Features:** Ist = Ziel (leer).
- **Fehlermodell:** `CognitionError` implementiert und konvertiert verlustfrei nach `ContextraError`.

## 5. Anforderungen & Schnittstellen (docs/SPEC.md §6.4) [S]

1. **Lokale Ports:** `DocumentSource`, `ConsolidationSink`, `HyperedgeSink`, `DecayPolicy` in `src/ports.rs`.
2. **Fehlermodell:** `CognitionError` mit `InvalidInput`, `PolicyViolation`, `Internal`, `BudgetExceeded`.
3. **Invertierungsvorbereitung (S-63):** Sobald die Integration in `contextra-db` erfolgt, werden engine-spezifische Bindungen aus `contextra-cognition` ausgelagert.
4. **Kommentierungsrichtlinien:** Einheitliches Comment-Anchors Schema über alle Module und Tests.
