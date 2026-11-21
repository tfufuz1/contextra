# contextra-ids — Implementierungsspezifikation

**Ebene:** E0 (Fundament)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch)
**Stand:** 21.11.2026, Abgleich mit SPEC v4.1

Verweist nur auf Kapitel und IDs der SPEC. Abweichungen des Ist-Zustands sind als **Ist-Abweichung** markiert, ungeklärte Punkte stehen unter **Offene Punkte (Entscheidung nötig)** in Abschnitt 6.

---

## 1. Zweck und Ebene

`contextra-ids` ist die kanonische Definition der Identitäts- und Mandantentypen und die **einzige Definition von Limits** (R4, ADR-33, ADR-52). Es ist die Aufnahmestelle der IDs aus dem aufzulösenden Crate `contextra-types` (REM-30).

Ebene E0, keine internen Abhängigkeiten (SPEC 3.1, 3.3). Ziel-Umfang nach SPEC 3.1: 2.000 Zeilen. Ist: 1.181 Zeilen `src` (Messung laut `CLAUDE.md`, 21.11.2026).

## 2. Vertrag

Soll nach SPEC 15: Typen `DocId`, `TenantId`, `CollectionId`, `SealOrd`, `SegOrd`, `ChunkSlot`, `TicketId`, `SubjectRef`, `PrincipalRef`. Limits `MAX_SEARCH_K`, `MAX_CHUNKS_PER_DOC`, `MAX_BATCH_BYTES`.

| Soll | Ist in `src/lib.rs` | Status |
|---|---|---|
| `DocId` | `pub struct DocId(pub u64)` bzw. `(pub u128)` per Feature `docid-128` | vorhanden, zweifach definiert (Feature) |
| `TenantId`, `CollectionId` | vorhanden | ok |
| `SealOrd`, `SegOrd`, `ChunkSlot` | nicht vorhanden | fehlt (Ordinale nach ADR-33 und 5.9 prüfen) |
| `TicketId` | nicht vorhanden | fehlt |
| `SubjectRef` | vorhanden | ok |
| `PrincipalRef` | nicht vorhanden; Ist `PersonalRef` | Ist-Abweichung: Name |
| `MAX_SEARCH_K = 1_000` | vorhanden (`pub const MAX_SEARCH_K: usize = 1_000`) | ok, einzige Definition (G-04) |
| `MAX_CHUNKS_PER_DOC`, `MAX_BATCH_BYTES` | nicht vorhanden | fehlt |

Weiterer Ist-Bestand ohne SPEC-Eintrag: `TOMBSTONE_BIT`, `MAX_REF_LEN`, `IdError`, `Result`, `EntityId`, `TxId`, `FileId`, `StoreId`, `Seq`, `Timestamp`, `ErasureId`, `GateId`, `DocRef`, `TenantScopeViolation`, `TenantScoped<T>`. Verbleib ist zu entscheiden (siehe 6).

Features: Soll keine (SPEC 15). Ist `std`, `docid-128`. Ist-Abweichung, siehe 3.

## 3. Abhängigkeiten

**Erlaubt:** keine internen (Anhang E: `contextra-ids = []`).

**Cargo-Auszug (Ist):**
```toml
[dependencies]
blake3 = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
rand = { workspace = true }
zeroize = { version = "1.8", features = ["zeroize_derive"] }
```

Externe Abhängigkeiten sind in SPEC 15 nicht geregelt. `rand` und `zeroize` sind dort nicht genannt (offen, siehe 6).

**Features:** `default = []` eingehalten (R3). Ist-Abweichung: `std = ["blake3/std", "serde/std"]` und `docid-128 = []` stehen im Soll nicht (SPEC 15: „Features: keine“). Entscheidung nötig, ob `docid-128` als Feature bleibt. SPEC 3.1 nennt keinen Grund dafür.

**Dev-Abhängigkeiten:** `bincode`, `serde_json`, `hex`.

## 4. Module

| Datei | Verantwortung (Ist) | Befund |
|---|---|---|
| `src/lib.rs` (982 Zeilen) | Alle Typen, Limits und Fehlerart `IdError` | Ist-Abweichung: eine Datei, Gliederung nach Soll offen |
| `src/tests.rs` | Unit-Tests T-ID1 bis T-ID6 | Testmodul im Produktcode-Verzeichnis, Verbleib zu klären |
| `tests/differential_ids.rs` | Differenztest gegen eingefrorene Altimplementierung | offen (Tests, siehe 7) |
| `tests/golden_tombstone_bit.rs` | Golden-Test für `TOMBSTONE_BIT` (Formatgarantie) | ok |
| `tests/ids_tests.rs` | Konformitäts- und Property-Tests | ok |
| `tests/legacy_reference/` | Eingefrorene Referenzen (`types_legacy.rs`, `vector_legacy.rs`) | Ist-Abweichung: Altbestand, Verbleib offen |
| `tests/golden/regen.sh` | Regenerierung der Golden-Daten | ok |

