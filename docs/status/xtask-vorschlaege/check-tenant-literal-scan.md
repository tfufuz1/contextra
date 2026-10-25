# Gate-Vorschlag: `check-tenant-literal-scan`

- **Anforderung**: REQ-TEN-003 (§4.1, §21.1)
- **Status**: `harness:planned`
- **Mechanismus**: `G` (Gate via String Literal Scan)

## 1. Ziel
Verhinderung mandantenübergreifender Leaks: Ein vom Aufrufer gelieferter `TenantId`-Wert darf im Produktionscode nicht durch fest verdrahtete Tenant-Konstanten (z. B. `"default_tenant"`, `"global"`) ersetzt oder ignoriert werden.

## 2. Eingaben
- Quellcode-Scan aller Request-Handler und API-Endpunkte in `crates/`.
- AST-Suche nach festen Strings als Konstruktoren von `TenantId`.

## 3. Ausgabe
- Liste aller `TenantId`-Instanziierungen.
- Exit-Code `0` bei ausschließlicher Ableitung aus der Transaktion oder authentifizierten Peer-Credentials.
- Exit-Code `1` bei hartkodierter Mandanten-Zuweisung im Produktionscode.

## 4. Selbsttest mit synthetischem Repository (REQ-QA-008)
- **Positive Control**: Code `let tenant = TenantId::from("default");` in einem MCP-Handler erzeugt Exit-Code `1`.
- **Negative Control**: `let tenant = tx.tenant_id();` passiert das Gate (Exit-Code `0`).
