use preinterpret::*;

fn main() {
    run!(
        let %[Hello _ World] = %[Hello World];
    );
}
