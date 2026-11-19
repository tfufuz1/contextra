# contextra-db — Crate-Spezifikation (Soll-Zustand, SPEC 2.1)

**Ebene:** E2 (Integration)
**Kern-Closure:** `ids`, `env`, `types`, `ports`, `privacy`, `erasure`, `crypto`, `store`, `vector`, `text`, `graph`, `rank`, `mvcc`, `checkpoint`, `durable-fs`
**Status:** Normativ (Soll-Zustand)
**Version:** 2.1
**Geltungsbereich:** `crates/contextra-db`
**Normative Quellen:** `docs/SPEC.md` §2.4, §2.5, §5, §6, §7, §8, §9, §10, §11, §15, INV-WIRING-2, INV-GATE-COMPLETENESS, INV-DERIVED-1.

---

## 1. Zweck

`contextra-db` verdrahtet die Domänenbausteine und ist der einzige Besitzer der Lineage (SPEC §2.5): `LineageRef`, `DerivedRef`, `LineageIndex`.

Der Saga-Koordinator bleibt `Chain` in `contextra-erasure`. Dieses Crate startet ihn, registriert die Schichten und hält den Key-Lock. Es enthält keine zweite Zustandsmaschine.

`LsmStorage` bleibt `pub(crate)` in `contextra-store`. Jeder Lesezugriff der Engine läuft über `TenantScopedStorage`. Dort sitzt die einzige Gate-Prüfstelle (SPEC §6.1).

`contextra-engine` re-exportiert dieses Crate und ist keine Schicht. S4-Caches, die dieses Crate selbst hält, registriert es als `Purgeable`. S5 ist der `LineageIndex` hier.

## 2. Architekturregeln

1. `#![forbid(unsafe_code)]`. Zero-Panic. `default = []`.
2. Keine Dev-Abhängigkeit auf `contextra-testkit` (R2).
3. Ein `Ok(())` ohne die beschriebene Wirkung ist ein Stub und ein Build-Fehler (INV-WIRING-2). Das gilt für Schreibpfade, `purge`, `witness` und leere `DataClass`-Zweige.
4. Kein Klartext des Löschziels in Logs (INV-12).
5. `recover` der `Chain` läuft zu Ende, bevor der erste Lesepfad öffnet (SPEC §5.5).

## 3. `TenantScopedStorage`

`get`, `get_at_seq`, `scan`, `scan_prefix`, Vektorsuche, Textsuche, Hybrid-Suche, Graph-Traversal, Export und Kompaktionsfilter rufen `ReadGate::check` oder `check_range` auf, bevor Chiffretext geöffnet oder ein Treffer ausgegeben wird.

| Pfad | Bei `Erased` |
|---|---|
| `get`, `get_at_seq` | `Err(ContextraError::Erased)`, bevor der `DocDek` geöffnet wird |
| `scan`, `scan_prefix` | Eintrag auslassen |
| Vektor, Text, Hybrid | Kandidat verwerfen, danach auf `k` auffüllen |
| Graph | Kante und Zielknoten, Pfad endet |
| Export | Eintrag auslassen |
| Kompaktion | `ContainsErased`: Range nicht ins Ergebnis. `RestrictedNeverDiscard`: Range behalten |

`Restricted` hat Vorrang. Ein Lesezugriff liefert nur mit geschriebenem Audit-Eintrag Daten, sonst `ContextraError::RestrictedReadAuditRequired`. Nach außen bleibt das Bild `not found` (siehe `contextra-types`).

`as_of` und offene Snapshots prüfen den aktuellen Gate-Stand (SPEC §9.2). Ein Pin in `contextra-mvcc` hält den Löschbeginn nicht auf.

## 4. Lineage (SPEC §8)

```rust
pub struct LineageRef {
    pub sources: NonEmpty<SourceRef>,
}

pub struct SourceRef {
    pub source_class: DataClass,
    pub source_id: SourceId,
    pub source_version: u64,
}

pub enum SourceId {
    Document(DocId),
    Subject(SubjectRef),
}

pub struct DerivedRef {
    pub layer: LayerId,
    pub segment_id: u64,
    pub offset: u64,
}
```

`NonEmpty<SourceRef>` ist bei Konstruktion kanonisch nach `(source_class, source_id, source_version)` sortiert und dedupliziert. `LineageIndex::derived_of(source, version)` liefert jede `DerivedRef`; bei mehreren Quellen wird die Ableitung unter jeder Quellreferenz eingetragen. Ein D-Schreibvorgang ohne `LineageRef` oder mit leerer, unsortierter oder duplizierter Quellenliste endet mit `ContextraError::ClassViolation` (INV-LINEAGE).

Schritt 2 liest den Index und übergibt die betroffenen Segmente an `TextLayer`, `VectorLayer` und `GraphLayer`. Ableitungen mit weiteren lebenden Quellen werden als `stale` markiert oder neu aufgebaut. Dieses Crate trifft die Markierung und gibt sie an die Schicht. Mixed-Segmente folgen ausschließlich `ErasureConfig::mixed_policy == BlockAll` und verbleiben in `Partial{ReviewRequired}`.

## 5. Verdrahtung der Saga

Beim Öffnen registriert `contextra-db` genau diese Produktiv-Implementierungen. Eine davon durch einen Erfolg meldenden Stub zu ersetzen, ist verboten.

