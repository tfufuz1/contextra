# AGENTS.md — `contextra-sys`

## 1. Rolle
`contextra-sys` bietet systemnahes Memory-Mapping (mmap), Prozess-Härtung und plattformspezifische I/O-Primitiven über eine sichere Fassade.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Unsafe-Insel 2 von 3)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types` (Spec §6.9, `capabilities.toml`)

## 3. Garantien & Anforderungen (§33, §6.7)
- **REQ-SYS-001:** Safe API Wrapper um OS-level Mmap und Prozess-Härtung (`harden_process`).
- **REQ-SYS-002:** Jeder `unsafe`-Block trägt ein explizites `// SAFETY:` Kommentar.
- **Durchsetzungsmechanismus (§27.6):** Plattform-Checks, Miri / Non-Linux Fail-Closed Fallback (`io::ErrorKind::Unsupported`).

## 4. Verbotene Muster
- Exponieren roher OS-Handles oder unsicherer Pointer in der öffentlichen Schnittstelle.
- `unsafe`-Blöcke ohne `// SAFETY:` Begründung.

## 5. Implementierte & konsumierte Ports (Anhang A)
- Stellt Mmap- und System-Primitiven für `contextra-store` und `contextra-vector` bereit.

## 6. Tests & Gates
- **Miri Gate:** Miri-Verifikation für Memory-Mapping Pointers.
- **Cross-Platform Tests:** Fail-Closed Verhalten auf Nicht-Linux Plattformen.

## 7. Offene Punkte
- keine.

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: kein Codebezug zu `contextra-store`; Treffer nur in Kommentaren bzw. in `Cargo.toml` (Stand 22.11.2026). Keine Änderung bis G-6.
