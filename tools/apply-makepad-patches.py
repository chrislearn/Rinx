#!/usr/bin/env python3
"""Apply Rinx's pending Makepad fixes to the checkout selected by Cargo.lock.

Cargo treats git dependencies as immutable. Applying a new patch also invalidates
makepad-platform's build output so the next build includes the source change.
"""
import argparse
import json
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Only verify that the patch is already applied")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    host = next(line.removeprefix("host: ") for line in
                subprocess.check_output(["rustc", "-vV"], text=True).splitlines() if line.startswith("host: "))
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--filter-platform", host, "--format-version", "1"], cwd=root))
    packages = [p for p in metadata["packages"] if p["name"] == "makepad-platform"]
    if len(packages) != 1:
        raise SystemExit("Expected exactly one pinned Makepad platform dependency")
    checkout = Path(packages[0]["manifest_path"]).parent.parent
    patch = root / "tools/patches/makepad-metal-dropped-draw-lists.patch"
    command = ["git", "-C", str(checkout), "apply"]
    if subprocess.run(command + ["--reverse", "--check", str(patch)], capture_output=True).returncode == 0:
        print("Makepad Metal draw-list patch is applied.")
        return
    if args.check:
        raise SystemExit("Makepad Metal draw-list patch is missing; run this tool without --check")
    subprocess.run(command + ["--check", str(patch)], check=True)
    subprocess.run(command + [str(patch)], check=True)
    subprocess.run(["cargo", "clean", "-p", "makepad-platform"], cwd=root, check=True)
    print("Applied Makepad Metal draw-list patch. Rebuild Rinx now.")


if __name__ == "__main__":
    main()
