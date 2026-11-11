# Entkopplungsbericht: contextra-infer-ollama (AUFTRAG P8)

## 1. Baseline V0

- **Stand origin/main**: Commit `5408826f8a42f5de339eb44b57de9bb03dc7412b`
- **Baseline Checks**:
  - `cargo check -p contextra-infer-ollama --no-default-features`: GRÜN
  - `cargo check -p contextra-infer-ollama --features client`: ROT (vorbestehender doppelter Syntax-/Implementationsblock am Ende von `src/embedding.rs` mit ungültigen Membern)
  - `cargo test -p contextra-infer-ollama`: 22 passed, 0 failed
  - `cargo clippy -p contextra-infer-ollama --all-targets -- -D warnings`: GRÜN (bei default features)
- **Baseline-Reparatur**:
  - Minimales Entfernen der unvollständigen Doppel-Implementierung von `Embedder` am Ende von `src/embedding.rs`.

---

## 2. Arbeitspakete & Commits

- **WP-0**: Baseline V0 erfassen und minimale Bereinigung der vorbestehenden Syntax-/Duplikatfehler in `embedding.rs`.
  - Befehle: `cargo check -p contextra-infer-ollama --features client`, `cargo test -p contextra-infer-ollama`
  - Ergebnis: GRÜN

- **WP-1**: Feature-Architektur und Manifest-Aktualisierung (`Cargo.toml`).
  - Feature `workspace = ["dep:contextra-types", "dep:contextra-ports"]` eingeführt.
  - `ports-compat` und `types-compat` als Aliase beibehalten (`= ["workspace"]`).
  - `contextra-types` und `contextra-ports` als optionale Pfad-Abhängigkeiten mit `version = "0.1.0"` deklariert; für Tests in `[dev-dependencies]` verankert.
  - Modul-Gate in `src/lib.rs` auf `#[cfg(feature = "workspace")] pub mod compat;` umgestellt.
  - Befehle: `cargo check -p contextra-infer-ollama --no-default-features`, `cargo check -p contextra-infer-ollama --features workspace`
  - Ergebnis: GRÜN

- **WP-2**: Fehlerabbildung, Traits & Mapping-Test.
  - Tabellengetriebener Unit-Test `test_ollama_error_mapping_table` in `src/compat/error.rs` für alle 9 `OllamaError`-Varianten nach `contextra_types::ContextraError` implementiert.
  - Display-Strings und Diskriminanten bitgleich verifiziert.
  - Befehle: `cargo test -p contextra-infer-ollama --features workspace`
  - Ergebnis: GRÜN

- **WP-3**: Kern-Entkopplung und Verlagerung aller ports/types-Abhängigkeiten nach `src/compat/`.
  - `with_ports_rng` aus `src/client/core.rs` nach `src/compat/client.rs` ausgelagert.
  - `src/client/trait_impls.rs` nach `src/compat/trait_impls.rs` verschoben (`LlmTextGenerator`, `LlmTextGeneratorStreaming`, `SegmentSynthesizer`).
  - `EmbeddingProvider`-Trait-Implementierung und Tests aus `src/embedding.rs` nach `src/compat/embedding.rs` ausgelagert.
  - `QueryRewriter`-Trait-Implementierung und `rewrite_structured` aus `src/query_rewriter.rs` nach `src/compat/query_rewriter.rs` ausgelagert.
  - Alle Unit-Tests mit Ports-Bezug (`src/client/tests.rs`) nach `src/compat/client_tests.rs` verschoben.
  - Befehle: `cargo check -p contextra-infer-ollama --no-default-features`, `cargo check -p contextra-infer-ollama --features client`, `cargo test -p contextra-infer-ollama --features workspace,client`
  - Ergebnis: GRÜN (52 unit tests, 7 campaign stub tests, 4 fake server tests, 5 golden tests, 1 backoff test bestanden)

---

## 3. Verifikation V1 bis V7

- **V1 (`--no-default-features`)**:
  - `cargo check -p contextra-infer-ollama --no-default-features` -> GRÜN
