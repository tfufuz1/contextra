# ADR 0009: Scope von Contextra v1 – Kern (14 Crates) oder Vollumfang (35 Crates)

| Feld | Wert |
|---|---|
| Status | **Proposed**. Nicht bindend, bis Status auf `Accepted` steht |
| Datum | 31.10.2026 |
| Betrifft | SPEC §3.2 (D-K1), PLAN K1, `capabilities.toml`, Workspace-Manifest |
| Ersetzt | Keine. Ergänzt SPEC 4.0 D-R1 und D-R5, die beide durch den Ist-Stand überholt sind |

## Kontext

Fassung 4.0 (30.10.2026) legt den Scope auf einen Kern von 14 Crates fest (D-R1). Die Löschkette ist ausgesetzt (D-R5). Die Reduktion ist im Repo nur teilweise umgesetzt. Am 31.10.2026 (`3a0b11699`, „RÜCKSTELLUNG MEISTE CRATES“) sind 21 der entfernten Crates wieder im Workspace. Der Workspace hat 35 Verzeichnisse, `capabilities.toml` 34 Einträge, `xtask` fehlt im Verzeichnis, und Feature-Flags verweisen auf nicht deklarierte Abhängigkeiten.

Der Repo-Stand und die Dokumente widersprechen sich. Das ist die Ausgangslage. Eine Entscheidung ist nötig, bevor weitere Arbeit am Build (PLAN K0) und an der Löschkette (PLAN K3) sinnvoll ist.

## Optionen

### Option A: Kern v1 (14 Crates)

Im Workspace bleiben nur die Crates aus SPEC 4.0 §5.1. Die übrigen 21 werden durch einen Archiv-Tag (`archive/v1-vollumfang`) gesichert und aus `Cargo.toml` entfernt. Verzeichnisse bleiben bestehen, bis der Tag gesetzt und geprüft ist.

| Vorteil | Nachteil |
|---|---|
| Kleinster Build-Umfang, schnelle Grünphase | Verschiebung von 21 Crates mit Abhängigkeitsbruch (u. a. `agent` → `db`, `mcp` → Inferenz) |
| Klarer Scope für Dokumentation | Arbeit an Produktlinien P1 und P3 wird zurückgestellt |
| Passt zur Vorgabe „Kern vor Breite“ | Die Entscheidung widerspricht dem Stand vom 31.10. und muss begründet werden |

### Option B: Vollumfang (35 Crates)

Alle Crates bleiben. Jede Fähigkeit trägt Reifegrad und Ring in `capabilities.toml`. Die Löschkette ist `experimental`. Das Produkt wird über Feature-Flags geschnitten.

| Vorteil | Nachteil |
|---|---|
| Keine Verschiebung, Verlust von Arbeit ausgeschlossen | Build-Oberfläche bleibt groß, K0 dauert länger |
| Ermöglicht die Produktlinien P1 bis P3 ohne Wiedereinbau | Gefahr, dass „experimental“ als Versprechen gelesen wird |
| Entspricht dem tatsächlichen Repo-Stand | Erfordert Gating-Disziplin (SPEC §9) |

### Option C: Hybrid (Zwischenschritt)

Option B für Build und Tests. Die 21 Crates werden in `capabilities.toml` mit Reifegrad `experimental` versehen und aus `default-members` des Workspace genommen. Eine Entscheidung über Archivierung folgt erst nach K0.

## Empfehlung

Option C. Sie verhindert Datenverlust, schafft die Grundlage für K0 und lässt die Entscheidung zwischen A und B offen, bis der Build grün ist. Begründung: Die Frage, welche Crates gebraucht werden, ist ohne lauffähigen Build nicht belastbar beantwortbar.

Diese Empfehlung ist ein Vorschlag des Autors dieses Dokuments. Sie ist nicht beschlossen.

## Konsequenzen

- SPEC 4.1 §3.2 bleibt offen, bis dieser ADR `Accepted` ist.
- Die Löschkette bleibt in jedem Fall `experimental` (D-K4). Dieser ADR entscheidet nicht über Löschgarantien.
- Die ADRs 0001 bis 0008 werden durch diesen ADR nicht gültig.

## Entscheidungsfrist

Vorschlag: vor Abschluss K0.5. Bei Nichtentscheidung gilt Option C als Zustand.

## Entscheidung

Offen.

| Datum | Entscheider | Ergebnis |
|---|---|---|
| – | – | – |
