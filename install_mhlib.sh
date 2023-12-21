#!/bin/bash
set -eux

ZIP_FILE=MultiHarp150_160_V3_1.zip
MHLIB_DIR=MHLib\ v3.1.0.0
DEMO_DIR=MultiHarp\ v3.1.0.0
MHLIB_TAR_GZ=$MHLIB_DIR/Linux/MHLib_v3.1.0.0_64bit.tar.gz
MHLIB=MHLib_v3.1.0.0_64bit

function download() {
  if [ ! -f "$ZIP_FILE" ]; then
    curl -O https://www.picoquant.com/dl_software/MultiHarp150/$ZIP_FILE 
  fi
}

function extract() {


  if [ ! -d $MHLIB ]; then
  if [ ! -d "$MHLIB_DIR" ]; then
    unzip "$ZIP_FILE" 
  fi
    tar -xzf "$MHLIB_TAR_GZ" 
  fi

  if [ ! -f $MHLIB/library/libmhlib.so ]; then
    cp $MHLIB/library/mhlib.so $MHLIB/library/libmhlib.so
  fi

}
function cleanup() {
  if [ -e "$MHLIB_DIR" ]; then
    echo "delete $MHLIB_DIR"
    rm -rf "$MHLIB_DIR"
  fi

  echo "check if $ZIP_FILE exists"
  if [ -e $ZIP_FILE ]; then
    echo "delete $ZIP_FILE"
    rm -rf "$ZIP_FILE"
  fi

  if [ -e "$DEMO_DIR" ]; then
    echo "delete $DEMO_DIR"
    rm -rf "$DEMO_DIR"
  fi
}

if [ -f $MHLIB/library/libmhlib.so ]; then
  echo "$MHLIB already exists"
  cleanup
  exit 0
fi

if [ ! -d $MHLIB ]; then
  download
fi
extract
cleanup
