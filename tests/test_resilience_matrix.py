"""Resilience runner reports BLOCKED, never PASSED, for a case it cannot run (019-T18, AC-021)."""
import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
spec = importlib.util.spec_from_file_location("v1_resilience_matrix", ROOT / "scripts" / "v1_resilience_matrix.py")
module = importlib.util.module_from_spec(spec)
sys.modules["v1_resilience_matrix"] = module
spec.loader.exec_module(module)


class ResilienceRunner(unittest.TestCase):
    def test_missing_fcntl_is_blocked_not_passed(self):
        original = module.fcntl
        module.fcntl = None
        try:
            with tempfile.TemporaryDirectory(dir="/tmp") as tmp:
                state, observation = module.case_stale_lock(Path(tmp))
        finally:
            module.fcntl = original
        self.assertEqual(state, "BLOCKED")
        self.assertIn("unavailable", observation)

    def test_runner_has_no_posix_only_top_level_imports(self):
        source = (ROOT / "scripts" / "v1_resilience_matrix.py").read_text(encoding="utf-8")
        top_level = [line for line in source.splitlines() if line.startswith("import ")]
        self.assertNotIn("import fcntl", top_level)
        self.assertNotIn("import signal", top_level)

    def test_local_cases_pass_on_this_host(self):
        with tempfile.TemporaryDirectory(dir="/tmp") as tmp:
            state, _ = module.case_restart_mid_write(Path(tmp))
        self.assertEqual(state, "PASSED")


if __name__ == "__main__":
    unittest.main()
