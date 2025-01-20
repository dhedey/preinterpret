#!/bin/bash

set -e

TRYBUILD=overwrite cargo test

# Remove the .wip folder created sometimes by try-build
rm -r ./wip