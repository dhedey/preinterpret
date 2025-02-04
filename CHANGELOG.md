# Major Version 0.3

## 0.3.0

### Variable Expansions

* `#x` now outputs the contents of `x` in a transparent group.
* `#..x` outputs the contents of `x` "flattened" directly to the output stream.

### New Commands

* Core commands:
  * `[!error! ...]` to output a compile error
  * `[!extend! #x += ...]` to performantly add extra characters to the stream 
  * `[!debug! ...]` to output its interpreted contents including none-delimited groups. Useful for debugging the content of variables.
  * `[!void! ...]` interprets its arguments but then ignores any outputs. It can be used inside destructurings.
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
  * `[!..group! ...]` which just outputs its contents as-is, useful where the grammar
    only takes a single item, but we want to output multiple tokens
  * `[!intersperse! { ... }]` which inserts separator tokens between each token tree in a stream.
  * `[!split! ...]`
  * `[!comma_split! ...]`
  * `[!zip! (#countries #flags #capitals)]` which can be used to combine multiple streams together
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

### Destructuring

Destructuring performs parsing of a token stream. It supports:

* Explicit punctuation, idents, literals and groups
* Variable bindings:
  * `#x` - Reads a token tree, writes a stream (opposite of `#x`)
  * `#..x` - Reads a stream, writes a stream (opposite of `#..x`)
  * `#>>x` - Reads a token tree, appends a token tree (can be read back with `!for! #y in #x { ... }`)
  * `#>>..x` - Reads a token tree, appends a stream (i.e. flatten it if it's a group)
  * `#..>>x` - Reads a stream, appends a group (can be read back with `!for! #y in #x { ... }`)
  * `#..>>..x` - Reads a stream, appends a stream
* Commands which don't output a value, like `[!set! ...]` or `[!extend! ...]`
* Named destructurings:
  * `(!stream! ...)` (TODO - decide if this is a good name)
  * `(!ident! ...)`
  * `(!punct! ...)`
  * `(!literal! ...)`
  * `(!group! ...)`
  * `(!raw! ...)`
  * `(!content! ...)`

### To come

* Add test to disallow source stuff in re-evaluated `{ .. }` blocks and command outputs
* Add `[!reinterpret! ...]` command for an `eval` style command.
* Support `[!set! #x]` to be `[!set! #x =]`. 
* `[!is_set! #x]`
* Support `'a'..'z'` in `[!range! 'a'..'z']`
* Destructurers => Transformers 
  * Implement pivot to transformers outputting things ... `@[#x = @IDENT]`...
  * `@TOKEN_TREE`
  * `@REST`
  * `@[UNTIL xxxx]`
  * `@[EXPECT xxxx]` expects the tokens (or source grammar inc variables), and outputs the matched tokens (instead of dropping them as is the default). Replaces `(!content!)`. We should also consider ignoring/unwrapping none-groups to be more permissive? (assuming they're also unwrapped during parsing).
  * `@[FIELDS { ... }]` and `@[SUBFIELDS { ... }]`
  * Add ability to add scope to interpreter state (copy on write?) (and commit/revert) and can then add:
    * `@[OPTIONAL ...]` and `@(...)?`
    * `@(REPEATED { ... })` (see below)
    * `[!match! ...]` command
    * `@[ANY { ... }]` (with `#..x` as a catch-all) like the [!match!] command but without arms...
    * `#(..)?`, `#(..)+`, `#(..),+`, `#(..)*`, `#(..),*`
* Consider:
  * Scrap `[!let!]` in favour of `[!parse! #x as #(...)]`
  * Scrap `[!void! ...]` in favour of `[!set! _ = ...]`
  * Destructurer needs to have different syntax. It's too confusingly similar!
    * Final decision: `@[#x = @IDENT]` because destructurers output (SEE BELOW FOR MOST OF THE WORKING)
    * Some other ideas considered:
    * `(>ident> #x)`? `(>fields> {})`? `(>comma_repeated> Hello)`
      * We can't use `<` otherwise it tries to open brackets and could be confused for rust syntax like: `(<x as y>)`
      * We need to test it with the auto-formatter in case it really messes it up
    * `#(>ident #x)` - not bad...
      * Then we can drop `(!stream!)` as it's just `#( ... )`
      * We need to test it with the auto-formatter in case it really messes it up
    * `[>ident (#x)]`
    * `<ident(#x)>`
    * `{[ident] #x}`
    * `(>ident> #x)`
    * `#IDENT { #x }`
    * `#IDENT { capture: #x }`
    * `#(IDENT #x)`
    * `@[#x = @IDENT]`
  * Scrap `#>>x` etc in favour of `@[#x += ...]`
* Support `[!index! ..]`:
  * `[!index! #x[0]]`
  * `[!index! #x[0..3]]`
  * `[!..index! #x[0..3]]` and other things like `[ ..=3]`
  * `[!index! [Hello World][...]]`
  => NB: This isn't in the grammar because of the risk of `#x[0]` wanting to mean `my_arr[0]` rather than "index my variable".
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

### Destructuring Notes WIP
```rust
// FINAL DESIGN ---
// ALL THESE CAN BE SUPPORTED:
[!for! (#x #y) in [!parse! #input as @(impl @IDENT for @IDENT),*] {

}]
// NICE
[!parse! #input as @[REPEATED {
    item: @(impl @[#trait = @IDENT] for @[#type = @TYPE]),
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
[!parse_for! #input as @(impl @[#x = @IDENT] for @[#y = @IDENT]),* {

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
// * #x is shorthand for @[#x = @TOKEN_TREE]
// * #..x) is shorthand for @[#x = @UNTIL_END] and #..x, is shorthand for @[CAPTURE #x = @[UNTIL_TOKEN ,]]
// * @[_ = ...]
// * @[#x = @IDENT for @IDENT]
// * @[#x += @IDENT for @IDENT]
// * @[REPEATED { ... }]
// * Can embed commands to output stuff too
// * Can output a group with: @[#x = @IDENT for @IDENT] [!group! #..x]

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
// @[#x = @(@IDENT)?]
// Along with:
// [!fields! { #x, my_var: #y, #z }]
// And if some field #z isn't set, it's outputted as null.

// Do we want something like !parse_for!? It needs to execute lazily - how?
// > Probably by passing some `OnOutput` hook to an output stream method
[!parse_for! #input as @(impl @[#x = @IDENT] for @[#y = @IDENT]),+ {

}]

// OUTSTANDING QUESTION:
// => Token streams are a good stand-in for arrays
// => Do we need a good stand-in for fields / key-value maps and other structured data?
//   => e.g. #x.hello = BLAH
//   => Has a stream-representation as { hello: [!group! BLAH], } but is more performant,
//      and can be destructured with `[@FIELDS { #hello }]` or read with `[!read! #x.hello]
// => And can transform output as #(.hello = @X) - Mixing/matching append output and field output will error
// => Debug impl is `[!fields! hello: [!group! BLAH],]`
// => Can be embedded into an output stream as a fields group
// => If necessary, can be converted to a stream and parsed back as `{ hello: [!group! BLAH], }`
```

* Pushed to 0.4:
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