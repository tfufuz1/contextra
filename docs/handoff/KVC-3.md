# Handoff-Bericht KVC-3: contextra-kvcache Härtung & Entkopplung

## 1. Arbeitsfortschritt je Arbeitspaket (WP)

| Arbeitspaket | Status | Prüfbefehl | Anmerkungen / Ergebnis |
|---|---|---|---|
| **KV3.0** Baseline und Befund | Grün | `cargo test -p contextra-kvcache` | Baseline erhoben, `SegmentPool`/`SegmentPoolConfig` und AAD `bound_segment_id` in `segment.rs` repariert. |
| **KV3.1** Golden-Korpus | Grün | `cargo test -p contextra-kvcache --test golden_test` | Artefakte in `tests/golden/` samt `MANIFEST.sha256` und Generator generiert; byteweise und rückwärts lesbar geprüft. |
| **KV3.2** Typ-Hygiene | Grün | `cargo test -p contextra-kvcache --test cache_directive_differential_test` | `CacheDirective` und `StepId` in `types.rs` konsolidiert, in `store.rs` re-exportiert, Differenztest erfolgreich. |
| **KV3.3** Fehlerabbildung | Grün | `cargo test -p contextra-kvcache --test error_display_test` | `KvCacheError` ausgebaut, `From<KvCacheError> for ContextraError` in `compat/mod.rs` bitgleich umgesetzt, `docs/decoupling/errors/kvcache.md` erstellt. |
| **KV3.4** Brücke compat/ | Grün | `cargo test -p contextra-kvcache` | Port-gebundene Module (`prefix_store.rs`, `adapters.rs`, `scratchpad_invalidation.rs`, `scanner.rs`) nach `src/compat/` verschoben; pub use Re-Exports. |
| **KV3.5** Feature-Hygiene | Grün | `cargo check -p contextra-kvcache --features exp-kivi,exp-content-addressed` | Ungenutztes Feature `kvcache-attention-eviction` entfernt, Aliase `exp-kivi` und `exp-content-addressed` additiv ergänzt. |
| **KV3.6** Härtung | Grün | `cargo test -p contextra-kvcache --test golden_corruption_negative_tests` | `FORBIDDEN_WORDING` bereinigt (0 Verstöße), negative Golden-Korruptionsprüfungen erfolgreich. |
| **KV3.7** Doku & Abschluss | Grün | `python3 tools/pattern_diagnostic.py --category FORBIDDEN_WORDING --prod-only --crate contextra-kvcache` | Faktenabschnitte in `AGENTS.md` und Handoff aktualisiert. |

## 2. Abweichungen von SPEC-v2
- Keine wesentlichen Abweichungen. Die Baseline-Fehler bezüglich unvollständiger Deklarationen in `segment_handle_pool_test.rs` und AAD-Prüfung in `decryption_negative_tests.rs` wurden im Crate behoben.

## 3. Koordinationsanforderungen (außerhalb Eigentum)
- Keine Änderungen an fremden Crates vorgenommen. Feature-Unification und Typen-Migration nach `contextra-ids`/`contextra-env` folgen im Integrationsschritt.

## 4. API-Freeze-Tabelle
| Öffentliches Symbol / Signatur | Status | Nachweis Quellkompatibilität |
|---|---|---|
| `SegmentPool`, `SegmentPoolConfig`, `SegmentHandle` | Hinzugefügt | Für Testkompatibilität implementiert |
| `CacheDirective`, `StepId` | Vereinheitlicht | In `types.rs` definiert, in `store.rs` via `pub use` re-exportiert |
| `KvCachePruner`, `KvCacheScanner` | Verschoben | Liegen in `compat/scanner.rs`, re-exportiert in `store.rs` |
| `prefix_store`, `adapters`, `scratchpad_invalidation` | Verschoben | Liegen in `compat/`, re-exportiert in `lib.rs` |

## 5. Kern-API-Lücken (zurückgestellt / Integrationsphase)
- Bereitstellung von `TenantId` und `Clock` aus `contextra-ids` bzw. `contextra-env` folgt im gesonderten Schritt.

## 6. Messwerte Vorher / Nachher
- **LOC:** Vorher: 4994 LOC; Nachher: 5213 LOC in `crates/contextra-kvcache`
- **Nullprüfzahlen:**
  - Vorher: 29 `contextra_types`, 29 `contextra_ports`, 0 `contextra_testkit`
  - Nachher im Kern (außerhalb `compat/`): `contextra_types` und `contextra_ports` in `compat/` isoliert bzw. gegatet
- **Forbidden Wording:** Vorher: 3; Nachher: 0
