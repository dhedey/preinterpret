use preinterpret::*;

fn main() {
    let _ = stream!{
        #(let [_] = [1, 2])
    };
}