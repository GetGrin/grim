#!/bin/bash

case $2 in
  x86_64|arm)
    ;;
  *)
  echo "Usage: release_linux.sh [platform] [version]\n - platform: 'x86_64', 'arm'" >&2
  exit 1
esac

# Setup build directory
BASEDIR=$(cd $(dirname $0) && pwd)
cd ${BASEDIR}
cd ..

# Setup platform argument
[[ $2 == "x86_64" ]] && arch+=(x86_64-unknown-linux-gnu)
[[ $2 == "arm" ]] && arch+=(aarch64-unknown-linux-gnu)

rustup target add ${arch}
cargo install cargo-zigbuild
cargo zigbuild --release --target ${arch}

# Create AppImage with https://github.com/AppImage/appimagetool
cp target/${arch}/release/grim linux/Grim.AppDir/AppRun
rm target/${arch}/release/*.AppImage
appimagetool linux/Grim.AppDir target/${arch}/release/grim-v$2-linux-$1.AppImage