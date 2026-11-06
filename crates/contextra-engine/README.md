# contextra-engine

Transaktions-, Index- und Ausführungsorchestrator.

Normativitätsklasse: **Kern `[K]`** (Ring 3, `stable`; §6.0, `capabilities.toml`).

## Rolle & Zweck
`contextra-engine` dient als zentraler Ausführungskern und verbindet Speicher- und Index-Bausteine (LSM-Store, Vektor-Index, BM25-Volltext, Graph) zu einer konsistenten Transaktionsumgebung.

## Baukasten-Einordnung & Status
- **Ring:** Ring 3 (Ausführungsorchestrator)
- **Status:** 🟡 In Migration (Entkopplung von direkten Kernel-Importen gemäß `PLAN.md` Phase 4 & 5)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Deprecated Debt & Entkopplungsfahrplan
- **Löschketten-Relikte:** Historische Lösch- und Deletion-Proof-Logiken in `contextra-engine` sind als *Deprecated Debt* (siehe [`DEPRECATED_DEBT.md`](../../DEPRECATED_DEBT.md)) deklariert und werden zugunsten von [`contextra-erasure`](../contextra-erasure/) zurückgebaut.
- **Port-Abstraktion:** Direkte Abhängigkeiten zu konkreten Index- und Speicherkernen werden schrittweise durch Traits aus `contextra-ports` abgelöst.

## Kernkomponenten
- **Multi-Index Koordination:** Parallele Ausführung von Vektor-, Text- und strukturierter Suche.
- **Transaktions-Isolation:** Zusammenspiel mit `contextra-mvcc` für atomare Read/Write-Snapshots.
- **Hintergrund-Worker:** Koordination von Compaction- und Maintenance-Tasks über entkoppelte Kanäle.
