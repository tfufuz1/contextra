# AGENTS – contextra

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (7, 8.14, D-66, D-89, D-95). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** Öffentliche API und Verdrahtung. Enthält keine Logik.

**Ring und Abhängigkeiten:** Ring 4. Erlaubt: `engine`, `durable-fs`, `types` (D-88 (4)). Erzeugt keinen Tresor (D-95).

## Fachprofil (Arbeitsrolle)

API-Designer für eingebettete Bibliotheken (Vorbild SQLite und `redb`), Semver- und Fehlerdesign. Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- Builder mit sicheren Defaults; Pflichtparameter per Typsystem erzwungen (`trybuild` Compile-Fail); minimale öffentliche Oberfläche.
- Fehlertypen mit Handlungsempfehlung, aber ohne Geheimnisse (HR-4).
- Dokumentation, die über Löschung nur die zulässige Außenaussage aus SPEC 6.4 verwendet; strikte Trennung `soft_delete` (Tombstone) und `erase` (Löschvorgang).

## Normative Anker

- **Einzige öffentliche Oberfläche (D-66, 7):** alle Typen `#[non_exhaustive]` (außer Konfigurationsbausteinen), alle Methoden `async`, mutierende Methoden cancel-safe (I-14). Format-Version und Crate-Version sind getrennt.
- **Drei Module (D-89):** `api` (Re-Exporte `Contextra`, `Tenant`, `Collection`), `wiring` (`system_defaults()`: `RealVfs`, Systemuhr, OS-CSPRNG), `error`. `EngineBuilder` liegt in `engine`; die Fassade stellt `Contextra::builder()` bereit.
- **Pflicht per Typ:** `TenantContext` (T-API-1, `trybuild`), `RetentionAttestation` (T-API-2), `backup_retention`, `erase_sla`, `passphrase`, `runtime`. Es gibt kein `delete`; `Collection::soft_delete` trägt in Rustdoc an erster Stelle den Satz, dass für Löschpflichten `erase` zu verwenden ist (D-51, T-API-3).
- **Lösch-API (7.3):** `erase(Ziel)` liefert nach `Gated` zurück; `report()` liefert den signierten Bericht; `RetentionCode` ist ein Enum ohne Freitext (E-L12).
- **Doku-Wortlaut:** Löschaussagen nur nach SPEC 6.4; als erstes die Warnung zum Passphrasenverlust (HR-3); Hinweis „keine Rechtsberatung“ (E-L8).

## Entscheidungshilfen

- Neue öffentliche Signatur? Jede Signatur ist eine Zusage (D-66): zuerst SPEC 7 ändern, dann API-Signatur-Test, dann Code.
- Logik in der Fassade? Gehört nach `engine` (T-ARCH-7).

## Verboten

Logik, Profile, Rewriter (`collection_profile`, `performance_profile`, `query_rewriter`), `EncryptionMode` und `Unencrypted` (D-05), `LicenseGate` (D-23), Embedder-Bezüge (D-24), Re-Exporte von `DocId`, `chunker`, `rank`, `DriftStatusProvider` (D-89), Tresor-Erzeugung (D-95), Fixdimension im Builder.

## Abnahme

`T-API-1` (trybuild), `T-API-2`, `T-API-3` (Doc-Test), `T-ARCH-3`, `T-ARCH-7`, `T-LINT-1`, `cargo-semver-checks`; Beispiele und Doc-Tests kompilieren.

## Arbeitspakete

WP-1.5 (Fassade nach D-89 schneiden), WP-9.1 (README, Rustdoc). Statusquelle: `.agents/jules/tasks/BACKLOG.md`.

## Prüfbefehle

```bash
cargo test -p contextra --locked
cargo test -p contextra --locked --doc
cargo test -p contextra-arch-tests --locked
```
