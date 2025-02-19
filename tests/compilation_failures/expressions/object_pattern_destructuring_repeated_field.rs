use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        #(let { x, x } = { x: 1 })
    };
}