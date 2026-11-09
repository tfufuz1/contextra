# Spezifikation: `contextra-erasure` (SPEC v4.0 §6.2)

> **Klasse A (Autarker Löschkern)** · Reifegrad: `beta` · `#![forbid(unsafe_code)]`

## 1. Zweck und Rolle
`contextra-erasure` ist der zentrale, autarke Baustein für die Datenbereinigung, Nachweiserbringung und Schlüsselvernichtung im Gesamtsystem. Datenhaltende Bausteine bieten lediglich neutrale Schnittstellen (`insert`, `remove`, `search`, `Maintain::compact()`, `inspect`). Sämtliche Transaktions-Rollbacks bezüglich DSGVO/Erasure, Saga-Zustandsmaschinen, Messaggregationen, Ed25519-signierte `DeletionReport`s, `RevocationLog`s und kryptografisches Shredding werden ausschließlich hier geführt.

## 2. Invarianten (Normativ)

- **E-04 / Duale Löschwirkung:** Ebene A (physische Bereinigung / Purge) und Ebene B (Schlüsselvernichtung / Vault Tombstone) wirken nur gemeinsam. Shredding allein erzeugt nie eine vollwertige Löschbehauptung.
- **INV-WITNESS-MINT-1 (Gesperrter Mint-Pfad):** Ein `ScanWitness` entsteht ausschließlich über den geschlossenen Pfad `ScanSession::finish`. Direkte Konstruktion von Witnesses außerhalb verifizierter Scans ist verboten. Scans mit `scanned == 0` oder nicht erfüllter Positivkontrolle (`control_matches == 0`) schlagen fehl.
- **INV-CLASS-MEASURED-1:** Die dokumentierte Messklasse (z. B. Mk1–Mk3) darf nie höher sein, als die tatsächlich durchgeführte Messung trägt.
- **INV-BOOT-1 / Lesesperre:** Das `ReadGate` sperrt Lesezugriffe auf betroffene Einheiten sofort ab (`GateLog`), bevor Bereinigungs- oder Vernichtungsschritte starten.
- **INV-NO-RESURRECTION-1:** Nach Start und Wiederanlauf (`RestoreGuard`) darf kein lebender Schlüssel einer gelöschten Einheit rekonstruiert werden.
- **INV-DEL-ORDER-1:** Schlüsselvernichtung erfolgt erst nach verifizierter Bereinigung und durablem Inventar (außer im expliziten Notfallpfad).
- **INV-FSYNC-1 / Vault Poisoning:** Jeder I/O- oder `fsync`-Fehler beim Überschreiben oder Nullen von Vault-Slots vergiftet den entsprechenden Shard-Handle permanent (Fail-Closed).

## 3. Saga-Zustandsmaschine

Die Saga durchläuft deterministische und idempotente Zustände:
1. `Requested`: Löschauftrag erfasst und im Journal persistiert.
2. `Gated`: Lesezugriff via `ReadGate` verriegelt (`GateLog`).
3. `Purging`: Physische Bereinigung in den Datenhaltern über `Purger`.
4. `Measured`: Unabhängige Überprüfung verbliebener Restdaten über `LayerScanner`.
5. `KeyDestroyed`: Überschreiben und Nullen der Schlüssel im `FileVault`.
6. `Completed`: Ausstellung des signierten Berichts (`DeletionReport` / `DeletionProof`).
7. `Restricted`: Alternativer Zustand bei aktiven Aufbewahrungspflichten (Retention Matrix).

## 4. Kanonische Schnittstellen (Einzige Definition)

Folgende Typen und Traits sind autark in `contextra-erasure` definiert:
- `Purger`: Trait für physische Bereinigungsaktionen (`purge`).
- `LayerScanner`: Trait für Scanner zur Messung von Restbeständen (`scan`).
- `KeyScope`: Bereichsbeschreibung (`Tenant`, `Collection`, `Document`).
- `WitnessLayer`: Identifikator für physische Schichten (`LsmMemtable`, `SsTableAllLevels`, `HnswIndex`, `WalAllSegments`, etc.).
- `ErasureId`: 16-Byte eindeutige Kennung einer Löschung.
- `PurgeOutcome`: Statistiken über bereinigte Datensätze und freigegebene Bytes.
- `ScanSession` & `ScanWitness`: Zustand und manipulationssicherer Zeuge der Messung.

## 5. Berichtsformat V5 & Ed25519-Signatur

Der kryptografische Nachweis (`DeletionProof` / `DeletionReport`) wird über kanonisches JSON serialisiert und mittels Ed25519 signiert.
Dritte (Auditoren, Aufsichtsbehörden) können Berichte standalone ohne Datenbankzugriff validieren (`DeletionProofVerifier::verify_standalone`).

## 6. Fehlermodell

Alle Fehler werden typisiert im `ErasureError` geführt:
- Keine Panics im Produktionscode (Zero-Panic, B9).
- I/O-Fehler führen zu Fail-Closed-Verhalten.
