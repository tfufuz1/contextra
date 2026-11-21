# contextra-sys — Implementierungsspezifikation

**Ebene:** E0 (Fundament, `unsafe`-Insel, G-02)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch). **Stand:** 21.11.2026, Abgleich mit SPEC v4.1. Verweist nur auf SPEC-Kapitel und IDs. Ist-Abweichungen sind markiert, ungeklärte Punkte stehen unter „Offene Punkte (Entscheidung nötig)“ in Abschnitt 6.

## 1. Zweck und Ebene
Systemnahe Funktionen: mlock, Dumpable, Direct-IO (SPEC 15, ADR-41). Zusammen mit `contextra-simd` die einzigen Crates mit erlaubtem `unsafe` (R5, G-02). Keine internen Abhängigkeiten (Anhang E). Ziel-Umfang: 2.500 Zeilen (SPEC 3.1). Ist-Umfang laut Zielspalte nicht gemessen.

## 2. Vertrag
SPEC 15 nennt keine Traits. Die öffentliche API besteht aus den Funktionen mlock, Dumpable und Direct-IO. Verweis: T-OPS-09 (mlock-Verweigerung führt zu `Refused`).
Ist: `mlock.rs`, `mmap.rs`, `posix.rs`, `durability.rs`, `error.rs`, `lib.rs`. Die Signaturen sind gegen SPEC 15 zu prüfen (offen, siehe 6).

## 3. Abhängigkeiten
**Erlaubt (Anhang E):** keine.
**Ist (Cargo.toml):** `memmap2`, `thiserror`, `tracing`.
**Ist-Abweichung:** `memmap2` und `tracing` sind in SPEC 15 nicht genannt. Entscheidung nötig (siehe 6).
**Features:** `default = []` wird vorausgesetzt (R3). Keine Features im Soll.

## 4. Module
| Datei | Verantwortung (Ist) | Befund |
|---|---|---|
| `src/mlock.rs` | Speichersperre | Soll (SPEC 15) |
| `src/posix.rs`, `src/mmap.rs` | POSIX-Aufrufe, Speicherabbildung | Soll, zu prüfen |
| `src/durability.rs` | Durability-Hilfen | zu prüfen, ob Soll (SPEC 15 nennt Direct-IO) |
| `src/acl_win32.rs` | Win32-ACL | SPEC 3.1: in Übergangsphase noch in `store`, siehe `tests/unsafe_islands.rs`. Ist-Abweichung: Verbleib offen |
| `src/vault.rs` | Vault-Bezug | nicht in SPEC 15 genannt (REM-01: Vault entfernt). Ist-Abweichung |
| `src/contract.rs` | Selbstdeklaration | REM-14, entfernen |

## 5. Entfernungen
| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| REM-14 | `src/contract.rs`, ANCHOR-Kommentare (11 Dateien) | `grep -rl "ANCHOR\[\|FILE-CONTEXT" crates/contextra-sys` leer |
| REM-01 | `src/vault.rs`, falls Vault-Bezug | `grep -rn "vault" crates/contextra-sys/src` leer |

## 6. Umbau
1. Selbstdeklarationen entfernen (nach Freigabe, CLAUDE.md §6 Punkt 9).
2. `vault.rs` prüfen und entfernen, falls der Bezug nur REM-01 ist.
3. Signaturen von mlock, Dumpable und Direct-IO gegen SPEC 15 abgleichen.

**Offene Punkte (Entscheidung nötig):**
- **O-1** `acl_win32.rs`: bleibt es in `sys` oder wandert es zu `store` zurück? SPEC 3.1 sagt Übergangsphase, Ziel nicht festgelegt.
- **O-2** Abhängigkeiten `memmap2` und `tracing`: nicht in SPEC 15.
- **O-3** `durability.rs` und `posix.rs`: Soll-Zugehörigkeit nicht in SPEC 15.

## 7. Tests
- T-OPS-09: mlock-Verweigerung führt zu `Refused` (SPEC 15).
- Unsafe-Inseln: `tests/unsafe_islands.rs` (G-02) auf Workspace-Ebene.

## 8. Abnahme
| Gate | Ist-Befund | Soll |
|---|---|---|
| G-02 | 6 Dateien mit `unsafe` (erlaubt, `sys` ist Insel), kein `forbid` | erlaubt |
| G-09 | 11 Dateien mit ANCHOR/FILE-CONTEXT, `contract.rs` vorhanden | verletzt (REM-14) |
| G-13 | 0 Treffer | eingehalten |
| G-12 | 0 Treffer | eingehalten |
