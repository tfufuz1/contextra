# SP-CODEC — `codec` (Kodierung, Prüfsummen, Vergleich, Siegelrahmen)

| | |
|---|---|
| Modul | `contextra-storage/src/codec/` |
| Besitzt | `varint`, `tags`, `checksums`, `comparator` (`cmp`), `seal` (SealFrame, AAD-Builder, KDF-Labels), Dateikopf, `FORMAT`- und `KEYS`-Codec |
| Konsumiert | SP-PORTS (`Cipher`, `Kdf`, `Mac`, `Checksum`) |
| Spec-Bezug | R9.1 §4.2, §5.2–§5.4 (FORMAT/KEYS), INV-FMT-1, INV-SEAL-1, INV-PRIV-1 |
| Phase | P0 (Primitive), P1 (Seal), G-F (Freeze) |

## 1 Auftrag und Abgrenzung
Reine, deterministische Funktionen über Bytes. **Darf nicht:** I/O, Zeit, Zufall (Nonce kommt als Parameter), Schlüsselbeschaffung (nur `&Key32`), Dateilogik. Fremde Formate (Redo-Frame, Run, Catalog, Ledger) codieren die jeweiligen Komponenten selbst, **nutzen** aber diese Primitive.

## 2 Schnittstelle
```rust
pub fn varint_put(out: &mut Vec<u8>, v: u64);  pub fn varint_get(b: &[u8]) -> Result<(u64, usize), FormatError>; // kanonisch, ≤10 Byte
pub fn tag(stamp: Stamp, kind: Kind) -> u64;   pub fn untag(t: u64) -> (Stamp, Kind);                       // Merge=6 ⇒ Unsupported
pub fn cmp_entry(a: &EntryKey, b: &EntryKey) -> Ordering; // (cluster, branch, user_key bytelexikografisch, tag ↓) – kein memcmp über Konkatenation
pub struct SealCtx<'a> { pub source: SealSource<'a>, pub aad: Aad<'a> }
pub fn seal(cipher:&dyn Cipher, kdf:&dyn Kdf, ctx:&SealCtx, nonce:[u8;24], pt:&[u8], out:&mut Vec<u8>) -> Result<(), SealError>;
pub fn unseal(cipher:&dyn Cipher, kdf:&dyn Kdf, ctx:&SealCtx, frame:&[u8], out:&mut Zeroizing<Vec<u8>>) -> Result<(), SealError>;
pub enum Aad<'a> { Block{instance,space: SpaceId /* canonical u64 */,cluster,key_epoch,block_uid}, RedoOp{instance,space: SpaceId /* canonical u64 */,cluster,key_epoch,op_uid,op_index}, BlobRecord{instance,space: SpaceId /* canonical u64 */,cluster,key_epoch,record_uid}, … }
```
## 3 Regeln
1. **Kanonizität (INV-FMT-1):** nicht-kanonisches varint, Überlängen, reservierte Bits ≠ 0, unbekannte Tags ⇒ `FormatError`, nie „tolerant lesen“.
2. **Längen vor Allokation prüfen (P-4):** jede `Vec::with_capacity` aus Wire-Länge nur nach Vergleich mit Restpuffer **und** konfiguriertem Limit.
3. **Eine Siegelform (L14):** `SealFrame := key_epoch u32 | nonce[24] | ct | tag[16]`; Cipher-Aufruf ist direkt `XChaCha20-Poly1305(quelle, nonce, AAD)`. Die Quelle ist bereits auf Schutzdomäne und Epoche begrenzt; es gibt keine zusätzliche HKDF/HChaCha-Ableitung pro Frame. AAD bindet stabile logische Identitäten (`instance`, `SpaceId` als kanonisches `u64`, Cluster, Epoche, Block-/Op-/Record-ID), nie Datei-, Blob-ID oder Offset ⇒ unveränderte Frames sind datei-/positionsunabhängig rohkopierbar. Identitätsbindung verhindert Cross-Cluster-Replay (Property-Test `rawcopy_roundtrip` plus AAD-Replay-Tests).
4. `Authenticated`-Modus: Nutzdaten bleiben Klartext und werden mit den ersten 16 Bytes des HMAC-SHA256-Tags authentisiert; Vergleich erfolgt constant-time. MAC-Eingabe ist kanonisch längencodiert und enthält feste Domain `ctxs-auth-v1`, Modus, Nonce, AAD und Payload. Moduswechsel pro Space ist unveränderlich. `Plain`: CRC32C ohne Authentizität. `personal`-Daten dürfen ausschließlich `Encrypted` sein.
5. Keine persistierten Inhalts-Hashes (INV-PRIV-1): `blake3` im Codec nur für Metadaten-Prüfsummen (Dateikopf, Manifest), nie über Nutzdaten; Lint-Test sucht `blake3::hash` auf Datenpfaden.
6. **Nonce-Politik:** 24-Byte-Zufallsnonce aus `Entropy`; Nonce-Kollisionsbudget dokumentieren (2⁻⁸⁰ bei ≤ 2³² Siegeln je Quelle; Rotation der Epoche erzwingt neue Quelle).
7. `FORMAT`: unveränderlich, MAC über RootKey-abgeleiteten Schlüssel; `KEYS`: RootKey (KEK-umhüllt), Space-DEKs (RootKey-umhüllt); **kein Clusterschlüssel** (INV-KEY-1).

