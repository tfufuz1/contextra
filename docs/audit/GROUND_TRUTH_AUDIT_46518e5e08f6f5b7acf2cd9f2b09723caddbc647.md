# Ground-Truth-Audit des Repository-Zustands (Teil A)

> **Audit-Metadaten**
> * **Commit-SHA:** `46518e5e08f6f5b7acf2cd9f2b09723caddbc647`
> * **Referenz-Spezifikation:** `MEMFUSE_SPEC_v2.md` (Teil A & Teil A2 Ring-0–4-Modell)
> * **Audit-Modus:** READ-ONLY AUDIT (Keine Änderungen an Produktionscode)
> * **Claim-Status:** `cargo xtask claim --crate workspace-audit --readonly` (reiner Lesezugriff)

---

## 1. Executive Summary & Top-3 Abweichungen (Soll vs. Ist)

1. **Unvollständige Ring-Migration (Crate-DAG vs. Ring-0–4-Modell):**
   Von den in `MEMFUSE_SPEC_v2.md` (§0.1 "Jetzt") geforderten 29 Ziel-Crates (27 Fach- + 2 Tooling-Crates) sind im Repository erst 11 Ziel-Crates vorhanden. 18 Ziel-Crates fehlen vollständig (z. B. `memfuse-types`, `memfuse-ports`, `memfuse-mvcc`, `memfuse-wire`, `memfuse-simd`, `memfuse-vector`, `memfuse-engine`, `memfuse-cognition`, `memfuse`). Parallel existieren noch 8 Alt-Crates aus der v1-Schichtenarchitektur (z. B. `memfuse-core`, `memfuse-index`, `memfuse-db`).

2. **Fehlgeschlagene Lint-Mechanik (Rust-Semantikverletzung E0453):**
   Das Root-Manifest `Cargo.toml` erzwingt global `[workspace.lints.rust] unsafe_code = "forbid"`. Dadurch schlägt jeglicher Kompilierungs- und Clippy-Lauf (`cargo clippy --workspace --all-targets --locked -- -D warnings`) fehl, da Untermodule wie `memfuse-index` oder `memfuse-store` mit `#[allow(unsafe_code)]` versuchen, Unsafe-Operationen (AVX512, `memmap2`) lokal freizugeben, was von Rust mit Fehler `E0453` verboten wird.

3. **Struktureller P5-Schichtenverstoß & KV-Cache-Bridge-Stub:**
   Der Orchestrator-Crate `memfuse-db` importiert in `crates/memfuse-db/Cargo.toml` direkt die aufwärts gelagerten Inferenz-Crates `memfuse-candle`, `memfuse-ollama` und `memfuse-embed` (Verstoß gegen Prinzip P5). Zudem wurde der KV-Cache-Bridge-Code (§9.2) am aktuellen HEAD als Stub verifiziert: `consult_segment` inkrementiert lediglich einen `AtomicU64`-Zähler, ohne echten Prefill-Skip durchzuführen.

---

## 2. Crate-Existenz & Zielarchitektur-Abgleich (§0.1 / §4)

Das folgende Tableau vergleicht die in `MEMFUSE_SPEC_v2.md` (§0.1 "Jetzt") definierten Ziel-Crates mit dem Ist-Zustand unter `crates/`:

