# Implementierungsspezifikation `contextra-types` (Löschkette)

*Ring 0 · Konzept-WP 2 · Abhängig von: WP-1 · Blockiert: 02, 03, 06–10 · Index: `00-INDEX-QUERSCHNITT.md`*

## 1. Zweck und Abgrenzung

`contextra-types` hält alle Datentypen, die von mehr als einem Crate der Löschkette gebraucht werden: Auftragsidentität, Ziel, Garantiemodell, Zustandsautomat, Fehler. Es enthält **keine** Logik mit I/O, keine Zeit, keinen Zufall.

**Nicht hier:** Traits (→ `contextra-ports`), `ErasureConfig` des Workers (→ `contextra-engine`), `StoreErasureConfig` (→ `contextra-store`), Gate-Datenstrukturen (→ `contextra-store`).

## 2. Ist-Zustand (✔ gelesen)

| Aspekt | Befund |
| --- | --- |
| Abhängigkeiten | `thiserror`, `serde`, `serde_json`, `bincode`, `ahash`, `blake3`, `sha2`, `bytes` (`Cargo.toml:20-28`). `blake3` ist verfügbar, es braucht **keine** neue Abhängigkeit |
| Fehler | `ContextraError` ist `#[non_exhaustive]`; neue Varianten **nur am Ende** (`error.rs:118-124`). Es gibt `error_dto.rs` (DTO-Abbildung, Golden-Tests **[?]**) |
| IDs (Domäne) | `TenantId(pub u64)` (`types/domain/ids.rs:30`), `CollectionId(pub u64)` (`:113`), `DocId(pub u64)` bzw. `u128` mit Feature `docid-128` (`:171,178`) |
| IDs (Löschkette, **bereits vorhanden**) | `contextra-types/src/ids.rs`: `ErasureId(pub [u8; 16])` (`:73`), `FileId([u8;16])` (`:99`), `StoreId` (`:125`), `GateId` (`:151`), `Seq(pub u64)` (`:177`), `Timestamp(pub u64)` (`:193`); Re-Export in `lib.rs:62` ✔ |
| Referenztypen (Entwurf v1) | `contextra-types/src/refs.rs`: `TenantRef`, `CollectionRef`, `SubjectRef`, `OperatorRef`, `DocKey` (Byte-Newtypes, max. 256 Byte), `DocRef` (`ids.rs:64`). Nutzer: `contextra-text` (`lib.rs`, `memory_index.rs`, `contract.rs`) ✔. Stufe 1 verwendet sie **nicht** (Subjekt-Erase = ADR-E) |
| Duplikate in `ports` | `ports::gate::Seq` und `ports::ledger::Timestamp` sind **eigene Kopien** der Typen aus `types`; `ports::observer::Timestamp = u64` ist ein Alias; `ports::observer::TargetKind` (6 Varianten) liegt in den Ports ✔ |
| Collection-Name | Kein Newtype; die Engine arbeitet mit `String` und `validate_identifier` (`checkpoint/src/store.rs`: `validate_identifier("Checkpoint name", name)`) |
| `RequestContext` | `{ tenant_id, tx_id }` (`witness.rs:49-54`) |

## 3. Dateien

| Datei | Aktion |
| --- | --- |
| `contextra-types/src/erasure.rs` | **NEU** (alle Typen aus §4, inklusive verschobenem `TargetKind`) |
| `contextra-types/src/ids.rs` | `ErasureId::{from_u128, as_u128}`, `Display` (§4.1) |
| `contextra-types/src/lib.rs` | `pub mod erasure;` und Re-Exports (`ErasureId`, `EraseTarget`, `Guarantee`, `ErasureState`, `ErasureStatus`, `ErasureError`, `LayerErrorKind`, …) |
| `contextra-types/src/error.rs` | Variante `Erased` **am Ende** anhängen (§6) |
| `contextra-types/src/error_dto.rs` | Abbildung für `Erased` (§6) |
| `contextra-types/tests/erasure_types.rs` | **NEU** (§8) |
| `contextra-types/benches/types_bench.rs` | optional: `canonical_bytes` |

