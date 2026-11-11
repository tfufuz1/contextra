# `contextra-prompt-guard`

> Autarker Sicherheitsbaustein (Ebene 0, Tier `experimental`) · Unsafe: `#![forbid(unsafe_code)]` · Rolle: Signatur- und phrasenbasierte Prompt-Injection-Erkennung und Quarantäne-Policy.

## 1. Zweck & Verantwortung

Erkennung bekannter Angriffsmuster (direkte Phrasen, whitespace-verschleierte Varianten, Base64-kodierte Payloads bis Tiefe 2, Unicode-Homoglyphen / Confusables) und Durchsetzung konfigurierbarer Quarantäne-Policies (`Strict`, `FlagOnly`, `Escalate`).

## 2. Modul-Karte

| Modul | Verantwortung |
|---|---|
| `contract.rs` | Minimale Umwelt-Traits (`Clock`) ohne externe Runtime-Kopplung |
| `guard.rs` | `PromptInjectionGuard`: Vornormalisierte Pattern-Erkennung, Skeletonisierung, Base64-Extraktion |
| `policy.rs` | `QuarantinePolicy`, `PromptInjectionConfig`, Standard-Redaktionsplatzhalter |
| `audit.rs` | `SecurityAuditLogger`, `SecurityAuditRecord`: Isoliertes Audit-Logging |
| `clock.rs` | `SystemClock`: Lokale Zeitabstraktion basierend auf Systemzeit |
| `compat/` | Kompatibilitätsbrücken (`error.rs`) |
| `error.rs` | `GuardError`: Typisierte Fehlerbehandlung via `thiserror` (`Io`, `Serialization`, `InvalidConfiguration`, `DecodeError`) |

## 3. Features

| Feature | Beschreibung |
|---|---|
| `std` | Standardbibliothek-Integration (Default) |
| `file-audit` | Datei-I/O für `SecurityAuditLogger` und Laden externer Konfigurationen (optional) |

## 4. Tests & Fuzzing

```bash
cargo test -p contextra-prompt-guard
cargo check -p contextra-prompt-guard --no-default-features
cargo check -p contextra-prompt-guard --features file-audit
cargo run -p contextra-prompt-guard --example standalone --no-default-features
```
