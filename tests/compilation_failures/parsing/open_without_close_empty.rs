use preinterpret::*;

fn main() {
    // Open empty group but forget to close
    run!(
        let @parser[#{
            parser.open('(');
            // Group is empty, but we never call close()
        }] = %[()];
    );
}
