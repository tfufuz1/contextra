# contextra-vector — Crate-Spezifikation (Soll-Zustand, SPEC 2.1)

**Ebene:** E1 (Domäne)
**Domänen-Closure:** `ids`, `env`, `simd`, `durable-fs`
**Verträge:** `ports` (`VectorIndex`), `erasure` (`VectorLayer`). Beide zählen nicht gegen die fünf (SPEC §15). `VectorIndex` wird hier implementiert und nicht neu definiert.
**Status:** Normativ (Soll-Zustand)
**Version:** 2.1
**Geltungsbereich:** `crates/contextra-vector`
**Normative Quellen:** `docs/SPEC.md` §4.7, §5.3 Schritte 3 und 8, §5.9 Schicht S2, §5.10, §6.2, §6.3, §7 (`VectorLayer`), INV-5, INV-HNSW-SLOT-1, VETO-F02, INV-ARTKEY-1.

---

## 1. Zweck

`contextra-vector` ist der HNSW-Index: ANN-Suche, skalare Quantisierung SQ8 und SQ4, Distanz über `contextra-simd` mit bitidentischem skalarem Fallback (INV-5).

Schicht **S2**. Persistente Vektor-Payloads (fp32 und SQ8) liegen je Slot unter dem `DocDek`. Die HNSW-Topologie einer Generation liegt unter einem `ArtKey`. Die Versiegelung ruft `contextra-db` über `contextra-crypto` auf. Dieses Crate persistiert Chiffretext und hält die Arbeitsarena.

## 2. Architekturregeln

1. `#![forbid(unsafe_code)]`. Zero-Panic. `default = []`.
2. Keine Abhängigkeit auf andere E1-Crates und keine Dev-Abhängigkeit auf `contextra-testkit`.
3. `VectorLayer` implementiert `ErasureLayer` aus `contextra-erasure`.
4. **VETO-F02.** Beim Löschen gibt es kein Rewiring und keine heuristische Neuvernetzung von Nachbarn.
5. **INV-HNSW-SLOT-1.** Ein `Slot` (`u32`, dicht je Collection) wird erst nach dem abgeschlossenen Neuaufbau der Generation `g+1` wiederverwendet.
6. Die `LiveMask` aus `contextra-erasure` ist der Kandidatenfilter. Eine Suche überspringt gelöschte Slots, bevor Distanz und Fusion laufen.

## 3. Rolle in der Löschsaga

Messung: `VectorScanner`, plus Black-Box-Probe des Orchestrators. Positivkontrolle `AC-1` ist Pflicht. `purge` ist idempotent (T-DEL-14).

### 3.1 Schritt 3 — Ebene A, Arena

Vor dem Übergang `PlaintextPurged` im Journal:

1. Die Slots der Einheit werden in der fp32-Arena und in der SQ8-Arena deterministisch genullt (`zeroize`).
2. Kanten bleiben stehen. Es wird kein Nachbar neu verbunden (VETO-F02).
3. Die Slot-Nummer bleibt bis zum Neuaufbau gesperrt.
4. Der persistente Snapshot-Payload ist bereits unter dem `DocDek`. Seine Unlesbarkeit entsteht in Schritt 5. Schritt 3 entfernt den Klartext aus der Arena.

Das Restresiduum `R-GRAPH` (Topologie ohne Nutzdaten, interne Slot-Nummern) bleibt im Bericht, bis Schritt 8 die Generation wechselt (REQ-DEL-N25).

### 3.2 Schritt 8 — Generation `g+1`

Auslöser, gemeinsam für mehrere Löschungen: Tombstone-Anteil ≥ `tombstone_rebuild_ratio` (Start 0,2), Frist `purge_rebuild_max_days` (Start 14 Tage), oder `seal-now`.

1. Ein Hintergrundworker baut Generation `g+1` nur aus lebenden Slots. Der Aufbau ist deterministisch (INV-INDEX-APPLY-1, INV-5).
2. Atomarer Umschalt-Commit auf `g+1` (`tmp` → `fsync` → `rename` → Verzeichnis-`fsync`).
3. `contextra-crypto` vernichtet den `ArtKey` der Generation `g` (INV-ARTKEY-1). Danach schließt der Koordinator `R-GRAPH`.
4. Slots der Generation `g` werden wiederverwendbar.

`VectorScanner` prüft Rohbytes der Embeddings und die dekodierte Sicht. Ohne bestandene Positivkontrolle ist der Lauf ungültig.

### 3.3 Suche

`search_knn` liest die `LiveMask` mit `SeqCst`, bevor ein Kandidat die Distanzberechnung erreicht (§6.3, §10.3). Verworfene Kandidaten zählt der Aufrufer in `contextra-db`, der die Trefferliste auf `k` auffüllt. Dieses Crate fusioniert nicht.

SQ4 ist eine Suchoptimierung. Die Löschpflicht gilt für jede persistierte Quantisierungsstufe, die Nutzbytes der Einheit enthält. SQ4-Bytes eines gelöschten Slots werden in Schritt 3 mit genullt.

## 4. Fehler (`VectorError`)

`#[non_exhaustive]`: `DimensionMismatch`, `InvalidFloat`, `CorruptedStorage`, `Io`, `GenerationSwitchFailed`, `SlotStillLive`. Ein Konnektivitätsfehler durch unterlassenes Rewiring ist kein Fehlerbild. Der Neuaufbau ist der einzige Reparaturweg.

## 5. `Cargo.toml`

```toml
[package]
name = "contextra-vector"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "HNSW vector index with generation rebuild and LiveMask filtering"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-simd = { workspace = true }
contextra-durable-fs = { workspace = true }
contextra-ports = { workspace = true }
contextra-erasure = { workspace = true }
ahash = { workspace = true }
parking_lot = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
proptest = "1.4"

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
quantize = []
```

`quantize` schaltet SQ4 zu. Die Löschpfade von fp32 und SQ8 sind nicht feature-gesteuert (R3).

## 6. Tests

| Kennung | Pflicht |
|---|---|
| T-DEL-6 | Nach `Frozen` liefert die Vektorsuche das Dokument nicht |
| T-DEL-14 | Zweites Arena-`purge` ist ein No-op |
| T-DEL-17 | Rohbytes der fp32- und SQ8-Arena enthalten das Embedding nicht mehr |
| T-DEL-19 | `remember` während gehaltener Slot-Sperre wird abgewiesen |

Ein Test, der nur die `LiveMask` setzt und die Arena nicht nullt, prüft höchstens Klasse L (§12).
