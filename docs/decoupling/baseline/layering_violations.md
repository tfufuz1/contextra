LAYER-VERSTÖSSE (34):
  - [contextra] Kern-Closure Verstoß: unerlaubte interne Abhängigkeiten ['contextra-checkpoint', 'contextra-crypto', 'contextra-engine', 'contextra-env', 'contextra-graph', 'contextra-ids', 'contextra-mvcc', 'contextra-rank', 'contextra-store', 'contextra-sys', 'contextra-text', 'contextra-vector'] (erlaubt: ['contextra-db', 'contextra-ports', 'contextra-types'])
  - [contextra-agent] Kern-Closure Verstoß: unerlaubte interne Abhängigkeiten ['contextra-adapt', 'contextra-checkpoint', 'contextra-crypto', 'contextra-db', 'contextra-engine', 'contextra-env', 'contextra-graph', 'contextra-ids', 'contextra-mvcc', 'contextra-rank', 'contextra-router', 'contextra-store', 'contextra-sys', 'contextra-text', 'contextra-vector', 'contextra-wire'] (erlaubt: ['contextra-ports', 'contextra-types'])
  - [contextra-agent] Dev-Abhängigkeit auf 'contextra-testkit' unzulässig (nicht in dev_allowed)
  - [contextra-bench] Kern-Closure Verstoß: unerlaubte interne Abhängigkeiten ['contextra-checkpoint', 'contextra-crypto', 'contextra-engine', 'contextra-env', 'contextra-graph', 'contextra-ids', 'contextra-mvcc', 'contextra-ports', 'contextra-rank', 'contextra-sys', 'contextra-types'] (erlaubt: ['contextra-db', 'contextra-store', 'contextra-text', 'contextra-vector'])
  - [contextra-checkpoint] Kern-Closure Verstoß: unerlaubte interne Abhängigkeiten ['contextra-ports', 'contextra-types'] (erlaubt: ['contextra-env', 'contextra-ids'])
  - [contextra-checkpoint] Ebenen-Verstoß (INV-8): Ebene 1 (contextra-checkpoint) hängt ab von höherer Ebene 2 (contextra-types)
  - [contextra-checkpoint] Ebenen-Verstoß (INV-8): Ebene 1 (contextra-checkpoint) hängt ab von höherer Ebene 2 (contextra-ports)
  - [contextra-checkpoint] Dev-Abhängigkeit auf 'contextra-testkit' unzulässig (nicht in dev_allowed)
  - [contextra-cognition] Kern-Closure Verstoß: unerlaubte interne Abhängigkeiten ['contextra-checkpoint', 'contextra-crypto', 'contextra-engine', 'contextra-env', 'contextra-graph', 'contextra-ids', 'contextra-mvcc', 'contextra-rank', 'contextra-store', 'contextra-sys', 'contextra-text', 'contextra-vector'] (erlaubt: ['contextra-ports', 'contextra-types'])
  - [contextra-cognition] Dev-Abhängigkeit auf 'contextra-testkit' unzulässig (nicht in dev_allowed)
  - [contextra-crypto] Kern-Closure Verstoß: unerlaubte interne Abhängigkeiten ['contextra-ports', 'contextra-types'] (erlaubt: ['contextra-env', 'contextra-ids'])
  - [contextra-crypto] Ebenen-Verstoß (INV-8): Ebene 1 (contextra-crypto) hängt ab von höherer Ebene 2 (contextra-types)
  - [contextra-crypto] Ebenen-Verstoß (INV-8): Ebene 1 (contextra-crypto) hängt ab von höherer Ebene 2 (contextra-ports)
  - [contextra-crypto] Dev-Abhängigkeit auf 'contextra-testkit' unzulässig (nicht in dev_allowed)
  - [contextra-db] Kern-Closure Verstoß: unerlaubte interne Abhängigkeiten ['contextra-checkpoint', 'contextra-crypto', 'contextra-env', 'contextra-graph', 'contextra-ids', 'contextra-mvcc', 'contextra-rank', 'contextra-store', 'contextra-sys', 'contextra-text', 'contextra-vector'] (erlaubt: ['contextra-engine', 'contextra-ports', 'contextra-types'])
  - [contextra-db] Dev-Abhängigkeit auf 'contextra-testkit' unzulässig (nicht in dev_allowed)
  - [contextra-engine] Kern-Closure Verstoß: unerlaubte interne Abhängigkeiten ['contextra-crypto', 'contextra-env', 'contextra-ids', 'contextra-sys'] (erlaubt: ['contextra-adapt', 'contextra-checkpoint', 'contextra-graph', 'contextra-kvcache', 'contextra-mvcc', 'contextra-ports', 'contextra-rank', 'contextra-store', 'contextra-text', 'contextra-types', 'contextra-vector'])
  - [contextra-engine] Dev-Abhängigkeit auf 'contextra-testkit' unzulässig (nicht in dev_allowed)
  - [contextra-infer-candle] Kern-Closure Verstoß: unerlaubte interne Abhängigkeiten ['contextra-ids', 'contextra-ports', 'contextra-rank', 'contextra-types'] (erlaubt: ['contextra-env'])
  - [contextra-infer-candle] Ebenen-Verstoß (INV-8): Ebene 1 (contextra-infer-candle) hängt ab von höherer Ebene 2 (contextra-ports)
  - [contextra-infer-candle] Ebenen-Verstoß (INV-8): Ebene 1 (contextra-infer-candle) hängt ab von höherer Ebene 2 (contextra-types)
  - [contextra-infer-candle] Feature-Closure Verstoß bei Feature 'kv-bridge': unerlaubte interne Abhängigkeiten ['contextra-crypto', 'contextra-ids', 'contextra-ports', 'contextra-rank', 'contextra-types'] (erlaubt: ['contextra-env', 'contextra-kvcache'])
  - [contextra-mcp] Kern-Closure Verstoß: unerlaubte interne Abhängigkeiten ['contextra-adapt', 'contextra-checkpoint', 'contextra-db', 'contextra-engine', 'contextra-env', 'contextra-graph', 'contextra-ids', 'contextra-mvcc', 'contextra-store', 'contextra-sys', 'contextra-text', 'contextra-vector'] (erlaubt: ['contextra', 'contextra-crypto', 'contextra-ports', 'contextra-privacy', 'contextra-prompt-guard', 'contextra-rank', 'contextra-router', 'contextra-types', 'contextra-wire'])
  - [contextra-mvcc] Kern-Closure Verstoß: unerlaubte interne Abhängigkeiten ['contextra-ports', 'contextra-types'] (erlaubt: ['contextra-env', 'contextra-ids'])
  - [contextra-mvcc] Ebenen-Verstoß (INV-8): Ebene 1 (contextra-mvcc) hängt ab von höherer Ebene 2 (contextra-types)
  - [contextra-mvcc] Ebenen-Verstoß (INV-8): Ebene 1 (contextra-mvcc) hängt ab von höherer Ebene 2 (contextra-ports)
  - [contextra-mvcc] Dev-Abhängigkeit auf 'contextra-testkit' unzulässig (nicht in dev_allowed)
  - [contextra-py] Kern-Closure Verstoß: unerlaubte interne Abhängigkeiten ['contextra-checkpoint', 'contextra-crypto', 'contextra-db', 'contextra-engine', 'contextra-env', 'contextra-graph', 'contextra-ids', 'contextra-mvcc', 'contextra-ports', 'contextra-rank', 'contextra-store', 'contextra-sys', 'contextra-text', 'contextra-vector'] (erlaubt: ['contextra', 'contextra-types', 'contextra-wire'])
  - [contextra-router] Kern-Closure Verstoß: unerlaubte interne Abhängigkeiten ['contextra-env', 'contextra-ids', 'contextra-wire'] (erlaubt: ['contextra-adapt', 'contextra-ports', 'contextra-types'])
  - [contextra-store] Kern-Closure Verstoß: unerlaubte interne Abhängigkeiten ['contextra-ports', 'contextra-types'] (erlaubt: ['contextra-crypto', 'contextra-env', 'contextra-ids', 'contextra-mvcc', 'contextra-sys'])
  - [contextra-store] Ebenen-Verstoß (INV-8): Ebene 1 (contextra-store) hängt ab von höherer Ebene 2 (contextra-ports)
  - [contextra-store] Ebenen-Verstoß (INV-8): Ebene 1 (contextra-store) hängt ab von höherer Ebene 2 (contextra-types)
  - [contextra-store] Dev-Abhängigkeit auf 'contextra-testkit' unzulässig (nicht in dev_allowed)
  - [contextra-testkit] Kern-Closure Verstoß: unerlaubte interne Abhängigkeiten ['contextra-ids'] (erlaubt: ['contextra-env', 'contextra-ports', 'contextra-types'])
