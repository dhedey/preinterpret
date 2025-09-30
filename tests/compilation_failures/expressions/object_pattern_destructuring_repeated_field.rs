use preinterpret::*;

fn main() {
    let _ = stream!{
        #(let { x, x } = { x: 1 })
    };
}