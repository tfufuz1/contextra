# contextra-erasure — Implementierungsspezifikation (SOLL-Zustand nach SPEC v6)

Stand: 17. Nov 2026 · Bezug: `docs/SPEC.md` v6 (§4, §9, §11 bis §15, Anhang A) · Geprüfter HEAD: `24487fa16` (16.11.2026)
Status dieses Dokuments: **Entwurf zur Freigabe.** Es ersetzt weder `docs/SPEC.md` noch `crates/contextra-erasure/SPEC.md`; Widersprüche sind nach SPEC §1.2 Befunde im Register.

Belegmarken wie in SPEC §1.1: **[V]** am HEAD gelesen oder gemessen · **[E]** Einschätzung, nicht belegt · **[D-Vorschlag]** Festlegung, die einen ADR braucht · **[A]** aus Audit-Bericht übernommen, hier nicht nachgeprüft.

---

## 0 Methode und Grenzen

| Punkt | Inhalt |
|---|---|
| Quellen | `docs/SPEC.md`, `docs/01_schluesselmodell.md`, `docs/02_loeschung.md`, `docs/Umsetzbarkeit_Loeschung_Contextra.md`, `reports/*`, gesamter Quelltext und alle Tests von `crates/contextra-erasure` |
| Toolchain | Rust 1.89.0 (Ubuntu-Paket `rustc-1.89`), 1 vCPU. `static.rust-lang.org` ist nicht erreichbar. |
| **Wichtige Einschränkung** | Im geklonten Repository fehlen `Cargo.toml` (Wurzel), `governance/`, `justfile`, `.github/`. Gebaut wurde in einem **Ersatz-Workspace** (`/tmp/ws`) mit dem Original-`Cargo.lock`, den `[workspace.dependencies]` aus den Crate-Manifesten rekonstruiert und **ohne Workspace-Lints**. Die Gates G-L1 bis G-L4, G-H1, `unwrap-gate`, `ratchet` sind **nicht** gelaufen. |
| Nicht ausgeführt | Clippy, Crash-Matrix-Läufe, Loom, Mutation, Benchmarks, `tools/*.py`-Gates |
| Heuristiken | „Kein `unwrap`/`expect`/`panic` in Produktionscode“ wurde per Skript über Zeilen vor dem ersten `#[cfg(test)]` je Datei geprüft; das ist eine Näherung. |

---

## 1 Ziel und Abgrenzung

**Ziel:** `contextra-erasure` erfüllt den Teil von SPEC §11 bis §14, der in diesem Crate liegt (Klasse P und E, Löschkette L0 bis L6, Zusagen Z-E1 bis Z-E12, Crash-Matrix, Berichte), bleibt dabei E0 (Kern-Closure leer, SPEC §2.2) und erfüllt den Bausteinvertrag (SPEC §5.4).

**Im Crate:** Saga, Gate, Journal, Tresor, Schlüsselhierarchie, Bereinigungssteuerung, Messung, Bericht, Ledger, Restore-Schutz.

**Nicht im Crate (Abhängigkeiten, eigene Schritte):**

| Thema | Ort | SPEC-Schritt |
|---|---|---|
| `PersonalRef` (128 Bit, zufällig) | `ids` | S-100 |
| K-Validator, `Validated<T>` | `rules` + `privacy` | S-104, S-106 |
| Zeitreise, Simulation, Takedown | `rules` | S-104 bis S-108 |
| Unabhängiger Prüfer | `contextra-verify` | S-110 |
| Verdrahtung in Lese-/Schreib-/Löschpfad | `db`, `store`, `engine`, `text`, `vector`, `kvcache` | nicht in SPEC §15 geplant (siehe E-D07) |

**Nicht-Ziele:** keine Rechtsaussagen (INV-7); keine Compliance-Zusagen ohne gemessene Stufe; keine Hersteller-Hintertür (Z-E12); keine In-Place-Migration der ID-Breite (D-16).

---

## 2 Ausgangslage (gemessen)

### 2.1 Build und Tests [V]

| Prüfung | Ergebnis |
|---|---|
| `cargo check --no-default-features` | grün, 0 Warnungen |
| Kombinationen aus `signing`, `vault`, `async` (7 Stück, alle ohne Default) | alle grün, 0 Warnungen |
| `cargo check --all-targets --keep-going` | **78 Fehler** in 2 Targets: `proof_crash_resume_matrix` (66), `proof_fail_closed_and_consistency` (12). Alle übrigen Targets kompilieren. |
| Tests der übrigen 42 Targets (`--lib` + `tests/*`) | 216 bestanden, 0 fehlgeschlagen, 1 ignoriert |
| `INV-11` (unsafe) | `#![forbid(unsafe_code)]`, 0 Vorkommen |
| `INV-12` (Panics) | 0 Treffer in Produktionscode (Heuristik) |
| `mod.rs` | keine |

Ursachen der 78 Fehler (Auszug, [V]): `ErasureCoordinator::current_phase` (12×), `advance_to_fence`/`_wal`/`_keys`/`_rewrite`/`_witness`, `PurgePhase::{Completed, Admission, Fence, KeyDestruction, SegmentRewrite, WitnessCollection}`, `LayerId::DocStore`, `WitnessLayer::{DocStore, VectorPayload}`, `FenceGateError::{TokenMismatch, NotFound}`, `MemoryWalSink::{record_wal_entry, dispose_scope_wal}`, `StepClock::system_time_nanos`, nicht exportierte `MemoryKeyStore`/`MemoryResidueScanner`, privates `WalSink`, geänderte Funktions-Signaturen.

`docs/decoupling/REPORT.md` im Crate behauptet „alle 41 Test-Suiten laufen fehlerfrei“ und `cargo test` GRÜN. **Das ist falsch** (Befund E-F00).

### 2.2 Größe [V]

83 Quelldateien, rund 22.900 Zeilen in `src/` (Zählung per `wc -l`, enthält Tests in `src`). Größte Dateien: `deletion_proof.rs` (2.743), `types_local.rs` (2.439), `kv_shredding.rs` (982), `saga/core.rs` (932), `vault/sealer.rs` (825).

---

## 3 Befunde gegenüber dem SOLL-Zustand

IDs `E-Fnn` sind für dieses Dokument vergeben; Übernahme ins Register (SPEC Anhang A, ab F-21) ist Teil von E-0.

### 3.1 Sicherheitskritisch (blockieren Z-E2, Z-E9, SPEC §11.4.4)

