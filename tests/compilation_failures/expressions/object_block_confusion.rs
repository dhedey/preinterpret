use preinterpret::*;

fn main() {
    let _ = stream!{
        #(let x = { 1 + 1 + 1 })
    };
}