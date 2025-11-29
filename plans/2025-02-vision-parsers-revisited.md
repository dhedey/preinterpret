# Parsers Revisited - JULY 2025

This has been superceded by @./2025-09-vision.md 

```rust
// PROPOSAL (subject to the object proposal above):
// * Various modes...
//   * EXPRESSION MODE
//     * OPENING SYNTAX:
//       * #{ ... } or #'name { ... } with a new lexical variable scope
//       * #( ... ) with no new scope (TBC if we need both - probably)
//     * CONTEXT:
//       * Is *not* stream-based - it consists of expressions rather than a stream
//       * It *may* have a contextual input parse stream
//     * One or more statements, terminated by ; (except maybe the last, which is returned)
//     * Idents mean variables.
//     * `let x; let _ = <value>; <value>; if <value> {<statements>} else {}`
//     * Error not to discard a non-None value with `let _ = <value>`
//     * If it has an input parse stream, it's allowed to parse by embedding parsers, e.g. @TOKEN_TREE
//     * Only the last statement can (optionally) output, with a value model of:
//       * Leaf(Bool | Int | Float | String)
//       * Object
//       * Stream
//       * None
//     * (In future we could add breaking from labelled block: https://blog.rust-lang.org/2022/11/03/Rust-1.65.0/#break-from-labeled-blocks)
//     * The actual type is only known at evaluation time.
//     * #var is equivalent to #(var)
//   * OUTPUT STREAM MODE
//     * OPENING SYNTAX: ~{ ... } or ~( ... ) or raw stream literal with r~( ... )
//       * SIDENOTES: I'd have liked to use $, but it clashes with $() repetition syntax inside macro rules
//       * And % might appear in real code as 1 % (2 + 3)
//       * But ~ seems to not be used much, and looks a bit like an s. So good enough?
//     * CONTEXT:
//       * Can make use of a parent output stream, by concatenating into it. 
//       * Is stream-based: Anything embedded to it can have their outputs concatenated onto the output stream
//       * Does *not* have an input parse stream
//     * Can embed [!commands! ...], #() and #{ ... }, but not parsers
//       * The output from #([...]) gets appended to the stream; ideally by passing it an optional output stream
//     * It has a corresponding pattern:
//      * SYNTAX:
//        * ~#( .. ) to start in expression mode OR
//        * ~@( .. ) ~@XXX or ~@[XXX ...] to start with a parser e.g. ~@REST can be used to parse any stream opaquely OR
//        * r~( .. ) to match a raw stream (it doesn't have a mode!)
//      * Can be used where patterns can be used, in a let expression or a match arm, e.g.
//        * let ~#( let x = @IDENT ) = r~(...)
//        * ~#( ... ) => { ... }
//      * CONTEXT:
//         * It implicitly creates a parse-stream for its contents
//         * It would never have an output stream
//         * It is the only syntax which can create a parse stream, other than the [!parse! ..] command
//   * NAMED PARSER
//     * OPENING SYNTAX: @XXX or @[XXX] or @[XXX <arguments>]
//     * CONTEXT:
//       * Is typically *not* stream-based - it may has its own arguments, which are typically a pseudo-object `{  }`
//       * It has an input parse stream
//     * Every named parser @XXX has a typed output, which is:
//       * EITHER its input token stream (for simple matchers, e.g. @IDENT)
//       * OR an #output OBJECT with at least two properties:
//         * input => all matched characters (a slice reference which can be dropped...)
//           (it might only be possible to performantly capture this after the syn fork)
//           THIS NOTE MIGHT BE RELEVANT, BUT I'M WRITING THIS AFTER 6 MONTHS LACKING CONTEXT:
//              This can capture the original tokens by using `let forked = input.fork()`
//              and then `let end_cursor = input.end();` and then consuming `TokenTree`s
//              from `forked` until `forked.cursor >= end_cursor` (making use of the
//              PartialEq implementation) 
//         * output_to => A lazy function, used to handle the output when #x is embedded into a stream output...
//        likely `input` or an error depending on the case.
//         * into_iterator
//       * ... other properties, depending on the parsee:
//         * e.g. a Rust ITEM might have quite a few (mostly lazy)
//   * PARSE-STREAM MODE
//     * OPENING SYNTAX:
//       * @{ ... } or @'block_name { ... } (as a possibly named block) -- both have variable scoping semantics
//       * We also support various repetitions such as: @{ ... }? and @{ ... }+ and @{ ... },+ - discussed in next section in more detail
//       * We likely don't want/need an un-scoped parse stream
//     * CONTEXT:
//       * It is stream-based: Any outputs of items in the stream are converted to an exact parse matcher
//       * Does have an input parse stream which it can mutate
//     * Expression/command outputs are converted into an exact parse matcher
//     * RETURN VALUE:
//       * Parse blocks can return a value with `#(return X)` or `#(return 'block_name X)` else return None
//       * By contrast, `@[STREAM ...]` works very similarly, but outputs its input stream
//   * COMMAND
//     * OPENING SYNTAX: [!command! ...]
//     * CONTEXT:
//       * Takes an optional output stream from its parent (for efficiency, if it returns a stream it can instead concat to that)
//       * May or may not be stream-based depending on the command...
//       * Does *not* have an input parse stream (I think? Maybe it's needed. TBC)
//     * Lots of commands can/should probably become functions
//     * A flexible utility for different kinds of functionality
//
// * Repetitions (including optional), e.g. @IDENT,* or @[IDENT ...],* or @{...},*
//   * Output their contents in a `Repeated` type, with an `items` property, and an into_iterator implementation?
//   * Unscoped parse streams can't be repeated, only parse blocks (because they create a new interpreter frame)
//   * If we don't move our cursor forward in a loop iteration, it's an error -- this prevents @{}* or @{x?}* causing an infinite loop
//   * We do *not* support backtracking.
//     * This avoids most kind of catastrophic backtracking explosions, e.g. (x+x+)+y)
//     * Things such as @IDENT+ @IDENT will not match `hello world` - this is OK I think
//     * Instead, we suggest people to use a match statement or something
//   * There is still an issue of REVERSION - "what happens to external state mutated this iteration repetition when a repetition is not possible?"
//     * This appears in lots of places:
//       * In ? or * blocks where a greedy parse attempt isn't fatal, but should continue
//       * In match arms, considering various stream parsers, some of which might fail after mutating state
//       * In nested * blocks, with different levels of reversion
//     * But it's only a problem with state lexically captured from a parent block
//     * We need some way to handle these cases:
//     (A) Immutable structures - We use efficient-ish immutable data structures, e.g. ImmutableList, Copy-on-write leaf types etc
//        ... so that we can clone them cheaply (e.g. https://github.com/orium/rpds) or even just some manually written tries/cons-lists
//     (B) Use CoW/extensions - But naively it still give O(N^2) if we're reverting occasional array.push(..)es
//        We could possibly add in some tweaks to avoid clones in some special cases (e.g. array.push and array.pop which don't touch a prefix at the start of a frame)
//     (C) Error on mutate - Any changes to variables outside the scope of a given refutable parser => it's a fatal error
//        to parse further until that scope is closed. (this needs to handle nested repetitions).
//     (D) Error on revert - at reversion time, we check there have been no changes to variables below that depth in the stack tree
//        (say by recording a "lowest_stack_touched: usize"), and panic if so; and tell people to use `return` instead; or move state changes to the end.
//        We could even prevent parsing in a conditional block after the reversion.
//        [!Brilliant!] We could introduce a @[REQUIRE  ] which takes the parent conditional block out of conditional mode and makes it a hard error instead. This could dramatically improve error messages, and allow parsing after mutation :). (Ideally they'd be some way of applying it to a specific conditional block, but I think parent is good enough)
//      (E) attempt expression...
//           Whenever we have a refutable frame, we split it in two in syntax: we have an explicit binding block where variable bindings from parent frames can't be mutable, but new variables can be defined... and an execution mode which is irrefutable.
//           - When the first part is executed, the interpreter locks parent variable
//           access from being mutable (and also forks a parse stream if one exists).
//           - Each option's LHS is tried in turn.
//             - If the LHS succeeds, it executes.
//             - Else, if it errors, it goes to the next one. If it's the last one, that error is thrown properly.
attempt {
  { let x = @IDENT } => { x },
  { let x = @LITERAL } => { x },
  // { @END } => { ... }, // Where @END throws unless there's an end 
  {} => { "Expected ident or literal".error(@CURSOR) },
}
// OPTIONAL, REPEATED semantics are basically special cases of `try_many`
// @[..] is a stream exact-parser which can be entered inside expression mode.
// The stream-literal pattern %[..] starts in this mode, but asserts the end too.
//
//     ==> E is probably the clearest and most general, albeit a bit verbose
//
//
// QUESTION:
// - What mode do we start in, in v2?
//   => Possibly expression mode, which could convert to an output with ~{ ... }
//
// =========
// EXAMPLES
// =========