| ID | Befund | Beleg [V] |
|---|---|---|
| E-F01 | **Fallback-Schlüssel nach „erased“.** Fehlt ein Tresor-Slot, fällt `VaultSealer::resolve_dek` auf `blake3::keyed_hash(k_unit, key)` zurück. Ein vernichteter Schlüssel bleibt damit ableitbar; Krypto-Shredding ist wirkungslos. Außerdem setzt die Funktion `tenant_tag = coll_tag = doc_tag`, `subject_tag = None`. | `src/vault/sealer.rs`, `resolve_dek` (ca. Z. 255 bis 290) |
| E-F02 | **`VaultSealer::seal` ist keine AEAD.** Es XOR-t mit einer 32-Byte-Maske (`keyed_hash(dek, "ctx/envelope/stream")`), wiederholt sich nach 32 Byte, ohne Nonce und ohne Tag. | `src/vault/sealer.rs`, `impl RecordSealer for VaultSealer` |
| E-F03 | **Selbstgebaute Stromchiffre mit deterministischer Nonce.** `seal_record` leitet die Nonce aus `dek ‖ doc ‖ key_bytes ‖ len` ab. Gleiche Eingabe ergibt gleichen Chiffretext; Länge ist exakt sichtbar (Z-E2: „keine exakten Längen“). Genutzt in `journal/appendlog.rs` und `scope/file.rs`. Tag-Vergleich per Handschleife statt `subtle` (obwohl Abhängigkeit vorhanden). | `src/crypto_local.rs` |
| E-F04 | **Stille Fallbacks in Ableitungen.** `derive_dek`, `mask_file_key`, `mask_epoch_key` ersetzen bei HKDF-Fehler den Schlüssel durch `blake3::keyed_hash`; `seal_record` setzt bei fehlendem Tag-Slice einen Null-Tag. | `src/crypto_local.rs`, `src/vault/keyring.rs`, `src/vault/slot.rs` |
| E-F05 | **Zufall nicht fehlbar.** `rand::rngs::OsRng.fill_bytes` (Panic statt `Result`) in `kv_shredding.rs` und `ed25519_proof.rs`. SPEC Z-E9 und `01_schluesselmodell.md` §1.5 verlangen fehlbaren Zufall. | `src/kv_shredding.rs` (Z. 245, 369, 375, 537 ff.) |
| E-F06 | **Nullschlüssel als Standard.** `FileVault::new_in_memory` nutzt `vault_id = [0;16]`, `mac_key = [0;32]`. SPEC §11.4.4: „kein Standardschlüssel“. | `src/vault.rs` |
| E-F07 | **Schlüsseltypen sind `Clone`.** `SubKey`, `GroupKek`, `RecordDek` kopieren Schlüsselmaterial (`01_schluesselmodell.md` §1.4). `ProtectedDek`, `Dek`, `UnitKey` ebenso. | `src/kv_shredding.rs`, `src/vault/sealer.rs`, `src/crypto_local.rs` |
| E-F08 | **`KeyRegistry` nicht persistent** (Master-KEK zufällig im RAM, [A] Audit N1; Konstruktor in `kv_shredding.rs` Z. 243 bis 257 gelesen) und `revoke_record(&str)` wirkungslos [A, Audit N2; Funktion bei Z. 466 vorhanden, Verhalten hier nicht neu ausgeführt]. | `src/kv_shredding.rs` |

### 3.2 Fachlich gegenüber SPEC §11 bis §14

