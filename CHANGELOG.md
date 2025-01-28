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

* Expression refactoring:
  * Get rid of source span, and instead provide it when getting an output
  * Also remove ToTokens etc
* `[!is_set! #x]`
* `[!str_split! { input: Value<LitStr>, separator: Value<LitStr>, }]`
* Change `(!content! ...)` to unwrap none groups, to be more permissive
* Add casts of other integers to char, via `char::from_u32(u32::try_from(x))`
* Add more tests
  * e.g. for various expressions
  * e.g. for long sums
* Destructurers
  * `(!fields! ...)` and `(!subfields! ...)`
  * Add ability to fork (copy on write?) / revert the interpreter state and can then add:
    * `(!optional! ...)`
    * `(!repeated! ...)` also forbid `#x` bindings inside of them unless a `[!settings! { ... }]` has been overriden
      * `{ item: (!stream! ...), minimum?: syn::int, maximum?: syn::int, separator?: (!stream! ...), after_each?: { ... }, before_all?: {}, after_all?: {}, }`
    * `[!match! ...]` command
    * `(!any! ...)` (with `#..x` as a catch-all) like the [!match!] command but without arms...
  * (MAYBE) `#(..)?`, `#(..)+`, `#(..),+`, `#(..)*`, `#(..),*` but I don't like them much
* Check all `#[allow(unused)]` and remove any which aren't needed
* Rework expression parsing, in order to:
  * Fix comments in the expression files
  * Enable lazy && and ||
  * Enable support for code blocks { .. } in expressions, and remove hacks where expression parsing stops at {} or .
  * Remove stack overflow possibilities when parsing a long nested expression
* Pushed to 0.4:
  * Fork of syn to:
    * Fix issues in Rust Analyzer
    * Improve performance (?)
    * Permit `[!parse_while! (!stream! ...) from #x { ... }]`
    * Fix `any_punct()` to ignore none groups
    * Groups can either be:
      * Raw Groups
      * Or created groups, where we store `DelimSpan` for re-parsing and accessing the open/close delimiters (this will let us improve `invalid_content_wrong_group`)
    * Better error messages
      * See e.g. invalid_content_too_short where ideally the error message would be on the last token in the stream. Perhaps End gets a span from the previous error?
      * See e.g. invalid_content_too_long where `unexpected token` is quite vague.
      Maybe we can't sensibly do better though...
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