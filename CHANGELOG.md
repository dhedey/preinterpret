# Major Version 0.3

## 0.3.0

### Variable Expansions

* `#x` now outputs the contents of `x` in a transparent group.
* `#..x` outputs the contents of `x` "flattened" directly to the output stream.

### New Commands

* Core commands:
  * `[!error! ...]` to output a compile error.
  * `[!set! #x += ...]` to performantly add extra characters to a variable's stream.
  * `[!set! _ = ...]` interprets its arguments but then ignores any outputs.
  * `[!stream! ...]` can be used to just output its interpreted contents. It's useful to create a stream value inside an expression.
  * `[!reinterpret! ...]` is like an `eval` command in scripting languages. It takes a stream, and parses/interprets it.
  * `[!settings! { ... }]` can be used to adjust the iteration limit.
* Expression commands:
  * The expression block `#(let x = 123; y /= x; y + 1)` which is discussed in more detail below.
* Control flow commands:
  * `[!if! <expression> { ... }]` and `[!if! <expression> { ... } !elif! <expression> { ... } !else! { ... }]`
  * `[!while! <expression> { ... }]`
  * `[!for! <destructuring> in [ ... ] { ... }]`
  * `[!loop! { ... }]`
  * `[!continue!]`
  * `[!break!]`
* Token-stream utility commands:
  * `[!is_empty! #stream]`
  * `[!length! #stream]` which gives the number of token trees in the token stream.
  * `[!group! ...]` which wraps the tokens in a transparent group. Can be useful if using token streams as iteration sources, e.g. in `!for!`.
  * `[!intersperse! { ... }]` which inserts separator tokens between each token tree in a stream.
  * `[!split! ...]` which can be used to split a stream with a given separating stream.
  * `[!comma_split! ...]` which can be used to split a stream on `,` tokens.
  * `[!zip! [#countries #flags #capitals]]` which can be used to combine multiple streams together.
* Destructuring commands:
  * `[!let! <destructuring> = ...]` does destructuring/parsing (see next section). Note `[!let! #..x = ...]` is equivalent to `[!set! #x = ...]`

### Expressions

Expressions can be evaluated with `#(...)` and are also used in the `!if!` and `!while!` loop conditions.

Expressions behave intuitively as you'd expect from writing regular rust code, except they are executed at compile time.

The `#(...)` expression block behaves much like a `{ .. }` block in rust. It supports multiple statements ending with `;` and optionally a final statement.
Statements are either expressions `EXPR` or `let x = EXPR`, `x = EXPR`, `x += EXPR` for some operator such as `+`.

