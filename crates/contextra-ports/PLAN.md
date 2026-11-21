# contextra-ports — Implementierungsspezifikation

**Ebene:** E0 (Fundament)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch)
**Stand:** 21.11.2026, Abgleich mit SPEC v4.1 (Ist gemessen am HEAD dieses Abgleichs)

Dieses Dokument verweist nur auf Kapitel und IDs der SPEC. Abweichungen des Ist-Zustands sind als **Ist-Abweichung** gekennzeichnet, ungeklärte Punkte stehen unter **Offene Punkte (Entscheidung nötig)** in Abschnitt 6.

---

## 1. Zweck und Ebene

`contextra-ports` enthält die Port-Traits und Datentransferobjekte (DTOs), über die die Crates miteinander sprechen, ohne sich zu kennen (SPEC 3.2, R9, ADR-32). Das Crate implementiert nichts. Eine Implementierung liegt in dem Crate, das den Zustand besitzt (SPEC 3.4).

Ebene E0 (SPEC 3.1, 3.2). Ziel-Umfang nach SPEC 3.1: rund 3.000 Zeilen (Ist: 6.755 Zeilen `src`, Messung dieses Abgleichs).

## 2. Vertrag

Soll-Bestand nach SPEC 4.4, 3.4, 6.2 und 15. Die Spalte „Ist“ beschreibt den Stand im Quellcode.

| Soll (SPEC) | Modulpfad laut Soll | Ist im Crate | Status |
|---|---|---|---|
| `ReadGate` (4.4) | `ports::erasure` | `gate.rs`, Trait `ReadGate` | Ist-Abweichung: Modul und Signatur nach 4.4 prüfen |
| `Visibility` (4.4) | `ports::version` | nicht vorhanden | fehlt |
| `ContentLifecycle` (4.4) | `ports::version` | nicht vorhanden | fehlt |
| `RecordSealer` (4.4) | `ports::crypto` | `keys.rs`, Trait `RecordSealer` | Ist-Abweichung: Modulpfad |
| `SegmentSealer` (4.4) | `ports::crypto` | nicht vorhanden | fehlt |
| `KeyDestroyer` (4.4) | `ports::crypto` | nicht vorhanden | fehlt |
| `SealScoped` (4.4) | `ports::erasure` | nicht vorhanden | fehlt |
| `ScrubTarget` (4.4) | `ports::erasure` | nicht vorhanden | fehlt |
| `KvStore` (3.4) | Port des Crates `store` | `kv.rs`, Trait `KvPrefixStore` | Ist-Abweichung: Name |
| `SegmentStore` (3.4) | Port des Crates `store` | nicht vorhanden | fehlt |
| `VectorIndex` (3.4) | Port des Crates `vector` | `vector_index.rs` | vorhanden, Signatur prüfen |
| `TextIndex` (3.4) | Port des Crates `text` | `text_index.rs` | vorhanden, Signatur prüfen |
| `GraphIndex` (3.4) | Port des Crates `graph` | `graph_index.rs` | vorhanden, Signatur prüfen |
| `Checkpointer` (3.4) | Port des Crates `checkpoint` | `checkpoint.rs`, Traits `Checkpoint`, `CheckpointCoordinator` | Ist-Abweichung: Name |
| `RootKeyProvider` (3.4) | Port des Crates `crypto` | nicht vorhanden; ähnlich `key_material.rs`, `KeyMaterialSource` | Entscheidung nötig (siehe 6) |
| `ErasureSink` (3.4) | Port des Crates `erasure` | nicht vorhanden; ähnlich `ledger.rs`, `LedgerSink` | Entscheidung nötig (siehe 6) |
| `EmbeddingProvider`, `Reranker` (6.2) | Ports | `embedding.rs`, `reranker.rs` | vorhanden |
| `ContextraError` (15) | Port-Modul | nicht definiert; Re-Export aus `contextra-types` | Ist-Abweichung (REM-30) |
| DTO-Modul `dto` (15): `SourceDoc`, `SearchQuery`, `SearchResponse`, `ContextPack`, `ErasureRequest`, `ScrubPlan`, `ScrubProgress`, `IoBudget` | `ports::dto` | kein Modul `dto`; `ScrubPlan`, `ScrubProgress`, `IoBudget` fehlen | fehlt |

