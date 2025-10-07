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
- Evaluation |     3ns
- Output     |     0ns

For loop adding up 1000 times
- Parsing    |    21ns
- Evaluation |  5704ns
- Output     |     0ns

For loop concatenating to stream 1000 tokens
- Parsing    |    27ns
- Evaluation |  3042ns
- Output     |   163ns

Lots of casts
- Parsing    |     7ns
- Evaluation |     3ns
- Output     |     0ns

Simple tuple impls
- Parsing    |    53ns
- Evaluation |   826ns
- Output     |    44ns

Accessing single elements of a large array
- Parsing    |    52ns
- Evaluation |  4283ns
- Output     |     0ns

Lazy iterator
- Parsing    |    48ns
- Evaluation |    34ns
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
- Parsing    |     7ns
- Evaluation |     3ns
- Output     |     0ns

For loop adding up 1000 times
- Parsing    |    19ns
- Evaluation |  4819ns
- Output     |     0ns

For loop concatenating to stream 1000 tokens
- Parsing    |    24ns
- Evaluation |  2582ns
- Output     |   129ns

Lots of casts
- Parsing    |     6ns
- Evaluation |     3ns
- Output     |     0ns

Simple tuple impls
- Parsing    |    47ns
- Evaluation |   725ns
- Output     |    34ns

Accessing single elements of a large array
- Parsing    |    45ns
- Evaluation |  3677ns
- Output     |     0ns

Lazy iterator
- Parsing    |    40ns
- Evaluation |    30ns
- Output     |     0ns
```