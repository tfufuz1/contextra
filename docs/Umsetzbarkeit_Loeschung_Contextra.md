# Umsetzbarkeitsanalyse: Löschspezifikation für Contextra

_Repository: `tfufuz1/contextra` (Shallow-Klon, Commit `8c4532ba`) · Bezug: Spezifikation 0.1 (README, 01_schluesselmodell, 02_loeschung) · Stand: 16.11.2026_

## 0. Ergebnis

| Frage | Befund |
|---|---|
| Technisch machbar? | **Ja**, aber nicht als Ergänzung des bestehenden Löschpfads. Die Bausteine (AES-256-GCM-SIV, Gruppen-KEK, Record-DEK, Widerrufs-Log mit Ed25519-Kette) existieren, sind aber nicht verdrahtet. |
| Klasse L (logisch entfernt) | Heute bereits erreichbar. Tombstones funktionieren. |
| Klasse K (Schlüssel vernichtet) | **Heute nicht erreichbar.** Der Schreibpfad speichert Klartext, und der Löschpfad widerruft keinen Schlüssel. |
| Klasse I/M | Nicht erreichbar ohne Entscheidung zur Indexstrategie (Abschnitt 4). |
| Größter Blocker | Der Master-KEK wird zufällig im RAM erzeugt und nie persistiert. Wer die Verschlüsselung ohne Schlüsselpersistenz einführt, verliert nach jedem Neustart alle Daten. |
| Aufwand | Groß. Grob 11 Arbeitspakete (Abschnitt 5), davon 3 mit Architekturentscheidung vorab. |
| Ist-Zustand des Spec | Die Befunde des Spec sind korrekt. Ich habe sie belegt und um 7 bisher nicht dokumentierte Befunde ergänzt (Abschnitt 2). |

## 1. Methodik

1. Klon des Repositories. Workspace-Struktur, `CLAUDE.md`, `reports/AUDIT_REPORT.md` gelesen.
2. Toolchain: `rustc-1.89` / `cargo-1.89` über apt (Ubuntu 24.04-Paket). `static.rust-lang.org` ist nicht erreichbar, dies entspricht dem Hinweis in `CLAUDE.md`.
3. Ausgeführt: `cargo test -p contextra --test deletion_e2e` (Build 2 m 32 s, 1 vCPU).
4. Experiment `spike_keys` mit vier Szenarien gegen die Bibliothek `contextra-erasure`. Die Datei wurde nach dem Lauf wieder entfernt. Der Klon ist unverändert (`git status` leer).
5. **Nicht selbst verifiziert:** Die Kompilierfehler der Proof-Tests in `contextra-erasure` (78 Fehler) und die 16 fehlgeschlagenen Store-Tests stammen aus `AUDIT_REPORT.md` und sind nicht reproduziert.

## 2. Prüfung der Spec-Befunde gegen den Code

### 2.1 Bestätigte Befunde

| Spec-Aussage | Ergebnis | Beleg |
|---|---|---|
| Löschen setzt nur Tombstones für `user_key` und `doc_key` | bestätigt | `delete_op`, `crates/contextra-db/src/collection/crud/delete.rs` |
| Alte Werte bleiben in der Transaktionshistorie | bestätigt | `record_keys_with_old_values` |
| `kv_delete_mode` wird nur validiert und angezeigt | bestätigt | `collection_profile.rs:113`, `contextra-mcp/src/tools_crud.rs:92` |
| T-DEL-1 schlägt fehl, `left: 1, right: 0` | **bestätigt im Lauf** | Ausgabe `assertion left == right failed … left: 1, right: 0` |
| Standard ist unverschlüsselt | bestätigt | `encryption_passphrase: None`, `contextra-db/src/lib.rs:309` |
| `contextra-erasure` ist kein Default-Member | bestätigt | `Cargo.toml`, `default-members` |
| T-DEL-3 prüft nur das Droppen, keinen Löschnachweis | bestätigt | `deletion_e2e.rs`, Testrumpf enthält kein Nachweis-Assert |

### 2.2 Abweichungen und neue Befunde

