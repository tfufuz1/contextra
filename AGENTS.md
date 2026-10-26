# Contextra — Agenten-Betriebsanleitung (AGENTS.md)

Stand: 2026-10-25 (Spezifikation Fassung 1.2)

## 1. Geltungsbereich und Rangfolge der Informationsquellen (§0, §1)
Diese Datei regelt die Arbeit aller autonomen Agenten im Repository.
Bei Konflikten gilt zwingend folgende strikte Rangfolge:
1. **Normative Gesamtspezifikation Fassung 1.2** (`docs/SPEC.md`; §0, §1)
2. **Vetos und Architekturprinzipien** (§4.1, §5)
3. **Capabilities Manifest** (`capabilities.toml`; INV-DOC-SINGLE-SOURCE-1)
4. **Agenten-Betriebsanleitung** (`AGENTS.md`)
5. **Produktionscode und grüne Quality Gates** (REQ-QA-010)

Quality Gates dürfen niemals durch Ausnahmen, Sonderregeln oder lokale Overrides geschwächt oder umgangen werden (REQ-QA-010).

## 2. Start
Arbeitskontext zu Beginn der Session laden:
- `cargo check --workspace`
- `cargo xtask preflight`

> Jules-spezifischer Bootstrap: siehe `.agents/jules/SESSION_BOOTSTRAP.md`

## 3. Ablauf
Jeder Task folgt diesem iterativen Ablauf:
1. Task lesen und Ziel verstehen.
2. Existenz von APIs, Dateien und Typen im Code prüfen (niemals Schnittstellen aus dem Gedächtnis annehmen).
3. Plan mit Dateiliste und Akzeptanzbefehlen erstellen.
4. Kleinste mögliche Änderung durchführen.
5. Verifizieren und PR-Text erstellen.

## 4. Verifizieren & Gate-Integrität (§27.5, §27.6, REQ-QA-010)
- „Fertig“ bedeutet ausnahmslos: Alle relevanten Gates, Lints und Tests auf dem Merge-Ergebnis sind grün (REQ-QA-010).
- Es ist streng verboten, Gates, Lints, Ratchets oder den Toolchain-Pin einzuschränken, abzuschwächen oder durch Ausnahmen zu umgehen (REQ-QA-010).
- Geschützte Pfade: `.github/**`, `xtask/**`, `justfile`, `rust-toolchain.toml`, `deny.toml`, `capabilities.toml`, `AGENTS.md`, `.agents/jules/setup/**`.

## 5. Kernregeln aus der Spezifikation (Kurzform)
- **Zero-Panic (§3, §27.6):** Kein `unwrap()`, `expect()` oder `panic!()` in Produktionspfaden; Fehler per `Result` propagieren.
- **Unsafe-Isolierung (§6.7):** `unsafe` ist streng isoliert auf die drei Unsafe-Inseln laut `capabilities.toml` (`contextra-simd`, `contextra-sys`, `contextra-wire`). Alle anderen Crates erzwingen `#![forbid(unsafe_code)]`.
- **Single-Node & Ein-Prozess-Garantie (§1, §4.1):** Embedded Library (`cargo add contextra`), kein Server, kein Sharding, kein P2P.
- **Determinismus (§3, §5, P28):** Zeit, Zufall und IDs ausschließlich über injizierte Ports (`Clock`, `Rng`, `IdGenerator`).
- **Löschaussagen & Transparenz (INV-CLAIMS-1, §4.3, §20.12):** Aussagen zur Löschung folgen zwingend der Außenformulierung (§4.3) und Claim-Taxonomie (§20.12). Werbe- oder Pauschalaussagen zur Löschwirkung oder Rechtskonformität sind unzulässig.
- **Kantenregeln (§6.10):** Unidirektionaler Datenfluss Ring 4 → 3 → 2 → 1 → 0. Der Kern ist geschlossen (§6.8).
