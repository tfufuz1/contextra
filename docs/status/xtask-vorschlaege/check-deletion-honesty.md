# Gate-Vorschlag: `check-deletion-honesty`

- **Anforderung**: REQ-DEL-008 (§20.8)
- **Status**: `harness:planned`
- **Mechanismus**: `G` (Gate via Mutation & Code Audit)

## 1. Ziel
Überprüfung der Ehrlichkeit der Löschmesswerte (`deletion-honesty`): Ein Messwert für die Löschung (z. B. überschriebene Bytes, verbliebene Chiffretext-SSTables) darf nur vom dedizierten Scanner der jeweiligen Schicht erzeugt werden, nicht von synthetischen Mock-Funktionen.

## 2. Eingaben
- Quellcode der Löschscanner in `contextra-store`, `contextra-crypto`, `contextra-privacy`.
- Statische Quellcode-Analyse auf harte Rückgabewerte (z. B. `return 100`) in Scanner-Signaturen.

## 3. Ausgabe
- Prüfbericht aller Scanner-Implementierungen.
- Exit-Code `0` bei echten Scan-Schleifen.
- Exit-Code `1` bei statischen oder unehrlichen Scanner-Rückgaben.

## 4. Selbsttest mit synthetischem Repository (REQ-QA-008)
- **Positive Control**: Eine synthetische Funktion `fn scan_bytes() -> usize { 1024 }` ohne I/O schlägt fehl (Exit-Code `1`).
- **Negative Control**: Ein echter Verzeichnis-Scanner passiert das Gate (Exit-Code `0`).
