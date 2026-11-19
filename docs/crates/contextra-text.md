# contextra-text — Crate-Spezifikation (Soll-Zustand, SPEC 2.1)

**Ebene:** E1 (Domäne)
**Domänen-Closure:** `ids`, `env`, `durable-fs`
**Verträge:** `ports` (`TextIndex`), `erasure` (`TextLayer`)
**Status:** Normativ (Soll-Zustand)
**Version:** 2.1
**Geltungsbereich:** `crates/contextra-text`
**Normative Quellen:** `docs/SPEC.md` §4.7, §5.3 Schritt 3, §5.9 Schicht S3, §5.10, §6.3, §7 (`TextLayer`), INV-5, INV-ARTKEY-1, INV-DERIVED-1.

---

## 1. Zweck

`contextra-text` ist der invertierte Volltextindex: deutscher Tokenizer, Kompositazerlegung, BM25 und BM25F, Block-Max-WAND.

Postings, Term-Wörterbuch und Dokumentlängen sind Index-Artefakte der Klasse D. Sie liegen je Segment unter einem zufälligen `ArtKey` und tragen eine `LineageRef` auf das Quelldokument. Die Versiegelung und die Vernichtung des `ArtKey` führt `contextra-crypto` aus, aufgerufen von `contextra-db`.

Ranking ist deterministisch. Bei gleichem BM25-Score gewinnt die kleinere `DocId` (INV-5).

## 2. Architekturregeln

1. `#![forbid(unsafe_code)]`. Zero-Panic. `default = []`.
2. Keine Abhängigkeit auf andere E1-Crates. Keine Dev-Abhängigkeit auf `contextra-testkit`.
3. `TextLayer` implementiert `ErasureLayer` aus `contextra-erasure`.
4. Posting-Iteratoren prüfen die `LiveMask`, bevor ein Treffer die Fusionsstufe erreicht (§6.2, §6.3).

## 3. BM25

$$BM25(D, Q) = \sum_{t \in Q} IDF(t) \cdot \frac{f(t, D) \cdot (k_1 + 1)}{f(t, D) + k_1 \cdot \left(1 - b + b \cdot \frac{|D|}{\mathrm{avgdl}}\right)}$$

Startwerte: $k_1 = 1{,}2$, $b = 0{,}75$. IDF, Dokumentlängen und Zähler liegen im Segment und werden bei jeder Neuschreibung neu berechnet (§5.10).

## 4. Rolle in der Löschsaga

Schicht **S3**, Ebene A und Ebene B. Messung: `TextScanner` (Rohbyte-Scan kanonischer Terme und dekodierte Sicht) plus Black-Box-Probe. Positivkontrolle `AC-1`. `purge` ist idempotent.

### 4.1 Schritt 3 — Segment-Neuschreibung

Nach dem Journal-Übergang `PlaintextPurged`:

1. Das betroffene Segment wird ohne den Scope neu geschrieben.
2. Terme, die nur im gelöschten Scope vorkamen, verlassen das Wörterbuch.
3. BM25-Statistik (IDF, Längen, Zähler) wird neu berechnet.
4. Das neue Segment wird unter einem frisch erzeugten `ArtKey` versiegelt. Der alte `ArtKey` wird sofort vernichtet (INV-ARTKEY-1).
5. Austausch über `contextra-durable-fs`: `tmp` → `fsync` → `rename` → Verzeichnis-`fsync`.

Die Neuschreibung ist der Löschschritt dieses Crates. Ein bloßes Ausblenden im Iterator reicht für Klasse P nicht.

### 4.2 Schritt 2 — Lineage

Die Kaskade selbst liegt in `contextra-db` (`LineageIndex`, INV-DERIVED-1). `TextLayer::resolve` liefert die Segmente, deren `LineageRef` auf den Scope zeigt. Ableitungen mit weiteren, nicht gelöschten Quellen werden als `stale` markiert oder mit dem neuen Segment neu aufgebaut. Dieses Crate entscheidet das nicht allein. Es setzt die Markierung um, die der Koordinator übergibt.

### 4.3 Suche

Ein Posting, dessen Slot in der `LiveMask` gelöscht ist, wird übersprungen. Die Statistik des noch nicht neu geschriebenen Segments kann bis zum Abschluss von Schritt 3 ein Restkanal sein. Nach der Neuschreibung ist die Statistik neu berechnet. Treffermengen sind bis dahin nur klassenweise zugesichert (§5.10).

## 5. Fehler (`TextError`)

`#[non_exhaustive]`: `CorruptedSegment`, `Tokenizer`, `InvalidPath`, `Io`, `ArtKeyRotationIncomplete`. Ein unvollständiger `ArtKey`-Wechsel lässt `purge` mit Fehler enden. Der Saga-Zustand bleibt unverändert, bis das Journal einen erfolgreichen Abschluss sieht (§11.1).

## 6. `Cargo.toml`

```toml
[package]
name = "contextra-text"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Inverted fulltext index, BM25, and segment rewrite under ArtKey"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-durable-fs = { workspace = true }
contextra-ports = { workspace = true }
contextra-erasure = { workspace = true }
bytes = { workspace = true }
unicode-segmentation = "1.11.0"
regex = "1.10"
ahash = { workspace = true }
thiserror = { workspace = true }
parking_lot = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
proptest = "1.4"

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
```

## 7. Tests

| Kennung | Pflicht |
|---|---|
| T-DEL-6 | Volltextsuche findet das Dokument nach `Frozen` nicht, und nach Schritt 3 auch nicht in der Rohsicht des neuen Segments |
| T-DEL-10 | Segment mit `LineageRef` auf das Quelldatum wird in Schritt 2 aufgelöst und in Schritt 3 neu geschrieben |
| T-DEL-14 | Zweites `purge` desselben Segments ist ein No-op |
| T-DEL-17 | Kanonische Terme, die nur im Scope vorkamen, fehlen in der Segmentdatei |

Der Beispielcode eines Rewrites enthält kein `unwrap` und kein `expect` (INV-4).
