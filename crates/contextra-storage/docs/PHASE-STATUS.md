# Phasenstatus — contextra-storage R9.1

Die Reihenfolge und Gates folgen [00-INDEX.md](00-INDEX.md), `SPEC.md` §14 sowie dem Audit-Register (§12.8, Anhang B.2/B.3). Die Anforderungen sind in [25-r8-audit-remediation.md](25-r8-audit-remediation.md) den Besitzer-SPs zugeordnet. Ein Phasenabschnitt wird erst als abgeschlossen markiert, wenn Implementierung, spezifizierte Verifikation und Gate-Evidenz vorliegen. Eine grüne Kompilierung allein schließt keine Phase ab.

| Phase | Stand | Umfang / offene Abnahme |
|---|---|---|
| P0 / G-0 | In Arbeit | Ports, Codec-Grundlagen, MemFs/FaultFs, Rang-/Epoch-Hilfen, Krypto-Referenz, Key-Wrapping, MemoryBudget und SpaceOptions sind teilweise umgesetzt. Der SPEC-Abgleich ergänzt Clock-Snapshot/Skew-Vertrag, Filesystem-Range-Reads/Locks/Capabilities, typisierte Metrik-/Observer-Dimensionen und den §9.4-KeyProvider mit In-Memory-Provider im `contextra-crypto-ref`. Der aktuelle 7-Crate-Check und Testlauf ist grün (siehe TRACE); offen bleiben vollständige Ports-Review, vollständige Conformance, unabhängige Referenzvektoren, CI-Gate und Reviewer-Freigabe. |
| P1 / WS | In Arbeit | Catalog-Open/Replay, gruppierte durabile Edits, CURRENT-Pointer, Recovery-Replay-Brücke, Teile von Memtable/MVCC und Commit-Publikation, E-67 Purge-Edits sowie monotone SpaceId-Reservierung sind umgesetzt. SpaceId ist im Port u64 und in vorhandenen AAD-, Redo-, KEYS- und Catalog-Feldern kanonisch big-endian. Catalog-/Redo-Crash-Sweeps bestehen. Offen: vollständiger FORMAT→KEYS→CURRENT/Anker-Open-Flow, Snapshot/Rollover, Vergabeintegration im FACADE/Committer, Gate-/Ledger-Scope-Integration und Ende-zu-Ende Committer/Recovery/Gate-Crash-Sweep. |
| P2 / G-F | In Arbeit | DataBlock, SubIndex, ClusterDirectory, Footer, RangeDelBlock, Bloom-Filter und opaker ExtensionBlock-Slot sowie physischer Run-Dateiaufbau, struktureller RunReader und Rohsektionen-Relokation mit injiziertem Prüfer sind implementiert; Skip-Scan-Grundlage und Catalog-Edits sind vorhanden. Offen: PackMeta-Codec, Entry-RunBuilder mit Seal-/Placement-Regeln, Punkt-/Iter-Reader, formatbewusste CRC-Prüfung für Rohkopie, Flush, Purge-Integration, Recovery-Abschluss, unabhängige Referenzvektoren und Formatfreeze. |
| P3 / G-W | Teilweise begonnen | Sequenzierter Commit-Publikationsadapter ist vorhanden. Guard-Fenster, vollständige SYS_IDEM-Durabilität, Admission/Reservierungen und Gruppenführer sind offen. |
| P4 / G-R | Teilweise begonnen | Run-Block/SubIndex/Directory/Footer/RangeDel/Bloom/Extension-Slot, struktureller Reader/Rohsektionen-Relokation und Blob-Record-Rahmen sind Codec-Grundlagen; Blob-GC-Referenzindex liefert getestete Remove-Kandidaten. PackMeta, vollständige Run-/Blob-Dateiformate, Writer/Rotation, Reader/KeyScope, formatbewusste Rohkopie-Prüfung, Referenzabgleich sowie vollständiger Cursor/Page-Lesepfad sind offen. |
| P5 / G-M | Teilweise begonnen | MemoryBudget, KeyCache-Grundlage, begrenzter FD-TableCache mit Pinning/LRU-Eviction sowie eine getestete DiskFull-Hysterese/Admission im Governor sind vorhanden. Scheduler-Anbindung, Flush/Compaction/Blob-GC/Scrub, Feed und gemeinsame Generation-Fencing-Integration sind offen. |
| P6 / G-E | Teilweise begonnen | Cluster-Lifecycle-Modell, Catalog-Replay und Scratch-Reserve-Grundlage sind vorhanden. Ende-zu-Ende Gate/Erase-Protokoll, Ledger, Purge, Checkpoints, Extensions und produktive Provideradapter sind offen. |
| P7 / G-V | Nicht begonnen | Clusterlokale Branches und Merge. |
| P8 / G-P | Nicht begonnen | Soak, Windows-Spike, Bridge und Cutover. |

