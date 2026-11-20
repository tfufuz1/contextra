# contextra-ids — Implementierungsspezifikation

**Ebene:** E0 (Fundament)
**Status:** Normativ, abgeleitet aus `docs/SPEC.md` v4.1
**Version:** 4.1
**Geltungsbereich:** `crates/contextra-ids`
**Normative Quellen:** SPEC 3.1, 3.2 (R3, R4, R10), 3.3, 5.9, 5.3, 7.2, 15, ADR-33, ADR-52, ADR-54, G-01, G-02, G-04, G-06, G-07, G-13, REM-30.

---

## 1. Zweck und Ebene

`contextra-ids` besitzt die Identitäts-, Ordinal- und Limit-Typen des Workspace. Das Crate hat keine internen Abhängigkeiten. Es enthält keine Logik über Persistenz, Verschlüsselung oder Löschung.

Die Quelle „ein Konzept, ein Besitzer“ (R4) gilt hier ohne Ausnahme: Jeder hier definierte Typ ist die einzige Definition im Workspace.

### 1.1 Ist-Befund (gemessen am HEAD `f8b44b2`)

| Messgröße | Wert | Befund |
|---|---|---|
| Quelldateien | 7 `.rs` (lib 982 Zeilen, Tests 199 + 70 + 78 + 109) | `src/lib.rs` ist ein Monolith mit 27 öffentlichen Typen |
| Feature-Schalter | `docid-128`, `std` | `docid-128` schaltet `DocId` zwischen `u64` und `u128` um. Verstößt gegen R3 und SPEC 5.9 |
| Abhängigkeiten | `blake3`, `serde`, `thiserror`, `rand`, `zeroize` | `blake3` und `serde` sind nach diesem PLAN überflüssig |
| Tests | 34 bestanden, 0 fehlgeschlagen | Grün, Compile ohne Warnungen auf 1.89.0 |
| `DocId` | Abgeleitet per `from_key` aus BLAKE3 (39 Bezeichner-Treffer `from_key` im Workspace) | Verstößt gegen SPEC 5.9 (zufällig, nie aus Inhalt) |
| `TenantId` | `u64`, Wert 0 reserviert | SPEC 5.3 und 5.9 verlangen 16 Byte Zufall |
| `CollectionId` | `u64` | SPEC 5.3 (AD) verlangt `u32 LE` |
| Löschtypen (`ErasureId`, `GateId`, `OperatorRef`, `PersonalRef`, `ScopeId`) | vorhanden | Entsprechen dem Altmodell. SPEC 7.2 nennt `TicketId`, `PrincipalRef`, `SubjectRef` |
| `TOMBSTONE_BIT` | `1 << 63` | 59 Bezeichner-Treffer im Workspace. Tombstones sind laut SPEC Provenienz und Bitmap, kein Bit in der Sequenz |
| `EntityId` | `u64`, `From<&str>` mit Hash-Fallback | 139 Treffer. SPEC ADR-23 definiert `EntityId` als BLAKE3-keyed 128 Bit im Graph |
| `TxId` | `u64` mit `INTERNAL_BASE` | 333 Treffer. SPEC ersetzt die Versionssequenz durch `u64` im Port `version` |
| `MAX_SEARCH_K` | `1_000` in diesem Crate (korrekt) | Im Workspace sechsmal definiert, drei davon mit `10_000` (`text`, `rank`, `vector`) |
| Fehlerart `IdError` | 7 Varianten, `#[non_exhaustive]` | Teilweise veraltet (`TenantSystemReserved`, `CollectionZero`) |

Ergebnis: Die öffentliche Oberfläche ist größer als der Soll-Bestand (SPEC 15) und enthält zwei Ableitungen aus Inhalt, die SPEC 5.9 verbietet.

---

## 2. Vertrag (öffentliche API)

Alle Typen sind `Copy`, außer die unten markierten. Alle Konstruktoren aus fertigen Bytes sind `const fn`.