| Ziel-Crate (§0.1 "Jetzt") | Ring / Kategorie | Status im Repository | Pfad / Anmerkung |
|---|---|---|---|
| `memfuse-types` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `memfuse-ports` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `memfuse-mvcc` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `memfuse-wire` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `memfuse-sys` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `memfuse-simd` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `memfuse-crypto` | Ring 0 | **Existiert** | `crates/memfuse-crypto` |
| `memfuse-vector` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `memfuse-text` | Ring 0 | **Existiert** | `crates/memfuse-text` |
| `memfuse-graph` | Ring 0 | **Existiert** | `crates/memfuse-graph` |
| `memfuse-rank` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `memfuse-adapt` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `memfuse-store` | Ring 1 | **Existiert** | `crates/memfuse-store` |
| `memfuse-kvcache` | Ring 1 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `memfuse-checkpoint` | Ring 1 | **Existiert** | `crates/memfuse-checkpoint` |
| `memfuse-infer-candle` | Ring 2 | **FEHLT** | Existiert nur als Alt-Crate `memfuse-candle` |
| `memfuse-infer-ollama` | Ring 2 | **FEHLT** | Existiert nur als Alt-Crate `memfuse-ollama` |
| `memfuse-infer-onnx` | Ring 2 | **FEHLT** | Existiert nur als Alt-Crate `memfuse-embed` |
| `memfuse-sandbox` | Ring 2 | **Existiert** | `crates/memfuse-sandbox` |
| `memfuse-engine` | Ring 3 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `memfuse-cognition` | Ring 3 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `memfuse-privacy` | Ring 3 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `memfuse-router` | Ring 3 | **Existiert** | `crates/memfuse-router` |
| `memfuse-agent` | Ring 3 | **Existiert** | `crates/memfuse-agent` |
| `memfuse` | Ring 4 | **FEHLT** | Target Composition Root fehlt |
| `memfuse-mcp` | Ring 4 | **Existiert** | `crates/memfuse-mcp` |
| `memfuse-py` | Ring 4 | **Existiert** | `crates/memfuse-py` |
| `memfuse-testkit` | Tooling | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `memfuse-bench` | Tooling | **Existiert** | `benchmarks/memfuse-bench` |

### Verbleibende Alt-Crates im Repository (noch nicht migriert):
- `memfuse-calibration` (Ring 0 / `memfuse-rank` Kandidat)
- `memfuse-candle` (Ring 2 / `memfuse-infer-candle` Kandidat)
- `memfuse-core` (Ring 0 / Zerschlagungs-Kandidat für `memfuse-types`, `memfuse-ports`, `memfuse-mvcc`)
- `memfuse-core-ipc-gen` (Ring 0 / `memfuse-wire` Kandidat)
- `memfuse-db` (Ring 3 / Zerschlagungs-Kandidat für `memfuse-engine`, `memfuse-cognition`)
- `memfuse-embed` (Ring 2 / `memfuse-infer-onnx` Kandidat)
- `memfuse-index` (Ring 0 / `memfuse-vector` Kandidat)
- `memfuse-ollama` (Ring 2 / `memfuse-infer-ollama` Kandidat)

---

## 3. Unsafe- & Lint-Politik Audit (§A2, ADR N03)

Gemäß Spec v2 (§A2, §0.4) gilt `#![forbid(unsafe_code)]` als Pflicht-Direktive in allen Produktions-Crates, ausgenommen der drei expliziten Unsafe-Inseln (`memfuse-sys`, `memfuse-simd`, `memfuse-wire`).

| Crate | `#![forbid(unsafe_code)]` im Code? | Reales Verhalten / Befund |
|---|---|---|
| `memfuse-agent` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `memfuse-calibration` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `memfuse-candle` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `memfuse-checkpoint` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `memfuse-core` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `memfuse-core-ipc-gen` | **NEIN** | Fehlt (FlatBuffers Generat verwendet `unsafe`) |
| `memfuse-crypto` | **JA (Konditional)** | `#![cfg_attr(not(test), forbid(unsafe_code))]` |
| `memfuse-db` | **NEIN** | **FEHLT** (Keine Forbid-Direktive in `src/lib.rs`) |
| `memfuse-embed` | **NEIN (Nur Deny)** | `#![deny(unsafe_code)]` statt `forbid` |
| `memfuse-graph` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `memfuse-index` | **NEIN (Nur Deny)** | `#![deny(unsafe_code)]` + AVX512 Unsafe-Blöcke |
| `memfuse-mcp` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `memfuse-ollama` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `memfuse-py` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `memfuse-router` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `memfuse-sandbox` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `memfuse-store` | **NEIN (Nur Deny)** | `#![deny(unsafe_code)]` + Mmap Unsafe-Blöcke |
| `memfuse-text` | **JA** | `#![forbid(unsafe_code)]` gesetzt |

