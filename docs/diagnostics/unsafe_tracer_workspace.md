# Unsafe Boundary & Safety Proof Audit Report

> **Werkzeug:** `unsafe_boundary_tracer.py`  
> **Erstellungszeit:** `2026-11-18T18:00:49.444594+00:00`  
> **Status/Befunde:** `340 Befunde erfasst`  

## Beschreibung
Comprehensive audit of unsafe code blocks, // SAFETY: proofs, pointer casts, FFI externs, and Unsafe Island adherence (M9 contract).

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `total_unsafe_blocks` | `308` |
| `missing_safety_proofs` | `3` |
| `non_island_violations` | `0` |
| `raw_pointer_operations` | `32` |

## Crate-Sicherheits-Status
| Crate | Unsafe-Insel? | `#![forbid(unsafe_code)]` | Unsafe Blöcke | Fehlende SAFETY-Beweise |
|---|---|---|---|---|
| `contextra-simd` | JA (Erlaubt) | **FEHLT** | 286 | 3 |
| `contextra-sys` | JA (Erlaubt) | **FEHLT** | 22 | 0 |

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `192` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `197` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `202` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `209` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `213` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `221` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `250` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `255` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `260` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `267` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `271` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `279` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `308` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `313` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `318` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `325` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `329` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `337` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `366` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `371` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `378` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `382` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `411` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `416` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `423` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `429` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `458` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `463` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `470` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `474` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `501` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `510` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `520` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `524` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `551` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `558` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `565` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `569` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `600` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `609` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `619` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs` | `623` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `37` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `42` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `44` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `46` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `53` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `61` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `63` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `65` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `71` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `82` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `108` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `113` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `120` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `127` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `130` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `136` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `140` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `159` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `164` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `171` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `179` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `185` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `189` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `209` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `214` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `216` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `218` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `225` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `228` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `233` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `235` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `237` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `243` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `255` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `291` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `296` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `303` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `306` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `310` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `313` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `319` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `325` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `354` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `359` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `366` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `369` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `374` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `380` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `386` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `412` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `414` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `416` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `418` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `420` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `422` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `424` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `437` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `442` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `444` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `451` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `453` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `454` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `459` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `461` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `463` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `466` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `468` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `470` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `473` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `475` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `481` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `485` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `501` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `506` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `508` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `515` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `517` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `518` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `523` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `525` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `527` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `529` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `532` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `534` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `536` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `538` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `541` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `543` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `549` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `553` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `570` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `578` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `580` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `582` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `584` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `591` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `593` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `594` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `599` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `601` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `603` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `605` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `607` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `610` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `612` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `614` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `616` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `618` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `624` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `634` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `656` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `658` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `660` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `662` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `664` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `666` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs` | `668` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `37` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `42` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `44` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `46` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `53` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `61` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `63` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `65` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `71` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `81` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `107` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `112` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `119` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `126` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `129` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `135` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `139` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `158` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `163` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `170` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `178` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `184` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `188` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `208` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `213` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `215` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `217` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `224` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `227` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `232` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `234` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `236` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `242` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `254` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `290` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `295` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `302` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `305` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `309` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `312` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `318` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `324` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `353` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `358` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `365` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `368` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `373` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `379` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `385` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `411` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `413` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `415` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `417` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `419` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `421` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `423` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `425` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `438` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `443` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `445` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `447` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `449` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `456` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `458` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `459` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `464` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `468` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `470` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `477` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `480` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `484` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `500` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `505` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `507` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `514` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `516` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `517` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `522` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `524` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `526` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `528` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `531` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `533` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `535` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `537` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `540` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `542` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `548` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `552` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `569` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `577` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `579` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `581` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `583` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `585` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `587` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `589` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `596` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `598` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `599` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `604` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `606` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `610` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `612` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `614` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `617` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `618` | Unsafe block/item without // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `624` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `626` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `628` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `631` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `641` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `663` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `666` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `668` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `670` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `672` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `674` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `676` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs` | `678` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `35` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `40` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `42` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `44` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `51` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `54` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `56` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `58` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `64` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `66` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `68` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `72` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `98` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `103` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `110` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `112` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `115` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `121` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `125` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `144` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `149` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `156` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `159` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `165` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs` | `169` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/acl_win32.rs` | `101` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/acl_win32.rs` | `128` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/acl_win32.rs` | `154` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/acl_win32.rs` | `212` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-sys` | `crates/contextra-sys/src/acl_win32.rs` | `215` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/acl_win32.rs` | `229` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-sys` | `crates/contextra-sys/src/acl_win32.rs` | `247` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-sys` | `crates/contextra-sys/src/acl_win32.rs` | `305` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/acl_win32.rs` | `307` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-sys` | `crates/contextra-sys/src/acl_win32.rs` | `330` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/acl_win32.rs` | `346` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-sys` | `crates/contextra-sys/src/acl_win32.rs` | `349` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/acl_win32.rs` | `389` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-sys` | `crates/contextra-sys/src/acl_win32.rs` | `390` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-sys` | `crates/contextra-sys/src/durability.rs` | `86` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/durability.rs` | `110` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/durability.rs` | `128` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/durability.rs` | `152` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/mlock.rs` | `96` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/mlock.rs` | `96` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-sys` | `crates/contextra-sys/src/mlock.rs` | `150` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/mlock.rs` | `150` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-sys` | `crates/contextra-sys/src/mlock.rs` | `179` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/mlock.rs` | `180` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-sys` | `crates/contextra-sys/src/mlock.rs` | `240` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/mlock.rs` | `241` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-sys` | `crates/contextra-sys/src/mlock.rs` | `248` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/mlock.rs` | `249` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-sys` | `crates/contextra-sys/src/mlock.rs` | `315` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-sys` | `crates/contextra-sys/src/mlock.rs` | `322` | Raw pointer cast (`as *const / as *mut`) |
| `contextra-sys` | `crates/contextra-sys/src/mmap.rs` | `45` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/posix.rs` | `53` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/posix.rs` | `90` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/posix.rs` | `127` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/posix.rs` | `188` | Unsafe block/item with valid // SAFETY: proof |
| `contextra-sys` | `crates/contextra-sys/src/vault.rs` | `88` | Unsafe block/item with valid // SAFETY: proof |

---
*Automatisch generiert durch `unsafe_boundary_tracer.py` für Agentic Engineering & Context Auditing.*