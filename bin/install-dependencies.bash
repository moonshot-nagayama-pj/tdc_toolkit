#!/usr/bin/env bash

# Used to install dependencies needed to run GitHub Actions

set -o errexit
set -o nounset
set -o pipefail

base_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null && pwd -P)"

# Print functions
stdmsg() {
  local IFS=' '
  printf '%s\n' "$*"
}

errmsg() {
  stdmsg "$*" 1>&2
}

tmp_dir_install_rye=$(mktemp --tmpdir -d 'install-rye.XXXXXXXX')

# Trap exit handler
trap_exit() {
  # It is critical that the first line capture the exit code. Nothing
  # else can come before this. The exit code recorded here comes from
  # the command that caused the script to exit.
  local exit_status="$?"

  rm -rf "${tmp_dir_install_rye}"

  if [[ ${exit_status} -ne 0 ]]; then
    errmsg 'There is an error installing the dependencies.'
    exit 1
  fi
}
trap trap_exit EXIT

# Some dependencies may already be installed; see the list
# of preinstalled tools:
# https://github.com/actions/runner-images/issues/9848
#
# Running apt-get install for a package that is already installed will
# not cause problems and is very fast, as it is essentially a no-op
# unless the --reinstall flag is passed.
sudo apt-get update
sudo apt-get install -y shfmt llvm-dev libclang-dev clang

# Ensure optional Rust components present
rustup component add clippy
rustup component add rustfmt

"${base_dir}"/install-rye.bash "${tmp_dir_install_rye}"