```rust
// Limits (einzige Definition im Workspace, ADR-52)
pub const MAX_SEARCH_K: usize = 1_000;
pub const MAX_CHUNKS_PER_DOC: u32 = 65_536;
pub const MAX_BATCH_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_REF_LEN: usize = 256;

// Dokument-Identität. Immer 128 Bit, zufällig, nie aus Inhalt oder Pfad (SPEC 5.9)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct DocId([u8; 16]);
impl DocId {
    pub fn generate() -> Result<Self, IdError>;
    pub fn generate_from<R: rand::RngCore + rand::CryptoRng>(rng: &mut R) -> Result<Self, IdError>;
    pub const fn from_bytes(b: [u8; 16]) -> Self;
    pub const fn as_bytes(&self) -> &[u8; 16];
}

// Mandant (SPEC 5.9: 16 Byte Zufall). Kein Wert ist reserviert.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TenantId([u8; 16]);
impl TenantId {
    pub fn generate() -> Result<Self, IdError>;
    pub const fn from_bytes(b: [u8; 16]) -> Self;
    pub const fn as_bytes(&self) -> &[u8; 16];
}

// Collection (SPEC 5.3: u32 LE im AD). Wert 0 ist gültig.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CollectionId(pub u32);

// Ordinale. Dicht, je Mandant monoton, nie wiederverwendet (SPEC 5.9).
// Vergabe erfolgt im Schreib-Task (db), nicht hier.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SealOrd(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SegOrd(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ChunkSlot(pub u32);

impl SealOrd {
    /// Nächster Ordinal. `None`, wenn der Wert `u32::MAX` erreicht würde.
    /// Der Wert `u32::MAX` ist reserviert (SPEC 5.9, Abweisung mit `Refused`).
    pub const fn checked_next(self) -> Option<Self>;
}
impl SegOrd { pub const fn checked_next(self) -> Option<Self>; }

// Löschauftrag und Principal (SPEC 7.2)
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TicketId([u8; 16]);           // Zufall, wie DocId
impl TicketId { pub fn generate() -> Result<Self, IdError>; pub const fn from_bytes(b: [u8; 16]) -> Self; pub const fn as_bytes(&self) -> &[u8; 16]; }

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PrincipalRef(String);         // Name aus der Konfiguration, 1..=MAX_REF_LEN Byte, kein Leerzeichen am Rand
impl PrincipalRef { pub fn try_new(name: &str) -> Result<Self, IdError>; pub fn as_str(&self) -> &str; }

// Betroffenen-Kennung. Klartext nie im Produkt außer im Aufrufer (SPEC 5.7, 7.2)
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Zeroize, ZeroizeOnDrop)]
pub struct SubjectRef([u8; 16]);         // KEIN Clone, KEIN Copy, KEIN Debug des Rohwerts
impl SubjectRef {
    pub fn generate() -> Result<Self, IdError>;
    pub const fn from_bytes(b: [u8; 16]) -> Self;
    pub fn as_bytes(&self) -> &[u8; 16];
}

pub enum IdError { /* siehe 7 */ }
pub type Result<T> = core::result::Result<T, IdError>;
```

**Präzisierungen zu `SubjectRef`.** Der Typ hat kein `Clone` (entgegen der Zeile in der ersten Fassung dieses PLAN, die `Clone` aus Ergonomie zuließ). Der Aufrufer übergibt `SubjectRef` by value oder by reference. `Debug` und `Display` schreiben `SubjectRef([REDACTED])`. Der Rohwert verlässt das Crate nur über `as_bytes`, und dort darf es keine Ausgabe geben.

**Präzisierungen zu `TenantId`.** Der Zufall kommt aus `OsRng` (R10). Ein Mandant mit Wert 0 existiert nicht. Die Sonderwert-Logik (`SYSTEM`, `DEFAULT`, `INVALID`) entfällt.

---

## 3. Abhängigkeiten

