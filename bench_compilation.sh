#!/bin/bash

set -e

cd "$(dirname "$0")"

# Usage: run_build_with_timings <profile>
run_build_with_timings() {
    local profile="$1"

    echo
    echo ">> Checking compilation timings ($profile profile)"
    echo
    cargo clean

    cargo build --lib --profile="$profile" --timings > /dev/null

    # Find the most recent timing file (cargo creates timestamped files)
    local timing_file
    timing_file=$(ls -t target/cargo-timings/cargo-timing-*.html 2>/dev/null | head -1) || true

    if [ -n "$timing_file" ] && [ -f "$timing_file" ]; then
        echo

        # Extract total time from the HTML summary table
        local total_time
        total_time=$(grep -o '<td>Total time:</td><td>[^<]*</td>' "$timing_file" | grep -o '[0-9.]*s' | head -1) || true
        echo "- Total lib build time      | $total_time"

        # Extract preinterpret duration from UNIT_DATA JSON
        # Use sed to extract just the duration value after finding the preinterpret entry
        local preinterpret_duration
        preinterpret_duration=$(sed -n '/"name": "preinterpret"/,/}/p' "$timing_file" | grep -o '"duration": [0-9.]*' | grep -o '[0-9.]*') || true
        if [ -n "$preinterpret_duration" ]; then
            echo "- preinterpret compile time | ${preinterpret_duration}s"
        fi

        # Extract syn duration from UNIT_DATA JSON
        local syn_duration
        syn_duration=$(sed -n '/"name": "syn"/,/}/p' "$timing_file" | grep -o '"duration": [0-9.]*' | grep -o '[0-9.]*') || true
        if [ -n "$syn_duration" ]; then
            echo "- syn compile time          | ${syn_duration}s"
        fi
    fi
}

run_build_with_timings "dev"
run_build_with_timings "release"
