# Gate-Vorschlag: `check-durability-integrity-chain`

- **Anforderung**: REQ-DUR-009 (§23.2)
- **Status**: `harness:planned`
- **Mechanismus**: `G` (Gate via AST- / Type-Scan)

## 1. Ziel
Sicherstellung, dass es im Produktionscode keinen persistenten Durabilitätsmodus ohne Integritätskette (HMAC-Chaining / CRC32) gibt. Der Modus `WalNoHmac` ist als unzulässig eingestuft.

## 2. Eingaben
- Quellcode der Crates `contextra-store`, `contextra-db`, `contextra-engine`.
- AST-Abfrage aller WAL- und Persistence-Konfigurationstypen.

## 3. Ausgabe
- Liste aller untersuchten Persistence-Enums und Konfigurationen.
- Exit-Code `0` bei 100% kryptografischer Absicherung aller Durabilitätsmodi.
- Exit-Code `1` falls ein Modus ohne Integritätskette deklariert ist.

## 4. Selbsttest mit synthetischem Repository (REQ-QA-008)
- **Positive Control**: Ein synthetischer Enum-Variante `DurabilityMode::NoProtection` erzeugt Exit-Code `1`.
- **Negative Control**: Der Produktionsstand mit erzwungenem HMAC-Chaining passiert das Gate (Exit-Code `0`).
