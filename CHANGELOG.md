# Major Version 0.3

## 0.3.0

### Variable Expansions

* `#x` now outputs the contents of `x` in a transparent group.
* `#..x` outputs the contents of `x` "flattened" directly to the output stream.

### New Commands

* Core commands:
  * `[!error! ...]` to output a compile error.
  * `[!set! #x += ...]` to performantly add extra characters to the stream.
  * `[!set! _ = ...]` interprets its arguments but then ignores any outputs.
  * `[!debug! ...]` to output its interpreted contents including none-delimited groups. Useful for debugging the content of variables.
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

An expression also supports embedding commands `[!xxx! ...]`, other expression blocks, variables and flattened variables. The value type outputted by a command depends on the command.

### Transforming

Transforming performs parsing of a token stream, whilst also outputting a stream. The input stream must be parsed in its entirety.

Transform streams (or substreams) can be redirected to set variables or append to variables. Commands can also be injected to add to the output.

Inside a transform stream, the following grammar is supported:

* `@(...)`, `@(#x = ...)`, `@(#x += ...)` and `@(_ = ...)` - Explicit transform (sub)streams which either output, set, append or discard its output.
* Explicit punctuation, idents, literals and groups. These aren't output by default, except directly inside a `@[EXACT ...]` transformer.
* Variable bindings:
  * `#x` - Reads a token tree, writes its content (opposite of `#x`). Equivalent to `@(x = @TOKEN_OR_GROUP_CONTENT)`
  * `#..x` - Reads a stream, writes a stream (opposite of `#..x`).
    * If it's at the end of the transformer stream, it's equivalent to `@(x = @REST)`.
    * If it's followed by a token `T` in the transformer stream, it's equivalent to `@(x = @[UNTIL T])`
  * `#>>x` - Reads a token tree, appends a token tree (can be read back with `!for! #y in #x { ... }`)
  * `#>>..x` - Reads a token tree, appends a stream (i.e. flatten it if it's a group)
  * `#..>>x` - Reads a stream, appends a group (can be read back with `!for! #y in #x { ... }`)
  * `#..>>..x` - Reads a stream, appends a stream
* Named destructurings:
  * `@IDENT` - Consumes and output any ident.
  * `@PUNCT` - Consumes and outputs any punctation
  * `@LITERAL` - Consumes and outputs any literal
  * `@[GROUP ...]` - Consumes a none-delimited group. Its arguments are used to transform the group's contents.
  * `@[EXACT ...]` - Interprets its arguments (i.e. variables are substituted, not bound; and command output is gathered) into an "exact match stream". And then expects to consume exactly the same stream from the input. It outputs the parsed stream.
* Commands: Their output is appended to the transform's output. Useful patterns include:
  * `@(inner = ...) [!stream! #inner]` - wraps the output in a transparent group

### To come

* Merge `GroupedVariable` and `ExpressionBlock` into an `ExplicitExpression`:
  * Either `#ident` or `#(...)` or `#{ ... }`...
    the latter defines a new variable stack frame, just like Rust
  * To avoid confusion (such as below) and teach the user to only include #var where
    necessary, only expression _blocks_ are allowed in an expression.
    * Confusion example: `let x; x = #(let x = 123; 5)`. This isn't allowed in normal
      rust because the inside is a `{ .. }` which defines a new scope.
* Support `#(x[..])` syntax for indexing arrays at read time (streams to follow in a separate task below after parsers are updated)
  * `#(x[0..3])` returns an array
  * `#(x[0..=3])` returns an array
  * ... and `#(x[0] = y)` can be used to set the item
  * Considering allowing assignments inside an expression
    * Implementation notes:
      * Separate `VariableData` and `VariableReference`, separate `let` and `set`
      * Three separate stacks for various calculations in evaluation
    * `let XX =` and `YY += y` are actually totally different...
      * With `let XX =`, `XX` is a _pattern_ and creates new variables.
      * With `YY += y` we're inside an expression already, and it only works with existing variables. The expression `YY` is converted into a destructuring at execution time.
      Note that the value side always executes first, before the place is executed.
    * Implementation realisations:
      * Rust reference very good: https://doc.rust-lang.org/reference/expressions.html#place-expressions-and-value-expressions
      * For the += operators etc:
        * The left hand side must resolve to a _single_ VariableData, e.g. `z` or
          `x.y["z"]`
        * In the rust reference, this is a "Place Expression"
        * We will have a `handle_assign_paired_binary_operation(&mut self, operation, other: Self)` (which for value types can resolve to the non-assign version)
      * For the = operator:
        * The left hand side will resolve non-Variable expressions, and be effectively
          left with something which can form a destructuring of the right hand side.
          e.g. `[x.y, z[x.a][12]] = [1, 2]`
        * In the rust reference, this is an "Assignee Expression" and is a generalization
          of place expressions
    * Test cases:
```rust
    // These examples should compile:
    let mut a = [0; 5];
    let mut b: u32 = 3;
    let c;
    let out = c = (a[2], _) = (4, 5);
    let out = a[1] += 2;
    let out = b = 2;
    // In the below, a = [0, 5], showing the right side executes first
    let mut a = [0; 2];
    let mut b = 0;
    a[b] += { b += 1; 5 };
    // This works:
    let (x, y);
    [x, .., y] = [1, 2, 3, 4]; // x = 1, y = 4
    // This works...
    // In other words, the assignee operation is executed incrementally,
    // The first assignment arr[0] = 1 occurs before being overwritten by
    // the arr[0] = 5 in the second section.
    let mut arr = [0; 2];
    (arr[0], arr[{arr[0] = 5; 1}]) = (1, 1);
    assert_eq!(arr, [5, 1]);
    // This doesn't work - two errors:
    // error[E0308]: mismatched types: expected `[{integer}]`, found `[{integer}; 4]`
    // error[E0277]: the size for values of type `[{integer}]` cannot be known at compilation time
    // (it looks like you can't just overwrite array subslices)
    let arr = [0; 5];
    arr[1..=4] = [1, 2, 3, 4];
    // This doesn't work.
    // error[E0368]: binary assignment operation `+=` cannot be applied to type `({integer}, {integer})`
    // https://doc.rust-lang.org/error_codes/E0368.html
    let (a, b) = (1, 1);
    (a, b) += (1, 2);
```
* Variable typing (stream / value / object to start with), including an `object` type, like a JS object:
  * Objects:
    * Backed by an indexmap
    * Can be created with `#({ a: x, ... })`
    * Can be read with `#(x.hello)` or `#(x["hello"])`
    * Debug impl is `#({ hello: [!group! BLAH], ["#world"]: Hi, })`
    * They have an input object
    * Fields can be read/written to with `#(x.hello)` or `#(x.hello.world)`
    * Can be destructured with `{ hello, world: _, ... }`
    * (Until we get custom type support into a syn fork), can be embedded into an output stream as a single token - e.g. `PREINTERPRET_OBJECT_2313`
    (The value can be looked up via a weak reference in the interpreter (as a central location), and the stream owning a reference to it to stop it being dropped). The final conversion to tokens can look up the object in the interpreter, and use its `stream()` function to either output the default
    stream for the object, or error and suggest fields the user should use instead.
    * Have `!zip!` support `{ objects }` 
  * Values:
    * When we parse a `#x` (`@(x = INFER_TOKEN_TREE)`) binding, it tries to parse a stream as a value before interpreting a `[!group! ...]` as a stream.
    * Output to final output as unwrapped content
* Method calls
  * Mutable methods notes:
    * They require either:
      * Reference semantics (e.g. using `Rc<RefCell<X>>` inside Object, Array) with explicit cloning
      * AND/OR Place semantics (e.g. each type supports being either a value or a reference to a path, so that an operator e.g. `+` can mutate)
    * I think we want reference semantics for object and array anyway. Unclear for stream.
      * Ideally we'd arrange it so that `x += ["Hello"] + ["World]` would append Hello and World; but `x += (["Hello"] + ["World])` would behave differently.
      * I think that means that `+=` becomes an operator inside an expression, and its LHS is a `PlaceValue` (`PlaceValue::Stream` or `PlaceValue::Array`)
  * Also add support for methods (for e.g. exposing functions on syn objects).
  * `.len()` on stream
  * `.push(x)` on array
  * Consider `.map(|<destructurer>| {})`
* TRANSFORMERS => PARSERS cont
  * Re-read the `Parsers Revisited` section below
  * Manually search for transform and rename to parse in folder names and file.
  * Parsers no longer output to a stream.
  * We let `#(let x)` INSIDE a `@(...)` bind to the same scope as its surroundings...
    Now some repetition like `@{..}*` needs to introduce its own scope.
    ==> Annoyingly, a `@(..)*` has to really push/output to an array internally to have good developer experience; which means we need to solve the "relatively performant staged/reverted interpreter state" problem regardless; and putting an artificial limitation on conditional parse streams to not do that doesn't really work. TBC - needs more consideration.
    ==> Maybe there is an alternative such as:
      * `@(..)*` returns an array of some output of it's inner thing
      * But things don't output so what does that mean??
      * I think in practice it will be quite an annoying restriction anyway
  * OLD: Support `@[x = ...]` and `@[let x = ...]` for individual parsers.
    * CHANGE OF THOUGHT: instead, support `#(x = @IDENT)` where `#` blocks inside parsers can embed @parsers and consume from a parse stream.
    * This makes it kinda like a `Parse` implementation code block.
    * This lets us just support variable definitions in expression statements.
  * Don't support `@(x = ...)` - instead we can have `#(x = @[STREAM ...])`
    * This can capture the original tokens by using `let forked = input.fork()`
      and then `let end_cursor = input.end();` and then consuming `TokenTree`s
      from `forked` until `forked.cursor >= end_cursor` (making use of the
      PartialEq implementation) 
  * Revisit `parse`
  * Scrap `#>>x` etc in favour of `#(a.push(@[XXX]))`
  * Scrap `[!let!]` in favour of `#(let <destructuring> = #x)`
  * `@TOKEN_TREE`
  * `@TOKEN_OR_GROUP_CONTENT` - Literal, Ident, Punct or None-group content.
  * `@INFER_TOKEN_TREE` - Infers values, falls back to Stream
  * `@[ANY_GROUP ...]`
  * `@REST`
  * `@[FIELDS { ... }]` and `@[SUBFIELDS { ... }]`
  * Add ability to add scope to interpreter state (see `Parsers Revisited`) and can then add:
    * `@[OPTIONAL ...]` and `@(...)?`
    * `@[REPEATED { ... }]` (see below)
    * `@[UNTIL @{...}]` - takes an explicit parse block or parser. Reverts any state change
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
* Support `#(x[..])` syntax for indexing streams
  * `#(x[0])` returns the item at that position of the array / OR the value at that position of the stream (using `INFER_TOKEN_TREE`)
  * `#(x[0..3])` returns a TokenStream
  * `#(x[0..=3])` returns a TokenStream
* Add `..` and `.., x` support to the array pattern
* Consider:
  * Moving control flow (`for` and `while`) to the expression side?
  * Dropping lots of the `group` wrappers?
  * If any types should have reference semantics instead of clone/value semantics?
  * Adding all of these: https://veykril.github.io/tlborm/decl-macros/minutiae/fragment-specifiers.html#ty
  * Adding `preinterpret::macro`
  * Adding `!define_command!`
  * Adding `!define_transformer!`
  * Whether `preinterpret` should start in expression mode?
    => Or whether to have `preinterpet::stream` / `preinterpret::run` / `preinterpret::define_macro` options?
    => Maybe `preinterpret::preinterpret` is marked as deprecated; starts in `stream` mode, and enables `[!set!]`?
* `[!is_set! #x]`
* Have UntypedInteger have an inner representation of either i128 or literal (and same with float)
* Maybe add a `@[REINTERPRET ..]` transformer.
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
// * Four modes:
//   * Output stream mode
//     * Outputs stream, does not parse
//     * Can embed [!commands! ...] and #()
//       * #([...]) gets appended to the stream
//   * #( ... ) = Expression mode.
//     * One or more statements, terminated by ; (except maybe the last)
//     * Idents mean variables.
//     * `let #x; let _ = <value>; <value>; if <value> {<statements>} else {}`
//     * Error not to discard a non-None value with `let _ = <value>`
//     * If embedded into a parse stream, it's allowed to parse by embedding parsers, e.g. @[STREAM ...]
//     * Only last statement can (optionally) output, with a value model of:
//       * Leaf(Bool | Int | Float | String)
//       * Object
//       * Stream
//       * None
//     * The actual type is only known at evaluation time.
//     * #var is equivalent to #(var)
//   * Named parser @XXX or @[XXX] or @[XXX <arguments>]
//     * Can parse; has no output.
//     * XXX is:
//       * A named destructurer (possibly taking further input)
//       * An { .. } object destructurer
//       * A @(...) transformer stream (output = input)
//   * @() or @{} = Parse block
//     * Expression/command outputs are parsed exactly.
//     * Can return a value with `#(return X)`
//     * (`@[STREAM ...]` is broadly equivalent, but does output the input stream)
//     * Its only output is an input stream reference
//       * ...and only if it's redirected inside a @[x = $(...)]
//   * [!command! ...]
//     * Can output but does not parse.
// * Every named parser has a typed output, which is:
//   * EITHER its input token stream (for simple matchers, e.g. @IDENT)
//   * OR an #output OBJECT with at least two properties:
//     * input => all matched characters (a slice reference which can be dropped...)
//       (it might only be possible to performantly capture this after the syn fork)
//     * output_to => A lazy function, used to handle the output when #x is in the final output...
//               likely `input` or an error depending on the case.
//     * into_iterator
//   * ... other properties, depending on the TRANSFORMER:
//     * e.g. a Rust ITEM might have quite a few (mostly lazy)
// * Repetitions (including optional), e.g. @IDENT,* or @[IDENT ...],* or @{...},*
//   * Output their contents in a `Repeated` type, with an `items` property, and an into_iterator implementation?
//   * Transform streams can't be repeated, only transform blocks (because they create a new interpreter frame)
//   * There is still an issue with "what happens to mutated state when a repetition is not possible?"
//     ... this is also true in a case statement... We need some way to rollback in these cases:
//     => Easier - Use of efficient-ish immutable data structures, e.g. ImmutableList, Copy-on-write leaf types etc
//        ... so that we can clone them cheaply (e.g. https://crates.io/crates/im-rc)
//     => Middle (best?) - Any changes to variables outside the scope of a given refutable parser => it's a fatal error
//        to parse further until that scope is closed. (this needs to handle nested repetitions)
//     => Hardest - Some kind of storing of diffs / layered stores without any O(N^2) behaviours
//
// EXAMPLE (Updated, 17th February)
[!parse! {
  input: [...],
  parser: #(
    let parsed = @{
      impl #(let the_trait = @IDENT) for #(let the_type = @IDENT)
      #(return { the_trait, the_type })
    },*
  ),
}]
// EXAMPLE IN PATTERN POSITION
#(
  let [!parser! // Takes an expression, which could include a #(...)
    let parsed = @(
      #(let item = {})
      impl #(item.the_trait = @IDENT) for #(item.the_type = @IDENT)
      #(return item)
    )*
  ] = [...]
)

