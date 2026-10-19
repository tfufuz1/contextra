# ADR-N28: Konsolidierung der Composition Root in der Fassade `contextra` und Entkopplung von `contextra-db`

* **Status:** Proposed
* **Datum:** 2026-10-19
* **Autoren:** Principal Architect (Contextra)
* **Betroffene Komponenten:** `crates/contextra`, `crates/contextra-engine`, `crates/contextra-db`, `crates/contextra-ports`
* **Spezifikationsreferenzen:** Spezifikation §00c (Teil A2), §04 (Crate-Graph), §20.1–§20.2 (Phasenplan 1a & 3b), Prinzipien P5, P27, P30

---

## 1. Ist-Zustand & Befund (mit Pfad- und Zeilennachweisen)

Eine präzise Codebase-Analyse am HEAD zeigt folgende Verteilung der Zuständigkeiten und Architekturverletzungen bezüglich der Composition Root und Re-Exports:

### 1.1 Struct `Contextra` in `crates/contextra-engine`
* **Pfad & Zeile:** `crates/contextra-engine/src/lib.rs:377–394`
* **Struktur:**
  ```rust
  pub struct Contextra {
      storage: Arc<LsmStorage>,
      next_tx: Arc<AtomicU64>,
      dimension: usize,
      config: ContextraConfig,
      expiry_reaper_interval: std::time::Duration,
      community_detection_threshold: u64,
      collections: tokio::sync::RwLock<ahash::AHashMap<String, Arc<Collection<LsmStorage>>>>,
      tenant_collections: tokio::sync::RwLock<TenantCollectionMap>,
      cancel_token: tokio_util::sync::CancellationToken,
      task_tracker: tokio_util::task::TaskTracker,
      embedder: parking_lot::RwLock<Option<Arc<dyn TextEmbeddingEngine>>>,
      orphan_registry: Arc<contextra_checkpoint::InstanceOrphanRegistry>,
      router: parking_lot::RwLock<Option<std::sync::Weak<dyn DriftStatusProvider>>>,
      calibrator: parking_lot::RwLock<Option<std::sync::Weak<parking_lot::Mutex<contextra_rank::IsotonicCalibrator>>>>,
      metrics_sink: parking_lot::RwLock<Arc<dyn contextra_ports::MetricsSink>>,
  }
  ```
* **Befund:** `Contextra` in `contextra-engine` vereint Speicher, Sammlungs-Mapping (`collections`, `tenant_collections`), Hintergrund-Task-Lifecycle (`cancel_token`, `task_tracker`), Einbettungs-Engine (`embedder`), Drift-Router (`router`), Kalibrierer (`calibrator`) und Metrik-Port (`metrics_sink`). Das Erzeugen und Verdrahten dieser Abhängigkeiten findet teilweise in `contextra-engine` (`src/contextra_impl/wiring.rs:17–89`) und teilweise in `contextra` (`crates/contextra/src/builder.rs:20–305`) statt.

### 1.2 Fassade `contextra` (Ring 4)
* **Pfad & Zeile:** `crates/contextra/src/lib.rs:1–107`, `crates/contextra/src/builder.rs:1–305`
* **Befund:** `crates/contextra` agiert als primäre Fassade und deklariert `ContextraBuilder` (`builder.rs:20`), um Instanzen von `Contextra` aufzubauen. `crates/contextra/src/lib.rs:20–29` re-exportiert jedoch wesentliche Typen nicht aus der Engine, sondern direkt aus `contextra-db`:
  ```rust
  pub use contextra_db::{
      chunker, execute_background_consolidation, memory_consolidation, Collection, CollectionConfig,
      Contextra, ContextraConfig, ContextraStats, DriftStatusProvider, EmbeddingBackend,
      EncryptionConfig, KekSource, SearchResult, TextEmbeddingEngine,
  };
  ```

### 1.3 Legacy Crate `contextra-db` als Re-Export-Hülle
* **Pfad & Zeile:** `crates/contextra-db/src/lib.rs:1–754` (754 LOC), `crates/contextra-db/src/` (5 Quellcodedateien, 2105 LOC gesamt)
* **Befund:**
  - `crates/contextra-db/src/lib.rs` re-exportiert Typen aus `contextra-engine`, `contextra-types`, `contextra-ports`, `contextra-store` und `contextra-checkpoint`.
  - Es existieren noch 4 Hilfsmodule in `contextra-db`: `homeostat.rs` (8 LOC), `multistep.rs` (710 LOC), `volatile_vault.rs` (488 LOC), `wiring.rs` (145 LOC).
  - Unter `crates/contextra-db/tests/` befinden sich 60 Integrationstest-Dateien, die historisch gewachsen sind und direkt gegen `contextra-db` testen.
  - Laut Spezifikation §20.1 und §20.2 (Phase 3b) muss `contextra-db` als reine Re-Export-Hülle bestehen bleiben und darf keine eigene Composition-Root-Logik oder primären Systeminstanziierungen beinhalten.

---

## 2. Soll-Zustand & Architekturmodell

