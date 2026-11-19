# contextra-erasure — Crate-Spezifikation (Soll-Zustand, gemäß Finaler Löscharchitektur)

**Ebene:** E0 (Fundament / Löschkern)
**Normative Quellen:** `docs/SPEC.md` §4 (Schlüssel), §5 (Löschsaga), §6 (ReadGate), §7 (ErasureLayer), §10 (Barrieren), §12 (Testkatalog), §13 (Invarianten), §17 (offene Punkte). Ferner `docs/Finale Löscharchitektur.md`.
**Status:** Normativ (Soll-Zustand)
**Geltungsbereich:** `crates/contextra-erasure`

Diese Spec ersetzt die Vorfassung. Die Vorfassung beschrieb eine 6-Schritte-Saga mit Schlüsselvernichtung vor Purge. Sie ist überholt.

---

## 1. Zweck

`contextra-erasure` ist der **einzige Löschkern** des Workspace. Er enthält:

- die Saga-Typen, die Zustandsmaschine und den Koordinator (Schritte 1 bis 9),
- den Journal-Port und die Recovery,
- den Lesesperren-Kern (`ReadGate`, `ErasedSet`, `LiveMask`, `Slot`),
- den Messkern (`Scanner<Unarmed>` und `Scanner<Armed>`, `LayerWitness`, `ResidualReport`),
- den Bericht `DeletionProof` V5 und die Signatur über den Port aus `contextra-crypto`,
- die Schnittstelle `ErasureLayer` für Schichten S1–S8.

Schlüsselmaterial, Verschlüsselung und Schlüsselhierarchie liegen **nicht** hier, sondern in `contextra-crypto` (R4). Der Tresor (`FileVault`) entfällt als eigene Schlüsselverwaltung. Seine Funktion geht in `KeyAnchor` (Port) und `contextra-crypto` auf.

## 2. Architekturregeln

1. E0: keine Abhängigkeiten auf E1- oder E2-Crates (`store`, `vector`, `text`, `graph`, `crypto`, `db`, `engine`). `contextra-crypto` ist E1 und wird über den Port `EffaceableKeySlot` bzw. über den Trait `KeyVault` angebunden, nicht direkt importiert. Die Abhängigkeit auf Krypto-Primitive entfällt damit (R4, §4.10 der SPEC).
2. `#![forbid(unsafe_code)]`. Zero-Panic (`unwrap`, `expect`, `panic!`, `todo!` verboten in Nicht-Test-Code).
3. `default = []` (R3). Keine feature-gesteuerten Löschpfade. Ed25519-Signatur ist Pflicht, kein Feature.
4. `ErasureLayer` wird hier definiert. E1-Crates implementieren ihn. Die Definition ist nicht in `contextra-ports`.
5. Kein `ControlEvidence::observed()`-Konstruktor außerhalb dieses Crates. `Scanner<Armed>` ist nur über `Chain::verify` nach bestandener Positivkontrolle erzeugbar (INV-WITNESS-MINT-1, Befund C-06).

## 3. Module

| Modul | Inhalt | SPEC |
|---|---|---|
| `scope` | `ErasureScope` (`TenantId`, `CollectionId`, `ErasureTarget`), `ScopeId` (128 Bit), `ScopeDescriptor` (verschlüsselt unter `K_saga`) | §5.3 Schritt 1 |
| `target` | `EraseTarget` (Document, Subject, Collection, Tenant), kanonische Kodierung | §5.1 |
| `state` | `SagaState` mit Übergangstabelle, `is_terminal`, `blocks_reads`, `Pending(reason)`, `Restricted{basis, until}`, `Partial{outstanding}` | §5.2, INV-9, INV-10 |
| `retention` | `RetentionCode`, `RetentionAttestation`, `blocks_erasure` | §5.3 Schritt 1 |
| `layer` | `ErasureLayer`, `LayerId`, `LayerScope`, `KeyCoverage`, `PurgeCx`, `PurgeOutcome` | §7 |
| `ports` | `Journal`, `GateSink`, `KeyVault`, `DiskReserve`, `PositiveControl`, `ReportSink`, `Signer`, `TimeStamper` | §5.3, §10 |
| `journal` | `ErasureJournal`: `JournalRecord`, `FileJournal` (fsync je Eintrag, Prüfsumme, Torn-Tail-Behandlung), Group-Commit (1–2 ms) | §5.3 Schritt 1, §5.5 |
| `coordinator` | `Chain`: der **einzige** Koordinator. Schritte 1–9, Takt 1 und Takt 2, Single-Flight, Eskalation, Retry | §5.3, §10.4 |
| `epoch` | `EpochSeal`: Re-Wrap, atomares KeyTable-Replace, Crash-Fortsetzung | §4.4, REQ-DEL-N13 |
| `gate` | `ReadGate`, `GateDecision`, `RangeGate`, `ErasedSet`, `LiveMask`, `Slot`, `SagaGate` | §6 |
| `fence` | Dekker-Protokoll (`SeqCst`), `#[must_use]`-Publikationsticket | §10.3 |
| `measure` | `Scanner<Unarmed>`, `Scanner<Armed>`, `ControlEvidence`, `LayerWitness`, `Inventory` (Pre-Destruction), `NeedleScan`, `DecryptProbe`, `CanaryProbe` | §5.8 |
| `report` | `DeletionProof` V5 (kanonisch), `Siegel`, `MeasureClass`, `ResidualReport`, `Certificate` (datensparsam), Außentext-Satzbibliothek | §5.3 Schritt 9 |
| `signing` | Ed25519-Signatur über den Port `Signer`, Zeitanker über `TimeStamper` | INV-PROOF-KEY-1 |
| `restore` | `RestoreGuard`, `ErasedSetMirror`-Port, `restore_reapplied` | §5.11 |
| `recipients` | `RecipientLedger` (ohne Nutzinhalt) | §5.12 |
| `error` | `ErasureError` (typisiert, §11) | §11.1 |