| # | Befund | Schwere | Beleg |
|---|---|---|---|
| N1 | **Master-KEK ist zufällig und nicht persistent.** `KeyRegistry::new()` erzeugt `master_kek` über `OsRng`. Die gewrappten KEKs existieren nur im Speicher. Nach Neustart ist jeder Chiffretext unlesbar. Im Experiment: `Err(Refused)` nach Neustart. | **kritisch** | `kv_shredding.rs:244`, Spike [1] |
| N2 | **`revoke_record(&str)` wirkt auf keinen Schlüssel.** Die Methode leert `record_deks: BTreeMap<String, …>`, in diese Map wird nirgends eingefügt. Der Schlüssel bleibt entschlüsselbar. Der tatsächlich wirksame Pfad ist `revoke_record_num(group, record)`. Die Spec nennt die `&str`-Variante als Schritt 2 des Löschablaufs. | **kritisch** | `kv_shredding.rs:224, 475`, Spike [2] |
| N3 | **Revocation-Log ist nicht in Produktion verdrahtet.** `with_revocation_log` und `RevocationLog::new_in_memory`/`open_or_create` werden nur in Tests aufgerufen. Ohne Log ist ein Widerruf nicht durabel, was Schritt 2 der Spec verlangt. | hoch | Suche nach `with_revocation_log` |
| N4 | **`EnterpriseRegulated` deklariert `CryptoShred`, erzwingt aber keine Verschlüsselung.** Die Validierung prüft nur Kombinationen mit `TombstoneOnly`. Ein Compliance-Profil läuft also mit Klartext auf dem Datenträger. Das ist eine falsche Compliance-Aussage. | **hoch** | `collection_profile.rs:177`, `validate()` |
| N5 | **Der Test `…after_compaction` führt keine Compaction aus.** Er ruft `insert`, `delete`, `flush` auf. Der Treffer liegt in `sst-…-000000.sst`, also in einer SSTable, nicht im WAL. Die Spec verlangt T-DEL-2 mit Compaction, dieser Test misst sie nicht. | mittel | `deletion_e2e.rs`, Treffer per temporärer Ausgabe ermittelt |
| N6 | **`derive_subkey` ist kein HKDF und wird im Produktivpfad nicht genutzt.** Die Spec beschreibt `MasterKey ──HKDF──> GroupKek`. Der Code verwendet BLAKE3-`derive_key`, und nur Tests rufen die Funktion auf. Die Registry erzeugt KEKs zufällig. | mittel | `kv_shredding.rs:85`, `shredding_envelope_test.rs:110` |
| N7 | **`DocId::from_key` ist ein 64-Bit-BLAKE3-Abschnitt.** Kollisionen zwischen Dokumenten derselben Collection teilen sich `record_id` und damit den Schlüssel. Die Spec fordert in 1.2 Eindeutigkeit. Bei Kollisionen wird ein fremder DEK widerrufen oder überschrieben. Die Geburtstagsschranke liegt bei etwa 5·10⁹ Dokumenten (50 %), ist also niedrig, aber nicht null und nicht abgesichert. | mittel | `contextra-erasure/src/types_local.rs:1653`, `contextra-ids/src/lib.rs:250` |
| N8 | **Keine AAD.** `encrypt_envelope_record` bindet den Chiffretext weder an Mandant, Collection noch Dokument. Die Bindung entsteht nur durch den Registry-Lookup (`wrapped_dek`-Vergleich). Die Spec fordert Additional Data in 1.1. | mittel | `kv_shredding.rs:~520` |
| N9 | **Kein `Erased`-Fehler im Erasure-Crate.** `Refused` wird für „fehlt“, „widerrufen“ und „nach Neustart fehlt“ gleich zurückgegeben. Die Spec verlangt eine Unterscheidung nach 1.3. `Erased` existiert nur als `GateDecision` in `contextra-ports`. | mittel | `error.rs:78`, Spike [1] und [3] |
| N10 | **Kopien von DEK-Material.** `decrypt_envelope_record` hält `dek_bytes` als `Vec<u8>` ohne `zeroize`. Die Spec verbietet Restkopien (1.4). | niedrig | `kv_shredding.rs`, Entschlüsselungspfad |
| N11 | **Gruppenwiderruf blockiert Neuanlage.** Nach `revoke_group` liefert `get_or_derive` für dieselbe `group_id` `Refused`. Eine neu angelegte Collection mit gleicher ID ist damit unbeschreibbar. Die Spec beschreibt kein Epochen-Rollover. | mittel | `kv_shredding.rs:~345` |

## 3. Experiment `spike_keys`

Ausgeführt mit dem Crate `contextra-erasure` (Default-Features, `signing` aktiv), `KeyRegistry::new()` je Szenario.

| Szenario | Ergebnis | Bewertung |
|---|---|---|
| [1] Lesen vor Neustart | `Ok(true)` | Normalbetrieb funktioniert |
| [1] Lesen nach Neustart | `Err(Refused)` | **Datenverlust**, Fehler nicht von „gelöscht“ unterscheidbar |
| [2] `revoke_record("42")` | `Ok("LESBAR")` | **Widerruf wirkungslos** |
| [3] `revoke_record_num` | `true`, danach `Err(Refused)` | Wirksamer Einzelwiderruf |
| [4] `revoke_group` | `Err(Refused)` | Wirksamer Gruppenwiderruf (O(1)) |

Die Szenarien [1] und [2] widerlegen die Spec-Annahme, dass die vorhandene Registry im Löschablauf der Spec einsatzfähig ist.

## 4. Architekturentscheidungen vor der Umsetzung

### 4.1 Schlüsselquelle des Master-Keys (blockierend)

| Option | Vorteile | Nachteile |
|---|---|---|
| A: Passphrase, Argon2id, Salt im Manifest | Keine Abhängigkeit vom OS, reproduzierbar | Passphrase-Verlust = Totalverlust, kein Schutz gegen RAM-Auslesen |
| B: OS-Keyring / TPM | Starker Schutz gegen Offline-Kopie | Plattformabhängig, auf Air-Gap-Systemen oft nicht verfügbar |
| C: Externe KMS | Zentrale Rotation | Widerspricht Air-Gap-Anforderung |

**Empfehlung:** A als Standard, B als optionales Plugin. Die Passphrase ist gleichzeitig der Wurzelschutz für die persistierten gewrappten KEKs.

### 4.2 Verschlüsselungsort

Der Schlüssel muss vor dem WAL angewendet werden, sonst verletzt der Schreibpfad Grundaussage 2 der Spec. Die Verschlüsselung im `contextra-store`-Blockpfad (Option B) hätte den Nachteil, dass Schlüssel-Gruppen dort nicht bekannt sind. **Empfehlung:** Envelope-Verschlüsselung in `contextra-db` auf Dokumentebene, mit AAD `(tenant, collection, doc_id, version)`.

