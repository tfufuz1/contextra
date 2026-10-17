# ADR-N26: Auflösung der Unsafe-Policy-Inkonsistenz (ADR-004 vs. ADR-N03)

* **Status:** Accepted
* **Datum:** 2026-09-30
* **Kontext / Auslöser:**
  Zwischen ADR-004 (Sovereign Core / Pure Rust Policy) und ADR-N03 (Drei erlaubte Unsafe-Inseln) bestand eine formale Inkonsistenz:
  ADR-004 forderte ursprünglich ein pauschales `#![forbid(unsafe_code)]` für die Layer 0–2 mit einer vagen, unpräzisen Ausnahme für "SIMD in `contextra-index`".
  ADR-N03 hingegen definierte präzise die drei explizit genehmigten, isolierten Unsafe-Inseln (`contextra-simd`, `contextra-sys`, `contextra-wire`) und erzwingt für alle anderen 30+ Crates im Workspace `#![forbid(unsafe_code)]`.
  Zur Herstellung absoluter Governance-Klarheit und Auditsicherheit muss diese Widersprüchlichkeit aufgelöst werden.

## Entscheidung

1. **Geltung von ADR-N03 als maßgebliche Unsafe-Policy:**
   ADR-N03 ersetzt und präzisiert die Unsafe-Regeln aus ADR-004 vollständig [S].

2. **Drei exklusive Unsafe-Inseln (`capabilities.toml`):**
   `unsafe`-Code ist im gesamten Repository ausnahmslos auf die folgenden drei Kapseln beschränkt [V]:
   * **`contextra-simd`:** Hardware-Intrinsics (AVX2, AVX-512, NEON) mit Laufzeit-Feature-Detection.
   * **`contextra-sys`:** Betriebssystem-Schnittstellen (Mmap, Dateisystem-ACLs, POSIX/Windows-Fsync).
   * **`contextra-wire`:** Autogenerierte FlatBuffers IPC-Bindings mit Verifier-Bounds-Checking.

3. **Striktes `#![forbid(unsafe_code)]` für den gesamten Rest:**
   Alle weiteren Crates im Workspace (inklusive `contextra-crypto`, `contextra-vector`, `contextra-store`, `contextra-db`, `contextra-mcp`, `contextra-engine`) erzwingen ausnahmslos `#![forbid(unsafe_code)]` am Crate-Root [V].
   Sämtliche `unsafe`-Blöcke in den Inseln erfordern zwingend einen unikat beschreibenden `// SAFETY:`-Proof-Kommentar sowie die Lints `unsafe_code = "deny"` und `undocumented_unsafe_blocks = "deny"` [V].

## Alternativen

### Option A: Pauschales Unsafe-Verbot ohne SIMD/SYS-Ausnahmen (Strikte Reinform von ADR-004)
* **Vorteile:** 0% Unsafe-Code im gesamten Quelltext.
* **Nachteile:** Unmöglichkeit von Zero-Copy-Mmap-Dateizugriffen und manuell optimierten SIMD-Vektor-Berechnungen (AVX-512/NEON); erhebliche Einbußen bei der Vektor-Suchperformance.
* **Bewertung:** Verworfen [S].

### Option B: Unkontrollierte dezentrale `unsafe`-Nutzung
* **Vorteile:** Entwickler können an jeder Stelle lokale Unsafe-Optimierungen einfügen.
* **Nachteile:** Zerstörung der Auditsicherheit; massives Risiko von Memory-Safety-Bugs (UB).
* **Bewertung:** Verworfen [S].

## Konsequenzen

* **Eindeutige Governance:** Klarheit über zulässige Unsafe-Standorte im gesamten Repository [V].
* **Auditsicherheit:** Audits müssen ausschließlich die drei Inseln (`contextra-simd`, `contextra-sys`, `contextra-wire`) überprüfen [V].
* **Maschinelles Gate:** Linter und Compiler verhindern unautorisierte `unsafe`-Nutzung außerhalb der Inseln sofort [V].

## Reversibilitäts-Bedingung

Eine Modifikation der Unsafe-Inseln erfordert eine formelle Erweiterung von ADR-N03 sowie ein Update der zentralen Governance-Tabelle in `capabilities.toml` [S].
