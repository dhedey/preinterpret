use preinterpret::*;

fn main() {
    let _ = stream!{
        #(true + 1)
    };
}
