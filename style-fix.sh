#!/bin/bash

set -e

cd "$(dirname "$0")"

cargo fmt;
cargo clippy --fix --tests --allow-dirty --allow-staged;