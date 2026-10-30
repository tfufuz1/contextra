# AGENTS.md — `contextra-types`

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. Restmodul `types/saos.rs` stammt aus dem entfernten Agenten-Kontext (SPEC §16, B7). Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

## 1. Rolle
`contextra-types` definiert die zentralen, produkt- und I/O-unabhängigen Domänentypen, Identifikatoren und Fehler-Enums des Gesamtsystems.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Typen-Kern, I/O-frei, deterministisch)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `(keine)` (Spec §6.9, `capabilities.toml`)

## 3. Garantien & Anforderungen (§33)
- **REQ-TYP-001:** Lossless DTO-Konvertierung für `ContextraError` (`ContextraErrorDto`).
- **REQ-TYP-002:** Breite-agnostische `DocId`-Instanziierung (`DocId::new`).
- **Durchsetzungsmechanismus (§27.6):** Statische Typenprüfung, `#![forbid(unsafe_code)]`, Unit-Tests.

## 4. Verbotene Muster
- Kein `unsafe` (`#![forbid(unsafe_code)]`).
- Keine Asynchronität oder I/O (keine `tokio`- oder FS-Abhängigkeiten).
- Keine Abhängigkeiten zu anderen Contextra-Crates.

## 5. Implementierte & konsumierte Ports (Anhang A)
- Konsumiert/Implementiert keine Ports direkt; Stellt reine Datenstrukturen (`DocId`, `TxId`, `TenantId`, `ContextraError`) bereit.

## 6. Tests & Gates
- **Fuzzing (REQ-QA-003):** Serde/Deserialisierungs-Fuzzing für DTOs.
- **Mutation-Score (REQ-QA-005):** Ziel-Mutation-Score >= 85 %.

## 7. Offene Punkte
- keine (P-01 bis P-14 bleiben juristische Prüfpunkte gemäß §20.13).
