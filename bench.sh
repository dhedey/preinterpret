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
#   of preinterpret itself - see `bench_compilation.sh` for that.

echo Preparing benchmark builds...
cargo build --bench basic --features benchmark --profile=dev
cargo build --bench basic --features benchmark --profile=release

# Note - the benchmarks themselves actually run *during* build time
# So at this point (courtesy of the build cache) we already have the benchmark results.
# But we print them in a block to make it easier to copy-paste them

echo
echo "Executing pre-run benchmark (dev profile)..."
cargo bench --bench basic --features benchmark --profile=dev

echo
echo "Executing pre-run benchmark (release profile)..."
cargo bench --bench basic --features benchmark --profile=release

echo
echo "If you want to get fresh results, run 'cargo clean' and re-run this script"