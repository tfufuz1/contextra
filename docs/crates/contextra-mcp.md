# contextra-mcp — Crate-Spezifikation (Soll-Zustand, SPEC 2.1)

**Ebene:** E3 (Endpunkt)
**Kern-Closure:** `contextra`, `types`, `ids`, `privacy`, `erasure`
**Status:** Normativ (Soll-Zustand)
**Version:** 2.1
**Geltungsbereich:** `crates/contextra-mcp`
**Normative Quellen:** `docs/SPEC.md` §1.2, §5.2, §5.3, §5.4, §6.3, §11, INV-7, INV-12, INV-CLAIMS-1.

---

## 1. Zweck

`contextra-mcp` spricht MCP über stdio als JSON-RPC 2.0. Jedes Tool ruft die Façade `contextra` auf. Die Gate-Prüfung sitzt damit auf demselben Pfad wie die Rust-API (SPEC §6.3).

Das Crate startet keine eigene Saga und signiert keine Berichte. Es gibt keine Module, die Erfolg melden, ohne die Façade aufzurufen (INV-WIRING-2). Entfernte Inferenz-Crates werden nicht importiert.

## 2. Architekturregeln

1. `#![forbid(unsafe_code)]`. Zero-Panic. `default = []`.
2. Antworten und Logs enthalten keinen Klartext eines Löschziels (INV-12).
3. Außentexte stammen aus der Satzbibliothek von `contextra-erasure` und nur zusammen mit dem Messwert, an den der Satz gebunden ist (INV-CLAIMS-1).
4. `Erased` und ein unbekannter Schlüssel sind in der Antwort dasselbe Fehlerbild.

## 3. Tools

### 3.1 `search_hybrid`

Eingabe: `query`, `collection`, `top_k`, optionales `as_of`. Ausgabe: Treffer, die das Gate freigegeben hat. Ein gelöschtes Dokument erscheint nicht. `as_of` ändert das nicht.

### 3.2 `get_document`

Eingabe: `collection`, `doc_id`. Ausgabe: Inhalt, wenn das Gate `Open` sagt.

Fehlt das Dokument oder ist es `Erased` oder `Restricted`, lautet die JSON-RPC-Fehlermeldung `not found`. Dieselbe Meldung, derselbe Code. Die Antwort enthält kein Feld, das die beiden Fälle trennt, und keinen Status `KeyRevoked`.

### 3.3 `erase_subject`

Eingabe: `subject_ref`, `attestation_code`, `operator_ref`.

Der Aufruf startet die Saga mit den Schritten 1 bis 9 über `Contextra::erase_subject` und kehrt nach Takt 1 zurück.

Ausgabe, nur diese Felder:

| Feld | Inhalt |
|---|---|
| `erasure_id` | `ErasureId` |
| `state` | `Frozen`, `Restricted` oder `Refused` |
| `seal` | `1`, wenn der Zustand `Frozen` ist, sonst keins |
| `class` | `L`, wenn Siegel 1 vorliegt, sonst keine Klasse |

Die Antwort ist kein `DeletionProof` und kein Beleg für Schlüsselvernichtung. `KeyDestroyed` ist zu diesem Zeitpunkt noch nicht erreicht. Der Name `KeyRevoked` wird nicht verwendet. Ein Satz wie „gelöscht" wird nicht gesendet.

`Refused` entsteht bei einer Aufbewahrungspflicht. `Restricted` entsteht bei Legal Hold. Beides ist kein Erfolg der Vernichtung.

### 3.4 `verify_erasure`

Eingabe: `erasure_id`. Ausgabe: der aktuelle `SagaState` aus SPEC §5.2 und, sobald ein Bericht existiert, die gemessene Klasse (`L`, `P`, `C*`, `C1`, `C2`, `C3`, `M`) und die vorliegenden Siegel.

Ein Außensatz wird nur angehängt, wenn die Satzbibliothek ihn an genau diesen Messwert bindet. Ohne bestandene Messung gibt es keinen Satz. „100 % gelöscht", „unwiederbringlich" und „DSGVO-konform" sind keine zulässigen Ausgaben.

### 3.5 Weitere Löschziele

`erase_document`, `drop_collection` und `drop_tenant` folgen demselben Ticket-Format wie `erase_subject`. Sie unterscheiden sich in `EraseTarget`, nicht im Statusvokabular.

## 4. Fehlerbild

Transportfehler sind JSON-RPC-Fehler ohne Payload-Echo. `JournalNotDurable` wird als Fehler von Takt 1 gemeldet und nicht als Ticket mit gesetzter Lesesperre. `CapabilityUnsupported` wird durchgereicht und nicht in einen Erfolg umgedeutet.

## 5. `Cargo.toml`

```toml
[package]
name = "contextra-mcp"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "MCP stdio server for the Contextra facade"

[lints]
workspace = true

[dependencies]
contextra = { workspace = true }
contextra-types = { workspace = true }
contextra-ids = { workspace = true }
contextra-privacy = { workspace = true }
contextra-erasure = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }

[features]
default = []
```

Keine Abhängigkeit auf `contextra-crypto` für eine eigene Signatur. Die Signatur bleibt im Berichtspfad von `contextra-erasure` über den Port aus `contextra-crypto`.

## 6. Tests

| Pflicht | Prüfung |
|---|---|
| T-DEL-11 | `search_hybrid` und `get_document` liefern einen gelöschten Scope nicht aus |
| Ticket | `erase_subject` enthält nach Takt 1 weder `KeyRevoked` noch `KeyDestroyed` noch einen `DeletionProof` |
| INV-12 | Die Fehlermeldung von `get_document` ist für einen unbekannten und einen gelöschten Schlüssel gleich |
| INV-CLAIMS-1 | `verify_erasure` ohne Messung enthält keinen Außensatz |
| INV-WIRING-2 | Kein Tool-Handler liefert `Ok`, ohne die Façade aufzurufen |
