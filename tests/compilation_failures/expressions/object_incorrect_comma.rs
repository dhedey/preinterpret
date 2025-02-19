use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        let a = 0;
        #({ a; b: 1 })
    };
}