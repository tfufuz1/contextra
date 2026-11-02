---
name: contextra-deletion-verifier
description: >-
  Use this skill to verify and develop the Contextra erasure pipeline: the erasure saga, the read gate (ErasedSet),
  the purge steps, the key destruction in the vault, the field measurements MK1 to MK3 with positive controls, and the signed erasure report.
---

# Löschpipeline prüfen

Der Skill führt durch Entwicklung und Prüfung des Löschkerns. Maßgeblich ist [`docs/SPEC.md`](../../../../docs/SPEC.md), Kapitel 11 und 12, Anhang D und F.10.

## 1. Prinzip (D-01)

Löschung in v1 stützt sich auf drei Säulen. Keine davon allein gilt als Löschung.

1. **Gate und Purge.** Das ReadGate sperrt ab `Gated` alle Lesepfade. Der Purge entfernt die Daten physisch.
2. **Schlüsselvernichtung.** Der Schlüssel der höchsten Ebene des Ziels wird aus dem Tresor entfernt.
3. **Messung.** Jede Schicht wird mit Positivkontrolle gemessen. Der Bericht ist signiert.

Die Reichweite der Schlüsselvernichtung wird je Schicht ausgewiesen (`key_coverage`, D-69). Die rechtliche Wertung bleibt beim Betreiber (E-L2).

## 2. Saga (SPEC 11.2)

```
Requested ─▶ Gated ─▶ KeyDestroyed ─▶ Purging ─▶ Verifying ─▶ Completed
    └──────────────── Pending(reason) ◀──── jeder Schritt ist idempotent
Refused (endgültig, vor dem Gate)
```

Bis `Gated` sind 4 `fsync` vorgesehen (D-58). Bei gleichzeitigen Anfragen teilen sich die Anfragen einen Gate-Log-`fsync` (D-78).

## 3. Prüfschritte

- [ ] Nach `Gated` liefert jeder Lesepfad `Erased`, auch Zeitreisen (T-GATE-1, T-VER-1).
- [ ] Nach `Gated` schreibt nichts mehr in das Ziel (I-1).
- [ ] Der Schlüssel der höchsten Ebene ist genullt, das Fehlen wird geprüft (T-VAULT-1 bis T-VAULT-4).
- [ ] Die WAL wird nie umgeschrieben; entfernt werden ganze Segmente (D-73, T-WAL-2).
- [ ] Dateischlüssel `F` werden nach dem Manifest-Swap genullt, dann folgt `unlink` (I-17).
- [ ] Nicht-Ziel-Records sind auf Record-Ebene bitgleich (I-3, T-ISO-1).
- [ ] `Completed` nur mit gültigen Zeugen aller Schichten (I-4, P1).
- [ ] Der Bericht enthält keine Inhalte, Schlüssel oder Klartext-Kennungen (P13, T-REPORT-1).

## 4. Messklassen (SPEC 12.1)

| Klasse | Prüfung |
|---|---|
| MK1 Struktur | entschlüsselte Strukturen und Steuerdateien gegen die Matcher |
| MK2 Rohscan | Rohbytes aller Dateien auf Klartext-Needles und bekannte Muster |
| MK3 Schlüsselprobe | Feld: Schlüssel weg, Ableitung scheitert. Lab: alter Schlüssel öffnet nichts |

Ein Zeuge ohne Positivkontrolle ist ungültig (HR-10). `LayerWitness` ist nur aus `Scanner<Armed>` konstruierbar (D-72, I-19).

## 5. Bericht (SPEC 11.7, F.10)

Kanonisches JSON mit sortierten Schlüsseln, ohne Gleitkommazahlen. Signatur Ed25519 über die Domäne `ctx/report/v1`. Die Pflichttexte stehen in SPEC Anhang D. Eine Signatur beweist Integrität und Herkunft, nicht die Richtigkeit des Inhalts (E-L11).

## 6. Prüfbefehle

```bash
cargo test -p contextra-erasure --locked   # ab WP-3.1, wenn das Crate Workspace-Mitglied ist
cargo test -p contextra-arch-tests --locked
```

Der Bericht-Prüfer `ctx-verify report` und der Feldscanner `ctx-verify scan` entstehen mit WP-4.4.

Weitere Details: [`references/deletion_architecture.md`](references/deletion_architecture.md).
