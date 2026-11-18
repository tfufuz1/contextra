# contextra-db — Crate-Spezifikation

**Ebene:** E2 (Integration)  
**Bindung:** `docs/SPEC.md` §2, §3, §4, §5, §6, §7, §8, §9

---

## 1 Zweck & Domäne

`contextra-db` ist der zentrale Integrationsmotor des Systems. Es vereint:
- Collection-Verwaltung und CRUD-Operationen
- Hybrid-Search Orchestrierung (Volltext + Vektor + Graph + Rank)
- Transaktionskoordination (2PC über alle 4 Indizes mit Intent-Log)
- Anbindung der Löschkette und Durchsetzung von `ReadGate` an allen Lesepfaden
- Zeitreise und bitemporale Filterung

## 2 Klassentrennung & Schreibpfad (SPEC §3)

1. **`DataClassPolicy`:**
   - `Enforced`: Jeder Schreibvorgang deklariert seine `DataClass` (`K`, `P`, `D`, `E`).
   - `put_k`: Akzeptiert ausschließlich `Validated<KPayload>` via `contextra-privacy`. PII-Fund führt zum Abbruch vor dem Schreiben.
   - `put_p`: Schlüssel ist `SubjectRef` oder `PersonalRef`. Verschlüsselung erfolgt mit individuellem Betroffenen-Schlüssel.
2. **Klartext-Verbot:** Der Schreibpfad übergibt Daten vor dem WAL an den Versiegelungs-Adapter (`seal`). Kein Klartext erreicht persistente Schichten.

## 3 Lesesperren-Durchsetzung (SPEC §6)

`contextra-db` garantiert, dass **kein Lesepfad Daten ohne Prüfung gegen `ReadGate` ausliefert**:

- Einzelabruf (`get`, `get_at_snapshot`): Prüfung vor Entschlüsselung.
- Scans (`scan`, `scan_prefix`): Filterung je Eintrag.
- Suchen (`search`, `hybrid_search`): Vorfilterung der Kandidaten vor Fusion und Reranking, Auffüllen auf `k`.
- Graph-Traversierung: Unterdrückung gesperrter Kanten und Knoten.
- Export: Gesperrte Dokumente werden übersprungen und im Export-Bericht lediglich anonym gezählt.

## 4 Löschkette: Teilnahme von `db`

- `delete_op` initiiert die Lösch-Saga über `ErasureCore`.
- `drop_collection` widerruft den `CollectionKEK` atomar in O(1). Nachfolgende Neuanlagen der Collection erhalten eine neue Epoche.
- Stellt `DbPurgeTarget` und `DbScanLayer` als Adapter für die Saga-Phasen 4 und 5 bereit.

## 5 Zeitreise

- Saubere Trennung von Transaktionszeit (`as_of: u64`) und Gültigkeitszeit (`valid_at: i32` Tage seit 1970-01-01).
- Gültigkeit wirkt als Vorfilter auf Kandidatenebene.
- Zugriffsrichtlinien und `ReadGate` werden immer im aktuellen Stand ausgewertet (Z-T5).

## 6 Invarianten

- `#![forbid(unsafe_code)]`.
- 2PC-Transaktionen sind crash-sicher und idempotent.