### Rust Lint-Inkompatibilität (Fehler E0453):
In der Root `Cargo.toml` ist konfiguriert:
```toml
[workspace.lints.rust]
unsafe_code = "forbid"
```
Wenn ein Unter-Crate (wie `memfuse-index` oder `memfuse-store`) versucht, für Mmap- oder AVX512-Routinen lokal `#[allow(unsafe_code)]` zu setzen, führt dies zu einem harten Rust-Kompilierungsfehler (`error[E0453]: allow(unsafe_code) incompatible with previous forbid`). Dies bestätigt den in Spec v2 (§A2) dokumentierten Defekt der Lint-Mechanik.

---

## 4. Panic-, Unwrap- & Expect-Inventar (Nicht-Test-Code)

Inventur aller `.unwrap()`, `.expect()` und `panic!` / `unreachable!` / `todo!` / `unimplemented!` Vorkommen in `src/` (ausschließlich Nicht-Test-Code) über alle 18 existierenden Workspace-Crates:

| Crate | `.unwrap()` | `.expect()` | `panic!`/`unreachable!`/`todo!` | Gesamt Panic-Risiko |
|---|---|---|---|---|
| `memfuse-agent` | 9 | 7 | 1 | **17** |
| `memfuse-calibration` | 15 | 1 | 0 | **16** |
| `memfuse-candle` | 93 | 4 | 5 | **102** |
| `memfuse-checkpoint` | 72 | 24 | 3 | **99** |
| `memfuse-core` | 111 | 64 | 21 | **196** |
| `memfuse-core-ipc-gen` | 16 | 0 | 0 | **16** |
| `memfuse-crypto` | 140 | 97 | 7 | **244** |
| `memfuse-db` | 491 | 272 | 27 | **790** |
| `memfuse-embed` | 13 | 0 | 3 | **16** |
| `memfuse-graph` | 589 | 41 | 1 | **631** |
| `memfuse-index` | 249 | 120 | 9 | **378** |
| `memfuse-mcp` | 76 | 27 | 9 | **112** |
| `memfuse-ollama` | 118 | 4 | 9 | **131** |
| `memfuse-py` | 21 | 3 | 4 | **28** |
| `memfuse-router` | 184 | 27 | 9 | **220** |
| `memfuse-sandbox` | 0 | 0 | 0 | **0** |
| `memfuse-store` | 45 | 270 | 4 | **319** |
| `memfuse-text` | 8 | 7 | 2 | **17** |
| **GESAMT (Workspace)** | **2.052** | **943** | **105** | **3.100** |

---

## 5. P5-Schichten-Verstoß Audit (§A2.1)

In `crates/memfuse-db/Cargo.toml` wurden folgende direkte Abhängigkeiten festgestellt:

```toml
[dependencies]
...
memfuse-ollama = { workspace = true }
memfuse-candle = { workspace = true }
memfuse-embed = { workspace = true, optional = true }
```

**Befund:** `memfuse-db` (Layer 2 Orchestrator) importiert direkt aufwärts gelagerte Inferenz-Crates aus Layer 3 (`memfuse-candle`, `memfuse-ollama`, `memfuse-embed`). Dies verletzt das Prinzip **P5 (Keine Aufwärts-Abhängigkeiten / Strict Layer Hierarchy)** eindeutig.

---

## 6. KV-Cache-Bridge Stub-Befund (§9.2)

In `crates/memfuse-candle/src/kv_bridge.rs` (Zeile 124–133) ist `consult_segment` wie folgt implementiert:

