# Gate-Vorschlag: `check-payload-key-wrapping`

- **Anforderung**: REQ-DEL-026 (§20.2b)
- **Status**: `harness:planned`
- **Mechanismus**: `G` (Gate via Type & Struct Layout Scan)

## 1. Ziel
Erzwingung der Epochen- und Dokumentgenauen Schlüsselvernichtung: Persistierte Payloads dürfen keine gewrappten Schlüssel tragen, sondern ausschließlich opake `KeyRef`-Identifier. Das direkte Speichern gewrappter Schlüssel im Dokumentenblock ist untersagt.

## 2. Eingaben
- Struct- und Layout-Analyse aller Typen, die das Trait `Serialize` / `FlatBuffer` implementieren.

## 3. Ausgabe
- Typenübersicht aller Persistenzstrukturen.
- Exit-Code `0` bei strenger Trennung von Payload und `ErasureKeyStore`.
- Exit-Code `1` bei Einbettung von `WrappedKey` in Daten-Payloads.

## 4. Selbsttest mit synthetischem Repository (REQ-QA-008)
- **Positive Control**: Ein Struct `struct DocumentPayload { data: Vec<u8>, wrapped_key: WrappedKey }` schlägt fehl (Exit-Code `1`).
- **Negative Control**: `struct DocumentPayload { data: Vec<u8>, key_ref: KeyRef }` passiert (Exit-Code `0`).
