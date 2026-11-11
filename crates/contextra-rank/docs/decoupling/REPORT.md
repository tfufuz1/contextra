# Entkopplungsbericht: contextra-rank (Auftrag P7)

**Crate:** `contextra-rank`
**Datum:** 2026-11-10
**Status:** Erfolgreich abgeschlossen

---

## 1. Baseline V0

Vor Beginn der Änderungen durchgeführte Prüfungen:
- `cargo check -p contextra-rank --no-default-features`: Erfolgreich (`dev` profile target(s) in 13.27s).
- `cargo check -p contextra-rank --all-targets`: Erfolgreich.
- `cargo test -p contextra-rank --lib`: Erfolgreich (19 passed; 0 failed).
- `cargo test -p contextra-rank`: Erfolgreich (alle Unit-, Integrations- und Doctests grün).
- `cargo clippy -p contextra-rank --lib -- -D warnings`: Sauber und warnungsfrei.
- `cargo clippy -p contextra-rank --all-targets -- -D warnings`: Pre-existing Warnungen in `tests/rank_invariants.rs` (`clippy::unwrap_used`, `clippy::indexing_slicing`).

---

## 2. Arbeitspakete & Commits

### WP-0: Baseline V0 & Fremdnutzer-Inventar
- Ermittlung der Baseline V0.
- Analyse aller externen Referenzen auf `contextra_rank` in Workspace-Crates und Benchmarks:
  - `crates/contextra-engine`: 12 Stellen (`src/fusion.rs`, `src/temporal_filter.rs`, `src/contextra_impl/wiring.rs`, `src/lib.rs`).
  - `crates/contextra-mcp`: 3 Stellen (`src/routing.rs`, `src/explain.rs`).
  - `crates/contextra-router`: 2 Stellen (`src/profile.rs` Kommentar/Doc).
  - `crates/contextra-infer-candle`: 2 Stellen (`src/compat/calibrator.rs`, `src/gasp.rs`).
  - `crates/contextra-db`: 1 Stelle (`src/lib.rs`).
  - `crates/contextra-types`: 1 Stelle (`src/types/saos.rs` Kommentar).
  - `benchmarks/contextra-bench`: 8 Stellen (`src/matrix.rs`).

### WP-1: Cargo.toml & Feature-Angleichung (`workspace`)
- Commit: `6d0e59225` - `contextra-rank: WP-1 introduce workspace feature and bridge wiring`
- `features.workspace = ["dep:contextra-types", "dep:contextra-ports"]` eingeführt.
- `features.types-compat = ["workspace"]` und `features.ports-compat = ["workspace"]` als abwärtskompatible Aliase eingerichtet.
- `features.default = ["std", "workspace", "exp-dibud", "exp-resonance", "experimental-rank"]`.
- Optionale interne Abhängigkeiten `contextra-types` und `contextra-ports` explizit mit relativem Pfad und `default-features = false` deklariert.
- `[dev-dependencies]` um `contextra-types` und `contextra-ports` ergänzt.
- `src/lib.rs`, `src/fusion/mod.rs`, `src/fusion/types.rs` und `src/compat/mod.rs` von `types-compat` auf `workspace` umgestellt.
- Verifikation:
  - `cargo check -p contextra-rank --no-default-features`: Grün.
  - `cargo check -p contextra-rank --all-targets`: Grün.
  - `cargo check -p contextra-rank --no-default-features --features workspace --all-targets`: Grün.

### WP-2: Kern-Doc-Kommentare & Null-Prüfung V5
- Commit: `98ed65b67` - `contextra-rank: WP-2 neutralize core doc comments for V5 zero-check`
- In `src/fusion/ext.rs` Doc-Kommentare neutralisiert, um Erwähnungen von `contextra_types` / `contextra_ports` im autarken Kern zu eliminieren.
- Verifikation:
  - V5 Null-Prüfung (`grep -rEn 'contextra_(types|ports|testkit)' crates/contextra-rank/src --include=*.rs | grep -v '/compat/'`): 0 Treffer (leer).
  - V6 Kern-Closure (`cargo tree -p contextra-rank -e normal --no-default-features --prefix none | grep '^contextra-' | sort -u`): exakt `contextra-rank v0.1.0`.

