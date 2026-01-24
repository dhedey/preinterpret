use preinterpret::*;

fn main() {
    run!(
        let @parser[#{
            parser.open('(');
            // Fail to consume the content
            parser.close(')');
        }] = %[(Hello World)];
    );
}
