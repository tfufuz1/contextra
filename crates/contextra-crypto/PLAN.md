# contextra-crypto — Implementierungsspezifikation

**Ebene:** E1 (Kern, sicherheitskritisch)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch). **Stand:** 21.11.2026, Abgleich mit SPEC v4.1. Verweist nur auf SPEC-Kapitel und IDs. Ist-Abweichungen sind markiert, ungeklärte Punkte stehen unter „Offene Punkte (Entscheidung nötig)“ in Abschnitt 6.

## 1. Zweck und Ebene
Alle kryptografischen Primitive: AEAD (AES-256-GCM-SIV), KDF (Argon2id, HKDF nur aus Wurzel), KeyTable, Anker, `RecordSealer`, `SegmentSealer`, `KeyDestroyer`, Wurzel-Backends (SPEC 15, 3.1, R4). Kryptoprimitive nur hier. Schlüsselhierarchie: SPEC 5.4 bis 5.8. Abhängigkeiten nach Anhang E: `ports`, `ids`, `env`, `durable-fs`. Ziel-Umfang: 4.000 Zeilen. Ist: laut Zielspalte 11.953 Zeilen (Altbestand).

## 2. Vertrag
Implementiert (Kapitel 3.4 und 4.4): `RecordSealer`, `SegmentSealer`, `KeyDestroyer`, `RootKeyProvider` (Wurzel-Backend). Die Traits sind in `contextra-ports` definiert (R9), Ist-Befund dort: `RecordSealer` liegt in Datei keys.rs im Crate contextra-ports, `RootKeyProvider` fehlt (siehe PLAN von `contextra-ports`, O-1).
Ist-Module, die SPEC 15 nennt: `aead.rs`, `kdf.rs`, `secret.rs`. Die übrigen Module sind unten als Ist-Abweichung markiert.

## 3. Abhängigkeiten
**Erlaubt (Anhang E):** `ports`, `ids`, `env`, `durable-fs`.
**Ist (Cargo.toml):** `contextra-ids`, `contextra-env` (erlaubt), dazu `aes-gcm-siv`, `argon2`, `hkdf`, `sha2`, `subtle`, `zeroize`, `rand`, `hmac`, `blake3`, `bincode`, `serde`.
**Ist-Abweichung:** `contextra-ports` und `contextra-durable-fs` fehlen in der Ist-Liste, obwohl Anhang E sie erlaubt und SPEC 15 sie verlangt (Ist-Befund gegen `cargo metadata` prüfen).
**Features:** `default = []` (R3). Ist-Abweichung: `std`, `tracing`, `test-utils` sind in SPEC 15 nicht genannt.

## 4. Module
| Datei | Verantwortung (Ist) | Befund |
|---|---|---|
| `src/aead.rs` | AEAD | Soll (SPEC 15) |
| `src/kdf.rs` | KDF | Soll (SPEC 15) |
| `src/secret.rs` | zeroize-Typen | Soll (SPEC 15, ohne `Debug`) |
| `src/crypto.rs`, `src/dek.rs` | Schlüsselklassen | Soll nach SPEC 5.4, zu prüfen |
| `src/kv_cipher.rs`, `src/kv_segment/*` | Verschlüsselung KV-Segmente, Eviction, Manager | SPEC 5.8, Zuordnung prüfen |
| `src/wal_crypto.rs`, `src/wal_completeness.rs` | WAL-Verschlüsselung | nicht in SPEC 15, offen |
| `src/audit_chain.rs`, `src/anti_tamper.rs` | Audit-Kette, Manipulationsschutz | nicht in SPEC 15, offen |
| `src/contract.rs` | Selbstdeklaration | REM-14 |
| `src/error.rs`, `src/lib.rs` | Fehler, Wurzel | Soll |

Soll-Module, die im Ist fehlen: `keytable` (5.5), `anchor` (5.6), `sealer` (`RecordSealer`, `SegmentSealer`), `destroyer` (`KeyDestroyer`, REQ-KEY-4), `root` (`Passphrase`, `KeyFile`; `VaultTransit` in M6), `cache` (REQ-KEY-6, REQ-CON-7).

## 5. Entfernungen
| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| REM-04 | Entfernungen laut SPEC 12 (Inhalt zu prüfen, offen) | siehe SPEC 12 |
| REM-11 | dito | siehe SPEC 12 |
| REM-35 | dito | siehe SPEC 12 |
| REM-14 | `contract.rs`, ANCHOR-Dateien (35) | `grep -rl "ANCHOR\[" crates/contextra-crypto` leer |

Die drei REM-Punkte sind in SPEC 15 genannt, ihr Inhalt steht in SPEC 12. Dieser Plan übernimmt keinen Inhalt, bis er gelesen und bestätigt ist.

## 6. Umbau
1. Soll-Module `keytable`, `anchor`, `sealer`, `destroyer`, `root`, `cache` nach SPEC 5.4 bis 5.8 bauen oder zuordnen.
2. Ist-Module außerhalb des Soll (`wal_*`, `audit_chain`, `anti_tamper`) prüfen: ob SPEC sie abdeckt.
3. `unwrap`/`expect`/`panic!` (180 Treffer in `src`) nach R6 abbauen, zuerst Produktcode.
4. Selbstdeklarationen entfernen (nach Freigabe).

**Offene Punkte (Entscheidung nötig):**
- **O-1** Inhalt von REM-04, REM-11, REM-35 aus SPEC 12 nachlesen.
- **O-2** `wal_crypto.rs`, `wal_completeness.rs`, `audit_chain.rs`, `anti_tamper.rs`, `kv_segment/`: SPEC 15 nennt sie nicht. Verbleib entscheiden.
- **O-3** Feature `test-utils`: entfernen oder in SPEC 15 aufnehmen?
- **O-4** Abhängigkeiten `hmac`, `sha2`, `subtle`, `bincode`: nicht in SPEC 15 genannt.

## 7. Tests
- T-KEY-01 bis T-KEY-06, T-ENC-03 (SPEC 15).
- Canary-Scanner T-ENC-01 (G-14).
- Ist-Befund: nicht gezählt.

## 8. Abnahme
| Gate | Ist-Befund | Soll |
|---|---|---|
| G-02 | kein `unsafe`, `forbid(unsafe_code)` in `lib.rs` | eingehalten |
| G-09 | 35 Dateien mit ANCHOR/FILE-CONTEXT, `contract.rs` | verletzt (REM-14) |
| G-12 | 1 Treffer außerhalb `env` | verletzt, Fundstelle prüfen |
| G-13 | 180 Treffer (inkl. Tests, grob) | Abbau nach R6, Ratchet nicht erhöhen |
