"""Release matrix tests (019-T14, AC-002, AC-012)."""
import copy
import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("v1_platform_matrix", ROOT / "scripts" / "v1_platform_matrix.py")
matrix_module = importlib.util.module_from_spec(spec)
sys.modules["v1_platform_matrix"] = matrix_module
spec.loader.exec_module(matrix_module)

ROWS = matrix_module.load_rows(ROOT / "scripts" / "v1_release_matrix_rows.json")
LINUX = "linux-x64-opencode-v2.0.32"
WINDOWS = "windows-x64-opencode-v2.0.32"
NO_FILES = lambda path: False  # noqa: E731


def bundle(row_id=LINUX, journey="E2E-01", signoff=None, sha="a" * 40):
    document = {
        "journey_id": journey, "requirement_ids": ["AC-012"], "owner_contracts": ["docs/X.md"],
        "row_id": row_id, "repo_sha": sha, "dirty": False, "os_build": "Ubuntu 24.04",
        "arch": "x86_64", "packaging_mode": "packaged", "agent_versions": {"opencode": "2.0.32"},
        "protocol": "acp", "code_commands": [],
        "system_trace": {"request_ids": [], "correlation_ids": [], "event_sequences": [], "receipt_refs": []},
        "user_artifacts": [], "negative_cases": [{"case": "x", "outcome": "REJECTED"}],
        "limitations": [], "reviewer": "owner", "accepted_at": "2026-11-10",
    }
    if signoff:
        document["user_signoff"] = {
            "reviewer": "owner", "date": "2026-11-10", "decision": signoff, "blocked_or_unknown": [],
        }
    return document


def build(host_class="NATIVE", system="linux", bundles=None, sha=None):
    return matrix_module.build_release_matrix(
        ROWS, system, host_class, "test", bundles or {}, None, sha
    )


def indexed(documents):
    return {
        (d["row_id"], d["journey_id"]): {"digest": matrix_module.canonical_digest(d), "document": d}
        for d in documents
    }


def cell(matrix, row_id, cell_id):
    return next(r for r in matrix["rows"] if r["row_id"] == row_id)["cells"][cell_id]


class HostDetection(unittest.TestCase):
    def detect(self, env=None, osrelease="6.8.0-generic", exists=NO_FILES, cgroup="0::/"):
        return matrix_module.detect_host_class(env or {}, osrelease, exists, cgroup)[0]

    def test_native(self):
        self.assertEqual(self.detect(), "NATIVE")

    def test_wsl2_markers(self):
        self.assertEqual(self.detect(env={"WSL_DISTRO_NAME": "Ubuntu"}), "WSL2")
        self.assertEqual(self.detect(osrelease="5.15.0-microsoft-standard-WSL2"), "WSL2")

    def test_container_markers(self):
        self.assertEqual(self.detect(exists=lambda p: p == "/.dockerenv"), "CONTAINER")
        self.assertEqual(self.detect(cgroup="12:cpu:/docker/abc"), "CONTAINER")

    def test_ci_image_needs_release_host_label_to_count(self):
        self.assertEqual(self.detect(env={"CI": "true"}), "CI_IMAGE")
        self.assertEqual(self.detect(env={"CI": "true", "LITECOWORK_RELEASE_HOST": "1"}), "NATIVE")


class NotCounted(unittest.TestCase):
    def test_non_native_hosts_never_pass_even_with_bundles(self):
        bundles = indexed([bundle(signoff="ACCEPT")])
        for host_class in ("WSL2", "CONTAINER", "CI_IMAGE"):
            matrix = build(host_class, bundles=bundles)
            for row in matrix["rows"]:
                self.assertFalse(row["counted"])
                for cell_id, c in row["cells"].items():
                    self.assertNotIn(c["state"], ("PASSED", "QUALIFIED"), (host_class, cell_id))

    def test_unavailable_cells_stay_unavailable_not_notcounted(self):
        matrix = build("CONTAINER")
        self.assertEqual(cell(matrix, WINDOWS, "E2E-04")["state"], "UNAVAILABLE")
        self.assertEqual(cell(matrix, WINDOWS, "E2E-08")["state"], "UNAVAILABLE")
        self.assertEqual(cell(matrix, LINUX, "E2E-04")["state"], "NOT_COUNTED")

    def test_generator_refuses_unlinked_pass(self):
        matrix = build()
        row = matrix["rows"][0]
        row["cells"]["E2E-01"] = {"state": "PASSED", "reason": "", "evidence_bundle_digest": None}
        with self.assertRaises(ValueError):
            matrix_module.assert_no_unlinked_pass(matrix)


