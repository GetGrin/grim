#!/bin/bash

usage="Usage: android.sh [type] [platform]\n - type: 'build', 'release', ''\n - platform, for build type: 'v7', 'v8', 'x86'"
case $1 in
  build|release)
    ;;
  *)
  printf "$usage"
  exit 1
esac

if [[ $1 == "build" ]]; then
  case $2 in
    v7|v8|x86)
      ;;
    *)
    printf "$usage"
    exit 1
  esac
fi

# Setup build directory
BASEDIR=$(cd $(dirname $0) && pwd)
cd ${BASEDIR}
cd ..

# Install platforms and tools
rustup target add armv7-linux-androideabi
rustup target add aarch64-linux-android
rustup target add x86_64-linux-android
cargo install cargo-ndk

success=0

### Build native code
function build_lib() {
  [[ $1 == "v7" ]] && arch=(armeabi-v7a)
  [[ $1 == "v8" ]] && arch=(arm64-v8a)
  [[ $1 == "x86" ]] && arch=(x86_64)

  sed -i -e 's/"rlib"/"cdylib","rlib"/g' Cargo.toml

  # Fix for https://stackoverflow.com/questions/57193895/error-use-of-undeclared-identifier-pthread-mutex-robust-cargo-build-liblmdb-s
  export CPPFLAGS="-DMDB_USE_ROBUST=0" && export CFLAGS="-DMDB_USE_ROBUST=0"
  cargo ndk -t ${arch} build --profile release-apk
  unset CPPFLAGS && unset CFLAGS
  cargo ndk -t ${arch} -o android/app/src/main/jniLibs build --profile release-apk
  if [ $? -eq 0 ]
  then
    success=1
  fi

  sed -i -e 's/"cdylib","rlib"/"rlib"/g' Cargo.toml
}

### Build application
function build_apk() {
  version=$(grep -m 1 -Po 'version = "\K[^"]*' Cargo.toml)

  cd android
  ./gradlew clean
  ./gradlew assembleSignedRelease

  # Setup release file name
  if [ -n $1 ]; then
    rm -rf grim-${version}-$1.apk
    mv app/build/outputs/apk/signedRelease/app-signedRelease.apk grim-${version}-$1.apk
  fi

  cd ..
}

# Remove build targets
rm -rf target/release-apk
rm -rf target/aarch64-linux-android
rm -rf target/x86_64-linux-android
rm -rf target/armv7-linux-androideabi
rm -rf android/app/src/main/jniLibs/*

if [[ $1 == "build" ]]; then
  build_lib $2
  [ $success -eq 1 ] && build_apk

  # Launch application at all connected devices.
  for SERIAL in $(adb devices | grep -v List | cut -f 1);
    do
      adb -s $SERIAL install ${apk_path}
      sleep 1s
      adb -s $SERIAL shell am start -n mw.gri.android/.MainActivity;
  done
else
  build_lib "v7"
  [ $success -eq 1 ] && build_lib "v8"
  [ $success -eq 1 ] && build_apk "arm"
  rm -rf android/app/src/main/jniLibs/*
  [ $success -eq 1 ] && build_lib "x86"
  [ $success -eq 1 ] && build_apk "x86_64"
fi