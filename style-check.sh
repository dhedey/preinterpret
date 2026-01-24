#!/bin/bash

set -e

cd "$(dirname "$0")"

cargo fmt --check;
cargo clippy --tests;