"""Seeded fake-token tests for scripts/v1_secret_scan.py (019-T05).

The fake token is assembled at run time so this file never contains a literal that
the source scan would report.
"""

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "v1_secret_scan.py"
FAKE_TOKEN = "gh" + "p_" + "A1b2C3d4" * 5  # 40 token characters after the prefix


def load_scanner():
    spec = importlib.util.spec_from_file_location("v1_secret_scan", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    sys.modules["v1_secret_scan"] = module
    spec.loader.exec_module(module)
    return module


def run_scan(*arguments: str) -> tuple[int, dict]:
    completed = subprocess.run(
        [sys.executable, str(SCRIPT), "--json", *arguments],
        capture_output=True,
        text=True,
        check=False,
    )
    return completed.returncode, json.loads(completed.stdout)


class SeededTokenTest(unittest.TestCase):
    def test_seeded_token_in_a_bundle_file_fails_with_a_redacted_report(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            bundle = Path(directory) / "bundle"
            bundle.mkdir()
            (bundle / "notes.txt").write_text(f"deploy with {FAKE_TOKEN}\n", encoding="utf-8")
            code, report = run_scan("--extra-path", str(bundle))
        self.assertNotEqual(code, 0)
        findings = report["findings"]
        self.assertEqual([item["pattern"] for item in findings], ["github token"])
        self.assertEqual(findings[0]["line"], 1)
        self.assertNotIn(FAKE_TOKEN, json.dumps(report))
        self.assertIn("[REDACTED:github token]", findings[0]["preview"])

    def test_seeded_token_in_an_evidence_file_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            evidence = Path(directory) / "evidence.json"
            evidence.write_text(json.dumps({"log": FAKE_TOKEN}), encoding="utf-8")
            code, report = run_scan("--extra-path", str(evidence))
        self.assertNotEqual(code, 0)
        self.assertEqual(len(report["findings"]), 1)

    def test_clean_bundle_passes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            bundle = Path(directory)
            (bundle / "clean.txt").write_text("nothing sensitive here\n", encoding="utf-8")
            code, report = run_scan("--extra-path", str(bundle))
        self.assertEqual(code, 0)
        self.assertEqual(report["findings"], [])

    def test_seeded_token_in_source_scanning_is_detected(self) -> None:
        scanner = load_scanner()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "module.py"
            source.write_text(f'KEY = "{FAKE_TOKEN}"\n', encoding="utf-8")
            result = scanner.ScanResult()
            scanner.scan_file(source, root, result)
        self.assertEqual([item.pattern for item in result.findings], ["github token"])

    def test_report_does_not_echo_unbounded_secret_context(self) -> None:
        scanner = load_scanner()
        long_line = (FAKE_TOKEN + " ") * 20
        self.assertLessEqual(len(scanner.redact(long_line.encode())), 120)


class ExceptionTableTest(unittest.TestCase):
    def setUp(self) -> None:
        self.scanner = load_scanner()
        self.finding = self.scanner.Finding(pattern="github token", path="a/b.rs", line=3, preview="x")
        self.entry = {
            "id": "SS-T",
            "path": "a/b.rs",
            "pattern": "github token",
            "reason": "fixture",
            "reviewer": "someone",
            "expires": "2030-01-01",
        }

    def test_unexpired_exception_covers_only_its_path_and_pattern(self) -> None:
        import datetime

        today = datetime.date(2029, 1, 1)
        other = self.scanner.Finding(pattern="github token", path="c/d.rs", line=1, preview="x")
        remaining, problems = self.scanner.apply_exceptions([self.finding, other], [self.entry], today)
        self.assertEqual(remaining, [other])
        self.assertEqual(problems, [])

    def test_expired_exception_fails_the_gate(self) -> None:
        import datetime

        remaining, problems = self.scanner.apply_exceptions([self.finding], [self.entry], datetime.date(2030, 1, 2))
        self.assertEqual(remaining, [self.finding])
        self.assertEqual(len(problems), 1)
        self.assertIn("expired", problems[0])

    def test_incomplete_exception_covers_nothing(self) -> None:
        import datetime

        entry = dict(self.entry)
        del entry["reviewer"]
        remaining, problems = self.scanner.apply_exceptions([self.finding], [entry], datetime.date(2029, 1, 1))
        self.assertEqual(remaining, [self.finding])
        self.assertIn("reviewer", problems[0])

    def test_committed_exceptions_are_complete_and_unexpired(self) -> None:
        import datetime

        entries = self.scanner.load_exceptions(self.scanner.EXCEPTIONS_FILE)
        self.assertTrue(entries)
        _, problems = self.scanner.apply_exceptions([], entries, datetime.date.today())
        self.assertEqual(problems, [])


if __name__ == "__main__":
    unittest.main()