| Crate | Zweck | Status |
|---|---|---|
| `rand` | `OsRng` für Zufalls-IDs | behalten, mit Feature `default-features = false` und `getrandom` |
| `zeroize` | `SubjectRef`-Vernichtung | behalten, Feature `zeroize_derive` |
| `thiserror` | `IdError` | behalten |
| `blake3` | bisher `from_key` | **entfernen** (Ableitungen aus Inhalt verboten, SPEC 5.9) |
| `serde` | bisher Serialisierung | **entfernen**. Store und Crypto kodieren die Felder explizit in fester Byte-Reihenfolge (SPEC 5.3, 5.5, 5.6) |

Interne Abhängigkeiten: keine (SPEC 3.3, `ids = []`).

Dev-Abhängigkeiten: `rand` (mit `std_rng`-Feature für `SeededRng`-artige Tests über `rand_chacha`), keine weiteren.

---

## 4. Module

```
src/
  lib.rs          # nur Modulerklärung, Re-Exports, #![forbid(unsafe_code)], #![deny(missing_docs)]
  limits.rs       # MAX_SEARCH_K, MAX_CHUNKS_PER_DOC, MAX_BATCH_BYTES, MAX_REF_LEN
  doc.rs          # DocId
  tenant.rs       # TenantId, CollectionId
  ordinals.rs     # SealOrd, SegOrd, ChunkSlot
  refs.rs         # TicketId, PrincipalRef, SubjectRef
  random.rs       # gemeinsamer Helfer: fill_random(rng, &mut [u8;16]) -> Result<(), IdError>
  error.rs        # IdError
tests/
  layout.rs       # Größe, Ausrichtung, Ordnung
  rng.rs          # OsRng-Fehlerpfad über generate_from
  redaction.rs    # Debug und Drop von SubjectRef
  limits.rs       # Konsistenz der Limits
  fixtures/       # Hex-Vektoren für from_bytes (keine Ableitung)
```

Entfallen: `src/tests.rs`, `tests/differential_ids.rs`, `tests/golden_tombstone_bit.rs`, `tests/ids_tests.rs` (wird durch `tests/layout.rs` ersetzt), `tests/legacy_reference/`, `tests/golden/` (JSON-Vektoren für `from_key` sind obsolet, die Serde-Vektoren entfallen mit `serde`), `INVENTAR.md`, `tests/golden/regen.sh`, `tests/golden/MANIFEST.sha256`.

Jede Datei in `src/` hat eine Zeile Modul-Dokumentation und keine Verweise auf andere Crates.

---

## 5. Entfernungen

Abnahme jeder Zeile: der angegebene Befund ist im Workspace nicht mehr vorhanden. Für Typen mit externen Verbrauchern ist die Ersetzung in Abschnitt 6 festgelegt.

