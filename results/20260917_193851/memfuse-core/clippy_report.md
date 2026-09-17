# Diagnose-Report: `memfuse-core` (clippy)

Fehler: **15**  |  Warnungen: **0**

## `crates/memfuse-core/src/tombstone.rs`  (1 Diagnose(n))

- **ERROR [clippy::identity_op]** @ `crates/memfuse-core/src/tombstone.rs:90:14`: this operation has no effect
  - _Hinweis:_ for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#identity_op
  - _Hinweis:_ `-D clippy::identity-op` implied by `-D warnings`
  - _Hinweis:_ to override `-D warnings` add `#[allow(clippy::identity_op)]`
  - _Hinweis:_ consider reducing it to

## `crates/memfuse-core/src/tx_buffer.rs`  (4 Diagnose(n))

- **ERROR [clippy::unnecessary_cast]** @ `crates/memfuse-core/src/tx_buffer.rs:849:45`: casting to the same type is unnecessary (`u64` -> `u64`)
  - _Hinweis:_ for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unnecessary_cast
  - _Hinweis:_ `-D clippy::unnecessary-cast` implied by `-D warnings`
  - _Hinweis:_ to override `-D warnings` add `#[allow(clippy::unnecessary_cast)]`
  - _Hinweis:_ try
- **ERROR [clippy::unnecessary_cast]** @ `crates/memfuse-core/src/tx_buffer.rs:904:41`: casting to the same type is unnecessary (`u64` -> `u64`)
  - _Hinweis:_ for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unnecessary_cast
  - _Hinweis:_ try
- **ERROR [clippy::unnecessary_cast]** @ `crates/memfuse-core/src/tx_buffer.rs:926:41`: casting to the same type is unnecessary (`u64` -> `u64`)
  - _Hinweis:_ for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unnecessary_cast
  - _Hinweis:_ try
- **ERROR [clippy::unnecessary_cast]** @ `crates/memfuse-core/src/tx_buffer.rs:979:45`: casting to the same type is unnecessary (`u64` -> `u64`)
  - _Hinweis:_ for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unnecessary_cast
  - _Hinweis:_ try

## `crates/memfuse-core/src/types/domain.rs`  (10 Diagnose(n))

- **ERROR [deprecated]** @ `crates/memfuse-core/src/types/domain.rs:1186:30`: use of deprecated associated function `types::domain::TenantId::new`: Nutze TenantId::try_new() oder TenantId::SYSTEM — new() umgeht INV-TENANT-1 und wird in einer künftigen Version entfernt.
  - _Hinweis:_ `-D deprecated` implied by `-D warnings`
  - _Hinweis:_ to override `-D warnings` add `#[allow(deprecated)]`
- **ERROR [deprecated]** @ `crates/memfuse-core/src/types/domain.rs:1187:30`: use of deprecated associated function `types::domain::TenantId::new`: Nutze TenantId::try_new() oder TenantId::SYSTEM — new() umgeht INV-TENANT-1 und wird in einer künftigen Version entfernt.
- **ERROR [deprecated]** @ `crates/memfuse-core/src/types/domain.rs:1188:67`: use of deprecated associated function `types::domain::TenantId::new`: Nutze TenantId::try_new() oder TenantId::SYSTEM — new() umgeht INV-TENANT-1 und wird in einer künftigen Version entfernt.
- **ERROR [deprecated]** @ `crates/memfuse-core/src/types/domain.rs:1196:28`: use of deprecated associated constant `types::domain::TenantId::DEFAULT`: Identisch zu TenantId::SYSTEM — nutze SYSTEM für Klarheit.
- **ERROR [deprecated]** @ `crates/memfuse-core/src/types/domain.rs:1197:28`: use of deprecated associated function `types::domain::TenantId::new`: Nutze TenantId::try_new() oder TenantId::SYSTEM — new() umgeht INV-TENANT-1 und wird in einer künftigen Version entfernt.
- **ERROR [deprecated]** @ `crates/memfuse-core/src/types/domain.rs:1198:28`: use of deprecated associated function `types::domain::TenantId::new`: Nutze TenantId::try_new() oder TenantId::SYSTEM — new() umgeht INV-TENANT-1 und wird in einer künftigen Version entfernt.
- **ERROR [deprecated]** @ `crates/memfuse-core/src/types/domain.rs:1220:32`: use of deprecated associated function `types::domain::TenantId::new`: Nutze TenantId::try_new() oder TenantId::SYSTEM — new() umgeht INV-TENANT-1 und wird in einer künftigen Version entfernt.
- **ERROR [deprecated]** @ `crates/memfuse-core/src/types/domain.rs:1280:27`: use of deprecated associated function `types::domain::TenantId::new`: Nutze TenantId::try_new() oder TenantId::SYSTEM — new() umgeht INV-TENANT-1 und wird in einer künftigen Version entfernt.
- **ERROR [clippy::unnecessary_cast]** @ `crates/memfuse-core/src/types/domain.rs:1501:43`: casting to the same type is unnecessary (`u64` -> `u64`)
  - _Hinweis:_ for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unnecessary_cast
  - _Hinweis:_ try
- **ERROR [clippy::useless_conversion]** @ `crates/memfuse-core/src/types/domain.rs:1818:34`: useless conversion to the same type: `u64`
  - _Hinweis:_ for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - _Hinweis:_ `-D clippy::useless-conversion` implied by `-D warnings`
  - _Hinweis:_ to override `-D warnings` add `#[allow(clippy::useless_conversion)]`
  - _Hinweis:_ consider removing `.into()`