## 4. Typen (normativ)

Alle öffentlichen Typen: `#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]`, soweit nicht anders vermerkt. Enums, die wachsen können: `#[non_exhaustive]`. Manuelles `Debug` dort, wo Inhalt (Schlüssel, Namen) auftreten kann (`[REDACTED]`, wie `KeyBuf`/`EraseTarget` im Bestand ✔).

### 4.1 Identität und Konsolidierung vorhandener Typen

`ErasureId` **existiert** (`ids.rs:73`, `[u8; 16]`, `Debug` zeigt nur 4 Byte). Es wird **nicht** neu definiert. Änderungen:

```rust
impl ErasureId {
    pub const fn from_u128(v: u128) -> Self { Self(v.to_be_bytes()) }
    pub const fn as_u128(&self) -> u128 { u128::from_be_bytes(self.0) }
}
impl core::fmt::Display for ErasureId { /* 32 Hex-Zeichen, Kleinbuchstaben */ }
```

`ErasureId` entsteht **nicht** in `types`. Der Treiber füllt 16 Byte über `Rng::fill_bytes` (K-12: `IdGen::next_id` liefert nur `u64`).

**Konsolidierung (verbindlich, Teil von WP-2):**

| Vorhanden | Aktion |
| --- | --- |
| `ports::gate::Seq`, `ports::ledger::Timestamp`, `ports::observer::Timestamp` | Löschen. `ports` re-exportiert `contextra_types::{Seq, Timestamp}` (`pub use`). Prüfauftrag: `rg "gate::Seq|ledger::Timestamp|observer::Timestamp" crates` |
| `ports::observer::TargetKind` | Nach `contextra-types/src/erasure.rs` verschieben (Variantennamen und `Display` unverändert: `Document`, `Subject`, `Collection`, `Tenant`, `DocumentReplace`, `Superseded`), `ports` re-exportiert. **Kein** zweites `TargetKind` in `types` |
| `types::FileId([u8;16])` vs. `ports::keys::FileId(u64)` | Letztere entfällt mit `keys.rs` (02 §3). Keine Umbenennung nötig |
| `GateId`, `StoreId` | Unverändert, Stufe 1 ohne Nutzung (Platzhalter bleiben stehen, weil `deny(dead_code)` nur den Store betrifft; **[?]** `rg "GateId|StoreId" crates` zeigt Nutzer außerhalb `types`/`ports`? Erwartung: keine → in WP-0 entscheiden: löschen oder behalten) |

### 4.2 Ziel

```rust
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EraseTarget {
    Document   { tenant: TenantId, collection: String, key: String },
    Collection { tenant: TenantId, collection: String },
    Tenant     { tenant: TenantId },
    // Subject { .. } erst mit ADR-E
}

impl EraseTarget {
    pub fn kind(&self) -> TargetKind;   // Document | Collection | Tenant (Variante aus §4.1)
    pub fn tenant(&self) -> TenantId;
    /// Syntaxprüfung ohne I/O.
    pub fn validate(&self) -> Result<(), ErasureError>;
    /// Kanonische, eindeutige Serialisierung (siehe unten).
    pub fn canonical_bytes(&self) -> Vec<u8>;
    /// BLAKE3 (keyed) über `canonical_bytes`; `key` ist der Log-Schlüssel (32 Byte).
    pub fn target_hash(&self, key: &[u8; 32]) -> [u8; 32];
}
```

**`validate`-Regeln** (jede Verletzung → `ErasureError::InvalidTarget(reason: &'static str)`):

1. `collection` und Dokument-`key` nicht leer; `collection`: Zeichenmenge und Länge wie `validate_identifier` der Engine (Regel dort übernehmen, nicht duplizieren: die Prüffunktion wandert nach `types` oder die Engine ruft sie zusätzlich auf; Entscheidung: Engine ruft sie auf, `types` prüft nur nicht-leer, kein NUL-Byte, UTF-8-Länge ≤ 255 für `collection` und ≤ 4096 für `key`).
2. `EraseTarget::Collection` mit `collection == "default"` → `InvalidTarget("default collection cannot be erased as collection")` (K-5).
3. `collection` darf nicht mit `__` beginnen (reservierte Namensräume).

