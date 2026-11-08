# SPEC: `contextra-privacy` — Bausteinspezifikation (Klasse A)

> **Status:** Normativ (SPEC v4.0, Welle 1) · **Klasse:** A · **Ring:** 3 · **Unsafe:** `#![forbid(unsafe_code)]`
> **Zweck:** Autarke PII-Erkennung, Cloud-Egress-Schutz (DLP), Surrogat-Token-Vaulting und Exfiltrationsschutz vor jedem LLM- oder Cloud-Gateway.

---

## 1. Baustein-Architektur & B1–B12 Garantien

1. **B1 / B2 (Keine internen Abhängigkeiten):** `contextra-privacy` hängt in der Basis-Konfiguration von keinen internen Workspace-Crates ab. Alle internen Normal- und Dev-Dependencies (`contextra-types`, `contextra-ports`, `contextra-crypto`, `contextra-testkit`) sind entfernt.
2. **B3 (Standard- & Crate-eigene Typen):** Alle öffentlichen APIs operieren ausschließlich auf Standardtypen (`String`, `&str`, `Vec<T>`, `Range<usize>`, `Duration`) und crate-eigenen Typen (`TenantId`, `TenantScoped`, `EgressPolicy`, `EgressDecision`, `Clock`, `SystemClock`).
3. **B4 (Fehlermodell):** Einheitlicher Hauptfehlertyp `EgressError` (`thiserror`, `#[non_exhaustive]`) sowie modulare Sub-Errors `EgressVaultError` und `ContextEditAuditError`. Keine unstrukturierten String-Fehler.
4. **B5 (Autarker Standalone-Build):** Kompiliert und testet isoliert außerhalb des Repos (`export_crate.py`, `extract_probe.py`).
5. **B6 (Feature-Minimalismus):** Standard-Features sind minimal. Das optionale Feature `ports-compat` ermöglicht bis Welle 6 verlustfreie Typkonvertierungen zu `contextra-ports` und `contextra-types`.
6. **B7 (Keine Fremdfachlogik / Lösch-Entkopplung):** Löschketten-, Saga-, Shredding- und DSGVO-Art.-17-Nachweislogik verbleiben exklusiv in `contextra-erasure`. Maskierung ist keine Löschlogik; PII-Vault-Entscheidungen sind neutral als Speicherisolierung modelliert (`pii_vault_requires_isolated_retention`); `SurrogateVault` bietet neutrale Wartungsmethoden `clear()` und `compact()`.
7. **B8 (Dokumentation & Messung):** `README.md`, `SPEC.md`, Moduldokumentation (`//!`), `# Errors`, Criterion-Benchmarks (`benches/bench_privacy.rs`).
8. **B9 (Zero-Panic):** Kein `unwrap()`, `expect()`, `panic!()` oder unkontrolliertes Indexing in Produktionspfaden. Mutex-Poisoning führt zu sicherem Fallback oder typisiertem Fehler.
9. **B10 (Sicherheit):** `#![forbid(unsafe_code)]` crate-weit forciert.
10. **B11 (Determinismus & P28):** Zeitabfragen erfolgen über den injizierten `Clock`-Port. Interne Maps (`scc::HashMap`) verwenden deterministische BLAKE3-BuildHasher (`DeterministicBuildHasher`).
11. **B12 (Lizenz & Toolchain):** Kompatibel mit Rust 2021 Edition und `rust-version = "1.89"`.

---

## 2. Invarianten

- **INV-EGRESS-BUDGET-1:** Fail-closed bei Überschreitung der maximalen Payload-Größe (`MAX_CLASSIFY_PAYLOAD_BYTES = 64 KiB`) oder Zeitlimits. Jede externe Wirkung ist budgetiert und auditiert.
- **INV-LOG-1 / HR-4:** Keine PII oder Klartexte in Log-Nachrichten, Audit-Traces oder Fehlermeldungen.
- **INV-ZERO-PANIC:** Produktionscode enthält null Panics auf beliebigen UTF-8-Grenzen (gesichert durch byte-offset-genaue Rückabbildung im `Normalizer`).
- **INV-EVASION-RESIST:** `RuleBasedEntityRecognizer` ist gehärtet gegen Obfuskation durch Zero-Width-Zeichen, Zalgo-Combining-Marks, Bidi-Controls, Fullwidth- und Math-Bold-Homoglyphen.

---

## 3. Konstanten & Grenzwerte

| Konstante | Wert | Bedeutung |
|---|---:|---|
| `MAX_CLASSIFY_PAYLOAD_BYTES` | 65.536 (64 KiB) | Maximale Payload-Länge für Egress-Klassifikation und Entity-Erkennung |
| `MAX_SEARCH_QUERY_BYTES` | 65.536 (64 KiB) | Maximale Abfragelänge im `EgressGateway` |
| `DEFAULT_EGRESS_GUARD_TIMEOUT` | 200 ms | Timeout für semantische Vektorprüfungen im `EgressGuard` |
| `DEFAULT_EGRESS_GUARD_THRESHOLD` | 0.85 | Cosine-Similarity-Schwellwert für Ähnlichkeitsblockierung |
| `DEFAULT_EGRESS_GUARD_MIN_BYTES` | 128 Bytes | Mindestlänge für semantische Ähnlichkeitsprüfung |

---

## 4. Concurrency- und Durability-Garantien

- `SurrogateVault`: Sitzungsgebundener, thread-sicherer Speicher mit `Mutex<BTreeMap<String, String>>`. Zeroize-on-Drop für Sitzungssalz und Klartext-Mappings.
- `BulkExfiltrationDetector`: Lock-arme, hochparallele Sitzungsverwaltung via `scc::HashMap` mit `DeterministicBuildHasher`. Feingranulare `parking_lot::Mutex`-Sperren pro Sitzungszähler. Keine Sperren über `.await`.
- `EgressGateway`: Vollständig thread-sicher (`Send + Sync`) für nebenläufige Anfragen über Tokio-Worker.