```rust
pub fn consult_segment<'a>(&self, segment: &ContextSegment<'a>) {
    self.consultations
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let _ = (
        segment.chunk_id,
        segment.text,
        segment.model_fingerprint,
        segment.rope_offset,
    );
}
```

In `crates/memfuse-mcp/src/lib.rs` (Zeile 694–707) wird dies beim Retrieval aufgerufen:
```rust
#[cfg(feature = "kv-bridge")]
if let Some(ref bridge) = self.kv_bridge {
    for res in &results {
        let chunk_id = DocId::from_key(&res.id).map(|d| d.as_u64()).unwrap_or(0);
        let text = ...;
        let segment = memfuse_core::traits::ContextSegment::new(chunk_id, text);
        bridge.consult_segment(&segment);
    }
}
```

**Befund:** Bestätigt. Der KV-Cache-Bridge Code führt keine tatsächliche Prefill-Einsparung oder KV-Token-Wiederverwendung durch, sondern hochzählt lediglich einen `AtomicU64`-Zähler und verwirft die Segmentdaten via `let _ = (...)`.

---

## 7. Clippy & Kompilierungs-Status am aktuellen HEAD

Der Befehl `cargo clippy --workspace --all-targets --locked -- -D warnings` wurde am aktuellen HEAD ausgeführt. Das Ergebnis ist ein sofortiger Abbruch due to `error[E0453]`.

### Auszug der ersten 50 Fehlerzeilen:

```
error[E0453]: allow(unsafe_code) incompatible with previous forbid
   --> crates/memfuse-store/src/wal/replay.rs:105:13
    |
105 |     #[allow(unsafe_code, clippy::type_complexity)]
    |             ^^^^^^^^^^^ overruled by previous forbid
    |
    = note: `forbid` lint level was set on command line (`-F unsafe_code`)

error: usage of an `unsafe` block
   --> crates/memfuse-store/src/wal/replay.rs:118:20
    |
118 |   ...   let mmap = unsafe {
    |  __________________^
...   |
125 | | ...           .map_err(|e| MemFuseError::Storage(format!("WAL mmap fa...
126 | | ...   };
    | |_______^
    |
    = note: requested on the command line with `-F unsafe-code`

error[E0453]: allow(unsafe_code) incompatible with previous forbid
   --> crates/memfuse-store/src/wal/replay.rs:153:13
    |
153 |     #[allow(unsafe_code)]
    |             ^^^^^^^^^^^ overruled by previous forbid
    |
    = note: `forbid` lint level was set on command line (`-F unsafe_code`)

error: usage of an `unsafe` block
   --> crates/memfuse-store/src/wal/replay.rs:171:24
    |
171 | ...   let mmap = unsafe { memmap2::Mmap::map(&std_file) }.map_err(|e| {
    |                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0453]: allow(unsafe_code) incompatible with previous forbid
    --> crates/memfuse-index/src/distance.rs:1071:1
     |
1071 | unsafe fn euclidean_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
...
error: could not compile `memfuse-index` (lib) due to 98 previous errors
error: could not compile `memfuse-store` (lib) due to 4 previous errors
```

---

## 8. Fazit & Empfehlung für Stabilisierung (Teil A)

Bevor neue Features entwickelt werden können, müssen gemäß `MEMFUSE_SPEC_v2.md` Teil A folgende Erstschritte erfolgen:
1. **Behebung E0453 Lint-Mechanik:** Entfernung von `unsafe_code = "forbid"` aus `[workspace.lints.rust]` in der Root `Cargo.toml`. Stattdessen Konfiguration von `#![forbid(unsafe_code)]` auf Ebene der jeweiligen Nicht-Insel-Crates.
2. **Abbau P5-Verstoß in `memfuse-db`:** Abkopplung der Inferenz-Abhängigkeiten aus `memfuse-db`.
3. **Schrittweise Entfrachtung von `.unwrap()` / `.expect()`:** Reduktion der 3.100 Panic-Risikostellen zur Herstellung der Zero-Panic-Doctrine.
