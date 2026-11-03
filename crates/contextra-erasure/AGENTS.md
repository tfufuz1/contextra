# AGENTS – contextra-erasure

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (8.12, 11, 12, 13.1, Anhang D und F.3, F.9, F.10). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** Löschkern: Saga, Gate, Journal, Tresor, Messung, signierter Bericht.

**Ring und Abhängigkeiten:** Ring 3. Erlaubt: `types`, `ports`, `crypto` (AR-3). Keine Ring-2-Crates; Schichten werden nur über `ErasureLayer` angesprochen (P12, D-34). Budget Erase-Produktcode ≤ 9.000 Zeilen (5.8).

## Fachprofil (Arbeitsrolle)

Principal Engineer für Löschprozesse in Datenbanken (Saga und Prozessmanager, Crash-Recovery, Schlüssellebenszyklus). Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- Saga und Zustandsautomat mit Intent-Journal als einziger Wahrheit; Idempotenz, Wiederanlauf aus jedem Zustand, `Pending` mit Grund; das Journal selbst trägt keine Löschziel-Inhalte (P13).
- Tresor auf Platte: Slot-Generationen, Grabstein, Verzeichnis-`fsync`; DEK-Cache-Eviction im Gate-Schritt; Hierarchie `T`/`C`/`S`/`D` (Vernichtung einer höheren Ebene macht alle darunter nicht mehr ableitbar).
- Gate als Lesesperre für alle Lesepfade, damit Leser nie Daten eines Ziels sehen, dessen Schlüssel weg ist.
- Messzeugen und Bericht: echte Positivkontrolle, kanonischer signierter Bericht mit Restrisiko-Liste.

## Normative Anker

- **Saga (11.2):** `Requested → Gated → KeyDestroyed → Purging → Verifying → Completed`, jederzeit `Pending(reason)`; `Refused` terminal vor dem Gate. Jeder Schritt idempotent (I-15). Normalpfad bis `Gated`: **4 `fsync`** (Journal `Requested+Intent`, Gate-Log, Tresor-Slot nullen, Journal `KeyDestroyed`; D-58); gleichzeitige Anfragen teilen eine Barriere-Phase und einen Gate-Log-`fsync` (D-78). `PendingReason`: `Io`, `SlaExceeded`, `LayerFailed(id)`, `WitnessMissing`, `DiskReserve`.
- **Nur die Schlüsselvernichtung** (Slot der höchsten Ebene genullt, `sync_data`, Fehlen verifiziert; D-79, T-VAULT-1) liegt auf dem O(1)-Pfad; alle Folgephasen (Purge je Schicht) sind wiederholbar und zielgrößenabhängig. Fehler setzen `Pending`, nie stilles Weiterlaufen (P2).
- **Gate (11.3, D-57):** Varianten `Erased` und `Restricted`; Intervallstruktur im Speicher, `gates.log` (AEAD, Hash-Kette). Gate-Log wird **vor** den Lesepfaden geladen (I-2). `Restricted` vernichtet nichts und wird nie verworfen (I-18, T-RESTR-1).
- **Tresor (11.5, F.3):** plattenresidente Open-Addressing-Tabelle aus 80-Byte-Slots (`T`, `C`, `S`, `D`, `F`), Lastfaktor ≤ 0,7, kein vollständiger RAM-Index, Waisen werden bei der Recovery genullt (D-83, T-CRASH-5). Reihenfolge I-9, I-9b, I-17.
- **Journal (11.6):** Hash-Kette, AEAD, `fsync` je Eintrag; nur `ErasureId`, Zustand, Zeit, Commitments (`K_commit(epoch)`), Zähler, Zeugen-Hashes, Retention-Code, Bereichs-Tags. Nach `Completed` reduziert (R13).
- **Messung (12):** `LayerWitness` nur aus `Scanner<Armed>`, der nur durch eine erfolgreiche Positivkontrolle entsteht (I-19, D-72). `matches ≠ 0` ⇒ `Pending(WitnessMissing)`. Die Feldmessung ist Selbstattestierung; unabhängig ist nur Lab oder `ctx-verify scan`.
- **Bericht (11.7, F.10):** kanonisches JSON, Ed25519 (`K_sign` abgeleitet), `field_measurement` getrennt von `lab_evidence`, Klassenverband G/K/P ohne Gesamtnote, `key_coverage` je Schicht, Restrisiken nach Anhang D, Pflichtsatz zur Signatur (E-L11), Hinweis „keine Rechtsberatung“. Keine Inhalte, Schlüssel oder Klartext-Kennungen.
- **Ledger und Restore-Guard (11.8, D-74):** `(commitment, range_set, gated_seq, completed_at)` außerhalb des Datenverzeichnisses über `LedgerSink`; Lebensdauer `backup_retention`; `next_seq = max(current_seq, ledger_floor) + 1` (I-20).
- **Aussagen:** nur der Wortlaut aus SPEC 6.4. Fehlt eine Schicht im Code, steht sie im Bericht als „nicht erfasst“.

## Entscheidungshilfen

- Neue Zeile in der Saga? Zuerst die Crash-Tabelle „Abbruch nach Schritt n → Zustand nach Recovery“ ergänzen, dann den Test, der jede Zeile erzeugt (FM-1 bis FM-14).
- Fertiger Teil „grün färben“? Nein: Positivkontrolle und Zeugen dürfen nie simuliert werden (HR-10, P1).
- Ein-Zeilen-Stubs sind ein Defekt-Backlog, kein Sollzustand.

## Verboten

Abhängigkeiten zu `store`, `text`, `vector`, `engine` (AR-3); Code aus `lab` oder `ctx-verify` (AR-4); Löschstatus ohne Zeugen (P1); Berichte, Journal oder Ledger mit Inhalten, Schlüsseln oder Klartext-Kennungen (P13); `Completed` ohne gültige Zeugen aller Schichten (I-4); Locks pro Dokument für Subject-, Collection- oder Tenant-Ziele (D-57).

## Abnahme

`T-GATE-1`, `T-GATEGROUP-1`, `T-VAULT-1` bis `T-VAULT-5`, `T-KEYCOV-1`, `T-FK-1` bis `T-FK-3`, `T-RAW-1` bis `T-RAW-6`, `T-PROOF-1`, `T-RESTORE-1` bis `T-RESTORE-3`, `T-MUT-1`, `T-CRASH-1` bis `T-CRASH-6`, `T-RESTR-1`, `T-CLASS-1`, `T-JRN-3`, `T-REPORT-1`, `T-SLA-1`; Abdeckung ≥ 90 % (Entscheidungslogik ≥ 95 %); `cargo-mutants` ohne Überlebende (Gate G-D); `BM-ERASE-LAT`, `BM-ERASE-O1`. Ein Kill an jedem `fsync`-Punkt konvergiert zu Vollendung oder `Pending`.

## Arbeitspakete

WP-3.1 (Skelett als Workspace-Mitglied, D-96), WP-4.1 (Tresor), WP-4.2 (Gate-Log, Saga, Journal, Recovery), WP-4.3 (Messung, Bericht, Ledger). Statusquelle: `.agents/jules/tasks/BACKLOG.md`.

## Prüfbefehle

```bash
cargo test -p contextra-erasure --locked
cargo test --locked --test tk_conformance_read_gate --test tk_conformance_record_sealer --test tk_conformance_file_keyring --test tk_conformance_ledger_sink
cargo mutants -p contextra-erasure
cargo test -p contextra-arch-tests --locked
```