| ID | Entfernen | Grund | Ersatz | Verbraucher (grep -w, Dateien) |
|---|---|---|---|---|
| E-IDS-01 | Feature `docid-128` und `ID_WIDTH_BITS` | R3, SPEC 5.9. Breite ist fest | `DocId([u8;16])` | in `DocId` gesamt 285 |
| E-IDS-02 | `DocId::from_key`, `hash_key_u64`, `hash_key_u128` | SPEC 5.9: keine Ableitung aus Inhalt | `DocId::generate` | 39 (`from_key`) |
| E-IDS-03 | `DocId::MIN`, `DocId::MAX`, `From<u64>` für `DocId` | Keine Sonderwerte, keine Ganzzahl-Konstruktion | `from_bytes` | prüfen bei Umbau |
| E-IDS-04 | `TOMBSTONE_BIT` | Tombstones sind Provenienz und Bitmap (SPEC 4.2, 6.5) | keine. Ersetzt durch `ChunkSlot`-Bitmaps | 59 |
| E-IDS-05 | `EntityId` (u64, `From<&str>`, `from_doc_id`) | ADR-23: Entitäts-ID gehört dem Graph (`contextra-graph`) und ist BLAKE3-keyed | `graph::EntityId` | 139 |
| E-IDS-06 | `TxId` (`INTERNAL_BASE`, `MAX_COLLECTION_SEQUENCE`, `is_valid_origin`, `try_from_internal_offset`) | Versionssequenz ist `u64` im Port `ports::version` (SPEC 4.4, REQ-VER-1) | `u64` (`Seq`) | 333 |
| E-IDS-07 | `Seq` mit `next()` | Sequenzen gehören `version`, Zeit gehört `env` | `u64` in `ports::version` | 11 |
| E-IDS-08 | `Timestamp` | Zeit ist `Clock` in `env` (R10) | `u64` Nanosekunden aus `Clock` | 12 |
| E-IDS-09 | `FileId`, `StoreId` | Altmodell (Datei-Identität). Dateien werden über `SegOrd` und Pfad im Mandantenverzeichnis geführt (SPEC 3.5) | `SegOrd`, Verzeichnispfad | 5 bzw. 3 |
| E-IDS-10 | `DocRef` (Struktur mit `tenant/collection/doc`) | SPEC kennt keinen zusammengesetzten Referenztyp | `(TenantId, CollectionId, DocId)` im Aufrufer | 3 |
| E-IDS-11 | `ErasureId` | Nach SPEC heißt der Auftrag `TicketId` (SPEC 7.2) | `TicketId` | 12 |
| E-IDS-12 | `GateId` | Gate ist mandantenweit und hat keine eigene Kennung (ADR-28) | keine | 4 |
| E-IDS-13 | `OperatorRef` | Entspricht `PrincipalRef` (ADR-38) | `PrincipalRef` | 10 |
| E-IDS-14 | `PersonalRef` | K-Dokumente enthalten keine Betroffenen-Referenz (SPEC 5.1, 5.7 Betroffenen-Index mit keyed Tag) | `subject_tag` in `crypto` | 2 |
| E-IDS-15 | `ScopeId` | Scope ist ein Enum in `erasure` (SPEC 7.2) | `erasure::Scope` | prüfen bei Umbau |
| E-IDS-16 | `TenantScoped<T>`, `TenantScopeViolation` | Mandantenbindung läuft über Schlüsselwahl und Schreib-Task (SPEC 5.4, 8.4). Ein Wrapper dupliziert das | keine | prüfen bei Umbau |
| E-IDS-17 | Byte-Identifier-Makro (`try_new(Vec<u8>)`) | nicht im SPEC-Bestand | `PrincipalRef::try_new` | prüfen bei Umbau |
| E-IDS-18 | Feature `std` | R3 erlaubt `std` nur in `durable-fs`, `simd`, `rank` | keine | 0 |
| E-IDS-19 | Abhängigkeiten `blake3`, `serde` | siehe Abschnitt 3 | keine | |
| E-IDS-20 | `IdError::TenantSystemReserved`, `IdError::CollectionZero` | Keine Sonderwerte mehr | keine | |
| E-IDS-21 | Tests und Artefakte aus Abschnitt 4 | siehe Abschnitt 4 | | |

**Limits.** Die bisherigen Konstanten `MAX_SEARCH_K = 1_000` und `MAX_REF_LEN = 256` bleiben. Die Konstanten `MAX_CHUNKS_PER_DOC` und `MAX_BATCH_BYTES` sind neu. SPEC 15 nennt sie, gibt aber keine Werte an. Dieser PLAN legt fest: `MAX_CHUNKS_PER_DOC = 65_536` (gleich der Segment-Chunkgröße in SPEC 5.8, damit ein Dokument höchstens einen Segment-Chunk belegt), `MAX_BATCH_BYTES = 4 MiB` (SPEC REQ-CON-1). Eine Änderung dieser Werte ist eine Änderung der Spezifikation.

---

## 6. Umbau (Reihenfolge, je Schritt ein Commit, Workspace grün)

