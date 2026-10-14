# AGENTS.md — contextra-license
> Ring 4 · experimental · Quelle: capabilities.toml · Spec: III.20 / K.16

## 1. Zweck
Lizenz- und Aktivierungsprüfung für Contextra Feature-Ringe (`Fast`, `Sovereign`, `Compliance`).
Validiert kryptographisch signierte Lizenz-Payloads (Ed25519) und stellt Implementierungen des `LicenseGate`-Traits bereit.
Gewährleistet den lizenzfreien Betrieb im Open-Source-Fast-Ring, während höherwertige Enterprise-Ringe geschützt werden.

## 2. Modul-Karte

| Datei | Verantwortung |
|---|---|
| `lib.rs` | Öffentliche Re-Exports (`SignedLicenseGate`, `OpenFastGate`) und Modul-Dokumentation |
| `signed_gate.rs` | Ed25519-basierte Signaturprüfung von `LicensePayload` mit Bincode-Deserialisierung |

## 3. Invarianten

- **INV-LICENSE-2**: Der Fast-Ring ist ohne Lizenzschlüssel bedingungslos gültig (`OpenFastGate` / Fast Ring Bypass in `SignedLicenseGate::check_ring`).
  *Prüfung*: `cargo test -p contextra-license --lib`
- **INV-LICENSE-PAYLOAD-STRICT**: Deserialisierung verwendet `bincode::options().with_fixint_encoding().reject_trailing_bytes()`, um manipulierte Payloads abzulehnen.
  *Prüfung*: `cargo test -p contextra-license --lib`
- **INV-LICENSE-CONSTANT-TIME**: Installations-ID Hash-Vergleich erfolgt ohne Timing-Leaks.
  *Prüfung*: `cargo test -p contextra-license --lib`

## 4. Verboten / Anti-Patterns

- **Keine unbeabsichtigte Fehlerlecks**: Bei ungültigen Lizenzen keine Detailgründe an unvollständige Aufrufer weitergeben, die Aufschluss über Aktivierungsdetails geben.
- **Keine variablen Bincode-Header**: Bincode-Parsing darf niemals nachgelagerte Byte-Reste akzeptieren (`reject_trailing_bytes`).
- **Keine Unsafe-Operationen**: `#![forbid(unsafe_code)]` ist im gesamten Crate strikt erzwungen.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- `SignedLicenseGate` ist immutable und FFI-/Thread-sicher (`Send + Sync`).
- Keine internen Mutexes oder Sperren vorhanden; Prüfungen sind rein zustandslos bzw. schreibgeschützt.
- Zeitstempelvergleiche nutzen die injizierte `Clock`-Abstraktion oder Systemzeit ohne zeitkritische Blockaden.

## 6. Verifikation

```bash
cargo test -p contextra-license
cargo check -p contextra-license
cargo xtask check-agents-integrity
cargo xtask doctrine-scan --crate contextra-license
```

## 7. Bekannte Lücken / SOLL

- Offline-Aktivierungstoken-Verlängerung ohne Systemuhr-Abweichung noch nicht als automatisches Dynamic-Renewal implementiert.
