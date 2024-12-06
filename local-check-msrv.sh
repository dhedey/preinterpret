#!/bin/bash

set -e

rustup install 1.61
rm Cargo.lock && rustup run 1.61 cargo check
