# Sicherheit

Maßgeblich: [`SPEC.md`](SPEC.md) Abschnitte 4.2 (D-55, D-56, D-63, D-67, D-80), 6.2 (H3, H4), 6.3 (HR-8), 17.3 und Anhang D.

## 1. Schwachstellen melden

**Meldeweg:** [Platzhalter – verschlüsselter Meldekanal vor dem ersten Release vom Maintainer einzutragen.] SPEC 17.3 verlangt einen verschlüsselten Meldeweg. Im Repository ist er derzeit nicht hinterlegt. Bitte keine Details über öffentliche Issues melden.

**Fristen.**

- Frühwarnung innerhalb von 24 Stunden und Meldung innerhalb von 72 Stunden bei aktiv ausgenutzten Schwachstellen oder schweren Vorfällen nach dem Cyber Resilience Act, Art. 14 (SPEC H4, D-67). Die Meldepflicht gilt laut SPEC seit dem 11.09.2026 über die ENISA Single Reporting Platform.
- Interne Bearbeitungsfristen nach Bestätigung (HR-8): kritisch 7 Tage, hoch 30 Tage.
- Unterstützungszeitraum: mindestens fünf Jahre Sicherheitsupdates je Hauptversion ab Veröffentlichung (D-67). Eine veröffentlichte Version gibt es noch nicht.

Die rechtliche Einordnung ist nicht Gegenstand dieses Dokuments. Sie steht in SPEC 6.2 und 6.7.

## 2. Sicherheitsmerkmale im Entwurf

- **Pflichtverschlüsselung.** Records und Blöcke mit AES-256-GCM-SIV. Keyslots mit Argon2id; Parameter gelten nur innerhalb fester Grenzen (`m` 64 MiB bis 4 GiB, `t` 3 bis 16, `p` 1 bis 16; D-06). Standard: 128 MiB, `t` 3, `p` 4 (D-68). Subschlüssel über HKDF; die Passphrase wird nie per HKDF abgeleitet (D-06).
- **Keine Schalter.** Es gibt keinen unverschlüsselten Modus, keinen Integritäts-Downgrade und keinen Schlüssel für ältere WAL-Formate (D-05, F-21).
- **Integrität.** WAL als HMAC-Kette, Dateikopf mit Format-Version und Algorithmus-IDs, Schlüsselwrap mit gebundener AAD (D-52, D-60, D-42).
- **Schlüsselvernichtung.** Löschung über den Schlüsselbaum. Dateischlüssel werden vor dem Entfernen einer Datei genullt (D-79, D-85).
- **Prozesshärtung.** Optionen `Required`, `BestEffort` (Standard) und `Off`: `RLIMIT_CORE = 0`, `PR_SET_DUMPABLE = 0`, `MADV_DONTDUMP` für Schlüsselseiten, `mlock` nach Möglichkeit. Diese Maßnahmen wirken prozessweit. Der Status steht in `health()` und im Bericht (D-63).
- **`fsync`.** Ein Fehler ist für die betroffene Datei fatal. Der Store wechselt in den Zustand `Poisoned`. Blindes Wiederholen findet nicht statt (D-55).
- **Unsafe-Code.** Nur in `contextra-sys` und `contextra-simd`. Alle anderen Crates setzen `#![forbid(unsafe_code)]`. Keine Speicherabbildung (D-29, D-94).
- **Netzwerk und Telemetrie.** Keine Netzwerk-Bibliothek im Produktionsbaum (HR-1, HR-2, D-54).
- **Logs und Fehler.** Keine Nutzerinhalte, Schlüssel, Klartext-Kennungen oder personenbezogenen Pfade (HR-4, T-LOG-1).

## 3. Bekannte Restrisiken

Alle Restrisiken mit Bedingungen stehen in SPEC Anhang D. Jeder Bericht eines Löschvorgangs führt die zutreffenden auf. Das betrifft insbesondere:

- `FlashRemanence`, `FsJournalOrCow`, `FreedBlockRemanence`: Medium und Dateisystem.
- `WrappedKeyRemanence`, `OldKeyslotRemanence`: Schlüsselreste auf dem Medium.
- `ProcessMemory`: Klartext im Prozessspeicher, Swap oder Coredump.
- `IndexTopologyInference`, `TextIndexUntilPurge`: Index-Rückstände bis zur Reparatur oder zum Rewrite.
- `ExternalCopies`, `ExternalProcessors`: Kopien außerhalb von Contextra.

## 4. Betreiberpflichten

- Passphrase sicher verwalten. Es gibt keine Wiederherstellung (HR-3).
- Volume-Verschlüsselung mit Schlüsselvernichtung einsetzen, wenn der Schutzbedarf hoch ist.
- Swap verschlüsseln oder deaktivieren. Coredumps abschalten.
- Backups und Ledger nach SPEC 16 (Punkt 5) und 11.8 behandeln. `backup_retention` muss mindestens so lang sein wie die Aufbewahrung der Sicherungen.

## 5. Lieferkette

- Signierte Quell- und Binärartefakte (Sigstore oder minisign), SBOM im Format CycloneDX, Bericht von `cargo-audit` und `cargo-deny` je Release (SPEC 17.1).
- Advisory-Ausnahmen in `deny.toml` nur mit Ablaufdatum (F-17, 17.3).
- Keine `continue-on-error`-Option auf Sicherheitsgates (17.3).
- Build mit `--locked` und einer Toolchain nach D-82. Die Version steht im Evidenzbündel.