| ID | Befund | Beleg [V] |
|---|---|---|
| E-F10 | **Zwei Domänenmodelle.** `erasure_domain.rs` (Tenant/Collection/Subject/Document, `SubjectRef`, `DocRef`) wird im Crate nirgends benutzt (nur `lib.rs` deklariert das Modul). Die Saga arbeitet mit `types_local::EraseTarget { ExactKeys, KeyRange }`. Versions- und Betroffenen-Ziele (§11.2, §11.4) sind nicht ausdrückbar. | `grep erasure_domain` |
| E-F11 | **Weitere Dubletten über F-17 hinaus.** Doppelt definiert: `FenceGate`, `WalSink`, `KeyStore`, `ResidueScanner`, `VerifiedVia` (3 Stellen: `types_local`, `measure/field`, `vault/table`), sowie in `erasure_domain` und `types_local`: `EraseMode`, `EraseTarget`, `ErasureStatus`, `KeyCoverage`, `PendingReason`, `Phase`, `RefusalReason`, `RetentionCode`, `RetentionAttestation`, `Timing`, `EraseReceipt`, `EraseRequest` (auch `saga/machine.rs`), `LayerId`, `LayerWitness`. Der Crate-REPORT löst das teils mit Doppel-Impl und Aliassen (`MemoryFenceGate`). | `grep 'pub (trait|struct|enum)'` |
| E-F12 | **Reihenfolge entspricht dem vorgezogenen Pfad.** `saga/core.rs` vernichtet Schlüssel (Schritt 3) vor dem Purge-Fenster (Schritt 5). SPEC §11.4 nennt als **Standardpfad** L0, L1, L2 (Bereinigung), L3 (Schlüsselvernichtung); der Schlüssel-zuerst-Pfad ist die Ausnahme (besondere Datenklasse oder knappe Frist). Die Bindungsregel (Schlüssel erst vernichten, wenn keine lesbar bleibende Version ihn bindet) ist nicht erkennbar. | `src/saga/core.rs`, Kopfkommentar |
| E-F13 | **Mehrbetroffene fehlen.** Kein Trockenlauf, kein `DeleteAll`/`RedactSplit`/`RestrictShared`, kein `Restricted(MultiSubjectReview)`, keine 64er-Grenze. | `grep` leer |
| E-F14 | **Hierarchie weicht von §11.3 ab.** Vorhanden: T/C/S/D-Slots, `DEK = HKDF(K_unit, salt = SHA-256("ctx/dek-salt/v2" ‖ T ‖ C ‖ S ‖ D), info = "ctx/dek/v2" ‖ doc_tag)`, höchstens **ein** Betroffener. SPEC: `DEK(v) = HKDF(DocumentKey(v) ‖ SubjectKey(s1..sn), salt = VersionRef(v), info = "contextra/dek/v1" ‖ n)`; „zwei Versionen teilen nie einen DEK“. | `src/crypto_local.rs`, `src/vault/sealer.rs` |
| E-F15 | **Gate: O(log n) statt O(1).** `ErasedSet` ist ein sortierter Vec mit Intervallprüfung; Kommentar: „`check` ist O(log n)“. SPEC §11.4.2: O(1) ohne I/O. | `src/gate/erased_set.rs` |
| E-F16 | **Positivkontrolle prüft nicht den echten Scanner.** `run_positive_control` sucht ein Muster mit eigener Schleife in einem synthetischen Puffer. `Armed(pub ())`, `ControlEvidence(pub ())`, `ControlEvidence::observed()` und `Scanner::<Armed>::armed()` sind öffentlich; die Typestate lässt sich fälschen (Z-E10). | `src/measure/positive_control.rs`, `src/types_local.rs` Z. 731 bis 745, 902 |
| E-F17 | **Bericht kennt Klassen G/K/P, nicht die vier Stufen.** Felder `mk1..mk3`, „Klassenverband G, K, P“. SPEC §11.4: Gesperrt, Bereinigt, Schlüssel vernichtet, Vollständig gemessen. | `src/report/build.rs` |
| E-F18 | **Signaturpräfix `ctx/report/v1`** (3 Stellen) statt `contextra/report/v1` (SPEC §14.1). Änderung ist eine Formatänderung (INV-1). | `src/report/signing.rs` |
| E-F19 | **Restore-Guard ist Stub.** `FileRestoreChecker::check` prüft nur Zeitregression; Wiederauftauchen ist „Stub: kein Wiederauftauchen erkannt“. Kein Zustand `Quarantine`, keine Prüfung der Bedingungen (a) unabhängige Senke/Anker, (b) Sicherungshorizont. | `src/restore/guard.rs` |
| E-F20 | **Zeitgrenze.** `erase_return_timeout` Default 500 ms; SPEC §14.5: harte Grenze 2 s, p99 L1 < 50 ms. | `src/saga/core.rs` |
| E-F21 | **`ErasureCore` hängt an `async`.** `saga::core` und `ErasureCore` existieren nur mit Feature `async`. SPEC §4.1 / D-13: `async` nicht Default. Ohne Entscheidung ist der Orchestrator im Default-Build des SOLL-Zustands nicht vorhanden. | `src/saga.rs` |
| E-F22 | **Golden-Korpus unvollständig.** `tests/golden/MANIFEST.sha256` führt 1 Datei; es liegen außerdem `deletion_proof_v1/v2/v3.json` und `erasure_error_display.json` dort. `regen.sh` würde `MANIFEST` nur aus `*.json` neu erzeugen und den `.golden` verlieren. Für `CTXFILE`, `CTRALEDG`, `CTXSCOM`, `CTXSLOK` (SPEC §6.1) gibt es keine Korpusprüfung. `CTXFILE` wird in drei Dateiarten verwendet (Journal, Tresor, Scope-Datei). | `tests/golden/`, `src/**` |
| E-F23 | **Fehlertyp.** `ErasureError` ohne `#[non_exhaustive]` (SPEC F-10); `Refused` bedeutet „Retention aktiv“ und wird laut [A] Audit N9 auch für „fehlt/widerrufen“ genutzt; kein `Erased`. Doppelte Varianten `Crypto`/`CryptoError`. | `src/error.rs` |
| E-F24 | **Feature-Hygiene.** Default `signing, vault, async`; SPEC: `signing, aead`. Feature `vault` (9 `cfg`-Stellen) → `aead` (D-11). | `Cargo.toml` |
| E-F25 | **Kein Verbraucher im Workspace** (SPEC F-10, W13; Audit B4 [A]). Alle Crate-Tests laufen gegen Mocks. | SPEC |
| E-F00 | **Falscher Crate-REPORT** (siehe 2.1) und veraltetes crate-lokales `SPEC.md` (verweist auf „SPEC v4.0 §3.1“, Autarkie-Score 99/100, ca. 12.464 Zeilen; gemessen sind rund 22.900). | `docs/decoupling/REPORT.md`, `SPEC.md` |

### 3.3 Erfüllt (nicht ändern, nur schützen)

- E0: keine internen Abhängigkeiten; `forbid(unsafe_code)`; Zero-Panic-Lint (`cfg_attr(not(test), deny(...))`); keine `mod.rs`.
- Wirkende Bausteine: Saga-Zustandsmaschine mit Idempotenz und Pending-Regel; `PurgePhase` (9 Phasen, nur +1 oder idempotent); `FileVault` mit 16 Shards und Poisoning; Ledger mit Hash-Kette; Report-Kanonisierung; Epoch-Commitment (`journal/epoch.rs`).

---

## 4 SOLL-Architektur

### 4.1 Features (SPEC §4.1, R10)

| Feature | Zweck | Default |
|---|---|---|
| `signing` | Ed25519 (`ed25519-dalek`), Berichte, Revocation-Log | ja |
| `aead` | AEAD (`aes-gcm-siv`); ersetzt `vault`; `vault` bleibt Alias bis S-70 | ja |
| `async` | `tokio`, `futures-util`, Treiber | **nein** (D-13), siehe E-D05 |
| `conformance` | Konformitätssuite (zieht aus `testkit`/`ports` hierher, S-64) | nein |

Ziel-Default: `["signing", "aead"]`.

### 4.2 Besitzer je Konzept (R4)

| Konzept | Einziger Besitzer | Zu entfernen |
|---|---|---|
| Löschziel, Modus, Status, Gründe, Quittung | `erasure_domain` (neu geschnitten, siehe 5/E-1) | Dubletten in `types_local`, `saga::machine::EraseRequest` |
| Ports (`Clock`, `IdGen`, `Vfs`, `FenceGate`, `WalSink`, `KeyStore`, `ResidueScanner`) | `types_local` (E0-lokal, R7) | Dubletten in `gate/fence.rs`, `journal/wal_sink.rs`, `vault/key_store.rs`, `measure/residue.rs` (nur als Re-Export) |
| `VerifiedVia` | `vault/table.rs` | `types_local`, `measure/field.rs` |
| Schlüsselmaterial | neues Modul `keys` (E-3) | `Clone`-Impls |
| Fehler | `ErasureError` (`#[non_exhaustive]`) + `Erased`-Unterscheidung | Doppelvarianten |

### 4.3 Abbildung Löschkette L0 bis L6 auf Code **[D-Vorschlag]**

| Stufe (SPEC §11.4) | Saga (`SagaState`) | Purge (`PurgePhase`) | Modul | Lücke |
|---|---|---|---|---|
| L0 Anfrage | `Requested` | — | `saga::core`, `ledger` | Ziel-Typ Subject/Dokument/Version (E-F10) |
| L1 Sperre | `Gated` | — | `gate::*`, `journal` | Trockenlauf, Mehrbetroffene (E-F13), O(1) (E-F15) |
| L2 Bereinigung | `Purging` | `Admitted` bis `WalRetiring` (1 bis 6) | `purge::*`, `ErasureLayer` | Reihenfolge Standard/vorgezogen (E-F12) |
| L3 Schlüsselvernichtung | `KeyDestroyed` | — | `vault::*`, `kv_shredding` | Bedingungen 1 bis 4, Bindungsregel (E-F14) |
| L4 Rotation, Sicherungsfenster | **fehlt** | — | `vault::rehash`, `vault::keyring` | neuer Zustand oder Unterphase nötig; Ankernachweis |
| L5 Messung | `Verifying` | `Witnessing` (7) | `measure::*` | echte Positivkontrolle (E-F16), Canary vor Sperre |
| L6 Bericht | `Completed` | `Finalizing`, `Released` (8, 9) | `report::*`, `ledger` | Stufen (E-F17), Spiegel in Senke |

