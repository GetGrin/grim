#!/bin/bash

case $1 in
  x86|arm)
    ;;
  *)
  echo "Usage: release_macos.sh [platform]\n - platform: 'x86', 'arm'" >&2
  exit 1
esac

if [[ ! -v SDKROOT ]]; then
    echo "MacOS SDKROOT is not set"
    exit 1
elif [[ -z "SDKROOT" ]]; then
    echo "MacOS SDKROOT is set to the empty string"
    exit 1
else
    echo "Use MacOS SDK: ${SDKROOT}"
fi

# Setup build directory
BASEDIR=$(cd $(dirname $0) && pwd)
cd ${BASEDIR}
cd ..

# Setup platform argument
[[ $1 == "x86" ]] && arch+=(x86_64-apple-darwin)
[[ $1 == "arm" ]] && arch+=(aarch64-apple-darwin)

# Start release build with zig linker for cross-compilation
cargo install cargo-zigbuild
cargo zigbuild --release --target ${arch}
rm .intentionally-empty-file.o
yes | cp -rf target/${arch}/release/grim scripts/macos/Grim.app/Contents/MacOS/grim-bin

### Sign .app before distribution:
### rcodesign generate-self-signed-certificate
### rcodesign sign --pem-file test.pem scripts/macos/Grim.app

# Create release package
FILE_NAME=Grim-0.1.0-macos-$1.zip
rm target/${arch}/release/${FILE_NAME}
cd scripts/macos
zip -r ${FILE_NAME} Grim.app
mv ${FILE_NAME} ../../target/${arch}/release