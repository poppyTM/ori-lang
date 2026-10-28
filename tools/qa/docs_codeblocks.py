#!/usr/bin/env python3
"""Validate complete `ori` fenced examples embedded in the documentation.

Only self-contained blocks are checked: the block must start with a `module`
declaration, contain an entry `main(`, and import only `ori.*` modules. Snippets
that reference external/project modules or are intentionally fragmentary are
skipped, because they cannot be type-checked in isolation.

This is deliberately conservative. It catches stale API names and syntax in
docs without turning every illustrative fragment into a test fixture.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ORI = Path(os.environ.get("ORI_BIN", ROOT / "compiler/target/debug/ori"))
if not ORI.is_file():
    ORI = Path(os.environ.get("ORI_BIN", "ori"))

FENCE = re.compile(r"```ori\n(.*?)```", re.DOTALL)
IMPORT = re.compile(r"^\s*import\s+([A-Za-z0-9_.]+)", re.MULTILINE)


def candidate_blocks() -> list[tuple[Path, int, str]]:
    paths: list[Path] = []
    for directory in ("docs/spec", "docs/language", "docs/guides"):
        paths.extend(sorted((ROOT / directory).glob("*.md")))
    blocks: list[tuple[Path, int, str]] = []
    for path in paths:
        text = path.read_text(encoding="utf-8")
        for index, match in enumerate(FENCE.finditer(text), 1):
            body = match.group(1)
            stripped = body.lstrip()
            if not stripped.startswith("module ") or "main(" not in body:
                continue
            imports = IMPORT.findall(body)
            if any(not module.startswith("ori.") for module in imports):
                continue
            blocks.append((path, index, body))
    return blocks


def main() -> int:
    if not ORI.is_file() and not os.access(ORI, os.X_OK):
        print(f"docs_codeblocks: compiler not found at {ORI}", file=sys.stderr)
        return 2

    blocks = candidate_blocks()
    failures: list[tuple[Path, int, str]] = []
    for path, index, body in blocks:
        with tempfile.NamedTemporaryFile(
            "w", suffix=".orl", delete=False, encoding="utf-8"
        ) as temporary:
            temporary.write(body)
            temporary_path = temporary.name
        try:
            result = subprocess.run(
                [str(ORI), "check", temporary_path],
                capture_output=True,
                text=True,
                timeout=30,
            )
            if result.returncode != 0:
                failures.append(
                    (
                        path.relative_to(ROOT),
                        index,
                        (result.stderr or result.stdout).strip(),
                    )
                )
        except subprocess.TimeoutExpired:
            failures.append((path.relative_to(ROOT), index, "check timed out"))
        finally:
            os.unlink(temporary_path)

    if failures:
        print(f"docs_codeblocks: {len(failures)} of {len(blocks)} block(s) failed")
        for path, index, message in failures:
            print(f"- {path} block {index}:", file=sys.stderr)
            for line in message.splitlines()[:8]:
                print(f"  {line}", file=sys.stderr)
        return 1

    print(f"docs_codeblocks: OK ({len(blocks)} self-contained block(s))")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