Die Reihenfolge folgt der Abhängigkeitslogik: zuerst Typen ohne externe Verbraucher, dann Typen mit Verbrauchern, zuletzt der Feature-Umbau. Ein Zwischenzustand mit Kompatibilitäts-Alias ist nach ADR-51 verboten. Jede Ersetzung ist deshalb im selben Commit wie die Entfernung.

| Schritt | Inhalt | Verbraucher-Anpassung | Prüfung |
|---|---|---|---|
| U-IDS-1 | Löschen ungenutzter Typen: `FileId`, `StoreId`, `DocRef`, `GateId`, `ScopeId`, `TenantScoped`, Makro, `IdError`-Varianten ohne Nutzer | keine | `cargo check --workspace` |
| U-IDS-2 | `SubjectRef` ohne `Clone`, `Debug` redigiert, `Zeroize` | Aufrufer in `contextra` und `mcp` anpassen (15 Dateien, Ownership) | Test `redaction` |
| U-IDS-3 | Neuer Typ `TicketId` plus Ersetzung `ErasureId` → `TicketId` (12 Dateien) | mechanisch | `grep -rn ErasureId crates` leer |
| U-IDS-4 | `OperatorRef` → `PrincipalRef` (10 Dateien). `PrincipalRef::try_new` verlangt gültige Namen | Konfigurationsloader in `contextra` liefert die Namen | Test `principal_name` |
| U-IDS-5 | `Seq` und `Timestamp` entfernen. Ersatz: `u64` in `ports::version`, `u64` Nanosekunden aus `Clock` (env). Reihenfolge: zuerst `env` (Schritt E-ENV-3), dann dieser Schritt | 11 + 12 Dateien | `grep -rn "ids::Seq\|ids::Timestamp"` leer |
| U-IDS-6 | `TxId` entfernen. Ersatz `u64`. Größter Schritt | 333 Verweise, mechanische Ersetzung mit Compiler-Führung | `grep -rn "TxId" crates` leer |
| U-IDS-7 | `EntityId` entfernen. Ersatz `graph::EntityId` (kommt mit `contextra-graph`-PLAN) | 139 Verweise. Bis dahin bleibt der Graph in M4b ausgeschaltet, `db` verweist nicht auf `EntityId` | `grep -rn "ids::EntityId"` leer |
| U-IDS-8 | `TOMBSTONE_BIT` entfernen. Ersatz über `ChunkSlot`-Bitmap im Aufrufer | 59 Verweise. Entspricht dem Umbau der Versionierung | `grep -rn TOMBSTONE_BIT` leer |
| U-IDS-9 | `DocId` auf 128 Bit fest, `docid-128` entfernen, `from_key` entfernen (39 Verweise, Aufrufer bekommen `generate` oder die Dokument-ID aus dem Quellsystem über die Idempotenz-Tabelle, SPEC REQ-ING-8) | 285 Verweise. Größter Eingriff mit Persistenzwirkung, Testdaten werden verworfen (ADR-44) | `ID_WIDTH_BITS` nicht mehr definiert |
| U-IDS-10 | Tests bereinigen und neue Tests aus Abschnitt 7 einführen | | `cargo test -p contextra-ids` |
| U-IDS-11 | `blake3`, `serde` aus `Cargo.toml` entfernen | | `cargo tree -p contextra-ids` ohne beide |
| U-IDS-12 | `MAX_SEARCH_K` in den Crates `text`, `rank`, `vector`, `graph`, `types` durch `use contextra_ids::MAX_SEARCH_K` ersetzen. Eigene Definitionen werden entfernt | Konsolidierung (ADR-52) | G-04 |

**Hinweis zur Reihenfolge.** U-IDS-5 und U-IDS-6 setzen voraus, dass `ports::version` bereits existiert. Der Port wird im PLAN für `ports` angelegt. Bis dahin verbleibt `TxId` bis U-IDS-6 als alleiniger Vertreter der Version. Der Zwischenzustand ist kein Kompatibilitäts-Alias, sondern der alte Stand, bis der Schritt aufgeführt wird.

