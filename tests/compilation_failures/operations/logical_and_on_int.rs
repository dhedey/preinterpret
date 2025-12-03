use preinterpret::*;

fn main() {
    let _ = stream!{
        #(1 && 2)
    };
}
