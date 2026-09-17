# Gesamt-Fehlerbericht (alle Crates, gruppiert nach Datei)

Dieser Bericht ist die direkte Grundlage für die Jules-Prompt-Erstellung:
jede unten stehende Datei-Gruppe ist ein Kandidat für EINEN isolierten Jules-Prompt.
Dateien im selben Crate, die thematisch zusammenhängen (z.B. Typen aus derselben
Modul-Familie), sollten in EINEM Prompt gebündelt werden, um Merge-Konflikte zu
vermeiden — siehe Design-Prinzip 'Datei- & Crate-Isolierung'.


## Crate: `memfuse-calibration`

### `crates/memfuse-calibration/src/isotonic.rs` (2 Fehler)

Fehlercodes: `clippy::question_mark`×2

- **[clippy::question_mark]** @ `crates/memfuse-calibration/src/isotonic.rs:201:21`: this `match` expression can be replaced with `?`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#question_mark
  - Hinweis: `-D clippy::question-mark` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::question_mark)]`
  - Hinweis: try instead
- **[clippy::question_mark]** @ `crates/memfuse-calibration/src/isotonic.rs:201:21`: this `match` expression can be replaced with `?`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#question_mark
  - Hinweis: `-D clippy::question-mark` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::question_mark)]`
  - Hinweis: try instead


## Crate: `memfuse-candle`

### `crates/memfuse-calibration/src/isotonic.rs` (1 Fehler)

Fehlercodes: `clippy::question_mark`×1

- **[clippy::question_mark]** @ `crates/memfuse-calibration/src/isotonic.rs:201:21`: this `match` expression can be replaced with `?`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#question_mark
  - Hinweis: `-D clippy::question-mark` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::question_mark)]`
  - Hinweis: try instead

### `crates/memfuse-crypto/src/egress_vault.rs` (1 Fehler)

Fehlercodes: `clippy::unnecessary_sort_by`×1

- **[clippy::unnecessary_sort_by]** @ `crates/memfuse-crypto/src/egress_vault.rs:372:13`: consider using `sort_by_key`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unnecessary_sort_by
  - Hinweis: `-D clippy::unnecessary-sort-by` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::unnecessary_sort_by)]`
  - Hinweis: try


## Crate: `memfuse-core`

### `crates/memfuse-core/src/tombstone.rs` (1 Fehler)

Fehlercodes: `clippy::identity_op`×1

- **[clippy::identity_op]** @ `crates/memfuse-core/src/tombstone.rs:90:14`: this operation has no effect
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#identity_op
  - Hinweis: `-D clippy::identity-op` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::identity_op)]`
  - Hinweis: consider reducing it to

### `crates/memfuse-core/src/tx_buffer.rs` (4 Fehler)

Fehlercodes: `clippy::unnecessary_cast`×4

- **[clippy::unnecessary_cast]** @ `crates/memfuse-core/src/tx_buffer.rs:849:45`: casting to the same type is unnecessary (`u64` -> `u64`)
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unnecessary_cast
  - Hinweis: `-D clippy::unnecessary-cast` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::unnecessary_cast)]`
  - Hinweis: try
- **[clippy::unnecessary_cast]** @ `crates/memfuse-core/src/tx_buffer.rs:904:41`: casting to the same type is unnecessary (`u64` -> `u64`)
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unnecessary_cast
  - Hinweis: try
- **[clippy::unnecessary_cast]** @ `crates/memfuse-core/src/tx_buffer.rs:926:41`: casting to the same type is unnecessary (`u64` -> `u64`)
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unnecessary_cast
  - Hinweis: try
- **[clippy::unnecessary_cast]** @ `crates/memfuse-core/src/tx_buffer.rs:979:45`: casting to the same type is unnecessary (`u64` -> `u64`)
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unnecessary_cast
  - Hinweis: try

### `crates/memfuse-core/src/types/domain.rs` (10 Fehler)

Fehlercodes: `deprecated`×8, `clippy::unnecessary_cast`×1, `clippy::useless_conversion`×1

- **[deprecated]** @ `crates/memfuse-core/src/types/domain.rs:1186:30`: use of deprecated associated function `types::domain::TenantId::new`: Nutze TenantId::try_new() oder TenantId::SYSTEM — new() umgeht INV-TENANT-1 und wird in einer künftigen Version entfernt.
  - Hinweis: `-D deprecated` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(deprecated)]`
- **[deprecated]** @ `crates/memfuse-core/src/types/domain.rs:1187:30`: use of deprecated associated function `types::domain::TenantId::new`: Nutze TenantId::try_new() oder TenantId::SYSTEM — new() umgeht INV-TENANT-1 und wird in einer künftigen Version entfernt.
- **[deprecated]** @ `crates/memfuse-core/src/types/domain.rs:1188:67`: use of deprecated associated function `types::domain::TenantId::new`: Nutze TenantId::try_new() oder TenantId::SYSTEM — new() umgeht INV-TENANT-1 und wird in einer künftigen Version entfernt.
- **[deprecated]** @ `crates/memfuse-core/src/types/domain.rs:1196:28`: use of deprecated associated constant `types::domain::TenantId::DEFAULT`: Identisch zu TenantId::SYSTEM — nutze SYSTEM für Klarheit.
- **[deprecated]** @ `crates/memfuse-core/src/types/domain.rs:1197:28`: use of deprecated associated function `types::domain::TenantId::new`: Nutze TenantId::try_new() oder TenantId::SYSTEM — new() umgeht INV-TENANT-1 und wird in einer künftigen Version entfernt.
- **[deprecated]** @ `crates/memfuse-core/src/types/domain.rs:1198:28`: use of deprecated associated function `types::domain::TenantId::new`: Nutze TenantId::try_new() oder TenantId::SYSTEM — new() umgeht INV-TENANT-1 und wird in einer künftigen Version entfernt.
- **[deprecated]** @ `crates/memfuse-core/src/types/domain.rs:1220:32`: use of deprecated associated function `types::domain::TenantId::new`: Nutze TenantId::try_new() oder TenantId::SYSTEM — new() umgeht INV-TENANT-1 und wird in einer künftigen Version entfernt.
- **[deprecated]** @ `crates/memfuse-core/src/types/domain.rs:1280:27`: use of deprecated associated function `types::domain::TenantId::new`: Nutze TenantId::try_new() oder TenantId::SYSTEM — new() umgeht INV-TENANT-1 und wird in einer künftigen Version entfernt.
- **[clippy::unnecessary_cast]** @ `crates/memfuse-core/src/types/domain.rs:1501:43`: casting to the same type is unnecessary (`u64` -> `u64`)
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unnecessary_cast
  - Hinweis: try
- **[clippy::useless_conversion]** @ `crates/memfuse-core/src/types/domain.rs:1818:34`: useless conversion to the same type: `u64`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - Hinweis: `-D clippy::useless-conversion` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::useless_conversion)]`
  - Hinweis: consider removing `.into()`


