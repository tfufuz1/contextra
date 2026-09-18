# Gesamt-Fehlerbericht (gruppiert nach Crate -> Datei)

Direkte Grundlage für Jules-Prompts: jede Datei-Gruppe = ein Kandidat für einen
isolierten, parallel ausführbaren Prompt.


## Crate: `memfuse-agent`

### `crates/memfuse-index/src/distance.rs` (72 Fehler)

Fehlercodes: `unsafe_code`×51, `E0453`×21

- **[E0453]** @ `crates/memfuse-index/src/distance.rs:61:10`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:120:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:132:23`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:136:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:150:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:162:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:166:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:180:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:192:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:196:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:285:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:298:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:303:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:322:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:335:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:340:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:359:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:372:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:377:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:521:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:524:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:539:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:550:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:578:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:581:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:591:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:600:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:616:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:619:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:629:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:637:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:695:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:698:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:707:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:716:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:728:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:731:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:746:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:757:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:786:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:789:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:799:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:808:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:825:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:828:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:838:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:846:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:862:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:865:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:879:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:891:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:922:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:925:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:934:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:944:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:958:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:961:1`: declaration of an `unsafe` function
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:977:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:980:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:989:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:998:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1009:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1012:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1026:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1038:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1068:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1071:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1080:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1090:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1103:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1106:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1110:5`: usage of an `unsafe` block

### `crates/memfuse-index/src/hnsw.rs` (22 Fehler)

Fehlercodes: `unsafe_code`×12, `E0453`×10

- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1469:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1475:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1479:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1493:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1499:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1503:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1568:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1570:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1581:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1582:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1592:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1595:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1624:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1628:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1679:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1682:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1713:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1716:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1743:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1747:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1797:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1800:5`: implementation of an `unsafe` method

### `crates/memfuse-index/src/persistence.rs` (2 Fehler)

Fehlercodes: `E0453`×1, `unsafe_code`×1

- **[E0453]** @ `crates/memfuse-index/src/persistence.rs:451:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/persistence.rs:462:20`: usage of an `unsafe` block

### `crates/memfuse-store/src/wal/replay.rs` (4 Fehler)

Fehlercodes: `E0453`×2, `unsafe_code`×2

- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block


## Crate: `memfuse-bench`

### `crates/memfuse-index/src/distance.rs` (72 Fehler)

Fehlercodes: `unsafe_code`×51, `E0453`×21

- **[E0453]** @ `crates/memfuse-index/src/distance.rs:61:10`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:120:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:132:23`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:136:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:150:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:162:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:166:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:180:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:192:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:196:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:285:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:298:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:303:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:322:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:335:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:340:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:359:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:372:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:377:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:521:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:524:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:539:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:550:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:578:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:581:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:591:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:600:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:616:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:619:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:629:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:637:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:695:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:698:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:707:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:716:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:728:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:731:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:746:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:757:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:786:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:789:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:799:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:808:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:825:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:828:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:838:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:846:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:862:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:865:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:879:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:891:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:922:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:925:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:934:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:944:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:958:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:961:1`: declaration of an `unsafe` function
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:977:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:980:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:989:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:998:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1009:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1012:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1026:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1038:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1068:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1071:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1080:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1090:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1103:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1106:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1110:5`: usage of an `unsafe` block

### `crates/memfuse-index/src/hnsw.rs` (22 Fehler)

Fehlercodes: `unsafe_code`×12, `E0453`×10

- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1469:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1475:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1479:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1493:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1499:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1503:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1568:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1570:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1581:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1582:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1592:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1595:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1624:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1628:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1679:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1682:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1713:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1716:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1743:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1747:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1797:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1800:5`: implementation of an `unsafe` method

### `crates/memfuse-index/src/persistence.rs` (2 Fehler)

Fehlercodes: `E0453`×1, `unsafe_code`×1

- **[E0453]** @ `crates/memfuse-index/src/persistence.rs:451:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/persistence.rs:462:20`: usage of an `unsafe` block

### `crates/memfuse-store/src/wal/replay.rs` (4 Fehler)

Fehlercodes: `E0453`×2, `unsafe_code`×2

- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block


## Crate: `memfuse-candle[kv-bridge]`

### `crates/memfuse-store/src/wal/replay.rs` (4 Fehler)

Fehlercodes: `E0453`×2, `unsafe_code`×2

- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block


## Crate: `memfuse-crypto`

### `crates/memfuse-crypto/src/anti_tamper.rs` (2 Fehler)

Fehlercodes: `unsafe_code`×2

- **[unsafe_code]** @ `crates/memfuse-crypto/src/anti_tamper.rs:126:9`: usage of an `unsafe` block
  - Hinweis: `-D unsafe-code` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(unsafe_code)]`
- **[unsafe_code]** @ `crates/memfuse-crypto/src/anti_tamper.rs:136:9`: usage of an `unsafe` block

### `crates/memfuse-crypto/src/kv_segment/segment.rs` (2 Fehler)

Fehlercodes: `unsafe_code`×2

- **[unsafe_code]** @ `crates/memfuse-crypto/src/kv_segment/segment.rs:216:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-crypto/src/kv_segment/segment.rs:226:9`: usage of an `unsafe` block

### `crates/memfuse-crypto/tests/kv_segment_proptests.rs` (1 Fehler)

Fehlercodes: `unsafe_code`×1

- **[unsafe_code]** @ `crates/memfuse-crypto/tests/kv_segment_proptests.rs:88:9`: usage of an `unsafe` block
  - Hinweis: `-D unsafe-code` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(unsafe_code)]`


## Crate: `memfuse-db`

### `crates/memfuse-index/src/diskann.rs` (2 Fehler)

Fehlercodes: `E0453`×1, `unsafe_code`×1

- **[E0453]** @ `crates/memfuse-index/src/diskann.rs:1562:25`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/diskann.rs:1563:28`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`

### `crates/memfuse-index/src/distance.rs` (72 Fehler)

Fehlercodes: `unsafe_code`×51, `E0453`×21

- **[E0453]** @ `crates/memfuse-index/src/distance.rs:61:10`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:120:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:132:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:136:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:150:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:162:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:166:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:180:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:192:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:196:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:285:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:298:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:303:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:322:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:335:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:340:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:359:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:372:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:377:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:521:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:524:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:539:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:550:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:578:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:581:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:591:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:600:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:616:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:619:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:629:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:637:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:695:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:698:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:707:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:716:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:728:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:731:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:746:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:757:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:786:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:789:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:799:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:808:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:825:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:828:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:838:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:846:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:862:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:865:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:879:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:891:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:922:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:925:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:934:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:944:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:958:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:961:1`: declaration of an `unsafe` function
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:977:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:980:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:989:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:998:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1009:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1012:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1026:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1038:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1068:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1071:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1080:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1090:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1103:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1106:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1110:5`: usage of an `unsafe` block

### `crates/memfuse-index/src/hnsw.rs` (22 Fehler)

Fehlercodes: `unsafe_code`×12, `E0453`×10

- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1469:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1475:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1479:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1493:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1499:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1503:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1568:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1570:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1581:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1582:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1592:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1595:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1624:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1628:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1679:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1682:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1713:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1716:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1743:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1747:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1797:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1800:5`: implementation of an `unsafe` method

### `crates/memfuse-index/src/persistence.rs` (2 Fehler)

Fehlercodes: `E0453`×1, `unsafe_code`×1

- **[E0453]** @ `crates/memfuse-index/src/persistence.rs:451:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/persistence.rs:462:20`: usage of an `unsafe` block

### `crates/memfuse-store/src/wal/replay.rs` (4 Fehler)

Fehlercodes: `E0453`×2, `unsafe_code`×2

- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block


## Crate: `memfuse-graph`

### `crates/memfuse-store/src/wal/replay.rs` (4 Fehler)

Fehlercodes: `E0453`×2, `unsafe_code`×2

- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block


## Crate: `memfuse-graph[edge-reinforcement-learning]`

### `crates/memfuse-store/src/wal/replay.rs` (4 Fehler)

Fehlercodes: `E0453`×2, `unsafe_code`×2

- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block


## Crate: `memfuse-index`

### `crates/memfuse-index/src/distance.rs` (193 Fehler)

Fehlercodes: `unsafe_code`×138, `E0453`×55

- **[E0453]** @ `crates/memfuse-index/src/distance.rs:61:10`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:61:10`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:120:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:120:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:132:23`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:132:23`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:136:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:136:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:150:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:150:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:162:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:162:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:166:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:166:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:180:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:180:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:192:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:192:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:196:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:196:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:285:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:285:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:298:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:298:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:303:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:303:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:322:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:322:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:335:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:335:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:340:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:340:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:359:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:359:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:372:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:372:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:377:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:377:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:521:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:521:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:524:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:524:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:539:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:539:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:550:45`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:550:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:578:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:578:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:581:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:581:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:591:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:591:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:600:19`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:600:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:616:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:616:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:619:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:619:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:629:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:629:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:637:19`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:637:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:695:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:695:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:698:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:698:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:707:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:707:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:716:19`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:716:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:728:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:728:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:731:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:731:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:746:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:746:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:757:45`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:757:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:786:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:786:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:789:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:789:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:799:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:799:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:808:19`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:808:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:825:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:825:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:828:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:828:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:838:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:838:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:846:19`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:846:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:862:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:862:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:865:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:865:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:879:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:879:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:891:45`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:891:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:922:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:922:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:925:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:925:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:934:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:934:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:944:19`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:944:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:958:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:958:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:961:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:961:1`: declaration of an `unsafe` function
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:977:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:977:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:980:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:980:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:989:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:989:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:998:19`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:998:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1009:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1009:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1012:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1012:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1026:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1026:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1038:45`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1038:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1068:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1068:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1071:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1071:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1080:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1080:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1090:19`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1090:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1103:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1103:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1106:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1106:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1110:5`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1110:5`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1133:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1147:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1153:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1170:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1184:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1190:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1218:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1238:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1244:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1335:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1341:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1351:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1363:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1374:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1380:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1389:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1413:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1426:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1432:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1449:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1464:51`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1491:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1495:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1498:5`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1513:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1519:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1534:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1554:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1564:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1570:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1585:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1607:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1618:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1624:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1647:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1672:51`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1699:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1703:1`: declaration of an `unsafe` function
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1789:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1808:32`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1815:32`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1828:32`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1841:34`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:2104:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:2138:30`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:2150:32`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:2270:34`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:2293:36`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:2302:36`: usage of an `unsafe` block

### `crates/memfuse-index/src/hnsw.rs` (44 Fehler)

Fehlercodes: `unsafe_code`×24, `E0453`×20

- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1469:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1469:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1475:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1475:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1479:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1479:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1493:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1493:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1499:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1499:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1503:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1503:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1568:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1568:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1570:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1570:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1581:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1581:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1582:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1582:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1592:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1592:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1595:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1595:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1624:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1624:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1628:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1628:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1679:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1679:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1682:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1682:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1713:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1713:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1716:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1716:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1743:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1743:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1747:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1747:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1797:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1797:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1800:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1800:5`: implementation of an `unsafe` method

### `crates/memfuse-index/src/persistence.rs` (4 Fehler)

Fehlercodes: `E0453`×2, `unsafe_code`×2

- **[E0453]** @ `crates/memfuse-index/src/persistence.rs:451:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/persistence.rs:451:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/persistence.rs:462:20`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/persistence.rs:462:20`: usage of an `unsafe` block


## Crate: `memfuse-index[experimental-diskann]`

### `crates/memfuse-index/src/diskann.rs` (4 Fehler)

Fehlercodes: `E0453`×2, `unsafe_code`×2

