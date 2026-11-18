# contextra-crypto — Crate-Spezifikation (Kryptographische Primitive)

**Ebene:** E1a (Kern-Closure: `ids`, `env`)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-crypto`  
**Referenz:** `docs/SPEC.md` §4, §5, §7

---

## 1. Zweck & Systemrolle

`contextra-crypto` implementiert die kryptographischen Kernalgorithmen für Contextra:
- **Authenticated Encryption with Associated Data (AEAD):** Schutz aller vertraulichen Daten der Klassen K und P vor unbefugter Einsichtnahme und Manipulation.
- **Key Derivation Functions (KDF):** Deterministische und kryptographisch getrennte Schlüsselableitung aus Master-, Mandanten- und Betroffenen-Schlüsseln.
- **Sichere Speicherverwaltung:** Garantierte Löschung (`ZeroizeOnDrop`) allen Schlüsselmaterials aus dem RAM.
- **Kryptographische Bereinigung:** Slot-Nullung und Vernichtung von DEKs für das DSGVO-Krypto-Shredding.

---

## 2. Kryptographische Standards & Algorithmen

| Funktion | Algorithmus | Standard / RFC | Parameter |
|---|---|---|---|
| **AEAD Chiffre** | AES-256-GCM-SIV | RFC 8452 | 256-Bit Key, 96-Bit Nonce, 128-Bit Tag. Misuse-resistant. |
| **Key Derivation** | HKDF-SHA-256 | RFC 5869 | Expand- und Extract-Routinen mit Domain Separation Strings |
| **Passphrase KDF** | Argon2id | RFC 9106 | $m=256\text{MB}$, $t=4$, $p=1$, 32-Byte Salt |
| **Zufall (RNG)** | OS CSPRNG | `getrandom` | Fehlbar (`try_fill_bytes`). Kein deterministischer Fallback |
| **Memory Sanitization** | Zeroize | Rust `zeroize` | `ZeroizeOnDrop` für alle Schlüsselstrukturen |

---

## 3. Schlüsselhierarchie & Domänen-Trennung (SPEC §4)

```
MasterKey (256 Bit)
   │
   ├── HKDF-Expand(info="contextra/tenant/v1" || tenant_id)
   │      │
   │      └── TenantKEK
   │             │
   │             ├── HKDF-Expand(info="contextra/collection/v1" || coll_id || epoch)
   │             │      │
   │             │      └── CollectionKEK
   │             │             │
   │             │             ├── HKDF-Expand(info="contextra/doc/v1" || doc_id || ver)
   │             │             │      └── DocumentDEK (Klasse K)
   │             │             │
   │             │             └── HKDF-Expand(info="contextra/subject/v1" || subject_ref)
   │             │                    │
   │             │                    └── SubjectKey (Klasse P)
   │             │                           │
   │             │                           └── HKDF-Expand(...)
   │             │                                  └── SubjectDocDEK
   │             │
   │             └── (Neue Epoche bei Collection-Neuanlage nach Drop)
   │
   └── HKDF-Expand(info="contextra/erasure/v1")
          └── ErasureKey (für Audit-Journal und Ledger)
```

### 3.1 AAD-Bindung (Additional Authenticated Data)
Jede AEAD-Verschlüsselung bindet zwingend Kontextinformationen ein:
$$\text{AAD} = \text{TenantId} \,\|\, \text{CollectionId} \,\|\, \text{DocId} \,\|\, \text{Version}$$
Ein Chiffretext, der in eine andere Collection, einen anderen Mandanten oder eine andere Version kopiert wird, schlägt bei der Entschlüsselung fehl.

---

## 4. Rolle in der DSGVO-Löschkette

- **Schritt 3 (Schlüsselvernichtung):** 
  - Überschreibt den Schlüsselslot mit kryptographischen Zufallsbytes aus dem OS CSPRNG.
  - Überschreibt den Slot anschließend mit Nullen (`0x00`).
  - Führt ein `fsync` auf die Keyring-Datei aus (`contextra-durable-fs`).
- **Schritt 4g (Slot-Bereinigung):**
  - Implementiert `ErasureLayer` zur physischen Bereinigung von Slot-Fragmenten und temporären FileKeys im Speicher und auf Platte.

---

## 5. Invarianten

1. **`#![forbid(unsafe_code)]`:** Vollständig speichersicher.
2. **Keine stillen Fallbacks:** Fehlt ein Schlüssel oder schlägt die Entschlüsselung fehl, MUSS sofort `Err(ContextraError::Crypto(...))` zurückgegeben werden. Es gibt keine Pseudo-Schlüssel oder Klartext-Notfallmodi.
3. **Schlüsseltypen ohne Debug & Clone:** Schlüsselmaterial implementiert weder `Clone`, `Debug` noch `Serialize`, um versehentliches Logging oder Kopieren im Speicher zu verhindern.
4. **Constant-Time Vergleiche:** Tags und Hashes werden ausschließlich mit `subtle::ConstantTimeEq` verglichen (Timing-Attack Schutz).
