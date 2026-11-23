# Phasenstatus — contextra-storage R8

Die Reihenfolge und Gates folgen [00-INDEX.md](00-INDEX.md), `SPEC.md` §14 und dem Audit-Addendum `docs/SPEC-R8.md` §12.8. Die Anforderungen sind in [25-r8-audit-remediation.md](25-r8-audit-remediation.md) den Besitzer-SPs zugeordnet. Ein Phasenabschnitt wird erst als abgeschlossen markiert, wenn Implementierung, spezifizierte Verifikation und Gate-Evidenz vorliegen. Eine grüne Kompilierung allein schließt keine Phase ab.

| Phase | Stand | Umfang / offene Abnahme |
|---|---|---|
| P0 / G-0 | In Arbeit | Workspace-Verkabelung, Codec-Primitive, MemFs, FaultFs, Rang-/Epoch-Hilfen, Krypto-Referenz, Key-Wrapping und testbarer MemKeyProvider begonnen/umgesetzt. Layering-/Lock-Order-/Canary-Abnahmen bestehen; `max_body_bytes=4 MiB` ist in den PORTS-Limits umgesetzt. Offen: vollständige Ports-Review, Segmentklassen-/MemoryBudget-Ports (A-09/A-11), KeyProvider-Conformance, unabhängige Referenztests, CI-Gate und Reviewer-Freigabe. |
| P1 / WS | In Arbeit | Catalog-Open/Record-Replay, gruppierter Record-Append mit einem Sync, provisorische Header/FORMAT/KEYS-/Seal-Codecs, Recovery-Key-/Anker-Entscheidungen und sequenzierter Committer-Publikationsadapter implementiert. `Storage::open` öffnet den Catalog mit injiziertem Fs/Root. Memtable, historische Root-Registry mit Durable-Sicht und Arc-Reclamation-Grundlage begonnen. Catalog-/Redo-Crash-Sweeps für EIO/TornWrite/LostFsync/ENOSPC bestehen. Offen: Warteschlangen-Gruppenführer/Indeterminate-Waiter, E-67 PurgeBatch/Graveyard/UnlinkAck/Complete, vollständiger FORMAT→KEYS→CURRENT/Anker-Open-Flow, Gate-Auflösung und Recovery-Replay gegen reale Memtable/Idem/Branch-Adapter, MVCC-Wurzeln aus tatsächlichen Commits, Stamp-Reserve-Integration, redo_mark/Rotation/Reserve-Recovery sowie Ende-zu-Ende Committer/Recovery/Gate Crash-Sweep. |
| P2 / G-F | Nicht begonnen | Catalog-Rollover/Faltung, Run-Format, Flush/Purge-Basis, Recovery-Abschluss, unabhängige Referenzvektoren und Formatfreeze. |
| P3 / G-W | Nicht begonnen | Guard-Schnellpfad, Idempotenz, Reservierungen. |
| P4 / G-R | Nicht begonnen | Blob, vollständiger Lesepfad und Run-Optimierungen. |
| P5 / G-M | Nicht begonnen | Wartung, Governor, Feed und Schlüsselcache-Integration. |
| P6 / G-E | Nicht begonnen | Cluster-/Erase-Ende-zu-Ende, Ledger, Checkpoints, Erweiterungen, Provideradapter. |
| P7 / G-V | Nicht begonnen | Clusterlokale Branches und Merge. |
| P8 / G-P | Nicht begonnen | Soak, Windows-Spike, Bridge und Cutover. |

## Ablösung von `contextra-store`

Der Alt-Crate ist bis zur P8-Ablöse im Workspace und bleibt während dieser Zeit unverändert. Aktuelle direkte Verbraucher umfassen `contextra-db`, `contextra-agent`, `contextra-engine`, `contextra-graph`, `contextra-cognition`, `contextra-checkpoint`, `contextra` und `contextra-py`; die vollständige Cargo-Abhängigkeitsprüfung gehört zu WP-24.1. Die neuen Storage-Crates hängen nicht vom Alt-Crate ab. Seine Entfernung und Archivierung erfolgen erst, wenn sämtliche Produktions-, Test-, Beispiel- und CI-Verbraucher umgestellt sind.

## Isolierte Arbeitsgrenzen

- Jede Phase besitzt nur die Dateien und Verträge ihrer SPs; Abhängigkeiten werden ausschließlich über niedrigere SPs/PORTS geführt.
- Änderungen an einer späteren Phase werden bis zum Erreichen ihrer Abhängigkeiten nicht vorgezogen.
- P0 und P1 sind aktuell die einzigen begonnenen Phasen. Nachfolgende Abschnitte bleiben unverändert, bis ihre Phase ansteht.

## Compile-Hinweis

Der Standard-Compile-Check ist wegen der installierten Rust-Version 1.85.1 gegenüber dem Workspace-Mindeststand 1.89 blockiert. Verifiziert mit Rust 1.85.1 (`--ignore-rust-version --offline`): `cargo check -p contextra-storage -p contextra-crypto-ref --features testing`; gezielte Config/Open-, Catalog-/Redo-Crash-Sweep-, Layering-, Lock-Order- und Canary-Tests bestanden. Andere Platzhalter- und Formatgates bleiben offen.
