# contextra-graph — Crate-Spezifikation (Soll-Zustand, SPEC 2.1)

**Ebene:** E1 (Domäne)
**Domänen-Closure:** `ids`, `env`, `durable-fs`
**Verträge:** `ports` (`GraphIndex`), `erasure` (`GraphLayer`). `GraphIndex` wird hier implementiert und nicht neu definiert.
**Status:** Normativ (Soll-Zustand)
**Version:** 2.1
**Geltungsbereich:** `crates/contextra-graph`
**Normative Quellen:** `docs/SPEC.md` §3.2, §4.7, §5.3 Schritt 8, §5.10, §6.3, §7, INV-5, INV-ARTKEY-1, INV-12.

---

## 1. Zweck

`contextra-graph` ist der Property-Graph für GraphRAG: Knoten, typisierte Kanten, Hyperedges, Traversierung.

Personenbezogene Knoten werden über `SubjectRef` adressiert. Kanten aus Klasse-K-Knoten auf Personen verwenden `PersonalRef`. Klartext-PII liegt nicht im Graphen.

## 2. Einordnung in die Löschkette

SPEC §7 führt keine eigene Graph-Schicht neben `VectorLayer`. Die HNSW-Topologie gehört zu Schicht S2 in `contextra-vector` und trägt das Residuum `R-GRAPH`.

Dieses Crate besitzt die **Property-Graph-Snapshots**. SPEC §4.7 behandelt Graph-Snapshots als Index-Artefakt unter einem `ArtKey` je Generation: Neuaufbau, danach Vernichtung des `ArtKey`. Dieselbe Regel gilt für die CSR-Dateien. Sie stehen im Format-Register. Eine Dateiklasse ohne Zuordnung bricht das Gate `surface-coverage` (REQ-DEL-N24). Bis SPEC §5.9 eine eigene Zeile nennt, registriert `contextra-db` diese Dateien unter dem Graph-Snapshot aus §4.7 und misst sie mit dem Generationswechsel in Schritt 8.

`GraphLayer` implementiert dafür `ErasureLayer` aus `contextra-erasure`. Der `LayerId` bezeichnet die Property-Graph-Generation. Er ist kein zweites S2 und ersetzt `VectorLayer` nicht.

## 3. Architekturregeln

1. `#![forbid(unsafe_code)]`. Zero-Panic. `default = []`.
2. Keine Abhängigkeit auf andere E1-Crates. Keine Dev-Abhängigkeit auf `contextra-testkit`.
3. Traversierung ist deterministisch bei gleichem Stand und gleichem Seed (INV-5). Tie-Break nach `DocId` aufsteigend.
4. Kein Klartext des Löschziels in Logs (INV-12).

## 4. Rolle in der Löschsaga

### 4.1 Lesepfad (jeder Zeitpunkt ab `Frozen`)

Jede Kante und jeder Zielknoten wird gegen das aktuelle `ReadGate` geprüft (§6.3). `Erased` bricht den Pfad ab. `Restricted` hat Vorrang und braucht einen Audit-Eintrag. Die Prüfung gilt für `as_of` und für offene Snapshots gegen den aktuellen Stand (§9.2).

### 4.2 Schritt 2

`resolve` liefert Knoten, Kanten und Hyperedges, deren `LineageRef` auf den Scope zeigt. Die Kaskadenentscheidung trifft der Koordinator in `contextra-db`.

### 4.3 Schritt 8 — Generation

1. Ein Hintergrundlauf baut Generation `g+1` nur aus Knoten und Kanten, die das `ReadGate` als `Open` sieht.
2. Hyperedges, die nach dem Entfernen unter ihre Mindestgröße fallen, werden nicht repariert und nicht geraten. Sie entfallen in `g+1`.
3. Atomarer Commit der CSR-Dateien über `contextra-durable-fs`.
4. Der `ArtKey` der Generation `g` wird sofort vernichtet (INV-ARTKEY-1).
5. Mehrere Löschungen teilen sich einen Neuaufbau. Auslöser wie bei S2: Tombstone-Anteil ≥ 0,2, Frist, `seal-now`.

Bis zum Umschalten bleibt die Topologie ein ausgewiesenes Residuum im Bericht, analog zu `R-GRAPH`, mit Frist. Das Crate vergibt die Klasse M nicht selbst.

`purge` ist idempotent (T-DEL-14). Ein zweiter Lauf auf einer bereits aktuellen Generation ist ein No-op.

## 5. Fehler (`GraphError`)

`#[non_exhaustive]`: `EntityNotFound`, `HyperedgeUnderflow`, `CorruptedStorage`, `Timeout`, `Io`, `GenerationSwitchFailed`. `EntityNotFound` ist das Bild eines Knotens, den das Gate als `Erased` meldet, sobald der Aufrufer das auf den öffentlichen Pfad mappt. Intern bleibt `GateDecision` erhalten, damit der Koordinator messen kann.

## 6. `Cargo.toml`

```toml
[package]
name = "contextra-graph"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Property graph, traversal, and generation rebuild under ArtKey"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-durable-fs = { workspace = true }
contextra-ports = { workspace = true }
contextra-erasure = { workspace = true }
bytes = { workspace = true }
parking_lot = { workspace = true }
ahash = { workspace = true }
thiserror = { workspace = true }

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
| T-DEL-6 | Traversierung erreicht das gelöschte Dokument nicht |
| T-DEL-10 | Knoten mit `LineageRef` werden in Schritt 2 aufgelöst und fehlen in Generation `g+1` |
| T-DEL-11 | Jede Kante und jeder Zielknoten prüft das Gate |
| T-DEL-14 | Zweites `purge` ist ein No-op |
