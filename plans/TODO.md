# 1.0 Todo List

This is the to-do-list for 1.0, revised as-of @./2025-09-vision.md

## (Interpreted) Stream Literals

- [x] Introduce `%[..]` and the corresponding pattern
- [x] Introduce `%{}` instead of `{}` and the corresponding pattern
- [x] Introduce `%raw[..]` (we don't need such a pattern, as it'll be equal to `@[EXACT(%raw[...])`, but perhaps we should advise of this)
- [x] Remove `[!raw! ...]` and replace with `#..(%raw[...])`
- [x] Remove `#..` because it's confusing and remove grouping from `#`. Instead have a `group()` method on streams. Search for all `#..` to remove. In future, perhaps `@EXPR` could add a group around it at a parser layer.
- [x] Allow %[], %raw[] directly in token streams, and search for all `#(%group` and `#(%raw` to amend.
- [x] Remove `[!stream! ...]` and replace with `%[...]`
- [x] Remove `[!set!]` and replace with `#(let x = %[ ... ])`
- [x] Remove `[!ignore!]` and replace with `#(let _ = %[ ... ])`
- [x] Add `%group[]` and remove `as group` and `[!group! ..]`
- [x] Remove `[!let!]` and replace with `#(let %[..] = %[..])`
- [x] Remove `StopCondition`
- [x] Fix the to_debug_string to add `%raw[..]` around punct groups including `#` or `%`
- [x] Fix grammar-peeking of none-groups so that e.g. `[!reinterpret! %group[#]var_name]` works
- [x] `4usize.to_string()` should return `4` but `debug_string` should return 4usize
- [x] Simplify the EXACT parser
- [x] Migrate `ErrorCommand`
- [x] Migrate `ReinterpretCommand`
- [x] Fix naming of `.string()` etc to `.to_string()`, `.to_stream()`, `.to_group()`, `.to_debug_string()`
- [x] Migrate the `Concat & Type Convert Commands` and `Concat & String Convert Commands`, and decide on method names for e.g. `string()`
  * `.to_literal()`, `.concat()`, `.to_ident()`, `.to_ident_camel()`, `.to_ident_snake()`, `.to_ident_upper_snake()`
  * String: `.to_lowercase()` <- maybe shouldn't concat, others can: `.to_snake_case()`, `.capitalize()` etc
- [x] Migrate `!is_empty!` and `!length!`
- [x] Migrate `!zip!` and `!zip_truncated!`
- [x] Implement some kind of support for variable length methods
- [x] Migrate `!intersperse!`
- [x] Move `.to_ident()` and friends to `to_value` via `.to_stream()`
- [x] Migrate `!split!` and `!comma_split!`
- [x] Add tests for object and string as iterables and test length
- [x] Add `into_iter()` and `to_vec()` to iterable
- [x] Add `next()`, `take(N) -> Vec` and `skip(N)` to iterator, and add tests

## Span changes

- [x] Remove spans from `ExpressionValue`, leave only on bindings or strem contents
- [x] Add `xx.with_span(%[])` which changes the value to stream and replaces the span of every token at the top iteration level of the stream

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
    * Ensure all `TODO[operation-refactor]` are done

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

## Control flow expressions (ideally requires Stream Literals)

Create the following expressions:
- [x] Blocks `{}`
- [x] `if`, `else`, `for`, `while`, `loop`
- [x] `continue`, `break`
- [x] Refactors:
  - [x] Rename `SourceExpression` => `Expression`, and inline the leaf parsing
  - [x] Rename `interpreted_stream.rs` to `output_stream.rs`
  - [x] Change `InterpretTo` to use `&self`
  - [x] Rename `InterpretToValue` to `Evaluate` and make it use `&self`

## Scopes & Blocks (requires control flow expressions, or at least no `!let!` command)

- [x] Add back `#{  }` in stream literals. Despite the brackets, the code/definitions get executed *in the parent scope*.
- [x] Scopes, Definitions, References and ControlFlowSegments exist at compile time:
  - [x] Scopes and segments are created everywhere they're needed
  - [x] Add in algorithm to mark references as final
