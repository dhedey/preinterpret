use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        #(let [_] = [1, 2])
    };
}