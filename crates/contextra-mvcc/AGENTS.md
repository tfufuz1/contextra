# AGENTS – contextra-mvcc

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (8.7, 5.4, 9.5, D-11, D-61, I-12). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** Sequenzen, Snapshots und SSI-Validierung für Mehrversions-Nebenläufigkeit.

**Ring und Abhängigkeiten:** Ring 2. Erlaubt: `types`, `ports`. Trägt `#![forbid(unsafe_code)]`.

## Fachprofil (Arbeitsrolle)

Transaktionsverarbeitungs-Ingenieur (Snapshot Isolation, Serializable Snapshot Isolation nach Cahill und PostgreSQL). Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- Sequenznummern, Sichtbarkeitsregeln, Write-Skew-Erkennung über rw-Abhängigkeiten, Konfliktgranularität, Snapshot-Registry, History-Horizon, Grenzen der Garbage Collection.
- Sentinel-Behandlung: `u64::MAX` darf nie als „alles sichtbar“ wirken und nie als `i64::MAX` enden.
- Sperrordnung, Loom-Modelltests, Proptest gegen ein Referenzmodell.

## Normative Anker

- **`min_read_snapshot` liefert `Option`.** Ohne offene Snapshots gilt `current_seq`, nie „alles sichtbar“ (8.7, I-12). Der frühere Treffer `unwrap_or(u64::MAX)` in `snapshot.rs` (`update_min`) widerspricht der SPEC (F-29).
- **Snapshot-Registry nach `tenant_tag`** (D-61). `revoke_covering` widerruft nur Snapshots des Mandanten des Ziels mit `seq ≤ gated_seq`; Snapshots anderer Mandanten bleiben unberührt (T-SNAP-1). Widerruf ist fail-closed (`SnapshotRevoked`).
- **Zeitreise-Versionen sind Löschziele (D-11):** Leser sehen entweder den vollständigen Altzustand oder `Erased`, nie einen Teilzustand (T-VER-1). `history_horizon` (Standard 24 h) begrenzt die Versionsmenge; seine Bereinigung ist keine Löschaussage (D-51).
- **Sperrordnung (5.4):** Dokumentlock → Commit-Barriere → Purge-Barriere → Journal. Shard-Locks nie verschachtelt. Keine Sperre über `.await` einer fremden Task.
- **Sequenz-Monotonie (I-20, D-74):** `next_seq = max(current_seq, ledger_floor) + 1`.
- Snapshots je Mandant ≤ 1024 (D-87, `Limit::*`).

## Verboten

Sentinel `u64::MAX` als „alles sichtbar“, `unsafe`, Abhängigkeiten zu `sys`, `store`, `vector`, `text`, `engine`, mandantenübergreifender Widerruf, Zeitquellen außerhalb von `Clock`.

## Abnahme

`T-SNAP-1`, `T-VER-1`, `T-CONC-1` (`loom`), `I-12` über `min_read_snapshot`, Write-Skew-Suiten, Proptests gegen das `ReferenceModel`; Miri läuft auf diesem Crate (15.1). Zusätzliche Tests: `tests/mv_snapshot_tests.rs`, `tests/mv_revoke_tests.rs`, `tests/tk_model_mvcc.rs`.

## Arbeitspakete

WP-3.3 (Sentinel-Fix gehört zum Store-Paket), `WP-3.3-mvcc` im Backlog. Statusquelle: `.agents/jules/tasks/BACKLOG.md`.

## Prüfbefehle

```bash
cargo test -p contextra-mvcc --locked
RUSTFLAGS="--cfg loom" cargo test -p contextra-mvcc --locked
cargo +nightly miri test -p contextra-mvcc
```
