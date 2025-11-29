#!/usr/bin/env bash
set -e pipefail

# Ensure we're in the book directory
cd "$(dirname "$0")"

# Ensure mdbook has been downloaded
/usr/bin/env bash ./download-mdbook.sh 

# TODO
# Replace with mdbook test once it actually works with external libraries
# See issue: https://github.com/rust-lang/mdBook/issues/394#issuecomment-2234216353
# I'm hopeful this will work once https://github.com/rust-lang/mdBook/pull/2503 is merged
bin/mdbook build
