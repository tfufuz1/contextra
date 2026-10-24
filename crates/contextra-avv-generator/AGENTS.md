# contextra-avv-generator — Agenten-Hinweise

## Zweck
Generierung rechtlicher Auftragsverarbeitungsverträge (AVV) und Datenschutzvereinbarungen für Contextra Compliance Management.

## Ring & Invarianten
- Ring: 3
- Async erlaubt: ja
- Unsafe erlaubt: nein

## Abhängigkeiten (`may_depend_on`, siehe `capabilities.toml`)
- `contextra-types`

## Bekannte Fallstricke
- Vorlagen-Strings müssen UTF-8-konform sein und rechtliche Platzhalter korrekt maskieren.

## Testbefehl
`cargo test -p contextra-avv-generator --locked`
