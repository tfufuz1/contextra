# contextra-types — Crate-Spezifikation (Soll-Zustand, SPEC 2.1)

**Ebene:** E0 (Fundament)
**Kern-Closure:** `ids`, `privacy`, `erasure`, `ports`
**Status:** Normativ (Soll-Zustand)
**Version:** 2.1
**Geltungsbereich:** `crates/contextra-types`
**Normative Quellen:** `docs/SPEC.md` §2.4, §2.5, §4.9, §6.1, §6.3, §11, §14, INV-12.

---

## 1. Zweck

`contextra-types` hält die integrationsweiten Datenmodelle ohne I/O.

Kanonisch hier (SPEC §2.5): `ContextraError`, `DocumentRecord`, `SearchHit`.

Zusätzlich, weil §2.5 sie keinem anderen Crate gibt und sie kein I/O enthalten: `ContextraConfig` und die Typen aus SPEC §14 (`EncryptionConfig`, `SecretString`, `GuardPolicy`, `DataClassPolicy`, `TenantPolicy`, `ErasureConfig`, `CustodyProfile`, `MixedPolicy`).

`ports` hängt von diesem Crate nicht ab. Die Kante zeigt nur hierher: `types` nutzt `ports::Custody` nicht als zweiten Schlüsseltyp. `CustodyProfile` ist die Konfigurationswahl K0–K3. `ports::Custody` bleibt der Slot und kennt K1–K3. `contextra-db` mappt `CustodyProfile::K1` auf `Custody::K1` und so weiter. `K0` erzeugt keinen Slot.

Saga-Typen (`SagaState`, `EraseTarget`, `ErasureError`, `ErasureTicket`) bleiben in `contextra-erasure`. Dieses Crate definiert sie nicht. Es bettet `ErasureError` ein.

## 2. Architekturregeln

1. `#![forbid(unsafe_code)]`. Zero-Panic. `default = []`. Kein `tokio`.
2. Keine Abhängigkeit auf E1, E2 oder E3. Keine Abhängigkeit auf `contextra-crypto`.
3. `Erased` und `NotFound` sind auf dem Lesepfad ununterscheidbar (SPEC §6.1, INV-12). `Display` und `Debug` beider Varianten sind die Zeichenkette `not found`.
4. Kein Freitext in einem Fehler, der ein Löschziel, eine Dokument-ID oder einen Pfad des Ziels nennt.
5. `Restricted` und `RestrictedReadAuditRequired` bleiben eigene Varianten, damit T-DEL-13 den Audit-Fehler benennen kann. Ihr Text ist die feste Zeichenkette `restricted` und nennt kein Ziel. MCP bildet alle vier Varianten auf `not found` ab.

## 3. `ContextraError`

`Debug` ist von Hand implementiert. `Erased` und `NotFound` schreiben beide nur `not found`. Ein abgeleitetes `Debug`, das den Variantennamen zeigt, ist für diese beiden Varianten verboten.

```rust
#[derive(thiserror::Error)]
#[non_exhaustive]
pub enum ContextraError {
    #[error("not found")]
    Erased,

    #[error("not found")]
    NotFound,

    #[error("restricted")]
    Restricted,

    #[error("restricted")]
    RestrictedReadAuditRequired,

    #[error("data class rejected")]
    ClassViolation,

    #[error("storage failure")]
    Storage,

    #[error("cryptographic operation failed")]
    Crypto,

    #[error("invalid input")]
    InvalidInput,

    #[error("snapshot unavailable")]
    SnapshotRevoked,

    #[error("io failure")]
    Io(#[source] std::io::Error),

    #[error("erasure failure")]
    Erasure(#[source] contextra_erasure::ErasureError),

    #[error("index corrupted")]
    CorruptedIndex,
}
```

Keine Variante trägt `String`, `DocId`, Pfad oder Grundtext.

`ErasureError` bleibt typisiert. `ShredVerificationFailed`, `CapabilityUnsupported`, `JournalNotDurable`, `PositiveControlFailed`, `RestoreBeyondHorizon` und die übrigen Varianten aus SPEC §11 leben in `contextra-erasure` und werden nicht hier nachgebaut (R4).

