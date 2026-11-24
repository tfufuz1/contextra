#!/usr/bin/env python3
import os
import re

VECTOR_DIR = "crates/acta-vector"

TYPES = {
    "ActaError", "DocId", "TxId", "DistanceMetric", "Result",
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
            lines.append(f"use acta_types::{types_imports[0]};")
        else:
            lines.append(f"use acta_types::{{{', '.join(types_imports)}}};")

    if ports_imports:
        if len(ports_imports) == 1:
            lines.append(f"use acta_ports::{ports_imports[0]};")
        else:
            lines.append(f"use acta_ports::{{{', '.join(ports_imports)}}};")

    if mvcc_imports:
        if len(mvcc_imports) == 1:
            lines.append(f"use acta_mvcc::{mvcc_imports[0]};")
        else:
            lines.append(f"use acta_mvcc::{{{', '.join(mvcc_imports)}}};")

    if unknown_imports:
        print(f"UNKNOWN IMPORTS: {unknown_imports}")
        lines.append(f"use acta_core::{{{', '.join(unknown_imports)}}};")

    return "\n".join(lines)

def process_file(filepath):
    with open(filepath, "r", encoding="utf-8") as f:
        content = f.read()

    original = content

    pattern = re.compile(r"use\s+acta_core::(\{[^}]+\}|[a-zA-Z0-9_:]+);")
    content = pattern.sub(replace_use_statement, content)

    content = content.replace("acta_core::ActaError", "acta_types::ActaError")
    content = content.replace("acta_core::DocId", "acta_types::DocId")
    content = content.replace("acta_core::TxId", "acta_types::TxId")
    content = content.replace("acta_core::DistanceMetric", "acta_types::DistanceMetric")
    content = content.replace("acta_core::Result", "acta_types::Result")
    content = content.replace("acta_core::ScoredDocument", "acta_types::ScoredDocument")
    content = content.replace("acta_core::MAX_SEARCH_K", "acta_types::MAX_SEARCH_K")
    content = content.replace("acta_core::SequenceLog", "acta_mvcc::SequenceLog")
    content = content.replace("acta_core::SeqLogChange", "acta_mvcc::SeqLogChange")
    content = content.replace("acta_core::TxBuffer", "acta_mvcc::TxBuffer")
    content = content.replace("acta_core::VectorIndex", "acta_ports::VectorIndex")
    content = content.replace("acta_core::traits::VectorIndex", "acta_ports::VectorIndex")

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
