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

* Variable typing (stream / value / object to start with), including an `object` type, like a JS object:
  * Separate `&mut Output` and `StreamOutput`, `ValueOutput = ExpressionOutput`, `ObjectOutput`
  * Objects:
    * Can be destructured with `@{ #hello, world: _, ... }` or read with `#(x.hello)` or `#(x["hello"])`
    * Debug impl is `[!object! { hello: [!group! BLAH], ["world"]: Hi, }]`
    * Fields can be lazy (for e.g. exposing functions on syn objects).
    * They have an input object
    * (Until we get custom type support into a syn fork), can be embedded into an output stream as a single token - e.g. `PREINTERPRET_OBJECT_2313`
    (The value can be looked up via a weak reference in the interpreter (as a central location), and the stream owning a reference to it to stop it being dropped). The final conversion to tokens can look up the object in the interpreter, and use its `stream()` function to either output the default
    stream for the object, or error and suggest fields the user should use instead.
  * Values:
    * Can output to an internal stream as `[!group! <content>]` so it can be read by other transformers.
    * When we parse a `#x` (`@(#x = INFER_TOKEN_TREE)`) binding, it tries to parse a stream as a value before interpreting a `[!group! ...]` as a stream.
    * Output to final output as unwrapped content
  * New expression & definition syntax (replaces `[!set!]`??)
    * `#(#x = [... stream ...])`
    * `#(#x += [... stream ...])` // += acts like "extend" with a stream LHS
    * `#(#x += [!group! ...])`
    * `#(#x = 1 + 2 + 3)` // (Expression) Value
    * `#(#x += 1)`        // += acts as plus with a value LHS
    * `#(#x = {})`        // Object
      * Fields can be read/written to with `#(x.hello)` or `#(x.hello.world)`
    * `#(#x = xxx {})`    // For some other custom type `xxx`.
    * `#([ ... stream ... ])`
    * `#({ a: x, ... })`
    * `#([ ... stream ... ].length)` ??
    * `#(let #x = 123; let #y = 123; let #z = {})`...
    * Support `#(x)` syntax for indexing streams:
      * `#(x[0])`
      * `#(x[0..3])`
      * `#..(x[0..3])` and other things like `[ ..=3]`
    ... basically - this becomes one big expression.
      * Equivalent to a `syn::Block` which is a `Vec<Stmt>`
      * ...where a `Stmt` is either a local `let` binding or an `expression`
* Destructurers => Transformers cont
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
    * `@[MATCH { ... }]` (with `#..x` as a catch-all) with optional arms...
    * `#(..)?`, `#(..)+`, `#(..),+`, `#(..)*`, `#(..),*`
  * Scrap `[!let!]` in favour of `[!parse! [...] as @(_ = ...)]`
* Consider:
  * Scrap `#>>x` etc in favour of `@(#x += ...)`
  * Adding all of these: https://veykril.github.io/tlborm/decl-macros/minutiae/fragment-specifiers.html#ty 
  * If the `[!split!]` command should actually be a transformer?
  * Adding `!define_macro!`
  * Adding `!define_command!`
  * Adding `!define_transformer!`
* `[!is_set! #x]`
* Have UntypedInteger have an inner representation of either i128 or literal (and same with float)
* Add `[!reinterpret! ...]` command for an `eval` style command, and maybe a `@[REINTERPRET ..]` transformer.
* Add casts of other integers to char, via `char::from_u32(u32::try_from(x))`
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

