#!/bin/bash

set -e

cd "$(dirname "$0")"

rustup install 1.63
rm Cargo.lock && rustup run 1.63 cargo check
