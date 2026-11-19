# contextra-mvcc — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §9 und `docs/crates/contextra-mvcc.md`. **Ausgangslage:** MVCC hat E2-Optionalabhängigkeiten und mehrere Produktions-Panikmuster, statt sich auf Snapshots und Widerruf zu beschränken.

1. Begrenze auf `ids, env`, Pins/Snapshot-Registry und `SnapshotRevoker`; entferne Types/Ports-Compat und fachliche Transaktions-/Storage-Brücken.
2. Modellieren die unabhängigen Zeitachsen `as_of` und `valid_at` ohne Gate-Ausnahme; Pinning darf keine gelöschten Daten wieder sichtbar machen.
3. Implementiere atomaren Revoker: alle Pins/Snapshots bis Zielsequenz werden ungültig, neue Leseversuche ergeben `SnapshotRevoked`/`Erased`.
4. Ersetze Produktions-Panics durch `MvccError`; prüfe Sequenzüberlauf, Sperrreihenfolge und Drop-/Reaper-Wege.
5. Ergänze Loom- und Property-Tests für concurrent pin/revoke, SSI-Determinismus und Recovery des Sequenzlogs.

**Abnahme:** Revocation-vor-Read-Test, Z-T5-Zeitreise-Test, Loom-Suite, keine höhere Abhängigkeit und Zero-Panic-Gate.
