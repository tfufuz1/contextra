# AGENTS.md — `contextra-graph`

## 1. Rolle
`contextra-graph` stellt strukturelle Graph-Traversierung, CSR-Repräsentation, PPR und Community Detection für GraphRAG bereit.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Graph-Engine)
- **Klasse:** Rand `[R]`
- **Reifegrad:** `experimental` (Anmerkung: In `capabilities.toml` aktuell als `stable` geführt, siehe `docs/status/ABW-L3.md`)
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`)

## 3. Garantien & Anforderungen (§33, §11, AGT-GRAPH-001)
- **AGT-GRAPH-001:** Transaktions-IDs in Graph-Operationen müssen strikt von `TxId::try_new()` oder `TxId::INTERNAL_BASE` abgeleitet werden.
- **REQ-GRP-001:** Deterministische CSR Traversal & Bi-temporale Edges.
- **Durchsetzungsmechanismus (§27.6):** `#![forbid(unsafe_code)]`, Graph-Traversal Property Tests.

## 4. Verbotene Muster
- **Keine Hintergrund-Threads ohne Port** (P-Threads Isolation).
- Kein `unsafe` (`#![forbid(unsafe_code)]`).
- Keine Verwendung veralteter `TxId::new()` Konstruktoren.

## 5. Implementierte & konsumierte Ports (Anhang A)
- Implementiert: `GraphIndex` Port.
- Konsumiert: `Clock` Port.

## 6. Tests & Gates
- **Property- & Graph-Tests:** Traversal & PathRAG Verification.

## 7. Offene Punkte
- keine.
