#!/bin/bash

HOST=https://code.gri.mw
REPO_NAME=$1
TAG=$2
DOWNLOAD_URL=${HOST}/${REPO_NAME}/releases/download/${TAG}

FILES=( "grim-${TAG}-android.apk" "grim-${TAG}-android-x86_64.apk" "grim-${TAG}-linux-arm.AppImage" "grim-${TAG}-linux-x86_64.AppImage" "grim-${TAG}-macos.zip" "grim-${TAG}-win-x86_64.zip" )

# Download release files
for f in "${FILES[@]}"; do
  wget -q ${DOWNLOAD_URL}/${f}
  echo Downloading ${f}...
  while [ ! -f ${f} ]; do
    sleep 5
    echo Retry ${f}...
    wget -q ${DOWNLOAD_URL}/${f}
  done
done

# Save release notes
INFO_URL=${HOST}/api/v1/repos/${REPO_NAME}/releases/tags/${TAG}
curl -s "${INFO_URL}" | jq -r '.body' > release_notes.txt