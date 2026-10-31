# Contextra — Jules Agent Context

> **Status 30.10.2026 (Reduktion):** Diese Regel gilt weiter, soweit sie nicht auf entfernte Crates oder entfernte Routinen verweist. Solche Stellen sind veraltet. Maßgeblich: `AGENTS.md` und `docs/SPEC.md` (Fassung 4.0).
> Version: 3.1 | Stand: 2026-10-26 | Permanent Ambient Context für Jules Sessions (Spezifikation v9)
>
> ⚠️ **FRISCHEGARANTIE**: Diese Datei regelt ausschließlich die Session-Prozessführung für Jules.
> Die tatsächlichen Code-Fakten, Crate-Strukturen, Invarianten und Implementierungsstände
> sind gemäß MECE-Prinzip (CONSTITUTION.md §Documentation Model) in folgenden Quellen verankert:
> - **Code-Zustand & Non-Obvious Decisions**: siehe `AGENTS.md`
> - **Dynamischer Projektstatus & Tag-Inventar**: siehe `WORKING_STATE.md`
> - **Architektur-Entscheidungen (ADRs)**: siehe `docs/decisions/README.md` (ADR-Index gemäß ADR-095)

---

## 🎯 Kontext-Ladeordnung & Modus Operandi für Sessions

Um Halluzinationen und veraltete Fakten zu vermeiden, gilt für jede Jules-Session folgende Lade- und Nachschlage-Reihenfolge:
1. **System & Arbeitsumgebung**: `.agents/jules/context/SESSION_START.md` (automatisch durch `refresh-context.sh` erzeugt)
2. **Prozessanleitung**: `.agents/jules/SESSION_BOOTSTRAP.md`
3. **Aktueller Code-Zustand & Invarianten**: `AGENTS.md` (Verifizierter Code-Befund, Non-Obvious Decisions)
4. **Verbindliche Architektur-Vorgaben**: `docs/decisions/README.md` (ADR-Index gemäß ADR-095)

> 📌 **Hinweis für Prompt-Erstellung**: Die automatische Erstellung von GitHub-Issues durch Workflows/Gates ist deaktiviert.

---

## 📜 Audit-Report & Remediation-Prompts Kontext-Anker

Produktive Remediation-Historie wird ausschließlich über Git-Commit-Historie und `docs/decisions/` nachvollzogen.

---

## 📐 Crate-Topologie & Referenzen

Alleinige Quelle für Ring-Zuordnungen und Abhängigkeiten ist `capabilities.toml`.

> ⚠️ **Warnung**: `WORKING_STATE.md` darf NICHT als Architektur-Quelle referenziert werden, da dessen "Layer"-Feld ein rein build-graph-abgeleitetes Sortierkriterium ist und inhaltlich von `capabilities.toml` abweicht (z. B. ist `contextra-privacy` laut `capabilities.toml` Ring 3, in `WORKING_STATE.md` jedoch als Layer 0 geführt).

---

## 📖 Crate AGENTS.md Laderegel <!-- doc-ref-ignore -->

**MANDATORY FIRST STEP:** Bevor Code in einer Crate bearbeitet wird, MUSS Jules die jeweilige `AGENTS.md` der Crate laden (sofern vorhanden). Sie enthält Modul-Karten, API-Signaturen, Anti-Patterns und Lock-Hierarchien.

Die vollständige Lookup-Tabelle steht in `.agents/jules/context/SESSION_START.md` (Abschnitt "Crate-AGENTS.md laden").

---

## 🚫 Architektur-Entscheidungen (ADRs)

Vollständige Liste und Verbindlichkeit aller Architektur-Entscheidungen: siehe `docs/decisions/README.md` (ADR-Index gemäß ADR-095) sowie `AGENTS.md` Abschnitt **"Non-Obvious Decisions"**.

---

## ✅ Existierende Typen & API-Disziplin

- **Existierende Typen**: Die Übersicht aller bereits verifizierten Ring-0 Typen befindet sich in `docs/TYPE_REGISTRY.md`. Vor jeder Neuimplementierung die Typübersicht prüfen.
- **Kritische Implementierungs-Muster**: Details zu `TxId`-Allokation, `fsync`-Fehlerbehandlung, `unsafe`-Einschränkungen und HMAC-Keys sind zentral in `AGENTS.md` unter **"Non-Obvious Decisions"** hinterlegt.
