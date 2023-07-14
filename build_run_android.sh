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
[[ $2 == "v7" ]] && arch+=(armeabi-v7a)
[[ $2 == "v8" ]] && arch+=(arm64-v8a)

# Setup platform path
[[ $2 == "v7" ]] && platform+=(armv7-linux-androideabi)
[[ $2 == "v8" ]] && platform+=(aarch64-linux-android)

export CPPFLAGS="-DMDB_USE_ROBUST=0" && export CFLAGS="-DMDB_USE_ROBUST=0" \
&& cargo ndk -t ${arch} build ${release_param[@]}

if [ $? -eq 0 ]
then
  yes | mkdir app/src/mail/jniLibs && cp -f target/${platform}/${type}/libgrim.so app/src/main/jniLibs/${arch}
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