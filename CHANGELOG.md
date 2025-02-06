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
  * `[!output! ...]` can be used to just output its interpreted contents. Normally it's a no-op, but it can be useful inside a transformer.
  * `[!settings! { ... }]` can be used to adjust the iteration limit.
* Expression commands:
  * `[!evaluate! <expression>]`
  * `[!assign! #x += <expression>]` for `+` and other supported operators
  * `[!range! 0..5]` outputs `0 1 2 3 4`
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
  * `[!group! ...]` which wraps the tokens in a transparent group. Useful with `!for!`.
  * `[!intersperse! { ... }]` which inserts separator tokens between each token tree in a stream.
  * `[!split! ...]` which can be used to split a stream with a given separating stream.
  * `[!comma_split! ...]` which can be used to split a stream on `,` tokens.
  * `[!zip! (#countries #flags #capitals)]` which can be used to combine multiple streams together.
* Destructuring commands:
  * `[!let! <destructuring> = ...]` does destructuring/parsing (see next section). Note `[!let! #..x = ...]` is equivalent to `[!set! #x = ...]`

### Expressions

Expressions can be evaluated with `[!evaluate! ...]` and are also used in the `if`, `for` and `while` loops. They operate on literals as values.

Currently supported are:
* Integer, Float, Bool, String and Char literals
* The operators: `+ - * / % & | ^`
* The lazy boolean operators: `|| &&`
* The comparison operators: `== != < > <= >=`
* The shift operators: `>> <<`
* Casting with `as` including to untyped integers/floats with `as int` and `as float`
* () and none-delimited groups for precedence
* Embedded `#x` grouped variables, whose contents are parsed as an expression
  and evaluated.
* `{ ... }` for creating sub-expressions, which are parsed from the resultant token stream.

Expressions behave intuitively as you'd expect from writing regular rust code, except they happen at compile time.

### Transforming

Transforming performs parsing of a token stream, whilst also outputting a stream. The input stream must be parsed in its entirety.

Transform streams (or substreams) can be redirected to set variables or append to variables. Commands can also be injected to add to the output.

Inside a transform stream, the following grammar is supported:

* `@(...)`, `@(#x = ...)`, `@(#x += ...)` and `@(_ = ...)` - Explicit transform (sub)streams which either output, set, append or discard its output.
* Explicit punctuation, idents, literals and groups. These aren't output by default, except directly inside a `@[EXACT ...]` transformer.
* Variable bindings:
  * `#x` - Reads a token tree, writes its content (opposite of `#x`). Equivalent to `@(#x = @TOKEN_OR_GROUP_CONTENT)`
  * `#..x` - Reads a stream, writes a stream (opposite of `#..x`).
    * If it's at the end of the transformer stream, it's equivalent to `@(#x = @REST)`.
    * If it's followed by a token `T` in the transformer stream, it's equivalent to `@(#x = @[UNTIL T])`
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
  * `@(#inner = ...) [!output! #inner]` - wraps the output in a transparent group

### To come

* Destructurers => Transformers
  * Scrap `[!let!]` in favour of `[!parse! #x as @(_ = ...)]`
  * `@TOKEN_TREE`
  * `@TOKEN_OR_GROUP_CONTENT` - Literal, Ident, Punct or None-group content.
  * `@[ANY_GROUP ...]`
  * `@REST`
  * `@[UNTIL xxxx]` - For now - takes a raw stream which is turned into an ExactStream.
  * `@[FIELDS { ... }]` and `@[SUBFIELDS { ... }]`
  * Add ability to add scope to interpreter state (copy on write?) (and commit/revert) and can then add:
    * `@[OPTIONAL ...]` and `@(...)?`
    * `@(REPEATED { ... })` (see below)
    * Potentially change `@UNTIL` to take a transform stream instead of a raw stream.
    * `[!match! ...]` command
    * `@[ANY { ... }]` (with `#..x` as a catch-all) like the [!match!] command but without arms...
    * `#(..)?`, `#(..)+`, `#(..),+`, `#(..)*`, `#(..),*`
* Consider:
  * If the `[!split!]` command should actually be a transformer?
  * Scrap `#>>x` etc in favour of `@(#x += ...)`
* `[!is_set! #x]`
* Support `[!index! ..]`:
  * `[!index! #x[0]]`
  * `[!index! #x[0..3]]`
  * `[!..index! #x[0..3]]` and other things like `[ ..=3]`
  * `[!index! [Hello World][...]]`
  => NB: This isn't in the grammar because of the risk of `#x[0]` wanting to mean `my_arr[0]` rather than "index my variable".