- **V2 (Targets & Feature-Kombinationen)**:
  - `cargo check -p contextra-infer-ollama --all-targets` -> GRÜN
  - `cargo check -p contextra-infer-ollama --no-default-features --features workspace --all-targets` -> GRÜN
  - `cargo check -p contextra-infer-ollama --all-features --all-targets` -> GRÜN
- **V3 (Tests)**:
  - `cargo test -p contextra-infer-ollama --lib` -> GRÜN (22 tests passed)
  - `cargo test -p contextra-infer-ollama --features workspace,client` -> GRÜN (69 tests passed, 2 ignored wie erwartet)
- **V4 (Clippy)**:
  - `cargo clippy -p contextra-infer-ollama --all-targets -- -D warnings` -> GRÜN (0 Warnungen)
- **V5 (Null-Prüfung)**:
  - `grep -rEn 'contextra_(types|ports|testkit)' crates/contextra-infer-ollama/src --include=*.rs | grep -v '/compat/'`
  - **Ergebnis: LEER (0 Treffer)**.
- **V6 (Kern-Closure)**:
  - `cargo tree -p contextra-infer-ollama -e normal --no-default-features --prefix none | grep '^contextra-' | sort -u`
  - **Ergebnis**:
    ```
    contextra-infer-ollama v0.1.0 (/app/crates/contextra-infer-ollama)
    ```
    Keinerlei interne `contextra-*`-Abhängigkeiten im Kern!
- **V7 (Workspace-Probe)**:
  - `cargo check -p contextra-mcp --all-targets` (der einzige Fremdnutzer von `contextra-infer-ollama` im Workspace):
    - Kompiliert `contextra-infer-ollama` fehlerfrei.
    - Vorbestehende Fehler im Workspace in unfertigen Nachbar-Branches (`contextra-store`, `contextra-checkpoint`) sind außerhalb des Eigentums und wurden nicht berührt.

---

## 4. Entscheidungen und Abweichungen

- **Keine internen Kern-Abhängigkeiten nötig**: `contextra-infer-ollama` benötigt weder `contextra-env` noch `contextra-ids` im Kern. Die lokalen Traits `LocalClock` und `LocalRng` in `src/error.rs` genügen vollständig und entkoppeln den HTTP-Client komplett.
- **Feature-Defaults**: Default-Features bleiben `[]` (unverändert leer).

---

## 5. Fassaden-Liste (T2-Fälle)

- `OllamaClient::with_ports_rng`: Liegt in `src/compat/client.rs` hinter `#[cfg(feature = "workspace")]` als inhärente Methode am identischen Pfad, ruft intern den Adapter für `LocalRng`.
- `OllamaEmbedder`: Implementiert den Ports-Trait `contextra_ports::EmbeddingProvider` in `src/compat/embedding.rs` hinter `#[cfg(feature = "workspace")]`.
- `OllamaQueryRewriter`: Implementiert `contextra_ports::QueryRewriter` und stellt `rewrite_structured` in `src/compat/query_rewriter.rs` hinter `#[cfg(feature = "workspace")]` bereit.

---

## 6. Fremdnutzer-Inventar

- `crates/contextra-mcp/Cargo.toml:16`: `ollama = ["dep:contextra-infer-ollama"]`
- `crates/contextra-mcp/src/config.rs:307`: `contextra_infer_ollama::OllamaEmbedder::new(...)`
- `crates/contextra-mcp/src/config.rs:460`: `contextra_infer_ollama::OllamaConfig { ... }`
- `crates/contextra-mcp/src/config.rs:465`: `contextra_infer_ollama::OllamaClient::with_config(...)`
- `crates/contextra-rank/src/lib.rs:27`: Doc-Kommentar-Erwähnung.

Für den späteren Default-Flip von `contextra-infer-ollama` muss `contextra-mcp` (wenn `workspace` aus `default` entfernt wird) ggf. `contextra-infer-ollama = { workspace = true, optional = true, features = ["workspace", "client"] }` in `crates/contextra-mcp/Cargo.toml` deklarieren.

---

## 7. Offene Punkte / Risiken

- Keine verbleibenden Risiken für `contextra-infer-ollama`. Alle Verträge und Golden-Tests sind bitgenau und stabil.
