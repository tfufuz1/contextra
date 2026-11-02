# Typ-Registry

Abgeleitet aus [`SPEC.md`](SPEC.md) Abschnitte 7, 8, 9 und 11 sowie Anhang C. Bei Widerspruch gilt die SPEC.

## 1. Regel

Jeder Domänenbegriff, Identifikator und Trait existiert genau einmal an seinem kanonischen Ort (SPEC 8.1, D-97). Doppelte Definitionen über Crate-Grenzen hinweg sind verboten. Re-Exporte sind nur einzeln und nur, wenn der Typ in einer Trait-Signatur vorkommt (AR-10). Glob-Re-Exporte (`pub use …::*`) sind verboten.

## 2. Kanonische Typen je Crate

### `contextra-types` (Ring 0, kein I/O)

| Typ | Zweck |
|---|---|
| `Tag` | 16-Byte-Tag (`[u8; 16]`-Newtype), Basis aller keyed Identifikatoren |
| `KeyBuf` | Schlüsselpuffer mit fester Länge, kein String (SPEC 8.1) |
| `DocRef { subject: Option<SubjectRef>, key: DocKey }` | Dokumentadresse; der Betroffene ist Teil der Identität (D-70, D-08) |
| `DocHandle` | opaker Suchtreffer mit `cluster_tag` und `doc_tag` (D-70) |
| `SearchPage { hits, exhausted }` | Suchergebnis; `exhausted` markiert fehlende Treffer (D-62) |
| `TenantRef`, `CollectionRef`, `SubjectRef`, `DocKey` | Adressen für Mandant, Collection, Betroffenen und Dokument |
| `ErasureId` | Kennung einer Löschung, abgeleitet aus Ziel-Commitment und Attestierung |
| `EraseTarget` | `Document`, `Subject`, `Collection`, `Tenant` (SPEC 7.3) |
| `EraseMode` | `Erase` oder `Restrict` |
| `Timing` | `Now` oder `Batched` |
| `RetentionAttestation { code, attested_by, attested_at }` | Pflichtangabe bei jedem Löschauftrag (D-15) |
| `RetentionCode` | geschlossene Liste: `NoObligation`, `ObligationExpired`, `ObligationActive`, `LegalClaimsPending`, `DataSubjectRequestedRestriction`, `OperatorOverride` (E-L12) |
| `ErasureStatus` | `Requested`, `Gated`, `KeyDestroyed`, `Purging`, `Verifying`, `Completed`, `Pending { reason }`, `Refused(reason)` |
| `PendingReason` | `Io`, `SlaExceeded`, `LayerFailed(id)`, `WitnessMissing`, `DiskReserve` (SPEC 11.2) |
| `RefusalReason` | Ablehnungsgründe, z. B. `RetentionObligation` |
| `KeyCoverage` | `full`, `unlinked`, `none` je Schicht (D-69) |
| `LayerWitness` | Zeuge einer Schicht; nur aus `Scanner<Armed>` konstruierbar (D-72, I-19) |
| `ResidualRisk` | geschlossener Katalog nach SPEC Anhang D |
| `GateVariant` | `Erased`, `Restricted` (SPEC 11.3) |
| `HealthReport`, `HealthState`, `DegradedReason` | Gesundheitsstatus (D-64, 14.5) |
| `Error` | geschlossene Enum: `Config`, `Io`, `Corrupt`, `Auth`, `Erased`, `Restricted`, `NotFound`, `Conflict`, `Limit`, `SnapshotRevoked`, `Cancelled`, `Refused`, `Internal` (SPEC 7.5) |
| `Conflict` | `IdCollision`, `Exists` (`SubjectImmutable` entfällt, D-40) |
| `Limit` | `Capacity`, `DiskReserve`, `Snapshots`, `QueryTooLarge` und weitere `Limit::*` (D-87) |

Alle öffentlichen Typen sind `#[non_exhaustive]`, außer Konfigurationsbausteinen (D-66). Fehlertexte sind fest und enthalten keine Nutzerinhalte (HR-4).

### `contextra-crypto` (Ring 1)

| Typ | Zweck |
|---|---|
| `Secret` | `zeroize`-on-drop, ohne `Debug`, `Display` und `Clone` (SPEC 7.1) |
| `KdfParams` | Argon2id-Parameter mit Grenzen aus D-06 |
| `KeySlot` | Slot mit `slot_id`, `anchor_id`, `kdf_id`, `m`, `t`, `p`, `slot_salt`, `wrapped_master` (SPEC F.2) |

### `contextra-ports` (Ring 0, nur Traits)

`StorageEngine`, `TextIndex`, `VectorIndex` (statisch dispatcht, D-03); `Vfs` (ein Trait, D-04); `Clock`, `Rng`, `IdGen`, `Metrics`; `ErasureLayer`, `ReadGate`, `RecordSealer`, `FileKeyring`, `LedgerSink` (dyn-fähig, `BoxFuture`, D-03).

