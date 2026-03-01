# Major Version 1.0

## 1.0.0

This moves preinterpret to an expression-based language, inspired by Rust, but with some twists to make writing code generation code quicker:
* Token streams as a native feature
* Flexible Javascript-like objects/arrays
* New expressions such as `attempt { .. }` for trying alternatives

### Variable Expansions

* `#x` outputs the contents of `x`.
* `#(x.to_group())` outputs the contents of `x` in a transparent group.

### New Commands

* Core commands:
  * Creating errors:
    * `%[].error("Error Message")` to output a compile error at the macro call site
    * `%[_].error("Error Message")` to output a compile error at the given line
    * `%[$token].error("Error Message")` to output a compile error at the span of the tokens
    * `%[].assert(<condition>, <message>)` to assert the condition is true, else output a compile error at the macro call site
    * `%[_].assert(<condition>, <message>)` to assert the condition is true, else output a compile error at the given line
    * `%[$token].assert(<condition>, <message>)` to assert the condition is true, else output a compile error at the span of the tokens
  * `#(x += %[...];)` to add extra tokens to a variable's stream.
  * `#(let _ = %[...];)` interprets its arguments but then ignores any outputs.
  * `%[...]` can be used to just output its interpreted contents. It's useful to create a stream value inside an expression.
  * `%[...].reinterpret_as_run()` is like an `eval` command in scripting languages. It takes a stream, and runs it as a preinterpret expression block content like `run!{ ... }`. Similarly, `%[...].reinterpret_as_stream()` runs it as a stream literal, like `stream!{ ... }`. Each is pure - the reinterpreted code can't read from or write to variables, and can only return values.
  * `preinterpret::set_iteration_limit(xxx)` can be used to adjust the iteration limit.
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
  * `<iterable>.is_empty()`
  * `<iterable>.len()` which for streams gives the number of token trees in the token stream.
  * `%group[...]` which wraps the tokens in a transparent group. Can be useful if using token streams as iteration sources, e.g. in `!for!`.
  * `<any iterable>.intersperse(<separator (any value)>, <options>?)` which inserts the separator between each value from the iterator, and returns a vec. This can then be handled as a vector, embedded in a stream, or mapped with `to_string()` or `to_stream()` as required.
  * `<stream>.split(<separator (stream)>, <options>?)` which can be used to split a stream with a given separating stream.
  * `[countries, flags, capitals].zip()` or `%{ countries, flags, capitals }.zip()` which can be used to combine multiple streams together.

### Expressions

Expressions can be evaluated with `#(...)` and are also used in the `!if!` and `!while!` loop conditions.

Expressions behave intuitively as you'd expect from writing regular rust code, except they are executed at compile time.

The `#(...)` expression block behaves much like a `{ .. }` block in rust. It supports multiple statements ending with `;` and optionally a final statement.

Statements are either expressions `EXPR` or `let x = EXPR`, `x = EXPR`, `x += EXPR` for some operator such as `+`.

Assigment:
* `let <pattern> = <value>` which supports patterns include ignore (`_`), array destructurings (`[a, b, ..]`), object destructurings `{ a: x, b, .. }`, and stream parsing `%[..]`.

The following are recognized values:
* Object literals `%{ x: "Hello", y, ["z"]: "World" }` behave similarly to Javascript objects.
* Token stream literals `%[...]` take any token stream, and support embedding `#variables` or `#(<..expressions..>)` inside them.
* Raw token stream literals `%raw[...]` are used to capture raw tokens, and are not interpreted (e.g. `#` has no special meaning).
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
* Casting with `as` including to untyped integers/floats with `as int` and `as float` and to a flattened stream with `as stream`.
* () and none-delimited groups for precedence

The following methods are supported:
* On all values:
  * `.clone()` - converts a reference to a mutable value. You will be told in an error if this is needed.
  * `.as_mut()` - converts an owned value to a mutable value. You will be told in an error if this is needed.
  * `.take()` - takes the value from a mutable reference, and replaces it with `None`. Useful instead of cloning.
  * `.debug()` - a debugging aid whilst writing code. Causes a compile error with the content of the value. Roughly equivalent to `%[_].error(x.to_debug_string())`
  * `.to_debug_string()` - returns the value's contents as a string for debugging purposes
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
  * `@[EXACT(%[..])]` - Takes an input stream expression. Expects to consume exactly that stream from the output (ignoring none-groups). It outputs the parsed stream.
* Commands: Their output is appended to the transform's output. Useful patterns include:
  * If `@(inner = ...)` then `inner.to_group()` - wraps the output in a transparent group

# Major Version 0.2

## 0.2.1

No changes, just doc updates.

## 0.2.0

* Rename the string case conversion commands to be less noisy by getting rid of the case suffix
* Fix some bugs with the string conversion algorithms and add a full test-suite
* Add new string conversion commands: `[!kebab! ...]`, `[!title! ...]` and `[!insert_spaces! ...]`
* Add new ident creation shorthands: `[!ident_camel! ...]`, `[!ident_snake! ...]` and `[!ident_upper_snake! ...]`
* Overhauled the README

# Major Version 0.1

## 0.1.0 - 0.1.3

* Initial version, with basic variable substitution and string utilities