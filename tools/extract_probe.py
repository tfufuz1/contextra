#!/usr/bin/env python3
"""
Entnahme-Probe: Beweist, dass ein Baustein ausserhalb des Repos funktioniert.

Was das Skript tut:
  1. Liest per `cargo metadata`, welche internen Crates der Baustein (normal) braucht.
  2. Kopiert genau diese Crates in ein leeres Verzeichnis (gleiche Ordnerstruktur).
  3. Schreibt ein neues Workspace-Cargo.toml mit den gemeinsamen Abschnitten
     ([workspace.package], [workspace.dependencies], [workspace.lints], [profile.*]).
  4. Fuehrt `cargo check` (oder `cargo test`) im Probe-Workspace aus.

Ergebnis:
  - Exit 0  -> der Baustein ist kopierbar. Der "Kopier-Umfang" steht in der Ausgabe.
  - Exit !=0 -> es gibt versteckte Kopplung (z.B. Pfade ausserhalb des Crates,
                fehlende Abhaengigkeiten, Workspace-Annahmen). Die cargo-Meldung zeigt wo.

Aufruf (im Repo-Root):
  python3 tools/extract_probe.py contextra-rank
  python3 tools/extract_probe.py contextra-rank --test        # inkl. dev-Abhaengigkeiten + cargo test
  python3 tools/extract_probe.py contextra-rank --keep        # Probe-Verzeichnis behalten
  python3 tools/extract_probe.py contextra-rank --features a,b
  python3 tools/extract_probe.py contextra-rank --out ../mein-projekt-libs --rename acme
      -> kopiert nach ../mein-projekt-libs und benennt alle Crates um:
         contextra-rank -> acme-rank, Rust-Pfad contextra_types -> acme_types.

Voraussetzung: Rust-Toolchain installiert, Internetzugang fuer crates.io.
Es werden NUR Dateien im Probe-Verzeichnis (Standard: temporaeres Verzeichnis) geschrieben.
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile


def run(cmd, cwd=None, env=None, check=True):
    r = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, text=True)
    if check and r.returncode != 0:
        sys.stderr.write(r.stdout[-3000:] + "\n" + r.stderr[-6000:] + "\n")
        sys.exit(r.returncode)
    return r


def load_meta(root):
    r = run(["cargo", "metadata", "--format-version", "1", "--no-deps"], cwd=root)
    return json.loads(r.stdout)


def closure(pkgs, start, with_dev):
    """Interne Abhaengigkeits-Huelle. dev-Abhaengigkeiten nur fuer den Start-Crate."""
    seen, stack = set(), [start]
    while stack:
        n = stack.pop()
        if n in seen:
            continue
        seen.add(n)
        p = pkgs[n]
        for d in p["dependencies"]:
            if d["name"] not in pkgs or not d.get("path"):
                continue
            is_dev = d["kind"] == "dev"
            if is_dev and not (with_dev and n == start):
                continue
            if d["kind"] == "build" or d["kind"] is None or is_dev:
                stack.append(d["name"])
    return seen


def apply_rename(text, prefix):
    """contextra-xyz -> prefix-xyz und contextra_xyz -> prefix_xyz (nur mit - oder _ danach)."""
    return re.sub(r"\bcontextra([-_])", prefix + r"\1", text)


def rename_tree(out, members, prefix):
    """Benennt Ordner um und ersetzt die Namen in Cargo.toml, .rs und .md der Kopie."""
    new_members = []
    for rel in members:
        new_rel = apply_rename(rel, prefix)
        if new_rel != rel:
            os.makedirs(os.path.dirname(os.path.join(out, new_rel)), exist_ok=True)
            shutil.move(os.path.join(out, rel), os.path.join(out, new_rel))
        new_members.append(new_rel)
    for dirpath, _dirs, files in os.walk(out):
        for fn in files:
            if fn.endswith((".rs", ".toml", ".md")):
                path = os.path.join(dirpath, fn)
                try:
                    txt = open(path, encoding="utf-8").read()
                except UnicodeDecodeError:
                    continue
                new = apply_rename(txt, prefix)
                if new != txt:
                    open(path, "w", encoding="utf-8").write(new)
    return new_members


def split_root_manifest(text):
    """Gibt alles ab [workspace.package] zurueck (ohne [workspace] members/default-members)."""
    m = re.search(r"^\[workspace\.package\]", text, re.M)
    if not m:
        sys.exit("Root-Cargo.toml hat keinen [workspace.package]-Abschnitt.")
    return text[m.start():]


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("crate")
    ap.add_argument("--test", action="store_true", help="dev-Abhaengigkeiten mitnehmen und cargo test ausfuehren")
    ap.add_argument("--keep", action="store_true", help="Probe-Verzeichnis nicht loeschen")
    ap.add_argument("--features", default="", help="Features des Bausteins, kommagetrennt")
    ap.add_argument("--out", default="", help="Zielverzeichnis (Standard: temporaer)")
    ap.add_argument("--target-dir", default="", help="gemeinsames cargo target-Verzeichnis (spart Compile-Zeit)")
    ap.add_argument("--rename", default="", help="neuer Praefix statt 'contextra' (z.B. acme -> acme-rank, acme_types)")
    args = ap.parse_args()

    root = os.getcwd()
    if not os.path.isfile(os.path.join(root, "Cargo.toml")):
        sys.exit("Bitte im Repo-Root ausfuehren.")
    meta = load_meta(root)
    pkgs = {p["name"]: p for p in meta["packages"]}
    if args.crate not in pkgs:
        sys.exit(f"Crate '{args.crate}' nicht im Workspace. Bekannt: {', '.join(sorted(pkgs))}")

    names = sorted(closure(pkgs, args.crate, args.test))
    internal = [n for n in names if n != args.crate]
    print(f"Baustein       : {args.crate}")
    print(f"Kopier-Umfang  : {len(internal)} interne(s) Crate(s) zusaetzlich -> {', '.join(internal) or '-'}")

    out = args.out or tempfile.mkdtemp(prefix=f"probe_{args.crate}_")
    if os.path.exists(out) and os.listdir(out):
        sys.exit(f"Zielverzeichnis {out} ist nicht leer.")
    os.makedirs(out, exist_ok=True)

    members = []
    for n in names:
        src = os.path.dirname(pkgs[n]["manifest_path"])
        rel = os.path.relpath(src, root)
        dst = os.path.join(out, rel)
        shutil.copytree(src, dst, ignore=shutil.ignore_patterns("target", ".git", "Cargo.lock"))
        members.append(rel)

    root_text = open(os.path.join(root, "Cargo.toml"), encoding="utf-8").read()
    if args.rename:
        if not re.fullmatch(r"[a-z][a-z0-9_]*", args.rename):
            sys.exit("--rename: nur Kleinbuchstaben, Ziffern und _ erlaubt, mit Buchstabe am Anfang.")
        members = rename_tree(out, members, args.rename)
        root_text = apply_rename(root_text, args.rename)
        args.crate = apply_rename(args.crate, args.rename)
    manifest = "[workspace]\nresolver = \"2\"\nmembers = [\n" + "".join(f'    "{m}",\n' for m in members) + "]\n\n"
    manifest += split_root_manifest(root_text)
    with open(os.path.join(out, "Cargo.toml"), "w", encoding="utf-8") as f:
        f.write(manifest)
    # Toolchain-Datei NICHT kopieren: ein neues Projekt hat sie nicht zwingend.

    env = dict(os.environ)
    if args.target_dir:
        env["CARGO_TARGET_DIR"] = args.target_dir
    cmd = ["cargo", "test" if args.test else "check", "-p", args.crate]
    if args.features:
        cmd += ["--features", args.features]
    print(f"Probe-Workspace: {out}")
    print("Ausfuehren     :", " ".join(cmd))
    r = run(cmd, cwd=out, env=env, check=False)
    tail = (r.stdout + r.stderr).strip().splitlines()[-12:]
    print("\n".join(tail))
    if r.returncode == 0:
        print(f"\nERGEBNIS: OK. '{args.crate}' laeuft isoliert ({len(names)} Crate(s) kopiert).")
    else:
        print(f"\nERGEBNIS: FEHLGESCHLAGEN (Exit {r.returncode}). Versteckte Kopplung, siehe Meldung oben.")
    if not args.keep and args.out == "":
        shutil.rmtree(out, ignore_errors=True)
    else:
        print(f"Probe-Verzeichnis behalten: {out}")
    sys.exit(r.returncode)


if __name__ == "__main__":
    main()
