#!/usr/bin/env bash
set -e pipefail

# Ensure we're in the book directory
cd "$(dirname "$0")"

# Ensure mdbook has been downloaded
/usr/bin/env bash ./download-mdbook.sh 

./bin/mdbook watch --open
