# Gate-Vorschlag: `check-audio-stt-deny`

- **Anforderung**: REQ-ARCH-017 (§4.1)
- **Status**: `harness:planned`
- **Mechanismus**: `G` (Gate via `cargo metadata`)

## 1. Ziel
Durchsetzung des funktionale Grenzen-Verbots: Der Contextra-Kern darf keine Echtzeit-Audio- oder STT-Funktionen (Speech-to-Text) enthalten. Das Gate analysiert den gesamten externen Abhängigkeitsbaum aller Workspace-Crates auf verbotene Crate-Namen und Keywords.

## 2. Eingaben
- `cargo metadata --format-version 1` Ausgabe des Workspace.
- Deny-Liste verbotener Crates/Keywords (z. B. `whisper`, `cpal`, `rodio`, `portaudio`, `vosk`, `deepspeech`, `stt`, `audio`).

## 3. Ausgabe
- Konsolenbericht über gescannte Abhängigkeiten.
- Exit-Code `0` bei Abwesenheit verbotener Audio/STT-Crates.
- Exit-Code `1` bei Fund einer unzulässigen Audio/STT-Abhängigkeit.

## 4. Selbsttest mit synthetischem Repository (REQ-QA-008)
- **Positive Control**: Ein synthetischer `Cargo.toml`-Eintrag für `whisper-rs` führt zu Exit-Code `1`.
- **Negative Control**: Ein sauberes Repository ohne Audio-Abhängigkeiten passiert mit Exit-Code `0`.