**`canonical_bytes`:** `[tag: u8][tenant: u64 LE][…]` mit `tag` = 1 (Document), 2 (Collection), 3 (Tenant). Strings als `[len: u32 LE][utf8 bytes]`. Injektiv (Test `TST-TYPES-02`). Format ist Teil des Journal-/Ledger-Vertrags und wird **eingefroren** (Golden-Test).

**`Debug`:** `EraseTarget::Document { tenant: 7, collection: "[REDACTED]", key: "[REDACTED]" }`. Kein Namensinhalt in Logs.

### 4.3 Grund und Politik

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub enum ErasureReason { UserRequest, LegalRequest, RetentionExpiry, OperatorAction, TenantOffboarding }
// bewusst ohne Freitext: der Grund landet im Ledger und darf keine personenbezogenen Daten tragen.

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Default)]
pub enum ErasureProfile { #[default] Standard, Regulated }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Default)]
pub enum ReinsertPolicy { #[default] Deny, AllowAfterSealed }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Default)]
pub enum CheckpointTaintPolicy { #[default] Drop, Rewrite, LegalHold }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Default)]
pub enum AwaitLevel { Accepted, #[default] Gated, Purged }

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ErasurePolicy {
    pub checkpoint: CheckpointTaintPolicy,
    pub reinsert: ReinsertPolicy,
    pub sla_secs: u64,                 // 0 = Defaultwert des Workers
    pub allow_g2_unavailable: bool,    // nur Stufe 2
}
```

### 4.4 Garantie

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Default)]
pub struct Guarantee {
    pub gated: bool,          // G1
    pub purged: bool,         // G3
    pub crypto_erased: bool,  // G2
}
impl Guarantee {
    pub const NONE: Self = Self { gated: false, purged: false, crypto_erased: false };
    /// Schnitt: eine Garantie gilt nur, wenn sie in beiden gilt.
    pub const fn meet(self, o: Self) -> Self;
    pub fn label(&self) -> &'static str; // "G0" | "G1" | "G1+G3" | "G1+G2" | "G1+G2+G3"
}
```

Invariante: `purged ⇒ gated`, `crypto_erased ⇒ gated`. Konstruktion nur über `Guarantee::new(g, p, c) -> Result<Self>`; die Felder sind öffentlich für Serde, `new` und `meet` bewahren die Invariante (Test `TST-TYPES-04`).

```rust
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub enum ErasureException {
    OutOfScopeExternalCopies,                       // Backups, Exporte, Replikate, Client-Caches, LLM-Anbieter
    LegalHold { checkpoint: String },
    NotMeasured { layer: String },
    FeatureDisabled { layer: String },
    Unprovenanced { objects: u64 },                 // Altbestand ohne derived_from
    OverlappingSnapshotPinned { min_seq: u64 },     // G3 verzögert durch Pin (I-11)
}
```

### 4.5 Zustandsautomat und Status

```rust
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub enum ErasureState {
    Requested, Planned, Gating, Gated, KeysDestroying,
    Purging, Verifying, Sealed, Completed,
    Failed { layer: Option<String>, attempt: u32 },
    Quarantined { layer: String },
}

impl ErasureState {
    /// Tabelle in §5. Reine Funktion, ohne I/O.
    pub fn can_transition_to(&self, next: &ErasureState) -> bool;
    pub fn reached(&self) -> Guarantee;   // Gated.. -> {gated}, Sealed/Completed -> {gated, purged}; KeysDestroying ändert nichts
    pub fn is_terminal(&self) -> bool;    // nur Completed
    pub fn gate_active(&self) -> bool;    // Gating..Sealed inkl. Failed/Quarantined nach Gated
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct LayerStatus { pub layer: String, pub state: LayerState, pub attempt: u32, pub remaining: Option<u64> }

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum LayerState { Pending, Gated, Purging, Purged, Verified, Failed, Quarantined }

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ErasureStatus {
    pub id: ErasureId,
    pub state: ErasureState,
    pub reached: Guarantee,
    pub layers: Vec<LayerStatus>,
    pub exceptions: Vec<ErasureException>,
    pub requested_at: u64,       // Unix-Nanos (Clock)
    pub gated_at: Option<u64>,
    pub sealed_at: Option<u64>,
    pub ledger_index: Option<u64>,
}
```

`ErasureStatus` enthält **nie** Schlüssel, Namen oder Terme. Test `TST-TYPES-06` (Serialisierung enthält keine Zielbytes).

### 4.6 Cursor und Fortschritt

```rust
/// Undurchsichtiger Wiederaufnahmepunkt eines Layers. Maximal 256 Byte (Journal-Limit).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Default)]
pub struct Cursor(Vec<u8>);
impl Cursor {
    pub const MAX_LEN: usize = 256;
    pub fn new(b: Vec<u8>) -> Result<Self, ErasureError>;   // > MAX_LEN → InvalidCursor
    pub fn as_bytes(&self) -> &[u8];
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct LayerProgress {
    pub cursor: Option<Cursor>,       // None = abgeschlossen
    pub units_done: u64,
    pub bytes_freed: u64,
}
```

### 4.7 Fehler

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum LayerErrorKind { Retryable, Permanent, Poisoned, ScopeViolation, Cancelled }

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ErasureError {
    #[error("invalid erase target: {0}")]                InvalidTarget(&'static str),
    #[error("invalid cursor")]                           InvalidCursor,
    #[error("erasure id reused with different plan")]    IdempotencyConflict,
    #[error("scope ambiguous (key collision)")]          ScopeAmbiguous,
    #[error("tenant isolation violated")]                TenantViolation,
    #[error("unknown erasure id")]                       NotFound,
    #[error("invalid state transition")]                 InvalidTransition,
    #[error("erasure journal poisoned")]                 JournalPoisoned,
    #[error("erasure ledger corrupt")]                   LedgerCorrupt,
    #[error("too many active erasure jobs ({active})")]  Busy { active: u32 },
    #[error("insufficient space: need {needed}, have {available}")] InsufficientSpace { needed: u64, available: u64 },
    #[error("layer {layer} failed ({kind:?})")]          Layer { layer: String, kind: LayerErrorKind },
    #[error("cancelled")]                                Cancelled,
    #[error("io: {0}")]                                  Io(String),   // Text ohne Pfad- und Schlüsselbytes
}
impl ErasureError { pub fn kind(&self) -> LayerErrorKind; /* Mapping für Retry-Politik */ }
impl From<ErasureError> for ContextraError { /* siehe §6 */ }
```

`kind()`-Mapping: `Io`, `Busy`, `InsufficientSpace` → `Retryable`; `JournalPoisoned` → `Poisoned`; `TenantViolation` → `ScopeViolation`; `Cancelled` → `Cancelled`; Rest → `Permanent`.

## 5. Zustandsübergänge (normativ für `can_transition_to`)

| Von | Nach (erlaubt) | Bedingung |
| --- | --- | --- |
| `Requested` | `Planned` | Plan durabel |
| `Planned` | `Gating` | – |
| `Gating` | `Gated` | `Gated`-Marker durabel |
| `Gated` | `KeysDestroying`, `Purging` | `KeysDestroying` nur Stufe 2 |
| `KeysDestroying` | `Purging` | Salt-Generation durabel |
| `Purging` | `Verifying`, `Failed` | – |
| `Verifying` | `Purging` (Restmenge > 0, `attempt += 1`), `Sealed`, `Failed` | – |
| `Sealed` | `Completed` | Gate-Lebensdauer beendet (I-9) |
| `Failed` | `Purging`, `Verifying`, `Quarantined` | nur ab `Gated` |
| `Quarantined` | `Purging` | Operator oder stündliche Wiederholung |
| `Requested`, `Planned`, `Gating` | `Failed` (nur vor `Gated`, Auftrag endet) | `Permanent` |

Alles andere ist verboten, insbesondere **jeder Rücksprung hinter `Gated`** und `Failed → Planned`. Ein Test erzeugt die Menge aller 11×11 Paare und vergleicht sie mit dieser Tabelle (`TST-TYPES-05`).

## 6. Änderung an `ContextraError` und DTO

1. Am **Ende** des Enums:

   ```rust
   /// Zugriff auf Daten, die einem Erase-Auftrag unterliegen. Absichtlich ohne Details.
   #[error("Operation rejected: target is subject to erasure")]
   Erased,
   ```

2. `From<ErasureError> for ContextraError`: `Busy`/`Io`/`InsufficientSpace` → `Storage(String)` mit festem Text ohne Nutzdaten; `InvalidTarget`/`InvalidCursor` → `InvalidInput`; `JournalPoisoned`/`LedgerCorrupt` → `Storage`; Rest → `Internal`.
3. `error_dto.rs`: Code `"ERASED"`, kein Detailfeld. **[?]** Golden-Tests für `error_dto` ergänzen; bestehende Tests dürfen nicht brechen, weil nur angehängt wird.
4. Jede `match`-Stelle auf `ContextraError` hat laut Dokumentation bereits einen `_`-Zweig (`error.rs:118-120`). Prüfauftrag: `rg "ContextraError::" --type rust -A2 | rg "match"` auf Matches ohne Wildcard (Erwartung: keine).

## 7. Stabilität und Serialisierung

- Serde-Formate dieser Typen landen im Journal **nicht**; das Journal nutzt handgeschriebene Layouts (03 §5.2). Serde dient API/MCP-Antworten und Tests.
- `ErasureStatus`/`Guarantee` werden als JSON ausgeliefert. Feldnamen sind eingefroren (Golden-Test).
- Kein `Default` für `ErasureId`, `EraseTarget`, `ErasureState` (kein stilles Nullobjekt).

## 8. Tests

| ID | Inhalt |
| --- | --- |
| TST-TYPES-01 | `validate`: alle Verletzungen aus §4.2, inklusive `Collection{default}` |
| TST-TYPES-02 | `canonical_bytes` injektiv (proptest über Tag, Tenant, Strings mit Trennzeichen-Angriffen wie `"a\0b"`) |
| TST-TYPES-03 | Golden-Test: feste Eingabe → feste Bytes und `target_hash` |
| TST-TYPES-04 | `Guarantee`-Invariante; `meet` ist kommutativ, assoziativ, idempotent |
| TST-TYPES-05 | Übergangsmatrix gleich §5 |
| TST-TYPES-06 | `ErasureStatus`-JSON enthält keine Schlüssel-/Namensbytes |
| TST-TYPES-07 | `Debug`/`Display` enthalten keinen Zielinhalt |
| TST-TYPES-08 | `Cursor::new` lehnt 257 Byte ab |
| TST-TYPES-09 | `ErasureError::kind` deckt alle Varianten ab (Exhaustivitätstest per `strum`-freiem Array) |
| TST-TYPES-10 | `ErasureId::from_u128(x).as_u128() == x`; `Display` hat 32 Zeichen; `Debug` zeigt höchstens 4 Byte |
| TST-TYPES-11 | `ports` re-exportiert **dieselben** Typen (`TypeId`-Gleichheit für `Seq`, `Timestamp`, `TargetKind`) |

## 9. Definition of Done

Build, Test, Clippy (`-D warnings`), `cargo deny` grün; `tests/layering.rs` grün (keine neue Kante); `rg "Erased" crates` zeigt nur die vorgesehenen Stellen; Doku-Kommentare auf jedem öffentlichen Element (Repo-Konvention „# Fehler / # Crash“ ✔ ports).

## 10. Offene Punkte

| Nr. | Frage | Vorschlag |
| --- | --- | --- |
| O-T1 | `validate_identifier` in `types` heben? | Nein. Engine ruft es zusätzlich auf (§4.2) |
| O-T2 | `ErasureReason` mit Freitext? | Nein (PII im Ledger) |
| O-T3 | Golden-Tests `error_dto` vorhanden? | **[?]** WP-0 |
