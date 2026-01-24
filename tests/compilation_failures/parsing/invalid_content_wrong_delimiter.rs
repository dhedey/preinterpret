use preinterpret::*;

fn main() {
    run!(
        let %[(#{ let _ = input.rest(); })] = %[[Hello World]];
    );
}