Anforderungen an den Vertrag: Alle Traits sind dyn-kompatibel (`Send + Sync`). Die Objektsicherheit wird per Compile-Test geprüft (SPEC 15). Das Crate führt keine Fehlerart außer `ContextraError` ein (SPEC 4.4, 6.8).

## 3. Abhängigkeiten

**Erlaubt (Anhang E, R1):** `contextra-ids`. **SPEC 15 nennt zusätzlich** `roaring`.

**Ist-Abweichungen (Cargo.toml):**
- `contextra-env`, `contextra-types`: verboten nach R1 und Anhang E. `contextra-types` ist nach REM-27 und ADR-33 aufzulösen. Ist-Abweichung, Entfernung in Abschnitt 5.
- `blake3`, `futures-util`, `bytes`, `serde`, `serde_json`, `ahash`, `thiserror`: SPEC 15 führt diese nicht auf. Ob sie bleiben, ist offen (siehe 6).

**Dev-Abhängigkeiten:** `proptest`, `tokio` (Features `rt`, `macros`). `tokio` ist in `deny.toml` mit Wrapper-Liste für `contextra-ports` eingetragen (Transition E29). Die Frage der tokio-Nutzung ist offen (siehe 6, Punkt 10 der CLAUDE.md).

**Features:** `default = []` (R3, G-06 eingehalten). `conformance`, `contract-tests`, `testkit` sind in SPEC 15 nicht genannt. Ist-Abweichung: Entscheidung über Verbleib nötig.

**Cargo-Auszug (Soll nach SPEC 15):**
```toml
[dependencies]
contextra-ids = { path = "../contextra-ids" }
roaring = { workspace = true }
thiserror = { workspace = true }
```

## 4. Module

Soll-Module nach 4.4 und 15: `error`, `erasure`, `version`, `crypto`, `kv`, `vector_index`, `text_index`, `graph_index`, `checkpoint`, `embedding`, `reranker`, `dto`. Ist-Dateien, die nicht im Soll stehen, sind unten als Ist-Abweichung markiert.

| Datei | Ist-Verantwortung (aus Modul-Doku) | Befund |
|---|---|---|
| `lib.rs` | Re-Exporte aus `contextra-types`, `BoxFuture`, `BoxStream`, Modulliste | Ist-Abweichung: Re-Exporte entfallen (REM-30) |
| `gate.rs` | `ReadGate` | Soll `ports::erasure` |
| `keys.rs` | Schlüssel- und Sealing-Traits | Soll `ports::crypto` |
| `erasure.rs` | `ErasureLayer`, `StorageBackend`, `ErasureTarget` | Soll `ports::erasure` (Traits anpassen) |
| `erasure_legacy.rs` | Alte Erasure-Traits, „Verlagerung nach contextra-erasure-spi“ | Ist-Abweichung: `erasure-spi` ist in SPEC nicht genannt |
| `kv.rs` | `KvPrefixStore` | Soll `kv` (Name klären) |
| `storage.rs` | `StorageEngine` | REM-36, entfernen |
| `vector_index.rs`, `text_index.rs`, `graph_index.rs`, `index.rs` | Index-Ports | Soll vorhanden; `index.rs` nur Re-Export |
| `checkpoint.rs` | `Checkpoint`, `CheckpointCoordinator` | Soll `checkpoint` |
| `embedding.rs`, `reranker.rs` | Embedding, LLM, Reranker | Soll 6.2; `TextGenerator`, `LlmTextGenerator` sind in SPEC 6.2 nicht genannt (offen) |
| `query_rewriter.rs`, `inference.rs` | LLM-Query-Umschreibung, Inferenz | REM-25, entfernen |
| `license.rs` | `LicenseGate`, `FeatureRing` | REM-03, entfernen |
| `clock.rs`, `env.rs`, `id_gen.rs`, `rng.rs`, `metrics.rs` | Re-Exporte aus `contextra-env` | Ist-Abweichung: R1 und R10 (Zeit und Zufall nur über `env`). Re-Exporte entfallen |
| `fs.rs`, `vfs.rs` | VFS-Port | nicht in SPEC 15 genannt, offen |
| `barrier.rs` | `CommitBarrier`, `PurgeBarrier` | nicht in SPEC 15 genannt, offen |
| `observer.rs` | `ErasureObserver` | nicht in SPEC 15 genannt, offen |
| `ledger.rs` | `LedgerSink` | Soll `ErasureSink` (siehe 6) |
| `key_material.rs` | `KeyMaterialSource` | Soll `RootKeyProvider` (siehe 6) |
| `kv_bridge_port.rs` | `KvBridgeStorage` | nicht in SPEC 15 genannt, offen |
| `lifecycle.rs`, `observability.rs`, `attention.rs`, `report_signer.rs`, `policy.rs`, `plugin.rs` | diverse Ports | nicht in SPEC 15 genannt, offen |
| `graph.rs` | `GraphCollectionMutation` | nicht in SPEC 15 genannt, offen |
| `scanner/` (`mod.rs`, `layer.rs`, `session.rs`, `witness.rs`) | `LayerScanner`, `Purger`, `LayerWitness` | Verweise auf „SPEC §12.7.4“ ins Leere, Abschnitt 12.7 existiert nicht. Ist-Abweichung |
| `conformance.rs`, `contract_tests.rs` | Konformitäts- und Vertragstests im Produktcode | Ist-Abweichung: Testwerkzeuge im Crate. Verbleib offen |
| `examples/standalone.rs` | Beispiel | offen |
| `tests/` | `generate_golden_env.rs`, `golden_env_parity.rs`, `plugin_registry_dependency_cycle.rs` | offen (siehe 7) |

