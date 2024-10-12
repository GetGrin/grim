#!/bin/bash

case $1 in
  debug|build)
    ;;
  *)
  echo "Usage: build_run.sh [type] where is type is 'debug' or 'build'" >&2
  exit 1
esac

# Setup build directory
BASEDIR=$(cd "$(dirname $0)" && pwd)
cd "${BASEDIR}" || return
cd ..

# Build application
type=$1
[[ ${type} == "build" ]] && release_param+=(--release)
cargo --config profile.release.incremental=true build "${release_param[@]}"

# Start application
if [ $? -eq 0 ]
then
  path=${type}
  [[ ${type} == "build" ]] && path="release"
  ./target/"${path}"/grim
fi
