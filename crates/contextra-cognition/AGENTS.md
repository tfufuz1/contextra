# AGENTS.md — `contextra-cognition`

---

> Ebene E2 · Reifegrad `experimental` · Unsafe: nein (`#![forbid(unsafe_code)]`) · Rolle: Konsolidierung und Langzeitgedächtnis: Aggregation, Synthese, Kontext-Kompaktierung, Wartungsplaner.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-11.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-cognition --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `engine`, `graph`, `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (docs/SPEC.md §3.1) | `types`, `ports`, `graph` (S-63 invertiert `engine`) |
| Features | `edge-reinforcement-learning`, `graph-connectivity-health` |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)` |

## 2. Modul-Karte

| Pfad | Verantwortung |
|---|---|
| `memory_consolidation.rs`, `consolidation_executor.rs`, `consolidation_locks.rs` | Gedächtniskonsolidierung, Sleep-Cycles, Invalidation-Locks |
| `aggregation_phase.rs`, `synthesis_phase.rs`, `semantic_aggregation_facade.rs`, `transitivity_veto.rs` | LeanRAG Aggregation, LLM-Synthese, Hyperedge-Verschmelzung, Transitivitäts-Veto |
| `context_compaction/`, `context.rs`, `channel.rs`, `graph_sink.rs`, `leanrag_input.rs` | Token-Budgetierung, Kontext-Kompaktierung, Kanäle, Graph-Senke, Vorverdichtung |
| `maintenance_scheduler.rs`, `maintenance_config.rs` | Periodische Wartung, Bereinigungs-Planer |

## 3. Verträge & Invarianten

1. **INV-RETENTION-1:** Ranking-Abwertung löscht nie; Löschung nur durch ausdrückliche Aktion oder opt-in `RetentionPolicy` (Dry-run, Karenz, Audit).
2. **INV-DERIVED-1:** Ableitungen (Zusammenfassungen, Fakten) hängen an Lineage; bei Löschung der Quelle neu ableiten oder als `stale` markieren: Zuständigkeit liegt bei `erasure`/`engine`-Ports, nicht hier.
3. LLM-Aufrufe nie unter Locks; Konsolidierungs-Locks haben feste Ordnung (`consolidation_locks.rs`).
4. **Zero-Panic-Doktrin:** Keine Panics in Produktionspfaden; Determinismus bei Sortierungen und Hashberechnungen.

## 4. Ports

Konsumiert `engine` (bis S-63), `GraphIndex`, `QueryRewriter`, `StorageEngine`, `Clock`, `Rng`.

## 5. Nebenläufigkeit & Performance

- Hintergrundarbeit über Planer mit Abbruch (`CancelToken`); kein ungeschützter geteilter Zustand außer in `consolidation_locks`.

## 6. Tests & Messung

```bash
cargo test -p contextra-cognition --locked
cargo clippy -p contextra-cognition --all-targets -- -D warnings
```

---

# Kommentar-Syntax (verbindlich nach Comment-Anchors VSC-Schema)

```comment_syntax
// =========================================================================
// 1. STANDARD-ANKER (behavior: "anchor")
// =========================================================================
// ANCHOR Allgemeines Lesezeichen für eine wichtige Code-Stelle
// TODO: Offene Aufgabe oder geplante Erweiterung
// FIXME - Bekannter Fehler, der behoben werden muss
// STUB Platzhalter für noch nicht fertiggestellten Code
// NOTE: Wichtiger Hinweis oder Erklärung zum Codeabschnitt
// REVIEW Code-Abschnitt, der noch überprüft werden muss

// =========================================================================
// 2. HIERARCHISCHE ANKER / REGIONEN (behavior: "region")
// =========================================================================
// SECTION Startet einen zusammenklappbaren Bereich in der Seitenleiste
    // TODO Verschachtelte Anker innerhalb der Region werden eingerückt
    // NOTE Auch normale Kommentare oder Code liegen hier drin
// !SECTION Beendet die Region (exakt gleicher Tag-Name mit vorangestelltem "!")

// =========================================================================
// 3. LINK-ANKER (behavior: "link")
// =========================================================================
// LINK pfad/zur/datei.ts                  -> Öffnet die angegebene Datei
// LINK pfad/zur/datei.ts:42               -> Öffnet die Datei in Zeile 42
// LINK pfad/zur/datei.ts#mein-ziel-anker  -> Springt zu einem benannten Anker
// LINK #lokaler-anker                     -> Springt zu einem Anker in derselben Datei
// LINK :15                                -> Springt zu Zeile 15 in derselben Datei

// =========================================================================
// 4. ANKER-ATTRIBUTE (IDs, EPICS & SORTIERUNG)
// =========================================================================
// Ziel-ID für LINK-Anker vergeben:
// ANCHOR[id=mein-ziel-anker] Diese Stelle kann per #mein-ziel-anker angesprungen werden
```