### 2.1 Ziel-Zuständigkeiten
1. **`crates/contextra` (Ring 4) als EINZIGE Composition Root:**
   - `crates/contextra` hält das ausschließliche Recht zur Verknüpfung und Verdrahtung von Subsystemen (`contextra-engine`, `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-privacy`, `contextra-router`, `contextra-rank`).
   - `ContextraBuilder` in `crates/contextra/src/builder.rs` steuert die Bereitstellung von `Arc<dyn TextEmbeddingEngine>`, `Arc<dyn StorageEngine>`, `Arc<dyn MetricsSink>` und `Arc<dyn Clock>`.
   - Bessere Isolation: `contextra-engine` ist ein rein interner Ring-3-Orchestrator ohne eigene Kenntnis von höheren Fassaden-Konstruktorzugängen.

2. **Entkopplung & Re-Export-Disziplin in `contextra-db`:**
   - `contextra-db` wird strikt auf `#[deprecated]`-Re-Exports beschränkt (§20.1 Strangler-Pattern).
   - Alle öffentlichen Symbole in `contextra-db` verweisen auf `contextra-engine` bzw. `contextra`.
   - Die verbleibenden Hilfsmodule in `contextra-db` (`multistep.rs`, `volatile_vault.rs`, `wiring.rs`) werden schrittweise in passende Fach-Crates (`contextra-engine`, `contextra-cognition` bzw. `contextra-ports`) verschoben.

### 2.2 Zyklenrisiko & Layering Enforcement
* **DAG-Hierarchie (P5):** Ring 4 (`contextra`) hängt von Ring 3 (`contextra-engine`), Ring 2 (`contextra-store` etc.), Ring 1 (`contextra-ports`) und Ring 0 (`contextra-types`) ab. Kein Crate aus Ring 0–3 darf von `contextra` oder `contextra-db` abhängen.
* **Absicherung:** `cargo run -p xtask -- check-ring-layering` und `cargo run -p xtask -- check-dag` erzwingen die Zyklenfreiheit maschinell.

---

## 3. Schnittstellen zu Nachbar-Aufgaben (P15 und P18)

* **Abhängigkeit zu P15 (KV-Store-Entscheidung):**
  Die Entkopplung der Composition Root beeinflusst die Injektion des In-Memory KV-Cache-Stores (`TenantIsolatedKvStore`). Gemäß P15 wird der KV-Store als optionaler Port/Resource im `ContextraBuilder` verdrahtet. Die Engine verbleibt bezüglich des gewählten KV-Persistence-Modells passiv.
* **Abhängigkeit zu P18 (System-Härtung / Phase-Exit Gates):**
  Die Konsolidierung der Composition Root bildet die zwingende Voraussetzung für die Bereinigung der Re-Export-Schichten in Phase 3b (§20.2). Die Stabilisierung der öffentlichen API (≤ 20 `pub fn` auf Fassadenebene) ist Teil des P18-Freigabe-Gates.

---

## 4. Migrationsschritte & Prüfbefehle

1. **Schritt 1 (Composition Root nach `contextra` verlagern):**
   - Implementierung aller `wiring`-Klassen und Builder-Konstruktionen exklusiv in `crates/contextra/src/builder.rs`.
   - *Prüfbefehl:* `cargo check -p contextra --all-targets`

2. **Schritt 2 (Re-Exports in `contextra-db` als `#[deprecated]` markieren):**
   - Ausstatten aller Typen-Re-Exports in `crates/contextra-db/src/lib.rs` mit `#[deprecated(note = "Verwende Crate 'contextra'")]`.
   - *Prüfbefehl:* `cargo check -p contextra-db --all-targets`

3. **Schritt 3 (Verschiebung der Restmodule aus `contextra-db`):**
   - Verschieben von `multistep.rs` und `volatile_vault.rs` nach `contextra-engine` bzw. `contextra-cognition`.
   - *Prüfbefehl:* `cargo test -p contextra-engine --all-targets`

4. **Schritt 4 (Umstellen aller internen Modultests auf `contextra`):**
   - Migration der 60 Testdateien unter `crates/contextra-db/tests/` zur direkten Verwendung der Fassade `contextra`.
   - *Prüfbefehl:* `cargo test --workspace --locked`

---

## 5. Risiken & Mitigationen

| Risiko | Schwere | Mitigation |
| :--- | :--- | :--- |
| **Breaking API Changes für bestehende Library-Nutzer** | Mittel | Beibehaltung des `contextra-db`-Crates mit Deprecation-Warnungen für einen vollen Release-Zyklus (§20.1). |
| **Zyklenbildung beim Refactoring von Ports/Wiring** | Hoch | CI-Gate `check-ring-layering` und `check-dag` in allen PRs erzwungen. |
| **Redundante Trait-Injektionen im Builder** | Niedrig | Einhalten des P27-Standards (`Arc<dyn Trait>` für thread-sichere Ports). |

---

## 6. Offene Fragen

1. Soll `contextra-db` nach Ablauf der Deprecation-Frist als leerer Re-Export-Crate für Abwärtskompatibilität ewig im Workspace verbleiben oder im Major-Release v2.0 vollständig gelöscht werden?
2. Wie sollen verbliebene Integrationstests in `crates/contextra-db/tests/` nach der Deprecation strukturiert werden (Verschiebung nach `tests/e2e/` oder Integration in `crates/contextra/tests/`)?
