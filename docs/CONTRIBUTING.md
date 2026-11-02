# Beitragen zu Contextra

<!-- doc-stamp: 2026-11-02 | Normativ: docs/SPEC.md §0.2, §2, §6.4, §15.4 -->

Vielen Dank für Ihr Interesse an der Entwicklung von Contextra. Dieses Dokument beschreibt die verbindlichen Richtlinien, Qualitätsstandards und Entwicklungsabläufe für alle Beitragenden und Implementierer.

---

## 1. Voraussetzungen & Normative Quellen

- **Rust-Toolchain:** Version nach SPEC §4.2 D-82 / [`rust-toolchain.toml`](../rust-toolchain.toml).
- **Normative Pflichtlektüre:**
  - [`docs/SPEC.md`](SPEC.md) (Einzige normative Quelle für den Soll-Zustand Release 1.0.0). Alle Byte-Formate stehen in Anhang F, der Umsetzungsplan in §15.4.
  - [`AGENTS.md`](../AGENTS.md) (12 unverhandelbare Kernregeln).
  - [`docs/CONSTITUTION.md`](CONSTITUTION.md) (Verfassungsprinzipien).

---

## 2. Entwicklungsablauf (SPEC §15.4)

1. **Ein PR = ein Arbeitspaket (WP) = max. ein Crate-Schnitt (REQ-CHG-2):**
   - Jede Code-Änderung muss exakt einem Arbeitspaket aus `docs/SPEC.md` §15.4 zugeordnet sein.
   - PRs dürfen nicht mehrere Schichten oder Crates unkoordiniert vermischen.
2. **Rot vor Grün (TDD):**
   - Jeder WP beginnt zwingend mit einem fehlschlagenden Test (Red Phase), der den Befund oder das Fehlen der Funktionalität beweist.
3. **Keine neuen Warnungen oder Sicherheitslücken (HR-12):**
   - Keine neuen `unsafe`-Blöcke außerhalb von `sys` und `simd`.
   - Keine neuen Abhängigkeiten ohne Begründung und Test.

---

## 3. Wortlaut-Lint und unzulässige Aussagen (SPEC §6.4, T-LINT-1)

Alle Quelltexte, Kommentare, Dokumentationen, Commit-Nachrichten und PR-Beschreibungen unterliegen dem strikten Wortlaut-Lint **T-LINT-1**. Die genaue Liste verbotener Begriffe und Begründungen ist verbindlich in [`docs/SPEC.md §6.4`](SPEC.md) festgelegt.

Unzulässig sind insbesondere werbliche Garantieversprechen, absolute Sicherheitsbehauptungen oder rechtliche Konformitätsaussagen für die Software.

**Zulässige Außenaussage (SPEC §6.4):**
> *„Contextra entfernt die Daten eines Löschziels aus dem Datenverzeichnis, aus den Indexen und aus den gespeicherten Zeitreise-Versionen und weist den Zustand je Schicht messtechnisch nach. Für jedes Ziel wird zusätzlich der zugehörige Schlüssel aus dem Schlüsseltresor entfernt; ob und wie das rechtlich zu werten ist, entscheidet der Betreiber. Nicht erfasst sind externe Sicherungen, Flash-Remanenz, Dateisystem-Journale, Prozessspeicher und externe Verarbeiter. Der Bericht listet diese Restrisiken.“*
