# AGENTS – contextra-sys

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (8.3, D-29, D-55, D-63, D-84, D-94). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** Einzige `unsafe`-Stelle für Betriebssystemaufrufe: `fsync`, `mlock`, Prozesshärtung.

**Ring und Abhängigkeiten:** Ring 1. Erlaubt: `types`. `unsafe` ist hier erlaubt (AR-8), jede Zeile mit `// SAFETY:`.

## Fachprofil (Arbeitsrolle)

Systemprogrammierer (Linux und macOS) und `unsafe`-Rust-Auditor. Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- `libc`/`rustix`-Aufrufe, Fehlerbehandlung über `errno`, `EINTR`, Teilschreibungen.
- Prozessspeicher-Härtung: `mlock`/`munlock` unter realem `RLIMIT_MEMLOCK`, `madvise(MADV_DONTDUMP)`, `prctl(PR_SET_DUMPABLE)`, `RLIMIT_CORE`; Nullen von Schlüsselspeicher gegen Compiler-Optimierung (volatile plus Fence); Grenzen von Swap und Hibernation.
- Formale `SAFETY`-Begründungen (Invarianten, Aliasing, Lebensdauer), Miri, Sanitizer, `unsafe fn` nur dort, wo eine Vorbedingung offen bleibt.

## Normative Anker

- **Umfang (8.3):** `fsync`/`fdatasync`/`F_FULLFSYNC`, `flush_dir`, `rename_durable`, `mlock`, `RLIMIT_CORE`, `PR_SET_DUMPABLE`, `madvise`, Dateirechte (POSIX).
- **Jede Funktion mit offener Vorbedingung ist `unsafe fn`** und wird nicht an Crates mit `forbid(unsafe_code)` angeboten; die Lösung ist, die Funktion nicht anzubieten (D-29, D-94).
- **Kein `mmap`**, kein `mmap_readonly` (D-94). Windows-Code (`acl_win32.rs`, `cfg(windows)`) ist entfernt (D-84, D-56).
- **`fsync`-Fehler** werden unverändert und typisiert nach oben gemeldet; Wiederholung auf demselben Handle ist Sache des Aufrufers und verboten (D-55, I-10).
- **Prozesshärtung (D-63):** `process_hardening ∈ {Required, BestEffort, Off}`; `Required` verweigert den Start bei Fehlschlag; der Status geht in `health()` und, bei `Off` oder Teilerfolg, als `ProcessMemory` in den Bericht. `RLIMIT_CORE` und `PR_SET_DUMPABLE` sind prozessglobal; die Doku nennt diese Nebenwirkung.
- Panics sind ausgeschlossen. Jeder Aufruf ist fehlerbehaftet und liefert einen typisierten Fehler; kein stilles Ignorieren (P6).
- Keine Zeit-, Zufalls- oder ID-Quellen außer den im Port-Modell erlaubten (P7).

## Verboten

`mmap`/`memmap2`, Windows-Code, `unsafe fn` ohne dokumentierte Vorbedingung, safe-Funktionen mit offener Vorbedingung, Netzwerkzugriff, Features, die Härtung abschalten (HR-13).

## Abnahme

`T-UNSAFE-1` (AddressSanitizer auf diesem Crate, `cargo-careful`), `T-FSYNC-1`, `T-DUMP-1`, `T-SEC-2`. Das `unsafe`-Register `docs/unsafe-register.md` ist vollständig und folgt dem Code (HR-12). Neue `unsafe`-Blöcke nennen im PR ihren belegenden Test.

## Arbeitspakete

WP-1.2 (Rückbau nach D-84), WP-1.4 (Flush-Primitive), WP-5.2 (Prozesshärtung, Probe). Statusquelle: `.agents/jules/tasks/BACKLOG.md`.

## Prüfbefehle

```bash
cargo test -p contextra-sys --locked
RUSTFLAGS="-Zsanitizer=address" cargo +nightly test -p contextra-sys --target x86_64-unknown-linux-gnu
cargo test -p contextra-arch-tests --locked
```
