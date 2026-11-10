#!/usr/bin/env python3
"""Audit the Tauri CSP and capability files for regressions (019-T06, AC-005).

Usage:
    python3 scripts/v1_csp_audit.py [--tauri-dir DIR] [--json]

Exit code 0 when the configuration is hardened, 1 otherwise. The audit only reads
files; it never changes configuration.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPOSITORY_ROOT = Path(__file__).resolve().parent.parent
TAURI_DIRECTORY = REPOSITORY_ROOT / "apps" / "litecowork-ui" / "src-tauri"

REQUIRED_NONE = ("object-src", "base-uri", "frame-ancestors", "form-action")
FORBIDDEN_PERMISSION_PREFIXES = ("shell:", "fs:", "http:", "process:", "opener:", "dialog:", "global-shortcut:")
FORBIDDEN_SCRIPT_TOKENS = ("'unsafe-eval'", "'unsafe-inline'", "'wasm-unsafe-eval'", "*", "data:", "blob:")


def parse_csp(csp: str) -> dict[str, list[str]]:
    directives: dict[str, list[str]] = {}
    for part in csp.split(";"):
        tokens = part.split()
        if tokens:
            directives[tokens[0].lower()] = tokens[1:]
    return directives


def audit_csp(csp: str, dangerous_disable: bool = False) -> list[str]:
    problems: list[str] = []
    if not csp:
        return ["app.security.csp is missing"]
    directives = parse_csp(csp)
    for name in REQUIRED_NONE:
        if directives.get(name) != ["'none'"]:
            problems.append(f"{name} must be exactly 'none'")
    script = directives.get("script-src")
    if script is None:
        problems.append("script-src is missing")
    else:
        if "'self'" not in script:
            problems.append("script-src must contain 'self'")
        for token in script:
            if token.lower() in FORBIDDEN_SCRIPT_TOKENS or token.lower().startswith(("http:", "https:", "ws:", "wss:")):
                problems.append(f"script-src contains forbidden source {token}")
    for name, sources in directives.items():
        if "'unsafe-eval'" in (s.lower() for s in sources):
            problems.append(f"{name} contains 'unsafe-eval'")
    if dangerous_disable:
        problems.append("dangerousDisableAssetCspModification is set")
    return problems


def audit_capabilities(capability_documents: list[dict]) -> list[str]:
    problems: list[str] = []
    for document in capability_documents:
        identifier = document.get("identifier", "<unnamed>")
        for permission in document.get("permissions", []):
            name = permission if isinstance(permission, str) else permission.get("identifier", "")
            if name.startswith(FORBIDDEN_PERMISSION_PREFIXES):
                problems.append(f"capability {identifier} grants {name}")
    return problems


def audit_directory(tauri_directory: Path) -> list[str]:
    configuration_path = tauri_directory / "tauri.conf.json"
    if not configuration_path.is_file():
        return [f"{configuration_path} is missing"]
    configuration = json.loads(configuration_path.read_text(encoding="utf-8"))
    security = configuration.get("app", {}).get("security", {})
    problems = audit_csp(
        security.get("csp", ""),
        bool(security.get("dangerousDisableAssetCspModification")),
    )
    documents = []
    for path in sorted((tauri_directory / "capabilities").glob("*.json")):
        documents.append(json.loads(path.read_text(encoding="utf-8")))
    problems.extend(audit_capabilities(documents))
    return problems


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tauri-dir", type=Path, default=TAURI_DIRECTORY)
    parser.add_argument("--json", action="store_true")
    arguments = parser.parse_args()
    problems = audit_directory(arguments.tauri_dir)
    if arguments.json:
        print(json.dumps({"ok": not problems, "problems": problems}, indent=2))
    elif problems:
        for problem in problems:
            print(f"FAIL: {problem}")
    else:
        print("PASS: CSP and capabilities are hardened")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
