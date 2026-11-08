# SPEC: contextra-types (Klasse B)

Normatives Spezifikationsdokument für den Klassen-B-Baustein `contextra-types` gemäß SPEC v4.0.

## 1. Zweck und Rolle
* **Rolle:** Fundament des Storage-Bundles: kanonische Identifikatoren, Kern-Datenstrukturen, Fehlerdomänen und Request-Kontext.
* **Klasse:** Klasse B (Storage-Bundle-Kerne).
* **Grenzen:**
  * Keine I/O- oder Netzwerkanbindungen.
  * Keine nicht-deterministischen Zeit- oder Entropie-Quellen in Kern-Typen.
  * Neutralität gegenüber konkreten Speichermodellen (LSM, HNSW, BM25).
  * Vollständige Entkopplung von der Löschdomäne (`contextra-erasure`).

## 2. Kanonische Identifikatoren und Invarianten
* `DocId`: Kanonische Dokumenten-ID (64-Bit Standard oder 128-Bit mit Feature `docid-128`).
* `TxId`: Monoton steigende Transaktions-ID mit geschützten System- und Benutzerbereichen.
* `TenantId`: Mandanten-Identifikator mit reserviertem System-Bereich (`SYSTEM_TENANT_ID`).
* `CollectionId`: Typsichere Sammlungs-Kennung.

## 3. Fehlerdomänen
* `ContextraError`: Zentraler typisierter Fehler-Enum für alle Kernoperationen des Storage-Bundles.
* Sub-Fehler: Modul-disjunkte Fehlerstrukturen mit Zero-Panic-Konstruktoren und deterministischer Konvertierung.

## 4. Garantien
* **Zero-Panic:** Keine Panics, ungeprüften Indexierungen oder Konstruktor-Abbrüche in Produktionscode.
* **Determinismus:** Bit-identische Repräsentationen und deterministische Sortier- und Hash-Ordnungen.
* **Kompatibilität:** Additive Features für `docid-128` und `auto-extraction-opt-out`.
