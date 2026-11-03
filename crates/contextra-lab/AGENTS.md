# AGENTS – contextra-lab

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (8.16, 12.1 bis 12.4, AR-4, Anhang F). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** Unabhängige Prüfinstanz für die Löschmessung, ohne Produktions-Decoder.

**Ring und Abhängigkeiten:** Ring dev (Mitglied des Workspace). Abhängig weder von `store`, `engine`, `erasure` noch von Produktions-Decodern in `crypto` (AR-4). Eigene Implementierungen der Formate (Anhang F) und eigene Krypto-Aufrufe über externe, geprüfte Crates.

## Fachprofil (Arbeitsrolle)

Messtechniker für den Nachweis von Restdaten (Digitalforensik-Labor) und Spezialist für Mutationstests. Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- Unabhängige Parser für alle On-Disk-Formate (F.1 bis F.10); Scanner für MK1 (Struktur), MK2 (Rohscan), MK3 (Schlüsselprobe).
- Differenzprüfung gegen die Produktion; Mutanten (bewusst unvollständige Löschung) als Prüfmaßstab für den Scanner selbst.
- Berichtsform für Auditoren.

## Normative Anker

- **Unabhängigkeit vor Wiederverwendung:** Ein gemeinsamer Fehler im Parser von Produktion und Prüfer würde jeden Befund verfälschen. Daher kein Import von Kernlogik, keine geteilten Decoder (AR-4, 12.3).
- **MK1 bis MK3 (12.1):** MK3 im Lab verwendet den vor der Löschung gesicherten DEK der Fixtur und prüft, dass verbliebene Envelopes und Blöcke nicht öffenbar sind; in Produktion existiert dieser Schlüssel nicht mehr.
- **Positivkontrolle (HR-10, T-RAW-2):** Ein Negativbefund ohne bestandene Positivkontrolle ist ungültig.
- **Funktionen (12.3):** Fixturen mit bekannten Mustern und gesicherten DEKs, Löschung über die öffentliche API, danach MK1 bis MK3, Differenzprüfung (T-DIFF-1), Restore-Szenarien, Mutationstests (T-MUT-1). Das Scanner-Modul wird von `ctx-verify scan` verwendet.
- **Kalibrierung:** Der Scanner wird an Mutanten kalibriert, nicht an der Produktion.
- Evidenz: Das Release-Evidenzbündel (Lab-Protokolle, Mutationsergebnisse, Toolchain, Build-Hash) wird gehasht und als `lab_evidence` im Bericht referenziert (12.4); Evidenz gehört ins Release-Artefakt, nicht in den Quellbaum (D-98).

## Verboten

Abhängigkeiten zu `store`, `engine`, `erasure`, Produktions-Decodern; `unsafe` (`forbid(unsafe_code)`); Anpassen des Scanners an die Produktion, um Befunde zu beseitigen; Netzwerk.

## Abnahme

`T-RAW-1` bis `T-RAW-6`, `T-DIFF-1`, `T-MUT-1`, `T-KEYCOV-1`; jeder Mutant wird erkannt, Golden-Formate sind stabil (`T-FMT-1`; Tests `scanner_tests`, `mutant_tests`, `golden_formats`, `diff_tests`); Fuzz der Parser (`fuzz/`).

## Arbeitspakete

WP-4.4 (`lab` und `ctx-verify`, Mutationstests). Statusquelle: `.agents/jules/tasks/BACKLOG.md`.

## Prüfbefehle

```bash
cargo test -p contextra-lab --locked
cargo test -p contextra-arch-tests --locked
```
