# TODO List

This is the to-do-list for 1.0, revised as-of @./2025-09-vision.md

## High priority

* Create `preinterpret::stream` and `preinterpret::run` and replace `preinterpret_assert_eq` with `run_assert_eq` / `stream_assert_eq`

## Method Calls

* TODO[operation-refactor]
    * BinaryOperation Migration
        * Add binary operation method resolution with type coercion/matching logic
            * OPTION A:
            * Untyped + ?int can be resolved like below
            * Int + Untyped can be resolved with a `MaybeTypedInt<X>`
            * OPTION B:
            * We implement addition at the `Integer` layer and do as we do now
        * Compute SHL/SHR on `Integer` using `.checked_shl(u32)` with an attempted cast to u32 via TryInto<u32>,
            i.e. we have a CoercedInt<u32> wrapper type which we use as the operand of the SHL/SHR operators
        * Migrate operators incrementally: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, etc.
        * No clone required for testing equality of streams, objects and arrays
    * CompoundAssignment Migration

```rust
// Possible UntypedInteger implementation
fn resolve_own_binary_operation(operation: &BinaryOperation) -> Option<MethodInterface> {
    Some(match operation {
        BinaryOperation::Paired(paired) => wrap_binary!([Op: operation, Span: output_span_range]
            (lhs: UntypedInteger, rhs: ExpressionInteger) -> ExecutionResult<ResolvedValue> {
                match rhs.value {
                    ExpressionIntegerValue::Untyped(rhs) => {
                        let lhs = lhs.parse_fallback()?;
                        let rhs = rhs.parse_fallback()?;
                        UntypedInteger::from_fallback(lhs.handle_paired_operation(operation, rhs)).to_resolved_value(output_span_range)
                    }
                    rhs => {
                        let lhs = lhs.to_kind(rhs.kind())?;
                        operation.evaluate(lhs, rhs)
                    }
                }
            }
        ),
        BinaryOperation::Integer(int_op) => wrap_binary!((lhs: UntypedInteger, rhs: ExpressionInteger) -> ExecutionResult<ExpressionValue> {
            lhs.handle_integer_binary_operation(rhs, int_op)
        }),
        _ => return None,
    })
}
```

## Span changes

* Remove span range from value:
    * Move it to a binding such as `Owned<T>` etc
    * Possibly can use `EvaluationError` (without a span!) inside a calculation, and adding the span in the evaluator (nb. it may still need to be able to propogate an `ExecutionInterrupt` internally)
* Except streams, which keep spans on literals/groups.
    * If someone wants to keep a value's span, they can keep it in a stream and coerce it; or store it as a tuple of a value with its span `[value, %[value]]`
