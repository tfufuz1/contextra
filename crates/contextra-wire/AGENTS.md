# AGENTS.md — `contextra-wire`

## 1. Rolle
`contextra-wire` verwaltet performante Zero-Copy FlatBuffers Wire-Protokolle und Deserialisierungs-Primitiven.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Unsafe-Insel 3 von 3)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types` (Spec §6.9, `capabilities.toml`)

## 3. Garantien & Anforderungen (§33, §6.7)
- **REQ-WIR-001:** FlatBuffers Schema-Generierung mit `file_identifier "CTRA";`.
- **REQ-WIR-002:** Safe Buffer Verification (`*_buffer_has_identifier`) vor Deserialisierung.
- **Durchsetzungsmechanismus (§27.6):** Post-Processing der FlatBuffers Generierung, Miri Fuzz-Tests.

## 4. Verbotene Muster
- `unsafe`-Blöcke in handgeschriebenem Code ohne `// SAFETY:` Begründung.
- Deserialisierung ohne Buffer-Bound & Identifier Check.

## 5. Implementierte & konsumierte Ports (Anhang A)
- Stellt `WireWiring` und Wire-Protokoll-Typen bereit.

## 6. Tests & Gates
- **Fuzzing (REQ-QA-003):** FlatBuffers IPC Deserialisierungs-Fuzzer.
- **Miri Gate:** Miri-Verifikation für FlatBuffers Reader.

## 7. Offene Punkte
- keine.

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: kein direkter Bezug zum Storage-Umbau. Es nutzt `contextra-store` nicht im Code (Stand 22.11.2026) und ändert sich bis G-6 nicht. Bei Fragen zum Kern gilt `docs/SPEC.md` (R8).
