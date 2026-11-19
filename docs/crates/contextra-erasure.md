# contextra-erasure — Crate-Spezifikation (Soll-Zustand, abgeleitet aus SPEC)

**Ebene:** E0 (Fundament / Löschkern)
**Normative Quelle:** `docs/SPEC.md` §5 (Löschkette), §6 (ReadGate), §7 (ErasureLayer), §12 (Testkatalog), §13 (Invarianten). Bei Widerspruch gilt die SPEC.
**Status:** Normativ (Soll-Zustand)
**Geltungsbereich:** `crates/contextra-erasure`

Dieses Dokument ersetzt die frühere Crate-Spec mit der alten Löschkette (`SagaState::KeyDestroyed`, `EraseTarget::KeyRange`, `ErasureCore`, `DeletionProof`, Purge-Window, WAL-Journal). Diese Teile existieren nicht mehr.

---

## 1. Zweck

`contextra-erasure` ist der Löschkern: die 6-Schritte-Löschkette mit Journal und Recovery, der In-Memory-ReadGate, der Tresor für Schlüssel-Vernichtung, die Messung über `Scanner<Armed>` und die Schnittstelle `ErasureLayer` für E1-Schichten. Es ist die einzige Löschmethode des Workspace.

## 2. Architekturregeln

1. E0: keine Abhängigkeiten auf E1-/E2-Crates (`store`, `vector`, `text`, `graph`, `crypto`, `types`, `ports`).
2. `#![forbid(unsafe_code)]`, Zero-Panic (`unwrap`, `expect`, `panic!`, `todo!` verboten in Nicht-Test-Code).
3. `default = []` (R3). Keine Feature-gesteuerten Löschpfade.
4. ErasureLayer wird hier definiert, E1-Crates implementieren ihn (SPEC §7).

## 3. Module

| Modul | Inhalt | SPEC |
|---|---|---|
| `chain::target` | `EraseTarget` (Document, Subject, Collection, Tenant), kanonische Kodierung, Vault-Tag | §5.1 |
| `chain::state` | `SagaState` mit Übergangstabelle, `is_terminal`, `blocks_reads`, `PendingReason` | §5.2, INV-9, INV-10 |
| `chain::retention` | `RetentionCode`, `RetentionAttestation`, `blocks_erasure` | §5.3 Schritt 1 |
| `chain::layer` | `ErasureLayer` (id, key_coverage, resolve, on_gate, purge, witness), `LayerScope`, `PurgeCx`, `PurgeOutcome` | §7 |
| `chain::ports` | Ports `Journal`, `GateSink`, `KeyVault`, `DiskReserve`, `PositiveControl`, `ReportSink`; `CommitBarrier` | §5.3, §10.1 |
| `chain::journal` | `JournalRecord`, `FileJournal` (fsync pro Eintrag, Prüfsumme, Torn-Tail-Behandlung) | §5.3 fsync-Anker, §5.5 |
| `chain::saga` | `Chain`: Schritte 1–6, Restrict/Lift, Eskalation, Recovery | §5.3, §5.5, §5.7 |
| `chain::adapters` | `SagaGateSink` (Textschlüssel-Layout), `FileVaultKeys` (Tresor, Konstantzeit-Prüfung) | §5.3 Schritt 3, §6 |
| `gate::saga_gate` | `SagaGate`, `ReadGate`, `GateDecision`, `RangeGate` | §6.3 |
| `vault` | `FileVault`: Slots, Tombstones, Shards, Durable-Schicht | §4, §5.3 Schritt 3 |
| `types_local` | Messtypen `Scanner`, `ControlEvidence`, `LayerWitness`, `LayerId`, `KeyCoverage`, `MeasureClass`; VFS-Port; `OperatorRef` | §6, §7 |
| `error` | `ErasureError` | §11.1 |

## 4. Zustände (SPEC §5.2)

Ein Datum durchläuft `Requested → Gated → KeyRevoked → Purging → Verifying → Completed`. Ein Art.-18-Zweig läuft über `Requested → Restricted → Gated` (Eskalation) oder `Restricted → Lifted`. Aufbewahrungspflicht führt zu `Refused`. Jeder nicht-terminale Zustand kann nach `Pending(reason)` gehen und kehrt in den gespeicherten Resume-Zustand zurück.

Die Übergangstabelle ist in `SagaState::can_transition_to` normativ kodiert. Terminal sind `Completed`, `Lifted`, `Refused` (INV-9). `Gated` ist irreversibel (INV-10).

## 5. Die 6 Schritte (SPEC §5.3) und ihre Umsetzung