Neu anzulegen nach Soll: `erasure`, `version`, `crypto`, `dto` als Modulverzeichnisse oder Dateien, je mit den Traits aus Abschnitt 2.

## 5. Entfernungen

| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| REM-03 | `license.rs` (Lizenzreste, `FeatureRing`) | `grep -rn "License\|FeatureRing" crates/contextra-ports/src` leer |
| REM-25 | `query_rewriter.rs`, `inference.rs` (LLM-Query-Rewriter) | `grep -rn "LlmTextGenerator\|QueryRewriter" crates/contextra-ports/src` leer |
| REM-30 | Importe aus `contextra-types`, Re-Export-Block in `lib.rs` | `grep -rn "contextra_types\|contextra-types" crates/contextra-ports` leer |
| REM-36 | `StorageEngine` (Duplikat zu `KvStore`) | `grep -rn "StorageEngine" crates/contextra-ports` leer |
| REM-27 | Abhängigkeit `ports → types` und `ports → env` (R1) | `python3 tools/check_spec_sync.py layering` ohne Befund für `contextra-ports` |

Hinweis: Die Entfernung der Re-Exporte aus `contextra-env` (Abschnitt 4) ergibt sich aus R1 und R10. Dies ist kein eigener REM-Punkt in SPEC 12, sondern folgt aus SPEC 3.3.

## 6. Umbau

Schritte in Reihenfolge, je Schritt ein Commit, Workspace grün. Dependents betroffen durch Signaturänderungen (siehe CLAUDE.md, Crate-Notizen „Offen:“).

1. Re-Exporte aus `contextra-types` entfernen. Benötigte Typen nach SPEC 4.4 und ADR-33 zuordnen: IDs nach `contextra-ids`, DTOs nach `ports::dto`, Rest zum Besitzer.
2. Re-Exporte aus `contextra-env` entfernen (R1, R10). Dependents importieren direkt aus `contextra-env`.
3. REM-03, REM-25, REM-36 entfernen (Abschnitt 5).
4. Module und Traits nach Abschnitt 2 anlegen oder umbenennen: `erasure`, `version`, `crypto`, `dto`.
5. Compile-Test für Objektsicherheit und DTO-Roundtrip-Test ergänzen (SPEC 15).
6. Abhängigkeiten auf `ids` und `roaring` reduzieren, sobald die Entscheidung unter „Offene Punkte“ getroffen ist.

