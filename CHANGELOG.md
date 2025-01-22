# Major Version 0.3

## 0.3.0

### Variable Expansions

* `#x` now outputs the contents of `x` in a transparent group.
* `#..x` outputs the contents of `x` "flattened" directly to the output stream.

### New Commands

* Core commands:
  * `[!error! ...]` to output a compile error
  * `[!extend! #x += ...]` to performantly add extra characters to the stream 
  * `[!let! <destructuring> = ...]` does destructuring/parsing (see next section)
* Expression commands:
  * `[!evaluate! <expression>]`
  * `[!assign! #x += <expression>]` for `+` and other supported operators
  * `[!range! 0..5]` outputs `0 1 2 3 4`
* Control flow commands:
  * `[!if! <expression> { ... }]` and `[!if! <expression> { ... } !else! { ... }]`
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

### Expressions

Expressions can be evaluated with `[!evaluate! ...]` and are also used in the `if`, `for` and `while` loops. They operate on literals as values.

Currently supported are:
* Integer, Float, Bool, String and Char literals
* The operators: `+ - * / % & | ^ || &&`
* The comparison operators: `== != < > <= >=`
* The shift operators: `>> <<`
* Casting with `as` including to untyped integers/floats with `as int` and `as float`
* () and none-delimited groups for precedence

Expressions behave intuitively as you'd expect from writing regular rust code, except they happen at compile time.

### Destructuring

Destructuring performs parsing of a token stream. It supports:

* Explicit punctuation, idents, literals and groups
* Variable bindings:
  * `#x` - Reads a token tree, writes a stream (opposite of #x)
  * `#..x` - Reads a stream, writes a stream (opposite of #..x)
  * `#>>x` - Reads a token tree, appends a token tree (opposite of for #y in #x)
  * `#>>..x` - Reads a token tree, appends a stream (i.e. flatten it if it's a group)
  * `#..>>x` - Reads a stream, appends a group (opposite of for #y in #x)
  * `#..>>..x` - Reads a stream, appends a stream
* Commands which don't output a value, like `[!set! ...]` or `[!extend! ...]`
* Named destructurings:
  * `(!stream! ...)` (TODO - decide if this is a good name)
  * `(!ident! ...)`
  * `(!punct! ...)`
  * `(!literal! ...)`
  * `(!group! ...)`

### To come

* Refactoring & testing
  * Move destructuring commands to separate file and tests
  * Add tests for `(!stream! ...)`, `(!group! ...)`, `(!ident! ...)`, `(!punct! ...)` including matching `'`, `(!literal! ...)`
  * Add compile error tests for all the destructuring errors
* `[!split! { stream: X, separator: X, drop_empty?: false, permit_trailing_separator?: true, }]` and `[!comma_split! ...]`
* `[!zip! ([Hello Goodbye] [World Friend])]` => `[(Hello World), (Goodbye Friend)]` and/or `[!zip! { streams: (#countries #flags #capitals), trim_to_shortest?: false }]` with `InterpretValue<AnyGrouped<Repeated<CodeInput>>>`
  * e.g. `[!for! (#country #flag #capital) in [!zip! (#countries #flags #capitals)]`
* `[!is_set! #x]`
* `[!debug!]` command to output an error with the current content of all variables.
* `[!str_split! { input: Value<LitStr>, separator: Value<LitStr>, }]`
* Add casts of other integers to char, via `char::from_u32(u32::try_from(x))`
* Add more tests
  * e.g. for various expressions
  * e.g. for long sums
* Rework `Error` as:
  * `ParseResult` with `ParseError::LowLevel(syn::Error)` | `ParseError::Contextual(syn::Error)`
  * `ExecutionInterrupt` with `ExecutionInterrupt::Err(syn::Error)` | `ExecutionInterrupt::ControlFlow(..)`
* Place destructuring
  * `(!optional! ...)`
  * `(!repeated! ...)` also forbid `#x` bindings inside of them unless a `[!settings! { ... }]` has been overriden
    * `{ item: (!stream! ...), minimum?: syn::int, maximum?: syn::int, separator?: (!stream! ...), after_each?: { ... }, before_all?: {}, after_all?: {}, }`
  * `(!raw! ...)`
  * `(!match! ...)` (with `#..x` as a catch-all)
  * `(!fields! ...)` and `(!subfields! ...)`
  * (MAYBE) `#(..)?`, `#(..)+`, `#(..),+`, `#(..)*`, `#(..),*` but I don't like them much
* Check all `#[allow(unused)]` and remove any which aren't needed
* Rework expression parsing, in order to:
  * Fix comments in the expression files
  * Enable lazy && and ||
  * Enable support for code blocks { .. } in expressions, and remove hacks where expression parsing stops at {} or .
* Pushed to 0.4:
  * Fork of syn to:
    * Fix issues in Rust Analyzer
    * Improve performance
    * Permit `[!parse_while! [!PARSE! ...] from #x { ... }]`
  * Further syn parsings (e.g. item, fields, etc)
* Work on book
  * Input paradigms:
    * Streams
    * StreamInput / ValueInput / CodeInput
  * Including documenting expressions
  * There are three main kinds of commands:
    * Those taking a stream as-is
    * Those taking some { fields }
    * Those taking some custom syntax, e.g. `!set!`, `!if!`, `!while!` etc

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