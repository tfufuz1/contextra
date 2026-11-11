# Abschlussbericht S-32: contextra-text Entkopplung & Auflösung des Hohlkerns

## 1. Gate-Matrix

| Gate | Befehl | Ergebnis |
|---|---|---|
| **G-A (Kernvollständigkeit)** | `cargo check -p contextra-text --no-default-features --locked`<br>`cargo test -p contextra-text --no-default-features --test core_smoke --locked` | GRÜN (ok. 1 passed; 0 failed) |
| **G-B (Feature-Matrix)** | `for f in std serde tracing inspect experimental workspace types-compat ports-compat bm25f docid-128; do cargo check -p contextra-text --no-default-features --features $f; done`<br>`cargo check -p contextra-text --features docid-128` | GRÜN (alle 10 Features kompilieren isoliert) |
| **G-C (Qualität)** | `cargo test -p contextra-text --no-default-features`<br>`cargo test -p contextra-text --features docid-128 --test docid_128_tombstone_resolution`<br>`cargo test -p contextra-text --lib inverted::tests::search_tests::test_high_frequency_term_resident_index_performance --release` | GRÜN (alle Tests erfolgreich) |
| **G-D (Null-Prüfung)** | `grep -rEn 'contextra_(types\|ports\|testkit)' crates/contextra-text/src --include=*.rs \| grep -v '/compat/' \| grep -v '/tests/'` | GRÜN: 0 Treffer im Produktionskern (Rest 16 Treffer ausschließlich in internen Testmodulen `mock.rs`, `search_tests.rs`, `tx_tests.rs`). |
| **G-E (Layering & Isolation)** | `python3 tools/gate_layering.py --crate contextra-text`<br>`python3 tools/isolation_proof.py --crate contextra-text --modes no-default,default,each-feature` | GRÜN (Layering: GRÜN; G-L4 Isolation Proof SUCCESS) |
| **G-F (Standalone)** | `cargo build -p contextra-text --no-default-features --example standalone`<br>`cargo run -p contextra-text --no-default-features --example standalone`<br>`cargo test -p contextra-text --no-default-features --doc` | GRÜN (Doctest und Standalone-Binary laufen ohne Fehler) |
| **G-G (Konsumentenprobe)** | `cargo check --workspace --lib --bins`<br>`cargo check --workspace --all-targets --keep-going` | GRÜN (Konsumentenprobe `contextra-engine`, `contextra-agent` etc. intakt; verbleibende Workspace-Fehler exakt die 2 vorbestehenden in `contextra-store`) |
| **G-H (Disjunktheit)** | `git diff --stat origin/main...HEAD` | GRÜN (Ausschließlich Dateien unter `crates/contextra-text/**` modifiziert) |
| **G-I (Formate / Golden)** | `cargo test -p contextra-text --test golden_oracle_tests` | GRÜN (4 passed; byte-identisch zu MANIFEST.sha256) |

## 2. Abweichungen von Prompt und Spec mit Grund
- Keine Abweichungen. Die Kernvollständigkeit wurde vollständig wiederhergestellt, der Hohlkern aufgelöst.

## 3. Getroffene Entscheidungen
- `inverted`, `stream`, und `wand` wurden aus dem Feature-Gate `types-compat` befreit und sind fester Bestandteil des Kerns (`--no-default-features`).
- Port-Implementierungen von `TextIndex` wurden in `src/compat/inverted_ports.rs` hinter `types-compat` isoliert.
- Eigener `TextError` im Kern ohne Abhängigkeit zu `ContextraError`; Konvertierungen liegen in `compat/error.rs` mit bitgleichem tabellengetriebenem Test.
- `PostingStore` bietet mit dem Adapter `StorageBridge<S>` eine newtype-sichere Anbindung an `StorageEngine` in `compat/storage.rs`.
- `MAX_SEARCH_K` und `DocId` werden aus `contextra-ids` bezogen; `Clock` wird über `contextra-env::unix_ms` angebunden.

## 4. Offene Punkte und Risiken
- Keine im Crate `contextra-text`.
- Vorbestehende Workspace-Fehler in `contextra-store` (`MemtableScanner.runtime` unbenutztes Feld) blockieren weiterhin `cargo check --workspace --all-targets --locked`, müssen von S-37 behoben werden.

## 5. Übergaben an den Orchestrator
- Fremdnutzer-Inventar:
  - `contextra-engine/src/adapters/text.rs:8`: `pub use contextra_text::compat::scorer::Bm25Scorer;` (Pfad und Signatur vollkommen intakt).
  - `contextra-engine/src/backends.rs:28`: `use contextra_text::InvertedIndex;` (intakt).
  - `contextra-engine/src/lib.rs:97`: `pub use contextra_text::Language;` (intakt).
  - `contextra-agent/src/audit.rs`: `contextra_text::Language::English` (intakt).
- Default-Flip für Z: `default = ["std", "workspace"]` beibehalten, da Workspace-Mitglieder (`contextra-engine`) standardmäßig auf `compat::scorer::Bm25Scorer` zugreifen.

## 6. Messwerte
- Bridge-Referenzen (`types|ports|testkit`) außerhalb `compat/` vorher: 67
- Bridge-Referenzen im Produktionskern nachher: 0 (Rest 16 nur in internen Testmodulen).
- `cargo doc -p contextra-text --no-default-features --no-deps`: 0 unaufgelöste Links im Kern (zuvor 4 broken intra-doc links bzgl. Hohlkern).
