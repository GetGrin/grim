#!/bin/bash
set -e

case $1 in
  x86_64|arm|universal)
    ;;
  *)
  echo "Usage: release_macos.sh [platform] [version]\n - platform: 'x86_64', 'arm', 'universal'" >&2
  exit 1
esac

if [[ "$OSTYPE" != "darwin"* ]]; then
  if [ -z ${SDKROOT+x} ]; then
    echo "MacOS SDKROOT is not set"
    exit 1
  else
    echo "Use MacOS SDK: ${SDKROOT}"
  fi
fi

# Setup build directory
BASEDIR=$(cd $(dirname $0) && pwd)
cd ${BASEDIR}
cd ..

# Setup platform
[[ $1 == "x86_64" ]] && arch+=(x86_64-apple-darwin)
[[ $1 == "arm" ]] && arch+=(aarch64-apple-darwin)

if [[ "$OSTYPE" != "darwin"* ]]; then
  # Start release build on non-MacOS with zig linker, requires zig 0.12.1
  rustup target add x86_64-apple-darwin
  rustup target add aarch64-apple-darwin
  [[ $1 == "universal" ]]; arch+=(universal2-apple-darwin)
  cargo install cargo-zigbuild
  cargo zigbuild --release --target ${arch}
else
  rustup target add ${arch}
  if [[ $1 == "universal" ]]; then
    cargo build --release --target x86_64-apple-darwin
    cargo build --release --target aarch64-apple-darwin
    lipo -create -output target/grim target/aarch64-apple-darwin/release/grim target/x86_64-apple-darwin/release/grim
  else
    cargo build --release --target ${arch}
  fi
fi

rm -f .intentionally-empty-file.o

yes | cp -rf target/${arch}/release/grim macos/Grim.app/Contents/MacOS

# Sign .app resources on change:
#rcodesign generate-self-signed-certificate
#rcodesign sign --pem-file cert.pem macos/Grim.app

# Create release package
FILE_NAME=grim-v$2-macos-$1.zip
cd macos
zip -r ${FILE_NAME} Grim.app
mv ${FILE_NAME} ../target/${arch}/release
