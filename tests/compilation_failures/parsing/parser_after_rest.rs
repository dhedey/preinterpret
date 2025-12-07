use preinterpret::*;

fn main() {
    run!(
        let @input[#{
            let _ = input.rest();
            // This is a _bad_ error - syn sticks it out at the end of Span::call_site(),
            // Which is not helpful here. It's quite hard to fix this properly, but let's
            // address some time before launch.
            let _ = input.token_tree();
        }] = %[Hello World];
    );
}