# AGENTS.md — `contextra-testkit`

> **Normative Vorgabe gemäß [docs/SPEC.md §8.15](docs/SPEC.md) und SPEC §15.4**

## 1. Rolle & Zweck
`contextra-testkit` stellt wiederverwendbare Testbausteine, deterministische Testuhren, speicherbasierte Referenzmodelle, In-Memory-Ports und Fehlerinjektions-Tools für Unit-, Integrations- und Crash-Tests bereit.

## 2. Ring, Abhängigkeiten & Einbindungsregeln
- **Ring:** dev (Test-Infrastruktur)
- **Erlaubte Einbindung:** **Ausschließlich als `dev-dependency`** in Tests oder Benchmarks (INV-EMBED-1).
- **Verbotene Einbindung:** Darf unter keinen Umständen in normalen `[dependencies]` eines Produktionscrates auftauchen.
- **Verbotene Muster:**
  - **Strikt kein `unsafe`** (`#![forbid(unsafe_code)]`).
  - Keine Produktionstypen mit Mock-Typen im Produktivcode vermischen.

## 3. Komponenten-Inventar (Soll-Zustand laut SPEC §8.15)

| Komponente | Inhalt & Verantwortung |
|---|---|
| `FaultVfs` | Hochentwickelter Fehlerinjektor: Absturzsimulation nach Operation $n$, abgerissene Schreibvorgänge an Byte $b$, `ENOSPC` (Disk voll), synchrone `EIO`-Fehler bei `sync_*`, verzögerte Sichtbarkeit (Page-Cache-Simulation), lückenlose Operations-Historie. |
| `ReferenceModel` | Unabhängiges BTreeMap-Orakel zur Validierung von Transaktionen, MVCC-Zeitreisen und Löschsemantiken. |
| `ManualClock` | Deterministisch steuerbare Testuhr (implementiert `Clock`-Port). |
| `InMemoryStore` | Schneller In-Memory Key-Value-Speicher (implementiert `StorageEngine`). Ersetzt altes `memory-only-storage` im Produktivcode. |
| `RecordingStorage` | Protokolliert jeden physisch geschriebenen Schlüssel zur Vollständigkeitsprüfung aller Operationen (T-COV-1). |
| `NeighborDigest` | BLAKE3-Hash über alle Nicht-Ziel-Schlüssel/Werte zur Verifikation, dass Löschungen unbeteiligte Daten unberührt lassen. |
| `LeakyLayer` | Absichtlich defekte `ErasureLayer`-Implementierung (vergisst Datenbereinigung) als Positivkontrolle für das Erasure Lab (T-RAW-2). |
| `CanaryFactory` | Generiert hochentropische Canary-Needles mit protokolliertem Seed für Löschmessungen. |
| `MemVfs`, `MemLedger` | Reine RAM-Implementierungen von `Vfs` und `LedgerSink`. |

## 4. Kritische Invarianten
1. **Deterministische Reproduzierbarkeit:** Jeder Testfall mit `FaultVfs` oder `ManualClock` ist über einen Seed exakt reproduzierbar.
2. **Keine Produktionsverunreinigung:** Kein Typ aus `contextra-testkit` darf im Produktions-Release kompiliert werden.

## 5. Crate-Arbeitspakete zum SOLL-Zustand (SPEC §15.4)

- **[x] WP-1.2: Veraltete Test-Mocks entfernen**
  - Rückbau von `in_memory_graph` und Graph-Fixtures vollständig abgeschlossen (V-RUECKBAU).
- **[ ] WP-2.6 & WP-2.10: `FaultVfs` erweitern für Crash-Matrix**
  - Implementierung feingranularer Fehlerinjektionen (Torn-Writes, `sync`-Abbrüche) für die LSM-Crash-Tests.
- **[ ] WP-5.1: `LeakyLayer` und `CanaryFactory` bereitstellen**
  - Implementierung der Positivkontroll-Fixtures für das Erasure Lab.

## 6. Verifikation & Selbstevaluierung
- **Selbsttests:** `FaultVfs`-Selbsttests auf deterministisches Verhalten bei I/O-Fehlern.
- **T-RAW-2:** Nachweis, dass `LeakyLayer` von den Erasure-Scannern zuverlässig als fehlerhaft erkannt wird.