### 4.3 Indexstrategie (blockierend für Klasse I)

Die Volltext-Posting-Listen enthalten Tokens im Klartext, die Vektor-Indizes Embeddings. Beide sind personenbezogen (Embedding-Inversion ist möglich). Der Schlüssel-Widerruf allein macht sie nicht unlesbar.

| Option | Klasse I | Kosten |
|---|---|---|
| A: Index nur für Tokens verschlüsselter Dokumente, Suche entschlüsselt Kandidaten | erreichbar | Suche zahlt Entschlüsselung je Kandidat |
| B: Index-Einträge pro Dokument mit DEK verschlüsselt | erreichbar | Ranking über Kandidaten, Index-Format ändert sich |
| C: Physische Neuschreibung der betroffenen SSTables nach Löschung | erreichbar mit Compaction-Kosten | Schreibverstärkung, nicht O(1) |

**Empfehlung:** B für Volltext, A für Vektor. Option C ist als Fallback für Altbestände zu planen.

### 4.4 Gruppenmodell

Ein Collection-Drop widerruft den KEK der `group_id`. Die Spec muss festlegen, dass eine neue Collection eine neue Epoche (`GroupEpochKey`) erhält, und wie `group_id` bei Mandanten vergeben wird.

## 5. Arbeitspakete

Reihenfolge ist verbindlich. AP1 ist Voraussetzung für jede Verschlüsselung im Schreibpfad.

| AP | Inhalt | Aufwand | Abhängig von |
|---|---|---|---|
| AP0 | ADR zu 4.1 bis 4.4 | S | — |
| AP1 | Persistenter Keystore: Master-Key aus Passphrase, gewrappte KEK/DEK atomar und mit `fsync` auf Platte, versionierter Container, Fail-closed beim Start | L | AP0 |
| AP2 | Revocation-Log in Produktion verdrahten, beim Öffnen laden. `revoke_record(&str)` entfernen oder auf `revoke_record_num` umleiten. Fehlerkategorie `Erased` einführen | M | AP1 |
| AP3 | Schreibpfad: Payload vor WAL verschlüsseln, AAD binden, Klartext-Verbot erzwingen | L | AP1 |
| AP4 | Lesepfad: Entschlüsselung, einheitliche Fehlerkodierung nach außen (1.3) | M | AP3 |
| AP5 | Versionierung: Jeder Update-Schreibvorgang erzeugt neuen DEK, alter DEK wird widerrufen (1.2) | M | AP3 |
| AP6 | `delete_op` gemäß Spec 2.2, Schritte 1 bis 7, inklusive Modus-Auswertung (2.3) | M | AP2, AP5 |
| AP7 | Indizes gemäß 4.3 umsetzen (Volltext, Vektor, Graph) | L | AP3, AP6 |
| AP8 | Profilvalidierung: `CryptoShred` erfordert aktive Verschlüsselung, sonst Fehler beim Öffnen (N4). **Sofort umsetzbar, kein Abhängigkeit** | S | — |
| AP9 | Erasure-Crate reparieren (78 Fehler in Proof-Tests) und in `default-members` aufnehmen | M | — |
| AP10 | Testkatalog gemäß Spec 2.8: T-DEL-1 grün, T-DEL-2 mit Compaction (Test umbenennen, wenn nicht implementiert), T-DEL-3 korrigieren, T-DEL-4 bis T-DEL-8 neu. Positivkontrolle (T-DEL-7) als Voraussetzung jeder Negativmessung | M | AP6, AP7 |

**Sofortmaßnahmen ohne Architekturentscheidung:** AP8, Umbenennung von `test_tdel_1_canary_bytes_erased_after_compaction` oder Einfügen einer Compaction, Korrektur von `test_tdel_3_drop_collection_e2e`, Entfernen von `revoke_record(&str)`. Diese vier Punkte verhindern eine falsche Compliance-Aussage, bevor die eigentliche Umsetzung beginnt.

## 6. Risiken

- **Datenverlust:** AP3 ohne AP1 vernichtet Daten bei jedem Neustart. Dies ist das größte Risiko der Reihenfolge.
- **Rollback nach Widerruf (Spec 2.2, Schritt 5):** Der Widerruf überlebt einen Rollback, der Tombstone nicht. Das Dokument ist dann unlesbar, der Index-Eintrag existiert aber noch. Die Spec muss festlegen, ob dies als Integritätsfehler im Bericht erscheint.
- **Kopien im RAM:** Wie in N10 beschrieben. Ein Restrisiko, das die Spec über 1.4 bereits als Annahme führt.
- **Passphrase als Schwachstelle:** Eine schwache Passphrase macht die gesamte Kette angreifbar. Argon2id-Parameter müssen festgelegt werden.
- **Leistung:** Zwei AES-GCM-SIV-Operationen je Dokument plus Index-Entschlüsselung bei Suche. Nicht gemessen.

## 7. Empfehlung

1. **Jetzt:** AP8 und die Sofortmaßnahmen aus Abschnitt 5. Das beseitigt die falsche Compliance-Aussage ohne Architekturrisiko.
2. **Vor der Implementierung:** Entscheidungen AP0 (4.1 bis 4.4), insbesondere Schlüsselquelle und Indexstrategie. Ohne diese Entscheidungen ist keine Klasse K oder höher erreichbar.
3. **Umsetzung:** AP1 bis AP7 strikt in dieser Reihenfolge. Jedes AP mit eigenem Testlauf und Messung.
4. **Spec-Korrekturen:** HKDF durch den tatsächlichen Ableitungsweg ersetzen oder den Code anpassen. Die `&str`-Variante des Widerrufs streichen. T-DEL-2 und T-DEL-3 präzisieren.

