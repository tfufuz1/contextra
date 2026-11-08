# Unsafe Boundary & Safety Proof Audit Report

> **Werkzeug:** `unsafe_boundary_tracer.py`
> **Erstellungszeit:** `2026-11-07T23:00:46.469252+00:00`
> **Status/Befunde:** `287 Befunde erfasst`

## Beschreibung
Comprehensive audit of unsafe code blocks, // SAFETY: proofs, pointer casts, FFI externs, and Unsafe Island adherence (M9 contract).

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `total_unsafe_blocks` | `269` |
| `missing_safety_proofs` | `25` |
| `non_island_violations` | `0` |
| `raw_pointer_operations` | `18` |

## Crate-Sicherheits-Status
| Crate | Unsafe-Insel? | `#![forbid(unsafe_code)]` | Unsafe Blöcke | Fehlende SAFETY-Beweise |
|---|---|---|---|---|
| `contextra-simd` | JA (Erlaubt) | **FEHLT** | 269 | 25 |

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `124` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `129` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `134` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `141` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `145` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `153` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `182` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `187` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `192` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `199` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `203` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `211` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `240` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `245` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `250` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `257` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `261` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `269` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `302` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `307` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `314` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `318` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `351` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `356` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `363` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `369` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `402` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `407` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `414` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `418` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `445` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `454` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `464` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `468` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `495` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `502` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `509` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `513` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `544` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `553` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `563` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `567` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `35` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `40` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `42` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `44` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `51` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `59` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `61` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `63` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `69` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `80` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `105` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `110` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `117` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `124` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `127` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `133` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `137` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `155` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `160` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `167` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `175` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `181` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `185` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `202` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `207` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `209` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `211` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `218` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `221` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `226` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `228` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `230` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `236` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `248` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `283` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `288` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `295` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `298` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `302` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `305` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `311` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `317` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `345` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `350` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `357` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `360` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `365` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `371` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `377` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `402` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `404` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `406` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `408` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `410` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `412` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `414` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `424` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `429` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `431` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `438` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `440` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `441` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `446` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `448` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `450` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `453` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `455` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `457` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `460` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `462` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `468` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `472` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `487` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `492` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `494` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `501` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `503` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `504` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `509` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `511` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `513` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `515` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `518` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `520` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `522` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `524` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `527` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `529` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `535` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `539` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `555` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `563` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `565` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `567` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `569` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `576` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `578` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `579` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `584` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `586` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `588` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `590` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `592` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `595` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `597` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `599` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `601` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `603` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `609` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `619` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `640` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `642` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `644` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `646` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `648` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `650` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `652` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `35` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `40` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `42` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `44` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `51` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `59` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `61` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `63` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `69` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `79` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `104` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `109` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `116` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `123` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `126` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `132` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `136` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `154` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `159` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `166` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `174` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `180` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `184` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `201` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `206` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `208` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `210` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `217` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `220` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `225` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `227` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `229` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `235` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `247` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `282` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `287` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `294` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `297` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `301` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `304` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `310` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `316` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `344` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `349` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `356` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `359` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `364` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `370` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `376` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `401` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `403` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `405` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `407` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `409` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `411` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `413` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `415` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `425` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `430` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `437` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `439` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `440` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `445` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `451` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `455` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `470` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `475` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `477` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `484` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `486` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `487` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `492` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `494` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `496` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `498` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `501` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `503` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `505` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `507` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `510` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `512` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `518` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `522` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `538` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `546` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `548` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `550` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `557` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `559` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `560` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `565` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `567` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `569` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `575` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `585` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `606` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `609` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `611` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `613` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `615` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `617` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `619` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `621` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `33` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `38` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `40` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `42` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `49` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `52` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `54` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `56` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `62` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `64` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `66` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `70` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `95` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `100` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `107` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `109` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `112` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `118` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `122` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `140` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `145` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `152` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `155` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `161` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `165` | Unsafe block/item with valid // SAFETY: proof |

---
*Automatisch generiert durch `unsafe_boundary_tracer.py` für Agentic Engineering & Context Auditing.*