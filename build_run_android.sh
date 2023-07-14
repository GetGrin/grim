#!/bin/bash

usage="Usage: build_and_run.sh [type] [platform]\n - type: debug, release\n - platform: v7, v8"
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

# Setup release argument
type=$1
[[ ${type} == "release" ]] && release_param+=(--release)

# Setup platform argument
[[ $2 == "v7" ]] && platform_param+=(armeabi-v7a)
[[ $2 == "v8" ]] && platform_param+=(arm64-v8a)

# Setup platform path
[[ $2 == "v7" ]] && platform_path+=(armv7-linux-androideabi)
[[ $2 == "v8" ]] && platform_path+=(aarch64-linux-android)

export CPPFLAGS="-DMDB_USE_ROBUST=0" && export CFLAGS="-DMDB_USE_ROBUST=0" \
&& cargo ndk -t ${platform_param} build ${release_param[@]}

if [ $? -eq 0 ]
then
  yes | cp -f target/${platform_path}/${type}/libgrim.so app/src/main/jniLibs/${platform_param}
  ./gradlew clean
  ./gradlew build
  # Install on several devices
  for SERIAL in $(adb devices | grep -v List | cut -f 1);
    do
      adb -s $SERIAL install app/build/outputs/apk/debug/app-debug.apk
      sleep 1s
      adb -s $SERIAL shell am start -n mw.gri.android/.MainActivity;
  done
fi