#!/usr/bin/env python3
"""
API Snapshot Generator for Contextra crates.
Parses Rust source files under src/ and extracts all `pub` items and impl methods.
Produces deterministic, byte-identical sorted text files.
"""

import os
import re
import sys
import argparse

def sanitize_signature(sig: str) -> str:
    """Normalize whitespace in Rust signatures."""
    # Strip comments
    sig = re.sub(r'//.*$', '', sig, flags=re.MULTILINE)
    # Replace newlines and multi-spaces with single space
    sig = re.sub(r'\s+', ' ', sig).strip()
    # Strip trailing block/semicolon if present
    sig = sig.rstrip('{;').strip()
    return sig

def get_module_path(crate_name: str, rel_path: str) -> str:
    """Derive module path from crate name and relative file path."""
    crate_mod = crate_name.replace('-', '_')
    parts = rel_path.split(os.sep)

    # Remove filename or handle mod.rs / lib.rs
    filename = parts[-1]
    dirs = parts[:-1]

    mod_parts = [crate_mod] + dirs
    if filename not in ('lib.rs', 'mod.rs'):
        mod_name = filename.rsplit('.', 1)[0]
        mod_parts.append(mod_name)

    return '::'.join([p for p in mod_parts if p])

def parse_rust_file(filepath: str, base_mod_path: str):
    """
    Parse a single .rs file for public items and public impl methods.
    Returns list of tuples: (full_item_path, signature)
    """
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    # Strip block comments
    content = re.sub(r'/\*.*?\*/', '', content, flags=re.DOTALL)

    items = []
    lines = content.splitlines()

    i = 0
    n = len(lines)

    in_test_cfg = False
    brace_depth = 0
    scope_stack = [] # list of (type, name, depth_where_opened)

    # Buffer for collecting multi-line statements
    while i < n:
        line = lines[i]
        stripped = line.strip()

        # Check for #[cfg(test)]
        if stripped.startswith('#[cfg(test)]'):
            # Skip next item or block if it's test
            in_test_cfg = True
            i += 1
            continue

        # Ignore comments
        if stripped.startswith('//'):
            i += 1
            continue

        # Track brace depth adjustments for active scope
        opens = line.count('{')
        closes = line.count('}')

        # Handle #[cfg(test)] mod tests { ... }
        if in_test_cfg:
            if opens > 0 or stripped.startswith('mod '):
                # We are entering a test module/item, skip until matching closes
                test_depth = opens - closes
                i += 1
                while i < n and test_depth > 0:
                    l = lines[i]
                    test_depth += l.count('{') - l.count('}')
                    i += 1
                in_test_cfg = False
                continue
            else:
                in_test_cfg = False

        # Build current module scope prefix
        current_mod_prefix = base_mod_path
        current_impl_prefix = ""

        for stype, sname, _ in scope_stack:
            if stype == 'mod':
                current_mod_prefix += f"::{sname}"
            elif stype == 'impl':
                current_impl_prefix = sname

        # Detect items on stripped line (collect multi-line signatures up to { or ;)
        if stripped.startswith(('pub ', 'impl ', 'pub(crate) ')):
            # Read ahead to get full item header/signature
            sig_lines = [stripped]
            j = i
            while j < n and not any(c in sig_lines[-1] for c in ('{', ';', '=')):
                j += 1
                if j < n:
                    sig_lines.append(lines[j].strip())

            full_sig_raw = ' '.join(sig_lines)
            clean_sig = sanitize_signature(full_sig_raw)

            # Check pub items
            if stripped.startswith('pub '):
                # pub fn, pub struct, pub enum, pub trait, pub const, pub type, pub mod, pub use
                # Exclude pub(crate) / pub(super)
                m_fn = re.match(r'pub\s+(?:async\s+)?(?:unsafe\s+)?fn\s+([a-zA-Z0-9_]+)', clean_sig)
                m_struct = re.match(r'pub\s+struct\s+([a-zA-Z0-9_]+)', clean_sig)
                m_enum = re.match(r'pub\s+enum\s+([a-zA-Z0-9_]+)', clean_sig)
                m_trait = re.match(r'pub\s+trait\s+([a-zA-Z0-9_]+)', clean_sig)
                m_const = re.match(r'pub\s+const\s+([a-zA-Z0-9_]+)', clean_sig)
                m_type = re.match(r'pub\s+type\s+([a-zA-Z0-9_]+)', clean_sig)
                m_mod = re.match(r'pub\s+mod\s+([a-zA-Z0-9_]+)', clean_sig)
                m_use = re.match(r'pub\s+use\s+(.+)', clean_sig)

                if m_fn:
                    item_name = m_fn.group(1)
                    if current_impl_prefix:
                        full_path = f"{current_impl_prefix}::{item_name}"
                    else:
                        full_path = f"{current_mod_prefix}::{item_name}"
                    items.append((full_path, clean_sig))
                elif m_struct:
                    item_name = m_struct.group(1)
                    full_path = f"{current_mod_prefix}::{item_name}"
                    items.append((full_path, clean_sig))
                elif m_enum:
                    item_name = m_enum.group(1)
                    full_path = f"{current_mod_prefix}::{item_name}"
                    items.append((full_path, clean_sig))
                elif m_trait:
                    item_name = m_trait.group(1)
                    full_path = f"{current_mod_prefix}::{item_name}"
                    items.append((full_path, clean_sig))
                elif m_const:
                    item_name = m_const.group(1)
                    full_path = f"{current_mod_prefix}::{item_name}"
                    items.append((full_path, clean_sig))
                elif m_type:
                    item_name = m_type.group(1)
                    full_path = f"{current_mod_prefix}::{item_name}"
                    items.append((full_path, clean_sig))
                elif m_mod:
                    item_name = m_mod.group(1)
                    full_path = f"{current_mod_prefix}::{item_name}"
                    items.append((full_path, clean_sig))
                elif m_use:
                    item_name = clean_sig
                    full_path = f"{current_mod_prefix}::reexport"
                    items.append((full_path, clean_sig))

            # Track impl blocks (for public structs/enums)
            elif stripped.startswith('impl '):
                # impl<...> TargetType for Trait or impl TargetType
                m_impl = re.match(r'impl(?:\<[^\>]+\>)?\s+(?:([a-zA-Z0-9_:]+)\s+for\s+)?([a-zA-Z0-9_:]+)', clean_sig)
                if m_impl:
                    target_type = m_impl.group(2)
                    if not target_type.startswith(('self::', 'super::')):
                        impl_path = f"{current_mod_prefix}::{target_type}" if '::' not in target_type else target_type
                        if '{' in line or (j < n and '{' in lines[j]):
                            scope_stack.append(('impl', impl_path, brace_depth + opens))

        elif stripped.startswith('mod ') and '{' in line:
            m_mod = re.match(r'mod\s+([a-zA-Z0-9_]+)', stripped)
            if m_mod:
                mod_name = m_mod.group(1)
                if mod_name != 'tests':
                    scope_stack.append(('mod', mod_name, brace_depth + opens))

        # Update brace depth and scope stack
        brace_depth += opens - closes
        while scope_stack and brace_depth < scope_stack[-1][2]:
            scope_stack.pop()

        i += 1

    return items

