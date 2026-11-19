# contextra-db — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §§3–10, 14 und `docs/crates/contextra-db.md`. **Ausgangslage:** 18.505 LOC/94 Dateien binden entfernte Konzepte (Ollama, Cognition, Extraction, License, Vault), enthalten Produktions-Panics und sind nicht die geforderte schlanke E2-Orchestrierung.

1. Vor jeder fachlichen Änderung friere Store/WAL/Index-Formate, Public API und Hybrid-Golden-Korpus ein; entferne anschließend alle nicht mehr im Workspace vorhandenen Domänen statt sie mit Compat-Shims zu erhalten.
2. Baue `TenantScopedStorage` als alleinigen Integrationskern mit Store, Vector, Text, Graph, Rank, MVCC und Checkpoint; erzwinge Tenant-Scope und die K/P/D-Klassen an den Schreibgrenzen.
3. Besitze das kanonische Lineage-Modell und die Kaskadenauflösung; führe 2PC mit `CommitBarrier::shared` und deterministischer Kompensations-/Recovery-Reihenfolge aus.
4. Verdrahte die Erasure-Saga: Gate auf allen get/scan/search/graph/export-Pfaden vor Decrypt/Rückgabe, Layer in definierter Reihenfolge, Key-Lock für Schreiben, Recovery ab Journalzustand.
5. Implementiere Vier-Signal-Hybrid-Suche durch E1-Kandidaten, Gate-Filter vor Fusion, RRF und deterministisches Auffüllen auf k; Zeitreise prüft immer den aktuellen Gate-Stand.
6. Lösche Altmodule und Produktions-Panics, statt sie zu verstecken; alle Hintergrundworker sind nachrangig, stoppbar und dürfen keine Löschinvariante umgehen.

**Abnahme:** T-DB1–10, T-DEL-2/4/6/10/11/12, 2PC-Crashmatrix, Tenant-Isolation, Golden-API-/Format-Diff und Zero-Panic-/Layering-Gates.
