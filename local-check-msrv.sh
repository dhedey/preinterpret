#!/bin/bash

set -e

rustup install 1.56.0
rm Cargo.lock && rustup run 1.56.0 cargo check