def generate_snapshot_for_crate(crate_dir: str, crate_name: str) -> str:
    """Scan all .rs files in src/ of crate and return deterministic formatted text."""
    src_dir = os.path.join(crate_dir, 'src')
    all_items = []

    for root, dirs, files in os.walk(src_dir):
        # Ignore tests directory
        if 'tests' in dirs:
            dirs.remove('tests')

        for file in sorted(files):
            if file.endswith('.rs'):
                full_file_path = os.path.join(root, file)
                rel_path = os.path.relpath(full_file_path, src_dir)

                base_mod_path = get_module_path(crate_name, rel_path)
                file_items = parse_rust_file(full_file_path, base_mod_path)
                all_items.extend(file_items)

    # Sort deterministically
    sorted_items = sorted(set(all_items), key=lambda x: (x[0], x[1]))

    lines = [f"{path} {sig}" for path, sig in sorted_items]
    return '\n'.join(lines) + '\n' if lines else ''

def main():
    parser = argparse.ArgumentParser(description="Generate API snapshots for Contextra crates.")
    parser.add_argument('--crate-dir', help="Path to crate directory (e.g., crates/contextra-ports)")
    parser.add_argument('--out', help="Output file path (e.g., api/contextra-ports.txt)")
    args = parser.parse_args()

    os.makedirs('api', exist_ok=True)

    if args.crate_dir and args.out:
        crate_name = os.path.basename(os.path.normpath(args.crate_dir))
        content = generate_snapshot_for_crate(args.crate_dir, crate_name)
        with open(args.out, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"Generated {args.out} ({len(content.splitlines())} items)")
    else:
        # Default behavior: generate for both contextra-ports and contextra-types
        targets = [
            ('crates/contextra-ports', 'contextra-ports', 'api/contextra-ports.txt'),
            ('crates/contextra-types', 'contextra-types', 'api/contextra-types.txt'),
        ]
        for cdir, cname, outpath in targets:
            content = generate_snapshot_for_crate(cdir, cname)
            with open(outpath, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Generated {outpath} ({len(content.splitlines())} items)")

if __name__ == '__main__':
    main()
