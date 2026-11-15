# Verbleibende Altlasten (vor/bei der Ueberfuehrung abzuarbeiten)

Das Archiv ist von Alt-Kennungen bereinigt (Projektname, Magic-Bytes, Spezifikationsverweise, Schluesselmaterial,
Golden-Binaerdateien). Folgende Punkte lassen sich ohne Compiler nicht gefahrlos automatisch entfernen und sind
daher markiert; sie werden beim Ueberfuehren des jeweiligen Crates gestrichen oder neu gebaut.

| Punkt | Marker / Fundstelle | Aktion |
|---|---|---|
| Legacy-WAL-Schluesselpfad (`allow_legacy_integrity_key_fallback`, `legacy_integrity_key`, `rekey_from_legacy`, `migrate_legacy_wal`) | `TODO(acta-altlast)` in 8 Dateien unter `acta-seg/store/{src/wal,tests}` | Ersatzlos streichen. Der statische Legacy-Schluessel samt Obfuskation wurde bereits entfernt (Z10: keine Standardschluessel). Journal-Schluessel nur als Parameter (5.1) |
| Selbst angelegte Schluesseldatei (`.wal_integrity_key`, `load_or_create_integrity_key`) | `TODO(acta-altlast)` in `wal/hmac.rs` | Streichen; `acta-seg` ist schluesselfrei (HR-9), kein Erzeugen von Schluesseln im Speicher |
| Alt-Magics (`MFxx`) | umbenannt in `AKDF`, `AMNF`, `ASGX`, `ASGT`, `AWL2`-`AWL4` | Platzhalter. Formate nach 5.1 neu festlegen (`ACTASEG\0`, Journal-Rahmen), Konstanten ersetzen |
| Alt-Testkennungen in Stringliteralen und Attributen | 28 Dateien mit Resttreffern (`INV-...`, `ADR-...`, `WP-...`) | Beim Crate-Umbau durch `T-*`-Kennungen aus Kap. 16.3 ersetzen |
| Sammel-Crate-Importe | `TODO(acta-hr3)` in 118 Dateien | Typ dem Crate zuordnen, das ihn braucht (HR-2/HR-3) |
| Altformat-Fallbacks (`legacy`-JSON, `bincode`-Fallback in Posting-Listen, Manifest-Pruefsumme `legacy_expected`) | `acta-history/checkpoint/src/{manifest,meta}.rs`, `acta-text/text/src/posting_list.rs` | Streichen: Acta hat keine Altformate (14.8, Z10) |
| Golden-Dateien | Binaer-Goldens und Schluesseldateien entfernt (`*.bin`, `.wal_integrity_key`, HNSW-/Ranking-JSON) | Nach dem Formatentwurf neu erzeugen, nicht uebernehmen |
| Tests gegen nicht mitgelieferte Alt-Referenz (`legacy_reference`) | `proof_format_constants.rs`, `differential_ids.rs` entfernt | Durch Golden-Tests nach 5.1 ersetzen |

## Ergaenzungen dieses Durchgangs

Orakel-Material, das bislang im Archiv fehlte: Absturz-, Loom-, Eigenschafts- und Spezifikationstests des Loeschkerns
(verteilt auf Gate, Ledger, Tresor, Berichte, Pruefer), Purge-/Tombstone-/Nebenlaeufigkeitstests des Textindex,
Fuzz-Targets fuer Parser, Segmentformat und Loeschnachweis, Fusion/Zeitachse/Kalibrierung im Ranking, Checkpoint-Quellen
und -Tests, Port-Vorlagen (`Vfs`, Text-/Vektorindex, Richtlinie, Beobachter), Tombstone-/Filter-/Budget-Typen,
Zeitreise-Filter, Lizenzbindungs-Tests, Modellinfo des Inferenz-Clients. Neu angelegt: `acta-access/` (Referenz fuer den ABAC-Neubau).
