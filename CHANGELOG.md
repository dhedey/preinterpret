# Major Version 0.3

## 0.3.0

### New Commands

* Expression commands:
  * `[!evaluate! ...]`
  * `[!increment! ...]`
* Control flow commands:
  * `[!if! COND { ... }]` and `[!if! COND { ... } !else! { ... }]`
* Token-stream utility commands:
  * `[!empty!]`
  * `[!is_empty! #stream]`
  * `[!length! #stream]` which gives the number of token trees in the token stream.
  * `[!group! ...]` which wraps the tokens in a transparent group. Useful with `!for!`.

### To come

* Use `[!let! #x = 12]` instead of `[!set! ..]`.
* Disallow `[!let! #x =]` and require `[!let! #x = [!empty!]]` (give a good error message).
* `[!while! cond {}]`
* `[!for! #x in [#y] {}]`... and make it so that whether commands or variable substitutions get replaced by groups depends on the interpreter context (likely only expressions should use groups)
* `[!range! 0..5]`
* `[!error! "message" token stream for span]`
* Remove `[!increment! ...]` and replace with `[!assign! #x += 1]`
* Support `!else if!` in `!if!`
* Token stream manipulation... and make it performant
  * Extend token stream
  * Consume from start of token stream
* Support string & char literals (for comparisons & casts) in expressions
* Add more tests
  * e.g. for various expressions
  * e.g. for long sums
  * Add compile failure tests
* Work on book
  * Including documenting expressions

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