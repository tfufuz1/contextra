# Gate-Vorschlag: `check-mmap-encrypted-volume`

- **Anforderung**: REQ-KEY-004 (§5)
- **Status**: `harness:planned`
- **Mechanismus**: `G` (Gate via Runtime-/Mount-Scan)

## 1. Ziel
Überprüfung der Systemumgebung bei `mmap`-Nutzung: Per `mmap` gelesene Vektor- und Indexdateien dürfen nur auf verschlüsselten Datenträgern (LUKS/dm-crypt, FileVault, BitLocker) liegen.

## 2. Eingaben
- Mount-Info des Host-Systems (`/proc/mounts`, `lsblk`, `mount`).
- Pfadkonfiguration des Contextra-Datenverzeichnisses.

## 3. Ausgabe
- Statusbericht über den Verschlüsselungsstatus des Mount-Points.
- Exit-Code `0` bei nachgewiesener Volume-Verschlüsselung oder Deaktivierung von `mmap`.
- Exit-Code `1` bei versuchtem `mmap`-Lesezugriff auf unverschlüsseltem Dateisystem.

## 4. Selbsttest mit synthetischem Repository (REQ-QA-008)
- **Positive Control**: Ein synthetischer Pfad auf tmpfs ohne Verschlüsselungs-Flag erzeugt Exit-Code `1`.
- **Negative Control**: Ein verschlüsselter LUKS-Loop-Back Mount passiert das Gate (Exit-Code `0`).
