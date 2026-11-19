# contextra-checkpoint — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §5.9/S6 und `docs/crates/contextra-checkpoint.md`. **Ausgangslage:** 4.017 LOC mit Ports/Types-Abhängigkeiten und Produktions-Paniken in Guard/Manifest/Orphan; der Sollkern sind verschlüsselte SSTable-Hardlinks.

1. Begrenze die Closure auf `ids, env`; entferne Compatibility-Ports und persistente Workflow-/Rollback-Altlogik außerhalb des Checkpoint-Zwecks.
2. Definiere versioniertes Checkpoint-Manifest mit Tenant/Collection, Sequenz, Schlüssel-Epoche und SSTable-Referenzen; Anlegen ausschließlich per Hardlink auf immutable `.sst`.
3. Verknüpfe Pins mit MVCC-Verfügbarkeit ohne selbst MVCC zu besitzen; Restore prüft Gate/Key-Epoch und verweigert widerrufene Daten.
4. Implementiere Sweep: bei Purge trennt es Hardlinks bis `gated_seq`, damit der Primärspeicher geschreddert/rewritten werden kann; keine Payload-Kopie.
5. Ersetze Produktions-Panics in Manifest, Guard und Orphan-Recovery durch `CheckpointError`; leere Defaults und sichere Crash-Recovery.

**Abnahme:** Hardlink-/kein-Copy-Test, Manifest-Golden/Crash-Test, Restore-Guard nach Erasure, Sweep-Test und Zero-Panic-/Layering-Gate.
