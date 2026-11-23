# Format-Goldens (WP-2.1, SPEC §19.4, §22 G-2)

## Herkunft

Die Dateien in `golden/` sind Byte-Referenzen aus `tools/reference/generator.py`. Der Generator ist ein unabhängiger Python-Codierer (nur Standardbibliothek, kein Import von Kerncode) und wird nach SPEC §10 gebaut. Die Goldens werden **nicht** aus dem Kern erzeugt.

| Datei | Gegenstand | Länge | Spec | Status |
|---|---|---|---|---|
| `golden/format_plain.golden` | FORMAT, Plain, Vektor 1 (Feldbits 0) | 53 B | §10.2 | Referenzwert, nicht gegen den Kern geprüft |
| `golden/format_plain_features.golden` | FORMAT, Plain, Vektor 2 (Bits gesetzt, prüft Byte-Reihenfolge der u64-Felder) | 53 B | §10.2 | wie oben |
| `golden/segment_header_plain.golden` | Segmentkopf, Plain | 36 B | §10.4 | wie oben |
| `golden/frame_body_plain.golden` | Frame ohne `frame_chk`, Plain, 2 Ops | 121 B (vor `frame_chk`) | §10.4 | **unvollständig** (L-2) |

Jede Datei enthält je Feld: Offset, Länge, Name, Hex und Bedeutung. Die letzte Zeile `GESAMT <hex>` ist das vollständige Golden.

## Aufruf

```
python3 tools/reference/generator.py --selftest   # Anhang A.1 und Längen; Exit 1 bei Abweichung
python3 tools/reference/generator.py              # schreibt die Golden-Dateien
python3 tools/reference/generator.py --check      # Erzeugung gegen Dateien; Exit 1 bei Abweichung
python3 tools/reference/verify_goldens.py [json]  # Gegenprobe mit bitweiser CRC-32C (unabhängig vom Generator)
```

`--check` ist der maschinelle Vergleich. Zur Nachvollziehbarkeit: `diff -r` zwischen einer Kopie des Verzeichnisses vor und nach `generator.py` muss leer sein. Ein Golden wird erst dann übernommen, wenn der Kern dieselben Bytes erzeugt (§19.4); dieser Schritt ist WP-2.2 und steht aus.

## Annahmen

- **A-1** `chk` jedes Kopfes ist `crc32c(0, Bytes vor chk)` im Sinne von SPEC §4 (`crc32c(seed, data) = !update(!seed, data)`). Die Spec nennt den Seed für Kopfprüfwerte nicht. Vor G-F zu bestätigen.
- **A-2** Ein Golden gilt nur zusammen mit den Primitiven aus Anhang A.1. Der Generator prüft sie im Selbsttest (CRC-32C, XXH64, Bloom-Hashes; 8 Vektoren).
- **A-3** `chain_seed` des Segmentkopfs bei `Plain` ist ein `u32` in little-endian (§10.1 gilt für alle Zahlen). Die Spec nennt nur die Länge (4 B).
- **A-4** `ops_payload` ist in den Frame-Vektoren reine Byte-Eingabe. Die Semantik der Op-Nutzlast steht nicht in §10 und wird nicht geprüft.
- **Testvektoren:** Die Werte `feature_req = 0x8` und `feature_opt = 1<<32` in `format_plain_features` dienen nur der Prüfung der Byte-Reihenfolge. Die Semantik dieser Bits ist nicht geprüft; `0x8` (COLD_TIER) gilt in Plain nicht zwingend.

## Lücken (nicht generiert, nicht geraten)

- **L-1 Run-Kopf:** §10.5 nennt `Run := Header` ohne Feldliste. Kein Golden. Ein Golden braucht eine Spec-Festlegung der Felder.
- **L-2 `frame_chk`:** §10.4 legt nicht fest, über welche Bytes der CRC läuft (ab `frame_len`, ab `stamp` oder nur ab `ops_head`) und was der „Kettenwert des Vorgängers“ für den ersten Frame ist (`chain_seed`? `chk` des Segmentkopfs?). Das Frame-Golden endet deshalb vor `frame_chk`.
- **L-3 `group_chk` und Gruppenkopf:** Die Feldliste steht in §10.4, die Abdeckung von `group_chk` und die Bedeutung von `group_flags` nicht. Nicht Teil dieser Stufe.
- **L-4 Op-Art und Op-Flags:** Die Werte von `op_kind` und die Bits von `frame flags` außer Bit 7 sind nicht aufgezählt. Der Generator erzeugt nur die Werte, die die Vektoren brauchen.
- **L-5 Segmentkopf `flags`:** Bedeutung nicht spezifiziert. Vektor: 0.
- **L-6 Nicht in dieser Stufe:** Modi `Authenticated` und `Encrypted` (Mac16 braucht BLAKE3, AEAD braucht XChaCha20-Poly1305, siehe UE-10 und WP-2.4), Seal-Rahmen (WP-2.6), KEYS, Catalog, Blob, Run-Körper.

## Unverifiziert

- Alle Werte sind mit zwei unabhängigen CRC-32C-Implementierungen (tabellenbasiert im Generator, bitweise in `verify_goldens.py`) und mit einer Längenrechnung aus den Feldtabellen geprüft. Ein Eintrag aus dem Entwurf PR #6069 stimmt byte-genau überein.
- Nicht verifiziert: Übereinstimmung mit dem Rust-Kern (WP-2.2), Annahmen A-1 und A-3 gegen eine Spec-Festlegung, die Lücken L-1 bis L-5.
- Der Bestandstest `golden_test.rs` prüft 51 B und ist **kein** Nachweis (§19.3). Er wird in G-2 ersetzt. Diese Stufe ändert ihn nicht.

## Abweichungen von der Spec-Prosa

- §10.1 sagt für Header und Footer: in allen Modi „zusätzlich ein CRC-32C“; in `Authenticated`/`Encrypted` „außerdem `Mac16`“. §10.2 legt für FORMAT genau einen `chk` fest, der in `Authenticated`/`Encrypted` den CRC **ersetzt** (65 B). Die Golden-Werte für `Plain` (53 B) sind in beiden Lesarten gleich. Vorschlag siehe PR-Beschreibung (SA-Entwurf).
- Anhang A.2 nennt für FORMAT einen Referenzlauf mit 47 B vor `chk`. Der Generator liefert 49 B, die Feldtabelle ergibt 49 B. Die 47 B lassen sich nicht reproduzieren; der Eintrag in A.2 ist zu bereinigen.
- Anhang A.2 nennt für das Siegel-Flag `0x80` als Abweichung zu „Bit 7“. Bei einem `u8`-Feld ist `0x80` gleich Bit 7. Der Hinweis trifft nur zu, wenn das Feld breiter ist.
