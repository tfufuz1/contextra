# SPEC: contextra-types (Klasse B)

> **Revision R12** (Repo-Abgleich, HEAD `60ea034`): Änderungen gemäß `SPEC.md` K-58…K-63 und Anhang B.5. Abschnitte mit Kennzeichnung „R12“ sind normativ.

Normatives Spezifikationsdokument für den Klassen-B-Baustein `contextra-types` gemäß SPEC v4.0.

## 1. Zweck und Rolle
* **Rolle:** Fundament des Storage-Bundles: kanonische Identifikatoren, Kern-Datenstrukturen, Fehlerdomänen und Request-Kontext.
* **Klasse:** Klasse B (Storage-Bundle-Kerne).
* **Grenzen:**
  * Keine I/O- oder Netzwerkanbindungen.
  * Keine nicht-deterministischen Zeit- oder Entropie-Quellen in Kern-Typen.
  * Neutralität gegenüber konkreten Speichermodellen (LSM, HNSW, BM25).
  * Vollständige Entkopplung von der Löschdomäne (`contextra-erasure`).
  * **R12 (Ebene L0):** `types` ist das unterste Fach der Werttypen. Keine Kante nach oben, keine Abhängigkeit auf `ports`, `storage`, `simd` oder andere Triebwerke (K-58).

## 2. Kanonische Identifikatoren und Invarianten
* `DocId`: Kanonische Dokumenten-ID (64-Bit Standard oder 128-Bit mit Feature `docid-128`).
* `Epoch(u64)` **(R12, K-59):** einziger Versionsbegriff des Systems. Monoton steigende, opake Sichtbarkeitsmarke. Konvertierung `Stamp ↔ Epoch` ausschließlich in `contextra-index-bridge`.
* `TxId` **(entfällt nach G-CUTOVER, K-59):** bisherige Kopie in `mvcc`, `vector`, `graph`, `checkpoint`. Bis dahin nur als `#[deprecated]`-Alias auf `Epoch`, keine neuen Verwendungen.
* `TenantId`: Mandanten-Identifikator mit reserviertem System-Bereich (`SYSTEM_TENANT_ID`).
* `CollectionId`: Typsichere Sammlungs-Kennung.

## 3. Fehlerdomänen
* `ContextraError`: Zentraler typisierter Fehler-Enum für alle Kernoperationen des Storage-Bundles.
* Sub-Fehler: Modul-disjunkte Fehlerstrukturen mit Zero-Panic-Konstruktoren und deterministischer Konvertierung.
* Modell-Neutralität: Speichermodell-spezifische Fehler wie `HnswDeletionError` sind als deprecated markiert und gehören in die jeweiligen Engines (`contextra-vector`).

## 4. Garantien
* **Zero-Panic:** Keine Panics, ungeprüften Indexierungen oder Konstruktor-Abbrüche in Produktionscode.
* **Determinismus:** Bit-identische Repräsentationen und deterministische Sortier- und Hash-Ordnungen.
* **Kompatibilität:** Additive Features für `docid-128` und `auto-extraction-opt-out`.

## 5. R12-Ergänzungen

- **INV-EPOCH-1:** Es existiert genau eine Definition von `Epoch` in `contextra-types`. `text::contract::Epoch` wird Re-Export. Kopien von `TxId` in anderen Crates sind nach G-CUTOVER verboten (Gate: Grep-Test auf `struct TxId`).
- **INV-TYPES-LAYER-1:** `contextra-types` darf keine Kanten zu Crates der Ebenen 1 bis 4 besitzen (`capabilities.toml`-Prüfung).
