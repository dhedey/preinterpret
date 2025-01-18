# Major Version 0.3

## 0.3.0

### New Commands

* Core commands:
  * `[!error! ...]`
  * `[!stream! ...]` command which just returns its contents, but can be used in places where a single item is expected when parsing.
* Expression commands:
  * `[!evaluate! ...]`
  * `[!assign! #x += ...]` for `+` and other supported operators
  * `[!range! 0..5]` outputs `0 1 2 3 4`
* Control flow commands:
  * `[!if! COND { ... }]` and `[!if! COND { ... } !else! { ... }]`
  * `[!while! COND {}]`
* Token-stream utility commands:
  * `[!empty!]`
  * `[!is_empty! #stream]`
  * `[!length! #stream]` which gives the number of token trees in the token stream.
  * `[!group! ...]` which wraps the tokens in a transparent group. Useful with `!for!`.
  * `[!intersperse! { .. }]` which inserts separator tokens between each token tree in a stream.

I considered disallowing commands like `[!set! #x =]` and requiring `[!set! #x = [!empty!]]`, but deemed it unhelpful, because then they have awkward edge-cases when embedding empty tokenstreams from declarative macros `($each_tt)*`.

### To come

* Consider [!..flattened! ] and getting rid of !group!
  => Add separate ControlFlowCode type
  => Make the type of commands static
  => Disallow most command types in expressions
  => Get rid of [!empty!] and [!stream!] and replace with [!..group!] or [!group!]
* Explore getting rid of lots of the span range stuff
* Support `!else if!` in `!if!`
* Support string & char literals (for comparisons & casts) in expressions
* Reconfiguring iteration limit, via `[!settings! { iteration_limit: 4000 }]`
* Other token stream commands
* Add tests for CommandStreamInput (via `[!split! ]` or `[!intersperse! ]`?):
  => From `#x = Hello World`
  => Or `#..x = [Hello World]`
  => But not `$x` - it has to be wrapped in `[]`
  => Improve tests for `[!range!]`
* Add more tests
  * e.g. for various expressions
  * e.g. for long sums
  * Add compile failure tests
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
* `[!split! { items: X, with: X, drop_trailing_empty?: true }]`
* `[!intersperse! { items: X, with: X, add_trailing?: false, override_final_with?: X }]` for adding something between each item, where each `X` is a `CommandStreamInput`
* `[!zip! ([Hello Goodbye] [World Friend])]` => `[(Hello World), (Goodbye Friend)]`
* See comment on `!assign!`
* Basic place parsing
  * Introduce `[!let! #..x = Hello World]` does parsing and is equivalent to `[!set! #x = Hello World]`
  * In parse land: #x matches a single token, #..x consumes the rest of a stream
  * Auto-expand transparent groups, like syn. Maybe even using syn `TokenBuffer` / `Cursor`!
  * Explicit Punct, Idents, Literals
  * `[!PARSER! ]` method to take a stream
  * `[!LITERAL! #x]` / `[!IDENT! #x]` bindings
  * `[!OPTIONAL! ...]` and/or possibly `#(..)?` and `[!is_set! #x]`?
  * Groups or `[!GROUP! ...]`
  * Regarding `#(..)+` and `#(..)*`...
    * Any binding could be set to an array, but it gets complicated fast with nested bindings.
    * Instead for now, we could push people towards capturing the input and parsing it with for loops and matches.
  * `#x` binding reads a token tree
  * `#..x` binding reads the rest of the stream
  * `[!RAW!]` for e.g. `[!while_parse! [!RAW! from] from #X]`
  * `[!match!]` (with `#..x` as a catch-all)
* Check all `#[allow(unused)]` and remove any which aren't needed
* Rework expression parsing
* Work on book
  * Input paradigms:
    * Streams
    * StreamInput / ValueInput / CodeInput
  * Including documenting expressions
  * There are three main kinds of commands:
    * Those taking a stream as-is
    * Those taking some { fields }
    * Those taking some custom syntax, e.g. `!set!`, `!if!`, `!while!`

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