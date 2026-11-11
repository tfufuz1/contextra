# contextra-crypto

Kryptographische Primitive, AEAD-Blockverschlüsselung, Schlüsselableitung, WAL-Integritätsketten und Zeroize-Schlüsselcontainer für die Contextra-Architektur.

## Rolle und Zweck
`contextra-crypto` ist ein autarker Ring-0-Baustein (Ebene 1, Foundation) zur Bereitstellung I/O-freier kryptographischer Primitive:
- **AEAD-Blockverschlüsselung:** Authentifizierte Verschlüsselung via AES-256-GCM-SIV (RFC 8452) mit Nonce-Misuse-Resistance und AAD-Integrität.
- **Schlüsselableitung (KDF):** Schlüsselhierarchie via HKDF-SHA256 (RFC 5869) und Passwort-/Passphrasen-Verarbeitung via Argon2id (RFC 9106).
- **WAL-Integrität:** Kryptographische Integritätsketten mittels HMAC-SHA256 für das Write-Ahead-Log.
- **Sichere Speicherhygiene:** Sämtliche sensiblen Schlüsselstrukturen implementieren `ZeroizeOnDrop` und maskieren sich in `Debug` und `Display`.

## Abgrenzung (Nicht-Ziele)
- **Keine Rechts-, Zertifizierungs- oder Compliance-Garantien:** Das Crate implementiert technische kryptographische Algorithmen; es liefert keine Aussagen über regulatorische Konformität.
- **Keine Löschketten- oder Audit-Nachweis-Logik:** Löschketten, Sagas, Deletion-Proofs und Auditberichte gehören ausschließlich nach `contextra-erasure`.
- **I/O- und Netzwerk-Freiheit:** Reine mathematische und kryptographische Routinen; Persistenz und Dateizugriffe erfolgen ausschließlich über Konsumenten-Ports.

## Reifegrad und Tier
- **Gruppe:** `foundation`
- **Ebene:** 1 (Ring 0)
- **Klasse:** A (autarker Baustein)
- **Tier:** `beta`

## Tatsächliche interne Abhängigkeiten (Minimaler Kern)
Ermittelt via `cargo tree -p contextra-crypto -e normal --no-default-features`:
- `contextra-ids`: Typisierte Bezeichner (`TenantId`, `DocId`, `TxId`, `TenantScoped`)
- `contextra-env`: Minimaler Umwelt- und Plattform-Kontext

*Optionale Integrations-Abhängigkeiten (hinter Feature `workspace`):*
- `contextra-types`: Typen für Workspace-Integration
- `contextra-ports`: Port-Definitionen für Workspace-Integration

## Feature-Tabelle

| Feature | Standard | Beschreibung |
|---|---|---|
| `kv-encryption` | Ja | KV-Segment-Verschlüsselung und Tenant-Isolierung für KV-Cache |
| `workspace` | Ja | Brücke für `ContextraError` und Workspace-Typen in `compat/` |
| `types-compat` | Nein | Alias für `workspace` |
| `ports-compat` | Nein | Alias für `workspace` |
| `cloud-egress-guard` | Nein | Zusätzliche Egress-Wächter-Strukturen |
| `test-utils` | Nein | Hilfsroutinen für Testläufe |

## Schnellstart / Minimalbeispiel

```rust
use contextra_crypto::aead::{open_record, seal_record_with_nonce};
use contextra_crypto::dek::Dek;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dek = Dek::new([0x42u8; 32]);
    let doc_id = [0x11u8; 16];
    let key_meta = b"user_record_key";
    let nonce = [0x99u8; 12];
    let plaintext = b"Sensible Nutzlast";

    let envelope = seal_record_with_nonce(&dek, doc_id, key_meta, plaintext, nonce)?;
    let opened = open_record(&dek, key_meta, &envelope)?;
    assert_eq!(opened.as_slice(), plaintext);

    Ok(())
}
```

Ausführung ohne optionale Features:
```bash
cargo run -p contextra-crypto --example standalone --no-default-features
```
