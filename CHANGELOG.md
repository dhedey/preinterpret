# Major Version 1.0

## 1.0.0

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
  * The expression block `#(let x = 123; let y = 1.0; y /= x; y + 1)` which is discussed in more detail below.
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