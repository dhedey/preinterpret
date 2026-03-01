#!/bin/bash

set -e

cd "$(dirname "$0")"

rustup install 1.71
rm Cargo.lock && rustup run 1.71 cargo check
