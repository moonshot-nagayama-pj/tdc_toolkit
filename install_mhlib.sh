#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

base_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" > /dev/null && pwd -P)"

# Configuration
cache_dir="${HOME}/.cache/install_mhlib"
zip_file='MultiHarp150_160_V3_1.zip'

# Use openssl dgst -sha3-512 to produce these hashes.
#
# PicoQuant has changed the contents of zip files before without
# changing the name or version, so it's useful to keep track of older
# hashes that don't match what was downloaded as well.
#
# Hash for the zipfile as downloaded on 2024-02-17
# 36f7a85d8124bcb7e5a085f791f44e1af4ca4c7242749344a41a0730b8e754be0cb90d8d33967af68b0b5e2119d43bf7e9d25c2d3ab1db238bf6c321d13413d6
#
# Hash for the zipfile as downloaded on 2024-05-08
zip_sha3_512='13c93f320f086f7e554d3a7ff2c704767564aea13773d7cb5b9b6e4ec9587ebd593b7a7618e4ac3f3f319dd9bfd6fb0478f69be4a246e5cf188c4cca5d5e309b'

# Hashes for specific files we care about in ${library_dir}.
library_sha3_512='errorcodes.h a63ece5c5b49e8812e5f2173faf7d094e1a53a7e84ee0630e474172da4bcd7c6ca97a561a8a73099df569aecd6bf52b2a78a78043dc9a63ab986fb6139b8a504
mhdefin.h f0152e93cfaaaca006dc97fbc18479a09a465fa3016579c0610a7bcb7993b7d2a6512ad23172515ba1b11d91bcebdfa31b90dbc3ee2b483ee2b0b5e1876bd5af
mhlib.h c85ab86f9656eab33402c9940327d2fefc676fbcb3c80b57c3950c5fa05f0a7e2233b110f3456eac8d83100445c073b9181375d19c89b2185cf0a58550333aed
mhlib.so dcc2c30a4054cbdecd9085797097742e5a43110d19acb848ffb4fd50006b500d0d04d604e6449ad61c24d361efe9655df797520a58fcd1f37023297c8b1d3525'

# Relative location of the Linux driver tarball inside the zip file
mhlib_tar_gz='MHLib v3.1.0.0/Linux/MHLib_v3.1.0.0_64bit.tar.gz'

# Relative to the base project directory
library_dir='MHLib_v3.1.0.0_64bit/library'

# End configuration

stdmsg() {
  local IFS=' '
  printf '%s\n' "$*"
}

errmsg() {
  stdmsg "$*" 1>&2
}

trap_exit() {
  # It is critical that the first line capture the exit code. Nothing
  # else can come before this.  The exit code recorded here comes from
  # the command that caused the script to exit.
  local exit_status="$?"

  if [[ "${exit_status}" -ne 0 ]]; then
    errmsg 'The script did not complete successfully.'
    errmsg 'The exit code was '"${exit_status}"
  fi
}
trap trap_exit EXIT

on_path() {
  if ! type "${1}" > /dev/null 2>&1; then
    # shellcheck disable=SC2016
    errmsg 'The command '"${1}"' was not found on the $PATH. Cannot continue.'
    exit 1
  fi
}

check_prerequisites() {
  on_path curl
  on_path cut
  on_path openssl
  on_path tar
  on_path unzip

  if [[ -e "${library_dir}" ]]; then
    errmsg 'It appears that the library directory '"${library_dir}"' already exists. If you wish to replace it, manually delete it (and its parent) first. Exiting 0.'
    exit 0
  fi
}

function validate_hash() {
  local file="$1"
  local expected_hash="$2"
  local actual_hash
  actual_hash=$(openssl dgst -sha3-512 "${file}" | cut -d ' ' -f 2)
  if [[ "${expected_hash}" != "${actual_hash}" ]]; then
    errmsg "A file did not match its expected hash."
    errmsg
    errmsg "PicoQuant has been known to silently change the content of files before, while retaining the same file name and version number. It is worth investigating further, by diffing the contents of an older zip with the current one. We may need to update the hashes in this script."
    errmsg
    errmsg "File: ${file}"
    errmsg "Expected: ${expected_hash}"
    errmsg "Actual:   ${actual_hash}"
    exit 1
  fi
}

function download() {
  if ! [[ -f "${cached_zip_file}" ]]; then
    mkdir -p "${cache_dir}"
    curl --output "${cached_zip_file}" 'https://www.picoquant.com/dl_software/MultiHarp150/'"${zip_file}"
  fi
  validate_hash "${cached_zip_file}" "${zip_sha3_512}"
}

function extract() {
  # We want everything inside the tarball, so we can simply extract it
  # directly into the project.
  unzip -p "${cached_zip_file}" "${mhlib_tar_gz}" |
    tar -C "${base_dir}" -xz
}

function validate_library() {
  while IFS=' ' read -r file hash; do
    validate_hash "${library_dir}/${file}" "${hash}"
  done <<< "${library_sha3_512}"
}

function copy_library() {
  # TODO unclear why this is necessary, adopted from original script
  cp "${library_dir}"/mhlib.so "${library_dir}"/libmhlib.so
}

cached_zip_file="${cache_dir}/${zip_file}"

check_prerequisites
download
extract
validate_library
copy_library
