# Phasenstatus — contextra-storage R8

Die Reihenfolge und Gates folgen [00-INDEX.md](00-INDEX.md), `SPEC.md` §14 und dem Audit-Addendum `docs/SPEC-R8.md` §12.8. Die Anforderungen sind in [25-r8-audit-remediation.md](25-r8-audit-remediation.md) den Besitzer-SPs zugeordnet. Ein Phasenabschnitt wird erst als abgeschlossen markiert, wenn Implementierung, spezifizierte Verifikation und Gate-Evidenz vorliegen. Eine grüne Kompilierung allein schließt keine Phase ab.

| Phase | Stand | Umfang / offene Abnahme |
|---|---|---|
| P0 / G-0 | In Arbeit | Workspace-Verkabelung, Codec-Primitive, MemFs, FaultFs, Rang-/Epoch-Hilfen, Krypto-Referenz, Key-Wrapping und testbarer MemKeyProvider begonnen/umgesetzt. Offen: vollständige Ports-Review, Segmentklassen-/MemoryBudget-Ports (A-09/A-11), KeyProvider-Conformance, Referenztests, CI-Gate und Reviewer-Freigabe. |
| P1 / WS | In Arbeit | Redo-Frame-Codec, Gruppenwriter/-reader, Emergency-Reservewriter und Catalog-Op-/Record-Teilmenge implementiert. Offen: vollständige Tail-/Mid-Log-Absturzmatrix, Segmentrotation und Reserve-Recovery, Catalog-Open/Replay, Committer/MVCC-Durchstich, Gate-/Erase-Wiring und Crash-Sweep. |
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

Der Standard-Compile-Check ist wegen der installierten Rust-Version 1.85.1 gegenüber dem Workspace-Mindeststand 1.89 blockiert. Der eingeschränkte Compile-Check mit `--ignore-rust-version --offline` für `contextra-storage` und `contextra-crypto-ref` war erfolgreich. Tests wurden nicht ausgeführt.
