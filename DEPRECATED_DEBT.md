# DEPRECATED_DEBT.md — Zentrales Schulden- & Migrationsregister

> **Status:** Verbindliches Architektur-Inventar für die Transformation vom Monolithen zum modularen Baukastensystem.
> **Geltungsbereich:** Workspace `tfufuz1/contextra`

---

## 1. Übersicht & Zweck

Im Zuge der Evolution von Contextra (von Robotik über SQLite-basierte Agenten bis hin zur monolithischen In-Core-Löschkette) sind tiefe Querabhängigkeiten und historische Altlasten entstanden.
Dieses Dokument deklariert diese Relikte offiziell als **Deprecated Debt** (technische Schulden). Es dient allen Entwicklern und Coding-Agenten als verbindlicher Fahrplan für den schrittweisen Rückbau und die Migration in autarke Lego-Bausteine.

---

## 2. Zentrale Schuldenbereiche

### DEBT-01: Monolithische Löschkette in Core- und Speicher-Crates
- **Betroffene Crates:** `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-crypto`, `contextra-db`, `contextra-mcp`, `contextra-engine`.
- **Beschreibung:** Ursprünglich war die mehrstufige Löschlogik (Saga, Gate, Vault, Journal, Witness, Audit Reports) hart in den Speicher- und Indexkernen verdrahtet. Dies führte zu über 660 verstreuten Fundstellen von Lösch- und Rechts-Relikten in an sich neutralen Datenstrukturen.
- **Zielarchitektur:** Vollständige Kapselung aller Lösch-Orchestrierungs- und Nachweislogiken in [`contextra-erasure`](crates/contextra-erasure/).
- **Migrationsanweisung für Speicherkerne:** Speicherkerne implementieren ausschließlich minimale, neutrale Bereinigungs-Ports (`Purger`, `LayerScanner` in `contextra-ports`). Alle direkten Abhängigkeiten zu Deletion-Proofs, Audit-Trails und DSGVO-Kontexten werden entfernt.

### DEBT-02: Historische Robotik- & SQLite-Agenten-Relikte
- **Betroffene Crates:** `contextra-agent`, `contextra-db`, `contextra-cognition`.
- **Beschreibung:** Frühe Prototypen basierten auf SQLite-Tabellenstrukturen und Robotik-spezifischen Feedback-Schleifen.
- **Zielarchitektur:**
  - `contextra-agent` ist eine autarke Workflow-Engine (`checkpoint → execute → commit → audit`) ohne externe Workflow-Frameworks oder eingebettete SQLite-Engines.
  - State-Snapshots und Checkpoints laufen über generische MVCC-/Storage-Ports.

### DEBT-03: Strangler-Fassade `contextra-core`
- **Betroffene Crates:** `contextra-core`.
- **Beschreibung:** `contextra-core` fungiert derzeit als Re-Export-Hülle für ausgelagerte Typen nach `contextra-types`, `contextra-ports` und `contextra-mvcc`.
- **Status:** ERLEDIGT (Paket B31). Alle Konsumenten wurden auf `contextra-types`, `contextra-ports` und `contextra-mvcc` migriert und `contextra-core` wurde aus dem Workspace entfernt.

### DEBT-04: Direkte Kernel-Importe (Fehlende Port-Abstraktion)
- **Betroffene Crates:** `contextra-engine`, `contextra-db`, `contextra-mcp`, `contextra-store`.
- **Beschreibung:** Komponenten greifen direkt auf konkrete Instanzen anderer Engines zu (z. B. direkter Aufruf von `LsmStorage` statt Nutzung des Traits `StorageEngine`).
- **Zielarchitektur:** Entkopplung über Inversion of Control. Crates akzeptieren generische Parameter `T: StorageEngine` oder `T: VectorIndex` aus `contextra-ports`.

### DEBT-05: Lokale API-Deprecations in `contextra-db`
- **Betroffene Crates:** `contextra-db`.
- **Beschreibung:** 40 dokumentierte Aufrufstellen veralteter Such- und Filter-Methoden (`MetadataFilter`, alte `search`-Signaturen).
- **Referenz:** Detailliert inventarisiert in [`crates/contextra-db/DEPRECATED_DEBT.md`](crates/contextra-db/DEPRECATED_DEBT.md).

---

## 3. Migrations-Phasenplan (Mapping auf PLAN.md)

| Debt ID | Phase in `PLAN.md` | Zielbaustein | Erledigungskriterium |
|---|---|---|---|
| **DEBT-01** | Phase 3 & 4 | `contextra-erasure` | 0 Löschketten-/Nachweisimporte in `store`, `vector`, `text`, `crypto`. |
| **DEBT-02** | Phase 4 & 5 | `contextra-agent` | Reine Port-Nutzung, keine persistenten Tabellen-Altlasten. |
| **DEBT-03** | Phase 1 & 6 | `contextra-types` / `ports` | **ERLEDIGT (Paket B31)**: `contextra-core` vollständig dekommissioniert und entfernt. |
| **DEBT-04** | Phase 2 & 5 | `contextra-ports` | Keine konkreten Cross-Engine-Importe in Engines. |
| **DEBT-05** | Phase 4 & 5 | `contextra-db` | Entfernung der Legacy-Such-Fassaden nach Deprecation-Frist. |

---

## 4. Regeln für neue Pull Requests
1. **Keine neuen Schulden:** Es dürfen keine neuen direkten Abhängigkeiten zwischen Ring-1/Ring-0-Speicherkernen eingeführt werden.
2. **Keine absoluten Rechts- oder Garantieversprechen:** Keine unzulässigen Marketingaussagen oder Versprechungen absoluter rechtlicher Garantien.
3. **Schrittweiser Abbau:** Jeder PR an betroffenen Crates soll nach Möglichkeit dokumentierte Schuldenstellen abbauen oder Ports einführen.
