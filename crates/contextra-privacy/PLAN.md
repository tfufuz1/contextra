# contextra-privacy — Crate-Spezifikation (Soll-Zustand, SPEC 2.1)

**Ebene:** E0 (Fundament)
**Kern-Closure:** `ids`, `env`, `ports`
**Status:** Normativ (Soll-Zustand)
**Version:** 2.1
**Geltungsbereich:** `crates/contextra-privacy`
**Normative Quellen:** `docs/SPEC.md` §2.4, §2.5, §3, §5.9 Schicht S4, §13 (INV-12, INV-LINEAGE).

---

## 1. Zweck

`contextra-privacy` besitzt die Datenklassifikation und den Schreibschutz für Klasse K.

Kanonisch hier (SPEC §2.5): `DataClass`, `Validated<T>`, `EntityRecognizer`.

Schicht **S4**: der `SurrogateVault` hält dokumentbezogene Surrogate im Arbeitsspeicher. Er implementiert `Purgeable` aus `contextra-ports`. `contextra-db` registriert ihn beim Koordinator. Dieses Crate definiert `ErasureLayer` nicht und importiert ihn nicht.

Eine Abhängigkeit auf `contextra-types` ist ausgeschlossen. `types` bindet `DataClass` aus diesem Crate. Die Gegenrichtung würde einen Zyklus schließen. Eine Abhängigkeit auf `contextra-crypto` ist ausgeschlossen (E0 hängt nicht von E1 ab). Die Schlüsseltypen bleiben in `contextra-crypto`.

## 2. Architekturregeln

1. `#![forbid(unsafe_code)]`. Zero-Panic. `default = []`.
2. Keine Abhängigkeit auf E1, E2 oder E3.
3. Fehler und Logs enthalten die Regel-ID und den Bytebereich, nie den erkannten Klartext (INV-12).
4. `Purgeable::invalidate` ohne Invalidierung und ohne `zeroize` ist verboten (INV-WIRING-2).

## 3. `DataClass` (SPEC §3.1)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataClass {
    /// Unternehmenswissen. `DocDek` der Einheit oder `ArtKey` des Artefakts, gewrappt unter `KEK[coll, e]`.
    K,
    /// Personenbezogene Daten. Adressiert über `SubjectRef`, verschlüsselt mit dem `DocDek` der Einheit.
    P,
    /// Abgeleitete Daten. `ArtKey` und verpflichtende `LineageRef`.
    D,
    /// Löschmaterial. `ErasureKey` und `K_saga`.
    E,
}
```

Es gibt kein `SubjectKey`. P-Daten werden nicht mit einem aus der KEK abgeleiteten Personenschlüssel verschlüsselt (SPEC §4.1). Die Löschung einer Einheit ist die Saga in `contextra-erasure`, nicht eine Laufzeitzusage dieses Crates.

K-Daten dürfen kein `SubjectRef` enthalten. Personenbezug in K läuft über `PersonalRef` (SPEC §3.2).

## 4. `Validated<T>` (SPEC §3.3)

K-Schreibpfade akzeptieren nur `Validated<T>`. Der Wrapper ist nur über `validate_k_payload` erzeugbar. Ein öffentlicher Konstruktor, der die Prüfung überspringt, existiert nicht.

`validate_k_payload` ruft `EntityRecognizer::find_first_pii`. Ein Treffer liefert `PersonalDataFound { entity_type, byte_range }` ohne den Substring. `into_inner` gibt den geprüften Wert an den Aufrufer in `contextra-db` zurück. Die `LineageRef` für Klasse D prüft `contextra-db` (INV-LINEAGE). Dieses Crate besitzt sie nicht.

## 5. `EntityRecognizer`

Mindestregeln: IBAN, E-Mail, Telefonnummer. `new` liefert `Result`. Eine ungültige Regel ist `PrivacyError::Regex`, kein Panic. `find_all_pii` sortiert nach `start` aufsteigend. Die Regelmenge ist fest. Ein Feature schaltet keine Regel ab (R3).

## 6. `SurrogateVault` (Schicht S4)

Der Vault liegt nur im Arbeitsspeicher. Er bildet ein Surrogat auf den Originalwert ab, damit ein Egress-Pfad Klartext-PII nicht verlässt und die Rücksubstitution nach der Rückkehr möglich ist.

```rust
impl Purgeable for SurrogateVault {
    fn invalidate(&self, doc_key: &[u8]) -> Result<(), PortError>;
}
```

`invalidate` nimmt `&self`. Der Vault hält die Tabelle hinter einer Sperre, entfernt jeden Eintrag des `doc_key` und zeroized den Originalwert, bevor die Sperre fällt. Ein unbekannter Schlüssel ist ein No-op und `Ok`. Ein Eintrag, der nach `invalidate` noch lesbar ist, ist ein Fehler. `Drop` zeroized die gesamte Tabelle.

`rehydrate` läuft nur im Prozess. Der Vault wird nicht persistiert, nicht in einen Checkpoint geschrieben und nicht geloggt.

## 7. Egress

`EgressGateway` gibt nur Werte vom Typ `GuardedPayload<Sanitized>` an einen Außenkanal. Der Übergang `Unsanitized` → `Sanitized` läuft durch den Recognizer. Ein Fund wird ersetzt oder der Versand endet mit `PrivacyError::EgressBlocked`. Ein Kanal ohne diesen Übergang existiert nicht.

`ContextEditAudit` speichert Regel-ID, Bytebereich und einen Hash des Surrogats. Er speichert keinen Klartext.

## 8. Fehler (`PrivacyError`)

`#[non_exhaustive]`: `Regex`, `PersonalDataFound`, `EgressBlocked`, `VaultPoisoned`. Keine Variante trägt den erkannten Text.

## 9. `Cargo.toml`

```toml
[package]
name = "contextra-privacy"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Data classes, class-K validation, and the purgeable surrogate vault"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-ports = { workspace = true }
regex = "1.10"
thiserror = { workspace = true }
zeroize = { version = "1.8", features = ["zeroize_derive"] }

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
```

## 10. Tests

| Pflicht | Prüfung |
|---|---|
| Klasse K | `validate_k_payload` lehnt eine IBAN ab und der Fehler enthält die IBAN nicht |
| Klasse K | Ein Wert ohne `Validated` ist am Typ des K-Schreibpfads nicht übergebbar |
| S4 | Nach `invalidate` ist der Originalwert weder über `rehydrate` noch im allozierten Puffer lesbar |
| S4 | Zweites `invalidate` desselben `doc_key` ist ein No-op |
| Zyklus | `cargo metadata` zeigt keine Kante auf `contextra-types` oder `contextra-crypto` |
