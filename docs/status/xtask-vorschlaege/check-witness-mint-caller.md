# Gate-Vorschlag: `check-witness-mint-caller`

- **Anforderung**: REQ-DEL-004 (§20.5)
- **Status**: `harness:planned`
- **Mechanismus**: `G` (Gate via Call-Graph Audit)

## 1. Ziel
Durchsetzung der Regel INV-WITNESS-MINT-1: Ein Schicht-Löschnachweis (`DeletionProof` / `UnverifiedProof`) darf nur direkt aus der jeweiligen Speicherschicht (`contextra-store`, `contextra-crypto`) ausgestellt werden.

## 2. Eingaben
- Call-Graph-Analyse aller Konstruktor-Aufrufe von `DeletionProof::try_new` und TLV-Minting-Methoden in `crates/`.

## 3. Ausgabe
- Bericht aller Aufrufstellen der Witness-Minting-Funktionen.
- Exit-Code `0` bei ausschließlicher Erzeugung innerhalb zugelassener Speicherschicht-Module.
- Exit-Code `1` bei illegaler Erzeugung außerhalb der Schicht.

## 4. Selbsttest mit synthetischem Repository (REQ-QA-008)
- **Positive Control**: Ein Minting-Aufruf in `contextra-router` schlägt fehl (Exit-Code `1`).
- **Negative Control**: Erzeugung in `contextra-crypto` passiert (Exit-Code `0`).
