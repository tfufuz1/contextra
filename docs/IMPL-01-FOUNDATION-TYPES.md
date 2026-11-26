# IMPL-01 — Fundament-Typen, Fehlerkatalog, Format-Header, Ablösung von `contextra-core`

**Phase:** P0 · **Ring:** L0 · **SOLL:** SPEC §2.2, §2.3, §2.6–2.8, §8.1, §11.5, Anhang B, Anhang C
**Crates:** `contextra-types` (primär), `contextra-wire`, `contextra-ports`, `contextra-core` (Rückbau)

## 1. Ziel

Alle Newtypes und Enums, auf die spätere Pakete aufsetzen, existieren als stabile, serialisierbare, getestete Typen in L0. Kein anderes Paket darf eigene ID-/Trust-/Handle-Typen definieren. `contextra-core` verschwindet.

## 2. IST

- `contextra-types/src/types/domain/ids.rs`: `TenantId`, `CollectionId`, `DocId(u128)` (u64 ohne Feature `docid-128`), `EntityId`, `TxId`; `DocId::from_key` hasht den String-Key (ADR-016/082).
- Fehler: `ContextraError` (`error.rs`, ~500 Zeilen), `ContextraErrorDto`.
- Keine Typen für `Gid`, `CellId`, `CellKey`, `ChunkId`, `Hlc`, `Seq`, `Lsn`, `Handle`, `TrustTier`, `TaintFlags`, `BeliefBeta`, `GroundingLevel`, `NsId`, `BranchId`, `SegmentId`.
- `contextra-core`: reine Re-Export-Fassade, 301 Dateien referenzieren `contextra_core::…`.
- `contextra-types` hängt an `bincode` (nicht kanonisch/rkyv) und `serde_json`.

Reproduktion: `grep -rEc 'contextra_core' crates --include=*.rs | grep -v ':0' | wc -l`

## 3. SOLL-Typen (Lieferumfang)

Neues Modul `contextra-types/src/ids2.rs` (Name final: `ids`, nach Entfernung der Alt-IDs, siehe §6).

```rust
#[repr(transparent)] #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Gid(pub u64);           // dicht, monoton pro Repo, NIE wiederverwendet; 0 = ungültig
pub struct NsId(pub u32);
pub struct BranchId(pub u32);
pub struct SegmentId(pub u64);
pub struct Lsn(pub u64);           // WAL-Position, strikt monoton
pub struct Seq(pub u64);           // globaler Commit-Zähler
pub struct Hlc(pub u64);           // 48 bit ms | 16 bit logisch
pub struct CellKey(pub [u8; 16]);
pub struct CellId { pub ns: NsId, pub key: CellKey }
pub struct ChunkId(pub [u8; 32]);  // BLAKE3-256; Phantom-Variante ContentHash<T>
pub struct LocalDocId(pub u32);    // segmentlokal; Name im Code: `DocId32` bis Alt-DocId entfernt ist
```

Regeln:

1. Alle Newtypes: `#[repr(transparent)]`, `Debug` ohne Inhaltsleck, `serde` nur über kanonische Codecs (§5), keine `From<u64>`-Konvertierung ohne Namen (`Gid::from_raw`, `Seq::from_raw`), damit Verwechslungen am Aufrufort sichtbar sind.
2. `Hlc`-Arithmetik: `Hlc::tick(&mut self, wall_ms: u64) -> Hlc` und `Hlc::observe(&mut self, remote: Hlc, wall_ms: u64) -> Result<Hlc, HlcDrift>`; Überlauf des 16-bit-Zählers erhöht ms, nie Panik. Uhr kommt **ausschließlich** über den `Clock`-Port (existiert: `contextra-ports/src/clock.rs`).
3. `Gid`-Vergabe ist **nicht** Teil von L0 (gehört zu IMPL-05, Alias-Baum); L0 liefert nur den Typ und `Gid::NONE`.

### 3.1 Handle (SPEC-02 §5)

Neues Modul in `contextra-types`, Kryptofunktion bleibt in `contextra-crypto` (L0), Typ kennt nur Bytes:

```rust
pub struct Handle { pub gid: Gid, pub mac: [u8; 2] }  // Textform "c:<crockford>-<mac2>"
impl Handle {
    pub fn encode(&self) -> String;                       // Crockford-Base32, ≥ 4 Zeichen, Kleinbuchstaben
    pub fn parse(s: &str) -> Result<ParsedHandle, HandleParseError>; // ParsedHandle { gid, mac_claimed }
}
```
