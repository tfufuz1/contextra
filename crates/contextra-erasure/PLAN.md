# contextra-erasure — Implementierungsspezifikation

**Ebene:** E1 (Kern, sicherheitskritisch, R2)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch). **Stand:** 21.11.2026, Abgleich mit SPEC v4.1. Verweist nur auf SPEC-Kapitel und IDs. Ist-Abweichungen sind markiert, ungeklärte Punkte stehen unter „Offene Punkte (Entscheidung nötig)“ in Abschnitt 6.

## 1. Zweck und Ebene
Löschkette: Log (CXEL, 7.8), Gate (CXGS, `TenantGate: ReadGate`, 7.8), Ticket, Klassifikation (7.3), Kette E0 bis E4 mit Ports (7.4), Zustandsautomat (7.5), Scrub-Planung (7.6, reine Planung), Prüfung (REQ-ER-3, REQ-ER-6), Halt (REQ-RET-2), Ledger (7.10) (SPEC 15, Kapitel 7). Abhängigkeiten nach Anhang E: `ports`, `ids`, `env`, `durable-fs`. Ziel-Umfang: 4.500 Zeilen (SPEC 3.1, REM-10). Ist: laut Zielspalte 6.708 Zeilen (Altbestand).

## 2. Vertrag
Implementiert: `ReadGate` (Besitz des Gates je Mandant, SPEC 3.4) über `TenantGate` in `gate.rs`. Der Registry-Port liegt in `erasure` (SPEC 3.4). Der `ErasureSink` ist Empfänger, nicht Implementierung dieses Crates.
Ist-Abweichung: Die Module entsprechen nicht dem Soll `log`, `gate`, `ticket`, `classify`, `chain`, `state`, `scrub_plan`, `review`, `hold`, `ledger`. Ist hat `chain/` mit `journal`, `layer`, `saga`, `state`, `target`, `retention`, `adapters`, `ports` sowie Gate-Modul `src/gate/saga_gate.rs`. Die Zuordnung zum Soll ist zu prüfen.

## 3. Abhängigkeiten
**Erlaubt (Anhang E):** `ports`, `ids`, `env`, `durable-fs`.
**Ist (Cargo.toml, normal):** `contextra-ids`, `contextra-env`, `contextra-durable-fs`, `subtle`, `thiserror`, `tracing`, `serde`, `serde_json`, `zeroize`, `blake3`, `sha2`, `hmac`, `bytes`, `parking_lot`.
**Ist-Abweichung:** `contextra-ports` fehlt in der Ist-Liste, obwohl Anhang E und SPEC 15 es verlangen. Ob die Löschkette ohne Port-Abhängigkeit auskommt, ist offen (siehe 6).
**Dev-Abhängigkeit:** `tokio` (Features `test-util`, `macros`, `rt`), Zeile 39 in `Cargo.toml`, Abschnitt `[dev-dependencies]`. Das ist keine normale Abhängigkeit. `deny.toml` erfasst sie trotzdem, weil dort kein Wrapper für `contextra-erasure` steht. Entscheidung für Punkt 10 der CLAUDE.md siehe Abschnitt 6.
**Features:** `default = []` (R3 eingehalten).

## 4. Module
| Datei | Verantwortung (Ist) | Befund |
|---|---|---|
| `src/chain/*` (adapters, journal, layer, mod, ports, retention, saga, state, target) | Löschkette | Soll-Zuordnung zu 7.4 und 7.5 prüfen |
| `src/gate/saga_gate.rs` | Gate (Saga-Anbindung) | Soll (`gate`, CXGS) |
| `src/io.rs` | E/A | zu prüfen |
| `src/error.rs`, `src/lib.rs` | Fehler, Wurzel | Soll |
| `src/vault.rs`, `src/vault/*` (durable, open, rehash, scrub, slot, table) | Tresor-Logik | **Ist-Abweichung**: Soll-Module nennen keinen Tresor. Herkunft ist `contextra-vault` (REM-01). Entfernen oder zuordnen, siehe 6 |
| `src/crypto_local.rs` | lokale Kopie von Krypto-Typen | **Ist-Abweichung**: Krypto nur in `contextra-crypto` (R4). Entfernen |
| `src/types_local.rs` | lokale Kopie von Typen | **Ist-Abweichung**: Typen aus `types` (REM-30). Entfernen |