Erfüllungsstufen: Gesperrt = L1 fertig · Bereinigt = L2 gemessen + Kontrolle bestanden · Schlüssel vernichtet = L3 und L4 fertig, Fenster abgelaufen · Vollständig gemessen = L5 und L6 mit Kontrolle und bestätigter Senke. Jede Stufe nur `reached` mit bestandener Positivkontrolle, sonst `not_reached`; nicht implementierte Messung ergibt `NotImplemented`.

---

## 5 Umsetzungsplan

Reihenfolge ist zwingend, soweit „Voraussetzung“ gesetzt ist. Größen sind Schätzungen **[E]** (S ≤ 2 Tage, M ≤ 1 Woche, L > 1 Woche, eine Person). Jeder Schritt folgt INV-3 (Commit A verschiebt/benennt um, Commit B baut um) und der Definition of Done aus SPEC §9.1.

### E-0 Build, Korpus, ehrliche Berichte

| | |
|---|---|
| Ziel | Der Crate baut mit `--all-targets`, `cargo test` ist die Wahrheit, Berichte stimmen. SPEC: S-102 (Teil), F-18. |
| Voraussetzung | Klärung, wo Wurzel-`Cargo.toml`, `governance/`, `justfile` liegen (Risiko R-01). |
| Arbeit | 1. Die beiden Proof-Tests gegen die **aktuelle** API neu formulieren; Assertions inhaltlich beibehalten, nicht abschwächen, nicht löschen (INV-10 sinngemäß). Wo die alte API ein Soll ausdrückt, das der Code nicht hat, wird der Test `#[ignore = "E-xx"]` mit Verweis markiert, nicht entfernt. 2. `erasure` in `default-members`. 3. `tests/golden/MANIFEST.sha256` vollständig (alle Dateien), `regen.sh` so ändern, dass sie nichts verliert (`sha256sum -c` als Prüftest). 4. `docs/decoupling/REPORT.md`, `SPEC.md`, `PLAN.md` des Crates auf den gemessenen Stand bringen; Verweise „SPEC v4.0“ auf `docs/SPEC.md` v6. 5. Befunde E-F00 bis E-F25 ins Register (Anhang A) übertragen. |
| Abnahme | `cargo check -p contextra-erasure --all-targets --locked` und `cargo test -p contextra-erasure --locked` grün; G-L3 grün; `sha256sum -c` grün; keine Aussage im Crate-REPORT ohne Beleg. |
| Größe | M |

### E-1 Typkonsolidierung (SPEC S-101 erweitert, S-24)

| | |
|---|---|
| Ziel | Ein Besitzer je Typ (4.2). Domänenmodell trägt Subject/Dokument/Version. |
| Voraussetzung | E-0 |
| Arbeit | 1. Entscheidung E-D01 (siehe 7), welches Modell führt. Vorschlag: `erasure_domain` führt, `EraseTarget` wird `Tenant | Collection | Subject | Document | Version`; `{ExactKeys, KeyRange}` wird interne Schichtsicht (`LayerScope`). 2. Dubletten aus E-F11 zusammenführen; Re-Exports mit `#[deprecated(since, note)]` (R9, INV-2); das `#![allow(deprecated)]` in `erasure_domain.rs` entfällt. 3. `FenceGate`, `WalSink`, `KeyStore`, `ResidueScanner`: je ein Trait, die zweite Definition wird `pub use`. Die Doppel-Impl von `InMemoryFenceGate` entfällt. 4. `vault` → `aead` mit Alias; 9 `cfg`-Stellen; `async` aus Default (nach E-D05). 5. `#[non_exhaustive]` an `ErasureError` und öffentlichen Enums; `Crypto`/`CryptoError` zusammenlegen; Fehler `Erased` einführen und `Refused` auf Retention beschränken (E-F23). 6. Kein Format ändert sich (G-F1). |
| Abnahme | G-A1 ohne unbeabsichtigte Diffs; G-F1 unverändert grün; `grep 'pub trait (FenceGate|WalSink|KeyStore|ResidueScanner)'` je genau 1 Treffer; `cargo check` je Feature. |
| Größe | M |

### E-2 Krypto fail-closed (Z-E2, Z-E9)

| | |
|---|---|
| Ziel | Keine Fallbacks, keine Eigenbau-Chiffre, keine Nullschlüssel, fehlbarer Zufall. |
| Voraussetzung | E-1 |
| Arbeit | 1. Lokaler Port `Entropy` (R7): `try_fill(&self, &mut [u8]) -> Result<(), ErasureError>`; Standard-Impl über `OsRng::try_fill_bytes`; Fake mit Fehlerinjektion. Alle `fill_bytes`-Stellen (E-F05) umstellen. Ein Fehler erzeugt weder Umschlag noch Ersatzschlüssel. 2. Alle Fallbacks aus E-F01 und E-F04 entfernen; HKDF-Fehler und fehlende Slots ergeben `ErasureError` bzw. `Erased` und werden nie zu einem Schlüssel. 3. Neuer Umschlag **v3** mit AES-256-GCM-SIV (`aead`), Zufalls-Nonce je Umschlag, AAD = Domänenpräfix ‖ Mandant ‖ Collection ‖ Ziel ‖ Version, Längenauffüllung in Klassen (Bucket-Größen als Konstante, per ADR festzulegen). Lesepfad für v2 bleibt (INV-1). Schreiben nur noch v3. Tag-Vergleich entfällt (AEAD). 4. `VaultSealer::seal/open` und `seal_record/open_record` ersetzen; Journal-/Scope-Dateien tragen die Umschlagversion im Kopf. Migration von v2: beim nächsten Rewrite, nie in place (D-16-Geist). 5. `Clone` von Schlüsseltypen entfernen (E-F07); Zugriff über `expose(|k| …)`; `Zeroizing` auch im Entschlüsselungspfad. 6. `FileVault::new_in_memory*` nur noch mit explizitem Schlüssel; Nullschlüssel nur unter `#[cfg(test)]`. |
| Abnahme | Z-E9: Fehlerinjektion an Zufalls-, Ableitungs- und Siegelpfad ergibt jeweils Fehler und **keine** Datei und **keinen** Schlüssel. Z-E2: Canary-Bytes-Suche über alle erzeugten Dateien ohne Treffer; Test, dass gleiche Klartexte verschiedene Chiffretexte ergeben; Test, dass zwei Klartexte unterschiedlicher Länge im selben Bucket gleich lang sind. Test, dass ein Dokument nach `destroy` auch bei vorhandenem `k_unit` nicht entschlüsselbar ist (Regression zu E-F01). Mutation-Score der Zieldateien ≥ Baseline. |
| Größe | L |
| Risiko | Formatänderung → Versionsanhebung mit Golden-Dateien v2 und v3 (INV-1, §6.2). |

