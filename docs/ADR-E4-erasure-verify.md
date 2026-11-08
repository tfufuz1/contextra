# ADR-E4: Unabhängige Verifikationsinstanz für Löschberichte und Feldscans

- **Status:** Angenommen (Entscheidungsvorlage zur Umsetzung von Paket E4)
- **Datum:** 07.11.2026
- **Autor:** Principal Rust Engineer & Architekt
- **Kontext:** Baukasten-Architektur, Paket E4, SPEC §8.17, §11.7, Anhang D, F.9, F.10

## 1. Kontext & Problemstellung

Im Rahmen der Modularisierung des Contextra-Ökosystems (Baukasten-Vertrag M1–M11, Entscheidung D-E1) wurde die Löschkette vollständig aus dem Speicherkern ausgelagert und in `contextra-erasure` gebündelt.
Die Frage ist nun, wie die forensische Verifikation kryptografisch signierter Löschberichte (`DeletionReport`) sowie unabhängige Feldscans (MK1 Strukturscan, MK2 Roh-Residuen, MK3 Schlüsselzerstörung im `FileVault`) architektonisch verortet werden sollen.

Historisch referenzierten Spezifikationen und Berichte `ctx-verify` und `contextra-lab`. Beide Crates existieren im aktuellen Repository-Stand nicht mehr.

## 2. Optionen & Kostenanalyse

### Option A: Kein Verifier
- **Beschreibung:** Es wird kein Verifier bereitgestellt. Berichte werden nur generiert; die Überprüfung wird externen Parteien ohne Referenzwerkzeug überlassen.
- **Vorteile:** 0 Zeilen Code, kein Wartungsaufwand.
- **Nachteile & Risiken:** Große Sicherheitslücke in der Auditierbarkeit. Keine maschinelle Validierung von Signaturen, Hash-Ketten, Zeitstempel-Monotonie oder Restrisiken. Keine unabhängige Prüfinstanz. Widerspricht SPEC §8.17, §11.7 und den Produktzielen von Phase 6.

### Option B: Verifier-Modul innerhalb von `contextra-erasure`
- **Beschreibung:** Ein Verifikations-Submodul (z. B. `contextra_erasure::verify`) wird direkt in `contextra-erasure` integriert.
- **Vorteile:** Einfache Wiederverwendung interner Typen ohne Schnittstellendopplung.
- **Nachteile & Risiken:**
  - **Architektur- und Audit-Konflikt:** Die erzeugende Instanz verifiziert sich selbst (keine Unabhängigkeit / Vier-Augen-Prinzip).
  - **Kopplung:** Dritte, die lediglich einen Löschbericht extern verifizieren möchten, müssten die gesamte `contextra-erasure`-Crate inklusive Vault, Saga, Ledger und Gate-Log einbinden.
  - Verletzt Baukasten-Prinzip der feingranularen Autarkie.

### Option C: Eigene Crate `contextra-erasure-verify` (Empfohlen)
- **Beschreibung:** Eine eigenständige Crate `crates/contextra-erasure-verify` (Ring Tooling).
- **Vorteile:**
  - **Echte forensische Unabhängigkeit:** Hängt **NICHT** an `contextra-erasure`. Liest ausschließlich Spezifikationsformate (kanonisches JSON F.10, Signaturen Ed25519, Shard-Layout F.3) oder bindet höchstens neutrale Typen aus `contextra-types` ein.
  - **Schlanker Footprint:** Kann als eigenständiges CLI-Tool und Library in Fremdsysteme oder Audit-Umgebungen exportiert werden.
  - **Zwang zur Positivkontrolle:** Garantiert, dass ein Negativbefund (keine Daten gefunden) nur nachgewiesen werden kann, wenn die Positivkontrolle erfolgreich war.
  - Erfüllt alle Baukasten-Kriterien M1–M11 (insb. M1 DAG, M2 Isolation, M3 Schnittstellenreinheit, M9 forbid unsafe).
- **Kosten:** Einmaliger Implementierungsaufwand (~600–800 Zeilen sauberer, isolierter Rust-Code).

## 3. Entscheidung & Empfehlung

**Empfehlung: Umsetzung von Option C (`contextra-erasure-verify`).**
Die Crate wird als Ring Tooling ohne Kante zu `contextra-erasure` realisiert, implementiert striktes Ed25519 `verify_strict`, kanonisches JSON-Parsing, Zeitstempel-Prüfung, Positivkontroll-Zwang für alle Zeugen und unabhängigen MK1/MK2/MK3-Feldscan mit Zeroize-geschützter Passphrasenübergabe via File Descriptor.
