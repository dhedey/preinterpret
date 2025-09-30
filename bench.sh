#!/bin/bash

set -e

cd "$(dirname "$0")"

# NOTES:
# - The `benchmark` feature is required to be enabled for the benchmarks to run,
# - The total "cost" of using preintepret on a fresh build is the sum of:
#   - The compile time for preinterpret itself (and its dependencies, e.g. syn)
#   - For each item:
#     - Overhead invoking the macro
#     - The measured Parsing + Evaluation + Output time in the benchmark
# - So the benchmark is useful, but should be considered alongside the compile time
#   of preinterpret itself...
cargo bench --features benchmark;