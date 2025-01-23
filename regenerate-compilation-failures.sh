#!/bin/bash

set -e

TRYBUILD=overwrite cargo test compilation_failures

# Remove the .wip folder created sometimes by try-build
if [ -d "./wip" ]; then
  rm -r ./wip
fi