### WP-3: Konformitätstests & Mapping-Tests in `src/compat/`
- Commit: `f065bf52c` - `contextra-rank: WP-3 add mapping and conformance unit tests in compat`
- In `src/compat/types.rs`:
  - `test_types_conformance_and_serde_equivalence` für DocId, TxId, FusionStrategy, SignalFailurePolicy, SignalFusionStrategies, FusionWeights und ConfigFingerprint hinzugefügt.
- In `src/compat/error.rs`:
  - `test_rank_error_to_contextra_error_mapping` für alle Varianten von `RankError` hinzugefügt (Prüfung von Typ-Variante und exaktem Display-Text).
- Verifikation:
  - `cargo test -p contextra-rank --lib`: 21 passed; 0 failed.

### WP-4: Verifikation aller Features und Abwärtskompatibilität
- Alle Einzelfeatures geprüft:
  - `cargo check -p contextra-rank --no-default-features`: Grün.
  - `cargo check -p contextra-rank --no-default-features --features std`: Grün.
  - `cargo check -p contextra-rank --no-default-features --features dibud`: Grün.
  - `cargo check -p contextra-rank --no-default-features --features exp-dibud`: Grün.
  - `cargo check -p contextra-rank --no-default-features --features exp-resonance`: Grün.
  - `cargo check -p contextra-rank --no-default-features --features experimental-rank`: Grün.
  - `cargo check -p contextra-rank --no-default-features --features workspace`: Grün.
  - `cargo test -p contextra-rank`: Grün (vollständige Test-Suite inklusive aller 22 Test-Targets).
- V7 Fremdnutzer-Probe:
  - `cargo check -p contextra-engine --all-targets`: Grün.
  - `cargo check -p contextra-mcp --all-targets`: Grün.
  - `cargo check -p contextra-router --all-targets`: Grün.
  - `cargo check -p contextra-infer-candle --all-targets`: Grün.

---

## 3. Ausgabe von V5 und V6

### V5: Null-Prüfung
```bash
$ grep -rEn 'contextra_(types|ports|testkit)' crates/contextra-rank/src --include=*.rs | grep -v '/compat/'
# Ergebnis: LEER (0 Treffer)
```

### V6: Kern-Closure
```bash
$ cargo tree -p contextra-rank -e normal --no-default-features --prefix none | grep '^contextra-' | sort -u
contextra-rank v0.1.0 (/app/crates/contextra-rank)
# Ergebnis: Exakt Teilmenge der erlaubten Abhängigkeiten (keine internen Crates im autarken Kern)
```

---

## 4. Fassaden-Liste (T2-Fälle)

Keine T2-Fassaden-Aufspaltung erforderlich:
- Alle öffentlichen Signaturen von `contextra-rank` blieben unverändert.
- Die lokalen Typen (`DocId`, `TxId`, `FusionWeights`, `ConfigFingerprint`, etc.) und Methoden im Kern harmonieren über die bestehenden und neuen `From`/`Into`-Implementierungen in `src/compat/` transparent mit `contextra-types` und `contextra-ports`.
- Alle Konsumenten (`engine`, `mcp`, `router`, `infer-candle`) kompilieren quellkompatibel ohne jegliche Anpassungen.

---

## 5. Fremdnutzer-Inventar & Übergaben für künftigen Default-Flip

Liste der externen Manifeste, die `contextra-rank` referenzieren:
1. `crates/contextra-engine/Cargo.toml`: `contextra-rank = { workspace = true }`
2. `crates/contextra-mcp/Cargo.toml`: `contextra-rank = { workspace = true }`
3. `crates/contextra-infer-candle/Cargo.toml`: `contextra-rank = { workspace = true }`

**Übergaben für späteren Default-Flip:**
Sobald `workspace` aus den Default-Features von `contextra-rank` entfernt wird, müssen Konsumenten, die `contextra-types` / `contextra-ports`-Brücken (wie `IsotonicPortsClockExt`, `ContextChunk`-Konvertierungen oder automatische `ContextraError`-From-Impls) nutzen, folgendes deklarieren:
```toml
contextra-rank = { workspace = true, features = ["workspace"] }
```
Crates, die ausschließlich den autarken Kern nutzen (z.B. Standalone-Pipelines oder CLI-Tools), können dann mit `default-features = false` arbeiten.

---

## 6. Offene Punkte / Risiken

- Keine verbleibenden Risiken für `contextra-rank`.
- Siehe `FINDINGS.md` für Notizen zu Vorab-Warnungen in Tests und Verhalten von `ResonanceConfig` bei Feature-Kombinationen.