**Fazit:** Die Spec ist umsetzbar. Die bestehende Löschung darf nicht als Grundlage dienen, weil die Schlüsselverwaltung nicht persistent und nicht verdrahtet ist. Die Aufwände liegen vor allem im Schlüsselmanagement (AP1) und in der Indexstrategie (AP7), nicht in der Krypto-Logik selbst.


---


Ich untersuche jetzt systematisch. Zuerst kläre ich die öffentliche API, damit die Experimente gegen die tatsächlichen Schnittstellen laufen.

Die Compaction ist über die Facade nicht öffentlich erreichbar, nur `compact_all` auf Store-Ebene. Das ist ein Befund für T-DEL-2. Jetzt ein gezieltes Experiment mit mehreren Szenarien gegen die öffentliche Facade, temporär als Testdatei, danach entfernt.

Die Ergebnisse sind aufschlussreich. Eine Zeile (S4-Insert) fehlt in der Ausgabe, daher wiederhole ich diesen Test gezielt und ergänze die Volltextsuche sowie die Dateizuordnung.

Die Befunde sind konsistent. Parallel starte ich den Build der Erasure-Tests, um die Compile-Fehler aus dem Audit selbst zu prüfen, und sammle währenddessen die statischen Fakten.

Wichtig: `contextra-erasure` enthält ein Modul `vault/` mit `keyring.rs` und `slot.rs`. Das könnte die Schlüsselpersistenz (N1) teilweise lösen. Das prüfe ich jetzt genau.

Das `FileVault`-Modul ist eine vollständige, persistente Schlüsselablage. Ich prüfe, ob und wo es genutzt wird und wie `k_enc` abgeleitet wird.

Es gibt also mehr als eine Schlüsselimplementierung. Ich prüfe Persistenz und Schlüsselherleitung im Vault-Modul, dann die Verdrahtung.

Die Vault-Persistenz nutzt `sync_data` (fsync), `k_enc` wird aber von außen übergeben und nicht abgeleitet. Die Crates `contextra-db`, `contextra` und `contextra-store` hängen nicht von `contextra-erasure` ab. Ich verifiziere das und sammle exakte Zeilennummern für die Befunde.

Der Erasure-Build bricht mit 66 Fehlern in einem Target ab. Ich zähle die Fehler pro Target genau, damit die Audit-Zahl (78) geprüft ist, und verifiziere die letzten Zeilenangaben.

Der Build stoppt beim ersten Fehler-Target. Ich starte ihn mit `--keep-going`, um die Audit-Zahl von 78 vollständig zu prüfen, und untersuche derweil die automatische Compaction, die im S2-Befund relevant ist.

Die Auto-Compaction wird bei `flush` ausgelöst, sobald eine Schwelle erreicht ist. Ich ermittle die Default-Werte und messe experimentell, ob ein gelöschter Canary nach Compaction tatsächlich verschwindet.

Der Schwellwert liegt bei 4 SSTables. Jetzt messe ich das Verhalten über mehrere Flush-Zyklen: Der Canary wird eingefügt, gelöscht, und ich prüfe nach jedem Schritt, ob er noch auf dem Datenträger liegt.

Die Messung zeigt etwas Unerwartetes: Auch bei 6 SSTables löst kein Flush eine Compaction aus, der Canary bleibt erhalten. Ich untersuche, warum, indem ich prüfe, welcher Pfad bei `flush` wirklich aufgerufen wird.

Das ist ein zentraler Befund. `maybe_compact` ist `#[doc(hidden)]` und hat außer einem Test-Helper keinen Aufrufer. Ich prüfe, ob es eine Hintergrundschleife (`run_loop`) gibt, die die Compaction sonst auslöst.

Es gibt eine Hintergrundschleife, die in `startup.rs` gestartet wird. Ich prüfe ihr Intervall und die Startbedingung.

Die Compaction läuft also zeitgesteuert im Hintergrund über `check_interval`. Mein Test war mit 0,2 s zu schnell. Ich ermittle den Default und wiederhole die Messung mit Wartezeit.

Das Default-Intervall beträgt 30 s. Ich wiederhole die Messung mit Wartezeiten, damit die Hintergrund-Compaction tatsächlich läuft.

Die Hintergrund-Compaction entfernt den Canary nach rund 30 s, und die SSTables werden zu einer zusammengeführt. Offen ist noch der Fall unterhalb der Schwelle: Bleibt der Wert erhalten, wenn weniger als 4 SSTables existieren?

Unterhalb der Schwelle bleibt der gelöschte Canary über 70 s bestehen. Der Erasure-Build mit `--keep-going` ist mit Exit 1 abgebrochen. Ich prüfe, warum.

`cargo test` kennt `--keep-going` nicht. Der Audit verwendete `cargo check`, das es unterstützt. Ich wiederhole den Check so, um die Fehler pro Target zu zählen.

Die Zahl ist verifiziert: 66 + 12 = **78 Fehler** in zwei Proof-Targets. Das stimmt exakt mit dem Audit überein. Abschließend prüfe ich, ob Volltext und Vektoren nach dem Löschen im Klartext auf dem Datenträger verbleiben. Das ist der Punkt Klasse I.

