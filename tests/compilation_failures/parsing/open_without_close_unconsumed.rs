use preinterpret::*;

fn main() {
    // Open group with content we DON'T fully consume
    run!(
        let @parser[#{
            parser.open('(');
            // Only parse one ident, leaving "World" unconsumed
            let _ = parser.ident();
            // Don't close - should trigger error from syn's drop glue
        }] = %[(Hello World)];
    );
}