- [x] At execution time:
  - [x] There needs to be some link between scope and stack frame
  - [x] Variable places go through `Unallocated` | `Occupied` | `Removed`
  - [x] Variables are read / written based on binding ids
- [x] Fix marking references as final:
  - [x] Use a second pass aligned with control flow order to set up scopes, segments and variables.
- [x] Improvements to final_value
  - [x] EmbeddedX should request value ownership of shared from expression land
  - [x] Disable transparent clone for streams
- [x] Fix all `TODO[scopes]`
  - [x] Fix / remove expensive parse stream forks
- [x] Tests
  - [x] Add test macro for asserting variable binding `is_final` information, e.g. with a `x[final]` and `x[not_final]` syntax?
  - [x] Add tests for things like `let x = %[1]; let x = %[2] + x; x`
  - [x] Add tests for things like `y[x] = x + 1`
  - [x] Add tests for things like `let x = %[1]; { let x = %[2] + x; }; x`
  - [x] Add test that `let x; x = { let x = 123; x = 456; 5 }`. resolves correctly with `x = 5`.
  - [x] Add tests involving for loops; and if/elsif/else blocks

## Attempt Expression (requires Scopes & Blocks)

- [x] Migrate remaining commands
  - [x] Use `None.configure_preinterpret()` for now
  - [x] Remove `!parse!`
- [ ] Add `attempt` expression - See @./2025-09-vision.md
  - [x] Replace `execution_err` with explicit error kinds, so we can handle them differently with respect to catching, e.g. `destructure_err`, `panic_err`, `user_err`, `assert_err`, `resolution_err`, `operation_err`
  - [x] Prevent mutating parent state in revertible block
  - [x] Add `if X` guards to attempt block
  - [x] Add a message to uncatchable errors explaining why the attempt block does not catch them, advising to use an assertion if these errors are intended to be caught.
  - [ ] Add `revert` keyword and replace error `is a not caught by attempt blocks` and `Guard condition evaluated to false.` with using it)
- [ ] Side-project: Make LateBound better to allow this, by upgrading to mutable before use
  - [ ] https://rust-lang.github.io/rfcs/2025-nested-method-calls.html

## Loop return behaviour

Ideally we want to allow returning/appending easily in a loop. Currently, we're trialing loops returning a vector (or possibly a vector of non-None values).

But this might just be unexpected / confusing.

Alternatively, we could have consider:
* Loops not returning anything (unless a `loop` uses a `break` perhaps, like in Rust)
* Embedded expressions having access to a `stream` variable, bound to the current contents of the stream, which they can append to.

So some possible things we can explore / consider:

- [ ] Either:
  - A: We remove vec-returns from loops
  - B: We only store values which are non-None in the array, we can therefore use `loop { break X }[0]` to get the return value
- [ ] Trial exposing the output stream as a variable binding `stream`. We need to have some way to make it kinda efficient though.
  - One kinda issue is that `stream` is a `&mut OutputStream` rather than a `Mutable<OutputStream>` if that's a problem. It's expensive to inter-convert these.
  - We also don't want people to be able to read from it - only mutate it: Conceptually considering some optimizations further down, this `stream` might actually be from some few levels above, using tail-return optimizations
  - Maybe we just have an `output(%[...])` command instead of exposing the stream variable?
  - Or even `output` statement so that we can do e.g. `output 'a %[..]` to reference a particular block.
  - But what does it mean in terms of the `Mutable<OutputStream>` to output to a parent stream?
    - It might be the same stream or not; depending on if there is an `output`
    expression in the middle layer. I think this is reasonable. Conceptually we may need to know that two labels refer to the same stream in some map somewhere.
    - It suggests that `output` is not a variable, but a statement so that it can be
    bound as late as possible.
