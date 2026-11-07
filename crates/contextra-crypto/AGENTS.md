# AGENTS.md — `contextra-crypto`

> **Ring 0** · Klasse `[K]` · Reifegrad `beta` · Unsafe: nein · Rolle: Reiner Kryptografiekern: AEAD, KDF, WAL-HMAC, Anti-Tamper. **Soll:** ohne Lösch-/Nachweislogik.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-crypto --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` |
| Features | `kv-encryption`, `cloud-egress-guard`, `test-utils` (dev-Selbstreferenz mit `test-utils`) |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- Kanten Ist = Soll, **inhaltlich aber nicht entkoppelt**: das Crate enthält noch die Löschdomäne (88 `DeletionProof`-, 31 Shredding-/Revocation-Treffer).
- `KmsProvider` liegt noch in `wal_crypto.rs:32` (Soll: `ports`).
- Konsumenten: `store`, `vector` (nur `wal_crypto`), `engine`, `erasure`, `kvcache`, `infer-candle`, `mcp`.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `aead.rs`, `kdf.rs`, `secret.rs`, `dek.rs`, `crypto.rs`, `error.rs` | **Verbleibt:** AEAD, Argon2id/HKDF, `SecretKey` (zeroize), `KeyManager`, `derive_dek` |
| `wal_crypto.rs`, `wal_completeness.rs`, `anti_tamper.rs`, `audit_chain.rs` | **Verbleibt:** WAL-HMAC-Kette, Vollständigkeit, Manipulationserkennung, HMAC-Audit-Kette |
| `deletion_proof/` (6 Dateien), `ed25519_proof.rs`, `kv_shredding.rs`, `revocation_log.rs` | **Wandert nach `contextra-erasure` (PLAN 3.1)** |
| `kv_cipher.rs`, `kv_segment/` | KV-Cache-Verschlüsselung; PLAN nennt sie nicht, Konsument `kvcache`: Zuordnung per ADR |

## 4. Verträge & Invarianten

1. **Nie eigene Primitive bauen (EP12).** Nur geprüfte Bibliotheken (`aes-gcm-siv`, `argon2`, `hkdf`, `hmac`, `ed25519-dalek`, `blake3`); Konstantzeit-Vergleich über `subtle`.
2. **KDF (INV-KDF-1):** Argon2id mit zufälligem, persistiertem Salt (`KdfHeader`), m ≥ 64 MiB, t ≥ 3. Kein HKDF direkt auf Passphrase, kein fester Salt.
3. **Schlüssel:** jeder Schlüssel hat genau einen Zweck und ein eigenes HKDF-Label (INV-KEYPURPOSE-1, `governance/key_purposes.toml`). Schlüsselmaterial implementiert `Zeroize`/`ZeroizeOnDrop`; keine API gibt es im Klartext zurück (INV-SCAN-1).
4. **I/O-frei.** Persistenz nur über Ports; keine `std::fs`.
5. **Zufall:** `rand::thread_rng` nur für Schlüssel/Salt/Nonce (Ausnahme ADR-098, `determinism-allow.toml`).
6. **Verbot für Neucode:** keine neuen `DeletionProof`-, Ed25519-Bericht-, Shredding- oder Revocation-Typen hier (Root-§5).

## 5. Ports

**Implementiert:** `KmsProvider` (lokal). **Konsumiert:** `ports` (u. a. `Clock`, `Rng`).

## 6. Nebenläufigkeit & Performance

- Zustandslos bis auf `KeyManager`; Locks (`parking_lot`) kurz, nie über Fremdaufrufe.
- Hot Path: AEAD pro Block. Benchmarks: `benches/crypto_benchmarks`; `deletion_proof_latency_bench` ist Erasure-Last und wandert mit.

## 7. Anti-Patterns

- Konstanten-Salt oder Passphrase→Schlüssel ohne Argon2id.
- Nonce-Wiederverwendung (Nonce-Quelle nur aus CSPRNG/Zähler mit Test).
- `==` auf MACs/Tags.
- Neue Berichts-/Beweistypen in `crypto` ablegen.

## 8. Tests & Messung

```bash
cargo test -p contextra-crypto --locked
cargo clippy -p contextra-crypto --all-targets -- -D warnings
cargo test -p contextra-crypto --locked --features test-utils
RUSTFLAGS="--cfg loom" cargo test -p contextra-crypto --locked
cargo mutants --config mutants-guarantees.toml -p contextra-crypto
```

- Testvektoren (RFC/KAT), Property-Tests (Roundtrip, Manipulation ⇒ Fehler), Fuzzing (WAL-Replay/Proof-Deserialisierung, Ziel `fuzz/`), `loom` für Zeroize-/Zugriffspfade.
- `mutants-guarantees.toml` verweist noch auf `kv_shredding.rs`/`deletion_proof.rs` (G2): nach Verlagerung auf `contextra-erasure` umstellen.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 3.1 | `deletion_proof/`, `ed25519_proof.rs`, `kv_shredding.rs`, `revocation_log.rs` nach `contextra-erasure`; danach Crate-README, Budget 3.200 Zeilen |
| 2.x | `KmsProvider` nach `ports` |
| 5.3 | `vector → crypto`-Kante (WAL-HMAC) lösen |


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-crypto:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-crypto:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-crypto:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| `KV` | 5 | `ANCHOR[contextra-crypto:KV-006]` |
| `AUDIT` | 4 | `ANCHOR[contextra-crypto:AUDIT-005]` |
| `CRY` | 2 | `ANCHOR[contextra-crypto:CRY-003]` |
| `KDF` | 2 | `ANCHOR[contextra-crypto:KDF-003]` |
| `WAL` | 2 | `ANCHOR[contextra-crypto:WAL-003]` |
| `PROOF` | 1 | `ANCHOR[contextra-crypto:PROOF-002]` |
| `REV` | 1 | `ANCHOR[contextra-crypto:REV-002]` |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-crypto:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-crypto` durch (`find crates/contextra-crypto -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

- **Modul** (`//!`): Zweck, Ring-Zugehörigkeit, Nutzer, wichtigste Typen, Invarianten, Verweis auf Nachbarmodule.
- **Öffentliche Items:** Zweck in einem Satz, dann bei Bedarf `# Errors` (welcher Fehler wann), `# Panics` nur wenn der Code abbrechen kann (sollte nach Zero-Panic-Regel nicht vorkommen: dann `FIXME`), `# Safety` bei `unsafe fn`, Vorbedingungen, Nachbedingungen, Nebenwirkungen (I/O, Zeit, Zufall, IDs). Zeit, Zufall und IDs laufen nur über `Clock`/`Rng`/`IdGen`; steht es im Code, steht es im Kommentar.
- **Nicht-triviale private Funktionen:** Warum sie existieren und was sie garantieren.
- **Nebenläufigkeit:** Sperren, atomare Operationen, Reihenfolge, Warum kein Deadlock.
- **Persistenz und Fehler:** Wo Daten WAL oder Dateisystem-Port passieren; `fsync`-Fehler werden propagiert.
- **Tests:** Ein Satz je Testfunktion oder -block: welche Invariante oder welches Verhalten bewiesen wird.
- **Architektur:** Welche Ports die Crate nutzt/anbietet, welche Abhängigkeitsrichtung gilt (DAG, `may_depend_on`), was bewusst *nicht* hier liegt.

## Rustdoc-Fallen (jeder Punkt bricht sonst `cargo clippy -D warnings` oder `cargo doc`)

- Kein `///` vor Nicht-Items (`unused_doc_comments`); keine Leerzeile zwischen `///` und Item (`empty_line_after_doc_comments`).
- Fortsetzungszeilen in Listen und Zitaten korrekt einrücken (`doc_lazy_continuation`). **Vier Leerzeichen Einrückung in Doc-Text erzeugen einen Doctest**; Code immer in Fences mit Sprache. Beispiele, die nicht kompilieren oder laufen sollen: `ignore` oder `text`, begründet.
- Intra-Doc-Links `[`Typ`]` nur auf existierende, erreichbare Items; spitze Klammern im Text (`Vec<u8>`) in Backticks setzen, sonst „unclosed HTML tag“.
- `// SAFETY:` ohne Leerzeile und ohne dazwischenliegende Zeilen direkt über dem unsafe-Ausdruck.
- Doc-Beispiele müssen gegen die tatsächliche Signatur kompilieren; im Zweifel `no_run`/`ignore` mit Grund.