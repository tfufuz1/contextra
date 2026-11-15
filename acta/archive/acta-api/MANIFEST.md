# Extraktion fuer `acta-api`

**Aufgabe:** HTTP/JSON-Schnittstelle, OIDC, mTLS, Streaming  
**Level:** L8 · **Budget:** 6000 LOC · **Spezifikation:** docs/SPEC.md Kap. 13.4, 18.5  
**Interne Abhaengigkeiten (Soll):** acta-node, acta-base  
**Umfang des Archivs:** 1 Dateien, 739 Zeilen (inkl. Tests)  

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
| `wire/src/jsonrpc.rs` | 739 | serde=6, unsafe=2, panic=12 |
