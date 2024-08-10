#!/bin/bash

# Usage to bump version
# ./version.sh patch
# ./version.sh minor
# ./version.sh major

# Setup base directory
BASEDIR=$(cd $(dirname $0) && pwd)
cd ${BASEDIR}
cd ..

# Exit script if command fails or uninitialized variables used
set -euo pipefail

# ==================================
# Verify repo is clean
# ==================================

# List uncommitted changes and
# check if the output is not empty
if [ -n "$(git status --porcelain)" ]; then
  # Print error message
  printf "\nError: repo has uncommitted changes\n\n"
  # Exit with error code
  exit 1
fi

# ==================================
# Get latest version from git tags
# ==================================

# List git tags sorted lexicographically
# so version numbers sorted correctly
GIT_TAGS=$(git tag --sort=version:refname)

# Get last line of output which returns the
# last tag (most recent version)
GIT_TAG_LATEST=$(echo "$GIT_TAGS" | tail -n 1)

# If no tag found, default to v0.1.0
if [ -z "$GIT_TAG_LATEST" ]; then
  GIT_TAG_LATEST="v0.1.0"
fi

# Strip prefix 'v' from the tag to easily increment
GIT_TAG_LATEST=$(echo "$GIT_TAG_LATEST" | sed 's/^v//')

# ==================================
# Increment version number
# ==================================

# Get version type from first argument passed to script
VERSION_TYPE="${1-}"
VERSION_NEXT=""

if [ "$VERSION_TYPE" = "patch" ]; then
  # Increment patch version
  VERSION_NEXT="$(echo "$GIT_TAG_LATEST" | awk -F. '{$NF++; print $1"."$2"."$NF}')"
elif [ "$VERSION_TYPE" = "minor" ]; then
  # Increment minor version
  VERSION_NEXT="$(echo "$GIT_TAG_LATEST" | awk -F. '{$2++; $3=0; print $1"."$2"."$3}')"
elif [ "$VERSION_TYPE" = "major" ]; then
  # Increment major version
  VERSION_NEXT="$(echo "$GIT_TAG_LATEST" | awk -F. '{$1++; $2=0; $3=0; print $1"."$2"."$3}')"
else
  # Print error for unknown versioning type
  printf "\nError: invalid VERSION_TYPE arg passed, must be 'patch', 'minor' or 'major'\n\n"
  # Exit with error code
  exit 1
fi

# ==================================
# Update Android build.gradle file
# and package version at Cargo.toml
# ==================================

# Update version in build.gradle

sed -i 's/versionName [0-9a-zA-Z -_]*/versionName "'"$VERSION_NEXT"'"/' android/app/build.gradle

# Update version in Cargo.toml
sed -i "s/^version = .*/version = \"$VERSION_NEXT\"/" Cargo.toml

# Update Cargo.lock as this changes when
# updating the version in your manifest
cargo update -p grim

# Commit the changes
git add .
git commit -m "release: v$VERSION_NEXT"

# ==================================
# Create git tag for new version
# ==================================

# Create a tag at master branch
git tag "v$VERSION_NEXT" master