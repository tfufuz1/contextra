# `contextra-prompt-guard`

> Autarker Sicherheitsbaustein (E0 / Klasse `[S]` laut SPEC-v2) · Unsafe: `#![forbid(unsafe_code)]` · Rolle: Signatur- und phrasenbasierte Prompt-Injection-Erkennung und Quarantäne-Policy.

## 1. Zweck & Verantwortung

Erkennung bekannter Angriffsmuster (direkte Phrasen, whitespace-verschleierte Varianten, Base64-kodierte Payloads bis Tiefe 2, Unicode-Homoglyphen / Confusables) und Durchsetzung konfigurierbarer Quarantäne-Policies (`Strict`, `FlagOnly`, `Escalate`).

## 2. Modul-Karte

| Modul | Verantwortung |
|---|---|
| `guard.rs` | `PromptInjectionGuard`: Vornormalisierte Pattern-Erkennung, Skeletonisierung, Base64-Extraktion |
| `policy.rs` | `QuarantinePolicy`, `PromptInjectionConfig`, Standard-Redaktionsplatzhalter |
| `audit.rs` | `SecurityAuditLogger`, `SecurityAuditRecord`: Isoliertes Audit-Logging |
| `clock.rs` | Trait `Clock` und `SystemClock`: Lokale Zeitabstraktion ohne Upstream-Kopplung |
| `error.rs` | `GuardError`: Typisierte Fehlerbehandlung via `thiserror` (`Io`, `Serialization`, `InvalidConfiguration`, `DecodeError`) |

## 3. Features

| Feature | Beschreibung |
|---|---|
| `std` | Standardbibliothek-Integration (Default) |
| `file-audit` | Datei-I/O für `SecurityAuditLogger` und Laden externer Konfigurationen (Default) |

## 4. Tests

```bash
cargo test -p contextra-prompt-guard
cargo check -p contextra-prompt-guard --no-default-features
cargo check -p contextra-prompt-guard --features file-audit
```
