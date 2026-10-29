# ADR-0098: Determinismus-Ausnahmen für Krypoto-Schlüssel und Salt-Generierung

- **Status:** Proposed — benötigt Maintainer-Freigabe
- **Datum:** 2026-10-29
- **Autoren:** Architecture Governance Team
- **Relevante Komponenten:** `contextra-store`, `contextra-crypto`, `governance/determinism-allow.toml`

## Kontext
Regel 3 der Agenten-Betriebsanleitung (P28) verbietet die direkte Nutzung von `rand::thread_rng` in Produktionscode zugunsten injizierter Rng-Ports.
Bei der Generierung von Kryptographie-Salts und Integritätsschlüsseln in Rebound-/Recovery-Pfaden (`recovery.rs`, `hmac.rs`) ist jedoch die unmittelbare Nutzung eines echten kryptographischen Entropie-Provisors zwingend erforderlich, um Sicherheitsabschwächungen durch vorhersagbare Pseudo-Zufallsgeneratoren zu verhindern.

## Entscheidung
1. `rand::thread_rng` wird explizit als Ausnahme für die Erzeugung kryptographischer Salts und HMAC-Integritätsschlüssel in `contextra-store/src/wal/recovery.rs` und `contextra-store/src/wal/hmac.rs` zugelassen.
2. Diese Ausnahmen werden in `governance/determinism-allow.toml` dokumentiert.
3. Für alle anderen (nicht-kryptographischen) Zufalls- und Zeitoperationen bleiben die Ports (`Clock`, `Rng`) zwingend vorgeschrieben.

## Konsequenzen
- **Positiv:** Gewährleistung kryptographischer Entropie für Integritätsschlüssel.
- **Negativ:** Explizit gepflegte Ausnahmeliste im Gate `determinism`.
