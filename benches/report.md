# Internal Benchmarks

The overall effective performance of a procedural macro crate is a combination of:
* Compile time of the macro code itself
* Invocation overhead of the macro
* Runtime of the macro

Slightly weirdly, on a development build, the macro code is built and run in development mode, but on a release build, it's built and run in `--release` mode.

Therefore both dev and release build time and execution time are relevant for the effective feel of the crate in Dev/CI.

The below builds were done on a fast Macbook. Your CI will vary but will typically run in about double the times.

## Development Performance

### Compile time

Script: `cargo clean && cargo build --timings`

```text
// October 2025, run on Apple Silicon M2 Pro
=> syn v2.0.106 1.2s
=> preinterpret v0.2.0 2.2s
```

### Basic Execution Benchmarks

Script: `cargo bench --features benchmark --profile=dev`

```text
// October 2025, run on Apple Silicon M2 Pro
Trivial Sum
- Parsing    |     8ns
- Analysis   |     1ns
- Evaluation |     4ns
- Output     |     0ns

For loop adding up 1000 times
- Parsing    |    24ns
- Analysis   |    12ns
- Evaluation |  6123ns
- Output     |     0ns

For loop concatenating to stream 1000 tokens
- Parsing    |    29ns
- Analysis   |     9ns
- Evaluation |  3402ns
- Output     |   163ns

Lots of casts
- Parsing    |     7ns
- Analysis   |     1ns
- Evaluation |     4ns
- Output     |     0ns

Simple tuple impls
- Parsing    |    54ns
- Analysis   |    25ns
- Evaluation |   853ns
- Output     |    42ns

Accessing single elements of a large array
- Parsing    |    67ns
- Analysis   |    32ns
- Evaluation |  5797ns
- Output     |     0ns

Lazy iterator
- Parsing    |    45ns
- Analysis   |    21ns
- Evaluation |    41ns
- Output     |     0ns
```

## Release Performance

### Compile time

Script: `cargo clean && cargo build --timings --release`

```text
// October 2025, run on Apple Silicon M2 Pro
=> syn v2.0.106 1.1s
=> preinterpret v0.2.0 3.3s
```

### Basic Execution Benchmarks

Script: `cargo bench --features benchmark`

```text
// October 2025, run on Apple Silicon M2 Pro
Trivial Sum
- Parsing    |     8ns
- Analysis   |     1ns
- Evaluation |     4ns
- Output     |     0ns

For loop adding up 1000 times
- Parsing    |    25ns
- Analysis   |    12ns
- Evaluation |  5335ns
- Output     |     0ns

For loop concatenating to stream 1000 tokens
- Parsing    |    26ns
- Analysis   |     8ns
- Evaluation |  2996ns
- Output     |   125ns

Lots of casts
- Parsing    |     6ns
- Analysis   |     0ns
- Evaluation |     4ns
- Output     |     0ns

Simple tuple impls
- Parsing    |    49ns
- Analysis   |    22ns
- Evaluation |   765ns
- Output     |    33ns

Accessing single elements of a large array
- Parsing    |    46ns
- Analysis   |    20ns
- Evaluation |  4137ns
- Output     |     0ns

Lazy iterator
- Parsing    |    38ns
- Analysis   |    18ns
- Evaluation |    36ns
- Output     |     0ns
```