# contextra-mvcc — Crate-Spezifikation (Soll-Zustand, SPEC 2.1)

**Ebene:** E1 (Domäne)
**Domänen-Closure:** `ids`, `env`
**Verträge:** `erasure` (`SnapshotRevoker`). Zählt nicht gegen die fünf (SPEC §15).
**Status:** Normativ (Soll-Zustand)
**Version:** 2.1
**Geltungsbereich:** `crates/contextra-mvcc`
**Normative Quellen:** `docs/SPEC.md` §6.2, §7 (`SnapshotRevoker`), §9, §5.3 Schritt 8 (`PendingDelete`).

---

## 1. Zweck

`contextra-mvcc` führt Snapshot-Isolation über `Seq`.

- `SnapshotRegistry` vergibt RAII-Pins und liefert `min_active_seqno`.
- `SnapshotRevoker` implementiert `ErasureLayer`. Er schließt Zeitreisen in Schritt 1 aus, indem jede spätere Prüfung den aktuellen Gate-Stand sieht.
- `TxBuffer` hält Schreibvorgänge einer Transaktion, bis der Commit in das WAL übergeben wird. Den Commit führt `contextra-db` aus.

`contextra-store` importiert dieses Crate nicht. `contextra-db` liest `min_active_seqno` und übergibt die Schranke an `PendingDelete`.

## 2. Architekturregeln

1. `#![forbid(unsafe_code)]`. Zero-Panic. `default = []`.
2. Keine Abhängigkeit auf ein anderes E1-Crate. Keine Dev-Abhängigkeit auf `contextra-testkit`.
3. Offene Pins blockieren Takt 1 nicht (SPEC §6.2).
4. `as_of` ändert die Gate-Entscheidung nicht (SPEC §9.2). „Erase gewinnt gegen Zeitreisen."
5. `min_active_seqno` benutzt kein `unwrap` und kein `unwrap_or` auf einer leeren Pin-Liste als Panic-Pfad. Die leere Liste ergibt `current_head`.

## 3. Pins

`acquire(seq)` liefert `SnapshotPin`. `Drop` trägt den Pin aus. `seq` ist monoton. Ein neuer Pin unterhalb der schon vergebenen Kopf-Sequenz ist `MvccError::SequenceRegressed`.

`min_active_seqno(current_head)` ist das Minimum der aktiven Pins oder `current_head`, wenn kein Pin hält.

Ein Pin bleibt gültig, bis er fällt. Er macht gelöschte Daten nicht lesbar. Jeder Lesezugriff über den Pin läuft in `contextra-db` durch `TenantScopedStorage` und damit durch das aktuelle `ReadGate`.

## 4. `SnapshotRevoker` (Schritt 1)

`on_gate` merkt den Scope und die `Seq` des `Frozen`-Zustands. Ab dieser `Seq` gilt:

- `validate` eines Pins, dessen Sicht den Scope enthalten würde, prüft den Scope nicht selbst gegen Klartext. Es liefert dem Aufrufer die Pflicht, `ReadGate::check` auszuführen. Ein Pin wird in Schritt 1 nicht verworfen, nur damit `PendingDelete` die Dateilebensdauer weiter kennt.
- Zeitreisen mit `as_of` kleiner als die Lösch-Sequenz sehen den Scope trotzdem als gesperrt, weil der Gate-Stand der aktuelle ist.

`purge` des Revokers ist ein No-op. Die physische Freigabe alter Dateien ist `PendingDelete` in `contextra-store`, gesteuert über `min_active_seqno`. `witness` ist lesend und bestätigt, dass der Scope in der Revocation-Tabelle steht. Die Klasse vergibt der Revoker nicht.

`revoke_before`, das Pins löscht, damit ein Purge nicht wartet, gibt es nicht. Das Warten auf Pins ist die Aufgabe von `PendingDelete`, nicht ein Grund, die Lesesperre zu verschieben.

## 5. Zwei Zeitachsen (SPEC §9.1)

Dieses Crate besitzt die Transaktionszeit (`Seq`, `as_of`). Die Gültigkeitszeit `valid_at` (`i32`, Tage seit 1970-01-01) trägt der Aufrufer in `contextra-db`. `None` bleibt „offen" und ist nicht `0`. Beide Achsen sind kombinierbar. Die Gate-Prüfung hängt von keiner der beiden ab.

## 6. Fehler (`MvccError`)

`#[non_exhaustive]`: `SequenceRegressed`, `WriteConflict`, `SerializationFailure`, `PinPoisoned`. `SnapshotRevoked` als eigener öffentliche Grund, der sich von einem normalen Lesefehler unterscheidet, wird auf dem Lesepfad nicht ausgegeben. Der Lesepfad liefert `ContextraError::Erased`.

## 7. `Cargo.toml`

```toml
[package]
name = "contextra-mvcc"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Snapshot pins and the erasure snapshot revoker"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-erasure = { workspace = true }
parking_lot = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
proptest = "1.4"

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
```

## 8. Tests

| Kennung | Pflicht |
|---|---|
| T-DEL-11 | Ein `as_of` vor der Löschung liefert denselben Gate-Entscheid wie ein aktuelles `get` |
| T-DEL-12 | Ein gehaltener Pin verzögert `Frozen` nicht |
| §9.2 | `min_active_seqno` bleibt der älteste Pin, auch nachdem `on_gate` lief |
| INV-4 | Die leere Pin-Liste ergibt `current_head` ohne `unwrap` |
