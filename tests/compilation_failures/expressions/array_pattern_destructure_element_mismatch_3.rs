use preinterpret::*;

fn main() {
    let _ = run!{
        let [_, _, .., _] = [1, 2];
    };
}