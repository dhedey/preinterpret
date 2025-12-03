use preinterpret::*;

fn main() {
    let _ = stream!{
        #(1.0 | 2.0)
    };
}