Der Lesepfad in `contextra-db` gibt für einen fehlenden und für einen gelöschten Schlüssel `ContextraError::Erased` zurück (SPEC §6.3). Aufrufer dürfen die beiden Fälle nicht auseinanderhalten. `NotFound` bleibt für interne Pfade, die nie ein Gate gesehen haben, und hat nach außen dasselbe Bild.

`From<ErasureError>` existiert. Eine Umwandlung nach `String` existiert nicht.

## 4. Dokumente und Treffer

```rust
pub struct DocumentRecord {
    pub tenant_id: TenantId,
    pub collection_id: CollectionId,
    pub doc_id: DocId,
    pub version: u64,
    pub data_class: DataClass,
    pub payload: Vec<u8>,
    pub created_at_millis: u64,
}

pub struct SearchHit {
    pub doc_id: DocId,
    pub score: f32,
    pub snippet: Option<String>,
}
```

`SearchHit` beschreibt einen Treffer, den das `ReadGate` bereits freigegeben hat. Ein gelöschtes Dokument wird kein `SearchHit`. `snippet` darf keinen Text eines anderen, gesperrten Dokuments enthalten.

`DocumentRecord` ist das Klartextmodell vor der Versiegelung. Persistiert wird es nur als Chiffretext durch `contextra-db`.

## 5. Konfiguration (SPEC §14)

`SecretString` implementiert `Zeroize` und `ZeroizeOnDrop`. Es hat kein `Clone`, kein `Debug`, kein `Serialize`.

```rust
pub enum EncryptionConfig {
    None,
    Required { passphrase: SecretString },
}
```

`encryption_passphrase: Option<String>` ist kein Feld dieses Crates und kein gültiger Ersatz (SPEC §4.9).

`ErasureConfig` enthält die Felder aus SPEC §14, einschließlich `ledger_dir`, `disk_reserve_bytes`, `custody`, `mixed_policy`, `kek_epoch_len`, `seal_after_deletion_max`, `purge_rebuild_max_days`, `key_backup_max_age`, `group_commit_window_ms`, `epoch_window`, `epoch_max`, `epoch_set_threshold`, `tombstone_rebuild_ratio`, `min_record_bytes` und `erased_set_mirror`.

`CustodyProfile` ist `{ K0, K1, K2, K3 }`. Die Validierung der Grenzwerte führt `Contextra::open` aus. Dieses Crate prüft keine Pfade.

`MixedPolicy` besitzt in Version 2.1 ausschließlich `BlockAll`. Persistierte Werte für andere Varianten werden beim Öffnen mit `ContextraError::InvalidInput` abgewiesen.

## 6. `Cargo.toml`

```toml
[package]
name = "contextra-types"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Shared models, ContextraError, and ContextraConfig"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-privacy = { workspace = true }
contextra-erasure = { workspace = true }
contextra-ports = { workspace = true }
thiserror = { workspace = true }
zeroize = { version = "1.8", features = ["zeroize_derive"] }

[features]
default = []
std = ["contextra-ids/std", "contextra-privacy/std", "contextra-erasure/std"]
```

Keine Abhängigkeit auf `serde` für `SecretString`. `DocumentRecord` und `SearchHit` dürfen `serde` nur hinter einem opt-in Feature `serde` serialisieren, und `SecretString` bleibt davon ausgenommen.

## 7. Tests

| Pflicht | Prüfung |
|---|---|
| INV-12 | `format!("{:?}", ContextraError::Erased)` ist `not found` und gleich dem Text von `NotFound` |
| INV-12 | Keine Fehler-Variante hat ein Feldfeld vom Typ `String` für ein Löschziel |
| R4 | `SagaState` ist in diesem Crate nicht definiert |
| §4.9 | `SecretString` implementiert `ZeroizeOnDrop` und ist nicht `Debug` |
| Zyklus | `contextra-ports` hängt in `cargo metadata` nicht von diesem Crate ab |