### E-3 Schlüsselhierarchie nach §11.3

| | |
|---|---|
| Ziel | `SubjectKey`, `DocumentKey(v)`, `DEK(v)` wie spezifiziert; Vernichtung nach den vier Bedingungen. |
| Voraussetzung | E-2, Entscheidungen E-D01, E-D02 |
| Arbeit | 1. Neues Modul `keys`: `RootKey(generation)`, `SubjectKey`, `DocumentKey`, `Dek`, alle `ZeroizeOnDrop`, ohne `Clone`/`Debug`-Inhalt/`Serialize`. 2. `DEK(v) = HKDF-SHA256(IKM = DocumentKey(v) ‖ SubjectKey(s1) ‖ … ‖ SubjectKey(sn), salt = VersionRef(v), info = "contextra/dek/v1" ‖ n)`. Sortierung der Subjects kanonisch (aufsteigend nach Referenz). Kein DEK-Cache über Vernichtung hinaus; Cache-Invalidierung bei `destroy`. 3. Liste der Betroffenen verschlüsselt im Dokumentkopf. 4. Neue Version bei Berichtigung, Hinzufügen oder Schwärzung → neuer `DocumentKey`. 5. Tresoreintrag: `destroy` ersetzt den Eintrag durch Grabstein, synchronisiert, rotiert die Tresordatei der Generation und vernichtet den Wurzelschlüssel; erst danach `Schlüssel vernichtet` (Bedingung 1 bis 3); Bedingung 4 (Sicherungsexporte abgelaufen) kommt aus E-8/Ledger. 6. **Bindungsregel**: `destroy(key)` prüft, ob eine lesbar bleibende Version den Schlüssel bindet; wenn ja Fehler `StillBound`, und die Version wird vorher als neue Version ohne Bindung abgelegt. 7. Persistente Wurzel gemäß E-D02: `RootKey` aus dem Tresor, nie zufällig pro Prozessstart; Start ohne Wurzel ist ein Fehler, kein Neuanlegen (Audit N1). 8. Epochen-Rollover für Gruppen (Audit N11): neue `group_id`-Epoche nach Drop. 9. `kv_shredding::revoke_record(&str)` entfernen oder auf die numerische Variante umleiten (Audit N2). |
| Abnahme | Tests: Zwei Versionen haben nie denselben DEK; Vernichtung eines Subjects macht jede gebundene Version unlesbar; Vernichtung nur der Version lässt den Subject-Schlüssel gelten; `StillBound`; Neustart liest persistierte Wurzel; Crash zwischen Grabstein und Rotation ist wiederholbar. Differenz- und Golden-Tests für `contextra/dek/v1` (Referenzvektoren aus RFC 5869 plus eigene). |
| Größe | L |

### E-4 Saga L0 bis L6 und Mehrbetroffene

| | |
|---|---|
| Ziel | Die Kette aus 4.3 mit Standard- und vorgezogenem Pfad, Trockenlauf, Modi, fsync-Fehlerpfad. |
| Voraussetzung | E-1, E-3 |
| Arbeit | 1. Reihenfolge: Standard L0 → L1 → L2 → L3 → L4 → L5 → L6; vorgezogen L0 → L1 → L3 → L2 → … (nur bei besonderer Datenklasse oder knapper Frist, Konfiguration, nie Default). Bisheriger Ablauf (Schlüssel vor Purge) wird zum vorgezogenen Pfad. 2. Zustand für L4 (z. B. `KeyRotating` oder Unterphase); `SagaState::can_transition_to` und `PurgePhase` bleiben streng monoton, `Pending` springt nie direkt zu `Completed`. 3. Trockenlauf über alle Versionen einschließlich Vorgängerversionen innerhalb `erase_return_timeout` (2 s, harte Grenze; Konfiguration validiert, Default 2 s). 4. Modi `DeleteAll` (nur ohne Konflikt), `RedactSplit` (neue Version ohne den Betroffenen), `RestrictShared`; Konfliktversionen sofort `Restricted(MultiSubjectReview)`; **kein** automatisches `DeleteAll` nach Fristablauf (Z-E3); Obergrenze 64 Betroffene je Version, darüber Pflicht `RedactSplit`. 5. `FsyncFailed`: Datei nie erneut als stabil annehmen, Inhalt aus Journal/Speicher in neue Datei schreiben, bei Misserfolg Bereich vergiften (§11.4.4). 6. Physische Tatsachen haben beim Recovery Vorrang vor dem Journalstand (§11.4.3). 7. Idempotenz je Schritt (`derive_open_key` auf neues Ziel-Modell anpassen; k_idx darf nicht `[0;32]` Default sein, E-F06-Muster). |
| Abnahme | Lebenszyklustest je Zeile L0 bis L6 (G-ER2); Trockenlauf-Tests je Modus (Z-E3); Absturz an jedem Durabilitätspunkt (Z-E4, E-9). Test: Rückkehr von L1 für eine Person mit 1.000 Versionen unter 50 ms p99 (Messung E-9). |
| Größe | L |

### E-5 Gate und Lesesperre

| | |
|---|---|
| Ziel | Nach L1 liefert kein Lesepfad Klartext (Z-E1), Prüfung vor Entschlüsselung, O(1). |
| Voraussetzung | E-1; Entscheidung E-D03 |
| Arbeit | 1. Zweistufiges Gate: Hash-Menge für exakte Ziel-/Versionsschlüssel (O(1) erwartet) plus Intervallstruktur für Bereichsziele; oder SPEC-Änderung auf O(log n) per ADR. 2. `Erased` für Berechtigte, **gleiches Fehlerbild wie „nicht vorhanden“** für Unberechtigte; Entschlüsselungsfehler nach außen einheitlich (`01_schluesselmodell.md` §1.3; Audit S6/X8: Fehlertexte verraten den Verschlüsselungszustand). 3. Prüfpunkte als Trait `ReadGuard`, damit Verbraucher (Suche, Antwort, Zeitreise, Simulation, Cache, Export, Vorschau, Sitzung) denselben Aufruf nutzen. 4. Schreibzugriffe auf gesperrte Ziele verweigern. 5. Z-E8: Rangfolge, Zählwerte und Fehlerbilder ändern sich nicht durch unzugängliche Dokumente (Vorgabe für Verbraucher, Konformitätstest im Crate). |
| Abnahme | `loom`-Linearisierungstest (`tests/loom_gate.rs` erweitern); Test je Lesepfad-Kategorie über die Konformitätssuite; Benchmark O(1) belegt (kein I/O im Pfad, per `FaultVfs`, der bei Zugriff scheitert). |
| Größe | M |

