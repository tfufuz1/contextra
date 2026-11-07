# AGENTS.md — `contextra-kvcache`

> **Ring 1** · Klasse `[K]` im SPEC-Katalog (SPEC §0.5 archiviert; Root-§10 K5) · Reifegrad `beta` · Unsafe: nein · Rolle: Verschlüsselter KV-Prefix-Cache für LLM-Zustände (Radix, Eviction, KIVI-Quantisierung).
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-kvcache --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `crypto`, `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types`, `crypto` |
| Features | `kv-encryption`, `kivi-quantization`, `content-addressed-kv-cache`, `kvcache-attention-eviction`, `kvcache-kivi-quant` |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- Kanten Ist = Soll.
- 6 Treffer mit Shredding-Bezug (`kv_shredding`): Beziehung zu `crypto::kv_shredding` klären, bevor `crypto` bereinigt wird (WP 3.1).
- Abhängig von `crypto::kv_cipher`/`kv_segment` (Zuordnung ADR).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `store.rs`, `service.rs`, `prefix_store.rs`, `adapters.rs` | Cache-Dienst, Port-Adapter `KvPrefixStore` |
| `radix.rs`, `segment.rs`, `eviction_worker.rs` | Radix-Baum, Segmente, Eviction |
| `quantize_kivi.rs`, `attention_score.rs`, `scratchpad_invalidation.rs` | Quantisierung, Aufmerksamkeits-Score, Invalidierung |

## 4. Verträge & Invarianten

1. **Mandantentrennung:** Inhalts-Adressierung darf nie tenantübergreifend treffen (`content_addressed_cross_tenant_isolation.rs`).
2. **AEAD vor Quantisierung-Reihenfolge** wie getestet (`kivi_aead_order_enforced.rs`).
3. Fehlt die Fähigkeit, ist der Cache **inaktiv** (kein stilles Fallback); Löschschicht `KvCache` ist `FeatureDisabled` (SPEC §4.7).
4. Pinned-Basis bleibt bei Eviction erhalten (`clm_pinned_base_preserved.rs`).

## 5. Ports

**Implementiert:** `KvPrefixStore`. **Konsumiert:** `ports`, `crypto`.

## 6. Nebenläufigkeit & Performance

- Eviction-Worker als Hintergrundakteur mit Abbruch (`cancellation.rs`); keine geteilten Veränderlichen außerhalb der Store-Struktur.

## 7. Anti-Patterns

- Cache-Treffer über Tenant-Grenzen.
- Eviction ohne Pinning-Prüfung.

## 8. Tests & Messung

```bash
cargo test -p contextra-kvcache --locked
cargo clippy -p contextra-kvcache --all-targets -- -D warnings
```

- 15 Testdateien (Isolation, Eviction, Golden-Decode, Roundtrip).

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 3.x | Shredding-Bezug klären |
| 1.1 | Klasse/Reifegrad in `capabilities.toml` verbindlich setzen |
