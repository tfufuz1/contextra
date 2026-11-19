# Löschkette: Spezifikations- und Umsetzungsprüfung

**Repository:** `tfufuz1/contextra`, HEAD `deeddfd11` (19.11.2026)
**Geprüfte Dokumente:**
- `SPEC.md` v2.0 (identisch mit `docs/SPEC.md` im Repo)
- `Contextra_Spezifikation_v3_0_Loescharchitektur.md`
- `Contextra_Spezifikation_v3_1_Loescharchitektur.md`
- `Contextra___Implementierungsspezifikation_IST___SOLL__...md` (bezieht sich auf Commit `a547a42`)
- `docs/crates/*.md` (normativ pro Crate, Rangfolge 2)

**Methode:** Statische Analyse. Quellcode, Manifeste, Git-Historie und Dateiinhalte wurden gelesen und ausgewertet. Es wurde **nicht** gebaut, getestet oder mit Clippy geprüft, weil keine Rust-Toolchain verfügbar ist. Alle Aussagen sind mit Befehlen aus Abschnitt 10 reproduzierbar.

---

## 1. Ergebnis in Kürze

Die Löschkette ist als Zustandsautomat im Repo weitgehend ausgearbeitet. Die Zustandslogik der Saga (Gate vor Vernichtung, durables Persistieren vor jedem Übergang, idempotenter Purge, Eskalation Restricted→Gated) ist sauber umgesetzt. Die Garantie selbst trägt aber aus fünf Gründen nicht:

1. **Das Schlüsselmodell kann die Garantie nicht liefern.** DocumentDEK und SubjectKey werden per HKDF aus dem CollectionKEK *abgeleitet* (`crates/contextra-crypto/src/kdf.rs:416, 433`). Ein Schlüssel, der sich aus einem noch lebenden Schlüssel neu berechnen lässt, ist nicht vernichtbar. Die O(1)-Löschung eines Betroffenen (SPEC §5.6, T-DEL-9) ist damit nicht umsetzbar.
2. **Die Löschkette ist nicht erreichbar.** Die Saga (`contextra-erasure/src/chain/saga.rs`) wird nur in Tests konstruiert. Keine einzige produktive `ErasureLayer`-Implementierung existiert. Das öffentliche `Collection::delete` setzt nur einen Tombstone.
3. **Die Messung ist eine Attrappe.** Die Positivkontrolle liest einen Trait-Boolean. `ControlEvidence::observed()` wird unbedingt konstruiert und ist in `contextra-types` öffentlich aufrufbar. Damit ist INV-WITNESS-MINT-1 gebrochen.
4. **Die Testbasis ist kaputt.** 266 Rust-Dateien enthalten nur das Wort `DELETED`. Davon sind 39 von 43 Test-Dateien der Crate `contextra-erasure` und 122 Test-Dateien der Crate `contextra-store`. Cargo bindet Testdateien automatisch ein, ein Bare-Identifier auf Top-Level ist kein gültiges Rust. `cargo test` für diese Crates kann daher nicht bauen.
5. **Die Spezifikationen widersprechen sich in zentralen Punkten.** Die normative Reihenfolge (Schlüsselvernichtung vor oder nach Purge), die Reversibilität des Gates und das Schlüsselmodell unterscheiden sich zwischen SPEC v2.0, v3.0, v3.1 und dem IST→SOLL-Plan. Es gibt drei konkurrierende Baselines.

**Empfehlung in einem Satz:** Schlüsselmodell aus v3.1 übernehmen (zufällige, gewrappte DEKs in einer KeyTable und verschlüsselte Index-Artefakte mit ArtKey), die Reihenfolge „Ebene-A-Bereinigung und Messung → Schlüsselvernichtung → Ebene-B-Hygiene" festlegen und die Verdrahtung vor jeder weiteren Feature-Arbeit herstellen. Die Automatenlogik des Codes kann dabei weitgehend bleiben.

---

## 2. Grenzen der Prüfung