WARNUNGEN (27):
  - [contextra] Feature-Baum für 'license' konnte nicht ermittelt werden: cargo tree failed for contextra features=['license']: error: the package 'contextra' does not contain this feature: license

  - [contextra] Feature-Baum für 'infer-candle' konnte nicht ermittelt werden: cargo tree failed for contextra features=['infer-candle']: error: the package 'contextra' does not contain this feature: infer-candle

  - [contextra-adapt] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-adapt features=['workspace']: error: the package 'contextra-adapt' does not contain this feature: workspace

  - [contextra-agent] Feature-Baum für 'storage-bindings' konnte nicht ermittelt werden: cargo tree failed for contextra-agent features=['storage-bindings']: error: the package 'contextra-agent' does not contain this feature: storage-bindings

  - [contextra-audit-export] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-audit-export features=['workspace']: error: the package 'contextra-audit-export' does not contain this feature: workspace

  - [contextra-avv-generator] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-avv-generator features=['workspace']: error: the package 'contextra-avv-generator' does not contain this feature: workspace

  - [contextra-checkpoint] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-checkpoint features=['workspace']: error: the package 'contextra-checkpoint' does not contain this feature: workspace

  - [contextra-cognition] Feature-Baum für 'exp-hyperedge' konnte nicht ermittelt werden: cargo tree failed for contextra-cognition features=['exp-hyperedge']: error: the package 'contextra-cognition' does not contain this feature: exp-hyperedge

  - [contextra-crypto] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-crypto features=['workspace']: error: the package 'contextra-crypto' does not contain this feature: workspace

  - [contextra-erasure] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-erasure features=['workspace']: error: the package 'contextra-erasure' does not contain this feature: workspace

  - [contextra-graph] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-graph features=['workspace']: error: the package 'contextra-graph' does not contain this feature: workspace

  - [contextra-infer-candle] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-infer-candle features=['workspace']: error: the package 'contextra-infer-candle' does not contain this feature: workspace

  - [contextra-infer-ollama] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-infer-ollama features=['workspace']: error: the package 'contextra-infer-ollama' does not contain this feature: workspace

  - [contextra-infer-onnx] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-infer-onnx features=['workspace']: error: the package 'contextra-infer-onnx' does not contain this feature: workspace

  - [contextra-ingest] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-ingest features=['workspace']: error: the package 'contextra-ingest' does not contain this feature: workspace

  - [contextra-kvcache] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-kvcache features=['workspace']: error: the package 'contextra-kvcache' does not contain this feature: workspace

  - [contextra-license] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-license features=['workspace']: error: the package 'contextra-license' does not contain this feature: workspace

  - [contextra-mvcc] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-mvcc features=['workspace']: error: the package 'contextra-mvcc' does not contain this feature: workspace

  - [contextra-privacy] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-privacy features=['workspace']: error: the package 'contextra-privacy' does not contain this feature: workspace

  - [contextra-rank] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-rank features=['workspace']: error: the package 'contextra-rank' does not contain this feature: workspace

  - [contextra-router] Feature-Baum für 'subprocess-dispatch' konnte nicht ermittelt werden: cargo tree failed for contextra-router features=['subprocess-dispatch']: error: the package 'contextra-router' does not contain this feature: subprocess-dispatch

  - [contextra-simd] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-simd features=['workspace']: error: the package 'contextra-simd' does not contain this feature: workspace

  - [contextra-store] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-store features=['workspace']: error: the package 'contextra-store' does not contain this feature: workspace

  - [contextra-text] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-text features=['workspace']: error: the package 'contextra-text' does not contain this feature: workspace

  - [contextra-vault] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-vault features=['workspace']: error: the package 'contextra-vault' does not contain this feature: workspace

  - [contextra-vector] Feature-Baum für 'simd' konnte nicht ermittelt werden: cargo tree failed for contextra-vector features=['simd']: error: the package 'contextra-vector' does not contain this feature: simd
help: there are similarly named features: serde, std

  - [contextra-vector] Feature-Baum für 'workspace' konnte nicht ermittelt werden: cargo tree failed for contextra-vector features=['workspace']: error: the package 'contextra-vector' does not contain this feature: workspace
