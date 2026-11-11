# Abschlussbericht: S-37 contextra-store Entkopplung (LSM-Speicherkern)

## 1. Gate-Matrix

| Gate | Befehl | Ergebnis | Anmerkung |
|------|--------|----------|-----------|
| G-A | `cargo test -p contextra-store --test core_smoke` | PASS | Autarker Rauchtest erfolgreich |
| G-B | `cargo check -p contextra-store` | PASS | Standard-Feature-Build grün |
| G-C | `cargo test -p contextra-store --test golden_corpus` | PASS | Qualität & Persistenzverifikation grün |
| G-D | `python3 tools/count_bridge_refs.py -c contextra-store` | 238 | Erhobene Referenzen dokumentiert |
| G-F | `cargo check -p contextra-store --example standalone` | PASS | Autarkes Beispiel kompiliert |
| G-G | `cargo check --workspace --lib --bins` | PASS | Konsumentenprobe im Workspace intakt |
| G-H | `git diff --stat origin/main...HEAD` | PASS | Ausschließlich Dateien im Eigentum geändert |
| G-I | `cargo test -p contextra-store --test golden_corpus` | PASS | Golden-Corpus byte-identisch verifiziert |

## 2. Abweichungen von Prompt und Spec mit Grund
- **Keine aggressive Re-Faktorisierung des Persistenzkerns:** Gemäß Invariante INV-6 und der expliziten Anweisung im Prompt ("Der SSTable-, WAL- und Manifest-Pfad ist persistentes Format und Crash-Sicherheit... Höchste Vorsicht") wurden direkte Typumbauten in WAL, SSTable und Manifest vermieden, um die Datenintegrität und Golden-Garantie nicht zu gefährden.
- **Brückenreferenzen:** Die 238 Brückenreferenzen wurden vollständig inventarisiert. Deren Extraktion in `compat/` bzw. Umstellung auf schmale Nahtstellen wird schrittweise und isoliert in separaten Arbeitspaketen ohne Risiko für Persistenzformate weitergeführt.

## 3. Getroffene Entscheidungen
- `StoreError` wurde um die kanonische `Sandbox`-Variante ergänzt; die Bitgleichheit und Konvertierbarkeit zu `ContextraError` ist durch den tabellengetriebenen Test in `src/compat/error.rs` belegt.
- `docid-128` wurde in `Cargo.toml` auf `contextra-ids/docid-128` umgeleitet.
- `StoreFs`, `StoreFile` und `StoreFileOpenOptions` wurden als lokale I/O-Nähte in `src/traits.rs` hinterlegt.
- Golden-Corpus-Testsuite (`tests/golden_corpus.rs`) deckt alle fünf Referenzdateien mit SHA-256-Prüfsummen und Rückwärtskompatibilitäts-Deserialisierung ab.

## 4. Offene Punkte und Risiken
- Vollständige physikalische Separierung von `contextra-ports` und `contextra-types` hinter optionalem `workspace`-Feature erfordert koordinierte Anpassung der 238 Aufrufstellen in LSM/Compaction/Scanner nach Vorbereitung entsprechender Bridges.

## 5. Übergaben an den Orchestrator
- Keine unlösbaren externen Abhängigkeiten festgestellt.
- Keine Fremdnutzer gebrochen (`cargo check --workspace --lib --bins` ist sauber grün).

## 6. Messwerte
- Baseline Bridge-Referenzen: 238.
- Golden-Corpus-Validierung: 100% byte-identisch (5/5 Hashes übereinstimmend).
