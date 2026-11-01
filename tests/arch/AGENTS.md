# AGENTS.md — `tests/arch`

> **Normative Vorgabe gemäß [docs/SPEC.md §8.18](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md)**

## 1. Rolle & Zweck
`tests/arch` ist die zentrale Validierungs- und Gate-Suite zur automatisierten Überwachung aller architektonischen Invarianten, Ring-Abhängigkeiten, Zero-Panic-Lints, Unsafe-Inseln und Supply-Chain-Vorgaben in Contextra.

## 2. Ring & Einordnung
- **Ring:** dev / Architektur-Gates
- **Verbotene Muster:** Darf niemals von Produktionscrates referenziert werden.

## 3. Testumfang & Überwachte Invarianten (Soll-Zustand laut SPEC §8.18)

| Test-Suite | Prüfgegenstand & Invariante |
|---|---|
| **T-ARCH-1** | Ring-DAG: Ring 4 → Ring 3 → Ring 2 → Ring 1 → Ring 0. Keine verbotenen Rückwärtskanten (Matrix SPEC §5.3). Keine I/O- oder Async-Typen in `contextra-types`. |
| **T-ARCH-2** | Raw Store Isolation: Nur `TenantScopedStorage` ruft Methoden des rohen `LsmStorage`. |
| **T-ARCH-3** | Entkopplung Löschkern: `contextra-erasure` besitzt keinerlei Abhängigkeiten zu Ring-2-Crates. Unabhängige Prüfer (`lab`, `ctx-verify`) teilen keinen Produktionscode. |
| **T-ARCH-4** | Single-Node-Garantie: Keine Netzwerk-, Socket-, HTTP- oder P2P-Crates im Abhängigkeitsbaum der Produktionscrates (`cargo tree`). |
| **T-UNSAFE-1** | Unsafe-Inseln: `unsafe` ist ausschließlich in `contextra-sys` und `contextra-simd` erlaubt. Alle anderen Crates erzwingen `#![forbid(unsafe_code)]`. Jeder `unsafe`-Block besitzt ein gültiges `// SAFETY:`-Kommentar. |
| **T-LINT-1 / 2** | Zero-Panic-Audit: Scan auf verbotene Makros (`panic!`, `todo!`, `unimplemented!`, `assert!`) und Methoden (`unwrap()`, `expect()`, ungeprüftes Index-Slicing) in allen Produktionspfaden. |
| **T-CFG-1 / 2** | Feature-Hygiene: Blockieren undeklarierter `cfg(feature = "...")` Schalter im gesamten Workspace. |
| **T-SUPPLY-1** | Lieferkette: Prüfung von Lizenzen, Audit-Advisories (`cargo audit`) und Version-Pinning (SPEC §10.6). |

## 4. Crate-Arbeitspakete zum SOLL-Zustand (PLAN.md)

- **[ ] WP-0.2: Undeklarierte Features blockierend erkennen**
  - CI-Gate zur Erkennung toter oder undeklarierter Feature-Gates.
- **[ ] WP-0.4: Wortlaut-Lint etablieren**
  - Blockieren unzulässiger Werbe-Aussagen ("100 % DSGVO-konform", etc.) im Code und in der Dokumentation.
- **[ ] WP-6.4: Unsafe-Insel-Audit und Linter fertigstellen**
  - Vollständige statische Durchsetzung von T-UNSAFE-1.
- **[ ] WP-6.5: Lieferketten-Tests (T-SUPPLY-1)**
  - Automatisierte Verifikation aller direkten und transitiven Abhängigkeiten.

## 5. Verifikation & Ausführung
```bash
cargo test -p tests-arch --test arch_invariants
cargo test -p tests-arch --test unsafe_islands
```
