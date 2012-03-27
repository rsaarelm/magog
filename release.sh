#!/bin/bash

# This script assumes the current environment is a 64-bit Linux.

mkdir -p packages/

SRC_DIR=$(pwd)

BUILD_ID=$(git log --pretty=format:%h -1)

# $1: cmake arguments, $2: target name
linux_build() {
  BUILD_DIR=$(mktemp -d)
  pushd $BUILD_DIR
  cmake $1 $SRC_DIR
  make
  popd
  mv $BUILD_DIR/telos $2
  rm -rf $BUILD_DIR

  strip $2
  # Package it so it can be made downloadable while retaining the executable
  # flag, even though it's just a single file.
  tar -cjvf $2.tar.bz2 $2

  rm $2
  mv $2.tar.bz2 packages/
}

windows_build() {
  # Windows build needs some extra tricks to get the non-crosscompiled tools working
  BUILD_DIR=$(mktemp -d)
  pushd $BUILD_DIR
  mkdir build
  cd build
  cmake $SRC_DIR
  make tools
  cd ..
  mkdir xbuild
  cd xbuild
  cmake $1 -DCMAKE_TOOLCHAIN_FILE=$SRC_DIR/cmake_scripts/Toolchain-mingw32.cmake $SRC_DIR
  make
  popd
  mv $BUILD_DIR/xbuild/telos.exe $2
  rm -rf $BUILD_DIR

  strip $2

  # XXX: UPX'd binaries may trip virus scanners.
  upx $2

  mv $2 packages/
}

linux_build "-DCMAKE_BUILD_TYPE=MinSizeRel" "telos-linux64-$BUILD_ID"
linux_build "-DCMAKE_BUILD_TYPE=MinSizeRel -DCMAKE_C_FLAGS=-m32 -DCMAKE_CXX_FLAGS=-m32" "telos-linux32-$BUILD_ID"
windows_build "-DCMAKE_BUILD_TYPE=MinSizeRel" "telos-$BUILD_ID.exe"