* Bindings such as `Owned<X>` have a span, which:
    * Typically refers to the span of the preinterpret code that created the value/binding
    * In some cases (e.g. source literals) it can refer to a source span
    * And we can add a `spanned(%[..])` method which overrides the span of the binding (it'll have to return a `OwnedFixedSpan<ExpressionValue>` which has different handling)
    * We can have a `bool.assert(message, span?)`

* We can add a `spanned(%[..])` method which overrides the span of the binding (it'll have to return a `NoOverrideSpanOwned<ExpressionValue>` which has different handling in the `ToResolvedValue` trait)

# (Interpreted) Stream Literals

* Introduce `%[..]` and `%raw[..]` instead of `[!stream! ...]` and `[!raw! ...]`
* Replace `[!set!]` with `#(let x = %[ ... ])`

# Control flow expressions (ideally requires Stream Literals)

Create the following expressions:
* Blocks `{}`
  * ... and move `let` statement to replace `[!let!]`
* `if`, `else`
* `for`, `while`, `loop`
  * These return an array of values from each iteration (possibly with an optimization to skip if the value will be ignored)
* `continue`
* `break`
  * Can be used to return a value from a `loop` expression. If present, the loop changes to not return an array
  * Could maybe be used to (return from even a labelled block? https://blog.rust-lang.org/2022/11/03/Rust-1.65.0/#break-from-labeled-blocks)

* Consider `GroupedVariable` and `ExpressionBlock`:
  * `ExpressionBlock` with a `#` prefix `#{ .. }` shouldn't exist
  * In an output-stream  `#var` or `#(..)` are possible
  * In an stream-parser, only `#(..)` is possible, and should return `None`
    * If looking to match on a value, you should use  `@[EXACT({ tokens: %[] })]` instead
  * In an expression, `#x` and `#(..)` are NOT allowed - this avoids confusion such as below:
    * Confusion example: `let x; x = #(let x = 123; 5)`. This isn't allowed in normal rust because the inside is a `{ .. }` which defines a new scope.

... and remove their commands


## Scopes & Blocks (requires control flow expressions, or at least no `!let!` command)

* Scopes exist at compile time, e.g. as a `ScopeId(usize)` and include:
  * A definition about whether the scope is irrevertible or not
  * Variable definitions ...and the last use of them (as a value irrevertible - if at all) - that usage can do a take for free, like in Rust
  * A parent scope
  * Each variable usage can be tied back to a definition
  * Each let expression
* Spans are only kept from source inside streams, otherwise it refers to a binding
* At execution time, there needs to be some link between scope and stack frame

# Attempt Expression (requires Scopes & Blocks)

See @./2025-09-vision.md


# Parser Changes

First, read the @./2025-09-vision.md

* Manually search for transform and rename to parse in folder names and file.
* Initial changes:
  * Parsers no longer output to a stream.
  * Scopes/frames can have a parse stream associated with them.
    * This can be read/resolved (as the nearest parent) by parsers, even in expression blocks
  * Don't support `@(#x = ...)` - instead we can have `#(let x = @[STREAM ...])`

* Various other changes from the vision doc

* Named parsers:
  * `@[CAPTURE_INPUT_STREAM <expression>]`
    * This returns the input stream. It can capture the original tokens by using `let forked = input.fork()` and then `let end_cursor = input.end();` and then consuming `TokenTree`s from `forked` until `forked.cursor >= end_cursor` (making use of the PartialEq implementation)
* Remove `#x` and `#..x` as variable binding / parsers, instead use `#(__out.x = @TOKEN_TREE.flatten())` / `@x=REST`

* Review the existing named parsers, and implement the following named parsers
  * `@?(..)`, `@+(..)`, `@,+(..)`, `@*(..)`, `@+(..)`
  * Consider if `@LITERAL` should infer to a value
  * `@CURSOR` - outputs a token with a span for outputting errors. If at end of an inner stream, it outputs the ident `END` with the span of the closing bracket.
  * `@TOKEN_OR_GROUP_CONTENT` - Literal, Ident, Punct or None-group content (using `ParsedTokenTree`) - (do we need this?)
  * `@INFER_TOKEN_TREE` - Infers parsing as a value, falls back to Stream - OR maybe we just do `@TOKEN_TREE.infer()` - possibly this should also strip none-groups
  * `@INTEGER`
  * `@[ANY_GROUP ...]`
  * `@[FORK @{ ...parser... }]` the parser block creates a `commit=false` variable, if this is set to `commit=true` then it commits the fork.
  * `@[PEEK ...]` which does a `@[FORK ...]` internally but never commits... question: Should it use it error (for use in an `attempt` block)? Or return a bool? Maybe we have two?
  * `@[FIELDS { ... }]` and `@[SUBFIELDS { ... }]`
  * Add ability to add scope to interpreter state (see `Parsers Revisited`) and can then add:
    * `@[REPEATED { ... }]` (see below)
    * `@[UNTIL @{...}]` - takes an explicit parse block or parser. Reverts any state change if it doesn't match.
```rust
@[REPEATED({
    separator?: %[], // Could really be a parse stream, but it has to be a value here, and realistically it's not important. This is evaluated only once at the start.
    min?: 0,
    max?: 1000000,
}) { <block> }]
```

# Utility methods

Implement the following. (NB - do we need to add support for )

* All value kinds:
  * `is_none()`, and similarly for other value kinds
* Streams:
  * `is_ident()` and similarly for other stream
* Bools:
  * `assert("Error message", %[<span>])`
* Strings:
  * `error(%[span])`

# Error improvements

* Distinguish a runtime error from a coding error (e.g. parse error, or "no method of type")
  * The latter should not be caught by `attempt` blocks
* If method resolution fails, perhaps we try finding a method with that name on other types

# Coding challenges

Implement 10 leet-code challenges and 10 parsing challenges (e.g. from `syn` docs) to ensure that the language is sufficiently comprehensive to use in practice.

# Final considerations

* Should `preinterpret` should start in expression mode?
  => Or whether to have `preinterpet::stream` / `preinterpret::run` options?
  => Maybe `preinterpret::preinterpret` is marked as deprecated; starts in `stream` mode, and enables `[!set!]`?

* Add `preinterpret::macro` - can this be a declarative macro? Would be slightly more efficient, as it just needs to wrap a call to `preinterpret::stream` or `preinterpret::run`...
* Add `LiteralPattern` (wrapping a `Literal`)
* Add `Eq` support on composite types and streams
* Have UntypedInteger have an inner representation of either i128 or literal (and same with float)
* CastTarget expansion:
  * Add `as iterator` and uncomment the test at the end of `test_range()`
  * Support a CastTarget of `array` using `into_iterator()`.
  * Add `as ident` and `as literal` casting and support it for string, array and stream using concat recursive.
  * Add casts of any integer to char, via `char::from_u32(u32::try_from(x))`
* TODO check
* Check all `#[allow(unused)]` and remove any which aren't needed
* Add benches somehow...
  * Taking a look at https://github.com/dtolnay/quote/tree/master/benches - the benches don't test the right thing for us:
  * It's built to run two ways - as an executable, and a proc-macro library. When `main.rs` runs:
    * It triggers `quote_benchmark::run_quote_benchmark!(_)` which runs itself as a proc-macro, i.e. via compiling `lib.rs`
    * In `lib.rs`, `crate::benchmark` resolves to creating the `run_quote_benchmark` proc-macro, which internally has a call to `quote!`
      This call happens during compilation time, leaving `timer::time("macro", ..)` to actually time the `proc_macro::TokenStream::from` invocation
    * And then the `main()` runs in `main.rs` which calls `lib::quote` which is created by `crate::benchmark!` looping back to wrap it in the `quote` function defined in `main.rs`, and returns the `proc_macro2::TokenStream`.
  * Basically, the benchmarks both test how long the outputted code takes to execute, not how long the `quote!` invocation itself takes (which is harder, because that happens at compile time)....
  * For us, we can split up the time into:
    * (One-off compilation of the whole preinterpret crate)
    * Invocation overhead per macro (partially unknowable), quite small
    * Execution of the macro:
      * Conversion to token stream v2 (if it's anything)
      * Parsing
      * Execution
      * Conversion back to normal token stream (if it's anything)
    * We can create an optional `bench` feature which creates a `stream_bench` macro which tries to do the following things 1000 times:
      * Conversion
      * Parsing
      * Execution
      * Conversion back
    * And returns a tuple of the four averages `(a, b, c, d)` - then we can execute this / record this somewhere, and keep track of it over time.

NB: `define_command`, `define_parser`, and parsing of Rust code pushed to v1.1

# Finish converting all commands to expressions

E.G.
* `#(%[#x #y].ident_lower_camel())`
* `preinterpret.settings({..})` (`preinterpret` is available as a variable pre-bound on the root frame)
* .. possibly keep the v0.2 commands in `deprecated` mode?

# Write book / Docs

* Introduction
  * A Rust-like interpreted language with JS-like value types, built for code-generation
  * Native stream and parsing support, and the `attempt` expression
  * Comparison with proc-macros and crabtime
    * Compare to https://www.reddit.com/r/rust/comments/1j42fgi/media_introducing_eval_macro_a_new_way_to_write i.e. https://crates.io/crates/crabtime - thoughts on crabtime:
    => Looks great!
    => Likely has faster compile times compared with preinterpret
    => Why don't they use a cheap hash of the code as a cache key?
        - I imagine the rust macro system takes care of not re-running it if it changes
        - The cachability is a big win compared to preinterpret (although preinterpret is faster on first run)
    => I can't imagine the span-chasing / error messages are great, because everything is translated to/from strings between the processes
        ... I wonder if there's any way to improve this?  Plausibly you could use the proc-macro bridge encoding scheme as per https://blog.jetbrains.com/rust/2022/07/07/procedural-macros-under-the-hood-part-ii/ to send handles onwards, or even delegate directly somehow?
        - The spans are better in preinterpret
    => Parsing isn't really a thing - they're not going after full macro stuff, probably wise
* Use cases
* Examples
* Cheat-sheet
* Values & Streams
* Parsing
* Explanation of each expression, showing how it can be defined in terms of other building blocks

And then we need to:
* Update the README to point to the book
* Update the module docstring to point to the book.

# Write marketing materials

* Publish v1.0
* Flashy infographic like `crabtime` with some examples.

# Stream-return optimizations [OPTIONAL]

Expression evaluation can come with a `OutputStyle::AppendToStream(&mut OutputStream)` rather than a `OutputStyle::OwnedValue`.

This can be used to optimize, e.g.:

* Stream Literals
  * Could output direct to the output stream
* Loops:
  * If they have an output location of stream, and no `break <value>`, we can pass the stream through as each iteration's output location
  * If they have an output location of none: don't output anything
* `stream +=` and `stream.append(..)` methods

This means that this is low-clone:
```rust
#(for i in 0..100 { %[println!("Hello world!");] })
```

# Value expansions [OPTIONAL]

Consider:
* Do we want some kind of slice object? (see `TODO[range-refactor]`)
    * We can make `ExpressionValue` deref into `ExpressionRef`, e.g. `ExpressionRef::Array(<slice>)`
    * Then we can make `SharedValue(Ref<ExpressionRef>)`, which can be constructed from a `Ref<ExpressionValue>` with a map!
    * And similarly `MutableValue(RefMut<ExpressionRefMut>)`
* Using ResolvedValue in place of ExpressionValue e.g. inside arrays / objects, so that we can destructure `let (x, y) = (a, b)` without clone/take
    * But then we end up with nested references which can be confusing!
    * CONCLUSION: Maybe we don't want this - to destructure it needs to be owned anyway?
* Consider TODO[interpret-to-value] and whether to expand to `ResolvedValue` or `CopyOnWriteValue` instead of `OwnedValue`?
    => The main issue is if it interferes with taking mutable references, but it's possibly OK, would need to see if it's a confusing problem in practice... (e.g. `let b = a[0]; a.push(1)` if `b` is a reference to `a[0]` then this is a problem when we push to `a`)
    => If a mutable reference is created and there are pending references, the variable data RefCell could be replaced with a cloned value and then mutated... But this can be more expensive, because e.g. `let b = a[0]; a.push(1)` results in the whole array `a` being copied in the `CoW` case; but only the `a[0]` being cloned in the "clone on assign" case.
    => Maybe we just stick to assignments being Owned/Cloned as currently

* Support `#(x[..])` syntax for indexing streams, like with arrays
    * `#(x[0])` returns the value at that position of the stream (using `INFER_TOKEN_TREE`)
    * `#(x[0..3])` returns a TokenStream
    * `#(x[0..=3])` returns a TokenStream
