# Gate-Vorschlag: `check-no-shred-fallback-keys`

- **Anforderung**: REQ-DEL-009 (§20.8)
- **Status**: `harness:planned`
- **Mechanismus**: `G` (Gate via Literal Scan)

## 1. Ziel
Garantie der Schlüsselunabhängigkeit: Es darf im Produktionscode keinen Fallback-, Standard- oder Testschlüssel zum Krypto-Shredding geben.

## 2. Eingaben
- AST- und String-Literal-Scan aller Dateien unter `crates/*/src/`.

## 3. Ausgabe
- Liste aller untersuchten Krypto-Schlüssel-Initialisierungen.
- Exit-Code `0` wenn alle Schlüssel dynamisch über den `KmsProvider` bezogen werden.
- Exit-Code `1` bei Fund von statischen Byte-Arrays oder Fallback-Schlüsseln.

## 4. Selbsttest mit synthetischem Repository (REQ-QA-008)
- **Positive Control**: Ein hartkodiertes Byte-Array `const FALLBACK_KEY: [u8; 32] = [0x42; 32];` schlägt fehl (Exit-Code `1`).
- **Negative Control**: Ein reiner `KmsProvider::derive_key`-Aufruf passiert (Exit-Code `0`).
