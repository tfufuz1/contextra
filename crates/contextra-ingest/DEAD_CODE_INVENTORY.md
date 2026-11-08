# Dead-Code-Inventar: `contextra-ingest` (E-07)

Gemäß SPEC v4.0 Regel 2 (E-07) ist das Löschen von Algorithmen oder Trait-Implementierungen verboten.
Die durch `tools/decouple_inspector.py -c contextra-ingest --dead-code` identifizierten 29 Symbole wurden geprüft:

## 1. HTML TreeSink Implementierung (`crates/contextra-ingest/src/formats/web/html.rs`)
Die folgenden 16 Methoden implementieren den Trait `html5ever::tree_builder::TreeSink`. Sie sind für die korrekte Funktion des externen HTML5-Parsers (`html5ever`) zwingend erforderlich und werden durch `html5ever` über dynamischen/statischen Trait-Dispatch aufgerufen:
- `parse_error`
- `get_document`
- `set_quirks_mode`
- `same_node`
- `elem_name`
- `get_template_contents`
- `append_based_on_parent_node`
- `append_before_sibling`
- `append_doctype_to_document`
- `add_attrs_if_missing`
- `remove_from_parent`
- `reparent_children`
- `is_mathml_annotation_xml_integration_point`
- `create_element`
- `create_comment`
- `create_pi`

## 2. Serde Visitor Implementierung (`crates/contextra-ingest/src/formats/structured/json.rs`)
Die folgenden 13 Methoden implementieren den Trait `serde::de::Visitor<'de>` für den flexiblen JSON-/JSONL-Parser. Sie werden durch den Serde-Deserializer dynamisch aufgerufen:
- `deserialize`
- `expecting`
- `visit_bool`
- `visit_i64`
- `visit_u64`
- `visit_f64`
- `visit_str`
- `visit_string`
- `visit_none`
- `visit_some`
- `visit_unit`
- `visit_seq`
- `visit_map`

## Fazit
Keines der identifizierten Symbole ist toter oder ungenutzter Code. Alle Methoden erfüllen aktive Verträge externer Parser-Frameworks. Keine Löschung erforderlich.