### E-6 Messung und Positivkontrolle

| | |
|---|---|
| Ziel | Z-E10: Nachweise nur mit Positivkontrolle am echten Scannerpfad. |
| Voraussetzung | E-1 |
| Arbeit | 1. `Armed`, `ControlEvidence` und `Scanner::armed` nicht mehr öffentlich konstruierbar (`pub(crate)`/privates Feld); `arm()` nur nach `run_positive_control` über **denselben** Scanner-Code wie der Echtscan (Dateien/Segmente über `Vfs`). 2. Canary-Dokument eines synthetischen Betroffenen wird vor L1 über denselben Pfad (Insert, Flush, Compaction) geschrieben; Negativmessung danach über alle Dateien. 3. Jedes nicht implementierte Verfahren liefert `NotImplemented` und nie `reached` (Prüfung per Test über alle `WitnessLayer`). 4. Messprotokoll enthält Zählwerte, keine Kennungen. |
| Abnahme | Test mit absichtlich verstümmeltem Scanner (liefert immer 0 Treffer) muss scheitern; Test, dass `Armed` außerhalb des Crates nicht erzeugbar ist (Compile-Fail, `trybuild`, falls als Dev-Abhängigkeit zulässig, sonst `doc`-compile_fail). |
| Größe | M |

### E-7 Bericht, Ledger, Pseudonyme

| | |
|---|---|
| Ziel | Vier Stufen, Domänenpräfix nach SPEC, keine stabilen Pseudonyme (Z-E7, D-31). |
| Voraussetzung | E-4, E-6; Entscheidungen E-D04, E-D06 |
| Arbeit | 1. Berichtsschema **v2**: Stufen `gesperrt`, `bereinigt`, `schluessel_vernichtet`, `vollstaendig_gemessen` je `reached`/`not_reached` mit Zeitpunkt, Messverfahren, Positivkontrolle, Zahl der Mitbetroffenen, Annahmen, Restrisiken. 2. Signaturpräfix `contextra/report/v1` im neuen Schema; Prüfpfad für `ctx/report/v1`-Berichte bleibt. 3. Ein Bericht gilt erst mit Ledger-Eintrag (§14.1). 4. Spiegelung in die Registersenke (Trait `RegistrySink`, E0-lokal); Stufe „Vollständig gemessen“ nur mit bestätigter Senke. 5. Commitments: epochengebunden (`journal/epoch.rs` nutzen), Anonymisierung nach Ablauf; Logs, Ledger, Metriken und Fehlertexte ohne Rohkennungen (Prüfung, wo `SubjectRef(Vec<u8>)` in `Debug`/`Display`/Logs landet). 6. Wortfilter: Berichtstexte nur Stufenbezeichnungen mit Messergebnis, nie „gelöscht“/„garantiert“ (INV-7, G-DOC1). |
| Abnahme | Golden `canonical_report_v2`; Rückwärts-Lesetest v1; Canary-Suche über alle Ausgaben (Z-E7); `doc_lint` grün. |
| Größe | M |

### E-8 Restore-Schutz

| | |
|---|---|
| Ziel | Z-E11 und §11.4.5. |
| Voraussetzung | E-4, E-7 |
| Arbeit | 1. Zustand `Quarantine`: Knoten liefert keine Daten, wenn Bedingung (a) oder (b) nicht erfüllt ist. 2. `FileRestoreChecker` mit echtem `RangeProbe`-Port (Stub entfernen); Ankerstand und Registersenke als Eingaben. 3. Beim Restore werden alle Sperren und Grabsteine erneut angewendet (`restore::replay`). 4. Sicherungsexporte unter Epochenschlüssel mit `T_backup`; Ablauf ist Eingang für Bedingung 4 in E-3. |
| Abnahme | Restore-Tests: mit Senke, ohne Senke, außerhalb des Sicherungshorizonts, Wiederauftauchen erkannt. |
| Größe | M |

### E-9 Nachweise und Gates

| | |
|---|---|
| Ziel | G-ER1 bis G-ER3 und die Zusagen sind automatisiert belegt (SPEC §14, §15 S-103, S-109). |
| Voraussetzung | E-4 bis E-8 |
| Arbeit | 1. **Crash-Matrix:** je Durabilitätspunkt L0 bis L6 und je Gate-/Tresorschreibstelle ein Szenario mit `FaultVfs`; Invarianten I1 bis I6 (7.2). 2. **Golden-Korpus** für `CTXFILE` (drei Arten, mit `file_kind` unterscheidbar), `CTRALEDG`, `CTXSCOM`, `CTXSLOK`, Umschlag v2/v3, Bericht v1/v2; Wurzel-Registrierung unter `tests/golden/` (S-02). 3. **Rückverfolgbarkeits-Matrix** Z-E1 bis Z-E12 → Testname (7.1); G-ER3 prüft per Skript, dass jede Zusage ≥ 1 grünen Test hat. 4. `T-VENDOR-BUNDLE` (Z-E12): lokal erzeugtes Supportpaket enthält nachweislich keine Inhalte, Schlüssel oder Kennungen; Test auf Netzwerkfreiheit (keine Socket-/DNS-Abhängigkeit im Kern). 5. **Leistungsmessung** §14.5 (`benches/bm_erase.rs` erweitern): L1 p99, „Bereinigt“ für 1.000 Dokumente, Obergrenze 24 h als dokumentiertes Ziel. 6. Konformitätssuite für Verbraucher (`conformance`-Feature). 7. Eingabe für `contextra-verify` (S-110): kanonische Berichte und Ledger-Ketten als Testvektoren. |
| Abnahme | G-ER1, G-ER2, G-ER3 grün; Messbericht nach SPEC §8.3 mit HEAD-Hash und Datum. |
| Größe | L |

### 5.1 Abhängigkeitsgraph und Wellen

```
E-0 ─► E-1 ─┬─► E-2 ─► E-3 ─► E-4 ─┬─► E-7 ─► E-8 ─┐
            ├─► E-5 ───────────────┤              ├─► E-9
            └─► E-6 ───────────────┘──────────────┘
```

