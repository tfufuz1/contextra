# contextra-erasure — Spezifikation (docs/SPEC.md §3.1) [S]

> Diese Spezifikation ist die normative Vorgabe für `contextra-erasure`. Sie beschreibt den kanonischen Zielzustand, den verifizierten Ist-Zustand und die einzuhaltenden Verträge als autarker Baustein.
> Die fachliche Spezifikation der Löschkette ist in `docs/erasure/Loeschkette_Implementierungsspezifikation.md` definiert.

**Pfad:** `crates/contextra-erasure/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Löschkette: Löschbeweise, Ledger, Signaturen, Read-Gates, Audit-Zeugen und Scanner-Traits. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | compliance |
| Ebene (Kern-Closure) | 0 |
| Kern-Closure | — (vollkommen autark) |
| Default-Features | `signing`, `vault`, `async` (erhält volles Verhalten) |
| Fehlertyp (R3) | `ErasureError` (lokal, thiserror, non_exhaustive) |
| Brücke (hinter Feature `workspace`, §1.1) | Keine internen Abhängigkeiten erforderlich |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features`.

## 3. Ist-Zustand (verifiziert)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | ca. 12.464 Zeilen in 66 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | keine (0) |
| Interne Dev-Abhängigkeiten (Ist) | keine (0) |
| Features (Ist, Anzahl) | 3 (`signing`, `vault`, `async`) |
| Default-Features (Ist) | `["signing", "vault", "async"]` |
| `#[test]`-Vorkommen (grep, src + tests) | > 139 |
| zero-panic: Produktionsbefunde | 0 (kein unwrap/expect/panic in src außerhalb Tests) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False (`#![forbid(unsafe_code)]`) |
| Determinismus: Produktionsbefunde | 0 unkontrollierte Ports (Uhren und Rng injizierbar) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Autarkie-Score (`decouple_inspector.py`) | 99/100 (Exzellent, Baustein der Klasse A) |

## 4. Ist-vs-Ziel-Abgleich [V]

- **Interne Abhängigkeiten:** Vollständig eliminiert (0 interne Kanten).
- **Features & Defaults:** Features `signing`, `vault`, `async` sind implementiert und default-aktiviert.
- **Klasse-A Konformität:** Erfüllt Kriterien B1–B12 (0 interne Abhängigkeiten, lokale Port- und Domänentypen in `types_local.rs`, `#![forbid(unsafe_code)]`, Zero-Panic-Doktrin).

## 5. Verifizierte Befunde und Architekturprinzipien [V]

- `contextra-erasure` agiert als autonomer Baukasten.
- Lokale Typendefinitionen für `TenantId`, `CollectionId`, `DocId`, `KeyScope`, `RequestContext`, `WitnessLayer`, `Purger`, `LayerScanner`.
- Krypto-Shredding (`kv_shredding.rs`) und Revocation-Log (`revocation_log.rs`) sind vollständig gekapselt.
- Ed25519-Signierung und Beweisprüfung sind hinter dem Feature `signing` isoliert.
- Tresor-Subsystem (`vault/`) mit 16 Shards und Fail-Closed-Verhalten ist hinter dem Feature `vault` isoliert.
- Asynchrone Schnittstellen und Tokio-Treiber sind hinter dem Feature `async` isoliert.

## 6. Anforderungen und Verträge [S]

### 6.1 Invarianten
1. **INV-CLASS-MEASURED-1:** Kein Löschbericht ohne verifizierte Scanner-Zeugen.
2. **INV-NO-RESURRECTION-1:** Keine Reaktivierung gelöschter Schlüssel nach Replay/Wiederanlauf.
3. **INV-VAULT-FAILCLOSED-1:** Jeder E/A- oder Synchronisationsfehler beim Überschreiben vergiftet das Vault-Handle permanent.
4. **INV-CLAIM-BIND-1:** Neutralität aller Berichte ohne rechtsverbindliche Garantiezusagen (SPEC §6.4).

### 6.2 Modul-Organisation
- Keine `mod.rs`-Dateien (SPEC D-90).
- Wurzeldatei `<m>.rs` liegt jeweils neben dem optionalen Modul-Verzeichnis `<m>/`.

## 7. Abnahme & Verifikation [S]

```bash
cargo check -p contextra-erasure --no-default-features
cargo check -p contextra-erasure --features signing
cargo check -p contextra-erasure --features vault
cargo check -p contextra-erasure --features async
cargo test -p contextra-erasure
python3 tools/decouple_inspector.py -c contextra-erasure --all
```
