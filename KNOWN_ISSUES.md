# Since Major Version 0.3.0

* `[MAX LITERAL]` - In an expression, the maximum negative integer literal is not valid, for example `-128i8`.
  * By comparison, in `rustc`, there is special handling for such literals, so that e.g. `-128i8` and `--(-128i8)` are accepted.
  * In rust analyzer, before the full pass, it seems to store literals as u128 and perform wrapping arithmetic on them. On a save / full pass, it does appear to reject literals which are out of bounds (e.g. 150i8).
  * We could fix this by special casing maximal signed integer literals (e.g. 128 for an i8) which have yet to be involved in a non-unary expression.

