#!/bin/bash

usage="Usage: android.sh [type] [platform|version]\n - type: 'build', 'release'\n - platform, for 'build' type: 'v7', 'v8', 'x86'\n - optional version for 'release' (needed on MacOS), example: '0.2.2'"
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
BASEDIR=$(cd "$(dirname "$0")" && pwd)
cd "${BASEDIR}" || exit 1
cd ..

# Install platforms and tools
rustup target add armv7-linux-androideabi
rustup target add aarch64-linux-android
rustup target add x86_64-linux-android
cargo install cargo-ndk

success=1

### Build native code
function build_lib() {
  [[ $1 == "v7" ]] && arch=armeabi-v7a
  [[ $1 == "v8" ]] && arch=arm64-v8a
  [[ $1 == "x86" ]] && arch=x86_64

  sed -i -e 's/"cdylib","rlib"]/"rlib"]/g' Cargo.toml
  sed -i -e 's/"rlib"]/"cdylib","rlib"]/g' Cargo.toml

  # Fix for https://stackoverflow.com/questions/57193895/error-use-of-undeclared-identifier-pthread-mutex-robust-cargo-build-liblmdb-s
  # Uncomment lines below for the 1st build:
  #export CPPFLAGS="-DMDB_USE_ROBUST=0" && export CFLAGS="-DMDB_USE_ROBUST=0"
  #cargo ndk -t ${arch} build --profile release-apk
  #unset CPPFLAGS && unset CFLAGS
  cargo ndk -t "${arch}" -o android/app/src/main/jniLibs build --profile release-apk
  if [ $? -eq 0 ]
  then
    success=1
  else
    success=0
  fi

  sed -i -e 's/"cdylib","rlib"]/"rlib"]/g' Cargo.toml
  rm -f Cargo.toml-e
}

### Build application
function build_apk() {
  cd android || exit 1
  ./gradlew clean
  # Build signed apk if keystore exists
  if [ ! -f keystore.properties ]; then
    ./gradlew assembleDebug
    apk_path=app/build/outputs/apk/debug/app-debug.apk
  else
    ./gradlew assembleSignedRelease
    apk_path=app/build/outputs/apk/signedRelease/app-signedRelease.apk
  fi

  if [[ $1 == "" ]]; then
    # Launch application at all connected devices.
    for SERIAL in $(adb devices | grep -v List | cut -f 1);
      do
        adb -s "$SERIAL" install ${apk_path}
        sleep 1s
        adb -s "$SERIAL" shell am start -n mw.gri.android/.MainActivity;
    done
  else
    if [[ "$OSTYPE" != "darwin"* ]]; then
      version=$(grep -m 1 -Po 'version = "\K[^"]*' Cargo.toml)
    else
      version=v$2
    fi
    # Setup release file name
    name=grim-${version}-android-$1.apk
    [[ $1 == "arm" ]] && name=grim-${version}-android.apk
    rm -f "${name}"
    mv ${apk_path} "${name}"

    # Calculate checksum
    checksum=grim-${version}-android-$1-sha256sum.txt
    [[ $1 == "arm" ]] && checksum=grim-${version}-android-sha256sum.txt
    rm -f "${checksum}"
    sha256sum "${name}" > "${checksum}"
  fi

  cd ..
}

rm -rf android/app/src/main/jniLibs/*

if [[ $1 == "build" ]]; then
  build_lib "$2"
  [ $success -eq 1 ] && build_apk
else
  rm -rf target/release-apk
  rm -rf target/aarch64-linux-android
  rm -rf target/x86_64-linux-android
  rm -rf target/armv7-linux-androideabi

  build_lib "v7"
  [ $success -eq 1 ] && build_lib "v8"
  [ $success -eq 1 ] && build_apk "arm" "$2"
  rm -rf android/app/src/main/jniLibs/*
  [ $success -eq 1 ] && build_lib "x86"
  [ $success -eq 1 ] && build_apk "x86_64" "$2"
fi
