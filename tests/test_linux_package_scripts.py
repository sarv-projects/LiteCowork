"""Linux deb packaging policy and signed checksum tests (019-T10)."""
import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHECKSUMS = ROOT / "scripts" / "linux-release-checksums.sh"
QUALIFY = ROOT / "scripts" / "qualify-linux-package.sh"
CONF = ROOT / "apps" / "litecowork-ui" / "src-tauri" / "tauri.conf.json"


class DebConfig(unittest.TestCase):
    def test_deb_declares_webkit_dependency_and_disabled_unit(self):
        deb = json.loads(CONF.read_text(encoding="utf-8"))["bundle"]["linux"]["deb"]
        self.assertTrue(any("libwebkit2gtk-4.1" in d for d in deb["depends"]))
        (dest, source), = deb["files"].items()
        self.assertEqual(dest, "/usr/lib/systemd/system/litecoworkd.service")
        self.assertTrue((CONF.parent / source).resolve().is_file())
        # Shipping the file does not enable it; no maintainer script is declared.
        for key in ("postInstallScript", "preInstallScript"):
            self.assertNotIn(key, deb)

    def test_scripts_are_lf_with_shebang(self):
        for script in (CHECKSUMS, QUALIFY):
            data = script.read_bytes()
            self.assertTrue(data.startswith(b"#!/usr/bin/env bash\n"))
            self.assertNotIn(b"\r", data)


@unittest.skipUnless(
    shutil.which("gpg") and shutil.which("gpg-agent") and shutil.which("bash"),
    "gpg and gpg-agent required",
)
class SignedChecksums(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(dir="/tmp")
        self.addCleanup(self.tmp.cleanup)
        self.home = Path(self.tmp.name) / "gnupg"
        self.home.mkdir(mode=0o700)
        self.env = {**os.environ, "GNUPGHOME": str(self.home)}
        subprocess.run(
            ["gpg", "--batch", "--passphrase", "", "--quick-generate-key",
             "release-test@example.invalid", "ed25519", "sign", "1d"],
            env=self.env, check=True, capture_output=True,
        )
        self.art = Path(self.tmp.name) / "art"
        self.art.mkdir()
        (self.art / "litecowork_0.1.0_amd64.deb").write_bytes(b"package-bytes")

    def run_script(self, *args):
        return subprocess.run(
            ["bash", str(CHECKSUMS), *args], env=self.env, capture_output=True, text=True
        )

    def sign(self):
        return self.run_script("sign", str(self.art), "release-test@example.invalid")

    def test_sign_then_verify(self):
        self.assertEqual(self.sign().returncode, 0)
        self.assertEqual(self.run_script("verify", str(self.art)).returncode, 0)

    def test_modified_package_fails_verification(self):
        self.assertEqual(self.sign().returncode, 0)
        (self.art / "litecowork_0.1.0_amd64.deb").write_bytes(b"tampered")
        self.assertNotEqual(self.run_script("verify", str(self.art)).returncode, 0)

    def test_modified_checksum_file_fails_signature(self):
        self.assertEqual(self.sign().returncode, 0)
        sums = self.art / "SHA256SUMS"
        sums.write_text("0" * 64 + "  litecowork_0.1.0_amd64.deb\n")
        self.assertNotEqual(self.run_script("verify", str(self.art)).returncode, 0)

    def test_missing_signature_fails(self):
        self.assertEqual(self.sign().returncode, 0)
        (self.art / "SHA256SUMS.asc").unlink()
        self.assertNotEqual(self.run_script("verify", str(self.art)).returncode, 0)

    def test_no_deb_refuses_to_sign(self):
        (self.art / "litecowork_0.1.0_amd64.deb").unlink()
        self.assertNotEqual(self.sign().returncode, 0)


class QualifyScript(unittest.TestCase):
    def test_missing_deb_fails_and_missing_tool_is_blocked_not_pass(self):
        result = subprocess.run(
            ["bash", str(QUALIFY), "inspect", "/nonexistent.deb"], capture_output=True, text=True
        )
        self.assertEqual(result.returncode, 1)
        if not shutil.which("dpkg-deb"):
            with tempfile.NamedTemporaryFile(suffix=".deb") as f:
                result = subprocess.run(
                    ["bash", str(QUALIFY), "inspect", f.name], capture_output=True, text=True
                )
            self.assertEqual(result.returncode, 3)
            self.assertIn("BLOCKED", result.stderr)


if __name__ == "__main__":
    unittest.main()
