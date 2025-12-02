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

- [x] Migrate the following `PairedBinaryOperation` across all types to being under the `binary_operations` of its TypeData. For each, when migration is complete, add it to the // MIGRATION LIST in operations.rs to check it's fully migrated
  - [x] `Addition`
  - [x] `Subtraction`, `Multiplication`, `Division` and `Remainder`
  - [x] `LogicalAnd` and `LogicalOr`
  - [x] `BitXor`, `BitAnd` and `BitOr`
  - [x] `Equal` and `NotEqual`
  - [x] `LessThan`, `LessThanOrEqual`, `GreaterThanOrEqual`, `GreaterThan`
- [x] Migrate the following `IntegerBinaryOperation`:
  - [x] `ShiftLeft` and `ShiftRight`
  - [x] Compute SHL/SHR on `Integer` via `.checked_shl(u32)` with an attempted cast to u32 via TryInto<u32>,
  which should massively reduce the number of implementataions we need to generate.
    i.e. we have a `CoercedInt<u32>` wrapper type which we use as the operand of the SHL/SHR operators
- [x] Remove the `evaluate_legacy` method, the `PairedBinaryOperation`, and all the dead code
- [x] CompoundAssignment migration
- [x] Enable `x += x` to work (using disable() / enable()) methods
- [x] Add tests that e.g. `swap(x, x)` still breaks with a borrowing error and we don't get UB
- [x] Combine `PairedBinaryOperation`, `IntegerBinaryOperation` and `CompoundAssignmentOperation` into a flattened `BinaryOperation`
- [x] Migrate `UntypedInteger` to use `FallbackInteger` like `UntypedFloat` (except a little harder because integers can overflow)
- [x] Migrate <, <=, >, >= from specific integer/float and untyped values to IntegerValue and FloatValue, in a similar way that we've done for paired arithmetic operators.
- [ ] Add `==` and `!=` support for all values (including streams, objects, arrays, parsers, unsupported literals, etc) and make it work with `AnyRef<..>` arguments for testing equality
- [ ] Add a new test file, `operations.rs`, and add tests to cover all the operations, including:
  - [ ] Cover all the binary operations with all valid type combinations
  - [ ] For integers/streams, this will involve for each paired operator `1 x untyped/untyped`, `n x typed/typed`, `n x typed/untyped` and `n x untyped/typed` where `n` is the number of integer/float types there are.
  - [ ] Create various examples of combinations / operations that don't compile, and create compilation failure tests for them in an `operations` folder, brainstorm ideas, but some ideas include:
    - [ ] Operations between invalid types; at least one for each operation. e.g. `1 + []` or `1u32 + 3.0`
    - [ ] Overflows, underflows, divide by 0s, etc
  - [ ] Have a think about floats and test for infinity and NaN. Can such values be created in preinterpret? If not, how might we support creating them? How does rust do it? Is there an easy preinterpret equivalent?
- [ ] Ensure all `TODO[operation-refactor]` and `TODO[compound-assignment-refactor]` are done

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
- [x] Add `attempt` expression - See @./2025-09-vision.md
  - [x] Replace `execution_err` with explicit error kinds, so we can handle them differently with respect to catching, e.g. `destructure_err`, `panic_err`, `user_err`, `assert_err`, `resolution_err`, `operation_err`
  - [x] Prevent mutating parent state in revertible block
  - [x] Add `if X` guards to attempt block
  - [x] Add a message to uncatchable errors explaining why the attempt block does not catch them, advising to use an assertion if these errors are intended to be caught.
  - [x] Add `revert` keyword and replace error `is a not caught by attempt blocks` and `Guard condition evaluated to false.` with using it

## Loop return behaviour & emit statement

Ideally we want to allow returning/appending easily in a loop. Currently, we're trialing loops returning a vector (or possibly a vector of non-None values).

But this might just be unexpected / confusing.

Alternatively, we could have consider:
* Loops not returning anything (unless a `loop` uses a `break` perhaps, like in Rust)
* Embedded expressions having access to a `stream` variable, bound to the current contents of the stream, which they can append to.

