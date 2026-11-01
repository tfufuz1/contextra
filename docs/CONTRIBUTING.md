# Beitragen zu Contextra

<!-- doc-stamp: 2026-11-01 | Normativ: docs/SPEC.md §0.2, §2, PLAN.md §1 -->

Vielen Dank für Ihr Interesse an der Entwicklung von Contextra. Dieses Dokument beschreibt die verbindlichen Richtlinien, Qualitätsstandards und Entwicklungsabläufe für alle Beitragenden und Implementierer.

---

## 1. Voraussetzungen & Normative Quellen

- **Rust-Toolchain:** Version 1.89+ laut [`rust-toolchain.toml`](../rust-toolchain.toml) (Edition 2021).
- **Normative Pflichtlektüre:**
  - [`docs/SPEC.md`](SPEC.md) (Einzige normative Quelle für den Soll-Zustand v2.0.0).
  - [`PLAN.md`](../PLAN.md) (Verbindliche Phasen 0–7, Gates G-A bis G-H, Arbeitspakete WP-0.1 bis WP-7.4).
  - [`AGENTS.md`](../AGENTS.md) (12 unverhandelbare Kernregeln).

---

## 2. Entwicklungsablauf (PLAN §1)

1. **Ein PR = ein Arbeitspaket (WP) = max. ein Crate-Schnitt (REQ-CHG-2):**
   - Jede Code-Änderung muss exakt einem Arbeitspaket aus `PLAN.md` zugeordnet sein.
   - PRs dürfen nicht mehrere Schichten oder Crates unkoordiniert vermischen.
2. **Rot vor Grün (TDD):**
   - Jeder WP beginnt zwingend mit einem fehlschlagenden Test (Red Phase), der den Befund oder das Fehlen der Funktionalität beweist.
   - Ohne roten Test wird kein Pull Request gemergt.
3. **Evidenzpflicht:**
   - Nach Fertigstellung eines WP werden Logs, Befehle und Testausgaben unter `evidence/<WP-ID>/` abgelegt (Befehl, Commit-ID, Ergebnis).
4. **Lokale Verifikation:**
   Vor jedem PR müssen alle blockierenden CI-Prüfungen lokal grün sein:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::indexing_slicing
   cargo check --workspace --all-targets
   cargo test --workspace --locked
   cargo test -p tests-arch
   ```

---

## 3. Die 12 unverhandelbaren Kernregeln (SPEC §0.2)

1. **Ring- & Schichten-Disziplin:** Abhängigkeiten zeigen nur abwärts (Ring 4 → 0). Innerhalb Ring 2 gilt strikt die Matrix SPEC §5.3.
2. **Zero-Panic in Produktion:** Kein `unwrap()`, `expect()`, `panic!()`, `todo!()`, `assert!()` oder ungeprüftes Index-Slicing im Produktionscode.
3. **Determinismus & Port-Purity:** Zeit, Zufall und IDs stammen ausschließlich aus `Clock`, `Rng` und `IdGen` aus `contextra-ports`.
4. **WAL-First Consistency:** Frame(s) → Gruppen-Commit → `sync_data` WAL → Memtable → Sichtbarkeit.
5. **I/O ausschließlich über `Vfs`:** Kein direkter Aufruf von `std::fs` oder `tokio::fs`. Alle Sync-Fehler propagieren.
6. **Unsafe nur in 2 Inseln:** `unsafe` ist ausschließlich in `contextra-sys` und `contextra-simd` erlaubt, mit lückenloser `// SAFETY:`-Dokumentation. Alle anderen Crates erzwingen `#![forbid(unsafe_code)]`.
7. **Mandantentrennung verpflichtend:** `TenantId` stammt aus dem Kontext. Zugriff auf den Store nur über `TenantScopedStorage`.
8. **Sicherheit ist kein Feature-Schalter:** AES-256-GCM-SIV und Argon2id ($m \ge 64\,\text{MiB}$) sind Pflicht ohne Opt-Out.
9. **Messen vor Behaupten:** Restbestände stammen aus echten Messungen (`scan_residue`) mit Positivkontrolle (Canary).
10. **Fail-Closed:** Unbekannte Zustände oder Krypto-Fehler führen zum geordneten Abbruch, nie zu stillschweigendem Weiterlaufen.
11. **Eine Typfamilie je Konzept:** Domänentypen in `contextra-types`, Kryptografie in `contextra-crypto`.
12. **Gate-getrieben:** Kein Phasenwechsel ohne bestandenes Release-Gate.