### Transformers Revisited
```rust
// * The current situation feels unclear/arbitrary/inflexible.
// * Better would be slightly more explicit.
//
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
//     * Not allowed to parse.
//     * Only last statement can (optionally) output, with a value model of:
//       * Leaf(Bool | Int | Float | String)
//       * Object
//       * Stream
//       * None
//     * The actual type is only known at evaluation time.
//     * #var is equivalent to #(var)
//   * @[XXX] or @[hello.world = XXX]
//     * Can parse; has no output.
//     * XXX is:
//       * A named destructurer (possibly taking further input)
//       * An { .. } object destructurer
//       * A @(...) transformer stream (output = input)
//   * @() = Transformer stream
//     * Can parse
//     * Its only output is an input stream reference
//       * ...and only if it's redirected inside a @[x = $(...)]
//   * [!command! ...]
//     * Can output but does not parse.
// * Every transformer or transport stream has a typed output, which is either:
//   * EITHER just its token tree / token stream (for simple matchers)
//   * OR an OBJECT with at least two properties:
//     * input => all matched characters (a slice reference which can be dropped...)
//       (it might only be possible to performantly capture this after the syn fork)
//     * stream => A lazy function, used to handle the output when #x is in the final output...
//               likely `input` or an error depending on the case.
//   * ... other properties, depending on the TRANSFORMER:
//     * ITEM might have quite a few
//     * STREAM has an `output` property (discussed below)
// * Drop @XXX syntax. Require: @[ ... ] instead, one of:
//   * @[XXX] or equivalently @[let _ = XXX ...]
//   * @[let x = XXX] or @[let x = XXX { ... }]
//   * @[let x.field = XXX ...]
//   * @[x = XXX]
//   * @[x.field += IDENT]
// * Explicit stream: @(...)
///  * Has an explicit output property via command output, e.g. [!output! ...] which appends both:
//     * Command output
//     * Output of inner transformer streams.
//   * It can be treated as a transformer and its output can be redirected with @[#x = @(...)] syntax.
//     But, on trying a few examples, we don't want to require such redirection.
//   * QUESTION: Do we actually use square brackets, i.e. @[#x = ...] (i.e. it's just without the IDENT)
//   * OR maybe as an explicit [ ... ] stream: @[#x = [...]]
//   * For inline destructurings (e.g. in for loops), we allow an implicit inner ... stream
// * EXACT then is just used for some sections where we're reading from variables.
//
// EXAMPLE
#(parsed = [])
[!parse! [...] as
  // OPTION 1
  @(
    #(let item = {})
    impl @[item.trait = IDENT] for @[item.type = IDENT]
    #(parsed += item)
  ),*
  // OPTION 2
  @[COMMA_REPEATED {
    before: #(let item = {}),
    item: @(impl @[#(item.trait) = IDENT] for @[#(item.type) = IDENT]),
    after: #(parsed += item),
  }]
]
[!for! @[{ #trait, #type }] in #(parsed.output) {
  impl #trait for #type {}
}]

```
### Transformer Notes WIP
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
// ALSO NICE
[!define_transformer! @INPUT = @(impl @IDENT for @IDENT),*]
[!for! (#x #y) in [!parse! #input as @INPUT] {

}]
// MAYBE - probably not though... 
[!parse_for! #input as @(impl @(#x = @IDENT) for @(#y = @IDENT)),* {

}]
// WHAT WIZARDRY IS THIS
[!define_transformer! @[REPEAT_EXACT @[FIELDS {
  matcher: @(#matcher = $(@REST)),
  repetitions: @(#repetitions = @LITERAL),
}]] = @[REINTERPRET [!for! #_ in [!range! 0..#repetitions] { $(#..matcher) }]]]

// SYNTAX PREFERENCE - output by default; use @ for destructurers
// * @( ... ) destructure stream, has an output
// * @( ... )? optional destructure stream, has an output
// * Can similarly have @(...),+ which handles a trailing ,
// * @X shorthand for @[X] for destructurers which can take no input, e.g. IDENT, TOKEN_TREE, TYPE etc
//   => NOTE: Each destructurer should return just its tokens by default if it has no arguments.
//   => It can also have its output over-written or other things outputted using e.g. @[TYPE { is_prefixed: X, parts: #(...), output: { #output } }]
// * #x is shorthand for @(#x = @TOKEN_OR_GROUP_CONTENT)
// * #..x) is shorthand for @(#x = @REST) and #..x, is shorthand for @(#x = @[UNTIL_TOKEN ,])
// * @(_ = ...)
// * @(#x = impl @IDENT for @IDENT)
// * @(#x += impl @IDENT for @IDENT)
// * @[REPEATED { ... }]
// * Can embed commands to output stuff too
// * Can output a group with: @(#x = @IDENT for @IDENT) [!output! #x]

// In this model, REPEATED is really clean and looks like this:
@[REPEATED {
  item: @(...),            // Captured into #item variable
  separator?: @(),         // Captured into #separator variable
  min?: 0,
  max?: 1000000,
  handle_item?: { #item }, // Default is to output the grouped item. #()+ instead uses `{ (#..item) }`
  handle_separator?: { },  // Default is to not output the separator
}]

// How does optional work?
// @(#x = @(@IDENT)?)
// Along with:
// [!object! { #x, my_var: #y, #z }]
// And if some field #z isn't set, it's outputted as null.
// Field access can be with #(variable.field)

// Do we want something like !parse_for!? It needs to execute lazily - how?
// > Probably by passing some `OnOutput` hook to an output stream method
[!parse_for! #input as @(impl @(#x = @IDENT) for @(#y = @IDENT)),+ {

}]
```

* Pushed to 0.4:
  * Get rid of needless cloning of commands/variables etc
  * Iterator variable type
  * Trial making `for` lazily read its input somehow? Some callback on `OutputStream` I guess...
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