Die Trait-Signaturen stehen im Rustdoc. Eine Abweichung von SPEC 8.2 ist ein Defekt.

### `contextra-erasure` (Ring 3, Skelett ausstehend)

`Scanner<Armed>` (Typzustand, `#[must_use]`, private Konstruktoren), `FileVault` (Implementierung von `RecordSealer` und `FileKeyring`), `Journal`, `Ledger`, `ErasedSet`, Berichtsbau. Der Typ `Scanner<Armed>` entsteht nur nach erfolgreicher Positivkontrolle (D-72).

### `contextra` (Ring 4)

Öffentliche Handles `Contextra`, `Tenant`, `Collection` (Re-Exporte aus `engine`); `EngineBuilder` liegt in `engine` (D-89). `system_defaults()` in `wiring`.

## 3. Entfernte oder ersetzte Typen

| Früherer Typ | Ersatz oder Entscheidung | Quelle |
|---|---|---|
| `DocId(u64)` (auch `u128` mit Feature `docid-128`) | `DocRef` und `DocHandle`, Identität über `doc_tag` | D-08, D-84 |
| `TenantPolicy` (Default `Optional`) | `TenantContext` ist Pflicht | D-07 |
| `EncryptionMode`, `Option<Arc<KeyManager>>` | Verschlüsselung ist nicht abschaltbar | D-05 |
| `allow_legacy_integrity_key_fallback`, `legacy-wal-key` | entfällt | D-05, D-84 |
| `Collection::delete` | `Collection::soft_delete` (logisch) | D-51 |
| `Conflict::SubjectImmutable` | entfällt; Betroffener ist Teil der Identität | D-40 |
| `Locator`-Familie, Region `LOC` | entfällt; `DocRef` adressiert direkt | D-70 |
| `AutoExtractionMode` | entfällt vollständig | D-27 |
| `LicenseGate`, `with_license_gate` | entfällt | D-23 |
| `TextEmbeddingEngine`, `embedding`, `reranker`, `query_rewriter` (Ports) | entfällt | D-24 |
| `saos`, `tombstone` (in `types`) | außerhalb des Soll; entfernen | SPEC 5.1, F-36 |
| zweites `Vfs`-Trait in `durable-fs` | entfernt; einziger Trait in `ports` | D-04, F-29 |
| `u64::MAX` als Sentinel für `min_read_snapshot` | `Option`; ohne Snapshots gilt `current_seq` | 8.7, I-12 |
| Feature-Schalter `encryption-at-rest`, `onnx`, `sandbox` u. a. | entfällt | D-84 |

Die vollständige Rückbauliste steht in SPEC D-84.

## 4. Konstanten und Labels (SPEC Anhang C)

| Zweck | Label |
|---|---|
| `tenant_tag` | `ctx/tenant/v1` |
| `coll_tag` | `ctx/coll/v1` |
| `subject_tag` | `ctx/subj/v1` |
| `doc_tag` | `ctx/doc/v2` |
| `term_tag` | `ctx/term/v1` |
| `name_tag` | `ctx/name/v1` |
| Commitment | `ctx/commit/v1` (mit Epochenschlüssel) |
| Schlüsselableitung | `ctx/enc/v1`, `ctx/idx/v1`, `ctx/unit/v1`, `ctx/sign/v1`, `ctx/wal/v1`, `ctx/ledger/v1`, `ctx/file/v1`, `ctx/fkmask/v1`, `ctx/dek/v2`, `ctx/dek-salt/v2` |
| Record-AAD | `ctx/rec/v2` |
| Keyslot-AAD | `ctx/slot/v1` |
| Vault-Slot-MAC | `ctx/vault-mac/v1` |
| Header-MAC | `ctx/hdr/v1` |
| Textbindung-AAD | `ctx/tbind/v1` |

Weitere Konstanten: Tag 16 Byte, Salt 32 Byte, Nonce 12 Byte, Tresor-Slot 80 Byte, Segmentziel 8 bis 32 MiB, SSTable-Ziel 32 MiB ab Level 1, `k_rrf = 60`, `commit_epoch = 365 d`, `ef_max = 4096`, Posting-Block 128, Tresor-Lastfaktor höchstens 0,7.

## 5. Pflege

Ein neuer Typ wird zuerst in der SPEC beschrieben und dann in dieser Registry eingetragen. Änderungen an Typen, die in Trait-Signaturen vorkommen, brauchen eine SPEC-Änderung mit Eintrag in Anhang H. Prüfbar ist die Regel über `contextra-arch-tests` (T-ARCH-6, T-ARCH-7).