* Have UntypedInteger have an inner representation of either i128 or literal (and same with float)
* Add `[!reinterpret! ...]` command for an `eval` style command.
* Add casts of other integers to char, via `char::from_u32(u32::try_from(x))`
* Get rid of needless cloning
* TODO check
* Check all `#[allow(unused)]` and remove any which aren't needed
* Work on book
  * Input paradigms:
    * Streams
    * StreamInput / ValueInput / CodeInput
  * Including documenting expressions
  * There are three main kinds of commands:
    * Those taking a stream as-is
    * Those taking some { fields }
    * Those taking some custom syntax, e.g. `!set!`, `!if!`, `!while!` etc

### Destructuring Notes WIP
```rust
// FINAL DESIGN ---
// ALL THESE CAN BE SUPPORTED:
[!for! (#x #y) in [!parse! #input as @(impl @IDENT for @IDENT),*] {

}]
// NICE
[!parse! #input as @[REPEATED {
    item: @(impl @(#trait = @IDENT) for @(#type = @TYPE)),
    item_output: {
      impl BLAH BLAH {
        ...
      }
    }
}]]
// ALSO NICE - although I don't know how we'd do custom inputs
[!define_parser! @INPUT = @(impl @IDENT for @IDENT),*]
[!for! (#x #y) in [!parse! #input as @INPUT] {

}]
// MAYBE - probably not though... 
[!parse_for! #input as @(impl @(#x = @IDENT) for @(#y = @IDENT)),* {

}]

// What if destructuring could also output something? (nb - OPTION 3 is preferred)

// OPTION 1
// * #(X ...) would append its output to the output stream, i.e. #(#output += X ...)
// * #(field: X ...) would append `field: output,` to the output stream.
// * #(#x = X)
// * #(void X) outputs nothing
// * #x and #..x still work in the same way (binding to variables?)

// OPTION 2 - everything explicit
// * #(void IDENT) outputs nothing
// * #(+IDENT) appends to #output (effectively it's an opt-in)
// * #(.field = IDENT) appends `field: ___,` to #output
// * #(#x = IDENT) sets #x
// * #(#x += IDENT) extends #x
// * #(#x.field = IDENT) appends `field: ___,` to #x

// OPTION 3 (preferred) - output by default; use @ for destructurers
// * @( ... ) destructure stream, has an output
// * @( ... )? optional destructure stream, has an output
// * Can similarly have @(...),+ which handles a trailing ,
// * @X shorthand for @[X] for destructurers which can take no input, e.g. IDENT, TOKEN_TREE, TYPE etc
//   => NOTE: Each destructurer should return just its tokens by default if it has no arguments.
//   => It can also have its output over-written or other things outputted using e.g. @[TYPE { is_prefixed: X, parts: #(...), output: { #output } }]
// * #x is shorthand for @[#x = @TOKEN_OR_GROUP_CONTENT]
// * #..x) is shorthand for @[#x = @UNTIL_END] and #..x, is shorthand for @[CAPTURE #x = @[UNTIL_TOKEN ,]]
// * @(_ = ...)
// * @(#x = impl @IDENT for @IDENT)
// * @(#x += impl @IDENT for @IDENT)
// * @[REPEATED { ... }]
// * Can embed commands to output stuff too
// * Can output a group with: @(#x = @IDENT for @IDENT) [!output! #x]

// In this model, REPEATED is really clean and looks like this:
@[REPEATED {
  item: #(...),            // Captured into #item variable
  separator?: #(),         // Captured into #separator variable
  min?: 0,
  max?: 1000000,
  handle_item?: { #item }, // Default is to output the grouped item. #()+ instead uses `{ (#..item) }`
  handle_separator?: { },  // Default is to not output the separator
}]

// How does optional work?
// @(#x = @(@IDENT)?)
// Along with:
// [!fields! { #x, my_var: #y, #z }]
// And if some field #z isn't set, it's outputted as null.

// Do we want something like !parse_for!? It needs to execute lazily - how?
// > Probably by passing some `OnOutput` hook to an output stream method
[!parse_for! #input as @(impl @(#x = @IDENT) for @(#y = @IDENT)),+ {

}]
```

* Pushed to 0.4:
  * Map/Field style variable type, like a JS object:
    * e.g. #x.hello = BLAH
    * Has a stream-representation as `{ hello: [!group! BLAH], }` but is more performant, and can be destructured with `[@FIELDS { #hello }]` or read with `[!read! #x.hello]` or `[!read! #x["hello"]]`
    * And can transform output as `#(.hello = @X)` (mixing/matching append output and field output will error)
    * Debug impl is `[!fields! hello: [!group! BLAH],]`
    * Can be embedded into an output stream as a fields group
    * If necessary, can be converted to a stream and parsed back as `{ hello: [!group! BLAH], }`
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