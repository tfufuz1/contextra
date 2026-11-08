# contextra-erasure-verify

Unabhaengige Verifikationsinstanz fuer signierte Loeschberichte und Feldscans gemaess SPEC §8.17, §11.7, Anhang D, F.9 und F.10.

## Uebersicht & Architektur

`contextra-erasure-verify` ist ein voellig autarker Baustein im Ring Tooling des Contextra-Baukastensystems.
Er wurde entwickelt, um Dritten und Betreibern eine unabhaengige Pruefmoeglichkeit zu geben, ohne den eigentlichen Speicherkern oder die Loeschkomponenten (`contextra-erasure`) einbinden zu muessen.

### Baukasten-Garantien (M1–M11)
- **M1 Strikte Abwaertsrichtung:** Ring Tooling. Keine Abhaengigkeit zum Loeschkern `contextra-erasure`, haengt ausschliesslich an `contextra-types`.
- **M2 Autarke Baubarkeit:** Baut und testet vollstaendig isoliert via `cargo test -p contextra-erasure-verify --locked`.
- **M3 Schnittstellenreinheit:** Exponiert nur kanonische Verifikations-Typen und Resultate.
- **M4 Injizierte Umwelt:** Passphrasen werden sicher ueber File Descriptoren uebergeben (`/proc/self/fd/<fd>`) und nach Benutzung via `Zeroize` im RAM geloescht.
- **M9 Speicher-Sicherheit:** `#![forbid(unsafe_code)]` im gesamten Produktionscode.
- **Wortlaut-Disziplin (SPEC 2.6):** Verzicht auf Marketing-Zusagen („100 % sicher geloescht“); messbare Fakten und Restrisiko-Pruefung.

## Funktionen

1. **Signierte Loeschberichte (`report`)**:
   - Strikte Kanonizitaetspruefung des JSON-Formats gemaess SPEC F.10 (NFC, kein Float, lexikographisch sortiert, keine redundanten Whitespaces).
   - Ed25519 `verify_strict` Signaturpruefung mit Public-Key-Fingerprint-Abgleich.
   - Zeitstempel-Monotonie (`requested <= gated <= key_destroyed <= completed`).
   - Zeugen-Pruefung (`field_measurement`): Jeder Zeuge MUSS eine bestandene Positivkontrolle (`control_ok == true`) und 0 Residuen (`matches == 0`) aufweisen.
   - Vollstaendigkeitspruefung der deklarierten Restrisiken nach SPEC Anhang D.

2. **Unabhaengiger Feldscan (`scan`)**:
   - **MK1 Strukturscan:** Prueft Manifest- und Indexstrukturen auf verbliebene Identifikatoren.
   - **MK2 Roh-Scan:** Sucht Byte-Muster / Klartext-Residuen in Speichersegmenten.
   - **MK3 Vault-Schluesselprobe:** Prueft `FileVault`-Shards (`.vlt` / `vault.dat`) nach Format F.3 auf nicht entwertete Schluesselslots.
   - **Positivkontroll-Zwang:** Ein Negativbefund (keine Daten gefunden) ist formal UNGUELTIG, wenn die Positivkontroll-Nadel nicht nachweislich detektiert wurde.

## CLI-Verwendung

```bash
# Loeschbericht pruefen
contextra-erasure-verify report --report /path/to/report.json --key /path/to/public.key

# Feldscan durchfuehren
contextra-erasure-verify scan --data-dir /path/to/data --target 0102030405060708090a0b0c0d0e0f10 --control-needle-file /path/to/canary.dat
```

### Exit-Codes

| Exit-Code | Ausgabe | Bedeutung |
|---|---|---|
| `0` | `VALID` | Pruefung erfolgreich bestanden |
| `2` | `INVALID(<grund>)` | Validierungsfehler (z. B. `bad_signature`, `missing_positive_control`, `mk2_raw_residue`) |

## Standalone-Beispiel

Ein vollstaendiges, eigenstaendiges Beispiel befindet sich unter `examples/standalone_verify.rs`:

```bash
cargo run --example standalone_verify -p contextra-erasure-verify
```
