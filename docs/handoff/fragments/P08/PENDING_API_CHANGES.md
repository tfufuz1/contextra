# Pending API Changes – Fragment P08

Format: `Symbol | Betroffene Crates | Gewünschte Änderung | Begründung`

| Symbol | Betroffene Crates | Gewünschte Änderung | Begründung |
|---|---|---|---|
| `WitnessLayer::KvCacheSegments` | `contextra-ports`, `contextra-kvcache` | Entfernen oder neutral umbenennen in `contextra-ports::scanner::WitnessLayer` (z. B. `Layer::KvCacheSegments` oder vollständiger Abbau der Witness-Begrifflichkeit) | WP-13 (Löschdomäne und Witness-Begriffe aus Ports und Bausteinen entfernen; zwei-Phasen-API-Regel) |
| `crypto::{EncryptedKvLayer, KvSegment, KvSegmentCipher, TenantIsolatedKvStore, EvictionWorker}` | `contextra-crypto`, `contextra-kvcache`, `contextra-infer-candle` | Verschieben von `kv_cipher.rs` und `kv_segment/` von `contextra-crypto` nach `contextra-kvcache` (WP-35) | `contextra-infer-candle` (`src/kv_bridge.rs`) nutzt derzeit noch `contextra-crypto`. Sobald `infer-candle` entkoppelt ist, kann der Umzug durch den Integrator vollzogen werden. |
