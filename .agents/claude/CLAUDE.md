# Contextra — Claude / Antigravity-Betriebsanleitung

<!-- doc-stamp: 2026-11-01 | Spec-Fassung: 2.0.0 (SOLL-Zustand Release 1.0.0) | PLAN: 2026-11-01 -->

> **Normative Richtlinie:** Diese Datei gilt für Claude und Antigravity. Maßgeblich sind ausnahmslos [docs/SPEC.md](file:///home/freddy/Projekte/contextra/docs/SPEC.md), [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md) und [AGENTS.md](file:///home/freddy/Projekte/contextra/AGENTS.md).

---

## 1. Arbeitsweise

- **Ein PR = ein Arbeitspaket (WP) = höchstens ein Crate-Schnitt (REQ-CHG-2).**
- **Rot vor Grün (TDD):** Vor dem Fix zwingend ein fehlschlagender Regressionstest.
- **Evidenzpflicht:** Logs und Befehle unter `evidence/<WP-ID>/` ablegen.
- APIs und Dateipfade im Code und in `docs/SPEC.md` verifizieren — niemals aus dem Gedächtnis annehmen.
- Kleinste mögliche Änderung. Kein Over-Engineering.

---

## 2. Die 12 Kernregeln (AGENTS.md §2, SPEC §0.2)

1. **Ring- & Schichten-Disziplin:** Abhängigkeiten zeigen strikt abwärts (Ring 4 → 0). Innerhalb Ring 2 gilt Matrix SPEC §5.3.
2. **Zero-Panic in Produktion:** Kein `unwrap()`, `expect()`, `panic!()`, `todo!()`, `assert!()` oder ungeprüftes Index-Slicing (`[i]`) im Produktionscode.
3. **Determinismus & Port-Purity:** Zeit, Zufall und IDs stammen ausschließlich aus den Ports `Clock`, `Rng`, `IdGen` aus `contextra-ports`.
4. **WAL-First Consistency:** Frame(s) → Gruppen-Commit → `sync_data` WAL → Memtable → Sichtbarkeit.
5. **I/O ausschließlich über `Vfs`:** Kein direkter Aufruf von `std::fs` oder `tokio::fs`. Alle Sync-Fehler propagieren.
6. **Unsafe ausschließlich in 2 Inseln (D-29):** Nur in `contextra-simd` und `contextra-sys`. Jeder Block mit `// SAFETY:`. Alle anderen Crates erzwingen `#![forbid(unsafe_code)]`.
7. **Mandantentrennung verpflichtend (D-07):** `TenantId` stammt aus dem Kontext (`TenantPolicy::Required`). Store-Zugriff nur über `TenantScopedStorage`.
8. **Sicherheit ist kein Feature-Schalter (D-05, D-06):** AES-256-GCM-SIV Pflicht ohne Opt-Out. KDF: Argon2id mit $m \ge 64\,\text{MiB}, t \ge 3, p \ge 1$.
9. **Messen vor Behaupten (D-35):** Restbestände stammen aus echten Messungen (`scan_residue` über `ErasureLayer`, E1/E2/E3) mit Positivkontrolle (Canary).
10. **Fail-Closed:** Unbekannte Zustände oder Krypto-Fehler führen sofort zum geordneten Abbruch.
11. **Eine Typfamilie je Konzept:** Reine Domänentypen in `contextra-types`, Krypto in `contextra-crypto`.
12. **Gate-getrieben:** Verbindliche Gate-Reihenfolge (G-A bis G-H, G0) gemäß `PLAN.md`.

---

## 3. Lokale Verifikation

Vor jedem Commit oder PR ausführen:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::indexing_slicing
cargo check --workspace --all-targets
cargo test --workspace --locked
cargo test -p tests-arch
```
