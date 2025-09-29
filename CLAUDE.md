# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Preinterpret is a Rust procedural macro crate that provides the `preinterpret!` macro - a code generation toolkit that simplifies declarative macro development. It combines functionality from quote, paste, and syn crates to enable:

- Variable definition and substitution with `[!set! #var = ...]` and `#var`
- Commands for concatenation, case conversion, and token manipulation like `[!ident! ...]`, `[!string! ...]`, `[!ident_snake! ...]`
- Control flow and parsing capabilities

## Plans and Tasks

There are various files in the `./plans` folder, the "current vision" is `2025-09-vision.md` and the "todo list" is `1_0-todo-list.md`.

## Architecture

The codebase is organized into several main modules:

- **src/lib.rs** - Main entry point and public API
- **src/interpretation/** - Core interpreter logic for processing commands and variables
  - `interpreter.rs` - Main interpreter implementation
  - `commands/` - Built-in command implementations (concat, transforming, control flow, etc.)
  - `bindings.rs` - Variable binding management
- **src/expressions/** - Expression evaluation system for advanced features
  - `evaluation/` - Expression evaluator with type resolution and frame management
- **src/transformation/** - Token stream transformation utilities
- **src/extensions/** - Helper traits and utilities for working with proc-macro2 tokens

## Development Commands

### Testing
- `cargo test` - Run all tests
- `cargo test --release` - Run tests in release mode
- `cargo miri test` - Run tests with Miri for memory safety

### Code Quality
- `./style-check.sh` - Check formatting and run clippy (equivalent to CI checks)
- `./style-fix.sh` - Auto-fix formatting and clippy issues
- `cargo fmt --check` - Check formatting only
- `cargo clippy --tests` - Run clippy lints

### Build & Check
- `cargo build` - Build the crate
- `cargo check` - Quick syntax/type check
- `./local-check-msrv.sh` - Test minimum supported Rust version (1.63)

### Documentation
- `cargo doc --open` - Build and open documentation
- `./book/test.sh` - Build the mdbook documentation

## Key Implementation Notes

This is a proc-macro crate (`proc-macro = true` in Cargo.toml) that:

- Uses syn 2.0 for parsing with full feature set enabled
- Supports minimum Rust version 1.63 (Edition 2021)
- Employs RefCell/Rc patterns for interior mutability in expression evaluation
- Uses trybuild for compile-time testing of macro expansions

The expression system (newer feature) provides advanced capabilities like mathematical operations and control flow, built on top of the core interpretation framework.

## Testing Strategy

Tests are located in the `tests/` directory and use trybuild for testing compilation failures. The CI runs tests on stable, beta, and nightly Rust versions with warnings treated as errors.

## Commit Strategy

* Run `style-fix.sh` before committing
* Use the conventional commits pattern for commit message prefixes.
