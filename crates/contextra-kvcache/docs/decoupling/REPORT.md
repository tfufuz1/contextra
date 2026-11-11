# Decoupling Report: contextra-kvcache

## 1. Baseline V0
- Befehl: `cargo check -p contextra-kvcache --no-default-features`
  - Ergebnis: Rot (18 Kompilierungsfehler bezüglich `contextra_ports`, `contextra_types`, `contextra_crypto`, `adapters`, `prefix_store`, `scratchpad_invalidation`).
- Befehl: `cargo check -p contextra-kvcache --all-targets`
  - Ergebnis: Grün.
- Befehl: `cargo test -p contextra-kvcache`
  - Ergebnis: Grün (40 Unit Tests, alle Integrationstests bestanden).
- Fremdnutzer-Inventar:
  - Außerhalb von `crates/contextra-kvcache` existieren keine direkten Verwendungen von `contextra_kvcache` in `crates/` oder `benchmarks/`.
- Feature-Matrix:
  - `std`, `workspace`, `ports-compat`, `crypto-compat`, `kv-encryption`, `content-addressed-kv-cache`, `exp-content-addressed`, `kvcache-kivi-quant`, `kivi-quantization`, `exp-kivi`.

## 2. Arbeitspakete & Commits
- WP-0: Baseline, Inventar und Findings dokumentiert (`a48bafa0d`).
- WP-1: Manifest (`Cargo.toml`) angepasst: `contextra-ids`, `contextra-env`, Feature `workspace` sowie Aliase (`ports-compat`, `types-compat`, `docid-128`) und Dev-Dependencies für Integrationstests deklariert (`c7739aea5`).
- WP-2: Pfadtausch `ids`/`env` im Kern: `TenantId`, `TxId` und `Clock` auf kanonische Typen umgestellt; `RequestId(pub u64)` als lokaler Typ vereinheitlicht mit `From`-Implementierungen in `compat/` (`05ffbf98d`).
- WP-3: Fehlerbehandlung im Kern auf `KvCacheError` umgestellt, Port-Re-Exports und Adapter unter Feature `workspace` bedingt, tabellengetriebenen Fehlerkonvertierungstest angelegt (`8c74bd457`).
- WP-4: Feature-Gating von `contextra-crypto` für `kv-encryption` verifiziert; ohne `kv-encryption` baut der Kern ohne Krypto-Abhängigkeit; Closure geprüft.
- WP-5: Unit-Tests bereinigt, V1..V7 Verifikationen erfolgreich durchgeführt.

## 3. Ausgabe von V5 und V6 am Ende
- V5 Null-Prüfung:
  `grep -rEn 'contextra_(types|ports|testkit)' crates/contextra-kvcache/src --include=*.rs | grep -v '/compat/'`
  Ergebnis: Vollsändig LEER (0 Treffer außerhalb von `src/compat/`).
- V6 Kern-Closure:
  - `cargo tree -p contextra-kvcache -e normal --no-default-features --prefix none | grep '^contextra-' | sort -u`:
    `contextra-env v0.1.0`
    `contextra-ids v0.1.0`
    `contextra-kvcache v0.1.0`
    (Exakt die erlaubte Kern-Teilmenge ohne Feature `kv-encryption`).
  - Mit `--features kv-encryption`:
    Zusätzlich `contextra-crypto`.

## 4. Entscheidungen und Abweichungen vom Prompt
- `RequestId` wurde als Newtype `RequestId(pub u64)` im Kern vereinheitlicht (bit- und typidentisch mit `contextra-ports::RequestId`), da es in allen Eviction-Worker- und Attention-Mappings als 64-Bit-Identifier genutzt wird. `From`-Konvertierungen in `src/compat/mod.rs` und flexible `impl Into<RequestId>` Signaturen sichern Abwärtskompatibilität.
- In `store.rs` wurde der Unit-Test `test_kv_cache_pruner_and_scanner_ports` nach `src/compat/scanner.rs` verlagert, damit im Kern keine Referenzen auf `contextra_ports::scanner` verbleiben.
- `KvCacheError` erhielt den Konstruktor `pin_budget_exceeded(...)`, um bit- und semantikgleiche Fehlermeldungen bei Pin-Budget-Überschreitung zu erzeugen.

## 5. Fassaden-Liste (T2-Fälle)
- Keine T2-Fassaden nötig, da alle Modulreexporte und Trait-Methoden unter Default-Features unverändert erhalten geblieben sind (`From<KvCacheError> for ContextraError` fängt alle `?`-Aufrufe transparent ab).

## 6. Fremdnutzer-Inventar & Übergaben
- Fremdnutzer-Inventar:
  Keine Workspace-Crates außerhalb unseres Eigentums binden `contextra-kvcache` direkt ein.
- Zukünftige Manifest-Änderungen für Default-Flip:
  Sobald `workspace` nicht mehr Standard ist, müssten Crates, die Ports-Traits wie `KvPrefixStore` oder `KvCachePruner` nutzen, `contextra-kvcache = { features = ["workspace"] }` deklarieren.

## 7. Offene Punkte / Risiken
- Keine Risiken für `contextra-kvcache`. Die Integrationstests und Unit-Tests sind zu 100% grün.
