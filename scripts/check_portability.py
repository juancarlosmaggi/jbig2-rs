#!/usr/bin/env python3
"""Verify the decoder-core dependency and license readiness gate."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PROHIBITED_LICENSE_MARKERS = (
    "AGPL",
    "GPL",
    "LGPL",
    "MPL",
    "COMMONS CLAUSE",
    "NON-COMMERCIAL",
    "NONCOMMERCIAL",
    "CC-BY-NC",
    "BUSL",
    "POLYFORM",
)


def run(command: list[str]) -> str:
    result = subprocess.run(
        command,
        cwd=ROOT,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    return result.stdout


def metadata() -> dict:
    return json.loads(
        run(["cargo", "metadata", "--format-version", "1", "--no-default-features"])
    )


def check_license_graph(meta: dict) -> None:
    packages = {package["id"]: package for package in meta["packages"]}
    nodes = {node["id"]: node for node in meta["resolve"]["nodes"]}
    resolved_ids = normal_dependency_ids(meta["resolve"]["root"], nodes)
    failures: list[str] = []

    for package_id in sorted(resolved_ids):
        package = packages[package_id]
        name = package["name"]
        license_expr = package.get("license")
        license_file = package.get("license_file")
        if not license_expr and not license_file:
            failures.append(f"{name}: unknown license")
            continue
        upper = (license_expr or license_file or "").upper()
        if any(marker in upper for marker in PROHIBITED_LICENSE_MARKERS):
            failures.append(f"{name}: prohibited license marker in {license_expr or license_file}")

    if failures:
        raise SystemExit("license audit failed:\n" + "\n".join(failures))


def normal_dependency_ids(root_id: str, nodes: dict[str, dict]) -> set[str]:
    seen: set[str] = set()
    stack = [root_id]
    while stack:
        package_id = stack.pop()
        if package_id in seen:
            continue
        seen.add(package_id)
        for dep in nodes[package_id]["deps"]:
            dep_kinds = dep.get("dep_kinds") or []
            if any(kind.get("kind") in (None, "normal") for kind in dep_kinds):
                stack.append(dep["pkg"])
    return seen


def check_root_metadata(meta: dict) -> None:
    root = next(package for package in meta["packages"] if package["name"] == "jbig2-rs")
    if root.get("license") != "MIT OR Apache-2.0":
        raise SystemExit(f"unexpected root license metadata: {root.get('license')!r}")
    if not root.get("repository"):
        raise SystemExit("missing repository metadata")


def main() -> None:
    tree = run(["cargo", "tree", "--no-default-features", "--edges", "normal"])
    print(tree.rstrip())

    meta = metadata()
    check_root_metadata(meta)
    check_license_graph(meta)
    print("portability dependency/license check passed")


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as error:
        sys.stderr.write(error.stdout)
        raise SystemExit(error.returncode)
