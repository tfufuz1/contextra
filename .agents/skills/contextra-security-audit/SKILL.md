---
name: contextra-security-audit
description: >-
  Use this skill to audit Contextra security defaults: Argon2id bounds, mandatory encryption without opt-out,
  mandatory tenant context, key zeroization, unsafe islands, process hardening and fail-closed behaviour, according to SPEC chapter 10.
---

# Sicherheits-Audit

Maßgeblich: [`docs/SPEC.md`](../../../../docs/SPEC.md), Kapitel 10, D-05, D-06, D-07, D-29, D-52, D-60, D-63, D-68 und HR-1 bis HR-4.

## 1. Sicherheits-Defaults (Soll)

| Befund | Soll (SPEC) |
|---|---|
| Passphrasen-Ableitung | nur Argon2id, `m` 64 MiB bis 4 GiB, `t` 3 bis 16, `p` 1 bis 16 (D-06); HKDF nur für Subschlüssel |
| Standardparameter | `m` 128 MiB, `t` 3, `p` 4 (D-68) |
| Verschlüsselung | immer aktiv, kein Klartextpfad, kein Schalter (D-05) |
| Mandanten | `TenantContext` Pflicht; `TenantPolicy` entfällt (D-07) |
| Schlüssel im Speicher | `Secret` ohne `Debug`, `Display`, `Clone`; `zeroize` beim Verwerfen |
| Unsafe | nur `contextra-sys` und `contextra-simd` (D-29); kein `mmap` (D-94) |
| Prozesshärtung | `Required`, `BestEffort` (Standard) oder `Off`; Status im Bericht (D-63) |
| Fehlerverhalten | fail-closed; `fsync`-Fehler sind fatal für die Datei (D-55) |
| Netzwerk | kein Netzwerkzugriff, keine Telemetrie (HR-1, HR-2) |

## 2. Prüfschritte

### a) Passphrasen-Ableitung (D-06)

```bash
grep -rn "hkdf\|Hkdf" crates/contextra-crypto/src | head -n 30
```

Jeder Aufruf mit der Passphrase als Eingabe ist ein Befund. HKDF ist nur für Subschlüssel aus `K_master` zulässig.

Die Untergrenze des Speicherparameters prüfen: `MIN_M_COST_KIB` in `crates/contextra-crypto/src/kdf.rs` muss 65536 KiB (64 MiB) betragen. Im Ist-Zustand steht 19456 (SPEC F-11). Das ist ein offener Befund.

### b) Schalter für Sicherheit (D-05, HR-13)

```bash
grep -rn "EncryptionMode\|allow_legacy\|legacy-wal-key\|encryption-at-rest\|Option<Arc<KeyManager>>" crates/
```

Erwartung: keine Treffer in Produktionscode.

### c) Mandantenpflicht (D-07, VETO-F10)

```bash
grep -rn "TenantPolicy" crates/
```

Erwartung: keine Treffer.

### d) Unsafe-Inseln (D-29)

```bash
grep -rln "unsafe" crates/*/src
grep -rn "mmap\|memmap2" crates/*/src
```

Erwartung: `unsafe` nur in `contextra-sys` und `contextra-simd`; `mmap` und `memmap2` keine Treffer.

### e) Logs und Fehler (HR-4, T-LOG-1)

Fehlertexte und Logaufrufe enthalten keine Inhalte, Schlüssel, Klartext-Kennungen oder Pfade mit Personenbezug.

## 3. Befunde

Melden Sie jeden Verstoß mit SPEC-ID (z. B. `D-06`, `D-05`, `D-07`), Datei und Zeile. Die Ergänzungen zur Checkliste stehen in [`references/security_checklist.md`](references/security_checklist.md).
