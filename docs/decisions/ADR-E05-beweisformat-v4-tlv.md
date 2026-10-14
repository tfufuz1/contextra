# ADR-E05: beweisformat-v4-tlv

* **Status:** Akzeptiert (aus Spezifikation §13 E05)
* **Datum:** 2026-10-14

## Entscheidung
Beweisformat v4: TLV, `u16`-Tag/`u32`-Länge Big-Endian, Critical-Bit `0x8000`, Domain-Tag 28 Byte; Tag-Tabelle generiert; `layer_id: u16`; Signatur Pflicht in `Attested`; zwei Beweisstufen über `stage`/`supersedes`.

## Begründung
Entspricht `proof_v4.rs`/`canon.rs`; Offset-Tabellen veralten, Golden-Vektoren nicht.

## Verworfen
CBOR, Little-Endian-Tabellen, Festlayouts
