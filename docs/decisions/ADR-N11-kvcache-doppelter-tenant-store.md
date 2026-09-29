# ADR-N11: Architektur-Klärung: Doppelter TenantIsolatedKvStore (contextra-kvcache vs. contextra-crypto)

* **Status:** Proposed
* **Datum:** 2026-09-28
* **Kontext:** Auftrag K-01 (Attention-basierte KV-Eviction verdrahten)

---

## 1. Belegter Ist-Zustand (Stand HEAD)

Im Repository existieren derzeit **zwei getrennte Implementierungen** der Datenstruktur `TenantIsolatedKvStore`:

1. **`crates/contextra-kvcache/src/store.rs` (`TenantIsolatedKvStore`)**:
   - **Eigenschaften:** Enthält den vollständigen In-Memory Prefix-Radix-Baum (`PrefixRadixTree`), Aufmerksamkeits-basierte Eviction (`AttentionScoreSource`, SnapKV/H2O-Algorithmus), Tiering und Guard-Schutz (`KvBlockGuard`).
   - **Nutzung:** Wird von **KEINEM** anderen Crate im Workspace importiert (`cargo tree -i contextra-kvcache` liefert 0 Konsumenten; es existiert nur ein Eintrag in `[workspace.dependencies]` der Root-`Cargo.toml`).

2. **`crates/contextra-crypto/src/kv_segment/store.rs` (`TenantIsolatedKvStore`)**:
   - **Eigenschaften:** Bietet AEAD-Verschlüsselung (`KvSegmentCipher`), kryptografisches Shredding, Zeroization (`ZeroizeOnDrop`) und reines LRU-Eviction.
   - **Nutzung:** Wird produktiv von allen wesentlichen Subsystemen verwendet: `contextra-engine`, `contextra-infer-candle` (`kv_bridge.rs`) und `contextra-mcp`.
   - **Defizit:** Die Crypto-Variante besitzt **keine Anbindung an Attention-Gewichte** oder den Attention-Exporter-Port (`AttentionScoreSource`).

---

## 2. Optionen zur Architektur-Konsolidierung

### Option 1: Konsumenten auf `contextra-kvcache` umstellen
* **Beschreibung:** Alle produktiven Konsumenten (`contextra-engine`, `contextra-infer-candle`, `contextra-mcp`) stellen ihren KV-Store von `contextra-crypto::kv_segment::TenantIsolatedKvStore` auf `contextra-kvcache::TenantIsolatedKvStore` um.
* **Erforderliche Änderungen:**
  - Neue Abhängigkeitskante von `contextra-infer-candle` / `contextra-engine` auf `contextra-kvcache` in `Cargo.toml`.
  - Aktualisierung von `capabilities.toml` und `xtask` zur Freigabe der neuen Crate-Graph-Kante.
* **Bewertung:**
  - *Layering (Ring-DAG):* Saubere Trennung zwischen Ring 0 (`contextra-crypto`) und Ring 1 (`contextra-kvcache`).
  - *Sicherheit:* `contextra-kvcache` nutzt intern bereits `contextra-crypto` für Verschlüsselung/Zeroize.
  - *Aufwand & Risiko:* Mittel bis hoch; erfordert koordinierte Refactorings über mehrere Ringe.

### Option 2: Attention-Eviction in `contextra-crypto` portieren
* **Beschreibung:** Der Attention-Port (`AttentionScoreSource`) und der Ranking-Algorithmus werden in die Crypto-Variante portiert.
* **Erforderliche Änderungen:**
  - Die Trait-Definition `AttentionScoreSource` muss aus `contextra-kvcache` in ein von `contextra-crypto` erreichbares Ring-0-Crate (z. B. `contextra-ports` oder `contextra-types`) verlagert werden.
  - `contextra-crypto/src/kv_segment/store.rs` wird um den `AttentionScoreSource`-Parameter in `evict_fair` erweitert.
* **Bewertung:**
  - *Layering:* Sehr gut, da Ring-0-Crates keine Zyklen aufbauen.
  - *Sicherheit:* Höchste Sicherheit, da AEAD-Encrypted Segmente direkt im Krypto-Sicherheitsbereich evictet werden.
  - *Aufwand & Risiko:* Gering bis mittel; erfordert Verschiebung der Trait-Definition in `contextra-ports`.

### Option 3: Deprecate / Entfernen von `contextra-kvcache`
* **Beschreibung:** `contextra-kvcache` wird als redundantes Crate depubliziert/entfernt oder vollständig in `contextra-crypto` integriert.
* **Bewertung:**
  - *Layering:* Reines Flattening.
  - *Risiko:* Verlust der Prefix-Radix-Optimierung, falls diese in `contextra-crypto` nicht nachgezogen wird.

---

## 3. Empfehlung & Entscheidungsvorbehalt

**Empfehlung der System-Architektur:** Option 2 (Portierung des `AttentionScoreSource`-Ports in `contextra-ports` und Integration in `contextra-crypto::kv_segment::TenantIsolatedKvStore`) bietet die sauberste Integration bei minimalem Risiko für bestehende produktive Pipeline-Pfade.

> **HINWEIS:** Die finale Entscheidung über die Option und die Freigabe der dafür erforderlichen `capabilities.toml`- und `Cargo.toml`-Änderungen **muss von einem menschlichen Architekten / Tech Lead getroffen werden**. In Auftrag K-01 wurde der Attention-Eviction-Pfad innerhalb von `contextra-kvcache` vollständig und regressionsfrei verdrahtet.
