---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "09"
---
## 9. Inferenz, KV-Cache v2 und Zero-Copy-IPC

> **Statuskorrektur, verbindlich (§A2.1):** Die KV-Cache-Bridge war in jeder Vorfassung dieser Spec als 🟢
> markiert. Eine Verifikation am Code (`candle/inference.rs:485`) hat ergeben, dass `generate_with_context`
> nur `format!("kv_cache_tensor:…")` als Platzhalter-„Tensor" ablegt, ein Treffer lediglich einen Zähler
> (`prefill_skip_count`) erhöht, ohne dass tatsächlich Prefill-Rechenzeit eingespart wird, und `generate()`
> in jedem Fall den vollständigen Text neu verarbeitet. **Der Status wird hiermit korrigiert auf 🔴** — die
> KV-Cache-Bridge ist spezifiziert, im Code nicht vorhanden, und bleibt 🔴, bis der in §9.2 genannte
> Golden-Test grün ist.

### 9.1 Zero-Copy-Eviction-Bridge (🟢, unverändert)

Durchgehende Zero-Copy-Datenpipeline auf Basis von `Bytes` und Mmap. FlatBuffers erlaubt direktes Auslesen
aus `&[u8]` ohne Heap-Allokation. Diese Eigenschaft ist von der KV-Cache-Neuspezifikation unberührt.

### 9.2 KV-Cache v2 (🔴, ersetzt §9.2 der Vorfassung vollständig)

