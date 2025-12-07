use preinterpret::*;

fn main() {
    run!(
        let @parser[#{
            parser.close('x');
        }] = %[Hello World];
    );
}
