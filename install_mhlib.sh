#!/bin/bash
set -eu

ZIP_FILE=MultiHarp150_160_V3_1.zip
MHLIB_DIR=MHLib\ v3.1.0.0
DEMO_DIR=MultiHarp\ v3.1.0.0
MHLIB_TAR_GZ=$MHLIB_DIR/Linux/MHLib_v3.1.0.0_64bit.tar.gz
MHLIB=MHLib_v3.1.0.0_64bit

function download() {
  if [ ! -f "src/multiharp_toolkit_rs/$ZIP_FILE" ]; then
    curl -O https://www.picoquant.com/dl_software/MultiHarp150/$ZIP_FILE 
    mv $ZIP_FILE src/multiharp_toolkit_rs/$ZIP_FILE
  fi
}

function extract() {

  if [ ! -d "src/multiharp_toolkit_rs/$MHLIB_DIR" ]; then
    unzip "src/multiharp_toolkit_rs/$ZIP_FILE" -d src/multiharp_toolkit_rs
  fi

  if [ ! -d src/multiharp_toolkit_rs/$MHLIB ]; then
    tar -xzf "src/multiharp_toolkit_rs/$MHLIB_TAR_GZ" -C src/multiharp_toolkit_rs
  fi

}
function cleanup() {
  if [ -e "src/multiharp_toolkit_rs/$MHLIB_DIR" ]; then
    echo "delete $MHLIB_DIR"
    rm -rf "src/multiharp_toolkit_rs/$MHLIB_DIR"
  fi

  echo "check if $ZIP_FILE exists"
  if [ -e src/multiharp_toolkit_rs/$ZIP_FILE ]; then
    echo "delete $ZIP_FILE"
    rm -rf "src/multiharp_toolkit_rs/$ZIP_FILE"
  fi

  if [ -e "src/multiharp_toolkit_rs/$DEMO_DIR" ]; then
    echo "delete $DEMO_DIR"
    rm -rf "src/multiharp_toolkit_rs/$DEMO_DIR"
  fi
}

if [ -d src/multiharp_toolkit_rs/$MHLIB ]; then
  echo "$MHLIB already exists"
  cleanup
  exit 0
fi

download
extract
cleanup