**Vorher (nicht mehr normativ — Grund für die Ablösung siehe Statuskorrektur oben):** eine `KvCacheBridge`
mit `scc::HashMap<SessionId, EncryptedKvSegment>` als RAM-Tier und `contextra_store::LsmStore` als
Fallback-Spill; `EncryptedKvSegment` mit `rope_offset` pro Segment. Diese Skizze existierte nur in der Spec,
nicht im Code, und ist außerdem architektonisch problematisch (siehe „Warum kein LSM-Spill" unten).

**Jetzt: drei Ausbaustufen, deklariert über `KvReusePolicy`, Crate `contextra-kvcache` (Ring 1).**

**Fakten aus der Verifikation (§A2, `CONTEXTRA_ZIELARCHITEKTUR_v2.md` §5.1):**
- Upstream (`candle-transformers 0.11.0`, `quantized_llama.rs`): KV ist pro Layer privat
  (`Option<(Tensor, Tensor)>`); jeder Schritt kopiert per `Tensor::cat` (O(n) je Layer und Token);
  `index_pos == 0` verwirft den Cache; Keys werden nach RoPE gecacht; Werte sind F32, weil `QMatMul` F32
  liefert. `LayerWeights.kv_cache` ist **privat**; öffentlich sind nur `forward` und `clear_kv_cache`.
- Größenordnung (Beispiel Llama-3.2-3B, 28 Layer, 8 KV-Köpfe, head_dim 128): 224 KiB/Token, ein 16-Token-Block
  ≈ 3,5 MiB, 2048 Token ≈ 448 MiB. Ein 2-GiB-RAM-Budget hält damit ≈ 4 gleichzeitige Prompts bei F32.
- **Warum kein LSM-Spill (Korrektur gegenüber der „Vorher"-Skizze):** KV-Blöcke sind Megabyte-groß. Eine
  Leveled-Compaction-LSM-Engine schreibt Werte dieser Größe mehrfach um (Größenordnung 10×,
  konfigurationsabhängig) — das ist Write-Amplifikation ohne Gegenwert. Persistenz für KV-Blöcke erfolgt
  stattdessen über **append-only Segmentdateien** in `contextra-kvcache` (Header, Prüfsumme, AEAD; Index wird
  beim Start aus den Segment-Headern aufgebaut). Die LSM-Engine (`contextra-store`) trägt höchstens Metadaten.

| Stufe | Inhalt | Modell | Persistenz | Gate vor Produktions-Default |
|---|---|---|---|---|
| **A** | In-RAM-Prefix-Reuse per `ModelWeights::clone()` nach Prefill an Segmentgrenzen (System-Prompt, Kontextsegmente); Gewichte/KV-Tensoren teilen sich per `Arc`, `cat` erzeugt neue Tensoren | Upstream unverändert, kein Fork | keine | Logit-Vergleich Clone vs. Neuberechnung (bitgleich erwartet bei gleicher Batch-Form) |
| **B** | Eigenes Llama-Modell (Fork von `quantized_llama.rs`, Lizenz vorab prüfen) mit erstklassigem `KvState`: `truncate`, `export_block`, `import_block`; Radix-Baum, Copy-on-Write, Referenzzählung; RAII-Guards geben Blöcke bei `Drop` zurück (cancel-sicher) | eigenes Modell, an Upstream-Logits verifiziert | RAM | Golden-, Property- (Export→Import identisch), Cancellation-Test |
| **C** | Spill in T2: verschlüsselte Segmentdateien, AEAD mit AAD aus Tenant/Fingerprint/Token-Hash/Positionsbereich; Löschen per Crypto-Shredding des Tenant-Schlüssels | eigenes Modell (B) | Platte | Isolations- und Crash-Test der Segmente |

**Entscheidungsstatus (offen, §A2.4 Nr. 2):** Stufe A liefert bereits Nutzen ohne Fork-Risiko und ist der
nächste konkrete Schritt. Ob B/C gebaut werden, entscheidet sich erst nach Messung von A — das ist keine
Verzögerung, sondern eine bewusste Sequenzierung, damit die Fork-Wartungslast (Risiko Nr. 2, §A2.4) nicht
ungeprüft eingegangen wird.

```rust
// contextra-ports — Vertrag, gilt ab Stufe B
pub struct PrefixKey {
    pub model: ModelFingerprint,
    pub tokenizer_hash: [u8; 32],
    pub layout: KvLayout,   // n_layer, n_kv_head, head_dim, dtype
    pub rope: RopeConfig,  // base, scaling
}

pub trait KvPrefixStore: Send + Sync {
    /// Längster exakt passender Prefix in Blöcken; kein Treffer ist kein Fehler.
    fn lookup(&self, tenant: TenantId, key: &PrefixKey, tokens: &[u32]) -> Option<KvPrefixHit>;
    fn insert(&self, tenant: TenantId, key: &PrefixKey, tokens: &[u32], blocks: Vec<KvBlock>) -> Result<(), KvError>;
}

// contextra-infer-candle — KV wird erstklassig, nicht privat (Stufe B)
pub struct KvState { layers: Vec<LayerKv>, pos: usize }
pub struct LayerKv { k: Tensor, v: Tensor }   // Keys nach RoPE, wie im Upstream-Modell

impl KvState {
    pub fn truncate(&mut self, pos: usize);
    pub fn export_block(&self, range: Range<usize>) -> Result<KvBlock, InferError>;
    pub fn import_block(&mut self, block: &KvBlock) -> Result<(), InferError>;
}
```

**Golden-Test-Kriterium (verschärft gegenüber der Vorfassung):** Die feste Toleranz 1e-4 aus der ursprünglichen
Zielarchitektur-Prüfung entfällt — quantisierte Kernel liefern bei unterschiedlicher Batch-Form (Prefill n
gegen m + Rest) nicht zwingend bitgleiche Summen. Stattdessen gilt: (i) identische Greedy-Tokenfolge über ein
festes Prompt-Set (≥ 200 Prompts × 64 Token); (ii) `max |Δlogit| ≤ 2 ×` gemessene Run-to-Run-Varianz des
Upstream-Modells. Zusätzlich: Tenant-Isolationstest (kein Teilen von Blöcken über Tenants oder Modelle hinweg
— sonst verrät Cache-Hit-Timing fremde Prompts), Cancellation-Test (Abbruch mitten im Prefill gibt alle
Blöcke zurück).

**Metrik-Korrektur (Sofortmaßnahme, unabhängig vom Stufenausbau):** `prefill_skip_count` zählt derzeit jeden
Cache-„Treffer" auf den Platzhalter hoch, obwohl kein Prefill übersprungen wird. Dieser Zähler wird korrigiert,
bevor er in Dashboards oder Abnahmekriterien verwendet wird (§20, Phase 0R).

### 9.3 AES-Schlüsselplan-Wiederverwendung — globaler Zustand entfernt (P29)

**Vorher (nicht mehr normativ):**

```rust
use aes_gcm_siv::Aes256GcmSiv;
use std::sync::OnceLock;

static CIPHER_INSTANCE: OnceLock<Aes256GcmSiv> = OnceLock::new();

pub fn cipher() -> &'static Aes256GcmSiv { unimplemented!() }
```

Dieser Entwurf verstößt gegen P29 (kein globaler veränderlicher Zustand, §3): ein einzelner globaler Cipher
kann nicht pro Tenant/Schlüssel rotiert werden und widerspricht der in `contextra-crypto` beschriebenen
Schlüsselhierarchie.

**Jetzt (verbindlich):** Die Cipher-Instanz wird **pro Schlüssel** im `KeyManager` (Instanzzustand, nicht
`static`) gecacht und thread-safe über `OnceCell`-Felder wiederverwendet:

```rust
pub struct KeyManager {
    ciphers: scc::HashMap<KeyId, OnceCell<Aes256GcmSiv>>,
    // Nonce-Strategie: ⚖️ Entscheidung ausstehend (§A2.4 Nr. 5). KEIN In-Memory-Zähler ohne Hochwasserstand.
}

impl KeyManager {
    pub fn cipher_for(&self, key_id: KeyId) -> Result<&Aes256GcmSiv, CryptoError>;
}
```

**Nonce-Strategie (⚖️ ausstehend, Fassung 2.1):** Fassung 2 sah einen deterministischen `AtomicU64`-Zähler statt
zufälliger Nonces vor. So ist das nicht übernehmbar: Ein Zähler im RAM beginnt nach einem Neustart wieder bei 0
und wiederholt Nonces unter demselben Schlüssel. Ist-Zustand im Repo: 4-Byte-Präfix je Schlüssel plus 8 Byte
`OsRng`, bewusst ohne persistierten Zähler. Optionen: (a) Status quo mit festgelegtem Nachrichtenbudget je Schlüssel
(bei 64 Zufallsbits liegt die Kollisionswahrscheinlichkeit bei etwa $n^2/2^{65}$, also $pprox 2^{-32}$ bei
$n pprox 9\cdot 10^4$ Nachrichten); (b) persistierter Epochenzähler (4 Byte, bei jedem Öffnen erhöht und vor der
ersten Nutzung `fsync`'d) ‖ `AtomicU64`-Zähler (8 Byte). AES-256-GCM-SIV ist nonce-misuse-resistent: eine
Wiederholung offenbart nur die Gleichheit identischer Klartexte, kompromittiert aber den Schlüssel nicht.
Schlüsselrotation erfolgt über gezielten Austausch des `OnceCell`-Eintrags je `KeyId`,
nicht über einen globalen Austausch.

### 9.4 Ring-2/3-Crates (vormals „Layer-3-Crates")

- **`contextra-infer-ollama`** (vormals `contextra-ollama`): `OllamaClient::generate(prompt, contextual_prefix) -> Result<String, OllamaError>`.
  Contextual-Chunk-Prefixing fügt Retrieval-Kontext als System-Präfix ein.
- **`contextra-infer-onnx`** (vormals `contextra-embed`): `EmbeddingModel::embed(texts) -> Result<Vec<Vec<f32>>, EmbedError>`
  (ONNX, aus `default-members` ausgeschlossen, §0.2/§0.3), `CrossEncoderReranker::rerank(query, candidates) -> Vec<SearchResult>`.
- **`contextra-agent`:** `AgentWorkflow`-Engine mit persistentem Zustand über `Checkpointable`. Unverändert im
  Zuschnitt, jetzt Ring 3.
- **`contextra-py`:** PyO3-Bindings. **Δ gegenüber Vorfassung:** die Panic-Strategie-Isolation war spezifiziert,
  aber am Root-Profil (`panic = "abort"`) real wirkungslos (`catch_unwind` ist im Release-Build mit
  `panic = "abort"` funktionslos). Ab dieser Fassung: Root-Profil `panic = "unwind"` (§0.2), `contextra-py` als
  regulärer Workspace-Member, `release-abort` nur für Binaries ohne FFI. Testpflicht: ein Panic muss im
  `maturin build --release`-Wheel als `PyErr` ankommen, nicht nur im Debug-Build.

`contextra-py` ist damit — anders als in der Vorfassung dokumentiert — reguläres Workspace-Mitglied, nicht
„eigener Workspace"; die davon abweichende Doku-Behauptung in `contextra-py-ci.yml` und den Python-Tests wird
im selben Zug korrigiert (§20, Phase 0R, Track T3).

---

<a id="10-sicherheit"></a>
