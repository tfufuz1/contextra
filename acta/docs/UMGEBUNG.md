# Entwicklungsumgebung (Cloud-VM) – Stand 2026-11-15

Verifizierte Befunde für Claude Code und Menschen. Nichts hiervon ist Teil der SPEC.

## Plattform
Ubuntu 24.04, x86_64, 2 vCPU, ca. 7,8 GiB RAM, ca. 31 GiB frei. Repo unter `/root/acta`. Das Shell-Arbeitsverzeichnis wird nach jedem Aufruf zurückgesetzt: absolute Pfade oder `cd /root/acta &&` verwenden. Vorhanden: git, gh, rg, jq, zip, gcc, clang, python3 (pip erreichbar), rustup 1.28.2 mit `stable` (1.97.0). Nicht vorhanden: conda, mold, nightly.

## Wo die Konfiguration greift
`CLAUDE.md`, `.claude/` (Settings, Hook, Skills, Agenten, Befehle) gelten nur, wenn Claude Code **im Repo-Wurzelverzeichnis** startet. In der Cloud-VM startet die Sitzung standardmaessig in `/home/claude`, das Repo liegt aber in `/root/acta`: dort zuerst `cd /root/acta` bzw. die Sitzung im Repo oeffnen, sonst wird nichts davon geladen. Die Bootstrap-Skripte greifen nur auf Linux x86_64 und nur, wenn `static.rust-lang.org` nicht erreichbar ist; lokal gilt `rustup` (nutzt `rust-toolchain.toml`). Standardpfad ist `$HOME/toolchains`.

## Netz (getestet)
| Ziel | Status |
|---|---|
| index.crates.io, static.crates.io/crates/… | erreichbar |
| conda.anaconda.org | erreichbar |
| pypi.org | erreichbar |
| static.rust-lang.org | blockiert (daher kein `rustup install`) |
| repo.anaconda.com, api.anaconda.org | blockiert |

## Einrichtung (automatisch per SessionStart-Hook)
`.claude/settings.json` startet `scripts/session-start.sh`. Es prüft und holt bei Bedarf nach:
1. `scripts/bootstrap-toolchain.sh`: Rust 1.89.0 aus conda-forge (`rust` + separat `rust-std`, ohne std folgt E0463), Symlink `~/.rustup/toolchains/1.89.0-x86_64-unknown-linux-gnu` (`rustup toolchain link 1.89.0` wird abgelehnt). Test: frischer Lauf und Wiederholung grün.
2. `scripts/bootstrap-tools.sh`: just 1.9.0, cargo-deny 0.20.2, cargo-audit 0.22.1, cargo-cyclonedx 0.5.9, cargo-mutants 27.1.0, cargo-nextest 0.9.99, cargo-llvm-cov 0.9.1 (conda-forge-Binaries, feste Versionen, Links nach `~/.cargo/bin`).
3. Optional, nicht im Hook (Kompilierung dauert Minuten): `cargo +stable install cargo-geiger --locked --root /root/toolchains/geiger` (0.14 braucht rustc ≥ 1.97, daher `+stable`; Symlink nach `~/.cargo/bin` setzen). G-SUPPLY verlangt den Geiger-Bericht (SPEC 16.6).
Die VM ist flüchtig: `/root/toolchains` kann nach einem Neustart fehlen, der Hook stellt es wieder her.

## Nicht verfügbar
- **nightly, Miri, Sanitizer, cargo-fuzz:** brauchen nightly von static.rust-lang.org (blockiert); conda-forge führt kein nightly und kein cargo-fuzz. Fuzzing und Miri (SPEC 3.8 nennt `cargo-fuzz`) laufen in CI mit Netz oder lokal. Nicht als „geprüft“ melden.
- **loom, proptest, trybuild** sind normale crates.io-Abhängigkeiten und funktionieren.

## Befunde aus dem Werkzeuglauf (offen, bewusst nicht „grün gefärbt“)
Stand: `cargo check`, `test`, `nextest` (8 Tests), `clippy -D warnings`, `fmt --check`, `cargo xtask gates` und `cargo deny check advisories|sources` sind grün. `cargo deny check licenses bans` ist rot:
1. **bans, `wildcards = "deny"`:** schlägt bei allen internen Pfad-Abhängigkeiten (`acta-x = { path = … }` ohne Version) an. Vorschlag: `allow-wildcard-paths = true` in `[bans]` (betrifft nur Pfad-Abhängigkeiten, die Workspace ist `publish = false`). Das ändert ein Gate und braucht eine bewusste Entscheidung (ADR).
2. **licenses:** 32 Fehler, ausschließlich die eigenen Crates (`LicenseRef-Proprietary`, nicht in der Whitelist). Vorschlag: `[licenses.private] ignore = true`. Fremdcode ist unauffällig. Ebenfalls Entscheidung nötig.
Zusätzlich: `acta-verify` steht in `rings.toml` unter `[levels]` (9) und `[isolated]` (1); der Wert unter `[isolated]` ist ohne Bedeutung, die Konfiguration sollte vereinheitlicht werden.

## Lauf- und Zeitangaben
`cargo check --workspace --all-targets`: ca. 22 s (warm), Gates ca. 2 s, `cargo test` wenige Sekunden. Kalt-Build mit Download ca. 1 bis 2 min. Lange Läufe im Hintergrund starten und das Log auswerten; Exit-Codes nie über eine Pipe ermitteln.
