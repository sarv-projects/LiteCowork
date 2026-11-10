#!/usr/bin/env bash
# Signed SHA256SUMS for the Linux deb release artifacts (019-T10, B-43 item 4).
#
# Usage:
#   scripts/linux-release-checksums.sh sign   <dir> <gpg-key-id>   writes <dir>/SHA256SUMS and SHA256SUMS.asc
#   scripts/linux-release-checksums.sh verify <dir>                 checks the signature, then the checksums
#
# The key is taken from the caller's GPG keyring (GNUPGHOME). This script never
# reads, prints or stores private key material; a release job imports the key
# into a protected environment before calling it.
set -Eeuo pipefail

usage() {
  printf 'usage: %s sign <dir> <key-id> | verify <dir>\n' "$0" >&2
  exit 2
}

[[ $# -ge 2 ]] || usage
command_name="$1"
artifact_dir="$2"
[[ -d "$artifact_dir" ]] || { printf 'error: %s is not a directory\n' "$artifact_dir" >&2; exit 2; }

case "$command_name" in
  sign)
    [[ $# -eq 3 ]] || usage
    key_id="$3"
    cd "$artifact_dir"
    shopt -s nullglob
    artifacts=(*.deb)
    [[ ${#artifacts[@]} -gt 0 ]] || { printf 'error: no .deb artifact in %s\n' "$artifact_dir" >&2; exit 1; }
    sha256sum "${artifacts[@]}" > SHA256SUMS
    rm -f SHA256SUMS.asc
    gpg --batch --yes --local-user "$key_id" --armor --detach-sign --output SHA256SUMS.asc SHA256SUMS
    printf 'signed %d artifact(s)\n' "${#artifacts[@]}"
    ;;
  verify)
    [[ $# -eq 2 ]] || usage
    cd "$artifact_dir"
    [[ -f SHA256SUMS && -f SHA256SUMS.asc ]] || { printf 'error: SHA256SUMS or SHA256SUMS.asc is missing\n' >&2; exit 1; }
    gpg --batch --verify SHA256SUMS.asc SHA256SUMS
    sha256sum --check --strict SHA256SUMS
    ;;
  *)
    usage
    ;;
esac
