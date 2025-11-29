#!/usr/bin/env bash
set -e pipefail

# Ensure we're in the book directory
cd "$(dirname "$0")"

MDBOOK_VER="v0.4.31"

## Check if the right version of mdbook is already downloaded
if [ -f ./bin/mdbook ]; then
    if ./bin/mdbook --version | grep $MDBOOK_VER > /dev/null; then
        echo "bin/mdbook @ $MDBOOK_VER is already downloaded"
        exit 0
    fi
    echo "bin/mdbook is already downloaded but the wrong version, so downloading the correct version"
fi

# Download and extract the mdbook binary into the bin folder
mkdir -p bin && cd bin

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    TAR_FILE="mdbook-${MDBOOK_VER}-x86_64-unknown-linux-musl.tar.gz"
else
    TAR_FILE="mdbook-${MDBOOK_VER}-x86_64-apple-darwin.tar.gz"
fi

echo "Downloading $TAR_FILE..."
curl -OL "https://github.com/rust-lang/mdBook/releases/download/${MDBOOK_VER}/${TAR_FILE}"
tar -xf "${TAR_FILE}"
rm "${TAR_FILE}"

echo "Extracted to bin/mdbook"
cd ..