The following are recognized values:
* Integer literals, with or without a suffix
* Float literals, with or without a suffix
* Boolean literals
* String literals
* Char literals
* Other literals
* Rust [ranges](https://doc.rust-lang.org/reference/expressions/range-expr.html)
* Token streams which are defined as `[...]`.

The following operators are supported:
* The numeric operators: `+ - * / % & | ^`
* The lazy boolean operators: `|| &&`
* The comparison operators: `== != < > <= >=`
* The shift operators: `>> <<`
* The concatenation operator: `+` can be used to concatenate strings and streams.
* Casting with `as` including to untyped integers/floats with `as int` and `as float`, to a grouped stream with `as group` and to a flattened stream with `as stream`.
* () and none-delimited groups for precedence

The following methods are supported:
* On all values:
  * `.clone()` - converts a reference to a mutable value. You will be told in an error if this is needed.
  * `.as_mut()` - converts an owned value to a mutable value. You will be told in an error if this is needed.
  * `.take()` - takes the value from a mutable reference, and replaces it with `None`. Useful instead of cloning.
  * `.debug()` - a debugging aid whilst writing code. Causes a compile error with the content of the value. Equivalent to `[!error! #(x.debug_string())]`
  * `.debug_string()` - returns the value's contents as a string for debugging purposes
* On arrays: `len()` and `push()`
* On streams: `len()`

An expression also supports embedding commands `[!xxx! ...]`, other expression blocks, variables and flattened variables. The value type outputted by a command depends on the command.

### Transforming

Transforming performs parsing of a token stream, whilst also outputting a stream. The input stream must be parsed in its entirety.

Transform streams (or substreams) can be redirected to set variables or append to variables. Commands can also be injected to add to the output.

Inside a transform stream, the following grammar is supported:

* `@(...)`, `@(#x = ...)`, `@(#x += ...)` and `@(_ = ...)` - Explicit transform (sub)streams which either output, set, append or discard its output.
* Explicit punctuation, idents, literals and groups. These aren't output by default, except directly inside a `@[EXACT ...]` transformer.

* Named destructurings:
  * `@IDENT` - Consumes and output any ident.
  * `@PUNCT` - Consumes and outputs any punctation
  * `@LITERAL` - Consumes and outputs any literal
  * `@REST` - Consumes the rest of the input, until the end of the stream or content of the current group
  * `@[UNTIL x]` - Consumes the rest of the input, until the end of stream OR until token `x`. `x` can be a group like `()` which matches the opening bracket `(`. 
  * `@[GROUP ...]` - Consumes a none-delimited group. Its arguments are used to transform the group's contents.
  * `@[EXACT ...]` - Interprets its arguments (i.e. variables are substituted, not bound; and command output is gathered) into an "exact match stream". And then expects to consume exactly the same stream from the input. It outputs the parsed stream.
* Commands: Their output is appended to the transform's output. Useful patterns include:
  * `@(inner = ...) [!stream! #inner]` - wraps the output in a transparent group

### To come
* Method calls continued
  * Add tests for TODO[access-refactor] (i.e. changing places to resolve correctly)
  * Add more impl_resolvable_argument_for
  * TODO[range-refactor] & some kind of more thought through typed reference support - e.g. slices, mutable slices?
  * TODO[operation-refactor]
    * Including no clone required for testing equality of streams, objects and arrays
  * Add better way of defining methods once / lazily, and binding them to an object type. 
* Consider:
  * Removing span range from value:
    * Moving it to a binding such as `Owned<T>` etc
    * Using `EvaluationError` (without a span!) inside the calculation, and adding the span in the evaluator (nb. it may still need to be able to propogate an `ExecutionInterrupt` internally)
  * Do we want some kind of slice object?
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
* Introduce `~(...)` and `r~(...)` streams instead of `[!stream! ...]` and `[!raw! ...]`
* Introduce interpreter stack frames
  * Read the `REVERSION` comment in the `Parsers Revisited` section below to consider approaches, which will work with possibly needing to revert state if a parser fails.
  * Design the data model => is it some kind of linked list of frames?
    * If we introduce functions in future, then a reference is a closure, in _lexical_ scope, which is different from stack frames.
    * We probably don't want to allow closures to start with.
    * Maybe we simply pass in a lexical parent frame when resolving variable references?
       * Possibly with some local cache of parent references, so we don't need to re-discover them if they're used multiple times in a loop
         e.g. `x` in `#(let x = 0; {{{ loop { x++; if x < 10 { break }} }}})`
  * Add a new frame inside loops, or wherever we see `{ ... }`
  * Merge `GroupedVariable` and `ExpressionBlock` into an `ExplicitExpression`:
    * Either `#ident` or `#(...)` or `#{ ... }`...
      the latter defines a new variable stack frame, just like Rust
    * To avoid confusion (such as below) and teach the user to only include #var where necessary, only expression _blocks_ are allowed in an expression.
      * Confusion example: `let x; x = #(let x = 123; 5)`. This isn't allowed in normal rust because the inside is a `{ .. }` which defines a new scope.
    * The `#()` syntax can be used in certain places (such as parse streams)
* Support for/while/loop/break/continue inside expressions
  * And allow them to start with an expression block with `{}` or `#{}` or a stream block with `~{}`
    QUESTION: Do we require either `#{}` or `~{}`? Maybe! That way we can give a helpful error message.
  * ... and remove the control flow commands `[!if! ...]` / `[!while! ...]` / `[!break! ...]` / `[!for! ...]`, `[!while! ...]` and `[!loop! ...]`
* TRANSFORMERS => PARSERS cont
  * Manually search for transform and rename to parse in folder names and file.
  * Parsers no longer output to a stream.
  * See all of `Parsers Revisited` below!
  * Remove `#x` and `#..x` as variable binding / parsers, instead use `#(let x = @TOKEN_TREE.flatten())` / `#(let x = @REST)`
  * We let `#(let x)` INSIDE a `@(...)` bind to the same scope as its surroundings...
    Now some repetition like `@{..}*` needs to introduce its own scope.
    ==> Annoyingly, a `@(..)*` has to really push/output to an array internally to have good developer experience; which means we need to solve the "relatively performant staged/reverted interpreter state" problem regardless; and putting an artificial limitation on conditional parse streams to not do that doesn't really work. TBC - needs more consideration.
  * Don't support `@(#x = ...)` - instead we can have `#(let x = @[STREAM ...])`
    * This can capture the original tokens by using `let forked = input.fork()` and then `let end_cursor = input.end();` and then consuming `TokenTree`s
      from `forked` until `forked.cursor >= end_cursor` (making use of the PartialEq implementation) 
  * Scrap `[!set!]` in favour of `#(x = ..)` and `#(x += ..)`
  * Scrap `[!let!]` in favour of `#(let <destructuring> = x)`
* Implement the following named parsers
  * Consider if `@LITERAL` should infer to a value
  * `@TOKEN_OR_GROUP_CONTENT` - Literal, Ident, Punct or None-group content (using `ParsedTokenTree`) - (do we need this?)
  * `@INFER_TOKEN_TREE` - Infers parsing as a value, falls back to Stream - OR maybe we just do `@TOKEN_TREE.infer()` - possibly this should also strip none-groups
  * `@INTEGER`
  * `@[ANY_GROUP ...]`
  * `@[FORK @{ ...parser... }]` the parser block creates a `commit=false` variable, if this is set to `commit=true` then it commits the fork.
  * `@[PEEK ...]` which does a `@[FORK ...]` internally
  * `@[FIELDS { ... }]` and `@[SUBFIELDS { ... }]`
  * Add ability to add scope to interpreter state (see `Parsers Revisited`) and can then add:
    * `@[OPTIONAL ...]` and `@(...)?`
    * `@[REPEATED { ... }]` (see below)
    * `@[UNTIL @{...}]` - takes an explicit parse block or parser. Reverts any state change if it doesn't match.
    * `[!match! ...]` command
    * `@[MATCH { ... }]` (with `#..x` as a catch-all) with optional arms...
    * `#(..)?`, `#(..)+`, `#(..),+`, `#(..)*`, `#(..),*`
```rust
@[REPEATED {
  item: @(...),            // Captured into #item variable
  separator?: @(),         // Captured into #separator variable
  min?: 0,
  max?: 1000000,
  handle_item?: { #item }, // Default is to output the grouped item. #()+ instead uses `{ (#..item) }`
  handle_separator?: { },  // Default is to not output the separator
}]
```
* Support `#(x[..])` syntax for indexing streams, like with arrays
  * `#(x[0])` returns the value at that position of the stream (using `INFER_TOKEN_TREE`)
  * `#(x[0..3])` returns a TokenStream
  * `#(x[0..=3])` returns a TokenStream
* Compare to https://www.reddit.com/r/rust/comments/1j42fgi/media_introducing_eval_macro_a_new_way_to_write i.e. https://crates.io/crates/crabtime - thoughts on crabtime:
 => Looks great!
 => Why don't they use a cheap hash of the code as a cache key?
    - The cachability is a big win compared to preinterpret (although preinterpret is faster on first run)
 => I can't imagine the span-chasing / error messages are great, because everything is translated to/from strings between the processes
    ... I wonder if there's any way to improve this?  Plausibly you could use the proc-macro bridge encoding scheme as per https://blog.jetbrains.com/rust/2022/07/07/procedural-macros-under-the-hood-part-ii/ to send handles onwards, or even delegate directly somehow?
    - The spans are better in preinterpret
 => Parsing isn't really a thing - they're not going after full macro stuff, probably wise 
* Add `LiteralPattern` (wrapping a `Literal`)
* Add `Eq` support on composite types and streams
* Consider:
  * Moving control flow (`for` and `while`) to the expression side? Possibly with an `output` auto-variable with `output += [!stream! ...]`
  * Dropping lots of the `group` wrappers?
  * If any types should have reference semantics instead of clone/value semantics?
  * Supporting anonymous functions, and support them in e.g. `intersperse`
  * Adding all of these: https://veykril.github.io/tlborm/decl-macros/minutiae/fragment-specifiers.html#ty
  * Adding `preinterpret::macro`
  * Adding `!define_command!`
  * Adding `!define_parser!`
  * Whether `preinterpret` should start in expression mode?
    => Or whether to have `preinterpet::stream` / `preinterpret::run` / `preinterpret::define_macro` options?
    => Maybe `preinterpret::preinterpret` is marked as deprecated; starts in `stream` mode, and enables `[!set!]`?
* `[!is_set! #x]`
* Have UntypedInteger have an inner representation of either i128 or literal (and same with float)
* Maybe add a `@[REINTERPRET ..]` parser.
* CastTarget expansion:
  * Add `as iterator` and uncomment the test at the end of `test_range()`
  * Support a CastTarget of `array` using `into_iterator()`.
  * Add `as ident` and `as literal` casting and support it for string, array and stream using concat recursive.
  * Add casts of other integers to char, via `char::from_u32(u32::try_from(x))`
* Put `[!set! ...]` inside an opt-in feature because it's quite confusing.
* TODO check
* Check all `#[allow(unused)]` and remove any which aren't needed
* Add benches inspired by this: https://github.com/dtolnay/quote/tree/master/benches
* Work on book
  * Input paradigms:
    * Streams
    * StreamInput / ValueInput / CodeInput
  * Including documenting expressions
  * There are three main kinds of commands:
    * Those taking a stream as-is
    * Those taking some { fields }
    * Those taking some custom syntax, e.g. `!set!`, `!if!`, `!while!` etc

### Parsers Revisited
```rust
// PROPOSAL (subject to the object proposal above):
// * Various modes...
//   * EXPRESSION MODE
//     * OPENING SYNTAX:
//       * #{ ... } or #'name { ... } with a new lexical variable scope
//       * #( ... ) with no new scope (TBC if we need both - probably)
//     * CONTEXT:
//       * Is *not* stream-based - it consists of expressions rather than a stream
//       * It *may* have a contextual input parse stream
//     * One or more statements, terminated by ; (except maybe the last, which is returned)
//     * Idents mean variables.
//     * `let x; let _ = <value>; <value>; if <value> {<statements>} else {}`
//     * Error not to discard a non-None value with `let _ = <value>`
//     * If it has an input parse stream, it's allowed to parse by embedding parsers, e.g. @TOKEN_TREE
//     * Only the last statement can (optionally) output, with a value model of:
//       * Leaf(Bool | Int | Float | String)
//       * Object
//       * Stream
//       * None
//     * (In future we could add breaking from labelled block: https://blog.rust-lang.org/2022/11/03/Rust-1.65.0/#break-from-labeled-blocks)
//     * The actual type is only known at evaluation time.
//     * #var is equivalent to #(var)
//   * OUTPUT STREAM MODE
//     * OPENING SYNTAX: ~{ ... } or ~( ... ) or raw stream literal with r~( ... )
//       * SIDENOTES: I'd have liked to use $, but it clashes with $() repetition syntax inside macro rules
//       * And % might appear in real code as 1 % (2 + 3)
//       * But ~ seems to not be used much, and looks a bit like an s. So good enough?
//     * CONTEXT:
//       * Can make use of a parent output stream, by concatenating into it. 
//       * Is stream-based: Anything embedded to it can have their outputs concatenated onto the output stream
//       * Does *not* have an input parse stream
//     * Can embed [!commands! ...], #() and #{ ... }, but not parsers
//       * The output from #([...]) gets appended to the stream; ideally by passing it an optional output stream
//     * It has a corresponding pattern:
//      * SYNTAX:
//        * ~#( .. ) to start in expression mode OR
//        * ~@( .. ) ~@XXX or ~@[XXX ...] to start with a parser e.g. ~@REST can be used to parse any stream opaquely OR
//        * r~( .. ) to match a raw stream (it doesn't have a mode!)
//      * Can be used where patterns can be used, in a let expression or a match arm, e.g.
//        * let ~#( let x = @IDENT ) = r~(...)
//        * ~#( ... ) => { ... }
//      * CONTEXT:
//         * It implicitly creates a parse-stream for its contents
//         * It would never have an output stream
//         * It is the only syntax which can create a parse stream, other than the [!parse! ..] command
//   * NAMED PARSER
//     * OPENING SYNTAX: @XXX or @[XXX] or @[XXX <arguments>]
//     * CONTEXT:
//       * Is typically *not* stream-based - it may has its own arguments, which are typically a pseudo-object `{  }`
//       * It has an input parse stream
//     * Every named parser @XXX has a typed output, which is:
//       * EITHER its input token stream (for simple matchers, e.g. @IDENT)
//       * OR an #output OBJECT with at least two properties:
//         * input => all matched characters (a slice reference which can be dropped...)
//           (it might only be possible to performantly capture this after the syn fork)
//           THIS NOTE MIGHT BE RELEVANT, BUT I'M WRITING THIS AFTER 6 MONTHS LACKING CONTEXT:
//              This can capture the original tokens by using `let forked = input.fork()`
//              and then `let end_cursor = input.end();` and then consuming `TokenTree`s
//              from `forked` until `forked.cursor >= end_cursor` (making use of the
//              PartialEq implementation) 
//         * output_to => A lazy function, used to handle the output when #x is embedded into a stream output...
//        likely `input` or an error depending on the case.
//         * into_iterator
//       * ... other properties, depending on the parsee:
//         * e.g. a Rust ITEM might have quite a few (mostly lazy)
//   * PARSE-STREAM MODE
//     * OPENING SYNTAX:
//       * @{ ... } or @'block_name { ... } (as a possibly named block) -- both have variable scoping semantics
//       * We also support various repetitions such as: @{ ... }? and @{ ... }+ and @{ ... },+ - discussed in next section in more detail
//       * We likely don't want/need an un-scoped parse stream
//     * CONTEXT:
//       * It is stream-based: Any outputs of items in the stream are converted to an exact parse matcher
//       * Does have an input parse stream which it can mutate
//     * Expression/command outputs are converted into an exact parse matcher
//     * RETURN VALUE:
//       * Parse blocks can return a value with `#(return X)` or `#(return 'block_name X)` else return None
//       * By contrast, `@[STREAM ...]` works very similarly, but outputs its input stream
//   * COMMAND
//     * OPENING SYNTAX: [!command! ...]
//     * CONTEXT:
//       * Takes an optional output stream from its parent (for efficiency, if it returns a stream it can instead concat to that)
//       * May or may not be stream-based depending on the command...
//       * Does *not* have an input parse stream (I think? Maybe it's needed. TBC)
//     * Lots of commands can/should probably become functions
//     * A flexible utility for different kinds of functionality
//
// * Repetitions (including optional), e.g. @IDENT,* or @[IDENT ...],* or @{...},*
//   * Output their contents in a `Repeated` type, with an `items` property, and an into_iterator implementation?
//   * Unscoped parse streams can't be repeated, only parse blocks (because they create a new interpreter frame)
//   * If we don't move our cursor forward in a loop iteration, it's an error -- this prevents @{}* or @{x?}* causing an infinite loop
//   * We do *not* support backtracking.
//     * This avoids most kind of catastrophic backtracking explosions, e.g. (x+x+)+y)
//     * Things such as @IDENT+ @IDENT will not match `hello world` - this is OK I think
//     * Instead, we suggest people to use a match statement or something
//   * There is still an issue of REVERSION - "what happens to external state mutated this iteration repetition when a repetition is not possible?"
//     * This appears in lots of places:
//       * In ? or * blocks where a greedy parse attempt isn't fatal, but should continue
//       * In match arms, considering various stream parsers, some of which might fail after mutating state
//       * In nested * blocks, with different levels of reversion
//     * But it's only a problem with state lexically captured from a parent block
//     * We need some way to handle these cases:
//     (A) Immutable structures - We use efficient-ish immutable data structures, e.g. ImmutableList, Copy-on-write leaf types etc
//        ... so that we can clone them cheaply (e.g. https://github.com/orium/rpds) or even just some manually written tries/cons-lists
//     (B) Use CoW/extensions - But naively it still give O(N^2) if we're reverting occasional array.push(..)es
//        We could possibly add in some tweaks to avoid clones in some special cases (e.g. array.push and array.pop which don't touch a prefix at the start of a frame)
//     (C) Error on mutate - Any changes to variables outside the scope of a given refutable parser => it's a fatal error
//        to parse further until that scope is closed. (this needs to handle nested repetitions).
//     (D) Error on revert - at reversion time, we check there have been no changes to variables below that depth in the stack tree
//        (say by recording a "lowest_stack_touched: usize"), and panic if so; and tell people to use `return` instead; or move state changes to the end.
//        We could even prevent parsing in a conditional block after the reversion.
//        [!Brilliant!] We could introduce a @[REQUIRE  ] which takes the parent conditional block out of conditional mode and makes it a hard error instead. This could dramatically improve error messages, and allow parsing after mutation :). (Ideally they'd be some way of applying it to a specific conditional block, but I think parent is good enough)
//
//     ==> D might be easiest, most flexible AND most performant... But it might not cover enough use cases.
//         ... but I think advising people to only mutate lexical-closure'd state at the end, when a parse is confirmed, is reasonably...
//
//
// QUESTION:
// - What mode do we start in, in v2?
//   => Possibly expression mode, which could convert to an output with ~{ ... }
//
// =========
// EXAMPLES
// =========

// EXAMPLE WITH !parse! COMMAND
#(
  let parsed = [!parse! {
    input: r~(
      impl A for X, impl B for Y
    ),
    parser: @{ // This @{ .. }, returns a `Repeated` type with an `into_iterator` over its items
      impl #(let the_trait = @IDENT) for #(let the_type = @IDENT)
      #(return { the_trait, the_type })
    },*
  }];

  // Assuming we are writing into an output stream (e.g. outputting from the macro),
  // we can auto-optimize - the last command of the expression block gets to write directly to the stream,
  // and by extension, each block of the for-loop gets to write directly to the stream
  for { the_trait, the_type } in parsed ~{
    impl #the_trait for #the_type {}
  }
)