## 4 Fehlerverhalten
`FormatError` (F-5/F-4) mit Offset-Kontext ohne Inhalte; AEAD-Fehler ⇒ `SealError::Auth` (F-4), nie Teilausgabe.

## 5 Tests
Proptest: `varint` Rundreise + Nicht-Kanonizität; `cmp` ≡ Referenz (Tupelordnung) inkl. Präfix-Fälle (`"a"` < `"a\0"`); `seal/unseal` Rundreise, Bit-Flip ⇒ `Auth`; AAD-Vertauschung ⇒ `Auth`; Rohkopie-Eigenschaft; `miri` auf gesamtes Modul; Fuzz-Ziel `decode_*`.

## 6 Lücken / Errata
- **E-07** v1 verwendet XChaCha20-Poly1305 direkt; keine zusätzliche Subkey-KDF oder gruppengeteilte Subkeys. HKDF ist nur für Root-/Space-/Integritätsschlüssel mit festen Labels zulässig. H-11 misst die echte Primitive auf Zielhardware.
- Referenzcodierer für diese Primitive: SP-VERIFY (WP-15.2).

## 7 Arbeitspakete
| ID | Titel | PW | Abhängig von | Phase | Abnahme |
|---|---|---|---|---|---|
| WP-02.1 | varint, Tags, CRC32C-Wrapper, `cmp_entry` + Proptests | 2.0 | WP-01.1 | P0 | Properties grün, miri sauber |
| WP-02.2 | SealFrame, AAD-Builder, Key-Scope, Rohkopie-Eigenschaft | 2.5 | WP-01.2, WP-04.1 | P1 | Rundreise-/Bit-Flip-/Rohkopie-Tests |
| WP-02.3 | Dateikopf, `FORMAT`-/`KEYS`-Codec | 1.5 | WP-02.1, WP-02.2 | P1 | Goldene Vektoren (nach G-F) |

## 8 Abnahme
Keine `unwrap/expect/panic/indexing`; 100 % der Decoder haben Fuzz-Ziel; alle Formatfehler typisiert.

## R9.1-Synchronisierung

`derive_subkey_fast` darf im SealFrame-Pfad nicht existieren: XChaCha20-Poly1305 erhält den bereits scope-/epoch-spezifischen Schlüssel direkt, `Authenticated` verwendet domain-separiertes HMAC-SHA256 über Modus, Nonce, AAD und Payload. Die G-F-Vektoren müssen diese Byteordnung und Schlüsselwahl unabhängig beweisen. R9.1 ändert kein Format; Referenznamen verwenden `OP*` für Recovery und `RACE-*` für Nebenläufigkeit.
