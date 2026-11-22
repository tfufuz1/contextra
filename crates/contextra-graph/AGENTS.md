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

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: Indexstruktur; hängt an `contextra-store` nur als `[dev-dependencies]` (16 Zeilen, alle in `#[cfg(test)]`).
- Vertrag IF-6: Graph-Strukturen kommen später als `ExtensionBuilder` bzw. Segment an den Kern; jeder Builder implementiert `rewrite_without` (Pflicht für ER3).
- Verboten: der Kern kennt keine Graphstrukturen.
- Arbeitspakete: nach G-6.
