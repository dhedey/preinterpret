#!/bin/bash

set -e

cargo fmt --check;
cargo clippy --tests;