#!/usr/bin/env bash

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

base_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null && pwd -P)"
project_dir="$(cd "${base_dir}/.." >/dev/null && pwd -P)"

# Some Python static analysis tools require the target directories to
# be passed on the command line
python_source_dirs=(python sample_python)

# cd to the project directory before running anything else
cd "${project_dir}"

stdmsg "Running uv sync --dev..."
uv sync --dev

stdmsg "Activating virtual environment..."
source .venv/bin/activate

stdmsg "Building and verifying Rust component..."
cargo fmt --all -- --check
cargo clippy
cargo test
cargo build
maturin develop

stdmsg "Checking Python type hints with mypy..."
mypy

stdmsg "Running pylint..."
pylint --extension-pkg-allow-list=tdc_toolkit "${python_source_dirs[@]}"

stdmsg "Checking import formatting with isort..."
isort "${python_source_dirs[@]}" --check --diff

stdmsg "Checking Python code formatting with black..."
black --check --diff "${python_source_dirs[@]}"

# Run shellcheck
# Recursively loop through all files and find all files with .sh extension and run shellcheck
stdmsg "Checking shell scripts with shellcheck..."
find . -type d \( -path ./MHLib_v3.1.0.0_64bit -o -path ./.venv \) -prune -o -type f \( -name "*.sh" -o -name "*.bash" \) -print0 | xargs -0 shellcheck --enable=all --external-sources

stdmsg "Checking shell script formatting with shfmt..."
shfmt --diff --simplify bin "${python_source_dirs[@]}" src

stdmsg "Running ruff..."
ruff check .
