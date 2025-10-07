use preinterpret::*;

fn main() {
    let _ = run!{
        let x = 0;
        %{ x, x } = %{ x: 1 };
        x
    };
}