# AGENTS.md — `contextra-core`

## 1. Rolle & Zweck (Strangler-Fassade / Deprecated Debt)
`contextra-core` ist die historische Kern-Bibliothek des Projekts, die sich in aktiver Dekommissionierung befindet (Ring 0, Strangler-Pattern gem. DEBT-03).
Alle Domänentypen, Schnittstellen und Fehler-Enums wurden in autarke, orthogonale Bausteine überführt:
- Domänentypen & Fehler -> [`contextra-types`](../contextra-types/)
- Port-Schnittstellen -> [`contextra-ports`](../contextra-ports/)
- Transaktions- und MVCC-Primitiven -> [`contextra-mvcc`](../contextra-mvcc/)
- Wire-Protokolle -> [`contextra-wire`](../contextra-wire/)

`contextra-core` existiert temporär ausschließlich als reine Re-Export-Fassade zur Erhaltung der Abwärtskompatibilität während der Migration.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Strangler-Fassade)
- **Klasse:** Kern `[K]` (historisch)
- **Reifegrad:** `deprecated`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-wire`.

## 3. Garantien & Architektur-Invarianten
- **Keine neue Eigenlogik:** Keine neuen Funktionen, Typen oder Implementierungen in diesem Crate.
- **Transparente Weiterleitung:** Alle Re-Exports müssen typtreu und verhaltensneutral delegieren.
- **Zero-Panic-Doktrin:** Vollständiges Verbot von `unwrap()` oder `panic!`.

## 4. Verbotene Muster
- **Verbot von Neu-Abhängigkeiten auf `contextra-core`:** Neue Module und refaktorisierte Crates MÜSSEN direkt `contextra-types` und `contextra-ports` importieren.
- Kein I/O, kein Netzwerk, keine asynchronen Executors im Crate.
- Keine Zirkulärbezüge zu höheren Ringen.

## 5. Schnittstellen & Datenfluss
- Reine Re-Export-Hülle für Übergangspfade.

## 6. Tests & Verifikation
- Verifikation, dass alle Re-Exports kompatibel mit Downstream-Crates bleiben.
- Tracking der Downstream-Referenzen zur Vorbereitung der endgültigen Löschung (Ziel: 0 Konsumenten).

## 7. Offene Punkte & Technische Schulden
- **DEBT-03:** Vollständige Dekommissionierung und Löschung von `contextra-core`, sobald alle Konsumenten auf `contextra-types` und `contextra-ports` umgestellt sind.
