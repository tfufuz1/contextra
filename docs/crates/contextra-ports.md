# contextra-ports — Crate-Spezifikation

**Ebene:** E2 (Integration)  
**Bindung:** `docs/SPEC.md` §2, §5, §6, §7, §10

---

## 1 Zweck & Domäne

`contextra-ports` definiert die Trait-Schnittstellen (Ports) für Inversion-of-Control zwischen dem Löschkern (`contextra-erasure`), den Speicherschichten (`store`, `vector`, `text`, `graph`) und der Orchestrierungsebene (`db`).

## 2 Definierte Port-Traits

1. **`ErasureLayer` (`erasure.rs`):** Schnittstelle für alle Subsysteme, die Daten physisch löschen und verifizieren (`resolve`, `on_gate`, `purge`, `witness`).
2. **`ReadGate` (`gate.rs`):** Hot-Path Lesesperre (`check`, `check_range`).
3. **`CommitBarrier` & `PurgeBarrier` (`barrier.rs`):** Asynchrone Synchronisationsprimitive zur Isolation von Gates und Kompaktionsläufen.
4. **`FileKeyring` & `RecordSealer` (`keys.rs`):** Abstraktion für Tresor- und Versiegelungsdienste.
5. **`LedgerSink` (`ledger.rs`):** Schnittstelle für das Append-Only Löschledger außerhalb des Datenverzeichnisses.
6. **`ErasureObserver` (`observer.rs`):** PII-freie Ereignisbenachrichtigung bei Phasenübergängen.
7. **`LayerScanner` & `LayerWitness` (`scanner.rs`):** Messungs- und Positivkontrollschnittstellen. Kanonischer Definitionsort für `LayerWitness` (INV-WITNESS-MINT-1).

## 3 Regeln

- Keine direkten Abhängigkeiten zu konkreten Implementierungs-Crates (`store`, `db` etc.).
- Alle Traits sind `Send + Sync`.
- `#![forbid(unsafe_code)]`.