Zielgliederung: Typen nach Kapitel 15 in Module zerlegen, bis `lib.rs` nur Re-Exporte enthält. Konkrete Dateinamen erst nach Entscheidung in 6.

## 5. Entfernungen

| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| REM-30 (Teil „IDs nach `ids`“) | Typen aus `contextra-types` übernehmen (ADR-33) | Gegenprobe: `grep -rn "contextra_types\|contextra-types" crates/contextra-ids/src` leer |
| REM-14 | Selbstdeklarationen (ANCHOR, FILE-CONTEXT, contract.rs) | `grep -rl "ANCHOR\[\|FILE-CONTEXT" crates/contextra-ids` leer |

Hinweis: In `tests/golden_tombstone_bit.rs` und `tests/golden/regen.sh` steht der Name `contextra-types` in Kommentaren oder Testtexten. Ob das als Verweis gilt, ist offen (G-07).

## 6. Umbau

Schritte in Reihenfolge, je Schritt ein Commit, Workspace grün.

1. Fehlende Soll-Typen anlegen: `SealOrd`, `SegOrd`, `ChunkSlot`, `TicketId`, `PrincipalRef`. Vorher Umfang prüfen, siehe Offene Punkte.
2. Limits `MAX_CHUNKS_PER_DOC` und `MAX_BATCH_BYTES` ergänzen (SPEC 15).
3. Typen aus `contextra-types` aufnehmen, die laut ADR-33 zu `ids` gehören.
4. `lib.rs` in Module zerlegen (Abschnitt 4).

**Offene Punkte (Entscheidung nötig):**
- **O-1 `PersonalRef` gegen `PrincipalRef`.** Umbenennen oder zwei Typen? SPEC 15 nennt `PrincipalRef`.
- **O-2 `DocId`-Doppeldefinition.** Feature `docid-128` beibehalten (Soll: keine Features)? Wenn ja, in SPEC 15 aufnehmen.
- **O-3 Verbleib der übrigen Typen** (`EntityId`, `FileId`, `StoreId`, `GateId`, `ErasureId`, `Timestamp`, `Seq`, `DocRef`, `TenantScoped`, `TOMBSTONE_BIT`, `MAX_REF_LEN`). Diese sind in SPEC 15 nicht genannt. Entscheidung, ob sie zu `ids`, zu ihrem Besitzer oder nach `ports::dto` gehören (ADR-33).
- **O-4 Externe Abhängigkeiten** `rand`, `zeroize`. In SPEC 15 nicht geregelt.
- **O-5 Ordinale und Limits** (`SealOrd`, `SegOrd`, `ChunkSlot`, `MAX_CHUNKS_PER_DOC`, `MAX_BATCH_BYTES`): Die Formate stehen in SPEC 5.8 und 5.9. Festlegung der Typbreiten nötig.

## 7. Tests

- Serialisierung, Ordinal-Monotonie, Limit-Konsistenz (SPEC 15, G-04).
- Vorhanden: `tests/golden_tombstone_bit.rs`, `tests/ids_tests.rs`, `tests/differential_ids.rs`, `src/tests.rs` (T-ID1 bis T-ID6).
- Ist-Befund: 35 `#[test]` (laut `CLAUDE.md`).
- Zu entscheiden: Verbleib von `tests/legacy_reference/` (Differenztest gegen eingefrorene Implementierung, O-3).

## 8. Abnahme

| Gate | Ist-Befund | Soll |
|---|---|---|
| G-01 | Build laut `CLAUDE.md` ok | `cargo check`, `test`, `clippy -D warnings` grün |
| G-02 | `#![forbid(unsafe_code)]` in `lib.rs`, kein `unsafe` | eingehalten |
| G-03 | keine internen Abhängigkeiten | eingehalten |
| G-04 | genau eine Definition `MAX_SEARCH_K` | eingehalten |
| G-06 | `default = []` | eingehalten |
| G-07 | Erwähnungen von `contextra-types` in Tests und Kommentaren (siehe 5) | offen |
| G-09 | 0 ANCHOR-Dateien, keine `contract.rs` | eingehalten |
| G-11 | keine `compat`-Verzeichnisse oder Features | eingehalten |
| G-12 | nicht betroffen (kein `Instant::now`/`SystemTime::now` in `src`) | eingehalten |
| G-13 | 0 Treffer `unwrap`/`expect`/`panic!` | eingehalten |
