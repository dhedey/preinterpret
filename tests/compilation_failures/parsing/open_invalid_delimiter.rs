use preinterpret::*;

fn main() {
    run!(
        let @parser[#{
            parser.open('x');
        }] = %[Hello World];
    );
}
