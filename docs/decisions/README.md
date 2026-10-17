# Architecture Decision Records (ADRs)

Das Verzeichnis `docs/decisions/` ist die einzige kanonische Quelle (Single Source of Truth) für Architecture Decision Records im Contextra-Projekt.

## Struktur & Lifecycle
- Jede Entscheidung wird als `ADR-NNN-slug.md` in diesem Verzeichnis geführt.
- Der Status jeder ADR (z. B. `Proposed`, `Accepted`, `Rejected`, `Deprecated`, `Superseded`) ist im YAML/Markdown-Front-Matter der jeweiligen Datei dokumentiert.

## Index der Normativen System-ADRs (N-Serie)

| ADR ID | Titel | Status |
| :--- | :--- | :--- |
| **ADR-N01** | Maschinelle Durchsetzung der Layer-Regeln (DAG & Ring-Schichtenmodell) | Accepted |
| **ADR-N02** | Sync-Kern — Synchroner Ring-0-Lese-Pfad und einheitliches `StorageEngine`-Trait | Accepted |
| **ADR-N03** | Drei erlaubte Unsafe-Inseln (`simd`, `sys`, `wire`), Deny/Forbid-Mechanik | Accepted |
| **ADR-N04** | Panic-Profile (`panic = "unwind"` / `release-abort`) | Accepted |
| **ADR-N05** | DocId Identifikatoren (64-Bit / 128-Bit Typisierung) | Accepted |
| **ADR-N06** | Konsistenzmodell — WAL als einzige Wahrheit (Akzeptanz Option B) | Accepted |
| **ADR-N07** | KV-Stufen A, B, C (Tombstone, Purge, Crypto-Shredding) | Accepted |
| **ADR-N08** | Generiertes Spec & Capability-Manifest | Accepted |
| **ADR-N09** | Crate-Schnittkriterien (IUCS / Internal Crate Standard) | Accepted |
| **ADR-N10** | Kein globaler veränderlicher Zustand | Accepted |
| **ADR-N11** | KVCache doppelter Tenant-Store | Accepted |
| **ADR-N12** | WAL High-Water-Mark Verankerung | Accepted |
| **ADR-N13** | WAL Frame V4 Header Integrität | Accepted |
| **ADR-N14** | Revocation-Log Persistence Coupling | Accepted |
| **ADR-N15** | Egress Guard and Decay Controller Consolidation | Accepted |
| **ADR-N16** | Orphaned Ports and No-Op Features Cleanup | Accepted |
| **ADR-N17** | F1 Lizenz-Gate als reine Integritätsprüfung | Accepted |
| **ADR-N18** | F2/F3 Löschbestätigung mit Status-Phase und gemessene Zeugnisse | Accepted |
| **ADR-N19** | F4 Konsistenzmodell — WAL/Store als einzige Wahrheit (Akzeptanz ADR-N06 Option B) | Accepted |
| **ADR-N20** | F5 Speicherkern — Feature Freeze, Messbasierte Evaluierung und redb-Ausstiegspfad | Accepted |
| **ADR-N21** | F6 Löschstrategie — Physische Bereinigung im Standard-Profil, Crypto-Shredding in sovereign | Accepted |
| **ADR-N22** | F7 Standard-Build-Umfang — Nur Core-Crates in `default-members` | Accepted |
| **ADR-N23** | F8 FC-Thompson Routing Feature-Gating bis zum Replay-Nachweis | Accepted |
| **ADR-N24** | F9 AVV-Generator und Audit-Export als Entwurfs- und Berichtswerkzeuge | Accepted |
| **ADR-N25** | F10 Panic-Isolation an der öffentlichen API (`guard_panic` und `Poisoned`-Zustand) | Accepted |
| **ADR-N26** | Auflösung der Unsafe-Policy-Inkonsistency (ADR-004 vs. ADR-N03) | Accepted |

## Index Schlüssel-ADRs (Refactored / Superseded / Core)

| ADR ID | Titel | Status |
| :--- | :--- | :--- |
| **ADR-001** | Speicherkern-Architektur — LSM-Tree für Persistenz | Superseded (durch ADR-N20) |
| **ADR-004** | Sovereign Core (Pure Rust Policy) | Refactored (durch ADR-N03 / ADR-N26) |
| **ADR-023** | Kompensierende Transaktion für Multi-Store `relate()` Operations | Superseded (durch ADR-N19) |
| **ADR-0XX** | `contextra-sandbox` — WASM Ausführungsgrenze & Bedrohungsmodell | Accepted |
