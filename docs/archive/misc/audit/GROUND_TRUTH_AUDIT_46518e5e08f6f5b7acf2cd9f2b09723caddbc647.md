# Ground-Truth-Audit des Repository-Zustands (Teil A)

> **Audit-Metadaten**
> * **Commit-SHA:** `46518e5e08f6f5b7acf2cd9f2b09723caddbc647`
> * **Referenz-Spezifikation:** `CONTEXTRA_SPEC_v2.md` (Teil A & Teil A2 Ring-0–4-Modell)
> * **Audit-Modus:** READ-ONLY AUDIT (Keine Änderungen an Produktionscode)
> * **Claim-Status:** `cargo xtask claim --crate workspace-audit --readonly` (reiner Lesezugriff)

---

## 1. Executive Summary & Top-3 Abweichungen (Soll vs. Ist)

1. **Unvollständige Ring-Migration (Crate-DAG vs. Ring-0–4-Modell):**
   Von den in `CONTEXTRA_SPEC_v2.md` (§0.1 "Jetzt") geforderten 29 Ziel-Crates (27 Fach- + 2 Tooling-Crates) sind im Repository erst 11 Ziel-Crates vorhanden. 18 Ziel-Crates fehlen vollständig (z. B. `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-wire`, `contextra-simd`, `contextra-vector`, `contextra-engine`, `contextra-cognition`, `contextra`). Parallel existieren noch 8 Alt-Crates aus der v1-Schichtenarchitektur (z. B. `contextra-core`, `contextra-index`, `contextra-db`).

2. **Fehlgeschlagene Lint-Mechanik (Rust-Semantikverletzung E0453):**
   Das Root-Manifest `Cargo.toml` erzwingt global `[workspace.lints.rust] unsafe_code = "forbid"`. Dadurch schlägt jeglicher Kompilierungs- und Clippy-Lauf (`cargo clippy --workspace --all-targets --locked -- -D warnings`) fehl, da Untermodule wie `contextra-index` oder `contextra-store` mit `#[allow(unsafe_code)]` versuchen, Unsafe-Operationen (AVX512, `memmap2`) lokal freizugeben, was von Rust mit Fehler `E0453` verboten wird.

3. **Struktureller P5-Schichtenverstoß & KV-Cache-Bridge-Stub:**
   Der Orchestrator-Crate `contextra-db` importiert in `crates/contextra-db/Cargo.toml` direkt die aufwärts gelagerten Inferenz-Crates `contextra-candle`, `contextra-ollama` und `contextra-embed` (Verstoß gegen Prinzip P5). Zudem wurde der KV-Cache-Bridge-Code (§9.2) am aktuellen HEAD als Stub verifiziert: `consult_segment` inkrementiert lediglich einen `AtomicU64`-Zähler, ohne echten Prefill-Skip durchzuführen.

---

## 2. Crate-Existenz & Zielarchitektur-Abgleich (§0.1 / §4)

Das folgende Tableau vergleicht die in `CONTEXTRA_SPEC_v2.md` (§0.1 "Jetzt") definierten Ziel-Crates mit dem Ist-Zustand unter `crates/`:

| Ziel-Crate (§0.1 "Jetzt") | Ring / Kategorie | Status im Repository | Pfad / Anmerkung |
|---|---|---|---|
| `contextra-types` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `contextra-ports` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `contextra-mvcc` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `contextra-wire` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `contextra-sys` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `contextra-simd` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `contextra-crypto` | Ring 0 | **Existiert** | `crates/contextra-crypto` |
| `contextra-vector` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `contextra-text` | Ring 0 | **Existiert** | `crates/contextra-text` |
| `contextra-graph` | Ring 0 | **Existiert** | `crates/contextra-graph` |
| `contextra-rank` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `contextra-adapt` | Ring 0 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `contextra-store` | Ring 1 | **Existiert** | `crates/contextra-store` |
| `contextra-kvcache` | Ring 1 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `contextra-checkpoint` | Ring 1 | **Existiert** | `crates/contextra-checkpoint` |
| `contextra-infer-candle` | Ring 2 | **FEHLT** | Existiert nur als Alt-Crate `contextra-candle` |
| `contextra-infer-ollama` | Ring 2 | **FEHLT** | Existiert nur als Alt-Crate `contextra-ollama` |
| `contextra-infer-onnx` | Ring 2 | **FEHLT** | Existiert nur als Alt-Crate `contextra-embed` |
| `contextra-sandbox` | Ring 2 | **Existiert** | `crates/contextra-sandbox` |
| `contextra-engine` | Ring 3 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `contextra-cognition` | Ring 3 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `contextra-privacy` | Ring 3 | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `contextra-router` | Ring 3 | **Existiert** | `crates/contextra-router` |
| `contextra-agent` | Ring 3 | **Existiert** | `crates/contextra-agent` |
| `contextra` | Ring 4 | **FEHLT** | Target Composition Root fehlt |
| `contextra-mcp` | Ring 4 | **Existiert** | `crates/contextra-mcp` |
| `contextra-py` | Ring 4 | **Existiert** | `crates/contextra-py` |
| `contextra-testkit` | Tooling | **FEHLT** | Ziel-Crate noch nicht angelegt |
| `contextra-bench` | Tooling | **Existiert** | `benchmarks/contextra-bench` |

