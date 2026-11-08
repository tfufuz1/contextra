# Contextra Baukasten – Prompt-Katalog v4.0

> Maßgeblich: `docs/SPEC.md` v4.0 (§8 Gates, §10 Entscheidungen, §12 Arbeitspakete).
> Dieses Dokument enthält den verbindlichen Prompt-Katalog für Agenten zur Durchführung der Baukasten-Transformation.

## §1 Werkzeug-Liste (Diagnose & Verifikation)

Die folgenden Werkzeuge aus `tools/` stehen bereit und laufen ohne externe Abhängigkeiten:
- `python3 tools/orchestrate_diagnostics.py` — Gesamtsuite aller Diagnosen ausführen
- `python3 tools/zero_panic_analyzer.py` — Zero-Panic-Audit (`unwrap`, `expect`, `panic!`, `assert!`)
- `python3 tools/unsafe_boundary_auditor.py` — Unsafe-Inseln und `// SAFETY:`-Invariante prüfen
- `python3 tools/determinism_guard.py` — Nicht-Determinismus (`Instant::now`, `thread_rng`) detektieren
- `python3 tools/type_registry_sync.py` — Typenkollisionen und Crate-übergreifende Duplikate prüfen
- `python3 tools/async_boundary_scanner.py` — Async-Grenzen (kein Async/Tokio in Ring 0)
- `python3 tools/fsync_durability_tracer.py` — Durability-Audit (`std::fs`-Nutzung, Fsync-Prüfungen)
- `python3 tools/port_contract_auditor.py` — Port-Verträge und Re-Exporte
- `python3 tools/error_topology_mapper.py` — Error-Topologie und INV-LOG-1 / HR-4 Leak-Checks
- `python3 tools/test_matrix_advisor.py` — Testabdeckungs- und Pyramidenanalyse
- `python3 tools/decouple_inspector.py` — Analyse von Kopplungen und Inventar
- `python3 tools/export-crate <crate>` — Prüfung auf Standalone-Exportierbarkeit (Gate G3)
- `python3 tools/gate_g1_layering.py` — Gate G1 Kantenprüfung gegen `capabilities.toml`

## §2 Missionen-Übersicht (Welle 0 bis 6)

### MISSION P-00: Repo-Leitplanken (WP-0) – ohne Crate-Quellcode
- Status: In Bearbeitung
- Scope: `AGENTS.md`, `docs/**`, `tools/**`, `tests/**`, `governance/**`.
- Ziele: Bereinigung Altdokumente, Validierung der Werkzeuge, Implementierung Gate G1 & G8, Schemaerweiterung `capabilities.toml` (`transitional`), Erstellung Funktionsinventar.

### MISSION P-01: Billige Blätter (WP-1) – Klasse-A-Bausteine
- Crates: `contextra-simd`, `contextra-rank`, `contextra-ingest`, `contextra-audit-export`, `contextra-avv-generator`, `contextra-privacy`, `contextra-license`.
- Ziel: Vollständige Entkopplung von `types`/`ports`, lokale Fehler- und Typdefinitionen, Standalone-Exportierbarkeit (Gate G3).

### MISSION P-02: Löschlogik-Zentralisierung (WP-2)
- Crates: `contextra-erasure`, `contextra-erasure-adapters`.
- Ziel: Isolierung der Löschlogik aus Speicher- und Index-Kernen, neutrale Invarianten (`insert/remove/search`, `Maintain::compact()`).

### MISSION P-03: Große Bausteine (WP-3)
- Crates: `contextra-text`, `contextra-vector`, `contextra-graph`, `contextra-kvcache`, `contextra-durable-fs`.
- Ziel: Autarke Index- und Cache-Kerne ohne Fremdtypen.

### MISSION P-04: Storage-Bundle (WP-4)
- Crates: `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-store`, `contextra-checkpoint`.
- Ziel: Kohärentes Storage-Bundle mit klaren Modulgrenzen.

### MISSION P-05: Messung und Doku (WP-5)
- Generierung von `docs/generated/baukasten-status.md`, Benchmarks und Doku-Synchronisation.