| Schritt | Code | Durabler Anker |
|---|---|---|
| 1 Validierung & Intent | `Chain::erase`: Retention-Prüfung, keyed-BLAKE3-Duplikaterkennung, `resolve` aller Schichten | fsync #1 (`Requested`) |
| 2 Gate | `Chain::gate_step`: exklusive `CommitBarrier`, `GateSink::activate`, bei Restricted zusätzlich `lift_restricted` | fsync #2 (`Gated`) |
| 3 Schlüsselvernichtung | `Chain::shred_step`: `on_gate` aller Schichten, `KeyVault::destroy`, `verify_absent`, dann `KeyRevoked` | fsync #3 (`KeyRevoked`) |
| 4 Purge | `Chain::purge`: Plattenreserve, Purge in Prioritätsreihenfolge, idempotent | Übergang `Verifying` |
| 5 Messung | `Chain::verify`: Positivkontrolle (`PositiveControl`), Witnesses mit `Scanner<Armed>`, Lesepfad muss `Erased` liefern | — |
| 6 Bericht & Abschluss | `ReportSink::publish` (signierter Bericht und Ledger, fsync #4), dann `Completed` | fsync #4 |

Die Reihenfolge ist normativ und wird im Code durch die Aufrufreihenfolge erzwungen. Ein Test prüft, dass der Gate vor der Schlüsselvernichtung aktiv ist (`gate_ist_aktiv_bevor_der_schluessel_vernichtet_wird`).

## 6. Fehlerpolitik (SPEC §11)

- Transiente I/O-Fehler (`Io`, `Storage`) setzen `Pending(IoBackoff)` mit Resume-Ziel.
- Logische Fehler (Rückstand im Scan, fehlende Positivkontrolle, fehlende Gate-Unterstützung) lassen den Zustand unverändert und geben den Fehler zurück.
- `ShredVerificationFailed` lässt den Zustand auf `Gated`.
- Der Zustand wird erst nach erfolgreichem Journal-Append (fsync) geändert.

## 7. Recovery (SPEC §5.5)

`Chain::recover` lädt das Journal, bildet den letzten Zustand jeder Saga und aktiviert sofort das Gate für `Gated`, `KeyRevoked`, `Purging`, `Verifying` und `Completed` (Erased) sowie `Restricted`. Offene Sagas werden zurückgegeben. `Chain::resume` setzt eine Saga fort, `Pending` wird zuerst in den Resume-Zustand überführt.

## 8. Lesesperre (SPEC §6)

`SagaGate::check` ist O(1) ohne I/O. Der Fast-Path ohne aktive Sperre ist atomar und lock-frei. Mit aktiver Sperre nimmt `check` ein Read-Lock und liest einen konsistenten Snapshot. Präfix-Lookup iteriert über die distinkten Präfixlängen. Restricted hat Vorrang vor Erased.

## 9. Testabdeckung (SPEC §12)

| Kennung | Abdeckung |
|---|---|
| T-DEL-4 | `t_del_4_nach_neustart_bleibt_completed_datum_erased` |
| T-DEL-8 | Collection-Gate über Präfix, Mandanten-Test `mandanten_loeschung_deckt_alle_dokumente_des_mandanten_ab` |
| T-DEL-12 | `t_del_12_crash_in_phase_3_neustart_setzt_fort_mit_aktivem_gate` |
| T-DEL-13 | Restrict/Lift-Tests; `RestrictedReadAuditRequired` existiert |
| T-DEL-14 | `t_del_14_purge_ist_idempotent` |
| T-DEL-15 | `t_del_15_verify_key_absent_nur_nach_vernichtung` |
| Positivkontrolle (T-DEL-7) | Gemessen über `ohne_positivkontrolle_ist_messung_ungueltig` (Gegenprobe) |

## 10. Abweichungen und offene Punkte

Die folgenden Punkte weichen vom SPEC-Wortlaut ab oder sind nicht umgesetzt. Sie sind bewusst benannt:

1. **Vault-Vernichtung:** Der Tresor nutzt seine Tombstone-Semantik (D-111/D-113) statt Zufalls-Überschreiben plus Nullen (§5.3 Schritt 3.3/3.4). Die Prüfung `verify_absent` bestätigt die Abwesenheit.
2. **Duplikaterkennung:** keyed BLAKE3 statt HMAC (BLAKE3 ist bereits Abhängigkeit).
3. **Subjekt-Löschung (T-DEL-9):** Ohne Subjekt-zu-Schlüssel-Index liefert `SagaGateSink` für `Subject` fail-closed `NotImplemented`.
4. **Positivkontrolle und Canary (§5.3 Schritt 5.1/5.3):** Die Kette konsumiert den Port `PositiveControl`. Das Einfügen und Suchen des Canary muss die Speicherschicht liefern.
5. **Signierter Bericht und Ledger (§5.3 Schritt 6):** Es gibt nur den Port `ReportSink`. Eine Ed25519-Implementierung mit hash-verkettetem Ledger außerhalb des Datenverzeichnisses fehlt.
6. **Krypto-Duplikate:** `crypto_local`, `sha2` und `hmac` liegen noch im Tresor-Kern. Die SPEC verlangt Krypto-Primitive ausschließlich in `contextra-crypto` (R4). Die Verschiebung ist offen, weil sie persistierte Tresor-Formate (INV-1) berührt.
7. **Schichten-Implementierungen:** In diesem Crate existieren nur Test-Doubles für `ErasureLayer`. Die E1-Implementierungen liegen in den jeweiligen Crates.
8. **Schlüssel-Layout:** `TextKeyLayout` (`t:{tenant}:{collection}:chunk:{doc}`) ist eine Konvention. Der Store muss dasselbe Layout verwenden.
9. **Cargo.lock:** Nach Entfernung der Abhängigkeiten ist die Lock-Datei des Workspace zu aktualisieren.
