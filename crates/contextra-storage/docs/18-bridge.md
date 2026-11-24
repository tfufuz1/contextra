# SP-BRIDGE — Asynchrone Fassade und Altstand-Adapter

| | |
|---|---|
| Crate | `contextra-storage-bridge` |
| Besitzt | Tokio/Async-Adapter, Blocking-Pool, Altstand-API-Abbildung, Signatur und Cutover-Kompatibilität |
| Konsumiert | ausschließlich öffentliche Storage-Fassade + PORTS; keine internen Module |
| Spec-Bezug | R8 §3.4, §12.8, §14; E-29–E-32, E-55, A-10 |
| Phase | P6–P8 (G-P) |

## 1 Auftrag
Bridge darf async sein, Kern bleibt synchron. Sie ist ein zeitlich begrenzter API-/Async-Übergang zu `contextra-storage`, keine dauerhafte `contextra-store`-Kompatibilitätsschicht und kein zweiter Persistenzkern. Migrations-I/O zum Lesen alter Daten gehört in das isolierte Werkzeug SP-CUTOVER, nicht in den Storage-Core.

## 2 Verträge
Alle `apply`-Aufrufe benötigen `IdemToken`; Cancellation nach Übergabe wird mit demselben Token wiederholt und ergibt `AlreadyApplied` oder den gespeicherten terminalen Status (z. B. `Erased`), nie ein zweites Apply. Eigener Blocking-Pool; kein Runtime-Aufruf aus Kern. Legacy KV erhält `LEGACY_KV` Space und Plain-Cluster je Tenant; personenbezogene Werte werden abgewiesen/ausgeschlossen (E-32). Rollback wird über temporären Branch modelliert oder `Unsupported` (E-31). MergeOperator entfällt; Read-Modify-Write verwendet Version-Guard + Retry (E-29).

Die Bridge mappt das kanonische R8-`ErasureReceipt` plus verifizierte Extension-/Anchor-Atteste in eine versionierte v15-`DeletionProof`. Signiert wird ausschließlich nach erfolgreicher Offline-Verifikation und vollständigem Witness-Mapping. Ein fehlender erforderlicher Layer-Witness, ausstehender HNSW-Rewire oder fehlende `Rebuildable`-Attestation ergibt `Incomplete`; das ist kein erfolgreicher Löschbeleg. `CompleteWithResidual` darf nur eine abgeschlossene Löschung mit ausdrücklich dokumentiertem, außerhalb des Vertrags liegendem Restrisiko abbilden, niemals fehlende Pflicht-Evidence. Storage-lokales `Done` wird nicht als gesamtsystemweites `Complete` ausgegeben.

## 3 Regeln
Keine direkte Abhängigkeit auf `contextra-store` Implementierungsdetails; Adapterfehler nennt Migrationshinweis, keine Dateninhalte. Cancellation darf Receipt nicht verschlucken. Blocking-Pool-Rückstau wird begrenzt und surfaced.

## 4 Invarianten / Umsetzung
INV-ATOM-1, INV-COMMIT-2, INV-BR-2. Test cancellation/retry nach Retire, duplicate/abweichender IdemToken, Guard-Konflikt, Altstand golden mapping, Thread-Pool-Sättigung, Receipt-/DeletionProof-Vektor mit fehlenden/ungültigen Attesten.

## 5 Arbeitspakete
WP-18.1 Blocking executor + idempotent async apply (P6) · WP-18.2 Key/space mapping (P6) · WP-18.3 transaction/rollback mapping (P7) · WP-18.4 API/signature + DeletionProof bridge (P8) · WP-18.5 differential cutover audit (P8). Abnahme: Differentialdatenvergleich, Proof-Verify-Vektoren und dokumentierte Unsupported-Matrix.

## R8.2-Nachtrag

**Spec-Bezug ergänzt:** R8.2 §9.7, §13.2.1; E-77, E-80.

Der `DeletionProof` (v15) übernimmt aus dem `ErasureReceipt` Status, Statuscaps (`BoundedBackupKeys`, `WeakKeyAssurance`, Host-/Geräterest) und `MeasureKind` je Messung unverändert; die Bridge darf einen Cap weder entfernen noch einen `Declared`-Befund als „verifiziert“ formulieren. Neue Fehlerklassen (`SnapshotTooOld`, `ErasePending`, `BackupBeyondHorizon`, `LedgerUnavailable`, `LedgerSameDevice`, `ProviderAssuranceInsufficient`) werden 1:1 auf Bridge-Fehler abgebildet. Keine neuen Arbeitspakete; die Abbildung wird in WP-18.4 mitgeführt.

## R9.1-Synchronisierung

Die Bridge darf `KeysDestroyed` nicht als physisches oder systemweites `Done` umdeuten: Ciphertext-Entfernung, Rebuildable-Owner und alle Atteste bleiben Voraussetzungen. Alle Leistungswerte sind Designhypothesen; die Pipeline- und Admission-Nachweise werden im Kern geführt und dürfen nicht als gemessene Bridge-Eigenschaft ausgegeben werden.
