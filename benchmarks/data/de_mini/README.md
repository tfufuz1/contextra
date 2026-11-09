# Deutscher Mini-Evaluationskorpus (`de_mini`)

- **Herkunft:** Eigens synthetisierter, fachbezogener Korpus für Information Retrieval und Hybrid-Retrieval-Evaluation im Contextra-Projekt.
- **Lizenz:** CC0 1.0 Universal (Public Domain Dedication) / MIT.
- **Autor:** Contextra Projektteam (2026).
- **Umfang:**
  - `corpus.jsonl`: 200 Dokumente aus 5 Fachthemenbereichen (Datenbankarchitektur, Vektorsuche, Textretrieval, Rangfusion, Graphstrukturen).
  - `queries.jsonl`: 50 natürlichsprachige Testabfragen (2 pro Unterthema).
  - `qrels.tsv`: 400 Relevanzurteile mit abgestuften Relevanz-Scores (2 = hochrelevant, 1 = partiell relevant).
- **Format:**
  - `corpus.jsonl`: `{"id": "doc_xxx", "title": "...", "text": "...", "domain": "...", "topic": "..."}`
  - `queries.jsonl`: `{"id": "q_xx", "text": "...", "domain": "..."}`
  - `qrels.tsv`: `query-id \t corpus-id \t score` (BEIR-kompatibel)
