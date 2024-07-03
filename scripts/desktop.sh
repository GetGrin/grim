#!/bin/bash

case $1 in
  debug|release)
    ;;
  *)
  echo "Usage: build_run.sh [type] where is type is 'debug' or 'release'" >&2
  exit 1
esac

# Setup build directory
BASEDIR=$(cd $(dirname $0) && pwd)
cd ${BASEDIR}
cd ..

# Build application
type=$1
[[ ${type} == "release" ]] && release_param+=(--release)
cargo build ${release_param[@]}

# Start application
if [ $? -eq 0 ]
then
  ./target/${type}/grim-bin
fi