- [ ] `break` / `continue` improvements:
  - Can return a value (from the last iteration of for / while loops)
  - Can specify a label, and return from a labelled block (https://blog.rust-lang.org/2022/11/03/Rust-1.65.0/#break-from-labeled-blocks)

## Parser Changes

First, read the @./2025-09-vision.md

* Manually search for transform and rename to parse in folder names and file.
* Initial changes:
  * Parsers no longer output to a stream.
  * Scopes/frames can have a parse stream associated with them.
    * This can be read/resolved (as the nearest parent) by parsers, even in expression blocks
  * Don't support `@(#x = ...)` - instead we can have `#(let x = @[STREAM ...])`
  * Consider a `parse %[ .. ] { /* parsers * / }` expression / block (no new scope!)

* Various other changes from the vision doc
* (Side thought) - How does selecting a parse stream come into it? And e.g. when we extend to method/function definitions... Some options:
  * `@'1 IDENT`
  * `@>ident`, `@'1>ident`
  * Pseudo-variables:
    * `@.ident()`, `@'1.ident()` and similarly `out += %[..]`, `out'x += %[..]`
    * Could define own method such as `assert_identical(@'1, @'2)`
  * `#(IDENT(@))` or `@IDENT` shorthand for `#(IDENT(@))`
  * `input.ident()`
  * One option - @ is sugar, we use labels (lifetimes) to define parsers:
    * `@IDENT` is sugar for `#(@IDENT)` which is sugar for `#(IDENT::<'current>())`.
    * `@x=IDENT` is shorthand for `#{ let x = @IDENT; }` (only in stream parser mode)
    * Any parsers with custom syntax require expression mode:
      `#{ let full = @CAPTURE { ..inner parser.. } }`
    * But then what is `@(..)` and repeat-friends syntax sugar for?
      * Something like `STREAM::<'current> { /* desugared */ }` could work
      * `@::<'current>(..)` could work, but it's a little weird to have the `@` and the identifier.
      * Or just don't allow an unsugared form, and require, `parse '1 { @( .. ) }` could work, where parse takes a label instead of a variable.
    * How would a parser take multiple input streams?
      * `let x = parse_same_ident::<'1, '2>(...)`

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
  * `@[ANY_GROUP { ...inner parser... }]`
  * `@[FORK { ...parser... }]` the parser block creates a `commit=false` variable, if this is set to `commit=true` then it commits the fork.
  * `@[PEEK { ... }]` which does a `@[FORK ...]` internally but never commits... question: Should it use it error (for use in an `attempt` block)? Or return a bool? Maybe we have two?
  * `@[FIELDS { ... }]` and `@[SUBFIELDS { ... }]`
  * Add ability to add scope to interpreter state (see `Parsers Revisited`) and can then add:
    * `@[REPEATED { ... }]` (see below)
    * `@[UNTIL { ... }]` - takes an explicit parse block or parser. Reverts any state change if it doesn't match.
```rust
@[REPEATED({
    separator?: %[], // Could really be a parse stream, but it has to be a value here, and realistically it's not important. This is evaluated only once at the start.
    min?: 0,
    max?: 1000000,
}) { <block> }]
```

## Methods and closures

- [ ] Introduce basic functions
  * Value type function `let my_func = |x, y, z| { ... };`
  * To start with, they are not closures (i.e. they can't capture any outer variables)
  * New node extension in the expression parser: invocation `(...)`
- [ ] Closures
  * A function may capture variable bindings from the parent scope, these are converted into a `VariableBinding::Closure(<closed_variable_id>)`
  * The closure consists of a set of bindings attached to the function value, either:
    - `ClosedVariable::Owned(Value)` if it's the last mention of the closed variable, so it can be moved in
    - `ClosedVariable::Referenced(Rc<RefCell<Value>>)` otherwise
  * Invocation requests `CopyOnWrite`, and can be on a shared function or an owned function
    (if it is the last usage of that value, as per normal red/owned binding rules)
    * If invocation is on an owned function, then owned values from the closure can be consumed
      by the invocation
    * Otherwise, the values are only available as shared/mut
- [ ] Optional arguments

## Utility methods

Implement the following:
* All value kinds:
  * `is_none()`, and similarly for other value kinds
* Streams:
  * `is_ident()` and similarly for other stream

## Repeat output bindings

* Use case: Easily create the below code, similar to a procedural macro. Notably creating tuples of all sizes.
  => Honestly, `map(|x| x.to_ident())` and existing `.intersperse(%[,])` is probably the cleanest combination
* We need maps or repeats. A simple join isn't enough for . Consider alternatives to the below syntax.
  * Option 0: Do nothing:
    - Use `for x in A..Z { let ident = x.ident(); $[x,] }`
    - Use `#(generics.intersperse(%[,]))`
  * Option 1: `%*(#generics,)` or `%(#generics),` like declarative macros.
    * All the variable bindings in the repeat must refer to arrays or streams (i.e. iterables) of the same length, similar to proc macros.
    * BUT sadly we'll often have arrays of objects, so we really want to map e.g. `arr[i].x`
  * Option 2: Python style iterator comprehension `#(%[x,] for x in generics)` using a `for` extension
              ... actually we already have this kinda with the for expression returning a list `for x in A..Z { x.ident() }`
  * Option 3: Map methods

Option 0 - Do nothing
```rust
// Impls `MyTrait` for tuples of size 0 to 10
preinterpret::run! {
  for N in 0..=10 {
    let comma_separated_types = %[];
    for name in 'A'..'Z'.take(N) {
      let ident = name.ident();
      comma_separated_types += %[#ident,];
    }
    %[
      impl<#comma_separated_types> MyTrait for (#comma_separated_types) {}
    ]
  }
}
```
Or even, with for expressions returning arrays:
```rust
// Impls `MyTrait` for tuples of size 0 to 10
preinterpret::run! {
  for N in 0..=10 {
    let comma_separated_types = (for name in 'A'..'Z'.take(N) { name.ident() }).intersperse(%[,]);
    %[
      impl<#comma_separated_types> MyTrait for (#comma_separated_types) {}
    ]
  }
}
```

Option 1 - Output repeat syntax, like declarative macros output binding
```rust
// Impls `MyTrait` for tuples of size 0 to 10
preinterpret::run! {
  for N in 0..=10 {
    let types = %[A B C D E F G H I J K L M N O P Q R S T].take(N);
    %[
        impl<%(#types),*> MyTrait for (%(#types,)*) {}
    ]
  }
}
```

Option 2 - Python-style for comprehensions? (or rust-style one-line for expressions)
```rust
// Impls `MyTrait` for tuples of size 0 to 10
preinterpret::run! {
  for N in 0..=10 {
    let type_params = [x.ident() for x in ('A'..).take(N)];
    // OR type_params = for x in A..Z { x.ident() }
    let tuple = %[( #(%[#x,] for x in type_params) )];
    let generics = %[< #(%[#x,] for x in type_params) >];
    %[
        impl#generics MyTrait for #tuple {}
    ]
  }
}
```

Option 3 - Maps:
```rust
// Impls `MyTrait` for tuples of size 0 to 10
preinterpret::run! {
  for N in 0..=10 {
    let idents = ('A'..).take(N).map(|x| x.ident());
    let type_params = %[< #(idents.map(|x| %[#x,])) >];
    let tuple = %[( #(idents.map(|x| %[#x,])) )];
    %[
        impl #type_params MyTrait for #tuple {}
    ]
  }
}
```

## Error improvements

* Distinguish a runtime error from a coding error (e.g. parse error, or "no method of type")
  * The latter should not be caught by `attempt` blocks
* If method resolution fails, perhaps we try finding a method with that name on other types

## Optimizations 

- [ ] Look at benchmarks and if anything should be sped up
- [ ] Speeding up scopes at runtime:
  - [ ] In the interpreter, store a flattened stack of variable values
  - [ ] `no_mutation_above` can be a stack offset
  - [ ] References store on them cached information - either up-front, via an `Rc<Cell<ReferenceContent::Resolved(ResolvedReference)>>` or via a "resolve on first execute"
    - Value's relative offset from the top of the stack
    - An is last use flag

## Match block [blocked on slices]

* Delay this probably - without enums it's not super important.
* We'll need to add destructuring references, and allow destructuring `x.as_ref()`
  and maybe `x.as_mut()`.
* To destructure owned objects, we probably want to do it as_ref first, and if that succeeds, we can commit to a proper owned destructuring.
* Poor man's enum with `{ type: "a", ... }` and `{ type: "b", ... }`

## Coding challenges

Implement 10 leet-code challenges and 10 parsing challenges (e.g. from `syn` docs) to ensure that the language is sufficiently comprehensive to use in practice.

## Final considerations

* Merge `assignee_frames` into `value_frames` as per comment as the top of `assignee_frames`
* Rename `EvaluationItem` to `RequestedValue` and consider making `RequestedValue::AssignmentCompletion` wrap an `Owned<()>` so that it becomes truly a value.
* Add `preinterpret::macro` - can this be a declarative macro? Would be slightly more efficient, as it just needs to wrap a call to `preinterpret::stream` or `preinterpret::run`...
* Add `LiteralPattern` (wrapping a `Literal`)
* Add `Eq` support on composite types and streams
* See `TODO[untyped]` - Have UntypedInteger/UntypedFloat have an inner representation of either value or literal, for improved efficiency / less weird `Span::call_site()` error handling
* Merge `HasValueType` into `ValueKind`
* Better handling of `configure_preinterpret` aligned with future parsers:
  * Add a `BespokeObject` value type, with an example subtype of `PreinterpretInterface`
  * Add a `preinterpret` variable to global scope of type `PreinterpretInterface`
  * Move `None.configure_preinterpret` to `PreinterpretInterface` and possibly split it out as `set_iteration_limit(..)`
* CastTarget revision:
  * The `as int` operator is not supported for string values
  * The `as char` operator is not supported for untyped integer values
  * Add casts of any integer to char, via `char::from_u32(u32::try_from(x))`
  * Should we remove/replace any CastTargets?
* TODO check
* Check all `#[allow(unused)]` and remove any which aren't needed

NB: `define_command`, `define_parser`, and parsing of Rust code pushed to v1.1

## Better handling of value sub-references

Returning refs of sub-values requires taking the enum outside of the reference, i.e. some `ExpressionValueRef<'a>`.

One option We can work it like `IterableRef`, but perhaps we can do better? 

- [ ] Trial if `Shared<ExpressionValue>` can actually store an `ExpressionValueRef<'a>` (which just stores `&'a`, not the `Ref` variable)...
  * This could be done by adding GATs (raising MSRV to 1.65) so that TypeData can have a `Ref<'T>`
  * And then `Shared<T>` can store a `<T as ..Target>::Type::Ref<'T>` (in the file, this can be encapsulated as a `HasRefType` trait, which can be blanket implemeted for types implementing `..Target`)
  * And then have a `AdvancedCellRef<T>` store a `<T as ..Target>::Type::Ref<'T>` which can be owned and we can manually call increase strong count etc on the `RefCell`.
  * To implement `AdvancedCellRef::map`, we'll need `TypeData::Ref<'T>` to implement Target in a self-fulfilling way. (i.e. `HasRefType { type Ref<'a>: HasRefParent<Parent = Self> }`, `HasRefParent { type Parent: HasRefType })`)
  * If this works, we can replace our `Ref<T>` with `T: HasRefType`
  * Migrate `IterableRef`

## Cloning

* Consider making Iterator non-clonable (which will unlock many more easy lazy implementations, of e.g. `take` using the non-clonable `Take`, and similar for other mapped iterators), i.e. `ExpressionValue` has a manual `clone() -> ExecutionResult<Self>` - this will simplify some things. But then, `to_string()` would want to take a `CopyOnWrite` so that where we clone, the iterator can potentially take owned, attempt cloen, else error.

## Finish converting all commands to expressions

E.G.
* `preinterpret.settings({..})` (`preinterpret` is available as a variable pre-bound on the root frame)
* .. possibly keep the v0.2 commands in `deprecated` mode?

## Write book / Docs

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
* Span handling
  * If someone wants to keep a value's span, they can keep it in a stream and coerce it; or store it as a tuple of a value with its span `%{ value: $x, span: %[$x] }`
* Parsing
* Explanation of each expression, showing how it can be defined in terms of other building blocks

And then we need to:
* Update the README to point to the book
* Update the module docstring to point to the book.

## Write marketing materials

* Publish v1.0
* Flashy infographic like `crabtime` with some examples.

## Stream-return optimizations [OPTIONAL]

Expression evaluation can come with an `OutputStyle::AppendToStream(&mut OutputStream)` rather than a `OutputStyle::OwnedValue`, which is handled in `ResolvedValue` (might need a new name!)

This can be used to optimize, e.g.:

* Methods
  * Using the `StreamOutput` return
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

## Value expansions [OPTIONAL]

Consider:
* Do we want some kind of slice object? (see `TODO[range-refactor]`)
    * We can make `ExpressionValue` deref into `ExpressionRef`, e.g. `ExpressionRef::Array(<slice>)`
    * Then we can make `SharedValue(Ref<ExpressionRef>)`, which can be constructed from a `Ref<ExpressionValue>` with a map!
    * And similarly `MutableValue(RefMut<ExpressionRefMut>)`
* Using ResolvedValue in place of ExpressionValue e.g. inside arrays / objects, so that we can destructure `let (x, y) = (a, b)` without clone/take
    * But then we end up with nested references which can be confusing!
    * CONCLUSION: Maybe we don't want this - to destructure it needs to be owned anyway?
* Consider whether to expand to storing `ResolvedValue` or `CopyOnWriteValue` in variables instead of `OwnedValue`?
    => The main issue is if it interferes with taking mutable references, but it's possibly OK, would need to see if it's a confusing problem in practice... (e.g. `let b = a[0]; a.push(1)` if `b` is a reference to `a[0]` then this is a problem when we push to `a`)
    => If a mutable reference is created and there are pending references, the variable data RefCell could be replaced with a cloned value and then mutated... But this can be more expensive, because e.g. `let b = a[0]; a.push(1)` results in the whole array `a` being copied in the `CoW` case; but only the `a[0]` being cloned in the "clone on assign" case.
    => Maybe we just stick to assignments being Owned/Cloned as currently

* Support `#(x[..])` syntax for indexing streams, like with arrays
    * `#(x[0])` returns the value at that position of the stream (using `INFER_TOKEN_TREE`)
    * `#(x[0..3])` returns a TokenStream
    * `#(x[0..=3])` returns a TokenStream

--------------------------------------------------------------------------------

# Descoped for 1.0

* Performance:
  * Use a small-vec optimization in some places
  * Get rid of needless cloning of commands/variables etc
  * Avoid needless token stream clones: Have `x += y` take `y` as OwnedOrRef, and either handles it as owned or shared reference (by first cloning)
* User-defined functions and parsers
* Parsers:
  * Fuller rust syntax parsing: all of https://veykril.github.io/tlborm/decl-macros/minutiae/fragment-specifiers.html#ty and more from syn (e.g. item, fields, etc)
* Fork of syn to:
  * Fix issues in Rust Analyzer
  * Add support for a more general `TokenBuffer`, and ensure that Cursor can work in a backwards-compatible way with that buffer. Support:
    * Storing a length
    * Embedding tokens directly without putting them into a `Group`
    * Possibling embedding a reference to a slice buffer inside a group
    * Ability to parse to a TokenBuffer or TokenBufferSlice
    * Possibly allowing some kind of embedding of Tokens whichcan be converted into a TokenStream.
    * Currently, `ParseBuffer` stores `unexpected` and has drop glue which is a hacky abstraction. We'll need to think of an alternative. Perhaps we change `ParseBuffer` to operate on top of a `TokenBuffer` ??
  * Allow variables to use CoW semantics. Variables can be Owned(ParseBuffer) or `Slice(ParseBufferSlice), where a ParseBufferSlice is some form of reference counting to a ParseBufferCore, and a FromLocation and ToLocation which are assumed to be at the same level.
  * Permit `[!parse_while! (!stream! ...) from #x { ... }]`
  * Fix `any_punct()` to ignore none groups
  * Groups can either be:
    * Raw Groups
    * Or created groups, where we store `DelimSpan` for re-parsing and accessing the open/close delimiters (this will let us improve `invalid_content_wrong_group`)
  * In future - improve performance of some other parts of syn
  * Better error messages
    * See e.g. invalid_content_too_short where ideally the error message would be on the last token in the stream. Perhaps End gets a span from the previous error?
    * See e.g. invalid_content_too_long where `unexpected token` is quite vague.
    Maybe we can't sensibly do better though...