## 4. Zustände (SPEC §5.2)

Kanonische Zustände: `Requested`, `Authorized`, `Frozen`, `LogicallyDeleted`, `PlaintextPurged`, `PlaintextMeasured`, `KeyDestroyed`, `CryptoMeasured`, `EpochSealed`, `Hygiene`, `HygieneMeasured`, `Reported`, `Completed`, `Restricted{basis, until}`, `Partial{outstanding}`, `Pending(reason)`, `Refused`, `Lifted`.

Terminal: `Completed`, `Refused`, `Lifted` (INV-9). `Frozen` und alle späteren Zustände blockieren Lesezugriffe (INV-10). Die Übergangstabelle ist in `SagaState::can_transition_to` normativ kodiert. Ungültige Übergänge sind Fehler, keine stillen No-ops.

Die Umbenennung der Vorfassung lautet: `Gated` → `Frozen`, `KeyRevoked` → `KeyDestroyed`, `Purging` → `PlaintextPurged`, `Verifying` → `PlaintextMeasured` bzw. `CryptoMeasured`.

## 5. Die Saga (Schritte 1–9)

| Schritt | Takt | Code | Durabler Anker | Siegel / Klasse |
|---|---|---|---|---|
| 1 Validierung, Intent, Lesesperre | 1 | `Chain::erase` → `Chain::freeze`: Scope-Auflösung, Retention-Prüfung, `K_saga`-Descriptor, Group-Commit, `ErasedSet` + `LiveMask` mit Fence | fsync (`Frozen`) | L / **Siegel 1** |
| 2 Logische Löschung, Kaskade | 2 | `Chain::logical_delete`: Key-Lock halten, `ErasureManifest` lesen, WAL-Tombstone, Lineage-Kaskade (INV-DERIVED-1), Mixed-Policy | WAL-Commit | L |
| 3 Ebene-A-Bereinigung | 2 | `Chain::purge_plaintext`: Arena-Nullung (`fp32`, `SQ8`), Caches zeroizen, MemTable, Text-Segment-Neuschreibung mit neuem `ArtKey` | Journal vor jedem Teilschritt | P (Vorbereitung) |
| 4 Messung Ebene A | 2 | `Chain::measure_plaintext`: Scanner mit Positivkontrolle `AC-1`, Pre-Destruction-Inventar | Bericht mit Positivkontrolle | P / **Siegel 2** |
| 5 Schlüsselvernichtung | 2 | `Chain::destroy_key`: optionale Karenz (§5.3 SPEC), Journal-Intent J1, `KeyTable`-Replace, `KeyRegistry`-Ereignis | fsync (Intent), fsync (Replace) | `KeyDestroyed` |
| 6 Messung Ebene B | 2 | `Chain::measure_crypto`: Nadelscan, Entschlüsselungsprobe (`KeyAbsent`), Kanarien-Probe | Bericht | C\* (vorläufig) |
| 7 Epoch Seal | 2 | `EpochSeal::run`: `KEK[e+1]`, Re-Wrap, `KeyTable`-Replace, `KEK[e]`-Vernichtung | Journal, fsync | C1 / C2 / C3 / **Siegel 3** |
| 8 Hygiene | Hintergrund | `Chain::hygiene`: Purge-Compaction, WAL-Entfernung mit Pflichtprüfung, Graph-Generation `g+1`, Checkpoint-Sweep, `HygieneMeasured` | Manifest-Commit | M / **Siegel 4** |
| 9 Bericht und Abschluss | — | `Chain::report`: `DeletionProof` V5, Ed25519, Zeitanker, Ledger, `K_saga`-Vernichtung | fsync | Abschluss |