Der Volltext-Klartext verbleibt nach Insert **und** Delete auf der Platte. Die Vektoren sind nicht als f32-Klartext auffindbar. Ich prüfe, wie der Vektorindex persistiert wird, damit die Aussage belastbar ist.

Das Inventar zeigt zwei Details: eine Datei `.wal_integrity_key` (32 Byte) liegt unverschlüsselt im Datenverzeichnis, und es gibt keine separate Vektordatei. Ich prüfe den Verschlüsselungsmodus auf dieselben Dateien und die verwendete KDF.

Der Argon2id-Pfad liegt in `contextra-crypto/src/kdf.rs`. Ich prüfe, ob der Passphrase-Modus der Storage-Schicht diesen KDF nutzt, und wo `SALT` geschrieben wird.

`contextra-crypto` hat einen vollständigen Argon2id-KDF mit OWASP-Parametern, der Store ruft ihn aber nicht auf. Ich ermittle, welche Ableitung der Store für die Passphrase tatsächlich nutzt.

Das ist ein wesentlicher Befund. Die Store-Verschlüsselung nutzt `KeyManager::try_new`, also **HKDF-SHA256 direkt aus der Passphrase, ohne Key-Stretching**. Der Argon2-Pfad `try_new_with_kdf` existiert. Ich prüfe, ob der Store ihn irgendwo verwendet.

Der Store verletzt eine dokumentierte Regel des eigenen Crates. Die Doku in `crypto.rs:86` schreibt für menschliche Passphrasen `try_new_with_kdf` vor. Ich prüfe außerdem die Nonce-Erzeugung, weil die Spec (1.5) zählerbasierte Nonces ausschließt.

Die Nonce-Konstruktion (4 Byte Zufallspräfix pro Öffnung plus 8 Byte Zähler ab 1) ist ein relevanter Befund für Spec-Punkt 1.5. AES-GCM-SIV mildert Nonce-Wiederverwendung ab. Den letzten offenen Punkt N4 prüfe ich empirisch: Öffnet der Store ein `EnterpriseRegulated`-Profil ohne Verschlüsselung, und landet der Canary dann auf der Platte?

Das `CollectionProfile` mit `kv_delete_mode` scheint am Builder vorbei zu laufen. Ich prüfe, ob es irgendwo an eine Collection oder den Builder gebunden wird.

Das ist ein relevanter Fund: Der MCP-Handler validiert das Profil, verwirft es danach und antwortet trotzdem mit `"kv_delete_mode": "CryptoShred"`. Ich korrigiere außerdem eine Zeilenangabe aus dem Vorbericht. `EnterpriseRegulated` steht in Zeile 188, nicht 177 (177 ist `EnterpriseShared`). Dazu sammle ich die exakten Zeilen aller Befunde, bevor ich den Repo-Zustand zurücksetze.

Die Epoch-Variante (`encrypt_envelope_record_epoch`) ist ein zweiter, paralleler API-Pfad. Ich prüfe, ob sie `derive_subkey` nutzt, und sichere danach den Repo-Zustand.

Der Epoch-Pfad nutzt ebenfalls einen zufälligen KEK und denselben, nicht persistierten Master-KEK. `derive_subkey` wird auch dort nicht verwendet. Das Repo ist sauber. Zum Schluss prüfe ich zwei Testlaufergebnisse, die der Bericht als Fakten braucht: die Profil-Matrix und den im Audit als fehlerhaft gemeldeten Compliance-Test.

Ich konnte die Datei nicht erzeugen, weil das Tool-Limit erreicht ist. Der Bericht steht daher vollständig hier. Ich kann ihn auf Wunsch in einem späteren Schritt als `.md`-Datei ablegen.

# Feingranulare Analyse: Löschung und Verschlüsselung in Contextra

_Repository: `tfufuz1/contextra`, Shallow-Klon, Commit `8c4532ba` · Toolchain: Rust 1.89.0 (apt) · Stand: 16.11.2026_

## 0. Methodik und Grenzen

- **Ausgeführt:** Tests und Experimente gegen den Build, statische Code-Lektüre mit Zeilennummern, Dateiinventar des Datenverzeichnisses.
- **Experimente:** temporäre Testdateien (`spike_fine.rs`, `spike_keys.rs`), nach dem Lauf entfernt. `git status` ist leer.
- **Nicht ausgeführt:** Der `contextra`-Gesamttest, Store- und MCP-Tests aus dem Audit, sowie ein Lauf der Vault-Module. Aussagen dazu stammen aus dem Audit oder der Code-Lektüre.

## 1. Korrekturen zum Vorbericht

| # | Vorbericht | Korrektur |
|---|---|---|
| K1 | `EnterpriseRegulated` in `collection_profile.rs:177` | Zeile **188**. Zeile 177 ist `EnterpriseShared`. |
| K2 | `derive_subkey` in `kv_shredding.rs:85` | Zeile **125** |
| K3 | Zeilenangaben mit `~` (`~520`, `~345`) | Exakt: `encrypt_envelope_record` ab 526, `get_or_derive` ab 340 |
| K4 | Argon2id als Empfehlung für Passphrase | Der Store nutzt derzeit **kein** Argon2 (siehe F-K3). |

## 2. Build- und Testbasis

