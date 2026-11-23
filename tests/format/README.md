# Format-Goldens (WP-2.1)

`goldens_plain.json` wird von `tools/reference/generator.py` erzeugt, nicht vom Kern.
Der Kern prüft seine Codierung gegen diese Bytes (WP-2.2, noch offen).

## Umfang

| Golden | Modus | Länge | Quelle |
|---|---|---|---|
| `format_plain` | Plain | 53 B | SPEC §10.2 (49 B vor `chk` + CRC-32C) |
| `segment_header_plain` | Plain | 36 B | SPEC §10.4 (Segmentkopf mit 4 B `chain_seed`) |

Die Primitive (CRC-32C, XXH64, Bloom-Hash) sind im Generator gegen Anhang A.1 geprüft.
`python3 tools/reference/generator.py --check` schlägt fehl, wenn sich Primitive oder Goldens ändern.

## Offene Punkte

- **A-1 (Annahme):** `chk` ist CRC-32C mit seed 0 über alle Bytes vor `chk`. Die Spec nennt
  den Seed für Kopfprüfwerte nicht. Die Annahme ist vor G-F zu bestätigen.
- **Modi `Authenticated` und `Encrypted`:** Goldens brauchen BLAKE3 (Mac16) und XChaCha20-Poly1305.
  Die Python-Standardbibliothek und `cryptography` bieten beides nicht. Diese Goldens fehlen,
  bis die Abhängigkeit festgelegt ist.
- **Abweichung von R7-Goldens:** `tests/format/golden_test.rs` im Bestand erwartet 51 B.
  Die Spec verlangt 53 B bei `Plain` (§10.2). Der Bestandstest ist deshalb kein Nachweis.
- **Redo-Frame, Gruppen, Seal, Catalog, Run, Blob:** nicht Teil von WP-2.1.
