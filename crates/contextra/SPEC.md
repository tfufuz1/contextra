# Spezifikation: `contextra`

**Ring:** Ring 4 (Boundary & Composition Roots)
**Normativität:** Normativ gemäß `docs/SPEC.md` §1.1, §6.3
**Klasse:** C (Workspace Fassade & Composition Root)
**MSRV:** 1.89.0
**Code-Disziplin:** `#![forbid(unsafe_code)]`, Zero-Panic

---

## 1. Zweck & Systemgrenzen
`contextra` fungiert als die offizielle, konsolidierte Hauptfassade und Composition Root für Anwendungsentwickler:
- **Scope:** Bereitstellung eines intuitiven `ContextraBuilder`, High-Level-Methoden für Speicher-, Index-, Rerank- und LLM-Interaktion.
- **Lizenz- und Feature-Entkopplung:** Ermöglicht den Build sowohl als vollkommen freie Kern-Fassade als auch mit optionalen Enterprise- und Lizenz-Features (`license`). Ohne das Feature `license` kompiliert die Fassade mit deterministischer `OpenFastGate`-Semantik.

---

## 2. Metriken & Werkzeug-Audit-Befunde
*Erfasst via `tools/decouple_inspector.py -c contextra`:*
- **LOC:** 1.180 LOC über 7 Rust-Dateien.
- **Typen-Inventar:** 1 Trait, 10 Structs, 9 Enums, 42 öffentliche Funktionen (`pub fn`).
- **Abhängigkeiten:** 4 interne Crates (`contextra-types`, `contextra-db`, `contextra-engine`, `contextra-license`).
- **Audit-Befund:** 2 Legacy-Verweise auf Purge-Begriffe; 0 Unsafe.

---

## 3. Architektur & Modulkarte

| Modul | Verantwortung | Primäre Typen |
|---|---|---|
| `lib.rs` | Hauptreexporte und Schnellstart-Prelude | `Contextra`, `ContextraConfig`, `Collection` |
| `builder.rs` | Typisierter Builder mit Feature-abhängiger Validierung | `ContextraBuilder` |
| `performance_profile.rs` | Vorkonfigurierte Systemprofile (LowMemory, Balanced, HighThroughput) | `PerformanceProfile` |
| `collection_profile.rs` | Indexierungs- und Vektor-Vorlagen | `CollectionProfile` |
| `agent_memory.rs` | High-Level Wrapper für Agenten-Arbeitsgedächtnis | `AgentMemoryStore` |
| `query_rewriter.rs` | Clientseitige Abfrageoptimierung | `QueryRewriter` |

---

## 4. Invarianten (INV-CTX-*)

| Invariante | Beschreibung | Verifikation |
|---|---|---|
| `INV-CTX-ZERO-UNSAFE` | `#![forbid(unsafe_code)]` wird ausnahmslos eingehalten. | `cargo check -p contextra` |
| `INV-CTX-OPEN-GATE-DEFAULT` | Ohne aktiviertes Feature `license` greift die Fassade auf `OpenFastGate` zurück und erfordert keinen Lizenzschlüssel. | `tests/builder_license_disabled.rs` |
| `INV-CTX-BUILDER-VALIDATE` | Der Builder validiert Speichergrenzen, Thread-Pools und VFS-Pfade vor dem Start (Fail-Closed). | `tests/builder_validation.rs` |

---

## 5. Zielzustand & Migrationspfad
Gemäß `docs/SPEC.md` §6.3:
1. Vollständige Kapselung der Lizenzlogik hinter das optionale Feature `license`.
2. Strikte Bewahrung von `agent_memory.rs` und `query_rewriter.rs` als bequeme Fassaden-Helfer.
