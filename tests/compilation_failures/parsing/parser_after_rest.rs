use preinterpret::*;

fn main() {
    run!(
        let @input[#{
            let _ = input.rest();
            let _ = input.token_tree();
        }] = %[Hello World];
    );
}