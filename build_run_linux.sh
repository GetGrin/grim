#!/bin/bash

case $1 in
  debug|release)
    ;;
  *)
  echo "Usage: build_and_run.sh [type] where is type is 'debug' or 'release'" >&2
  exit 1
esac

type=$1
[[ ${type} == "release" ]] && release_param+=(--release)
cargo build ${release_param[@]} --target x86_64-unknown-linux-gnu

if [ $? -eq 0 ]
then
  ./target/x86_64-unknown-linux-gnu/${type}/grim
fi