# AGENTS.md — `contextra-kvcache`

> **Klasse:** A (Stand-alone Baukasten) · Reifegrad `beta` · Unsafe: nein (`#![forbid(unsafe_code)]`) · Rolle: Verschlüsselter KV-Prefix-Cache für LLM-Zustände (Radix, Eviction, KIVI-Quantisierung).
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-kvcache --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | keine (transitional: `contextra-types`, `contextra-ports`, `contextra-crypto` hinter optionalen Features) |
| Contextra-Abhängigkeiten **Soll** (SPEC §5) | keine |
| Features | `kv-encryption`, `kivi-quantization`, `content-addressed-kv-cache`, `kvcache-attention-eviction`, `kvcache-kivi-quant`, `ports-compat`, `crypto-compat` |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- Autarker Stand-alone Baustein (Klasse A).
- Interne Workspace-Abhängigkeiten wurden entkoppelt und als Übergangs-Features (`ports-compat`, `crypto-compat`) optionalisiert.
- Keine juristische Löschkettenlogik (DSGVO Art. 17, Sagarecord, Witness, Purger): Datenhaltende Schicht bietet nur neutrales Invalidate/Evict/Destroy.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `types.rs`, `error.rs`, `cipher.rs` | Lokale Domaintypen, typisierte Fehler (`KvCacheError`), `SegmentCipher`-Trait |
| `store.rs`, `service.rs`, `prefix_store.rs`, `adapters.rs` | Cache-Dienst, Port-Adapter `KvPrefixStore` |
| `radix.rs`, `segment.rs`, `eviction_worker.rs` | Radix-Baum, Segmente, Eviction |
| `quantize_kivi.rs`, `attention_score.rs`, `scratchpad_invalidation.rs` | Quantisierung, Aufmerksamkeits-Score, Invalidierung |
| `compat.rs` | Übergangs-Konvertierungen für `ports-compat` und `crypto-compat` |

## 4. Verträge & Invarianten

1. **Mandantentrennung (INV-KV-1):** Inhalts-Adressierung darf nie tenantübergreifend treffen (`content_addressed_cross_tenant_isolation.rs`).
2. **Krypto-Invalidierung (INV-KV-2):** `SegmentKey::destroy()` überschreibt das Schlüsselmaterial sicher und macht Segmente unlesbar.
3. **Zero-Panic (INV-KV-3):** Kein `unwrap()`/`expect()` im Produktivcode; `KvCacheError` für alle Fehlerfälle.
4. **AEAD vor Quantisierung-Reihenfolge** wie getestet (`kivi_aead_order_enforced.rs`).
5. Pinned-Basis bleibt bei Eviction erhalten (`clm_pinned_base_preserved.rs`).

## 5. Ports

**Angeboten:** `SegmentCipher` (kryptografische Block-/Segmentverschlüsselung), `KvSpillStorage`.
**Optional / Compat:** `KvPrefixStore` (hinter `ports-compat`).

## 6. Nebenläufigkeit & Performance

- Eviction-Worker als Hintergrundakteur mit Abbruch; keine geteilten Mutables außerhalb der Store-Struktur.

## 7. Anti-Patterns

- Cache-Treffer über Tenant-Grenzen hinweg.
- Eviction ohne Pinning-Prüfung.
- Löschbeweise, DSGVO-Ketten oder Saga-Logik im Speicherbaustein.

## 8. Tests & Messung

```bash
cargo test -p contextra-kvcache
cargo test -p contextra-kvcache --no-default-features
cargo clippy -p contextra-kvcache --all-targets -- -D warnings
cargo bench -p contextra-kvcache --no-run
```
