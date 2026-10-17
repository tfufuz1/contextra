# ADR-N18: F2/F3 Löschbestätigung mit Status-Phase und gemessene Zeugnisse

* **Status:** Accepted
* **Datum:** 2026-09-30
* **Kontext / Auslöser:**
  Die Einhaltung datenschutzrechtlicher Vorgaben (DSGVO Art. 17 "Recht auf Vergessenwerden") sowie auditiersicherer Compliance-Standards erfordert eine lückenlose Nachweisbarkeit bei der Löschung persönlicher oder vertraulicher Daten aus Speicherschichten (`contextra-store`), Vektorindizes und Sekundärstrukturen.
  In verteilten oder crash-gefährdeten Umgebungen reicht ein blinder boolescher Rückgabewert (`true`/`false`) nicht aus. Zudem birgt das reine Behaupten ("Asserting") von Löschnachweisen ohne tatsächliche physische Prüfung des Speicherzustands das Risiko von Fehlattestierungen bei verbleibenden Datenresten (Residuen).

## Entscheidung

1. **Garantierte Quittung mit Phasen-Status:**
   Jeder Aufruf der Lösch-Schnittstelle `forget()` (Dokument, Collection, Tenant) gibt ausnahmslos eine strukturierte Löschquittung (`ErasureReceipt` / `DeleteOutcome`) zurück [V]. Die Quittung enthält explizit die erreichte Löschphase der crash-konsistenten zustandbasierten Purge-State-Machine (`PurgeState` / `PurgeOrchestrator`: `Admitted`, `Flush`, `PurgeCompaction`, `Dispose`, `WalRetire`, `Witness`, `Finalize`, `Release`) [V].

2. **Gemessene statt behauptete Zeugnisse (INV-WITNESS-MINT-1):**
   Löschzeugnisse (`ErasureFootprint`, `LayerWitness`, `LayerCleanupProof`) dürfen niemals rein zählerbasiert mintiert oder behauptet werden [V]. Sie werden stets durch tatsächliche physische Scans der Speicherschichten gemessen und verifiziert (`LayerCleanupProof::new_after_verified_empty` ist restriktiv auf verifizierte Null-Scans beschränkt) [V].

## Alternativen

### Option A: Feuer-und-Vergessen mit einfachem `bool`-Status
* **Vorteile:** Minimaler API-Overhead; einfache Implementierung.
* **Nachteile:** Keinerlei Nachweisbarkeit über den tatsächlichen physischen Bereinigungszustand; unzureichend für DSGVO-Audits.
* **Bewertung:** Verworfen [S].

### Option B: Zählerbasierte / Behauptete Löschzeugnisse
* **Vorteile:** Schnelle Zeugnis-Ausstellung ohne I/O-Scan über physische SSTables/WALs.
* **Nachteile:** Gefahr falscher Attestierungen (`INV-WITNESS-MINT-1`), falls Fragmente in verwaisten Blöcken oder WALs verbleiben.
* **Bewertung:** Verworfen [S].

## Konsequenzen

* **Crash-Konsistenz:** Bei Unterbrechungen kann der `PurgeOrchestrator` anhand der in der Quittung und im Intent verankerten Phase die Bereinigung deterministisch wiederaufnehmen [V].
* **Auditsicherheit:** Erzeugte kryptografische Nachweise (`ErasureFootprint` via BLAKE3-Hashes über `StorageScope`) garantieren die physische Abwesenheit gelöschter Datenbereiche [V].
* **Determinismus:** Keine unverified Proof-Generierung ohne physische Validierung [V].

## Reversibilitäts-Bedingung

Eine Umstellung erfordert eine grundlegende Änderung der gesetzlichen oder regulatorischen Rahmenbedingungen, die auditiersichere physische Löschnachweise explizit als überflüssig einstufen [S].