class NativeHost(unittest.TestCase):
    def test_other_os_row_is_not_run(self):
        matrix = build("NATIVE", "linux")
        self.assertEqual(cell(matrix, WINDOWS, "E2E-01")["state"], "NOT_RUN")
        self.assertTrue(next(r for r in matrix["rows"] if r["row_id"] == LINUX)["counted"])
        self.assertFalse(next(r for r in matrix["rows"] if r["row_id"] == WINDOWS)["counted"])

    def test_pass_needs_linked_valid_bundle_and_signoff_for_qualified(self):
        self.assertEqual(cell(build(), LINUX, "E2E-01")["state"], "NOT_RUN")
        passed = cell(build(bundles=indexed([bundle()])), LINUX, "E2E-01")
        self.assertEqual(passed["state"], "PASSED")
        self.assertIsNotNone(passed["evidence_bundle_digest"])
        qualified = cell(build(bundles=indexed([bundle(signoff="ACCEPT")])), LINUX, "E2E-01")
        self.assertEqual(qualified["state"], "QUALIFIED")
        partial = cell(build(bundles=indexed([bundle(signoff="PARTIAL")])), LINUX, "E2E-01")
        self.assertEqual(partial["state"], "PASSED")

    def test_bundle_for_another_row_does_not_count(self):
        matrix = build(bundles=indexed([bundle(row_id=WINDOWS)]))
        self.assertEqual(cell(matrix, LINUX, "E2E-01")["state"], "NOT_RUN")
        self.assertEqual(cell(matrix, WINDOWS, "E2E-01")["state"], "NOT_RUN")

    def test_stale_bundle(self):
        matrix = build(bundles=indexed([bundle(sha="a" * 40)]), sha="b" * 40)
        self.assertEqual(cell(matrix, LINUX, "E2E-01")["state"], "STALE")

    def test_invalid_bundle_on_disk_is_ignored(self):
        with tempfile.TemporaryDirectory(dir="/tmp") as tmp:
            good, bad = bundle(), bundle(journey="E2E-02")
            del bad["reviewer"]
            (Path(tmp) / "good.json").write_text(json.dumps(good))
            (Path(tmp) / "bad.json").write_text(json.dumps(bad))
            (Path(tmp) / "junk.json").write_text("not json")
            loaded = matrix_module.load_bundles(Path(tmp))
        self.assertEqual(list(loaded), [(LINUX, "E2E-01")])


class Declared(unittest.TestCase):
    def test_every_row_has_all_cells(self):
        matrix = build()
        expected = set(matrix_module.E2E_IDS) | set(matrix_module.CAPABILITIES)
        for row in matrix["rows"]:
            self.assertEqual(set(row["cells"]), expected)

    def test_exactly_two_rows_and_macos_unsupported(self):
        matrix = build()
        self.assertEqual([r["row_id"] for r in matrix["rows"]], [WINDOWS, LINUX])
        self.assertEqual(matrix["unsupported"][0]["os"], "macos")

    def test_three_rows_rejected(self):
        document = copy.deepcopy(ROWS)
        document["rows"].append(copy.deepcopy(document["rows"][0]))
        with tempfile.TemporaryDirectory(dir="/tmp") as tmp:
            path = Path(tmp) / "rows.json"
            path.write_text(json.dumps(document))
            with self.assertRaises(ValueError):
                matrix_module.load_rows(path)

    def test_output_is_deterministic(self):
        first = matrix_module.render_release_markdown(build("CONTAINER"))
        second = matrix_module.render_release_markdown(build("CONTAINER"))
        self.assertEqual(first, second)
        self.assertEqual(json.dumps(build(), sort_keys=True), json.dumps(build(), sort_keys=True))

    def test_default_output_is_not_the_retired_tree(self):
        self.assertTrue(matrix_module.DEFAULT_OUT.startswith("docs/release/"))
        source = (ROOT / "scripts" / "v1_platform_matrix.py").read_text(encoding="utf-8")
        self.assertNotIn("implementation/QUALIFICATION", source)


if __name__ == "__main__":
    unittest.main()
