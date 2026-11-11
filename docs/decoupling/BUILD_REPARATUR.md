# Build-Reparatur (Stufe S-00X)

Stand: 2026-11-11 · HEAD bei Bearbeitung: `9a287ace4` (Shell-Commit auf `8b41ffd`)
Toolchain: Rust **1.89.0** (Ubuntu `rustc-1.89` / `cargo-1.89`, MSRV des Projekts)

## Ergebnis

| Prüfung | Ergebnis |
| --- | --- |
| `cargo check --workspace --lib --bins --locked` | **grün** (alle Libraries und Binaries) |
| `cargo test` für `contextra-checkpoint`, `contextra-mvcc`, `contextra-router`, `contextra-infer-candle` (Lib) | **grün** (59 / 41 / 52 / 82 Tests) |
| `cargo check --workspace --all-targets --locked` | **rot**, 35 Test-/Example-/Bench-Targets, siehe „Offene Punkte“ |

Der Stand ist damit: Der Produktionscode baut. Die Test-Targets sind veraltet und brauchen eine eigene Stufe.

## Fünf Defekte aus dem Bericht: Befund am HEAD

| # | Befund am HEAD `9a287ace4` | Maßnahme |
| --- | --- | --- |
| 1 | `infer-candle/Cargo.toml`: doppelter `thiserror` | zweite Zeile entfernt |
| 2 | `ports/src/lib.rs`: doppelter Re-Export `KeyMaterialSource` | zweite Zeile entfernt |
| 3 | `mvcc/src/lib.rs`: doppelter `types`-Block, doppelte Fehler-Re-Exports, doppelte Tombstone-Namen | `types` nur einmal; Fehler-Re-Export einmal; Tombstone-Namen nur aus `tombstone` (R4) |
| 4 | `checkpoint`: `mod source;` gegen `crate::snapshot_source`, beide Dateien existieren | `snapshot_source.rs` ist der einzige Besitzer; `source.rs` entfernt (siehe unten) |
| 5 | `store/src/sstable/builder.rs`: „nicht geschlossener Block“ | **Nicht reproduzierbar.** Die im Bericht genannten Zeilen 568–612 sind am HEAD syntaktisch korrekt, `contextra-store` baut ohne Fehler. Kein INV-6-Eingriff nötig. |

Zusätzlich gefunden und behoben (nicht im Bericht enthalten), jeweils mit Begründung im Code:

- `checkpoint`: Der Crate war halb migriert. `store.rs` mischte `ContextraError` und `CheckpointError`. Der Typ wurde vereinheitlicht. Die Workspace-Brücke liegt jetzt in `src/compat/snapshot_bridge.rs`, hinter Feature `workspace`. `IdWidthMismatch` als Variante ergänzt.
- `text`-Hohlkern (Bericht, R12): am Compiler-Befund nicht geändert, wird in der Umsetzungsreihenfolge als S-32 geführt.
- `engine`: Der Adapter `adapters/text.rs` war ein veralteter Doppelgänger von `contextra_text::compat::scorer::Bm25Scorer` und ist jetzt ein Re-Export. Die `FusionWeights`/`FusionStrategy`-Grenze zu `contextra-rank` konvertiert explizit. `SignalFailurePolicy` ist ein Alias auf `contextra_types::OnSignalFailure` (identische Varianten).
- `router`: fehlende Imports für `LyapunovDriftWatcher`/`LyapunovResult` aus `contextra-adapt`.
- `cognition`: `async-trait` als Abhängigkeit ergänzt. `CapacityExceeded` (existiert nicht) durch `MemoryLimitExceeded` ersetzt. `DocId`-Brücke an der Konsumentengrenze (siehe Restschuld).
- `facade` (`contextra`): `GateToken` existiert nicht mehr. Der Pfad läuft jetzt über `LicenseGate::authorize`, der Test-Mock delegiert an `OpenFastGate`.
- `agent`: Adapter auf `BoxFuture` umgestellt (kein `async-trait`). Bytes werden als JSON-Array von `u8` abgebildet (verlustfrei). Der Checkpoint-Zugriff läuft über einen kombinierten Trait `AgentCheckpointStore`.
- `mcp`: Orphan-Regel durch den lokalen Newtype `PromptGuardDetector` gelöst. Fehlender Match-Arm für `#[non_exhaustive] EgressError` ergänzt.
- `bench`: fehlende Abhängigkeiten `contextra-rank`, `contextra-types` ergänzt. Die Module `locomo` und `long_mem_eval` **fehlen im Repository** (nur Fixtures vorhanden). `path_rag_sweep` und das Binary `contextra-bench` stehen daher hinter Feature `datasets` (nicht Default).
- `infer-candle`: Golden-Test hinter `required-features = ["test-utils"]`.
- `Cargo.lock`: durch die oben genannten Abhängigkeitsänderungen synchronisiert (4 Zeilen, `thiserror` bzw. `async-trait`-Kante).

