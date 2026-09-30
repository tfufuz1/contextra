# ADR-N03: Drei erlaubte Unsafe-Inseln, Deny/Forbid-Mechanik und Unsafe-Transition-Tracking

* **Status:** Final
* **Datum:** 2026-09-22
* **Kontext / Auslöser:**
  Contextra folgt dem Grundsatz der maximalen Speichersicherheit (Pure Rust Policy / Sovereign Core, ADR-004). Jedoch verlangen plattform- und hardwarenahe Optimierungen (wie Zero-Copy Mmap-I/O oder SIMD-Vector-Math) nach der Nutzung von `unsafe` Rust-Blöcken.
  Wildwuchs von `unsafe`-Code über verschiedene Domänen- und Datenbank-Crates hinweg würde die Auditierbarkeit zerstören und Memory-Safety-Bugs wie Undefined Behavior, Out-of-Bounds-Reads oder Use-After-Free riskieren.

## Entscheidungen

1. **Standardmäßiges `#![forbid(unsafe_code)]` im Workspace:**
   Das Wurzel-`Cargo.toml` erzwingt global `[workspace.lints.rust] unsafe_code = "deny"` und `undocumented_unsafe_blocks = "deny"`. Sämtliche Fach-, Speicher-, Engine- und Protokoll-Crates durchsetzen am Crate-Root `#![forbid(unsafe_code)]`.

2. **Drei explizit genehmigte Unsafe-Inseln:**
   `unsafe`-Code ist ausschließlich in drei isolierten Kapseln erlaubt:
   * **`contextra-sys`:** Systemnahe OS-Schnittstellen (wie FFI, Mmap-Dateizugriffe). Stellt sichere Fassaden (z. B. `contextra_sys::mmap_readonly`) bereit.
   * **`contextra-simd`:** Vektor-Distanzberechnungen und Hardware-Intrinsics (AVX2, AVX-512, NEON) mit strenger Runtime-Feature-Detection (`is_x86_feature_detected!`).
   * **Übergangs-Whitelists (`contextra-vector`, `contextra-store` / `contextra-crypto` Test-Only):** Temporäre `unsafe`-Nutzung auf dem Migrationspfad MUSS zwingend in einer lokalen `UNSAFE_TRANSITION.md`-Datei nachverfolgt werden.

3. **Verpflichtendes `UNSAFE_TRANSITION.md`-Tracking:**
   Tritt aus historischen Gründen `unsafe`-Code in Whitelist-Crates auf, MUSS jede Fundstelle mit einer Tracking-ID (z. B. `TRANS-VEC-001`), Quellpfad, Begründung, Ziel-Crate und Migrationsstatus in `UNSAFE_TRANSITION.md` dokumentiert sein.
   Nach erfolgreicher Migration in `contextra-sys` oder `contextra-simd` wird das Crate unverzüglich auf `#![forbid(unsafe_code)]` zurückgestellt.

4. **Verpflichtende `// SAFETY:`-Dokumentationsregel:**
   Jeder verbleibende `unsafe`-Block erfordert ausnahmslos einen vorangestellten `// SAFETY:`-Kommentar, der die mathematischen oder speicherbezogenen Preconditions und Invarianten belegt. Word-identische Copy-Paste-Kommentare sind unzulässig (ADR-035).

## Konsequenzen

* **Maximale Auditsicherheit:** Sicherheitsaudits müssen nur die isolierten Inseln (`contextra-sys`, `contextra-simd`) und aktive `UNSAFE_TRANSITION.md`-Whitelists prüfen.
* **Keine Korruption im Fachcode:** 95%+ des Gesamtrepositories (inklusive `contextra-db`, `contextra-agent`, `contextra-mcp`, `contextra-py`) bleiben garantiert frei von Unsafe-Code.
* **Maschinelles Enforcement:** Compiler und Linter schlagen bei unautorisierten `unsafe`-Blöcken in geschützten Crates sofort mit E0133/Linter-Error fehl.
