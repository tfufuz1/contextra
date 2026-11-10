# ABSCHLUSSBERICHT: contextra-infer-onnx (MISSION infer-onnx)

## 1. Übersicht & Zielerreichung
Gemäß Auftrag `infer-onnx` und `docs/decoupling/JULES_RAHMEN.md` wurden folgende Arbeitspakete erfolgreich ausgeführt:
- **WP-O1 (Fakes hinter `test-utils`):** `FakeEmbedder` und `FakeReranker` sind jetzt strikt hinter `#[cfg(any(test, feature = "test-utils"))]` gegated. Manifest-Ziele (`benchmarks`, `examples`, `tests`) wurden konfiguriert.
- **WP-O2 (Leere Features gestrichen):** Die leeren Features `experimental` und `inspect` wurden vollständig aus `Cargo.toml` entfernt. `download` und Kern wurden unabhängig geprüft.
- **WP-O3 (Additive Brücke für `EmbeddingProvider`):** `src/compat/embedding_provider.rs` implementiert `contextra_ports::EmbeddingProvider` für `OnnxEmbedder` und `FakeEmbedder` hinter Feature `workspace`.
- **WP-O4 (Nachweise & Metadaten):**
  - Schema v3 in `capabilities.fragment.toml` hinterlegt (`group = "llm"`, `level = 0`, `tier = "beta"`, `feature_closure_allowed = ["contextra-ports", "contextra-types"]`).
  - Bereinigung des Altlasten-Blocks in `AGENTS.md`.
  - Gate G-L2 Null-Prüfung ohne `compat/` ist vollständig leer (0 Treffer).
  - Standalone-Isolationsprüfung erfolgreich durchlaufen.
  - Zero-Panic Analyzer: 0 Prod-Verstöße.
  - Determinismus Guard: 0 Prod-Verstöße.

---

## 2. Gate-Matrix: Gate × Befehl × Ergebnis

| Gate / Prüfung | Ausgeführter Befehl | Ergebnis |
|---|---|---|
| **G-B (Minimal)** | `cargo check -p contextra-infer-onnx --no-default-features --offline` | GRÜN (0 Fehler) |
| **G-B (Feature download)** | `cargo check -p contextra-infer-onnx --no-default-features --features download --offline` | GRÜN (0 Fehler) |
| **G-B (Feature test-utils)** | `cargo check -p contextra-infer-onnx --no-default-features --features test-utils --offline` | GRÜN (0 Fehler) |
| **G-B (Feature workspace)** | `cargo check -p contextra-infer-onnx --no-default-features --features workspace --offline` | GRÜN (0 Fehler) |
| **G-B (workspace + test-utils)** | `cargo check -p contextra-infer-onnx --no-default-features --features "workspace,test-utils" --offline` | GRÜN (0 Fehler) |
| **G-B (Tests default)** | `cargo test -p contextra-infer-onnx --all-targets --offline` | GRÜN (4 Tests bestanden) |
| **G-B (Tests all-features)** | `cargo test -p contextra-infer-onnx --all-targets --features "workspace,test-utils" --offline` | GRÜN (10 Tests + Benches + Examples) |
| **G-B (Clippy lib)** | `cargo clippy -p contextra-infer-onnx --lib --features "workspace,test-utils" --offline -- -D warnings` | GRÜN (0 Warnungen) |
| **G-L2 (Null-Prüfung)** | `grep -rEn 'contextra_(types|ports|testkit)' crates/contextra-infer-onnx/src --include=*.rs \| grep -v '/compat/'` | GRÜN (0 Treffer) |
| **G-L1 (Tree Closure)** | `cargo tree -p contextra-infer-onnx -e normal --no-default-features -f '{p}'` | GRÜN (Keine internen Abhängigkeiten im Kern) |
| **G-L4 (Isolation)** | Reiner Workspace-Export mit nur erlaubten Abhängigkeiten | GRÜN (Build & Tests erfolgreich) |
| **Zero-Panic** | `python3 tools/zero_panic_analyzer.py --crate contextra-infer-onnx` | GRÜN (0 Prod-Verstöße) |
| **Determinismus** | `python3 tools/determinism_guard.py --crate contextra-infer-onnx` | GRÜN (0 Verstöße) |

---

## 3. Abweichungen von Spec / Prompt mit Grund
- Keine Abweichungen vom Auftrag. Alle Anforderungen wurden additiv und abwärtskompatibel umgesetzt.

---

## 4. Getroffene Entscheidungen
- `test-utils` wurde nicht in `default` aufgenommen (`default = []` bleibt unverändert), da kein fremdes Mitglied die Fakes über Default einbindet.
- `EmbeddingProvider` wurde additiv für `OnnxEmbedder` und `FakeEmbedder` implementiert, sodass sowohl Produktions- als auch Test-Setups in Verbrauchern wie `contextra-engine` oder `contextra-mcp` davon profitieren können.

---

## 5. Offene Punkte / Risiken & Übergaben an Z
- **Fremdnutzer:**
  - `crates/contextra-engine/src/contextra_impl/lifecycle.rs:198`: Bindet `contextra_infer_onnx::OnnxEmbedder` ein.
  - `crates/contextra-mcp/src/config.rs:316, 321`: Bindet `contextra_infer_onnx::OnnxEmbedder` ein.
- **Cargo.lock:** Wurde maschinell für `contextra-ports` und `contextra-types` als optionale Workspace-Abhängigkeiten synchronisiert.
- **Hinweis zu `ort` & `onnx`-Feature:** Im Workspace ist `ort` rc.13 gelockt, welches `ndarray 0.17` erfordert, während `Cargo.toml` historisch `rc.12` und `ndarray 0.15` deklarierte. Dies ist eine bestehende Vorbedingung im Workspace und wurde nicht verändert (Scope Guard).