- **[E0453]** @ `crates/memfuse-index/src/diskann.rs:1562:25`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/diskann.rs:1562:25`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/diskann.rs:1563:28`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[unsafe_code]** @ `crates/memfuse-index/src/diskann.rs:1563:28`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`

### `crates/memfuse-index/src/distance.rs` (193 Fehler)

Fehlercodes: `unsafe_code`×138, `E0453`×55

- **[E0453]** @ `crates/memfuse-index/src/distance.rs:61:10`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:61:10`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:120:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:120:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:132:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:132:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:136:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:136:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:150:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:150:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:162:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:162:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:166:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:166:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:180:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:180:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:192:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:192:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:196:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:196:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:285:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:285:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:298:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:298:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:303:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:303:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:322:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:322:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:335:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:335:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:340:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:340:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:359:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:359:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:372:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:372:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:377:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:377:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:521:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:521:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:524:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:524:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:539:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:539:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:550:45`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:550:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:578:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:578:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:581:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:581:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:591:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:591:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:600:19`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:600:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:616:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:616:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:619:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:619:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:629:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:629:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:637:19`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:637:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:695:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:695:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:698:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:698:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:707:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:707:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:716:19`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:716:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:728:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:728:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:731:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:731:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:746:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:746:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:757:45`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:757:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:786:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:786:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:789:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:789:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:799:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:799:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:808:19`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:808:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:825:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:825:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:828:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:828:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:838:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:838:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:846:19`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:846:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:862:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:862:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:865:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:865:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:879:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:879:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:891:45`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:891:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:922:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:922:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:925:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:925:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:934:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:934:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:944:19`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:944:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:958:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:958:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:961:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:961:1`: declaration of an `unsafe` function
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:977:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:977:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:980:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:980:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:989:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:989:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:998:19`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:998:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1009:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1009:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1012:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1012:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1026:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1026:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1038:45`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1038:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1068:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1068:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1071:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1071:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1080:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1080:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1090:19`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1090:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1103:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1103:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1106:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1106:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1110:5`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1110:5`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1133:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1147:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1153:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1170:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1184:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1190:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1218:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1238:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1244:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1335:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1341:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1351:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1363:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1374:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1380:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1389:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1413:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1426:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1432:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1449:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1464:51`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1491:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1495:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1498:5`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1513:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1519:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1534:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1554:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1564:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1570:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1585:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1607:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1618:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1624:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1647:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1672:51`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1699:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1703:1`: declaration of an `unsafe` function
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1789:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1808:32`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1815:32`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1828:32`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1841:34`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:2104:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:2138:30`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:2150:32`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:2270:34`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:2293:36`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:2302:36`: usage of an `unsafe` block

### `crates/memfuse-index/src/hnsw.rs` (44 Fehler)

Fehlercodes: `unsafe_code`×24, `E0453`×20

- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1469:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1469:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1475:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1475:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1479:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1479:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1493:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1493:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1499:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1499:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1503:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1503:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1568:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1568:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1570:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1570:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1581:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1581:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1582:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1582:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1592:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1592:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1595:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1595:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1624:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1624:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1628:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1628:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1679:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1679:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1682:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1682:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1713:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1713:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1716:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1716:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1743:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1743:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1747:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1747:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1797:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1797:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1800:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1800:5`: implementation of an `unsafe` method

### `crates/memfuse-index/src/persistence.rs` (4 Fehler)

Fehlercodes: `E0453`×2, `unsafe_code`×2

- **[E0453]** @ `crates/memfuse-index/src/persistence.rs:451:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/persistence.rs:451:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/persistence.rs:462:20`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/persistence.rs:462:20`: usage of an `unsafe` block


## Crate: `memfuse-mcp`

### `crates/memfuse-index/src/distance.rs` (72 Fehler)

Fehlercodes: `unsafe_code`×51, `E0453`×21

- **[E0453]** @ `crates/memfuse-index/src/distance.rs:61:10`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:120:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:132:23`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:136:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:150:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:162:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:166:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:180:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:192:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:196:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:285:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:298:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:303:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:322:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:335:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:340:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:359:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:372:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:377:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:521:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:524:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:539:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:550:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:578:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:581:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:591:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:600:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:616:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:619:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:629:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:637:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:695:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:698:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:707:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:716:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:728:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:731:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:746:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:757:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:786:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:789:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:799:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:808:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:825:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:828:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:838:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:846:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:862:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:865:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:879:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:891:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:922:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:925:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:934:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:944:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:958:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:961:1`: declaration of an `unsafe` function
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:977:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:980:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:989:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:998:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1009:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1012:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1026:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1038:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1068:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1071:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1080:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1090:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1103:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1106:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1110:5`: usage of an `unsafe` block

### `crates/memfuse-index/src/hnsw.rs` (22 Fehler)

Fehlercodes: `unsafe_code`×12, `E0453`×10

- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1469:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1475:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1479:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1493:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1499:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1503:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1568:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1570:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1581:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1582:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1592:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1595:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1624:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1628:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1679:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1682:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1713:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1716:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1743:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1747:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1797:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1800:5`: implementation of an `unsafe` method

### `crates/memfuse-index/src/persistence.rs` (2 Fehler)

Fehlercodes: `E0453`×1, `unsafe_code`×1

- **[E0453]** @ `crates/memfuse-index/src/persistence.rs:451:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/persistence.rs:462:20`: usage of an `unsafe` block

### `crates/memfuse-store/src/wal/replay.rs` (4 Fehler)