| ID | Fakt | Beleg |
|---|---|---|
| B1 | `T-DEL-1` (`test_tdel_1_canary_bytes_erased_after_compaction`) schlägt fehl: `left: 1, right: 0` | `cargo test -p contextra --test deletion_e2e` |
| B2 | `test_tdel_3_drop_collection_e2e` ist grün, prüft aber keinen Löschnachweis | Testrumpf ohne Nachweis-Assertion |
| B3 | `cargo check -p contextra-erasure --all-targets --keep-going`: **78 Fehler** in zwei Targets: `proof_crash_resume_matrix` (66) und `proof_fail_closed_and_consistency` (12). Der Audit-Wert 78 ist damit bestätigt. | `/tmp/erasure_check.log` |
| B4 | `contextra-db`, `contextra`, `contextra-store`, `contextra-engine`, `contextra-mcp` hängen **nicht** von `contextra-erasure` ab. | `Cargo.toml`-Grep |
| B5 | `contextra-erasure` ist nicht in `default-members` | `Cargo.toml` |
| B6 | `collection_profile_matrix`: 1 bestanden, 0 fehlgeschlagen. Die Matrix prüft nur die **Validierung** der Kombinationen, nicht die Wirkung. | `cargo test -p contextra --test collection_profile_matrix` |

## 3. Architektur-Inventar der Schlüsselkomponenten

| Komponente | Ort | Persistent | Im Löschpfad verdrahtet | Ableitung |
|---|---|---|---|---|
| `KeyRegistry` | `contextra-erasure/src/kv_shredding.rs` | **nein** (RAM) | **nein** | Master-KEK zufällig (`OsRng`, Zeile 244), KEK zufällig, DEK zufällig. `derive_subkey` (Zeile 125) nur in Tests. |
| `RevocationLog` | `contextra-erasure/src/revocation_log.rs` | ja (`write_atomic`) | **nein**, nur in Tests über `with_revocation_log` (`shredding_envelope_test.rs:85`) | Ed25519-Hash-Kette |
| `FileVault` / `VaultKeyring` | `contextra-erasure/src/vault/` (3194 Zeilen) | ja (`sync_data`, `durable.rs:93, 603`) | **nein**, nur Erasure-Tests und Bench | `k_enc` wird von außen übergeben, nicht abgeleitet |
| `contextra-vault` | `crates/contextra-vault` | — | **nein**, aus Workspace ausgeschlossen (`exclude`) | — |
| `KeyManager` (Store-Verschlüsselung) | `contextra-crypto/src/crypto.rs` | über `SALT` | ja | HKDF-SHA256 direkt aus Passphrase (Zeile 105) |
| Argon2id-Pfad | `contextra-crypto/src/crypto.rs` `try_new_with_kdf`, `kdf.rs` | — | **nein**, kein Produktionsaufrufer | OWASP-Parameter |

## 4. Messungen am Datenträger

Alle Messungen mit `ContextraConfig { dimension: 4, consolidation_enabled: false, .. }`, Canary `CANARY_SENSITIVE_DATA_987654321_E2E_DELETE_TEST`.

| ID | Szenario | Ergebnis |
|---|---|---|
| S1 | Insert (Metadaten mit Canary) + `flush` | Canary in **1** Datei: `sst-…15-000000.sst`. Plaintext ab dem Schreiben. |
| S2 | `delete` + `flush` | Canary weiterhin in derselben SST. `get` liefert `Ok(false)` (Session und nach Reopen). |
| S3 | Vektorsuche nach `delete` | Leeres Ergebnis (Session und nach Reopen). API-Ebene korrekt. |
| S4 | Verschlüsselter Store (`with_encryption_passphrase`): Insert + `flush` | Canary in **0** Dateien. Store-Verschlüsselung wirkt. |
| S5 | Verschlüsselter Store: `delete` + `flush` | Canary weiterhin 0. Dateien: 6 → 8. Die alte Version liegt verschlüsselt auf dem Datenträger und ist mit der Passphrase lesbar. |
| S6 | Falsche Passphrase bzw. keine Passphrase bei verschlüsseltem Store | Falsche Passphrase: `Decryption failed: aead::Error`. Keine Passphrase: `Checksum mismatch`. Die beiden Fehler **verraten den Verschlüsselungszustand**. |
| S7 | `search_text` vor/nach `delete` | 1 → 0 Treffer. API-Ebene korrekt. |
| S8 | Sechs Flushes ohne Wartezeit | Canary bleibt in 1 Datei, 6 SSTables. Keine Compaction. |
| S9 | Insert, Delete, 3 Filler-Flushes, dann Wartezeit | Nach Wartezeit (≤35 s): Canary in **0** Dateien, 1 SSTable. Compaction wirkt. |
| S10 | 2 SSTables (Insert + Delete), 70 s Wartezeit | Canary bleibt, 2 SSTables. Unterhalb der Schwelle keine Compaction. |
| S11 | Volltext-Token und Vektor-Rohbytes auf Platte | Text `Geheimer Inhalt` in SST **vor und nach** Delete gefunden. Vektor-Rohbytes (f32) nicht gefunden. |
| S12 | Dateiinventar nach Close (unverschlüsselt) | `.wal_integrity_key` (32 B, Klartext), `LOCK` (0 B), `MANIFEST` (159 B), `SALT` (32 B), `sst-…23-000000.sst` (121 239 B), `wal-…0.log` (0 B) |

## 5. Code-Fakten des Löschpfads

