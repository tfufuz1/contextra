# contextra — Crate-Spezifikation (Soll-Zustand, SPEC 2.1)

**Ebene:** E2 (Façade)
**Kern-Closure:** `db`, `types`, `ids`, `privacy`, `erasure`
**Status:** Normativ (Soll-Zustand)
**Version:** 2.1
**Geltungsbereich:** `crates/contextra`
**Normative Quellen:** `docs/SPEC.md` §1, §2.4, §5.3, §5.4, §11.2, §14, INV-7, INV-CLAIMS-1.

---

## 1. Zweck

`contextra` ist die Rust-API für Anwendungen. Sie öffnet die Engine, führt Schreib- und Suchaufrufe auf `contextra-db` zurück und startet Löschungen.

Die Façade enthält keine zweite Saga, keinen Cache und keine Lineage. S4 und S5 liegen nicht hier. `contextra-engine` ist ein deprecated Re-Export von `contextra-db` und ebenfalls keine Schicht.

## 2. Architekturregeln

1. `#![forbid(unsafe_code)]`. Zero-Panic. `default = []`.
2. Keine Abhängigkeit auf ein entferntes Crate (SPEC §2.2). Kein Feature `candle`, `onnx` oder `ollama`.
3. Keine Aussage „O(1)", „DSGVO-konform", „gelöscht" oder „unwiederbringlich" in der API-Dokumentation, solange sie nicht an eine gemessene Klasse gebunden ist (INV-7, INV-CLAIMS-1).
4. Collection-Löschung und Subjekt-Löschung geben das `ErasureTicket` von Takt 1 zurück. Sie beenden die Saga nicht.

## 3. Öffnen

`Contextra::open` prüft vor dem ersten Lesen (SPEC §14):

- `data_dir` ist vorhanden und beschreibbar.
- `erasure.ledger_dir` ist kein Unterverzeichnis von `data_dir`.
- `erasure.disk_reserve_bytes` ist mindestens 1 GiB.
- Bei `EncryptionConfig::Required` ist die Passphrase nicht leer.
- `mixed_policy` ist `BlockAll`.
- Im Regulated-Tier ist `erased_set_mirror` gesetzt.
- `CustodyProfile::K2` und `K3` ohne bestandene Konformitätssuite liefern `CapabilityUnsupported`.

Danach ruft `open` `contextra-db` auf. Dort läuft `Chain::recover`, bevor der Lesepfad öffnet. Fehlt das `ReadGate` bei `GuardPolicy::Required`, schlägt `open` fehl (SPEC §11.2).

`ContextraConfig` wird aus `contextra-types` re-exportiert und hier nicht ein zweites Mal definiert (R4).

## 4. Schreiben und Suche

`insert_k` nimmt `Validated<T>`. `insert_p` nimmt `SubjectRef` und liefert `PersonalRef`. `insert` einer D-Klasse ohne `LineageRef` reicht den Fehler `ClassViolation` aus `contextra-db` durch.

`hybrid_search`, `get` und `scan` laufen über `TenantScopedStorage`. `get` auf einen gelöschten und auf einen nie geschriebenen Schlüssel liefert `ContextraError::Erased` mit dem Außentext `not found`.

## 5. Löschen

```rust
pub async fn erase_subject(
    &self,
    subject: SubjectRef,
    attestation: RetentionAttestation,
) -> Result<ErasureTicket, ContextraError>;

pub async fn drop_collection(
    &self,
    collection: CollectionId,
    attestation: RetentionAttestation,
) -> Result<ErasureTicket, ContextraError>;
```

Das Ticket nennt `ErasureId` und den Zustand nach Takt 1: `Frozen`, `Restricted` oder `Refused`. Es ist kein `DeletionProof`. Der Zustand `KeyRevoked` existiert nicht. Der kanonische Zustand nach der Schlüsselvernichtung heißt `KeyDestroyed` und tritt in Takt 2 ein, nicht in dieser Rückgabe.

`erase_subject` und `drop_collection` dokumentieren die Saga als die Schritte 1 bis 9. Sie dokumentieren keine konstante Laufzeit. Die Schlüsselvernichtung eines einzelnen `DocDek` ist ein Schritt der Saga. Die Saga insgesamt ist es nicht.

Ein Profil `enterprise_regulated` setzt `GuardPolicy::Required`, `DataClassPolicy::Enforced`, `EncryptionConfig::Required` und ein `ledger_dir`, das der Aufrufer außerhalb von `data_dir` übergibt. Das Profil ist eine Voreinstellung der Konfiguration. Es ist keine Compliance-Bescheinigung.

## 6. `Cargo.toml`

```toml
[package]
name = "contextra"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Embedded hybrid-search facade"

[lints]
workspace = true

[dependencies]
contextra-db = { workspace = true }
contextra-types = { workspace = true }
contextra-ids = { workspace = true }
contextra-privacy = { workspace = true }
contextra-erasure = { workspace = true }

[features]
default = []
```

`uuid` mit ungeprüftem `UUIDv4` ist keine Abhängigkeit. IDs entstehen in `contextra-ids`.

## 7. Tests

| Pflicht | Prüfung |
|---|---|
| §14 | `ledger_dir` unter `data_dir` lässt `open` fehlschlagen |
| §14 | Leere Passphrase bei `Required` lässt `open` fehlschlagen |
| T-DEL-5 | `EncryptionConfig::None` liefert ein Ticket, dessen späterer Bericht höchstens Klasse L trägt |
| Lesepfad | `get` nach `Frozen` und `get` einer unbekannten `DocId` haben denselben `Display`-Text |
| INV-7 | Die Rustdoc-Texte von `erase_subject` und `drop_collection` enthalten weder `O(1)` noch `DSGVO-konform` |
