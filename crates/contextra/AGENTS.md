# Contextra — AI-Assistenten-Kontext (`contextra`)

<!-- doc-stamp: 2026-10-31 | Ring: 4 | Normativ: docs/SPEC.md §4, PLAN.md -->

> **Für AI-Assistenten & Implementierer:** Diese Datei beschreibt den Crate `contextra`.
> `contextra` dient als einzige Composition Root und öffentliche Hauptfassade des Gesamtsystems.

---

## 1. Crate-Topologie & Rolle

- **Ring 4 Composition Root:**
  - `ContextraBuilder`: Konfiguration und deterministische Assemblierung aller Teilsysteme (`store`, `vector`, `text`, `crypto`, `engine`, `db`).
  - Re-Export der primären APIs für Rust-Anwendungsentwickler.
- **Invariante A1 / Ring 4:** Kein Crate im Workspace darf von `contextra` abhängen. `contextra` bindet untergeordnete Schichten ein, niemals umgekehrt.
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`.
- **Zero Panic (P7):** Keine `unwrap()`- oder `panic!`-Aufrufe.

---

## 2. Bau- und Test-Kommandos

```bash
cargo check -p contextra
cargo test -p contextra --locked
```

---

## 3. Relevante Arbeitspakete (PLAN §12)

- **S-02 / S-03:** Sichere Defaults im `ContextraBuilder`: Verschlüsselung standardmäßig aktiv (`EncryptionMode::Required`), Mandantenprüfung erzwingen (`TenantMode::Required`).
- **S-05:** Passphrase über `SecretString` statt Klartext-String im Builder verwalten.
- **Entkopplung:** Optionale Features (`candle`, `onnx`, `audit-export`) sauber hinter Feature-Flags isolieren.
