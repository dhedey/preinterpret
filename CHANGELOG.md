# Major Version 0.3

## 0.3.0

### New Commands

* Expression commands:
  * `[!evaluate! ...]`
  * `[!assign! #x += ...]` for `+` and other supported operators
* Control flow commands:
  * `[!if! COND { ... }]` and `[!if! COND { ... } !else! { ... }]`
  * `[!while! cond {}]`
* Token-stream utility commands:
  * `[!empty!]`
  * `[!is_empty! #stream]`
  * `[!length! #stream]` which gives the number of token trees in the token stream.
  * `[!group! ...]` which wraps the tokens in a transparent group. Useful with `!for!`.
  * Disallow `[!let! #x =]` and require `[!let! #x = [!empty!]]` (give a good error message).

### To come

* ? Use `[!let! #x = 12]` instead of `[!set! ..]`.
* `[!for! #x in [#y] {}]`
* `[!range! 0..5]`
* `[!error! "message" token stream for span]`
* Reconfiguring iteration limit
* Support `!else if!` in `!if!`
* Token stream manipulation... and make it performant (maybe by storing either TokenStream for extension; or `ParseStream` for consumption)
  * Extend token stream `[!extend! #x += ...]`
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