// EXAMPLE WITH !parse! COMMAND
#(
  let parsed = [!parse! {
    input: r~(
      impl A for X, impl B for Y
    ),
    parser: @{ // This @{ .. }, returns a `Repeated` type with an `into_iterator` over its items
      impl #(let the_trait = @IDENT) for #(let the_type = @IDENT)
      #(return { the_trait, the_type })
    },*
  }];

  // Assuming we are writing into an output stream (e.g. outputting from the macro),
  // we can auto-optimize - the last command of the expression block gets to write directly to the stream,
  // and by extension, each block of the for-loop gets to write directly to the stream
  for { the_trait, the_type } in parsed ~{
    impl #the_trait for #the_type {}
  }
)

// EXAMPLE WITH STREAM-PARSING-PATTERN
#(
  let ~#( // Defines a stream pattern, starting in expression mode, and can be used to bind variables
    let parsed = @{
      #(let item = {})
      impl #(item.the_trait = @IDENT) for #(item.the_type = @IDENT)
      #(return item)
    }
  ) = r~(...)

  for { the_trait, the_type } in parsed ~{
    impl #the_trait for #the_type {}
  }
) 

// Example pre-defined with no arguments:
[!define_parser! @IMPL_ITEM @{ // Can either start as #{ ... } or @{ ... }
  impl #(let the_trait = @IDENT) for #(let the_type = @IDENT)
  #(return { the_trait, the_type })
}]
for { the_trait, the_type } in [!parse! { input, parser: @IMPL_ITEM,* }] ~{
  impl #the_trait for #the_type {}
}

// Example pre-defined 2 with arguments:
[!define_parser! @[IMPL_ITEM /* arguments named-parser or parse-stream */] {
  @(impl #(let the_trait = @IDENT) for #(let the_type = @IDENT));
  { the_trait, the_type }
}]
for { the_trait, the_type } in [!parse! { input, parser: @IMPL_ITEM,* }] ~{
  impl #the_trait for #the_type {}
}
```