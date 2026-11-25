# ADR-N14: Kopplung von KeyRegistry-Widerrufen an den RevocationLog (Split-Brain Behebung)

* **Status:** Proposed
* **Datum:** 2026-10-04
* **Kontext:** Befund FIND-01 (Kritischer Split-Brain bei KeyRegistry-Widerrufen)

---

## 1. Belegter Ist-Zustand

In `crates/contextra-crypto/src/kv_shredding.rs` führt das Rufen von `KeyRegistry::revoke_group()` oder `KeyRegistry::revoke_record()` nur zu einer Aktualisierung der in-speicher-bezogenen Datenstrukturen (`self.revoked_groups` bzw. `self.groups`).

Obwohl `KeyRegistry` über ein optionales Feld `pub revocation_log: Option<Arc<RevocationLog>>` verfügt, rufen weder `revoke_group` noch `revoke_record` die Methode `self.revocation_log.append(...)` auf.

### Konsequenz
Bei einem Prozessneustart gehen alle in-speicher vermerkten Widerrufe verloren, sofern der Aufrufer den `RevocationLog` nicht manuell separat aktualisiert hat. Dies verletzt das zentrale Löschnachweis-/GDPR-Versprechen (Crypto-Shredding) unter Neustart- und Crash-Szenarien.

---

## 2. Optionen zur Behebung

### Option 1: Automatische Synchrone Eintrags-Persistierung in `revoke_group`/`revoke_record`
* **Beschreibung:** `revoke_group` und `revoke_record` prüfen `self.revocation_log`. Wenn ein Log vorhanden ist, wird vor dem Löschen der in-speicher Keys synchron `log.append(...)` aufgerufen. Falls das Schreiben fehlschlägt, schlägt der Aufruf fehl (`Result<bool>`), um unpersistierte Widerrufe zu verhindern.
* **Vorteile:**
  - Erfüllt das WAL-First / Persistenz-Prinzip strikt.
  - Verhindert jegliches Split-Brain zwischen RAM und Festplatte.
* **Nachteile:**
  - `revoke_group` und `revoke_record` Schnittstelle muss von `fn(...) -> bool` auf `fn(...) -> Result<bool>` geändert werden.
* **Aufwand/Risiko:** Gering (ca. 1 Personentag).

### Option 2: Verpflichtender `RevocationLog` im Konstruktor (`KeyRegistry::new`)
* **Beschreibung:** `KeyRegistry::new` erfordert die Übergabe eines `Arc<RevocationLog>` (kein `Option`). Beim Start liest die `KeyRegistry` alle historischen Widerrufe aus dem `RevocationLog` ein und rehydriert `self.revoked_groups`.
* **Vorteile:**
  - Garantiert, dass eine `KeyRegistry` niemals ohne dauerhaften Widerrufslog betrieben werden kann.
* **Nachteile:**
  - Test-Fixtures müssen Mocks oder Tempdir-Logs bereitstellen.
* **Aufwand/Risiko:** Mittel (ca. 2 Personentage).

---

## 3. Empfehlung der Architektur
**Option 1 in Kombination mit Rehydration aus Option 2**: `KeyRegistry::revoke_group` schreibt synchron in `RevocationLog`, und `KeyRegistry::open_with_log` rehydriert beim Start den In-Memory-Status aus dem Log.