Fehlercodes: `E0453`×2, `unsafe_code`×2

- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block


## Crate: `memfuse-router`

### `crates/memfuse-index/src/distance.rs` (72 Fehler)

Fehlercodes: `unsafe_code`×51, `E0453`×21

- **[E0453]** @ `crates/memfuse-index/src/distance.rs:61:10`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:120:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:132:23`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:136:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:150:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:162:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:166:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:180:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:192:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:196:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:285:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:298:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:303:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:322:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:335:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:340:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:359:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:372:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:377:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:521:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:524:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:539:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:550:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:578:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:581:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:591:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:600:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:616:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:619:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:629:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:637:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:695:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:698:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:707:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:716:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:728:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:731:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:746:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:757:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:786:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:789:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:799:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:808:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:825:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:828:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:838:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:846:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:862:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:865:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:879:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:891:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:922:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:925:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:934:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:944:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:958:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:961:1`: declaration of an `unsafe` function
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:977:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:980:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:989:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:998:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1009:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1012:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1026:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1038:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1068:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1071:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1080:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1090:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1103:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1106:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1110:5`: usage of an `unsafe` block

### `crates/memfuse-index/src/hnsw.rs` (22 Fehler)

Fehlercodes: `unsafe_code`×12, `E0453`×10

- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1469:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1475:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1479:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1493:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1499:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1503:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1568:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1570:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1581:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1582:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1592:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1595:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1624:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1628:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1679:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1682:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1713:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1716:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1743:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1747:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1797:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1800:5`: implementation of an `unsafe` method

### `crates/memfuse-index/src/persistence.rs` (2 Fehler)

Fehlercodes: `E0453`×1, `unsafe_code`×1

- **[E0453]** @ `crates/memfuse-index/src/persistence.rs:451:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/persistence.rs:462:20`: usage of an `unsafe` block

### `crates/memfuse-store/src/wal/replay.rs` (4 Fehler)

Fehlercodes: `E0453`×2, `unsafe_code`×2

- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block


## Crate: `memfuse-router[bandit-routing]`

### `crates/memfuse-index/src/distance.rs` (72 Fehler)

Fehlercodes: `unsafe_code`×51, `E0453`×21

- **[E0453]** @ `crates/memfuse-index/src/distance.rs:61:10`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:120:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:132:23`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:136:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:150:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:162:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:166:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:180:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:192:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:196:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:285:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:298:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:303:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:322:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:335:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:340:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:359:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:372:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:377:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:521:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:524:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:539:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:550:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:578:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:581:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:591:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:600:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:616:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:619:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:629:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:637:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:695:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:698:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:707:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:716:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:728:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:731:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:746:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:757:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:786:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:789:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:799:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:808:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:825:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:828:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:838:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:846:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:862:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:865:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:879:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:891:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:922:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:925:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:934:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:944:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:958:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:961:1`: declaration of an `unsafe` function
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:977:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:980:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:989:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:998:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1009:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1012:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1026:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1038:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1068:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1071:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1080:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1090:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1103:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1106:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1110:5`: usage of an `unsafe` block

### `crates/memfuse-index/src/hnsw.rs` (22 Fehler)

Fehlercodes: `unsafe_code`×12, `E0453`×10

- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1469:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1475:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1479:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1493:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1499:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1503:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1568:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1570:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1581:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1582:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1592:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1595:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1624:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1628:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1679:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1682:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1713:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1716:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1743:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1747:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1797:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1800:5`: implementation of an `unsafe` method

### `crates/memfuse-index/src/persistence.rs` (2 Fehler)

Fehlercodes: `E0453`×1, `unsafe_code`×1

- **[E0453]** @ `crates/memfuse-index/src/persistence.rs:451:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/persistence.rs:462:20`: usage of an `unsafe` block

### `crates/memfuse-store/src/wal/replay.rs` (4 Fehler)

Fehlercodes: `E0453`×2, `unsafe_code`×2

- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block


## Crate: `memfuse-router[egress-sherman-morrison]`

### `crates/memfuse-index/src/distance.rs` (72 Fehler)

Fehlercodes: `unsafe_code`×51, `E0453`×21

