use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        #(x = [+ 1]; 1 x)
    };
}