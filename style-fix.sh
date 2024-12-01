#!/bin/bash

set -e

cargo fmt;
cargo clippy --fix --tests;