### Verbleibende Alt-Crates im Repository (noch nicht migriert):
- `contextra-calibration` (Ring 0 / `contextra-rank` Kandidat)
- `contextra-candle` (Ring 2 / `contextra-infer-candle` Kandidat)
- `contextra-core` (Ring 0 / Zerschlagungs-Kandidat für `contextra-types`, `contextra-ports`, `contextra-mvcc`)
- `contextra-core-ipc-gen` (Ring 0 / `contextra-wire` Kandidat)
- `contextra-db` (Ring 3 / Zerschlagungs-Kandidat für `contextra-engine`, `contextra-cognition`)
- `contextra-embed` (Ring 2 / `contextra-infer-onnx` Kandidat)
- `contextra-index` (Ring 0 / `contextra-vector` Kandidat)
- `contextra-ollama` (Ring 2 / `contextra-infer-ollama` Kandidat)

---

## 3. Unsafe- & Lint-Politik Audit (§A2, ADR N03)

Gemäß Spec v2 (§A2, §0.4) gilt `#![forbid(unsafe_code)]` als Pflicht-Direktive in allen Produktions-Crates, ausgenommen der drei expliziten Unsafe-Inseln (`contextra-sys`, `contextra-simd`, `contextra-wire`).

| Crate | `#![forbid(unsafe_code)]` im Code? | Reales Verhalten / Befund |
|---|---|---|
| `contextra-agent` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `contextra-calibration` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `contextra-candle` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `contextra-checkpoint` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `contextra-core` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `contextra-core-ipc-gen` | **NEIN** | Fehlt (FlatBuffers Generat verwendet `unsafe`) |
| `contextra-crypto` | **JA (Konditional)** | `#![cfg_attr(not(test), forbid(unsafe_code))]` |
| `contextra-db` | **NEIN** | **FEHLT** (Keine Forbid-Direktive in `src/lib.rs`) |
| `contextra-embed` | **NEIN (Nur Deny)** | `#![deny(unsafe_code)]` statt `forbid` |
| `contextra-graph` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `contextra-index` | **NEIN (Nur Deny)** | `#![deny(unsafe_code)]` + AVX512 Unsafe-Blöcke |
| `contextra-mcp` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `contextra-ollama` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `contextra-py` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `contextra-router` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `contextra-sandbox` | **JA** | `#![forbid(unsafe_code)]` gesetzt |
| `contextra-store` | **NEIN (Nur Deny)** | `#![deny(unsafe_code)]` + Mmap Unsafe-Blöcke |
| `contextra-text` | **JA** | `#![forbid(unsafe_code)]` gesetzt |

### Rust Lint-Inkompatibilität (Fehler E0453):
In der Root `Cargo.toml` ist konfiguriert:
```toml
[workspace.lints.rust]
unsafe_code = "forbid"
```
Wenn ein Unter-Crate (wie `contextra-index` oder `contextra-store`) versucht, für Mmap- oder AVX512-Routinen lokal `#[allow(unsafe_code)]` zu setzen, führt dies zu einem harten Rust-Kompilierungsfehler (`error[E0453]: allow(unsafe_code) incompatible with previous forbid`). Dies bestätigt den in Spec v2 (§A2) dokumentierten Defekt der Lint-Mechanik.

---

## 4. Panic-, Unwrap- & Expect-Inventar (Nicht-Test-Code)

Inventur aller `.unwrap()`, `.expect()` und `panic!` / `unreachable!` / `todo!` / `unimplemented!` Vorkommen in `src/` (ausschließlich Nicht-Test-Code) über alle 18 existierenden Workspace-Crates:

