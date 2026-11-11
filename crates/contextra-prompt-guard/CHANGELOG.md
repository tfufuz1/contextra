# Changelog

All notable changes to `contextra-prompt-guard` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
- Autarke Umwelt-Abstraktion `contract::Clock`.
- Kompatibilitätsmodul `compat::error`.
- Known-Answer-Tests (KATs) für Unicode-Evasion (Homoglyphen, Zero-Width-Zeichen, Bidi-Controls, Case-Insensitivity).
- Durchsatz- und Laufzeitschranken-Tests zur Vermeidung von DoS- oder ReDoS-Mustern.
- Property-Tests für Determinismus und Idempotenz von Erkennung und Quarantäne-Redaktion.
- Fuzzing-Harness unter `fuzz/fuzz_targets/fuzz_prompt_injection_guard.rs`.
- Standalone-Minimalbeispiel `examples/standalone.rs`.
- Vollständige Metadaten und `capabilities.fragment.toml` (Schema v3).

### Changed
- Default-Features auf `["std"]` reduziert (Entkopplung von `file-audit` aus den Standard-Features gemäß AD-9).
