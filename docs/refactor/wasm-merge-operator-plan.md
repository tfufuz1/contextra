# WASM MergeOperator Architecture & Refactoring Plan

**Stand:** 2026-09-30
**Status:** Analyse & Entwurf (Keine Implementierung)
**Ziel:** Sichere Integration untrusted WASM Guest Code als `MergeOperator` für SSTable-Wertefusion (§4.12, §4.18).

---

## 1. Aktueller Verdrahtungsstand

In der bestehenden Architektur ist der `MergeOperator` als synchroner Trait definiert:

- **Trait-Definition:** `crates/contextra-store/src/compaction/merge_operator.rs`
  ```rust
  pub trait MergeOperator: Send + Sync {
      fn merge(&self, existing_val: &[u8], new_val: &[u8]) -> Result<Vec<u8>>;
  }
  ```
- **Compaction Engine Integration:** `crates/contextra-store/src/compaction/engine.rs`
  Die Compaction-Engine führt bei zweifachen Werten bisher ein Last-Write-Wins (Sequence-Number Ordering) durch. Das Trait `MergeOperator` existiert als Schnittstelle, wird aber in der aktuellen Compaction-Loop noch nicht als WASM-Funktion aufgerufen.
- **Isolations-Boundary (Ring 2):** `crates/contextra-sandbox`
  `contextra-sandbox` stellt die WASM-Ausführungsgrenze bereit (`WasmExecutor`, `WasmCapabilities`, `MergeOperatorCapabilities::pure()`).
  - Strict Capability Profile: `MergeOperatorCapabilities::pure()` deaktiviert I/O, Netzwerk, Dateisystem, Stdin/Stdout und PRNG (`random_seed: None`).
  - Determinismus & Isolation: Vollständige Wasmtime-Memory-Isolation (Max. 16 Memory Pages = 1 MB RAM).

---

## 2. Fehlende Teile & Technische Anforderungen

Für eine produktive WASM-MergeOperator-Implementierung fehlen folgende Bausteine:

1. **WASM Host-Guest Adapter Struct (`WasmMergeOperator`):**
   - Ein Adapter in `contextra-sandbox` oder `contextra-engine`, der `MergeOperator` implementiert und den `WasmExecutor` aufruft.
   - **Reine Funktion (Pure Function):** Kein WASI-I/O, kein Dateisystem-, Netzwerk- oder Uhrzugriff. Übergabe der Slices via Stdin/Memory-Offset und Lesen des Ergebnisses aus Stdout/Memory.

2. **Budgeting & Timeout-Limits:**
   - **`max_fuel` Default:** Standardmäßig `10_000_000` CPU Ticks pro Fusionsaufruf.
   - **Wall-Clock-Timeout:** Harte Obergrenze von `5_000 ms` (5 Sekunden) pro Einzelwert-Fusion.

3. **Fail-Safe & Invarianten-Schutz (`FuelExhausted` / Trap Handling):**
   - Falls das WASM-Guest-Modul fehlschlägt (z. B. `SandboxError::FuelExhausted`, Allocation Failure, Divide-by-Zero Trap):
     - **Kein Compaction-Abbruch:** Der Fehler wird isoliert gefangen.
     - **Tracing:** Ausgabe einer `tracing::warn!`-Meldung mit Key-Identifikator.
     - **Fallback-Verhalten:** Beide Versionen (`existing_val` aus älterer SSTable-Tier und `new_val` aus neuerer Tier) bleiben unverändert mit ihren ursprünglichen Sequence Numbers in der kompaktierten SSTable erhalten.

---

## 3. Betroffene Crates & Ringe

| Crate | Ring | Rolle / Änderungsumfang |
| :--- | :--- | :--- |
| `contextra-store` | Ring 1 | Compaction Engine Merge-Loop Aufruf des `MergeOperator` Traits mit Fail-Safe Fallback |
| `contextra-sandbox` | Ring 2 | Implementierung des `WasmMergeOperator` Adapters unter Verwendung von `MergeOperatorCapabilities::pure()` |
| `contextra-engine` | Ring 3 | Konfigurations-Injektion und Registrierung des `WasmMergeOperator` an der `Collection` / `LsmStorage` Schnittstelle |

---

## 4. Aufwandsschätzung

| Arbeitspaket | Aufwand (Personenstunden) | Beschreibung |
| :--- | :---: | :--- |
| **WP 1: Host-Guest Memory Protocol** | 4 h | Binäres Protobuf/Bincode-Zero-Copy-Protocol für Host-to-Guest Value Passing |
| **WP 2: WasmMergeOperator Adapter** | 6 h | Implementierung des `MergeOperator` Traits über `WasmExecutor` mit Fuel-Budgetierung |
| **WP 3: Compaction Engine Fail-Safe Loop** | 6 h | Integration in `merge_sstables_inner()` mit Fallback zur Erhaltung beider Versionen bei WASM-Traps |
| **WP 4: Integrationstests & Fuzzing** | 8 h | Unit- & Property-Tests mit synthetischen WAT/WASM-Trap-Modulen und Fuel-Exhaustion-Szenarien |
| **Gesamtaufwand** | **24 h (~3 Arbeitstage)** | |