| ID | Fakt | Beleg |
|---|---|---|
| D1 | `delete_op` liest alte Werte, ruft `stage_text_delete`, `stage_graph_entity_delete`, löscht `user_key` und `doc_key`, ruft `record_keys_with_old_values` und `index.delete`. | `crates/contextra-db/src/collection/crud/delete.rs` |
| D2 | Kein Schritt ruft `KeyRegistry`, `revoke_*` oder `KvDeleteMode` auf. | Suche nach `kv_delete_mode`/`KvDeleteMode`: nur Profil und MCP-Anzeige |
| D3 | `kv_delete_mode` wird nicht ausgewertet. | Suchergebnis |

## 6. Code-Fakten Schlüssel und Kryptografie

| ID | Fakt | Beleg |
|---|---|---|
| F-K1 | `KeyRegistry::new()` erzeugt `master_kek` zufällig. Nach Neustart ist jeder Chiffretext unlesbar: `Err(Refused)` (Spike). | `kv_shredding.rs:243-244` |
| F-K2 | `revoke_record(&str)` leert `record_deks: BTreeMap<String, …>`. In diese Map wird nie eingefügt. Der Schlüssel bleibt lesbar (Spike: `Ok("LESBAR")`). Wirksam ist nur `revoke_record_num` (Zeile 443). | `kv_shredding.rs:224, 466, 475` |
| F-K3 | `KeyManager::try_new` (HKDF-SHA256 direkt aus Passphrase, Zeile 88-105) wird vom Store aufgerufen (`startup.rs:97`, `migration.rs:93`). Die Doku in Zeile 86 schreibt für menschliche Passphrasen **MUSS** `try_new_with_kdf` (Argon2id) vor. Die Vorschrift wird verletzt. | `crypto.rs:86`, `lsm/recovery/startup.rs:97` |
| F-K4 | Nonce-Schema: 4 Byte Zufallspräfix pro `KeyManager`-Instanz, 8 Byte Zähler ab 1 pro Öffnung. Die Spec (1.5) verlangt zufällige Nonces oder einen Zähler ohne Reset. Mildernd: AES-GCM-SIV ist nonce-misuse-resistant. Rechnung: Kollision des 32-Bit-Präfixes bei n Öffnungen ≈ n²/2³³, also ≈ 1,2 % bei 10⁴ Öffnungen. | `crypto.rs:14, 27-31, 115-122` |
| F-K5 | Kein AAD in `encrypt_envelope_record` (Zeile 549-551) und `encrypt_envelope_record_epoch` (832-834). Bindung nur über Registry-Lookup. | Code |
| F-K6 | Epoch-Pfad (`get_or_derive_epoch`, `encrypt_envelope_record_epoch`) ebenfalls mit zufälligem KEK und dem nicht persistenten Master-KEK. Der Vault-KeyStore nutzt diesen Pfad (`vault/key_store.rs:148, 165`). | Code |
| F-K7 | `decrypt_envelope_record` hält `dek_bytes` (Zeile 609) als `Vec<u8>` ohne `zeroize`. Der Verschlüsselungspfad nullt dagegen (Zeile 568). | Code |
| F-K8 | `get_or_derive` (Zeile 340-342) und `encrypt` verweigern nach `revoke_group` mit `Refused`. Eine neue Collection mit gleicher `group_id` ist unbeschreibbar. Kein Epochen-Rollover im Produktionspfad. | Code |
| F-K9 | Fehlervarianten: `Refused` für „fehlt“, „widerrufen“ und „nach Neustart“. Kein `Erased`-Fehler im Erasure-Crate. `Erased` existiert nur als `GateDecision` in `contextra-ports`. | `error.rs:78`, `ports/gate.rs:108` |
| F-K10 | `DocId::from_key` ist ein 64-Bit-BLAKE3-Abschnitt (`contextra-ids/src/lib.rs:250`). Die Geburtstagsschranke für 50 % Kollision liegt bei ≈ 5,1·10⁹ Dokumenten pro Collection (Rechnung). | Code |
| F-K11 | `derive_subkey` nutzt BLAKE3-`derive_key`, nicht HKDF. Die Spec nennt HKDF. | `kv_shredding.rs:125-131` |

## 7. Code-Fakten Profile, API und Compaction

| ID | Fakt | Beleg |
|---|---|---|
| P1 | `EnterpriseRegulated` (Zeile 188) setzt `kv_delete_mode: CryptoShred`, `PerformanceProfile::Compliance`. `validate()` prüft nur die Kombination mit `TombstoneOnly` und `integrity_proof_active`. Keine Verschlüsselungspflicht. | `collection_profile.rs:186-188`, `validate()` |
| P2 | Der MCP-Handler `create_collection` ruft `tier.resolve()`, validiert, verwirft das Profil und öffnet die Collection mit `self.db.collection(name)`. Die Antwort enthält trotzdem `"kv_delete_mode": "CryptoShred"`. Der Client erhält damit eine falsche Aussage. | `contextra-mcp/src/tools_crud.rs:55, 85, 92` |
| P3 | Der Builder nimmt nur `PerformanceProfile`, nicht `CollectionProfile`. Das Collection-Profil mit `kv_delete_mode` ist öffentlich nicht an eine Collection gebunden. | `contextra/src/builder.rs:257-266` |
| P4 | Compaction ist über die Facade nicht öffentlich auslösbar. `compact_all` existiert nur auf Store-Ebene. Der Test-Helper `maybe_compact_for_test` ist nicht öffentlich. | `contextra-store/src/compaction/engine.rs:226`, `lsm/engine.rs:377` |
| P5 | Die automatische Compaction ist eine Hintergrundschleife `run_loop` mit `check_interval` = 30 s (Default), gestartet in `lsm/recovery/startup.rs:466`. Schwellwert: `min(min_sstables_per_tier, l0_compaction_trigger)` = 4 SSTables (`config.rs:85-86`). | Code, S9, S10 |
| P6 | `LsmStorage::maybe_compact` ist `#[doc(hidden)]`. | `lsm/flush.rs:17` |
| P7 | Standard: `encryption_passphrase: None` (`contextra-db/src/lib.rs:309`). | Code |
| P8 | `.wal_integrity_key` wird im Klartext neben den WAL gespeichert. Der Code dokumentiert das selbst als Sicherheitshinweis. | `contextra-store/src/wal/hmac.rs:224-249` |
| P9 | Vektorindex: HNSW mit SQ8-Quantisierung und mmap-Persistenz (`hnsw/types/persist.rs`). Keine eigene Datei im Inventar (S12), die Ablageform wurde nicht abschließend ermittelt. | `contextra-vector/src/hnsw/types/persist.rs` |

