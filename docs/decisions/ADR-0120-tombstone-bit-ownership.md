# ADR-0120: Eigentümerschaft und Formatstabilität von TOMBSTONE_BIT

**Status:** Akzeptiert  
**Datum:** 2026-11-12  
**Autor:** Principal Rust Engineer  
**Bezug:** SPEC(11).md §3 INV-1, §4.1 R4, §6.1, S-30B, ADR-0121

---

## Kontext

SPEC(11).md S-30B lautet: „types/src/tombstone.rs ersatzlos entfernen (Re-Exports in types/lib.rs
Zeilen 49 und 73; R9)." Diese Anweisung ist für die Formatkonstante `TOMBSTONE_BIT` **unvollständig**.

`TOMBSTONE_BIT` (`1u64 << 63 = 0x8000_0000_0000_0000`) ist kein Domänentyp, sondern eine
**bitgenaue Persistenz-Formatkonstante**:

- In SSTable- und WAL-Einträgen von `contextra-store` codiert Bit 63 der Sequenznummer eine
  Soft-Deletion (Tombstone).
- `contextra-mvcc` nutzt dieselbe Semantik für MVCC-Snapshots (`snapshot.rs`, `ssi.rs`, `seq_log.rs`).
- `contextra-text` und `contextra-vector` lesen dasselbe Bitformat beim Scan-Schritt.

Zählung am HEAD `fcd387021`: 56 Verwendungen in `store`, `mvcc`, `text`, `vector` (außerhalb von
`tests/`-Verzeichnissen). Davon liegt die kanonische Definition bereits in `contextra-ids/src/lib.rs`
(Zeile 31), eine Duplikatdefinition in `contextra-types/src/types/domain/misc.rs` (Zeile 72) und
eine weitere in `contextra-vector/src/types.rs`.

Ein ersatzloses Entfernen aus `types` **ohne** Korrektur der Verbraucher würde:
1. Compile-Fehler in `store`, `mvcc`, `text`, `vector` erzeugen (wenn diese `contextra_types::TOMBSTONE_BIT` importieren).
2. Oder eine neue direkte Kern-Abhängigkeit von `mvcc` → `types` erzwingen (verletzt INV-8 / R1).

## Entscheidung

1. **Kanonische Quelle ist `contextra-ids`** (E0).  
   `contextra_ids::TOMBSTONE_BIT: u64 = 1 << 63` ist die einzige normative Definition.  
   Keine andere Stelle definiert den Wert neu; alle Duplikate werden durch `pub use contextra_ids::TOMBSTONE_BIT;` ersetzt.

2. **Golden-Test vor jeder Änderung** (`crates/contextra-ids/tests/golden_tombstone_bit.rs`):  
   Der Test prüft bitgenaue Wertgleichheit, LE-Serialisierung und Maskierungssemantik.  
   Er muss grün sein, bevor das erste Commit dieser Migrationsreihe gemergt wird.

3. **Deprecation in `contextra-types`** (INV-2):  
   `misc::TOMBSTONE_BIT` und `tombstone::*` erhalten `#[deprecated(since = "0.2.0", note = "Use contextra_ids::TOMBSTONE_BIT per ADR-0120")]`.  
   Der Re-Export bleibt bis S-70. Kein öffentlicher Pfad verschwindet ohne `#[deprecated]`.

4. **Migration der Verbraucher** (Schritt S-30B B):  
   `store`, `mvcc`, `text`, `vector` importieren fortan direkt `contextra_ids::TOMBSTONE_BIT`.  
   Diese Crates haben `contextra-ids` bereits als Kernabhängigkeit — keine neue Kante entsteht.

5. **Die Typen `SeqBitTombstone` und `TombstoneSemanticsCheck`** aus `tombstone.rs` werden  
   depreciert und als Re-Export aus `contextra-mvcc` oder als lokale Impl in `mvcc` gepflegt.  
   Entscheidung der Besitzerschaft obliegt ADR-0121-Folge-ADR (S-30B).

## Konsequenzen

- INV-1 (Formatgleichheit): Bit-Wert ändert sich nicht.  
- INV-2 (Pfadgleichheit): `contextra_types::TOMBSTONE_BIT` bleibt via `#[deprecated]` nutzbar bis S-70.  
- R4 (Einziger Besitzer je Konzept): `ids` ist der Besitzer von Sequenz-Konstanten.  
- G-L2: Kein E0/E1-Kern erhält eine neue Kante auf `types`.

## Prüfverfahren

```
cargo nextest run -p contextra-ids  # golden_tombstone_bit muss grün sein
sha256sum -c tests/golden/**/MANIFEST.sha256  # G-F1
cargo check --workspace --all-targets --locked   # INV-4
```