## Ablösung von `contextra-store`

Der Alt-Crate ist bis zur P8-Ablöse im Workspace und bleibt während dieser Zeit unverändert. Aktuelle direkte Verbraucher umfassen `contextra-db`, `contextra-agent`, `contextra-engine`, `contextra-graph`, `contextra-cognition`, `contextra-checkpoint`, `contextra` und `contextra-py`; die vollständige Cargo-Abhängigkeitsprüfung gehört zu WP-24.1. Die neuen Storage-Crates hängen nicht vom Alt-Crate ab. Seine Entfernung und Archivierung erfolgen erst, wenn sämtliche Produktions-, Test-, Beispiel- und CI-Verbraucher umgestellt sind.

## Arbeitsstatus

Die Tabelle bildet Komponentenfortschritt ab, keine Freigabe der Phasen-Gates. Einzelne spätere Grundlagen wurden vorgezogen, wo die aktuelle SPEC dafür bereits einen stabilen Vertrag vorgibt. Kein Gate ist dadurch automatisch abgeschlossen.

## Normative Entscheidungen mit Implementierungsblockern

`crates/contextra-storage/SPEC.md` ist die normative Quelle; diese Statusdatei und die SPs dürfen sie nicht überschreiben. A-23/E-72 ist teilweise umgesetzt: `SpaceId u64`, reservierte IDs 0…255, Benutzer-ID ab 256, High-Water-Reserve und Big-Endian-Kodierung in vorhandenen AAD-, Redo-, KEYS- und Catalog-Feldern sind vorhanden. Space-Gate-/Ledger-Scope-Tags, alle Referenzvektoren und End-to-End-Recovery müssen noch folgen; G-0/G-F ist deshalb nicht bestanden. Außerdem beweist lokales `LOCK` keine Exklusivität restaurierter Kopien auf verschiedenen Hosts; Restore-Schreibzugriff verlangt externes Fencing und bleibt sonst `RestoreQuarantine` (A-24/E-73), mit Zwei-Host-Nachweis bei G-CUTOVER.

## Compile- und Testhinweis

**Aktuelle Ausführungsumgebung, 2026-11-23:** Mit freigegebener Cargo-
Netzwerkauflösung bestanden `cargo check` und `cargo test` für alle sieben
Storage-Crates mit `--ignore-rust-version --features testing`; der Testlauf
benötigt `CARGO_TARGET_DIR=/tmp/contextra-storage-target`, weil das Workspace-
`target/` schreibgeschützt ist. `rustup`, `cargo clippy`, `cargo fmt` und
`cargo deny` sind nicht installiert; diese Basis-Gates bleiben ungeprüft. Der
Rust-1.89.0-Pin bleibt extern offen. Details und Befehle stehen in `TRACE.md`.

Die installierte Rust-Version 1.85.1 liegt unter dem Workspace-Mindeststand 1.89. Teilverifikation mit `--ignore-rust-version --offline`: `cargo check -p contextra-storage -p contextra-crypto-ref --features testing` und `cargo test -p contextra-storage --features testing` bestanden, einschließlich der gezielten Crash-, Codec-, Cache-, Canary- und Run-Dateiaufbau-Tests. Das ist keine vollständige Release-Abnahme. Nicht implementierte Besitzer-Module und alle offenen Crash-/Restore-/Format-Gates bleiben ausdrücklich offen.
