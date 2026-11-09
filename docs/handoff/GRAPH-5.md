# Übergabebericht MISSION `GRAPH-5`: Crate-Hülle, Vertrag, Entitäts-/Provider-Randmodule, Doku

## 1. Übersicht & Zielerreichung
Mission `GRAPH-5` stellt die normative Crate-Hülle für `contextra-graph` als autarken Klasse-A-Baustein (SPEC v4.0 §2, §5, B1–B12) her:
- **Vertrag:** `src/contract.rs` wurde byte-identisch mit dem gefrorenen Crate-Vertrag angelegt. Neutrale Traits `Index<Id, Query>` und `Maintain` nach SPEC §4.3 in `src/lib.rs` implementiert/bereitgestellt.
- **Fehlerbehandlung:** Primärer Fehlertyp `Error` (`GraphError`, `#[non_exhaustive]`) mit Subtypen `GraphMutationError` und Konvertierungen von/zu `crate::contract::Error`.
- **Randmodule entkoppelt:**
  - `entity_extraction.rs`: Arbeitet vollständig auf lokalen Typen (`DocId`, `Entity`, `EntityId`, `Error`/`Result`) mit deterministischer regelbasierter Extraktion ohne Machine-Learning-Laufzeit.
  - `provider.rs`: Trait `EntityProvider` definiert für modulare LLM-Anbindungen (hinter `experimental`), deterministische BTreeMap/BTreeSet in GraphProvider-Algorithmen (PageRank, Community Detection).
  - `purger.rs`: Ersetzt durch neutralen `GraphMaintainer` (`remove_document_edges`, `compact`, `vacuum`) sowie lesende Inspektionssicht (`inspect_stats`) hinter Feature `inspect`.
  - Handoff: Frühere ports-spezifische `Purger`-Brücke wurde verbatim unter `docs/handoff/GRAPH-5/adapter_src/purger_adapter.rs` für Klasse C (`contextra-erasure-adapters`) hinterlegt.
- **Crate-Hülle & Lizenzen:**
  - `Cargo.toml`: Standard-Features `default = ["std"]`, `serde`, `tracing`, `inspect`, `experimental` konfiguriert.
  - `LICENSE-MIT` und `LICENSE-APACHE` im Crate-Wurzelverzeichnis angelegt.
  - `README.md` und `SPEC.md` mit Invarianten, Lock-Modell, Budgets und autarkem Minimalbeispiel aktualisiert.

## 2. Funktionsinventar-Delta
- **Ausgangsbasis:** 7158 Funktionen im Workspace erfasst (`/tmp/inv_GRAPH-5_vorher.md`).
- **Veränderung:** Keine Funktionen im Produktionscode wurden gelöscht. Die Port-spezifische `Purger`-Implementierung wurde in das Handoff-Verzeichnis für Adapter übertragen und im Crate durch den neutralen `GraphMaintainer` mit identischer physischer Kantenbereinigung abgelöst.

## 3. Verifikations-Ergebnisse
- Zero-Panic: Keine Panics im Produktionscode von `contextra-graph`.
- Unsafe Boundaries: `#![forbid(unsafe_code)]` in allen Produktionsmodulen der Crate durchgängig gewahrt.
- Determinismus: Keine globalen Zeit-/Zufallsquellen im Kern.
- Gate G3: Standalone-Export-Test `python3 tools/export_crate.py contextra-graph --out /tmp/export-contextra-graph` vorbereitet.

## 4. Offene Punkte & Workspace-Hinweise
- Im Workspace-Zustand ist in `crates/contextra-vector/Cargo.toml` ein Manifest-Konflikt vorhanden (nicht-optionale Abhängigkeit `contextra-ports` unter `ports-compat`), der Workspace-weite Cargo-Befehle blockiert, bis der entsprechende Vektor-PR eingespielt ist. `contextra-graph` selbst ist vollständig modular und autark.