**Reihenfolge ist normativ** (INV-DEL-ORDER-1). Ein Test prüft, dass das Gate aktiv ist, bevor der Schlüssel vernichtet wird (`gate_ist_aktiv_bevor_der_schluessel_vernichtet_wird`). Ein weiterer Test prüft, dass Schritt 3 und 4 vor Schritt 5 liegen (`purge_vor_schluesselvernichtung`).

## 6. Fehlerpolitik (SPEC §11)

- Transiente I/O-Fehler setzen `Pending(IoBackoff)` mit Resume-Ziel und Retry-Backoff. Kein `Ok(verified: false)`.
- Logische Fehler (Rückstand im Scan, fehlende Positivkontrolle, fehlende Schichtunterstützung) lassen den Zustand unverändert und geben den Fehler zurück.
- `ShredVerificationFailed` lässt den Zustand auf `Frozen`.
- `CapabilityUnsupported` für K2 und K3 ohne Konformitätssuite.
- Der Zustand wird erst nach erfolgreichem Journal-Append (`fsync`) geändert.
- Ein nicht durabler Journal-Flush lässt Takt 1 `Err` liefern und setzt keine Lesesperre.
- `erase_sla`-Überschreitung setzt Ticket-Status `Overdue` und erzeugt einen Alarm.

## 7. Recovery (SPEC §5.5)

`Chain::recover` läuft **vor** Freigabe des Lesepfads. Es lädt das Journal, bildet den letzten Zustand jeder Saga, füllt `ErasedSet` und `LiveMask` für alle Scopes in `Frozen`, `Restricted`, `PlaintextPurged` und allen späteren Zuständen. Danach setzt es jede offene Saga an ihrem Resume-Punkt fort. `Chain::resume` setzt eine einzelne Saga fort. `Pending` wird zuerst in den Resume-Zustand überführt.

## 8. Lesesperre (SPEC §6)

- `SagaGate::check` ist O(1), ohne I/O, ohne Blockierung.
- Der Fast-Path ohne aktive Sperre ist atomar und lock-frei (`LiveMask`-Lesen mit `SeqCst`).
- Mit aktiver Sperre liest `check` einen konsistenten Snapshot.
- Präfix-Lookup iteriert über die distinkten Präfixlängen.
- `Restricted` hat Vorrang vor `Erased`.
- Zeitreisen (`as_of`) und offene Snapshots werden gegen den **aktuellen** Stand geprüft.

## 9. Messung und Nachweis (SPEC §5.8)

- `Scanner<Unarmed>` ist ein reiner Beobachter. Er kann nicht messen, sondern nur Positivkontrollen ausführen.
- `Scanner<Armed>` entsteht ausschließlich in `Chain::verify_control`, nach bestandener Positivkontrolle `AC-1`. Der Konstruktor ist `pub(crate)`.
- Jede Pflichtschicht liefert zwei Quellen (D-13): den besitzenden Scanner der Schicht und die Black-Box-Probe des Orchestrators. Eine reine `witness()`-Selbstauskunft genügt nicht.
- Eine Liste ohne Schichten ist ungültig. `verify` ohne mindestens eine gemessene Schicht meldet `NoLayerMeasured`, nie einen leeren Bericht (Befund C-04).
- Der Lesepfad-Test (Schritt 6, Entschlüsselungsprobe) ist eine echte Leseabfrage über den regulären Lesepfad, nicht `gate.decision(target)` direkt nach `activate` (Befund C-07).
- Verification und Validation sind getrennte Zustände im Bericht.

## 10. Bericht und Signatur (SPEC §5.3 Schritt 9)

- `DeletionProof` V5 wird in **kanonischer Kodierung** serialisiert. Mehrdeutige Formate (z. B. `bincode`) sind für signierte Felder verboten.
- Pflichtfelder: Klasse, Siegel 1–4 (verkettet), `legal_state`, `backup_window`, `assumptions[]`, `residuals[]`, Schicht-Digests mit Positivkontrollen, `Custody`, `DestroyAttestation`, Epoch-Seal-Referenz, `subject_ref_hash`, Pflicht-Ausschlüsse, `Verification`- und `Validation`-Status (Concurrence-Signatur im Regulated-Tier).
- Signatur: **Ed25519** über den Port `Signer`. Der Signaturschlüssel ist kein Datenschlüssel (INV-PROOF-KEY-1).
- Zeitanker: optional RFC-3161-Stempel über den Port `TimeStamper`.
- `certificate()` erzeugt die einseitige Bescheinigung ausschließlich aus Berichtsfeldern. Sie enthält keine IDs, keine Dateinamen und keine Inhalte (REQ-DEL-N21).
- Außentexte entstehen nur aus der geschlossenen Satzbibliothek. Jeder Satz ist an einen gemessenen Wert gebunden (INV-CLAIMS-1, Gate `claim-generator`).
- Ein verändertes Feld, ein falsches Siegel oder ein fehlendes Pflichtfeld führt zu einem Fehler bei `doctor verify-report` (REQ-DEL-N9, T-DEL-22).