## 8. Status der Spec-Testkatalog-Einträge

| Kennung | Status | Fakt |
|---|---|---|
| T-DEL-1 | **rot** | B1, verifiziert |
| T-DEL-2 | **fehlt in Form** | Test heißt „after_compaction“, führt keine Compaction aus. Nach S9 entfernt eine Compaction den Canary, aber nur bei Schwelle und Timer (P5). Ohne Wartezeit schlägt der Test fehl. |
| T-DEL-3 | **Abweichung bestätigt** | B2. Der Test prüft nur das Droppen. |
| T-DEL-4 | **API-Ebene grün, ohne Fehlerkategorie** | Nach Reopen `Ok(false)` (S2). Ein `Erased`-Fehler existiert nicht. |
| T-DEL-5 | **nicht implementiert** | Matrix (B6) prüft nur Validierung. Keine Wirkung von `CryptoShred`. |
| T-DEL-6 | **API grün, Klasse I rot** | S3 und S7 leer, aber Text in SST (S11). |
| T-DEL-7 | **implizit vorhanden** | Canary-Scanner findet Muster (S1). Kein eigener Test. |
| T-DEL-8 | **nicht implementiert** | `drop_collection` ohne Assertion. |

## 9. Befundkatalog

Schwere: **K** = kritisch (Datenverlust oder falsche Löschaussage), **H** = hoch (Compliance-Aussage nicht gedeckt), **M** = mittel, **N** = niedrig.

| ID | Schwere | Befund | Belege |
|---|---|---|---|
| X1 | K | Master-KEK nicht persistent: Datenverlust nach Neustart bei Aktivierung der Schlüsselverschlüsselung | F-K1, Spike |
| X2 | K | `revoke_record(&str)` wirkungslos | F-K2, Spike |
| X3 | K | Klartext auf Platte ab Schreiben im Standardmodus | S1, S11 |
| X4 | H | `EnterpriseRegulated` und MCP melden `CryptoShred`, ohne Wirkung | P1, P2 |
| X5 | H | `KeyManager::try_new` statt dokumentiertem Argon2id-Pfad | F-K3 |
| X6 | H | Vektor- und Textindex: Tokens im Klartext nach Löschung | S11 |
| X7 | M | Gelöschte Werte bleiben bis zur Compaction-Schwelle (≥4 SSTables) und Timer bestehen | S10, P5 |
| X8 | M | Verschlüsselungszustand über Fehlermeldungen erkennbar | S6 |
| X9 | M | Keine AAD | F-K5 |
| X10 | M | Keine Epochen-Rotation, Collection-Neuanlage nach Drop blockiert | F-K8 |
| X11 | M | 78 Compile-Fehler in `contextra-erasure`-Tests | B3 |
| X12 | M | Nonce-Präfix-Kollision bei vielen Öffnungen (mit AES-GCM-SIV gemildert) | F-K4 |
| X13 | N | DEK-Kopie ohne `zeroize` | F-K7 |
| X14 | N | `.wal_integrity_key` im Klartext | P8 |
| X15 | N | 64-Bit-`DocId` | F-K10 |

## 10. Offene Punkte

- Die Vault-Persistenz (`durable.rs`, fsync-Pfad) ist nur gelesen, nicht per Experiment geprüft.
- Die 2 fehlgeschlagenen `contextra`-Tests aus dem Audit (`open_compliance_profile_succeeds_without_license`) und die 16 Store-Fehler sind nicht reproduziert.
- Die Ablageform des Vektorindex (SQ8 vs. Blockformat) ist nicht abschließend bestimmt. Die Aussage „keine Vektor-Rohbytes“ gilt nur für f32.
- Verhalten bei Compaction-Konflikten mit laufenden Snapshots (`min_active_seqno`) ist nicht getestet.
- Die Aussage „Embedding-Inversion möglich“ aus dem Vorbericht ist eine Annahme aus der Literatur und kein Messwert.

## 11. Reproduktion

```bash
export PATH=/usr/lib/rust-1.89/bin:$PATH
cd tfufuz1/contextra
cargo test -p contextra --test deletion_e2e          # B1
cargo test -p contextra --test collection_profile_matrix   # B6
cargo check -p contextra-erasure --all-targets --keep-going   # B3
# Experimente: Testdatei unter crates/contextra/tests/ anlegen, Szenarien S1 bis S12 wie oben,
# danach entfernen. Spike-Code für F-K1 und F-K2 unter crates/contextra-erasure/examples/.
```
