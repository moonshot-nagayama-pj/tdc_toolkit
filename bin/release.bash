#!/usr/bin/env bash

# Usage: release.bash [bump]
#
# If [bump] is not provided, the pre-release component (e.g. -alpha.0)
# will be dropped from the current version in Cargo.toml, and that
# value used for the release.
#
# If [bump] is provided, it must be one of 'major', 'minor', or
# 'patch'; the corresponding version component will be incremented
# before release.

set -o errexit
set -o nounset
set -o pipefail

# Print functions
stdmsg() {
  local IFS=' '
  printf '%s\n' "$*"
}

errmsg() {
  stdmsg "$*" 1>&2
}

# Trap exit handler
trap_exit() {
  # It is critical that the first line capture the exit code. Nothing
  # else can come before this. The exit code recorded here comes from
  # the command that caused the script to exit.
  local exit_status="$?"

  if [[ ${exit_status} -ne 0 ]]; then
    errmsg 'The script did not complete successfully.'
    errmsg 'The exit code was '"${exit_status}"
  fi
}
trap trap_exit EXIT

# Get the base and project directories
base_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null && pwd -P)"
project_dir="$(cd "${base_dir}/.." >/dev/null && pwd -P)"

get_cargo_toml_version() {
  cargo metadata --format-version 1 |
    jq --raw-output '.packages[] | select(.name=="tdc_toolkit") | .version'
}

remove_pre_release() {
  [[ ${1} =~ ([0-9]+[.][0-9]+[.][0-9]+)-.+ ]]
  stdmsg "${BASH_REMATCH[1]}"
}

validate_git_state() {
  # Ensure that the script is run from the main branch
  current_branch="$(git rev-parse --abbrev-ref HEAD)"
  if [[ ${current_branch} != "main" ]]; then
    errmsg "Error: This script must be run on the main branch." >&2
    exit 1
  fi

  # Ensure that there are no uncommitted changes
  if ! git diff-index --quiet HEAD --; then
    errmsg "Error: There are uncommitted changes in the repository. Please commit or stash them before running this script."
    exit 1
  fi
}

main() {
  validate_git_state
  stdmsg "Run checks before starting the release process..."
  "${base_dir}/check.bash"

  initial_cargo_toml_version=$(get_cargo_toml_version)
  initial_cargo_toml_version_no_prerelease=$(remove_pre_release "${initial_cargo_toml_version}")
  cargo set-version "${initial_cargo_toml_version_no_prerelease}"

  if [[ -n ${bump} ]]; then
    cargo set-version --bump "${bump}"
  fi

  release_version=$(get_cargo_toml_version)

  # Create branch 'release-<version>'
  branch_name="release-${release_version}"
  stdmsg "Creating release branch '${branch_name}'..."
  git checkout -b "${branch_name}"

  stdmsg "Committing release version..."
  git commit -am "Release version: ${release_version}"
  stdmsg "Creating tag 'v${release_version}'..."
  git tag -am "Release version: ${release_version}" "v${release_version}"
  git push origin "${branch_name}"
  git push origin "v${release_version}"

  # set-version does not allow version downgrades and the author is
  # very skeptical of allowing this despite the obvious use cases
  # (especially in the absence of a good way to bump to an alpha
  # version)
  #
  # https://github.com/killercup/cargo-edit/issues/863
  #
  # Instead, use a much more fragile approach.
  next_version=$(cargo set-version --dry-run --bump patch 2>&1 | sed -n -e 's/.*to \([0-9][.][0-9][.][0-9]\)/\1/p')
  next_version="${next_version}-alpha.0"
  cargo set-version "${next_version}"

  stdmsg "Committing next version..."
  git commit -am "Start next version: ${next_version}"
  git push origin "${branch_name}"

  # Create a pull request
  pull_request_url="https://github.com/moonshot-nagayama-pj/pnpq/pull/new/${branch_name}"
  stdmsg "Please check the pull request at ${pull_request_url}."
  if command -v xdg-open &>/dev/null; then
    xdg-open "${pull_request_url}"
  elif command -v open &>/dev/null; then
    open "${pull_request_url}"
  fi
}

# Ensure that the project root is the working directory
cd "${project_dir}"

# load any arguments
bump=''
if [[ $# -eq 1 ]]; then
  bump="${1}"
elif [[ $# -gt 1 ]]; then
  errmsg 'Too many positional parameters were passed. Not sure what to do. Exiting.'
  exit 64
fi

main

stdmsg "The script completed successfully."
