use preinterpret::*;

fn main() {
    run!(
        let @parser[#{
            parser.open('(');
            let _ = parser.ident();
            parser.close(']');
        }] = %[(Hello)];
    );
}
