# AGENTS.md — `contextra-erasure`

> **Normative Vorgabe gemäß [docs/SPEC.md §8.9](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md)**

## 1. Rolle & Zweck
`contextra-erasure` ist der entkoppelte, zentrale Löschkern von Contextra. Er orchestriert die Lösch-Saga, führt das kryptografische Löschjournal, verwaltet aktive Lese- und Suchsperren (`ReadGate` via `ErasedSet`), steuert den Daten-Tresor (`vault`), koordiniert die Residuen-Messungen über alle `ErasureLayer` und erzeugt den kryptografisch signierten Löschbericht (`ctx-erasure-report/1`).

## 2. Ring, Abhängigkeiten & Verbote
- **Ring:** Ring 3 (Löschkern)
- **Erlaubte Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-crypto`, `serde_json`, `tokio` (nur Tasks und Timer).
- **Verbotene Abhängigkeiten:**
  - **Kennt keine Ring-2-Crates:** Streng verboten sind Abhängigkeiten zu `contextra-store`, `contextra-text`, `contextra-vector` oder `contextra-mvcc` (Invariante AR-3). Die Kopplung erfolgt ausschließlich über Traits aus `contextra-ports`!
- **Verbotene Muster:**
  - **Strikt kein `unsafe`** (`#![forbid(unsafe_code)]`).
  - Kein Verschlucken von Fehlern: Jeder Fehler persistiert den Zustand `Pending` mit exponentiellem Backoff.

## 3. Modul-Inventar (Soll-Zustand laut SPEC §8.9)

| Modul | Inhalt & Verantwortung |
|---|---|
| `scope` | Löst `EraseTarget` in einen `Matcher` (Schlüsselbereiche und exakte Keys) sowie `doc_tag`-Mengen über den `StorageEngine`-Port auf (FI-E01). |
| `journal` | Append-only `erasure/journal.log`: Hash-Kette, Frame-Länge, Typ, Nutzlast (AEAD unter `K_journal`), CRC32; `fsync` je Eintrag (FI-E02..E04). |
| `erased_set` | `erasure/erased.set`: Verwaltet aktive Gates (Bereiche, Commitments, `gated_seq`); atomarer Austausch; implementiert den Port `ReadGate` (FI-E05..E07). |
| `vault` | Daten-Tresor (`vault/vault.dat`): Slot-Verwaltung, In-Memory-Index, DEK-LRU, `create_slot`, `dek`, `destroy`, `compact`; implementiert `RecordSealer` (FI-E08..E13). |
| `saga` | Deterministische Zustandsmaschine für den Löschablauf: `Requested → Gated → Drained → Rewritten → Cleaned → Measured → Completed` (FI-E14..E17). |
| `scheduler` | Steuerung von Purge-Fenstern, Deduplikation nach Commitment, `Timing::Now`, SLA-Alarme (FI-E18). |
| `measure` | Orchestriert E1/E2/E3-Messungen über `ErasureLayer::scan_residue` und Positivkontrollen (`positive_control`) (FI-E19..E21). |
| `classify` | Reine zustandslose Funktion `Evidence → ErasureClass` (G/K/P-Verband) (FI-E22). |
| `report` | Erstellung, kanonische Serialisierung und Ed25519-Signierung des Löschberichts `ctx-erasure-report/1` (FI-E23..E25). |
| `ledger` | Abgeschlossene Löschungen für den Restore-Guard (`erasure/ledger.log`); implementiert `LedgerSink` (FI-E26..E28). |
| `recover` | Crash-Recovery: Stellt unvollständige Löschungen aus dem Journal wieder her (FI-E29). |
| `checklist` | Maschinenlesbarer Export der Revisions- und Löschcheckliste (FI-E30). |

## 4. Kritische Invarianten
1. **Unmittelbares Gating (I-7):** Sobald der Zustand `Gated` im Journal persistiert ist, blockiert `ErasedSet` fail-closed alle Lese- und Suchzugriffe auf die Zielschlüssel.
2. **Crash-Resilienz (I-11):** Nach einem unvorhergesehenen Absturz setzt `recover()` den Zustand aller unvollständigen Sagas vor der Freigabe von Schreib- oder Lesepfaden wieder auf.
3. **Messen vor Behaupten:** Kein Bericht wird ausgestellt, ohne dass alle drei Messklassen (E1 Rohscan, E2 Decoder-Scan, E3 Schlüsselabwesenheit) durch maschinelle Zeugen (`LayerScan`) belegt sind.

## 5. Crate-Arbeitspakete zum SOLL-Zustand (PLAN.md)

- **[ ] WP-3.1: Löschtypen integration (`contextra-types`)**
- **[ ] WP-3.5: Journal und ErasedSet implementieren**
  - Implementierung von `erasure/journal.log` mit Hash-Kette und atomarem `erased.set`.
- **[ ] WP-3.6: ReadGate-Logik bereitstellen**
  - Fail-closed Implementierung von `ReadGate`.
- **[ ] WP-3.10: Saga und Wiederanlauf (`recover()`)**
  - Zustandsmaschine und Idempotenz-Recovery.
- **[ ] WP-3.11: Erase-Schnittstelle & Batch-Löschung**
  - Koordination von Mehrfachlöschungen und Fristen.
- **[ ] WP-3.12: Betroffenen-Löschung (D-14)**
  - Auflösung aller Dokumente einer `SubjectId` über den Locator-Index.
- **[ ] WP-5.2: Bericht v1 & Verifier (D-13)**
  - Erzeugung des signierten `ctx-erasure-report/1`.

## 6. Verifikation & Tests
- **T-REPORT-1 / T-PROOF-1:** Validierung des kanonischen Berichtsformats und der Signaturprüfung.
- **T-CLASS-1:** Vollständige Testmatrix für die Klassifikationslogik (`classify`).
- **T-MUT-1:** Mutationstests auf der Saga-Zustandsmaschine (kein Schritt darf übersprungen werden).
- **Loom & Fuzzing:** Nebenläufigkeitstests auf `ErasedSet` und Fuzzing des Journal-Parsers.
