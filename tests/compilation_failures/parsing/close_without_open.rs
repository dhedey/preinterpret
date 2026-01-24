use preinterpret::*;

fn main() {
    run!(
        let @parser[#{
            parser.close(')');
        }] = %[Hello World];
    );
}
