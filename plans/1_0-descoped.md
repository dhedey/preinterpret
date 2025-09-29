* Performance:
  * Use a small-vec optimization in some places
  * Get rid of needless cloning of commands/variables etc
  * Avoid needless token stream clones: Have `x += y` take `y` as OwnedOrRef, and either handles it as owned or shared reference (by first cloning)
* User-defined functions and parsers
* Parsers:
  * Fuller rust syntax parsing: all of https://veykril.github.io/tlborm/decl-macros/minutiae/fragment-specifiers.html#ty and more from syn (e.g. item, fields, etc)
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