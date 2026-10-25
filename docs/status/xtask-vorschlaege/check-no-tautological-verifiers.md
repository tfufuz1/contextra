# Gate-Vorschlag: `check-no-tautological-verifiers`

- **Anforderung**: REQ-DEL-010 (§20.8, §29)
- **Status**: `harness:planned`
- **Mechanismus**: `G` (Gate via AST & Control-Flow Analysis)

## 1. Ziel
Ausschluss tautologischer Verifizierer: Keine Prüf- oder Verifikationsfunktion für Löschnachweise oder Schlüssel darf unbedingte Wahrheit (`return true` / `Ok(())` ohne Signatur-/Hashprüfung) liefern.

## 2. Eingaben
- Control-Flow-Graph (CFG) aller Methoden, die das Trait `Verifier` oder `verify` implementieren.

## 3. Ausgabe
- Bericht aller Verifikationspfade.
- Exit-Code `0` bei echten kryptografischen/mathematischen Prüfschritten auf jedem Pfad.
- Exit-Code `1` bei unbedingter Rückgabe von `true`/`Ok(())`.

## 4. Selbsttest mit synthetischem Repository (REQ-QA-008)
- **Positive Control**: `fn verify(&self) -> bool { true }` schlägt fehl (Exit-Code `1`).
- **Negative Control**: `fn verify(&self) -> bool { ed25519::verify(...) }` passiert (Exit-Code `0`).