## 11. Fehler-Hierarchie

`ErasureError` ist `#[non_exhaustive]` und enthält mindestens:

| Variante | Bedeutung | Zustand |
|---|---|---|
| `Io`, `Storage` | transient | `Pending(IoBackoff)` |
| `ScopeNotFound`, `TenantMismatch`, `NotAuthorized` | logisch | unverändert |
| `RetentionBlocks` | Aufbewahrungspflicht | `Refused` |
| `PositiveControlFailed` | Messlauf ungültig | unverändert, Wiederholung |
| `NoLayerMeasured` | Bericht ohne Schichten | unverändert |
| `ShredVerificationFailed` | Vernichtung nicht bestätigt | `Frozen` |
| `KeyAbsent` | Entschlüsselungsprobe (erwartet) | Messergebnis |
| `CapabilityUnsupported` | K2/K3 ohne Suite | unverändert |
| `RestrictedReadAuditRequired` | Lesezugriff ohne Audit | — |
| `JournalNotDurable` | Flush fehlgeschlagen | Takt 1 `Err`, keine Sperre |
| `RestoreBeyondHorizon` | abgelaufenes Schlüssel-Backup | verweigert |
| `UnsupportedSubject` | Subjekt-Löschung ohne Index | fail-closed (siehe §12 Punkt 3) |

## 12. Abweichungen und offene Punkte

Die folgenden Punkte sind bewusst benannt. Sie entsprechen SPEC §17.

1. **Concurrence und Verification-Gate:** Die Concurrence-Signatur ist im Regulated-Tier Pflicht, aber der Tier-Schalter selbst ist offen (SPEC O-K1 und Betriebsentscheidung).
2. **Subjekt-Löschung (T-DEL-9):** Ohne Subjekt-zu-DocDek-Index liefert der `SagaGateSink` für `Subject` fail-closed `NotImplemented`. Der Index muss im Store (`contextra-db`/`contextra-store`) geliefert werden.
3. **Karenz (`erasure_grace`):** Die Rücknahme führt nur nach `Restricted`, nie nach `Lifted`. Die rechtliche Bestätigung steht aus (SPEC O-K1).
4. **Schichten-Implementierungen:** In diesem Crate existieren nur Test-Doubles für `ErasureLayer`. Produktive Implementierungen liegen in E1-Crates und werden in `contextra-db` verdrahtet (INV-WIRING-2).
5. **Canary-Einfügung und -Suche:** Die Speicherschicht liefert den Port `PositiveControl`. Ohne Produktionsimplementierung ist die Messung ungültig.
6. **Krypto-Duplikate:** `crypto_local`, `sha2`, `hmac` im Erasure-Kern sind zu entfernen (R4, Befund C-10, C-13). Die Migration persistierter Formate folgt INV-1.
7. **Ed25519-Implementierung:** Der Port `Signer` ist nötig. Die produktive Implementierung wird in `contextra-crypto` bereitgestellt. Test-Schlüssel dürfen nicht in Produktion gelangen.
8. **Mixed-Policy:** Die Standard-Policy `BlockAll` ist implementiert. `RedactAndRekey` und `DeleteAll` brauchen Betreiber-Eingaben und sind über die Konfiguration gesteuert.

## 13. Testabdeckung (SPEC §12)

| Kennung | Abdeckung |
|---|---|
| T-DEL-1 … T-DEL-6 | Verschlüsselung, Verifikation, Neustart, Klasse ohne Schlüssel, Suche |
| T-DEL-7 | Positivkontrolle `Scanner<Unarmed>` → `Scanner<Armed>` |
| T-DEL-8 | Collection-Drop über `KEK[coll, *]` |
| T-DEL-12 | Crash in Schritt 3, Neustart, Gate sofort aktiv |
| T-DEL-13 | Restrict und Lift, `RestrictedReadAuditRequired` |
| T-DEL-14 | `purge` idempotent |
| T-DEL-15 | `verify_key_absent` nur nach vollständiger Nullung |
| T-DEL-16 bis T-DEL-26 | Siehe SPEC §12. Implementierung über `contextra-testkit` (Erasure Lab) |

Pflicht-Negativtests: Ohne Positivkontrolle ist die Messung ungültig. Ein Bericht ohne gemessene Schicht wird abgewiesen. `Scanner<Armed>` ist ohne Positivkontrolle nicht konstruierbar (Compile-Fehler-Test).