[!for! { the_trait, the_type } in parsed {
  impl #the_trait for #the_type {}
}]
// Example inlined:
[!for! { the_trait, the_type } in [!parse! {
  input: [...],
  parser: @( // This is implicitly an expression, which returns a `Repeated` type with an `into_iterator` over its items.
    impl #(let the_trait = @IDENT) for #(let the_type = @IDENT)
    #(return { the_trait, the_type })
  ),*
}] {
  impl #the_trait for #the_type {}
}]

// Example pre-defined:
[!parser! @IMPL_ITEM {
  @(impl);
  let the_trait = @IDENT;
  @(for);
  let the_type = @IDENT;
  return { the_trait, the_type };
}]
[!for! { the_trait, the_type } in [!parse! { input, parser: @IMPL_ITEM,* }] {
  impl #the_trait for #the_type {}
}]

// Example pre-defined 2:
[!parser! @[IMPL_ITEM /* argument parse stream */] {
  @(impl #(let the_trait = @IDENT) for #(let the_type = @IDENT));
  { the_trait, the_type }
}]
[!for! { the_trait, the_type } in [!parse! { input, parser: @IMPL_ITEM,* }] {
  impl #the_trait for #the_type {}
}]
```

* Pushed to 0.4:
  * Performance:
    * Use a small-vec optimization in some places
    * Get rid of needless cloning of commands/variables etc
    * Support `+=` inside expressions to allow appending of token streams
      * Variable reference would need to be a sub-type of stream
      * Then `#x += [] + []` could resolve to two variable reference appends and then a return null
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