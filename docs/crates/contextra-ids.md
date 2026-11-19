# contextra-ids — Crate-Spezifikation (Soll-Zustand, SPEC 2.1)

**Ebene:** E0 (Fundament)
**Kern-Closure:** leer
**Status:** Normativ (Soll-Zustand)
**Version:** 2.1
**Geltungsbereich:** `crates/contextra-ids`
**Normative Quellen:** `docs/SPEC.md` §2.5, §3.2, §6.2, §13 (INV-4, INV-12).

---

## 1. Zweck

`contextra-ids` ist der einzige Besitzer der Identitätstypen (R4, SPEC §2.5).

Kanonisch hier:

- `DocId`, `TenantId`, `CollectionId`, `SubjectRef`, `PersonalRef`
- `ErasureId`, `GateId`, `OperatorRef`, `Seq`, `Timestamp`, `ScopeId`
- `TenantScoped<T>`, `TenantScopeViolation`, `IdError`

`DocId` ist immer 128 Bit. Ein Feature `docid-128` existiert nicht (R3). Die Breite eines Löschpfads ist keine Feature-Weiche.

Die Ableitung `BLAKE3-keyed(K_scope, tenant ‖ collection ‖ user_key)` führt `contextra-crypto` aus, aufgerufen von `contextra-db`. Dieses Crate hält den Newtype und die Ordnungsrelation. Die Kollisionsprüfung beim Insert liegt in `contextra-db`. Die dichte `u32`-`Slot`-Abbildung liegt beim `ReadGate` in `contextra-erasure`.

`SubjectRef` ist eine Zufalls-ID. Es ist kein HKDF-Schlüssel und kein `DocDek`.

Namen außerhalb von §2.5 (`EntityId`, `DocKey`, `DocRef`, `TenantRef`, `CollectionRef`, `FileId`, `StoreId`) werden hier nicht als zweite Identität definiert. `TxId` ist `Seq`.

## 2. Architekturregeln

1. Keine Abhängigkeit auf ein anderes Workspace-Crate.
2. `#![forbid(unsafe_code)]`. Zero-Panic. `default = []`.
3. Zufalls-IDs entstehen über `OsRng::try_fill_bytes`. Ein RNG-Fehler liefert `IdError::EntropyExhaustion` und keine ID.
4. `SubjectRef` hat kein `Clone`, kein `Debug` des Rohwerts, kein `Serialize` des Rohwerts. `Zeroize` und `ZeroizeOnDrop` gelten.
5. Logs, `Display` und `Debug` von `SubjectRef` und `PersonalRef` enthalten nur den geschwärzten Typnamen (INV-12).

## 3. `DocId`

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct DocId(pub [u8; 16]);

const _: () = {
    assert!(core::mem::size_of::<DocId>() == 16);
    assert!(core::mem::align_of::<DocId>() == 16);
};
```

`Ord` ist byteweise aufsteigend und der Tie-Break der Suche (INV-5). `DocId::from_bytes` ist der einzige Konstruktor aus fertigen Bytes. Es gibt keinen Konstruktor aus `u64`.

## 4. Betroffenen-Referenzen (SPEC §3.2)

```rust
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Zeroize, ZeroizeOnDrop)]
pub struct SubjectRef([u8; 16]);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Zeroize, ZeroizeOnDrop)]
pub struct PersonalRef([u8; 16]);
```

`generate` füllt 16 Byte über `OsRng::try_fill_bytes`. `from_bytes` ist für das Lesen persistierter Referenzen. `Debug` und `Display` schreiben `SubjectRef([REDACTED])` bzw. `PersonalRef([REDACTED])`.

`PersonalRef` ist der Verweis aus einem K-Dokument auf P-Daten. Das K-Dokument enthält kein `SubjectRef`.

## 5. Lösch- und Audit-Identitäten

| Typ | Layout | Rolle |
|---|---|---|
| `ScopeId` | `[u8; 16]` | 128-Bit-Scope aus §5.3 Schritt 1. `Copy`. Wird im Katalog auf den `Slot` aufgelöst, nicht hier. |
| `ErasureId` | `[u8; 16]` | Eine Saga. `generate` wie oben. `Debug` zeigt höchstens die ersten vier Bytes. |
| `GateId` | `[u8; 16]` | Kennung eines Lesesperr-Eintrags. `Copy`. |
| `OperatorRef` | `[u8; 16]` | Operator einer Lösch- oder Restrict-Aktion in der Audit-Kette. Kein Betroffenen-Identifikator und kein Ersatz für `SubjectRef`. |
| `Seq` | `u64` | Transaktionszeit, streng monoton im Aufrufer. `Seq::ZERO` ist 0. |
| `Timestamp` | `u64` | Unix-Nanosekunden. Die Uhr liegt in `contextra-env`. |

## 6. Mandant

`TenantId(u64)` und `CollectionId(u64)` reservieren den Wert 0 für `SYSTEM`. `try_new` lehnt 0 für einen Benutzer-Mandanten und eine Benutzer-Collection ab.

```rust
pub struct TenantScoped<T> {
    tenant_id: TenantId,
    value: T,
}
```

`into_inner_checked` liefert `TenantScopeViolation::Mismatch`, wenn der erwartete Mandant abweicht. Ein stilles Durchreichen gibt es nicht.

## 7. Fehler (`IdError`)

`#[non_exhaustive]`: `EmptyKey`, `EmptyIdentifier`, `IdentifierTooLong`, `TenantSystemReserved`, `CollectionZero`, `TenantScopeViolation { expected, actual }`, `EntropyExhaustion`. Die Entropie-Variante trägt keine Rohbytes.

## 8. `Cargo.toml`

```toml
[package]
name = "contextra-ids"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Canonical identity types for Contextra"

[lints]
workspace = true

[dependencies]
zeroize = { version = "1.8", features = ["zeroize_derive"] }
rand = { workspace = true }
thiserror = { workspace = true }

[features]
default = []
std = []
```

`docid-128` ist kein Feature. `serde` serialisiert `SubjectRef` und `PersonalRef` nicht.

## 9. Tests

| Pflicht | Prüfung |
|---|---|
| Layout | `size_of::<DocId>() == 16` in jeder Build-Konfiguration |
| Feature | `cargo metadata` zeigt kein Feature `docid-128` |
| RNG | `SubjectRef::generate` liefert bei RNG-Fehler `EntropyExhaustion` und kein Objekt |
| Drop | Nach Drop von `SubjectRef` sind die 16 Byte `0x00` |
| Anzeige | `Debug` von `SubjectRef` enthält keine der 16 Byte |
| Ordnung | `DocId`-`Ord` ist byteweise aufsteigend |
