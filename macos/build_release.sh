#!/bin/bash
set -e

case $2 in
  x86_64|arm|universal)
    ;;
  *)
  echo "Usage: release_macos.sh [version] [platform]\n - platform: 'x86_64', 'arm', 'universal'" >&2
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
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

[[ $2 == "x86_64" ]] && arch+=(x86_64-apple-darwin)
[[ $2 == "arm" ]] && arch+=(aarch64-apple-darwin)
[[ $2 == "universal" ]]; arch+=(universal2-apple-darwin)

# Start release build with zig linker, requires zig 0.12.1
cargo install cargo-zigbuild
cargo zigbuild --release --target ${arch}
rm -rf .intentionally-empty-file.o

yes | cp -rf target/${arch}/release/grim macos/Grim.app/Contents/MacOS

# Sign .app resources on change:
#rcodesign generate-self-signed-certificate
#rcodesign sign --pem-file cert.pem macos/Grim.app

# Create release package
FILE_NAME=grim-v$1-macos-$2.zip
rm -rf target/${arch}/release/${FILE_NAME}
cd macos
zip -r ${FILE_NAME} Grim.app
mv ${FILE_NAME} ../target/${arch}/release
