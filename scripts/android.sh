#!/bin/bash

usage="Usage: build_run_android.sh [type] [platform]\n - type: 'debug', 'release'\n - platform: 'v7', 'v8'"
case $1 in
  debug|release)
    ;;
  *)
  printf "$usage"
  exit 1
esac

case $2 in
  v7|v8)
    ;;
  *)
  printf "$usage"
  exit 1
esac

# Setup build directory
BASEDIR=$(cd $(dirname $0) && pwd)
cd ${BASEDIR}
cd ..

# Setup release argument
type=$1
[[ ${type} == "release" ]] && release_param="--profile release-apk"

# Setup platform argument
[[ $2 == "v7" ]] && arch+=(armeabi-v7a)
[[ $2 == "v8" ]] && arch+=(arm64-v8a)

# Setup platform path
[[ $2 == "v7" ]] && platform+=(armv7-linux-androideabi)
[[ $2 == "v8" ]] && platform+=(aarch64-linux-android)

# Install platform
[[ $2 == "v7" ]] && rustup target install armv7-linux-androideabi
[[ $2 == "v8" ]] && rustup target install aarch64-linux-android

# Build native code
cargo install cargo-ndk

rm -rf target/aarch64-linux-android
rm -rf target/armv7-linux-androideabi
mkdir -p android/app/src/main/jniLibs

sed -i -e 's/"rlib"/"rlib","cdylib"/g' Cargo.toml

# temp fix for https://stackoverflow.com/questions/57193895/error-use-of-undeclared-identifier-pthread-mutex-robust-cargo-build-liblmdb-s
success=0
export CPPFLAGS="-DMDB_USE_ROBUST=0" && export CFLAGS="-DMDB_USE_ROBUST=0"
cargo ndk -t ${arch} -o android/app/src/main/jniLibs build ${release_param}
unset CPPFLAGS && unset CFLAGS
cargo ndk -t ${arch} -o android/app/src/main/jniLibs build ${release_param}
if [ $? -eq 0 ]
then
  success=1
fi

sed -i -e 's/"rlib","cdylib"/"rlib"/g' Cargo.toml

# Build Android application and launch at all connected devices
if [ $success -eq 1 ]
then
  cd android

  # Setup gradle argument
  [[ $1 == "release" ]] && gradle_param+=(assembleRelease)
  [[ $1 == "debug" ]] && gradle_param+=(build)

  ./gradlew clean
  ./gradlew ${gradle_param}

  # Setup apk path
  [[ $1 == "release" ]] && apk_path+=(app/build/outputs/apk/release/app-release.apk)
  [[ $1 == "debug" ]] && apk_path+=(app/build/outputs/apk/debug/app-debug.apk)

  for SERIAL in $(adb devices | grep -v List | cut -f 1);
    do
      adb -s $SERIAL install ${apk_path}
      sleep 1s
      adb -s $SERIAL shell am start -n mw.gri.android/.MainActivity;
  done
fi