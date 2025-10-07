use preinterpret::*;

fn main() {
    let _ = run!{
        let x = %{ 1 + 1 + 1 };
    };
}