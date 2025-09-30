use preinterpret::*;

fn main() {
    let _ = stream!{
        let a = 0;
        #({ a; b: 1 })
    };
}