---
name: contextra-doctor
description: >-
  Use this skill to diagnose and repair Contextra build failures, workspace manifest inconsistencies,
  compiler errors, Vfs compliance issues, and Cargo dependency issues according to docs/SPEC.md.
---

# Contextra Doctor – Build & Workspace Reparaturanleitung

Dieser Skill leitet Entwickler und Agenten an, den Contextra-Workspace in Übereinstimmung mit [docs/SPEC.md §5](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md) sauber, baubar und lint-frei zu halten.

---

## 1. Workspace-Integrität & Manifest-Prüfung

### Prüfung
```bash
cargo metadata --no-deps --format-version 1 > /dev/null
```
- **Exit-Code 0**: Workspace-Manifeste und Pfade sind konsistent.
- **Exit-Code 101**: Mindestens ein Workspace-Mitglied oder eine Pfad-Abhängigkeit existiert nicht im Dateisystem.

### Soll-Workspace (SPEC §5.1)
Der Ziel-Workspace besteht ausschließlich aus:
- **Produktion (14 Crates)**: `contextra-types`, `contextra-ports`, `contextra-sys`, `contextra-simd`, `contextra-durable-fs`, `contextra-crypto`, `contextra-mvcc`, `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-rank`, `contextra-erasure`, `contextra-engine`, `contextra`.
- **Tooling/Dev (4 Crates/Pakete)**: `contextra-testkit`, `contextra-lab`, `ctx-verify`, `tests/arch`.

Alle anderen Crates dürfen weder in `members` noch in `dependencies` referenziert werden.

---

## 2. Bekannte Compiler- und Baufehler (Runbooks)

### Runbook 1: Ungültige Feature-Flags oder `unexpected_cfgs`
- **Befund**: `#![allow(unexpected_cfgs)]` maskiert fehlerhafte Features (wie toten `wal-integrity` Code).
- **Behebung**:
  - `allow(unexpected_cfgs)` entfernen.
  - Nicht deklarierte Feature-Gates tilgen und die Funktionalität verbindlich in den Produktivpfad integrieren oder entfernen.

### Runbook 2: Vfs-Konsistenz im Store (D-04)
- **Befund**: Direkte Nutzung von `std::fs` oder `tokio::fs` im Store.
- **Behebung**:
  - Store nutzt ausschließlich den `Vfs`-Port aus `contextra-ports`.
  - Fehler bei Dateioperationen und `sync_*` werden ausnahmslos mit `?` propagiert.

### Runbook 3: Snapshot-Sentinel (D-C02)
- **Befund**: SSI-Commits scheitern mit „snapshot too old“, weil `prune_bound` auf `u64::MAX` fällt und Tombstone-Bits maskiert werden.
- **Behebung**:
  - `Seq` auf `NonZeroU64` ohne Sentinel umstellen (WP-2.2).
  - Maskierung in `ssi.rs` durch Fehlerprüfung ersetzen (WP-2.1).