- **[E0453]** @ `crates/memfuse-index/src/distance.rs:61:10`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:120:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:132:23`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:136:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:150:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:162:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:166:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:180:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:192:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:196:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:285:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:298:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:303:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:322:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:335:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:340:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:359:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:372:23`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:377:23`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:521:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:524:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:539:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:550:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:578:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:581:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:591:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:600:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:616:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:619:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:629:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:637:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:695:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:698:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:707:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:716:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:728:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:731:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:746:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:757:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:786:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:789:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:799:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:808:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:825:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:828:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:838:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:846:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:862:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:865:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:879:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:891:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:922:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:925:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:934:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:944:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:958:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:961:1`: declaration of an `unsafe` function
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:977:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:980:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:989:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:998:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1009:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1012:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1026:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1038:45`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1068:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1071:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1080:9`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1090:19`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/distance.rs:1103:9`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1106:1`: declaration of an `unsafe` function
- **[unsafe_code]** @ `crates/memfuse-index/src/distance.rs:1110:5`: usage of an `unsafe` block

### `crates/memfuse-index/src/hnsw.rs` (22 Fehler)

Fehlercodes: `unsafe_code`×12, `E0453`×10

- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1469:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1475:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1479:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1493:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1499:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1503:24`: usage of an `unsafe` block
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1568:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1570:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1581:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1582:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1592:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1595:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1624:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1628:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1679:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1682:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1713:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1716:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1743:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1747:5`: implementation of an `unsafe` method
- **[E0453]** @ `crates/memfuse-index/src/hnsw.rs:1797:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/hnsw.rs:1800:5`: implementation of an `unsafe` method

### `crates/memfuse-index/src/persistence.rs` (2 Fehler)

Fehlercodes: `E0453`×1, `unsafe_code`×1

- **[E0453]** @ `crates/memfuse-index/src/persistence.rs:451:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-index/src/persistence.rs:462:20`: usage of an `unsafe` block

### `crates/memfuse-store/src/wal/replay.rs` (4 Fehler)

Fehlercodes: `E0453`×2, `unsafe_code`×2

- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block


## Crate: `memfuse-store`

### `crates/memfuse-store/src/wal/replay.rs` (8 Fehler)

Fehlercodes: `E0453`×4, `unsafe_code`×4

- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block


## Crate: `memfuse-store[block-cache-v2]`

### `crates/memfuse-store/src/sstable.rs` (2 Fehler)

Fehlercodes: `E0053`×2

- **[E0053]** @ `crates/memfuse-store/src/sstable.rs:182:59`: method `weight` has an incompatible type for trait
  - Hinweis: expected signature `fn(&BlockWeighter, &(_, _), &bytes::Bytes) -> u64`    found signature `fn(&BlockWeighter, &(_, _), &bytes::Bytes) -> u32`
  - Hinweis: change the output type to match the trait
- **[E0053]** @ `crates/memfuse-store/src/sstable.rs:182:59`: method `weight` has an incompatible type for trait
  - Hinweis: expected signature `fn(&BlockWeighter, &(_, _), &bytes::Bytes) -> u64`    found signature `fn(&BlockWeighter, &(_, _), &bytes::Bytes) -> u32`
  - Hinweis: change the output type to match the trait

### `crates/memfuse-store/src/wal/replay.rs` (8 Fehler)

Fehlercodes: `E0453`×4, `unsafe_code`×4

- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block


## Crate: `memfuse-store[fault-injection]`

### `crates/memfuse-store/src/wal/replay.rs` (8 Fehler)

Fehlercodes: `E0453`×4, `unsafe_code`×4

- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - Hinweis: `forbid` lint level was set on command line (`-F unsafe_code`)
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block


## Crate: `memfuse-text`

### `crates/memfuse-text/tests/alloc_profiler.rs` (5 Fehler)

Fehlercodes: `unsafe_code`×5

- **[unsafe_code]** @ `crates/memfuse-text/tests/alloc_profiler.rs:25:1`: implementation of an `unsafe` trait
  - Hinweis: requested on the command line with `-F unsafe-code`
- **[unsafe_code]** @ `crates/memfuse-text/tests/alloc_profiler.rs:28:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-text/tests/alloc_profiler.rs:33:19`: usage of an `unsafe` block
- **[unsafe_code]** @ `crates/memfuse-text/tests/alloc_profiler.rs:38:5`: implementation of an `unsafe` method
- **[unsafe_code]** @ `crates/memfuse-text/tests/alloc_profiler.rs:42:9`: usage of an `unsafe` block