- Kein `cargo build`, `cargo test` oder `cargo clippy`. Ein Download der Rust-Toolchain wurde nicht erfolgreich durchgeführt (keine Antwort von `static.rust-lang.org`). Die Kompilierbarkeit des übrigen Workspace ist daher **ungeprüft**.
- Die Modul-Erreichbarkeit wurde über `mod`-Deklarationen ermittelt. `#[path]`-Attribute wurden nicht ausgewertet.
- Die IST→SOLL-Datei bezieht sich auf `a547a42` (26.10.2026). Mehrere ihrer Befunde gelten am HEAD nicht mehr. Beispiel: B-21 („`Collection::delete` gibt den Key-Lock vorzeitig frei") ist am HEAD nicht bestätigt. Der Lock wird erst nach `commit()` freigegeben (`crud/delete.rs:16–22`). Die IST-Befunde müssen vor einer Umsetzung neu verifiziert werden.
- Juristische Aussagen sind Sache des Verantwortlichen (INV-7, SPEC §1.2). Die Punkte P-xx aus v3.1 sind fachanwaltlich zu prüfen.

---

## 3. Kernbefund: Abgeleitete Schlüssel tragen keine Löschgarantie

### 3.1 Sachverhalt

| Ebene | Spezifikation (SPEC v2.0 §4.1) | Implementierung (`kdf.rs`) |
|---|---|---|
| DocumentDEK | HKDF aus CollectionKEK(collection_id, epoch) | `derive_document_dek(coll_kek, doc_id, version)`, Zeile 416 |
| SubjectKey | HKDF aus CollectionKEK | `derive_subject_key(coll_kek, subject_ref)`, Zeile 433 |
| SubjectDocDEK | HKDF aus SubjectKey | `derive_subject_doc_dek(...)` |

Die Vernichtung wird aber an einzelnen Schlüsseln beschrieben:

- SPEC §3.2 Regel 2: „Vernichtung des `SubjectKey` macht alle Daten dieses Betroffenen unlesbar — in O(1)."
- SPEC §5.1: Einzeldokument → `DocumentDEK`-Vernichtung.
- `docs/crates/contextra-crypto.md:18`: „Atomare Nullung (`zeroize`) von DEKs und SubjectKeys für den DSGVO-Löschpfad."

**Folge:** Solange der CollectionKEK existiert, lässt sich jeder abgeleitete Schlüssel jederzeit neu berechnen. Das Nullen eines Speicherslots eines abgeleiteten SubjectKey wirkt nicht. Die Vernichtung eines einzelnen DocumentDEK ist strukturell unmöglich. Die einzige wirksame Operation wäre die Vernichtung des CollectionKEK, und die betrifft alle Dokumente der Collection.

Die Spezifikation widerspricht sich zusätzlich selbst: §3.1 schreibt für Klasse K „Löschbar pro Betroffenen: Nein (nur Collection-Drop)", §5.1 verlangt Einzeldokument-Löschung für K.

### 3.2 Lösung

v3.1 §4.5 und §4.8 lösen das Problem: DocDeks werden **zufällig** erzeugt, mit dem KEK gewrappt und in einer **KeyTable** mit Ersetzen-Semantik abgelegt. Dann ist die Vernichtung eines Eintrags mit Überschreiben, fsync und atomarem Ersetzen der KeyTable real. v3.0 §5.3 beschreibt denselben Mechanismus. Die Spezifikation SPEC v2.0 muss hier umgestellt werden. Das Crate-Dokument `contextra-crypto.md` widerspricht sich ebenfalls und muss mit angepasst werden.

---

## 4. Widersprüche und Fehler innerhalb der SPEC v2.0

Schwere: **K** = kritisch (Garantie bricht), **H** = hoch (Umsetzung nicht eindeutig oder falsch), **M** = mittel, **N** = niedrig.

| ID | Schwere | Widerspruch / Fehler | Stellen | Vorschlag |
|---|---|---|---|---|
| S-01 | K | Abgeleitete DEK/SubjectKey vs. O(1)-Vernichtung und Einzeldokument-Löschung | §3.2(2), §4.1, §5.1, §5.6, T-DEL-9 | Zufällige, gewrappte Schlüssel in KeyTable (v3.1 §4.5) |
| S-02 | K | Wo liegen Schlüssel? §5.3 Schritt 3 verlangt „Vault", §2.1 schließt `vault` aus und verbietet die Reaktivierung, §16 definiert Vault als Modul in `erasure`. Die KeyTable/Keyring ist nirgends im Hierarchiebaum §4.1 | §2.1, §5.3, §16 | Begriff und Ort festlegen (ein Modul, ein Name) |
| S-03 | K | D-Klasse hat **keine Verschlüsselungsregel**. Abgeleitete Daten (Embeddings, Postings, Graph) sind nach Schlüsselvernichtung lesbar, bis Schritt 4 greift. Klasse K ist für D nicht definiert. T-DEL-1 verlangt dagegen „alle persistierten Bytes verschlüsselt" | §3.1, §5.3 Schritt 4(b), §5.4, T-DEL-1 | ArtKey je Index-Artefakt (v3.1 §4.7), D-Klasse explizit verschlüsseln |
| S-04 | K | **Positivkontrolle widerspricht der Verschlüsselung.** Der Canary wird über den regulären, verschlüsselten Schreibpfad eingefügt. Ein Rohdatei-Scan findet ihn dann nie. Gleichzeitig verlangt T-DEL-1, dass kein Klartext-Canary in Dateien liegt. Das Scan-Medium ist nicht definiert | §5.3 Schritt 5.2, T-DEL-1, T-DEL-7 | Scan-Medium definieren (Ebene A im RAM oder entschlüsselte Sicht). Positivkontrolle auf Ebene A ausführen, nicht auf Chiffrat-Dateien |
| S-05 | K | **Reversibilität des Gates.** INV-10 („Gate wird nie zurückgenommen") und „Gated: reversibel Nein" stehen gegen v3.1 §6.3 („Unfreeze bis Schritt 5"). Siehe Abschnitt 5 | §5.2, INV-10, v3.1 §6.3 | Restricted = reversibel (Art. 18), Gated = irreversibel. Kein Unfreeze nach Gate |
| S-06 | K | **Crash-Idempotenz von Schritt 3.** Nach Nullung des Slots, aber vor `KeyRevoked`, findet der Neustart den Schlüssel nicht mehr. §5.5 sagt „Gated → Schritt 3 starten", definiert aber nicht „Schlüssel fehlt = Erfolg" | §5.3 Schritt 3, §5.5 | Idempotenzregel explizit festlegen: Fehlender Schlüssel nach Vernichtungsintent gilt als vollzogen, Verifikation gegen den Slot |
| S-07 | H | **ReadGate-Reload nach Neustart** lädt Gated, KeyRevoked, Purging, Verifying. **Restricted fehlt.** Nach einem Neustart wären Art.-18-eingeschränkte Daten ohne Audit lesbar. Widerspricht fail-closed (§6.1) und T-DEL-13 | §5.5, §5.2, §5.7, T-DEL-4, T-DEL-13 | Restricted in die Liste aufnehmen |
| S-08 | H | **Lineage-Typen am falschen Ort.** `LineageRef`, `DerivedRef`, `LineageIndex` sind in `contextra-db` (E2) kanonisch. §8.3(1) verlangt aber, dass jeder D-Schreibvorgang in E1 (`store`, `vector`, `text`) eine `LineageRef` angibt. Das verletzt R1 (E1 darf nicht nach E2) | §2.5, §8.2, §8.3 | `LineageRef`/`DerivedRef` nach E0 (`ids` oder `types`), `LineageIndex` in `db` |
| S-09 | H | **Krypto-Primitive und Signatur.** §4.5 sagt, Primitive liegen nur in `contextra-crypto`. §4.2 verlangt Ed25519 in `contextra-erasure` (Feature `signing`). Der Ed25519-Schlüssel steht nicht im Hierarchiebaum. §5.3 Schritt 1.2 verlangt „HMAC-basierte Duplikaterkennung", HMAC kommt in §4.2 nicht vor und der HMAC-Schlüssel ist unbenannt | §4.2, §4.5, §5.3 Schritt 1.2, §5.3 Schritt 6 | Signatur-Schlüssel in den Hierarchiebaum aufnehmen, HMAC-Schlüssel benennen, Feature-Verhalten bei fehlendem Signing festlegen |
| S-10 | H | **Testregel widerspricht dem Testkatalog.** „Jeder Test MUSS die vollständige Saga inkl. Purge durchlaufen" steht gegen T-DEL-5 (L-Stufe ohne Purge), T-DEL-3, -7, -14, -15 (Unit-Tests) | §12, T-DEL-5 | Regel auf Abnahmetests begrenzen |
| S-11 | M | **Reihenfolge Schritt 1.** Nummerierung nennt „1.3 resolve" vor „1.4 Journal Requested (fsync #1)". Der Code schreibt das Journal zuerst. Die Spezifikation betont „normativ, nicht vertauschen" | §5.3 Schritt 1 | Reihenfolge festlegen und Code oder Text angleichen |
| S-12 | M | **`forbid` vs. `allow`.** §2.6 setzt `unsafe_code = "forbid"` im Workspace, erlaubt aber Ausnahmen per `#[allow]` in `sys`/`simd`. `forbid` lässt kein `allow` zu, die Ausnahme ist also technisch unmöglich | §2.6, R5 | Workspace `deny`, per Crate `#![forbid(unsafe_code)]` (so ist es im Repo bereits) |
| S-13 | M | **Argon2id-Parameter.** SPEC §4.2: m=256 MB, t=4, p=1. v3.1 §4.2 (K1): m ≥ 64 MiB, t ≥ 3, ≥ 250 ms | §4.2 vs. v3.1 §4.2 | Eine Quelle festlegen |
| S-14 | M | **Gleiches Fehlerbild.** §6.1 fordert, dass ein gelöschtes Dokument von einem nie existierenden nicht unterscheidbar ist. Nirgends wird festgelegt, dass auch nie existierende IDs `Erased` liefern. `Restricted` liefert dagegen eine eigene Fehlerart (T-DEL-13), ist also unterscheidbar | §6.1, §6.2, §11, T-DEL-13 | Entscheidung dokumentieren: Erased für beides, oder bewusste Ausnahme |
| S-15 | M | **Wortregel vs. INV-7.** §5.4: „gelöscht" ab Klasse K. INV-7: „gelöscht nur mit gemessener Stufe". Klasse K ist laut §5.3 Schritt 5 nicht gemessen | §5.4, INV-7 | Entweder K misst oder INV-7 präzisieren |
| S-16 | M | **Hintergrundbetrieb nicht spezifiziert.** Das Ziel ist Löschung „unbemerkt im Hintergrund". Die Spec enthält aber keine I/O-Obergrenze, keine Drosselung, keinen Backoff für `Pending`, keinen Alarm bei überfälligen Sagas. `PurgeBarrier::purge()` ist exklusiv über den **gesamten** Purge und blockiert damit Kompaktionen für die Dauer. Die 1-GB-Reserve ist absolut und ohne Verhalten für den Fall, dass sie dauerhaft unterschritten bleibt | §5.3 Schritt 4, §10.2, §14 | Siehe Abschnitt 7.3 |
| S-17 | M | **Owner-Tabelle §2.5 unvollständig.** Nicht eingetragen sind u. a. `SecretString` (§14), `EraseReceipt`, `SagaGate`, `KeyCoverage`, `ControlEvidence`, `ErasureError`, `LayerScope`, `SourceId`, `ScanSession`. Schreibweise `TenantKEK` (§4.1) vs. `TenantKek` (§2.5) | §2.5, §4.1, §8.2, §14 | Vollständige Tabelle, einheitliche Schreibweise |
| S-18 | M | **Undefinierte Begriffe.** `retention.code` (in Code: `RetentionAttestation.code`, definiert), `Journal`, `SagaGate`, `Pending`-Vorzustand (wird nicht persistiert, Wiederaufnahme unklar), `Scanner<Armed>`-Medium | §5.2, §5.3, §5.5 | Glossar-Ergänzung |
| S-19 | N | Zählfehler: fsync `#4` wird in Schritt 5 und Schritt 6 zweimal verwendet. „Phase 3" in T-DEL-12 meint „Schritt 3" | §5.3, T-DEL-12 | Korrigieren |
| S-20 | N | Kompaktion: „ContainsErased → Range nicht in Ergebnis aufnehmen" ist mehrdeutig (Eintrag verwerfen oder Kompaktion überspringen?). `RestrictedNeverDiscard` ist nicht ausdefiniert | §6.2, §6.3 | Verhalten je Fall festlegen |
| S-21 | N | §15: „max. 5 Mitglieder" für Kern-Closure ist ohne Herleitung. Feature-Unifikation ist mit `cargo metadata` nicht vollständig prüfbar | §15 | Herleiten oder streichen; Prüfmethode nennen (`cargo tree -e features`) |

---

## 5. Widersprüche zwischen den Dokumenten (Governance)

Es gibt drei konkurrierende Baselines: SPEC v2.0 („FINAL, einzige Quelle der Wahrheit", 18.11.2026), v3.0 und v3.1 (26.10.2026, ersetzen „2.2.1" bzw. v3.0) und IST→SOLL (bezieht sich auf „Spec 2.2/2.3", Commit `a547a42`). SPEC v2.0 erklärt alle früheren Spezifikationen für ersetzt, übernimmt aber die Änderungen aus v3.x nicht.

| Thema | SPEC v2.0 | v3.0 | v3.1 | IST→SOLL | Code HEAD |
|---|---|---|---|---|---|
| **Reihenfolge Schlüssel vs. Purge** | Schlüssel-zuerst: Gate → Vernichtung (Schritt 3) → Purge (Schritt 4) | Purge-zuerst | Purge-zuerst („Key-first" ausdrücklich verworfen) | Purge-zuerst (INV-DEL-ORDER-1) | Schlüssel-zuerst (`saga.rs` `shred_step` vor `purge`) |
| **Gate reversibel?** | Nein (INV-10) | Gate ist Lesesperre, Karenz nicht geregelt | `Frozen` mit Unfreeze bis Schritt 5 | Folgt v3.1 | Gate irreversibel |
| **Schlüsselmodell** | Abgeleitet (HKDF aus KEK) | Zufällig, gewrappt, KeyTable | Zufällig, gewrappt, KeyTable, KEK-Epochen | Zufällig, KeyTable (D1–D6) | Abgeleitet (`kdf.rs`) |
| **Verschlüsselung D-Klasse** | Nicht geregelt | Nicht geregelt | ArtKey je Index-Artefakt | ArtKey, Blob-Segmente (Neubau) | Nicht geprüft (siehe 6.5) |
| **Positivkontrolle** | Canary über Schreibpfad | Canary, Rohdatei-Scan | Nadelscan, Pre-Destruction-Inventar | Scanner Unarmed/Armed | Trait-Boolean (Attrappe) |
| **Lineage** | Pflicht für jede D-Schreibung | — | Kaskade über Index | Offen: „Produktentscheid vor v1.0" (E-L1) | Nicht vorhanden |
| **Argon2id** | m=256 MB, t=4 | — | m ≥ 64 MiB, t ≥ 3 | — | ungeprüft |
| **Signatur Berichte** | Ed25519 | — | Ed25519, Siegel | — | Nicht vorhanden |

**Rechtlich relevant:** Der Konflikt Gate-Reversibilität ist nicht nur technisch. Eine „Karenz" nach Sperrung der Lesbarkeit kollidiert mit der Pflicht zur unverzüglichen Löschung. Die Trennung Restricted (Art. 18, reversibel) und Gated (Löschung, irreversibel) ist die Lösung, die beide Dokumente gleichzeitig erfüllt. Die juristische Bewertung ist fachanwaltlich zu bestätigen.

---

## 6. Befunde im Code (HEAD `deeddfd11`)

### 6.1 Build und Tests

**C-01 (K): 266 Rust-Dateien enthalten nur `DELETED`.**

| Crate | Verzeichnis | Anzahl |
|---|---|---|
| contextra-store | tests | 122 |
| contextra-erasure | src | 70 |
| contextra-erasure | tests | 39 |
| contextra-store | src | 20 |
| contextra-store | fuzz | 4 |
| contextra-crypto | src | 3 |
| contextra-store | benches | 3 |
| contextra-store | examples | 2 |
| contextra-mvcc | src | 1 |
| contextra-erasure | benches | 1 |
| contextra-erasure | fuzz | 1 |

- In `contextra-erasure` sind nur 4 von 43 Testdateien echt (`chain_spec_tests.rs`, `loom_erasure_models.rs`, `t_vault_core.rs`, `vault_slot_layout.rs`). Die `erasure/Cargo.toml` setzt kein `autotests = false`, also werden alle Dateien kompiliert.
- In `contextra-store` gibt es zusätzlich `[[test]]`-Einträge mit `required-features`. Auto-Discovery ist dort nicht abgeschaltet.
- Keine der DELETED-Dateien ist über `mod` eingebunden. Der Build der Bibliothek ist deshalb vermutlich nicht betroffen, die Testziele aber sehr wohl.
- Herkunft: `578d0e282 Shell-Commit` (erasure), `7125c19c6 Shell-Commit` (store, heute). `erasure/src/saga.rs` war bereits im Vorgänger-Commit 7 Bytes groß.

**C-02 (H): Verwaiste Dateien.** 70 von 93 Dateien in `contextra-erasure/src` sind nicht im Modulbaum erreichbar. Dazu gehören `journal/`, `ledger/`, `measure/`, `report/` (inkl. `signing.rs`), `deletion_proof.rs`, `ed25519_proof.rs`, `purge/`, `saga/` (Altversion), `gate/` (Teile) und `vault/keyring.rs`. Weitere verwaiste Dateien: `store` 23 (385 Zeilen), `checkpoint` 3 (531 Zeilen), `crypto` 3, `mvcc` 2. SPEC §0 verlangt die Entfernung von Altlasten.

### 6.2 Verdrahtung

**C-03 (K): Die Löschkette ist nicht erreichbar.**
- `Chain` und `ChainDeps` werden außerhalb von Tests nicht konstruiert (`ChainDeps {` nur in `tests/chain_spec_tests.rs:233`).
- `contextra` (Facade) referenziert `contextra_erasure` nicht.
- `contextra-db` nutzt nur `ReadGate`, `SagaGate`, `GateDecision` aus `contextra-erasure`, also den Lesepfad.
- `Collection::delete` (`contextra-db/src/collection/crud/delete.rs`) setzt einen Tombstone und keinen Schlüssel, ruft kein Gate auf und startet keine Saga. Die Dokumentation im Kopf nennt noch `contextra-engine`.

**C-04 (K): Keine produktive `ErasureLayer`-Implementierung.** Eine Suche nach `impl … ErasureLayer for` außerhalb von Tests ergibt nichts. Purge (Schritt 4) und Witness (Schritt 5) iterieren damit über eine leere Menge. `verify` würde einen Bericht mit null Zeugen veröffentlichen. Es gibt keine Prüfung „mindestens eine Schicht gemessen". Dieser Fehler wird sichtbar, sobald die Verdrahtung kommt.

**C-05 (K): Ports ohne Produktionsimplementierung.** Für `ReportSink` und `PositiveControl` (Canary) gibt es nur die Trait-Deklaration. `canary_observed()` und `publish()` sind nicht implementiert.

### 6.3 Messung

**C-06 (K): Die Positivkontrolle ist eine Attrappe.**
- `verify` (`chain/saga.rs:~300`) ruft `canary_observed()` auf, einen `Result<bool>`. Das Ergebnis des Scanners fließt nicht in die Messung ein.
- Danach konstruiert `verify` `ControlEvidence::observed()` unbedingt und übergibt es an `arm()`.
- `ControlEvidence::observed()` ist in `contextra-types/src/erasure.rs:451` als **`pub const fn`** definiert. Jede Crate kann damit `Scanner<Armed>` erzeugen. Die Erasure-Variante ist `pub(crate)`, die Types-Variante nicht. INV-WITNESS-MINT-1 ist damit gebrochen.

**C-07 (H): Der Lesepfad-Test ist tautologisch.** Schritt 5.4 prüft `gate.decision(target) == Erased` unmittelbar nach `gate.activate(target)`. Die Spezifikation verlangt eine echte Leseabfrage über die Pfade aus §6.2.

**C-08 (H): Keine unabhängige Messquelle.** IST-Entscheidung D-13 verlangt zwei Quellen je Pflichtschicht (Scanner der Schicht plus Black-Box-Probe des Orchestrators). Am HEAD gibt es nur die Selbstauskunft des Layers über `witness()`. Das gilt, sobald Layer existieren.

### 6.4 Krypto

**C-09 (K): Abgeleitete Schlüssel.** Siehe Abschnitt 3.

**C-10 (H): Zweite Schlüsselhierarchie in `contextra-erasure`.** `crypto_local.rs` enthält eigenes HKDF, eigenes `SecretKey<const N>`, `derive_dek(unit, ancestor_salts, doc)` mit eigenem Modell (`UnitKey`, `AncestorSalts`) und Abhängigkeiten auf `sha2` und `hmac`. Das verletzt §4.5 und R4 und erzeugt ein zweites, inkompatibles Schlüsselmodell.

**C-11 (H): Keine Ed25519-Signatur.** Im gesamten Code gibt es keine Verwendung von `ed25519`. `report/signing.rs` ist ein Platzhalter und nicht eingebunden. Die Testdateien `proof_ed25519_external_verify.rs` und `deletion_proof_*` sind Platzhalter.

**C-12 (H): `encryption_passphrase: Option<String>`** in `contextra-db/src/lib.rs:221` (und im Builder). SPEC §4.4 nennt dieses Muster ausdrücklich einen Verstoß. Die Passphrase ist kein `SecretString` mit `ZeroizeOnDrop`.

**C-13 (M): `sha2` außerhalb von `contextra-crypto`:** `contextra-types`, `contextra-erasure`, `contextra-engine`. `hmac` zusätzlich in `contextra-erasure`. §4.5 erlaubt das nicht.

### 6.5 Architekturregeln R1–R6

Maschinelle Prüfung aller 24 Manifeste.

| Regel | Ergebnis |
|---|---|
| R1 (keine Aufwärtskanten) | **Erfüllt**, keine direkte Aufwärtskante gefunden. Die Lineage-Typlage (S-08) ist ein Problem auf Spezifikationsebene. |
| R2 (keine Dev-Abhängigkeit nach oben) | **Verletzt:** `contextra-checkpoint` (E1) und `contextra-engine` (E2) haben `contextra-testkit` (E3) als Dev-Dependency. |
| R3 (`default = []`) | **Verletzt** in 7 Crates: `contextra` (`embedded`), `contextra-durable-fs` (`std`, `crc`), `contextra-graph` (vier Features, darunter `experimental-graph`), `contextra-rank` (vier Features, darunter `experimental-rank`), `contextra-simd` (`std`), `contextra-testkit` (`testkit`, `erasure-conformance`), `contextra-text` (`std`). |
| R4 (ein Besitzer je Typ) | **Verletzt**, siehe unten. |
| R5 (`forbid(unsafe_code)`) | **Erfüllt** in 22 Crates. `sys` und `simd` sind die dokumentierten Ausnahmen. Der Workspace setzt `deny`, nicht `forbid` (siehe S-12). |
| R6 (kein `unwrap`/`expect`/`panic!` in Nicht-Test-Code) | **Ungeprüft** durch Clippy. Grob gezählt: 402 Vorkommen von `unwrap`/`expect` in `src`-Dateien von erasure, ports, types, crypto und ids, ohne Berücksichtigung von `#[cfg(test)]`. |

**R4-Verletzungen (Doppeldefinitionen):**

| Typ | Definitionen |
|---|---|
| `ReadGate`, `GateDecision`, `RangeGate` | `contextra-ports/src/gate.rs:104,115,128` und `contextra-erasure/src/gate/saga_gate.rs:25,36,46` |
| `ErasureLayer` | `contextra-ports/src/erasure.rs:206` und `contextra-erasure/src/chain/layer.rs:41` |
| `Scanner<State>` | `contextra-ports/src/erasure.rs:156`, `contextra-types/src/erasure.rs:485`, `contextra-erasure/src/types_local.rs:137` |
| `LayerWitness` | `contextra-ports/src/scanner/witness.rs:17`, `contextra-types/src/erasure.rs:559`, `contextra-erasure/src/types_local.rs:225` |
| `RetentionAttestation` | `contextra-types/src/erasure.rs:105`, `contextra-erasure/src/chain/retention.rs:58` |
| `ControlEvidence` | `contextra-types/src/erasure.rs:451` (öffentlich), `contextra-erasure/src/types_local.rs:112` (`pub(crate)`) |
| `SecretKey` | `contextra-erasure/src/crypto_local.rs` und `contextra-crypto` |

SPEC §2.5 nennt die kanonischen Besitzer: `ErasureLayer`, `Scanner`, `LayerWitness`, `RetentionAttestation`, `ReadGate` gehören zu `contextra-erasure`. Die Duplikate in `ports` und `types` sind damit Verstöße. `ports` hängt laut §2.4 nur von `env` und `types` ab, und `ports` enthält trotzdem Erasure-Typen.

### 6.6 Positives

- Die Zustandsmaschine (`chain/saga.rs`) setzt Gate vor Vernichtung, persistiert jeden Übergang vor dem Umschalten, macht Purge idempotent (T-DEL-14), eskaliert Restricted→Gated und verweigert bei Aufbewahrungspflicht (`Refused`). Das ist eine solide Grundlage.
- Keine Aufwärtskanten in der Abhängigkeitsstruktur.
- `forbid(unsafe_code)` ist in allen Crates außer `sys` und `simd` gesetzt.
- Das Repo hat die Altlasten aus SPEC §0 bereits entfernt (`reports/`, `docs/diagnostics/`, `capabilities.toml`, `tools/gate_layering.py`, `ALTLASTEN-AUDIT.md` u. a.).

### 6.7 Nicht geprüft

- Ob Text- und Vektor-Artefakte verschlüsselt persistiert werden (`contextra-text`, `contextra-vector`). Der IST-Plan nennt den Textindex als Präfix-Keys im `StorageEngine` (B-13, „ungeprüft"). Das ist die Grundlage für S-03 und muss am Code bestätigt werden.
- Ob die Konfigurationsvalidierung (§14) existiert.
- `check_disk_reserve`, `run_purge_window` und `verify_key_absent`, die die SPEC namentlich nennt, sind nicht vorhanden (je 0 Treffer).

---

## 7. Bewertung der Ansätze auf Umsetzbarkeit

### 7.1 Vergleich

| Ansatz | Kern | Trägt die Löschgarantie? | Umsetzbarkeit | Urteil |
|---|---|---|---|---|
| **SPEC v2.0** | Abgeleitete DEKs, Schlüssel-zuerst, sechs Schritte, ReadGate, Typestates | **Nein.** Abgeleitete Schlüssel sind nicht vernichtbar (S-01). D-Klasse ohne Verschlüsselungsregel (S-03). | Automatenlogik ist vorhanden. Das Schlüsselmodell muss getauscht werden. | Rahmen übernehmen, Schlüsselmodell ersetzen |
| **v3.0** | Zufällige DocDeks in KeyTable, Purge-zuerst, Ebenen A/B | Ja, für K und P | Mittel. Die Begründung der Reihenfolge ist schwach (30-Tage-Verzögerung). | Durch v3.1 überholt |
| **v3.1** | Wie v3.0, plus KEK-Epochen und Epoch Seal, Custody K1–K3, ArtKey für Index-Artefakte, Zwei-Spuren-Backup, Restricted, Fast-Path | Ja. Stärkster Ansatz. K1 belegbar, K2/K3 ausdrücklich nicht ausgeliefert | **Hoch.** 24 Abschnitte, neue Ports (`EffaceableKeySlot`, `ErasedSetMirror`, `RecipientLedger`), Epoch-Seal-Mechanik, Backup-Trennung. Mehrere P-Punkte rechtlich offen. Parameterkonflikt mit SPEC (S-13) | Schlüsselmodell, ArtKey und Fast-Path übernehmen. Epoch Seal, Backup-Trennung und K2/K3 in eine Folgeversion |
| **IST→SOLL** | Zehn-Phasen-Plan, übernimmt die v3.1-Reihenfolge | Der Plan trägt, sofern die Baseline stimmt | **Hoch.** Phase 0 (Build-Baseline) ist blockierend. Mehrere IST-Befunde sind am HEAD veraltet (z. B. B-21). Entscheidungspunkte E-B1 bis E-N1 offen | Als Gerüst nutzen, nach Neuprüfung am HEAD |

### 7.2 Empfehlung: Hybrid „Ebene A zuerst, dann Schlüssel, dann Hygiene"

Die drei Ansätze sind in einem Punkt unvereinbar (Reihenfolge), in den übrigen Punkten lassen sie sich zusammenführen. Die Begründung für die Reihenfolge liegt in der Messbarkeit:

- **Ebene A (RAM: Arena, Caches, Postings im Speicher, Antwort-Cache)** liegt im Klartext vor und muss vor der Vernichtung bereinigt und gemessen werden, weil danach keine entschlüsselte Sicht mehr existiert. Das ist das Argument aus v3.1 §6.2.
- **Ebene B (persistente Reste)** wird durch die Schlüsselvernichtung unlesbar. Der Zeitpunkt der Vernichtung ist deshalb die wirksamste Maßnahme. Eine Verzögerung nach v3.0/v3.1 bringt keinen Sicherheitsgewinn, solange jede Ebene-B-Struktur verschlüsselt ist. Das Argument „Blöcke mit fremden Dokumenten müssen vorher neu geschrieben werden" entfällt, wenn Records einzeln unter DocDek liegen (v3.1 §4.5).
- **Hygiene (Kompaktion, Graph-Neuaufbau, Scrub)** ist Defense in Depth und darf nach der Vernichtung im Hintergrund laufen.

Die Reihenfolge lautet damit:

1. Gate setzen (`Gated`, irreversibel).
2. Ebene A bereinigen und messen (Positivkontrolle auf Ebene A).
3. Pre-Destruction-Inventar als **Metadaten** (Key-IDs, Blockzuordnung, keine Klartexte).
4. Schlüssel vernichten, verifizieren (`KeyRevoked`).
5. Ebene B messen (Nadelscan, Chiffrat-Identität).
6. Hygiene im Hintergrund, terminiert.
7. Bericht, Ledger, `Completed`.

Das entspricht SPEC v2.0 in der Schrittreihenfolge, übernimmt aber die Messlogik aus v3.1 und löst S-04 (Positivkontrolle), S-05 (Gate-Reversibilität) und S-01 (Schlüsselmodell) auf.

**Voraussetzung:** Alle persistierten Index-Artefakte müssen verschlüsselt sein (ArtKey, S-03). Ohne das gilt Schritt 4 nicht als Löschung der Ebene B.

### 7.3 Hintergrundbetrieb („unbemerkt, aber vollständig")

Damit die Löschung unbemerkt bleibt, sollte die Spezifikation regeln:

- **I/O-Budget und Drosselung** für den Purge-Scheduler, damit Abfragen nicht spürbar verlangsamt werden.
- **Keine globale exklusive Barriere** über den gesamten Purge. Die `PurgeBarrier` sollte pro Segment oder SSTable gelten, damit Kompaktionen nicht für die gesamte Laufzeit blockiert werden.
- **Pending mit Backoff** und Wiederaufnahme. Der Vorzustand muss im Journal stehen (S-18).
- **Alarm für überfällige Sagas** als Zustandsmetrik über `MetricsSink` ohne Nutzinhalte (INV-12).
- **Plattenreserve:** Die Reserve als absoluten Mindestwert (1 GB) beibehalten und zusätzlich einen relativen Schwellwert festlegen, der sich am tatsächlichen Platzbedarf der Rewrites orientiert. Verhalten bei dauerhafter Unterschreitung festlegen.

---

## 8. Empfohlenes Vorgehen

**Phase 0: Baseline herstellen (blockierend).**
- Entscheiden, was mit den 266 `DELETED`-Dateien geschieht (siehe Entscheidung D-7).
- `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets` grün bekommen.
- Die IST-Befunde am HEAD neu verifizieren. Viele sind inzwischen überholt, andere sind neu.

**Phase 1: Entscheidungen als ADR festhalten** (Abschnitt 9).

**Phase 2: Schlüsselmodell umbauen.**
- `kdf.rs`: DocDek, SubjectKey und SubjectDocDek zufällig erzeugen und gewrappt speichern (KeyTable, v3.1 §4.5 und §4.8).
- Duplikate in `crypto_local.rs` entfernen. Eine Hierarchie.
- `encryption_passphrase` durch `SecretString` ersetzen (§4.4).

**Phase 3: Verschlüsselung der D-Klasse.** ArtKey je Index-Artefakt (Text-Segmente, Graph-Snapshot). Vektor-Payload unter DocDek.

**Phase 4: Typen konsolidieren (R4).** Die Doppeldefinitionen aus 6.5 auf die kanonischen Besitzer reduzieren. `ControlEvidence::observed()` nur noch aus einem autorisierten Modul erzeugbar machen (Erasure `pub(crate)` ist korrekt, `types` muss weg).

**Phase 5: Layer und Verdrahtung.**
- Produktive Implementierungen von `ErasureLayer` für Store, Text, Vector, Graph, Crypto und MVCC.
- `ReportSink` mit Ed25519 (Feature-Verhalten nach S-09 festlegen).
- `Collection::delete` bzw. ein neuer Löschpfad ruft die Saga auf. Die öffentliche API bekommt eine klare Löschoperation mit Ergebnis (`EraseReceipt`).
- Mindestens eine Layer-Schicht muss gemessen sein, sonst ist `Completed` nicht erreichbar (C-04).

**Phase 6: Messung echt machen.**
- Positivkontrolle auf Ebene A, nicht über `canary_observed()`-Boolean.
- Lesepfad-Test über echte Abfragen (C-07).
- Unabhängige Black-Box-Probe (C-08).

**Phase 7: Tests.** Die Testkatalog-Lücken (Abschnitt 9.2) schließen. Die gelöschten Testdateien nur dann wiederherstellen, wenn sie zum neuen Modell passen.

**Phase 8: Hintergrundbetrieb** (Abschnitt 7.3) und Prüfung der R2/R3-Verstöße.

---

## 9. Entscheidungen

### 9.1 Benötigte Entscheidungen

| ID | Frage | Empfehlung |
|---|---|---|
| D-1 | Welches Dokument ist normativ? | SPEC v2.0 als Rahmen. Die Abschnitte §4, §6, §8, §9 aus v3.1 per ADR übernehmen. v3.0, IST→SOLL und „2.2.1" als Referenz archivieren, nicht als Quelle |
| D-2 | Schlüsselmodell: abgeleitet oder gespeichert? | Gespeichert und gewrappt (v3.1 §4.5) |
| D-3 | Reihenfolge Schlüssel vs. Purge | Hybrid aus 7.2: Ebene A zuerst, dann Schlüssel, dann Hygiene |
| D-4 | Gate reversibel? | Restricted (Art. 18) ist reversibel, Gated (Löschung) ist irreversibel. Kein Unfreeze nach Gate. Rechtliche Prüfung |
| D-5 | Verschlüsselung der D-Klasse | Ja, mit ArtKey je Artefakt. Voraussetzung für D-3 |
| D-6 | Lineage | Verbindlich für v1.0 oder bewusst verschoben? Die SPEC sagt Pflicht, der IST-Plan sagt „offen" |
| D-7 | Umgang mit den 266 `DELETED`-Dateien | Die Tests gegen das neue Modell neu schreiben. Vorhandene Testideen aus dem Commit vor `578d0e282` als Referenz nutzen. Stubs entfernen |
| D-8 | Umgang mit den „Shell-Commits" | Klären, wer oder was diese Commits erzeugt (349 von 5.943). Ich kann die Ursache aus dem Repo nicht ableiten |

### 9.2 Testlücken

Diese Tests fehlen im Katalog oder existieren nur als Platzhalter:

- Positivkontrolle mit **negativem** Fall (Canary nicht gefunden → Messung ungültig) auf Ebene A.
- Restricted-Zustand nach Neustart bleibt wirksam (S-07).
- Crash zwischen Vernichtung und `KeyRevoked`, Wiederaufnahme mit fehlendem Schlüssel (S-06).
- Einzeldokument-Löschung K (S-01).
- Subject-Löschung mit gespeichertem Schlüssel, O(1) (T-DEL-9 mit Gegenprobe: CollectionKEK lebt, Daten bleiben unlesbar).
- Lesepfad-Test über echte Abfragen (C-07).
- Compaction-Verhalten bei `ContainsErased` (S-20).
- Konfigurationsvalidierung §14.
- Plattenreserve unterschritten: Pending, Wiederaufnahme, Alarm.

---

## 10. Reproduktion

```bash
cd contextra

# C-01: Platzhalter-Dateien
python3 -c "import os
for dp,_,fs in os.walk('crates'):
  for f in fs:
    p=os.path.join(dp,f)
    if f.endswith('.rs') and open(p,errors='ignore').read().strip()=='DELETED': print(p)" | wc -l

# C-01: Erasure-Testdateien, echt vs. Platzhalter
for f in crates/contextra-erasure/tests/*.rs; do [ "$(tr -d '\n' < $f)" = DELETED ] && echo "D $f" || echo "  $f"; done

# S-01 / C-09: abgeleitete Schlüssel
sed -n 416,445p crates/contextra-crypto/src/kdf.rs

# C-03: Löschkette in der Produktion?
grep -rn "contextra_erasure" crates/contextra/src crates/contextra-db/src | grep -v tests

# C-04 / C-05: Produktive Implementierungen der Ports
grep -rn --include=*.rs "impl.*ErasureLayer for\|impl.*ReportSink for" crates | grep -v /tests/

# C-06: Positivkontrolle und öffentlicher Mint
grep -n "pub const fn observed" crates/contextra-types/src/erasure.rs
sed -n 210,260p crates/contextra-erasure/src/chain/saga.rs | grep -n "observed\|canary"

# C-11: Ed25519
grep -rli --include=*.rs ed25519 crates | wc -l

# C-12: Passphrase als String
grep -n "encryption_passphrase" crates/contextra-db/src/lib.rs

# R2 / R3
python3 -c "import tomllib,glob
for f in glob.glob('crates/*/Cargo.toml'):
  d=tomllib.load(open(f,'rb'))
  if d.get('features',{}).get('default') not in (None,[]): print(f, d['features']['default'])"

# Git: Herkunft
git log --oneline -S"DELETED" -- crates/contextra-erasure/tests | head
git log --format=%s | grep -c "^Shell-Commit"
```

---

## 11. Abschließender Hinweis

Die Dokumente sind inhaltlich ausgereift, die größte Arbeit ist die Entscheidung zwischen ihnen. Das Repo ist derzeit weder baubar noch testbar, und die Löschkette ist nicht angeschlossen. Vor einer Umsetzung sollten D-1 bis D-4 und D-7 entschieden sein. Danach kann die Phase-0-Baseline in einem eigenen Branch erstellt werden.