| Schicht | Implementierung | Registriert als |
|---|---|---|
| S1, S1b | `StoreLayer` | `ErasureLayer` |
| S2 | `VectorLayer` | `ErasureLayer` |
| S3 | `TextLayer` | `ErasureLayer` |
| S4 | Block-Cache, `er_cache`, `SurrogateVault`, plus Embedding- und Antwort-Cache, sobald sie existieren | `Purgeable`, gebündelt als Cache-Schicht |
| S5 | `LineageIndex` | `ErasureLayer` (`LineageLayer`) |
| S6 | `CheckpointLayer` | `ErasureLayer` |
| S7 | `CryptoLayer` | `ErasureLayer` |
| Graph-Snapshots | `GraphLayer` | Dateiklasse aus SPEC §4.7, gemessen mit dem Generationswechsel |
| Zeitreise | `SnapshotRevoker` | `ErasureLayer`, Schritt 1 |

Der Subjekt-Index bildet `SubjectRef` auf die `DocDek`-Einheiten ab. Fehlt er, liefert der Start einer Subjekt-Löschung `ErasureError::UnsupportedSubject` und setzt keine Lesesperre (Fail-Closed).

Property-Graph-Dateien stehen im Format-Register unter dem Graph-Snapshot aus §4.7. Eine Dateiklasse ohne Zeile bricht `surface-coverage`.

## 6. Key-Lock und Schreibpfad

`remember` erzeugt den `DocDek` in `contextra-crypto` und legt die `DocId` als 128-Bit-Wert an. Die Kollisionsprüfung lehnt die zweite Belegung derselben `DocId` ab.

Der Key-Lock einer Einheit wird in Schritt 2 genommen und bis einschließlich Schritt 5 gehalten (SPEC §4.5). `remember` und Update auf einer gesperrten Einheit liefern `ContextraError::Erasure`.

`insert` prüft die Klasse, bevor es schreibt:

- `K`: der Payload ist `Validated<_>`. Sonst `ClassViolation`.
- `P`: der Aufruf trägt ein `SubjectRef`. Der `DocDek` gehört zu dieser Einheit.
- `D`: eine `LineageRef` ist gesetzt. Sonst `ClassViolation`.
- `E`: nur der Koordinator schreibt Löschmaterial.

Danach nimmt der Schreiber `CommitBarrier::shared`, lässt `contextra-crypto` versiegeln und schreibt Chiffretext in Store, Vector, Text und Graph. Ein Zweig, der den Payload nicht übergibt und trotzdem `Ok(())` liefert, ist verboten.

`EncryptionConfig::None` versiegelt nicht. Die Saga kann dann höchstens Klasse L erreichen. Das Öffnen mit `None` ist erlaubt und wird im Bericht als Grenze sichtbar. Ein stiller Klartext-Fallback bei `Required` ist verboten.

## 7. Takt 1 und Recovery

`erase_document`, `erase_subject`, `drop_collection` und `drop_tenant` rufen `Chain::erase` auf und geben das `ErasureTicket` zurück, sobald Takt 1 durabel ist. Sie warten nicht auf Klasse C oder M. Ein fehlgeschlagener Journal-Flush liefert `ErasureError::JournalNotDurable` und setzt keine Lesesperre.

`open` ruft `Chain::recover` auf, bevor `TenantScopedStorage` Anfragen annimmt.

## 8. Hybrid-Suche

Die Suche holt Kandidaten aus Vector, Text und Graph, verwirft gesperrte Kandidaten und füllt auf `k` auf. `contextra-rank` sieht nur zugelassene Kandidaten. Der Tie-Break ist `DocId` aufsteigend. `er_cache` wird nur mit diesen Kandidaten gefüllt.

## 9. `Cargo.toml`

```toml
[package]
name = "contextra-db"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Hybrid-search orchestration, lineage, and erasure wiring"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-types = { workspace = true }
contextra-ports = { workspace = true }
contextra-privacy = { workspace = true }
contextra-erasure = { workspace = true }
contextra-crypto = { workspace = true }
contextra-store = { workspace = true }
contextra-vector = { workspace = true }
contextra-text = { workspace = true }
contextra-graph = { workspace = true }
contextra-rank = { workspace = true }
contextra-mvcc = { workspace = true }
contextra-checkpoint = { workspace = true }
contextra-durable-fs = { workspace = true }
tokio = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
proptest = "1.4"

[features]
default = []
```

`contextra-testkit` steht weder unter `[dependencies]` noch unter `[dev-dependencies]`. `contextra-vault` und die entfernten Crates aus SPEC §2.2 stehen hier nicht.

## 10. Tests

| Kennung | Pflicht |
|---|---|
| T-DEL-4 | Nach Neustart bleibt der Scope `Erased`, weil `recover` vor dem Lesepfad lief |
| T-DEL-9 | Subjekt-Löschung vernichtet jeden `DocDek` des Index. Ohne Index: `UnsupportedSubject` |
| T-DEL-10 | D-Ableitungen der Quelle sind nach Schritt 2 erfasst und nach der Hygiene gemessen |
| T-DEL-11 | Jeder Pfad aus §6.3 prüft das Gate |
| T-DEL-12 | Crash in Schritt 3: Lesepfad bleibt zu, bis `recover` die Sperre gesetzt hat |
| T-DEL-20 | `BlockAll` lässt das Mixed-Segment gesperrt |
| INV-WIRING-2 | Die registrierten Schichten sind die Produktivtypen, kein Typ mit leerem `purge` |
