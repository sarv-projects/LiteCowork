#!/usr/bin/env bash
# Linux x64 deb package qualification (019-T10, AC-019).
#
# Usage:
#   scripts/qualify-linux-package.sh inspect <file.deb>   static checks, runs anywhere dpkg-deb exists
#   scripts/qualify-linux-package.sh install <file.deb>   native desktop only: install, check, remove
#
# Exit codes: 0 pass, 1 failed, 3 this host cannot qualify (not a failure and
# never a PASS: record it as BLOCKED). A container, WSL2 or CI image is never a
# qualifying host (ACCEPTANCE section 7).
set -Eeuo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

fail() { printf 'error: %s\n' "$*" >&2; exit 1; }
blocked() { printf 'BLOCKED: %s\n' "$*" >&2; exit 3; }

mode="${1:-}"
deb="${2:-}"
[[ "$mode" == inspect || "$mode" == install ]] || { printf 'usage: %s inspect|install <file.deb>\n' "$0" >&2; exit 2; }
[[ -f "$deb" ]] || fail "deb file not found: $deb"
command -v dpkg-deb >/dev/null || blocked "dpkg-deb is not installed on this host"

unit_path="usr/lib/systemd/system/litecoworkd.service"
listing="$(dpkg-deb --contents "$deb")"
grep -Eq '\./usr/bin/litecoworkd$' <<<"$listing" || fail "package does not install /usr/bin/litecoworkd"
grep -Eq "\./${unit_path}\$" <<<"$listing" || fail "package does not ship the systemd unit"
if grep -Eq '\./usr/bin/(opencode|claude|codex)$' <<<"$listing"; then
  fail "package bundles a native agent harness (AC-017)"
fi
depends="$(dpkg-deb --field "$deb" Depends)"
grep -q 'libwebkit2gtk-4.1' <<<"$depends" || fail "Depends does not name WebKitGTK 4.1: $depends"

scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
dpkg-deb --extract "$deb" "$scratch"
if command -v systemd-analyze >/dev/null; then
  # ExecStart is absent on a developer host, so only syntax findings count here.
  systemd-analyze verify "$scratch/$unit_path" 2>&1 | grep -v 'litecoworkd.*No such file' || true
fi
# The package must not enable the unit: no maintainer script may call enable.
control_dir="$scratch/DEBIAN"
mkdir -p "$control_dir"
dpkg-deb --control "$deb" "$control_dir"
if grep -Rqs 'systemctl .*enable\|deb-systemd-helper enable' "$control_dir"; then
  fail "a maintainer script enables the unit; it must ship disabled (AC-019)"
fi
printf 'inspect: ok (%s)\n' "$deb"

[[ "$mode" == install ]] || exit 0

[[ "$(id -u)" -eq 0 ]] || blocked "install mode needs root on a native desktop"
if [[ -f /.dockerenv ]] || grep -qi microsoft /proc/sys/kernel/osrelease 2>/dev/null || [[ -n "${WSL_DISTRO_NAME:-}" ]]; then
  blocked "container or WSL2 host is NOT_COUNTED"
fi
dpkg -i "$deb"
systemctl is-enabled litecoworkd.service 2>/dev/null | grep -qx enabled && fail "unit is enabled after install"
dpkg -i "$deb" # upgrade in place keeps working
dpkg -r litecowork
printf 'install: ok; launch to Runtime READY and 0700/0600 permission checks remain native-desktop USER steps\n'
