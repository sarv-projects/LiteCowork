"""Tests for scripts/v1_csp_audit.py (019-T06, AC-005)."""

import importlib.util
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "v1_csp_audit.py"
spec = importlib.util.spec_from_file_location("v1_csp_audit", SCRIPT)
audit = importlib.util.module_from_spec(spec)
sys.modules["v1_csp_audit"] = audit
spec.loader.exec_module(audit)

GOOD = (
    "default-src 'self'; connect-src ipc: http://ipc.localhost; "
    "img-src 'self' asset: http://asset.localhost data:; style-src 'self' 'unsafe-inline'; "
    "script-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'"
)


class CspAuditTests(unittest.TestCase):
    def test_repository_configuration_passes(self) -> None:
        self.assertEqual(audit.audit_directory(audit.TAURI_DIRECTORY), [])

    def test_good_csp_passes(self) -> None:
        self.assertEqual(audit.audit_csp(GOOD), [])

    def test_unsafe_eval_fails(self) -> None:
        self.assertTrue(audit.audit_csp(GOOD.replace("script-src 'self'", "script-src 'self' 'unsafe-eval'")))

    def test_remote_script_origin_fails(self) -> None:
        self.assertTrue(audit.audit_csp(GOOD.replace("script-src 'self'", "script-src 'self' https://cdn.example")))

    def test_removing_object_src_fails(self) -> None:
        problems = audit.audit_csp(GOOD.replace("object-src 'none'; ", ""))
        self.assertTrue(any("object-src" in p for p in problems))

    def test_dangerous_disable_fails(self) -> None:
        self.assertTrue(audit.audit_csp(GOOD, dangerous_disable=True))

    def test_shell_and_fs_permissions_fail(self) -> None:
        for name in ("shell:allow-open", "fs:default"):
            problems = audit.audit_capabilities([{"identifier": "x", "permissions": ["core:default", name]}])
            self.assertEqual(len(problems), 1)

    def test_core_default_passes(self) -> None:
        self.assertEqual(audit.audit_capabilities([{"identifier": "x", "permissions": ["core:default"]}]), [])


if __name__ == "__main__":
    unittest.main()
