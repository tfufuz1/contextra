# contextra-license — Agenten-Hinweise

## Zweck
Durchsetzung von Lizenzprüfungen, Feature-Ringen (`Fast`, `Sovereign`, `Compliance`) und OpenFast-Fail-Closed-Garantien.

## Ring & Invarianten
- Ring: 4 (Lizenz- und Mandanten-Governance)
- Async erlaubt: ja
- Unsafe erlaubt: nein (`#![forbid(unsafe_code)]`)

## Abhängigkeiten (`may_depend_on`, siehe `capabilities.toml`)
- `contextra-types`
- `thiserror`

## Bekannte Fallstricke
- Fail-Closed-Verhalten muss bei abgelaufenen oder ungültigen Signaturen strikt aufrechterhalten werden.

## Testbefehl
`cargo test -p contextra-license --locked`