| Welle | Schritte | Ergebnis |
|---|---|---|
| 0 | E-0 | Ehrlicher Build und Korpus |
| 1 | E-1, danach E-5 und E-6 parallel zu E-2 | Eine Typwelt; Gate und Messung belastbar |
| 2 | E-2, E-3 | Sichere Krypto und Hierarchie |
| 3 | E-4 | Vollständige Löschkette |
| 4 | E-7, E-8 | Berichte, Restore |
| 5 | E-9 | Nachweise |

---

## 6 Gesamt-Definition of Done (zusätzlich zu SPEC §9.1)

1. Kein Befund E-F00 bis E-F25 steht offen oder ist als `Won't fix` mit ADR begründet.
2. `cargo check --workspace --all-targets --locked`, `cargo test -p contextra-erasure --locked` und `cargo hack check --each-feature -p contextra-erasure` sind grün (INV-4, G-L3).
3. Gates GL, GH, GF, GD, GE, GC sowie G-ER1 bis G-ER3 sind grün und blockierend (D-23); neue Gates stehen nicht in `not_run_baseline`.
4. Jede Zusage Z-E1 bis Z-E12 hat einen benannten grünen Test (7.1).
5. Kein Fallback-Schlüssel, kein Standardschlüssel, kein nicht fehlbarer Zufall im Produktionscode; per `grep`-Gate (`erasure_leak_prober.py` oder neues Skript) abgesichert.
6. Crate-lokale `README.md`, `SPEC.md`, `PLAN.md`, `REPORT.md` und `capabilities.fragment.toml` nennen nur gemessene Werte; `examples/` mit lauffähigem Beispiel nur mit dem Kern (SPEC §5.4).
7. Entnehmbarkeitsnachweis nach SPEC §8.4 (G-L2, G-L1, G-L3, G-L4, G-H1 in dieser Reihenfolge grün; `extractable` mit `verified_at` gesetzt).
8. Keine Produktaussage verletzt INV-7.

---

## 7 Nachweis-Matrizen

### 7.1 Zusagen → Tests (Soll; Namen sind Vorschläge)

| Zusage | Prüft | Vorgesehener Test (neu oder zu erweitern) | Phase |
|---|---|---|---|
| Z-E1 | Kein Klartext nach L1, auch `Superseded` | `z_e1_read_paths_*`, `loom_gate` erweitern | E-5 |
| Z-E2 | AEAD, keine Pseudonyme, keine exakten Längen | `z_e2_canary_bytes_scan`, `z_e2_bucketed_lengths`, `z_e2_random_nonce` | E-2 |
| Z-E3 | Kein `DeleteAll` bei Konflikt/Timeout | `z_e3_dry_run_modes` | E-4 |
| Z-E4 | Crash-Invarianz L0 bis L6 | `crash_matrix_l0..l6` (bestehende `crash_matrix_recovery.rs` erweitern) | E-9 |
| Z-E5 | K ohne Personendaten | liegt in `rules`/G-PII1; hier nur `PersonalRef`-Nutzung | S-106 |
| Z-E6 | Löschung ändert Versionsgraph nicht | Vorher-Nachher-Test mit `rules` | E-9 (mit S-104) |
| Z-E7 | Keine Rohkennungen in Logs/Ledger/Metriken/Fehlern | `z_e7_canary_in_all_outputs` | E-7 |
| Z-E8 | Zugriff vor Suche/Statistik | Konformitätstest `conformance::access_before_rank` | E-5 |
| Z-E9 | Fail-closed | `z_e9_fault_injection_{entropy,hkdf,seal}` | E-2 |
| Z-E10 | Positivkontrolle | `z_e10_garbled_scanner_must_fail` | E-6 |
| Z-E11 | Restore-Schutz bedingt | `z_e11_restore_{with_sink,without_sink,out_of_horizon}` | E-8 |
| Z-E12 | Anbieterneutral | `T-VENDOR-BUNDLE` | E-9 |

### 7.2 Crash-Invarianten (SPEC §14.2)

I1 durable Sperre geht nie verloren · I2 kein Klartext eines gesperrten Ziels · I3 Schlüssel geht nie ohne Absicht verloren · I4 Saga vollendet oder `Pending` bei aktiver Sperre · I5 Ledger, Manifest, Gate-Log konsistent · I6 Ordinale nie wiederverwendet.

### 7.3 Bestehende Tests, die erhalten bleiben müssen

Alle 216 grünen Tests (42 Targets) dürfen nur geändert werden, wenn ihr Soll sich durch E-1 bis E-8 begründet ändert; die Testanzahl vorher und nachher wird je Schritt berichtet (wie S-12b). Tests, die das fehlerhafte Verhalten aus 3.1 festschreiben, werden durch Tests ersetzt, nicht gelöscht, und im Commit begründet.

---

## 8 Offene Entscheidungen (ADR in `docs/decisions/`)

| ID | Frage | Vorschlag | Blockiert |
|---|---|---|---|
| E-D01 | Welches Domänenmodell führt (`erasure_domain` vs `types_local::EraseTarget`)? | `erasure_domain` führt; Bereichsziele werden interne Schichtsicht | E-1, E-3, E-4 |
| E-D02 | Quelle und Persistenz des Wurzelschlüssels (Audit 4.1) | Passphrase + Argon2id (Parameter festlegen) als Standard, OS-Keyring/TPM als optionales Plugin; Air-Gap-tauglich. Verlust = Totalverlust, dokumentieren | E-3 |
| E-D03 | Gate O(1) oder SPEC auf O(log n) ändern | Hash-Menge für exakte Schlüssel plus Intervallstruktur | E-5 |
| E-D04 | Berichtsformat: Domänenpräfix `ctx/report/v1` → `contextra/report/v1` | neues Schema v2, alter Prüfpfad bleibt | E-7 |
| E-D05 | `ErasureCore` ohne `async` bereitstellen (synchroner Kern + optionaler Async-Treiber) oder `async` begründet im Default | synchroner Kern, `async` nur Treiber | E-1 |
| E-D06 | Dauer der epochengebundenen Commitments (SPEC D-31 offen) | Betreiberkonfiguration mit Mindestwert; Begründung im Löschkonzept | E-7 |
| E-D07 | Verdrahtung von `erasure` in `db`/`store`/`engine`/Indizes (SPEC §15 plant keine Schritte) | eigener Schritt-Block S-112 ff. nach E-5; ohne ihn sind Z-E1, Z-E2 und Z-E8 nur auf Crate-Ebene belegt | Gesamtabnahme |
| E-D08 | Bucket-Größen für Längenauffüllung (Z-E2) | Zweierpotenzen ab 256 Byte; Overhead messen | E-2 |
| E-D09 | Rohkennung `SubjectRef(Vec<u8>)`: Verwendung nur als Handle (`PersonalRef`) | `SubjectRef` wird typisierter Handle ohne `Debug`-Inhalt | E-1, E-7 |
| E-D10 | `ControlEvidence`/`Armed` nicht öffentlich: Auswirkung auf `testkit`-Konformitätssuite | Testfabrik hinter `conformance`-Feature | E-6 |