---

## 7. Tests

| ID | Prüfung | Ort |
|---|---|---|
| T-ID-01 | `size_of::<DocId>() == 16`, `align_of == 1`, für jede Build-Konfiguration | `tests/layout.rs` |
| T-ID-02 | `size_of::<TenantId>() == 16`, `size_of::<CollectionId>() == 4`, `size_of::<SealOrd>() == 4` | `tests/layout.rs` |
| T-ID-03 | `DocId`-`Ord` ist byteweise aufsteigend (Property-Test mit 10.000 Paaren) | `tests/layout.rs` |
| T-ID-04 | `DocId::generate_from` mit einem RNG, der `Err` liefert, liefert `IdError::EntropyExhaustion` und keine ID | `tests/rng.rs` |
| T-ID-05 | `SubjectRef::generate_from` mit gleichem Fehler-RNG liefert `EntropyExhaustion`. Die Fehlervariante trägt keine Rohbytes | `tests/rng.rs` |
| T-ID-06 | `Debug` und `Display` von `SubjectRef` enthalten keine der 16 Byte des Werts (Test über alle 16 Einzelbytes) | `tests/redaction.rs` |
| T-ID-07 | Nach Drop eines `SubjectRef` sind die 16 Byte des Speicherorts `0x00` (Test mit `MaybeUninit` und `ManuallyDrop`) | `tests/redaction.rs` |
| T-ID-08 | `SealOrd::checked_next(SealOrd(u32::MAX - 1)) == None`, `SealOrd(0).checked_next() == Some(SealOrd(1))` | `tests/limits.rs` |
| T-ID-09 | `PrincipalRef::try_new` lehnt leere Namen, Namen über `MAX_REF_LEN` und führende/folgende Leerzeichen ab | `tests/limits.rs` |
| T-ID-10 | Die Konstanten sind konsistent: `MAX_SEARCH_K == 1_000`, `MAX_CHUNKS_PER_DOC * 1 <= u32::MAX`, `MAX_BATCH_BYTES == 4 * 1024 * 1024` | `tests/limits.rs` |
| T-ID-11 | Fixture-Test: `from_bytes` und `as_bytes` sind verlustfrei für 64 Hex-Vektoren aus `tests/fixtures/` | `tests/layout.rs` |

Nicht testen: Verteilung der Zufallsbytes. Das ist die Aufgabe des Betriebssystems.

---

## 8. Abnahme

| Gate | Prüfung für `contextra-ids` |
|---|---|
| G-01 | `cargo check -p contextra-ids`, `cargo test -p contextra-ids`, `cargo test -p contextra-ids --no-default-features`, `cargo clippy -p contextra-ids -- -D warnings` grün (Toolchain 1.89.0) |
| G-02 | `#![forbid(unsafe_code)]` in `src/lib.rs` |
| G-04 | `grep -rn "const MAX_SEARCH_K" crates \| wc -l` ist `1`, Definition in `contextra-ids/src/limits.rs` |
| G-06 | `default = []` in `Cargo.toml` |
| G-07 | `grep -rn "docid-128\|from_key\|TOMBSTONE_BIT\|TxId\|EntityId\|ID_WIDTH_BITS" crates/contextra-ids` leer |
| G-13 | `grep -n "unwrap()\|expect(\|panic!(" crates/contextra-ids/src -r` leer außerhalb `#[cfg(test)]` |
| G-03 | `layering.rs` bestätigt: `ids` hat keine Abhängigkeit auf ein Workspace-Crate |
| Dokumentation | `PLAN.md` referenziert nur Kapitel und IDs aus SPEC v4.1 |

**Reihenfolge im Repository:** Dieses PLAN wird als erster Commit der Fundamentreihe eingecheckt. Die Umsetzung U-IDS-1 bis U-IDS-12 folgt, wenn `contextra-env` (E-ENV-3) bereit ist, weil U-IDS-5 davon abhängt.
