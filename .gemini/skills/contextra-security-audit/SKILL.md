---
name: contextra-security-audit
description: >-
  Use this skill to audit and enforce Contextra security defaults, Argon2id KDF derivation, Zeroize key management,
  mandatory tenant isolation (VETO-F10), encryption by default, and fail-closed security properties.
---

# Contextra Security Audit – Sicherheits- & Datenschutz-Auditierung

Dieser Skill leitet Agenten und Entwickler an, die Sicherheits- und Datenschutzvorgaben (Befunde S-01 bis S-19 aus [PLAN.md §6](file:///home/freddy/Projekte/contextra/PLAN.md#L200)) im Code zu prüfen und durchzusetzen.

---

## 1. Zentrale Sicherheits-Defaults (Inversionsliste)

In Contextra gelten ausnahmslos sichere Standardeinstellungen („Secure by Default“):

| Befund | Unsicherer Ist-Zustand | Normativer Soll-Zustand (SPEC §0.2 / PLAN §12.3) | Fundstelle |
|---|---|---|---|
| **S-01** | Passphrase via HKDF-SHA256 | **Argon2id KDF**: $m \ge 64\,\text{MiB}, t \ge 3, p \ge 1$ (`try_new_with_kdf`) | `crates/contextra-crypto/src/crypto.rs` |
| **S-02** | `encryption_mode: None` | **`EncryptionMode::Required`**: Klartext nur über explizite Opt-in-Typen | `crates/contextra/src/builder.rs` |
| **S-03** | `TenantPolicy::Optional` | **`TenantPolicy::Required`**: Strikte Mandantentrennung als Pflicht | `crates/contextra-engine/src/lib.rs` |
| **S-04** | Auto-Extraktion `Enabled` | **`AutoExtractionMode::Disabled`**: Keine automatische Weiterleitung an LLM | `crates/contextra-types/src/types/domain/misc.rs` |
| **S-05** | Secret als Plain `String` | **`SecretString` / `Zeroizing<T>`**: Kein Klonen von Passphrasen in ungelöschte Puffer | `crates/contextra/src/builder.rs` |
| **S-06** | ZIP-Bomb in ODF | **Hard-Limit auf dekomprimierte Bytes**: Budgetprüfung vor `read_to_end` | `crates/contextra-ingest/src/formats/odf/` |
| **S-07** | EPUB Fail-Open | **Fail-Closed bei DRM/Krypto-Fehlern**: Unlesbare/verschlüsselte Bücher ablehnen | `crates/contextra-ingest/src/formats/web/epub.rs` |
| **S-16** | WAL Key im Klartext | **`KeyManager`-geschützt**: Kein Speichern von `.wal_integrity_key` im Klartext | `crates/contextra-store/src/wal/hmac.rs` |

---

## 2. Audit-Checkliste für den Entwickler

### 1. KDF-Ableitung (S-01)
Suche nach Aufrufen von `KeyManager::try_new`:
```bash
grep -rn "KeyManager::try_new(" crates/
```
- **Prüfung**: Produktionspfade müssen `KeyManager::try_new_with_kdf` oder die Argon2id-Ableitung verwenden.
- Die Konstante `MIN_M_COST_KIB` in `contextra-crypto/src/kdf.rs` muss mindestens 65536 (64 MiB) betragen.

### 2. Mandanten-Isolation (VETO-F10 & S-03)
Suche nach mandantenlosen Aufrufen:
```bash
grep -rn "TenantPolicy::Optional" crates/
grep -rn "\.collection(" crates/contextra-mcp/
```
- **Prüfung**: `contextra_delete` und Lese-Operationen müssen zwingend die Mandanten-ID aus dem `RequestContext` validieren.

### 3. Geheimnis-Bereinigung (Zeroize, S-05 & S-15)
- Alle sensiblen Schlüsselstrukturen (z. B. `DocDek`, `Kek`, `MasterKey`) müssen `Zeroize` oder `ZeroizeOnDrop` implementieren.
- Niemals `format!("{:?}", secret)` verwenden; `Debug`-Implementierungen müssen `[REDACTED]` ausgeben.

---

## 3. Automatische Sicherheits-Checks

Führe zur Auditierung folgende Kommandos aus:

```bash
# 1. Suche nach unverschlüsselten Builder-Defaults
grep -rn "encryption_mode.*None" crates/contextra/src/

# 2. Suche nach unsicheren ZIP-Readern
grep -rn "read_to_end" crates/contextra-ingest/

# 3. Supply-Chain & Dependency Audit
cargo deny check advisories || true
```

Weitere Details: siehe [references/security_checklist.md](file:///home/freddy/Projekte/contextra/.agents/skills/contextra-security-audit/references/security_checklist.md).
