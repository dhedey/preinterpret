use preinterpret::*;

fn main() {
    let _ = stream!{
        #(let x = [1, 2]; x.debug())
    };
}