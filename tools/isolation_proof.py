import os
import shutil
import subprocess
import sys
import tempfile

def main():
    root = os.path.abspath(os.getcwd())
    print("=== M2 ISOLATIONSBEWEIS FÜR CONTEXTRA-VECTOR ===")

    with tempfile.TemporaryDirectory() as tmpdir:
        print(f"Erstelle isolierten Workspace in {tmpdir}...")

        # Erlaubte Abhängigkeiten laut PLAN.md und capabilities.toml:
        # contextra-types, contextra-ports, contextra-mvcc, contextra-simd, contextra-sys
        needed_crates = [
            "contextra-types",
            "contextra-ports",
            "contextra-mvcc",
            "contextra-simd",
            "contextra-sys",
            "contextra-vector"
        ]

        crates_dir = os.path.join(tmpdir, "crates")
        os.makedirs(crates_dir, exist_ok=True)

        for c in needed_crates:
            src = os.path.join(root, "crates", c)
            dst = os.path.join(crates_dir, c)
            shutil.copytree(src, dst)
            cargo_path = os.path.join(dst, "Cargo.toml")
            with open(cargo_path, "r") as f:
                ct = f.read()
            ct = ct.replace('contextra-testkit = { workspace = true }', '')
            with open(cargo_path, "w") as f:
                f.write(ct)
            print(f"Kopiert und isoliert: {c}")

        import tomllib
        with open(os.path.join(root, "Cargo.toml"), "rb") as f:
            d = tomllib.load(f)

        deps = d.get("workspace", {}).get("dependencies", {})

        lines = []
        lines.append('[workspace]\nresolver = "2"')
        lines.append('members = [')
        for c in needed_crates:
            lines.append(f'    "crates/{c}",')
        lines.append(']\n')
        lines.append('[workspace.package]')
        lines.append('version = "0.1.0"\nedition = "2021"\nrust-version = "1.89"\nlicense = "MIT OR Apache-2.0"\n')
        lines.append('[workspace.lints.rust]\nunsafe_code = "deny"\nunsafe_op_in_unsafe_fn = "deny"\n')
        lines.append('[workspace.lints.clippy]\nunwrap_used = "deny"\nexpect_used = "deny"\n')
        lines.append('[workspace.dependencies]')
        for c in needed_crates:
            lines.append(f'{c} = {{ path = "crates/{c}" }}')

        for k, v in deps.items():
            if k not in needed_crates:
                if isinstance(v, str):
                    lines.append(f'{k} = "{v}"')
                elif isinstance(v, dict) and "path" not in v:
                    parts = []
                    for vk, vv in v.items():
                        if isinstance(vv, bool):
                            parts.append(f'{vk} = {str(vv).lower()}')
                        elif isinstance(vv, list):
                            parts.append(f'{vk} = {repr(vv)}')
                        else:
                            parts.append(f'{vk} = "{vv}"')
                    lines.append(f'{k} = {{ {", ".join(parts)} }}')

        with open(os.path.join(tmpdir, "Cargo.toml"), "w") as f:
            f.write("\n".join(lines) + "\n")

        # Cargo.lock kopieren
        shutil.copyfile(os.path.join(root, "Cargo.lock"), os.path.join(tmpdir, "Cargo.lock"))

        # Tests ausführen
        cmds = [
            ["cargo", "build", "-p", "contextra-vector", "--no-default-features", "--offline"],
            ["cargo", "build", "-p", "contextra-vector", "--offline"],
            ["cargo", "test", "-p", "contextra-vector", "--lib", "--offline", "--", "--test-threads=1"]
        ]

        for cmd in cmds:
            print(f"\nFühre aus: {' '.join(cmd)}")
            res = subprocess.run(cmd, cwd=tmpdir, capture_output=True, text=True)
            if res.returncode != 0:
                print("FEHLER!")
                print(res.stdout)
                print(res.stderr)
                sys.exit(1)
            else:
                print("ERFOLGREICH!")

    print("\n=== M2 ISOLATIONSBEWEIS VOLLSTÄNDIG BESTANDEN! ===")

if __name__ == "__main__":
    main()
