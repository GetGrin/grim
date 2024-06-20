#!/bin/bash

case $1 in
  x86|arm)
    ;;
  *)
  echo "Usage: release_macos.sh [platform]\n - platform: 'x86', 'arm'" >&2
  exit 1
esac

# Setup build directory
BASEDIR=$(cd $(dirname $0) && pwd)
cd ${BASEDIR}
cd ..

# Setup platform argument
[[ $1 == "x86" ]] && arch+=(x86_64-unknown-linux-gnu)
[[ $1 == "arm" ]] && arch+=(aarch64-unknown-linux-gnu)

# Start release build with zig linker for cross-compilation
cargo install cargo-zigbuild
cargo zigbuild --release --target ${arch}

# Create AppImage with https://github.com/AppImage/appimagetool
cp target/${arch}/release/grim linux/Grim.AppDir/AppRun
rm target/${arch}/release/*.AppImage
appimagetool linux/Grim.AppDir
mv *.AppImage target/${arch}/release/Grim-0.1.0-linux-$1.AppImage