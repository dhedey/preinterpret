use preinterpret::*;

fn main() {
    let _ = stream!{
        #(5 < 6.4)
    };
}