**Offene Punkte (Entscheidung nötig):**
- **O-1 Namen.** `KvPrefixStore` gegen `KvStore`, `Checkpoint`/`CheckpointCoordinator` gegen `Checkpointer`, `LedgerSink` gegen `ErasureSink`, `KeyMaterialSource` gegen `RootKeyProvider`. SPEC 3.4 und 15 nennen die Soll-Namen. Umbenennen oder SPEC ändern? Entscheidung des Nutzers.
- **O-2 Async im Kern.** Der frühere PLAN.md enthielt eine Entscheidung „O-PRT-1 Async im Kern“. Sie ist in der SPEC nicht verankert (keine Fundstelle für „O-PRT“). Außerdem nennt `deny.toml` für `contextra-ports` die Transition „REQ-ASYNC-1“, und diese ID fehlt in SPEC v4.1. Entscheidung nötig, bevor Traits async werden.
- **O-3 Abhängigkeiten außerhalb `ids`/`roaring`.** `blake3`, `futures-util`, `bytes`, `serde`, `serde_json`, `ahash`, `thiserror`: behalten oder entfernen? SPEC 15 nennt nur `ids` und `roaring`.
- **O-4 Features.** `conformance`, `contract-tests`, `testkit`: entfernen oder in SPEC 15 aufnehmen?
- **O-5 Verbleib der Module ohne SPEC-Eintrag** (`plugin`, `policy`, `observer`, `report_signer`, `attention`, `barrier`, `kv_bridge_port`, `graph`, `scanner`, `lifecycle`, `observability`, `vfs`/`fs`, `erasure_legacy`, `conformance`, `contract_tests`). In SPEC 12 oder 15 nicht genannt.
- **O-6 SPEC-Verweis.** `scanner` verweist auf „SPEC §12.7.4“, das in SPEC v4.1 nicht existiert. Korrektur durch den Nutzer.
- **O-7 `ReadGate`-Signatur.** SPEC 4.4 verlangt `state(SealOrd) -> SealState` und `seq() -> u64`. Ist-Signatur gegen 4.4 prüfen und Abweichung entscheiden.

## 7. Tests

- Compile-Test: Objektsicherheit aller Traits (SPEC 15).
- DTO-Roundtrip für `ports::dto` (SPEC 15).
- Bestehende Tests im Ist: `generate_golden_env.rs`, `golden_env_parity.rs`, `plugin_registry_dependency_cycle.rs`. Verbleib entscheidet O-5.
- Crate-lokale Tests: keine über die SPEC-Vorgaben hinaus, bis O-5 entschieden ist.
- Ist-Befund: 30 `#[test]`-Funktionen (laut `CLAUDE.md` des Crates, Messung 21.11.2026).

## 8. Abnahme

Gates aus SPEC 11.6, die dieses Crate betreffen:

| Gate | Ist-Befund | Soll |
|---|---|---|
| G-01 | `cargo check`, `test`, `clippy -D warnings` | grün (Toolchain 1.89.0 laut `rust-toolchain.toml`; Sandbox nutzt `stable`, SYNC-4) |
| G-02 | `#![forbid(unsafe_code)]` in `lib.rs` | eingehalten |
| G-03 | Abhängigkeiten `env`, `types` (Ist-Abweichung) | `python3 tools/check_spec_sync.py layering` ohne Befund für dieses Crate |
| G-04 | 0 Definitionen von `MAX_SEARCH_K` | eingehalten (Soll im Crate nicht definiert, Definition nur in `ids`) |
| G-06 | `default = []` | eingehalten |
| G-07 | Re-Exporte aus `contextra-types` vorhanden | verletzt, siehe REM-30 |
| G-09 | 17 Dateien mit `ANCHOR[`/`FILE-CONTEXT`, `INVENTAR.md` vorhanden, keine `contract.rs` | verletzt (REM-14); Entfernung ist eigene Aufgabe nach Freigabe (CLAUDE.md §6 Punkt 9) |
| G-11 | keine `compat`-Module oder -Features | eingehalten |
| G-12 | 1 Treffer in `graph_index.rs`, Zeile 43, nur in einem Doc-Kommentar | kein Codeaufruf, Fund aber zu bereinigen (Doku-Text) |
| G-13 | 74 Treffer (`unwrap`, `expect`, `panic!`, grob, inkl. Tests) | Ratchet nicht erhöhen, Abbau nach R6 |
