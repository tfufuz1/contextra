# contextra-sys

`contextra-sys` ist eine Unsafe-Insel für OS-Systemaufrufe (Ring 0).

## Zweck

Kapselt plattformspezifische `unsafe`-Systemaufrufe: Memory Mapping (`mmap`), Memory Locking (`mlock`/`munlock`), Win32 Owner-only ACLs und POSIX File Descriptor Manipulationen.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 0 (Unsafe-Insel)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![allow(unsafe_code)]` — Isoliert in `contextra-sys`, Safe Public API.

## Öffentliche API-Übersicht

- **Memory Mapping:** `mmap_readonly`
- **Memory Locking:** `mem_lock`, `mem_unlock`
- **Win32 ACL:** `set_restrictive_file_acl`, `verify_file_acl_owner_only`
- **POSIX:** `reopen_and_dup2`

## Architektur & Verweise

Details zur Isolierung von Systemaufrufen (§0.4 / §A2 D1) finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze).
