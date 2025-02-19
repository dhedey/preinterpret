use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        #(let [_, .., _, .., _] = [1, 2, 3, 4])
    };
}