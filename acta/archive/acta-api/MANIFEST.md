# Extraktion fuer `acta-api`

**Aufgabe:** HTTP/JSON-Schnittstelle, OIDC, mTLS, Streaming  
**Level:** L8 · **Budget:** 6000 LOC · **Spezifikation:** docs/SPEC.md Kap. 13.4, 18.5  
**Interne Abhaengigkeiten (Soll):** acta-node, acta-base  
**Umfang des Archivs:** 2 Dateien, 1337 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `wire` | Nur Referenz (HTTP/JSON per axum ist Neubau, 18.5) |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| serde | 6 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| unsafe | 2 | unsafe (G-UNSAFE: nur acta-sys) |
| panic | 12 | unwrap/expect/panic (G-LINT) |

**Externe Abhaengigkeiten der Altquelle:** alle innerhalb der Whitelist.

## Dateien

| Datei (relativ zu `archive/acta-api/`) | LOC | Befunde |
|---|---:|---|
| `types/src/error_dto.rs` | 599 | panic=3, serde=4 |
| `wire/src/jsonrpc.rs` | 738 | panic=13, serde=6, unsafe=2 |
