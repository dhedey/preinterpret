use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        #(let x = { 1 + 1 + 1 })
    };
}