# acta-crypto

acta-crypto: AEAD, HKDF, HMAC, Ed25519, Sealed<P>, Schluesseltypen, HashChain, Padding.

> Teil des Acta-Workspace. Verbindliche Vorgaben: Wurzel-`CLAUDE.md` und `docs/SPEC.md`.

## Einordnung
- **Level:** 1 (`rings.toml`)
- **Größenbudget:** 4200 LOC Produktionscode (G-HUB); Überschreitung nur mit ADR
- **Aufgabe laut SPEC 3.3:** AEAD (XChaCha20-Poly1305), HKDF, HMAC, Ed25519, `Sealed<P>`, Schlüsseltypen (`Zeroizing`), `HashChain`, Auffüllung (Padding)
- **Erlaubte interne Abhängigkeiten (SPEC 3.3):** base
- **Ist-Abhängigkeiten (Cargo.toml):** acta-base
- **Externe Abhängigkeiten:** chacha20poly1305, ed25519-dalek, hkdf, hmac, sha2, subtle, thiserror, unicode-normalization, zeroize (argon2 entfällt: Argon2id nach SPEC 4.1 nur für `FileAnchor`, also in `acta-vault`)
- **Dev-Abhängigkeiten (zählen nicht für G1):** proptest
- **Referenz im Quelltext:** Level L1 · Budget 4200 LOC · Spezifikation docs/SPEC.md Kap. 4.1, 4.2, 4.5

## Status
- **Stand:** implementiert (Teil)
- **Produktionscode (ohne Tests/Leerzeilen/Kommentare):** 530 LOC von 4200 Budget
- **Umgesetzt (neu nach SPEC 4.1, 4.2, 4.5, 6.2 bzw. 2.4):**
  - `aead`: Umschlag v1 (XChaCha20-Poly1305, Nonce aus `CryptoRng`, Ebenen `Personal`/`Knowledge`/`Derived` als Typen, AAD mit Ebene, Mandant, Keyspace, Datensatz, Version). Der Trait `SealKey` ist versiegelt und liefert nur den Cipher, nie das Schlüsselmaterial.
  - `key`: Geheimnistypen `DataKey`, `SubjectKey`, `DocumentKey`, `SegmentKey`, `MasterKey`, `IndexSecret`. Ohne `Default`, `Clone`, `Debug`, `Serialize`. `Zeroizing`-Speicher.
  - `kdf`: HKDF-SHA256, DEK(v) mit Sortierung nach Tag und Dublettenprüfung, HK(v), DirKey, ClusterId, VK(t,g), K_tag(t), `SubjectTag` mit NFC-Normalisierung.
  - `pad`: Größenklassen nach 4.5 (Vielfaches von 256 bis 4096, darüber 2^(k-3)); Eigenschaften per Proptest.
  - `mac`: HMAC-SHA256 mit Konstantzeit-Vergleich. `hash`: SHA-256 und `HashChain` (mehrere Instanzen).
  - `sign`: Ed25519 mit Domänenpräfix (`acta/report/v1`, `acta/checkpoint/v1`).
  - `domain`: alle 23 Präfixe aus 4.1, geprüft auf Eindeutigkeit.
- **Abweichungen von der Spezifikation (bewusst):**
  - `seal`/`open` nehmen `K: SealKey` statt `&DataKey` (3.7). Sonst bräuchten `SegmentKey` und andere Schlüsselarten einen Umweg. Die Grenze bleibt: kein rohes Material nach außen.
  - Betroffenenzahl pro Version auf 255 begrenzt (`u8` in der DEK-Formel). Anhang A nennt bis 256. Das ist ein Widerspruch im Spec-Text, der zu klären ist.
  - `HashChain`: Genesis-Vorgänger ist der Nullwert, Eintrag = SHA-256(LEDGER ‖ chain_id ‖ prev ‖ canonical). Die Festlegung der Eingaberreihenfolge trifft die Spec nicht vollständig.
- **Festwerte und Tests:** RFC 5869 (HKDF), RFC 4231 (HMAC), RFC 8032 (Ed25519, Test 1) übernommen bzw. geprüft. Umschlag-Festwert aus PyNaCl/libsodium (XChaCha20-Poly1305). Ableitungs-Festwerte aus einer unabhängigen Python-Referenz. Mutationsprüfung: Entfernen des Klassenbytes aus der AAD und der Sortierung der DEK-Betroffenen lässt die Tests scheitern.
- **Offen:** Schlüssel `K_dedup`, Log-Schlüssel, Commitment-Schlüssel `CommitKey` (gehören zu `acta-ledger` und `acta-vault`). Argon2id in `acta-vault`. Der Gate G-CRY (kein `Default`/`Clone`/`Debug`/`Serialize` auf Geheimnissen, keine `let _ =`) ist in `xtask` noch nicht implementiert. Die Einhaltung ist hier durch Konstruktion und Prüfung von Hand erreicht.
- **Archiv-Quelle:** `archive/acta-crypto/` (siehe `archive/acta-crypto/MANIFEST.md`). Übernommen: RFC-Testvektoren aus `crypto/tests/rfc_vectors.rs` (ohne AES-GCM-SIV). Der Archivordner bleibt bestehen, bis das Crate vollständig überführt ist.

## Regeln für diesen Crate
- Abhängigkeiten nur nach unten: `level(A) > level(B)`; keine Querkanten (G1).
- Keine Alias-Re-Exports, kein `include!`, kein `#[deprecated]` (HR-6).
- Ein Fehlertyp je Crate, `thiserror`, `#[non_exhaustive]`, kein `Other(String)` (HR-4).
- Workspace-Lints gelten (`[lints] workspace = true`): kein `unwrap`, `expect`, `panic`, Indexierung mit `[]` auf Produktionspfaden.
- `unsafe` ist hier verboten (G-UNSAFE, nur `acta-sys` darf es).
- Keine Zeit- oder Zufallsquellen außerhalb der injizierten Ports `Clock` und `CryptoRng` (G3).
- Verbotene Formulierungen aus SPEC 1.6 (Liste A) nicht verwenden (G-WORD).
- Vor Abschluss: `cargo xtask gates`, `cargo test -p acta-crypto`, `cargo clippy -p acta-crypto --all-targets -- -D warnings`, `cargo fmt --all -- --check`.

## Vorgehen bei Überführung aus dem Archiv
1. Archivdateien lesen, nicht blind kopieren. Namen, Importe und Verweise auf frühere Spezifikationsstände sind auf Acta umzuschreiben (siehe `archive/INDEX.md`).
2. Jede Funktion gegen den Lebenszyklus in SPEC Kap. 19 und die Zusagen in 1.5 und 16.3 prüfen. Funktionen ohne Eintrag dort werden nicht implementiert (SPEC 0.4 Regel 3).
3. Tests mit Positivkontrolle mitführen (SPEC 16.4).
4. Nach vollständiger, grün geprüfter Überführung wird der Archiv-Ordner dieses Crates entfernt.