Soll-Module ohne Ist-Entsprechung: `ticket`, `classify` (7.3), `review`, `hold`, `ledger` (7.10), `scrub_plan` (7.6).

## 5. Entfernungen
| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| REM-10 | Löschkette in Altform, nach Kapitel 7 neu aufbauen (Zustand laut SPEC 12, offen) | siehe SPEC 12 |
| REM-16 | laut SPEC 15 genannt, Inhalt in SPEC 12 nachlesen (offen) | siehe SPEC 12 |
| R4 | `crypto_local.rs` | `ls crates/contextra-erasure/src/crypto_local.rs` nicht vorhanden |
| REM-30 | `types_local.rs` und Typen aus `contextra-types` | `grep -rn "contextra_types" crates/contextra-erasure` leer |
| REM-01 | `src/vault*`, sofern Herkunft `contextra-vault` | `grep -rn "vault" crates/contextra-erasure/src` leer, nach Entscheidung |
| REM-14 | `contract.rs` (nicht vorhanden), ANCHOR-Dateien (0) | `grep -rl "ANCHOR\[" crates/contextra-erasure` leer |

## 6. Umbau
1. Module nach Soll (Kapitel 7.3 bis 7.10) ordnen. Je Modul ein Commit.
2. `crypto_local.rs` und `types_local.rs` entfernen, Typen aus den Besitzern beziehen.
3. Abhängigkeit `contextra-ports` herstellen (Port-Implementierungen, ReadGate).
4. `unwrap`/`expect`/`panic!` (42 Treffer in `src`) nach R6 abbauen.
5. Die `Instant::now`/`SystemTime::now`-Treffer (3) auf `Clock` aus `env` umstellen (R10, G-12).

**Offene Punkte (Entscheidung nötig):**
- **O-1 Tresor (`vault/`).** Verbleib entscheiden. SPEC 15 nennt keinen Tresor in `erasure`.
- **O-2 Tokio (Punkt 10 der CLAUDE.md).** Die Dev-Abhängigkeit ist die einzige tokio-Nutzung von `erasure`. Zwei Wege: (a) `tokio` als Dev-Abhängigkeit entfernen und die Tests umschreiben, (b) in `deny.toml` einen Wrapper für `contextra-erasure` mit TRANSITION-Vermerk eintragen. Entscheidung des Nutzers.
- **O-3 Fehlende Abhängigkeit `contextra-ports`.** Umbau nötig, bevor `ReadGate` implementiert werden kann.
- **O-4 Inhalt von REM-10 und REM-16** nach SPEC 12 lesen und bestätigen.

## 7. Tests
- T-ER-01 bis T-ER-26 (Anteile, SPEC 15), T-LOG-01.
- G-14: CI-Lauf T-ENC-01 und T-LOG-01 grün.
- Ist-Befund: nicht gezählt.

## 8. Abnahme
| Gate | Ist-Befund | Soll |
|---|---|---|
| G-02 | kein `unsafe`, `forbid(unsafe_code)` in `lib.rs` | eingehalten |
| G-03 | `contextra-ports` fehlt in Ist, `tokio` nur dev | offen (O-3, O-2) |
| G-05 | Netzwerk-Crates im normalen Baum: keine gefunden (`cargo tree -e normal`) | eingehalten |
| G-12 | 3 Treffer `Instant::now`/`SystemTime::now` | verletzt |
| G-13 | 42 Treffer | Abbau nach R6 |