| Crate | `.unwrap()` | `.expect()` | `panic!`/`unreachable!`/`todo!` | Gesamt Panic-Risiko |
|---|---|---|---|---|
| `contextra-agent` | 9 | 7 | 1 | **17** |
| `contextra-calibration` | 15 | 1 | 0 | **16** |
| `contextra-candle` | 93 | 4 | 5 | **102** |
| `contextra-checkpoint` | 72 | 24 | 3 | **99** |
| `contextra-core` | 111 | 64 | 21 | **196** |
| `contextra-core-ipc-gen` | 16 | 0 | 0 | **16** |
| `contextra-crypto` | 140 | 97 | 7 | **244** |
| `contextra-db` | 491 | 272 | 27 | **790** |
| `contextra-embed` | 13 | 0 | 3 | **16** |
| `contextra-graph` | 589 | 41 | 1 | **631** |
| `contextra-index` | 249 | 120 | 9 | **378** |
| `contextra-mcp` | 76 | 27 | 9 | **112** |
| `contextra-ollama` | 118 | 4 | 9 | **131** |
| `contextra-py` | 21 | 3 | 4 | **28** |
| `contextra-router` | 184 | 27 | 9 | **220** |
| `contextra-sandbox` | 0 | 0 | 0 | **0** |
| `contextra-store` | 45 | 270 | 4 | **319** |
| `contextra-text` | 8 | 7 | 2 | **17** |
| **GESAMT (Workspace)** | **2.052** | **943** | **105** | **3.100** |

---

## 5. P5-Schichten-Verstoß Audit (§A2.1)

In `crates/contextra-db/Cargo.toml` wurden folgende direkte Abhängigkeiten festgestellt:

```toml
[dependencies]
...
contextra-ollama = { workspace = true }
contextra-candle = { workspace = true }
contextra-embed = { workspace = true, optional = true }
```

**Befund:** `contextra-db` (Layer 2 Orchestrator) importiert direkt aufwärts gelagerte Inferenz-Crates aus Layer 3 (`contextra-candle`, `contextra-ollama`, `contextra-embed`). Dies verletzt das Prinzip **P5 (Keine Aufwärts-Abhängigkeiten / Strict Layer Hierarchy)** eindeutig.

---

## 6. KV-Cache-Bridge Stub-Befund (§9.2)

In `crates/contextra-candle/src/kv_bridge.rs` (Zeile 124–133) ist `consult_segment` wie folgt implementiert:

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

In `crates/contextra-mcp/src/lib.rs` (Zeile 694–707) wird dies beim Retrieval aufgerufen:
```rust
#[cfg(feature = "kv-bridge")]
if let Some(ref bridge) = self.kv_bridge {
    for res in &results {
        let chunk_id = DocId::from_key(&res.id).map(|d| d.as_u64()).unwrap_or(0);
        let text = ...;
        let segment = contextra_core::traits::ContextSegment::new(chunk_id, text);
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
   --> crates/contextra-store/src/wal/replay.rs:105:13
    |
105 |     #[allow(unsafe_code, clippy::type_complexity)]
    |             ^^^^^^^^^^^ overruled by previous forbid
    |
    = note: `forbid` lint level was set on command line (`-F unsafe_code`)

error: usage of an `unsafe` block
   --> crates/contextra-store/src/wal/replay.rs:118:20
    |
118 |   ...   let mmap = unsafe {
    |  __________________^
...   |
125 | | ...           .map_err(|e| ContextraError::Storage(format!("WAL mmap fa...
126 | | ...   };
    | |_______^
    |
    = note: requested on the command line with `-F unsafe-code`

error[E0453]: allow(unsafe_code) incompatible with previous forbid
   --> crates/contextra-store/src/wal/replay.rs:153:13
    |
153 |     #[allow(unsafe_code)]
    |             ^^^^^^^^^^^ overruled by previous forbid
    |
    = note: `forbid` lint level was set on command line (`-F unsafe_code`)

error: usage of an `unsafe` block
   --> crates/contextra-store/src/wal/replay.rs:171:24
    |
171 | ...   let mmap = unsafe { memmap2::Mmap::map(&std_file) }.map_err(|e| {
    |                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0453]: allow(unsafe_code) incompatible with previous forbid
    --> crates/contextra-index/src/distance.rs:1071:1
     |
1071 | unsafe fn euclidean_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
...
error: could not compile `contextra-index` (lib) due to 98 previous errors
error: could not compile `contextra-store` (lib) due to 4 previous errors
```

---

## 8. Fazit & Empfehlung für Stabilisierung (Teil A)

Bevor neue Features entwickelt werden können, müssen gemäß `CONTEXTRA_SPEC_v2.md` Teil A folgende Erstschritte erfolgen:
1. **Behebung E0453 Lint-Mechanik:** Entfernung von `unsafe_code = "forbid"` aus `[workspace.lints.rust]` in der Root `Cargo.toml`. Stattdessen Konfiguration von `#![forbid(unsafe_code)]` auf Ebene der jeweiligen Nicht-Insel-Crates.
2. **Abbau P5-Verstoß in `contextra-db`:** Abkopplung der Inferenz-Abhängigkeiten aus `contextra-db`.
3. **Schrittweise Entfrachtung von `.unwrap()` / `.expect()`:** Reduktion der 3.100 Panic-Risikostellen zur Herstellung der Zero-Panic-Doctrine.
