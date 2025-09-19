use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        #(let x = [1, 2]; x.debug())
    };
}