// EXAMPLE WITH STREAM-PARSING-PATTERN
#(
  let ~#( // Defines a stream pattern, starting in expression mode, and can be used to bind variables
    let parsed = @{
      #(let item = {})
      impl #(item.the_trait = @IDENT) for #(item.the_type = @IDENT)
      #(return item)
    }
  ) = r~(...)

  for { the_trait, the_type } in parsed ~{
    impl #the_trait for #the_type {}
  }
) 

// Example pre-defined with no arguments:
[!define_parser! @IMPL_ITEM @{ // Can either start as #{ ... } or @{ ... }
  impl #(let the_trait = @IDENT) for #(let the_type = @IDENT)
  #(return { the_trait, the_type })
}]
for { the_trait, the_type } in [!parse! { input, parser: @IMPL_ITEM,* }] ~{
  impl #the_trait for #the_type {}
}

// Example pre-defined 2 with arguments:
[!define_parser! @[IMPL_ITEM /* arguments named-parser or parse-stream */] {
  @(impl #(let the_trait = @IDENT) for #(let the_type = @IDENT));
  { the_trait, the_type }
}]
for { the_trait, the_type } in [!parse! { input, parser: @IMPL_ITEM,* }] ~{
  impl #the_trait for #the_type {}
}
```

### Pushed to 0.4:
* Performance:
  * Use a small-vec optimization in some places
  * Get rid of needless cloning of commands/variables etc
  * Avoid needless token stream clones: Have `x += y` take `y` as OwnedOrRef, and either handles it as owned or shared reference (by first cloning)
* Iterators:
  * Iterator value type (with an inbuilt iteration count / limit check)
  * Allow `for` lazily reading from iterators
  * Support unbounded iterators `[!range! xx..]`
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
* Further syn parsings (e.g. item, fields, etc)

# Major Version 0.2

## 0.2.0

* Rename the string case conversion commands to be less noisy by getting rid of the case suffix
* Fix some bugs with the string conversion algorithms and add a full test-suite
* Add new string conversion commands: `[!kebab! ...]`, `[!title! ...]` and `[!insert_spaces! ...]`
* Add new ident creation shorthands: `[!ident_camel! ...]`, `[!ident_snake! ...]` and `[!ident_upper_snake! ...]`
* Overhauled the README

# Major Version 0.1

## 0.1.0 - 0.1.3

* Initial version, with basic variable substitution and string utilities