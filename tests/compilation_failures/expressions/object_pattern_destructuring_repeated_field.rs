use preinterpret::*;

fn main() {
    let _ = run!{
        let %{ x, x } = %{ x: 1 };
    };
}