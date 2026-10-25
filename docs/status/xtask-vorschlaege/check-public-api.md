# Gate-Vorschlag: `check-public-api`

- **Anforderung**: REQ-ARCH-014 (§27.6, §30)
- **Status**: `harness:planned`
- **Mechanismus**: `G` (Gate via `cargo public-api`)

## 1. Ziel
Prüfung aller öffentlichen Symbole und Funktionssignaturen des `contextra`-Workspace auf verbotene Schnittstellenmuster (INV-SCAN-1). Das Gate stellt sicher, dass:
1. Keine Klartext-Krypto-Schlüssel über öffentliche Typen oder Rückgabewerte exponiert werden.
2. `*_for_test`-Hilfsfunktionen und Bypass-Konstruktoren in Release-Features rigoros ausgeschlossen sind.

## 2. Eingaben
- Output von `cargo public-api` für alle Kern-Crates (`contextra-*`).
- `capabilities.toml` zur Identifikation aller Ring-0 und Kern-Crates.
- Regelsatz verbotener Namens- und Typpattern (z. B. `SecretKey`, `PrivateKeyBytes`, `for_test`).

## 3. Ausgabe
- Konsolenausgabe mit Angabe der geprüften Crates und Signaturen.
- Exit-Code `0` bei Validität, `1` bei Erkennung verbotener öffentlicher Signaturen.
- Maschinell lesbarer JSON-Bericht unter `target/gates/public-api-report.json`.

## 4. Selbsttest mit synthetischem Repository (REQ-QA-008)
- **Positive Control**: Ein synthetisches Crate mit einer verbotenen Methode `pub fn export_raw_secret_key_for_test(&self) -> Vec<u8>` schlägt fehl (Exit-Code `1`).
- **Negative Control**: Ein sauberes Crate mit Typestate-Safe KeyRef-Signaturen passiert das Gate (Exit-Code `0`).
