---
name: contextra-security-audit
description: >-
  Use this skill to audit and enforce Contextra security defaults, Argon2id KDF derivation, Zeroize key management,
  mandatory tenant isolation (VETO-F10), encryption by default (no opt-out), and fail-closed security properties.
---

# Contextra Security Audit – Sicherheits- & Datenschutz-Auditierung

Dieser Skill leitet Agenten und Entwickler an, die normativen Sicherheits- und Datenschutzvorgaben (SPEC §10) im Code zu prüfen und durchzusetzen.

---

## 1. Zentrale Sicherheits-Defaults (SPEC §10)

In Contextra gelten ausnahmslos sichere Standardeinstellungen („Secure by Default“):

| Befund | Unsicherer Ist-Zustand | Normativer Soll-Zustand (SPEC §10) | Fundstelle |
|---|---|---|---|
| **S-01** | Passphrase via HKDF-SHA256 | **Argon2id KDF**: $m \ge 64\,\text{MiB}, t \ge 3, p \ge 1$ (`derive_master`). HKDF für Passphrasen ist verboten. | `contextra-crypto/src/kdf.rs` |
| **D-05** | `encryption_mode: None` | **`EncryptionMode::Required`**: Pflichtverschlüsselung **ohne Opt-Out**. Kein Klartext-Blockpfad. | `contextra/src/builder.rs` |
| **D-07** | `TenantPolicy::Optional` | **`TenantPolicy::Required`**: Strikte Mandantentrennung verpflichtend ohne Opt-Out. | `contextra-engine/src/lib.rs` |
| **D-27** | Auto-Extraktion `Enabled` | **Vollständig entfallen in v1**: Kein unbemerkter PII-Abfluss an externe Dienste. | `contextra-types` |
| **S-05** | Secret als Plain `String` | **`SecretBytes` / `Zeroizing<T>`**: Kein Klonen, kein Plaintext-Logging (`Debug`), automatische Speicher-Nullung. | `contextra-types/src/secret.rs` |
| **D-29** | Unsafe-Streuung | **Nur 2 Inseln (`sys`, `simd`)**: Alle anderen Crates erzwingen `#![forbid(unsafe_code)]`. | Gesamter Workspace |

---

## 2. Audit-Checkliste für den Entwickler

### 1. KDF-Ableitung (S-01, D-06)
Suche nach veralteten HKDF-Aufrufen auf Passphrasen:
```bash
grep -rn "derive_master" crates/
```
- **Prüfung**: Die Konstante `MIN_M_COST_KIB` in `contextra-crypto/src/kdf.rs` muss mindestens 65536 (64 MiB) betragen.

### 2. Mandanten-Isolation (VETO-F10 & D-07)
Suche nach mandantenlosen Aufrufen oder `Optional`:
```bash
grep -rn "TenantPolicy::Optional" crates/
grep -rn "TenantPolicy" crates/
```
- **Prüfung**: Jeder Store-Zugriff muss über `TenantScopedStorage` erfolgen (T-ARCH-2).

### 3. Geheimnis-Bereinigung (Zeroize, S-05)
- Alle sensiblen Schlüsselstrukturen (z. B. `KeySet`, `SecretBytes`, `SecretKey32`) müssen `Zeroize` oder `Zeroizing<T>` nutzen.
