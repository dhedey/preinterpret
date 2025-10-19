#!/bin/bash

set -e

cd "$(dirname "$0")"

rustup install 1.68
rm Cargo.lock && rustup run 1.68 cargo check
