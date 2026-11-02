# Task-Karten

Nicht normativ. Eine Task-Karte beschreibt ein abgegrenztes Arbeitspaket für Jules. Sie setzt Scope und Grenzen fest und verweist auf die SPEC.

## Pflichtfelder

| Feld | Inhalt |
|---|---|
| `id` | eindeutige Kennung der Karte |
| `title` | kurze Beschreibung |
| `wp` | Arbeitspaket aus SPEC 15.4, z. B. `WP-3.3` |
| `crate` | betroffenes Crate |
| `goal` | gewünschte Wirkung, prüfbar |
| `non_goals` | ausdrücklich nicht zu ändern |
| `scope` | erlaubte Pfade |
| `forbidden` | verbotene Pfade (mindestens `AGENTS.md`, `docs/SPEC.md`, `rust-toolchain.toml`, `deny.toml`, `governance/**`, `.github/**`, `.agents/jules/setup/**`) |
| `spec_ids` | betroffene SPEC-IDs (I-, FM-, T-, BM-, D-, HR-) |
| `acceptance` | Befehle, die bestehen müssen |
| `evidence_required` | was im PR belegt wird |

Ein Budget (`files`, `lines`, `iterations`) ist sinnvoll, um Sitzungen zu begrenzen.

## Beispiel

```toml
id        = "T-2026-0101"
title     = "WAL: Fehler beim Flush propagieren statt verschlucken"
wp        = "WP-3.3"
crate     = "contextra-store"
goal      = "Ein fehlgeschlagener Flush liefert einen Fehler an den Aufrufer, kein stilles Verwerfen."
non_goals = ["Keine Änderung an der Kompaktion", "Keine neuen Abhängigkeiten"]
scope     = ["crates/contextra-store/src/wal/flusher.rs", "crates/contextra-store/tests/wal_*.rs"]
forbidden = ["AGENTS.md", "docs/SPEC.md", "rust-toolchain.toml", "deny.toml", "governance/**", ".github/**", ".agents/jules/setup/**"]
spec_ids  = ["P6", "I-10", "FM-5", "D-55", "T-FSYNC-1"]
acceptance = ["cargo test -p contextra-store --locked --test wal_lifecycle_crash_point_matrix"]
evidence_required = ["Neuer Test für einen I/O-Fehler beim Flush (rot, dann grün)"]

[budget]
files = 6
lines = 250
iterations = 6
```

Die Vorfassung verwendete Invariantennamen wie `INV-WAL-POISON-EXPLICIT`, die nicht in der SPEC stehen. Verwendet werden die IDs aus SPEC 13.1 und 13.2.