These are things we definitely want to do:

- [x] Add an `OutputHandler` to the interpreter, with a stack of `OutputStream`.
- [x] Add `emit` statement which outputs into the parent stream literal
- [x] Remove vec-returns from loops, replace with `emit`
- [x] Change to error at parse if people use preinterpret keywords including `emit`, `attempt` and `revert` as variable names.
- [x] Add sensible use-case tests and compilation failure tests using `emit`
- [x] Disallow `emit` in revertible segment into stream outside of segment

## Break / Continue Improvements

- [x] `break` can include an optional expresion, and can be used to return a value
- [x] Loops can be labelled, e.g. with `'outer: loop { .. }` or `'inner: for { .. }`.
- [x] `break` / `continue` can specify a label, and return from that label
- [x] Add tests to control_flow.rs and compilation failure tests covering various scenarios
- [x] Blocks can be labelled, and `break` can be used to return from a labelled block (https://blog.rust-lang.org/2022/11/03/Rust-1.65.0/#break-from-labeled-blocks)

## Parser Changes

First, read the @./2025-11-vision.md

- [x] We store input in the interpreter
- [x] Create new `Parser` value kind
- [x] Add ParserHandle to `InputHandler` and use some generational map to store ParseStacks (or import slotmap)
  - [x] Look at https://donsz.nl/blog/arenas/
  - [ ] If using slotmap / generational-arena, replace the arena implementation too
- [x] Create (temporary) `parse X => |Y| { }` expression
- [x] Bind `input` to `Parser` at the start of each parse expression
- [ ] Create `@input[...]` expression
  - [ ] Create a `ConsumeStream` which wraps a `SourceStream`
    - [ ] We add `consume()` method which takes a `&mut ConsumingInterpreter` which for now can wrap a `ParseHandle` and `&mut Interpreter`
    - [ ] Expression return values are swallowed
- [ ] Add remaining parser methods below
- [ ] Delete the transformers folder
  - [ ] Change stream pattern to also be `@input[...]` - which binds the input
  - [ ] Write equivalent tests
- [ ] Reversion works in attempt blocks, via forking and committing or rolling back the fork, fix `TODO[parser-input-in-interpreter]`
- [ ] Address any remaining `TODO[parser-no-output]` and `TODO[parsers]`
- [ ] Add tests for all the methods on Parser, and for nested parse statements

`Parser` methods:
- [x] `ident()`, `is_ident()`
- [x] `literal()`, `is_literal()`
- [x] `integer()`, `is_integer()`
- [x] `float()`, `is_float()`
- [x] `char()`, `is_char()`
- [x] `string()`, `is_string()`
- [x] `end()`, `is_end()`
- [ ] `read(<stream>)` - uses `stream.parse_exact_match`
- [ ] `rest()`
- [ ] `any_ident()`
- [ ] `until(%[,])` (see until transformer)
- [ ] `error()` etc
- [ ] `token_tree()`
- [ ] `span()` or `cursor()` -- maybe? outputs a token with a span for outputting errors. If at end of an inner stream, it outputs the ident `END` with the span of the closing bracket.

And all of these from normal macros:
- [ ] block: a block (i.e. a block of statements and/or an expression, surrounded by braces)
- [ ] expr: an expression
- [ ] ident: an identifier (this includes keywords)
- [ ] item: an item, like a function, struct, module, impl, etc.
- [ ] lifetime: a lifetime (e.g. 'foo, 'static, …)
- [ ] literal: a literal (e.g. "Hello World!", 3.14, '🦀', …)
- [ ] meta: a meta item; the things that go inside the #[...] and #![...] attributes
- [ ] pat: a pattern
- [ ] path: a path (e.g. foo, ::std::mem::replace, transmute::<_, int>, …)
- [ ] stmt: a statement
- [ ] tt: a single token tree
- [ ] ty: a type
- [ ] vis: a possible empty visibility qualifier (e.g. pub, pub(in crate), …)

Consider if we want separate types for e.g.
* `Span`
* `TokenTree`

Repeat bindings (only inside a `consume` statement)
* `@(..)?`, `@(..)+`, `@(..),+`, `@(..)*`, `@(..),*`

Future methods once we have closures:
* `input.any_group(|inner| { })`
* Something for `input.fields({ ... })` and `input.subfields({ ... })`, whose fields are closures
* Possibly some support for `input.peek` and `fork` - although this is handled by the attempt statement
```rust
input.repeated(
  %{
    separator?: %[],
    min?: 0,
    max?: 1000000,
  },
  |inner| {
    // ...
  }
)
```

Later:
- [ ] Support for starting to parse a `input.open('(')` in the left part of an attempt arm and completing in the right arm `input.close(')')` - there needs to be some error checking in the parse stream stack. We probably can't allow closing in the LHS of an attempt arm. We should record a reason on the new parse buffer and raise if it doesn't match
- [ ] Or even `input.read("hello (")` / `input.read(")")`

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
- [ ] Break/continue label resolution in functions/closures
  * Functions and closures must resolve break/continue labels statically
  * Break and continue statements should not leak out of function boundaries
  * This needs to be validated during the control flow pass
- [ ] Optional arguments
- [ ] Add `map`, `filter`, `flatten`, `flatmap`
- [ ] Add `stream.parse(|input| { ... })`
- [ ] Add `let captured = input.capture(|input| { ... })`
  * This returns the parsed input stream. It can capture the original tokens by using `let forked = input.fork()` and then `let end_cursor = input.end();` and then consuming `TokenTree`s from `forked` until `forked.cursor >= end_cursor` (making use of the PartialEq implementation)

## Utility methods

Implement the following:
* All value kinds:
  * `is_none()`, and similarly for other value kinds
  * A `kind()` method which returns a logical name for the value kind, which could be used in a `match` statement

* Streams:
  * `is_ident()` and similarly for other stream

## Repeat output bindings

- [ ] Implement option 1 below (i.e. repeat syntax). Maps will follow separately.

--
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

- [x] Distinguish a runtime error from a coding error (e.g. parse error, or "no method of type")
- [ ] If method resolution fails, perhaps we try finding a method with that name on other types

## Optimizations 

- [ ] Look at benchmarks and if anything should be sped up
- [ ] Speeding up stream literal processing
  - [ ] When interpreting a stream literal, we can avoid having to go through error handling pathways to get an `output` from the intepreter by storing a `OutputInterpreter<'a>` which wraps an `&mut OutputStream` and a pointer to an Intepreter, and can be converted back into/from an `Interpreter` easily
  - [ ] Possibly similarly for an `InputInterpreter<'a>` when processing a `ConsumeStream`
- [ ] Speeding up scopes at runtime:
  - [ ] In the interpreter, store a flattened stack of variable values
  - [ ] `no_mutation_above` can be a stack offset
  - [ ] References store on them cached information - either up-front, via an `Rc<Cell<ReferenceContent::Resolved(ResolvedReference)>>` or via a "resolve on first execute"
    - Value's relative offset from the top of the stack
    - An is last use flag
- Address `TODO[performance]`

## Deferred

The following are less important tasks which maybe we don't even want/need to do.

- [x] Side-project: Make LateBound better to allow this, by upgrading to mutable before use
  - [x] https://rust-lang.github.io/rfcs/2025-nested-method-calls.html
  - [x] x += x for x copy, by resolving Owned before Mutable / Shared
- [ ] Allow adding lifetimes to stream literals `%'a[]` and then `emit 'a`, with `'root` being the topmost. Or maybe just `emit 'root` honestly. Can't really see the use case for the others.
  - [ ] Note that `%'a[((#{ emit 'a %[x] }))]` should yield `x(())`
  - [ ] Note that we need to prevent or revert outputting to root in revertible segments

## Match block [blocked on slices]

* Delay this probably - without enums it's not super important.
* We'll need to add destructuring references, and allow destructuring `x.as_ref()`
  and maybe `x.as_mut()`.
* To destructure owned objects, we probably want to do it as_ref first, and if that succeeds, we can commit to a proper owned destructuring.
* Poor man's enum with `{ type: "a", ... }` and `{ type: "b", ... }`

## Coding challenges

Implement 10 leet-code challenges and 10 parsing challenges (e.g. from `syn` docs) to ensure that the language is sufficiently comprehensive to use in practice.

## Final considerations

- [x] Merge `assignee_frames` into `value_frames` as per comment as the top of `assignee_frames`
- [x] Rename `EvaluationItem` to `RequestedValue` and consider making `RequestedValue::AssignmentCompletion` wrap an `Owned<()>` so that it becomes truly a value.
- [x] Merge `HasValueType` with `ValueKind`
* Add `preinterpret::macro` - can this be a declarative macro? Would be slightly more efficient, as it just needs to wrap a call to `preinterpret::stream` or `preinterpret::run`...
* Add `LiteralPattern` (wrapping a `Literal`)
* Add `Eq` support on composite types and streams
* See `TODO[untyped]` - Have UntypedInteger/UntypedFloat have an inner representation of either value or literal, for improved efficiency / less weird `Span::call_site()` error handling
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
  We can use `_xyz: Unused<T>` in some places to reduce the size of types.
* Do we want to add support for various rust types?

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

* Consider making Iterator non-clonable (which will unlock many more easy lazy implementations, of e.g. `take` using the non-clonable `Take`, and similar for other mapped iterators), i.e. `ExpressionValue` has a manual `clone() -> ExecutionResult<Self>` - this will simplify some things. But then, `to_string()` would want to take a `CopyOnWrite` so that where we clone, the iterator can potentially take owned, attempt clone, else error.

## Write book / Docs

- [ ] [PAGE] Introduction - covering:
  * Motivation
  * A Rust-like interpreted language with JS-like value types, built for code-generation
  * Comparison with proc-macros and crabtime
    * Native stream and parsing support, the `attempt` expression
  - [ ] [PAGE] Cheat-sheet
- [ ] [PAGE] Syntax
  - [ ] [PAGE] Values  (linking to each expression value)
  - [ ] [PAGE] Control Flow
- [ ] [PAGE] Expression Values - discusses expression value hierachy, expression model similar to rust, object/array similar to JS. [subpage for each main expression value kind, including sytax, examples and creating the value, and methods on the value]
  - [ ] [PAGE] Stream
  - [ ] [PAGE] Integers
  - [ ] [PAGE] Floats
  - [ ] [PAGE] Char
  - [ ] [PAGE] Boolean
  - [ ] [PAGE] String
  - [ ] [PAGE] Array
  - [ ] [PAGE] Object
  - [ ] [PAGE] Range
  - [ ] [PAGE] Iterable
  - [ ] [PAGE] Iterator
  - [ ] [PAGE] None
- [ ] [PAGE] Guides
  - [ ] [PAGE] Errors and Spans
      - NB: If someone wants to keep a value's span, they can keep it in a stream and coerce it; or store it as a tuple of a value with its span `%{ value: $x, span: %[$x] }`
  - [ ] [PAGE] Parsing
- [ ] Examples (tbc)

And then we need to:
- [ ] Update the README to point to the book
- [ ] Update the module docstring to point to the book.

Sidenote - crabtime comparison:
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


## Write marketing materials

* Publish v1.0
* Flashy infographic like `crabtime` with some examples.
* Mimimal readme, focusing on key use-cases, and pointing out at the docs.

## Stream-return optimizations [OPTIONAL]

Expression evaluation can come with an `OutputStyle::AppendToStream(&mut OutputStream)` rather than a `OutputStyle::OwnedValue`, which is handled in `ArgumentValue` (might need a new name!)

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
* Using ArgumentValue in place of ExpressionValue e.g. inside arrays / objects, so that we can destructure `let (x, y) = (a, b)` without clone/take
    * But then we end up with nested references which can be confusing!
    * CONCLUSION: Maybe we don't want this - to destructure it needs to be owned anyway?
* Consider whether to expand to storing `ArgumentValue` or `CopyOnWriteValue` in variables instead of `OwnedValue`?
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