"""Policy tests for .github/workflows (019-T03).

Stdlib only: a regular-expression pass is enough for the properties checked here
and keeps the test runnable in the locked Python environment without extra packages.
"""

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = sorted((ROOT / ".github" / "workflows").glob("*.yml"))
SHA_PIN = re.compile(r"^[0-9a-f]{40}$")


def unpinned_actions(text: str) -> list[str]:
    """Returns every `uses:` reference that is not pinned to a full commit SHA."""
    bad = []
    for match in re.finditer(r"^\s*-?\s*uses:\s*(\S+)", text, re.MULTILINE):
        reference = match.group(1)
        if reference.startswith("./"):
            continue
        _, _, ref = reference.partition("@")
        if not SHA_PIN.match(ref):
            bad.append(reference)
    return bad


def policy_violations(text: str) -> list[str]:
    violations = [f"action not pinned to a full SHA: {item}" for item in unpinned_actions(text)]
    if not re.search(r"^permissions:\s*\n\s+contents:\s*read\s*$", text, re.MULTILINE):
        violations.append("top-level permissions must be exactly contents: read")
    if not re.search(r"^concurrency:", text, re.MULTILINE) or "cancel-in-progress: true" not in text:
        violations.append("concurrency must cancel superseded runs")
    if "pull_request_target" in text:
        violations.append("pull_request_target is forbidden")
    if re.search(r"\$\{\{\s*secrets\.", text):
        violations.append("pull_request workflows must not read repository secrets")
    if "persist-credentials: false" not in text:
        violations.append("checkout must not persist credentials")
    return violations


class WorkflowPolicyTest(unittest.TestCase):
    def test_workflows_exist(self) -> None:
        self.assertTrue(WORKFLOWS, "no workflow files found")

    def test_committed_workflows_satisfy_policy(self) -> None:
        for workflow in WORKFLOWS:
            with self.subTest(workflow=workflow.name):
                self.assertEqual(policy_violations(workflow.read_text(encoding="utf-8")), [])

    def test_tag_reference_fails_the_pinning_check(self) -> None:
        text = "steps:\n  - uses: actions/checkout@v4\n"
        self.assertEqual(unpinned_actions(text), ["actions/checkout@v4"])

    def test_short_sha_fails_the_pinning_check(self) -> None:
        self.assertEqual(unpinned_actions("  - uses: a/b@11d5960\n"), ["a/b@11d5960"])

    def test_full_sha_passes_the_pinning_check(self) -> None:
        sha = "11d5960a326750d5838078e36cf38b85af677262"
        self.assertEqual(unpinned_actions(f"  - uses: a/b@{sha} # v4\n"), [])

    def test_secret_reference_is_a_violation(self) -> None:
        text = "permissions:\n  contents: read\nconcurrency:\n  cancel-in-progress: true\npersist-credentials: false\nenv:\n  T: ${{ secrets.TOKEN }}\n"
        self.assertIn("pull_request workflows must not read repository secrets", policy_violations(text))

    def test_codeowners_covers_workflows_and_tauri_config(self) -> None:
        owners = (ROOT / ".github" / "CODEOWNERS").read_text(encoding="utf-8")
        for path in ("/.github/", "/apps/litecowork-ui/src-tauri/tauri.conf.json", "/apps/litecowork-ui/src-tauri/capabilities/"):
            self.assertIn(path, owners)


if __name__ == "__main__":
    unittest.main()
