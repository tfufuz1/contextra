#!/usr/bin/env python3
import os
import re

VECTOR_DIR = "crates/contextra-vector"

TYPES = {
    "ContextraError", "DocId", "TxId", "DistanceMetric", "Result",
    "ScoredDocument", "MAX_SEARCH_K", "HnswDeletionError"
}

PORTS = {
    "VectorIndex", "VectorIndexStats"
}

MVCC = {
    "SequenceLog", "SeqLogChange", "TxBuffer", "IndexOp"
}

def replace_use_statement(match):
    items_str = match.group(1).strip()
    if items_str.startswith("{") and items_str.endswith("}"):
        items_inner = items_str[1:-1].strip()
    else:
        items_inner = items_str

    items = [i.strip() for i in items_inner.split(",") if i.strip()]

    types_imports = []
    ports_imports = []
    mvcc_imports = []
    unknown_imports = []

    for item in items:
        clean_item = item
        if item.startswith("error::"):
            clean_item = item.replace("error::", "")
        elif item.startswith("types::"):
            clean_item = item.replace("types::", "")
        elif item.startswith("traits::"):
            clean_item = item.replace("traits::", "")

        if clean_item in TYPES:
            types_imports.append(clean_item)
        elif clean_item in PORTS:
            ports_imports.append(clean_item)
        elif clean_item in MVCC:
            mvcc_imports.append(clean_item)
        else:
            unknown_imports.append(item)

    lines = []
    if types_imports:
        if len(types_imports) == 1:
            lines.append(f"use contextra_types::{types_imports[0]};")
        else:
            lines.append(f"use contextra_types::{{{', '.join(types_imports)}}};")

    if ports_imports:
        if len(ports_imports) == 1:
            lines.append(f"use contextra_ports::{ports_imports[0]};")
        else:
            lines.append(f"use contextra_ports::{{{', '.join(ports_imports)}}};")

    if mvcc_imports:
        if len(mvcc_imports) == 1:
            lines.append(f"use contextra_mvcc::{mvcc_imports[0]};")
        else:
            lines.append(f"use contextra_mvcc::{{{', '.join(mvcc_imports)}}};")

    if unknown_imports:
        print(f"UNKNOWN IMPORTS: {unknown_imports}")
        lines.append(f"use contextra_core::{{{', '.join(unknown_imports)}}};")

    return "\n".join(lines)

def process_file(filepath):
    with open(filepath, "r", encoding="utf-8") as f:
        content = f.read()

    original = content

    pattern = re.compile(r"use\s+contextra_core::(\{[^}]+\}|[a-zA-Z0-9_:]+);")
    content = pattern.sub(replace_use_statement, content)

    content = content.replace("contextra_core::ContextraError", "contextra_types::ContextraError")
    content = content.replace("contextra_core::DocId", "contextra_types::DocId")
    content = content.replace("contextra_core::TxId", "contextra_types::TxId")
    content = content.replace("contextra_core::DistanceMetric", "contextra_types::DistanceMetric")
    content = content.replace("contextra_core::Result", "contextra_types::Result")
    content = content.replace("contextra_core::ScoredDocument", "contextra_types::ScoredDocument")
    content = content.replace("contextra_core::MAX_SEARCH_K", "contextra_types::MAX_SEARCH_K")
    content = content.replace("contextra_core::SequenceLog", "contextra_mvcc::SequenceLog")
    content = content.replace("contextra_core::SeqLogChange", "contextra_mvcc::SeqLogChange")
    content = content.replace("contextra_core::TxBuffer", "contextra_mvcc::TxBuffer")
    content = content.replace("contextra_core::VectorIndex", "contextra_ports::VectorIndex")
    content = content.replace("contextra_core::traits::VectorIndex", "contextra_ports::VectorIndex")

    if content != original:
        with open(filepath, "w", encoding="utf-8") as f:
            f.write(content)

def main():
    for root, dirs, files in os.walk(VECTOR_DIR):
        for file in files:
            if file.endswith(".rs"):
                process_file(os.path.join(root, file))

if __name__ == "__main__":
    main()
