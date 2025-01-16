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
* Other commands:
  * `[!error! ..]`

I considered disallowing commands like `[!set! #x =]` and requiring `[!set! #x = [!empty!]]`, but deemed it unhelpful, because then they have awkward edge-cases when embedding empty tokenstreams from declarative macros `($each_tt)*`.

### To come
  
* Create fields parsing macro
* Add compile tests for incorrectly formatted nested commands, e.g. an error inside an if
* Grouping... Proposal:
  * Command arguments which are streams should be surrounded by `[ ... ]`... a `[!command! ...]` or `#x` may also be used instead.
    * Add test that I can load !error! spans from `#x = Hello World` or `#..x = [Hello World]`
* ? Use `[!let! #x = 12]` instead of `[!set! ...]`
  * ...Or maybe not. Maybe `[!let! #..x = Hello World]` does parsing and is equivalent to `[!set! #x = Hello World]`
* Fix `if` and `while` to read expression until braces
* Support string & char literals (for comparisons & casts) in expressions
* Add more tests
  * e.g. for various expressions
  * e.g. for long sums
  * Add compile failure tests
* `[!range! 0..5]` outputs `0 1 2 3 4`
* Reconfiguring iteration limit
* Support `!else if!` in `!if!`
* `[!extend! #x += ...]` to make such actions more performant
* Support `!for!` so we can use it for simple generation scenarios without needing macros at all:
  * Complexities:
    * Parsing `,`
      * When parsing `Punctuated<X>` in syn, it typically needs to know how to parse a full X (e.g. X of an item)
      * But ideally we want to defer the parsing of the next level...
      * This means we may need to instead do something naive for now, e.g. split on `,` in the token stream
    * Iterating over already parsed structure; e.g. if we parse a struct and want to iterate over the contents of the path of the type of the first field?
      * Do we store such structured variables?
  * Option 1 - Simple. For consumes 1 token tree per iteration.
    * This means that we may need to pre-process the stream...
    * Examples:
      * `[!for! #x in [Hello World] {}]`
    * Questions:
      * How does this extend to parsing scenarios, such as a punctuated `,`?
        * It doesn't explicitly...
        * But `[!for! #x in [!split! [Hello, World,] on ,]]` could work, if `[!split!]` outputs transparent groups, and discards empty final items
          * `[!comma_split! Hello, World,]`
          * `[!split! { input: [Hello, World,], separator: [,], ignore_empty_at_end?: true, require_trailing_separator?: false, }]`
      * How does this handle zipping / unzipping and un-grouping, and/or parsing a more complicated group?
        * Proposal: The parsing operates over the content of a single token tree, possibly via explicitly ungrouping.
        * e.g. `[!for! (#country #flag #capital) in [!zip! (#countries #flags #capitals)]`
        * To get this to work, we'd need to:
          * Make it so that the `#x` binding consumes a single token tree, and auto-expands zero or more transparent group/s
            * (Incidentally this is also how the syn Cursor type iteration works, roughly)
          * And possibly have `#..x` consumes the remainder of the stream??
          * Make it so that `[!set! #x = ...]` wraps the `...` in a transparent group so it's consistent.
        * And we can support basic parsing, via:
          * Various groupings or `[!GROUP!]` for a transparent group
          * Consuming various explicit tokens
          * `[!OPTIONAL! ...]`
  * Option 2 - we specify some stream-processor around each value
    * `[!for! [!EACH! #x] in [Hello World]]`
    * `[!for! [!SPLIT! #x,] in [Hello, World,]]`
    * Even though this might be more performant, I'm not too much of a fan of this, as it's hard to understand
    * Perhaps we can leave it to the `[!while_parse! #x[!OPTIONAL! ,] from #X]` style commands?
* `[!split!]` and `[!split_no_trailing!]`
* `[!zip! ([Hello Goodbye] [World Friend])]` => `[(Hello World), (Goodbye Friend)]`
* Basic place parsing
  * In parse land: #x matches a single token, #..x consumes the rest of a stream
  * Auto-expand transparent groups, like syn. Maybe even using syn `TokenBuffer` / `Cursor`!
  * Explicit Punct, Idents, Literals
  * `[!STREAM! ]` method to take a stream
  * `[!LITERAL! #x]` / `[!IDENT! #x]` bindings
  * `[!OPTIONAL! ...]`
  * Groups or `[!GROUP! ...]`
  * `#x` binding reads a token tree
  * `#..x` binding reads the rest of the stream
  * `[!RAW!]` for e.g. `[!while_parse! [!RAW! from] from #X]`
* `[!match!]` (with `#..x` as a catch-all)
* Check all #[allow(unused)] and remove any which aren't needed
* Rework expression parsing
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