## Crate: `memfuse-crypto`

### `crates/memfuse-crypto/src/egress_vault.rs` (1 Fehler)

Fehlercodes: `clippy::unnecessary_sort_by`×1

- **[clippy::unnecessary_sort_by]** @ `crates/memfuse-crypto/src/egress_vault.rs:372:13`: consider using `sort_by_key`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unnecessary_sort_by
  - Hinweis: `-D clippy::unnecessary-sort-by` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::unnecessary_sort_by)]`
  - Hinweis: try


## Crate: `memfuse-db`

### `crates/memfuse-db/tests/docid_128_consolidation.rs` (1 Fehler)

Fehlercodes: `E0432`×1

- **[E0432]** @ `crates/memfuse-db/tests/docid_128_consolidation.rs:3:17`: unresolved import `memfuse_db::volatile_vault`
  - Hinweis: found an item that was configured out


## Crate: `memfuse-embed`

### `crates/memfuse-calibration/src/isotonic.rs` (1 Fehler)

Fehlercodes: `clippy::question_mark`×1

- **[clippy::question_mark]** @ `crates/memfuse-calibration/src/isotonic.rs:201:21`: this `match` expression can be replaced with `?`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#question_mark
  - Hinweis: `-D clippy::question-mark` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::question_mark)]`
  - Hinweis: try instead


## Crate: `memfuse-graph`

### `crates/memfuse-crypto/src/egress_vault.rs` (1 Fehler)

Fehlercodes: `clippy::unnecessary_sort_by`×1

- **[clippy::unnecessary_sort_by]** @ `crates/memfuse-crypto/src/egress_vault.rs:372:13`: consider using `sort_by_key`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unnecessary_sort_by
  - Hinweis: `-D clippy::unnecessary-sort-by` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::unnecessary_sort_by)]`
  - Hinweis: try

### `crates/memfuse-graph/src/csr.rs` (1 Fehler)

Fehlercodes: `dead_code`×1

- **[dead_code]** @ `crates/memfuse-graph/src/csr.rs:352:19`: methods `hyperedges_for_entity` and `insert_hyperedge` are never used
  - Hinweis: `-D dead-code` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[expect(dead_code)]` or `#[allow(dead_code)]`

### `crates/memfuse-graph/src/session_dag.rs` (1 Fehler)

Fehlercodes: `clippy::too_many_arguments`×1

- **[clippy::too_many_arguments]** @ `crates/memfuse-graph/src/session_dag.rs:180:5`: this function has too many arguments (8/7)
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_arguments
  - Hinweis: `-D clippy::too-many-arguments` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::too_many_arguments)]`


## Crate: `memfuse-index`

### `crates/memfuse-crypto/src/egress_vault.rs` (1 Fehler)

Fehlercodes: `clippy::unnecessary_sort_by`×1

- **[clippy::unnecessary_sort_by]** @ `crates/memfuse-crypto/src/egress_vault.rs:372:13`: consider using `sort_by_key`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unnecessary_sort_by
  - Hinweis: `-D clippy::unnecessary-sort-by` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::unnecessary_sort_by)]`
  - Hinweis: try


## Crate: `memfuse-ollama`

### `crates/memfuse-calibration/src/isotonic.rs` (1 Fehler)

Fehlercodes: `clippy::question_mark`×1

- **[clippy::question_mark]** @ `crates/memfuse-calibration/src/isotonic.rs:201:21`: this `match` expression can be replaced with `?`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#question_mark
  - Hinweis: `-D clippy::question-mark` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::question_mark)]`
  - Hinweis: try instead


## Crate: `memfuse-store`

### `crates/memfuse-crypto/src/egress_vault.rs` (1 Fehler)

Fehlercodes: `clippy::unnecessary_sort_by`×1

- **[clippy::unnecessary_sort_by]** @ `crates/memfuse-crypto/src/egress_vault.rs:372:13`: consider using `sort_by_key`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unnecessary_sort_by
  - Hinweis: `-D clippy::unnecessary-sort-by` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::unnecessary_sort_by)]`
  - Hinweis: try