## Entscheidungen

1. **Ein Besitzer je Naht.** Checkpoint hält `snapshot_source` als einzige Naht. Die Brücke zu `StorageEngine` liegt in `compat/`.
2. **Fehler werden an der Grenze umgewandelt**, nie intern gemischt. `From<CheckpointError> for ContextraError` existiert bereits in `compat/error.rs`.
3. **Duplikate werden nicht stillschweigend gelöscht.** Wo eine Kopie wie `DocId` in `rank` noch existiert, wird an der Konsumentengrenze konvertiert und als Schuld geführt (P3: Differenztest vor Ablösung).
4. **Feature-Gating statt Löschen** bei fehlendem Code (`bench`). Gate ist sichtbar im Manifest kommentiert.

## Abweichungen von `Architekturentscheidungen.md`

- Der Adapter-Newtype für `SnapshotSource` wurde nicht eingeführt. Die bestehende Blanket-Impl über `StorageEngine` kompiliert ohne Konflikte. Neubewertung nötig, sobald Test-Fakes oder fremde Typen kollidieren.
- `FusionWeights`-, `FusionStrategy`- und `DocId`-Duplikate bleiben bestehen. Ihre Ablösung gehört zu S-10C und S-51, nicht zur Reparatur.

## Offene Punkte (Test-Targets, 35 Targets, 108 Fehler)

| Crate | Targets | Ursache (Kurzform) |
| --- | --- | --- |
| `contextra-db` | 10 | Tests und Beispiele importieren Module, die nicht mehr existieren (`memory_consolidation`, `maintenance_scheduler`, `context_compaction`, `consolidation_executor`, `MetadataFilter`). `drop_collection` mit altem dritten Argument. |
| `contextra-engine` | 7 | Veraltete Testfelder (`SearchResult::signals`, `SignalContribution::{modality, rrf_rank, original_score}`), `DecayControllerConfig` ohne `base_half_life_ms`, `drop_collection` mit drei Argumenten. |
| `contextra-cognition` | 8 | Tests verweisen auf entfernte Module (`leanrag…`, `lsm`-Variable), ein Beispiel `standalone_consolidation`. |
| `contextra` (Facade) | 5 | `drop_collection` mit drei Argumenten in Integrationstests. |
| `contextra-agent` | 3 | `minimal_pipeline`, `audit_isolation_test`, Lib-Tests. |
| `contextra-bench` | 1 | Bench `competitive_kv`. |
| `contextra-infer-candle` | 0 | erledigt (Feature). |
| `contextra-py` | 1 | Beispiel `standalone_python_bridge`. |

Die drei-Argumente-Form von `drop_collection` ist ein echter API-Drift: `engine` nimmt zwei Argumente, Tests und Python-Bindings übergeben einen Deletion-Proof-Schlüssel. Hier muss entschieden werden, welche API gilt, bevor Tests angepasst werden.

## Nächste Schritte

1. Entscheidung zur `drop_collection`-API (zwei oder drei Argumente).
2. Test-Targets je Crate in eigener Stufe bereinigen (veraltete Tests entfernen oder migrieren, nicht still gaten).
3. `locomo`/`long_mem_eval` aus dem Original wiederherstellen oder die Benchmarks ausdrücklich streichen.
4. Ausstehende Ein-Besitzer-Migration: `DocId` in `rank`, `FusionWeights`/`FusionStrategy` in `engine`.
5. Gate-Abnahme nach INV-9 mit Toolchain 1.89 und `cargo hack` (in dieser Umgebung nicht ausgeführt).
