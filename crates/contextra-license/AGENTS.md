# AGENTS.md — `contextra-license`

> **Ring 4 (Extension)** · Klasse `[E]` (Extension) · Reifegrad `experimental` · Unsafe: nein · Rolle: Signierte Lizenzprüfung (`LicenseGate`).
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-license --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` |
| Features | keine |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- Wird von der Fassade hart gezogen (Ist); Soll: optional.
- Der Kern kennt **kein Lizenzkonzept** (SPEC §4.7); Anbindung nur über den Port `LicenseGate`.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `lib.rs`, `signed_gate.rs` | `SignedLicenseGate` |

## 4. Verträge & Invarianten

1. **Degradieren, nie sperren (INV-LICENSE-1/2/3):** Verstoß, Ablauf oder fehlende Aktivierung degradieren auf Basisfunktion, nie Datenverlust oder -sperre; niedrigste Klasse ist immer `Ok`.
2. Falsche Installationsbindung ⇒ `NotActivated` ohne Informationsleck.
3. Zeit über `Clock`.

## 5. Ports

**Implementiert:** `LicenseGate`.

## 6. Nebenläufigkeit & Performance

- Zustandslos.

## 7. Anti-Patterns

- Lizenzprüfung im Datenpfad von Store/Index.
- Datenverweigerung bei Ablauf.

## 8. Tests & Messung

```bash
cargo test -p contextra-license --locked
cargo clippy -p contextra-license --all-targets -- -D warnings
```

- 13 Testdateien.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 5.3 | optional machen |
