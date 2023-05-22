#!/bin/bash

case $1 in
  debug|release)
    ;;
  *)
  echo "Usage: build_and_run.sh [type] where is type is 'debug' or 'release'" >&2
  exit 1
esac

type=$1
[[ ${type} == "release" ]] && release_param+=(--release)
export CPPFLAGS="-DMDB_USE_ROBUST=0" && export CFLAGS="-DMDB_USE_ROBUST=0" && cargo ndk -t arm64-v8a build ${release_param[@]}

if [ $? -eq 0 ]
then
  yes | cp -f target/aarch64-linux-android/${type}/libgrim.so app/src/main/jniLibs/arm64-v8a
  ./gradlew clean
  ./gradlew build
  #./gradlew installDebug
  adb install app/build/outputs/apk/debug/app-debug.apk
  sleep 1s
  adb shell am start -n mw.gri.android/.MainActivity
fi