---

## 9 Risiken und Rückfallverfahren

| ID | Risiko | Gegenmaßnahme |
|---|---|---|
| R-01 | Wurzel-`Cargo.toml`, `governance/`, `justfile`, `.github/` fehlen im Klon; `[lints] workspace = true` ist hier nicht auswertbar. | Vor E-0 klären, ob der Klon unvollständig ist oder die Dateien in einem anderen Branch liegen. Ohne sie sind G-L1 bis G-L4 und G-H1 für diesen Plan nicht prüfbar. |
| R-02 | Umschlag v3 macht Altdaten unlesbar, wenn der Lesepfad für v2 fehlt. | Lesepfad v2 bleibt; Golden v2/v3; Reihenfolge erst Leser, dann Schreiber (SPEC §6.2). |
| R-03 | Entfernen der Fallbacks deckt Tests auf, die sich auf sie stützen. | Tests werden nicht abgeschwächt, sondern gegen die korrekte Semantik neu geschrieben. Rückfall nach SPEC §9.5 Punkt 2 (B per `git revert`, A bleibt). |
| R-04 | Persistente Wurzel ohne Wiederherstellungsweg verursacht Totalverlust. | E-D02 vor Implementierung; Neuanlegen ohne Wurzel ist Fehler; Dokumentation im Löschkonzept. |
| R-05 | Mehrbetroffene Dokumente: automatische Entscheidung zulasten Dritter. | D-29: Trockenlauf Pflicht, kein Timeout-`DeleteAll`, Review. |
| R-06 | Zählungen und Messwerte dieses Dokuments altern. | Nur mit HEAD-Hash und Datum (D-15); Ersatz durch generierten Bericht (SPEC §8.3). |
| R-07 | Audit-Aussagen [A] (N1, N2, N9) wurden hier nicht neu ausgeführt. | In E-0 per Spike-Test (`spike_keys`-Szenarien) nachvollziehen und als Regressionstests anlegen. |
| R-08 | `ErasureCore`-Pfad (`async`) wurde hier nur kompiliert und in den vorhandenen Tests ausgeführt, nicht unter Last oder Crash. | E-9, Crash-Matrix. |

Rückfall bei roten Gates, Differenztests und Format-Hashes: SPEC §9.5 Punkte 2 bis 4 gelten unverändert.

---

## Anhang A Reproduktion der Messungen

```bash
# Toolchain
apt-get install -y rustc-1.89 cargo-1.89
export PATH=/usr/lib/rust-1.89/bin:$PATH CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0

# Ersatz-Workspace (nur weil die Wurzel-Cargo.toml im Klon fehlt)
mkdir -p /tmp/ws/crates && cp -r crates/contextra-erasure /tmp/ws/crates/ && cp Cargo.lock /tmp/ws/
# /tmp/ws/Cargo.toml: [workspace] members = ["crates/contextra-erasure"];
# [workspace.package] version="0.1.0", edition="2021", license="MIT OR Apache-2.0", rust-version="1.89";
# [workspace.dependencies] ahash=0.8, thiserror=2, tracing=0.1, serde (derive), serde_json, bytes, parking_lot=0.12,
#   tokio (rt, sync, time, macros, rt-multi-thread), tempfile=3; [workspace.lints.rust] leer

cargo check -p contextra-erasure --no-default-features
for f in signing vault async signing,vault signing,async vault,async; do
  cargo check -p contextra-erasure --no-default-features --features "$f"; done
cargo check -p contextra-erasure --all-targets --keep-going     # 78 Fehler
# Tests ohne die beiden Proof-Targets: --lib plus --test <name> je Datei in tests/
cargo test -p contextra-erasure --no-fail-fast --lib --test <name> ...   # 216 bestanden
```

Strukturabfragen:
```bash
grep -rn -E 'pub (trait|struct|enum) (FenceGate|WalSink|KeyStore|ResidueScanner|VerifiedVia|LayerId|EraseTarget)\b' crates/contextra-erasure/src
grep -rn 'erasure_domain' crates/contextra-erasure/src crates/contextra-erasure/tests
grep -rn -E 'OsRng|fill_bytes' crates/contextra-erasure/src
grep -rn -E 'blake3::keyed_hash' crates/contextra-erasure/src/{crypto_local.rs,vault}
```

## Anhang B Betroffene Dateien (Übersicht)

| Phase | Dateien (Auszug) |
|---|---|
| E-0 | `tests/proof_crash_resume_matrix.rs`, `tests/proof_fail_closed_and_consistency.rs`, `tests/golden/*`, `docs/decoupling/REPORT.md`, `SPEC.md`, `PLAN.md`, `Cargo.toml` (Wurzel) |
| E-1 | `src/erasure_domain.rs`, `src/types_local.rs`, `src/gate/fence.rs`, `src/journal/wal_sink.rs`, `src/vault/key_store.rs`, `src/measure/residue.rs`, `src/measure/field.rs`, `src/vault/table.rs`, `src/saga/machine.rs`, `src/error.rs`, `Cargo.toml` |
| E-2 | `src/crypto_local.rs`, `src/vault/sealer.rs`, `src/vault/keyring.rs`, `src/vault/slot.rs`, `src/journal/appendlog.rs`, `src/scope/file.rs`, `src/kv_shredding.rs`, `src/ed25519_proof.rs`, `src/vault.rs` |
| E-3 | neu `src/keys.rs`, `src/vault/*`, `src/kv_shredding.rs`, `src/revocation_log.rs` |
| E-4 | `src/saga/*`, `src/purge/*`, `src/orchestrator.rs`, `src/restricted.rs`, `src/replace.rs`, `src/supersede.rs` |
| E-5 | `src/gate/*`, `tests/loom_gate.rs`, `tests/gate_linearization_test.rs` |
| E-6 | `src/measure/*`, `src/types_local.rs` |
| E-7 | `src/report/*`, `src/ledger/*`, `src/journal/epoch.rs` |
| E-8 | `src/restore/*` |
| E-9 | `tests/*`, `benches/bm_erase.rs`, `tests/golden/*`, neu `tests/z_e*.rs` |
