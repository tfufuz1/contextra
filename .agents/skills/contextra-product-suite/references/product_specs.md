# Contextra Product Family Specifications

Detaillierte Spezifikation der Produktprofile nach PLAN.md §11 und SPEC.md §4.

## 1. EP6 Guard (Privacy Firewall)
- **Problem**: Unternehmen dürfen keine personenbezogenen Daten (PII) ungeprüft an externe LLM-APIs senden.
- **Lösung**: Lokale Vorverarbeitung.
- **Komponenten**:
  - PII-Mustererkennung mit Prüfziffern-Validierung (IBAN Mod-97, Luhn für Kreditkarten, deutsche Steuer-ID).
  - Reversible Pseudonymisierung / Tokenisierung vor API-Egress.
  - Fail-closed Egress-Policy: Standardmäßig Blockade unautorisierter Netzwerkzugriffe.

## 2. EP1 Memory (Embedded RAG Engine)
- **Problem**: Externe Vektordatenbanken sind komplex, unterstützen oft keine deutsche Sprachmorphologie und bieten keine belegbare Löschung.
- **Lösung**: Lokale, eingebettete Single-Node-Bibliothek mit HNSW, BM25 (inkl. Kompositazerlegung), RRF und 6-Stufen-Löschung.
- **Grenzen**: Single-Node bis zu 10^7 Vektoren je Knoten. Kein verteiltes Sharding.

## 3. EP2 Proof (Compliance & DSGVO Audit)
- **Problem**: DSGVO Art. 17 verlangt den Nachweis erfolgter Löschungen gegenüber Aufsichtsbehörden.
- **Lösung**: Kryptografisch signierte Ed25519-Löschberichte (Proof V5) ohne Klartext-IDs.
