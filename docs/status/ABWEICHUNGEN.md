# Statusbericht Abweichungen (Nicht-normativ)

Dieses Dokument erfasst verbleibende technische oder architektonische Abweichungen zwischen dem aktuellen Quellcode-Implementierungsstand und der normativen Gesamtspezifikation (`CONTEXTRA_SPEZIFIKATION.md`, Fassung 1.1) gemäß §28.5. Diese Datei ist nicht normativ.

---

| Datei | Zeile(n) | Spezifikationsabschnitt | Beschreibung & Begründung |
|---|---|---|---|
| `crates/contextra-core/src/lib.rs` | 1-50 | §6.0, §6.10 | `contextra-core` ist laut Soll-Architektur vollständig aufzulösen. Im Code verbleibt es vorübergehend als Kompatibilitäts-Fassade (Re-Export über Ring-0-Crates), um schrittweise Strangler-Migrationen ohne Breaking Changes in internen Tests zu ermöglichen. |
| `crates/contextra-db/Cargo.toml` | 15-40 | §6.0, §6.10 | `contextra-db` importiert in der Übergangsphase verbliebene Hilfstypen aus `contextra-adapt`. Der Quellcode der Berechnungen nutzt nun konsolidierte Regler (`PoolController`), die Vollentkopplung aus dem Manifest folgt im nächsten Refactoring-Zyklus. |
| `crates/contextra-privacy/src/egress.rs` | 120 | §19.1, REQ-PRV-001 | Homoglyph- und Base64-Obfuskations-Erkennung im Egress-Vault sind als Heuristiken implementiert. Eine vollständige semantische Entschlüsselung erfordert zukünftige Wasmtime-Plugin-Erweiterungen. |

---

*Stand: 2026-10-25 — Erstellt im Rahmen der Dokumentationsausrichtung an Spezifikation v1.1.*
