#!/bin/bash

cd "$(dirname "$0")"

# Skip if Go not found.
if ! command -v go >/dev/null 2>&1
then
    echo "Go could not be found"
    exit 0
fi

go_os=$1
go_arch=$2

echo "Go build for os: $go_os, arch: $go_arch"

# Setup vars for Android.
if [[ "$go_os" == "android" ]]; then
  # Setup NDK root path env.
  if [[ -z "$ANDROID_NDK_HOME" ]]; then
    NDK_VERSION=$(cat ../android/app/build.gradle | grep 'ndkVersion' | cut -d \' -f 2)
    ANDROID_NDK_HOME=$ANDROID_HOME/ndk/$NDK_VERSION
  fi
  # Setup NDK host path.
  if [[ "$(uname)" == "Darwin" ]]; then
    arch_host=darwin-x86_64
  else
    if [[ "$(uname -m)" == "aarch64" ]]; then
      arch_host=linux-arm64
    else
      arch_host=linux-x86_64
    fi
  fi
  # Setup NDK target arch.
  if [[ "$go_arch" == "arm64" ]]; then
    arch_bin_prefix=aarch64-linux-android
  elif [[ "$go_arch" == "arm" ]]; then
    arch_bin_prefix=armv7a-linux-androideabi
  else
    arch_bin_prefix=x86_64-linux-android
  fi

  # Build for current target.
  CGO_ENABLED=1 GOOS=$1 GOARCH=$2 CC="${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/${arch_host}/bin/${arch_bin_prefix}35-clang" CXX="${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/${arch_path}/bin/${arch_bin_prefix}35-clang++" go build -C "../tor/webtunnel" -ldflags="-s -w" -o "$3" code.gri.mw/WEB/webtunnel/main/client
else
  if [[ "$go_os" == "windows" ]]; then
    extra_flag="-H=windowsgui"
  fi
  GOOS=$1 GOARCH=$2 go build -C "../tor/webtunnel" -ldflags="-s -w ${extra_flag}" -o "$3" code.gri.mw/WEB/webtunnel/main/client
fi

